//! Acceptance adapter for the supervised Combined-20260908 component-256 append.
//! Default is read-only. `--apply` publishes a separate recovery receipt and finalizes
//! the isolated save identity; neither mode runs WeiDU or rewrites the failed receipt.
use bg_engine::{
    cli::{accept_supervised_recovery, inspect_supervised_recovery, SupervisedRecoveryRequest},
    digest::sha256_bytes,
    recovery_receipt::{AppendMissingComponentAuthorization, ArtifactReplacement, EvidenceFile},
    session::{FrozenIdentity, SessionStore},
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const ROOT: &str = r"C:\Users\chris\CEBG-Tests\Combined-20260908";
const RECIPE_SHA256: &str = "40712587d45b8361d9be1393bc5125a851dbd358b44a7a7aff620af822868a3d";
const FIXED_SHA256: &str = "453906a1157bdccfcf4778eafe86c891061bb5acf4a2f254afda94ec5b44fdeb";
const FIXED_LENGTH: u64 = 558_607;
const FIXED_COMMIT: &str = "29e123a79b9f03334ab88ce93e28c300b287a8e0";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.len() > 1 || arguments.first().is_some_and(|arg| arg != "--apply") {
        return Err("usage: accept_combined_bridge_recovery [--apply]".into());
    }
    let root = fs::canonicalize(ROOT)?;
    let replay = SessionStore::open(&root)?.replay()?;
    let created = replay.created();
    if created.install_id != "install-d976d775d15d76c8bdc0"
        || created.recipe_payload_sha256 != RECIPE_SHA256
    {
        return Err("not the approved Combined-20260908 campaign".into());
    }

    let relative_recovery = PathBuf::from(".chriz/recoveries/sod-256-append-20260909");
    let recovery = root.join(&relative_recovery);
    let intent: serde_json::Value =
        serde_json::from_slice(&fs::read(recovery.join("intent.json"))?)?;
    if intent["source_commit"] != FIXED_COMMIT
        || intent["fixed_archive_sha256"] != FIXED_SHA256
        || intent["component"] != 256
        || intent["preserved_active_rows"] != 364
    {
        return Err("prepared recovery intent differs from the approved bridge fix".into());
    }

    let mut evidence = BTreeMap::new();
    for item in intent["protected_files"]
        .as_array()
        .ok_or("missing protected history")?
    {
        let path = fs::canonicalize(item["path"].as_str().ok_or("missing protected path")?)?;
        let relative = path.strip_prefix(&root)?.to_path_buf();
        evidence.insert(
            relative,
            item["sha256"]
                .as_str()
                .ok_or("missing protected digest")?
                .to_owned(),
        );
    }
    for entry in fs::read_dir(&recovery)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err("unexpected recovery directory entry".into());
        }
        let path = entry.path();
        evidence.insert(
            path.strip_prefix(&root)?.to_path_buf(),
            sha256_bytes(&fs::read(&path)?),
        );
    }
    if evidence
        .get(&relative_recovery.join("before-state.zip"))
        .map(String::as_str)
        != intent["backup_sha256"].as_str()
    {
        return Err("scoped recovery backup changed".into());
    }

    let original_step = PathBuf::from(
        ".chriz/attempts/attempt-656080e90d82e0a12f12/steps/0116-c656b8d09bdc1ad9/attempt-0001",
    );
    for name in [
        "before.log",
        "after.log",
        "stdout.log",
        "stderr.log",
        "weidu.debug.log",
        "invocation.json",
        "process-result.json",
    ] {
        let path = original_step.join(name);
        evidence.insert(path.clone(), sha256_bytes(&fs::read(root.join(&path))?));
    }

    let original = created
        .artifact_identities
        .iter()
        .find(|item| item.id == "local-playtest-chriz-sod-remix-80210e083ff3")
        .ok_or("original SoD artifact identity missing")?
        .clone();
    let replacement = FrozenIdentity {
        id: original.id.clone(),
        version: FIXED_COMMIT.to_owned(),
        length: FIXED_LENGTH,
        sha256: FIXED_SHA256.to_owned(),
    };
    let installed_components = vec![
        100, 110, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 210, 197, 187,
        200, 215, 220, 225, 245, 230, 240, 250, 255, 260, 265, 270, 280, 290, 900, 910,
    ];
    let mut final_components = installed_components.clone();
    final_components.push(256);
    let authorization = AppendMissingComponentAuthorization {
        run_id: "chriz-sod-remix-bg2".to_owned(),
        component: 256,
        installed_components,
        final_components,
        source_commit: FIXED_COMMIT.to_owned(),
        preserved_active_rows: 364,
        new_files: 21,
        edited_files: 6,
        removed_files: 0,
        later_sibling_write_overlaps: 0,
        source_evidence: relative_recovery.join("source-provenance.json"),
        compatibility_evidence: relative_recovery.join("compatibility-evidence.json"),
    };
    let operation_stems = [
        "chriz-sod-remix-256",
        "hiddengameplayoptions-bg2-attempt2",
        "chriz-bg-rebalance-bg2",
        "chriz-bg-modpack-bg2",
        "cdtweaks-spell-save-penalties-bg2",
        "safana-bg2",
        "chriz-bg-modpack-late-companions-bg2",
        "spell-rev-npc-spellbooks-bg2",
        "spell-rev-lightning-bg2",
        "klatu-armor-thieving-bg2",
        "buffbot-bg2",
    ]
    .map(|name| relative_recovery.join(name))
    .to_vec();

    let request = SupervisedRecoveryRequest {
        managed_root: root,
        recovery_id: "sod-256-append-20260909".to_owned(),
        replacement: ArtifactReplacement {
            run_id: "chriz-sod-remix-bg2".to_owned(),
            original,
            replacement,
        },
        replacement_archive: Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().ok_or("missing workspace")?
            .join("target/combined-playtest-20260908/recovery-20260909/chriz-sod-remix-29e123a-recovery-20260909.zip"),
        before_log: original_step.join("before.log"),
        partial_log: original_step.join("after.log"),
        partial_output: original_step.join("stdout.log"),
        operation_stems,
        append_missing_component: Some(authorization),
        evidence: evidence.into_iter().map(|(path, sha256)| EvidenceFile {
            path: PathBuf::from(path.to_string_lossy().replace('\\', "/")),
            sha256,
        }).collect(),
    };

    if arguments.is_empty() {
        let logs = inspect_supervised_recovery(&request)?;
        println!(
            "Read-only Combined bridge recovery acceptance passed: {} exact components. No files changed.",
            logs.iter().map(|log| log.components.len()).sum::<usize>()
        );
    } else {
        let receipt = accept_supervised_recovery(&request)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "kind": receipt.kind,
                "install_id": receipt.install_id,
                "version": receipt.effective_version(),
                "launch_path": receipt.final_state.launch_path,
                "save_root": receipt.final_state.managed_save_root,
                "components": receipt.final_state.logs.iter().map(|log| log.components.len()).sum::<usize>(),
                "original_campaign_outcome": "fresh_copy_required",
                "gameplay_smoke_tested": false
            }))?
        );
    }
    Ok(())
}
