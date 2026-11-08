use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bg_engine::decision::{
    Answer, AnswerSource, DecisionOutcome, PendingDecision, Question, QuestionKind, ServedDownload,
    UnusableKind, UnusableReason,
};
use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::events::{EngineEvent, EventSink, StepOutcome};
use bg_engine::games::GameRole;
use bg_engine::lock::{LockError, TargetLock};
use bg_engine::manifest::{GameRoot, Phase, Postcondition, RunArg};
use bg_engine::orchestrator::{
    run_campaign, run_campaign_controlled, AcquisitionOutcome, AnsweredAcquisition,
    ArtifactAcquirer, ArtifactKind, ArtifactMaterializer, BuiltInvocation, CampaignClock,
    CampaignOutcome, CampaignPreflight, CampaignRecorder, CampaignRequest, InstallLogVerifier,
    InstallReconciliation, InvocationBuilder, LeftOutArtifact, MaterializationOutcome,
    MaterializationTask, MutationCheck, MutationKind, ProcessResult, ProcessRunner, ReceiptDraft,
    ReceiptDraftOutcome, ReceiptWriter, StagingService, StepAttempt, StepFailure,
};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::resolve::{InstallPlan, PlannedRun};
use bg_engine::session::{
    CampaignCreated, FrozenIdentity, SessionEvent, SessionReplay, SessionStore,
    SourceGameFingerprints,
};
use bg_engine::weidu::runner::RunnerControlHandle;

#[cfg(unix)]
fn create_directory_link(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, destination)
}

#[cfg(windows)]
fn create_directory_link(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(source, destination)
}

const RECIPE_PAYLOAD: &[u8] = b"PK\x03\x04signed alpha recipe";
const RECIPE_ENVELOPE: &[u8] = br#"{"recipe_id":"alpha","version":"0.1.0"}"#;
const LEGACY_EMPTY_POSTCONDITION_PLAN_JSON: &str = concat!(
    r#"{"runs":[{"run_id":"eefix-bg1","mod_id":"eefix","target":"bg1","phase":"bg1-preparation","components":[0,2],"args":[],"artifact_id":"eefix","weidu_artifact_id":"weidu","prompt_scripts":[]},"#,
    r#"{"run_id":"eefix-bg2","mod_id":"eefix","target":"bg2","phase":"bg2-preparation","components":[0,2],"args":[],"artifact_id":"eefix","weidu_artifact_id":"weidu","prompt_scripts":[]},"#,
    r#"{"run_id":"eet-core","mod_id":"eet","target":"bg2","phase":"eet-initialization","components":[0],"args":[{"kind":"staged-root","value":"bg1"}],"artifact_id":"eet","weidu_artifact_id":"weidu","prompt_scripts":[]}]}"#,
);
const LEGACY_EMPTY_POSTCONDITION_PLAN_SHA256: &str =
    "83e0d4f0436f5e85d7f672c84aa89fa1f14f98675fb2324ff7b18fa8ed866b78";

#[derive(Clone, Default)]
struct RecordingSink(Arc<Mutex<Vec<EngineEvent>>>);

impl RecordingSink {
    fn events(&self) -> Vec<EngineEvent> {
        self.0.lock().unwrap().clone()
    }
}

impl EventSink for RecordingSink {
    fn emit(&self, event: EngineEvent) {
        self.0.lock().unwrap().push(event);
    }
}

struct Fixture {
    _temp: tempfile::TempDir,
    request: CampaignRequest,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let registry_root = temp.path().join("registry");
        let cache_root = temp.path().join("cache");
        std::fs::create_dir(&registry_root).unwrap();
        std::fs::create_dir(&cache_root).unwrap();
        let registry_root = std::fs::canonicalize(registry_root).unwrap();
        let cache_root = std::fs::canonicalize(cache_root).unwrap();
        let managed_root = std::fs::canonicalize(temp.path())
            .unwrap()
            .join("managed-alpha");

        let runs = vec![
            executable_run(
                "eefix-bg1",
                "eefix",
                GameRoot::Bg1,
                Phase::Bg1Preparation,
                &[0, 2],
                vec![],
            ),
            executable_run(
                "eefix-bg2",
                "eefix",
                GameRoot::Bg2,
                Phase::Bg2Preparation,
                &[0, 2],
                vec![],
            ),
            executable_run(
                "eet-core",
                "eet",
                GameRoot::Bg2,
                Phase::EetInitialization,
                &[0],
                vec![RunArg::StagedRoot(GameRoot::Bg1)],
            ),
        ];
        let plan = InstallPlan { runs };
        let selection = NormalizedSelection {
            platform: "windows".to_owned(),
            features: BTreeMap::from([("recommended".to_owned(), true)]),
            inputs: BTreeMap::new(),
        };
        let created = CampaignCreated {
            install_id: "install-alpha".to_owned(),
            attempt_id: "attempt-001".to_owned(),
            managed_root: managed_root.clone(),
            cache_root,
            recipe_payload: RECIPE_PAYLOAD.to_vec(),
            recipe_payload_sha256: sha256_bytes(RECIPE_PAYLOAD),
            recipe_envelope: RECIPE_ENVELOPE.to_vec(),
            recipe_envelope_sha256: sha256_bytes(RECIPE_ENVELOPE),
            selection_sha256: selection_digest(&selection).unwrap(),
            normalized_selection: selection,
            plan_sha256: plan_digest(&plan).unwrap(),
            source_games: SourceGameFingerprints {
                bg1: Some("11".repeat(32)),
                bg2: "22".repeat(32),
                standalone_role: None,
            },
            artifact_identities: vec![
                frozen("eefix", "33", 10),
                frozen("weidu", "66", 40),
                frozen("eet", "44", 20),
            ],
            tool_identities: vec![frozen("weidu", "55", 30)],
            staged_bg1: Some(managed_root.join("bg1")),
            staged_bg2: managed_root.join("game"),
        };
        let request = CampaignRequest {
            registry_root,
            created,
            plan,
        };
        Self {
            _temp: temp,
            request,
        }
    }
}

impl Fixture {
    /// A standalone Icewind Dale: Enhanced Edition campaign: one source, one staged root, and
    /// only main-phase runs.
    fn standalone() -> Self {
        let mut fixture = Self::new();
        let managed_root = fixture.request.created.managed_root.clone();
        let plan = InstallPlan {
            runs: vec![
                executable_run(
                    "iwdfix-main",
                    "eefix",
                    GameRoot::Bg2,
                    Phase::Main,
                    &[0, 2],
                    vec![],
                ),
                executable_run(
                    "iwdtweaks-main",
                    "eet",
                    GameRoot::Bg2,
                    Phase::Main,
                    &[0],
                    vec![RunArg::StagedRoot(GameRoot::Bg2)],
                ),
            ],
        };
        let created = &mut fixture.request.created;
        created.plan_sha256 = plan_digest(&plan).unwrap();
        created.source_games = SourceGameFingerprints {
            bg1: None,
            bg2: "22".repeat(32),
            standalone_role: Some(GameRole::Iwdee),
        };
        created.staged_bg1 = None;
        created.staged_bg2 = managed_root.join("game");
        fixture.request.plan = plan;
        fixture
    }
}
fn frozen(id: &str, byte: &str, length: u64) -> FrozenIdentity {
    FrozenIdentity {
        id: id.to_owned(),
        version: "test".to_owned(),
        sha256: byte.repeat(32),
        length,
    }
}

fn executable_run(
    run_id: &str,
    artifact_id: &str,
    target: GameRoot,
    phase: Phase,
    components: &[u32],
    args: Vec<RunArg>,
) -> PlannedRun {
    PlannedRun {
        run_id: run_id.to_owned(),
        mod_id: artifact_id.to_owned(),
        target,
        phase,
        components: components.to_vec(),
        args,
        postconditions: vec![],
        artifact_id: artifact_id.to_owned(),
        weidu_artifact_id: "weidu".to_owned(),
        prompt_scripts: vec![],
    }
}

#[derive(Default)]
struct FakeDeps {
    trace: Vec<String>,
    acquisitions: BTreeMap<String, usize>,
    acquired_identities: Vec<(String, u64, ArtifactKind)>,
    fail_once: Option<String>,
    materialization_results: VecDeque<MaterializationOutcome>,
    recovery_results: VecDeque<InstallReconciliation>,
    after_results: VecDeque<InstallReconciliation>,
    runner_active: bool,
    lock_probe: Option<(PathBuf, PathBuf)>,
    lock_was_held_at_receipt: bool,
    receipts: Vec<ReceiptDraft>,
    recorded_campaigns: Vec<CampaignCreated>,
    record_saw_durable_ledger: bool,
    record_was_first_dependency: bool,
    now: u64,
    staged_files: Vec<(GameRole, PathBuf, Vec<u8>)>,
    pause_after_operation: Option<String>,
    controls: Option<RunnerControlHandle>,
    /// Payloads whose acquisition reports an unusable download.
    leave_out: BTreeMap<String, String>,
    /// Placing steps that report a download whose files cannot be placed, by step id.
    unplaceable: BTreeMap<String, String>,
    /// Further runs that need a left-out payload, by payload id.
    dependents: BTreeMap<String, Vec<String>>,
    /// Runs that lose components when a payload is left out, by payload id.
    reduced: BTreeMap<String, Vec<(String, Vec<u32>)>>,
    /// Injected failures reconciling CEBG records, independent of mandatory selections.
    invalid_history: Vec<String>,
    /// Every `plan_without` request, to see replays recompute it.
    plan_without_calls: Vec<Vec<String>>,
    /// Manual pauses of the fake campaign; each continues at once.
    checkpoints: Vec<bg_engine::manual_checkpoint::FrozenCheckpoint>,
    /// Checkpoint plans handed over, to see that they lack left-out runs.
    checkpoint_plans: Vec<InstallPlan>,
    /// The plan final verification received.
    verified_plans: Vec<InstallPlan>,
    /// Answers this run gives to questions, by kind, as a preset or as the player.
    answers: BTreeMap<QuestionKind, (Answer, AnswerSource)>,
    /// How often the campaign asked for an answer.
    answer_requests: usize,
    /// Payloads or tools whose link serves a changed file that passed every check.
    changed: BTreeMap<String, ServedDownload>,
    /// Tools whose changed file came from a host the recipe did not review.
    unreviewed: BTreeMap<String, ServedDownload>,
    /// What applying "install it" finds, in turn; `Accepted` once they are used up.
    answered_results: VecDeque<AnsweredAcquisition>,
    /// Every "install it" applied, by artifact id.
    answered_calls: Vec<String>,
    /// The link keeps serving the changed file even after "install it" was applied.
    keep_changed: bool,
    /// Payloads whose reviewed archive is back in the download folder.
    reviewed_back: Vec<String>,
    /// Further runs that need a failed run's components, by failed run id.
    failed_dependents: BTreeMap<String, Vec<String>>,
    /// Further components of a run that need one of its failed components, by run id.
    failed_with: BTreeMap<String, Vec<u32>>,
    /// The attempts whose evidence was read back, in order.
    recovered_attempts: Vec<u32>,
    /// Every `plan_without_parts` request, to see replays not recompute it.
    plan_without_parts_calls: Vec<Vec<(String, Vec<u32>)>>,
    /// What each `plan_without_parts` request said was in the game.
    in_game_at_questions: Vec<Vec<(String, Vec<u32>)>>,
    /// Source games that look modded, with the question to ask before copying them.
    source_questions: BTreeMap<GameRole, Question>,
}

impl CampaignRecorder for FakeDeps {
    fn record_campaign(&mut self, created: &CampaignCreated) -> Result<(), StepFailure> {
        self.record_was_first_dependency = self.trace.is_empty();
        let replay = SessionStore::open(&created.managed_root)
            .and_then(|store| store.replay())
            .map_err(|error| StepFailure::new(error.to_string()))?;
        self.record_saw_durable_ledger = replay.created() == created;
        self.recorded_campaigns.push(created.clone());
        Ok(())
    }
}

impl FakeDeps {
    fn maybe_fail(&mut self, operation: &str) -> Result<(), StepFailure> {
        if self.fail_once.as_deref() == Some(operation) {
            self.fail_once = None;
            return Err(StepFailure::new(format!("injected failure at {operation}")));
        }
        Ok(())
    }
}

impl CampaignPreflight for FakeDeps {
    fn initial(
        &mut self,
        _request: &CampaignRequest,
        _replay: &SessionReplay,
    ) -> Result<(), StepFailure> {
        self.trace.push("preflight".to_owned());
        self.maybe_fail("preflight")
    }

    fn recheck_before_mutation(&mut self, check: &MutationCheck) -> Result<(), StepFailure> {
        self.trace
            .push(format!("check:{}:{}", check.kind.as_str(), check.step_id));
        self.maybe_fail(&format!("check:{}:{}", check.kind.as_str(), check.step_id))?;
        self.maybe_fail(&format!("check:{}", check.step_id))
    }
}

impl ArtifactAcquirer for FakeDeps {
    fn acquire(
        &mut self,
        identity: &FrozenIdentity,
        kind: ArtifactKind,
    ) -> Result<AcquisitionOutcome, StepFailure> {
        let operation = format!("acquire:{}", identity.id);
        self.trace.push(operation.clone());
        *self.acquisitions.entry(identity.id.clone()).or_default() += 1;
        self.acquired_identities
            .push((identity.id.clone(), identity.length, kind));
        if self.pause_after_operation.as_deref() == Some(&operation) {
            self.controls.as_ref().unwrap().pause_after_boundary();
        }
        self.maybe_fail(&operation)?;
        // Once "install it" was applied, as after a withdrawal, the reviewed file is there,
        // unless the link keeps serving the changed one.
        if (self.answered_calls.contains(&identity.id) && !self.keep_changed)
            || self.reviewed_back.contains(&identity.id)
        {
            return Ok(AcquisitionOutcome::Ready);
        }
        if let Some(served) = self.changed.get(&identity.id) {
            return Ok(AcquisitionOutcome::Ask {
                kind: QuestionKind::ChangedDownload,
                served: served.clone(),
            });
        }
        if let Some(served) = self.unreviewed.get(&identity.id) {
            return Ok(AcquisitionOutcome::Ask {
                kind: QuestionKind::UnreviewedTool,
                served: served.clone(),
            });
        }
        Ok(match self.leave_out.get(&identity.id) {
            Some(reason) => AcquisitionOutcome::LeaveOut {
                reason: UnusableReason::new(
                    UnusableKind::Layout {
                        message: reason.clone(),
                    },
                    reason.clone(),
                ),
            },
            None => AcquisitionOutcome::Ready,
        })
    }

    fn artifact_name(&self, artifact_id: &str) -> String {
        format!("The {artifact_id} mod")
    }

    fn decision_answer(
        &mut self,
        decision: &PendingDecision,
    ) -> Result<Option<(Answer, AnswerSource)>, StepFailure> {
        self.trace.push(format!("answer?:{}", decision.decision_id));
        self.answer_requests += 1;
        Ok(self.answers.get(&decision.question.kind).copied())
    }

    fn reviewed_is_available(
        &mut self,
        identity: &FrozenIdentity,
        _kind: ArtifactKind,
    ) -> Result<bool, StepFailure> {
        self.trace.push(format!("reviewed?:{}", identity.id));
        Ok(self.reviewed_back.contains(&identity.id))
    }

    fn acquire_answered(
        &mut self,
        identity: &FrozenIdentity,
        _kind: ArtifactKind,
        question: &Question,
    ) -> Result<AnsweredAcquisition, StepFailure> {
        assert_eq!(identity, &question.reviewed);
        self.trace.push(format!("install-anyway:{}", identity.id));
        self.answered_calls.push(identity.id.clone());
        Ok(self
            .answered_results
            .pop_front()
            .unwrap_or(AnsweredAcquisition::Accepted))
    }

    fn plan_without(
        &mut self,
        plan: &InstallPlan,
        left_out: &[String],
    ) -> Result<InstallPlan, StepFailure> {
        self.plan_without_calls.push(left_out.to_vec());
        if let Some(invalid_history) = left_out.iter().find(|id| self.invalid_history.contains(id))
        {
            return Err(StepFailure::new(format!(
                "CEBG history cannot be reconciled for {invalid_history}"
            )));
        }
        let mut runs = Vec::new();
        for run in &plan.runs {
            if left_out.contains(&run.artifact_id)
                || left_out.iter().any(|id| {
                    self.dependents
                        .get(id)
                        .is_some_and(|runs| runs.contains(&run.run_id))
                })
            {
                continue;
            }
            let mut run = run.clone();
            for (run_id, components) in left_out
                .iter()
                .filter_map(|id| self.reduced.get(id))
                .flatten()
            {
                if *run_id == run.run_id {
                    run.components = components.clone();
                }
            }
            runs.push(run);
        }
        Ok(InstallPlan { runs })
    }

