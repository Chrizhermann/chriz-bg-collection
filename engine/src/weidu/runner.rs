//! Streaming child-process supervision for one resolved WeiDU invocation.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::process::{ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use serde::{Deserialize, Serialize};

use crate::digest::sha256_bytes;
use crate::events::{EngineEvent, EventSink, Stream};

use super::invocation::{Invocation, ResolvedPrompt};
use super::process_group::ProcessGroup;

const DISPLAY_CHUNK_BYTES: usize = 4 * 1024;
const DISPLAY_CHANNEL_CAPACITY: usize = 64;
const LAST_OUTPUT_BYTES: usize = 4 * 1024;
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(20);
/// Runner-owned create-once evidence proving which authored prompt answers reached child stdin.
pub const PROMPT_RESULTS_FILE_NAME: &str = "prompt-results.jsonl";
/// While this variable exists, whatever its value, `cmd.exe` skips the current directory when
/// resolving a bare command name. Some shells set it (Claude Code's does); a normal Windows
/// session does not, and mods rely on that default.
const NO_DEFAULT_CURRENT_DIRECTORY_IN_EXE_PATH: &str = "NoDefaultCurrentDirectoryInExePath";

/// One prompt answer successfully written and flushed to the child process.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptAnswerEvidence {
    /// Zero-based position in the exact authored prompt sequence.
    pub index: usize,
    /// SHA-256 of the expected output bytes that unlocked this answer.
    pub expected_output_sha256: String,
    /// SHA-256 of the exact answer bytes written to child stdin.
    pub answer_sha256: String,
}

/// Parse the runner-authored prompt evidence stream.
pub fn parse_prompt_results(bytes: &[u8]) -> io::Result<Vec<PromptAnswerEvidence>> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            serde_json::from_slice(line)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        })
        .collect()
}

/// A user decision delivered after the runner requests attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerControl {
    /// Rearm the silence watchdog without interrupting the process.
    ContinueWaiting,
    /// Explicitly terminate the supervised process tree.
    Cancel,
}

/// Process-wide cancellation handle that forwards Ctrl+C to the one active runner.
///
/// A cancellation requested just before runner registration is retained and delivered as
/// soon as the runner attaches. Registrations are exclusive because campaign orchestration
/// never overlaps WeiDU processes.
#[derive(Clone, Default)]
pub struct RunnerControlHandle {
    state: Arc<Mutex<RunnerControlState>>,
}

#[derive(Default)]
struct RunnerControlState {
    generation: u64,
    active: Option<(u64, Sender<RunnerControl>)>,
    cancel_requested: bool,
    pause_requested: bool,
}

impl RunnerControlHandle {
    /// Create a handle with no active runner and no pending cancellation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach the next runner, or return `None` if another runner is already active.
    pub fn register(&self) -> Option<RunnerControlRegistration> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let (generation, cancel_requested) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.active.is_some() {
                return None;
            }
            state.generation = state.generation.wrapping_add(1);
            let generation = state.generation;
            state.active = Some((generation, sender.clone()));
            (generation, state.cancel_requested)
        };
        if cancel_requested {
            let _ = sender.send(RunnerControl::Cancel);
        }
        Some(RunnerControlRegistration {
            receiver,
            generation,
            state: Arc::clone(&self.state),
        })
    }

    /// Rearm the silence watchdog for the active runner, if any.
    pub fn continue_waiting(&self) {
        let sender = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active
            .as_ref()
            .map(|(_, sender)| sender.clone());
        if let Some(sender) = sender {
            let _ = sender.send(RunnerControl::ContinueWaiting);
        }
    }

    /// Persist a cancellation request and forward it to the active runner, if any.
    pub fn cancel(&self) {
        let sender = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.cancel_requested = true;
            state.active.as_ref().map(|(_, sender)| sender.clone())
        };
        if let Some(sender) = sender {
            let _ = sender.send(RunnerControl::Cancel);
        }
    }

    /// Whether cancellation was requested, for work outside a runner, such as a download or
    /// extraction that must not start, or must stop waiting for another one of the same entry.
    pub fn cancel_requested(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .cancel_requested
    }

    /// Request a cooperative campaign stop after the current durable pipeline boundary.
    ///
    /// Unlike [`Self::cancel`], this never sends a control message to the active child process.
    pub fn pause_after_boundary(&self) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pause_requested = true;
    }

    /// Whether a cooperative boundary stop has been requested for this campaign.
    pub fn pause_requested(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pause_requested
    }
}

/// Exclusive attachment between one campaign control handle and one runner invocation.
pub struct RunnerControlRegistration {
    receiver: Receiver<RunnerControl>,
    generation: u64,
    state: Arc<Mutex<RunnerControlState>>,
}

impl RunnerControlRegistration {
    /// Control receiver passed directly to [`run`].
    pub fn receiver(&self) -> &Receiver<RunnerControl> {
        &self.receiver
    }
}

impl Drop for RunnerControlRegistration {
    fn drop(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state
            .active
            .as_ref()
            .is_some_and(|(generation, _)| *generation == self.generation)
        {
            state.active = None;
        }
    }
}

