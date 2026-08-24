//! Progress and diagnostic events, and the sinks that consume them.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Progress/diagnostic events emitted by the engine (serialisable for the future Tauri bridge).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineEvent {
    /// An install phase (see [`crate::manifest::Phase`]) has begun.
    PhaseStarted {
        /// Human-readable phase name.
        name: String,
    },
    /// A single install step has begun.
    StepStarted {
        /// Step id, stable across a run and used to correlate later events.
        id: String,
        /// Human-readable description of the step.
        label: String,
    },
    /// A step reported quantitative progress.
    StepProgress {
        /// Step id, as in [`EngineEvent::StepStarted`].
        id: String,
        /// Units of work completed so far. Producers keep this within the JS-safe
        /// integer range (`< 2^53`) — the bridge serialises it as a JSON number.
        done: u64,
        /// Total units of work expected (same range rule as `done`).
        total: u64,
    },
    /// One line of output captured from a child process.
    ConsoleLine {
        /// Step id the line belongs to.
        step_id: String,
        /// Which of the child's streams the line came from.
        stream: Stream,
        /// The line itself, without its trailing newline.
        line: String,
    },
    /// A step ended; `outcome` says how.
    StepFinished {
        /// Step id, as in [`EngineEvent::StepStarted`].
        id: String,
        /// How the step ended.
        outcome: StepOutcome,
    },
    /// A mod cannot be fetched automatically; the user has to download it.
    ManualDownloadNeeded {
        /// Manifest id of the mod that needs downloading.
        mod_id: String,
        /// Page the user should download the archive from.
        page: String,
        /// SHA-256 the downloaded archive must have.
        expected_sha256: String,
        /// Directory the downloaded archive should be dropped into, in native
        /// display form. A `String` rather than `PathBuf` so the event always
        /// serialises (serde rejects non-Unicode native paths); the engine keeps
        /// the real `PathBuf` and only reports it here.
        drop_dir: String,
    },
    /// Something went wrong, either inside a step or at the run level.
    Error {
        /// Step the error belongs to, or `None` for run-level errors.
        step_id: Option<String>,
        /// Human-readable error message.
        message: String,
    },
}

/// Which standard stream a captured [`EngineEvent::ConsoleLine`] came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stream {
    /// The child's standard output.
    Stdout,
    /// The child's standard error.
    Stderr,
}

/// How a step ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepOutcome {
    /// The step ran and succeeded.
    Succeeded,
    /// The step ran and failed.
    Failed,
    /// The step was not run (already installed, or excluded by the preset).
    Skipped,
}

/// Receives engine events. Implementations must be cheap and never panic.
pub trait EventSink: Send {
    /// Consume one event.
    fn emit(&self, event: EngineEvent);
}

impl EventSink for Box<dyn EventSink> {
    fn emit(&self, event: EngineEvent) {
        (**self).emit(event);
    }
}

impl<T: EventSink + Sync> EventSink for Arc<T> {
    fn emit(&self, event: EngineEvent) {
        (**self).emit(event);
    }
}

/// Writes one human-readable line per event to stderr.
///
/// Write failures (closed or non-blocking stderr) are ignored: reporting progress
/// must never take the engine down. Each event is written under the stderr lock,
/// so lines from concurrent sinks do not interleave mid-line.
pub struct ConsoleSink;

impl ConsoleSink {
    /// Render `event` as the single line `ConsoleSink` prints for it (no newline).
    pub fn render(event: &EngineEvent) -> String {
        match event {
            EngineEvent::PhaseStarted { name } => format!("== {name} =="),
            EngineEvent::StepStarted { id, label } => format!("-> [{id}] {label}"),
            EngineEvent::StepProgress { id, done, total } => format!("   [{id}] {done}/{total}"),
            EngineEvent::ConsoleLine {
                step_id,
                stream,
                line,
            } => {
                let prefix = match stream {
                    Stream::Stdout => "   ",
                    Stream::Stderr => "!  ",
                };
                format!("{prefix}[{step_id}] {line}")
            }
            EngineEvent::StepFinished { id, outcome } => {
                let outcome = format!("{outcome:?}").to_lowercase();
                format!("<- [{id}] {outcome}")
            }
            EngineEvent::ManualDownloadNeeded {
                mod_id,
                page,
                expected_sha256,
                drop_dir,
            } => format!(
                "?? manual download needed for {mod_id}: {page} \
                 (drop into {drop_dir}, sha256 {expected_sha256})"
            ),
            EngineEvent::Error { step_id, message } => {
                let step_id = step_id.as_deref().unwrap_or("-");
                format!("!! [{step_id}] {message}")
            }
        }
    }
}

impl EventSink for ConsoleSink {
    fn emit(&self, event: EngineEvent) {
        use std::io::Write as _;
        let line = Self::render(&event);
        let stderr = std::io::stderr();
        let mut out = stderr.lock();
        // Deliberately fallible-and-ignored: a closed stderr must not panic the engine.
        let _ = writeln!(out, "{line}");
    }
}

/// Forwards events over a crossbeam channel.
///
/// Delivery policy: `emit` uses a blocking `send`. On an **unbounded** channel (the
/// recommended configuration — see [`ChannelSink::unbounded`]) this never blocks. On a
/// bounded channel it applies back-pressure: a consumer that stops draining will stall
/// the engine, which is the intended trade-off over silently dropping progress events
/// (a lost `StepFinished` would leave the UI hanging). A dropped receiver is not an
/// error — events are silently discarded, so a UI that closes mid-run does not break
/// the install.
pub struct ChannelSink(
    /// The sending half of the channel events are forwarded on.
    pub crossbeam_channel::Sender<EngineEvent>,
);

