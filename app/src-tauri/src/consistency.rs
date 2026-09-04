//! Cheap launcher checks against the successful receipt; no full game-file scan.

use bg_engine::{
    manifest::GameRoot,
    receipt::{FinalLogReceipt, InstallReceipt},
    weidu::log::parse_active_entries,
};
use serde::Serialize;
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConsistencySummary {
    pub state: String,
    pub detail: String,
    pub component_count: usize,
    pub mod_count: usize,
}

pub fn inspect(managed_root: &Path) -> ConsistencySummary {
    let result = fs::read(managed_root.join(".chriz/install-receipt.json"))
        .map_err(|error| error.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<InstallReceipt>(&bytes).map_err(|error| error.to_string())
        })
        .and_then(|receipt| {
            receipt
                .final_state
                .ok_or_else(|| "No completed installation record.".to_owned())
        })
        .and_then(|state| check_logs(managed_root, &state.logs));
    result.unwrap_or_else(|_| ConsistencySummary {
        state: "unavailable".to_owned(),
        detail: "The installed mod list could not be checked.".to_owned(),
        component_count: 0,
        mod_count: 0,
    })
}

fn check_logs(root: &Path, logs: &[FinalLogReceipt]) -> Result<ConsistencySummary, String> {
    if logs.is_empty() {
        return Err("No final mod lists.".to_owned());
    }
    let mut count = 0;
    let mut mods = BTreeSet::new();
    let mut changed = false;
    for expected in logs {
        let folder = match expected.target {
            GameRoot::Bg1 => "bg1",
            GameRoot::Bg2 => "game",
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bg_engine::receipt::LogComponentReceipt;

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
        fs::write(&path, "~mod/setup.tp2~ #0 #11 // Changed selection\n").unwrap();
        assert_eq!(check_logs(temp.path(), &logs).unwrap().state, "changed");
        fs::remove_file(path).unwrap();
        assert!(check_logs(temp.path(), &logs).is_err());
    }
}
