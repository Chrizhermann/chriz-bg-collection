use std::fs;
use std::path::{Path, PathBuf};

use bg_engine::digest::{plan_digest, sha256_bytes};
use bg_engine::manifest::{GameRoot, Phase};
use bg_engine::receipt::{
    ArtifactCacheOutcome, ArtifactReceipt, FinalLogReceipt, FinalReceiptState, InstallReceipt,
    LogComponentReceipt, ReceiptOutcome, ReceiptVersions, RECEIPT_SCHEMA_VERSION,
};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::recovery_receipt::{
    publish, read_completed_state, verify_evidence, ArtifactReplacement, EvidenceFile,
    RecoveryKind, RecoveryReceipt, RECOVERY_RECEIPT_SCHEMA_VERSION,
};
use bg_engine::resolve::{InstallPlan, PlannedRun};
use bg_engine::session::FrozenIdentity;
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    base: InstallReceipt,
    recovery: RecoveryReceipt,
}

fn frozen(id: &str, version: &str, byte: &str, length: u64) -> FrozenIdentity {
    FrozenIdentity {
        id: id.to_owned(),
        version: version.to_owned(),
        sha256: byte.repeat(32),
        length,
    }
}

fn artifact(identity: &FrozenIdentity) -> ArtifactReceipt {
    ArtifactReceipt {
        id: identity.id.clone(),
        version: identity.version.clone(),
        original_url: "https://example.invalid/original.zip".to_owned(),
        final_url: "https://example.invalid/original.zip".to_owned(),
        length: identity.length,
        sha256: identity.sha256.clone(),
        cache_outcome: ArtifactCacheOutcome::Hit,
    }
}

fn planned_run(
    run_id: &str,
    target: GameRoot,
    artifact_id: &str,
    components: &[u32],
) -> PlannedRun {
    PlannedRun {
        run_id: run_id.to_owned(),
        mod_id: run_id.to_owned(),
        target,
        phase: match target {
            GameRoot::Bg1 => Phase::Bg1Preparation,
            GameRoot::Bg2 => Phase::Main,
        },
        components: components.to_vec(),
        args: Vec::new(),
        postconditions: Vec::new(),
        artifact_id: artifact_id.to_owned(),
        weidu_artifact_id: "weidu".to_owned(),
        prompt_scripts: Vec::new(),
    }
}

fn receipt_bytes(receipt: &InstallReceipt) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(receipt).unwrap();
    bytes.push(b'\n');
    bytes
}

