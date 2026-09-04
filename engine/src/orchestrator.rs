//! Resumable, fail-closed orchestration for one isolated two-root EET campaign.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// Terminal state of one orchestration call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CampaignOutcome {
    /// Every selected logical step completed and the receipt sink accepted the result.
    Complete,
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
    fn acquire(&mut self, identity: &FrozenIdentity, kind: ArtifactKind)
        -> Result<(), StepFailure>;
}

/// Stages independent game copies and applies the deterministic save identity boundary.
pub trait StagingService {
    /// Stage or safely resume one source-game copy. BG1 staging also isolates its save identity.
    fn stage(&mut self, role: GameRole) -> Result<(), StepFailure>;

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

    /// Verify the complete final active stack against the exact frozen run plan.
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
    let schedule = build_schedule(request)?;
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

    let progress = Progress::from_replay(&replay, &schedule)?;
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
        write_terminal_receipt(dependencies, request, &replay, &outcome)?;
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
            write_terminal_receipt(dependencies, request, &store.replay()?, &outcome)?;
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
    };
    let outcome = machine.run()?;
    if outcome == CampaignOutcome::Complete {
        sink.emit(EngineEvent::CampaignFinished {
            install_id: request.created.install_id.clone(),
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
    Install(usize),
    FinalIdentity,
    FinalVerify,
    Receipt,
}

fn build_schedule(request: &CampaignRequest) -> Result<Vec<PipelineStep>, OrchestratorError> {
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
    for (index, run) in request.plan.runs.iter().enumerate() {
        schedule.push(PipelineStep {
            id: format!("install:{}", run.run_id),
            label: format!("Install {}", run.mod_id),
            kind: PipelineKind::Install(index),
        });
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
}

impl Progress {
    fn from_replay(
        replay: &SessionReplay,
        schedule: &[PipelineStep],
    ) -> Result<Self, OrchestratorError> {
        let positions = schedule
            .iter()
            .enumerate()
            .map(|(index, step)| (step.id.as_str(), index))
            .collect::<BTreeMap<_, _>>();
        let mut completed_prefix = 0_usize;
        let mut attempts = BTreeMap::<String, u32>::new();
        let mut unresolved = None;
        let mut last_failed = BTreeMap::new();
        let mut fresh_copy_required = None;

        for record in replay.records.iter().skip(1) {
            match &record.event {
                SessionEvent::StepStarted { step_id, attempt } => {
                    let position = positions.get(step_id.as_str()).ok_or_else(|| {
                        OrchestratorError::InvalidCampaign(format!(
                            "ledger contains unknown step {step_id:?}"
                        ))
                    })?;
                    if *position != completed_prefix {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "ledger started {step_id:?} out of order; expected {:?}",
                            schedule.get(completed_prefix).map(|step| &step.id)
                        )));
                    }
                    let expected_attempt = attempts
                        .get(step_id)
                        .copied()
                        .unwrap_or(0)
                        .checked_add(1)
                        .ok_or_else(|| {
                            OrchestratorError::InvalidCampaign(format!(
                                "attempt counter overflow for {step_id:?}"
                            ))
                        })?;
                    if *attempt != expected_attempt {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "step {step_id:?} attempt {attempt} is not monotonic; expected {expected_attempt}"
                        )));
                    }
                    attempts.insert(step_id.clone(), *attempt);
                    unresolved = Some((step_id.clone(), *attempt));
                }
                SessionEvent::StepCompleted { step_id, attempt } => {
                    if unresolved.as_ref() != Some(&(step_id.clone(), *attempt)) {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "completion for {step_id:?} has no matching intent"
                        )));
                    }
                    completed_prefix += 1;
                    unresolved = None;
                    last_failed.remove(step_id);
                }
                SessionEvent::StepFailed {
                    step_id, attempt, ..
                } => {
                    if unresolved.as_ref() != Some(&(step_id.clone(), *attempt)) {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "failure for {step_id:?} has no matching intent"
                        )));
                    }
                    unresolved = None;
                    last_failed.insert(step_id.clone(), *attempt);
                }
                SessionEvent::FreshCopyRequired {
                    step_id,
                    attempt,
                    detail,
                } => {
                    let matches_active = unresolved.as_ref() == Some(&(step_id.clone(), *attempt));
                    let upgrades_failure = last_failed.get(step_id) == Some(attempt);
                    if !matches_active && !upgrades_failure {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "fresh-copy seal for {step_id:?} has no matching intent"
                        )));
                    }
                    unresolved = None;
                    last_failed.remove(step_id);
                    fresh_copy_required = Some((step_id.clone(), detail.clone()));
                }
                SessionEvent::Created(_) => {
                    return Err(OrchestratorError::InvalidCampaign(
                        "ledger repeats campaign creation".to_owned(),
                    ))
                }
            }
        }

        Ok(Self {
            completed_prefix,
            attempts,
            unresolved,
            last_failed,
            fresh_copy_required,
        })
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

