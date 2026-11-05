//! Resumable, fail-closed orchestration for one isolated two-root EET campaign.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::decision::{
    answered_decisions, Answer, AnswerSource, AnsweredDecision, FailedPart, LeaveOutClosure,
    PendingDecision, Question, QuestionKind, RecordedAnswer, ServedDownload, UnusableReason,
};
use crate::digest::{plan_digest, sha256_bytes};
use crate::error::EngineError;
use crate::events::{EngineEvent, EventSink, StepOutcome};
use crate::games::GameRole;
use crate::lock::{LockError, TargetLock};
use crate::manifest::{GameRoot, Phase};
use crate::postcondition;
use crate::preflight::is_creator_protected_destination;
use crate::resolve::{InstallPlan, PlannedRun};
use crate::session::{CampaignCreated, FrozenIdentity, SessionEvent, SessionReplay, SessionStore};
use crate::weidu::runner::RunnerControlHandle;

/// Frozen inputs and lock location for one new build or exact resume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignRequest {
    /// App-data root containing target lock files, outside the managed installation.
    pub registry_root: PathBuf,
    /// Exact immutable campaign identity expected on every resume.
    pub created: CampaignCreated,
    /// Exact resolved plan whose digest is frozen in [`Self::created`].
    pub plan: InstallPlan,
}

/// Whether a frozen object is a mod payload or an executable tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    /// Installer payload that may be materialized into one or both staged roots.
    Payload,
    /// Executable helper, such as the pinned WeiDU binary.
    Tool,
}

/// One unique payload publication into one staged root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaterializationTask {
    /// Frozen payload identity.
    pub artifact_id: String,
    /// Staged root receiving the payload.
    pub target: GameRoot,
}

impl MaterializationTask {
    /// Stable ledger step id used for publication and crash reconciliation.
    pub fn step_id(&self) -> String {
        format!(
            "materialize:{}:{}",
            self.artifact_id,
            game_root_id(self.target)
        )
    }
}

/// What acquiring one frozen artifact found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcquisitionOutcome {
    /// The artifact is in the cache and can be used.
    Ready,
    /// The payload's current download cannot be used, for example because its archive layout
    /// changed beyond the safe fallback. When the recipe can do without this mod and without
    /// everything that depends on it, the player is asked whether to continue without them.
    LeaveOut {
        /// Why, by kind, with the display-safe sentence recorded in the ledger and the receipt.
        reason: UnusableReason,
    },
    /// The link serves a newer or different file that passed every check, or a changed tool
    /// came from a host the recipe did not review. Nothing about it is recorded yet: the
    /// player is asked first ([`QuestionKind::ChangedDownload`] or
    /// [`QuestionKind::UnreviewedTool`]).
    Ask {
        /// Which question.
        kind: QuestionKind,
        /// What the link served.
        served: ServedDownload,
    },
}

/// What applying the answer "install it" to a changed download found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnsweredAcquisition {
    /// Exactly the file that was asked about is recorded and can be used.
    Accepted,
    /// The situation the question asked about no longer holds, for example because the
    /// reviewed file is now in the download folder. The step acquires its artifact anew.
    Withdrawn {
        /// What changed.
        why: String,
    },
    /// The file asked about is gone and the link serves yet another one. This attempt ends;
    /// the next one asks anew.
    LinkChanged {
        /// Display-safe detail for the ledger.
        detail: String,
    },
}

/// A payload whose download could not be used, so the campaign continued without it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeftOutArtifact {
    /// Frozen payload identity that is left out.
    pub artifact_id: String,
    /// Why its download could not be used.
    pub reason: String,
    /// Planned runs no longer installed because of it, including runs of mods that need it.
    pub runs: Vec<String>,
    /// Planned runs still installed, but with fewer components or other prompt answers.
    pub changed_runs: Vec<String>,
    /// The run whose installer failed, when the mod was left out at its install step rather
    /// than because of its download.
    pub failed_run: Option<String>,
}

/// A materialization either reconciled to completion or proved this copy unsafe to resume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterializationOutcome {
    /// The frozen publication manifest and every published byte are exact.
    Complete,
    /// Unknown, missing, truncated, or contradictory target bytes require a new copy.
    FreshCopyRequired {
        /// Evidence summary suitable for diagnostics and the UI.
        reason: String,
    },
    /// The payload's current download cannot be placed, for example because a file of its
    /// changed archive would overwrite another one, and no attempt of this step has written
    /// anything. When the recipe can do without this mod and without everything that depends
    /// on it, and none of their files are placed yet, the player is asked whether to continue
    /// without them; otherwise this copy needs a fresh start, as for
    /// [`MaterializationOutcome::FreshCopyRequired`].
    LeaveOut {
        /// Why, by kind, with the display-safe sentence recorded in the ledger and the receipt.
        reason: UnusableReason,
    },
}

/// A durable attempt location. Callers must create evidence files only inside this root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepAttempt {
    /// Stable logical pipeline step id.
    pub step_id: String,
    /// One-based attempt number for this step.
    pub attempt: u32,
    /// Windows-safe, deterministic evidence root below the frozen campaign attempt.
    pub evidence_root: PathBuf,
}

/// Invocation plus its semantic identity, which is persisted before a child can spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltInvocation<T> {
    /// Backend-specific, fully prepared process description.
    pub invocation: T,
    /// Digest of every consequential invocation field.
    pub identity_digest: String,
}

/// Process evidence needed by the post-run log reconciler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessResult {
    /// Child exit code; log evidence, not this value alone, decides success.
    pub exit_code: i32,
}

/// Evidence verdict for an interrupted or newly finished WeiDU attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallReconciliation {
    /// Exact expected tail plus complete debug success evidence is durable.
    ProvenDone,
    /// The log is unchanged and rollback/failure evidence proves this exact suffix retryable.
    Retry {
        /// Exact component suffix that may be invoked again.
        remaining: Vec<u32>,
    },
    /// A strict successful prefix committed; only this exact suffix remains.
    PartialPrefix {
        /// Exact remaining component suffix.
        remaining: Vec<u32>,
    },
    /// The installer went on after a failed component, as WeiDU does, and every requested
    /// component was tried. Nothing can run again: it would install out of the planned order.
    FailedAmong {
        /// Requested components not installed, in run order.
        failed: Vec<u32>,
        /// Components installed after the first failure, in run order.
        installed_after: Vec<u32>,
    },
    /// Removal, extra, reorder, or incomplete/contradictory evidence forbids reuse.
    FreshCopyRequired {
        /// Evidence summary suitable for diagnostics and the UI.
        reason: String,
    },
}

impl InstallReconciliation {
    /// Construct a proven unchanged/rolled-back retry verdict.
    pub fn retry(remaining: Vec<u32>) -> Self {
        Self::Retry { remaining }
    }
}

/// Kind of target-affecting operation checked immediately before it mutates or spawns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationKind {
    /// Copy or resume-copy one pristine source game.
    Stage,
    /// Publish one verified payload into a staged root.
    Materialize,
    /// Build a target-local WeiDU invocation; this writes tool/debug files.
    InvocationBuild,
    /// Spawn the prepared child process.
    ProcessSpawn,
    /// Patch and read back the final BG2 save identity.
    FinalIdentity,
}

impl MutationKind {
    /// Stable diagnostic spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stage => "stage",
            Self::Materialize => "materialize",
            Self::InvocationBuild => "invocation-build",
            Self::ProcessSpawn => "process-spawn",
            Self::FinalIdentity => "final-identity",
        }
    }
}

/// Exact safety check requested immediately before a target mutation or process spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationCheck {
    /// Logical step whose work is about to occur.
    pub step_id: String,
    /// Affected staged game root.
    pub target: GameRoot,
    /// Kind of mutation or spawn.
    pub kind: MutationKind,
}

/// Narrow, display-safe backend failure. The orchestrator persists it before stopping.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct StepFailure {
    message: String,
    before_preparation: bool,
}

impl StepFailure {
    /// Create a non-empty failure detail for the campaign ledger.
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            message: if message.trim().is_empty() {
                "campaign operation failed without detail".to_owned()
            } else {
                message
            },
            before_preparation: false,
        }
    }

    /// Marks a failure of an install attempt that happened before it prepared anything for
    /// WeiDU, such as getting the WeiDU executable, or while placing recipe configuration files,
    /// which a retry repeats safely. The orchestrator records the same pre-spawn evidence as for
    /// a failed safety guard, so that the run can be retried.
    pub fn before_preparation(self) -> Self {
        Self {
            before_preparation: true,
            ..self
        }
    }

    fn into_message(self) -> String {
        self.message
    }
}

/// Terminal result serialized into an immutable attempt receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptDraftOutcome {
    /// The complete plan and final verification succeeded.
    Succeeded,
    /// A retryable step failed.
    Failed {
        /// Stable pipeline step id.
        step_id: String,
        /// Durable failure detail.
        detail: String,
    },
    /// Existing evidence made further mutation unsafe.
    FreshCopyRequired {
        /// Stable pipeline step id.
        step_id: String,
        /// Durable reconciliation detail.
        detail: String,
    },
}

/// Immutable terminal receipt hand-off assembled from the durable campaign ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptDraft {
    /// Frozen install identity.
    pub install_id: String,
    /// Frozen campaign-attempt identity.
    pub attempt_id: String,
    /// Durable campaign-attempt start timestamp from ledger record zero.
    pub started_at_millis: u64,
    /// Durable terminal timestamp from the last ledger record before publication.
    pub completed_at_millis: u64,
    /// Successful, failed, or unsafe-to-resume terminal outcome.
    pub outcome: ReceiptDraftOutcome,
    /// Complete frozen campaign identity used to derive receipt digests and source pins.
    pub created: CampaignCreated,
    /// Exact plan whose digest was frozen in [`Self::created`].
    pub plan: InstallPlan,
    /// Payloads left out because their downloads could not be used, in ledger order.
    pub left_out: Vec<LeftOutArtifact>,
    /// The plan actually installed when payloads were left out; `None` means [`Self::plan`].
    pub installed_plan: Option<InstallPlan>,
    /// Every answer to a question about a download, in ledger order, earlier "No" answers
    /// included.
    pub decisions: Vec<AnsweredDecision>,
}

impl ReceiptDraft {
    /// The plan this campaign installs: the frozen plan minus everything left out.
    pub fn effective_plan(&self) -> &InstallPlan {
        self.installed_plan.as_ref().unwrap_or(&self.plan)
    }
}

/// Terminal state of one orchestration call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CampaignOutcome {
    /// Every selected logical step completed and the receipt sink accepted the result.
    Complete,
    /// The current step completed durably and no later pipeline step was started.
    Paused {
        /// Last pipeline step proven complete before orchestration stopped.
        after_step_id: String,
    },
    /// A retryable operation failed; no later step ran.
    Failed {
        /// Logical step that remains pending.
        step_id: String,
        /// Durable failure summary.
        reason: String,
    },
    /// Existing evidence is unsafe or ambiguous; this target must not be mutated again.
    FreshCopyRequired {
        /// Logical step whose evidence could not be reconciled.
        step_id: String,
        /// Durable evidence summary.
        reason: String,
    },
    /// A step asked the player a question and waits for the answer. Nothing is decided yet;
    /// the step stays unresolved, and a resume continues once an answer is there.
    DecisionRequired {
        /// The step that asks.
        step_id: String,
        /// The question's id.
        decision_id: String,
    },
}

