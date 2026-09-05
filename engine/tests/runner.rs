use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use bg_engine::events::{ChannelSink, EngineEvent};
use bg_engine::weidu::invocation::{Invocation, ResolvedPrompt};
use bg_engine::weidu::runner::{
    run, run_controlled, RunOutcome, RunnerControl, RunnerControlHandle, RunnerRequest,
};
use crossbeam_channel::{Receiver, Sender};
use tempfile::TempDir;

const MOCK_CHILD: &str = env!("CARGO_BIN_EXE_mock-child");

struct Harness {
    _temp: TempDir,
    controls: Sender<RunnerControl>,
    events: Receiver<EngineEvent>,
    outcomes: Receiver<RunOutcome>,
    thread: Option<thread::JoinHandle<()>>,
    raw_log: PathBuf,
}

impl Harness {
    fn start(
        mode: &str,
        mode_args: impl IntoIterator<Item = OsString>,
        prompts: Vec<ResolvedPrompt>,
        silence_threshold: Duration,
    ) -> Self {
        Self::start_program(
            PathBuf::from(MOCK_CHILD),
            std::iter::once(OsString::from(mode)).chain(mode_args),
            prompts,
            silence_threshold,
        )
    }

    fn start_program(
        program: PathBuf,
        args: impl IntoIterator<Item = OsString>,
        prompts: Vec<ResolvedPrompt>,
        silence_threshold: Duration,
    ) -> Self {
        let temp = TempDir::new().unwrap();
        let raw_log = temp.path().join("attempt-output.log");
        let request = RunnerRequest {
            invocation: Invocation {
                program,
                cwd: temp.path().to_path_buf(),
                args: args.into_iter().collect(),
                prompts,
                debug_path: temp.path().join("weidu.debug.log"),
                identity_digest: "runner-test".to_owned(),
            },
            step_id: "install:testmod".to_owned(),
            output_log: raw_log.clone(),
            silence_threshold,
        };
        let (sink, events) = ChannelSink::unbounded();
        let (controls, control_rx) = crossbeam_channel::unbounded();
        let (outcome_tx, outcomes) = crossbeam_channel::bounded(1);
        let thread = thread::spawn(move || {
            let outcome = run(request, control_rx, sink);
            let _ = outcome_tx.send(outcome);
        });

        Self {
            _temp: temp,
            controls,
            events,
            outcomes,
            thread: Some(thread),
            raw_log,
        }
    }

