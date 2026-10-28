//! Hotpatches for completed installations: new WeiDU components of one pinned mod release,
//! installed after everything else, so that running games get them too.
//!
//! A hotpatch is offered only to an installation the strict rule in [`gate`] accepts: the
//! collection as published, its game files exactly as the installer and earlier hotpatches left
//! them, and choices close to the recommended setup. Base receipts and saves stay immutable.
//! Every file a hotpatch changes keeps an earlier copy in its transaction record, so Undo puts
//! the game folder back byte for byte; transactions follow [`transaction`].
pub mod catalog;
pub mod gate;
#[cfg(all(test, windows))]
mod tests;
mod transaction;

use crate::{
    digest::sha256_bytes,
    game_files::GameFiles,
    patches::files,
    preflight::{PreflightHost, SystemPreflight},
    registry::ManagedInstallRecord,
    Manifest,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use transaction::{
    apply, apply_with_progress, launch_guard, restore, restore_with_progress, verified_log_suffix,
};

/// Where transaction records live, below the managed root.
pub const STATE: &str = ".chriz/hotpatches/v1";

pub struct Context<'a> {
    pub record: &'a ManagedInstallRecord,
    pub app_data: &'a Path,
    /// Where the installation is locked, independent of `app_data`.
    pub locks: &'a crate::lock::TargetLockNamespace,
    pub cache: &'a Path,
}

impl Context<'_> {
    pub fn game(&self) -> PathBuf {
        self.record.managed_root.join("game")
    }
    fn state(&self) -> PathBuf {
        self.record.managed_root.join(STATE)
    }
    fn backups(&self) -> PathBuf {
        self.app_data
            .join("hotpatch-backups")
            .join(&self.record.install_id)
    }
}

/// One changelog line as the player reads it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInfo {
    pub title: String,
    pub summary: String,
    pub save_applicability: String,
    pub condition_note: Option<String>,
}

/// The next hotpatch for an installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotpatchInfo {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub changes: Vec<ChangeInfo>,
}

/// A hotpatch the installation has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedHotpatch {
    pub id: String,
    pub title: String,
}

/// What Updates shows for one installation. Checking never writes the game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotpatchPreview {
    pub install_id: String,
    /// `available`, `refused`, `up-to-date` or `needs-recovery`.
    pub state: String,
    pub detail: String,
    pub base_recipe_version: String,
    pub hotpatch: Option<HotpatchInfo>,
    pub applied: Vec<AppliedHotpatch>,
    /// Why the next hotpatch is refused.
    pub refusals: Vec<String>,
    /// Differences from the recommended setup that the rule allows.
    pub notes: Vec<String>,
    pub review_token: Option<String>,
    pub can_undo: bool,
    pub undo_title: Option<String>,
    pub can_restore: bool,
    pub full_backup_bytes: String,
    pub save_backup_bytes: String,
    pub available_bytes: String,
    pub backup_path: String,
}

/// Digest of what a recipe installs: its collection, artifacts, mods and presets, without the
/// folder it was read from. A frozen installation recipe has the digest of the release it was
/// made from, so this identifies the collection as published.
pub fn recipe_content_digest(manifest: &Manifest) -> Result<String, String> {
    Ok(sha256_bytes(
        &serde_json::to_vec(&(
            &manifest.collection,
            &manifest.artifacts,
            &manifest.mods,
            &manifest.presets,
        ))
        .map_err(files::err)?,
    ))
}

fn validate_context(ctx: &Context<'_>) -> Result<(), String> {
    let record = ctx.record;
    files::safe_relative(&record.install_id)?;
    if record.install_id.contains('/') {
        return Err("Invalid installation ID.".into());
    }
    let root = &record.managed_root;
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
            .any(|part| matches!(part, "steamapps" | "program files" | "program files (x86)"))
    {
        return Err("Hotpatches are restricted to a separate managed game installation.".into());
    }
    let completed = crate::recovery_receipt::read_completed_state(root)?;
    if completed.bg2_engine_name != record.engine_name
        || completed.managed_save_root != record.managed_save_root
        || completed.launch_path != record.launch_path
        || crate::stage::read_engine_name(ctx.game()).map_err(files::err)? != record.engine_name
    {
        return Err("The installation no longer matches its completed record.".into());
    }
    let receipt = root.join(".chriz/install-receipt.json");
    files::regular(&receipt)?;
    if files::hash(&receipt)? != record.receipt_sha256 {
        return Err("The original installation receipt changed.".into());
    }
    Ok(())
}

/// The language folder WeiDU installs into, as `weidu.conf` names it.
fn language_folder(game: &Path) -> Result<String, String> {
    let text = String::from_utf8(files::read(&game.join("weidu.conf"), 65536)?)
        .map_err(|_| "weidu.conf is not text.".to_string())?;
    let value = text
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            key.trim()
                .eq_ignore_ascii_case("lang_dir")
                .then(|| value.trim().to_owned())
        })
        .ok_or("weidu.conf names no language folder.")?;
    if value.is_empty()
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("weidu.conf names an unsupported language folder.".into());
    }
    Ok(value)
}