    fn plan_without_parts(
        &mut self,
        plan: &InstallPlan,
        left_out: &[String],
        failed: &[(String, Vec<u32>)],
        in_game: &[(String, Vec<u32>)],
    ) -> Result<InstallPlan, StepFailure> {
        self.plan_without_parts_calls.push(failed.to_vec());
        self.in_game_at_questions.push(in_game.to_vec());
        for (run_id, _) in failed {
            if let Some(run) = plan.runs.iter().find(|run| &run.run_id == run_id) {
                if self.invalid_history.contains(&run.artifact_id) {
                    return Err(StepFailure::new(format!(
                        "CEBG history cannot be reconciled for {}",
                        run.artifact_id
                    )));
                }
            }
        }
        let mut reduced = self.plan_without(plan, left_out)?;
        for (run_id, components) in failed {
            let dependents = self
                .failed_dependents
                .get(run_id)
                .cloned()
                .unwrap_or_default();
            let leaving = self.failed_with.get(run_id).cloned().unwrap_or_default();
            reduced.runs.retain_mut(|run| {
                if &run.run_id == run_id {
                    run.components.retain(|component| {
                        !components.contains(component) && !leaving.contains(component)
                    });
                    !run.components.is_empty()
                } else {
                    !dependents.contains(&run.run_id)
                }
            });
        }
        Ok(reduced)
    }
}

impl StagingService for FakeDeps {
    fn stage(&mut self, role: GameRole) -> Result<(), StepFailure> {
        let role_name = match role {
            GameRole::BgeeSod => "bg1",
            GameRole::Bg2ee => "bg2",
            GameRole::Iwdee => "game",
        };
        let operation = format!("stage:{role_name}");
        self.trace.push(operation.clone());
        self.maybe_fail(&operation)?;
        for (_, path, contents) in self
            .staged_files
            .iter()
            .filter(|(file_role, _, _)| *file_role == role)
        {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| StepFailure::new(error.to_string()))?;
            }
            std::fs::write(path, contents).map_err(|error| StepFailure::new(error.to_string()))?;
        }
        Ok(())
    }

    fn source_question(&mut self, role: GameRole) -> Result<Option<Question>, StepFailure> {
        Ok(self.source_questions.get(&role).cloned())
    }

    fn finalize_identity(&mut self) -> Result<(), StepFailure> {
        self.trace.push("identity:final".to_owned());
        self.maybe_fail("identity:final")
    }
}

impl ArtifactMaterializer for FakeDeps {
    fn materialize(
        &mut self,
        task: &MaterializationTask,
    ) -> Result<MaterializationOutcome, StepFailure> {
        let operation = task.step_id();
        self.trace.push(operation.clone());
        if self.pause_after_operation.as_deref() == Some(&operation) {
            self.controls.as_ref().unwrap().pause_after_boundary();
        }
        self.maybe_fail(&operation)?;
        if let Some(reason) = self.unplaceable.get(&operation) {
            return Ok(MaterializationOutcome::LeaveOut {
                reason: UnusableReason::new(
                    UnusableKind::Collision {
                        path: "Shared/file.txt".to_owned(),
                        message: reason.clone(),
                    },
                    reason.clone(),
                ),
            });
        }
        Ok(self
            .materialization_results
            .pop_front()
            .unwrap_or(MaterializationOutcome::Complete))
    }
}

impl InvocationBuilder for FakeDeps {
    type Invocation = String;

    fn build(
        &mut self,
        run: &PlannedRun,
        components: &[u32],
        _attempt: &StepAttempt,
    ) -> Result<BuiltInvocation<Self::Invocation>, StepFailure> {
        let operation = format!("build:{}:{components:?}", run.run_id);
        self.trace.push(operation.clone());
        self.maybe_fail(&format!("build:{}", run.run_id))?;
        Ok(BuiltInvocation {
            invocation: run.run_id.clone(),
            identity_digest: format!("digest-{}-{components:?}", run.run_id),
        })
    }
}

impl ProcessRunner<String> for FakeDeps {
    fn run(
        &mut self,
        invocation: String,
        _attempt: &StepAttempt,
    ) -> Result<ProcessResult, StepFailure> {
        assert!(!self.runner_active, "WeiDU runs overlapped");
        self.runner_active = true;
        let operation = format!("run:{invocation}");
        self.trace.push(operation.clone());
        let result = self
            .maybe_fail(&operation)
            .map(|()| ProcessResult { exit_code: 0 });
        self.runner_active = false;
        result
    }
}

impl InstallLogVerifier for FakeDeps {
    fn manual_checkpoints(&self) -> Vec<bg_engine::manual_checkpoint::FrozenCheckpoint> {
        self.checkpoints.clone()
    }

    fn manual_checkpoint(
        &mut self,
        checkpoint: &bg_engine::manual_checkpoint::FrozenCheckpoint,
        plan: &InstallPlan,
    ) -> Result<bool, StepFailure> {
        self.trace
            .push(format!("checkpoint:{}", checkpoint.checkpoint_id));
        self.checkpoint_plans.push(plan.clone());
        Ok(true)
    }

    fn recover(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
    ) -> Result<InstallReconciliation, StepFailure> {
        self.trace.push(format!("recover:{}", run.run_id));
        self.recovered_attempts.push(attempt.attempt);
        Ok(self
            .recovery_results
            .pop_front()
            .unwrap_or_else(|| InstallReconciliation::retry(run.components.clone())))
    }

    fn snapshot_before(
        &mut self,
        run: &PlannedRun,
        _attempt: &StepAttempt,
    ) -> Result<(), StepFailure> {
        self.trace.push(format!("snapshot-before:{}", run.run_id));
        Ok(())
    }

    fn record_pre_spawn_failure(
        &mut self,
        run: &PlannedRun,
        components: &[u32],
        _attempt: &StepAttempt,
        kind: MutationKind,
        _failure: &StepFailure,
    ) -> Result<String, StepFailure> {
        self.trace.push(format!(
            "pre-spawn-failure:{}:{}:{components:?}",
            kind.as_str(),
            run.run_id
        ));
        Ok(sha256_bytes(b"synthetic durable guard evidence"))
    }

    fn record_invocation(
        &mut self,
        run: &PlannedRun,
        _attempt: &StepAttempt,
        identity_digest: &str,
    ) -> Result<(), StepFailure> {
        self.trace.push(format!(
            "record-invocation:{}:{identity_digest}",
            run.run_id
        ));
        Ok(())
    }

    fn sync_and_reconcile(
        &mut self,
        run: &PlannedRun,
        _attempt: &StepAttempt,
        _result: ProcessResult,
    ) -> Result<InstallReconciliation, StepFailure> {
        self.trace.push(format!("sync-reconcile:{}", run.run_id));
        Ok(self
            .after_results
            .pop_front()
            .unwrap_or(InstallReconciliation::ProvenDone))
    }

    fn verify_final(&mut self, plan: &InstallPlan) -> Result<(), StepFailure> {
        self.trace.push("verify:final".to_owned());
        self.verified_plans.push(plan.clone());
        self.maybe_fail("verify:final")
    }
}

impl ReceiptWriter for FakeDeps {
    fn write(&mut self, draft: &ReceiptDraft) -> Result<(), StepFailure> {
        self.trace.push("receipt".to_owned());
        self.receipts.push(draft.clone());
        if let Some((registry, target)) = &self.lock_probe {
            self.lock_was_held_at_receipt = matches!(
                TargetLock::try_acquire(registry, target),
                Err(LockError::Contended { .. })
            );
        }
        self.maybe_fail("receipt")
    }
}

impl CampaignClock for FakeDeps {
    fn now_millis(&mut self) -> Result<u64, StepFailure> {
        self.now += 1;
        Ok(self.now)
    }
}

fn completed_step_ids(root: &Path) -> Vec<String> {
    SessionStore::open(root)
        .unwrap()
        .replay()
        .unwrap()
        .records
        .into_iter()
        .filter_map(|record| match record.event {
            SessionEvent::StepCompleted { step_id, .. } => Some(step_id),
            _ => None,
        })
        .collect()
}

#[test]
fn campaign_index_is_published_after_record_zero_and_before_preflight() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        fail_once: Some("preflight".to_owned()),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert!(matches!(
        outcome,
        CampaignOutcome::Failed { ref step_id, .. } if step_id == "preflight"
    ));
    let durable = SessionStore::open(&fixture.request.created.managed_root)
        .unwrap()
        .replay()
        .unwrap()
        .created()
        .clone();
    assert_eq!(deps.recorded_campaigns, vec![durable]);
    assert!(deps.record_saw_durable_ledger);
    assert!(deps.record_was_first_dependency);
    assert_eq!(
        deps.trace.iter().position(|entry| entry == "preflight"),
        Some(0)
    );
}

#[test]
fn a_new_campaign_creates_missing_destination_ancestors_before_claiming_its_root() {
    let mut fixture = Fixture::new();
    let temp_root = std::fs::canonicalize(fixture._temp.path()).unwrap();
    fixture.request.created.managed_root = temp_root
        .join("Games")
        .join("NewCollection")
        .join("managed-alpha");
    fixture.request.created.staged_bg1 = Some(fixture.request.created.managed_root.join("bg1"));
    fixture.request.created.staged_bg2 = fixture.request.created.managed_root.join("game");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert_eq!(outcome, CampaignOutcome::Complete);
    assert!(fixture.request.created.managed_root.is_dir());
    assert_eq!(
        SessionStore::open(&fixture.request.created.managed_root)
            .unwrap()
            .replay()
            .unwrap()
            .created()
            .managed_root,
        fixture.request.created.managed_root
    );
}

fn seed_unresolved(request: &CampaignRequest, completed: &[&str], unresolved: &str) {
    std::fs::create_dir(&request.created.managed_root).unwrap();
    let store = SessionStore::create(
        &request.created.managed_root,
        SessionEvent::Created(Box::new(request.created.clone())),
    )
    .unwrap();
    for step_id in completed {
        store
            .append(SessionEvent::StepStarted {
                step_id: (*step_id).to_owned(),
                attempt: 1,
            })
            .unwrap();
        store
            .append(SessionEvent::StepCompleted {
                step_id: (*step_id).to_owned(),
                attempt: 1,
            })
            .unwrap();
    }
    store
        .append(SessionEvent::StepStarted {
            step_id: unresolved.to_owned(),
            attempt: 1,
        })
        .unwrap();
}

const THROUGH_STAGING: &[&str] = &[
    "preflight",
    "acquire:eefix",
    "acquire:weidu",
    "acquire:eet",
    "stage:bg1",
    "stage:bg2",
];

const THROUGH_MATERIALIZATION: &[&str] = &[
    "preflight",
    "acquire:eefix",
    "acquire:weidu",
    "acquire:eet",
    "stage:bg1",
    "stage:bg2",
    "materialize:eefix:bg1",
    "materialize:eefix:bg2",
    "materialize:eet:bg2",
];

#[test]
fn complete_build_uses_stable_order_one_acquisition_per_identity_and_holds_target_lock() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        lock_probe: Some((
            fixture.request.registry_root.clone(),
            fixture.request.created.managed_root.clone(),
        )),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert_eq!(outcome, CampaignOutcome::Complete);
    assert_eq!(
        completed_step_ids(&fixture.request.created.managed_root),
        vec![
            "preflight",
            "acquire:eefix",
            "acquire:weidu",
            "acquire:eet",
            "stage:bg1",
            "stage:bg2",
            "materialize:eefix:bg1",
            "materialize:eefix:bg2",
            "materialize:eet:bg2",
            "install:eefix-bg1",
            "install:eefix-bg2",
            "install:eet-core",
            "identity:final",
            "verify:final",
            "receipt",
        ]
    );
    assert_eq!(
        deps.acquisitions,
        BTreeMap::from([
            ("eefix".to_owned(), 1),
            ("eet".to_owned(), 1),
            ("weidu".to_owned(), 1),
        ])
    );
    assert_eq!(
        deps.acquired_identities
            .iter()
            .filter(|(id, _, _)| id == "weidu")
            .cloned()
            .collect::<Vec<_>>(),
        vec![("weidu".to_owned(), 40, ArtifactKind::Tool)],
        "the enclosing WeiDU archive must be acquired once as a tool"
    );
    assert!(deps.lock_was_held_at_receipt);
    let receipt = deps.receipts.last().unwrap();
    assert!(receipt.left_out.is_empty() && receipt.installed_plan.is_none());
    assert_eq!(deps.verified_plans, vec![fixture.request.plan.clone()]);
    assert!(deps.plan_without_calls.is_empty());
    assert!(deps.trace.iter().any(|entry| entry == "stage:bg1"));
    assert!(deps.trace.iter().any(|entry| entry == "stage:bg2"));
    assert!(deps
        .trace
        .iter()
        .any(|entry| entry == "materialize:eefix:bg1"));
    assert!(deps
        .trace
        .iter()
        .any(|entry| entry == "materialize:eefix:bg2"));
    assert!(deps.trace.iter().any(|entry| entry == "build:eet-core:[0]"));
    assert!(fixture.request.plan.runs[2]
        .args
        .contains(&RunArg::StagedRoot(GameRoot::Bg1)));

    let phase_names = sink
        .events()
        .into_iter()
        .filter_map(|event| match event {
            EngineEvent::PhaseStarted { name } => Some(name),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        phase_names,
        vec![
            "Prepare Baldur's Gate: Enhanced Edition",
            "Prepare Baldur's Gate II: Enhanced Edition",
            "Build the EET campaign",
        ]
    );
}

#[test]
fn pause_waits_for_a_durable_boundary_and_resume_starts_with_the_next_step() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let controls = RunnerControlHandle::new();
    let mut deps = FakeDeps {
        pause_after_operation: Some("acquire:eefix".to_owned()),
        controls: Some(controls.clone()),
        ..FakeDeps::default()
    };

    let outcome = run_campaign_controlled(&fixture.request, &mut deps, &sink, &controls).unwrap();

    assert_eq!(
        outcome,
        CampaignOutcome::Paused {
            after_step_id: "acquire:eefix".to_owned(),
        }
    );
    assert_eq!(
        completed_step_ids(&fixture.request.created.managed_root),
        vec!["preflight", "acquire:eefix"]
    );
    assert!(deps.receipts.is_empty(), "pause is not a terminal receipt");

    let mut resumed = FakeDeps::default();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(resumed.acquisitions.get("eefix"), None);
    assert_eq!(resumed.acquisitions.get("weidu"), Some(&1));
}

#[test]
fn legacy_empty_postcondition_plan_digest_still_resumes() {
    let mut fixture = Fixture::new();
    let legacy_plan: InstallPlan =
        serde_json::from_str(LEGACY_EMPTY_POSTCONDITION_PLAN_JSON).unwrap();
    assert_eq!(legacy_plan, fixture.request.plan);
    fixture.request.plan = legacy_plan;
    assert_eq!(
        serde_json::to_string(&fixture.request.plan).unwrap(),
        LEGACY_EMPTY_POSTCONDITION_PLAN_JSON
    );
    assert_eq!(
        plan_digest(&fixture.request.plan).unwrap(),
        LEGACY_EMPTY_POSTCONDITION_PLAN_SHA256
    );
    fixture.request.created.plan_sha256 = LEGACY_EMPTY_POSTCONDITION_PLAN_SHA256.to_owned();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        fail_once: Some("stage:bg2".to_owned()),
        ..FakeDeps::default()
    };

    assert!(matches!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Failed { ref step_id, .. } if step_id == "stage:bg2"
    ));
    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
}

#[test]
fn normal_proven_install_completes_with_authored_markers() {
    let mut fixture = Fixture::new();
    add_first_run_marker(&mut fixture, "merged and ready");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        staged_files: vec![(
            GameRole::BgeeSod,
            first_marker_path(&fixture),
            b"merged and ready\n".to_vec(),
        )],
        ..FakeDeps::default()
    };

    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(completed_step_ids(&fixture.request.created.managed_root)
        .contains(&"install:eefix-bg1".to_owned()));
}

#[test]
fn changed_game_content_does_not_stop_a_proven_install_or_its_resume() {
    let mut fixture = Fixture::new();
    add_first_run_marker(&mut fixture, "merged and ready");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        staged_files: vec![(
            GameRole::BgeeSod,
            first_marker_path(&fixture),
            b"player-edited text\n".to_vec(),
        )],
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(completed_step_ids(&fixture.request.created.managed_root)
        .contains(&"install:eefix-bg1".to_owned()));
    assert!(deps
        .trace
        .iter()
        .any(|entry| entry.starts_with("build:eefix-bg2:")));
    let replay = SessionStore::open(&fixture.request.created.managed_root)
        .unwrap()
        .replay()
        .unwrap();
    assert_eq!(replay.unresolved_step(), None);
    assert!(replay.fresh_copy_required().is_none());
    let split = deps.trace.len();
    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(!deps.trace[split..]
        .iter()
        .any(|entry| entry.starts_with("build:") || entry.starts_with("run:")));
}

#[test]
fn unresolved_proven_install_completes_with_authored_markers() {
    let mut fixture = Fixture::new();
    add_first_run_marker(&mut fixture, "merged and ready");
    seed_unresolved(
        &fixture.request,
        THROUGH_MATERIALIZATION,
        "install:eefix-bg1",
    );
    write_marker_file(&fixture, b"merged and ready\n");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        recovery_results: VecDeque::from([InstallReconciliation::ProvenDone]),
        ..FakeDeps::default()
    };

    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(!deps
        .trace
        .iter()
        .any(|entry| entry.starts_with("build:eefix-bg1:")));
}

