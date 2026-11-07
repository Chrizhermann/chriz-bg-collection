//! Questions the installer asks the player, and their answers.
//!
//! Usable mod downloads from an official link continue by default.
//! An unavailable mod and its real dependents are left out and recorded. These question
//! kinds remain readable for old campaigns and explicit command-line answers.
//!
//! A source that looks modded asks once before copying, as the shipped installer decided.
//! Failed components still ask Try again / Skip / Stop, with Skip unavailable for an
//! incomplete core. Changed installer tools still require a player's decision.
//! Questions and answers are recorded in the campaign ledger before being applied.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::digest::sha256_bytes;
use crate::orchestrator::ArtifactKind;
use crate::resolve::PlannedRun;
use crate::session::{FrozenIdentity, LedgerRecord, SessionEvent};

/// Which question the installer asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    /// A newer or different download passed every check: install it anyway?
    ChangedDownload,
    /// A mod's download cannot be installed: continue without it and its dependents?
    UnusableDownload,
    /// A changed installer tool came from a host the recipe did not review: use it anyway?
    UnreviewedTool,
    /// A source game looks modded already: install on top of it anyway?
    ModdedSource,
    /// A component's installer stopped with an error: continue without it?
    FailedComponent,
}

impl QuestionKind {
    /// Continue through content differences and unavailable mods. A failed component
    /// and changed installer tools still require a player's answer, as do modded sources.
    pub(crate) const fn installer_default(self, artifact: ArtifactKind) -> Option<Answer> {
        if matches!(artifact, ArtifactKind::Tool) {
            return None;
        }
        match self {
            Self::ChangedDownload | Self::UnusableDownload => Some(Answer::Yes),
            Self::ModdedSource | Self::UnreviewedTool | Self::FailedComponent => None,
        }
    }

    /// Every kind, in a stable order.
    pub const ALL: [Self; 5] = [
        Self::ChangedDownload,
        Self::UnusableDownload,
        Self::UnreviewedTool,
        Self::ModdedSource,
        Self::FailedComponent,
    ];

    /// Stable ledger and JSON spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ChangedDownload => "changed_download",
            Self::UnusableDownload => "unusable_download",
            Self::UnreviewedTool => "unreviewed_tool",
            Self::ModdedSource => "modded_source",
            Self::FailedComponent => "failed_component",
        }
    }

    /// Spelling of the `--answer <kind>=<answer>` command-line flag.
    pub const fn flag_name(self) -> &'static str {
        match self {
            Self::ChangedDownload => "changed-download",
            Self::UnusableDownload => "unusable-download",
            Self::UnreviewedTool => "unreviewed-tool",
            Self::ModdedSource => "modded-source",
            Self::FailedComponent => "failed-component",
        }
    }
}

/// The player's answer to a question.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Answer {
    /// Install the changed download, or continue without the unusable mod.
    Yes,
    /// Stop the installation; a resume asks the link again.
    No,
    /// Leave an optional mod out and continue, with everything that needs it. Offered for a
    /// changed download only when the recipe can do without the mod, and for a failed component
    /// only when the recipe can do without that component.
    LeaveOut,
    /// Run the failed component's installer again. Offered only for a failed component, whose
    /// evidence proves that the failed attempt left nothing of it behind.
    Retry,
}

impl Answer {
    /// Spelling on the command line: `yes`, `no`, `leave-out` or `retry`.
    pub const fn flag_value(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::LeaveOut => "leave-out",
            Self::Retry => "retry",
        }
    }

    /// Parses the command-line spelling.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "yes" => Ok(Self::Yes),
            "no" => Ok(Self::No),
            "leave-out" => Ok(Self::LeaveOut),
            "retry" => Ok(Self::Retry),
            other => Err(format!(
                "unknown answer {other:?}; use yes, no, leave-out or retry"
            )),
        }
    }
}

/// Who gave an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerSource {
    /// The player, through `decide` or the app.
    Player,
    /// An answer preset for the run, by `--answer` or an installer default.
    Preset,
}

