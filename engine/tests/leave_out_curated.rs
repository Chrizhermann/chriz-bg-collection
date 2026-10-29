//! Leaving out a mod whose download cannot be used, on the real curated recipe, both with and
//! without Spell Revisions.
//!
//! Each case checks the result three ways: it only removes (every remaining run is a run of
//! the original plan, in the same order, with a subset of its components); it removes exactly
//! what unticking the mod would remove, without anything unticking would add; and runs that
//! keep all their components stay identical, including prompt answers and recipe checks.

use std::collections::BTreeSet;
use std::path::PathBuf;

use bg_engine::manifest::Feature;
use bg_engine::recipe_view::{evaluate, plan_without_artifacts, SelectionEvaluation};
use bg_engine::resolve::{InstallPlan, Selection};
use bg_engine::Manifest;

const SPELL_REVISIONS: &str = "mod:spell-rev";
/// Sword Coast Stratagems is the second mod left out here. Its download is used with and
/// without Spell Revisions, and other mods need it in both: BG Rebalance's SCS adjustments
/// (one of which also needs Spell Revisions, so the dependents differ between the two
/// configurations), and a mandatory Modpack part that requires SCS together with BG1 NPC.
/// It also holds an alternative off: Item Randomiser's "Keep Watcher's Keep statues" is a
/// default that conflicts with the SCS core, so leaving SCS out must not bring it back. SCS
/// itself contains several choice groups, which have to go with it.
const STRATAGEMS: &str = "mod:stratagems";

fn curated() -> Manifest {
    Manifest::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../recipes/curated-full-current"),
    )
    .expect("curated recipe should load")
}

/// The recommended preset, with Spell Revisions switched on or off. The plain recipe defaults
/// differ from it only by one Hidden Gameplay Options run that is unrelated to both mods left
/// out here, so they are not evaluated separately.
fn recommended(manifest: &Manifest, spell_revisions: bool) -> Selection {
    let mut selection = Selection {
        platform: "windows".to_owned(),
        choices: manifest.presets["chris-recommended"].selections.clone(),
    };
    selection.set_feature(SPELL_REVISIONS, spell_revisions);
    selection
}

fn feature<'a>(manifest: &'a Manifest, id: &str) -> &'a Feature {
    manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == id)
        .unwrap_or_else(|| panic!("missing feature {id}"))
}

/// The one download every run of `mod_feature` installs from.
fn artifact_of(manifest: &Manifest, mod_feature: &str) -> String {
    let artifacts = manifest
        .collection
        .features
        .iter()
        .filter(|feature| {
            feature.id == mod_feature || feature.parent.as_deref() == Some(mod_feature)
        })
        .flat_map(|feature| &feature.components)
        .map(|component| {
            let run = manifest
                .collection
                .runs
                .iter()
                .find(|run| run.run_id == component.run_id)
                .unwrap();
            manifest.mods[&run.mod_id].artifact_id.clone()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(artifacts.len(), 1, "{mod_feature}: {artifacts:?}");
    artifacts.into_iter().next().unwrap()
}

fn runs_from<'a>(plan: &'a InstallPlan, artifact: &str) -> Vec<&'a str> {
    plan.runs
        .iter()
        .filter(|run| run.artifact_id == artifact)
        .map(|run| run.run_id.as_str())
        .collect()
}

/// Every remaining run is a run of `original`, in the same order, with a subset of its
/// components in the same order.
fn assert_only_removes(original: &InstallPlan, remaining: &InstallPlan) {
    let mut next = 0;
    for run in &remaining.runs {
        let offset = original.runs[next..]
            .iter()
            .position(|candidate| candidate.run_id == run.run_id)
            .unwrap_or_else(|| {
                panic!(
                    "run {} is not in the original plan, or out of order",
                    run.run_id
                )
            });
        let source = &original.runs[next + offset];
        next += offset + 1;
        let mut components = source.components.iter();
        assert!(
            run.components
                .iter()
                .all(|component| components.any(|candidate| candidate == component)),
            "run {} gained or reordered components: {:?} from {:?}",
            run.run_id,
            run.components,
            source.components
        );
        assert!(!run.components.is_empty(), "run {} is empty", run.run_id);
        assert_eq!(run.mod_id, source.mod_id);
        assert_eq!(run.artifact_id, source.artifact_id);
        assert_eq!(run.weidu_artifact_id, source.weidu_artifact_id);
        assert_eq!(run.args, source.args);
    }
}