#[test]
fn missing_game_marker_does_not_repeat_a_proven_unresolved_install() {
    let mut fixture = Fixture::new();
    add_first_run_marker(&mut fixture, "merged and ready");
    seed_unresolved(
        &fixture.request,
        THROUGH_MATERIALIZATION,
        "install:eefix-bg1",
    );
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        recovery_results: VecDeque::from([InstallReconciliation::ProvenDone]),
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(!deps
        .trace
        .iter()
        .any(|entry| entry.starts_with("build:eefix-bg1:") || entry.starts_with("run:eefix-bg1:")));
    assert!(deps
        .trace
        .iter()
        .any(|entry| entry.starts_with("build:eefix-bg2:")));
    let replay = SessionStore::open(&fixture.request.created.managed_root)
        .unwrap()
        .replay()
        .unwrap();
    assert_eq!(replay.unresolved_step(), None);
    assert!(replay.fresh_copy_required().is_none());
}

#[test]
fn last_failed_proven_install_completes_with_authored_markers() {
    let mut fixture = Fixture::new();
    add_first_run_marker(&mut fixture, "merged and ready");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: VecDeque::from([InstallReconciliation::retry(vec![0, 2])]),
        // The player stops at the failed run.
        answers: answer(QuestionKind::FailedComponent, Answer::No),
        ..FakeDeps::default()
    };
    assert!(matches!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Failed { ref step_id, .. } if step_id == "install:eefix-bg1"
    ));
    write_marker_file(&fixture, b"merged and ready\n");
    deps.recovery_results
        .push_back(InstallReconciliation::ProvenDone);
    let split = deps.trace.len();

    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(!deps.trace[split..]
        .iter()
        .any(|entry| entry.starts_with("build:eefix-bg1:")));
}

#[test]
fn missing_game_marker_does_not_repeat_a_failed_install_later_proven_done() {
    let mut fixture = Fixture::new();
    add_first_run_marker(&mut fixture, "merged and ready");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: VecDeque::from([InstallReconciliation::retry(vec![0, 2])]),
        answers: answer(QuestionKind::FailedComponent, Answer::No),
        ..FakeDeps::default()
    };
    assert!(
        matches!(run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Failed { ref step_id, .. } if step_id == "install:eefix-bg1")
    );
    deps.recovery_results
        .push_back(InstallReconciliation::ProvenDone);
    let split = deps.trace.len();
    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(!deps.trace[split..]
        .iter()
        .any(|entry| entry.starts_with("build:eefix-bg1:") || entry.starts_with("run:eefix-bg1:")));
    assert!(deps.trace[split..]
        .iter()
        .any(|entry| entry.starts_with("build:eefix-bg2:")));
}

#[test]
fn failure_stops_at_the_same_step_and_resume_does_not_repeat_completed_work() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        fail_once: Some("stage:bg2".to_owned()),
        ..FakeDeps::default()
    };

    let first = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
    assert!(matches!(
        first,
        CampaignOutcome::Failed { ref step_id, .. } if step_id == "stage:bg2"
    ));
    assert!(!deps
        .trace
        .iter()
        .any(|entry| entry.starts_with("materialize:")));
    assert!(!deps.trace.iter().any(|entry| entry.starts_with("run:")));
    assert_eq!(deps.receipts.len(), 1);
    assert!(matches!(
        &deps.receipts[0].outcome,
        ReceiptDraftOutcome::Failed { step_id, detail }
            if step_id == "stage:bg2" && detail.contains("injected failure")
    ));
    assert_ne!(
        deps.receipts[0].attempt_id,
        fixture.request.created.attempt_id
    );

    let split = deps.trace.len();
    let second = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
    assert_eq!(second, CampaignOutcome::Complete);
    let resumed = &deps.trace[split..];
    assert_eq!(resumed.first().map(String::as_str), Some("preflight"));
    assert!(resumed.iter().any(|entry| entry == "stage:bg2"));
    assert!(!resumed.iter().any(|entry| entry == "stage:bg1"));
    assert!(!resumed.iter().any(|entry| entry.starts_with("acquire:")));
    assert!(matches!(
        deps.receipts.last().map(|draft| &draft.outcome),
        Some(ReceiptDraftOutcome::Succeeded)
    ));
    assert_eq!(
        deps.receipts.last().unwrap().attempt_id,
        fixture.request.created.attempt_id
    );
}

fn add_first_run_marker(fixture: &mut Fixture, marker: &str) {
    fixture.request.plan.runs[0]
        .postconditions
        .push(Postcondition::TextFileMarkers {
            path: "override/dlc-merge.txt".to_owned(),
            required: vec![marker.to_owned()],
            forbidden: vec!["old state".to_owned()],
            max_bytes: 4096,
        });
    fixture.request.created.plan_sha256 = plan_digest(&fixture.request.plan).unwrap();
}

fn first_marker_path(fixture: &Fixture) -> PathBuf {
    fixture
        .request
        .created
        .staged_bg1
        .as_ref()
        .unwrap()
        .join("override/dlc-merge.txt")
}

fn write_marker_file(fixture: &Fixture, contents: &[u8]) {
    let path = first_marker_path(fixture);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

#[test]
fn receipt_bytes_are_stable_after_a_crash_before_ledger_completion() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let mut first_deps = FakeDeps {
        now: 100,
        ..FakeDeps::default()
    };

    assert_eq!(
        run_campaign(&fixture.request, &mut first_deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    let first = first_deps.receipts.last().unwrap().clone();
    let replay = SessionStore::open(&fixture.request.created.managed_root)
        .unwrap()
        .replay()
        .unwrap();
    let last = replay.records.last().unwrap();
    assert!(matches!(
        last.event,
        SessionEvent::StepCompleted { ref step_id, .. } if step_id == "receipt"
    ));
    std::fs::remove_file(
        fixture
            .request
            .created
            .managed_root
            .join(".chriz/ledger")
            .join(format!("{:010}.json", last.sequence)),
    )
    .unwrap();

    let mut resumed_deps = FakeDeps {
        now: 900,
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed_deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );

    assert_eq!(resumed_deps.receipts, vec![first]);
}

#[test]
fn resumed_preflight_failure_gets_its_own_terminal_receipt() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        fail_once: Some("stage:bg2".to_owned()),
        ..FakeDeps::default()
    };
    assert!(matches!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Failed { ref step_id, .. } if step_id == "stage:bg2"
    ));
    let first_id = deps.receipts.last().unwrap().attempt_id.clone();

    deps.fail_once = Some("preflight".to_owned());
    assert!(matches!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Failed { ref step_id, .. } if step_id == "preflight"
    ));

    let resumed = deps.receipts.last().unwrap();
    assert!(matches!(
        &resumed.outcome,
        ReceiptDraftOutcome::Failed { step_id, .. } if step_id == "preflight"
    ));
    assert_ne!(resumed.attempt_id, first_id);
}

#[test]
fn unresolved_materialization_reconciles_in_place_before_later_steps() {
    let fixture = Fixture::new();
    seed_unresolved(&fixture.request, THROUGH_STAGING, "materialize:eefix:bg1");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
    assert_eq!(outcome, CampaignOutcome::Complete);
    assert_eq!(
        deps.trace
            .iter()
            .filter(|entry| entry.as_str() == "materialize:eefix:bg1")
            .count(),
        1
    );
}

#[test]
fn unresolved_materialization_with_unknown_or_truncated_bytes_requires_a_fresh_copy() {
    for reason in ["unknown published path", "recorded file is truncated"] {
        let fixture = Fixture::new();
        seed_unresolved(&fixture.request, THROUGH_STAGING, "materialize:eefix:bg1");
        let sink = RecordingSink::default();
        let mut deps = FakeDeps {
            materialization_results: VecDeque::from([MaterializationOutcome::FreshCopyRequired {
                reason: reason.to_owned(),
            }]),
            ..FakeDeps::default()
        };

        let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
        assert!(matches!(
            outcome,
            CampaignOutcome::FreshCopyRequired { reason: ref found, .. } if found == reason
        ));
        assert!(matches!(
            deps.receipts.last().map(|draft| &draft.outcome),
            Some(ReceiptDraftOutcome::FreshCopyRequired { detail, .. }) if detail == reason
        ));
        assert!(!deps.trace.iter().any(|entry| entry.starts_with("run:")));
        let replay = SessionStore::open(&fixture.request.created.managed_root)
            .unwrap()
            .replay()
            .unwrap();
        assert_eq!(replay.unresolved_step(), None);
        let seal = replay
            .fresh_copy_required()
            .expect("fresh-copy verdict must be durable");
        assert_eq!(seal.step_id, "materialize:eefix:bg1");
        assert_eq!(seal.detail, reason);

        deps.trace.clear();
        let receipt = deps.receipts.last().cloned().unwrap();
        let resumed = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
        assert_eq!(resumed, outcome);
        assert_eq!(deps.trace, vec!["receipt"]);
        assert_eq!(deps.receipts.last(), Some(&receipt));
    }
}

fn interrupt_first_install() -> (Fixture, RecordingSink, FakeDeps) {
    let fixture = Fixture::new();
    seed_unresolved(
        &fixture.request,
        THROUGH_MATERIALIZATION,
        "install:eefix-bg1",
    );
    let sink = RecordingSink::default();
    (fixture, sink, FakeDeps::default())
}

#[test]
fn unresolved_run_with_complete_tail_is_recorded_without_rerunning() {
    let (fixture, sink, mut deps) = interrupt_first_install();
    deps.recovery_results
        .push_back(InstallReconciliation::ProvenDone);
    let builds_before = deps
        .trace
        .iter()
        .filter(|entry| entry.starts_with("build:eefix-bg1:"))
        .count();

    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(
        deps.trace
            .iter()
            .filter(|entry| entry.starts_with("build:eefix-bg1:"))
            .count(),
        builds_before
    );
}

#[test]
fn unresolved_run_retries_unchanged_attempt_or_only_the_strict_remaining_suffix() {
    for (recovery, expected_build) in [
        (
            InstallReconciliation::retry(vec![0, 2]),
            "build:eefix-bg1:[0, 2]",
        ),
        (
            InstallReconciliation::PartialPrefix { remaining: vec![2] },
            "build:eefix-bg1:[2]",
        ),
    ] {
        let (fixture, sink, mut deps) = interrupt_first_install();
        deps.recovery_results.push_back(recovery);

        assert_eq!(
            run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
            CampaignOutcome::Complete
        );
        assert!(deps.trace.iter().any(|entry| entry == expected_build));
    }
}

#[test]
fn disturbed_or_ambiguous_run_evidence_never_spawns_again() {
    for reason in [
        "component removed",
        "unexpected extra component",
        "component order changed",
        "debug evidence incomplete",
    ] {
        let (fixture, sink, mut deps) = interrupt_first_install();
        deps.recovery_results
            .push_back(InstallReconciliation::FreshCopyRequired {
                reason: reason.to_owned(),
            });
        let runs_before = deps
            .trace
            .iter()
            .filter(|entry| entry.as_str() == "run:eefix-bg1")
            .count();

        let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
        assert!(matches!(
            outcome,
            CampaignOutcome::FreshCopyRequired { reason: ref found, .. } if found == reason
        ));
        assert_eq!(
            deps.trace
                .iter()
                .filter(|entry| entry.as_str() == "run:eefix-bg1")
                .count(),
            runs_before
        );
    }
}

#[test]
fn invocation_build_and_spawn_each_receive_an_immediate_safety_recheck() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();
    run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    for run in ["eefix-bg1", "eefix-bg2", "eet-core"] {
        let build = deps
            .trace
            .iter()
            .position(|entry| entry.starts_with(&format!("build:{run}:")))
            .unwrap();
        assert_eq!(
            deps.trace[build - 1],
            format!("check:invocation-build:install:{run}")
        );
        let spawn = deps
            .trace
            .iter()
            .position(|entry| entry == &format!("run:{run}"))
            .unwrap();
        assert_eq!(
            deps.trace[spawn - 1],
            format!("check:process-spawn:install:{run}")
        );
    }
}

#[test]
fn rejected_install_guards_record_the_exact_suffix_without_starting_the_runner() {
    for kind in ["invocation-build", "process-spawn"] {
        let fixture = Fixture::new();
        let sink = RecordingSink::default();
        let mut deps = FakeDeps {
            fail_once: Some(format!("check:{kind}:install:eefix-bg2")),
            ..FakeDeps::default()
        };
        let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
        assert!(
            matches!(outcome, CampaignOutcome::Failed { ref step_id, .. } if step_id == "install:eefix-bg2")
        );
        assert!(deps
            .trace
            .contains(&format!("pre-spawn-failure:{kind}:eefix-bg2:[0, 2]")));
        assert!(!deps.trace.contains(&"run:eefix-bg2".to_owned()));
        let terminal = SessionStore::open(&fixture.request.created.managed_root)
            .unwrap()
            .replay()
            .unwrap();
        assert!(
            matches!(&terminal.records.last().unwrap().event, SessionEvent::StepFailed { detail, .. }
            if detail.contains("Pre-spawn evidence SHA-256: "))
        );
        deps.trace.clear();
        assert_eq!(
            run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
            CampaignOutcome::Complete
        );
        assert!(!deps.trace.contains(&"run:eefix-bg1".to_owned()));
        assert_eq!(
            deps.trace
                .iter()
                .filter(|entry| entry.as_str() == "run:eefix-bg2")
                .count(),
            1
        );
    }
}

#[test]
fn a_nonempty_unclaimed_root_is_rejected_before_any_campaign_service_runs() {
    let fixture = Fixture::new();
    std::fs::create_dir(&fixture.request.created.managed_root).unwrap();
    std::fs::write(
        fixture.request.created.managed_root.join("foreign.txt"),
        b"not ours",
    )
    .unwrap();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();

    let error = run_campaign(&fixture.request, &mut deps, &sink).unwrap_err();
    assert!(error.to_string().contains("not empty"), "{error}");
    assert!(deps.trace.is_empty());
    assert!(!fixture.request.created.managed_root.join(".chriz").exists());
}

#[test]
fn a_file_in_the_missing_destination_chain_is_rejected_without_creating_below_it() {
    let mut fixture = Fixture::new();
    let blocking_parent = fixture._temp.path().join("Games");
    std::fs::write(&blocking_parent, b"not a directory").unwrap();
    fixture.request.created.managed_root = blocking_parent.join("managed-alpha");
    fixture.request.created.staged_bg1 = Some(fixture.request.created.managed_root.join("bg1"));
    fixture.request.created.staged_bg2 = fixture.request.created.managed_root.join("game");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();

    let error = run_campaign(&fixture.request, &mut deps, &sink).unwrap_err();

    assert!(
        error.to_string().contains("direct non-reparse directory"),
        "{error}"
    );
    assert!(blocking_parent.is_file());
    assert!(!fixture.request.created.managed_root.exists());
    assert!(deps.trace.is_empty());
}

#[test]
fn a_link_in_the_missing_destination_chain_is_rejected_without_creating_below_it() {
    let mut fixture = Fixture::new();
    let actual_parent = fixture._temp.path().join("actual-parent");
    let linked_parent = fixture._temp.path().join("linked-parent");
    std::fs::create_dir(&actual_parent).unwrap();
    create_directory_link(&actual_parent, &linked_parent).expect("create linked parent fixture");
    fixture.request.created.managed_root = linked_parent.join("managed-alpha");
    fixture.request.created.staged_bg1 = Some(fixture.request.created.managed_root.join("bg1"));
    fixture.request.created.staged_bg2 = fixture.request.created.managed_root.join("game");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();

    let error = run_campaign(&fixture.request, &mut deps, &sink).unwrap_err();

    assert!(
        error.to_string().contains("direct non-reparse directory"),
        "{error}"
    );
    assert!(!actual_parent.join("managed-alpha").exists());
    assert!(deps.trace.is_empty());
}

#[test]
fn lock_registry_inside_the_target_is_rejected_without_creating_the_target() {
    let mut fixture = Fixture::new();
    fixture.request.registry_root = fixture.request.created.managed_root.join("registry");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();

    let error = run_campaign(&fixture.request, &mut deps, &sink).unwrap_err();
    assert!(error.to_string().contains("registry"), "{error}");
    assert!(!fixture.request.created.managed_root.exists());
    assert!(deps.trace.is_empty());
}

#[test]
fn final_identity_or_receipt_failure_never_claims_campaign_completion() {
    for failed_step in ["identity:final", "receipt"] {
        let fixture = Fixture::new();
        let sink = RecordingSink::default();
        let mut deps = FakeDeps {
            fail_once: Some(failed_step.to_owned()),
            ..FakeDeps::default()
        };

        let first = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
        assert!(matches!(
            first,
            CampaignOutcome::Failed { ref step_id, .. } if step_id == failed_step
        ));
        if failed_step == "identity:final" {
            assert!(!deps.trace.iter().any(|entry| entry == "verify:final"));
        }
        assert!(matches!(
            deps.receipts.last().map(|draft| &draft.outcome),
            Some(ReceiptDraftOutcome::Failed { step_id, .. }) if step_id == failed_step
        ));

        assert_eq!(
            run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
            CampaignOutcome::Complete
        );
    }
}

