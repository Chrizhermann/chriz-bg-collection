use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::events::{EngineEvent, EventSink};
use bg_engine::games::GameRole;
use bg_engine::lock::{LockError, TargetLock};
use bg_engine::manifest::{GameRoot, Phase, RunArg};
use bg_engine::orchestrator::{
    run_campaign, ArtifactAcquirer, ArtifactKind, ArtifactMaterializer, BuiltInvocation,
    CampaignClock, CampaignOutcome, CampaignPreflight, CampaignRequest, InstallLogVerifier,
    InstallReconciliation, InvocationBuilder, MaterializationOutcome, MaterializationTask,
    MutationCheck, ProcessResult, ProcessRunner, ReceiptDraft, ReceiptWriter, StagingService,
    StepAttempt, StepFailure,
};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::resolve::{InstallPlan, PlannedRun};
use bg_engine::session::{
    CampaignCreated, FrozenIdentity, SessionEvent, SessionReplay, SessionStore,
    SourceGameFingerprints,
};

const RECIPE_PAYLOAD: &[u8] = b"PK\x03\x04signed alpha recipe";
const RECIPE_ENVELOPE: &[u8] = br#"{"recipe_id":"alpha","version":"0.1.0"}"#;

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
            artifact_identities: vec![frozen("eefix", "33", 10), frozen("eet", "44", 20)],
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
        artifact_id: artifact_id.to_owned(),
        weidu_artifact_id: "weidu".to_owned(),
        prompt_scripts: vec![],
    }
}

#[derive(Default)]
struct FakeDeps {
    trace: Vec<String>,
    acquisitions: BTreeMap<String, usize>,
    fail_once: Option<String>,
    materialization_results: VecDeque<MaterializationOutcome>,
    recovery_results: VecDeque<InstallReconciliation>,
    after_results: VecDeque<InstallReconciliation>,
    runner_active: bool,
    lock_probe: Option<(PathBuf, PathBuf)>,
    lock_was_held_at_receipt: bool,
    now: u64,
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
        self.maybe_fail(&format!("check:{}", check.step_id))
    }
}

impl ArtifactAcquirer for FakeDeps {
    fn acquire(
        &mut self,
        identity: &FrozenIdentity,
        _kind: ArtifactKind,
    ) -> Result<(), StepFailure> {
        let operation = format!("acquire:{}", identity.id);
        self.trace.push(operation.clone());
        *self.acquisitions.entry(identity.id.clone()).or_default() += 1;
        self.maybe_fail(&operation)
    }
}

impl StagingService for FakeDeps {
    fn stage(&mut self, role: GameRole) -> Result<(), StepFailure> {
        let role = match role {
            GameRole::BgeeSod => "bg1",
            GameRole::Bg2ee => "bg2",
        };
        let operation = format!("stage:{role}");
        self.trace.push(operation.clone());
        self.maybe_fail(&operation)
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

    fn verify_final(&mut self, _plan: &InstallPlan) -> Result<(), StepFailure> {
        self.trace.push("verify:final".to_owned());
        self.maybe_fail("verify:final")
    }
}

impl ReceiptWriter for FakeDeps {
    fn write(&mut self, _draft: &ReceiptDraft) -> Result<(), StepFailure> {
        self.trace.push("receipt".to_owned());
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
    assert!(deps.lock_was_held_at_receipt);
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

    let split = deps.trace.len();
    let second = run_campaign(&fixture.request, &mut deps, &sink).unwrap();
    assert_eq!(second, CampaignOutcome::Complete);
    let resumed = &deps.trace[split..];
    assert_eq!(resumed.first().map(String::as_str), Some("preflight"));
    assert!(resumed.iter().any(|entry| entry == "stage:bg2"));
    assert!(!resumed.iter().any(|entry| entry == "stage:bg1"));
    assert!(!resumed.iter().any(|entry| entry.starts_with("acquire:")));
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
        assert!(!deps.trace.iter().any(|entry| entry.starts_with("run:")));
        assert_eq!(
            SessionStore::open(&fixture.request.created.managed_root)
                .unwrap()
                .replay()
                .unwrap()
                .unresolved_step(),
            Some("materialize:eefix:bg1")
        );
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
            assert!(!deps.trace.iter().any(|entry| entry == "receipt"));
        }

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
