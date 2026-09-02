use std::path::{Path, PathBuf};

use bg_engine::games::{GameRole, Storefront};
use bg_engine::manifest::GameRoot;
use bg_engine::orchestrator::{ReceiptDraft, ReceiptDraftOutcome, ReceiptWriter};
use bg_engine::receipt::{
    ArtifactCacheOutcome, ArtifactReceipt, FinalLogReceipt, FinalReceiptState, InstallReceipt,
    LogComponentReceipt, LogDiffReceipt, ManagedReceiptWriter, PromptReceipt, ReceiptEvidence,
    ReceiptOutcome, ReceiptStore, ReceiptVersions, RunReceipt, RunTiming, SourceGameReceipt,
    WeiDuToolReceipt, RECEIPT_SCHEMA_VERSION,
};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::resolve::{InstallPlan, PlannedRun};
use bg_engine::session::{CampaignCreated, FrozenIdentity, SourceGameFingerprints};
use tempfile::TempDir;

fn plan() -> InstallPlan {
    InstallPlan {
        runs: vec![PlannedRun {
            run_id: "eet-core".to_owned(),
            mod_id: "eet".to_owned(),
            target: GameRoot::Bg2,
            phase: bg_engine::manifest::Phase::EetInitialization,
            components: vec![0],
            args: Vec::new(),
            artifact_id: "eet".to_owned(),
            weidu_artifact_id: "weidu".to_owned(),
            prompt_scripts: Vec::new(),
        }],
    }
}

fn frozen(id: &str, byte: &str, length: u64) -> FrozenIdentity {
    FrozenIdentity {
        id: id.to_owned(),
        version: if id == "weidu" {
            "24900".to_owned()
        } else {
            "2.7.3".to_owned()
        },
        sha256: byte.repeat(32),
        length,
    }
}

fn success_receipt(root: &Path) -> InstallReceipt {
    let component = LogComponentReceipt {
        tp2: "eet/eet.tp2".to_owned(),
        language: 0,
        component: 0,
    };
    InstallReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        install_id: "install-001".to_owned(),
        attempt_id: "attempt-001".to_owned(),
        evidence_attempt_id: "attempt-001".to_owned(),
        managed_root: root.to_path_buf(),
        staged_bg1: root.join("bg1"),
        staged_bg2: root.join("game"),
        started_at_millis: 10,
        completed_at_millis: 20,
        outcome: ReceiptOutcome::Succeeded,
        versions: ReceiptVersions {
            application: "0.1.0-alpha.1".to_owned(),
            engine: "0.1.0".to_owned(),
            manifest_schema: 2,
            recipe: "2026.09.03-alpha.1".to_owned(),
        },
        source_games: vec![
            SourceGameReceipt {
                role: GameRole::BgeeSod,
                storefront: Storefront::Steam,
                version: "2.7.3.0".to_owned(),
                fingerprint: "11".repeat(32),
            },
            SourceGameReceipt {
                role: GameRole::Bg2ee,
                storefront: Storefront::Steam,
                version: "2.7.3.0".to_owned(),
                fingerprint: "22".repeat(32),
            },
        ],
        recipe_payload_sha256: "33".repeat(32),
        recipe_envelope_sha256: "44".repeat(32),
        selection_sha256: "55".repeat(32),
        normalized_selection: NormalizedSelection {
            platform: "windows".to_owned(),
            features: std::collections::BTreeMap::from([("recommended".to_owned(), true)]),
            inputs: std::collections::BTreeMap::new(),
        },
        plan_sha256: "66".repeat(32),
        plan: plan(),
        artifacts: vec![ArtifactReceipt {
            id: "eet".to_owned(),
            version: "2.7.3".to_owned(),
            original_url: "https://example.invalid/eet.zip".to_owned(),
            final_url: "https://cdn.example.invalid/eet.zip".to_owned(),
            length: 1_024,
            sha256: "77".repeat(32),
            cache_outcome: ArtifactCacheOutcome::Downloaded,
        }],
        weidu_tools: vec![WeiDuToolReceipt {
            id: "weidu".to_owned(),
            version: "24900".to_owned(),
            length: 2_048,
            sha256: "88".repeat(32),
        }],
        runs: vec![RunReceipt {
            run_id: "eet-core".to_owned(),
            target: GameRoot::Bg2,
            components: vec![0],
            prompts: vec![PromptReceipt {
                expected_output: "BG1 path?".to_owned(),
                answer: "<staged-bg1>".to_owned(),
                matched: true,
            }],
            timing: RunTiming {
                started_at_millis: 12,
                completed_at_millis: 18,
            },
            exit_code: 0,
            warnings: vec!["upstream informational warning".to_owned()],
            invocation_sha256: "99".repeat(32),
            stdout_sha256: "aa".repeat(32),
            stderr_sha256: "bb".repeat(32),
            debug_sha256: "cc".repeat(32),
            log_diff: LogDiffReceipt {
                before_sha256: "dd".repeat(32),
                after_sha256: "ee".repeat(32),
                added: vec![component.clone()],
                removed: Vec::new(),
            },
        }],
        final_state: Some(FinalReceiptState {
            logs: vec![FinalLogReceipt {
                target: GameRoot::Bg2,
                sha256: "ff".repeat(32),
                components: vec![component],
            }],
            bg1_engine_name: "Chriz BG Collection - BG1 - abc123".to_owned(),
            bg2_engine_name: "Chriz BG Collection - abc123".to_owned(),
            managed_save_root: root.join("Documents/Chriz BG Collection - abc123"),
            launch_path: root.join("game/InfinityLoader.exe"),
            verification_summary: "exact final WeiDU.log matched".to_owned(),
        }),
    }
}

