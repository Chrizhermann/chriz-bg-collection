//! Explicit, append-only manual WeiDU additions at frozen component boundaries.
//!
//! Recipe bytes and managed run evidence never change. Each checkpoint has an immutable
//! baseline and a separate create-once acceptance receipt; ordinary resume cannot accept drift.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::digest::sha256_bytes;
use crate::manifest::{Artifact, GameRoot, ModFile};
use crate::receipt::LogComponentReceipt;
use crate::resolve::InstallPlan;
use crate::weidu::log::{has_new_uninstall_comment, parse_active_entries, parse_terminal_statuses};

/// Stable user-selected boundary in the original recipe plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualBreakpoint {
    pub run_id: String,
    pub component: u32,
}

/// Displayable component boundary; disabled slots explain the invocation constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckpointComponent {
    pub run_id: String,
    pub mod_id: String,
    pub mod_label: String,
    pub component: u32,
    pub component_label: String,
    pub phase: String,
    pub target: GameRoot,
    pub unavailable_reason: Option<String>,
}

/// Boundary frozen before the first game mutation, including its split execution run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FrozenCheckpoint {
    pub checkpoint_id: String,
    pub after_run_id: String,
    pub after: CheckpointComponent,
    pub next: Option<CheckpointComponent>,
}

pub fn options(plan: &InstallPlan) -> Vec<CheckpointComponent> {
    plan.runs.iter().flat_map(|run| run.components.iter().map(move |component| {
        let inside = Some(component) != run.components.last();
        // These installer families have coordinated multi-component passes. Their
        // complete invocation is the initial explicit indivisible group.
        let indivisible = matches!(run.mod_id.to_ascii_lowercase().as_str(), "stratagems" | "eet" | "eet_end");
        CheckpointComponent {
            run_id: run.run_id.clone(), mod_id: run.mod_id.clone(), component: *component,
            mod_label: run.mod_id.clone(), component_label: format!("Component {component}"),
            phase: serde_json::to_value(run.phase).expect("phase serialization").as_str().expect("phase string").to_owned(), target: run.target,
            unavailable_reason: (inside && indivisible).then(|| "This installer must finish its coordinated component group before pausing.".into()),
        }
    })).collect()
}

pub fn label_component(component: &mut CheckpointComponent, mods: &BTreeMap<String, ModFile>) {
    if let Some(installer) = mods.get(&component.mod_id) {
        component.mod_label = installer.name.clone();
        if let Some(named) = installer
            .components
            .iter()
            .find(|c| c.id == component.component)
        {
            component.component_label = named.name.clone();
        }
    }
}

/// Split only requested batches; preserve arguments, component-owned prompts and final checks.
pub fn split_plan(
    plan: &InstallPlan,
    anchors: &[ManualBreakpoint],
) -> Result<(InstallPlan, Vec<FrozenCheckpoint>), String> {
    let candidates = options(plan);
    let mut selected = BTreeSet::new();
    for anchor in anchors {
        let candidate = candidates.iter().find(|c| c.run_id == anchor.run_id && c.component == anchor.component)
            .ok_or_else(|| format!("Pause after {} component {} is no longer selected. Remove or correct that pause.", anchor.run_id, anchor.component))?;
        if let Some(reason) = &candidate.unavailable_reason {
            return Err(reason.clone());
        }
        if !selected.insert((anchor.run_id.clone(), anchor.component)) {
            return Err("The same pause was selected twice.".into());
        }
    }
    let mut runs = Vec::new();
    let mut checkpoints = Vec::new();
    for run in &plan.runs {
        let mut start = 0;
        for (index, component) in run.components.iter().enumerate() {
            let pause = selected.contains(&(run.run_id.clone(), *component));
            let last = index + 1 == run.components.len();
            if !pause && !last {
                continue;
            }
            let mut segment = run.clone();
            if !last {
                segment.run_id = format!("{}--manual-{}", run.run_id, component);
            }
            segment.components = run.components[start..=index].to_vec();
            segment
                .prompt_scripts
                .retain(|script| segment.components.contains(&script.component.component));
            if !last {
                segment.postconditions.clear();
            }
            if pause {
                let pos = candidates
                    .iter()
                    .position(|c| c.run_id == run.run_id && c.component == *component)
                    .unwrap();
                checkpoints.push(FrozenCheckpoint {
                    checkpoint_id: format!("manual-{}", checkpoints.len() + 1),
                    after_run_id: segment.run_id.clone(),
                    after: candidates[pos].clone(),
                    next: candidates.get(pos + 1).cloned(),
                });
            }
            runs.push(segment);
            start = index + 1;
        }
    }
    if runs
        .iter()
        .map(|r| &r.run_id)
        .collect::<BTreeSet<_>>()
        .len()
        != runs.len()
    {
        return Err("Manual pause execution run identities collide with the recipe.".into());
    }
    Ok((InstallPlan { runs }, checkpoints))
}

