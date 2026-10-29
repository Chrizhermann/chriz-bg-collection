//! Explicit whole-mod transitions; never a generic WeiDU reinstall or a new install.
mod buffbot;
#[cfg(test)]
mod tests;
mod transaction;

use crate::{
    digest::sha256_bytes,
    manifest::GameRoot,
    patches::files::{self, err},
    preflight::{PreflightHost, SystemPreflight},
    registry::ManagedInstallRecord,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
pub use transaction::{active_upgrade, apply, launch_guard, restore, AppliedUpgrade};
pub use transaction::{apply_with_progress, restore_with_progress};

pub struct Context<'a> {
    pub record: &'a ManagedInstallRecord,
    pub app_data: &'a Path,
    /// Where the installation is locked, independent of `app_data`.
    pub locks: &'a crate::lock::TargetLockNamespace,
    pub cache: &'a Path,
    pub recipe: &'a Path,
}
impl Context<'_> {
    fn game(&self) -> PathBuf {
        self.record.managed_root.join("game")
    }
    fn state(&self) -> PathBuf {
        self.record.managed_root.join(".chriz/mod-upgrades/v1")
    }
    fn backups(&self) -> PathBuf {
        self.app_data
            .join("mod-upgrade-backups")
            .join(&self.record.install_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpgradePreview {
    pub install_id: String,
    pub upgrade_id: String,
    pub title: String,
    pub state: String,
    pub detail: String,
    pub base_recipe_version: String,
    pub installed_version: Option<String>,
    pub target_version: String,
    pub review_token: Option<String>,
    pub can_undo: bool,
    pub can_restore: bool,
    pub full_backup_bytes: String,
    pub save_backup_bytes: String,
    pub available_bytes: String,
    pub backup_path: String,
    pub save_compatibility: String,
    pub history: Vec<UpgradeHistoryItem>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeHistoryItem {
    pub transaction_id: String,
    pub from_version: String,
    pub to_version: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    install_id: String,
    managed_root: PathBuf,
    base_receipt_sha256: String,
    recipe_sha256: String,
    selection_sha256: String,
    previous: Option<String>,
    hashes: BTreeMap<String, String>,
}

fn validate_context(ctx: &Context<'_>) -> Result<(), String> {
    let r = ctx.record;
    files::safe_relative(&r.install_id)?;
    if r.install_id.contains('/') || r.schema_version != crate::registry::REGISTRY_SCHEMA_VERSION {
        return Err("Unsupported installation record.".into());
    }
    files::direct(&r.managed_root)?;
    files::direct(&ctx.game())?;
    if !r.managed_root.is_absolute()
        || fs::canonicalize(&r.managed_root).map_err(err)? != r.managed_root
        || crate::preflight::is_creator_protected_destination(&r.managed_root)
        || r.managed_root
            .to_string_lossy()
            .to_ascii_lowercase()
            .split(['\\', '/'])
            .any(|p| matches!(p, "steamapps" | "program files" | "program files (x86)"))
    {
        return Err("Updates require a separate, registered game folder.".into());
    }
    let receipt = r.managed_root.join(".chriz/install-receipt.json");
    files::regular(&receipt)?;
    if files::hash(&receipt)? != r.receipt_sha256 {
        return Err("Original installation receipt changed.".into());
    }
    let complete = crate::recovery_receipt::read_completed_state(&r.managed_root)?;
    if complete.bg1_engine_name.is_none() {
        // Recorded mod upgrades target the Enhanced Edition Trilogy only.
        return Err("Mod upgrades are available only for Baldur's Gate installations.".into());
    }
    if complete.bg2_engine_name != r.engine_name
        || complete.managed_save_root != r.managed_save_root
        || complete.launch_path != r.launch_path
        || crate::stage::read_engine_name(ctx.game()).map_err(err)? != r.engine_name
    {
        return Err("The installation no longer matches its recorded game/profile.".into());
    }
    Ok(())
}

/// Also used before downloads: an independent supported subset must not inherit
/// newly selected features merely because unrelated parts of the recipe changed.
fn selected_version(ctx: &Context<'_>) -> Result<Option<String>, String> {
    let b = crate::cli::verify_installed_baseline(ctx.record).map_err(err)?;
    // The route below reasons about the frozen plan's BuffBot run; without the mods that were
    // left out, the installed mod list is not that plan.
    if !b.receipt.left_out.is_empty() {
        return Err("Some mods were left out of this installation because their downloads could not be used. Whole-mod updates are unavailable for this copy.".into());
    }
    let Some(run) = b
        .receipt
        .plan
        .runs
        .iter()
        .find(|r| r.run_id == buffbot::RUN_ID)
    else {
        return Ok(None);
    };
    let old = &b.manifest.mods[&run.mod_id];
    let target = crate::Manifest::load(ctx.recipe).map_err(err)?;
    let new = target
        .mods
        .get(&run.mod_id)
        .ok_or("BuffBot is not offered by this collection.")?;
    let target_run = target
        .collection
        .runs
        .iter()
        .find(|r| r.run_id == run.run_id)
        .ok_or("The new collection has no matching BuffBot step.")?;
    let payload = &target.artifacts[&new.artifact_id];
    let tool = target
        .artifacts
        .get(&new.weidu_artifact_id)
        .and_then(|a| a.tool.as_ref());
    if run.target != GameRoot::Bg2
        || target_run.mod_id != run.mod_id
        || target_run.phase != run.phase
        || target_run.postconditions != run.postconditions
        || run.components != [1, 0]
        || target_run.components != [1, 0]
        || old.language != 0
        || new.language != 0
        || !old.tp2.eq_ignore_ascii_case("buffbot/setup-buffbot.tp2")
        || new.tp2 != old.tp2
        || new.invocation_mode != old.invocation_mode
        || !run.args.is_empty()
        || !target_run.args.is_empty()
        || !run.prompt_scripts.is_empty()
        || old
            .components
            .iter()
            .chain(&new.components)
            .any(|c| c.stdin.is_some() || !c.prompts.is_empty())
        || payload.source.sha256 != buffbot::NEW_HASH
        || payload.version != buffbot::NEW_VERSION
        || tool.is_none_or(|t| t.sha256 != crate::patches::catalog::TOOL_HASH)
        || b.manifest.collection.game_build != target.collection.game_build
    {
        return Err(
            "This BuffBot version, language or setup is outside the supported update route.".into(),
        );
    }
    let artifact = &b.manifest.artifacts[&run.artifact_id];
    let replacement = b.replacements.get(&run.run_id);
    let (hash, version) = replacement.map_or(
        (artifact.source.sha256.as_str(), artifact.version.as_str()),
        |r| (r.sha256.as_str(), r.version.as_str()),
    );
    if ![
        (buffbot::OLD_HASH, buffbot::OLD_VERSION),
        (buffbot::NEW_HASH, buffbot::NEW_VERSION),
    ]
    .contains(&(hash, version))
    {
        return Err("This first upgrade supports recorded BuffBot 1.8.3-alpha only.".into());
    }
    Ok(Some(version.into()))
}

fn plan(ctx: &Context<'_>, old_source: &Path, new_source: &Path) -> Result<Plan, String> {
    crate::manual_checkpoint::game_update_guard(&ctx.record.managed_root)?;
    validate_context(ctx)?;
    if selected_version(ctx)?.as_deref() != Some(buffbot::OLD_VERSION) {
        return Err("The supported old BuffBot version is not installed.".into());
    }
    if !crate::patches::verified_log_suffix(&ctx.record.managed_root)?.is_empty() {
        return Err("A later game fix follows BuffBot. Undo that fix before updating BuffBot, or leave this update unapplied.".into());
    }
    buffbot::validate_sources(old_source, new_source)?;
    buffbot::validate_before(&ctx.game(), old_source)?;
    if !String::from_utf8(files::read(&ctx.game().join("weidu.conf"), 65536)?)
        .map_err(err)?
        .trim()
        .eq_ignore_ascii_case("lang_dir = en_US")
    {
        return Err("This BuffBot upgrade requires the recorded English game language.".into());
    }
    let log = files::read(&ctx.game().join("WeiDU.log"), 16 * 1024 * 1024)?;
    verify_tail(&log, buffbot::OLD_VERSION)?;
    let inventory = files::inventory(&ctx.game(), false)?;
    let mut hashes = BTreeMap::new();
    for (name, stamp) in inventory {
        if !stamp.directory
            && (buffbot::writable(&name)
                || buffbot::runtime_mutable(&name)
                || [
                    "engine.lua",
                    "chitin.key",
                    "baldur.exe",
                    "weidu.conf",
                    "infinityloader.ini",
                    "lua51.dll",
                    "luaprovider.dll",
                ]
                .contains(&name.as_str()))
        {
            files::regular(&ctx.game().join(&name))?;
            hashes.insert(name.clone(), files::hash(&ctx.game().join(name))?);
        }
    }
    let store = crate::session::SessionStore::open(&ctx.record.managed_root).map_err(err)?;
    Ok(Plan {
        install_id: ctx.record.install_id.clone(),
        managed_root: ctx.record.managed_root.clone(),
        base_receipt_sha256: ctx.record.receipt_sha256.clone(),
        recipe_sha256: ctx.record.recipe_sha256.clone(),
        selection_sha256: store
            .replay()
            .map_err(err)?
            .created()
            .selection_sha256
            .clone(),
        previous: transaction::head(&ctx.record.managed_root)?,
        hashes,
    })
}

fn verify_tail(bytes: &[u8], version: &str) -> Result<(), String> {
    let text = std::str::from_utf8(bytes).map_err(err)?;
    let rows = crate::weidu::log::parse_active_entries(text).map_err(err)?;
    if rows.len() < 2 {
        return Err("BuffBot is not the final complete mod in this installation.".into());
    }
    let start = rows.len() - 2;
    if rows.iter().enumerate().any(|(i, r)| {
        if i < start {
            r.tp2_key == "buffbot/setup-buffbot.tp2"
        } else {
            r.tp2_key != "buffbot/setup-buffbot.tp2"
                || r.language != 0
                || r.component != [1, 0][i - start]
                || r.annotation.as_deref().is_none_or(|a| {
                    a.rsplit_once(": ")
                        .is_none_or(|(_, v)| v.trim().trim_start_matches('v') != version)
                })
        }
    }) {
        return Err("This upgrade requires BuffBot 1 then 0 at the end of the installed mod list, with no later mods.".into());
    }
    Ok(())
}

fn token(plan: &Plan) -> Result<String, String> {
    Ok(sha256_bytes(
        &serde_json::to_vec(&(
            buffbot::UPGRADE_ID,
            buffbot::OLD_HASH,
            buffbot::NEW_HASH,
            plan,
        ))
        .map_err(err)?,
    ))
}

pub fn inspect(ctx: &Context<'_>) -> Result<ModUpgradePreview, String> {
    crate::manual_checkpoint::game_update_guard(&ctx.record.managed_root)?;
    validate_context(ctx)?;
    let history = transaction::history(&ctx.record.managed_root)?;
    let mut preview=ModUpgradePreview {
        install_id:ctx.record.install_id.clone(),upgrade_id:buffbot::UPGRADE_ID.into(),
        title:"Update BuffBot in this installation".into(),state:"unsupported".into(),detail:String::new(),
        base_recipe_version:ctx.record.recipe_version.clone(),installed_version:None,target_version:buffbot::NEW_VERSION.into(),
        review_token:None,can_undo:false,can_restore:false,full_backup_bytes:"0".into(),save_backup_bytes:"0".into(),
        available_bytes:SystemPreflight.available_space(ctx.app_data).map_err(err)?.to_string(),backup_path:ctx.backups().display().to_string(),
        save_compatibility:"Works with your current run. Keeps existing saves, BuffBot settings and existing text references. A full backup is recommended.".into(),
        history:history.iter().map(|h| UpgradeHistoryItem { transaction_id:h.id.clone(),from_version:buffbot::OLD_VERSION.into(),to_version:buffbot::NEW_VERSION.into(),state:h.state.clone() }).collect(),
    };
    if let Some(last) = history.last() {
        if last.state == "prepared" || last.state == "restoring" {
            preview.state = "needs-recovery".into();
            preview.detail="The BuffBot update did not finish. Restore its affected-file backup before playing; do not reinstall the collection.".into();
            preview.can_restore = transaction::check_restore(ctx, &last.dir, false).is_ok();
            if !preview.can_restore {
                preview.detail.push_str(" Automatic restoration could not be verified. Keep this folder and export diagnostics.");
            }
            return Ok(preview);
        }
        if last.state == "applied" {
            active_upgrade(&ctx.record.managed_root)?;
            preview.state = "applied".into();
            preview.installed_version = Some(buffbot::NEW_VERSION.into());
            preview.detail="BuffBot was updated in this folder. Other mods and the base collection version are unchanged.".into();
            preview.can_undo = transaction::check_restore(ctx, &last.dir, true).is_ok();
            if !preview.can_undo {
                preview.detail.push_str(
                    " A later fix or relevant file change prevents Undo; undo later fixes first.",
                );
            }
            return Ok(preview);
        }
    }
    match selected_version(ctx) {
        Ok(None) => {
            preview.state = "not-selected".into();
            preview.detail = "BuffBot was not selected. This update will not add it.".into();
            return Ok(preview);
        }
        Ok(Some(v)) if v == buffbot::NEW_VERSION => {
            preview.state = "already-current".into();
            preview.installed_version = Some(v);
            preview.detail =
                "This installation already includes the supported BuffBot version.".into();
            return Ok(preview);
        }
        Ok(v) => preview.installed_version = v,
        Err(e) => {
            preview.detail = e;
            return Ok(preview);
        }
    }
    // Downloads/extraction are confined to cache, never the game folder.
    let payloads = transaction::payloads(ctx)?;
    match plan(ctx, &payloads.old, &payloads.new) {
        Err(e) => preview.detail = e,
        Ok(p) => {
            preview.state = "available".into();
            preview.review_token = Some(token(&p)?);
            preview.detail="Upgrade BuffBot 1.8.3 to 1.8.4 here. Unidentified items stop revealing their names in BuffBot. No other mods are reinstalled.".into();
            preview.full_backup_bytes =
                files::total(&files::inventory(&ctx.game(), false)?).to_string();
            preview.save_backup_bytes = files::inventory(&ctx.record.managed_save_root, true)
                .map(|i| files::total(&i).to_string())
                .unwrap_or_else(|_| "unavailable".into());
        }
    }
    Ok(preview)
}