/// What an official link served, for a changed download or an unreviewed tool.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServedDownload {
    /// File name the server gave the download, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Byte length.
    pub length: u64,
    /// Lowercase SHA-256.
    pub sha256: String,
    /// The recipe's link.
    pub link: String,
    /// The address that answered with the file.
    pub final_url: String,
    /// Every host of every segment of the download, lowercase, in the order first reached.
    pub hosts: Vec<String>,
    /// Those of [`Self::hosts`] that the recipe did not review.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unreviewed_hosts: Vec<String>,
}

/// Why a mod's download cannot be installed, by kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UnusableKind {
    /// The link answered a status that means it has no file, on every attempt.
    HttpStatus {
        /// HTTP status, 404 or 410.
        status: u16,
        /// Attempts that got it.
        attempts: u32,
    },
    /// The link announced more bytes than a changed download may have.
    TooLarge {
        /// Announced or received length.
        announced: u64,
        /// Largest length the installer accepts.
        limit: u64,
    },
    /// The link served something that is not the archive, such as an error page.
    NotArchive {
        /// What the file looked like.
        problem: String,
        /// Attempts that got it.
        attempts: u32,
    },
    /// The link redirected to plain HTTP.
    Downgrade,
    /// The changed archive's data is damaged.
    Damaged {
        /// Damaged archive entry, when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entry: Option<String>,
        /// Downloads in a row that were damaged.
        attempts: u32,
    },
    /// The changed archive is packed so that it cannot be installed.
    Layout {
        /// What is wrong with it.
        message: String,
    },
    /// Files of the changed archive would replace or meet files of the game or another mod.
    Collision {
        /// The file in the way.
        path: String,
        /// What is in the way.
        message: String,
    },
    /// Another answer of the link that the installer cannot use.
    Other {
        /// What happened.
        message: String,
    },
}

/// Why a mod's download cannot be installed: the kind, and the full sentence the ledger and
/// receipt keep as before.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnusableReason {
    /// Structured kind, from which the player's short text is derived.
    pub kind: UnusableKind,
    /// The complete sentence for details, the ledger and the receipt.
    pub detail: String,
}

impl UnusableReason {
    /// A reason of `kind` with its full sentence.
    pub fn new(kind: UnusableKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    /// The player's short text, which completes "<mod> can't be installed (…)".
    pub fn short(&self) -> &'static str {
        match self.kind {
            UnusableKind::HttpStatus { .. } => "its link no longer has the file",
            UnusableKind::TooLarge { .. } => "its link now serves a far larger download",
            UnusableKind::NotArchive { .. } => "its link no longer serves the archive",
            UnusableKind::Downgrade => "its link now leads to an insecure address",
            UnusableKind::Damaged { .. } => "the newer download is damaged",
            UnusableKind::Layout { .. } => {
                "the newer download is packed in a way the installer can't read"
            }
            UnusableKind::Collision { .. } => {
                "the newer download would replace files of another mod"
            }
            UnusableKind::Other { .. } => "its download can't be used",
        }
    }
}

/// What leaving a payload out changes in the plan being installed, computed when the question
/// is asked (with the closure of everything that needs it) and applied as recorded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaveOutClosure {
    /// Runs no longer installed, in plan order: the mod's own and those of mods that need it.
    pub removed_runs: Vec<String>,
    /// Runs still installed in another form, as they would be installed.
    pub changed_runs: Vec<PlannedRun>,
    /// Digest of the plan that would remain.
    pub plan_sha256: String,
}

/// What failed at an install step, for a [`QuestionKind::FailedComponent`] question.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailedPart {
    /// The planned run whose installer stopped; its step is `install:<run_id>`.
    pub run_id: String,
    /// Its components that WeiDU did not install, in run order. The first one failed.
    pub components: Vec<u32>,
    /// Later components that WeiDU installed all the same: it goes on after a failed component.
    /// Then each of [`Self::components`] failed, and none can be tried again, because that
    /// would install it after these.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub installed_after: Vec<u32>,
    /// Those of [`Self::components`] that the run still installs once the failed one is left
    /// out, in run order, when WeiDU stopped at it. The others need it and leave with it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub continues_with: Vec<u32>,
    /// Why the installation cannot go on without the failed component. Set exactly when the
    /// question offers no leaving out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<String>,
    /// The installer's own error line, when it printed one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The attempt that ran the installer, when a later one asks after a stop: nothing ran then.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_attempt: Option<u32>,
}