/// A reported addition, including the unverified author/version annotation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualAddition {
    pub tp2: String,
    pub language: u32,
    pub component: u32,
    pub description: Option<String>,
}

/// Durable snapshots captured only after the selected invocation fully exited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckpointBaseline {
    pub checkpoint: FrozenCheckpoint,
    pub bg1_log: String,
    pub bg2_log: String,
    pub diagnostics: BTreeMap<String, DiagnosticEvidence>,
    pub managed_files: BTreeMap<String, String>,
    pub game_files: BTreeMap<GameRoot, String>,
}

/// Separate immutable provenance; does not alter the frozen recipe or managed receipts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualAcceptance {
    pub checkpoint_id: String,
    pub target: GameRoot,
    pub after_run_id: String,
    pub review_token: String,
    pub additions: Vec<ManualAddition>,
    pub bg1_log: String,
    pub bg2_log: String,
    pub diagnostics: BTreeMap<String, DiagnosticEvidence>,
    pub managed_files: BTreeMap<String, String>,
    /// Hashes of the inspected local TP2 files, not verified upstream archive identities.
    pub manual_sources: BTreeMap<String, String>,
    pub game_files: BTreeMap<GameRoot, String>,
}

/// The exact checked bytes retained until explicit acceptance; never reread at publication.
pub struct ReviewedCheckpoint {
    pub status: ManualCheckpointStatus,
    acceptance: ManualAcceptance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticEvidence {
    pub sha256: String,
    pub length: u64,
    pub terminal_results: usize,
    pub failure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualCheckpointStatus {
    pub checkpoint_id: String,
    pub target: GameRoot,
    pub target_path: PathBuf,
    pub after: CheckpointComponent,
    pub next: Option<CheckpointComponent>,
    pub additions: Vec<ManualAddition>,
    pub diagnostics: Vec<String>,
    pub review_token: String,
    pub customized: bool,
}

fn directory(root: &Path, id: &str) -> Result<PathBuf, String> {
    if !id.starts_with("manual-") || !id[7..].bytes().all(|b| b.is_ascii_digit()) {
        return Err("Invalid manual checkpoint identity.".into());
    }
    Ok(root.join(".chriz/manual-checkpoints").join(id))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    direct(path)?;
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn direct(path: &Path) -> Result<(), String> {
    let mut current = Some(path);
    while let Some(p) = current {
        if p.exists() {
            let metadata = fs::symlink_metadata(p).map_err(|e| e.to_string())?;
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes() & 0x400 != 0 {
                    return Err("Manual checkpoint paths cannot use reparse points.".into());
                }
            }
            if metadata.file_type().is_symlink() {
                return Err("Manual checkpoint paths cannot use symbolic links.".into());
            }
        }
        current = p.parent();
    }
    Ok(())
}

fn write_once<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    direct(path)?;
    let parent = path.parent().ok_or("Checkpoint has no parent folder.")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    if path.exists() {
        if fs::read(path).map_err(|e| e.to_string())? == bytes {
            return Ok(());
        }
        return Err(
            "Existing checkpoint evidence differs; retain the folder and diagnostics.".into(),
        );
    }
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary
        .write_all(&bytes)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    temporary
        .persist_noclobber(path)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn read_logs(root: &Path) -> Result<(String, String), String> {
    let read = |name: &str| -> Result<String, String> {
        let path = root.join(name).join("WeiDU.log");
        direct(&path)?;
        match fs::read(&path) {
            Ok(bytes) => String::from_utf8(bytes)
                .map_err(|_| "WeiDU.log is not valid UTF-8; preserve it and request help.".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(e) => Err(e.to_string()),
        }
    };
    Ok((read("bg1")?, read("game")?))
}

/// Staged game folders present below `root`: `game` always, and `bg1` for an EET install.
///
/// A standalone install has no `bg1` folder. An EET folder that disappears after a pause began
/// is still detected, because its inventory and logs no longer match the saved baseline.
fn staged_game_folders(root: &Path) -> Result<Vec<(GameRoot, &'static str)>, String> {
    let mut folders = Vec::new();
    match fs::symlink_metadata(root.join("bg1")) {
        Ok(_) => folders.push((GameRoot::Bg1, "bg1")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    folders.push((GameRoot::Bg2, "game"));
    Ok(folders)
}

fn diagnostic_files(root: &Path) -> Result<BTreeMap<String, DiagnosticEvidence>, String> {
    let mut files = BTreeMap::new();
    for (_, target) in staged_game_folders(root)? {
        for entry in fs::read_dir(root.join(target)).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.to_ascii_lowercase().ends_with(".debug") {
                continue;
            }
            direct(&entry.path())?;
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            if !metadata.is_file() {
                return Err(format!("Cannot safely inspect diagnostic {name}."));
            }
            let mut reader =
                std::io::BufReader::new(fs::File::open(entry.path()).map_err(|e| e.to_string())?);
            let mut digest = Sha256::new();
            let mut length = 0_u64;
            let mut terminal_results = 0;
            let mut failure = false;
            let mut carry = Vec::new();
            loop {
                let bytes = reader.fill_buf().map_err(|e| e.to_string())?;
                if bytes.is_empty() {
                    break;
                }
                digest.update(bytes);
                length += bytes.len() as u64;
                carry.extend_from_slice(bytes);
                let text = String::from_utf8_lossy(&carry);
                failure |= text.contains("NOT INSTALLED DUE TO ERRORS")
                    || text.contains("ERROR: ")
                    || text.contains("FATAL ERROR");
                let used = bytes.len();
                if let Some(end) = carry.iter().rposition(|b| *b == b'\n') {
                    terminal_results +=
                        parse_terminal_statuses(&String::from_utf8_lossy(&carry[..=end])).len();
                    carry.drain(..=end);
                }
                // Keep bounded overlap for failure markers even for a pathological long line.
                if carry.len() > 8192 {
                    carry.drain(..carry.len() - 8192);
                }
                reader.consume(used);
            }
            terminal_results += parse_terminal_statuses(&String::from_utf8_lossy(&carry)).len();
            files.insert(
                format!("{target}/{name}"),
                DiagnosticEvidence {
                    sha256: hex::encode(digest.finalize()),
                    length,
                    terminal_results,
                    failure,
                },
            );
        }
    }
    Ok(files)
}

pub fn begin(
    root: &Path,
    checkpoint: &FrozenCheckpoint,
    plan: &InstallPlan,
    mods: &BTreeMap<String, ModFile>,
    artifacts: &BTreeMap<String, Artifact>,
) -> Result<(), String> {
    let path = directory(root, &checkpoint.checkpoint_id)?.join("baseline.json");
    if path.exists() {
        let baseline: CheckpointBaseline = read_json(&path)?;
        if &baseline.checkpoint != checkpoint {
            return Err("Frozen checkpoint differs from the saved pause.".into());
        }
        return Ok(());
    }
    let (bg1_log, bg2_log) = read_logs(root)?;
    write_once(
        &path,
        &CheckpointBaseline {
            checkpoint: checkpoint.clone(),
            bg1_log,
            bg2_log,
            diagnostics: diagnostic_files(root)?,
            managed_files: managed_inventory(root, plan, mods, artifacts)?,
            game_files: game_inventory(root)?,
        },
    )
}

fn game_inventory(root: &Path) -> Result<BTreeMap<GameRoot, String>, String> {
    let mut result = BTreeMap::new();
    for (target, folder) in staged_game_folders(root)? {
        let directory = root.join(folder);
        let mut entries = BTreeMap::new();
        for entry in walkdir::WalkDir::new(&directory).follow_links(false) {
            let entry = entry.map_err(|e| e.to_string())?;
            let relative = entry
                .path()
                .strip_prefix(&directory)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/")
                .to_ascii_lowercase();
            if relative.is_empty() {
                continue;
            }
            if !relative.contains('/')
                && (relative == "weidu.log"
                    || relative.ends_with(".debug")
                    || relative.starts_with("setup-") && relative.ends_with(".exe"))
            {
                continue;
            }
            direct(entry.path())?;
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            if metadata.is_dir() {
                entries.insert(relative, "directory".to_owned());
            } else if metadata.is_file() {
                let modified = metadata
                    .modified()
                    .map_err(|e| e.to_string())?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_nanos();
                entries.insert(relative, format!("{}:{modified}", metadata.len()));
            } else {
                return Err("Game folder contains an unsupported file type.".into());
            }
        }
        result.insert(
            target,
            sha256_bytes(&serde_json::to_vec(&entries).map_err(|e| e.to_string())?),
        );
    }
    Ok(result)
}

fn managed_inventory(
    root: &Path,
    plan: &InstallPlan,
    mods: &BTreeMap<String, ModFile>,
    artifacts: &BTreeMap<String, Artifact>,
) -> Result<BTreeMap<String, String>, String> {
    let mut namespaces = BTreeSet::new();
    for run in &plan.runs {
        let folder = if run.target == GameRoot::Bg1 {
            "bg1"
        } else {
            "game"
        };
        let artifact = artifacts
            .get(&run.artifact_id)
            .ok_or("Missing managed artifact.")?;
        for name in &artifact.archive.publish_roots {
            namespaces.insert(format!("{folder}/{}", name.replace('\\', "/")));
        }
        let installer = mods.get(&run.mod_id).ok_or("Missing managed installer.")?;
        let pieces = installer.tp2.split(['/', '\\']).collect::<Vec<_>>();
        if let Ok(path) = case_path(&root.join(folder), &pieces) {
            direct(&path)?;
            if let Ok(bytes) = fs::read(path) {
                let source = String::from_utf8_lossy(&bytes);
                if let Some(backup) = backup_path(&source) {
                    let backup = backup.replace('\\', "/");
                    if !backup.contains(':')
                        && !backup.starts_with('/')
                        && !backup.split('/').any(|p| matches!(p, "." | ".."))
                    {
                        namespaces.insert(format!("{folder}/{backup}"));
                    }
                }
            }
        }
    }
    let mut result = BTreeMap::new();
    for namespace in namespaces {
        let path = root.join(&namespace);
        if !path.exists() {
            result.insert(namespace, "absent".into());
            continue;
        }
        for entry in walkdir::WalkDir::new(path).follow_links(false) {
            let entry = entry.map_err(|e| e.to_string())?;
            direct(entry.path())?;
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/")
                .to_ascii_lowercase();
            if metadata.is_dir() {
                result.insert(relative, "directory".into());
            } else if metadata.is_file() {
                let modified = metadata
                    .modified()
                    .map_err(|e| e.to_string())?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_nanos();
                result.insert(relative, format!("{}:{modified}", metadata.len()));
            } else {
                return Err("Managed source/backup contains a non-regular file.".into());
            }
        }
    }
    Ok(result)
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|tail| tail.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|tail| tail.starts_with('/'))
}

fn reserved_backup_paths(
    root: &Path,
    target: GameRoot,
    plan: &InstallPlan,
    mods: &BTreeMap<String, ModFile>,
) -> Result<BTreeSet<String>, String> {
    let folder = root.join(if target == GameRoot::Bg1 {
        "bg1"
    } else {
        "game"
    });
    let mut paths = BTreeSet::new();
    for run in plan.runs.iter().filter(|r| r.target == target) {
        let installer = mods.get(&run.mod_id).ok_or("Missing managed installer.")?;
        let pieces = installer.tp2.split(['/', '\\']).collect::<Vec<_>>();
        if let Ok(path) = case_path(&folder, &pieces) {
            direct(&path)?;
            let source_bytes = fs::read(path).map_err(|e| e.to_string())?;
            let source = String::from_utf8_lossy(&source_bytes);
            if let Some(backup) = backup_path(&source) {
                paths.insert(backup.replace('\\', "/").to_ascii_lowercase());
            }
        }
    }
    Ok(paths)
}

pub fn has_baseline(root: &Path, checkpoint: &FrozenCheckpoint) -> Result<bool, String> {
    Ok(directory(root, &checkpoint.checkpoint_id)?
        .join("baseline.json")
        .exists())
}

pub fn paused_status(
    root: &Path,
    checkpoint: &FrozenCheckpoint,
) -> Result<ManualCheckpointStatus, String> {
    let baseline: CheckpointBaseline =
        read_json(&directory(root, &checkpoint.checkpoint_id)?.join("baseline.json"))?;
    if baseline.checkpoint != *checkpoint {
        return Err("Frozen checkpoint differs from the saved pause.".into());
    }
    Ok(ManualCheckpointStatus {
        checkpoint_id: checkpoint.checkpoint_id.clone(),
        target: checkpoint.after.target,
        target_path: root.join(if checkpoint.after.target == GameRoot::Bg1 {
            "bg1"
        } else {
            "game"
        }),
        after: checkpoint.after.clone(),
        next: checkpoint.next.clone(),
        additions: Vec::new(),
        diagnostics: Vec::new(),
        review_token: String::new(),
        customized: is_customized(root)?,
    })
}

pub fn acceptance(
    root: &Path,
    checkpoint: &FrozenCheckpoint,
) -> Result<Option<ManualAcceptance>, String> {
    let path = directory(root, &checkpoint.checkpoint_id)?.join("acceptance.json");
    if !path.exists() {
        return Ok(None);
    }
    let accepted: ManualAcceptance = read_json(&path)?;
    if accepted.checkpoint_id != checkpoint.checkpoint_id
        || accepted.after_run_id != checkpoint.after_run_id
        || accepted.target != checkpoint.after.target
    {
        return Err("Manual acceptance differs from its frozen checkpoint.".into());
    }
    Ok(Some(accepted))
}

/// Irreversible for the life of an installation, independent of later WeiDU.log edits.
pub fn is_customized(root: &Path) -> Result<bool, String> {
    let state = root.join(".chriz/manual-checkpoints");
    if !state.exists() {
        return Ok(false);
    }
    direct(&state)?;
    for entry in fs::read_dir(state).map_err(|e| e.to_string())? {
        let path = entry
            .map_err(|e| e.to_string())?
            .path()
            .join("acceptance.json");
        if path.exists() {
            let accepted: ManualAcceptance = read_json(&path)?;
            if !accepted.additions.is_empty() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn game_update_guard(root: &Path) -> Result<(), String> {
    if is_customized(root)? {
        return Err("Customized — manual mods added. Game hotpatches and whole-mod upgrades are disabled for this installation. CEBG app updates remain available.".into());
    }
    Ok(())
}

/// Manual WeiDU may run from Downloads, outside either staged game directory.
pub fn check_manual_processes() -> Result<(), String> {
    check_manual_processes_with(&crate::preflight::SystemPreflight)
}

/// Injected form of [`check_manual_processes`].
pub fn check_manual_processes_with(
    host: &dyn crate::preflight::PreflightHost,
) -> Result<(), String> {
    for executable in host.running_executable_paths().map_err(|e| e.to_string())? {
        let name = executable
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        if name == "weidu.exe" || name.starts_with("setup-") && name.ends_with(".exe") {
            return Err(format!(
                "Close the manual installer {} before checking or continuing.",
                executable.display()
            ));
        }
    }
    Ok(())
}

fn same_rows(before: &str, after: &str) -> Result<bool, String> {
    let a = parse_active_entries(before).map_err(|e| e.to_string())?;
    let b = parse_active_entries(after).map_err(|e| e.to_string())?;
    Ok(a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| {
            a.tp2_key == b.tp2_key
                && a.language == b.language
                && a.component == b.component
                && a.annotation == b.annotation
        })
        && !has_new_uninstall_comment(before, after))
}

/// Validate additions in installation order, including source and backup namespaces.
pub fn review(
    root: &Path,
    checkpoint: &FrozenCheckpoint,
    plan: &InstallPlan,
    mods: &BTreeMap<String, ModFile>,
    artifacts: &BTreeMap<String, Artifact>,
) -> Result<ReviewedCheckpoint, String> {
    let baseline: CheckpointBaseline =
        read_json(&directory(root, &checkpoint.checkpoint_id)?.join("baseline.json"))?;
    if baseline.checkpoint != *checkpoint {
        return Err("Saved checkpoint differs from the frozen plan.".into());
    }
    let managed_files = managed_inventory(root, plan, mods, artifacts)?;
    if baseline.managed_files != managed_files {
        return Err("Managed source or backup files changed during the manual pause. Reinstalling or editing managed components is unsupported, even when their WeiDU.log rows look unchanged. Restore your checkpoint backup before continuing.".into());
    }
    let (bg1, bg2) = read_logs(root)?;
    let game_files = game_inventory(root)?;
    let (before, after, other_before, other_after) = match checkpoint.after.target {
        GameRoot::Bg1 => (&baseline.bg1_log, &bg1, &baseline.bg2_log, &bg2),
        GameRoot::Bg2 => (&baseline.bg2_log, &bg2, &baseline.bg1_log, &bg1),
    };
    if !same_rows(other_before, other_after)? {
        return Err("The other game folder changed. Install manual mods only into the exact folder shown at this pause; BG1 changes after import are not imported again.".into());
    }
    let other_target = if checkpoint.after.target == GameRoot::Bg1 {
        GameRoot::Bg2
    } else {
        GameRoot::Bg1
    };
    if baseline.game_files.get(&other_target) != game_files.get(&other_target) {
        return Err("Files in the other game folder changed. Use only the exact target shown at this pause.".into());
    }
    let previous = parse_active_entries(before).map_err(|e| e.to_string())?;
    let current = parse_active_entries(after).map_err(|e| e.to_string())?;
    if has_new_uninstall_comment(before, after)
        || current.len() < previous.len()
        || !previous.iter().zip(&current).all(|(a, b)| {
            a.tp2_key == b.tp2_key
                && a.language == b.language
                && a.component == b.component
                && a.annotation == b.annotation
        })
    {
        return Err("Existing installed components were removed, reordered or changed. This feature accepts only new appended WeiDU components. Keep the logs intact and restore your checkpoint backup before retrying.".into());
    }
    let mut identities: BTreeSet<_> = previous
        .iter()
        .map(|e| (e.tp2_key.clone(), e.component))
        .collect();
    let mut namespaces = BTreeSet::new();
    for run in &plan.runs {
        if run.target != checkpoint.after.target {
            continue;
        }
        let artifact = artifacts
            .get(&run.artifact_id)
            .ok_or("Managed artifact missing.")?;
        for path in &artifact.archive.publish_roots {
            namespaces.insert(path.replace('\\', "/").to_ascii_lowercase());
        }
    }
    let reserved_backups = reserved_backup_paths(root, checkpoint.after.target, plan, mods)?;
    let mut manual_backups = BTreeMap::<String, String>::new();
    let mut manual_sources = BTreeMap::new();
    let mut additions = Vec::new();
    for entry in &current[previous.len()..] {
        let path = entry.tp2_key.clone();
        let components: Vec<_> = path.split('/').collect();
        if path.contains(':')
            || path.starts_with('/')
            || components
                .iter()
                .any(|p| p.is_empty() || matches!(*p, "." | ".."))
        {
            return Err("A manual TP2 uses an unsafe or non-local path.".into());
        }
        if !identities.insert((path.clone(), entry.component)) {
            return Err("A manual entry reinstalls an already installed component.".into());
        }
        if plan
            .runs
            .iter()
            .filter(|r| r.target == checkpoint.after.target)
            .any(|r| {
                mods.get(&r.mod_id)
                    .is_some_and(|m| m.tp2.replace('\\', "/").eq_ignore_ascii_case(&path))
            })
        {
            return Err(format!("Manual installer {} belongs to the managed recipe, including future components. Choose a different mod or a fresh recipe selection.", entry.tp2));
        }
        let stem = components
            .last()
            .unwrap()
            .strip_suffix(".tp2")
            .unwrap_or(components.last().unwrap())
            .trim_start_matches("setup-");
        if namespaces.iter().any(|reserved| {
            paths_overlap(reserved, &path)
                || paths_overlap(reserved, stem)
                || paths_overlap(reserved, &format!("setup-{stem}.tp2"))
        }) {
            return Err(format!(
                "Manual installer {} conflicts with a managed source or backup namespace.",
                entry.tp2
            ));
        }
        // Inspect the actual BACKUP declaration too; sharing a backup folder can
        // destroy uninstall history even when TP2 filenames differ.
        let target = root.join(if checkpoint.after.target == GameRoot::Bg1 {
            "bg1"
        } else {
            "game"
        });
        let tp2_path = case_path(&target, &components)?;
        direct(&tp2_path)?;
        let source_bytes = fs::read(&tp2_path)
            .map_err(|e| format!("Cannot inspect manual installer {}: {e}", entry.tp2))?;
        let source = String::from_utf8_lossy(&source_bytes);
        manual_sources.insert(path.clone(), sha256_bytes(&source_bytes));
        let backup = backup_path(&source)
            .ok_or_else(|| format!("Cannot verify the BACKUP folder declared by {}.", entry.tp2))?;
        let backup = backup.replace('\\', "/").to_ascii_lowercase();
        let backup_root = backup.split('/').next().unwrap_or("");
        if backup.starts_with('/')
            || backup.contains(':')
            || backup
                .split('/')
                .any(|p| matches!(p, ".." | ".") || p.is_empty())
            || namespaces
                .iter()
                .chain(&reserved_backups)
                .any(|reserved| paths_overlap(reserved, &backup))
            || matches!(backup_root, "override" | "lang" | "data" | ".chriz")
        {
            return Err(format!(
                "Manual installer {} uses an unsafe or managed BACKUP folder.",
                entry.tp2
            ));
        }
        if manual_backups
            .iter()
            .any(|(owner, used)| owner != &path && paths_overlap(used, &backup))
        {
            return Err("Manual installers share a conflicting BACKUP namespace.".into());
        }
        manual_backups.insert(path.clone(), backup);
        additions.push(ManualAddition {
            tp2: path,
            language: entry.language,
            component: entry.component,
            description: entry.annotation.clone(),
        });
    }
    let diagnostics = diagnostic_files(root)?;
    if additions.is_empty() && baseline.game_files != game_files {
        return Err("Game files changed without appended WeiDU components. File-copy mods are outside this feature; restore the checkpoint files before continuing without additions.".into());
    }
    let mut summaries = Vec::new();
    for (name, content) in &diagnostics {
        if baseline.diagnostics.get(name) == Some(content) {
            continue;
        }
        if content.failure {
            return Err(format!("Manual installer diagnostics report a failure in {name}. Keep the diagnostic and restore/fix the manual install before continuing."));
        }
        summaries.push(format!(
            "{name}: {} bytes, {} terminal result(s), SHA-256 {}",
            content.length, content.terminal_results, content.sha256
        ));
    }
    if !additions.is_empty() && summaries.is_empty() {
        summaries.push("No new installer debug log was available. Log additions and namespaces were checked; successful installation and compatibility are not proven.".into());
    }
    summaries.push("Only appended WeiDU components are supported. Managed source/backup checks compare file names, sizes and modification times; they do not prove arbitrary resource compatibility or detect deliberate metadata-preserving edits. File-copy mods are outside this feature. Reported versions are author text, not source hashes.".into());
    let review_token = sha256_bytes(
        &serde_json::to_vec(&(
            root,
            &baseline,
            &bg1,
            &bg2,
            &diagnostics,
            &additions,
            &manual_sources,
            &game_files,
        ))
        .map_err(|e| e.to_string())?,
    );
    let acceptance = ManualAcceptance {
        checkpoint_id: checkpoint.checkpoint_id.clone(),
        target: checkpoint.after.target,
        after_run_id: checkpoint.after_run_id.clone(),
        review_token: review_token.clone(),
        additions: additions.clone(),
        bg1_log: bg1,
        bg2_log: bg2,
        diagnostics,
        managed_files,
        manual_sources,
        game_files,
    };
    Ok(ReviewedCheckpoint {
        acceptance,
        status: ManualCheckpointStatus {
            checkpoint_id: checkpoint.checkpoint_id.clone(),
            target: checkpoint.after.target,
            target_path: root.join(if checkpoint.after.target == GameRoot::Bg1 {
                "bg1"
            } else {
                "game"
            }),
            after: checkpoint.after.clone(),
            next: checkpoint.next.clone(),
            additions,
            diagnostics: summaries,
            review_token,
            customized: is_customized(root)?,
        },
    })
}

fn case_path(root: &Path, pieces: &[&str]) -> Result<PathBuf, String> {
    let mut path = root.to_path_buf();
    for piece in pieces {
        direct(&path)?;
        let entries = fs::read_dir(&path)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut found = entries
            .iter()
            .filter(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(piece));
        let first = found
            .next()
            .ok_or_else(|| format!("Manual installer path {piece} is missing."))?;
        if found.next().is_some() {
            return Err("Ambiguous manual installer path.".into());
        }
        path = first.path();
    }
    Ok(path)
}

fn backup_path(source: &str) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut offset = 0;
    let mut found_backup = false;
    while offset < bytes.len() {
        if bytes[offset].is_ascii_whitespace() {
            offset += 1;
            continue;
        }
        if bytes[offset..].starts_with(b"//") {
            while offset < bytes.len() && bytes[offset] != b'\n' {
                offset += 1;
            }
            continue;
        }
        if bytes[offset..].starts_with(b"/*") {
            offset += 2;
            while offset + 1 < bytes.len() && &bytes[offset..offset + 2] != b"*/" {
                offset += 1;
            }
            offset += 2;
            continue;
        }
        if matches!(bytes[offset], b'~' | b'"' | b'%') {
            let delimiter = bytes[offset];
            offset += 1;
            let start = offset;
            while offset < bytes.len() && bytes[offset] != delimiter {
                offset += 1;
            }
            if offset == bytes.len() {
                return None;
            }
            if found_backup {
                let literal = &source[start..offset];
                return (!literal.contains('%')).then_some(literal);
            }
            offset += 1;
            continue;
        }
        if found_backup {
            return None;
        }
        let start = offset;
        while offset < bytes.len()
            && (bytes[offset].is_ascii_alphanumeric() || bytes[offset] == b'_')
        {
            offset += 1;
        }
        if offset == start {
            offset += 1;
            continue;
        }
        found_backup = bytes[start..offset].eq_ignore_ascii_case(b"BACKUP");
    }
    None
}

/// Caller retains the CEBG target lock and rechecks processes before this acceptance.
pub fn accept(
    root: &Path,
    checkpoint: &FrozenCheckpoint,
    reviewed: &ReviewedCheckpoint,
) -> Result<(), String> {
    write_once(
        &directory(root, &checkpoint.checkpoint_id)?.join("acceptance.json"),
        &reviewed.acceptance,
    )
}

/// Include accepted additions exactly at their checkpoint, preserving both log histories.
///
/// `frozen` is the frozen execution plan, which places every checkpoint; `plan` is the part
/// of it being installed, which lacks the runs of left-out mods. A checkpoint after a
/// left-out run keeps its place between the installed runs around it.
pub fn expected_components(
    root: &Path,
    frozen: &InstallPlan,
    plan: &InstallPlan,
    mods: &BTreeMap<String, ModFile>,
    checkpoints: &[FrozenCheckpoint],
    target: GameRoot,
) -> Result<Vec<LogComponentReceipt>, String> {
    let mut expected = Vec::new();
    for frozen_run in &frozen.runs {
        if frozen_run.target != target {
            continue;
        }
        if let Some(run) = plan.runs.iter().find(|run| run.run_id == frozen_run.run_id) {
            let installer = mods.get(&run.mod_id).ok_or("Missing frozen installer.")?;
            expected.extend(run.components.iter().map(|component| LogComponentReceipt {
                tp2: installer.tp2.replace('\\', "/").to_ascii_lowercase(),
                language: installer.language,
                component: *component,
            }));
        }
        for checkpoint in checkpoints
            .iter()
            .filter(|c| c.after_run_id == frozen_run.run_id)
        {
            if let Some(accepted) = acceptance(root, checkpoint)? {
                expected.extend(accepted.additions.into_iter().map(|addition| {
                    LogComponentReceipt {
                        tp2: addition.tp2,
                        language: addition.language,
                        component: addition.component,
                    }
                }));
            }
        }
    }
    Ok(expected)
}

/// Continuation must recheck the exact accepted snapshots while holding the target lock.
pub fn verify_accepted_current(
    root: &Path,
    checkpoint: &FrozenCheckpoint,
    plan: &InstallPlan,
    mods: &BTreeMap<String, ModFile>,
    artifacts: &BTreeMap<String, Artifact>,
) -> Result<(), String> {
    let accepted =
        acceptance(root, checkpoint)?.ok_or("Manual checkpoint has not been accepted.")?;
    let (bg1, bg2) = read_logs(root)?;
    if bg1 != accepted.bg1_log
        || bg2 != accepted.bg2_log
        || accepted.diagnostics != diagnostic_files(root)?
        || accepted.managed_files != managed_inventory(root, plan, mods, artifacts)?
    {
        return Err("Game history, source/backup files or diagnostics changed after checkpoint acceptance. Check the files and retain all WeiDU.log rows.".into());
    }
    if accepted.game_files != game_inventory(root)? {
        return Err("Game files changed after checkpoint acceptance. Restore the accepted checkpoint state before continuing.".into());
    }
    let folder = root.join(if checkpoint.after.target == GameRoot::Bg1 {
        "bg1"
    } else {
        "game"
    });
    for (path, expected) in &accepted.manual_sources {
        let path = case_path(&folder, &path.split('/').collect::<Vec<_>>())?;
        direct(&path)?;
        if sha256_bytes(&fs::read(path).map_err(|e| e.to_string())?) != *expected {
            return Err("A manual TP2 changed after acceptance.".into());
        }
    }
    Ok(())
}