/// Structural campaign errors. Backend step failures are returned as [`CampaignOutcome`].
#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// The external target lock could not be safely acquired.
    #[error(transparent)]
    Lock(#[from] LockError),
    /// The append-only session store could not be created, replayed, or updated.
    #[error(transparent)]
    Engine(#[from] EngineError),
    /// Frozen inputs or the durable pipeline history are internally inconsistent.
    #[error("invalid campaign: {0}")]
    InvalidCampaign(String),
    /// A new or resumed target could not be safely claimed as installer-owned.
    #[error("unsafe managed target {path}: {reason}")]
    UnsafeTarget {
        /// Rejected target or ancestor.
        path: PathBuf,
        /// Exact ownership/safety failure.
        reason: String,
    },
    /// A terminal result could not be persisted as an immutable attempt receipt.
    #[error("could not publish terminal receipt: {0}")]
    TerminalReceipt(String),
    /// The durable campaign ledger could not be linked from the application-data index.
    #[error("could not publish campaign start index: {0}")]
    CampaignIndex(String),
}

/// Publishes the minimal restart pointer after ledger record zero is durable and verified.
pub trait CampaignRecorder {
    /// Record this exact campaign identity without replacing an earlier pointer.
    fn record_campaign(&mut self, created: &CampaignCreated) -> Result<(), StepFailure>;
}

/// Performs full initial validation and the last-moment process/TLK rechecks.
pub trait CampaignPreflight {
    /// Validate all frozen campaign inputs before any payload/staging operation.
    fn initial(
        &mut self,
        request: &CampaignRequest,
        replay: &SessionReplay,
    ) -> Result<(), StepFailure>;

    /// Recheck active processes and TLK exclusivity immediately before target work.
    fn recheck_before_mutation(&mut self, check: &MutationCheck) -> Result<(), StepFailure>;
}

/// Acquires one frozen payload/tool into the shared content-addressed cache.
pub trait ArtifactAcquirer {
    /// Acquire and reverify one identity. Implementations rely on the cache's own digest lock.
    fn acquire(
        &mut self,
        identity: &FrozenIdentity,
        kind: ArtifactKind,
    ) -> Result<AcquisitionOutcome, StepFailure>;

    /// The part of `plan` that remains when the `left_out` payloads are not installed,
    /// together with everything that depends on them. Fails when the installation cannot
    /// continue without them, for example because the recipe always installs one of them.
    ///
    /// Called only when a payload is left out. The ledger records which runs that removed or
    /// changed, so a resume rebuilds the remaining plan from that record, even after an app
    /// update that derives remaining plans differently.
    fn plan_without(
        &mut self,
        plan: &InstallPlan,
        left_out: &[String],
    ) -> Result<InstallPlan, StepFailure> {
        let _ = (plan, left_out);
        Err(StepFailure::new(
            "This installer cannot continue without a mod whose download cannot be used.",
        ))
    }

    /// [`Self::plan_without`], leaving out also `failed`: components, by planned run, that an
    /// installer could not install, together with everything that depends on them. What
    /// depends on `in_game`, the components installed so far by planned run, stays. Fails when
    /// the installation cannot continue without them, for example because the recipe always
    /// installs one of them.
    ///
    /// Called only when a run's installer failed. The ledger records the result with the
    /// question that offers it, as for [`Self::plan_without`].
    fn plan_without_parts(
        &mut self,
        plan: &InstallPlan,
        left_out: &[String],
        failed: &[(String, Vec<u32>)],
        in_game: &[(String, Vec<u32>)],
    ) -> Result<InstallPlan, StepFailure> {
        let _ = (plan, left_out, failed, in_game);
        Err(StepFailure::new(
            "This installer cannot continue without a part whose installer failed.",
        ))
    }

    /// The player-facing name of a frozen artifact, for messages.
    fn artifact_name(&self, artifact_id: &str) -> String {
        artifact_id.to_owned()
    }

    /// One sentence that names the question, for the installation log and for the reason of
    /// a "No" answer.
    fn describe_question(&self, question: &Question) -> String {
        let name = self.artifact_name(&question.artifact_id);
        match (&question.kind, &question.reason) {
            (QuestionKind::FailedComponent, _) => {
                format!("a part of {name} could not be installed.")
            }
            (QuestionKind::UnusableDownload, Some(reason)) => {
                format!("{name} can't be installed ({}).", reason.short())
            }
            (QuestionKind::UnreviewedTool, _) => format!(
                "a newer version of {name} was found, from a host the collection did not review."
            ),
            _ => format!("a newer version of {name} was found."),
        }
    }

    /// The answer to `decision`, a question this installation asked and the ledger has not
    /// answered: the player's, when one is waiting, else a preset answer for this run. `None`
    /// keeps the installation waiting for the player. Without an answer service every question
    /// waits.
    fn decision_answer(
        &mut self,
        decision: &PendingDecision,
    ) -> Result<Option<(Answer, AnswerSource)>, StepFailure> {
        let _ = decision;
        Ok(None)
    }

    /// Whether the download folder holds the reviewed archive of `identity` now, looking only
    /// there and never asking its link.
    fn reviewed_is_available(
        &mut self,
        identity: &FrozenIdentity,
        kind: ArtifactKind,
    ) -> Result<bool, StepFailure> {
        let _ = (identity, kind);
        Ok(false)
    }

    /// Applies the answer "install it" to `question`, a changed download of `identity`: records
    /// exactly the file that was asked about, getting it again from its link when the download
    /// folder lost it, but only with the same length and SHA-256.
    fn acquire_answered(
        &mut self,
        identity: &FrozenIdentity,
        kind: ArtifactKind,
        question: &Question,
    ) -> Result<AnsweredAcquisition, StepFailure> {
        let _ = (identity, kind, question);
        Err(StepFailure::new(
            "This installer cannot install a download that differs from the reviewed one.",
        ))
    }
}

/// Stages independent game copies and applies the deterministic save identity boundary.
pub trait StagingService {
    /// Stage or safely resume one source-game copy. BG1 staging also isolates its save identity.
    fn stage(&mut self, role: GameRole) -> Result<(), StepFailure>;

    /// The question to ask before the source game of `role` is copied, when it looks modded
    /// already ([`QuestionKind::ModdedSource`]): installing on top of it may fail later.
    /// `None` copies it without asking.
    fn source_question(&mut self, role: GameRole) -> Result<Option<Question>, StepFailure> {
        let _ = role;
        Ok(None)
    }

    /// Patch and read back final BG2 identity after every EET-end/tail run.
    fn finalize_identity(&mut self) -> Result<(), StepFailure>;
}

/// Publishes one verified payload under an exact frozen publication manifest.
pub trait ArtifactMaterializer {
    /// Reconcile/publish one artifact-target pair using its stable step identity.
    fn materialize(
        &mut self,
        task: &MaterializationTask,
    ) -> Result<MaterializationOutcome, StepFailure>;
}

/// Builds one target-local invocation after its immediate process/TLK recheck.
pub trait InvocationBuilder {
    /// Backend-specific prepared invocation type.
    type Invocation;

    /// Build the exact remaining component suffix and return its semantic digest.
    fn build(
        &mut self,
        run: &PlannedRun,
        components: &[u32],
        attempt: &StepAttempt,
    ) -> Result<BuiltInvocation<Self::Invocation>, StepFailure>;
}

/// Executes one prepared child synchronously; campaign orchestration never overlaps runs.
pub trait ProcessRunner<I> {
    /// Spawn, supervise, and join the child before returning.
    fn run(&mut self, invocation: I, attempt: &StepAttempt) -> Result<ProcessResult, StepFailure>;
}

/// Owns durable before/debug/raw/after evidence and strict WeiDU.log reconciliation.
pub trait InstallLogVerifier {
    /// Explicit frozen manual boundaries; absent in ordinary campaigns.
    fn manual_checkpoints(&self) -> Vec<crate::manual_checkpoint::FrozenCheckpoint> {
        Vec::new()
    }

    /// Persist a durable baseline or verify explicit acceptance. False means remain paused.
    /// `plan` is the plan being installed, which lacks runs of left-out payloads.
    fn manual_checkpoint(
        &mut self,
        _checkpoint: &crate::manual_checkpoint::FrozenCheckpoint,
        _plan: &InstallPlan,
    ) -> Result<bool, StepFailure> {
        Err(StepFailure::new(
            "Manual checkpoints are not supported by this backend.",
        ))
    }
    /// Reconcile an earlier attempt whose logical step did not complete.
    fn recover(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
    ) -> Result<InstallReconciliation, StepFailure>;

    /// Persist and sync the exact before-log snapshot before invocation construction.
    fn snapshot_before(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
    ) -> Result<(), StepFailure>;

    /// Sync evidence proving an immediate safety guard returned before the process runner.
    /// Return its SHA-256 to bind the exact record into the terminal ledger failure.
    fn record_pre_spawn_failure(
        &mut self,
        run: &PlannedRun,
        components: &[u32],
        attempt: &StepAttempt,
        kind: MutationKind,
        failure: &StepFailure,
    ) -> Result<String, StepFailure>;

    /// Persist and sync the prepared invocation digest before spawn intent.
    fn record_invocation(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
        identity_digest: &str,
    ) -> Result<(), StepFailure>;

    /// Sync raw/debug/after evidence and reconcile it before the ledger can advance.
    fn sync_and_reconcile(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
        result: ProcessResult,
    ) -> Result<InstallReconciliation, StepFailure>;

    /// The installer's own error line of a finished attempt that failed, for the question to
    /// the player. Reading it is a courtesy: `None` when there is none or it cannot be read.
    fn failure_message(&mut self, attempt: &StepAttempt) -> Option<String> {
        let _ = attempt;
        None
    }

    /// Whether `attempt` left evidence of its own. An attempt that runs an installer writes its
    /// evidence before anything else; an attempt that only asks writes none.
    fn attempt_left_evidence(&mut self, attempt: &StepAttempt) -> Result<bool, StepFailure> {
        let _ = attempt;
        Ok(true)
    }

    /// Verify the complete final active stack against the plan being installed: the exact
    /// frozen run plan, minus the runs of left-out payloads.
    fn verify_final(&mut self, plan: &InstallPlan) -> Result<(), StepFailure>;
}

/// Accepts every terminal campaign hand-off. Task 14 supplies immutable persistence.
pub trait ReceiptWriter {
    /// Publish terminal evidence without replacing an earlier attempt receipt.
    fn write(&mut self, draft: &ReceiptDraft) -> Result<(), StepFailure>;
}

/// Injected clock used by deterministic receipt/state-machine tests.
pub trait CampaignClock {
    /// Current Unix epoch time in milliseconds.
    fn now_millis(&mut self) -> Result<u64, StepFailure>;
}

/// Complete injected side-effect boundary used by the synchronous state machine.
pub trait CampaignDependencies:
    CampaignRecorder
    + CampaignPreflight
    + ArtifactAcquirer
    + StagingService
    + ArtifactMaterializer
    + InvocationBuilder
    + ProcessRunner<<Self as InvocationBuilder>::Invocation>
    + InstallLogVerifier
    + ReceiptWriter
    + CampaignClock
{
}

impl<T> CampaignDependencies for T where
    T: CampaignRecorder
        + CampaignPreflight
        + ArtifactAcquirer
        + StagingService
        + ArtifactMaterializer
        + InvocationBuilder
        + ProcessRunner<<T as InvocationBuilder>::Invocation>
        + InstallLogVerifier
        + ReceiptWriter
        + CampaignClock
{
}

/// Lock, freeze, validate, execute, reconcile, and receipt one isolated EET campaign.
///
/// The target lock is deliberately retained in this stack frame until the terminal outcome
/// has been produced, including receipt publication. No cache digest lock is taken here:
/// acquisition implementations own that lock internally.
pub fn run_campaign<D, S>(
    request: &CampaignRequest,
    dependencies: &mut D,
    sink: &S,
) -> Result<CampaignOutcome, OrchestratorError>
where
    D: CampaignDependencies,
    S: EventSink,
{
    run_campaign_controlled(request, dependencies, sink, &RunnerControlHandle::new())
}

/// Run a campaign with cooperative pause and force-cancel controls supplied by the caller.
pub fn run_campaign_controlled<D, S>(
    request: &CampaignRequest,
    dependencies: &mut D,
    sink: &S,
    controls: &RunnerControlHandle,
) -> Result<CampaignOutcome, OrchestratorError>
where
    D: CampaignDependencies,
    S: EventSink,
{
    let mut schedule = build_schedule(request, &dependencies.manual_checkpoints())?;
    reject_protected_or_relative_target(&request.created.managed_root)?;
    reject_lock_registry_overlap(request)?;

    emit_started(sink, "lock", "Lock managed installation");
    let target_lock =
        TargetLock::try_acquire(&request.registry_root, &request.created.managed_root)?;
    emit_finished(sink, "lock", StepOutcome::Succeeded);

    emit_started(sink, "freeze", "Freeze campaign recipe and identity");
    let (store, replay, resumed) = open_or_create_campaign(request, &target_lock)?;
    emit_finished(sink, "freeze", StepOutcome::Succeeded);
    sink.emit(EngineEvent::CampaignStarted {
        install_id: request.created.install_id.clone(),
        resumed,
    });

    let progress = Progress::from_replay(&replay, &mut schedule, &request.plan)?;
    dependencies
        .record_campaign(replay.created())
        .map_err(|failure| OrchestratorError::CampaignIndex(failure.into_message()))?;
    if let Some((step_id, reason)) = progress.fresh_copy_required.clone() {
        let outcome = CampaignOutcome::FreshCopyRequired {
            step_id: step_id.clone(),
            reason: reason.clone(),
        };
        sink.emit(EngineEvent::Error {
            step_id: Some(step_id.clone()),
            message: format!(
                "A fresh managed copy is required: {}",
                nonempty_detail(&reason)
            ),
        });
        emit_finished(sink, &step_id, StepOutcome::Failed);
        write_terminal_receipt(dependencies, request, &replay, &progress, &outcome)?;
        drop(target_lock);
        return Ok(outcome);
    }
    if resumed && progress.completed_prefix > 0 {
        emit_started(sink, "preflight", "Recheck campaign safety before resume");
        if let Err(failure) = dependencies.initial(request, &replay) {
            let reason = failure.into_message();
            sink.emit(EngineEvent::Error {
                step_id: Some("preflight".to_owned()),
                message: reason.clone(),
            });
            emit_finished(sink, "preflight", StepOutcome::Failed);
            let outcome = CampaignOutcome::Failed {
                step_id: "preflight".to_owned(),
                reason,
            };
            write_terminal_receipt(dependencies, request, &store.replay()?, &progress, &outcome)?;
            return Ok(outcome);
        }
        emit_finished(sink, "preflight", StepOutcome::Succeeded);
    }
    let mut machine = CampaignMachine {
        request,
        dependencies,
        sink,
        store,
        schedule,
        progress,
        active_phase: None,
        controls,
    };
    let outcome = machine.run()?;
    if outcome == CampaignOutcome::Complete {
        sink.emit(EngineEvent::CampaignFinished {
            install_id: request.created.install_id.clone(),
        });
    } else if let CampaignOutcome::Paused { after_step_id } = &outcome {
        sink.emit(EngineEvent::CampaignPaused {
            install_id: request.created.install_id.clone(),
            after_step_id: after_step_id.clone(),
        });
    } else if let CampaignOutcome::DecisionRequired {
        step_id,
        decision_id,
    } = &outcome
    {
        sink.emit(EngineEvent::DecisionRequired {
            install_id: request.created.install_id.clone(),
            step_id: step_id.clone(),
            decision_id: decision_id.clone(),
        });
    }
    drop(target_lock);
    Ok(outcome)
}

#[derive(Debug, Clone)]
struct PipelineStep {
    id: String,
    label: String,
    kind: PipelineKind,
}

#[derive(Debug, Clone)]
enum PipelineKind {
    Preflight,
    Acquire(FrozenIdentity, ArtifactKind),
    Stage(GameRole),
    Materialize(MaterializationTask),
    /// One run of the plan being installed (fewer components when a dependency was left out).
    Install(Box<PlannedRun>),
    ManualCheckpoint(Box<crate::manual_checkpoint::FrozenCheckpoint>),
    FinalIdentity,
    FinalVerify,
    Receipt,
}

fn build_schedule(
    request: &CampaignRequest,
    checkpoints: &[crate::manual_checkpoint::FrozenCheckpoint],
) -> Result<Vec<PipelineStep>, OrchestratorError> {
    let actual_plan = plan_digest(&request.plan)?;
    if actual_plan != request.created.plan_sha256 {
        return Err(OrchestratorError::InvalidCampaign(format!(
            "execution plan digest {actual_plan} does not match frozen {}",
            request.created.plan_sha256
        )));
    }

    let mut archive_identities = BTreeMap::<String, FrozenIdentity>::new();
    for identity in request.created.artifact_identities.iter().cloned() {
        if archive_identities
            .insert(identity.id.clone(), identity.clone())
            .is_some()
        {
            return Err(OrchestratorError::InvalidCampaign(format!(
                "frozen archive id {:?} appears more than once",
                identity.id
            )));
        }
    }
    let mut tool_identities = BTreeMap::<String, FrozenIdentity>::new();
    for identity in request.created.tool_identities.iter().cloned() {
        if tool_identities
            .insert(identity.id.clone(), identity.clone())
            .is_some()
        {
            return Err(OrchestratorError::InvalidCampaign(format!(
                "frozen tool id {:?} appears more than once",
                identity.id
            )));
        }
    }

    let mut acquisition_ids = Vec::new();
    let mut seen_acquisitions = BTreeSet::new();
    for run in &request.plan.runs {
        validate_run(run, &archive_identities, &tool_identities)?;
        for id in [&run.artifact_id, &run.weidu_artifact_id] {
            if seen_acquisitions.insert(id.clone()) {
                acquisition_ids.push(id.clone());
            }
        }
    }

    let mut materializations = Vec::new();
    let mut seen_materializations = BTreeSet::new();
    for run in &request.plan.runs {
        let task = MaterializationTask {
            artifact_id: run.artifact_id.clone(),
            target: run.target,
        };
        if seen_materializations.insert(task.clone()) {
            materializations.push(task);
        }
    }

    let mut schedule = vec![PipelineStep {
        id: "preflight".to_owned(),
        label: "Check source games, disk space, processes, and files".to_owned(),
        kind: PipelineKind::Preflight,
    }];
    for id in acquisition_ids {
        let identity = archive_identities.get(&id).cloned().ok_or_else(|| {
            OrchestratorError::InvalidCampaign(format!("run references unfrozen archive id {id:?}"))
        })?;
        let kind = if tool_identities.contains_key(&id) {
            ArtifactKind::Tool
        } else {
            ArtifactKind::Payload
        };
        schedule.push(PipelineStep {
            id: format!("acquire:{id}"),
            label: format!("Acquire {id}"),
            kind: PipelineKind::Acquire(identity, kind),
        });
    }
    schedule.extend([
        PipelineStep {
            id: "stage:bg1".to_owned(),
            label: "Stage Baldur's Gate: Enhanced Edition".to_owned(),
            kind: PipelineKind::Stage(GameRole::BgeeSod),
        },
        PipelineStep {
            id: "stage:bg2".to_owned(),
            label: "Stage Baldur's Gate II: Enhanced Edition".to_owned(),
            kind: PipelineKind::Stage(GameRole::Bg2ee),
        },
    ]);
    for task in materializations {
        schedule.push(PipelineStep {
            id: task.step_id(),
            label: format!(
                "Prepare {} for {}",
                task.artifact_id,
                game_root_label(task.target)
            ),
            kind: PipelineKind::Materialize(task),
        });
    }
    for run in &request.plan.runs {
        schedule.push(PipelineStep {
            id: format!("install:{}", run.run_id),
            label: format!("Install {}", run.mod_id),
            kind: PipelineKind::Install(Box::new(run.clone())),
        });
        for checkpoint in checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.after_run_id == run.run_id)
        {
            schedule.push(PipelineStep {
                id: format!("checkpoint:{}", checkpoint.checkpoint_id),
                label: "Paused for your mods".to_owned(),
                kind: PipelineKind::ManualCheckpoint(Box::new(checkpoint.clone())),
            });
        }
    }
    schedule.extend([
        PipelineStep {
            id: "identity:final".to_owned(),
            label: "Finalize isolated save identity".to_owned(),
            kind: PipelineKind::FinalIdentity,
        },
        PipelineStep {
            id: "verify:final".to_owned(),
            label: "Verify the complete installed stack".to_owned(),
            kind: PipelineKind::FinalVerify,
        },
        PipelineStep {
            id: "receipt".to_owned(),
            label: "Record installation result".to_owned(),
            kind: PipelineKind::Receipt,
        },
    ]);

    let mut ids = BTreeSet::new();
    for step in &schedule {
        validate_step_id(&step.id)?;
        if !ids.insert(step.id.clone()) {
            return Err(OrchestratorError::InvalidCampaign(format!(
                "duplicate pipeline step id {:?}",
                step.id
            )));
        }
    }
    Ok(schedule)
}

fn validate_run(
    run: &PlannedRun,
    archive_identities: &BTreeMap<String, FrozenIdentity>,
    tool_identities: &BTreeMap<String, FrozenIdentity>,
) -> Result<(), OrchestratorError> {
    validate_step_fragment("run id", &run.run_id)?;
    if run.components.is_empty() {
        return Err(OrchestratorError::InvalidCampaign(format!(
            "run {:?} has no selected components",
            run.run_id
        )));
    }
    if run.target != run.phase.game_root() {
        return Err(OrchestratorError::InvalidCampaign(format!(
            "run {:?} targets {:?}, but phase {:?} targets {:?}",
            run.run_id,
            run.target,
            run.phase,
            run.phase.game_root()
        )));
    }
    if !archive_identities.contains_key(&run.artifact_id)
        || tool_identities.contains_key(&run.artifact_id)
    {
        return Err(OrchestratorError::InvalidCampaign(format!(
            "run {:?} references missing payload {:?}",
            run.run_id, run.artifact_id
        )));
    }
    if !archive_identities.contains_key(&run.weidu_artifact_id)
        || !tool_identities.contains_key(&run.weidu_artifact_id)
    {
        return Err(OrchestratorError::InvalidCampaign(format!(
            "run {:?} references missing tool {:?}",
            run.run_id, run.weidu_artifact_id
        )));
    }
    Ok(())
}

