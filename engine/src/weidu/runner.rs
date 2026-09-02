//! Streaming child-process supervision for one resolved WeiDU invocation.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};

use crate::events::{EngineEvent, EventSink, Stream};

use super::invocation::{Invocation, ResolvedPrompt};

const DISPLAY_CHUNK_BYTES: usize = 4 * 1024;
const DISPLAY_CHANNEL_CAPACITY: usize = 64;
const LAST_OUTPUT_BYTES: usize = 4 * 1024;
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// A user decision delivered after the runner requests attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerControl {
    /// Rearm the silence watchdog without interrupting the process.
    ContinueWaiting,
    /// Explicitly terminate the supervised process tree.
    Cancel,
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

    let has_prompts = !request.invocation.prompts.is_empty();
    let mut command = Command::new(&request.invocation.program);
    command
        .current_dir(&request.invocation.cwd)
        .args(&request.invocation.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(if has_prompts {
            Stdio::piped()
        } else {
            Stdio::null()
        });

    let mut child = match command.spawn() {
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
        ),
        spawn_reader(stderr, Stream::Stderr, output_tx, Arc::clone(&raw_log)),
    ];

    let mut matcher = PromptMatcher::new(&request.invocation.prompts);
    let mut last_output = Vec::new();
    let mut stdout_closed = false;
    let mut stderr_closed = false;
    let mut exit_status = None;
    let mut attention_active = false;
    let mut silence_deadline = Instant::now() + request.silence_threshold;
    let mut cancelled = false;

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
            match child.try_wait() {
                Ok(status) => exit_status = status,
                Err(error) => {
                    emit_error(
                        &sink,
                        &request.step_id,
                        format!("could not query WeiDU process state: {error}"),
                    );
                    cancelled = true;
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
                reason: "WeiDU produced no output before the silence threshold; it may be waiting for an unexpected prompt."
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
                if !attention_active {
                    silence_deadline = Instant::now() + request.silence_threshold;
                }
                sink.emit(EngineEvent::ConsoleLine {
                    step_id: request.step_id.clone(),
                    stream,
                    line: String::from_utf8_lossy(&bytes).into_owned(),
                });
                if let Err(error) = matcher.observe(stream, &bytes, &mut stdin) {
                    emit_error(
                        &sink,
                        &request.step_id,
                        format!("could not answer an authored WeiDU prompt: {error}"),
                    );
                    stdin = None;
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
                match stream {
                    Stream::Stdout => stdout_closed = true,
                    Stream::Stderr => stderr_closed = true,
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                stdout_closed = true;
                stderr_closed = true;
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
    join_readers(readers, &sink, &request.step_id);
    if let Ok(mut log) = raw_log.lock() {
        let _ = log.flush();
    }

    if cancelled {
        RunOutcome::Cancelled
    } else {
        let status = exit_status.or_else(|| child.wait().ok());
        RunOutcome::Exited {
            code: status.and_then(|status| status.code()).unwrap_or(-1),
        }
    }
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

fn spawn_reader<R: Read + Send + 'static>(
    mut reader: R,
    stream: Stream,
    output: Sender<OutputMessage>,
    raw_log: Arc<Mutex<File>>,
) -> JoinHandle<()> {
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
                        .and_then(|mut log| log.write_all(&bytes));
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

fn join_readers(readers: Vec<JoinHandle<()>>, sink: &impl EventSink, step_id: &str) {
    for reader in readers {
        if reader.join().is_err() {
            emit_error(
                sink,
                step_id,
                "a WeiDU output reader thread panicked".to_owned(),
            );
        }
    }
}

fn remember_output(tail: &mut Vec<u8>, bytes: &[u8]) {
    tail.extend_from_slice(bytes);
    if tail.len() > LAST_OUTPUT_BYTES {
        tail.drain(..tail.len() - LAST_OUTPUT_BYTES);
    }
}

struct PromptMatcher<'a> {
    prompts: &'a [ResolvedPrompt],
    next: usize,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl<'a> PromptMatcher<'a> {
    fn new(prompts: &'a [ResolvedPrompt]) -> Self {
        Self {
            prompts,
            next: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    fn observe(
        &mut self,
        stream: Stream,
        bytes: &[u8],
        stdin: &mut Option<ChildStdin>,
    ) -> io::Result<()> {
        let Some(prompt) = self.prompts.get(self.next) else {
            return Ok(());
        };
        let buffer = match stream {
            Stream::Stdout => &mut self.stdout,
            Stream::Stderr => &mut self.stderr,
        };
        buffer.extend_from_slice(bytes);

        if contains(buffer, &prompt.expected_output) {
            let writer = stdin
                .as_mut()
                .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "child stdin closed"))?;
            writer.write_all(&prompt.answer)?;
            writer.flush()?;
            self.next += 1;
            self.stdout.clear();
            self.stderr.clear();
            if self.next == self.prompts.len() {
                *stdin = None;
            }
        } else {
            let keep = prompt.expected_output.len().saturating_sub(1);
            if buffer.len() > keep {
                buffer.drain(..buffer.len() - keep);
            }
        }
        Ok(())
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

#[cfg(windows)]
struct ProcessGroup {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl ProcessGroup {
    fn new() -> io::Result<Self> {
        use std::ffi::c_void;
        use std::mem::{size_of, zeroed};
        use std::ptr;
        use windows_sys::Win32::System::JobObjects::{
            JobObjectExtendedLimitInformation, SetInformationJobObject,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };

        #[link(name = "kernel32")]
        extern "system" {
            fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> *mut c_void;
        }

        // SAFETY: null attributes/name request a private unnamed Job Object; the returned handle
        // is checked and owned by `ProcessGroup` until Drop closes it.
        let handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: this Windows POD is valid when zero-initialized and the API receives its exact
        // pointer and byte size for `JobObjectExtendedLimitInformation`.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `handle` is live and `limits` points to a correctly sized value for this class.
        let configured = unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            let error = io::Error::last_os_error();
            // SAFETY: `handle` is the live handle returned above and is closed exactly once here.
            unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
            return Err(error);
        }
        Ok(Self { handle })
    }

    fn attach(&self, child: &Child) -> io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;

        // SAFETY: both handles are live for the duration of the call. Ownership is unchanged.
        if unsafe { AssignProcessToJobObject(self.handle, child.as_raw_handle()) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn terminate(&self, _child: &mut Child) -> io::Result<()> {
        use windows_sys::Win32::System::JobObjects::TerminateJobObject;

        // SAFETY: the Job Object handle is live; explicit cancellation authorizes termination.
        if unsafe { TerminateJobObject(self.handle, 1) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(windows)]
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        // SAFETY: this type owns the live handle and Drop runs once. KILL_ON_JOB_CLOSE is the
        // crash-safety backstop required for descendants that outlive their parent.
        unsafe { windows_sys::Win32::Foundation::CloseHandle(self.handle) };
    }
}

#[cfg(not(windows))]
struct ProcessGroup;

#[cfg(not(windows))]
impl ProcessGroup {
    fn new() -> io::Result<Self> {
        Ok(Self)
    }

    fn attach(&self, _child: &Child) -> io::Result<()> {
        Ok(())
    }

    fn terminate(&self, child: &mut Child) -> io::Result<()> {
        child.kill()
    }
}
