//! Cheap launcher checks against the successful receipt; no full game-file scan.

use bg_engine::{
    manifest::GameRoot, receipt::FinalLogReceipt, recovery_receipt::read_completed_state,
    weidu::log::parse_active_entries,
};
use serde::Serialize;
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstalledComponentSummary {
    pub target: String,
    pub tp2: String,
    pub component: u32,
    pub title: Option<String>,
    pub version: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConsistencySummary {
    pub state: String,
    pub detail: String,
    pub component_count: usize,
    pub mod_count: usize,
    pub components: Vec<InstalledComponentSummary>,
}

pub fn inspect(managed_root: &Path) -> ConsistencySummary {
    let result =
        read_completed_state(managed_root).and_then(|state| check_logs(managed_root, &state.logs));
    result.unwrap_or_else(|_| ConsistencySummary {
        state: "unavailable".to_owned(),
        detail: "The installed mod list could not be checked.".to_owned(),
        component_count: 0,
        mod_count: 0,
        components: Vec::new(),
    })
}

fn check_logs(root: &Path, logs: &[FinalLogReceipt]) -> Result<ConsistencySummary, String> {
    if logs.is_empty() {
        return Err("No final mod lists.".to_owned());
    }
    let mut count = 0;
    let mut mods = BTreeSet::new();
    let mut changed = false;
    let mut components = Vec::new();
    for expected in logs {
        let (folder, target) = match expected.target {
            GameRoot::Bg1 => ("bg1", "BG1"),
            GameRoot::Bg2 => ("game", "BG2"),
        };
        let path = root.join(folder).join("WeiDU.log");
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > 16 * 1024 * 1024
        {
            return Err("Mod list is not a regular bounded file.".to_owned());
        }
        let bytes = fs::read(path).map_err(|error| error.to_string())?;
        let entries = parse_active_entries(&String::from_utf8_lossy(&bytes))
            .map_err(|error| error.to_string())?;
        count += entries.len();
        mods.extend(entries.iter().map(|entry| entry.tp2_key.clone()));
        changed |= entries.len() != expected.components.len()
            || entries
                .iter()
                .zip(&expected.components)
                .any(|(actual, recorded)| {
                    actual.tp2_key != recorded.tp2
                        || actual.language != recorded.language
                        || actual.component != recorded.component
                });
        let mut consumed = vec![false; entries.len()];
        for recorded in &expected.components {
            let match_index = entries.iter().enumerate().position(|(index, actual)| {
                !consumed[index]
                    && actual.tp2_key == recorded.tp2
                    && actual.language == recorded.language
                    && actual.component == recorded.component
            });
            if let Some(index) = match_index {
                consumed[index] = true;
                let actual = &entries[index];
                let (title, version) = display_annotation(actual.annotation.as_deref());
                components.push(InstalledComponentSummary {
                    target: target.to_owned(),
                    tp2: actual.tp2.clone(),
                    component: actual.component,
                    title,
                    version,
                    status: "installed".to_owned(),
                });
            } else {
                components.push(InstalledComponentSummary {
                    target: target.to_owned(),
                    tp2: recorded.tp2.clone(),
                    component: recorded.component,
                    title: None,
                    version: None,
                    status: "missing".to_owned(),
                });
            }
        }
        components.extend(
            entries
                .iter()
                .zip(consumed)
                .filter_map(|(actual, consumed)| {
                    (!consumed).then(|| {
                        let (title, version) = display_annotation(actual.annotation.as_deref());
                        InstalledComponentSummary {
                            target: target.to_owned(),
                            tp2: actual.tp2.clone(),
                            component: actual.component,
                            title,
                            version,
                            status: "extra".to_owned(),
                        }
                    })
                }),
        );
    }
    Ok(ConsistencySummary {
        state: if changed { "changed" } else { "matches" }.to_owned(),
        detail: if changed {
            "The mod list has changed since CEBG installed this game."
        } else {
            "The mod list matches the completed installation."
        }
        .to_owned(),
        component_count: count,
        mod_count: mods.len(),
        components,
    })
}

fn display_annotation(annotation: Option<&str>) -> (Option<String>, Option<String>) {
    let Some(annotation) = annotation.map(str::trim).filter(|value| !value.is_empty()) else {
        return (None, None);
    };
    match annotation.rsplit_once(": ") {
        Some((title, version)) if !title.trim().is_empty() && !version.trim().is_empty() => (
            Some(title.trim().to_owned()),
            Some(version.trim().to_owned()),
        ),
        _ => (Some(annotation.to_owned()), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bg_engine::{
        digest::{plan_digest, sha256_bytes},
        receipt::{
            FinalReceiptState, InstallReceipt, LogComponentReceipt, ReceiptOutcome, ReceiptVersions,
        },
        recipe_view::NormalizedSelection,
        recovery_receipt::{
            publish, EvidenceFile, RecoveryKind, RecoveryReceipt, RECOVERY_RECEIPT_SCHEMA_VERSION,
        },
        resolve::InstallPlan,
    };
    use std::collections::BTreeMap;

    fn final_state(root: &Path) -> FinalReceiptState {
        let component = |tp2: &str| LogComponentReceipt {
            tp2: tp2.to_owned(),
            language: 0,
            component: 0,
        };
        FinalReceiptState {
            logs: vec![
                FinalLogReceipt {
                    target: GameRoot::Bg1,
                    sha256: "11".repeat(32),
                    components: vec![component("bg1/setup.tp2")],
                },
                FinalLogReceipt {
                    target: GameRoot::Bg2,
                    sha256: "22".repeat(32),
                    components: vec![component("bg2/setup.tp2")],
                },
            ],
            bg1_engine_name: "Test - abc123".to_owned(),
            bg2_engine_name: "Test - abc123".to_owned(),
            managed_save_root: root.join("saves"),
            launch_path: root.join("game/InfinityLoader.exe"),
            verification_summary: "exact logs matched".to_owned(),
        }
    }

    fn write_logs(root: &Path) {
        fs::create_dir_all(root.join("bg1")).unwrap();
        fs::create_dir_all(root.join("game")).unwrap();
        fs::write(root.join("bg1/WeiDU.log"), "~bg1/setup.tp2~ #0 #0\n").unwrap();
        fs::write(root.join("game/WeiDU.log"), "~bg2/setup.tp2~ #0 #0\n").unwrap();
    }

    fn failed_base(root: &Path) -> InstallReceipt {
        let plan = InstallPlan { runs: Vec::new() };
        InstallReceipt {
            schema_version: bg_engine::receipt::RECEIPT_SCHEMA_VERSION,
            install_id: "install-test".to_owned(),
            attempt_id: "attempt-failed".to_owned(),
            evidence_attempt_id: "attempt-failed".to_owned(),
            managed_root: root.to_path_buf(),
            staged_bg1: root.join("bg1"),
            staged_bg2: root.join("game"),
            started_at_millis: 1,
            completed_at_millis: 2,
            outcome: ReceiptOutcome::Failed {
                step_id: "install:test".to_owned(),
                detail: "fixture failure".to_owned(),
            },
            versions: ReceiptVersions {
                application: "0.1.0".to_owned(),
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
                features: BTreeMap::new(),
                inputs: BTreeMap::new(),
            },
            plan_sha256: plan_digest(&plan).unwrap(),
            plan,
            artifacts: Vec::new(),
            weidu_tools: Vec::new(),
            runs: Vec::new(),
            final_state: None,
        }
    }

    fn pretty_json<T: serde::Serialize>(value: &T) -> Vec<u8> {
        let mut bytes = serde_json::to_vec_pretty(value).unwrap();
        bytes.push(b'\n');
        bytes
    }

    #[test]
    fn lazy_check_detects_changed_components_but_ignores_log_comments() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("game")).unwrap();
        let path = temp.path().join("game/WeiDU.log");
        let logs = vec![FinalLogReceipt {
            target: GameRoot::Bg2,
            sha256: "unused-semantic-check".to_owned(),
            components: vec![LogComponentReceipt {
                tp2: "mod/setup.tp2".to_owned(),
                language: 0,
                component: 10,
            }],
        }];
        fs::write(
            &path,
            "// regenerated header\n~mod/setup.tp2~ #0 #10 // Description: v1\n",
        )
        .unwrap();
        let checked = check_logs(temp.path(), &logs).unwrap();
        assert_eq!(checked.state, "matches");
        assert_eq!((checked.mod_count, checked.component_count), (1, 1));
        assert_eq!(checked.components[0].title.as_deref(), Some("Description"));
        assert_eq!(checked.components[0].version.as_deref(), Some("v1"));
        assert_eq!(checked.components[0].status, "installed");
        fs::write(&path, "~mod/setup.tp2~ #0 #11 // Changed selection\n").unwrap();
        let changed = check_logs(temp.path(), &logs).unwrap();
        assert_eq!(changed.state, "changed");
        assert_eq!(changed.components[0].status, "missing");
        assert_eq!(changed.components[1].status, "extra");
        fs::write(&path, "~mod/setup.tp2 #0 #10 // damaged row\n").unwrap();
        assert!(check_logs(temp.path(), &logs).is_err());
        fs::remove_file(path).unwrap();
        assert!(check_logs(temp.path(), &logs).is_err());
    }

    #[test]
    fn inspect_accepts_an_ordinary_successful_receipt() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        write_logs(&root);
        fs::create_dir_all(root.join(".chriz")).unwrap();
        let mut receipt = failed_base(&root);
        receipt.outcome = ReceiptOutcome::Succeeded;
        receipt.final_state = Some(final_state(&root));
        fs::write(
            root.join(".chriz/install-receipt.json"),
            pretty_json(&receipt),
        )
        .unwrap();

        let checked = inspect(&root);
        assert_eq!(checked.state, "matches");
        assert_eq!(checked.component_count, 2);
    }

    #[test]
    fn inspect_accepts_a_verified_recovery_without_rewriting_the_failed_receipt() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        write_logs(&root);
        let attempts = root.join(".chriz/attempts/attempt-failed");
        fs::create_dir_all(&attempts).unwrap();
        let base = failed_base(&root);
        let base_bytes = pretty_json(&base);
        fs::write(attempts.join("receipt.json"), &base_bytes).unwrap();
        let evidence_path = root.join(".chriz/recovery-evidence.txt");
        fs::write(&evidence_path, b"verified recovery").unwrap();
        let recovery = RecoveryReceipt {
            schema_version: RECOVERY_RECEIPT_SCHEMA_VERSION,
            kind: RecoveryKind::SupervisedRecovery,
            recovery_id: "recovery-test".to_owned(),
            install_id: base.install_id.clone(),
            managed_root: root.clone(),
            base_attempt_id: base.attempt_id.clone(),
            base_receipt_sha256: sha256_bytes(&base_bytes),
            base_recipe_version: base.versions.recipe.clone(),
            base_recipe_payload_sha256: base.recipe_payload_sha256.clone(),
            base_plan_sha256: base.plan_sha256.clone(),
            replacements: Vec::new(),
            evidence: vec![EvidenceFile {
                path: ".chriz/recovery-evidence.txt".into(),
                sha256: sha256_bytes(b"verified recovery"),
            }],
            completed_at_millis: 3,
            final_state: final_state(&root),
        };
        publish(&root, &recovery).unwrap();

        let checked = inspect(&root);
        assert_eq!(checked.state, "matches");
        assert_eq!(checked.component_count, 2);
        let stored_base = fs::read(attempts.join("receipt.json")).unwrap();
        assert_eq!(stored_base, base_bytes);
    }
}