impl ChannelSink {
    /// Create a sink on a fresh unbounded channel and return it with its receiver.
    pub fn unbounded() -> (ChannelSink, crossbeam_channel::Receiver<EngineEvent>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (ChannelSink(tx), rx)
    }
}

impl EventSink for ChannelSink {
    fn emit(&self, event: EngineEvent) {
        // A dropped receiver is not an error: the consumer simply stopped listening.
        let _ = self.0.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// One of every [`EngineEvent`] variant, for exhaustive smoke tests.
    fn sample_events() -> Vec<EngineEvent> {
        vec![
            EngineEvent::PhaseStarted {
                name: "bg1-pre-merge".to_string(),
            },
            EngineEvent::StepStarted {
                id: "eet/0".to_string(),
                label: "EET core".to_string(),
            },
            EngineEvent::StepProgress {
                id: "eet/0".to_string(),
                done: 3,
                total: 7,
            },
            EngineEvent::ConsoleLine {
                step_id: "eet/0".to_string(),
                stream: Stream::Stdout,
                line: "Copying files...".to_string(),
            },
            EngineEvent::ConsoleLine {
                step_id: "eet/0".to_string(),
                stream: Stream::Stderr,
                line: "WARNING: something".to_string(),
            },
            EngineEvent::StepFinished {
                id: "eet/0".to_string(),
                outcome: StepOutcome::Succeeded,
            },
            EngineEvent::StepFinished {
                id: "eet/1".to_string(),
                outcome: StepOutcome::Failed,
            },
            EngineEvent::StepFinished {
                id: "eet/2".to_string(),
                outcome: StepOutcome::Skipped,
            },
            EngineEvent::ManualDownloadNeeded {
                mod_id: "bg1re".to_string(),
                page: "https://example.invalid/bg1re".to_string(),
                expected_sha256: "abc123".to_string(),
                drop_dir: "downloads/manual".to_string(),
            },
            EngineEvent::Error {
                step_id: Some("eet/0".to_string()),
                message: "weidu exited with 1".to_string(),
            },
            EngineEvent::Error {
                step_id: None,
                message: "no game directory".to_string(),
            },
        ]
    }

    #[test]
    fn channel_sink_delivers_in_order() {
        let (tx, rx) = crossbeam_channel::unbounded();
        let sink = ChannelSink(tx);

        let sent = vec![
            EngineEvent::PhaseStarted {
                name: "main".to_string(),
            },
            EngineEvent::StepStarted {
                id: "a".to_string(),
                label: "first".to_string(),
            },
            EngineEvent::StepFinished {
                id: "a".to_string(),
                outcome: StepOutcome::Succeeded,
            },
        ];
        for event in &sent {
            sink.emit(event.clone());
        }

        for expected in &sent {
            assert_eq!(&rx.recv().unwrap(), expected);
        }
    }

    #[test]
    fn channel_sink_ignores_dropped_receiver() {
        let (tx, rx) = crossbeam_channel::bounded(1);
        drop(rx);
        let sink = ChannelSink(tx);

        // Must not panic, even with nobody listening.
        sink.emit(EngineEvent::PhaseStarted {
            name: "main".to_string(),
        });
        sink.emit(EngineEvent::Error {
            step_id: None,
            message: "nobody is listening".to_string(),
        });
    }

    #[test]
    fn console_sink_does_not_panic() {
        let sink = ConsoleSink;
        for event in sample_events() {
            sink.emit(event);
        }
    }

    #[test]
    fn events_round_trip_through_json() {
        for event in sample_events() {
            let json = serde_json::to_string(&event).unwrap();
            let back: EngineEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(back, event, "round-trip mismatch for {json}");
        }

        let json = serde_json::to_string(&EngineEvent::StepFinished {
            id: "eet/0".to_string(),
            outcome: StepOutcome::Succeeded,
        })
        .unwrap();
        assert!(json.contains(r#""type":"step_finished""#), "{json}");
        assert!(json.contains(r#""outcome":"succeeded""#), "{json}");
    }

    /// Forces the call through the trait so wrapper impls (not autoderef) are exercised.
    fn emit_via_trait<S: EventSink>(sink: &S, event: EngineEvent) {
        sink.emit(event);
    }

    #[test]
    fn box_and_arc_sinks_forward() {
        let (sink, rx) = ChannelSink::unbounded();
        let tx = sink.0.clone();

        let boxed: Box<dyn EventSink> = Box::new(ChannelSink(tx.clone()));
        emit_via_trait(
            &boxed,
            EngineEvent::PhaseStarted {
                name: "boxed".to_string(),
            },
        );

        let shared = Arc::new(ChannelSink(tx));
        let clone = Arc::clone(&shared);
        emit_via_trait(
            &clone,
            EngineEvent::PhaseStarted {
                name: "arced".to_string(),
            },
        );

        assert_eq!(
            rx.recv().unwrap(),
            EngineEvent::PhaseStarted {
                name: "boxed".to_string()
            }
        );
        assert_eq!(
            rx.recv().unwrap(),
            EngineEvent::PhaseStarted {
                name: "arced".to_string()
            }
        );
    }
}
