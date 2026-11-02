//! Read-only, registry-bound installed-set comparison. Never starts an installer.
use super::*;
use crate::installed_updates::{
    compare_installed_recipe, InstalledRecipeBaseline, InstalledUpdatePlan,
};
use crate::recovery_receipt::RecoveryReceipt;
use crate::registry::ManagedInstallRecord;

#[cfg(test)]
mod tests;

fn problem(error: impl std::fmt::Display) -> CliError {
    CliError::new("installed_update_unavailable", error.to_string())
}

/// Compares a registered completed installation with an already trusted bundled recipe.
/// The caller holds the target lock. An unknown history or manually changed log is not accepted.
pub fn inspect_installed_update(
    record: &ManagedInstallRecord,
    target_recipe: &Path,
) -> Result<InstalledUpdatePlan, CliError> {
    let baseline = verify_installed_baseline(record)?;
    // Game updates compare and change the recorded mod list run by run. Without some of its
    // mods that list is not the recipe's, so no update route has been checked for it.
    if !baseline.receipt.left_out.is_empty() {
        return Err(problem("Some mods were left out of this installation because their downloads could not be used. Automatic game updates are unavailable for this copy; app updates still work."));
    }
    let target = load_recipe(target_recipe)?;
    let target_version = recipe_release_version(target_recipe)?
        .ok_or_else(|| problem("The target collection has no recorded release version."))?;
    if let (Ok(current), Ok(target)) = (
        semver::Version::parse(&baseline.receipt.versions.recipe),
        semver::Version::parse(&target_version),
    ) {
        if target < current {
            return Err(problem("This installation was created with a newer collection. Update CEBG before comparing its mods."));
        }
    }
    let mut comparison = compare_installed_recipe(
        &InstalledRecipeBaseline {
            install_id: &record.install_id,
            recipe_version: &baseline.receipt.versions.recipe,
            manifest: &baseline.manifest,
            selection: &baseline.receipt.normalized_selection,
            plan: &baseline.receipt.plan,
            recovered_artifacts: &baseline.replacements,
        },
        &target,
        &target_version,
    );
    if baseline.recovered {
        comparison
            .detail
            .push_str(" Recorded repair versions were included in this comparison.");
    }
    if baseline.description_fix {
        comparison.detail.push_str(" An existing description-only fix remains recorded separately; it does not count as a whole-mod upgrade.");
    }
    if baseline.upgraded {
        comparison.detail.push_str(
            " The completed BuffBot upgrade is included. The base collection version is unchanged.",
        );
    }
    Ok(comparison)
}

pub(crate) struct VerifiedInstalledBaseline {
    pub manifest: Manifest,
    pub receipt: InstallReceipt,
    pub replacements: BTreeMap<String, crate::session::FrozenIdentity>,
    pub recovered: bool,
    pub description_fix: bool,
    pub upgraded: bool,
}