/// `remaining` is `original` reduced to what `unticked` (the same selection with the mod
/// unticked) still installs; whatever unticking would add instead is not there. A run that
/// keeps all its components is unchanged, down to its prompt answers and recipe checks.
fn assert_matches_unticking(
    original: &InstallPlan,
    remaining: &InstallPlan,
    unticked: &InstallPlan,
) {
    let expected = original
        .runs
        .iter()
        .filter_map(|run| {
            let kept = unticked.components_for(&run.run_id).unwrap_or(&[]);
            let components = run
                .components
                .iter()
                .copied()
                .filter(|component| kept.contains(component))
                .collect::<Vec<_>>();
            (!components.is_empty()).then_some((run.run_id.as_str(), components))
        })
        .collect::<Vec<_>>();
    let actual = remaining
        .runs
        .iter()
        .map(|run| (run.run_id.as_str(), run.components.clone()))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    for run in &remaining.runs {
        let source = original
            .runs
            .iter()
            .find(|candidate| candidate.run_id == run.run_id)
            .unwrap();
        if run.components == source.components {
            assert_eq!(run, source, "unchanged run {} differs", run.run_id);
        } else {
            assert!(run.postconditions.is_empty(), "{}", run.run_id);
        }
    }
}

/// Components of `plan` that `features` install.
fn installed_components(
    manifest: &Manifest,
    plan: &InstallPlan,
    features: &[&str],
) -> Vec<(String, u32)> {
    features
        .iter()
        .flat_map(|id| &feature(manifest, id).components)
        .filter(|component| {
            plan.components_for(&component.run_id)
                .is_some_and(|components| components.contains(&component.component))
        })
        .map(|component| (component.run_id.clone(), component.component))
        .collect()
}

/// Features the selection asks for that are off only because they conflict with a feature of
/// `mod_feature`.
fn alternatives_held_off<'a>(
    manifest: &'a Manifest,
    evaluation: &SelectionEvaluation,
    mod_feature: &str,
) -> Vec<&'a str> {
    let of_mod = |id: &str| {
        id == mod_feature || feature(manifest, id).parent.as_deref() == Some(mod_feature)
    };
    manifest
        .collection
        .features
        .iter()
        .filter(|candidate| {
            evaluation
                .normalized_selection
                .features
                .get(&candidate.id)
                .copied()
                .unwrap_or(false)
                && !evaluation.view.control(&candidate.id).unwrap().selected
                && candidate
                    .conflicts
                    .iter()
                    .any(|conflict| of_mod(&conflict.feature_id))
        })
        .map(|candidate| candidate.id.as_str())
        .collect()
}

/// Selected features of other mods that directly require a feature of `mod_feature`.
fn direct_dependents<'a>(
    manifest: &'a Manifest,
    evaluation: &SelectionEvaluation,
    mod_feature: &str,
) -> Vec<&'a str> {
    let of_mod = |id: &str| {
        id == mod_feature || feature(manifest, id).parent.as_deref() == Some(mod_feature)
    };
    manifest
        .collection
        .features
        .iter()
        .filter(|candidate| {
            !of_mod(&candidate.id)
                && evaluation.view.control(&candidate.id).unwrap().selected
                && candidate.requires.iter().any(|required| of_mod(required))
        })
        .map(|candidate| candidate.id.as_str())
        .collect()
}

fn leave_out(manifest: &Manifest, evaluation: &SelectionEvaluation, artifact: &str) -> InstallPlan {
    let remaining = plan_without_artifacts(
        manifest,
        &evaluation.normalized_selection,
        &evaluation.plan,
        &BTreeSet::from([artifact.to_owned()]),
    )
    .unwrap_or_else(|error| panic!("leaving out {artifact}: {error}"));
    assert_only_removes(&evaluation.plan, &remaining);
    assert!(runs_from(&remaining, artifact).is_empty());
    remaining
}