fn setup() -> (TempDir, PathBuf, ReceiptStore) {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("managed");
    std::fs::create_dir_all(root.join(".chriz/attempts/attempt-001")).unwrap();
    let store = ReceiptStore::open(&root, "install-001").unwrap();
    (temp, root, store)
}

#[test]
fn publishes_a_complete_success_receipt_create_once_in_both_locations() {
    let (_temp, root, store) = setup();
    let receipt = success_receipt(&root);
    let canonical_root = std::fs::canonicalize(&root).unwrap();

    let published = store.publish(&receipt).unwrap();

    assert_eq!(
        published.attempt_receipt,
        canonical_root.join(".chriz/attempts/attempt-001/receipt.json")
    );
    assert_eq!(
        published.install_receipt,
        Some(canonical_root.join(".chriz/install-receipt.json"))
    );
    let stored: InstallReceipt =
        serde_json::from_slice(&std::fs::read(&published.attempt_receipt).unwrap()).unwrap();
    assert_eq!(stored, receipt);
    assert_eq!(stored.evidence_attempt_id, "attempt-001");
    assert_eq!(stored.managed_root, root);
    assert_eq!(stored.staged_bg1, root.join("bg1"));
    assert_eq!(stored.staged_bg2, root.join("game"));
    assert!(stored.normalized_selection.features["recommended"]);
    assert_eq!(
        std::fs::read(root.join(".chriz/install-receipt.json")).unwrap(),
        std::fs::read(&published.attempt_receipt).unwrap()
    );

    // Exact replay is crash-safe and does not replace either create-once file.
    let before = std::fs::metadata(&published.attempt_receipt)
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(store.publish(&receipt).unwrap(), published);
    assert_eq!(
        std::fs::metadata(&published.attempt_receipt)
            .unwrap()
            .modified()
            .unwrap(),
        before
    );
}

#[test]
fn a_retry_cannot_replace_an_existing_success_receipt() {
    let (_temp, root, store) = setup();
    let original = success_receipt(&root);
    let published = store.publish(&original).unwrap();
    let before = std::fs::read(&published.attempt_receipt).unwrap();
    let mut changed = original;
    changed.completed_at_millis += 1;

    let error = store.publish(&changed).unwrap_err();

    assert!(error.to_string().contains("create-once"), "{error}");
    assert_eq!(std::fs::read(&published.attempt_receipt).unwrap(), before);
    assert_eq!(
        std::fs::read(root.join(".chriz/install-receipt.json")).unwrap(),
        before
    );
}