#[derive(Debug)]
struct Progress {
    completed_prefix: usize,
    attempts: BTreeMap<String, u32>,
    unresolved: Option<(String, u32)>,
    last_failed: BTreeMap<String, u32>,
    fresh_copy_required: Option<(String, String)>,
    /// Payloads left out so far, in ledger order, those of failed runs included.
    left_out: Vec<LeftOutArtifact>,
    /// Components, by planned run, left out because their installer failed, in ledger order.
    failed_parts: Vec<(String, Vec<u32>)>,
    /// The frozen plan minus everything left out so far.
    effective_plan: InstallPlan,
    /// The question that the unresolved step asked and that was not withdrawn.
    pending: Option<PendingDecision>,
    /// The install step that left a failed component out and goes on, with the components its
    /// next attempt installs. Later attempts prove their own remaining components.
    continuing: Option<(String, Vec<u32>)>,
    /// The attempt that ran the installer, by step, for a step asking again after a stop: the
    /// attempts that only asked have no evidence of their own.
    evidence_attempts: BTreeMap<String, u32>,
}

impl Progress {
    /// Replays the ledger against the schedule. A recorded leave-out rebuilds the remaining
    /// plan from the runs it records, checks it against the recorded digest, and removes the
    /// steps that are no longer needed from the rest of `schedule`, exactly as when it
    /// happened.
    fn from_replay(
        replay: &SessionReplay,
        schedule: &mut Vec<PipelineStep>,
        frozen_plan: &InstallPlan,
    ) -> Result<Self, OrchestratorError> {
        let mut progress = Self {
            completed_prefix: 0,
            attempts: BTreeMap::new(),
            unresolved: None,
            last_failed: BTreeMap::new(),
            fresh_copy_required: None,
            left_out: Vec::new(),
            failed_parts: Vec::new(),
            effective_plan: frozen_plan.clone(),
            pending: None,
            continuing: None,
            evidence_attempts: BTreeMap::new(),
        };

        for record in replay.records.iter().skip(1) {
            if !matches!(
                record.event,
                SessionEvent::DecisionRequired { .. }
                    | SessionEvent::DecisionAnswered { .. }
                    | SessionEvent::DecisionWithdrawn { .. }
            ) {
                progress.pending = None;
            }
            match &record.event {
                SessionEvent::DecisionRequired {
                    step_id,
                    attempt,
                    decision_id,
                    question,
                } => {
                    if progress.unresolved.as_ref() != Some(&(step_id.clone(), *attempt)) {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "question {decision_id} for {step_id:?} has no matching intent"
                        )));
                    }
                    let asks_own_artifact = match &schedule[progress.completed_prefix].kind {
                        PipelineKind::Acquire(identity, kind) => {
                            identity == &question.reviewed && *kind == question.artifact_kind
                        }
                        PipelineKind::Materialize(task) => {
                            task.artifact_id == question.artifact_id
                                && question.kind == QuestionKind::UnusableDownload
                        }
                        PipelineKind::Stage(role) => {
                            question.kind == QuestionKind::ModdedSource
                                && question.artifact_id == crate::decision::source_subject(*role)
                        }
                        PipelineKind::Install(run) => {
                            question.kind == QuestionKind::FailedComponent
                                && question.artifact_id == run.artifact_id
                        }
                        _ => false,
                    };
                    if !asks_own_artifact {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "question {decision_id} at step {step_id:?} is not about what that step acquires, places or installs"
                        )));
                    }
                    if let Some(evidence) = question
                        .failed
                        .as_ref()
                        .and_then(|failed| failed.evidence_attempt)
                    {
                        progress.evidence_attempts.insert(step_id.clone(), evidence);
                    }
                    progress.pending = Some(PendingDecision {
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
                    let Some(pending) = progress
                        .pending
                        .as_mut()
                        .filter(|pending| &pending.decision_id == decision_id)
                    else {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "answer to {decision_id} has no standing question"
                        )));
                    };
                    if !pending.question.offers(*answer) {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "answer to {decision_id} is not one the question offers"
                        )));
                    }
                    pending.answer = Some(RecordedAnswer {
                        answer: *answer,
                        source: *source,
                        answered_at: record.recorded_at,
                    });
                }
                SessionEvent::DecisionWithdrawn { decision_id, .. } => {
                    if progress
                        .pending
                        .as_ref()
                        .is_none_or(|pending| &pending.decision_id != decision_id)
                    {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "withdrawal of {decision_id} has no standing question"
                        )));
                    }
                    progress.pending = None;
                }
                SessionEvent::StepStarted { step_id, attempt } => {
                    if schedule.get(progress.completed_prefix).map(|step| &step.id) != Some(step_id)
                    {
                        return Err(OrchestratorError::InvalidCampaign(
                            if schedule.iter().any(|step| &step.id == step_id) {
                                format!(
                                    "ledger started {step_id:?} out of order; expected {:?}",
                                    schedule.get(progress.completed_prefix).map(|step| &step.id)
                                )
                            } else {
                                format!("ledger contains unknown step {step_id:?}")
                            },
                        ));
                    }
                    let expected_attempt = progress.next_attempt(step_id)?;
                    if *attempt != expected_attempt {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "step {step_id:?} attempt {attempt} is not monotonic; expected {expected_attempt}"
                        )));
                    }
                    progress.attempts.insert(step_id.clone(), *attempt);
                    progress.unresolved = Some((step_id.clone(), *attempt));
                    progress.continuing = None;
                }
                SessionEvent::StepCompleted { step_id, attempt } => {
                    if progress.unresolved.as_ref() != Some(&(step_id.clone(), *attempt)) {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "completion for {step_id:?} has no matching intent"
                        )));
                    }
                    progress.completed_prefix += 1;
                    progress.unresolved = None;
                    progress.last_failed.remove(step_id);
                }
                SessionEvent::ArtifactLeftOut {
                    step_id,
                    attempt,
                    artifact_id,
                    detail,
                    plan_sha256,
                    removed_runs,
                    changed_runs,
                    failed_run,
                } => {
                    if progress.unresolved.as_ref() != Some(&(step_id.clone(), *attempt)) {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "left-out record for {step_id:?} has no matching intent"
                        )));
                    }
                    if let Some(failed) = failed_run {
                        let PipelineKind::Install(run) = &schedule[progress.completed_prefix].kind
                        else {
                            return Err(OrchestratorError::InvalidCampaign(format!(
                                "ledger leaves out a failed run at step {step_id:?}, which installs nothing"
                            )));
                        };
                        let run = (**run).clone();
                        if run.run_id != failed.run_id || run.artifact_id != *artifact_id {
                            return Err(OrchestratorError::InvalidCampaign(format!(
                                "ledger leaves out failed run {:?} of {artifact_id:?} at the step of run {:?}",
                                failed.run_id, run.run_id
                            )));
                        }
                        let plan = apply_plan_change(
                            &progress.effective_plan,
                            removed_runs,
                            changed_runs,
                        )
                        .map_err(|detail| {
                            OrchestratorError::InvalidCampaign(format!(
                                "the record that the installation continues without the failed part of {:?} does not fit the plan before it: {detail}",
                                run.run_id
                            ))
                        })?;
                        validate_reduced_plan(
                            &progress.effective_plan,
                            &plan,
                            &progress.left_out_ids(),
                        )?;
                        check_failed_run_plan(
                            &progress.effective_plan,
                            &plan,
                            &progress.installed_runs(schedule),
                            &run,
                            &failed.components,
                            &failed.continues_with,
                        )
                        .map_err(OrchestratorError::InvalidCampaign)?;
                        if plan_digest(&plan)? != *plan_sha256 {
                            return Err(OrchestratorError::InvalidCampaign(format!(
                                "the plan without the failed part of {:?} differs from the one recorded when it was left out",
                                run.run_id
                            )));
                        }
                        // A record of an earlier version left out everything it did not install.
                        let left_out = if failed.left_out.is_empty() {
                            failed.components.clone()
                        } else {
                            failed.left_out.clone()
                        };
                        progress.record_failed_run(schedule, &run, detail, left_out, plan);
                        progress.finish_failed_run(step_id, &failed.continues_with);
                        continue;
                    }
                    let step = &schedule[progress.completed_prefix];
                    let at_materialization = match &step.kind {
                        PipelineKind::Acquire(identity, ArtifactKind::Payload)
                            if &identity.id == artifact_id =>
                        {
                            false
                        }
                        PipelineKind::Materialize(task) if &task.artifact_id == artifact_id => true,
                        _ => {
                            return Err(OrchestratorError::InvalidCampaign(format!(
                                "ledger leaves out {artifact_id:?} at step {step_id:?}, which neither acquires nor places that payload"
                            )))
                        }
                    };
                    let mut ids = progress.left_out_ids();
                    ids.push(artifact_id.clone());
                    // The record, not a new derivation, says what remains: a resume after an
                    // app update installs exactly what the campaign chose when it happened.
                    let plan = apply_plan_change(
                        &progress.effective_plan,
                        removed_runs,
                        changed_runs,
                    )
                    .map_err(|detail| {
                        OrchestratorError::InvalidCampaign(format!(
                            "the record that the installation continues without {artifact_id:?} does not fit the plan before it: {detail}"
                        ))
                    })?;
                    validate_reduced_plan(&progress.effective_plan, &plan, &ids)?;
                    if plan_digest(&plan)? != *plan_sha256 {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "the plan without {artifact_id:?} differs from the one recorded when it was left out"
                        )));
                    }
                    if at_materialization {
                        let placed = placed_payloads_left_behind(
                            &schedule[..progress.completed_prefix],
                            &progress.left_out_ids(),
                            artifact_id,
                            &plan,
                        );
                        if !placed.is_empty() {
                            return Err(OrchestratorError::InvalidCampaign(format!(
                                "ledger leaves out {artifact_id:?} although files of {placed:?} were already placed"
                            )));
                        }
                    }
                    progress.record_left_out(schedule, artifact_id, detail, plan);
                    progress.completed_prefix += 1;
                    progress.unresolved = None;
                    progress.last_failed.remove(step_id);
                }
                SessionEvent::StepFailed {
                    step_id, attempt, ..
                } => {
                    if progress.unresolved.as_ref() != Some(&(step_id.clone(), *attempt)) {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "failure for {step_id:?} has no matching intent"
                        )));
                    }
                    progress.unresolved = None;
                    progress.last_failed.insert(step_id.clone(), *attempt);
                }
                SessionEvent::FreshCopyRequired {
                    step_id,
                    attempt,
                    detail,
                } => {
                    let matches_active =
                        progress.unresolved.as_ref() == Some(&(step_id.clone(), *attempt));
                    let upgrades_failure = progress.last_failed.get(step_id) == Some(attempt);
                    if !matches_active && !upgrades_failure {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "fresh-copy seal for {step_id:?} has no matching intent"
                        )));
                    }
                    progress.unresolved = None;
                    progress.last_failed.remove(step_id);
                    progress.fresh_copy_required = Some((step_id.clone(), detail.clone()));
                }
                SessionEvent::Created(_) => {
                    return Err(OrchestratorError::InvalidCampaign(
                        "ledger repeats campaign creation".to_owned(),
                    ))
                }
            }
        }

        Ok(progress)
    }

    /// Payloads left out whole because of their downloads. A mod left out after its installer
    /// failed is not among them: parts of it may be installed already.
    fn left_out_ids(&self) -> Vec<String> {
        self.left_out
            .iter()
            .filter(|left_out| left_out.failed_run.is_none())
            .map(|left_out| left_out.artifact_id.clone())
            .collect()
    }

    /// The planned runs whose install steps completed before the current step.
    fn installed_runs(&self, schedule: &[PipelineStep]) -> BTreeSet<String> {
        schedule[..self.completed_prefix]
            .iter()
            .filter_map(|step| match &step.kind {
                PipelineKind::Install(run) => Some(run.run_id.clone()),
                _ => None,
            })
            .collect()
    }

    /// Records one left-out payload whose acquisition step is at `completed_prefix`, and
    /// drops the later steps that `plan` no longer needs. Earlier steps stay as they ran.
    fn record_left_out(
        &mut self,
        schedule: &mut Vec<PipelineStep>,
        artifact_id: &str,
        reason: &str,
        plan: InstallPlan,
    ) -> &LeftOutArtifact {
        let remaining = plan
            .runs
            .iter()
            .map(|run| (run.run_id.as_str(), run))
            .collect::<BTreeMap<_, _>>();
        let mut runs = Vec::new();
        let mut changed_runs = Vec::new();
        for run in &self.effective_plan.runs {
            match remaining.get(run.run_id.as_str()) {
                None => runs.push(run.run_id.clone()),
                Some(kept) if *kept != run => changed_runs.push(run.run_id.clone()),
                Some(_) => {}
            }
        }
        let tail = schedule.split_off(self.completed_prefix + 1);
        schedule.extend(tail.into_iter().filter_map(|mut step| {
            let keep = match &mut step.kind {
                PipelineKind::Acquire(identity, _) => plan.runs.iter().any(|run| {
                    run.artifact_id == identity.id || run.weidu_artifact_id == identity.id
                }),
                PipelineKind::Materialize(task) => plan
                    .runs
                    .iter()
                    .any(|run| run.artifact_id == task.artifact_id && run.target == task.target),
                PipelineKind::Install(run) => match remaining.get(run.run_id.as_str()) {
                    Some(kept) => {
                        **run = (*kept).clone();
                        true
                    }
                    None => false,
                },
                PipelineKind::Preflight
                | PipelineKind::Stage(_)
                | PipelineKind::ManualCheckpoint(_)
                | PipelineKind::FinalIdentity
                | PipelineKind::FinalVerify
                | PipelineKind::Receipt => true,
            };
            keep.then_some(step)
        }));
        self.effective_plan = plan;
        self.left_out.push(LeftOutArtifact {
            artifact_id: artifact_id.to_owned(),
            reason: reason.to_owned(),
            runs,
            changed_runs,
            failed_run: None,
        });
        self.left_out
            .last()
            .expect("left-out payload was just recorded")
    }

    /// Records that the `failed` components of `run`, whose install step is at
    /// `completed_prefix`, are left out, with `plan` what remains, and drops the later steps
    /// that `plan` no longer needs. The step itself now installs the run as `plan` has it.
    fn record_failed_run(
        &mut self,
        schedule: &mut Vec<PipelineStep>,
        run: &PlannedRun,
        reason: &str,
        failed: Vec<u32>,
        plan: InstallPlan,
    ) -> &LeftOutArtifact {
        if let (PipelineKind::Install(current), Some(kept)) = (
            &mut schedule[self.completed_prefix].kind,
            plan.runs.iter().find(|kept| kept.run_id == run.run_id),
        ) {
            **current = kept.clone();
        }
        self.record_left_out(schedule, &run.artifact_id, reason, plan);
        self.failed_parts.push((run.run_id.clone(), failed));
        let left_out = self
            .left_out
            .last_mut()
            .expect("left-out part was just recorded");
        left_out.failed_run = Some(run.run_id.clone());
        left_out
    }

    /// Ends the attempt of `step_id` that left a failed component out. The step is over unless
    /// its run goes on with `continues_with`.
    fn finish_failed_run(&mut self, step_id: &str, continues_with: &[u32]) {
        self.unresolved = None;
        self.pending = None;
        self.last_failed.remove(step_id);
        if continues_with.is_empty() {
            self.completed_prefix += 1;
        } else {
            self.continuing = Some((step_id.to_owned(), continues_with.to_vec()));
        }
    }

    fn next_attempt(&self, step_id: &str) -> Result<u32, OrchestratorError> {
        self.attempts
            .get(step_id)
            .copied()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| {
                OrchestratorError::InvalidCampaign(format!(
                    "attempt counter overflow for {step_id:?}"
                ))
            })
    }
}

/// What a question that a crash kept from being recorded asks about.
struct UnaskedQuestion {
    /// The attempt that ran the installer.
    ran: u32,
    /// What it did not install.
    failed: Vec<u32>,
    /// What it installed after the failure.
    installed_after: Vec<u32>,
}