/// Observable end state of one child-process supervision attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    /// The child exited on its own with an operating-system exit code.
    Exited {
        /// Process exit code, or `-1` when the platform did not supply one.
        code: i32,
    },
    /// The caller explicitly cancelled the complete process tree.
    Cancelled,
    /// The output log, process group, or child process could not be created safely.
    SpawnFailed,
}

/// Engine-owned inputs needed to supervise one process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerRequest {
    /// Fully validated WeiDU process description.
    pub invocation: Invocation,
    /// Stable run/step identity attached to emitted events.
    pub step_id: String,
    /// Create-once raw stdout/stderr evidence log for this attempt.
    pub output_log: std::path::PathBuf,
    /// Quiet interval after which the UI must ask whether to keep waiting.
    pub silence_threshold: Duration,
}

/// Registers this invocation with a shared campaign control handle, then supervises it.
///
/// The registration remains active until [`run`] has terminated and joined the complete
/// process tree. A second concurrent registration fails closed as [`RunOutcome::SpawnFailed`].
pub fn run_controlled<S: EventSink>(
    request: RunnerRequest,
    controls: &RunnerControlHandle,
    sink: S,
) -> RunOutcome {
    let Some(registration) = controls.register() else {
        emit_error(
            &sink,
            &request.step_id,
            "another WeiDU runner is already registered for this campaign".to_owned(),
        );
        return RunOutcome::SpawnFailed;
    };
    let outcome = run(request, registration.receiver().clone(), sink);
    drop(registration);
    outcome
}

/// Runs one WeiDU invocation while streaming output and accepting explicit controls.
///
/// Silence only emits [`EngineEvent::AttentionRequired`]. The runner deliberately has no
/// automatic timeout kill path: a real WeiDU process is terminated only after
/// [`RunnerControl::Cancel`].
pub fn run<S: EventSink>(
    request: RunnerRequest,
    controls: Receiver<RunnerControl>,
    sink: S,
) -> RunOutcome {
    run_with_host_path(request, controls, sink, std::env::var_os("PATH").as_deref())
}

