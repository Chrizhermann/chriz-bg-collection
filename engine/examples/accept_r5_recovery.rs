//! One supervised acceptance adapter for the already completed r5 repair.
//! Default is read-only. `--apply` finalizes save identity and publishes metadata;
//! neither mode runs an install, copies a game, or changes the historical ledger.
use bg_engine::{
    cli::{accept_supervised_recovery, inspect_supervised_recovery, SupervisedRecoveryRequest},
    digest::sha256_bytes,
    recovery_receipt::{ArtifactReplacement, EvidenceFile},
    session::{FrozenIdentity, SessionStore},
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.len() > 1 || arguments.first().is_some_and(|arg| arg != "--apply") {
        return Err("usage: accept_r5_recovery [--apply]".into());
    }
    let root = fs::canonicalize(r"C:\Users\chris\Games\CEBG-Curated-20260905-r5")?;
    let replay = SessionStore::open(&root)?.replay()?;
    let created = replay.created();
    if created.install_id != "install-e6325c7c98451ad4901e"
        || created.recipe_payload_sha256
            != "cb220e5de749a779ec0e87337534fd803c11805375011e4b2856984120b4da34"
    {
        return Err("not the approved r5 campaign".into());
    }
    let relative_recovery = PathBuf::from(".chriz/recoveries/modpack-alpha5-20260906");
    let recovery = root.join(&relative_recovery);
    let intent: serde_json::Value =
        serde_json::from_slice(&fs::read(recovery.join("intent.json"))?)?;
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
    // Keep the backup tied to its PRE-repair digest, not merely its current bytes.
    if evidence
        .get(&relative_recovery.join("before-state.zip"))
        .map(String::as_str)
        != intent["backup_sha256"].as_str()
    {
        return Err("scoped recovery backup changed".into());
    }
    let original_step = PathBuf::from(
        ".chriz/attempts/attempt-aba3cbb32e1956c690b8/steps/0110-22791e178261b9e3/attempt-0001",
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
        evidence.insert(path.clone(), sha256_bytes(&fs::read(root.join(path))?));
    }
    let original = created
        .artifact_identities
        .iter()
        .find(|item| item.id == "chriz-bg-modpack-0.2.0-alpha.1")
        .ok_or("original modpack identity missing")?
        .clone();
    let replacement = FrozenIdentity {
        id: original.id.clone(),
        version: "0.2.0-alpha.5".to_owned(),
        length: 1_335_026,
        sha256: "2278c839f60e019bedba355cb794176a052d24b68db2851248af926580840b33".to_owned(),
    };
    let request = SupervisedRecoveryRequest {
        managed_root: root,
        recovery_id: "modpack-alpha5-20260906".to_owned(),
        replacement: ArtifactReplacement {
            run_id: "chriz-bg-modpack-bg2".to_owned(),
            original,
            replacement,
        },
        replacement_archive: Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("missing workspace")?
            .join("target/alpha9-source-verification/chriz-bg-modpack-v0.2.0-alpha.5.zip"),
        before_log: original_step.join("before.log"),
        partial_log: original_step.join("after.log"),
        partial_output: original_step.join("stdout.log"),
        operation_stems: [
            "uninstall",
            "install",
            "cdtweaks-spell-save-penalties-bg2",
            "spell-rev-npc-spellbooks-bg2",
            "buffbot-bg2",
        ]
        .map(|name| relative_recovery.join(name))
        .to_vec(),
        evidence: evidence
            .into_iter()
            .map(|(path, sha256)| EvidenceFile {
                path: PathBuf::from(path.to_string_lossy().replace('\\', "/")),
                sha256,
            })
            .collect(),
    };
    if arguments.is_empty() {
        let logs = inspect_supervised_recovery(&request)?;
        println!(
            "Read-only recovery acceptance passed: {} exact components. No files changed.",
            logs.iter().map(|log| log.components.len()).sum::<usize>()
        );
    } else {
        let receipt = accept_supervised_recovery(&request)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "kind": receipt.kind, "install_id": receipt.install_id,
                "version": receipt.effective_version(), "launch_path": receipt.final_state.launch_path,
                "save_root": receipt.final_state.managed_save_root,
                "components": receipt.final_state.logs.iter().map(|log| log.components.len()).sum::<usize>(),
                "original_campaign_outcome": "fresh_copy_required", "gameplay_smoke_tested": false
            }))?
        );
    }
    Ok(())
}