#[test]
fn failure_receipts_are_immutable_and_never_claim_install_success() {
    let (_temp, root, store) = setup();
    let mut receipt = success_receipt(&root);
    receipt.outcome = ReceiptOutcome::Failed {
        step_id: "install:eet-core".to_owned(),
        detail: "unexpected prompt".to_owned(),
    };
    receipt.final_state = None;

    let published = store.publish(&receipt).unwrap();

    assert!(published.attempt_receipt.is_file());
    assert_eq!(published.install_receipt, None);
    assert!(!root.join(".chriz/install-receipt.json").exists());
    let stored: InstallReceipt =
        serde_json::from_slice(&std::fs::read(published.attempt_receipt).unwrap()).unwrap();
    assert_eq!(stored.outcome, receipt.outcome);
}

#[test]
fn task13_receipt_seam_publishes_the_real_receipt_and_registry_record() {
    let temp = TempDir::new().unwrap();
    let managed = temp.path().join("managed");
    let cache = temp.path().join("cache");
    let app_data = temp.path().join("app-data");
    std::fs::create_dir_all(managed.join(".chriz/attempts/attempt-001")).unwrap();
    std::fs::create_dir_all(managed.join("game")).unwrap();
    std::fs::write(
        managed.join("game/InfinityLoader.exe"),
        b"verified launcher",
    )
    .unwrap();
    std::fs::create_dir_all(&cache).unwrap();
    let store = ReceiptStore::open(&managed, "install-001").unwrap();
    let registry = bg_engine::registry::ManagedInstallRegistry::open_or_create(&app_data).unwrap();
    let canonical_managed = std::fs::canonicalize(&managed).unwrap();
    let full = success_receipt(&canonical_managed);
    let evidence = ReceiptEvidence {
        versions: full.versions.clone(),
        source_games: full.source_games.clone(),
        artifacts: full.artifacts.clone(),
        weidu_tools: full.weidu_tools.clone(),
        runs: full.runs.clone(),
        final_state: full.final_state.clone(),
    };
    let selected = NormalizedSelection {
        platform: "windows".to_owned(),
        features: std::collections::BTreeMap::new(),
        inputs: std::collections::BTreeMap::new(),
    };
    let frozen_plan = plan();
    let created = CampaignCreated {
        install_id: "install-001".to_owned(),
        attempt_id: "attempt-001".to_owned(),
        managed_root: canonical_managed,
        cache_root: std::fs::canonicalize(cache).unwrap(),
        recipe_payload: b"recipe".to_vec(),
        recipe_payload_sha256: bg_engine::digest::sha256_bytes(b"recipe"),
        recipe_envelope: b"envelope".to_vec(),
        recipe_envelope_sha256: bg_engine::digest::sha256_bytes(b"envelope"),
        selection_sha256: bg_engine::digest::selection_digest(&selected).unwrap(),
        normalized_selection: selected,
        plan_sha256: bg_engine::digest::plan_digest(&frozen_plan).unwrap(),
        source_games: SourceGameFingerprints {
            bg1: "11".repeat(32),
            bg2: "22".repeat(32),
        },
        artifact_identities: vec![FrozenIdentity {
            id: "eet".to_owned(),
            version: "2.7.3".to_owned(),
            sha256: "77".repeat(32),
            length: 1_024,
        }],
        tool_identities: vec![FrozenIdentity {
            id: "weidu".to_owned(),
            version: "24900".to_owned(),
            sha256: "88".repeat(32),
            length: 2_048,
        }],
        staged_bg1: managed.join("bg1"),
        staged_bg2: managed.join("game"),
    };
    let draft = ReceiptDraft {
        install_id: "install-001".to_owned(),
        attempt_id: "attempt-001".to_owned(),
        started_at_millis: 10,
        completed_at_millis: 20,
        outcome: ReceiptDraftOutcome::Succeeded,
        created,
        plan: frozen_plan,
    };
    let mut writer = ManagedReceiptWriter::new(
        store,
        registry,
        "Chriz BG Collection Alpha".to_owned(),
        evidence,
    );

    ReceiptWriter::write(&mut writer, &draft).unwrap();

    let published: InstallReceipt = serde_json::from_slice(
        &std::fs::read(managed.join(".chriz/install-receipt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(published.plan, draft.plan);
    assert_eq!(published.plan_sha256, draft.created.plan_sha256);
    assert_eq!(
        published.recipe_payload_sha256,
        draft.created.recipe_payload_sha256
    );
    let cards = bg_engine::registry::ManagedInstallRegistry::open_or_create(&app_data)
        .unwrap()
        .list()
        .unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].record.install_id, "install-001");
    assert_eq!(cards[0].record.managed_root, draft.created.managed_root);
}

#[test]
fn real_writer_publishes_failure_attempt_without_claiming_or_registering_success() {
    let temp = TempDir::new().unwrap();
    let managed = temp.path().join("managed");
    let cache = temp.path().join("cache");
    let app_data = temp.path().join("app-data");
    std::fs::create_dir_all(managed.join(".chriz/attempts/attempt-001")).unwrap();
    std::fs::create_dir_all(&cache).unwrap();
    let store = ReceiptStore::open(&managed, "install-001").unwrap();
    let registry = bg_engine::registry::ManagedInstallRegistry::open_or_create(&app_data).unwrap();
    let canonical_managed = std::fs::canonicalize(&managed).unwrap();
    let full = success_receipt(&canonical_managed);
    let evidence = ReceiptEvidence {
        versions: full.versions,
        source_games: full.source_games,
        artifacts: Vec::new(),
        weidu_tools: Vec::new(),
        runs: Vec::new(),
        final_state: None,
    };
    let selected = full.normalized_selection;
    let frozen_plan = plan();
    let created = CampaignCreated {
        install_id: "install-001".to_owned(),
        attempt_id: "attempt-001".to_owned(),
        managed_root: canonical_managed.clone(),
        cache_root: std::fs::canonicalize(cache).unwrap(),
        recipe_payload: b"recipe".to_vec(),
        recipe_payload_sha256: bg_engine::digest::sha256_bytes(b"recipe"),
        recipe_envelope: b"envelope".to_vec(),
        recipe_envelope_sha256: bg_engine::digest::sha256_bytes(b"envelope"),
        selection_sha256: bg_engine::digest::selection_digest(&selected).unwrap(),
        normalized_selection: selected,
        plan_sha256: bg_engine::digest::plan_digest(&frozen_plan).unwrap(),
        source_games: SourceGameFingerprints {
            bg1: "11".repeat(32),
            bg2: "22".repeat(32),
        },
        artifact_identities: vec![frozen("eet", "77", 1_024)],
        tool_identities: vec![frozen("weidu", "88", 2_048)],
        staged_bg1: canonical_managed.join("bg1"),
        staged_bg2: canonical_managed.join("game"),
    };
    let draft = ReceiptDraft {
        install_id: "install-001".to_owned(),
        attempt_id: "terminal-0000000004-aaaaaaaaaaaaaaaa".to_owned(),
        started_at_millis: 10,
        completed_at_millis: 15,
        outcome: ReceiptDraftOutcome::Failed {
            step_id: "stage:bg2".to_owned(),
            detail: "copy failed".to_owned(),
        },
        created,
        plan: frozen_plan,
    };
    let mut writer = ManagedReceiptWriter::new(
        store,
        registry,
        "Chriz BG Collection Alpha".to_owned(),
        evidence,
    );

    ReceiptWriter::write(&mut writer, &draft).unwrap();

    let attempt = managed.join(".chriz/attempts/terminal-0000000004-aaaaaaaaaaaaaaaa/receipt.json");
    let receipt: InstallReceipt = serde_json::from_slice(&std::fs::read(attempt).unwrap()).unwrap();
    assert!(matches!(
        receipt.outcome,
        ReceiptOutcome::Failed { ref step_id, .. } if step_id == "stage:bg2"
    ));
    assert!(receipt.final_state.is_none());
    assert!(!managed.join(".chriz/install-receipt.json").exists());
    assert!(
        bg_engine::registry::ManagedInstallRegistry::open_or_create(&app_data)
            .unwrap()
            .list()
            .unwrap()
            .is_empty()
    );
}