fn run_with_host_path<S: EventSink>(
    request: RunnerRequest,
    controls: Receiver<RunnerControl>,
    sink: S,
    #[cfg_attr(not(unix), allow(unused_variables))] host_path: Option<&std::ffi::OsStr>,
) -> RunOutcome {
    let raw_log = match create_output_log(&request.output_log) {
        Ok(file) => Arc::new(Mutex::new(file)),
        Err(error) => {
            emit_error(
                &sink,
                &request.step_id,
                format!(
                    "could not create raw process log {}: {error}",
                    request.output_log.display()
                ),
            );
            return RunOutcome::SpawnFailed;
        }
    };
    let stdout_path = request.output_log.with_file_name("stdout.log");
    let stdout_log = match create_output_log(&stdout_path) {
        Ok(file) => file,
        Err(error) => {
            emit_error(
                &sink,
                &request.step_id,
                format!(
                    "could not create stdout log {}: {error}",
                    stdout_path.display()
                ),
            );
            return RunOutcome::SpawnFailed;
        }
    };
    let stderr_path = request.output_log.with_file_name("stderr.log");
    let stderr_log = match create_output_log(&stderr_path) {
        Ok(file) => file,
        Err(error) => {
            emit_error(
                &sink,
                &request.step_id,
                format!(
                    "could not create stderr log {}: {error}",
                    stderr_path.display()
                ),
            );
            return RunOutcome::SpawnFailed;
        }
    };
    let prompt_results_path = request.output_log.with_file_name(PROMPT_RESULTS_FILE_NAME);
    let mut prompt_results_log = match create_output_log(&prompt_results_path) {
        Ok(file) => file,
        Err(error) => {
            emit_error(
                &sink,
                &request.step_id,
                format!(
                    "could not create prompt evidence log {}: {error}",
                    prompt_results_path.display()
                ),
            );
            return RunOutcome::SpawnFailed;
        }
    };

    let process_group = match ProcessGroup::new() {
        Ok(group) => group,
        Err(error) => {
            emit_error(
                &sink,
                &request.step_id,
                format!("could not create WeiDU process group: {error}"),
            );
            return RunOutcome::SpawnFailed;
        }
    };

    #[cfg(unix)]
    if let Some(attempt_dir) = request.invocation.debug_path.parent() {
        if !attempt_dir.as_os_str().is_empty() {
            for (_, dir) in unix_isolated_env_dirs(attempt_dir) {
                let _ = std::fs::create_dir_all(dir);
            }
        }
    }
    #[cfg(unix)]
    let _unix_tool_bin = match prepare_unix_tool_bin(&request.invocation.program, host_path) {
        Ok(dir) => dir,
        Err(error) => {
            emit_error(
                &sink,
                &request.step_id,
                format!("could not prepare Unix helper shims: {error}"),
            );
            return RunOutcome::SpawnFailed;
        }
    };
    #[cfg(unix)]
    let unix_tool_bin_path = Some(_unix_tool_bin.path());
    #[cfg(not(unix))]
    let unix_tool_bin_path: Option<&std::path::Path> = None;

    let mut child = match weidu_command(&request.invocation, unix_tool_bin_path, host_path).spawn()
    {
        Ok(child) => child,
        Err(error) => {
            emit_error(
                &sink,
                &request.step_id,
                format!(
                    "could not start WeiDU process {}: {error}",
                    request.invocation.program.display()
                ),
            );
            return RunOutcome::SpawnFailed;
        }
    };

    if let Err(error) = process_group.attach(&child) {
        let _ = child.kill();
        let _ = child.wait();
        emit_error(
            &sink,
            &request.step_id,
            format!("could not supervise WeiDU process tree: {error}"),
        );
        return RunOutcome::SpawnFailed;
    }

    let stdout = child
        .stdout
        .take()
        .expect("stdout was configured as piped before spawn");
    let stderr = child
        .stderr
        .take()
        .expect("stderr was configured as piped before spawn");
    let mut stdin = child.stdin.take();

    let (output_tx, output_rx) = crossbeam_channel::bounded(DISPLAY_CHANNEL_CAPACITY);
    let readers = vec![
        spawn_reader(
            stdout,
            Stream::Stdout,
            output_tx.clone(),
            Arc::clone(&raw_log),
            stdout_log,
        ),
        spawn_reader(
            stderr,
            Stream::Stderr,
            output_tx,
            Arc::clone(&raw_log),
            stderr_log,
        ),
    ];

    let mut matcher = PromptMatcher::new(&request.invocation.prompts);
    let mut last_output = Vec::new();
    let mut stdout_closed = false;
    let mut stderr_closed = false;
    let mut exit_status = None;
    let mut attention_active = false;
    let mut silence_deadline = Instant::now() + request.silence_threshold;
    let mut cancelled = false;
    let mut supervision_failed = false;

    loop {
        while let Ok(control) = controls.try_recv() {
            match control {
                RunnerControl::ContinueWaiting => {
                    if attention_active {
                        attention_active = false;
                        silence_deadline = Instant::now() + request.silence_threshold;
                    }
                }
                RunnerControl::Cancel => {
                    cancelled = true;
                    break;
                }
            }
        }
        if cancelled {
            break;
        }

        if exit_status.is_none() {
            match process_group.poll_exit(&mut child) {
                Ok(status) => exit_status = status,
                Err(error) => {
                    emit_error(
                        &sink,
                        &request.step_id,
                        format!("could not query WeiDU process state: {error}"),
                    );
                    cancelled = true;
                    supervision_failed = true;
                    break;
                }
            }
        }

        if exit_status.is_some() && stdout_closed && stderr_closed {
            break;
        }

        if exit_status.is_none() && !attention_active && Instant::now() >= silence_deadline {
            sink.emit(EngineEvent::AttentionRequired {
                step_id: request.step_id.clone(),
                reason: "WeiDU has produced no output for a while. The installation is still running; it may be processing a slow step or waiting for an unexpected prompt."
                    .to_owned(),
                last_output: String::from_utf8_lossy(&last_output).into_owned(),
            });
            attention_active = true;
            continue;
        }

        let wait = if exit_status.is_some() || attention_active {
            PROCESS_POLL_INTERVAL
        } else {
            PROCESS_POLL_INTERVAL.min(silence_deadline.saturating_duration_since(Instant::now()))
        };
        match output_rx.recv_timeout(wait) {
            Ok(OutputMessage::Chunk { stream, bytes }) => {
                remember_output(&mut last_output, &bytes);
                attention_active = false;
                silence_deadline = Instant::now() + request.silence_threshold;
                sink.emit(EngineEvent::ConsoleLine {
                    step_id: request.step_id.clone(),
                    stream,
                    line: String::from_utf8_lossy(&bytes).into_owned(),
                });
                match matcher.observe(stream, &bytes, &mut stdin) {
                    Ok(Some(answered)) => {
                        if let Err(error) = append_prompt_result(&mut prompt_results_log, &answered)
                        {
                            emit_error(
                                &sink,
                                &request.step_id,
                                format!("could not persist authored prompt evidence: {error}"),
                            );
                            supervision_failed = true;
                            cancelled = true;
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        emit_error(
                            &sink,
                            &request.step_id,
                            format!("could not answer an authored WeiDU prompt: {error}"),
                        );
                        supervision_failed = true;
                        cancelled = true;
                        break;
                    }
                }
            }
            Ok(OutputMessage::Closed(stream)) => match stream {
                Stream::Stdout => stdout_closed = true,
                Stream::Stderr => stderr_closed = true,
            },
            Ok(OutputMessage::ReadFailed { stream, error }) => {
                emit_error(
                    &sink,
                    &request.step_id,
                    format!("could not read WeiDU {stream:?}: {error}"),
                );
                supervision_failed = true;
                cancelled = true;
                break;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                if !stdout_closed || !stderr_closed {
                    emit_error(
                        &sink,
                        &request.step_id,
                        "WeiDU output readers disconnected before both streams closed".to_owned(),
                    );
                    supervision_failed = true;
                    cancelled = true;
                    break;
                }
            }
        }
    }

    if cancelled {
        stdin = None;
        if let Err(error) = process_group.terminate(&mut child) {
            emit_error(
                &sink,
                &request.step_id,
                format!("could not terminate WeiDU process tree cleanly: {error}"),
            );
            let _ = child.kill();
        }
        // Closing the configured Windows Job Object is the crash-safe backstop that also
        // releases any descendant-held stdout/stderr pipe handles before reader joins.
        drop(process_group);
        let _ = child.wait();
    }

    drop(stdin);
    if !join_readers(readers, &sink, &request.step_id) {
        supervision_failed = true;
    }
    if let Ok(mut log) = raw_log.lock() {
        if log.flush().and_then(|()| log.sync_all()).is_err() {
            supervision_failed = true;
        }
    } else {
        supervision_failed = true;
    }
    if prompt_results_log
        .flush()
        .and_then(|()| prompt_results_log.sync_all())
        .is_err()
    {
        supervision_failed = true;
    }
    // A component can fail before it asks, and WeiDU then exits with an error: what its missing
    // answers mean is for the run's evidence to say. Only a success needs every answer here.
    let exited_with_failure = exit_status.is_some_and(|status| !status.success());
    if !cancelled && !exited_with_failure && !matcher.complete() {
        emit_error(
            &sink,
            &request.step_id,
            format!(
                "WeiDU exited before {} authored prompt answer(s) were observed and written",
                matcher.remaining()
            ),
        );
        supervision_failed = true;
    }

    if supervision_failed {
        RunOutcome::SpawnFailed
    } else if cancelled {
        RunOutcome::Cancelled
    } else {
        let status = exit_status.or_else(|| child.wait().ok());
        RunOutcome::Exited {
            code: status.and_then(|status| status.code()).unwrap_or(-1),
        }
    }
}

/// Configures, without spawning, the supervised child for one invocation.
fn weidu_command(
    invocation: &Invocation,
    #[cfg_attr(not(unix), allow(unused_variables))] unix_tool_bin: Option<&std::path::Path>,
    #[cfg_attr(not(unix), allow(unused_variables))] host_path: Option<&std::ffi::OsStr>,
) -> Command {
    let has_prompts = !invocation.prompts.is_empty();
    let mut command = Command::new(&invocation.program);
    command
        .current_dir(&invocation.cwd)
        .args(&invocation.args)
        // Mods run bare batch names from the game folder (EET: `AT_NOW ~shortcut.bat~`,
        // `AT_NOW ~call arch_var.bat~`). Everything else is inherited unchanged.
        .env_remove(NO_DEFAULT_CURRENT_DIRECTORY_IN_EXE_PATH)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(if has_prompts {
            Stdio::piped()
        } else {
            Stdio::null()
        });

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

        // Keep WeiDU in the background; its output and prompts still use our pipes.
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(unix)]
    {
        // Isolate `HOME` / `XDG_*` per managed campaign (`.chriz/weidu-env`) for `AT_NOW` child
        // tools and shells rather than touching the player's real `$HOME`; statically linked WeiDU
        // 251 still resolves `%USER_DIRECTORY%` via `getpwuid(getuid())` across multi-step runs
        // such as `EET` and `EET_end`.
        if let Some(attempt_dir) = invocation.debug_path.parent() {
            for (name, dir) in unix_isolated_env_dirs(attempt_dir) {
                command.env(name, dir);
            }
        }
        if let Some(tool_bin) = unix_tool_bin {
            command.env("PATH", prepended_unix_path(tool_bin, host_path));
        }
    }
    super::process_group::configure_command(&mut command);
    command
}

