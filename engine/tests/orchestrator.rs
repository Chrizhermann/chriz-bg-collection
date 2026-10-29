use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::events::{EngineEvent, EventSink, StepOutcome};
use bg_engine::games::GameRole;
use bg_engine::lock::{LockError, TargetLock};
use bg_engine::manifest::{GameRoot, Phase, Postcondition, RunArg};
use bg_engine::orchestrator::{
    run_campaign, run_campaign_controlled, AcquisitionOutcome, ArtifactAcquirer, ArtifactKind,
    ArtifactMaterializer, BuiltInvocation, CampaignClock, CampaignOutcome, CampaignPreflight,
    CampaignRecorder, CampaignRequest, InstallLogVerifier, InstallReconciliation,
    InvocationBuilder, LeftOutArtifact, MaterializationOutcome, MaterializationTask, MutationCheck,
    MutationKind, ProcessResult, ProcessRunner, ReceiptDraft, ReceiptDraftOutcome, ReceiptWriter,
    StagingService, StepAttempt, StepFailure,
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
                bg1: "11".repeat(32),
                bg2: "22".repeat(32),
            },
            artifact_identities: vec![
                frozen("eefix", "33", 10),
                frozen("weidu", "66", 40),
                frozen("eet", "44", 20),
            ],
            tool_identities: vec![frozen("weidu", "55", 30)],
            staged_bg1: managed_root.join("bg1"),
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
    /// Further runs that need a left-out payload, by payload id.
    dependents: BTreeMap<String, Vec<String>>,
    /// Runs that lose components when a payload is left out, by payload id.
    reduced: BTreeMap<String, Vec<(String, Vec<u32>)>>,
    /// Payloads the fake recipe cannot do without.
    essential: Vec<String>,
    /// Every `plan_without` request, to see replays recompute it.
    plan_without_calls: Vec<Vec<String>>,
    /// Manual pauses of the fake campaign; each continues at once.
    checkpoints: Vec<bg_engine::manual_checkpoint::FrozenCheckpoint>,
    /// Checkpoint plans handed over, to see that they lack left-out runs.
    checkpoint_plans: Vec<InstallPlan>,
    /// The plan final verification received.
    verified_plans: Vec<InstallPlan>,
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
        Ok(match self.leave_out.get(&identity.id) {
            Some(reason) => AcquisitionOutcome::LeaveOut {
                reason: reason.clone(),
            },
            None => AcquisitionOutcome::Ready,
        })
    }

    fn plan_without(
        &mut self,
        plan: &InstallPlan,
        left_out: &[String],
    ) -> Result<InstallPlan, StepFailure> {
        self.plan_without_calls.push(left_out.to_vec());
        if let Some(essential) = left_out.iter().find(|id| self.essential.contains(id)) {
            return Err(StepFailure::new(format!(
                "{essential} is a fixed part of this collection"
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
}

impl StagingService for FakeDeps {
    fn stage(&mut self, role: GameRole) -> Result<(), StepFailure> {
        let role_name = match role {
            GameRole::BgeeSod => "bg1",
            GameRole::Bg2ee => "bg2",
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
        self.maybe_fail(&operation)?;
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
        _attempt: &StepAttempt,
    ) -> Result<InstallReconciliation, StepFailure> {
        self.trace.push(format!("recover:{}", run.run_id));
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
    fixture.request.created.staged_bg1 = fixture.request.created.managed_root.join("bg1");
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
fn normal_proven_install_checks_postconditions_before_completion() {
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
fn failed_normal_postcondition_requires_fresh_copy_and_stops_next_run() {
    let mut fixture = Fixture::new();
    add_first_run_marker(&mut fixture, "merged and ready");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        staged_files: vec![(
            GameRole::BgeeSod,
            first_marker_path(&fixture),
            b"not ready\n".to_vec(),
        )],
        ..FakeDeps::default()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
    let first_receipt = deps.receipts.last().unwrap().clone();

    assert!(matches!(
        outcome,
        CampaignOutcome::FreshCopyRequired { ref step_id, ref reason }
            if step_id == "install:eefix-bg1" && reason.contains("postcondition")
    ));
    assert!(!completed_step_ids(&fixture.request.created.managed_root)
        .contains(&"install:eefix-bg1".to_owned()));
    assert!(!deps
        .trace
        .iter()
        .any(|entry| entry.starts_with("build:eefix-bg2:")));
    let replay = SessionStore::open(&fixture.request.created.managed_root)
        .unwrap()
        .replay()
        .unwrap();
    assert_eq!(replay.unresolved_step(), None);
    assert_eq!(
        replay
            .fresh_copy_required()
            .map(|seal| seal.step_id.as_str()),
        Some("install:eefix-bg1")
    );

    deps.recovery_results
        .push_back(InstallReconciliation::retry(vec![0, 2]));
    let split = deps.trace.len();
    let resumed_outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
    assert_eq!(resumed_outcome, outcome);
    assert_eq!(deps.receipts.last(), Some(&first_receipt));
    assert_eq!(deps.recovery_results.len(), 1);
    assert!(!deps.trace[split..].iter().any(|entry| entry == "preflight"));
    assert!(!deps.trace[split..]
        .iter()
        .any(|entry| entry == "recover:eefix-bg1"));
    assert!(!deps.trace[split..]
        .iter()
        .any(|entry| entry.starts_with("build:") || entry.starts_with("run:")));
}

#[test]
fn unresolved_proven_install_checks_postconditions_before_completion() {
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
fn failed_unresolved_proven_postcondition_requires_fresh_copy_without_rerun() {
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

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert!(matches!(
        outcome,
        CampaignOutcome::FreshCopyRequired { ref step_id, ref reason }
            if step_id == "install:eefix-bg1" && reason.contains("postcondition")
    ));
    assert!(!deps.trace.iter().any(|entry| entry.starts_with("build:")));
    assert!(!deps.trace.iter().any(|entry| entry.starts_with("run:")));
    let replay = SessionStore::open(&fixture.request.created.managed_root)
        .unwrap()
        .replay()
        .unwrap();
    assert_eq!(replay.unresolved_step(), None);
    assert_eq!(
        replay
            .fresh_copy_required()
            .map(|seal| seal.step_id.as_str()),
        Some("install:eefix-bg1")
    );
}

#[test]
fn last_failed_proven_install_checks_postconditions_before_completion() {
    let mut fixture = Fixture::new();
    add_first_run_marker(&mut fixture, "merged and ready");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: VecDeque::from([InstallReconciliation::retry(vec![0, 2])]),
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
fn failed_last_failed_proven_postcondition_requires_fresh_copy_without_rerun() {
    let mut fixture = Fixture::new();
    add_first_run_marker(&mut fixture, "merged and ready");
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        after_results: VecDeque::from([InstallReconciliation::retry(vec![0, 2])]),
        ..FakeDeps::default()
    };
    assert!(matches!(
        run_campaign(&fixture.request, &mut deps, &sink).unwrap(),
        CampaignOutcome::Failed { ref step_id, .. } if step_id == "install:eefix-bg1"
    ));
    deps.recovery_results
        .push_back(InstallReconciliation::ProvenDone);
    let split = deps.trace.len();

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    assert!(matches!(
        outcome,
        CampaignOutcome::FreshCopyRequired { ref step_id, ref reason }
            if step_id == "install:eefix-bg1" && reason.contains("postcondition")
    ));
    assert!(!deps.trace[split..]
        .iter()
        .any(|entry| entry.starts_with("build:") || entry.starts_with("run:")));
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
    fixture.request.created.staged_bg1 = fixture.request.created.managed_root.join("bg1");
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
    fixture.request.created.staged_bg1 = fixture.request.created.managed_root.join("bg1");
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

fn eefix_unusable_deps() -> FakeDeps {
    FakeDeps {
        leave_out: BTreeMap::from([("eefix".to_owned(), LAYOUT_CHANGED.to_owned())]),
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
    assert_eq!(resumed.plan_without_calls, vec![vec!["eefix".to_owned()]]);
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

#[test]
fn resume_refuses_when_the_plan_without_a_left_out_mod_comes_out_differently() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let controls = RunnerControlHandle::new();
    let mut deps = FakeDeps {
        pause_after_operation: Some("acquire:weidu".to_owned()),
        controls: Some(controls.clone()),
        ..eefix_unusable_deps()
    };
    run_campaign_controlled(&fixture.request, &mut deps, &sink, &controls).unwrap();

    // A backend that no longer drops `needs-eefix` would install a different plan.
    let mut changed = FakeDeps::default();
    let error = run_campaign(&fixture.request, &mut changed, &sink).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("differs from the one recorded when it was left out"),
        "{error}"
    );
    assert!(changed.acquisitions.is_empty());
}

#[test]
fn a_mod_the_recipe_cannot_do_without_fails_its_step_instead_of_being_left_out() {
    let fixture = Fixture::with_dependents();
    let sink = RecordingSink::default();
    let mut deps = FakeDeps {
        essential: vec!["eefix".to_owned()],
        ..eefix_unusable_deps()
    };

    let outcome = run_campaign(&fixture.request, &mut deps, &sink).unwrap();

    let CampaignOutcome::Failed { step_id, reason } = outcome else {
        panic!("expected a failed acquisition, got {outcome:?}");
    };
    assert_eq!(step_id, "acquire:eefix");
    assert!(reason.contains(LAYOUT_CHANGED), "{reason}");
    assert!(reason.contains("fixed part of this collection"), "{reason}");
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
        reason.contains("cannot continue without this tool"),
        "{reason}"
    );
    assert!(deps.plan_without_calls.is_empty());
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