impl FailedPart {
    /// Whether the component can be tried again: nothing was installed after it.
    pub fn can_retry(&self) -> bool {
        self.installed_after.is_empty()
    }

    /// The components that leaving out leaves out by name: the one that failed, or each of them
    /// when WeiDU went on after the failure and tried them all.
    pub fn left_out(&self) -> Vec<u32> {
        if self.can_retry() {
            self.components.iter().take(1).copied().collect()
        } else {
            self.components.clone()
        }
    }
}

/// Whether `part` lists components of `whole`, in the same order.
pub fn in_order(whole: &[u32], part: &[u32]) -> bool {
    let mut rest = whole.iter();
    part.iter()
        .all(|component| rest.any(|candidate| candidate == component))
}

/// Whether `continues` lists components of `failed` after its first one, in the same order:
/// what a failed run may still install after leaving out the component that failed.
pub fn continues_after_failure(failed: &[u32], continues: &[u32]) -> bool {
    in_order(failed.get(1..).unwrap_or_default(), continues)
}

/// One question, exactly as the ledger keeps it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    /// Which question.
    pub kind: QuestionKind,
    /// Frozen artifact the question is about.
    pub artifact_id: String,
    /// Whether that artifact is a mod's payload or a tool.
    pub artifact_kind: ArtifactKind,
    /// The frozen reviewed identity of the artifact.
    pub reviewed: FrozenIdentity,
    /// What the link served: for a changed download and an unreviewed tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub served: Option<ServedDownload>,
    /// Why the download cannot be installed: for an unusable download.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<UnusableReason>,
    /// What leaving the mod out would remove: always for an unusable download, and for a
    /// changed download of a mod or a failed component that the recipe can do without.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leave_out: Option<LeaveOutClosure>,
    /// What failed: for a failed component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed: Option<FailedPart>,
}

impl Question {
    /// The token that binds an answer to exactly this question: SHA-256 of its JSON.
    pub fn token(&self) -> String {
        sha256_bytes(&serde_json::to_vec(self).expect("a question always serializes"))
    }

    /// The stable id of this question in one attempt of one step: the first 16 hex digits of
    /// SHA-256 over the step, the attempt, the kind, the artifact, and the served SHA-256 or
    /// the reason's digest. A new attempt gets a new id, so an earlier answer never answers
    /// again.
    pub fn decision_id(&self, step_id: &str, attempt: u32) -> String {
        let subject = match (&self.served, &self.reason) {
            (Some(served), _) => served.sha256.clone(),
            (None, Some(reason)) => {
                sha256_bytes(&serde_json::to_vec(reason).expect("a reason always serializes"))
            }
            (None, None) => String::new(),
        };
        let identity = format!(
            "{step_id}\0{attempt}\0{}\0{}\0{subject}",
            self.kind.as_str(),
            self.artifact_id
        );
        sha256_bytes(identity.as_bytes())[..16].to_owned()
    }

    /// Whether `answer` is one of the answers this question offers.
    pub fn offers(&self, answer: Answer) -> bool {
        match (self.kind, answer) {
            (QuestionKind::FailedComponent, Answer::No) => true,
            (QuestionKind::FailedComponent, Answer::Retry) => {
                self.failed.as_ref().is_some_and(FailedPart::can_retry)
            }
            (QuestionKind::FailedComponent, Answer::Yes) | (_, Answer::Retry) => false,
            (_, Answer::Yes | Answer::No) => true,
            (QuestionKind::UnusableDownload | QuestionKind::ModdedSource, Answer::LeaveOut) => {
                false
            }
            (_, Answer::LeaveOut) => self.leave_out.is_some(),
        }
    }

    /// Whether the answer leaves the mod out.
    pub fn leaves_out(&self, answer: Answer) -> bool {
        match self.kind {
            QuestionKind::UnusableDownload => answer == Answer::Yes,
            QuestionKind::ChangedDownload
            | QuestionKind::UnreviewedTool
            | QuestionKind::FailedComponent => answer == Answer::LeaveOut,
            QuestionKind::ModdedSource => false,
        }
    }