fn write_base(root: &Path, receipt: &InstallReceipt) -> Vec<u8> {
    let path = root
        .join(".chriz/attempts")
        .join(&receipt.attempt_id)
        .join("receipt.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let bytes = receipt_bytes(receipt);
    fs::write(path, &bytes).unwrap();
    bytes
}

fn final_state(root: &Path, bg1_sha256: &str, bg2_sha256: &str) -> FinalReceiptState {
    FinalReceiptState {
        logs: vec![
            FinalLogReceipt {
                target: GameRoot::Bg1,
                sha256: bg1_sha256.to_owned(),
                components: vec![LogComponentReceipt {
                    tp2: "bg1/setup-bg1.tp2".to_owned(),
                    language: 0,
                    component: 1,
                }],
            },
            FinalLogReceipt {
                target: GameRoot::Bg2,
                sha256: bg2_sha256.to_owned(),
                components: vec![
                    LogComponentReceipt {
                        tp2: "mods/setup-changed.tp2".to_owned(),
                        language: 0,
                        component: 10,
                    },
                    LogComponentReceipt {
                        tp2: "mods/setup-changed.tp2".to_owned(),
                        language: 0,
                        component: 20,
                    },
                ],
            },
        ],
        bg1_engine_name: "CEBG-Recovery-BG1".to_owned(),
        bg2_engine_name: "CEBG-Recovery-BG2".to_owned(),
        managed_save_root: root.parent().unwrap().join("managed-save"),
        launch_path: root.join("Baldur.exe"),
        verification_summary: "Recovered stack exactly matches the frozen plan.".to_owned(),
    }
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("managed");
    fs::create_dir_all(root.join("bg1")).unwrap();
    let root = fs::canonicalize(root).unwrap();
    let original_bg1 = frozen("bg1-artifact", "1.0", "11", 101);
    let original_changed = frozen("changed-artifact", "alpha.1", "22", 202);
    let plan = InstallPlan {
        runs: vec![
            planned_run("bg1-run", GameRoot::Bg1, &original_bg1.id, &[1]),
            planned_run(
                "changed-run",
                GameRoot::Bg2,
                &original_changed.id,
                &[10, 20],
            ),
        ],
    };
    let base = InstallReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        install_id: "install-001".to_owned(),
        attempt_id: "base-attempt".to_owned(),
        evidence_attempt_id: "base-attempt".to_owned(),
        managed_root: root.clone(),
        staged_bg1: root.join("bg1"),
        staged_bg2: root.clone(),
        started_at_millis: 100,
        completed_at_millis: 200,
        outcome: ReceiptOutcome::Failed {
            step_id: "run:changed-run".to_owned(),
            detail: "original artifact failed".to_owned(),
        },
        versions: ReceiptVersions {
            application: "0.1.0-alpha.8".to_owned(),
            engine: "0.1.0".to_owned(),
            manifest_schema: 2,
            recipe: "0.1.0-alpha.8".to_owned(),
        },
        source_games: Vec::new(),
        recipe_payload_sha256: "33".repeat(32),
        recipe_envelope_sha256: "44".repeat(32),
        selection_sha256: "55".repeat(32),
        normalized_selection: NormalizedSelection {
            platform: "windows".to_owned(),
            features: Default::default(),
            inputs: Default::default(),
        },
        plan_sha256: plan_digest(&plan).unwrap(),
        plan,
        artifacts: vec![artifact(&original_bg1), artifact(&original_changed)],
        weidu_tools: Vec::new(),
        runs: Vec::new(),
        final_state: None,
    };
    let base_bytes = write_base(&root, &base);

    let bg1_path = PathBuf::from("bg1/WeiDU.log");
    let bg2_path = PathBuf::from("WeiDU.log");
    let bg1_bytes = b"BG1 final log\n";
    let bg2_bytes = b"BG2 final log\n";
    fs::write(root.join(&bg1_path), bg1_bytes).unwrap();
    fs::write(root.join(&bg2_path), bg2_bytes).unwrap();
    let bg1_sha256 = sha256_bytes(bg1_bytes);
    let bg2_sha256 = sha256_bytes(bg2_bytes);
    let recovery = RecoveryReceipt {
        schema_version: RECOVERY_RECEIPT_SCHEMA_VERSION,
        kind: RecoveryKind::SupervisedRecovery,
        recovery_id: "recovery-001".to_owned(),
        install_id: base.install_id.clone(),
        managed_root: root.clone(),
        base_attempt_id: base.attempt_id.clone(),
        base_receipt_sha256: sha256_bytes(&base_bytes),
        base_recipe_version: base.versions.recipe.clone(),
        base_recipe_payload_sha256: base.recipe_payload_sha256.clone(),
        base_plan_sha256: base.plan_sha256.clone(),
        replacements: vec![ArtifactReplacement {
            run_id: "changed-run".to_owned(),
            original: original_changed,
            replacement: frozen("changed-artifact", "alpha.5", "66", 606),
        }],
        evidence: vec![
            EvidenceFile {
                path: bg1_path,
                sha256: bg1_sha256.clone(),
            },
            EvidenceFile {
                path: bg2_path,
                sha256: bg2_sha256.clone(),
            },
        ],
        completed_at_millis: 300,
        final_state: final_state(&root, &bg1_sha256, &bg2_sha256),
    };
    Fixture {
        _temp: temp,
        root,
        base,
        recovery,
    }
}

#[test]
fn valid_recovery_links_failed_base_and_reports_truthful_effective_version() {
    let fixture = fixture();

    fixture.recovery.validate(&fixture.base).unwrap();

    assert_eq!(
        fixture.recovery.effective_version(),
        "0.1.0-alpha.8 (repaired)"
    );
    assert_eq!(
        serde_json::to_value(&fixture.recovery).unwrap()["kind"],
        "supervised_recovery"
    );
}