#[test]
fn changed_plan_and_out_of_order_ledger_fail_before_execution() {
    let mut changed = Fixture::new();
    changed.request.plan.runs[0].components.push(99);
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();
    let error = run_campaign(&changed.request, &mut deps, &sink).unwrap_err();
    assert!(error.to_string().contains("plan digest"), "{error}");
    assert!(!changed.request.created.managed_root.exists());
    assert!(deps.trace.is_empty());

    let fixture = Fixture::new();
    std::fs::create_dir(&fixture.request.created.managed_root).unwrap();
    let store = SessionStore::create(
        &fixture.request.created.managed_root,
        SessionEvent::Created(Box::new(fixture.request.created.clone())),
    )
    .unwrap();
    store
        .append(SessionEvent::StepStarted {
            step_id: "acquire:eefix".to_owned(),
            attempt: 1,
        })
        .unwrap();
    let mut deps = FakeDeps::default();
    let error = run_campaign(&fixture.request, &mut deps, &sink).unwrap_err();
    assert!(error.to_string().contains("out of order"), "{error}");
    assert!(deps.trace.is_empty());
}

#[test]
fn selected_pipeline_steps_never_emit_skipped() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();
    run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert!(!sink.events().iter().any(|event| matches!(
        event,
        EngineEvent::StepFinished {
            outcome: bg_engine::events::StepOutcome::Skipped,
            ..
        }
    )));
}

impl Fixture {
    /// The default campaign plus a run that needs `eefix` and one that loses a component
    /// without it, for leaving `eefix` out.
    fn with_dependents() -> Self {
        let mut fixture = Self::new();
        let request = &mut fixture.request;
        request.plan.runs.push(executable_run(
            "tweaks-bg2",
            "tweaks",
            GameRoot::Bg2,
            Phase::Main,
            &[1, 2],
            vec![],
        ));
        request.plan.runs.push(executable_run(
            "needs-eefix",
            "needs-eefix",
            GameRoot::Bg2,
            Phase::Main,
            &[0],
            vec![],
        ));
        request.created.plan_sha256 = plan_digest(&request.plan).unwrap();
        request
            .created
            .artifact_identities
            .extend([frozen("tweaks", "77", 50), frozen("needs-eefix", "88", 60)]);
        request
            .created
            .artifact_identities
            .sort_by(|left, right| left.id.cmp(&right.id));
        fixture
    }

    /// The plan that remains without `eefix`: EET core and the reduced tweaks run.
    fn plan_without_eefix(&self) -> InstallPlan {
        let mut tweaks = self.request.plan.runs[3].clone();
        tweaks.components = vec![1];
        InstallPlan {
            runs: vec![self.request.plan.runs[2].clone(), tweaks],
        }
    }
}

const LAYOUT_CHANGED: &str = "its archive layout changed";

/// Dependencies of a recipe in which `needs-eefix` needs `eefix` and `tweaks-bg2` has one
/// component that needs it.
fn eefix_recipe_deps() -> FakeDeps {
    FakeDeps {
        dependents: BTreeMap::from([("eefix".to_owned(), vec!["needs-eefix".to_owned()])]),
        reduced: BTreeMap::from([("eefix".to_owned(), vec![("tweaks-bg2".to_owned(), vec![1])])]),
        ..FakeDeps::default()
    }
}

/// A preset answer for this run that continues without a mod whose download cannot be used,
/// as `--answer unusable-download=yes` gives it.
fn continue_without() -> BTreeMap<QuestionKind, (Answer, AnswerSource)> {
    BTreeMap::from([(
        QuestionKind::UnusableDownload,
        (Answer::Yes, AnswerSource::Preset),
    )])
}

fn eefix_unusable_deps() -> FakeDeps {
    FakeDeps {
        leave_out: BTreeMap::from([("eefix".to_owned(), LAYOUT_CHANGED.to_owned())]),
        answers: continue_without(),
        ..eefix_recipe_deps()
    }
}

fn left_out_records(root: &Path) -> Vec<(String, String, String, String)> {
    SessionStore::open(root)
        .unwrap()
        .replay()
        .unwrap()
        .records
        .into_iter()
        .filter_map(|record| match record.event {
            SessionEvent::ArtifactLeftOut {
                step_id,
                artifact_id,
                detail,
                plan_sha256,
                ..
            } => Some((step_id, artifact_id, detail, plan_sha256)),
            _ => None,
        })
        .collect()
}

#[test]
fn an_unusable_download_leaves_out_its_mod_and_everything_that_needs_it() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = eefix_unusable_deps();

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert_eq!(outcome, CampaignOutcome::Complete);
    let root = &fixture.request.created.managed_root;
    assert_eq!(
        completed_step_ids(root),
        vec![
            "preflight",
            "acquire:weidu",
            "acquire:eet",
            "acquire:tweaks",
            "stage:bg1",
            "stage:bg2",
            "materialize:eet:bg2",
            "materialize:tweaks:bg2",
            "install:eet-core",
            "install:tweaks-bg2",
            "identity:final",
            "verify:final",
            "receipt",
        ]
    );
    let remaining = fixture.plan_without_eefix();
    assert_eq!(
        left_out_records(root),
        vec![(
            "acquire:eefix".to_owned(),
            "eefix".to_owned(),
            LAYOUT_CHANGED.to_owned(),
            plan_digest(&remaining).unwrap(),
        )]
    );
    assert!(
        !deps.trace.iter().any(|entry| entry == "acquire:needs-eefix"
            || entry.starts_with("materialize:eefix")
            || entry.starts_with("materialize:needs-eefix")
            || entry.starts_with("build:eefix")
            || entry.starts_with("build:needs-eefix"))
    );
    assert!(deps
        .trace
        .iter()
        .any(|entry| entry == "build:tweaks-bg2:[1]"));
    assert_eq!(deps.verified_plans, vec![remaining.clone()]);
    let receipt = deps.receipts.last().unwrap();
    assert_eq!(receipt.plan, fixture.request.plan);
    assert_eq!(receipt.installed_plan.as_ref(), Some(&remaining));
    assert_eq!(receipt.effective_plan(), &remaining);
    assert_eq!(
        receipt.left_out,
        vec![LeftOutArtifact {
            artifact_id: "eefix".to_owned(),
            reason: LAYOUT_CHANGED.to_owned(),
            runs: vec![
                "eefix-bg1".to_owned(),
                "eefix-bg2".to_owned(),
                "needs-eefix".to_owned(),
            ],
            changed_runs: vec!["tweaks-bg2".to_owned()],
            failed_run: None,
        }]
    );
    let events = sink.events();
    assert!(events.contains(&EngineEvent::ArtifactLeftOut {
        artifact_id: "eefix".to_owned(),
        reason: LAYOUT_CHANGED.to_owned(),
        runs: vec![
            "eefix-bg1".to_owned(),
            "eefix-bg2".to_owned(),
            "needs-eefix".to_owned(),
        ],
        changed_runs: vec!["tweaks-bg2".to_owned()],
        failed_run: None,
    }));
    assert!(events.contains(&EngineEvent::StepFinished {
        id: "acquire:eefix".to_owned(),
        outcome: StepOutcome::Skipped,
    }));
}

#[test]
fn resume_after_a_left_out_mod_rebuilds_the_recorded_plan_without_deciding_again() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let controls = RunnerControlHandle::new();
    let mut deps = FakeDeps {
        pause_after_operation: Some("acquire:weidu".to_owned()),
        controls: Some(controls.clone()),
        ..eefix_unusable_deps()
    };
    assert_eq!(
        run_campaign_controlled(&fixture.request, &mut deps, &sink, &controls).unwrap(),
        CampaignOutcome::Paused {
            after_step_id: "acquire:weidu".to_owned(),
        }
    );

    // The download may be usable now; the recorded decision still stands, and nothing that
    // the remaining plan does not need is acquired, staged or installed.
    let mut resumed = eefix_recipe_deps();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    // The ledger says what remains; nothing derives it again.
    assert!(resumed.plan_without_calls.is_empty());
    assert_eq!(resumed.acquisitions.get("eefix"), None);
    assert_eq!(resumed.acquisitions.get("needs-eefix"), None);
    assert_eq!(resumed.acquisitions.get("eet"), Some(&1));
    assert!(resumed
        .trace
        .iter()
        .any(|entry| entry == "build:tweaks-bg2:[1]"));
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.left_out.len(), 1);
    assert_eq!(receipt.installed_plan, Some(fixture.plan_without_eefix()));

    // Once complete, resuming again does not repeat any work.
    let mut again = eefix_recipe_deps();
    assert_eq!(
        run_campaign(&fixture.request, &mut again, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(again.acquisitions.is_empty());
}

/// An app update may derive the plan without a mod differently, or not at all. A campaign that
/// left the mod out before still resumes with exactly the plan it recorded then.
#[test]
fn resume_keeps_the_recorded_plan_when_the_app_now_derives_it_differently() {
    for changed in [
        // No longer drops `needs-eefix`, and keeps all of `tweaks-bg2`.
        FakeDeps::default(),
        // Cannot leave `eefix` out at all.
        FakeDeps {
            invalid_history: vec!["eefix".to_owned()],
            ..eefix_recipe_deps()
        },
    ] {
        let fixture = Fixture::with_dependents();
        let sink = RecordingSink::default();
        let controls = RunnerControlHandle::new();
        let mut deps = FakeDeps {
            pause_after_operation: Some("acquire:weidu".to_owned()),
            controls: Some(controls.clone()),
            ..eefix_unusable_deps()
        };
        run_campaign_controlled(&fixture.request, &mut deps, &sink, &controls).unwrap();

        let mut resumed = changed;
        assert_eq!(
            run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
            CampaignOutcome::Complete
        );
        assert!(resumed.plan_without_calls.is_empty());
        assert!(!resumed
            .trace
            .iter()
            .any(|entry| entry == "acquire:needs-eefix" || entry.starts_with("build:eefix")));
        assert!(resumed
            .trace
            .iter()
            .any(|entry| entry == "build:tweaks-bg2:[1]"));
        let receipt = resumed.receipts.last().unwrap();
        assert_eq!(receipt.installed_plan, Some(fixture.plan_without_eefix()));
        assert_eq!(resumed.verified_plans, vec![fixture.plan_without_eefix()]);
    }
}

const UNPLACEABLE: &str = "a file of its changed archive would overwrite another one";

#[test]
fn a_changed_archive_that_cannot_be_placed_is_left_out_while_none_of_its_files_are() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let controls = RunnerControlHandle::new();
    let mut deps = FakeDeps {
        unplaceable: BTreeMap::from([("materialize:eefix:bg1".to_owned(), UNPLACEABLE.to_owned())]),
        pause_after_operation: Some("materialize:eefix:bg1".to_owned()),
        controls: Some(controls.clone()),
        answers: continue_without(),
        ..eefix_recipe_deps()
    };

    assert_eq!(
        run_campaign_controlled(&fixture.request, &mut deps, &sink, &controls).unwrap(),
        CampaignOutcome::Paused {
            after_step_id: "materialize:eefix:bg1".to_owned(),
        }
    );
    let root = &fixture.request.created.managed_root;
    let remaining = fixture.plan_without_eefix();
    assert_eq!(
        left_out_records(root),
        vec![(
            "materialize:eefix:bg1".to_owned(),
            "eefix".to_owned(),
            UNPLACEABLE.to_owned(),
            plan_digest(&remaining).unwrap(),
        )]
    );
    let record = SessionStore::open(root)
        .unwrap()
        .replay()
        .unwrap()
        .records
        .into_iter()
        .find_map(|record| match record.event {
            SessionEvent::ArtifactLeftOut {
                removed_runs,
                changed_runs,
                ..
            } => Some((removed_runs, changed_runs)),
            _ => None,
        })
        .unwrap();
    assert_eq!(record.0, ["eefix-bg1", "eefix-bg2", "needs-eefix"]);
    assert_eq!(record.1, vec![remaining.runs[1].clone()]);
    assert!(sink.events().contains(&EngineEvent::StepFinished {
        id: "materialize:eefix:bg1".to_owned(),
        outcome: StepOutcome::Skipped,
    }));

    // The resume continues without the mod, even with a derivation that would keep it.
    let mut resumed = FakeDeps::default();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(resumed.plan_without_calls.is_empty());
    assert_eq!(
        completed_step_ids(root)
            .into_iter()
            .filter(|step| step.starts_with("materialize:") || step.starts_with("install:"))
            .collect::<Vec<_>>(),
        [
            "materialize:eet:bg2",
            "materialize:tweaks:bg2",
            "install:eet-core",
            "install:tweaks-bg2",
        ]
    );
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.installed_plan.as_ref(), Some(&remaining));
    assert_eq!(
        receipt.left_out,
        vec![LeftOutArtifact {
            artifact_id: "eefix".to_owned(),
            reason: UNPLACEABLE.to_owned(),
            runs: vec![
                "eefix-bg1".to_owned(),
                "eefix-bg2".to_owned(),
                "needs-eefix".to_owned(),
            ],
            changed_runs: vec!["tweaks-bg2".to_owned()],
            failed_run: None,
        }]
    );
}

/// A mod that is not installed must not leave files behind, because other mods detect mods by
/// their files. So once files of the mod, or of a mod leaving with it, are placed, the copy
/// needs a fresh start as before.
#[test]
fn a_changed_archive_cannot_be_left_out_once_files_of_it_or_its_dependents_are_placed() {
    for (step, dependents, placed) in [
        // Its files for BG1 are placed already.
        ("materialize:eefix:bg2", BTreeMap::new(), "eefix"),
        // EET core, placed already, needs the tweaks.
        (
            "materialize:tweaks:bg2",
            BTreeMap::from([("tweaks".to_owned(), vec!["eet-core".to_owned()])]),
            "eet",
        ),
    ] {
        let fixture = Fixture::with_dependents();
        let sink = RecordingSink::default();
        let mut deps = FakeDeps {
            unplaceable: BTreeMap::from([(step.to_owned(), UNPLACEABLE.to_owned())]),
            dependents,
            ..FakeDeps::default()
        };

        let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

        let CampaignOutcome::FreshCopyRequired { step_id, reason } = outcome else {
            panic!("expected a sealed copy, got {outcome:?}");
        };
        assert_eq!(step_id, step);
        assert!(reason.starts_with(UNPLACEABLE), "{reason}");
        assert!(
            reason.ends_with(&format!(
                "The installation cannot leave it out at this point, because files of {placed} are already in this game copy."
            )),
            "{reason}"
        );
        assert!(left_out_records(&fixture.request.created.managed_root).is_empty());
        assert!(deps.receipts.last().unwrap().left_out.is_empty());
    }
}

#[test]
fn a_part_the_recipe_cannot_do_without_seals_the_copy_when_it_cannot_be_placed() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        unplaceable: BTreeMap::from([("materialize:eefix:bg1".to_owned(), UNPLACEABLE.to_owned())]),
        invalid_history: vec!["eefix".to_owned()],
        ..eefix_recipe_deps()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    let CampaignOutcome::FreshCopyRequired { step_id, reason } = outcome else {
        panic!("expected a sealed copy, got {outcome:?}");
    };
    assert_eq!(step_id, "materialize:eefix:bg1");
    assert!(reason.contains(UNPLACEABLE), "{reason}");
    assert!(
        reason.contains("CEBG history cannot be reconciled"),
        "{reason}"
    );
    assert!(left_out_records(&fixture.request.created.managed_root).is_empty());
}

#[test]
fn an_unreconciled_cebg_record_stops_omission_before_mutation() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        invalid_history: vec!["eefix".to_owned()],
        ..eefix_unusable_deps()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    let CampaignOutcome::Failed { step_id, reason } = outcome else {
        panic!("expected a failed acquisition, got {outcome:?}");
    };
    assert_eq!(step_id, "acquire:eefix");
    assert!(reason.contains(LAYOUT_CHANGED), "{reason}");
    assert!(
        reason.contains("CEBG history cannot be reconciled"),
        "{reason}"
    );
    assert!(left_out_records(&fixture.request.created.managed_root).is_empty());
    assert!(deps.receipts.last().unwrap().left_out.is_empty());
}

#[test]
fn a_tool_whose_download_cannot_be_used_is_never_left_out() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        leave_out: BTreeMap::from([("weidu".to_owned(), "its program does not run".to_owned())]),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    let CampaignOutcome::Failed { step_id, reason } = outcome else {
        panic!("expected a failed acquisition, got {outcome:?}");
    };
    assert_eq!(step_id, "acquire:weidu");
    assert!(
        reason.starts_with(
            "The weidu mod can't be installed (the newer download is packed in a way the installer can't read), and the collection can't be installed without it. Try again later; if it keeps happening, please report it.\n"
        ),
        "{reason}"
    );
    assert!(reason.ends_with("its program does not run"), "{reason}");
    assert!(deps.plan_without_calls.is_empty());
    // A tool is never a question.
    assert_eq!(deps.answer_requests, 0);
}

