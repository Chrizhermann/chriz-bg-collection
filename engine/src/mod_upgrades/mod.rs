//! History and recovery for the retired BuffBot 1.8.3 -> 1.8.4 transition.
//! Current updates use `hotpatch::update`; existing transaction records stay readable.
mod buffbot;
#[cfg(test)]
mod tests;
mod transaction;

use crate::{
    digest::sha256_bytes,
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
pub use transaction::restore_with_progress;
pub use transaction::{active_upgrade, launch_guard, restore, AppliedUpgrade};

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
    #[cfg(test)]
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
    // No folder-name heuristic (such as `steamapps` or Program Files): what keeps an update
    // inside a CEBG copy is that the root is a registered installation whose receipt matches.
    if !r.managed_root.is_absolute()
        || fs::canonicalize(&r.managed_root).map_err(err)? != r.managed_root
        || crate::preflight::is_creator_protected_destination(&r.managed_root)
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

pub fn inspect(ctx: &Context<'_>) -> Result<ModUpgradePreview, String> {
    crate::manual_checkpoint::game_update_guard(&ctx.record.managed_root)?;
    validate_context(ctx)?;
    let history = transaction::history(&ctx.record.managed_root)?;
    let mut preview = ModUpgradePreview {
        install_id: ctx.record.install_id.clone(),
        upgrade_id: buffbot::UPGRADE_ID.into(),
        title: "Historical BuffBot update".into(),
        state: "retired".into(),
        detail: "This legacy update is no longer offered. Check BuffBot under mod updates.".into(),
        base_recipe_version: ctx.record.recipe_version.clone(),
        installed_version: None,
        target_version: buffbot::NEW_VERSION.into(),
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
        save_compatibility:
            "Restoration uses the recorded affected-file backup. Saved games are not changed."
                .into(),
        history: history
            .iter()
            .map(|h| UpgradeHistoryItem {
                transaction_id: h.id.clone(),
                from_version: buffbot::OLD_VERSION.into(),
                to_version: buffbot::NEW_VERSION.into(),
                state: h.state.clone(),
            })
            .collect(),
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
    if history.last().is_some_and(|last| last.state == "restored") {
        preview.installed_version = Some(buffbot::OLD_VERSION.into());
        preview.detail = "The historical BuffBot update was restored from its backup.".into();
    }
    Ok(preview)
}