pub(crate) fn verify_installed_baseline(
    record: &ManagedInstallRecord,
) -> Result<VerifiedInstalledBaseline, CliError> {
    let root = canonical_direct_directory(&record.managed_root, "managed installation")?;
    if metadata_is_reparse(&fs::symlink_metadata(&record.managed_root).map_err(problem)?) {
        return Err(problem("Linked installation folders are not supported."));
    }
    if root != record.managed_root
        || record.schema_version != crate::registry::REGISTRY_SCHEMA_VERSION
    {
        return Err(problem(
            "The installation folder or record format has changed.",
        ));
    }
    let stable_path = root.join(".chriz/install-receipt.json");
    let stable_bytes = bounded_file(&root, &stable_path)?;
    if !sha256_bytes(&stable_bytes).eq_ignore_ascii_case(&record.receipt_sha256) {
        return Err(problem(
            "The original installation record has changed. No game files were modified.",
        ));
    }
    let store = SessionStore::open(&root).map_err(problem)?;
    let replay = store.replay().map_err(problem)?;
    let created = replay.created();
    if created.install_id != record.install_id
        || created.managed_root != root
        || created.recipe_payload_sha256 != record.recipe_sha256
    {
        return Err(problem(
            "The saved recipe belongs to a different installation.",
        ));
    }
    let frozen: FrozenCliRecipe =
        serde_json::from_slice(&created.recipe_payload).map_err(problem)?;
    validate_frozen_cli_recipe(created, &frozen)?;
    let value: serde_json::Value = serde_json::from_slice(&stable_bytes).map_err(problem)?;
    let mut replacements = BTreeMap::new();
    let mut recovered = false;
    let base: InstallReceipt = if value.get("kind").is_some() {
        let receipt: RecoveryReceipt = serde_json::from_value(value).map_err(problem)?;
        validate_cli_identifier(&receipt.base_attempt_id, "recovery attempt").map_err(problem)?;
        let path = root
            .join(".chriz/attempts")
            .join(&receipt.base_attempt_id)
            .join("receipt.json");
        let bytes = bounded_file(&root, &path)?;
        if !sha256_bytes(&bytes).eq_ignore_ascii_case(&receipt.base_receipt_sha256) {
            return Err(problem(
                "The original record for this repaired installation has changed.",
            ));
        }
        let base: InstallReceipt = serde_json::from_slice(&bytes).map_err(problem)?;
        receipt.validate(&base).map_err(problem)?;
        crate::recovery_receipt::verify_evidence(&root, &receipt.evidence).map_err(problem)?;
        for replacement in receipt.replacements {
            replacements.insert(replacement.run_id, replacement.replacement);
        }
        recovered = true;
        base
    } else {
        let receipt: InstallReceipt = serde_json::from_value(value).map_err(problem)?;
        if receipt.outcome != ReceiptOutcome::Succeeded {
            return Err(problem(
                "This installation is incomplete. Recover it before checking mod updates.",
            ));
        }
        receipt
    };
    validate_report_receipt(&base, &base.attempt_id, created, &frozen, &replay).map_err(problem)?;
    // Mods left out because their downloads could not be used are verified like every other
    // record; each caller decides whether its work can follow such an installation.
    // A mod installed from a changed download is installed with that file, not the pin.
    for artifact in &base.artifacts {
        if artifact.served_difference.is_none() {
            continue;
        }
        for run in base
            .plan
            .runs
            .iter()
            .filter(|run| run.artifact_id == artifact.id)
        {
            replacements.entry(run.run_id.clone()).or_insert_with(|| {
                crate::session::FrozenIdentity {
                    id: artifact.id.clone(),
                    version: artifact.version.clone(),
                    sha256: artifact.sha256.clone(),
                    length: artifact.length,
                }
            });
        }
    }
    let completed = crate::recovery_receipt::read_completed_state(&root).map_err(problem)?;
    if completed.bg2_engine_name != record.engine_name
        || completed.managed_save_root != record.managed_save_root
        || completed.launch_path != record.launch_path
    {
        return Err(problem(
            "The completed installation no longer matches its launcher record.",
        ));
    }
    let history = root.join(".chriz/patches");
    if history.exists() {
        validate_direct_directory(&history, "patch history")?;
        for entry in fs::read_dir(&history).map_err(problem)? {
            let entry = entry.map_err(problem)?;
            validate_direct_directory(&entry.path(), "patch history entry")?;
            if entry.file_name() != "cebg-v1" {
                return Err(problem("This installation has an update history this version of CEBG cannot interpret."));
            }
        }
    }
    let suffix = crate::patches::verified_log_suffix(&root).map_err(problem)?;
    let upgrade = crate::mod_upgrades::active_upgrade(&root).map_err(problem)?;
    if let Some(u) = &upgrade {
        let source = base.plan.runs.iter().find(|r| r.run_id == u.run_id);
        if source.is_none_or(|r| {
            r.mod_id != "buffbot" || r.target != GameRoot::Bg2 || r.components != [1, 0]
        }) || completed
            .logs
            .iter()
            .find(|l| l.target == GameRoot::Bg2)
            .is_none_or(|l| l.sha256 != u.before_log_sha256)
        {
            return Err(problem("The recorded BuffBot upgrade does not follow this installation's completed mod list."));
        }
    }
    let mut seen = BTreeSet::new();
    for final_log in &completed.logs {
        if !seen.insert(final_log.target) {
            return Err(problem(
                "The completed installation has duplicate mod-list records.",
            ));
        }
        let game = match final_log.target {
            GameRoot::Bg1 => &created.staged_bg1,
            GameRoot::Bg2 => &created.staged_bg2,
        };
        let bytes = bounded_file(&root, &game.join("WeiDU.log"))?;
        let active = parse_active_entries(&String::from_utf8_lossy(&bytes)).map_err(problem)?;
        let mut expected = final_log.components.clone();
        if final_log.target == GameRoot::Bg2 {
            expected.extend(suffix.clone());
        }
        if active.len() != expected.len()
            || active.iter().zip(&expected).any(|(actual, expected)| {
                normalize_tp2_key(&actual.tp2) != normalize_tp2_key(&expected.tp2)
                    || actual.language != expected.language
                    || actual.component != expected.component
            })
            || ((final_log.target == GameRoot::Bg1 || suffix.is_empty())
                && !sha256_bytes(&bytes).eq_ignore_ascii_case(
                    upgrade
                        .as_ref()
                        .filter(|_| final_log.target == GameRoot::Bg2)
                        .map_or(&final_log.sha256, |u| &u.log_sha256),
                ))
        {
            return Err(problem("The installed mod list changed outside CEBG's recorded work. Automatic game updates are unavailable for this copy; app updates still work."));
        }
    }
    if seen != BTreeSet::from([GameRoot::Bg1, GameRoot::Bg2]) {
        return Err(problem(
            "The completed installation is missing a verified game mod list.",
        ));
    }
    let old = Manifest {
        root: root.clone(),
        collection: frozen.collection,
        artifacts: frozen.artifacts,
        mods: frozen.mods,
        presets: frozen.presets,
    };
    if let Some(u) = &upgrade {
        replacements.insert(
            u.run_id.clone(),
            crate::session::FrozenIdentity {
                id: "buffbot-1.8.4-alpha".into(),
                version: u.version.clone(),
                sha256: u.artifact_sha256.clone(),
                length: 4_241_150,
            },
        );
    }
    Ok(VerifiedInstalledBaseline {
        manifest: old,
        receipt: base,
        replacements,
        recovered,
        description_fix: !suffix.is_empty(),
        upgraded: upgrade.is_some(),
    })
}

fn bounded_file(root: &Path, path: &Path) -> Result<Vec<u8>, CliError> {
    let relative = path.strip_prefix(root).map_err(problem)?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, std::path::Component::Normal(_)) {
            return Err(problem("Invalid installation record path."));
        }
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(problem)?;
        if metadata.file_type().is_symlink() || metadata_is_reparse(&metadata) {
            return Err(problem("Linked installation records are not supported."));
        }
    }
    let metadata = fs::metadata(path).map_err(problem)?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 * 1024 {
        return Err(problem(
            "Installation record is not a supported regular file.",
        ));
    }
    fs::read(path).map_err(problem)
}