    /// Checks that the parts this kind needs are present and nothing else is.
    pub fn validate(&self) -> Result<(), String> {
        if self.artifact_id != self.reviewed.id {
            return Err("a question names another artifact than its reviewed identity".to_owned());
        }
        let wants_failed_part = self.kind == QuestionKind::FailedComponent;
        let shape_ok = self.failed.is_some() == wants_failed_part
            && match self.kind {
                QuestionKind::ChangedDownload => self.served.is_some() && self.reason.is_none(),
                QuestionKind::UnusableDownload => {
                    self.served.is_none()
                        && self.reason.is_some()
                        && self.leave_out.is_some()
                        && self.artifact_kind == ArtifactKind::Payload
                }
                QuestionKind::UnreviewedTool => {
                    self.served.is_some()
                        && self.reason.is_none()
                        && self.artifact_kind == ArtifactKind::Tool
                }
                // About a source game: only its identity, as `reviewed` names it.
                QuestionKind::ModdedSource => {
                    self.served.is_none()
                        && self.reason.is_none()
                        && self.leave_out.is_none()
                        && is_source_subject(&self.artifact_id)
                }
                // About one mod's installer: the question offers leaving it out, or says why not.
                QuestionKind::FailedComponent => {
                    self.served.is_none()
                        && self.reason.is_none()
                        && self.artifact_kind == ArtifactKind::Payload
                        && self.failed.as_ref().is_some_and(|failed| {
                            !failed.components.is_empty()
                                && continues_after_failure(
                                    &failed.components,
                                    &failed.continues_with,
                                )
                                && failed.required.is_some() != self.leave_out.is_some()
                                && (failed.can_retry()
                                    || (failed.continues_with.is_empty()
                                        && self.leave_out.is_some()))
                        })
                }
            };
        if !shape_ok {
            return Err(format!(
                "a {} question lacks a part it needs or has one it may not have",
                self.kind.as_str()
            ));
        }
        if self.artifact_kind == ArtifactKind::Tool && self.leave_out.is_some() {
            return Err("an installer tool is never left out".to_owned());
        }
        if let Some(served) = &self.served {
            if !is_lower_hex(&served.sha256, 64) {
                return Err("the served SHA-256 must be 64 lowercase hex digits".to_owned());
            }
        }
        if let Some(closure) = &self.leave_out {
            if !is_lower_hex(&closure.plan_sha256, 64) {
                return Err("the remaining plan digest must be 64 lowercase hex digits".to_owned());
            }
        }
        Ok(())
    }
}

/// The subject a [`QuestionKind::ModdedSource`] question names for the source game of `role`:
/// A source-game subject, never an artifact id of the recipe.
pub const fn source_subject(role: crate::games::GameRole) -> &'static str {
    match role {
        crate::games::GameRole::BgeeSod => "source-bg1",
        crate::games::GameRole::Bg2ee => "source-bg2",
        crate::games::GameRole::Iwdee => "source-iwd",
    }
}

fn is_source_subject(value: &str) -> bool {
    matches!(value, "source-bg1" | "source-bg2" | "source-iwd")
}