#[test]
fn recovery_accepts_failed_or_fresh_copy_base_but_never_success() {
    let fixture = fixture();
    let mut fresh_copy = fixture.base.clone();
    fresh_copy.outcome = ReceiptOutcome::FreshCopyRequired {
        step_id: "run:changed-run".to_owned(),
        detail: "needs supervised repair".to_owned(),
    };
    let mut recovery = fixture.recovery.clone();
    recovery.base_receipt_sha256 = sha256_bytes(&receipt_bytes(&fresh_copy));
    assert!(recovery.validate(&fresh_copy).is_ok());

    let mut succeeded = fixture.base.clone();
    succeeded.outcome = ReceiptOutcome::Succeeded;
    succeeded.final_state = Some(fixture.recovery.final_state.clone());
    recovery.base_receipt_sha256 = sha256_bytes(&receipt_bytes(&succeeded));

    assert!(recovery
        .validate(&succeeded)
        .unwrap_err()
        .contains("failed"));
}

#[test]
fn recovery_rejects_any_broken_base_lineage_link() {
    let fixture = fixture();
    let mut cases = Vec::new();
    let mut changed = fixture.recovery.clone();
    changed.recovery_id = ".".to_owned();
    cases.push(changed);
    let mut changed = fixture.recovery.clone();
    changed.base_attempt_id = "..".to_owned();
    cases.push(changed);
    let mut changed = fixture.recovery.clone();
    changed.install_id.push_str("-other");
    cases.push(changed);
    let mut changed = fixture.recovery.clone();
    changed.base_attempt_id.push_str("-other");
    cases.push(changed);
    let mut changed = fixture.recovery.clone();
    changed.base_receipt_sha256 = "00".repeat(32);
    cases.push(changed);
    let mut changed = fixture.recovery.clone();
    changed.base_recipe_version.push_str("-other");
    cases.push(changed);
    let mut changed = fixture.recovery.clone();
    changed.base_recipe_payload_sha256 = "00".repeat(32);
    cases.push(changed);
    let mut changed = fixture.recovery.clone();
    changed.base_plan_sha256 = "00".repeat(32);
    cases.push(changed);

    for changed in cases {
        assert!(changed.validate(&fixture.base).is_err(), "{changed:#?}");
    }
}

#[test]
fn replacement_must_name_one_plan_run_and_its_exact_original_artifact() {
    let fixture = fixture();
    let mut unknown_run = fixture.recovery.clone();
    unknown_run.replacements[0].run_id = "unknown-run".to_owned();
    assert!(unknown_run.validate(&fixture.base).is_err());

    let mut wrong_original = fixture.recovery.clone();
    wrong_original.replacements[0].original.version = "invented".to_owned();
    assert!(wrong_original.validate(&fixture.base).is_err());

    let mut wrong_replacement = fixture.recovery.clone();
    wrong_replacement.replacements[0].replacement.id = "different-artifact".to_owned();
    assert!(wrong_replacement.validate(&fixture.base).is_err());

    let mut empty_version = fixture.recovery.clone();
    empty_version.replacements[0].replacement.version.clear();
    assert!(empty_version.validate(&fixture.base).is_err());

    let mut invalid_digest = fixture.recovery.clone();
    invalid_digest.replacements[0].replacement.sha256 = "not-a-digest".to_owned();
    assert!(invalid_digest.validate(&fixture.base).is_err());

    let mut zero_length = fixture.recovery.clone();
    zero_length.replacements[0].replacement.length = 0;
    assert!(zero_length.validate(&fixture.base).is_err());

    let mut duplicate_run = fixture.recovery.clone();
    duplicate_run
        .replacements
        .push(duplicate_run.replacements[0].clone());
    assert!(duplicate_run.validate(&fixture.base).is_err());
}

