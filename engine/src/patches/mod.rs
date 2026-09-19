//! A deliberately narrow existing-install patch path. Base receipts and saves are immutable.
pub mod catalog;
pub mod description;
mod files;
mod transaction;
#[cfg(all(test, windows))]
mod transaction_tests;

use crate::{
    digest::sha256_bytes,
    preflight::{PreflightHost, SystemPreflight},
    registry::ManagedInstallRecord,
};
use files::{err, Inventory};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
pub use transaction::{apply, launch_guard, restore, verified_log_suffix};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchPreview {
    pub install_id: String,
    pub patch_id: String,
    pub title: String,
    pub state: String,
    pub detail: String,
    pub base_recipe_version: String,
    pub applied_patch_ids: Vec<String>,
    pub review_token: Option<String>,
    pub can_undo: bool,
    pub can_restore: bool,
    pub full_backup_bytes: String,
    pub save_backup_bytes: String,
    pub available_bytes: String,
    pub backup_path: String,
}

pub struct Context<'a> {
    pub record: &'a ManagedInstallRecord,
    pub app_data: &'a Path,
    pub cache: &'a Path,
}
impl Context<'_> {
    pub fn game(&self) -> PathBuf {
        self.record.managed_root.join("game")
    }
    pub fn state(&self) -> PathBuf {
        self.record.managed_root.join(".chriz/patches/cebg-v1")
    }
    pub fn backups(&self) -> PathBuf {
        self.app_data
            .join("patch-backups")
            .join(&self.record.install_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Plan {
    pub description: description::DescriptionPlan,
    pub hashes: BTreeMap<String, String>,
    pub base_receipt_sha256: String,
}

fn validate_context(ctx: &Context<'_>) -> Result<(), String> {
    files::safe_relative(&ctx.record.install_id)?;
    if ctx.record.install_id.contains('/') {
        return Err("Invalid installation ID.".into());
    }
    let root = &ctx.record.managed_root;
    files::direct(root)?;
    files::direct(&ctx.game())?;
    if !root.is_absolute()
        || root.parent().is_none()
        || root.components().count() < 3
        || crate::preflight::is_creator_protected_destination(root)
        || root
            .to_string_lossy()
            .to_ascii_lowercase()
            .split(['\\', '/'])
            .any(|s| matches!(s, "steamapps" | "program files" | "program files (x86)"))
    {
        return Err("Patching is restricted to a separate managed game installation.".into());
    }
    let completed = crate::recovery_receipt::read_completed_state(root)?;
    if completed.bg2_engine_name != ctx.record.engine_name
        || completed.managed_save_root != ctx.record.managed_save_root
        || crate::stage::read_engine_name(&ctx.game()).map_err(err)? != ctx.record.engine_name
    {
        return Err("Installation profile no longer matches its completed record.".into());
    }
    // Base receipt stays immutable, including on installations completed by supervised recovery.
    let receipt = root.join(".chriz/install-receipt.json");
    files::regular(&receipt)?;
    if files::hash(&receipt)? != ctx.record.receipt_sha256 {
        return Err("Installation receipt changed.".into());
    }
    Ok(())
}

fn tables(game: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    description::TABLES
        .iter()
        .map(|name| {
            Ok((
                (*name).to_owned(),
                files::read(&game.join("override").join(name), 16 * 1024 * 1024)?,
            ))
        })
        .collect()
}

fn plan(ctx: &Context<'_>, catalog: &catalog::Catalog) -> Result<Plan, String> {
    validate_context(ctx)?;
    let game = ctx.game();
    if String::from_utf8(files::read(&game.join("weidu.conf"), 65536)?)
        .map_err(err)?
        .trim()
        .to_ascii_lowercase()
        != "lang_dir = en_us"
    {
        return Err("This first patch supports English installations only.".into());
    }
    let log = files::read(&game.join("WeiDU.log"), 16 * 1024 * 1024)?;
    let kitlist = files::read(&game.join("override/KITLIST.2DA"), 16 * 1024 * 1024)?;
    let tlk = files::read(&game.join("lang/en_US/dialog.tlk"), 256 * 1024 * 1024)?;
    let description = description::plan(
        &kitlist,
        &tables(&game)?,
        &log,
        &tlk,
        &catalog.supported_versions,
    )?;
    let mut paths = vec![
        "WeiDU.log".to_string(),
        "weidu.conf".into(),
        "engine.lua".into(),
        "chitin.key".into(),
        "Baldur.exe".into(),
        "override/KITLIST.2DA".into(),
    ];
    paths.extend(description::TABLES.iter().map(|p| format!("override/{p}")));
    for entry in walkdir::WalkDir::new(game.join("lang")).follow_links(false) {
        let entry = entry.map_err(err)?;
        if entry.file_type().is_symlink() {
            return Err("Linked language resources are unsupported.".into());
        }
        if entry
            .path()
            .extension()
            .is_some_and(|x| x.eq_ignore_ascii_case("tlk"))
        {
            paths.push(
                entry
                    .path()
                    .strip_prefix(&game)
                    .map_err(err)?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    if game.join("dialog.tlk").exists() {
        paths.push("dialog.tlk".into());
    }
    if game.join("dialogf.tlk").exists() {
        paths.push("dialogf.tlk".into());
    }
    let mut hashes = BTreeMap::new();
    for name in paths {
        let p = game.join(&name);
        files::regular(&p)?;
        hashes.insert(name, files::hash(&p)?);
    }
    Ok(Plan {
        description,
        hashes,
        base_receipt_sha256: ctx.record.receipt_sha256.clone(),
    })
}

fn fingerprint(plan: &Plan, catalog: &catalog::Catalog, id: &str) -> Result<String, String> {
    Ok(sha256_bytes(
        &serde_json::to_vec(&(plan, catalog, id)).map_err(err)?,
    ))
}

pub fn inspect(ctx: &Context<'_>, catalog: &catalog::Catalog) -> Result<PatchPreview, String> {
    validate_context(ctx)?;
    let mut preview = PatchPreview {
        install_id: ctx.record.install_id.clone(),
        patch_id: catalog.patch_id.clone(),
        title: catalog.title.clone(),
        state: "unsupported".into(),
        detail: String::new(),
        base_recipe_version: ctx.record.recipe_version.clone(),
        applied_patch_ids: Vec::new(),
        review_token: None,
        can_undo: false,
        can_restore: false,
        full_backup_bytes: "0".into(),
        save_backup_bytes: "0".into(),
        available_bytes: SystemPreflight
            .available_space(ctx.app_data)
            .map_err(err)?
            .to_string(),
        backup_path: ctx.backups().display().to_string(),
    };
    if let Some((dir, state)) = transaction::latest(ctx)? {
        match state.as_str() {
            "applied" => {
                preview.state = "applied".into();
                preview.detail="Description links repaired. Kit abilities and the base collection version are unchanged.".into();
                preview.applied_patch_ids.push(catalog.patch_id.clone());
                preview.can_undo = transaction::verify_post(ctx, &dir).is_ok();
                if !preview.can_undo {
                    preview.detail.push_str(
                        " Files have changed since patching; automatic undo is unavailable.",
                    );
                }
                return Ok(preview);
            }
            "prepared" | "restoring" => {
                preview.state = "needs-recovery".into();
                preview.detail="A patch did not finish. Restore its affected-file backup before playing. Your original installation receipt is preserved.".into();
                preview.can_restore = transaction::restorable(ctx, &dir).is_ok();
                return Ok(preview);
            }
            "restored" => {}
            _ => return Err("Unknown patch transaction state.".into()),
        }
    }
    let plan = match plan(ctx, catalog) {
        Ok(p) => p,
        Err(reason) => {
            if reason == "No installed supported Artisan components" {
                preview.state = "not-selected".into();
                preview.detail = "None of the Artisan components repaired by this fix are installed. Nothing will be added.".into();
                return Ok(preview);
            }
            preview.detail=format!("Not assessed for patching: {reason} Your game has not been changed. Use a new installation for other updates.");
            return Ok(preview);
        }
    };
    if plan.description.changes.is_empty() {
        preview.state = "already-fixed".into();
        preview.detail="These description links already match this game's installed kit text. No patch is needed.".into();
        return Ok(preview);
    }
    if ctx.game().join(catalog::ADAPTER).exists() {
        preview.detail =
            "An existing copy of this adapter is present. Automatic replacement is not supported."
                .into();
        return Ok(preview);
    }
    let game_inventory = files::inventory(&ctx.game(), false)?;
    preview.full_backup_bytes = files::total(&game_inventory).to_string();
    preview.save_backup_bytes = files::inventory(&ctx.record.managed_save_root, true)
        .map(|i| files::total(&i))
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "unavailable".into());
    preview.state = "available".into();
    preview.detail = catalog.summary.clone();
    preview.review_token = Some(fingerprint(&plan, catalog, &ctx.record.install_id)?);
    Ok(preview)
}

fn unchanged_hashes(game: &Path, hashes: &BTreeMap<String, String>) -> Result<(), String> {
    for (name, expected) in hashes {
        let path = game.join(name);
        files::regular(&path)?;
        if files::hash(&path)? != *expected {
            return Err(format!("File changed: {name}"));
        }
    }
    Ok(())
}
fn close_check(game: &Path, hashes: &BTreeMap<String, String>) -> Result<(), String> {
    close_check_inner(game, hashes, false)
}
fn close_check_inner(
    game: &Path,
    hashes: &BTreeMap<String, String>,
    restoring: bool,
) -> Result<(), String> {
    crate::preflight::recheck_target_before_mutation(game, "en_US").map_err(err)?;
    let mut paths = Vec::new();
    for name in hashes.keys() {
        let path = game.join(name);
        if restoring && writable(&name.to_ascii_lowercase()) && !path.exists() {
            files::direct(path.parent().ok_or("Missing restore parent")?)?;
        } else {
            files::regular(&path)?;
            paths.push(path);
        }
    }
    SystemPreflight
        .probe_exclusive_writable_files(&paths)
        .map_err(|e| {
            format!(
                "Close the game or other writer: {} ({})",
                e.path.display(),
                e.source
            )
        })?;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Target-specific external WeiDU processes must count too; unrelated modding is allowed.
        let script="$t=$env:CEBG_PATCH_TARGET; $p=@(Get-CimInstance Win32_Process -ErrorAction Stop | Where-Object { $_.Name -match '^(weidu|setup-.+)\\.exe$' -and $_.CommandLine -and $_.CommandLine.IndexOf($t,[StringComparison]::OrdinalIgnoreCase) -ge 0 }); if($p.Count){exit 2}";
        let target = game
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_owned();
        let status = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .env("CEBG_PATCH_TARGET", target)
            .creation_flags(0x08000000)
            .status()
            .map_err(err)?;
        if !status.success() {
            return Err("Close any installer targeting this game before patching.".into());
        }
    }
    Ok(())
}

fn changed(before: &Inventory, after: &Inventory) -> Vec<String> {
    before
        .keys()
        .chain(after.keys())
        .filter(|k| before.get(*k) != after.get(*k))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn writable(name: &str) -> bool {
    [
        "override/bgclatxt.2da",
        "override/clastext.2da",
        "override/sodcltxt.2da",
        "weidu.log",
        "weidu.conf",
    ]
    .contains(&name)
}
fn adapter_scope(name: &str) -> bool {
    name == "akcb_kit_descriptions" || name.starts_with("akcb_kit_descriptions/")
}
fn allowed(name: &str) -> bool {
    // Exact footprint observed with pinned WeiDU 249, not an entire directory wildcard.
    writable(name)
        || [
            "akcb_kit_descriptions",
            "akcb_kit_descriptions/lib",
            "akcb_kit_descriptions/backup",
            "akcb_kit_descriptions/backup/0",
            "akcb_kit_descriptions/setup-akcb_kit_descriptions.tp2",
            "akcb_kit_descriptions/lib/kit_strref.tpa",
            "akcb_kit_descriptions/backup/0/args.0",
            "akcb_kit_descriptions/backup/0/args.0.text",
            "akcb_kit_descriptions/backup/0/bgclatxt.2da",
            "akcb_kit_descriptions/backup/0/clastext.2da",
            "akcb_kit_descriptions/backup/0/sodcltxt.2da",
            "akcb_kit_descriptions/backup/0/mappings.0",
            "akcb_kit_descriptions/backup/0/move.0",
            "akcb_kit_descriptions/backup/0/other.0",
            "akcb_kit_descriptions/backup/0/readln.0",
            "akcb_kit_descriptions/backup/0/readln.0.text",
            "akcb_kit_descriptions/backup/0/tlkpath.0",
            "akcb_kit_descriptions/backup/0/uninstall.0",
        ]
        .contains(&name)
}