/// Whether `value` is a decision id: 16 lowercase hexadecimal digits.
pub fn is_decision_id(value: &str) -> bool {
    is_lower_hex(value, 16)
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Preset answers for one run, given as `--answer <kind>=<answer>`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PresetAnswers(BTreeMap<QuestionKind, Answer>);

impl PresetAnswers {
    /// No explicit preset answers; the installer may supply its own defaults.
    pub fn none() -> Self {
        Self::default()
    }

    /// Parses `<kind>=<answer>` values. A kind may be given once; `leave-out` is no answer to
    /// an unusable download, whose `yes` already leaves the mod out. A failed component takes
    /// `leave-out` or `no`: a preset `retry` would run the same failure again without end.
    pub fn parse(values: &[String]) -> Result<Self, String> {
        let mut answers = BTreeMap::new();
        for value in values {
            let (kind, answer) = value
                .split_once('=')
                .ok_or_else(|| format!("use <question>=<answer> for an answer, not {value:?}"))?;
            let kind = QuestionKind::ALL
                .into_iter()
                .find(|candidate| candidate.flag_name() == kind)
                .ok_or_else(|| {
                    format!(
                        "unknown question {kind:?}; use changed-download, unusable-download, unreviewed-tool, modded-source or failed-component"
                    )
                })?;
            let answer = Answer::parse(answer)?;
            if kind == QuestionKind::UnusableDownload && answer == Answer::LeaveOut {
                return Err(
                    "unusable-download is answered with yes (continue without it) or no".to_owned(),
                );
            }
            if kind == QuestionKind::FailedComponent
                && !matches!(answer, Answer::LeaveOut | Answer::No)
            {
                return Err("failed-component is answered with leave-out or no".to_owned());
            }
            if answer == Answer::Retry && kind != QuestionKind::FailedComponent {
                return Err(format!("{} is not answered with retry", kind.flag_name()));
            }
            if answers.insert(kind, answer).is_some() {
                return Err(format!("{} is answered more than once", kind.flag_name()));
            }
        }
        Ok(Self(answers))
    }

    /// The preset answer to questions of `kind`.
    pub fn get(&self, kind: QuestionKind) -> Option<Answer> {
        self.0.get(&kind).copied()
    }

    /// Whether no answer is preset.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A question the ledger holds for the step that is still unresolved, and its answer once the
/// ledger has one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingDecision {
    /// The unresolved step.
    pub step_id: String,
    /// Its attempt.
    pub attempt: u32,
    /// The question's id.
    pub decision_id: String,
    /// The question as recorded.
    pub question: Question,
    /// The recorded answer, its source and when it was recorded.
    pub answer: Option<RecordedAnswer>,
}

/// An answer the ledger recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordedAnswer {
    /// The answer.
    pub answer: Answer,
    /// Who gave it.
    pub source: AnswerSource,
    /// When the ledger recorded it, in Unix milliseconds.
    pub answered_at: u64,
}

/// The question that the ledger holds for its unresolved step and that was not withdrawn, with
/// its answer when one is recorded. `None` when no step is unresolved or it has no question.
pub fn pending_decision(records: &[LedgerRecord]) -> Option<PendingDecision> {
    let mut unresolved: Option<(String, u32)> = None;
    let mut pending: Option<PendingDecision> = None;
    for record in records {
        match &record.event {
            SessionEvent::StepStarted { step_id, attempt } => {
                unresolved = Some((step_id.clone(), *attempt));
                pending = None;
            }
            SessionEvent::StepCompleted { .. }
            | SessionEvent::StepFailed { .. }
            | SessionEvent::FreshCopyRequired { .. }
            | SessionEvent::ArtifactLeftOut { .. } => {
                unresolved = None;
                pending = None;
            }
            SessionEvent::DecisionRequired {
                step_id,
                attempt,
                decision_id,
                question,
            } => {
                pending = Some(PendingDecision {
                    step_id: step_id.clone(),
                    attempt: *attempt,
                    decision_id: decision_id.clone(),
                    question: question.as_ref().clone(),
                    answer: None,
                });
            }
            SessionEvent::DecisionAnswered {
                decision_id,
                answer,
                source,
                ..
            } => {
                if let Some(pending) = pending
                    .as_mut()
                    .filter(|pending| &pending.decision_id == decision_id)
                {
                    pending.answer = Some(RecordedAnswer {
                        answer: *answer,
                        source: *source,
                        answered_at: record.recorded_at,
                    });
                }
            }
            SessionEvent::DecisionWithdrawn { decision_id, .. } => {
                if pending
                    .as_ref()
                    .is_some_and(|pending| &pending.decision_id == decision_id)
                {
                    pending = None;
                }
            }
            SessionEvent::Created(_) => {}
        }
    }
    pending
        .filter(|pending| unresolved.as_ref() == Some(&(pending.step_id.clone(), pending.attempt)))
}

/// What became of an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionOutcome {
    /// The answer took effect: its step stopped after "No", ended its attempt to try again,
    /// completed with exactly the file asked about, or left the mod out with the recorded
    /// closure.
    Applied,
    /// The question was withdrawn after the answer, because the situation it asked about no
    /// longer held; the answer had no effect.
    Withdrawn,
    /// Applying the answer did not succeed, for example because the accepted file was gone and
    /// the link served yet another one; the attempt ended and a later one asked anew.
    Failed,
    /// The step has not ended since the answer.
    Pending,
}

/// Every answer the ledger recorded, in order, with its question and what became of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnsweredDecision {
    /// Step that asked.
    pub step_id: String,
    /// Its attempt.
    pub attempt: u32,
    /// The question's id.
    pub decision_id: String,
    /// The question as recorded.
    pub question: Question,
    /// The answer.
    pub answer: RecordedAnswer,
    /// What became of the answer, by the ledger records that follow it.
    pub outcome: DecisionOutcome,
}