#[cfg(unix)]
fn unix_campaign_env_root(attempt_dir: &std::path::Path) -> std::path::PathBuf {
    for ancestor in attempt_dir.ancestors() {
        if ancestor.file_name() == Some(std::ffi::OsStr::new(".chriz")) {
            return ancestor.join("weidu-env");
        }
    }
    attempt_dir.to_path_buf()
}

#[cfg(unix)]
fn unix_isolated_env_dirs(
    attempt_dir: &std::path::Path,
) -> [(&'static str, std::path::PathBuf); 4] {
    let root = unix_campaign_env_root(attempt_dir);
    [
        ("HOME", root.join("home")),
        ("XDG_DATA_HOME", root.join("xdg-data")),
        ("XDG_CONFIG_HOME", root.join("xdg-config")),
        ("XDG_CACHE_HOME", root.join("xdg-cache")),
    ]
}

#[cfg(unix)]
fn prepare_unix_tool_bin(
    program: &std::path::Path,
    host_path: Option<&std::ffi::OsStr>,
) -> io::Result<tempfile::TempDir> {
    let dir = tempfile::Builder::new()
        .prefix("chriz-bg-weidu-bin-")
        .tempdir()?;
    let weidu_target = if program.is_absolute() {
        program.to_path_buf()
    } else if program.components().count() > 1 {
        std::env::current_dir()?.join(program)
    } else {
        let local = std::env::current_dir()?.join(program);
        if local.exists() {
            local
        } else if let Some(name) = program.to_str() {
            crate::preflight::find_executable_in_path_var(host_path, &[name]).unwrap_or(local)
        } else {
            local
        }
    };
    std::os::unix::fs::symlink(&weidu_target, dir.path().join("weidu"))?;
    if let Some(lua_target) = crate::preflight::find_executable_in_path_var(
        host_path,
        crate::preflight::UNIX_LUA_CANDIDATES,
    ) {
        write_unix_lua_shim(&dir.path().join("lua"), &lua_target)?;
    }
    Ok(dir)
}

#[cfg(unix)]
fn write_unix_lua_shim(
    shim_path: &std::path::Path,
    lua_target: &std::path::Path,
) -> io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::OpenOptionsExt;

    // WeiDU's `AT_NOW` without `EXACT` lowercases commands on Unix (for example `lua eet/lib/tlk_cnt.lua`
    // in EET while the directory on case-sensitive filesystems is `EET/`). Resolve a missing `$1`
    // script path case-insensitively segment by segment and fail closed on case ambiguity.
    let mut script = Vec::from(
        br#"#!/bin/sh
if [ "$#" -ge 1 ]; then
    target="$1"
    case "$target" in
        -*|"")
            ;;
        *)
            if [ ! -e "$target" ] && [ ! -L "$target" ]; then
                case "$target" in
                    /*)
                        cur="/"
                        rest="${target#/}"
                        ;;
                    *)
                        cur="."
                        rest="$target"
                        ;;
                esac
                ok=1
                while [ -n "$rest" ]; do
                    case "$rest" in
                        */*)
                            seg="${rest%%/*}"
                            rest="${rest#*/}"
                            ;;
                        *)
                            seg="$rest"
                            rest=""
                            ;;
                    esac
                    case "$seg" in
                        ""|.)
                            continue
                            ;;
                        ..)
                            if [ "$cur" = "/" ]; then
                                :
                            elif [ "$cur" = "." ]; then
                                cur=".."
                            else
                                cur="$cur/.."
                            fi
                            continue
                            ;;
                    esac
                    if [ "$cur" = "." ]; then
                        cand="$seg"
                    elif [ "$cur" = "/" ]; then
                        cand="/$seg"
                    else
                        cand="$cur/$seg"
                    fi
                    if [ -e "$cand" ] || [ -L "$cand" ]; then
                        cur="$cand"
                        continue
                    fi
                    if [ "$cur" = "/" ]; then
                        glob_prefix=""
                    else
                        glob_prefix="$cur"
                    fi
                    seg_lc=$(printf '%s' "$seg" | PATH="$PATH:/usr/bin:/bin" LC_ALL=C tr 'A-Z' 'a-z')
                    found=""
                    ambiguous=0
                    for entry in "$glob_prefix"/* "$glob_prefix"/.[!.]* "$glob_prefix"/..?*; do
                        [ -e "$entry" ] || [ -L "$entry" ] || continue
                        base="${entry##*/}"
                        base_lc=$(printf '%s' "$base" | PATH="$PATH:/usr/bin:/bin" LC_ALL=C tr 'A-Z' 'a-z')
                        if [ "$base_lc" = "$seg_lc" ]; then
                            if [ -n "$found" ]; then
                                ambiguous=1
                                break
                            fi
                            found="$base"
                        fi
                    done
                    if [ "$ambiguous" -eq 0 ] && [ -n "$found" ]; then
                        if [ "$cur" = "." ]; then
                            cur="$found"
                        elif [ "$cur" = "/" ]; then
                            cur="/$found"
                        else
                            cur="$cur/$found"
                        fi
                    else
                        ok=0
                        break
                    fi
                done
                if [ "$ok" -eq 1 ] && { [ -e "$cur" ] || [ -L "$cur" ]; }; then
                    shift
                    set -- "$cur" "$@"
                fi
            fi
            ;;
    esac