/// `reduced` with every run in `installed` exactly as `previous` has it, even one it left out:
/// those runs are in the game already, so leaving out a later failed part never changes them.
fn adopt_installed_runs(
    previous: &InstallPlan,
    reduced: InstallPlan,
    installed: &BTreeSet<String>,
) -> InstallPlan {
    let runs = previous
        .runs
        .iter()
        .filter_map(|before| {
            if installed.contains(&before.run_id) {
                return Some(before.clone());
            }
            reduced
                .runs
                .iter()
                .find(|run| run.run_id == before.run_id)
                .cloned()
        })
        .collect();
    InstallPlan { runs }
}

/// `plan`, the plan after leaving out the failed part of `run`, with what WeiDU installed of
/// `run` restored: all its components but `not_installed`, which are in the game whatever else
/// left with the failed one, with their prompt answers. When the run `goes_on`, also returns
/// the components after the first of `not_installed` that it still installs. Nothing else of
/// `plan` changes.
fn keep_installed_part(
    plan: InstallPlan,
    previous: &InstallPlan,
    run: &PlannedRun,
    not_installed: &[u32],
    goes_on: bool,
) -> (InstallPlan, Vec<u32>) {
    let done = run
        .components
        .iter()
        .copied()
        .filter(|component| !not_installed.contains(component))
        .collect::<Vec<_>>();
    let kept = plan.runs.iter().find(|kept| kept.run_id == run.run_id);
    let continues = kept.filter(|_| goes_on).map_or_else(Vec::new, |kept| {
        not_installed[1..]
            .iter()
            .copied()
            .filter(|component| kept.components.contains(component))
            .collect::<Vec<_>>()
    });
    let mut merged = run.clone();
    merged.components = run
        .components
        .iter()
        .copied()
        .filter(|component| done.contains(component) || continues.contains(component))
        .collect();
    merged.postconditions.clear();
    merged.prompt_scripts = merged
        .components
        .iter()
        .filter_map(|component| {
            let source = if done.contains(component) {
                Some(run)
            } else {
                kept
            };
            source?
                .prompt_scripts
                .iter()
                .find(|script| script.component.component == *component)
                .cloned()
        })
        .collect();
    let mut position = 0;
    let mut runs = Vec::with_capacity(plan.runs.len() + 1);
    let order = previous
        .runs
        .iter()
        .map(|before| before.run_id.as_str())
        .collect::<Vec<_>>();
    let rank = |run_id: &str| order.iter().position(|id| *id == run_id);
    for kept in plan.runs {
        if kept.run_id != run.run_id {
            if rank(&kept.run_id) < rank(&run.run_id) {
                position += 1;
            }
            runs.push(kept);
        }
    }
    if !merged.components.is_empty() {
        runs.insert(position, merged);
    }
    (InstallPlan { runs }, continues)
}

/// `reduced`, the plan without the failed part of `run`, keeping all that is in the game: the
/// runs in `installed` and what WeiDU installed of `run`. Also returns the components that `run`
/// goes on with, as [`keep_installed_part`] does, and fails when the plan is not part of
/// `previous` or does not keep what is in the game.
fn plan_keeping_installed(
    previous: &InstallPlan,
    reduced: InstallPlan,
    installed: &BTreeSet<String>,
    left_out: &[String],
    run: &PlannedRun,
    not_installed: &[u32],
    goes_on: bool,
) -> Result<(InstallPlan, Vec<u32>), String> {
    let (plan, continues) = keep_installed_part(
        adopt_installed_runs(previous, reduced, installed),
        previous,
        run,
        not_installed,
        goes_on,
    );
    validate_reduced_plan(previous, &plan, left_out).map_err(|error| error.to_string())?;
    check_failed_run_plan(previous, &plan, installed, run, not_installed, &continues)?;
    Ok((plan, continues))
}

/// Fails unless `plan`, the plan after leaving out the failed part of `run`, keeps what is in
/// the game already: every run in `installed` exactly as `previous` has it, and of `run` exactly
/// the components but `failed`, which WeiDU installed, and `continues`, those of `failed` that
/// it still installs (or nothing of it, when it has neither). Otherwise the game would hold
/// parts the plan no longer has.
fn check_failed_run_plan(
    previous: &InstallPlan,
    plan: &InstallPlan,
    installed: &BTreeSet<String>,
    run: &PlannedRun,
    failed: &[u32],
    continues: &[u32],
) -> Result<(), String> {
    if failed.is_empty() || !crate::decision::in_order(&run.components, failed) {
        return Err(format!(
            "components {failed:?} are not components of run {:?}",
            run.run_id
        ));
    }
    if !crate::decision::continues_after_failure(failed, continues) {
        return Err(format!(
            "components {continues:?} do not follow the failed one in run {:?}",
            run.run_id
        ));
    }
    for before in previous
        .runs
        .iter()
        .filter(|before| installed.contains(&before.run_id))
    {
        if plan.runs.iter().find(|kept| kept.run_id == before.run_id) != Some(before) {
            return Err(format!(
                "run {:?} is installed already, and leaving out the failed part would change it",
                before.run_id
            ));
        }
    }
    let expected = run
        .components
        .iter()
        .copied()
        .filter(|component| !failed.contains(component) || continues.contains(component))
        .collect::<Vec<_>>();
    match plan.runs.iter().find(|kept| kept.run_id == run.run_id) {
        None if expected.is_empty() => Ok(()),
        Some(kept) if !expected.is_empty() && kept.components == expected => Ok(()),
        _ => Err(format!(
            "run {:?} would keep other components than the {expected:?} it installed and goes on with",
            run.run_id
        )),
    }
}

/// What leaving a payload out changes in `previous`: the runs `reduced` no longer has, in plan
/// order, and the runs it keeps in another form, as `reduced` has them.
fn plan_change(previous: &InstallPlan, reduced: &InstallPlan) -> (Vec<String>, Vec<PlannedRun>) {
    let kept = reduced
        .runs
        .iter()
        .map(|run| (run.run_id.as_str(), run))
        .collect::<BTreeMap<_, _>>();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    for run in &previous.runs {
        match kept.get(run.run_id.as_str()) {
            None => removed.push(run.run_id.clone()),
            Some(kept) if *kept != run => changed.push((*kept).clone()),
            Some(_) => {}
        }
    }
    (removed, changed)
}

/// Applies a recorded [`plan_change`] to `previous`. Fails when the record names runs that
/// `previous` does not have, or names one run twice.
fn apply_plan_change(
    previous: &InstallPlan,
    removed: &[String],
    changed: &[PlannedRun],
) -> Result<InstallPlan, String> {
    let planned = previous
        .runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut removed_ids = BTreeSet::new();
    for run_id in removed {
        if !planned.contains(run_id.as_str()) {
            return Err(format!("removed run {run_id:?} is not planned"));
        }
        if !removed_ids.insert(run_id.as_str()) {
            return Err(format!("run {run_id:?} is removed twice"));
        }
    }
    let mut changed_by_id = BTreeMap::new();
    for run in changed {
        if !planned.contains(run.run_id.as_str()) {
            return Err(format!("changed run {:?} is not planned", run.run_id));
        }
        if removed_ids.contains(run.run_id.as_str())
            || changed_by_id.insert(run.run_id.as_str(), run).is_some()
        {
            return Err(format!("run {:?} is recorded twice", run.run_id));
        }
    }
    Ok(InstallPlan {
        runs: previous
            .runs
            .iter()
            .filter(|run| !removed_ids.contains(run.run_id.as_str()))
            .map(|run| {
                changed_by_id
                    .get(run.run_id.as_str())
                    .map_or_else(|| run.clone(), |changed| (*changed).clone())
            })
            .collect(),
    })
}

/// The payloads whose files a completed step already placed on a game although `plan` no
/// longer installs them there: the payload being left out, or a mod leaving with it. Payloads
/// left out before (`left_out`) never had files placed, since a leave-out requires that.
fn placed_payloads_left_behind(
    completed: &[PipelineStep],
    left_out: &[String],
    artifact_id: &str,
    plan: &InstallPlan,
) -> Vec<String> {
    let mut placed = Vec::<String>::new();
    for step in completed {
        let PipelineKind::Materialize(task) = &step.kind else {
            continue;
        };
        let still_installed = task.artifact_id != artifact_id
            && plan
                .runs
                .iter()
                .any(|run| run.artifact_id == task.artifact_id && run.target == task.target);
        if !still_installed
            && !left_out.contains(&task.artifact_id)
            && !placed.contains(&task.artifact_id)
        {
            placed.push(task.artifact_id.clone());
        }
    }
    placed
}

/// Fails closed when a backend's remaining plan is not exactly a part of the plan before it:
/// leaving a payload out may only remove runs, components and the prompts that went with
/// them, never add, reorder or retarget anything.
fn validate_reduced_plan(
    previous: &InstallPlan,
    reduced: &InstallPlan,
    left_out: &[String],
) -> Result<(), OrchestratorError> {
    let invalid = |detail: String| {
        OrchestratorError::InvalidCampaign(format!(
            "the plan without {left_out:?} is not part of the plan before it: {detail}"
        ))
    };
    let mut previous_runs = previous.runs.iter();
    for run in &reduced.runs {
        let Some(before) = previous_runs.find(|before| before.run_id == run.run_id) else {
            return Err(invalid(format!(
                "run {:?} is new or out of order",
                run.run_id
            )));
        };
        if run.mod_id != before.mod_id
            || run.target != before.target
            || run.phase != before.phase
            || run.args != before.args
            || run.artifact_id != before.artifact_id
            || run.weidu_artifact_id != before.weidu_artifact_id
        {
            return Err(invalid(format!(
                "run {:?} changed its identity",
                run.run_id
            )));
        }
        if left_out.contains(&run.artifact_id) {
            return Err(invalid(format!(
                "run {:?} still installs a left-out payload",
                run.run_id
            )));
        }
        let mut before_components = before.components.iter();
        if run.components.is_empty()
            || !run
                .components
                .iter()
                .all(|component| before_components.any(|before| before == component))
        {
            return Err(invalid(format!(
                "run {:?} has components that were not planned in this order",
                run.run_id
            )));
        }
        if run
            .prompt_scripts
            .iter()
            .any(|script| !run.components.contains(&script.component.component))
        {
            return Err(invalid(format!(
                "run {:?} answers prompts of components it does not install",
                run.run_id
            )));
        }
        if !run.postconditions.is_empty() && run.postconditions != before.postconditions {
            return Err(invalid(format!(
                "run {:?} has checks that were not planned",
                run.run_id
            )));
        }
    }
    Ok(())
}

fn build_receipt_draft(
    request: &CampaignRequest,
    replay: &SessionReplay,
    progress: &Progress,
    outcome: ReceiptDraftOutcome,
) -> Result<ReceiptDraft, OrchestratorError> {
    let first = replay.records.first().ok_or_else(|| {
        OrchestratorError::InvalidCampaign("campaign ledger has no creation record".to_owned())
    })?;
    let last = replay.records.last().ok_or_else(|| {
        OrchestratorError::InvalidCampaign("campaign ledger has no terminal timestamp".to_owned())
    })?;
    let attempt_id = if matches!(outcome, ReceiptDraftOutcome::Succeeded) {
        request.created.attempt_id.clone()
    } else {
        let outcome_identity = match &outcome {
            ReceiptDraftOutcome::Succeeded => unreachable!("success uses the campaign attempt id"),
            ReceiptDraftOutcome::Failed { step_id, detail } => {
                format!("failed\0{step_id}\0{detail}")
            }
            ReceiptDraftOutcome::FreshCopyRequired { step_id, detail } => {
                format!("fresh-copy-required\0{step_id}\0{detail}")
            }
        };
        format!(
            "terminal-{:010}-{}",
            last.sequence,
            &sha256_bytes(outcome_identity.as_bytes())[..16]
        )
    };
    Ok(ReceiptDraft {
        install_id: request.created.install_id.clone(),
        attempt_id,
        started_at_millis: first.recorded_at,
        completed_at_millis: last.recorded_at,
        outcome,
        created: request.created.clone(),
        plan: request.plan.clone(),
        left_out: progress.left_out.clone(),
        installed_plan: (!progress.left_out.is_empty()).then(|| progress.effective_plan.clone()),
        decisions: answered_decisions(&replay.records),
    })
}

fn write_terminal_receipt<D>(
    dependencies: &mut D,
    request: &CampaignRequest,
    replay: &SessionReplay,
    progress: &Progress,
    outcome: &CampaignOutcome,
) -> Result<(), OrchestratorError>
where
    D: ReceiptWriter,
{
    let draft_outcome = match outcome {
        CampaignOutcome::Complete => {
            return Err(OrchestratorError::InvalidCampaign(
                "success receipt must be written by the receipt pipeline step".to_owned(),
            ));
        }
        CampaignOutcome::Paused { .. } | CampaignOutcome::DecisionRequired { .. } => {
            return Err(OrchestratorError::InvalidCampaign(
                "a paused campaign must not publish a terminal receipt".to_owned(),
            ));
        }
        CampaignOutcome::Failed { step_id, reason } => ReceiptDraftOutcome::Failed {
            step_id: step_id.clone(),
            detail: reason.clone(),
        },
        CampaignOutcome::FreshCopyRequired { step_id, reason } => {
            ReceiptDraftOutcome::FreshCopyRequired {
                step_id: step_id.clone(),
                detail: reason.clone(),
            }
        }
    };
    let draft = build_receipt_draft(request, replay, progress, draft_outcome)?;
    dependencies
        .write(&draft)
        .map_err(|failure| OrchestratorError::TerminalReceipt(failure.into_message()))
}

struct CampaignMachine<'a, D, S> {
    request: &'a CampaignRequest,
    dependencies: &'a mut D,
    sink: &'a S,
    store: SessionStore,
    schedule: Vec<PipelineStep>,
    progress: Progress,
    active_phase: Option<Phase>,
    controls: &'a RunnerControlHandle,
}

