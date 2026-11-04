//! Christopher's strict rule: a hotpatch is offered only to an installation that is the
//! recommended setup or differs from it in small ways. Everything else is refused, because a
//! patch tested on the recommended setup is not known to work on it.
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::catalog::{Base, Catalog, GateRules, Hotpatch};
use super::transaction::History;
use super::Context;
use crate::cli::VerifiedInstalledBaseline;
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
    for id in &rules.required_mods {
        if !installed_mods.contains_key(id) {
            let name = manifest
                .mods
                .get(id)
                .map_or_else(|| id.clone(), |installer| installer.name.clone());
            comparison
                .refusals
                .push(format!("{name} was left out. Hotpatches need it."));
        }
    }
    // A required mod that was left out is refused above; counting it again would say it twice.
    let counted = comparison
        .deviations
        .iter()
        .filter(|deviation| {
            !deviation.free
                && !(deviation.difference == ModDifference::LeftOut
                    && rules.required_mods.contains(&deviation.mod_id))
        })
        .collect::<Vec<_>>();
    let listed = counted
        .iter()
        .map(|deviation| describe(deviation))
        .collect::<Vec<_>>()
        .join(", ");
    if counted.len() > rules.max_changed_mods {
        comparison.refusals.push(format!(
            "{} mods differ from the recommended setup; hotpatches allow at most {}: {listed}.",
            counted.len(),
            rules.max_changed_mods
        ));
    } else if !counted.is_empty() {
        comparison.notes.push(format!(
            "Allowed difference from the recommended setup ({} of at most {}): {listed}.",
            counted.len(),
            rules.max_changed_mods
        ));
    }
    let free = comparison
        .deviations
        .iter()
        .filter(|deviation| deviation.free)
        .map(|deviation| deviation.name.as_str())
        .collect::<Vec<_>>();
    if !free.is_empty() {
        comparison.notes.push(format!(
            "Your own choices in {} do not affect hotpatches.",
            free.join(", ")
        ));
    }
    Ok(comparison)
}

