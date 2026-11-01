//! Questions the installer asks the player about a download, and their answers.
//!
//! Three situations stop an installation with a question instead of deciding silently:
//!
//! - **changed download**: the official link of a mod or tool serves a file other than the
//!   reviewed one, and that file passed every check.
//! - **unusable download**: a mod's download cannot be installed, so the mod and everything that
//!   needs it would be left out.
//! - **unreviewed tool**: a changed installer tool came, at least in part, from a host the
//!   recipe did not review, and the download folder holds no reviewed copy to use instead.
//!
//! The campaign worker records the question in the ledger ([`crate::session::SessionEvent`])
//! and stops. An answer reaches it as a create-once answer file, written under the target lock
//! by `decide`, or as a preset answer given for one run; the worker records the answer in the
//! ledger before it applies it.

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
}

impl QuestionKind {
    /// Every kind, in a stable order.
    pub const ALL: [Self; 3] = [
        Self::ChangedDownload,
        Self::UnusableDownload,
        Self::UnreviewedTool,
    ];

    /// Stable ledger and JSON spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ChangedDownload => "changed_download",
            Self::UnusableDownload => "unusable_download",
            Self::UnreviewedTool => "unreviewed_tool",
        }
    }

    /// Spelling of the `--answer <kind>=<answer>` command-line flag.
    pub const fn flag_name(self) -> &'static str {
        match self {
            Self::ChangedDownload => "changed-download",
            Self::UnusableDownload => "unusable-download",
            Self::UnreviewedTool => "unreviewed-tool",
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
    /// changed download only when the recipe can do without the mod.
    LeaveOut,
}

impl Answer {
    /// Spelling on the command line: `yes`, `no` or `leave-out`.
    pub const fn flag_value(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::LeaveOut => "leave-out",
        }
    }

    /// Parses the command-line spelling.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "yes" => Ok(Self::Yes),
            "no" => Ok(Self::No),
            "leave-out" => Ok(Self::LeaveOut),
            other => Err(format!(
                "unknown answer {other:?}; use yes, no or leave-out"
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
    /// A preset answer given for one run (`--answer`), for tests and automation.
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
    /// changed download of a mod that the recipe can do without.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leave_out: Option<LeaveOutClosure>,
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
            (_, Answer::Yes | Answer::No) => true,
            (QuestionKind::UnusableDownload, Answer::LeaveOut) => false,
            (_, Answer::LeaveOut) => self.leave_out.is_some(),
        }
    }

    /// Whether the answer leaves the mod out.
    pub fn leaves_out(&self, answer: Answer) -> bool {
        match self.kind {
            QuestionKind::UnusableDownload => answer == Answer::Yes,
            QuestionKind::ChangedDownload | QuestionKind::UnreviewedTool => {
                answer == Answer::LeaveOut
            }
        }
    }

    /// Checks that the parts this kind needs are present and nothing else is.
    pub fn validate(&self) -> Result<(), String> {
        if self.artifact_id != self.reviewed.id {
            return Err("a question names another artifact than its reviewed identity".to_owned());
        }
        let shape_ok = match self.kind {
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
    /// No preset answers: every question stops the installation.
    pub fn none() -> Self {
        Self::default()
    }

    /// Parses `<kind>=<answer>` values. A kind may be given once; `leave-out` is no answer to
    /// an unusable download, whose `yes` already leaves the mod out.
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
                        "unknown question {kind:?}; use changed-download, unusable-download or unreviewed-tool"
                    )
                })?;
            let answer = Answer::parse(answer)?;
            if kind == QuestionKind::UnusableDownload && answer == Answer::LeaveOut {
                return Err(
                    "unusable-download is answered with yes (continue without it) or no".to_owned(),
                );
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

/// Every answer the ledger recorded, in order, with its question and whether it was withdrawn
/// later.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnsweredDecision {
    /// Step that asked.
    pub step_id: String,
    /// The question's id.
    pub decision_id: String,
    /// The question as recorded.
    pub question: Question,
    /// The answer.
    pub answer: RecordedAnswer,
    /// Whether the question was withdrawn after it was answered, because the situation it
    /// answered no longer held.
    pub withdrawn: bool,
}

/// Every answered question in `records`, in ledger order.
pub fn answered_decisions(records: &[LedgerRecord]) -> Vec<AnsweredDecision> {
    let mut questions = BTreeMap::<&str, (&str, &Question)>::new();
    let mut answered = Vec::<AnsweredDecision>::new();
    for record in records {
        match &record.event {
            SessionEvent::DecisionRequired {
                step_id,
                decision_id,
                question,
                ..
            } => {
                questions.insert(decision_id.as_str(), (step_id.as_str(), question.as_ref()));
            }
            SessionEvent::DecisionAnswered {
                decision_id,
                answer,
                source,
                ..
            } => {
                if let Some((step_id, question)) = questions.get(decision_id.as_str()) {
                    answered.push(AnsweredDecision {
                        step_id: (*step_id).to_owned(),
                        decision_id: decision_id.clone(),
                        question: (*question).clone(),
                        answer: RecordedAnswer {
                            answer: *answer,
                            source: *source,
                            answered_at: record.recorded_at,
                        },
                        withdrawn: false,
                    });
                }
            }
            SessionEvent::DecisionWithdrawn { decision_id, .. } => {
                if let Some(decision) = answered
                    .iter_mut()
                    .rev()
                    .find(|decision| &decision.decision_id == decision_id)
                {
                    decision.withdrawn = true;
                }
            }
            _ => {}
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
        for invalid in [
            vec!["changed-download".to_owned()],
            vec!["changed=yes".to_owned()],
            vec!["changed-download=maybe".to_owned()],
            vec!["unusable-download=leave-out".to_owned()],
            vec![
                "changed-download=yes".to_owned(),
                "changed-download=no".to_owned(),
            ],
        ] {
            assert!(PresetAnswers::parse(&invalid).is_err(), "{invalid:?}");
        }
    }
}