impl<D, S> CampaignMachine<'_, D, S>
where
    D: CampaignDependencies,
    S: EventSink,
{
    fn run(&mut self) -> Result<CampaignOutcome, OrchestratorError> {
        while self.progress.completed_prefix < self.schedule.len() {
            let step = self.schedule[self.progress.completed_prefix].clone();
            let outcome = match step.kind.clone() {
                PipelineKind::Install(run) => self.run_install(&step, *run)?,
                _ => self.run_simple(&step)?,
            };
            if let Some(outcome) = outcome {
                let replay = self.store.replay()?;
                if !matches!(
                    outcome,
                    CampaignOutcome::Paused { .. } | CampaignOutcome::DecisionRequired { .. }
                ) {
                    write_terminal_receipt(
                        self.dependencies,
                        self.request,
                        &replay,
                        &self.progress,
                        &outcome,
                    )?;
                }
                return Ok(outcome);
            }
            if self.progress.completed_prefix < self.schedule.len()
                && self.controls.pause_requested()
            {
                return Ok(CampaignOutcome::Paused {
                    after_step_id: step.id,
                });
            }
        }
        Ok(CampaignOutcome::Complete)
    }

    fn run_simple(
        &mut self,
        step: &PipelineStep,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        let attempt = self.begin_or_resume(step)?;
        if let Some(pending) = self.progress.pending.clone() {
            if pending.step_id != step.id || pending.attempt != attempt.attempt {
                return Err(OrchestratorError::InvalidCampaign(format!(
                    "question {} belongs to {:?}, not to the next step {:?}",
                    pending.decision_id, pending.step_id, step.id
                )));
            }
            return self.continue_decision(step, &attempt, pending);
        }
        let execution = match &step.kind {
            PipelineKind::ManualCheckpoint(checkpoint) => {
                // The preceding WeiDU invocation has exited and its exact log/debug
                // evidence is complete. Never hand over a suspended child process.
                match self
                    .dependencies
                    .manual_checkpoint(checkpoint, &self.progress.effective_plan)
                {
                    Ok(false) => {
                        return Ok(Some(CampaignOutcome::Paused {
                            after_step_id: step.id.clone(),
                        }))
                    }
                    Ok(true) => Ok(SimpleExecution::Complete),
                    Err(error) => Err(error),
                }
            }
            PipelineKind::Preflight => {
                let replay = self.store.replay()?;
                self.dependencies
                    .initial(self.request, &replay)
                    .map(|()| SimpleExecution::Complete)
            }
            PipelineKind::Acquire(identity, kind) => self.acquire_execution(identity, *kind),
            PipelineKind::Stage(role) => {
                // A game that looks modded is asked about once, before anything is copied; a
                // Yes in an earlier attempt of this step still stands.
                let question = if self.source_answered_yes(step)? {
                    Ok(None)
                } else {
                    self.dependencies.source_question(*role)
                };
                match question {
                    Ok(Some(question)) => Ok(SimpleExecution::Ask(Box::new(question))),
                    Ok(None) => self.stage_source(step, *role),
                    Err(failure) => Err(failure),
                }
            }
            PipelineKind::Materialize(task) => match self
                .recheck(step, task.target, MutationKind::Materialize)
                .and_then(|()| self.dependencies.materialize(task))
            {
                Ok(MaterializationOutcome::Complete) => Ok(SimpleExecution::Complete),
                Ok(MaterializationOutcome::FreshCopyRequired { reason }) => {
                    Ok(SimpleExecution::FreshCopyRequired(reason))
                }
                Ok(MaterializationOutcome::LeaveOut { reason }) => {
                    Ok(self.leave_out_at_materialization(task, reason))
                }
                Err(failure) => Err(failure),
            },
            PipelineKind::FinalIdentity => self
                .recheck(step, GameRoot::Bg2, MutationKind::FinalIdentity)
                .and_then(|()| self.dependencies.finalize_identity())
                .map(|()| SimpleExecution::Complete),
            PipelineKind::FinalVerify => self
                .dependencies
                .verify_final(&self.progress.effective_plan)
                .map(|()| SimpleExecution::Complete),
            PipelineKind::Receipt => (|| {
                let replay = self
                    .store
                    .replay()
                    .map_err(|error| StepFailure::new(error.to_string()))?;
                let draft = build_receipt_draft(
                    self.request,
                    &replay,
                    &self.progress,
                    ReceiptDraftOutcome::Succeeded,
                )
                .map_err(|error| StepFailure::new(error.to_string()))?;
                self.dependencies
                    .write(&draft)
                    .map(|()| SimpleExecution::Complete)
            })(),
            PipelineKind::Install(_) => unreachable!("install steps use run_install"),
        };
        self.finish_simple(step, &attempt, execution)
    }

    /// Ends or continues a simple step as `execution` says.
    fn finish_simple(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        execution: Result<SimpleExecution, StepFailure>,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        match execution {
            Ok(SimpleExecution::Complete) => {
                self.complete_step(step, attempt)?;
                Ok(None)
            }
            Ok(SimpleExecution::Ask(question)) => self.ask(step, attempt, *question),
            Ok(SimpleExecution::FreshCopyRequired(reason)) => {
                self.seal_fresh_copy(step, attempt, reason).map(Some)
            }
            Err(failure) => self.failed(step, attempt, failure.into_message()),
        }
    }

    /// Acquires `identity` and says how its step goes on: complete, or with a question.
    fn acquire_execution(
        &mut self,
        identity: &FrozenIdentity,
        kind: ArtifactKind,
    ) -> Result<SimpleExecution, StepFailure> {
        match self.dependencies.acquire(identity, kind)? {
            AcquisitionOutcome::Ready => Ok(SimpleExecution::Complete),
            AcquisitionOutcome::LeaveOut { reason } => {
                self.unusable_question(identity, kind, reason)
            }
            AcquisitionOutcome::Ask {
                kind: question_kind,
                served,
            } => self.changed_question(identity, kind, question_kind, served),
        }
    }

    /// The question whether to continue without `identity`, whose download cannot be used, and
    /// without everything that needs it. A tool, or a payload the recipe cannot do without,
    /// fails the step instead: that is never a question.
    fn unusable_question(
        &mut self,
        identity: &FrozenIdentity,
        kind: ArtifactKind,
        reason: UnusableReason,
    ) -> Result<SimpleExecution, StepFailure> {
        if kind == ArtifactKind::Tool {
            return Err(self.cannot_do_without(&identity.id, &reason, None));
        }
        let mut ids = self.progress.left_out_ids();
        ids.push(identity.id.clone());
        let plan = match self.dependencies.plan_without(&self.request.plan, &ids) {
            Ok(plan) => plan,
            Err(failure) => {
                return Err(self.cannot_do_without(&identity.id, &reason, Some(failure)))
            }
        };
        let leave_out = self.closure(&plan, &ids)?;
        Ok(SimpleExecution::Ask(Box::new(Question {
            kind: QuestionKind::UnusableDownload,
            artifact_id: identity.id.clone(),
            artifact_kind: kind,
            reviewed: identity.clone(),
            served: None,
            reason: Some(reason),
            leave_out: Some(leave_out),
            failed: None,
        })))
    }

    /// The question whether to install a changed download of `identity`. Leaving the mod out
    /// is offered too when the recipe can do without it; never for a tool.
    fn changed_question(
        &mut self,
        identity: &FrozenIdentity,
        kind: ArtifactKind,
        question_kind: QuestionKind,
        served: ServedDownload,
    ) -> Result<SimpleExecution, StepFailure> {
        let fits = match question_kind {
            QuestionKind::ChangedDownload => true,
            QuestionKind::UnreviewedTool => kind == ArtifactKind::Tool,
            QuestionKind::UnusableDownload
            | QuestionKind::ModdedSource
            | QuestionKind::FailedComponent => false,
        };
        if !fits {
            return Err(StepFailure::new(format!(
                "acquiring {} asked a {} question, which does not fit it",
                identity.id,
                question_kind.as_str()
            )));
        }
        let leave_out = if kind == ArtifactKind::Payload {
            let mut ids = self.progress.left_out_ids();
            ids.push(identity.id.clone());
            match self.dependencies.plan_without(&self.request.plan, &ids) {
                Ok(plan) => Some(self.closure(&plan, &ids)?),
                // A part the recipe cannot do without is only offered to install or stop.
                Err(_) => None,
            }
        } else {
            None
        };
        Ok(SimpleExecution::Ask(Box::new(Question {
            kind: question_kind,
            artifact_id: identity.id.clone(),
            artifact_kind: kind,
            reviewed: identity.clone(),
            served: Some(served),
            reason: None,
            leave_out,
            failed: None,
        })))
    }

    /// What leaving out `ids` changes in the plan being installed, for a question.
    fn closure(&self, plan: &InstallPlan, ids: &[String]) -> Result<LeaveOutClosure, StepFailure> {
        validate_reduced_plan(&self.progress.effective_plan, plan, ids)
            .map_err(|error| StepFailure::new(error.to_string()))?;
        let (removed_runs, changed_runs) = plan_change(&self.progress.effective_plan, plan);
        Ok(LeaveOutClosure {
            removed_runs,
            changed_runs,
            plan_sha256: plan_digest(plan).map_err(|error| StepFailure::new(error.to_string()))?,
        })
    }

    /// The stop of a step whose artifact cannot be used and that the collection cannot do
    /// without: a tool, or a part the recipe always installs.
    fn cannot_do_without(
        &self,
        artifact_id: &str,
        reason: &UnusableReason,
        failure: Option<StepFailure>,
    ) -> StepFailure {
        let mut message = format!(
            "{} can't be installed ({}), and the collection can't be installed without it. Try again later; if it keeps happening, please report it.\n{}",
            self.dependencies.artifact_name(artifact_id),
            reason.short(),
            nonempty_detail(&reason.detail)
        );
        if let Some(failure) = failure {
            message.push('\n');
            message.push_str(&failure.into_message());
        }
        StepFailure::new(message)
    }

    /// The question whether to continue without the payload of `task`, whose files cannot be
    /// placed, when none of its files or those of the payloads leaving with it are placed yet.
    /// Otherwise this copy needs a fresh start, because a mod that is not installed must not
    /// leave files behind: other mods detect mods by their files. A part the recipe cannot do
    /// without needs a fresh start too.
    fn leave_out_at_materialization(
        &mut self,
        task: &MaterializationTask,
        reason: UnusableReason,
    ) -> SimpleExecution {
        let detail = nonempty_detail(&reason.detail);
        let earlier = self.progress.left_out_ids();
        let mut ids = earlier.clone();
        ids.push(task.artifact_id.clone());
        let plan = match self.dependencies.plan_without(&self.request.plan, &ids) {
            Ok(plan) => plan,
            Err(failure) => {
                return SimpleExecution::FreshCopyRequired(format!("{detail}\n{failure}"));
            }
        };
        let placed = placed_payloads_left_behind(
            &self.schedule[..self.progress.completed_prefix],
            &earlier,
            &task.artifact_id,
            &plan,
        );
        if !placed.is_empty() {
            return SimpleExecution::FreshCopyRequired(format!(
                "{detail}\nThe installation cannot leave it out at this point, because files of {} are already in this game copy.",
                placed.join(", ")
            ));
        }
        let leave_out = match self.closure(&plan, &ids) {
            Ok(leave_out) => leave_out,
            Err(failure) => {
                return SimpleExecution::FreshCopyRequired(format!("{detail}\n{failure}"));
            }
        };
        let Some(identity) = self
            .request
            .created
            .artifact_identities
            .iter()
            .find(|identity| identity.id == task.artifact_id)
        else {
            return SimpleExecution::FreshCopyRequired(format!(
                "{detail}\nThe payload {} has no frozen identity.",
                task.artifact_id
            ));
        };
        SimpleExecution::Ask(Box::new(Question {
            kind: QuestionKind::UnusableDownload,
            artifact_id: task.artifact_id.clone(),
            artifact_kind: ArtifactKind::Payload,
            reviewed: identity.clone(),
            served: None,
            reason: Some(reason),
            leave_out: Some(leave_out),
            failed: None,
        }))
    }

    /// Asks the player about `run`, whose installer did not install `not_installed` in this
    /// attempt, or in `evidence`, the earlier one that ran it: whether to go on without what
    /// failed and everything that needs it, when the recipe can do without it, and whether to
    /// try again, when nothing was installed after it. A part the recipe cannot do without that
    /// cannot be tried again either leaves no answer: this copy needs a fresh start.
    fn ask_failed_component(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        run: &PlannedRun,
        not_installed: &[u32],
        installed_after: &[u32],
        evidence: Option<u32>,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        let reviewed = self
            .request
            .created
            .artifact_identities
            .iter()
            .find(|identity| identity.id == run.artifact_id)
            .cloned()
            .ok_or_else(|| {
                OrchestratorError::InvalidCampaign(format!(
                    "run {:?} installs {:?}, which has no frozen identity",
                    run.run_id, run.artifact_id
                ))
            })?;
        let (leave_out, continues_with, required) =
            match self.plan_without_failed(run, not_installed, installed_after.is_empty()) {
                Ok((plan, continues)) => {
                    let closure =
                        self.closure(&plan, &self.progress.left_out_ids())
                            .map_err(|failure| {
                                OrchestratorError::InvalidCampaign(failure.into_message())
                            })?;
                    (Some(closure), continues, None)
                }
                Err(why_not) => (None, Vec::new(), Some(why_not)),
            };
        let question = Question {
            kind: QuestionKind::FailedComponent,
            artifact_id: run.artifact_id.clone(),
            artifact_kind: ArtifactKind::Payload,
            reviewed,
            served: None,
            reason: None,
            leave_out,
            failed: Some(FailedPart {
                run_id: run.run_id.clone(),
                components: not_installed.to_vec(),
                installed_after: installed_after.to_vec(),
                continues_with,
                required: required.clone(),
                error: self.dependencies.failure_message(
                    &evidence
                        .map_or_else(|| attempt.clone(), |evidence| self.attempt(step, evidence)),
                ),
                evidence_attempt: evidence,
            }),
        };
        if let (Some(why_not), false) = (required, installed_after.is_empty()) {
            let reason = format!(
                "{} {}. Its installer went on after the error, so this copy cannot try it again.",
                self.dependencies.describe_question(&question),
                why_not.trim_end_matches('.')
            );
            return self.seal_fresh_copy(step, attempt, reason).map(Some);
        }
        if let Some(evidence) = evidence {
            self.progress
                .evidence_attempts
                .insert(step.id.clone(), evidence);
        }
        self.ask(step, attempt, question)
    }

    /// The question that `attempt` was about to ask again after a stop, when a crash came
    /// before it was recorded: `attempt` left no evidence, so it ran nothing, and the attempt
    /// that ran the installer still shows that it went past a failure.
    fn question_left_unasked(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        run: &PlannedRun,
    ) -> Result<Option<UnaskedQuestion>, StepFailure> {
        let Some(failed) = self.progress.last_failed.get(&step.id).copied() else {
            return Ok(None);
        };
        if self.dependencies.attempt_left_evidence(attempt)? {
            return Ok(None);
        }
        let ran = self
            .progress
            .evidence_attempts
            .get(&step.id)
            .copied()
            .unwrap_or(failed);
        let evidence = self.attempt(step, ran);
        Ok(match self.dependencies.recover(run, &evidence)? {
            InstallReconciliation::FailedAmong {
                failed,
                installed_after,
            } => Some(UnaskedQuestion {
                ran,
                failed,
                installed_after,
            }),
            _ => None,
        })
    }

    /// The plan that remains without the failed part of `run`, and the components of
    /// `not_installed` that the run then still installs. When the run `goes_on`, the first of
    /// `not_installed` failed and the others wait; otherwise all of them failed. Gives the
    /// reason instead when the collection cannot do without them, or when that would change
    /// what is installed.
    fn plan_without_failed(
        &mut self,
        run: &PlannedRun,
        not_installed: &[u32],
        goes_on: bool,
    ) -> Result<(InstallPlan, Vec<u32>), String> {
        let ids = self.progress.left_out_ids();
        let mut parts = self.progress.failed_parts.clone();
        parts.push((
            run.run_id.clone(),
            if goes_on {
                vec![not_installed[0]]
            } else {
                not_installed.to_vec()
            },
        ));
        let installed = self.progress.installed_runs(&self.schedule);
        let mut in_game = self
            .progress
            .effective_plan
            .runs
            .iter()
            .filter(|before| installed.contains(&before.run_id))
            .map(|before| (before.run_id.clone(), before.components.clone()))
            .collect::<Vec<_>>();
        in_game.push((
            run.run_id.clone(),
            run.components
                .iter()
                .copied()
                .filter(|component| !not_installed.contains(component))
                .collect(),
        ));
        let reduced = self
            .dependencies
            .plan_without_parts(&self.request.plan, &ids, &parts, &in_game)
            .map_err(StepFailure::into_message)?;
        plan_keeping_installed(
            &self.progress.effective_plan,
            reduced,
            &installed,
            &ids,
            run,
            not_installed,
            goes_on,
        )
    }

    /// Ends this attempt of an install step whose next attempt follows at once, because the
    /// attempt's evidence proves what it may run. The ledger records it as a failed attempt;
    /// the installation is not stopping, so the player sees no failure.
    fn next_attempt_follows(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        detail: &str,
    ) -> Result<(), OrchestratorError> {
        self.store.append(SessionEvent::StepFailed {
            step_id: step.id.clone(),
            attempt: attempt.attempt,
            detail: detail.to_owned(),
        })?;
        self.progress.unresolved = None;
        self.progress.pending = None;
        self.progress
            .last_failed
            .insert(step.id.clone(), attempt.attempt);
        self.log(step, format!("{}: {detail}", step.label));
        Ok(())
    }

    /// Leaves the failed component of an answered question out with exactly the closure
    /// recorded when it was asked; nothing derives it again. Its step ends, unless its run has
    /// components left that do not need the failed one: then the step goes on with those.
    fn leave_out_failed_component(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        run: &PlannedRun,
        pending: &PendingDecision,
    ) -> Result<(), OrchestratorError> {
        let question = &pending.question;
        let invalid = |what: &str| {
            OrchestratorError::InvalidCampaign(format!(
                "question {} has no {what} to leave its component out with",
                pending.decision_id
            ))
        };
        let closure = question
            .leave_out
            .as_ref()
            .ok_or_else(|| invalid("closure"))?;
        let part = question
            .failed
            .as_ref()
            .ok_or_else(|| invalid("failed part"))?;
        let failed = part.left_out();
        if failed.is_empty() {
            return Err(invalid("failed component"));
        }
        let plan = apply_plan_change(
            &self.progress.effective_plan,
            &closure.removed_runs,
            &closure.changed_runs,
        )
        .map_err(|detail| {
            OrchestratorError::InvalidCampaign(format!(
                "the closure recorded with question {} does not fit the plan: {detail}",
                pending.decision_id
            ))
        })?;
        if plan_digest(&plan)? != closure.plan_sha256 {
            return Err(OrchestratorError::InvalidCampaign(format!(
                "the plan without components {failed:?} of {:?} differs from the one recorded with question {}",
                run.run_id, pending.decision_id
            )));
        }
        let detail = format!(
            "its installer stopped with an error at components {failed:?} of run {}, and you chose to leave them out",
            run.run_id
        );
        self.store.append(SessionEvent::ArtifactLeftOut {
            step_id: step.id.clone(),
            attempt: attempt.attempt,
            artifact_id: run.artifact_id.clone(),
            detail: detail.clone(),
            plan_sha256: closure.plan_sha256.clone(),
            removed_runs: closure.removed_runs.clone(),
            changed_runs: closure.changed_runs.clone(),
            failed_run: Some(crate::session::FailedRun {
                run_id: run.run_id.clone(),
                components: part.components.clone(),
                left_out: failed.clone(),
                continues_with: part.continues_with.clone(),
            }),
        })?;
        let left_out = self
            .progress
            .record_failed_run(&mut self.schedule, run, &detail, failed, plan)
            .clone();
        self.progress
            .finish_failed_run(&step.id, &part.continues_with);
        self.log(
            step,
            format!(
                "The installation continues without it and without what needs it: {}",
                self.dependencies.describe_question(question)
            ),
        );
        self.sink.emit(EngineEvent::ArtifactLeftOut {
            artifact_id: left_out.artifact_id,
            reason: left_out.reason,
            runs: left_out.runs,
            changed_runs: left_out.changed_runs,
            failed_run: left_out.failed_run,
        });
        if part.continues_with.is_empty() {
            emit_finished(self.sink, &step.id, StepOutcome::Skipped);
        }
        Ok(())
    }

    /// Records `question` for this step, then goes on as its answer allows: at once with a
    /// preset answer, else the installation waits for the player.
    fn ask(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        question: Question,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        let decision_id = question.decision_id(&step.id, attempt.attempt);
        // The same question came back after it was withdrawn in this attempt, as when a link
        // alternates between two files: this attempt ends instead of asking in a loop.
        let withdrawn_before = self.store.replay()?.records.iter().any(|record| {
            matches!(
                &record.event,
                SessionEvent::DecisionWithdrawn { decision_id: withdrawn, .. }
                    if *withdrawn == decision_id
            )
        });
        if withdrawn_before {
            return self.failed(
                step,
                attempt,
                format!(
                    "The same question came back after it no longer applied: {} Its link keeps changing; try again later.",
                    self.dependencies.describe_question(&question)
                ),
            );
        }
        self.store.append(SessionEvent::DecisionRequired {
            step_id: step.id.clone(),
            attempt: attempt.attempt,
            decision_id: decision_id.clone(),
            question: Box::new(question.clone()),
        })?;
        self.log(
            step,
            format!(
                "Question {decision_id}: {}",
                self.dependencies.describe_question(&question)
            ),
        );
        let pending = PendingDecision {
            step_id: step.id.clone(),
            attempt: attempt.attempt,
            decision_id,
            question,
            answer: None,
        };
        self.progress.pending = Some(pending.clone());
        self.continue_decision(step, attempt, pending)
    }

    /// Applies the answer to the standing question of this step, once there is one. Without
    /// an answer the installation stops again with the same question: nothing is downloaded
    /// or computed again.
    fn continue_decision(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        mut pending: PendingDecision,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        let waiting = |pending: &PendingDecision| {
            Ok(Some(CampaignOutcome::DecisionRequired {
                step_id: pending.step_id.clone(),
                decision_id: pending.decision_id.clone(),
            }))
        };
        let answer = match pending.answer {
            Some(recorded) => recorded.answer,
            None => {
                let found = match self.dependencies.decision_answer(&pending) {
                    Ok(found) => found,
                    Err(failure) => {
                        // An answer that cannot be used ends this attempt; the next one asks
                        // anew, with a new id.
                        self.withdraw(
                            step,
                            attempt,
                            &pending,
                            &format!("its answer cannot be used: {failure}"),
                        )?;
                        return self.failed(step, attempt, failure.into_message());
                    }
                };
                let Some((answer, source)) = found else {
                    return waiting(&pending);
                };
                if !pending.question.offers(answer) {
                    self.log(
                        step,
                        format!(
                            "The answer {} does not fit question {}; waiting for another answer.",
                            answer.flag_value(),
                            pending.decision_id
                        ),
                    );
                    return waiting(&pending);
                }
                let record = self.store.append(SessionEvent::DecisionAnswered {
                    step_id: step.id.clone(),
                    attempt: attempt.attempt,
                    decision_id: pending.decision_id.clone(),
                    answer,
                    source,
                })?;
                self.log(
                    step,
                    format!(
                        "Answer to question {}: {} ({}).",
                        pending.decision_id,
                        answer.flag_value(),
                        match source {
                            AnswerSource::Player => "given by the player",
                            AnswerSource::Preset => "preset for this run",
                        }
                    ),
                );
                pending.answer = Some(RecordedAnswer {
                    answer,
                    source,
                    answered_at: record.recorded_at,
                });
                self.progress.pending = Some(pending.clone());
                answer
            }
        };
        if answer == Answer::No {
            let reason = format!(
                "You chose to stop: {}",
                self.dependencies.describe_question(&pending.question)
            );
            return self.failed(step, attempt, reason);
        }
        if pending.question.kind == QuestionKind::FailedComponent {
            let PipelineKind::Install(run) = &step.kind else {
                return Err(OrchestratorError::InvalidCampaign(format!(
                    "question {} about a failed component stands at step {:?}, which installs nothing",
                    pending.decision_id, step.id
                )));
            };
            return if answer == Answer::Retry {
                self.next_attempt_follows(step, attempt, "you chose to try again")
                    .map(|()| None)
            } else {
                self.leave_out_failed_component(step, attempt, run, &pending)
                    .map(|()| None)
            };
        }
        if pending.question.kind == QuestionKind::ModdedSource {
            let PipelineKind::Stage(role) = step.kind else {
                return Err(OrchestratorError::InvalidCampaign(format!(
                    "question {} about a source game stands at step {:?}, which copies none",
                    pending.decision_id, step.id
                )));
            };
            let execution = self.stage_source(step, role);
            return self.finish_simple(step, attempt, execution);
        }
        if pending.question.leaves_out(answer) {
            return self.apply_leave_out(step, attempt, pending);
        }
        self.apply_install_anyway(step, attempt, pending)
    }

    /// Leaves the payload of an answered question out with exactly the closure recorded when it
    /// was asked; nothing derives it again. A placing step plans again first: when its files
    /// can be placed now, the question is withdrawn and the mod is installed.
    fn apply_leave_out(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        pending: PendingDecision,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        let question = &pending.question;
        let closure = question.leave_out.clone().ok_or_else(|| {
            OrchestratorError::InvalidCampaign(format!(
                "question {} has no closure to leave its mod out with",
                pending.decision_id
            ))
        })?;
        let at_materialization = match &step.kind {
            // The reviewed archive may have come back to the download folder meanwhile, as when
            // another installation downloaded it: then the mod is installed after all. This
            // looks only into the download folder; nothing asks the link.
            PipelineKind::Acquire(identity, kind) => {
                match self.dependencies.reviewed_is_available(identity, *kind) {
                    Ok(false) => false,
                    Ok(true) => {
                        self.withdraw(
                            step,
                            attempt,
                            &pending,
                            "the reviewed version is in the download folder now",
                        )?;
                        let execution = self.acquire_execution(identity, *kind);
                        return self.finish_simple(step, attempt, execution);
                    }
                    Err(failure) => return self.failed(step, attempt, failure.into_message()),
                }
            }
            PipelineKind::Materialize(task) => {
                let planned = self
                    .recheck(step, task.target, MutationKind::Materialize)
                    .and_then(|()| self.dependencies.materialize(task));
                match planned {
                    Ok(MaterializationOutcome::LeaveOut { reason }) => {
                        let current = Question {
                            reason: Some(reason.clone()),
                            ..question.clone()
                        };
                        if current.decision_id(&step.id, attempt.attempt) != pending.decision_id {
                            self.withdraw(
                                step,
                                attempt,
                                &pending,
                                &format!(
                                    "its files now cannot be placed for another reason: {}",
                                    reason.detail
                                ),
                            )?;
                            let execution = self.leave_out_at_materialization(task, reason);
                            return self.finish_simple(step, attempt, Ok(execution));
                        }
                    }
                    Ok(MaterializationOutcome::Complete) => {
                        self.withdraw(
                            step,
                            attempt,
                            &pending,
                            "its files could be placed after all",
                        )?;
                        self.complete_step(step, attempt)?;
                        return Ok(None);
                    }
                    Ok(MaterializationOutcome::FreshCopyRequired { reason }) => {
                        self.withdraw(
                            step,
                            attempt,
                            &pending,
                            "this game copy needs a fresh start",
                        )?;
                        return self.seal_fresh_copy(step, attempt, reason).map(Some);
                    }
                    Err(failure) => return self.failed(step, attempt, failure.into_message()),
                }
                true
            }
            _ => {
                return Err(OrchestratorError::InvalidCampaign(format!(
                    "question {} cannot leave a mod out at step {:?}",
                    pending.decision_id, step.id
                )))
            }
        };
        let artifact_id = question.artifact_id.clone();
        let plan = apply_plan_change(
            &self.progress.effective_plan,
            &closure.removed_runs,
            &closure.changed_runs,
        )
        .map_err(|detail| {
            OrchestratorError::InvalidCampaign(format!(
                "the closure recorded with question {} does not fit the plan: {detail}",
                pending.decision_id
            ))
        })?;
        if plan_digest(&plan)? != closure.plan_sha256 {
            return Err(OrchestratorError::InvalidCampaign(format!(
                "the plan without {artifact_id:?} differs from the one recorded with question {}",
                pending.decision_id
            )));
        }
        if at_materialization {
            let placed = placed_payloads_left_behind(
                &self.schedule[..self.progress.completed_prefix],
                &self.progress.left_out_ids(),
                &artifact_id,
                &plan,
            );
            if !placed.is_empty() {
                self.withdraw(
                    step,
                    attempt,
                    &pending,
                    &format!(
                        "files of {} are already in this game copy",
                        placed.join(", ")
                    ),
                )?;
                return self
                    .seal_fresh_copy(
                        step,
                        attempt,
                        format!(
                            "The installation cannot leave {artifact_id} out at this point, because files of {} are already in this game copy.",
                            placed.join(", ")
                        ),
                    )
                    .map(Some);
            }
        }
        let detail = match (&question.reason, &question.served) {
            (Some(reason), _) => nonempty_detail(&reason.detail),
            (None, Some(served)) => format!(
                "its official link now serves a different file ({}), and you chose to leave it out",
                describe_served(served)
            ),
            (None, None) => "you chose to leave it out".to_owned(),
        };
        self.leave_out(step, attempt, &artifact_id, &detail, plan)?;
        Ok(None)
    }

    /// Installs the changed download of an answered question: exactly the file asked about.
    fn apply_install_anyway(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        pending: PendingDecision,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        let PipelineKind::Acquire(identity, kind) = &step.kind else {
            return Err(OrchestratorError::InvalidCampaign(format!(
                "question {} cannot install a download at step {:?}",
                pending.decision_id, step.id
            )));
        };
        match self
            .dependencies
            .acquire_answered(identity, *kind, &pending.question)
        {
            Ok(AnsweredAcquisition::Accepted) => {
                self.complete_step(step, attempt)?;
                Ok(None)
            }
            Ok(AnsweredAcquisition::Withdrawn { why }) => {
                self.withdraw(step, attempt, &pending, &why)?;
                let execution = self.acquire_execution(identity, *kind);
                self.finish_simple(step, attempt, execution)
            }
            Ok(AnsweredAcquisition::LinkChanged { detail }) => self.failed(step, attempt, detail),
            Err(failure) => self.failed(step, attempt, failure.into_message()),
        }
    }

    /// Copies the source game of `role` after the immediate safety recheck.
    fn stage_source(
        &mut self,
        step: &PipelineStep,
        role: GameRole,
    ) -> Result<SimpleExecution, StepFailure> {
        self.recheck(step, role_target(role), MutationKind::Stage)
            .and_then(|()| self.dependencies.stage(role))
            .map(|()| SimpleExecution::Complete)
    }

    /// Whether the player already answered Yes to this staging step's question about its
    /// source game, in any attempt.
    fn source_answered_yes(&self, step: &PipelineStep) -> Result<bool, OrchestratorError> {
        let records = self.store.replay()?.records;
        let asked = records
            .iter()
            .filter_map(|record| match &record.event {
                SessionEvent::DecisionRequired {
                    step_id,
                    decision_id,
                    question,
                    ..
                } if *step_id == step.id && question.kind == QuestionKind::ModdedSource => {
                    Some(decision_id.clone())
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        Ok(records.iter().any(|record| {
            matches!(
                &record.event,
                SessionEvent::DecisionAnswered { decision_id, answer: Answer::Yes, .. }
                    if asked.contains(decision_id)
            )
        }))
    }

    /// Withdraws the standing question of this step: the situation it asked about changed.
    fn withdraw(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        pending: &PendingDecision,
        why: &str,
    ) -> Result<(), OrchestratorError> {
        let why = nonempty_detail(why);
        self.store.append(SessionEvent::DecisionWithdrawn {
            step_id: step.id.clone(),
            attempt: attempt.attempt,
            decision_id: pending.decision_id.clone(),
            why: why.clone(),
        })?;
        self.progress.pending = None;
        self.log(
            step,
            format!("Question {} no longer applies: {why}", pending.decision_id),
        );
        Ok(())
    }

    /// A technical-log line on `step`.
    fn log(&self, step: &PipelineStep, line: String) {
        self.sink.emit(EngineEvent::ConsoleLine {
            step_id: step.id.clone(),
            stream: crate::events::Stream::Stdout,
            line,
        });
    }

    /// Fails this attempt of `step` with `reason`.
    fn failed(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        reason: String,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        self.fail_step(step, attempt, &reason)?;
        Ok(Some(CampaignOutcome::Failed {
            step_id: step.id.clone(),
            reason,
        }))
    }

    /// Durably ends an acquisition or placing step by leaving its payload out, then continues
    /// with the remaining plan. The ledger records which runs that removed or changed, and a
    /// resume rebuilds the plan from that record.
    fn leave_out(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        artifact_id: &str,
        reason: &str,
        plan: InstallPlan,
    ) -> Result<(), OrchestratorError> {
        let mut ids = self.progress.left_out_ids();
        ids.push(artifact_id.to_owned());
        validate_reduced_plan(&self.progress.effective_plan, &plan, &ids)?;
        let (removed_runs, changed_runs) = plan_change(&self.progress.effective_plan, &plan);
        self.store.append(SessionEvent::ArtifactLeftOut {
            step_id: step.id.clone(),
            attempt: attempt.attempt,
            artifact_id: artifact_id.to_owned(),
            detail: reason.to_owned(),
            plan_sha256: plan_digest(&plan)?,
            removed_runs,
            changed_runs,
            failed_run: None,
        })?;
        let left_out = self
            .progress
            .record_left_out(&mut self.schedule, artifact_id, reason, plan)
            .clone();
        self.progress.unresolved = None;
        self.progress.pending = None;
        self.progress.last_failed.remove(&step.id);
        self.progress.completed_prefix += 1;
        self.sink.emit(EngineEvent::ArtifactLeftOut {
            artifact_id: left_out.artifact_id,
            reason: left_out.reason,
            runs: left_out.runs,
            changed_runs: left_out.changed_runs,
            failed_run: left_out.failed_run,
        });
        emit_finished(self.sink, &step.id, StepOutcome::Skipped);
        Ok(())
    }

    fn run_install(
        &mut self,
        step: &PipelineStep,
        run: PlannedRun,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        if self.active_phase != Some(run.phase) {
            self.sink.emit(EngineEvent::PhaseStarted {
                name: friendly_phase(run.phase).to_owned(),
            });
            self.active_phase = Some(run.phase);
        }

        // A standing question comes first: the run's evidence was reconciled before it was
        // asked, and nothing runs until it is answered.
        if let Some(pending) = self.progress.pending.clone() {
            if pending.step_id != step.id {
                return Err(OrchestratorError::InvalidCampaign(format!(
                    "question {} belongs to {:?}, not to the next step {:?}",
                    pending.decision_id, pending.step_id, step.id
                )));
            }
            let attempt = self.attempt(step, pending.attempt);
            emit_started(self.sink, &step.id, &format!("Resume {}", step.label));
            return self.continue_decision(step, &attempt, pending);
        }

        let mut components = match &self.progress.continuing {
            Some((step_id, components)) if *step_id == step.id => components.clone(),
            _ => run.components.clone(),
        };
        let mut current = if let Some((unresolved_id, attempt)) = &self.progress.unresolved {
            if unresolved_id != &step.id {
                return Err(OrchestratorError::InvalidCampaign(format!(
                    "unresolved step {unresolved_id:?} is not the next pipeline step {:?}",
                    step.id
                )));
            }
            let attempt = self.attempt(step, *attempt);
            emit_started(self.sink, &step.id, &format!("Resume {}", step.label));
            match self.question_left_unasked(step, &attempt, &run) {
                Ok(Some(question)) => {
                    return self.ask_failed_component(
                        step,
                        &attempt,
                        &run,
                        &question.failed,
                        &question.installed_after,
                        Some(question.ran),
                    );
                }
                Ok(None) => {}
                Err(failure) => {
                    let reason = format!("could not reconcile interrupted evidence: {failure}");
                    return self.seal_fresh_copy(step, &attempt, reason).map(Some);
                }
            }
            match self.dependencies.recover(&run, &attempt) {
                Ok(InstallReconciliation::ProvenDone) => {
                    if let Some(outcome) = self.postcondition_failure(step, &run, &attempt)? {
                        return Ok(Some(outcome));
                    }
                    self.complete_step(step, &attempt)?;
                    return Ok(None);
                }
                Ok(InstallReconciliation::Retry { remaining })
                | Ok(InstallReconciliation::PartialPrefix { remaining }) => {
                    components = validate_remaining(&run, &remaining, None)?;
                    self.fail_step(
                        step,
                        &attempt,
                        "interrupted attempt reconciled; continuing its exact remaining suffix",
                    )?;
                    self.begin_new(step)?
                }
                Ok(InstallReconciliation::FailedAmong {
                    failed,
                    installed_after,
                }) => {
                    return self.ask_failed_component(
                        step,
                        &attempt,
                        &run,
                        &failed,
                        &installed_after,
                        None,
                    );
                }
                Ok(InstallReconciliation::FreshCopyRequired { reason }) => {
                    return self.seal_fresh_copy(step, &attempt, reason).map(Some);
                }
                Err(failure) => {
                    let reason = format!("could not reconcile interrupted evidence: {failure}");
                    return self.seal_fresh_copy(step, &attempt, reason).map(Some);
                }
            }
        } else if let Some(attempt) = self.progress.last_failed.get(&step.id).copied() {
            // The ledger can only seal the attempt that failed last, even when that one only
            // asked and the evidence is another attempt's.
            let failed_attempt = self.attempt(step, attempt);
            let attempt = self
                .progress
                .evidence_attempts
                .get(&step.id)
                .copied()
                .unwrap_or(attempt);
            let evidence_attempt = self.attempt(step, attempt);
            match self.dependencies.recover(&run, &evidence_attempt) {
                Ok(InstallReconciliation::ProvenDone) => {
                    if let Some(outcome) =
                        self.postcondition_failure(step, &run, &failed_attempt)?
                    {
                        return Ok(Some(outcome));
                    }
                    let reconciliation = self.begin_new(step)?;
                    self.complete_step(step, &reconciliation)?;
                    return Ok(None);
                }
                Ok(InstallReconciliation::Retry { remaining })
                | Ok(InstallReconciliation::PartialPrefix { remaining }) => {
                    components = validate_remaining(&run, &remaining, None)?;
                    self.begin_new(step)?
                }
                // The player stopped at this failure before: nothing runs, the question stands
                // again. An attempt that only asks keeps pointing at the one that ran.
                Ok(InstallReconciliation::FailedAmong {
                    failed,
                    installed_after,
                }) => {
                    let asking = self.begin_new(step)?;
                    return self.ask_failed_component(
                        step,
                        &asking,
                        &run,
                        &failed,
                        &installed_after,
                        Some(evidence_attempt.attempt),
                    );
                }
                Ok(InstallReconciliation::FreshCopyRequired { reason }) => {
                    return self
                        .seal_fresh_copy(step, &failed_attempt, reason)
                        .map(Some);
                }
                Err(failure) => {
                    let reason = format!("could not reconcile failed attempt evidence: {failure}");
                    return self
                        .seal_fresh_copy(step, &failed_attempt, reason)
                        .map(Some);
                }
            }
        } else {
            self.begin_new(step)?
        };

        loop {
            let execution = self.execute_install_attempt(step, &run, &components, &current);
            let reconciliation = match execution {
                Ok(reconciliation) => reconciliation,
                Err(failure) => {
                    let reason = failure.into_message();
                    self.fail_step(step, &current, &reason)?;
                    return Ok(Some(CampaignOutcome::Failed {
                        step_id: step.id.clone(),
                        reason,
                    }));
                }
            };
            match reconciliation {
                InstallReconciliation::ProvenDone => {
                    if let Some(outcome) = self.postcondition_failure(step, &run, &current)? {
                        return Ok(Some(outcome));
                    }
                    self.complete_step(step, &current)?;
                    return Ok(None);
                }
                InstallReconciliation::Retry { remaining } => {
                    let remaining = validate_remaining(&run, &remaining, Some(&components))?;
                    if remaining != components {
                        let reason =
                            "WeiDU made no durable change; the exact run remains retryable";
                        self.fail_step(step, &current, reason)?;
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "retry verdict for {:?} changed the requested suffix",
                            step.id
                        )));
                    }
                    // The installer stopped at the first of `remaining` and left it, and those
                    // after it, uninstalled. The player decides how the installation goes on.
                    return self.ask_failed_component(step, &current, &run, &remaining, &[], None);
                }
                InstallReconciliation::FailedAmong {
                    failed,
                    installed_after,
                } => {
                    return self.ask_failed_component(
                        step,
                        &current,
                        &run,
                        &failed,
                        &installed_after,
                        None,
                    );
                }
                InstallReconciliation::PartialPrefix { remaining } => {
                    let remaining = validate_remaining(&run, &remaining, Some(&components))?;
                    if remaining.len() >= components.len() {
                        let reason = "partial-prefix evidence did not reduce the remaining suffix";
                        return self
                            .seal_fresh_copy(step, &current, reason.to_owned())
                            .map(Some);
                    }
                    self.next_attempt_follows(
                        step,
                        &current,
                        "strict component prefix verified; continuing only the exact suffix",
                    )?;
                    components = remaining;
                    current = self.begin_new(step)?;
                }
                InstallReconciliation::FreshCopyRequired { reason } => {
                    return self.seal_fresh_copy(step, &current, reason).map(Some);
                }
            }
        }
    }

    fn execute_install_attempt(
        &mut self,
        step: &PipelineStep,
        run: &PlannedRun,
        components: &[u32],
        attempt: &StepAttempt,
    ) -> Result<InstallReconciliation, StepFailure> {
        self.dependencies.snapshot_before(run, attempt)?;
        self.recheck_install_before_spawn(
            step,
            run,
            components,
            attempt,
            MutationKind::InvocationBuild,
        )?;
        let built = match self.dependencies.build(run, components, attempt) {
            Ok(built) => built,
            Err(failure) if failure.before_preparation => {
                return Err(self.pre_spawn_failure(
                    run,
                    components,
                    attempt,
                    MutationKind::InvocationBuild,
                    failure,
                ));
            }
            Err(failure) => return Err(failure),
        };
        if built.identity_digest.trim().is_empty() {
            return Err(StepFailure::new("invocation identity digest is empty"));
        }
        self.dependencies
            .record_invocation(run, attempt, &built.identity_digest)?;
        self.recheck_install_before_spawn(
            step,
            run,
            components,
            attempt,
            MutationKind::ProcessSpawn,
        )?;
        let result = self.dependencies.run(built.invocation, attempt)?;
        self.dependencies.sync_and_reconcile(run, attempt, result)
    }

    fn recheck_install_before_spawn(
        &mut self,
        step: &PipelineStep,
        run: &PlannedRun,
        components: &[u32],
        attempt: &StepAttempt,
        kind: MutationKind,
    ) -> Result<(), StepFailure> {
        self.recheck(step, run.target, kind)
            .map_err(|failure| self.pre_spawn_failure(run, components, attempt, kind, failure))
    }

    /// Records durable evidence that `failure` happened before WeiDU could start and binds that
    /// record into the ledger failure, so that retrying can prove the run never started.
    fn pre_spawn_failure(
        &mut self,
        run: &PlannedRun,
        components: &[u32],
        attempt: &StepAttempt,
        kind: MutationKind,
        failure: StepFailure,
    ) -> StepFailure {
        match self
            .dependencies
            .record_pre_spawn_failure(run, components, attempt, kind, &failure)
        {
            Ok(digest) => {
                StepFailure::new(format!("{failure}\nPre-spawn evidence SHA-256: {digest}"))
            }
            Err(error) => error,
        }
    }

    fn postcondition_failure(
        &mut self,
        step: &PipelineStep,
        run: &PlannedRun,
        attempt: &StepAttempt,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        let root = match run.target {
            GameRoot::Bg1 => &self.request.created.staged_bg1,
            GameRoot::Bg2 => &self.request.created.staged_bg2,
        };
        let Err(error) = postcondition::verify(root, &run.postconditions) else {
            return Ok(None);
        };
        let reason = format!(
            "run {:?} postcondition verification failed after installed state was proven: {error}",
            run.run_id
        );
        self.seal_fresh_copy(step, attempt, reason).map(Some)
    }

    fn begin_or_resume(&mut self, step: &PipelineStep) -> Result<StepAttempt, OrchestratorError> {
        if let Some((unresolved_id, attempt)) = &self.progress.unresolved {
            if unresolved_id != &step.id {
                return Err(OrchestratorError::InvalidCampaign(format!(
                    "unresolved step {unresolved_id:?} is not the next pipeline step {:?}",
                    step.id
                )));
            }
            emit_started(self.sink, &step.id, &format!("Resume {}", step.label));
            return Ok(self.attempt(step, *attempt));
        }
        self.begin_new(step)
    }

    fn begin_new(&mut self, step: &PipelineStep) -> Result<StepAttempt, OrchestratorError> {
        let attempt = self.progress.next_attempt(&step.id)?;
        self.store.append(SessionEvent::StepStarted {
            step_id: step.id.clone(),
            attempt,
        })?;
        self.progress.attempts.insert(step.id.clone(), attempt);
        self.progress.unresolved = Some((step.id.clone(), attempt));
        self.progress.continuing = None;
        emit_started(self.sink, &step.id, &step.label);
        Ok(self.attempt(step, attempt))
    }

    fn complete_step(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
    ) -> Result<(), OrchestratorError> {
        self.store.append(SessionEvent::StepCompleted {
            step_id: step.id.clone(),
            attempt: attempt.attempt,
        })?;
        self.progress.unresolved = None;
        self.progress.pending = None;
        self.progress.last_failed.remove(&step.id);
        self.progress.completed_prefix += 1;
        emit_finished(self.sink, &step.id, StepOutcome::Succeeded);
        Ok(())
    }

    fn fail_step(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        reason: &str,
    ) -> Result<(), OrchestratorError> {
        self.store.append(SessionEvent::StepFailed {
            step_id: step.id.clone(),
            attempt: attempt.attempt,
            detail: nonempty_detail(reason),
        })?;
        self.progress.unresolved = None;
        self.progress.pending = None;
        self.progress
            .last_failed
            .insert(step.id.clone(), attempt.attempt);
        self.sink.emit(EngineEvent::Error {
            step_id: Some(step.id.clone()),
            message: nonempty_detail(reason),
        });
        emit_finished(self.sink, &step.id, StepOutcome::Failed);
        Ok(())
    }

    fn seal_fresh_copy(
        &mut self,
        step: &PipelineStep,
        attempt: &StepAttempt,
        reason: String,
    ) -> Result<CampaignOutcome, OrchestratorError> {
        self.store.append(SessionEvent::FreshCopyRequired {
            step_id: step.id.clone(),
            attempt: attempt.attempt,
            detail: nonempty_detail(&reason),
        })?;
        self.progress.unresolved = None;
        self.progress.pending = None;
        self.progress.last_failed.remove(&step.id);
        self.progress.fresh_copy_required = Some((step.id.clone(), reason.clone()));
        self.emit_fresh_copy(step, &reason);
        Ok(CampaignOutcome::FreshCopyRequired {
            step_id: step.id.clone(),
            reason,
        })
    }

    fn emit_fresh_copy(&self, step: &PipelineStep, reason: &str) {
        self.sink.emit(EngineEvent::Error {
            step_id: Some(step.id.clone()),
            message: format!(
                "A fresh managed copy is required: {}",
                nonempty_detail(reason)
            ),
        });
        emit_finished(self.sink, &step.id, StepOutcome::Failed);
    }

    fn recheck(
        &mut self,
        step: &PipelineStep,
        target: GameRoot,
        kind: MutationKind,
    ) -> Result<(), StepFailure> {
        self.dependencies.recheck_before_mutation(&MutationCheck {
            step_id: step.id.clone(),
            target,
            kind,
        })
    }

    fn attempt(&self, step: &PipelineStep, attempt: u32) -> StepAttempt {
        let safe_key = &sha256_bytes(step.id.as_bytes())[..16];
        StepAttempt {
            step_id: step.id.clone(),
            attempt,
            evidence_root: self
                .request
                .created
                .managed_root
                .join(".chriz")
                .join("attempts")
                .join(&self.request.created.attempt_id)
                .join("steps")
                .join(format!("{:04}-{safe_key}", self.progress.completed_prefix))
                .join(format!("attempt-{attempt:04}")),
        }
    }
}

