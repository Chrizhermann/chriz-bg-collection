//! Hotpatch requirements and CEBG record checks. Player choices and additions do not
//! limit updates; the transaction backs up the game as it stands before applying them.
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::catalog::{Base, Catalog, GateRules, Hotpatch};
use super::transaction::History;
use super::Context;
use crate::cli::{PlayerMods, VerifiedInstalledBaseline};
use crate::game_files::{Difference, GameFiles};
use crate::recipe_view::{evaluate, evaluate_preset};
use crate::resolve::{InstallPlan, PlannedRun};
use crate::Manifest;

/// How one mod differs from the recommended setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModDifference {
    Added,
    LeftOut,
    Changed,
}

/// One mod that differs from the recommended setup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDeviation {
    pub mod_id: String,
    pub name: String,
    pub difference: ModDifference,
    /// Differences in this mod never count.
    pub free: bool,
}

/// The installed setup compared with the recommended one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SetupComparison {
    pub deviations: Vec<ModDeviation>,
    pub refusals: Vec<String>,
    pub notes: Vec<String>,
}

/// Compare the mods `installed` runs with the recommended preset of the same recipe.
/// A mod differs when it was added, left out, or installed with other components, settings or
/// answers; the comparison works on whole mods, so splitting one mod into several runs changes
/// nothing.
pub fn compare_setup(
    manifest: &Manifest,
    installed: &InstallPlan,
    rules: &GateRules,
) -> Result<SetupComparison, String> {
    let recommended = evaluate_preset(manifest, &rules.recommended_preset, "windows")
        .map_err(|error| format!("The recommended setup could not be evaluated: {error}"))?
        .plan;
    let by_mod = |plan: &InstallPlan| {
        let mut out: BTreeMap<String, Vec<PlannedRun>> = BTreeMap::new();
        for run in &plan.runs {
            out.entry(run.mod_id.clone()).or_default().push(run.clone());
        }
        out
    };
    let installed_mods = by_mod(installed);
    let recommended_mods = by_mod(&recommended);
    let mut comparison = SetupComparison::default();
    let ids = installed_mods
        .keys()
        .chain(recommended_mods.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for id in ids {
        let difference = match (installed_mods.get(&id), recommended_mods.get(&id)) {
            (Some(_), None) => ModDifference::Added,
            (None, Some(_)) => ModDifference::LeftOut,
            (Some(have), Some(want)) if have != want => ModDifference::Changed,
            _ => continue,
        };
        let name = manifest
            .mods
            .get(&id)
            .map_or_else(|| id.clone(), |installer| installer.name.clone());
        comparison.deviations.push(ModDeviation {
            free: rules.free_mods.contains(&id),
            mod_id: id,
            name,
            difference,
        });
    }
    Ok(comparison)
}

/// Everything the gate found for one installation and its next hotpatch.
pub(super) struct Assessment {
    pub refusals: Vec<String>,
    pub notes: Vec<String>,
    pub base: Option<Base>,
    pub baseline: Option<VerifiedInstalledBaseline>,
    /// The game folder as it is now, when it could be read.
    pub current: Option<GameFiles>,
    /// The next hotpatch for this installation's collection release, if any is left.
    pub next: Option<Hotpatch>,
}

/// The rule's verdict, cheapest checks first; every reason found is reported together.
pub(super) fn assess(ctx: &Context<'_>, catalog: &Catalog, history: &History) -> Assessment {
    // Mods the player added keep the baseline; removed or moved installer mods do not.
    let baseline = crate::cli::verify_installed_baseline(ctx.record, PlayerMods::Allowed)
        .map_err(|error| error.to_string());
    assess_with(ctx, catalog, history, baseline)
}

/// [`assess`] with the installation's verified recipe and receipt already read.
pub(super) fn assess_with(
    ctx: &Context<'_>,
    catalog: &Catalog,
    history: &History,
    baseline: Result<VerifiedInstalledBaseline, String>,
) -> Assessment {
    let mut refusals = Vec::new();
    let mut notes = Vec::new();
    let mut base = None;
    let mut claimed_base = None;
    let mut verified = None;
    let mut current = None;
    if let Err(error) = crate::manual_checkpoint::is_customized(&ctx.record.managed_root) {
        refusals.push(error);
    }
    for (folder, what) in [
        (".chriz/patches", "an earlier game fix"),
        (".chriz/mod-upgrades", "a whole-mod update"),
    ] {
        if ctx.record.managed_root.join(folder).exists() {
            refusals.push(format!(
                "This installation has {what} recorded. Hotpatches cannot follow it."
            ));
        }
    }
    match baseline {
        Ok(baseline) => {
            if baseline.manifest.collection.game != catalog.game {
                refusals.push("These hotpatches are for another game.".into());
            }
            match super::recipe_content_digest(&baseline.manifest) {
                Ok(digest) => {
                    let version = &baseline.receipt.versions.recipe;
                    if let Some(published) = catalog
                        .bases
                        .iter()
                        .find(|base| base.content_sha256.eq_ignore_ascii_case(&digest))
                    {
                        base = Some(published.clone());
                    } else if let Some(published) = catalog
                        .bases
                        .iter()
                        .find(|base| base.recipe_version == *version)
                    {
                        refusals.push("The collection's mod list was changed before this installation was made. Hotpatches are only for the collection as published.".into());
                        // Its hotpatches are named, so the refusal says what it withholds.
                        claimed_base = Some(published.clone());
                    }
                    // Any other collection version has no hotpatches to offer or refuse.
                }
                Err(error) => refusals.push(error),
            }
            match evaluate(
                &baseline.manifest,
                &baseline.receipt.normalized_selection.to_selection(),
            ) {
                Ok(installed) => {
                    match compare_setup(&baseline.manifest, &installed.plan, &catalog.gate) {
                        Ok(comparison) => {
                            refusals.extend(comparison.refusals);
                            notes.extend(comparison.notes);
                        }
                        Err(error) => refusals.push(error),
                    }
                }
                Err(error) => refusals.push(format!(
                    "This installation's choices could not be read again: {error}"
                )),
            }
            verified = Some(baseline);
        }
        Err(error) => refusals.push(error),
    }
    match history.expected_state(&ctx.record.managed_root) {
        Ok(expected) => match crate::game_files::capture(&ctx.game()) {
            Ok(now) => {
                // The player's own changes are named, never refused: the transaction records the
                // folder as it is now, so Undo gives back exactly this.
                let changed =
                    expected.differences(&now, |name| catalog.runtime_files.matches(name));
                if !changed.is_empty() {
                    notes.push(format!(
                        "Your own changes since installing do not stop hotpatches; the hotpatch is installed after them: {}.",
                        summarize(&changed)
                    ));
                }
                current = Some(now);
            }
            Err(error) => refusals.push(format!("The game folder could not be read: {error}")),
        },
        Err(error) => refusals.push(error),
    }
    let mut next = None;
    if let Some(release) = base.as_ref().or(claimed_base.as_ref()) {
        let applied = history.applied_ids();
        next = catalog
            .hotpatches
            .iter()
            .find(|hotpatch| {
                hotpatch.bases.contains(&release.id) && !applied.contains(&hotpatch.id)
            })
            .cloned();
        if let (Some(next), Some(baseline)) = (&next, &verified) {
            for required in &next.requires {
                if !applied.contains(required) {
                    let title = catalog
                        .hotpatch(required)
                        .map_or(required.as_str(), |hotpatch| hotpatch.title.as_str());
                    refusals.push(format!("Apply the hotpatch {title:?} first."));
                }
            }
            for requirement in &next.requires_components {
                // A real dependency must be installed now. Original selections can include
                // a download that was later omitted, and players may add the dependency.
                let present = if let Some(installer) =
                    baseline.manifest.mods.get(&requirement.mod_id)
                {
                    let rows = crate::patches::files::read(
                        &ctx.game().join("WeiDU.log"),
                        16 * 1024 * 1024,
                    )
                    .and_then(|bytes| {
                        crate::weidu::log::parse_active_entries(&String::from_utf8_lossy(&bytes))
                            .map_err(|error| error.to_string())
                    });
                    match rows {
                        Ok(rows) => {
                            let key = crate::weidu::log::normalize_tp2_key(&installer.tp2);
                            rows.iter().any(|row| {
                                row.tp2_key == key && requirement.any_of.contains(&row.component)
                            })
                        }
                        Err(error) => {
                            refusals.push(error);
                            continue;
                        }
                    }
                } else {
                    false
                };
                if !present {
                    let name = baseline
                        .manifest
                        .mods
                        .get(&requirement.mod_id)
                        .map_or(requirement.mod_id.as_str(), |installer| {
                            installer.name.as_str()
                        });
                    refusals.push(format!(
                        "This hotpatch needs {name} with component {}.",
                        requirement
                            .any_of
                            .iter()
                            .map(u32::to_string)
                            .collect::<Vec<_>>()
                            .join(" or ")
                    ));
                }
            }
        }
    }
    // Players may add mods themselves, including this hotpatch's own components (for example with
    // the mod's setup); applying on top of them would fail, so the check says so first.
    if let Some(next) = &next {
        match super::transaction::installed_components(&ctx.game(), next) {
            Ok(present) => {
                let already = next
                    .components
                    .iter()
                    .copied()
                    .filter(|component| present.contains(component))
                    .collect::<Vec<_>>();
                if already.len() == next.components.len() {
                    refusals.push("Everything this hotpatch installs is already in your game; it was installed outside the app.".into());
                } else if !already.is_empty() {
                    refusals.push(format!(
                        "Part of this hotpatch is already in your game (components {}), installed outside the app, so it cannot be applied on top.",
                        already
                            .iter()
                            .map(u32::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
            Err(error) => refusals.push(error),
        }
    }
    Assessment {
        refusals,
        notes,
        base,
        baseline: verified,
        current,
        next,
    }
}

/// A few changed names, enough to find them.
pub(super) fn summarize(changed: &BTreeMap<String, Difference>) -> String {
    let mut parts = changed
        .iter()
        .take(5)
        .map(|(name, difference)| {
            let what = match difference {
                Difference::Added => "added",
                Difference::Removed => "removed",
                Difference::Changed => "changed",
            };
            format!("{name} ({what})")
        })
        .collect::<Vec<_>>();
    if changed.len() > 5 {
        parts.push(format!("and {} more", changed.len() - 5));
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::Selection;

    fn recipe() -> Manifest {
        Manifest::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../recipes/curated-full-current"),
        )
        .unwrap()
    }

    fn rules() -> GateRules {
        GateRules {
            recommended_preset: "chris-recommended".into(),
            max_changed_mods: 2,
            free_mods: ["cdtweaks", "leui-iwdee", "infinity-ui", "eeex"]
                .into_iter()
                .map(String::from)
                .collect(),
            required_mods: ["stratagems"].into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn the_recommended_setup_has_no_differences() {
        let manifest = recipe();
        let selection = Selection {
            platform: "windows".into(),
            choices: manifest.presets["chris-recommended"].selections.clone(),
        };
        let plan = evaluate(&manifest, &selection).unwrap().plan;
        let comparison = compare_setup(&manifest, &plan, &rules()).unwrap();
        assert_eq!(comparison, SetupComparison::default());
    }
}