/// The token a player confirms: it changes with the catalog, the hotpatch, the installation's
/// records and every byte of its game folder.
fn review_token(
    ctx: &Context<'_>,
    catalog: &catalog::Catalog,
    hotpatch: &catalog::Hotpatch,
    current: &GameFiles,
    head: Option<&str>,
) -> Result<String, String> {
    Ok(sha256_bytes(
        &serde_json::to_vec(&(
            &catalog.digest,
            &hotpatch.id,
            &ctx.record.install_id,
            &ctx.record.receipt_sha256,
            sha256_bytes(&serde_json::to_vec(current).map_err(files::err)?),
            head,
        ))
        .map_err(files::err)?,
    ))
}

/// Check one installation against the catalog. Reads the whole game folder; never writes.
pub fn inspect(ctx: &Context<'_>, catalog: &catalog::Catalog) -> Result<HotpatchPreview, String> {
    crate::patches::launch_guard(&ctx.record.managed_root)?;
    crate::mod_upgrades::launch_guard(&ctx.record.managed_root)?;
    validate_context(ctx)?;
    let history = transaction::History::load(&ctx.record.managed_root, &ctx.record.install_id)?;
    let mut preview = HotpatchPreview {
        install_id: ctx.record.install_id.clone(),
        state: "refused".into(),
        detail: String::new(),
        base_recipe_version: ctx.record.recipe_version.clone(),
        hotpatch: None,
        applied: history
            .applied_ids()
            .into_iter()
            .map(|id| AppliedHotpatch {
                title: catalog
                    .hotpatch(&id)
                    .map_or_else(|| id.clone(), |hotpatch| hotpatch.title.clone()),
                id,
            })
            .collect(),
        refusals: Vec::new(),
        notes: Vec::new(),
        review_token: None,
        can_undo: false,
        undo_title: None,
        can_restore: false,
        full_backup_bytes: "0".into(),
        save_backup_bytes: "0".into(),
        available_bytes: SystemPreflight
            .available_space(ctx.app_data)
            .map_err(files::err)?
            .to_string(),
        backup_path: ctx.backups().display().to_string(),
    };
    if let Some(unfinished) = history.unfinished() {
        preview.state = "needs-recovery".into();
        preview.detail = "A hotpatch did not finish. Restore the game files before playing. Your saves and the original installation record are untouched.".into();
        preview.can_restore = transaction::restorable(ctx, catalog, unfinished).is_ok();
        return Ok(preview);
    }
    let assessment = gate::assess(ctx, catalog, &history);
    if let (Some(target), Some(current)) = (history.undo_target(), &assessment.current) {
        preview.undo_title = Some(catalog.hotpatch(&target.hotpatch_id).map_or_else(
            || target.hotpatch_id.clone(),
            |hotpatch| hotpatch.title.clone(),
        ));
        preview.can_undo = transaction::undoable(ctx, catalog, target, current).is_ok();
    }
    preview.notes = assessment.notes;
    let Some(next) = assessment.next else {
        // A refusal still explains why later hotpatches would not be offered.
        preview.refusals = assessment.refusals;
        if assessment.base.is_none() && !preview.refusals.is_empty() {
            preview.detail =
                "Hotpatches are not offered for this installation. Your game has not been changed."
                    .into();
            return Ok(preview);
        }
        preview.state = "up-to-date".into();
        preview.detail = if assessment.base.is_some() && !preview.applied.is_empty() {
            "Every hotpatch for your collection version is installed.".into()
        } else {
            "There is no hotpatch for this installation yet.".into()
        };
        return Ok(preview);
    };
    preview.hotpatch = Some(HotpatchInfo {
        id: next.id.clone(),
        title: next.title.clone(),
        summary: next.summary.clone(),
        changes: next
            .changes
            .iter()
            .map(|change| ChangeInfo {
                title: change.title.clone(),
                summary: change.summary.clone(),
                save_applicability: serde_json::to_value(change.save_applicability)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .unwrap_or_else(|| "unknown".into()),
                condition_note: change.condition_note.clone(),
            })
            .collect(),
    });
    if !assessment.refusals.is_empty() {
        preview.refusals = assessment.refusals;
        preview.detail = "This hotpatch is not offered for this installation, because it differs too much from the recommended setup. Your game has not been changed.".into();
        return Ok(preview);
    }
    let current = assessment
        .current
        .ok_or("The game folder could not be read.")?;
    preview.full_backup_bytes = current
        .files
        .values()
        .map(|identity| identity.length)
        .sum::<u64>()
        .to_string();
    preview.save_backup_bytes = files::inventory(&ctx.record.managed_save_root, true)
        .map(|inventory| files::total(&inventory).to_string())
        .unwrap_or_else(|_| "unavailable".into());
    preview.review_token = Some(review_token(
        ctx,
        catalog,
        &next,
        &current,
        history.head().as_deref(),
    )?);
    preview.state = "available".into();
    preview.detail = next.summary.clone();
    Ok(preview)
}