fi
exec '"#
            .as_slice(),
    );
    for &byte in lua_target.as_os_str().as_bytes() {
        if byte == b'\'' {
            script.extend_from_slice(br"'\''");
        } else {
            script.push(byte);
        }
    }
    script.extend_from_slice(b"' \"$@\"\n");

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o755)
            .open(shim_path)?;
        file.write_all(&script)?;
    }
    Ok(())
}

#[cfg(unix)]
fn prepended_unix_path(
    tool_bin: &std::path::Path,
    host_path: Option<&std::ffi::OsStr>,
) -> std::ffi::OsString {
    let base = host_path
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| std::ffi::OsStr::new("/usr/bin:/bin"));
    let mut paths = vec![tool_bin.to_path_buf()];
    paths.extend(std::env::split_paths(base).filter(|entry| !entry.as_os_str().is_empty()));
    std::env::join_paths(paths).unwrap_or_else(|_| {
        let mut fallback = std::ffi::OsString::from(tool_bin.as_os_str());
        fallback.push(":");
        fallback.push(base);
        fallback
    })
}

fn create_output_log(path: &std::path::Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

fn emit_error(sink: &impl EventSink, step_id: &str, message: String) {
    sink.emit(EngineEvent::Error {
        step_id: Some(step_id.to_owned()),
        message,
    });
}

enum OutputMessage {
    Chunk { stream: Stream, bytes: Vec<u8> },
    Closed(Stream),
    ReadFailed { stream: Stream, error: io::Error },
}

fn spawn_reader<R, W, V>(
    mut reader: R,
    stream: Stream,
    output: Sender<OutputMessage>,
    raw_log: Arc<Mutex<W>>,
    mut stream_log: V,
) -> JoinHandle<()>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
    V: Write + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0; DISPLAY_CHUNK_BYTES];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    let _ = output.send(OutputMessage::Closed(stream));
                    break;
                }
                Ok(count) => {
                    let bytes = buffer[..count].to_vec();
                    let write_result = raw_log
                        .lock()
                        .map_err(|_| io::Error::other("raw output log lock was poisoned"))
                        .and_then(|mut log| log.write_all(&bytes))
                        .and_then(|()| stream_log.write_all(&bytes));
                    if let Err(error) = write_result {
                        let _ = output.send(OutputMessage::ReadFailed { stream, error });
                        break;
                    }
                    if output.send(OutputMessage::Chunk { stream, bytes }).is_err() {
                        break;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => {
                    let _ = output.send(OutputMessage::ReadFailed { stream, error });
                    break;
                }
            }
        }
    })
}