#[test]
fn a_manual_pause_after_a_left_out_run_keeps_its_place_and_sees_the_remaining_plan() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let options = bg_engine::manual_checkpoint::options(&fixture.request.plan);
    let after = options
        .iter()
        .find(|option| option.run_id == "eefix-bg2" && option.component == 2)
        .unwrap()
        .clone();
    let mut deps = FakeDeps {
        checkpoints: vec![bg_engine::manual_checkpoint::FrozenCheckpoint {
            checkpoint_id: "manual-1".to_owned(),
            after_run_id: "eefix-bg2".to_owned(),
            after,
            next: None,
        }],
        ..eefix_unusable_deps()
    };

    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    let steps = completed_step_ids(&fixture.request.created.managed_root);
    let position = |id: &str| steps.iter().position(|step| step == id).unwrap();
    assert!(position("materialize:tweaks:bg2") < position("checkpoint:manual-1"));
    assert!(position("checkpoint:manual-1") < position("install:eet-core"));
    assert_eq!(deps.checkpoint_plans, vec![fixture.plan_without_eefix()]);
}

// ----- Questions about downloads -----

fn ledger_events(root: &Path) -> Vec<SessionEvent> {
    SessionStore::open(root)
        .unwrap()
        .replay()
        .unwrap()
        .records
        .into_iter()
        .map(|record| record.event)
        .collect()
}

/// Every question in the ledger: its step, id and question.
fn asked(root: &Path) -> Vec<(String, String, Question)> {
    ledger_events(root)
        .into_iter()
        .filter_map(|event| match event {
            SessionEvent::DecisionRequired {
                step_id,
                decision_id,
                question,
                ..
            } => Some((step_id, decision_id, *question)),
            _ => None,
        })
        .collect()
}

fn count_events(root: &Path, kind: &str) -> usize {
    ledger_events(root)
        .iter()
        .filter(|event| {
            let name = match event {
                SessionEvent::DecisionRequired { .. } => "required",
                SessionEvent::DecisionAnswered { .. } => "answered",
                SessionEvent::DecisionWithdrawn { .. } => "withdrawn",
                SessionEvent::ArtifactLeftOut { .. } => "left-out",
                _ => "",
            };
            name == kind
        })
        .count()
}

fn served(byte: &str) -> ServedDownload {
    ServedDownload {
        filename: Some("newer.zip".to_owned()),
        length: 99,
        sha256: byte.repeat(32),
        link: "https://example.invalid/newer.zip".to_owned(),
        final_url: "https://cdn.example.invalid/newer.zip".to_owned(),
        hosts: vec![
            "example.invalid".to_owned(),
            "cdn.example.invalid".to_owned(),
        ],
        unreviewed_hosts: vec!["cdn.example.invalid".to_owned()],
    }
}

fn answer(kind: QuestionKind, answer: Answer) -> BTreeMap<QuestionKind, (Answer, AnswerSource)> {
    BTreeMap::from([(kind, (answer, AnswerSource::Player))])
}

fn waiting_for(outcome: &CampaignOutcome) -> (String, String) {
    match outcome {
        CampaignOutcome::DecisionRequired {
            step_id,
            decision_id,
        } => (step_id.clone(), decision_id.clone()),
        other => panic!("expected a question, got {other:?}"),
    }
}

/// An unusable download stops the campaign with a question: the closure of everything that
/// needs the mod is computed and recorded, nothing is left out, no receipt is written, and the
/// step stays unresolved. A resume without an answer stops again with the same question and
/// asks the link nothing.
#[test]
fn an_unusable_download_asks_and_a_resume_without_an_answer_asks_nothing_again() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        leave_out: BTreeMap::from([("eefix".to_owned(), LAYOUT_CHANGED.to_owned())]),
        ..eefix_recipe_deps()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    let (step_id, decision_id) = waiting_for(&outcome);
    assert_eq!(step_id, "acquire:eefix");
    let questions = asked(root);
    assert_eq!(questions.len(), 1);
    let question = &questions[0].2;
    assert_eq!(questions[0].1, decision_id);
    assert_eq!(question.kind, QuestionKind::UnusableDownload);
    assert_eq!(question.reason.as_ref().unwrap().detail, LAYOUT_CHANGED);
    let closure = question.leave_out.as_ref().unwrap();
    let remaining = fixture.plan_without_eefix();
    assert_eq!(
        closure.removed_runs,
        ["eefix-bg1", "eefix-bg2", "needs-eefix"]
    );
    assert_eq!(closure.changed_runs, vec![remaining.runs[1].clone()]);
    assert_eq!(closure.plan_sha256, plan_digest(&remaining).unwrap());
    assert_eq!(count_events(root, "left-out"), 0);
    assert!(deps.receipts.is_empty());
    assert_eq!(
        SessionStore::open(root)
            .unwrap()
            .replay()
            .unwrap()
            .unresolved_step(),
        Some("acquire:eefix")
    );
    assert!(sink.events().contains(&EngineEvent::DecisionRequired {
        install_id: "install-alpha".to_owned(),
        step_id: "acquire:eefix".to_owned(),
        decision_id: decision_id.clone(),
    }));

    let mut resumed = eefix_recipe_deps();
    let again = run_campaign(&fixture.request, &mut resumed, &sink).unwrap();
    assert_eq!(waiting_for(&again), (step_id, decision_id));
    assert_eq!(resumed.acquisitions.get("eefix"), None);
    assert!(resumed.plan_without_calls.is_empty());
    assert_eq!(resumed.answer_requests, 1);
    assert_eq!(count_events(root, "required"), 1);
}

/// "Continue without them" leaves the mod out with exactly the closure recorded with the
/// question, even when the recipe would now derive another one, and is applied once.
#[test]
fn continuing_without_a_mod_applies_the_recorded_closure_once() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        leave_out: BTreeMap::from([("eefix".to_owned(), LAYOUT_CHANGED.to_owned())]),
        ..eefix_recipe_deps()
    };
    let (_, decision_id) = waiting_for(&run_campaign(&fixture.request, &mut deps, &sink).unwrap());

    // This derivation would keep everything; the recorded closure still applies.
    let mut resumed = FakeDeps {
        answers: answer(QuestionKind::UnusableDownload, Answer::Yes),
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(resumed.plan_without_calls.is_empty());
    assert_eq!(resumed.acquisitions.get("eefix"), None);
    let remaining = fixture.plan_without_eefix();
    assert_eq!(
        left_out_records(root),
        vec![(
            "acquire:eefix".to_owned(),
            "eefix".to_owned(),
            LAYOUT_CHANGED.to_owned(),
            plan_digest(&remaining).unwrap(),
        )]
    );
    assert_eq!(resumed.verified_plans, vec![remaining.clone()]);
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.installed_plan.as_ref(), Some(&remaining));
    assert_eq!(receipt.decisions.len(), 1);
    assert_eq!(receipt.decisions[0].decision_id, decision_id);
    assert_eq!(receipt.decisions[0].answer.answer, Answer::Yes);
    assert_eq!(receipt.decisions[0].answer.source, AnswerSource::Player);
    assert_eq!(receipt.decisions[0].outcome, DecisionOutcome::Applied);

    // Applied once: resuming the complete campaign changes nothing.
    let mut again = FakeDeps {
        answers: answer(QuestionKind::UnusableDownload, Answer::Yes),
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut again, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(count_events(root, "answered"), 1);
    assert_eq!(count_events(root, "left-out"), 1);
    assert_eq!(again.answer_requests, 0);
}

/// "Stop the installation" fails the attempt and nothing is left out. The next resume begins a
/// new attempt that asks the link again, and a question that comes back has a new id.
#[test]
fn stopping_ends_the_attempt_and_the_next_attempt_asks_again_with_a_new_id() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        answers: answer(QuestionKind::UnusableDownload, Answer::No),
        ..eefix_unusable_deps()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    let CampaignOutcome::Failed { step_id, reason } = outcome else {
        panic!("expected a stop, got {outcome:?}");
    };
    assert_eq!(step_id, "acquire:eefix");
    assert_eq!(
        reason,
        "You chose to stop: The eefix mod can't be installed (the newer download is packed in a way the installer can't read)."
    );
    assert_eq!(count_events(root, "left-out"), 0);
    let receipt = deps.receipts.last().unwrap();
    assert_eq!(receipt.decisions.len(), 1);
    assert_eq!(receipt.decisions[0].answer.answer, Answer::No);
    let first_id = receipt.decisions[0].decision_id.clone();

    let mut resumed = FakeDeps {
        leave_out: BTreeMap::from([("eefix".to_owned(), LAYOUT_CHANGED.to_owned())]),
        ..eefix_recipe_deps()
    };
    let (_, second_id) = waiting_for(&run_campaign(&fixture.request, &mut resumed, &sink).unwrap());
    assert_eq!(resumed.acquisitions.get("eefix"), Some(&1));
    assert_ne!(first_id, second_id);
    assert_eq!(count_events(root, "required"), 2);
}

/// The link of a mod serves a newer file that passed every check. Nothing records it until the
/// player says "Install it"; leaving the mod out is offered too, since the recipe can do without
/// it. The answer installs exactly that file.
#[test]
fn a_changed_download_is_installed_only_once_the_player_says_so() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        ..eefix_recipe_deps()
    };

    let (step_id, decision_id) =
        waiting_for(&run_campaign(&fixture.request, &mut deps, &sink).unwrap());
    assert_eq!(step_id, "acquire:eefix");
    let question = asked(root).remove(0).2;
    assert_eq!(question.kind, QuestionKind::ChangedDownload);
    assert_eq!(question.served, Some(served("ab")));
    assert!(question.offers(Answer::LeaveOut));
    assert!(deps.answered_calls.is_empty());

    let mut resumed = FakeDeps {
        answers: answer(QuestionKind::ChangedDownload, Answer::Yes),
        ..eefix_recipe_deps()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(resumed.answered_calls, ["eefix"]);
    assert_eq!(resumed.acquisitions.get("eefix"), None);
    assert!(completed_step_ids(root).contains(&"acquire:eefix".to_owned()));
    assert_eq!(count_events(root, "left-out"), 0);
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.decisions.len(), 1);
    assert_eq!(receipt.decisions[0].decision_id, decision_id);
    assert_eq!(receipt.installed_plan, None);
}

/// "Leave it out and continue" for an optional mod with a changed download leaves it and
/// everything that needs it out, exactly as for an unusable download.
#[test]
fn leaving_out_an_optional_changed_mod_takes_what_needs_it_along() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        answers: answer(QuestionKind::ChangedDownload, Answer::LeaveOut),
        ..eefix_recipe_deps()
    };

    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(deps.plan_without_calls, vec![vec!["eefix".to_owned()]]);
    assert!(deps.answered_calls.is_empty());
    let remaining = fixture.plan_without_eefix();
    let records = left_out_records(root);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].0, "acquire:eefix");
    assert_eq!(
        records[0].2,
        format!(
            "its official link now serves a different file (newer.zip, 99 bytes, SHA-256 {}), and you chose to leave it out",
            "ab".repeat(32)
        )
    );
    assert_eq!(records[0].3, plan_digest(&remaining).unwrap());
    let receipt = deps.receipts.last().unwrap();
    assert_eq!(receipt.installed_plan.as_ref(), Some(&remaining));
    assert_eq!(
        receipt.left_out[0].runs,
        ["eefix-bg1", "eefix-bg2", "needs-eefix"]
    );
    assert_eq!(receipt.decisions[0].answer.answer, Answer::LeaveOut);
}

/// A history reconciliation failure cannot be bypassed by leaving out a changed download.
#[test]
fn a_changed_download_cannot_be_omitted_with_unreconciled_cebg_history() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        invalid_history: vec!["eefix".to_owned()],
        answers: BTreeMap::from([(
            QuestionKind::ChangedDownload,
            (Answer::LeaveOut, AnswerSource::Preset),
        )]),
        ..eefix_recipe_deps()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    waiting_for(&outcome);
    let question = asked(root).remove(0).2;
    assert_eq!(question.leave_out, None);
    assert!(!question.offers(Answer::LeaveOut));
    assert_eq!(count_events(root, "answered"), 0);
    assert_eq!(count_events(root, "left-out"), 0);
}

/// A changed tool from a host the recipe did not review is asked about, never offered for
/// leaving out, and "Install it" gets exactly that file.
#[test]
fn an_unreviewed_tool_is_asked_about_and_never_offered_for_leaving_out() {
    let fixture = Fixture::new();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        unreviewed: BTreeMap::from([("weidu".to_owned(), served("cd"))]),
        ..FakeDeps::default()
    };

    let (step_id, _) = waiting_for(&run_campaign(&fixture.request, &mut deps, &sink).unwrap());
    assert_eq!(step_id, "acquire:weidu");
    let question = asked(root).remove(0).2;
    assert_eq!(question.kind, QuestionKind::UnreviewedTool);
    assert_eq!(question.artifact_kind, ArtifactKind::Tool);
    assert_eq!(question.leave_out, None);
    assert!(deps.plan_without_calls.is_empty());

    let mut resumed = FakeDeps {
        answers: answer(QuestionKind::UnreviewedTool, Answer::Yes),
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(resumed.answered_calls, ["weidu"]);
}

/// "Install it" finds the reviewed file back in the download folder: the question is withdrawn,
/// never applied, and the step acquires as without it.
#[test]
fn a_question_whose_reviewed_file_is_back_is_withdrawn_and_never_applied() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        answers: answer(QuestionKind::ChangedDownload, Answer::Yes),
        answered_results: VecDeque::from([AnsweredAcquisition::Withdrawn {
            why: "the reviewed version is in the download folder now".to_owned(),
        }]),
        ..eefix_recipe_deps()
    };

    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(count_events(root, "withdrawn"), 1);
    assert_eq!(count_events(root, "left-out"), 0);
    // Acquired once for the question and once more after the withdrawal.
    assert_eq!(deps.acquisitions.get("eefix"), Some(&2));
    let receipt = deps.receipts.last().unwrap();
    assert_eq!(receipt.decisions[0].outcome, DecisionOutcome::Withdrawn);
}

/// A link that keeps switching back to the same changed file after its question was withdrawn
/// ends the attempt instead of asking in a loop.
#[test]
fn the_same_question_after_its_withdrawal_ends_the_attempt() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        answers: answer(QuestionKind::ChangedDownload, Answer::Yes),
        answered_results: VecDeque::from([AnsweredAcquisition::Withdrawn {
            why: "its link serves the reviewed version again".to_owned(),
        }]),
        keep_changed: true,
        ..eefix_recipe_deps()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    let CampaignOutcome::Failed { step_id, reason } = outcome else {
        panic!("expected a failed attempt, got {outcome:?}");
    };
    assert_eq!(step_id, "acquire:eefix");
    assert!(reason.contains("Its link keeps changing"), "{reason}");
}

/// The file the player accepted is gone and the link serves yet another one: the attempt ends,
/// and the next one asks anew with a new id.
#[test]
fn an_accepted_file_whose_link_changed_again_ends_the_attempt_and_is_asked_anew() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        answers: answer(QuestionKind::ChangedDownload, Answer::Yes),
        answered_results: VecDeque::from([AnsweredAcquisition::LinkChanged {
            detail: "its link changed again".to_owned(),
        }]),
        ..eefix_recipe_deps()
    };
    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
    assert!(matches!(
        outcome,
        CampaignOutcome::Failed { ref step_id, ref reason }
            if step_id == "acquire:eefix" && reason == "its link changed again"
    ));

    let mut resumed = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ef"))]),
        ..eefix_recipe_deps()
    };
    waiting_for(&run_campaign(&fixture.request, &mut resumed, &sink).unwrap());
    let questions = asked(root);
    assert_eq!(questions.len(), 2);
    assert_ne!(questions[0].1, questions[1].1);
    assert_eq!(questions[1].2.served, Some(served("ef")));
}

/// A placing step asks before it leaves a mod out whose files meet another mod's. On resume it
/// plans again: once the files can be placed after all, the question is withdrawn and the mod
/// is installed.
#[test]
fn a_placing_question_is_withdrawn_when_the_files_can_be_placed_after_all() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        unplaceable: BTreeMap::from([("materialize:eefix:bg1".to_owned(), UNPLACEABLE.to_owned())]),
        ..eefix_recipe_deps()
    };
    let (step_id, _) = waiting_for(&run_campaign(&fixture.request, &mut deps, &sink).unwrap());
    assert_eq!(step_id, "materialize:eefix:bg1");
    let question = asked(root).remove(0).2;
    assert_eq!(question.kind, QuestionKind::UnusableDownload);
    assert!(matches!(
        question.reason.as_ref().unwrap().kind,
        UnusableKind::Collision { .. }
    ));

    let mut resumed = FakeDeps {
        answers: answer(QuestionKind::UnusableDownload, Answer::Yes),
        ..eefix_recipe_deps()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(count_events(root, "withdrawn"), 1);
    assert_eq!(count_events(root, "left-out"), 0);
    assert!(completed_step_ids(root).contains(&"materialize:eefix:bg1".to_owned()));
    assert!(resumed
        .trace
        .iter()
        .any(|entry| entry.starts_with("build:eefix-bg1")));
}