enum SimpleExecution {
    Complete,
    FreshCopyRequired(String),
    /// The player is asked this question before the step goes on.
    Ask(Box<Question>),
}

/// "name, N bytes, SHA-256 x" of a served file, for the ledger.
fn describe_served(served: &ServedDownload) -> String {
    match &served.filename {
        Some(filename) => format!(
            "{filename}, {} bytes, SHA-256 {}",
            served.length, served.sha256
        ),
        None => format!("{} bytes, SHA-256 {}", served.length, served.sha256),
    }
}

fn validate_remaining(
    run: &PlannedRun,
    remaining: &[u32],
    attempted: Option<&[u32]>,
) -> Result<Vec<u32>, OrchestratorError> {
    if remaining.is_empty() || !run.components.ends_with(remaining) {
        return Err(OrchestratorError::InvalidCampaign(format!(
            "reconciliation for {:?} returned a non-suffix component set {remaining:?}",
            run.run_id
        )));
    }
    if let Some(attempted) = attempted {
        if !attempted.ends_with(remaining) {
            return Err(OrchestratorError::InvalidCampaign(format!(
                "reconciliation for {:?} escaped attempted suffix {attempted:?}",
                run.run_id
            )));
        }
    }
    Ok(remaining.to_vec())
}

fn open_or_create_campaign(
    request: &CampaignRequest,
    target_lock: &TargetLock,
) -> Result<(SessionStore, SessionReplay, bool), OrchestratorError> {
    let root = &request.created.managed_root;
    let existed = match fs::symlink_metadata(root) {
        Ok(metadata) => {
            ensure_direct_directory(root, &metadata)?;
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            claim_new_root(root)?;
            false
        }
        Err(source) => {
            return Err(OrchestratorError::UnsafeTarget {
                path: root.clone(),
                reason: source.to_string(),
            })
        }
    };
    let canonical = fs::canonicalize(root).map_err(|source| OrchestratorError::UnsafeTarget {
        path: root.clone(),
        reason: source.to_string(),
    })?;
    if !same_path(&canonical, target_lock.target()) {
        return Err(OrchestratorError::UnsafeTarget {
            path: root.clone(),
            reason: format!(
                "claimed root {} does not match target lock identity {}",
                canonical.display(),
                target_lock.target().display()
            ),
        });
    }

    let state_root = canonical.join(".chriz");
    let (store, resumed) = if state_root.exists() {
        validate_owned_top_level(&canonical)?;
        (SessionStore::open(&canonical)?, true)
    } else {
        if existed
            && fs::read_dir(&canonical)
                .map_err(|source| OrchestratorError::UnsafeTarget {
                    path: canonical.clone(),
                    reason: source.to_string(),
                })?
                .next()
                .is_some()
        {
            return Err(OrchestratorError::UnsafeTarget {
                path: canonical,
                reason: "existing unclaimed managed root is not empty".to_owned(),
            });
        }
        (
            SessionStore::create(
                &canonical,
                SessionEvent::Created(Box::new(request.created.clone())),
            )?,
            false,
        )
    };
    let replay = store.replay()?;
    replay.validate_resume(&request.created)?;
    Ok((store, replay, resumed))
}