fn join_readers(readers: Vec<JoinHandle<()>>, sink: &impl EventSink, step_id: &str) -> bool {
    let mut joined = true;
    for reader in readers {
        if reader.join().is_err() {
            joined = false;
            emit_error(
                sink,
                step_id,
                "a WeiDU output reader thread panicked".to_owned(),
            );
        }
    }
    joined
}

fn append_prompt_result(file: &mut File, result: &PromptAnswerEvidence) -> io::Result<()> {
    serde_json::to_writer(&mut *file, result).map_err(io::Error::other)?;
    file.write_all(b"\n")?;
    file.flush()?;
    file.sync_data()
}

fn remember_output(tail: &mut Vec<u8>, bytes: &[u8]) {
    tail.extend_from_slice(bytes);
    if tail.len() > LAST_OUTPUT_BYTES {
        tail.drain(..tail.len() - LAST_OUTPUT_BYTES);
    }
}

/// Answers the authored prompts in their order. WeiDU goes on after a failed component, so a
/// prompt can come while one before it never will; that one stays unanswered, and the run's
/// evidence says whether that was right.
struct PromptMatcher<'a> {
    prompts: &'a [ResolvedPrompt],
    next: usize,
    answered: usize,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl<'a> PromptMatcher<'a> {
    fn new(prompts: &'a [ResolvedPrompt]) -> Self {
        Self {
            prompts,
            next: 0,
            answered: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    fn observe(
        &mut self,
        stream: Stream,
        bytes: &[u8],
        stdin: &mut Option<ChildStdin>,
    ) -> io::Result<Option<PromptAnswerEvidence>> {
        let waiting = &self.prompts[self.next..];
        let Some(longest) = waiting
            .iter()
            .map(|prompt| prompt.expected_output.len())
            .max()
        else {
            return Ok(None);
        };
        let buffer = match stream {
            Stream::Stdout => &mut self.stdout,
            Stream::Stderr => &mut self.stderr,
        };
        buffer.extend_from_slice(bytes);

        let asked = waiting
            .iter()
            .position(|prompt| contains(buffer, &prompt.expected_output));
        if let Some(offset) = asked {
            let index = self.next + offset;
            let prompt = &self.prompts[index];
            let writer = stdin
                .as_mut()
                .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "child stdin closed"))?;
            writer.write_all(&prompt.answer)?;
            writer.flush()?;
            let answered = PromptAnswerEvidence {
                index,
                expected_output_sha256: sha256_bytes(&prompt.expected_output),
                answer_sha256: sha256_bytes(&prompt.answer),
            };
            self.next = index + 1;
            self.answered += 1;
            self.stdout.clear();
            self.stderr.clear();
            if self.next == self.prompts.len() {
                *stdin = None;
            }
            return Ok(Some(answered));
        } else {
            let keep = longest.saturating_sub(1);
            if buffer.len() > keep {
                buffer.drain(..buffer.len() - keep);
            }
        }
        Ok(None)
    }

    fn complete(&self) -> bool {
        self.answered == self.prompts.len()
    }

    fn remaining(&self) -> usize {
        self.prompts.len() - self.answered
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::io::{self, Cursor, Write};
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::thread;

    use crossbeam_channel::bounded;

    use crate::events::{ChannelSink, Stream};

    use super::{join_readers, spawn_reader, weidu_command, Invocation, OutputMessage};

    #[derive(Default)]
    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("synthetic evidence write failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn evidence_write_failure_is_reported_by_the_reader() {
        let (sender, receiver) = bounded(1);
        let reader = spawn_reader(
            Cursor::new(b"child output".to_vec()),
            Stream::Stdout,
            sender,
            Arc::new(Mutex::new(Vec::<u8>::new())),
            FailingWriter,
        );

        assert!(matches!(
            receiver.recv().unwrap(),
            OutputMessage::ReadFailed {
                stream: Stream::Stdout,
                ..
            }
        ));
        reader.join().unwrap();
    }

    #[test]
    fn child_inherits_the_environment_except_the_current_directory_search_opt_out() {
        let command = weidu_command(
            &Invocation {
                program: PathBuf::from("weidu.exe"),
                cwd: PathBuf::from("game"),
                args: Vec::new(),
                prompts: Vec::new(),
                debug_path: PathBuf::from("weidu.debug.log"),
                identity_digest: "environment-test".to_owned(),
            },
            None,
            None,
        );

        // On Windows only `NoDefaultCurrentDirectoryInExePath` is removed; on Unix HOME and XDG
        // directories are also scoped to the attempt directory so nothing touches `$HOME`.
        #[cfg(not(unix))]
        let expected = [(
            OsStr::new("NoDefaultCurrentDirectoryInExePath"),
            None::<&OsStr>,
        )];
        #[cfg(unix)]
        let expected = [
            (OsStr::new("HOME"), Some(OsStr::new("home"))),
            (
                OsStr::new("NoDefaultCurrentDirectoryInExePath"),
                None::<&OsStr>,
            ),
            (OsStr::new("XDG_CACHE_HOME"), Some(OsStr::new("xdg-cache"))),
            (
                OsStr::new("XDG_CONFIG_HOME"),
                Some(OsStr::new("xdg-config")),
            ),
            (OsStr::new("XDG_DATA_HOME"), Some(OsStr::new("xdg-data"))),
        ];
        assert_eq!(command.get_envs().collect::<Vec<_>>(), expected);
    }

    #[cfg(unix)]
    #[test]
    fn unix_runner_terminates_orphaned_descendant_holding_pipes_on_leader_exit() {
        use std::ffi::OsString;
        use std::time::{Duration, Instant};

        let temp = tempfile::tempdir().unwrap();
        let attempt_dir = temp.path().join("attempt");
        std::fs::create_dir_all(&attempt_dir).unwrap();
        let command_line = "sleep 30 & echo leader-done ; exit 0".to_owned();
        let controls = super::RunnerControlHandle::new();
        let (sink, _events) = ChannelSink::unbounded();
        let invocation = Invocation {
            program: PathBuf::from("/bin/sh"),
            cwd: temp.path().to_path_buf(),
            args: vec![OsString::from("-c"), OsString::from(command_line)],
            prompts: Vec::new(),
            debug_path: attempt_dir.join("weidu.debug.log"),
            identity_digest: "orphan-pipe-test".to_owned(),
        };
        let started = Instant::now();
        let outcome = super::run_controlled(
            super::RunnerRequest {
                step_id: "step:orphan-pipe".to_owned(),
                invocation,
                output_log: attempt_dir.join("output.log"),
                silence_threshold: Duration::from_secs(30),
            },
            &controls,
            sink,
        );
        assert!(
            matches!(outcome, super::RunOutcome::Exited { code: 0 }),
            "unexpected outcome: {outcome:?}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "runner hung waiting on orphaned descendant pipe"
        );
    }

    #[test]
    fn reader_thread_panic_is_a_failed_join() {
        let (sink, _events) = ChannelSink::unbounded();
        let reader = thread::spawn(|| panic!("synthetic reader panic"));

        assert!(!join_readers(vec![reader], &sink, "install:test"));
    }

    #[cfg(unix)]
    #[test]
    fn unix_campaign_env_root_resolves_chriz_weidu_env_and_falls_back() {
        let chriz_attempt =
            PathBuf::from("/tmp/managed/.chriz/attempts/attempt-1/steps/0001-eet/attempt-0001");
        assert_eq!(
            super::unix_campaign_env_root(&chriz_attempt),
            PathBuf::from("/tmp/managed/.chriz/weidu-env")
        );
        let fallback_attempt = PathBuf::from("/tmp/standalone/attempt-0001");
        assert_eq!(
            super::unix_campaign_env_root(&fallback_attempt),
            fallback_attempt
        );
    }

    #[cfg(unix)]
    #[test]
    fn unix_runner_exposes_weidu_and_lua53_shim_on_path() {
        use std::ffi::OsString;
        use std::os::unix::fs::PermissionsExt;
        use std::time::Duration;

        let temp = tempfile::tempdir().unwrap();
        let fake_bin = temp.path().join("fake-bin");
        let managed = temp.path().join("managed");
        let attempt_dir = managed.join(".chriz/attempts/a1/steps/0001-eet/attempt-0001");
        let game = managed.join("game");
        std::fs::create_dir_all(&fake_bin).unwrap();
        std::fs::create_dir_all(&attempt_dir).unwrap();
        std::fs::create_dir_all(&game).unwrap();

        let lua53 = fake_bin.join("lua5.3");
        std::fs::write(
            &lua53,
            b"#!/bin/sh\n[ \"$1\" = \"--check\" ] && exit 0\nexit 2\n",
        )
        .unwrap();
        std::fs::set_permissions(&lua53, std::fs::Permissions::from_mode(0o755)).unwrap();

        let host_path = std::env::join_paths([
            &fake_bin,
            &PathBuf::from("/usr/bin"),
            &PathBuf::from("/bin"),
        ])
        .unwrap();

        let script = "which weidu >/dev/null && which lua >/dev/null && lua --check && printf '%s' \"$XDG_DATA_HOME\" > xdg.out";
        let controls = super::RunnerControlHandle::new();
        let registration = controls.register().unwrap();
        let (sink, _events) = ChannelSink::unbounded();
        let invocation = Invocation {
            program: PathBuf::from("/bin/sh"),
            cwd: game.clone(),
            args: vec![OsString::from("-c"), OsString::from(script)],
            prompts: Vec::new(),
            debug_path: attempt_dir.join("weidu.debug.log"),
            identity_digest: "unix-shim-test".to_owned(),
        };
        let outcome = super::run_with_host_path(
            super::RunnerRequest {
                step_id: "install:eet".to_owned(),
                invocation,
                output_log: attempt_dir.join("output.log"),
                silence_threshold: Duration::from_secs(30),
            },
            registration.receiver().clone(),
            sink,
            Some(host_path.as_os_str()),
        );
        assert!(
            matches!(outcome, super::RunOutcome::Exited { code: 0 }),
            "unexpected outcome: {outcome:?}"
        );
        assert_eq!(
            std::fs::read_to_string(game.join("xdg.out")).unwrap(),
            managed.join(".chriz/weidu-env/xdg-data").to_string_lossy()
        );
    }

    #[cfg(unix)]
    #[test]
    fn unix_lua_shim_resolves_lowercased_script_and_rejects_ambiguous_case() {
        use std::ffi::OsString;
        use std::os::unix::fs::PermissionsExt;
        use std::time::Duration;

        let temp = tempfile::tempdir().unwrap();
        let fake_bin = temp.path().join("fake-bin");
        let managed = temp.path().join("managed");
        let attempt_dir = managed.join(".chriz/attempts/a1/steps/0001-eet/attempt-0001");
        let game = managed.join("game");
        let eet_lib = game.join("EET/lib");
        std::fs::create_dir_all(&fake_bin).unwrap();
        std::fs::create_dir_all(&attempt_dir).unwrap();
        std::fs::create_dir_all(&eet_lib).unwrap();
        std::fs::write(eet_lib.join("tlk_cnt.lua"), b"tlk-ok").unwrap();

        let lua53 = fake_bin.join("lua5.3");
        std::fs::write(
            &lua53,
            b"#!/bin/sh\n[ -r \"$1\" ] || exit 1\n/bin/cat \"$1\" > \"$2\"\n",
        )
        .unwrap();
        std::fs::set_permissions(&lua53, std::fs::Permissions::from_mode(0o755)).unwrap();

        let host_path = std::env::join_paths([&fake_bin]).unwrap();
        let controls = super::RunnerControlHandle::new();

        let outcome = {
            let registration = controls.register().unwrap();
            let (sink, _events) = ChannelSink::unbounded();
            super::run_with_host_path(
                super::RunnerRequest {
                    step_id: "install:eet".to_owned(),
                    invocation: Invocation {
                        program: PathBuf::from("/bin/sh"),
                        cwd: game.clone(),
                        args: vec![
                            OsString::from("-c"),
                            OsString::from("lua eet/lib/tlk_cnt.lua out.txt"),
                        ],
                        prompts: Vec::new(),
                        debug_path: attempt_dir.join("weidu.debug.log"),
                        identity_digest: "unix-lua-shim-case-ok".to_owned(),
                    },
                    output_log: attempt_dir.join("output-ok.log"),
                    silence_threshold: Duration::from_secs(30),
                },
                registration.receiver().clone(),
                sink,
                Some(host_path.as_os_str()),
            )
        };
        assert!(
            matches!(outcome, super::RunOutcome::Exited { code: 0 }),
            "unexpected outcome: {outcome:?}"
        );
        assert_eq!(
            std::fs::read_to_string(game.join("out.txt")).unwrap(),
            "tlk-ok"
        );

        std::fs::create_dir_all(game.join("DupDir")).unwrap();
        if std::fs::create_dir(game.join("DUPDIR")).is_ok() {
            std::fs::write(game.join("DupDir/s.lua"), b"one").unwrap();
            std::fs::write(game.join("DUPDIR/s.lua"), b"two").unwrap();
            let reg_dup = controls.register().unwrap();
            let (sink_dup, _events_dup) = ChannelSink::unbounded();
            let outcome_dup = super::run_with_host_path(
                super::RunnerRequest {
                    step_id: "install:eet-dup".to_owned(),
                    invocation: Invocation {
                        program: PathBuf::from("/bin/sh"),
                        cwd: game.clone(),
                        args: vec![
                            OsString::from("-c"),
                            OsString::from("lua dupdir/s.lua dup.out"),
                        ],
                        prompts: Vec::new(),
                        debug_path: attempt_dir.join("weidu-dup.debug.log"),
                        identity_digest: "unix-lua-shim-case-dup".to_owned(),
                    },
                    output_log: attempt_dir.join("output-dup.log"),
                    silence_threshold: Duration::from_secs(30),
                },
                reg_dup.receiver().clone(),
                sink_dup,
                Some(host_path.as_os_str()),
            );
            assert!(
                !matches!(outcome_dup, super::RunOutcome::Exited { code: 0 }),
                "ambiguous case collision unexpectedly succeeded: {outcome_dup:?}"
            );
            assert!(!game.join("dup.out").exists());
        }
    }
}