/// The files of a placing step meet another obstacle on resume than the one asked about: the
/// old question is withdrawn and the new one asked, with a new id.
#[test]
fn a_placing_question_whose_obstacle_changed_is_asked_anew() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        unplaceable: BTreeMap::from([("materialize:eefix:bg1".to_owned(), UNPLACEABLE.to_owned())]),
        ..eefix_recipe_deps()
    };
    waiting_for(&run_campaign(&fixture.request, &mut deps, &sink).unwrap());

    let mut resumed = FakeDeps {
        unplaceable: BTreeMap::from([(
            "materialize:eefix:bg1".to_owned(),
            "another file is in the way now".to_owned(),
        )]),
        answers: BTreeMap::from([(
            QuestionKind::UnusableDownload,
            (Answer::Yes, AnswerSource::Player),
        )]),
        ..eefix_recipe_deps()
    };
    // The player's answer was for the first obstacle; the new question gets this run's answer.
    let outcome = run_campaign(&fixture.request, &mut resumed, &sink).unwrap();
    assert!(
        matches!(
            outcome,
            CampaignOutcome::Paused { .. } | CampaignOutcome::Complete
        ),
        "{outcome:?}"
    );
    let questions = asked(root);
    assert_eq!(questions.len(), 2);
    assert_ne!(questions[0].1, questions[1].1);
    assert_eq!(
        questions[1].2.reason.as_ref().unwrap().detail,
        "another file is in the way now"
    );
    assert_eq!(count_events(root, "withdrawn"), 1);
    let left_out = left_out_records(root);
    assert_eq!(left_out.len(), 1);
    assert_eq!(left_out[0].2, "another file is in the way now");
}

/// CEBG history reconciliation errors and unavailable tools remain blockers.
#[test]
fn unreconciled_cebg_records_and_unavailable_tools_never_ask() {
    for (fixture, mut deps, step) in [
        (
            Fixture::with_dependents(),
            FakeDeps {
                invalid_history: vec!["eefix".to_owned()],
                ..eefix_unusable_deps()
            },
            "acquire:eefix",
        ),
        (
            Fixture::new(),
            FakeDeps {
                leave_out: BTreeMap::from([(
                    "weidu".to_owned(),
                    "its program does not run".to_owned(),
                )]),
                answers: continue_without(),
                ..FakeDeps::default()
            },
            "acquire:weidu",
        ),
    ] {
        let sink = RecordingSink::default();
        let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
        let CampaignOutcome::Failed { step_id, reason } = outcome else {
            panic!("expected a stop, got {outcome:?}");
        };
        assert_eq!(step_id, step);
        assert!(
            reason.contains("and the collection can't be installed without it. Try again later; if it keeps happening, please report it."),
            "{reason}"
        );
        assert_eq!(deps.answer_requests, 0);
        assert!(asked(&fixture.request.created.managed_root).is_empty());
    }
}

/// The ledger keeps question events in its hash chain and refuses those that break the rules:
/// one standing question per step, answered once, withdrawn only while it stands, never ended
/// while unanswered, and with an id derived from its question.
#[test]
fn the_ledger_keeps_questions_in_order_and_refuses_broken_ones() {
    let fixture = Fixture::new();
    let root = fixture.request.created.managed_root.clone();
    std::fs::create_dir_all(&root).unwrap();
    let store = SessionStore::create(
        &root,
        SessionEvent::Created(Box::new(fixture.request.created.clone())),
    )
    .unwrap();
    store
        .append(SessionEvent::StepStarted {
            step_id: "acquire:eefix".to_owned(),
            attempt: 1,
        })
        .unwrap();
    let question = Question {
        kind: QuestionKind::ChangedDownload,
        artifact_id: "eefix".to_owned(),
        artifact_kind: ArtifactKind::Payload,
        reviewed: frozen("eefix", "33", 10),
        served: Some(served("ab")),
        reason: None,
        leave_out: None,
        failed: None,
    };
    let decision_id = question.decision_id("acquire:eefix", 1);
    let required = |id: &str| SessionEvent::DecisionRequired {
        step_id: "acquire:eefix".to_owned(),
        attempt: 1,
        decision_id: id.to_owned(),
        question: Box::new(question.clone()),
    };
    let answered = |answer: Answer| SessionEvent::DecisionAnswered {
        step_id: "acquire:eefix".to_owned(),
        attempt: 1,
        decision_id: decision_id.clone(),
        answer,
        source: AnswerSource::Player,
    };
    let completed = SessionEvent::StepCompleted {
        step_id: "acquire:eefix".to_owned(),
        attempt: 1,
    };

    assert!(store.append(answered(Answer::Yes)).is_err());
    assert!(store.append(required("0123456789abcdef")).is_err());
    assert!(store
        .append(SessionEvent::DecisionRequired {
            step_id: "acquire:eet".to_owned(),
            attempt: 1,
            decision_id: question.decision_id("acquire:eet", 1),
            question: Box::new(question.clone()),
        })
        .is_err());
    store.append(required(&decision_id)).unwrap();
    assert!(store.append(required(&decision_id)).is_err());
    assert!(store.append(completed.clone()).is_err());

    let replay = store.replay().unwrap();
    assert_eq!(replay.unresolved_step(), Some("acquire:eefix"));
    let pending = replay.pending_decision().unwrap();
    assert_eq!(pending.decision_id, decision_id);
    assert_eq!(pending.answer, None);
    // Every record survives a round trip through its JSON exactly.
    for record in &replay.records {
        let bytes = serde_json::to_vec(record).unwrap();
        assert_eq!(
            &serde_json::from_slice::<bg_engine::session::LedgerRecord>(&bytes).unwrap(),
            record
        );
    }

    store.append(answered(Answer::Yes)).unwrap();
    assert!(store.append(answered(Answer::No)).is_err());
    let pending = store.replay().unwrap().pending_decision().unwrap();
    assert_eq!(
        pending.answer.map(|recorded| recorded.answer),
        Some(Answer::Yes)
    );
    store
        .append(SessionEvent::DecisionWithdrawn {
            step_id: "acquire:eefix".to_owned(),
            attempt: 1,
            decision_id: decision_id.clone(),
            why: "the reviewed file is back".to_owned(),
        })
        .unwrap();
    assert_eq!(store.replay().unwrap().pending_decision(), None);
    assert!(store
        .append(SessionEvent::DecisionWithdrawn {
            step_id: "acquire:eefix".to_owned(),
            attempt: 1,
            decision_id: decision_id.clone(),
            why: "again".to_owned(),
        })
        .is_err());
    store.append(completed).unwrap();
    assert_eq!(store.replay().unwrap().unresolved_step(), None);
}

/// After "Stop", a link that works again on the next attempt is no question any more.
#[test]
fn a_link_that_works_again_on_the_next_attempt_asks_nothing() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        answers: answer(QuestionKind::UnusableDownload, Answer::No),
        ..eefix_unusable_deps()
    };
    assert!(matches!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Failed { .. }
    ));

    let mut resumed = eefix_recipe_deps();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(resumed.acquisitions.get("eefix"), Some(&1));
    assert_eq!(resumed.answer_requests, 0);
    assert_eq!(count_events(root, "required"), 1);
    assert_eq!(count_events(root, "left-out"), 0);
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.installed_plan, None);
    assert_eq!(receipt.decisions.len(), 1);
    assert_eq!(receipt.decisions[0].answer.answer, Answer::No);
}

/// The installation stopped right after it recorded an answer, before applying it: the resume
/// applies the recorded answer without asking for one again.
#[test]
fn a_recorded_answer_is_applied_after_a_stop_before_its_application() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        ..eefix_recipe_deps()
    };
    let (step_id, decision_id) =
        waiting_for(&run_campaign(&fixture.request, &mut deps, &sink).unwrap());
    SessionStore::open(root)
        .unwrap()
        .append(SessionEvent::DecisionAnswered {
            step_id,
            attempt: 1,
            decision_id,
            answer: Answer::Yes,
            source: AnswerSource::Player,
        })
        .unwrap();

    let mut resumed = eefix_recipe_deps();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(resumed.answer_requests, 0);
    assert_eq!(resumed.answered_calls, ["eefix"]);
    assert_eq!(count_events(root, "answered"), 1);
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.decisions[0].outcome, DecisionOutcome::Applied);
}

/// The installation stopped right after it withdrew a question, before it went on: the resume
/// acquires as without the question and asks nothing.
#[test]
fn a_withdrawn_question_stays_withdrawn_after_a_stop_before_the_step_went_on() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        ..eefix_recipe_deps()
    };
    let (step_id, decision_id) =
        waiting_for(&run_campaign(&fixture.request, &mut deps, &sink).unwrap());
    let store = SessionStore::open(root).unwrap();
    store
        .append(SessionEvent::DecisionAnswered {
            step_id: step_id.clone(),
            attempt: 1,
            decision_id: decision_id.clone(),
            answer: Answer::Yes,
            source: AnswerSource::Player,
        })
        .unwrap();
    store
        .append(SessionEvent::DecisionWithdrawn {
            step_id,
            attempt: 1,
            decision_id,
            why: "the reviewed version is in the download folder now".to_owned(),
        })
        .unwrap();

    let mut resumed = eefix_recipe_deps();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(resumed.answer_requests, 0);
    assert!(resumed.answered_calls.is_empty());
    assert_eq!(resumed.acquisitions.get("eefix"), Some(&1));
    assert_eq!(count_events(root, "required"), 1);
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.decisions[0].outcome, DecisionOutcome::Withdrawn);
}

/// "Leave it out" finds the reviewed archive back in the download folder: the question is
/// withdrawn and the mod installed, without asking its link while the question waited.
#[test]
fn leaving_out_a_mod_whose_reviewed_file_is_back_installs_it() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        ..eefix_recipe_deps()
    };
    waiting_for(&run_campaign(&fixture.request, &mut deps, &sink).unwrap());

    // Waiting asks neither the link nor the download folder.
    let mut waiting = eefix_recipe_deps();
    waiting_for(&run_campaign(&fixture.request, &mut waiting, &sink).unwrap());
    assert!(!waiting
        .trace
        .iter()
        .any(|entry| entry.starts_with("acquire:") || entry.starts_with("reviewed?:")));

    let mut resumed = FakeDeps {
        reviewed_back: vec!["eefix".to_owned()],
        answers: answer(QuestionKind::ChangedDownload, Answer::LeaveOut),
        ..eefix_recipe_deps()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(count_events(root, "withdrawn"), 1);
    assert_eq!(count_events(root, "left-out"), 0);
    assert_eq!(resumed.acquisitions.get("eefix"), Some(&1));
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.installed_plan, None);
    assert_eq!(receipt.decisions[0].outcome, DecisionOutcome::Withdrawn);
}