fn claim_new_root(root: &Path) -> Result<(), OrchestratorError> {
    let parent = root
        .parent()
        .ok_or_else(|| OrchestratorError::UnsafeTarget {
            path: root.to_path_buf(),
            reason: "managed root has no parent".to_owned(),
        })?;
    let mut existing = parent;
    let mut missing = Vec::new();
    loop {
        match fs::symlink_metadata(existing) {
            Ok(metadata) => {
                ensure_direct_directory(existing, &metadata)?;
                ensure_direct_ancestors(existing)?;
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(existing.to_path_buf());
                existing = existing
                    .parent()
                    .ok_or_else(|| OrchestratorError::UnsafeTarget {
                        path: root.to_path_buf(),
                        reason: "managed root has no existing directory ancestor".to_owned(),
                    })?;
            }
            Err(source) => {
                return Err(OrchestratorError::UnsafeTarget {
                    path: existing.to_path_buf(),
                    reason: format!("could not inspect managed root ancestor: {source}"),
                })
            }
        }
    }

    for directory in missing.into_iter().rev() {
        let parent = directory
            .parent()
            .ok_or_else(|| OrchestratorError::UnsafeTarget {
                path: directory.clone(),
                reason: "managed root ancestor has no parent".to_owned(),
            })?;
        let parent_metadata =
            fs::symlink_metadata(parent).map_err(|source| OrchestratorError::UnsafeTarget {
                path: parent.to_path_buf(),
                reason: format!("could not recheck managed root ancestor: {source}"),
            })?;
        ensure_direct_directory(parent, &parent_metadata)?;
        ensure_direct_ancestors(parent)?;
        fs::create_dir(&directory).map_err(|source| OrchestratorError::UnsafeTarget {
            path: directory.clone(),
            reason: format!("could not create managed root ancestor: {source}"),
        })?;
        let metadata =
            fs::symlink_metadata(&directory).map_err(|source| OrchestratorError::UnsafeTarget {
                path: directory.clone(),
                reason: format!("could not inspect created managed root path: {source}"),
            })?;
        ensure_direct_directory(&directory, &metadata)?;
    }

    let parent_metadata =
        fs::symlink_metadata(parent).map_err(|source| OrchestratorError::UnsafeTarget {
            path: parent.to_path_buf(),
            reason: format!("could not recheck managed root parent: {source}"),
        })?;
    ensure_direct_directory(parent, &parent_metadata)?;
    ensure_direct_ancestors(parent)?;
    fs::create_dir(root).map_err(|source| OrchestratorError::UnsafeTarget {
        path: root.to_path_buf(),
        reason: format!("could not claim direct managed root: {source}"),
    })
}