    fn wait_for_event(
        &self,
        timeout: Duration,
        predicate: impl Fn(&EngineEvent) -> bool,
    ) -> EngineEvent {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let event = self
                .events
                .recv_timeout(remaining)
                .unwrap_or_else(|error| panic!("event did not arrive before timeout: {error}"));
            if predicate(&event) {
                return event;
            }
        }
    }

    fn wait_outcome(&mut self, timeout: Duration) -> RunOutcome {
        match self.outcomes.recv_timeout(timeout) {
            Ok(outcome) => {
                self.thread.take().unwrap().join().unwrap();
                outcome
            }
            Err(error) => {
                let _ = self.controls.send(RunnerControl::Cancel);
                let _ = self.outcomes.recv_timeout(Duration::from_secs(3));
                if let Some(thread) = self.thread.take() {
                    thread.join().unwrap();
                }
                panic!("runner did not finish before timeout: {error}");
            }
        }
    }

    fn console_text(&self) -> String {
        self.events
            .try_iter()
            .filter_map(|event| match event {
                EngineEvent::ConsoleLine { line, .. } => Some(line),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn stdout_log(&self) -> PathBuf {
        self.raw_log.with_file_name("stdout.log")
    }

    fn stderr_log(&self) -> PathBuf {
        self.raw_log.with_file_name("stderr.log")
    }

    fn prompt_results_log(&self) -> PathBuf {
        self.raw_log.with_file_name("prompt-results.jsonl")
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = self.controls.send(RunnerControl::Cancel);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[test]
fn answer_waits_for_the_authored_prompt_even_when_fragmented_without_a_newline() {
    let mut harness = Harness::start(
        "fragmented-prompt",
        [],
        vec![ResolvedPrompt {
            expected_output: b"Choose option: ".to_vec(),
            answer: b"7\n".to_vec(),
        }],
        Duration::from_secs(2),
    );

    assert_eq!(
        harness.wait_outcome(Duration::from_secs(5)),
        RunOutcome::Exited { code: 0 }
    );
    assert!(harness.console_text().contains("answer:7"));
    assert!(fs::read(&harness.raw_log)
        .unwrap()
        .windows(b"Choose option: ".len())
        .any(|window| window == b"Choose option: "));
    let results = fs::read_to_string(harness.prompt_results_log()).unwrap();
    let result: serde_json::Value = serde_json::from_str(results.trim()).unwrap();
    assert_eq!(result["index"], 0);
    assert_eq!(result["expected_output_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(result["answer_sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn child_exit_before_an_authored_prompt_is_not_accepted() {
    let mut harness = Harness::start(
        "quiet",
        [OsString::from("0")],
        vec![ResolvedPrompt {
            expected_output: b"Never emitted: ".to_vec(),
            answer: b"yes\n".to_vec(),
        }],
        Duration::from_secs(2),
    );

    assert_eq!(
        harness.wait_outcome(Duration::from_secs(5)),
        RunOutcome::SpawnFailed
    );
}

#[test]
fn failed_prompt_answer_write_is_not_accepted() {
    let oversized_answer = vec![b'x'; 8 * 1024 * 1024];
    let mut harness = Harness::start(
        "emit-prompt-exit",
        [],
        vec![ResolvedPrompt {
            expected_output: b"Choose option: ".to_vec(),
            answer: oversized_answer,
        }],
        Duration::from_secs(2),
    );

    assert_eq!(
        harness.wait_outcome(Duration::from_secs(5)),
        RunOutcome::SpawnFailed
    );
    assert_eq!(fs::metadata(harness.prompt_results_log()).unwrap().len(), 0);
}

#[test]
fn unmatched_prompt_alerts_once_and_stays_alive_until_explicit_cancel() {
    let threshold = Duration::from_millis(80);
    let mut harness = Harness::start(
        "unmatched-prompt",
        [],
        vec![ResolvedPrompt {
            expected_output: b"Expected prompt: ".to_vec(),
            answer: b"yes\n".to_vec(),
        }],
        threshold,
    );

    let attention = harness.wait_for_event(Duration::from_secs(3), |event| {
        matches!(event, EngineEvent::AttentionRequired { .. })
    });
    match attention {
        EngineEvent::AttentionRequired {
            step_id,
            last_output,
            ..
        } => {
            assert_eq!(step_id, "install:testmod");
            assert!(last_output.contains("Unexpected prompt"));
        }
        _ => unreachable!(),
    }

    thread::sleep(threshold * 3);
    assert!(matches!(
        harness.outcomes.try_recv(),
        Err(crossbeam_channel::TryRecvError::Empty)
    ));
    assert_eq!(
        harness
            .events
            .try_iter()
            .filter(|event| matches!(event, EngineEvent::AttentionRequired { .. }))
            .count(),
        0,
        "one silence interval must not spam repeated alerts"
    );

    harness.controls.send(RunnerControl::Cancel).unwrap();
    assert_eq!(
        harness.wait_outcome(Duration::from_secs(5)),
        RunOutcome::Cancelled
    );
}

#[test]
fn legitimately_quiet_child_can_continue_waiting_and_exit_normally() {
    let mut harness = Harness::start(
        "quiet",
        [OsString::from("300")],
        vec![],
        Duration::from_millis(200),
    );

    harness.wait_for_event(Duration::from_secs(3), |event| {
        matches!(event, EngineEvent::AttentionRequired { .. })
    });
    harness
        .controls
        .send(RunnerControl::ContinueWaiting)
        .unwrap();

    assert_eq!(
        harness.wait_outcome(Duration::from_secs(5)),
        RunOutcome::Exited { code: 0 }
    );
    assert!(harness.console_text().contains("quiet-exit"));
}

#[test]
fn no_prompt_invocation_receives_closed_stdin() {
    let mut harness = Harness::start("expect-eof", [], vec![], Duration::from_secs(2));

    assert_eq!(
        harness.wait_outcome(Duration::from_secs(5)),
        RunOutcome::Exited { code: 0 }
    );
    assert!(harness.console_text().contains("stdin-closed"));
}

#[test]
fn invalid_utf8_is_lossy_for_display_but_exact_in_the_attempt_log() {
    const RAW: &[u8] = b"raw-\x80-bytes";
    const ERROR: &[u8] = b"err-\xff-bytes";
    let mut harness = Harness::start("invalid-utf8", [], vec![], Duration::from_secs(2));

    assert_eq!(
        harness.wait_outcome(Duration::from_secs(5)),
        RunOutcome::Exited { code: 0 }
    );
    assert!(harness.console_text().contains("raw-\u{fffd}-bytes"));
    assert!(fs::read(&harness.raw_log)
        .unwrap()
        .windows(RAW.len())
        .any(|window| window == RAW));
    assert_eq!(fs::read(harness.stdout_log()).unwrap(), RAW);
    assert_eq!(fs::read(harness.stderr_log()).unwrap(), ERROR);
}

#[test]
fn large_simultaneous_stdout_and_stderr_do_not_deadlock() {
    const BYTES_PER_STREAM: usize = 1024 * 1024;
    let mut harness = Harness::start(
        "large-streams",
        [OsString::from(BYTES_PER_STREAM.to_string())],
        vec![],
        Duration::from_secs(3),
    );

    assert_eq!(
        harness.wait_outcome(Duration::from_secs(15)),
        RunOutcome::Exited { code: 0 }
    );
    assert_eq!(
        fs::metadata(&harness.raw_log).unwrap().len(),
        (BYTES_PER_STREAM * 2) as u64
    );
}

#[cfg(windows)]
#[test]
fn cancel_terminates_the_complete_process_tree() {
    let marker_temp = TempDir::new().unwrap();
    let marker = marker_temp.path().join("descendant-survived.txt");
    let mut harness = Harness::start(
        "spawn-child",
        [marker.as_os_str().to_owned()],
        vec![],
        Duration::from_secs(3),
    );
    harness.wait_for_event(Duration::from_secs(5), |event| {
        matches!(event, EngineEvent::ConsoleLine { line, .. } if line.contains("child-spawned"))
    });

    harness.controls.send(RunnerControl::Cancel).unwrap();
    assert_eq!(
        harness.wait_outcome(Duration::from_secs(5)),
        RunOutcome::Cancelled
    );
    thread::sleep(Duration::from_millis(900));
    assert!(!marker.exists(), "cancelled descendant wrote its marker");
}

#[cfg(windows)]
#[test]
fn shared_control_handle_cancels_and_joins_the_complete_process_tree() {
    let temp = TempDir::new().unwrap();
    let marker = temp.path().join("shared-control-descendant-survived.txt");
    let raw_log = temp.path().join("shared-control-output.log");
    let request = RunnerRequest {
        invocation: Invocation {
            program: PathBuf::from(MOCK_CHILD),
            cwd: temp.path().to_path_buf(),
            args: vec![OsString::from("spawn-child"), marker.as_os_str().to_owned()],
            prompts: vec![],
            debug_path: temp.path().join("weidu.debug.log"),
            identity_digest: "shared-control-test".to_owned(),
        },
        step_id: "install:shared-control".to_owned(),
        output_log: raw_log.clone(),
        silence_threshold: Duration::from_secs(3),
    };
    let controls = RunnerControlHandle::new();
    let (sink, events) = ChannelSink::unbounded();
    let (outcome_tx, outcomes) = crossbeam_channel::bounded(1);
    let runner_controls = controls.clone();
    let runner_thread = thread::spawn(move || {
        let outcome = run_controlled(request, &runner_controls, sink);
        let _ = outcome_tx.send(outcome);
    });

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .expect("child process did not start before timeout");
        if matches!(event, EngineEvent::ConsoleLine { line, .. } if line.contains("child-spawned"))
        {
            break;
        }
    }

    controls.cancel();
    assert_eq!(
        outcomes
            .recv_timeout(Duration::from_secs(5))
            .expect("runner did not finish after shared cancellation"),
        RunOutcome::Cancelled
    );
    runner_thread.join().unwrap();
    thread::sleep(Duration::from_millis(900));
    assert!(!marker.exists(), "cancelled descendant wrote its marker");
    assert!(
        fs::read(&raw_log)
            .unwrap()
            .windows(b"child-spawned".len())
            .any(|window| window == b"child-spawned"),
        "cancellation discarded the durable raw process log"
    );
}

#[test]
fn cancellation_requested_before_runner_registration_is_not_lost() {
    let controls = RunnerControlHandle::new();
    controls.cancel();

    let registration = controls
        .register()
        .expect("register runner after cancellation");

    assert_eq!(
        registration
            .receiver()
            .recv_timeout(Duration::from_secs(1))
            .expect("pending cancellation was not delivered"),
        RunnerControl::Cancel
    );
}

#[test]
fn shared_control_handle_forwards_continue_waiting_to_the_active_runner() {
    let controls = RunnerControlHandle::new();
    let registration = controls.register().expect("register active runner");

    controls.continue_waiting();

    assert_eq!(
        registration
            .receiver()
            .recv_timeout(Duration::from_secs(1))
            .expect("continue-waiting decision was not delivered"),
        RunnerControl::ContinueWaiting
    );
}

#[test]
fn pause_request_is_sticky_without_interrupting_the_active_runner() {
    let controls = RunnerControlHandle::new();
    let registration = controls.register().expect("register active runner");

    controls.pause_after_boundary();

    assert!(controls.pause_requested());
    assert!(registration.receiver().try_recv().is_err());
}

#[test]
fn process_spawn_failure_is_reported_without_panicking() {
    let mut harness = Harness::start_program(
        Path::new("definitely-missing-runner-test.exe").to_path_buf(),
        [],
        vec![],
        Duration::from_secs(1),
    );

    assert_eq!(
        harness.wait_outcome(Duration::from_secs(3)),
        RunOutcome::SpawnFailed
    );
    assert!(harness
        .events
        .try_iter()
        .any(|event| matches!(event, EngineEvent::Error { step_id: Some(id), .. } if id == "install:testmod")));
}