#[test]
fn evidence_manifest_is_nonempty_unique_bounded_and_target_relative() {
    let fixture = fixture();
    let mut empty = fixture.recovery.clone();
    empty.evidence.clear();
    assert!(empty.validate(&fixture.base).is_err());

    let mut duplicate = fixture.recovery.clone();
    duplicate.evidence.push(duplicate.evidence[0].clone());
    assert!(duplicate.validate(&fixture.base).is_err());

    for unsafe_path in ["../escape", "C:/escape", "/rooted", "bad:name"] {
        let mut unsafe_receipt = fixture.recovery.clone();
        unsafe_receipt.evidence[0].path = PathBuf::from(unsafe_path);
        assert!(
            unsafe_receipt.validate(&fixture.base).is_err(),
            "accepted {unsafe_path:?}"
        );
    }
}

#[test]
fn final_state_requires_both_logs_unique_components_and_contained_launch_path() {
    let fixture = fixture();
    let mut missing_bg1 = fixture.recovery.clone();
    missing_bg1
        .final_state
        .logs
        .retain(|log| log.target == GameRoot::Bg2);
    assert!(missing_bg1.validate(&fixture.base).is_err());

    let mut duplicate_component = fixture.recovery.clone();
    let repeated = duplicate_component.final_state.logs[1].components[0].clone();
    duplicate_component.final_state.logs[1]
        .components
        .push(repeated);
    assert!(duplicate_component.validate(&fixture.base).is_err());

    let mut escaped_launch = fixture.recovery.clone();
    escaped_launch.final_state.launch_path = fixture.root.parent().unwrap().join("other.exe");
    assert!(escaped_launch.validate(&fixture.base).is_err());
}

#[test]
fn evidence_verification_detects_tamper_before_publication() {
    let fixture = fixture();
    verify_evidence(&fixture.root, &fixture.recovery.evidence).unwrap();

    let outside = fixture.root.parent().unwrap().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("evidence.txt"), b"outside evidence\n").unwrap();
    let linked = fixture.root.join("linked");
    if create_directory_symlink(&outside, &linked).is_ok() {
        let linked_evidence = vec![EvidenceFile {
            path: PathBuf::from("linked/evidence.txt"),
            sha256: sha256_bytes(b"outside evidence\n"),
        }];
        assert!(verify_evidence(&fixture.root, &linked_evidence).is_err());
    }

    fs::write(fixture.root.join("WeiDU.log"), b"tampered\n").unwrap();

    assert!(verify_evidence(&fixture.root, &fixture.recovery.evidence).is_err());
    assert!(publish(&fixture.root, &fixture.recovery).is_err());
    assert!(!fixture.root.join(".chriz/install-receipt.json").exists());
}

#[test]
fn publish_is_create_once_and_completed_state_accepts_only_success_or_recovery() {
    let published_fixture = fixture();
    let path = publish(&published_fixture.root, &published_fixture.recovery).unwrap();
    assert_eq!(
        path,
        published_fixture.root.join(".chriz/install-receipt.json")
    );
    assert_eq!(
        publish(&published_fixture.root, &published_fixture.recovery).unwrap(),
        path
    );
    assert_eq!(
        read_completed_state(&published_fixture.root).unwrap(),
        published_fixture.recovery.final_state
    );

    let mut conflicting = published_fixture.recovery.clone();
    conflicting
        .final_state
        .verification_summary
        .push_str(" changed");
    assert!(publish(&published_fixture.root, &conflicting)
        .unwrap_err()
        .contains("different bytes"));

    let failed_fixture = fixture();
    fs::write(
        failed_fixture.root.join(".chriz/install-receipt.json"),
        receipt_bytes(&failed_fixture.base),
    )
    .unwrap();
    assert!(read_completed_state(&failed_fixture.root)
        .unwrap_err()
        .contains("not succeeded"));

    let success_fixture = fixture();
    let mut success = success_fixture.base.clone();
    success.outcome = ReceiptOutcome::Succeeded;
    let mut ordinary_final_state = success_fixture.recovery.final_state.clone();
    ordinary_final_state.logs.clear();
    success.final_state = Some(ordinary_final_state.clone());
    fs::write(
        success_fixture.root.join(".chriz/install-receipt.json"),
        receipt_bytes(&success),
    )
    .unwrap();
    assert_eq!(
        read_completed_state(&success_fixture.root).unwrap(),
        ordinary_final_state
    );
}

#[cfg(windows)]
fn create_directory_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(original, link)
}

#[cfg(not(windows))]
fn create_directory_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}