fn validate_owned_top_level(root: &Path) -> Result<(), OrchestratorError> {
    for entry in fs::read_dir(root).map_err(|source| OrchestratorError::UnsafeTarget {
        path: root.to_path_buf(),
        reason: source.to_string(),
    })? {
        let entry = entry.map_err(|source| OrchestratorError::UnsafeTarget {
            path: root.to_path_buf(),
            reason: source.to_string(),
        })?;
        let name = entry.file_name();
        let allowed = [".chriz", "bg1", "game"]
            .iter()
            .any(|expected| os_name_eq(&name.to_string_lossy(), expected));
        if !allowed {
            return Err(OrchestratorError::UnsafeTarget {
                path: entry.path(),
                reason: "resumed target contains an unowned top-level entry".to_owned(),
            });
        }
        let metadata = fs::symlink_metadata(entry.path()).map_err(|source| {
            OrchestratorError::UnsafeTarget {
                path: entry.path(),
                reason: source.to_string(),
            }
        })?;
        ensure_direct_directory(&entry.path(), &metadata)?;
    }
    Ok(())
}

fn reject_protected_or_relative_target(root: &Path) -> Result<(), OrchestratorError> {
    if !root.is_absolute() {
        return Err(OrchestratorError::UnsafeTarget {
            path: root.to_path_buf(),
            reason: "managed root must be absolute".to_owned(),
        });
    }
    if is_creator_protected_destination(root) {
        return Err(OrchestratorError::UnsafeTarget {
            path: root.to_path_buf(),
            reason: "creator reference installations are read-only".to_owned(),
        });
    }
    Ok(())
}

fn reject_lock_registry_overlap(request: &CampaignRequest) -> Result<(), OrchestratorError> {
    if !request.registry_root.is_absolute() {
        return Err(OrchestratorError::UnsafeTarget {
            path: request.registry_root.clone(),
            reason: "target-lock registry must be absolute".to_owned(),
        });
    }
    let target = prospective_identity(&request.created.managed_root)?;
    let registry = prospective_identity(&request.registry_root)?;
    if paths_overlap(&target, &registry) {
        return Err(OrchestratorError::UnsafeTarget {
            path: request.registry_root.clone(),
            reason: "target-lock registry must be outside the managed target".to_owned(),
        });
    }
    Ok(())
}

fn prospective_identity(path: &Path) -> Result<PathBuf, OrchestratorError> {
    let absolute = std::path::absolute(path).map_err(|source| OrchestratorError::UnsafeTarget {
        path: path.to_path_buf(),
        reason: format!("could not resolve absolute path: {source}"),
    })?;
    let mut existing = absolute.as_path();
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(existing) {
            Ok(metadata) => {
                ensure_direct_directory(existing, &metadata)?;
                ensure_direct_ancestors(existing)?;
                let mut canonical = fs::canonicalize(existing).map_err(|source| {
                    OrchestratorError::UnsafeTarget {
                        path: existing.to_path_buf(),
                        reason: source.to_string(),
                    }
                })?;
                for component in suffix.iter().rev() {
                    canonical.push(component);
                }
                return Ok(canonical);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let name = existing
                    .file_name()
                    .ok_or_else(|| OrchestratorError::UnsafeTarget {
                        path: absolute.clone(),
                        reason: "path has no existing directory ancestor".to_owned(),
                    })?;
                suffix.push(name.to_os_string());
                existing = existing
                    .parent()
                    .ok_or_else(|| OrchestratorError::UnsafeTarget {
                        path: absolute.clone(),
                        reason: "path has no existing directory ancestor".to_owned(),
                    })?;
            }
            Err(source) => {
                return Err(OrchestratorError::UnsafeTarget {
                    path: existing.to_path_buf(),
                    reason: source.to_string(),
                })
            }
        }
    }
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    path_is_within(left, right) || path_is_within(right, left)
}

fn path_is_within(path: &Path, base: &Path) -> bool {
    let path = path.components().collect::<Vec<_>>();
    let base = base.components().collect::<Vec<_>>();
    path.len() >= base.len()
        && path
            .iter()
            .zip(base.iter())
            .all(|(left, right)| component_eq(left.as_os_str(), right.as_os_str()))
}

fn component_eq(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> bool {
    #[cfg(windows)]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn ensure_direct_ancestors(path: &Path) -> Result<(), OrchestratorError> {
    let mut current = Some(path);
    while let Some(candidate) = current {
        if let Ok(metadata) = fs::symlink_metadata(candidate) {
            if is_link_or_reparse(&metadata) {
                return Err(OrchestratorError::UnsafeTarget {
                    path: candidate.to_path_buf(),
                    reason: "symbolic links, junctions, and reparse points are forbidden"
                        .to_owned(),
                });
            }
        }
        current = candidate.parent();
    }
    Ok(())
}

fn ensure_direct_directory(path: &Path, metadata: &fs::Metadata) -> Result<(), OrchestratorError> {
    if is_link_or_reparse(metadata) || !metadata.is_dir() {
        return Err(OrchestratorError::UnsafeTarget {
            path: path.to_path_buf(),
            reason: "expected a direct non-reparse directory".to_owned(),
        });
    }
    Ok(())
}

fn is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink() || has_windows_reparse_attribute(metadata)
}

#[cfg(windows)]
fn has_windows_reparse_attribute(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn has_windows_reparse_attribute(_metadata: &fs::Metadata) -> bool {
    false
}

fn same_path(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn os_name_eq(left: &str, right: &str) -> bool {
    #[cfg(windows)]
    {
        left.eq_ignore_ascii_case(right)
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn validate_step_id(id: &str) -> Result<(), OrchestratorError> {
    if id.is_empty()
        || id.len() > 256
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'))
    {
        return Err(OrchestratorError::InvalidCampaign(format!(
            "pipeline step id {id:?} is not ledger-safe"
        )));
    }
    Ok(())
}

fn validate_step_fragment(label: &str, value: &str) -> Result<(), OrchestratorError> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(OrchestratorError::InvalidCampaign(format!(
            "{label} {value:?} is not safe in a pipeline id"
        )));
    }
    Ok(())
}

fn emit_started(sink: &impl EventSink, id: &str, label: &str) {
    sink.emit(EngineEvent::StepStarted {
        id: id.to_owned(),
        label: label.to_owned(),
    });
}

fn emit_finished(sink: &impl EventSink, id: &str, outcome: StepOutcome) {
    sink.emit(EngineEvent::StepFinished {
        id: id.to_owned(),
        outcome,
    });
}

fn nonempty_detail(detail: &str) -> String {
    if detail.trim().is_empty() {
        "campaign operation failed without detail".to_owned()
    } else {
        detail.to_owned()
    }
}

fn role_target(role: GameRole) -> GameRoot {
    match role {
        GameRole::BgeeSod => GameRoot::Bg1,
        GameRole::Bg2ee => GameRoot::Bg2,
    }
}

fn game_root_id(root: GameRoot) -> &'static str {
    match root {
        GameRoot::Bg1 => "bg1",
        GameRoot::Bg2 => "bg2",
    }
}

fn game_root_label(root: GameRoot) -> &'static str {
    match root {
        GameRoot::Bg1 => "Baldur's Gate: Enhanced Edition",
        GameRoot::Bg2 => "Baldur's Gate II: Enhanced Edition",
    }
}

fn friendly_phase(phase: Phase) -> &'static str {
    match phase {
        Phase::Bg1Preparation => "Prepare Baldur's Gate: Enhanced Edition",
        Phase::Bg2Preparation => "Prepare Baldur's Gate II: Enhanced Edition",
        Phase::EetInitialization => "Build the EET campaign",
        Phase::Main => "Install the curated collection",
        Phase::EetFinalization => "Finalize EET",
        Phase::PostEetEnd => "Apply reviewed final compatibility fixes",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ComponentRef;
    use crate::recipe_view::{
        evaluate, plan_without_parts, recipe_run_id, LeaveOutError, SelectionEvaluation,
    };
    use crate::resolve::Selection;
    use crate::Manifest;

    fn recommended(manifest: &Manifest, spell_revisions: bool) -> SelectionEvaluation {
        let mut selection = Selection {
            platform: "windows".to_owned(),
            choices: manifest.presets["chris-recommended"].selections.clone(),
        };
        selection.set_feature("mod:spell-rev", spell_revisions);
        evaluate(manifest, &selection).unwrap()
    }

    fn curated() -> Manifest {
        Manifest::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../recipes/curated-full-current"),
        )
        .unwrap()
    }

    /// `components` of the planned `run`, as the recipe names them.
    fn recipe_parts<'a>(
        manifest: &'a Manifest,
        run: &'a PlannedRun,
        components: impl IntoIterator<Item = &'a u32> + 'a,
    ) -> impl Iterator<Item = ComponentRef> + 'a {
        let run_id = recipe_run_id(manifest, &run.run_id).unwrap();
        components.into_iter().map(move |component| ComponentRef {
            run_id: run_id.clone(),
            component: *component,
        })
    }

    /// What the plan would be when `not_installed` of `run`, the first of them failed, are not
    /// in the game, with the runs before it. `Ok(None)` when the failed one is a fixed part.
    fn plan_after_failure(
        manifest: &Manifest,
        evaluation: &SelectionEvaluation,
        position: usize,
        not_installed: &[u32],
        goes_on: bool,
    ) -> Result<(InstallPlan, BTreeSet<ComponentRef>), LeaveOutError> {
        let plan = &evaluation.plan;
        let run = &plan.runs[position];
        let mut in_game = BTreeSet::new();
        for before in &plan.runs[..position] {
            in_game.extend(recipe_parts(manifest, before, &before.components));
        }
        let done = run
            .components
            .iter()
            .filter(|component| !not_installed.contains(component));
        in_game.extend(recipe_parts(manifest, run, done));
        let failed = recipe_parts(manifest, run, &not_installed[..1]).collect();
        let reduced = plan_without_parts(
            manifest,
            &evaluation.normalized_selection,
            plan,
            &BTreeSet::new(),
            &failed,
            &in_game,
        )?;
        let installed = plan.runs[..position]
            .iter()
            .map(|run| run.run_id.clone())
            .collect();
        let (kept, _) =
            plan_keeping_installed(plan, reduced, &installed, &[], run, not_installed, goes_on)
                .unwrap_or_else(|error| panic!("{}: {error}", run.run_id));
        Ok((kept, in_game))
    }

    /// Everything that leaves `plan` with `failed` has a reason in what is gone: it is `failed`
    /// or belongs to its feature, or its feature needs something of which nothing remains.
    /// Nothing leaves that needs only what is in the game or still planned.
    fn assert_only_what_needs_it_leaves(
        manifest: &Manifest,
        plan: &InstallPlan,
        kept: &InstallPlan,
        in_game: &BTreeSet<ComponentRef>,
        failed: &ComponentRef,
    ) {
        let parts = |plan: &InstallPlan| {
            plan.runs
                .iter()
                .flat_map(|run| recipe_parts(manifest, run, &run.components).collect::<Vec<_>>())
                .collect::<BTreeSet<_>>()
        };
        let planned = parts(plan);
        let mut remaining = parts(kept);
        remaining.extend(in_game.iter().cloned());
        let features = &manifest.collection.features;
        let selected = |id: &str| {
            features
                .iter()
                .find(|feature| feature.id == id)
                .unwrap()
                .components
                .iter()
                .any(|part| remaining.contains(part))
        };
        // A feature without components, a mod, is there when something below it is.
        let there = |id: &str| {
            selected(id)
                || features
                    .iter()
                    .any(|feature| feature.parent.as_deref() == Some(id) && selected(&feature.id))
        };
        for feature in features {
            let lost = feature
                .components
                .iter()
                .any(|part| planned.contains(part) && !remaining.contains(part) && part != failed);
            if !lost || feature.components.contains(failed) {
                continue;
            }
            let needs_what_is_gone = feature.parent.as_deref().is_some_and(|id| !there(id))
                || feature.requires.iter().any(|id| !there(id))
                || (!feature.requires_any.is_empty()
                    && !feature.requires_any.iter().any(|id| there(id)));
            assert!(
                needs_what_is_gone,
                "{} left with {failed:?} although all it needs is there",
                feature.id
            );
        }
    }

    /// SCS's last component fails after the rest of SCS was installed: only it leaves, and what
    /// needs SCS stays, such as BG Rebalance's repairs of the spells SCS installed.
    #[test]
    fn the_repairs_of_an_installed_mod_stay_when_its_last_component_fails() {
        let manifest = curated();
        let evaluation = recommended(&manifest, true);
        let plan = &evaluation.plan;
        let position = plan
            .runs
            .iter()
            .position(|run| run.run_id == "stratagems-bg2")
            .unwrap();
        assert_eq!(plan.runs[position].components.last(), Some(&8190));

        let (kept, _) =
            plan_after_failure(&manifest, &evaluation, position, &[8190], true).unwrap();

        let repairs = kept.components_for("chriz-bg-rebalance-bg2").unwrap();
        assert!(
            repairs.contains(&100) && repairs.contains(&120),
            "{repairs:?}"
        );
        let mut expected = plan.clone();
        expected.runs[position].components.pop();
        expected.runs[position].postconditions.clear();
        let parts = |plan: &InstallPlan| {
            plan.runs
                .iter()
                .map(|run| (run.run_id.clone(), run.components.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(parts(&kept), parts(&expected));
    }

    /// Whichever component of the recommended setup fails, the question has an answer at the
    /// point where it fails, whether WeiDU went on or stopped there: the component leaves with
    /// what needs it, or it is one of the fixed parts. Nothing else may come of it, with or
    /// without Spell Revisions, and nothing leaves that needs only what is there.
    #[test]
    fn every_planned_component_can_leave_where_it_fails_or_is_a_fixed_part() {
        let manifest = curated();
        for spell_revisions in [true, false] {
            let evaluation = recommended(&manifest, spell_revisions);
            let plan = &evaluation.plan;
            let mut fixed = BTreeSet::new();
            for (position, run) in plan.runs.iter().enumerate() {
                for (index, component) in run.components.iter().enumerate() {
                    let last = index + 1 == run.components.len();
                    let failed = recipe_parts(&manifest, run, [component]).next().unwrap();
                    for (not_installed, goes_on) in [
                        (std::slice::from_ref(component), last),
                        (&run.components[index..], true),
                    ] {
                        match plan_after_failure(
                            &manifest,
                            &evaluation,
                            position,
                            not_installed,
                            goes_on,
                        ) {
                            Ok((kept, in_game)) => assert_only_what_needs_it_leaves(
                                &manifest, plan, &kept, &in_game, &failed,
                            ),
                            Err(LeaveOutError::Required { title, .. }) => {
                                fixed.insert(title);
                            }
                            Err(error) => panic!("{failed:?}: {error}"),
                        }
                    }
                }
            }
            assert_eq!(
                fixed.iter().map(String::as_str).collect::<Vec<_>>(),
                [
                    "Core fixes and Game Text Update",
                    "EET core import",
                    "Merge Siege of Dragonspear DLC",
                    "Standard EET finalization"
                ]
            );
        }
    }
}