/// An answer whose application failed stays in the history but is not what took effect: the
/// answer of the next attempt is.
#[test]
fn only_the_answer_that_took_effect_is_applied_in_the_history() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ab"))]),
        answers: answer(QuestionKind::ChangedDownload, Answer::Yes),
        answered_results: VecDeque::from([AnsweredAcquisition::LinkChanged {
            detail: "its link changed again".to_owned(),
        }]),
        ..eefix_recipe_deps()
    };
    assert!(matches!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Failed { .. }
    ));
    let mut resumed = FakeDeps {
        changed: BTreeMap::from([("eefix".to_owned(), served("ef"))]),
        answers: answer(QuestionKind::ChangedDownload, Answer::Yes),
        ..eefix_recipe_deps()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    let receipt = resumed.receipts.last().unwrap();
    let outcomes = receipt
        .decisions
        .iter()
        .map(|decision| {
            (
                decision.question.served.clone().unwrap().sha256,
                decision.outcome,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes,
        [
            ("ab".repeat(32), DecisionOutcome::Failed),
            ("ef".repeat(32), DecisionOutcome::Applied),
        ]
    );
}

/// The ledger refuses histories whose answers contradict what follows: an answer the question
/// does not offer, a completed step after "No", a leave-out after "Install it", and a leave-out
/// that removes something else than the closure recorded with the question.
#[test]
fn the_ledger_refuses_answers_that_contradict_their_outcome() {
    let closure = bg_engine::decision::LeaveOutClosure {
        removed_runs: vec!["eefix-bg1".to_owned(), "eefix-bg2".to_owned()],
        changed_runs: Vec::new(),
        plan_sha256: "77".repeat(32),
    };
    let changed = Question {
        kind: QuestionKind::ChangedDownload,
        artifact_id: "eefix".to_owned(),
        artifact_kind: ArtifactKind::Payload,
        reviewed: frozen("eefix", "33", 10),
        served: Some(served("ab")),
        reason: None,
        leave_out: Some(closure.clone()),
        failed: None,
    };
    let tool = Question {
        kind: QuestionKind::UnreviewedTool,
        artifact_id: "weidu".to_owned(),
        artifact_kind: ArtifactKind::Tool,
        reviewed: frozen("weidu", "66", 40),
        served: Some(served("cd")),
        reason: None,
        leave_out: None,
        failed: None,
    };
    let left_out = |removed: Vec<String>| SessionEvent::ArtifactLeftOut {
        step_id: "acquire:eefix".to_owned(),
        attempt: 1,
        artifact_id: "eefix".to_owned(),
        detail: "left out".to_owned(),
        plan_sha256: "77".repeat(32),
        removed_runs: removed,
        changed_runs: Vec::new(),
        failed_run: None,
    };
    let completed = |step: &str| SessionEvent::StepCompleted {
        step_id: step.to_owned(),
        attempt: 1,
    };
    // (question, answer, the end that must be refused, an end that fits)
    let cases = [
        (
            tool.clone(),
            Answer::LeaveOut,
            None,
            SessionEvent::StepFailed {
                step_id: "acquire:weidu".to_owned(),
                attempt: 1,
                detail: "stopped".to_owned(),
            },
        ),
        (
            changed.clone(),
            Answer::No,
            Some(completed("acquire:eefix")),
            SessionEvent::StepFailed {
                step_id: "acquire:eefix".to_owned(),
                attempt: 1,
                detail: "You chose to stop".to_owned(),
            },
        ),
        (
            changed.clone(),
            Answer::Yes,
            Some(left_out(closure.removed_runs.clone())),
            completed("acquire:eefix"),
        ),
        (
            changed.clone(),
            Answer::LeaveOut,
            Some(left_out(vec!["eefix-bg1".to_owned()])),
            left_out(closure.removed_runs.clone()),
        ),
    ];
    for (question, given, refused, fits) in cases {
        let fixture = Fixture::new();
        let root = fixture.request.created.managed_root.clone();
        std::fs::create_dir_all(&root).unwrap();
        let store = SessionStore::create(
            &root,
            SessionEvent::Created(Box::new(fixture.request.created.clone())),
        )
        .unwrap();
        let step_id = format!("acquire:{}", question.artifact_id);
        store
            .append(SessionEvent::StepStarted {
                step_id: step_id.clone(),
                attempt: 1,
            })
            .unwrap();
        let decision_id = question.decision_id(&step_id, 1);
        store
            .append(SessionEvent::DecisionRequired {
                step_id: step_id.clone(),
                attempt: 1,
                decision_id: decision_id.clone(),
                question: Box::new(question.clone()),
            })
            .unwrap();
        let answered = store.append(SessionEvent::DecisionAnswered {
            step_id: step_id.clone(),
            attempt: 1,
            decision_id,
            answer: given,
            source: AnswerSource::Player,
        });
        if question.offers(given) {
            answered.unwrap();
        } else {
            assert!(answered.is_err(), "{given:?} is not offered");
            continue;
        }
        if let Some(refused) = refused {
            assert!(store.append(refused.clone()).is_err(), "{refused:?}");
        }
        store.append(fits).unwrap();
    }
}

/// What the first three runs of `Fixture::with_dependents` report, and then `tweaks-bg2`
/// failing at its first component.
fn tweaks_fail_at_their_first_component() -> VecDeque<InstallReconciliation> {
    VecDeque::from([
        InstallReconciliation::ProvenDone,
        InstallReconciliation::ProvenDone,
        InstallReconciliation::ProvenDone,
        InstallReconciliation::retry(vec![1, 2]),
    ])
}

fn needs_eefix_needs_the_tweaks() -> BTreeMap<String, Vec<String>> {
    BTreeMap::from([("tweaks-bg2".to_owned(), vec!["needs-eefix".to_owned()])])
}

/// The answer "Leave it out and continue".
fn leave_it_out() -> BTreeMap<QuestionKind, (Answer, AnswerSource)> {
    answer(QuestionKind::FailedComponent, Answer::LeaveOut)
}

/// The `WeiDU` runs of a campaign, as `build:<run>:<components>`.
fn builds(deps: &FakeDeps) -> Vec<&str> {
    deps.trace
        .iter()
        .filter(|entry| entry.starts_with("build:"))
        .map(String::as_str)
        .collect()
}

/// The run that failed, as its one left-out record says.
fn left_out_run(root: &Path) -> bg_engine::session::FailedRun {
    let mut records = ledger_events(root)
        .into_iter()
        .filter_map(|event| match event {
            SessionEvent::ArtifactLeftOut { failed_run, .. } => failed_run,
            _ => None,
        });
    let failed = records.next().expect("one run was left out");
    assert_eq!(records.next(), None);
    failed
}

/// A failed component stops the installation with a question and decides nothing: what would
/// leave with it is recorded with the question, and without an answer a resume only asks again.
/// The answer "Leave it out" applies what was recorded, and the run goes on with the components
/// that do not need the failed one.
#[test]
fn a_failed_component_waits_for_the_answer_and_a_resume_applies_it() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: tweaks_fail_at_their_first_component(),
        failed_dependents: needs_eefix_needs_the_tweaks(),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    let CampaignOutcome::DecisionRequired {
        step_id,
        decision_id,
    } = outcome
    else {
        panic!("{outcome:?}");
    };
    assert_eq!(step_id, "install:tweaks-bg2");
    assert_eq!(count_events(root, "left-out"), 0);
    assert!(!builds(&deps).iter().any(|run| run.contains("needs-eefix")));
    let questions = asked(root);
    assert_eq!(questions.len(), 1, "{questions:?}");
    assert_eq!(questions[0].1, decision_id);
    let question = &questions[0].2;
    assert_eq!(question.kind, QuestionKind::FailedComponent);
    assert_eq!(question.artifact_id, "tweaks");
    let failed = question.failed.as_ref().unwrap();
    assert_eq!(failed.run_id, "tweaks-bg2");
    assert_eq!(failed.components, vec![1, 2]);
    assert_eq!(failed.continues_with, vec![2]);
    assert_eq!(failed.required, None);
    let closure = question.leave_out.as_ref().unwrap();
    assert_eq!(closure.removed_runs, vec!["needs-eefix".to_owned()]);
    assert_eq!(closure.changed_runs.len(), 1);
    assert_eq!(closure.changed_runs[0].components, vec![2]);
    assert!(sink.events().iter().any(|event| matches!(
        event,
        EngineEvent::ConsoleLine { line, .. }
            if line.contains(&decision_id) && line.contains("could not be installed")
    )));

    // Without an answer a resume changes nothing and runs nothing.
    let mut waiting = FakeDeps::default();
    let outcome = run_campaign(&fixture.request, &mut waiting, &sink).unwrap();
    assert!(matches!(outcome, CampaignOutcome::DecisionRequired { .. }));
    assert_eq!(count_events(root, "required"), 1);
    assert!(builds(&waiting).is_empty(), "{:?}", waiting.trace);
    assert!(!waiting
        .trace
        .iter()
        .any(|entry| entry.starts_with("recover:")));

    let mut answered = FakeDeps {
        answers: leave_it_out(),
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut answered, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(builds(&answered), vec!["build:tweaks-bg2:[2]"]);
    assert!(answered.plan_without_parts_calls.is_empty());
    let mut tweaks = fixture.request.plan.runs[3].clone();
    tweaks.components = vec![2];
    let remaining = InstallPlan {
        runs: vec![
            fixture.request.plan.runs[0].clone(),
            fixture.request.plan.runs[1].clone(),
            fixture.request.plan.runs[2].clone(),
            tweaks,
        ],
    };
    assert_eq!(answered.verified_plans, vec![remaining.clone()]);
    let failed_run = left_out_run(root);
    assert_eq!(failed_run.components, vec![1, 2]);
    assert_eq!(failed_run.left_out, vec![1]);
    assert_eq!(failed_run.continues_with, vec![2]);
    let receipt = answered.receipts.last().unwrap();
    assert!(matches!(receipt.outcome, ReceiptDraftOutcome::Succeeded));
    assert_eq!(receipt.installed_plan.as_ref(), Some(&remaining));
    assert_eq!(receipt.left_out.len(), 1);
    assert_eq!(
        receipt.left_out[0].failed_run.as_deref(),
        Some("tweaks-bg2")
    );
    assert_eq!(receipt.left_out[0].runs, vec!["needs-eefix".to_owned()]);
    assert_eq!(
        receipt.left_out[0].changed_runs,
        vec!["tweaks-bg2".to_owned()]
    );
    assert_eq!(receipt.decisions.len(), 1);
    assert_eq!(receipt.decisions[0].outcome, DecisionOutcome::Applied);
    // The step goes on, so it is not over when the component is left out.
    assert!(!sink.events().contains(&EngineEvent::StepFinished {
        id: "install:tweaks-bg2".to_owned(),
        outcome: StepOutcome::Skipped,
    }));
    assert!(sink.events().contains(&EngineEvent::StepFinished {
        id: "install:tweaks-bg2".to_owned(),
        outcome: StepOutcome::Succeeded,
    }));
}

/// A stop right after the answer loses nothing: the record says which components the step goes
/// on with, whether it stopped before the next attempt began or during it.
#[test]
fn the_components_a_run_goes_on_with_survive_a_stop_after_the_answer() {
    for pause in [true, false] {
        let fixture = Fixture::with_dependents();
        let root = &fixture.request.created.managed_root;
        let sink = RecordingSink::default();
        let mut failing = FakeDeps {
            after_results: tweaks_fail_at_their_first_component(),
            ..FakeDeps::default()
        };
        run_campaign(&fixture.request, &mut failing, &sink).unwrap();

        let controls = RunnerControlHandle::new();
        if pause {
            controls.pause_after_boundary();
        }
        let mut answered = FakeDeps {
            answers: leave_it_out(),
            fail_once: (!pause).then(|| "build:tweaks-bg2".to_owned()),
            ..FakeDeps::default()
        };
        let outcome =
            run_campaign_controlled(&fixture.request, &mut answered, &sink, &controls).unwrap();
        if pause {
            assert!(
                matches!(&outcome, CampaignOutcome::Paused { .. }),
                "{outcome:?}"
            );
            assert!(builds(&answered).is_empty(), "{:?}", answered.trace);
        } else {
            assert!(
                matches!(&outcome, CampaignOutcome::Failed { step_id, .. } if step_id == "install:tweaks-bg2"),
                "{outcome:?}"
            );
            assert_eq!(builds(&answered), vec!["build:tweaks-bg2:[2]"]);
        }

        let mut resumed = FakeDeps::default();
        assert_eq!(
            run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
            CampaignOutcome::Complete
        );
        assert_eq!(builds(&resumed)[0], "build:tweaks-bg2:[2]");
        assert!(resumed.plan_without_parts_calls.is_empty());
        assert_eq!(count_events(root, "left-out"), 1);
        assert_eq!(
            resumed.verified_plans[0].components_for("tweaks-bg2"),
            Some(&[2][..])
        );
    }
}

/// Everything that needs the failed component leaves with it in one question: the player is not
/// asked about each part in turn, and what left is not run.
#[test]
fn what_needs_the_failed_component_leaves_with_it_in_one_question() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: tweaks_fail_at_their_first_component(),
        failed_dependents: needs_eefix_needs_the_tweaks(),
        failed_with: BTreeMap::from([("tweaks-bg2".to_owned(), vec![2])]),
        answers: leave_it_out(),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert_eq!(outcome, CampaignOutcome::Complete);
    assert_eq!(count_events(root, "required"), 1);
    let remaining = InstallPlan {
        runs: fixture.request.plan.runs[..3].to_vec(),
    };
    assert_eq!(deps.verified_plans, vec![remaining.clone()]);
    assert!(!deps
        .trace
        .iter()
        .any(|entry| entry.starts_with("build:needs-eefix")));
    let root = &fixture.request.created.managed_root;
    let remaining = InstallPlan {
        runs: fixture.request.plan.runs[..3].to_vec(),
    };
    assert_eq!(deps.verified_plans, vec![remaining.clone()]);
    assert!(!deps
        .trace
        .iter()
        .any(|entry| entry.starts_with("build:needs-eefix")));
    let recorded = ledger_events(root)
        .into_iter()
        .filter_map(|event| match event {
            SessionEvent::ArtifactLeftOut {
                step_id,
                artifact_id,
                plan_sha256,
                removed_runs,
                failed_run,
                ..
            } => Some((step_id, artifact_id, plan_sha256, removed_runs, failed_run)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(recorded.len(), 1, "{recorded:?}");
    let (step_id, artifact_id, plan_sha256, removed_runs, failed_run) = &recorded[0];
    assert_eq!(step_id, "install:tweaks-bg2");
    assert_eq!(artifact_id, "tweaks");
    assert_eq!(*plan_sha256, plan_digest(&remaining).unwrap());
    assert_eq!(
        removed_runs,
        &vec!["tweaks-bg2".to_owned(), "needs-eefix".to_owned()]
    );
    let failed_run = failed_run.as_ref().unwrap();
    assert_eq!(failed_run.run_id, "tweaks-bg2");
    assert_eq!(failed_run.components, vec![1, 2]);
    assert_eq!(failed_run.left_out, vec![1]);
    assert!(failed_run.continues_with.is_empty());
    let receipt = deps.receipts.last().unwrap();
    assert!(matches!(receipt.outcome, ReceiptDraftOutcome::Succeeded));
    assert_eq!(receipt.installed_plan.as_ref(), Some(&remaining));
    assert_eq!(receipt.left_out.len(), 1);
    assert_eq!(
        receipt.left_out[0].failed_run.as_deref(),
        Some("tweaks-bg2")
    );
    assert_eq!(
        receipt.left_out[0].runs,
        vec!["tweaks-bg2".to_owned(), "needs-eefix".to_owned()]
    );
    let events = sink.events();
    assert!(events.contains(&EngineEvent::StepFinished {
        id: "install:tweaks-bg2".to_owned(),
        outcome: StepOutcome::Skipped,
    }));
    assert!(events.iter().any(|event| matches!(
        event,
        EngineEvent::ArtifactLeftOut { artifact_id, .. } if artifact_id == "tweaks"
    )));
}

/// Two components of one run can fail independently: each has its own question, and the second
/// is asked with the plan the first left.
#[test]
fn a_second_failure_in_a_run_that_went_on_asks_again() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut results = tweaks_fail_at_their_first_component();
    results.push_back(InstallReconciliation::retry(vec![2]));
    let mut deps = FakeDeps {
        after_results: results,
        answers: leave_it_out(),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert_eq!(outcome, CampaignOutcome::Complete);
    assert_eq!(count_events(root, "required"), 2);
    assert_eq!(count_events(root, "left-out"), 2);
    assert_eq!(
        deps.plan_without_parts_calls.last().unwrap(),
        &vec![
            ("tweaks-bg2".to_owned(), vec![1]),
            ("tweaks-bg2".to_owned(), vec![2]),
        ]
    );
    let builds = builds(&deps);
    assert_eq!(builds[builds.len() - 2], "build:tweaks-bg2:[2]");
    assert_eq!(builds[builds.len() - 1], "build:needs-eefix:[0]");
    assert_eq!(
        deps.verified_plans[0].components_for("tweaks-bg2"),
        None,
        "nothing of the run is left"
    );
    let receipt = deps.receipts.last().unwrap();
    assert_eq!(receipt.left_out.len(), 2);
    assert_eq!(receipt.decisions.len(), 2);
}

/// "Try again" ends the attempt without leaving anything out and runs the same components again.
#[test]
fn trying_again_runs_the_same_components_again() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: tweaks_fail_at_their_first_component(),
        answers: answer(QuestionKind::FailedComponent, Answer::Retry),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert_eq!(outcome, CampaignOutcome::Complete);
    assert_eq!(
        builds(&deps)
            .into_iter()
            .filter(|run| run.starts_with("build:tweaks-bg2"))
            .collect::<Vec<_>>(),
        vec!["build:tweaks-bg2:[1, 2]", "build:tweaks-bg2:[1, 2]"]
    );
    assert_eq!(count_events(root, "left-out"), 0);
    assert!(ledger_events(root).iter().any(|event| matches!(
        event,
        SessionEvent::StepFailed { step_id, detail, .. }
            if step_id == "install:tweaks-bg2" && detail == "you chose to try again"
    )));
    let receipt = deps.receipts.last().unwrap();
    assert_eq!(receipt.installed_plan, None);
    assert_eq!(receipt.decisions.len(), 1);
    assert_eq!(receipt.decisions[0].outcome, DecisionOutcome::Applied);
    // Trying again is no failure to report.
    assert!(!sink
        .events()
        .iter()
        .any(|event| matches!(event, EngineEvent::Error { .. })));
}

/// "Stop" ends the attempt as a failure in the player's words. A resume runs the component
/// again, and when it fails again, the player is asked again with a new question.
#[test]
fn stopping_at_a_failed_component_asks_again_when_the_retry_fails() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: tweaks_fail_at_their_first_component(),
        answers: answer(QuestionKind::FailedComponent, Answer::No),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert_eq!(
        outcome,
        CampaignOutcome::Failed {
            step_id: "install:tweaks-bg2".to_owned(),
            reason: "You chose to stop: a part of The tweaks mod could not be installed."
                .to_owned(),
        }
    );
    assert_eq!(count_events(root, "left-out"), 0);

    let mut resumed = FakeDeps {
        after_results: VecDeque::from([InstallReconciliation::retry(vec![1, 2])]),
        ..FakeDeps::default()
    };
    let outcome = run_campaign(&fixture.request, &mut resumed, &sink).unwrap();
    assert!(
        matches!(&outcome, CampaignOutcome::DecisionRequired { step_id, .. } if step_id == "install:tweaks-bg2"),
        "{outcome:?}"
    );
    let questions = asked(root);
    assert_eq!(questions.len(), 2);
    assert_ne!(questions[0].1, questions[1].1);
}

/// When a run installed some components before one failed, those stay and the plan keeps the
/// run with exactly them. A resume after that rebuilds the recorded plan without asking the
/// recipe again.
#[test]
fn a_run_that_failed_after_installing_some_parts_keeps_them_and_resumes_with_the_recorded_plan() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: VecDeque::from([
            InstallReconciliation::ProvenDone,
            InstallReconciliation::ProvenDone,
            InstallReconciliation::ProvenDone,
            InstallReconciliation::PartialPrefix { remaining: vec![2] },
            InstallReconciliation::retry(vec![2]),
        ]),
        // The next run then fails before it starts, which ends this attempt.
        fail_once: Some("build:needs-eefix".to_owned()),
        answers: leave_it_out(),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
    assert!(
        matches!(&outcome, CampaignOutcome::Failed { step_id, .. } if step_id == "install:needs-eefix"),
        "{outcome:?}"
    );
    let mut tweaks = fixture.request.plan.runs[3].clone();
    tweaks.components = vec![1];
    let remaining = InstallPlan {
        runs: vec![
            fixture.request.plan.runs[0].clone(),
            fixture.request.plan.runs[1].clone(),
            fixture.request.plan.runs[2].clone(),
            tweaks,
            fixture.request.plan.runs[4].clone(),
        ],
    };

    let mut resumed = FakeDeps::default();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(resumed.plan_without_parts_calls.is_empty());
    assert_eq!(resumed.verified_plans, vec![remaining.clone()]);
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.installed_plan.as_ref(), Some(&remaining));
    assert_eq!(
        receipt.left_out[0].changed_runs,
        vec!["tweaks-bg2".to_owned()]
    );
    assert!(receipt.left_out[0].runs.is_empty());
}

/// An earlier version left the rest of a failed run out without asking, in a record that names
/// neither what was left out by name nor what went on. A campaign with such a record resumes
/// with it, and it reaches the receipt.
#[test]
fn a_failed_run_recorded_by_an_earlier_version_still_replays() {
    let sink = RecordingSink::default();
    let probe = Fixture::with_dependents();
    let mut failing = FakeDeps {
        after_results: tweaks_fail_at_their_first_component(),
        ..FakeDeps::default()
    };
    run_campaign(&probe.request, &mut failing, &sink).unwrap();
    let completed = completed_step_ids(&probe.request.created.managed_root);

    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let mut tweaks = fixture.request.plan.runs[3].clone();
    tweaks.components = vec![1];
    let mut remaining = fixture.request.plan.clone();
    remaining.runs[3] = tweaks.clone();
    remaining.runs.remove(4);
    seed_unresolved(
        &fixture.request,
        &completed.iter().map(String::as_str).collect::<Vec<_>>(),
        "install:tweaks-bg2",
    );
    let event = SessionEvent::ArtifactLeftOut {
        step_id: "install:tweaks-bg2".to_owned(),
        attempt: 1,
        artifact_id: "tweaks".to_owned(),
        detail: "its installer stopped with an error".to_owned(),
        plan_sha256: plan_digest(&remaining).unwrap(),
        removed_runs: vec!["needs-eefix".to_owned()],
        changed_runs: vec![tweaks],
        failed_run: Some(bg_engine::session::FailedRun {
            run_id: "tweaks-bg2".to_owned(),
            components: vec![2],
            left_out: Vec::new(),
            continues_with: Vec::new(),
        }),
    };
    let record = serde_json::to_value(&event).unwrap();
    let failed_run = record["data"]["failed_run"].as_object().unwrap();
    assert_eq!(failed_run.len(), 2, "{failed_run:#?}");
    SessionStore::open(root).unwrap().append(event).unwrap();

    let mut resumed = FakeDeps::default();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );

    assert!(builds(&resumed).is_empty(), "{:?}", resumed.trace);
    assert_eq!(resumed.verified_plans, vec![remaining.clone()]);
    let receipt = resumed.receipts.last().unwrap();
    assert_eq!(receipt.installed_plan.as_ref(), Some(&remaining));
    assert_eq!(
        receipt.left_out[0].failed_run.as_deref(),
        Some("tweaks-bg2")
    );
}