fn describe(deviation: &ModDeviation) -> String {
    match deviation.difference {
        ModDifference::Added => format!("{} added", deviation.name),
        ModDifference::LeftOut => format!("{} left out", deviation.name),
        ModDifference::Changed => format!("{} changed", deviation.name),
    }
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
    let baseline =
        crate::cli::verify_installed_baseline(ctx.record).map_err(|error| error.to_string());
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
    match crate::manual_checkpoint::is_customized(&ctx.record.managed_root) {
        Ok(false) => {}
        Ok(true) => refusals.push("You added your own mods during installation pauses. Hotpatches are only for installations without them.".into()),
        Err(error) => refusals.push(error),
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
            if baseline.recovered {
                refusals.push("This installation was repaired with replacement mod versions. Hotpatches are only for installations with the collection's own versions.".into());
            }
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
                Ok(evaluated) => {
                    // A mod the installer left out because its download could not be used is
                    // missing like one the player left out, and counts the same.
                    let installed = baseline
                        .receipt
                        .installed_plan
                        .as_ref()
                        .unwrap_or(&evaluated.plan);
                    match compare_setup(&baseline.manifest, installed, &catalog.gate) {
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
                let changed =
                    expected.differences(&now, |name| catalog.runtime_files.matches(name));
                if !changed.is_empty() {
                    refusals.push(format!(
                        "Files in the game folder changed since the installer and its hotpatches left it: {}.",
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
                let present = baseline.receipt.effective_plan().runs.iter().any(|run| {
                    run.mod_id == requirement.mod_id
                        && run
                            .components
                            .iter()
                            .any(|component| requirement.any_of.contains(component))
                });
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
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../recipes/iwd-current"),
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

    fn plan(manifest: &Manifest, changes: &[(&str, bool)]) -> InstallPlan {
        let mut selection = Selection {
            platform: "windows".into(),
            choices: manifest.presets["chris-recommended"].selections.clone(),
        };
        for (feature, on) in changes {
            selection.set_feature(feature, *on);
        }
        evaluate(manifest, &selection).unwrap().plan
    }

    fn quest_mods(manifest: &Manifest) -> Vec<String> {
        manifest
            .collection
            .features
            .iter()
            .filter(|feature| {
                feature.category == "quests"
                    && feature.id.starts_with("mod:")
                    && feature.requires.is_empty()
                    && feature.requires_any.is_empty()
                    && feature.conflicts.is_empty()
            })
            .map(|feature| feature.id.clone())
            .collect()
    }

    #[test]
    fn the_recommended_setup_has_no_differences() {
        let manifest = recipe();
        let comparison = compare_setup(&manifest, &plan(&manifest, &[]), &rules()).unwrap();
        assert_eq!(comparison, SetupComparison::default());
    }

    #[test]
    fn tweaks_anthology_choices_are_free_and_never_counted() {
        let manifest = recipe();
        let tweaks = manifest
            .collection
            .features
            .iter()
            .filter(|feature| {
                feature.parent.as_deref() == Some("mod:cdtweaks")
                    && feature.decision == crate::manifest::Decision::Optional
                    && feature.choice_group.is_none()
                    && feature.requires.is_empty()
                    && feature.requires_any.is_empty()
                    && feature.conflicts.is_empty()
            })
            .take(6)
            .map(|feature| (feature.id.as_str(), true))
            .collect::<Vec<_>>();
        assert_eq!(tweaks.len(), 6, "the recipe offers optional tweaks");
        let comparison = compare_setup(&manifest, &plan(&manifest, &tweaks), &rules()).unwrap();
        assert!(comparison.refusals.is_empty(), "{comparison:?}");
        assert_eq!(comparison.deviations.len(), 1);
        assert!(comparison.deviations[0].free);
        assert_eq!(comparison.notes.len(), 1);
        assert!(
            comparison.notes[0].contains("Tweaks Anthology"),
            "{:?}",
            comparison.notes
        );
    }

    #[test]
    fn two_extra_quest_mods_are_allowed_and_a_third_is_refused() {
        let manifest = recipe();
        let quests = quest_mods(&manifest);
        assert!(quests.len() >= 3, "the recipe offers optional quest mods");
        let two = quests[..2]
            .iter()
            .map(|id| (id.as_str(), true))
            .collect::<Vec<_>>();
        let comparison = compare_setup(&manifest, &plan(&manifest, &two), &rules()).unwrap();
        assert!(comparison.refusals.is_empty(), "{comparison:?}");
        assert!(comparison.notes[0].starts_with("Allowed difference"));
        let three = quests[..3]
            .iter()
            .map(|id| (id.as_str(), true))
            .collect::<Vec<_>>();
        let comparison = compare_setup(&manifest, &plan(&manifest, &three), &rules()).unwrap();
        assert_eq!(comparison.refusals.len(), 1, "{comparison:?}");
        assert!(comparison.refusals[0].contains("3 mods differ"));
        assert!(comparison.refusals[0].contains("added"));
    }

    #[test]
    fn leaving_out_scs_is_refused_even_alone() {
        let manifest = recipe();
        let comparison = compare_setup(
            &manifest,
            &plan(&manifest, &[("mod:stratagems", false)]),
            &rules(),
        )
        .unwrap();
        assert_eq!(comparison.refusals.len(), 1, "{comparison:?}");
        assert!(
            comparison.refusals[0].contains("was left out"),
            "{:?}",
            comparison.refusals
        );
    }

    #[test]
    fn a_left_out_companion_counts_like_an_added_quest_mod() {
        let manifest = recipe();
        let quests = quest_mods(&manifest);
        let companion = manifest
            .collection
            .features
            .iter()
            .find(|feature| feature.category == "companions" && feature.id.starts_with("mod:"))
            .unwrap()
            .id
            .clone();
        let changes = [
            (companion.as_str(), false),
            (quests[0].as_str(), true),
            (quests[1].as_str(), true),
        ];
        let comparison = compare_setup(&manifest, &plan(&manifest, &changes), &rules()).unwrap();
        assert_eq!(comparison.refusals.len(), 1, "{comparison:?}");
        assert!(comparison.refusals[0].contains("left out"));
    }
}
