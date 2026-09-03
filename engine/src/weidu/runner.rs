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
            match child.try_wait() {
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
    if !cancelled && !matcher.complete() {
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
    ) -> io::Result<Option<PromptAnswerEvidence>> {
        let Some(prompt) = self.prompts.get(self.next) else {
            return Ok(None);
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
            let answered = PromptAnswerEvidence {
                index: self.next,
                expected_output_sha256: sha256_bytes(&prompt.expected_output),
                answer_sha256: sha256_bytes(&prompt.answer),
            };
            self.next += 1;
            self.stdout.clear();
            self.stderr.clear();
            if self.next == self.prompts.len() {
                *stdin = None;
            }
            return Ok(Some(answered));
        } else {
            let keep = prompt.expected_output.len().saturating_sub(1);
            if buffer.len() > keep {
                buffer.drain(..buffer.len() - keep);
            }
        }
        Ok(None)
    }

    fn complete(&self) -> bool {
        self.next == self.prompts.len()
    }

    fn remaining(&self) -> usize {
        self.prompts.len() - self.next
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
    use std::io::{self, Cursor, Write};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use crossbeam_channel::bounded;

    use crate::events::{ChannelSink, Stream};

    use super::{join_readers, spawn_reader, OutputMessage};

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
    fn reader_thread_panic_is_a_failed_join() {
        let (sink, _events) = ChannelSink::unbounded();
        let reader = thread::spawn(|| panic!("synthetic reader panic"));

        assert!(!join_readers(vec![reader], &sink, "install:test"));
    }
}