/// What WeiDU reports when component 1 of `tweaks-bg2` fails and it goes on with component 2.
fn went_past_the_first_tweak() -> InstallReconciliation {
    InstallReconciliation::FailedAmong {
        failed: vec![1],
        installed_after: vec![2],
    }
}

/// WeiDU goes on after a failed component. The installer asks all the same: leaving it out
/// keeps what WeiDU installed after it, and trying again is not offered, because that would
/// install the component out of order. "Stop" keeps the question: a resume runs nothing and
/// asks again, however often, from the attempt that ran the installer.
#[test]
fn a_component_the_installer_went_past_is_left_out_or_stopped_at_but_not_tried_again() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut results = tweaks_fail_at_their_first_component();
    results.pop_back();
    results.push_back(went_past_the_first_tweak());
    let mut deps = FakeDeps {
        after_results: results,
        failed_dependents: needs_eefix_needs_the_tweaks(),
        answers: answer(QuestionKind::FailedComponent, Answer::No),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert!(
        matches!(&outcome, CampaignOutcome::Failed { step_id, .. } if step_id == "install:tweaks-bg2"),
        "{outcome:?}"
    );
    let question = asked(root).remove(0).2;
    assert!(!question.offers(Answer::Retry));
    assert!(question.offers(Answer::LeaveOut));
    let failed = question.failed.as_ref().unwrap();
    assert_eq!(failed.components, vec![1]);
    assert_eq!(failed.installed_after, vec![2]);
    assert!(failed.continues_with.is_empty());
    assert_eq!(failed.evidence_attempt, None);
    let closure = question.leave_out.as_ref().unwrap();
    assert_eq!(closure.removed_runs, vec!["needs-eefix".to_owned()]);
    assert_eq!(closure.changed_runs[0].components, vec![2]);
    let ran = count_events(root, "required");
    assert_eq!(ran, 1);

    for asked_again in 2..=3 {
        let mut resumed = FakeDeps {
            recovery_results: VecDeque::from([went_past_the_first_tweak()]),
            failed_dependents: needs_eefix_needs_the_tweaks(),
            ..FakeDeps::default()
        };
        let outcome = run_campaign(&fixture.request, &mut resumed, &sink).unwrap();
        assert!(
            matches!(&outcome, CampaignOutcome::DecisionRequired { .. }),
            "{outcome:?}"
        );
        assert!(builds(&resumed).is_empty(), "{:?}", resumed.trace);
        assert_eq!(resumed.recovered_attempts, vec![1], "the attempt that ran");
        let questions = asked(root);
        assert_eq!(questions.len(), asked_again);
        let last = questions.last().unwrap();
        assert_eq!(last.2.failed.as_ref().unwrap().evidence_attempt, Some(1));

        if asked_again == 2 {
            let mut stopped = FakeDeps {
                answers: answer(QuestionKind::FailedComponent, Answer::No),
                ..FakeDeps::default()
            };
            assert!(matches!(
                run_campaign(&fixture.request, &mut stopped, &sink).unwrap(),
                CampaignOutcome::Failed { .. }
            ));
        }
    }

    let mut answered = FakeDeps {
        answers: leave_it_out(),
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut answered, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert!(!builds(&answered)
        .iter()
        .any(|run| run.starts_with("build:tweaks-bg2")));
    assert!(!builds(&answered)
        .iter()
        .any(|run| run.starts_with("build:needs-eefix")));
    assert_eq!(
        answered.verified_plans[0].components_for("tweaks-bg2"),
        Some(&[2][..])
    );
    let failed_run = left_out_run(root);
    assert_eq!(failed_run.components, vec![1]);
    assert_eq!(failed_run.left_out, vec![1]);
    assert!(failed_run.continues_with.is_empty());
}

/// A nonprefix failure with unreconciled CEBG records cannot safely be retried or omitted.
#[test]
fn unreconciled_history_after_a_nonprefix_failure_needs_a_fresh_copy() {
    let fixture = Fixture::with_dependents();
    let root = &fixture.request.created.managed_root;
    let sink = RecordingSink::default();
    let mut results = tweaks_fail_at_their_first_component();
    results.pop_back();
    results.push_back(went_past_the_first_tweak());
    let mut deps = FakeDeps {
        after_results: results,
        invalid_history: vec!["tweaks".to_owned()],
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    let CampaignOutcome::FreshCopyRequired { step_id, reason } = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(step_id, "install:tweaks-bg2");
    assert!(
        reason.contains("CEBG history cannot be reconciled for tweaks"),
        "{reason}"
    );
    assert!(reason.contains("cannot try it again"), "{reason}");
    assert_eq!(count_events(root, "required"), 0);
}

/// The components a failed run installed stay even when the feature of the failed component
/// also owns them: they are in the game, so the plan keeps them and only the rest leaves.
#[test]
fn components_installed_before_the_failure_stay_whatever_leaves_with_it() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: VecDeque::from([
            InstallReconciliation::ProvenDone,
            InstallReconciliation::ProvenDone,
            InstallReconciliation::ProvenDone,
            InstallReconciliation::PartialPrefix { remaining: vec![2] },
            InstallReconciliation::retry(vec![2]),
        ]),
        failed_with: BTreeMap::from([("tweaks-bg2".to_owned(), vec![1])]),
        answers: leave_it_out(),
        ..FakeDeps::default()
    };

    assert_eq!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(
        deps.verified_plans[0].components_for("tweaks-bg2"),
        Some(&[1][..])
    );
}

/// The results of a campaign whose run `failing` fails, after all runs before it installed.
fn results_failing_at(fixture: &Fixture, failing: &str) -> VecDeque<InstallReconciliation> {
    let position = fixture
        .request
        .plan
        .runs
        .iter()
        .position(|run| run.run_id == failing)
        .unwrap();
    let mut results = VecDeque::from(vec![InstallReconciliation::ProvenDone; position]);
    let failed_components = fixture.request.plan.runs[position].components.clone();
    results.push_back(InstallReconciliation::retry(failed_components));
    results
}

/// A history reconciliation failure does not permit leaving out a failed run. Retrying can
/// produce the missing evidence; an incompatible preset leaves the question open.
#[test]
fn a_failed_part_with_unreconciled_history_offers_only_to_try_again_or_stop() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: results_failing_at(&fixture, "eet-core"),
        answers: leave_it_out(),
        invalid_history: vec!["eet".to_owned()],
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert!(
        matches!(&outcome, CampaignOutcome::DecisionRequired { step_id, .. } if step_id == "install:eet-core"),
        "{outcome:?}"
    );
    let root = &fixture.request.created.managed_root;
    assert_eq!(count_events(root, "left-out"), 0);
    let questions = asked(root);
    let question = &questions[0].2;
    assert_eq!(question.leave_out, None);
    assert!(!question.offers(Answer::LeaveOut));
    assert!(question.offers(Answer::Retry));
    assert!(question.failed.as_ref().unwrap().required.is_some());

    let mut retried = FakeDeps {
        answers: answer(QuestionKind::FailedComponent, Answer::Retry),
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut retried, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(retried.receipts.last().unwrap().installed_plan, None);
}

/// A run in the game stays in the plan, even when it needs a part that fails later: the
/// installer never uninstalls, so that is no reason to refuse leaving the part out.
#[test]
fn a_run_in_the_game_stays_when_a_part_it_needs_fails_later() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: results_failing_at(&fixture, "tweaks-bg2"),
        answers: leave_it_out(),
        failed_dependents: BTreeMap::from([(
            "tweaks-bg2".to_owned(),
            vec!["eefix-bg1".to_owned()],
        )]),
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert_eq!(outcome, CampaignOutcome::Complete);
    let root = &fixture.request.created.managed_root;
    assert_eq!(count_events(root, "left-out"), 1);
    let receipt = deps.receipts.last().unwrap();
    let installed = receipt.installed_plan.as_ref().unwrap();
    assert!(installed.components_for("eefix-bg1").is_some());
    let planned = fixture.request.plan.components_for("tweaks-bg2").unwrap();
    assert_eq!(installed.components_for("tweaks-bg2"), Some(&planned[1..]));
    let in_game = &deps.in_game_at_questions[0];
    assert!(in_game.iter().any(|(run, _)| run == "eefix-bg1"));
    assert!(in_game.contains(&("tweaks-bg2".to_owned(), Vec::new())));
}

fn modded_bg1_question() -> Question {
    Question {
        kind: QuestionKind::ModdedSource,
        artifact_id: "source-bg1".to_owned(),
        artifact_kind: ArtifactKind::Payload,
        reviewed: FrozenIdentity {
            id: "source-bg1".to_owned(),
            version: "2.7.3.0".to_owned(),
            sha256: "11".repeat(32),
            length: 0,
        },
        served: None,
        reason: None,
        leave_out: None,
        failed: None,
    }
}

/// A source game that looks modded asks once, at its staging step and before anything is
/// copied. No stops; Yes copies it and continues, and the answer stands for later attempts.
#[test]
fn a_modded_source_game_asks_before_it_is_copied() {
    let fixture = Fixture::new();
    let sink = RecordingSink::default();
    let modded = || FakeDeps {
        source_questions: BTreeMap::from([(GameRole::BgeeSod, modded_bg1_question())]),
        ..FakeDeps::default()
    };

    // Without an answer it waits, and nothing is copied.
    let mut waiting = modded();
    let outcome = run_campaign(&fixture.request, &mut waiting, &sink).unwrap();
    assert!(
        matches!(&outcome, CampaignOutcome::DecisionRequired { step_id, .. } if step_id == "stage:bg1"),
        "{outcome:?}"
    );
    assert!(!waiting
        .trace
        .iter()
        .any(|entry| entry.starts_with("stage:")));
    let root = &fixture.request.created.managed_root;
    assert_eq!(asked(root).len(), 1);

    // No stops this attempt.
    let mut stopped = FakeDeps {
        answers: BTreeMap::from([(
            QuestionKind::ModdedSource,
            (Answer::No, AnswerSource::Player),
        )]),
        ..modded()
    };
    let outcome = run_campaign(&fixture.request, &mut stopped, &sink).unwrap();
    assert!(
        matches!(&outcome, CampaignOutcome::Failed { step_id, reason } if step_id == "stage:bg1" && reason.starts_with("You chose to stop")),
        "{outcome:?}"
    );
    assert!(!stopped
        .trace
        .iter()
        .any(|entry| entry.starts_with("stage:")));

    // The next attempt asks anew; Yes copies the game and the installation completes.
    let mut accepted = FakeDeps {
        answers: BTreeMap::from([(
            QuestionKind::ModdedSource,
            (Answer::Yes, AnswerSource::Preset),
        )]),
        fail_once: Some("stage:bg2".to_owned()),
        ..modded()
    };
    let outcome = run_campaign(&fixture.request, &mut accepted, &sink).unwrap();
    assert!(
        matches!(&outcome, CampaignOutcome::Failed { step_id, .. } if step_id == "stage:bg2"),
        "{outcome:?}"
    );
    assert!(accepted.trace.iter().any(|entry| entry == "stage:bg1"));
    assert_eq!(asked(root).len(), 2);

    // The answer stands: a resume asks no more and completes.
    let mut resumed = modded();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(resumed.answer_requests, 0);
    assert_eq!(asked(root).len(), 2);
    let decisions = &resumed.receipts.last().unwrap().decisions;
    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[1].question.kind, QuestionKind::ModdedSource);
    assert_eq!(decisions[1].answer.answer, Answer::Yes);
}

#[test]
fn a_modded_iwd_source_asks_once_and_keeps_the_answer_on_resume() {
    let fixture = Fixture::standalone();
    let sink = RecordingSink::default();
    let modded = || {
        let mut question = modded_bg1_question();
        question.artifact_id = "source-iwd".to_owned();
        question.reviewed.id = question.artifact_id.clone();
        FakeDeps {
            source_questions: BTreeMap::from([(GameRole::Iwdee, question)]),
            ..FakeDeps::default()
        }
    };

    let mut waiting = modded();
    let outcome = run_campaign(&fixture.request, &mut waiting, &sink).unwrap();
    assert!(
        matches!(outcome, CampaignOutcome::DecisionRequired { step_id, .. } if step_id == "stage:game")
    );
    assert!(!waiting
        .trace
        .iter()
        .any(|entry| entry.starts_with("stage:")));

    let mut accepted = FakeDeps {
        answers: answer(QuestionKind::ModdedSource, Answer::Yes),
        fail_once: Some("stage:game".to_owned()),
        ..modded()
    };
    assert!(
        matches!(run_campaign(&fixture.request, &mut accepted, &sink).unwrap(), CampaignOutcome::Failed { step_id, .. } if step_id == "stage:game")
    );

    let mut resumed = modded();
    assert_eq!(
        run_campaign(&fixture.request, &mut resumed, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(resumed.answer_requests, 0);
    assert_eq!(asked(&fixture.request.created.managed_root).len(), 1);
    let decisions = &resumed.receipts.last().unwrap().decisions;
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].question.artifact_id, "source-iwd");
    assert_eq!(decisions[0].answer.answer, Answer::Yes);
}

#[test]
fn a_failed_iwd_component_uses_the_shared_question_and_can_retry() {
    let fixture = Fixture::standalone();
    let sink = RecordingSink::default();
    let mut failed = FakeDeps {
        after_results: results_failing_at(&fixture, "iwdfix-main"),
        ..FakeDeps::default()
    };
    let outcome = run_campaign(&fixture.request, &mut failed, &sink).unwrap();
    assert!(
        matches!(outcome, CampaignOutcome::DecisionRequired { step_id, .. } if step_id == "install:iwdfix-main")
    );
    let questions = asked(&fixture.request.created.managed_root);
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].2.kind, QuestionKind::FailedComponent);
    assert!(questions[0].2.offers(Answer::Retry));
    let mut retried = FakeDeps {
        answers: answer(QuestionKind::FailedComponent, Answer::Retry),
        ..FakeDeps::default()
    };
    assert_eq!(
        run_campaign(&fixture.request, &mut retried, &sink).unwrap(),
        CampaignOutcome::Complete
    );
    assert_eq!(retried.receipts.last().unwrap().installed_plan, None);
}

#[test]
fn standalone_campaign_stages_one_game_without_a_bg1_step() {
    let fixture = Fixture::standalone();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert_eq!(outcome, CampaignOutcome::Complete);
    assert_eq!(
        completed_step_ids(&fixture.request.created.managed_root),
        vec![
            "preflight",
            "acquire:eefix",
            "acquire:weidu",
            "acquire:eet",
            "stage:game",
            "materialize:eefix:bg2",
            "materialize:eet:bg2",
            "install:iwdfix-main",
            "install:iwdtweaks-main",
            "identity:final",
            "verify:final",
            "receipt",
        ]
    );
    assert!(deps.trace.iter().any(|entry| entry == "stage:game"));
    assert!(!deps.trace.iter().any(|entry| entry == "stage:bg1"));
    let labels = sink
        .events()
        .into_iter()
        .filter_map(|event| match event {
            EngineEvent::StepStarted { label, .. } => Some(label),
            EngineEvent::PhaseStarted { name } => Some(name),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(labels
        .iter()
        .any(|label| label == "Stage Icewind Dale: Enhanced Edition"));
    assert!(labels
        .iter()
        .any(|label| label == "Prepare eefix for Icewind Dale: Enhanced Edition"));
    assert!(
        labels.iter().all(|label| !label.contains("Baldur")),
        "a standalone campaign must not name Baldur's Gate: {labels:?}"
    );
    assert!(!fixture.request.created.managed_root.join("bg1").exists());
}

#[test]
fn standalone_campaign_rejects_runs_outside_the_main_phase_or_game_root() {
    for (target, phase) in [
        (GameRoot::Bg1, Phase::Bg1Preparation),
        (GameRoot::Bg2, Phase::Bg2Preparation),
        (GameRoot::Bg2, Phase::EetInitialization),
    ] {
        let mut fixture = Fixture::standalone();
        fixture.request.plan.runs[0].target = target;
        fixture.request.plan.runs[0].phase = phase;
        fixture.request.created.plan_sha256 = plan_digest(&fixture.request.plan).unwrap();
        let sink = RecordingSink::default();
        let mut deps = FakeDeps::default();

        let error = run_campaign(&fixture.request, &mut deps, &sink).unwrap_err();

        let message = error.to_string();
        assert!(
            message.contains(
                "a standalone Icewind Dale: Enhanced Edition installation runs only the main phase"
            ),
            "{phase:?}: {message}"
        );
        assert!(deps.trace.is_empty(), "{phase:?}: {:?}", deps.trace);
    }
}

#[test]
fn standalone_resume_rejects_a_bg1_folder_in_the_managed_root() {
    let fixture = Fixture::standalone();
    seed_unresolved(&fixture.request, &["preflight"], "acquire:eefix");
    std::fs::create_dir(fixture.request.created.managed_root.join("bg1")).unwrap();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps::default();

    let error = run_campaign(&fixture.request, &mut deps, &sink).unwrap_err();

    assert!(
        error.to_string().contains("unowned top-level entry"),
        "{error}"
    );
}