fn build_receipt_draft(
    request: &CampaignRequest,
    replay: &SessionReplay,
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
    })
}

fn write_terminal_receipt<D>(
    dependencies: &mut D,
    request: &CampaignRequest,
    replay: &SessionReplay,
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
    let draft = build_receipt_draft(request, replay, draft_outcome)?;
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
                PipelineKind::Install(index) => self.run_install(&step, index)?,
                _ => self.run_simple(&step)?,
            };
            if let Some(outcome) = outcome {
                let replay = self.store.replay()?;
                write_terminal_receipt(self.dependencies, self.request, &replay, &outcome)?;
                return Ok(outcome);
            }
        }
        Ok(CampaignOutcome::Complete)
    }

    fn run_simple(
        &mut self,
        step: &PipelineStep,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        let attempt = self.begin_or_resume(step)?;
        let execution = match &step.kind {
            PipelineKind::Preflight => {
                let replay = self.store.replay()?;
                self.dependencies
                    .initial(self.request, &replay)
                    .map(|()| SimpleExecution::Complete)
            }
            PipelineKind::Acquire(identity, kind) => self
                .dependencies
                .acquire(identity, *kind)
                .map(|()| SimpleExecution::Complete),
            PipelineKind::Stage(role) => {
                let target = role_target(*role);
                self.recheck(step, target, MutationKind::Stage)
                    .and_then(|()| self.dependencies.stage(*role))
                    .map(|()| SimpleExecution::Complete)
            }
            PipelineKind::Materialize(task) => self
                .recheck(step, task.target, MutationKind::Materialize)
                .and_then(|()| self.dependencies.materialize(task))
                .map(|outcome| match outcome {
                    MaterializationOutcome::Complete => SimpleExecution::Complete,
                    MaterializationOutcome::FreshCopyRequired { reason } => {
                        SimpleExecution::FreshCopyRequired(reason)
                    }
                }),
            PipelineKind::FinalIdentity => self
                .recheck(step, GameRoot::Bg2, MutationKind::FinalIdentity)
                .and_then(|()| self.dependencies.finalize_identity())
                .map(|()| SimpleExecution::Complete),
            PipelineKind::FinalVerify => self
                .dependencies
                .verify_final(&self.request.plan)
                .map(|()| SimpleExecution::Complete),
            PipelineKind::Receipt => (|| {
                let replay = self
                    .store
                    .replay()
                    .map_err(|error| StepFailure::new(error.to_string()))?;
                let draft =
                    build_receipt_draft(self.request, &replay, ReceiptDraftOutcome::Succeeded)
                        .map_err(|error| StepFailure::new(error.to_string()))?;
                self.dependencies
                    .write(&draft)
                    .map(|()| SimpleExecution::Complete)
            })(),
            PipelineKind::Install(_) => unreachable!("install steps use run_install"),
        };

        match execution {
            Ok(SimpleExecution::Complete) => {
                self.complete_step(step, &attempt)?;
                Ok(None)
            }
            Ok(SimpleExecution::FreshCopyRequired(reason)) => {
                self.seal_fresh_copy(step, &attempt, reason).map(Some)
            }
            Err(failure) => {
                let reason = failure.into_message();
                self.fail_step(step, &attempt, &reason)?;
                Ok(Some(CampaignOutcome::Failed {
                    step_id: step.id.clone(),
                    reason,
                }))
            }
        }
    }

    fn run_install(
        &mut self,
        step: &PipelineStep,
        run_index: usize,
    ) -> Result<Option<CampaignOutcome>, OrchestratorError> {
        let run = self.request.plan.runs[run_index].clone();
        if self.active_phase != Some(run.phase) {
            self.sink.emit(EngineEvent::PhaseStarted {
                name: friendly_phase(run.phase).to_owned(),
            });
            self.active_phase = Some(run.phase);
        }

        let mut components = run.components.clone();
        let mut current = if let Some((unresolved_id, attempt)) = &self.progress.unresolved {
            if unresolved_id != &step.id {
                return Err(OrchestratorError::InvalidCampaign(format!(
                    "unresolved step {unresolved_id:?} is not the next pipeline step {:?}",
                    step.id
                )));
            }
            let attempt = self.attempt(step, *attempt);
            emit_started(self.sink, &step.id, &format!("Resume {}", step.label));
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
                Ok(InstallReconciliation::FreshCopyRequired { reason }) => {
                    return self.seal_fresh_copy(step, &attempt, reason).map(Some);
                }
                Err(failure) => {
                    let reason = format!("could not reconcile interrupted evidence: {failure}");
                    return self.seal_fresh_copy(step, &attempt, reason).map(Some);
                }
            }
        } else if let Some(attempt) = self.progress.last_failed.get(&step.id).copied() {
            let evidence_attempt = self.attempt(step, attempt);
            match self.dependencies.recover(&run, &evidence_attempt) {
                Ok(InstallReconciliation::ProvenDone) => {
                    if let Some(outcome) =
                        self.postcondition_failure(step, &run, &evidence_attempt)?
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
                Ok(InstallReconciliation::FreshCopyRequired { reason }) => {
                    return self
                        .seal_fresh_copy(step, &evidence_attempt, reason)
                        .map(Some);
                }
                Err(failure) => {
                    let reason = format!("could not reconcile failed attempt evidence: {failure}");
                    return self
                        .seal_fresh_copy(step, &evidence_attempt, reason)
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
                    let reason = "WeiDU made no durable change; the exact run remains retryable";
                    self.fail_step(step, &current, reason)?;
                    if remaining != components {
                        return Err(OrchestratorError::InvalidCampaign(format!(
                            "retry verdict for {:?} changed the requested suffix",
                            step.id
                        )));
                    }
                    return Ok(Some(CampaignOutcome::Failed {
                        step_id: step.id.clone(),
                        reason: reason.to_owned(),
                    }));
                }
                InstallReconciliation::PartialPrefix { remaining } => {
                    let remaining = validate_remaining(&run, &remaining, Some(&components))?;
                    if remaining.len() >= components.len() {
                        let reason = "partial-prefix evidence did not reduce the remaining suffix";
                        return self
                            .seal_fresh_copy(step, &current, reason.to_owned())
                            .map(Some);
                    }
                    self.fail_step(
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
        self.recheck(step, run.target, MutationKind::InvocationBuild)?;
        let built = self.dependencies.build(run, components, attempt)?;
        if built.identity_digest.trim().is_empty() {
            return Err(StepFailure::new("invocation identity digest is empty"));
        }
        self.dependencies
            .record_invocation(run, attempt, &built.identity_digest)?;
        self.recheck(step, run.target, MutationKind::ProcessSpawn)?;
        let result = self.dependencies.run(built.invocation, attempt)?;
        self.dependencies.sync_and_reconcile(run, attempt, result)
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
    let metadata =
        fs::symlink_metadata(parent).map_err(|source| OrchestratorError::UnsafeTarget {
            path: parent.to_path_buf(),
            reason: format!("managed root parent must already exist: {source}"),
        })?;
    ensure_direct_directory(parent, &metadata)?;
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