/// Every answered question in `records`, in ledger order, with what became of each answer.
pub fn answered_decisions(records: &[LedgerRecord]) -> Vec<AnsweredDecision> {
    let mut questions = BTreeMap::<&str, (&str, u32, &Question)>::new();
    let mut answered = Vec::<AnsweredDecision>::new();
    // The answer whose step has not ended yet.
    let mut open: Option<usize> = None;
    for record in records {
        let ended = |decision: &AnsweredDecision, step_id: &str, attempt: u32| {
            decision.step_id == step_id && decision.attempt == attempt
        };
        match &record.event {
            SessionEvent::DecisionRequired {
                step_id,
                attempt,
                decision_id,
                question,
            } => {
                questions.insert(
                    decision_id.as_str(),
                    (step_id.as_str(), *attempt, question.as_ref()),
                );
            }
            SessionEvent::DecisionAnswered {
                decision_id,
                answer,
                source,
                ..
            } => {
                if let Some((step_id, attempt, question)) = questions.get(decision_id.as_str()) {
                    answered.push(AnsweredDecision {
                        step_id: (*step_id).to_owned(),
                        attempt: *attempt,
                        decision_id: decision_id.clone(),
                        question: (*question).clone(),
                        answer: RecordedAnswer {
                            answer: *answer,
                            source: *source,
                            answered_at: record.recorded_at,
                        },
                        outcome: DecisionOutcome::Pending,
                    });
                    open = Some(answered.len() - 1);
                }
            }
            SessionEvent::DecisionWithdrawn { decision_id, .. } => {
                if let Some(index) =
                    open.filter(|index| answered[*index].decision_id == *decision_id)
                {
                    answered[index].outcome = DecisionOutcome::Withdrawn;
                    open = None;
                }
            }
            SessionEvent::StepFailed {
                step_id, attempt, ..
            } => {
                if let Some(index) =
                    open.filter(|index| ended(&answered[*index], step_id, *attempt))
                {
                    answered[index].outcome =
                        if matches!(answered[index].answer.answer, Answer::No | Answer::Retry) {
                            DecisionOutcome::Applied
                        } else {
                            DecisionOutcome::Failed
                        };
                    open = None;
                }
            }
            SessionEvent::StepCompleted { step_id, attempt }
            | SessionEvent::ArtifactLeftOut {
                step_id, attempt, ..
            } => {
                if let Some(index) =
                    open.filter(|index| ended(&answered[*index], step_id, *attempt))
                {
                    answered[index].outcome = DecisionOutcome::Applied;
                    open = None;
                }
            }
            SessionEvent::FreshCopyRequired {
                step_id, attempt, ..
            } => {
                if let Some(index) =
                    open.filter(|index| ended(&answered[*index], step_id, *attempt))
                {
                    answered[index].outcome = DecisionOutcome::Failed;
                    open = None;
                }
            }
            SessionEvent::StepStarted { .. } | SessionEvent::Created(_) => {}
        }
    }
    answered
}
#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> FrozenIdentity {
        FrozenIdentity {
            id: "snowytoes".to_owned(),
            version: "1.6".to_owned(),
            sha256: "11".repeat(32),
            length: 10,
        }
    }

    fn changed(sha: &str) -> Question {
        Question {
            kind: QuestionKind::ChangedDownload,
            artifact_id: "snowytoes".to_owned(),
            artifact_kind: ArtifactKind::Payload,
            reviewed: identity(),
            served: Some(ServedDownload {
                filename: Some("snowytoes.zip".to_owned()),
                length: 12,
                sha256: sha.repeat(32),
                link: "https://example.invalid/snowytoes.zip".to_owned(),
                final_url: "https://example.invalid/snowytoes.zip".to_owned(),
                hosts: vec!["example.invalid".to_owned()],
                unreviewed_hosts: Vec::new(),
            }),
            reason: None,
            leave_out: None,
            failed: None,
        }
    }

    fn failed_component(leave_out: bool) -> Question {
        Question {
            kind: QuestionKind::FailedComponent,
            served: None,
            leave_out: leave_out.then(|| LeaveOutClosure {
                removed_runs: Vec::new(),
                changed_runs: Vec::new(),
                plan_sha256: "44".repeat(32),
            }),
            failed: Some(FailedPart {
                run_id: "snowytoes-bg2".to_owned(),
                components: vec![10, 20, 30],
                installed_after: Vec::new(),
                continues_with: if leave_out { vec![30] } else { Vec::new() },
                required: (!leave_out).then(|| "a fixed part".to_owned()),
                error: None,
                evidence_attempt: None,
            }),
            ..changed("22")
        }
    }

    #[test]
    fn a_decision_id_is_the_same_within_an_attempt_and_new_for_another() {
        let question = changed("22");
        let first = question.decision_id("acquire:snowytoes", 1);
        assert!(is_decision_id(&first), "{first}");
        assert_eq!(first, question.decision_id("acquire:snowytoes", 1));
        assert_ne!(first, question.decision_id("acquire:snowytoes", 2));
        assert_ne!(first, question.decision_id("materialize:snowytoes:bg2", 1));
        assert_ne!(first, changed("33").decision_id("acquire:snowytoes", 1));
        let unusable = |message: &str| Question {
            kind: QuestionKind::UnusableDownload,
            served: None,
            reason: Some(UnusableReason::new(
                UnusableKind::Layout {
                    message: message.to_owned(),
                },
                message,
            )),
            leave_out: Some(LeaveOutClosure {
                removed_runs: vec!["snowytoes-bg2".to_owned()],
                changed_runs: Vec::new(),
                plan_sha256: "44".repeat(32),
            }),
            ..changed("22")
        };
        assert_eq!(
            unusable("no installer").decision_id("acquire:snowytoes", 1),
            unusable("no installer").decision_id("acquire:snowytoes", 1)
        );
        assert_ne!(
            unusable("no installer").decision_id("acquire:snowytoes", 1),
            unusable("another problem").decision_id("acquire:snowytoes", 1)
        );
    }

    #[test]
    fn a_token_binds_the_exact_question() {
        let question = changed("22");
        assert_eq!(question.token(), changed("22").token());
        assert_ne!(question.token(), changed("33").token());
        let mut renamed = changed("22");
        renamed.served.as_mut().unwrap().filename = Some("other.zip".to_owned());
        assert_ne!(question.token(), renamed.token());
    }

    #[test]
    fn leave_out_is_offered_only_for_a_mod_the_recipe_can_do_without() {
        let mut question = changed("22");
        assert!(question.offers(Answer::Yes));
        assert!(question.offers(Answer::No));
        assert!(!question.offers(Answer::LeaveOut));
        question.leave_out = Some(LeaveOutClosure {
            removed_runs: vec!["snowytoes-bg2".to_owned()],
            changed_runs: Vec::new(),
            plan_sha256: "44".repeat(32),
        });
        assert!(question.offers(Answer::LeaveOut));
        assert!(question.leaves_out(Answer::LeaveOut));
        assert!(!question.leaves_out(Answer::Yes));
        question.validate().unwrap();
        question.artifact_kind = ArtifactKind::Tool;
        assert!(question.validate().is_err());
    }

    #[test]
    fn a_failed_component_offers_leaving_out_only_when_the_recipe_can_do_without_it() {
        let optional = failed_component(true);
        optional.validate().unwrap();
        assert!(optional.offers(Answer::LeaveOut));
        assert!(optional.offers(Answer::Retry));
        assert!(optional.offers(Answer::No));
        assert!(!optional.offers(Answer::Yes));
        assert!(optional.leaves_out(Answer::LeaveOut));
        assert!(!optional.leaves_out(Answer::Retry));

        let fixed = failed_component(false);
        fixed.validate().unwrap();
        assert!(!fixed.offers(Answer::LeaveOut));
        assert!(fixed.offers(Answer::Retry));
        assert!(fixed.offers(Answer::No));

        // Only this question takes "try again".
        assert!(!changed("22").offers(Answer::Retry));
    }

    /// WeiDU goes on after a failed component. What it installed after it is in the game, so
    /// trying again would install the failed one out of order: only leaving out and stopping
    /// are left, and everything it reported as failed leaves by name.
    #[test]
    fn a_component_failed_before_others_were_installed_cannot_be_tried_again() {
        let mut question = failed_component(true);
        let failed = question.failed.as_mut().unwrap();
        failed.components = vec![10, 30];
        failed.installed_after = vec![20];
        failed.continues_with = Vec::new();
        question.validate().unwrap();
        assert!(question.offers(Answer::LeaveOut));
        assert!(question.offers(Answer::No));
        assert!(!question.offers(Answer::Retry));
        assert_eq!(question.failed.as_ref().unwrap().left_out(), vec![10, 30]);

        // With nothing to offer but stopping it would be no question.
        let mut fixed = failed_component(false);
        fixed.failed.as_mut().unwrap().installed_after = vec![20];
        assert!(fixed.validate().is_err());

        // And nothing goes on after WeiDU tried everything.
        let mut goes_on = question.clone();
        goes_on.failed.as_mut().unwrap().continues_with = vec![30];
        assert!(goes_on.validate().is_err());

        let stopped_at_it = failed_component(true);
        assert_eq!(stopped_at_it.failed.as_ref().unwrap().left_out(), vec![10]);
    }

    #[test]
    fn a_failed_component_question_must_describe_what_failed() {
        let mut lost = failed_component(true);
        lost.failed = None;
        assert!(lost.validate().is_err());

        let mut foreign = changed("22");
        foreign.failed = failed_component(true).failed;
        assert!(foreign.validate().is_err());

        let mut contradictory = failed_component(true);
        contradictory.failed.as_mut().unwrap().required = Some("a fixed part".to_owned());
        assert!(contradictory.validate().is_err());

        let mut unplanned = failed_component(true);
        unplanned.failed.as_mut().unwrap().continues_with = vec![10];
        assert!(unplanned.validate().is_err());
        unplanned.failed.as_mut().unwrap().continues_with = vec![30, 20];
        assert!(unplanned.validate().is_err());

        let mut empty = failed_component(true);
        empty.failed.as_mut().unwrap().components.clear();
        empty.failed.as_mut().unwrap().continues_with.clear();
        assert!(empty.validate().is_err());
    }

    #[test]
    fn preset_answers_parse_each_question_once() {
        let answers = PresetAnswers::parse(&[
            "changed-download=leave-out".to_owned(),
            "unusable-download=yes".to_owned(),
        ])
        .unwrap();
        assert_eq!(
            answers.get(QuestionKind::ChangedDownload),
            Some(Answer::LeaveOut)
        );
        assert_eq!(
            answers.get(QuestionKind::UnusableDownload),
            Some(Answer::Yes)
        );
        assert_eq!(answers.get(QuestionKind::UnreviewedTool), None);
        for answer in ["leave-out", "no"] {
            let answers = PresetAnswers::parse(&[format!("failed-component={answer}")]).unwrap();
            assert_eq!(
                answers.get(QuestionKind::FailedComponent),
                Some(Answer::parse(answer).unwrap())
            );
        }
        for invalid in [
            vec!["changed-download".to_owned()],
            vec!["changed=yes".to_owned()],
            vec!["changed-download=maybe".to_owned()],
            vec!["unusable-download=leave-out".to_owned()],
            vec!["failed-component=yes".to_owned()],
            // A preset "try again" would repeat the same failure without end.
            vec!["failed-component=retry".to_owned()],
            vec!["changed-download=retry".to_owned()],
            vec![
                "changed-download=yes".to_owned(),
                "changed-download=no".to_owned(),
            ],
        ] {
            assert!(PresetAnswers::parse(&invalid).is_err(), "{invalid:?}");
        }
    }
}

#[cfg(test)]
mod overwrite_defaults_tests {
    use super::*;

    #[test]
    fn downloads_and_missing_mods_continue_but_sources_failures_and_tools_ask() {
        for kind in [
            QuestionKind::ChangedDownload,
            QuestionKind::UnusableDownload,
        ] {
            assert_eq!(
                kind.installer_default(ArtifactKind::Payload),
                Some(Answer::Yes)
            );
            assert_eq!(kind.installer_default(ArtifactKind::Tool), None);
        }
        for kind in [
            QuestionKind::ModdedSource,
            QuestionKind::FailedComponent,
            QuestionKind::UnreviewedTool,
        ] {
            assert_eq!(kind.installer_default(ArtifactKind::Payload), None);
        }
    }
}
