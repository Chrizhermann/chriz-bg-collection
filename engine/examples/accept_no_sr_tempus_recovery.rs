//! Supervised acceptance adapter for the completed No-SR Tempus tail repair.
//! Default is read-only. `--apply` finalizes save identity and publishes metadata;
//! neither mode runs WeiDU, reopens the historical ledger, or replaces game files.
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

const REPLACEMENT_SHA256: &str = "25480a8e597d316d3cf1799f641971f3b6edb113eea24da7f45a8dd70b0a9ef4";
const REPLACEMENT_LENGTH: u64 = 1_369_825;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.len() > 1 || arguments.first().is_some_and(|arg| arg != "--apply") {
        return Err("usage: accept_no_sr_tempus_recovery [--apply]".into());
    }
    let root = fs::canonicalize(r"C:\Users\chris\Games\ChrizEasyBG-No-SR-Test")?;
    let replay = SessionStore::open(&root)?.replay()?;
    let created = replay.created();
    if created.install_id != "install-c9dc8c3e0e0b0a3476e0"
        || created.recipe_payload_sha256
            != "38d783a4683933404054478bd3e229ee51b9c5bab0040c48ed3fa7fe86f3e848"
    {
        return Err("not the approved No-SR campaign".into());
    }

    let relative_recovery = PathBuf::from(".chriz/recoveries/tempus-v032-20260907");
    let recovery = root.join(&relative_recovery);
    let intent: serde_json::Value =
        serde_json::from_slice(&fs::read(recovery.join("intent.json"))?)?;
    let replacement_intent = intent
        .get("replacement")
        .ok_or("missing replacement release identity")?;
    if replacement_intent["id"] != "chriz-bg-rebalance-0.3.1"
        || replacement_intent["version"] != "0.3.2"
        || replacement_intent["reference"] != "v0.3.2"
        || replacement_intent["url"]
            != "https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.3.2/chriz-bg-rebalance-v0.3.2.zip"
        || replacement_intent["expected_filename"] != "chriz-bg-rebalance-v0.3.2.zip"
        || replacement_intent["sha256"] != REPLACEMENT_SHA256
        || replacement_intent["length"] != REPLACEMENT_LENGTH
    {
        return Err("prepared replacement differs from the approved v0.3.2 release".into());
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
        ".chriz/attempts/attempt-e6b6f3a5aacc11891051/steps/0106-fab86e09514d1f0a/attempt-0001",
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
        .find(|item| item.id == "chriz-bg-rebalance-0.3.1")
        .ok_or("original BG Rebalance identity missing")?
        .clone();
    let replacement = FrozenIdentity {
        id: original.id.clone(),
        version: "0.3.2".to_owned(),
        length: REPLACEMENT_LENGTH,
        sha256: REPLACEMENT_SHA256.to_owned(),
    };
    let request = SupervisedRecoveryRequest {
        managed_root: root,
        recovery_id: "tempus-v032-20260907".to_owned(),
        replacement: ArtifactReplacement {
            run_id: "chriz-bg-rebalance-bg2".to_owned(),
            original,
            replacement,
        },
        replacement_archive: Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("missing workspace")?
            .join("target/alpha15-source-verification/chriz-bg-rebalance-v0.3.2.zip"),
        before_log: original_step.join("before.log"),
        partial_log: original_step.join("after.log"),
        partial_output: original_step.join("stdout.log"),
        operation_stems: [
            "uninstall",
            "install",
            "chriz-bg-modpack-bg2",
            "cdtweaks-spell-save-penalties-bg2",
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
            "Read-only No-SR recovery acceptance passed: {} exact components. No files changed.",
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