#[test]
fn leaving_out_spell_revisions_removes_it_and_what_needs_it_and_adds_nothing() {
    let manifest = curated();
    let with = evaluate(&manifest, &recommended(&manifest, true)).unwrap();
    let without = evaluate(&manifest, &recommended(&manifest, false)).unwrap();
    let artifact = artifact_of(&manifest, SPELL_REVISIONS);
    assert_eq!(
        runs_from(&with.plan, &artifact),
        [
            "spell-rev-core-bg2",
            "spell-rev-npc-spellbooks-bg2",
            "spell-rev-lightning-bg2"
        ]
    );
    assert!(runs_from(&without.plan, &artifact).is_empty());
    let dependents = direct_dependents(&manifest, &with, SPELL_REVISIONS);
    assert!(!dependents.is_empty());
    let alternatives = alternatives_held_off(&manifest, &with, SPELL_REVISIONS);
    // The Artisan's Kitpack and Bardic Wonders parts marked "Unavailable with Spell
    // Revisions" are ticked by the preset and come back when Spell Revisions is unticked.
    assert!(!alternatives.is_empty());
    assert!(!installed_components(&manifest, &without.plan, &alternatives).is_empty());

    let remaining = leave_out(&manifest, &with, &artifact);

    assert_matches_unticking(&with.plan, &remaining, &without.plan);
    assert!(installed_components(&manifest, &remaining, &dependents).is_empty());
    assert!(installed_components(&manifest, &remaining, &alternatives).is_empty());
    // The Lightning Bolt choice group belongs to Spell Revisions and goes with it.
    assert!(installed_components(
        &manifest,
        &remaining,
        &[
            "feature:spell-rev:component-80",
            "feature:spell-rev:component-81"
        ]
    )
    .is_empty());
}

#[test]
fn leaving_out_scs_with_spell_revisions_removes_its_dependents_and_adds_nothing() {
    leaving_out_scs(true);
}

#[test]
fn leaving_out_scs_without_spell_revisions_removes_its_dependents_and_adds_nothing() {
    leaving_out_scs(false);
}

fn leaving_out_scs(spell_revisions: bool) {
    let manifest = curated();
    let selection = recommended(&manifest, spell_revisions);
    let evaluation = evaluate(&manifest, &selection).unwrap();
    let mut unticked_selection = selection;
    unticked_selection.set_feature(STRATAGEMS, false);
    let unticked = evaluate(&manifest, &unticked_selection).unwrap();
    let artifact = artifact_of(&manifest, STRATAGEMS);
    let spell_revisions_artifact = artifact_of(&manifest, SPELL_REVISIONS);
    assert_eq!(runs_from(&evaluation.plan, &artifact), ["stratagems-bg2"]);
    assert_eq!(
        runs_from(&evaluation.plan, &spell_revisions_artifact).is_empty(),
        !spell_revisions
    );
    let dependents = direct_dependents(&manifest, &evaluation, STRATAGEMS);
    assert!(dependents.contains(&"feature:chriz-bg-rebalance:component-101"));
    assert_eq!(
        dependents.contains(&"feature:chriz-bg-rebalance:mandatory-components"),
        spell_revisions
    );
    let alternatives = alternatives_held_off(&manifest, &evaluation, STRATAGEMS);
    assert!(alternatives.contains(&"feature:randomiser:component-10300"));
    assert!(!installed_components(&manifest, &unticked.plan, &alternatives).is_empty());

    let remaining = leave_out(&manifest, &evaluation, &artifact);

    assert_matches_unticking(&evaluation.plan, &remaining, &unticked.plan);
    assert!(installed_components(&manifest, &remaining, &dependents).is_empty());
    assert!(installed_components(&manifest, &remaining, &alternatives).is_empty());
    // Every option of SCS's own choice groups goes with it, picked or not.
    let scs_choices = manifest
        .collection
        .features
        .iter()
        .filter(|feature| {
            feature
                .choice_group
                .as_deref()
                .is_some_and(|group| group.starts_with("choice:stratagems:"))
        })
        .map(|feature| feature.id.as_str())
        .collect::<Vec<_>>();
    assert!(!installed_components(&manifest, &evaluation.plan, &scs_choices).is_empty());
    assert!(installed_components(&manifest, &remaining, &scs_choices).is_empty());
    // Spell Revisions stays exactly as planned: kept when selected, never added otherwise.
    assert_eq!(
        runs_from(&remaining, &spell_revisions_artifact),
        runs_from(&evaluation.plan, &spell_revisions_artifact)
    );
    for run_id in runs_from(&remaining, &spell_revisions_artifact) {
        assert_eq!(
            remaining.runs.iter().find(|run| run.run_id == run_id),
            evaluation.plan.runs.iter().find(|run| run.run_id == run_id)
        );
    }
}
