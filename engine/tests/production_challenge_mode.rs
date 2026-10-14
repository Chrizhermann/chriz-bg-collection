//! Challenge Mode in both production profiles: the bundled public-alpha `manifest/` and the
//! curated `recipes/curated-full-current/`. Static recipe evaluation only; no game install.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bg_engine::manifest::FeatureGroupVariant;
use bg_engine::recipe_view::{
    evaluate, evaluate_preset, feature_group_changes, FeatureGroupState, FeatureGroupTarget,
    SelectionEvaluation,
};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const GROUP: &str = "challenge-mode";
const MEMBERS: [&str; 4] = [
    "feature:chriz-bg-rebalance:component-301",
    "feature:chriz-bg-modpack:component-235",
    "feature:chriz-bg-rebalance:component-110",
    "feature:chriz-sod-remix:component-257",
];
/// Normal defaults that are explicitly not Challenge Mode members.
const NORMAL_DEFAULTS: [&str; 4] = [
    "feature:chriz-bg-rebalance:component-302", // Flail of the Ages balance
    "feature:chriz-bg-rebalance:component-130", // Elemental arrows: BG2 damage
    "feature:chriz-bg-rebalance:component-310", // Party physical resistance, 90% cap
    "feature:chriz-bg-rebalance:component-111", // Dragon wing buffet spacing
];
const DRAGON_MELEE: &str = "feature:chriz-bg-rebalance:component-110";
/// Christopher, 2026-10-14: Challenge Mode picks strict potion uniqueness (Rebalance 330)
/// instead of the recommended 331, where Potions of Genius still stack.
const POTION_CHOICE: &str = "potion-uniqueness";
const STRICT_POTIONS: &str = "feature:chriz-bg-rebalance:component-330";
const GENIUS_STACKS: &str = "feature:chriz-bg-rebalance:component-331";
/// Tweaks Anthology runs only in the curated profile (Christopher, 2026-10-14).
const CURATED_ONLY_MEMBERS: [&str; 1] = ["feature:cdtweaks:component-2760"];

fn members(dir: &str) -> Vec<&'static str> {
    let mut members = MEMBERS.to_vec();
    if dir.ends_with("curated-full-current") {
        members.extend(CURATED_ONLY_MEMBERS);
    }
    members
}

fn profiles() -> Vec<(&'static str, Manifest)> {
    ["../manifest", "../recipes/curated-full-current"]
        .into_iter()
        .map(|dir| {
            let manifest =
                Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(dir)).unwrap();
            (dir, manifest)
        })
        .collect()
}

fn recommended_selection(manifest: &Manifest) -> Selection {
    Selection {
        platform: "windows".to_owned(),
        choices: manifest.presets["chris-recommended"].selections.clone(),
    }
}

fn apply(selection: &Selection, changes: &BTreeMap<String, bool>) -> Selection {
    let mut selection = selection.clone();
    for (feature, selected) in changes {
        selection.set_feature(feature, *selected);
    }
    selection
}

fn installed_components(evaluation: &SelectionEvaluation) -> BTreeSet<(String, u32)> {
    evaluation
        .plan
        .runs
        .iter()
        .flat_map(|run| {
            run.components
                .iter()
                .map(|component| (run.run_id.clone(), *component))
        })
        .collect()
}

fn member_components(manifest: &Manifest, members: &[&str]) -> BTreeSet<(String, u32)> {
    manifest
        .collection
        .features
        .iter()
        .filter(|feature| members.contains(&feature.id.as_str()))
        .flat_map(|feature| {
            feature
                .components
                .iter()
                .map(|component| (component.run_id.clone(), component.component))
        })
        .collect()
}

fn selected(evaluation: &SelectionEvaluation) -> BTreeMap<String, bool> {
    evaluation
        .view
        .controls
        .iter()
        .map(|control| (control.id.clone(), control.selected))
        .collect()
}

#[test]
fn both_profiles_define_challenge_mode_with_exactly_their_optional_members() {
    for (dir, manifest) in profiles() {
        let members = members(dir);
        let groups = &manifest.collection.feature_groups;
        assert_eq!(groups.len(), 1, "{dir}");
        let group = &groups[0];
        assert_eq!(group.id, GROUP, "{dir}");
        assert_eq!(group.title, "Challenge Mode", "{dir}");
        assert_eq!(group.members, members, "{dir}");
        assert_eq!(
            group.variants,
            [FeatureGroupVariant {
                choice_group: POTION_CHOICE.to_owned(),
                option: STRICT_POTIONS.to_owned(),
                label: Some("Potions: strict (Potions of Genius don't stack either)".to_owned()),
            }],
            "{dir}"
        );
        assert!(
            group
                .description
                .contains("switch each one on or off under its own mod"),
            "{dir}"
        );
        // No preset stores the group; it is derived from member selections.
        for preset in manifest.presets.values() {
            assert!(
                !preset.selections.contains_key(GROUP),
                "{dir} {}",
                preset.id
            );
            for member in &members {
                // Chris's Recommended may name a member only to keep it off (Emotion 301).
                assert!(
                    preset
                        .selections
                        .get(*member)
                        .is_none_or(|value| value == "off"),
                    "{dir} {} {member}",
                    preset.id
                );
            }
        }
    }
}

#[test]
fn chris_recommended_keeps_challenge_mode_off_with_every_member_available() {
    for (dir, manifest) in profiles() {
        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let state = evaluation.group(GROUP).unwrap();
        assert_eq!(state.state, FeatureGroupState::Off, "{dir}");
        assert!(state.selected_members.is_empty(), "{dir}");
        assert_eq!(state.available_members, members(dir), "{dir}");
        assert!(state.unavailable_members.is_empty(), "{dir}");
        assert_eq!(state.disabled_reason, None, "{dir}");
        assert_eq!(state.variants.len(), 1, "{dir}");
        assert!(!state.variants[0].selected, "{dir}");
        assert!(state.variants[0].available, "{dir}");
        assert!(
            evaluation.view.control(GENIUS_STACKS).unwrap().selected,
            "{dir}"
        );
        let view = &evaluation.view.groups[0].variants[0];
        assert_eq!(view.options, [STRICT_POTIONS, GENIUS_STACKS], "{dir}");
        assert_eq!(view.recipe_default, [GENIUS_STACKS], "{dir}");
        for feature in NORMAL_DEFAULTS {
            assert!(
                evaluation.view.control(feature).unwrap().selected,
                "{dir} {feature}"
            );
        }
    }
}

#[test]
fn turning_challenge_mode_on_selects_exactly_its_changes_and_off_restores_recommended() {
    for (dir, manifest) in profiles() {
        let members = members(dir);
        let base = recommended_selection(&manifest);
        let recommended = evaluate(&manifest, &base).unwrap();

        let on_changes =
            feature_group_changes(&manifest, &base, GROUP, FeatureGroupTarget::On).unwrap();
        let mut expected = members
            .iter()
            .map(|id| ((*id).to_owned(), true))
            .collect::<BTreeMap<_, _>>();
        expected.insert(STRICT_POTIONS.to_owned(), true);
        expected.insert(GENIUS_STACKS.to_owned(), false);
        assert_eq!(on_changes, expected, "{dir}");
        let on_selection = apply(&base, &on_changes);
        let on = evaluate(&manifest, &on_selection).unwrap();
        assert_eq!(
            on.group(GROUP).unwrap().state,
            FeatureGroupState::On,
            "{dir}"
        );

        // Only the members and the potion choice change; every other control stays.
        let before = selected(&recommended);
        let after = selected(&on);
        for (id, was_selected) in &before {
            if members.contains(&id.as_str()) || id == STRICT_POTIONS {
                assert!(!was_selected && after[id], "{dir} {id}");
            } else if id == GENIUS_STACKS {
                assert!(*was_selected && !after[id], "{dir} {id}");
            } else {
                assert_eq!(after[id], *was_selected, "{dir} {id} changed");
            }
        }
        let added = installed_components(&on)
            .difference(&installed_components(&recommended))
            .cloned()
            .collect::<BTreeSet<_>>();
        let removed = installed_components(&recommended)
            .difference(&installed_components(&on))
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut picked = members.clone();
        picked.push(STRICT_POTIONS);
        assert_eq!(added, member_components(&manifest, &picked), "{dir}");
        assert_eq!(
            removed,
            member_components(&manifest, &[GENIUS_STACKS]),
            "{dir}"
        );
        // Existing runs keep their order; a run is added only when it consists solely of
        // member components (Baeloth 235 has its own run).
        let recommended_runs = recommended
            .plan
            .runs
            .iter()
            .map(|run| run.run_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            on.plan
                .runs
                .iter()
                .map(|run| run.run_id.as_str())
                .filter(|run_id| recommended_runs.contains(run_id))
                .collect::<Vec<_>>(),
            recommended_runs,
            "{dir}"
        );
        let member_components = member_components(&manifest, &picked);
        for run in on
            .plan
            .runs
            .iter()
            .filter(|run| !recommended_runs.contains(&run.run_id.as_str()))
        {
            assert!(
                run.components
                    .iter()
                    .all(|component| member_components.contains(&(run.run_id.clone(), *component))),
                "{dir}: added run {} contains non-member components",
                run.run_id
            );
        }
        assert_eq!(on.findings, recommended.findings, "{dir}");
        for feature in NORMAL_DEFAULTS {
            assert!(
                on.view.control(feature).unwrap().selected,
                "{dir} {feature}"
            );
        }

        let off_changes =
            feature_group_changes(&manifest, &on_selection, GROUP, FeatureGroupTarget::Off)
                .unwrap();
        let off = evaluate(&manifest, &apply(&on_selection, &off_changes)).unwrap();
        assert_eq!(
            off.group(GROUP).unwrap().state,
            FeatureGroupState::Off,
            "{dir}"
        );
        assert_eq!(
            off.normalized_selection, recommended.normalized_selection,
            "{dir}"
        );
        assert_eq!(off.plan, recommended.plan, "{dir}");
        assert!(off.view.control(GENIUS_STACKS).unwrap().selected, "{dir}");
    }
}

#[test]
fn a_hand_changed_member_makes_the_group_mixed_without_touching_the_others() {
    for (dir, manifest) in profiles() {
        let base = recommended_selection(&manifest);
        let mut selection = base.clone();
        selection.set_feature(MEMBERS[3], true);
        let evaluation = evaluate(&manifest, &selection).unwrap();
        let state = evaluation.group(GROUP).unwrap();
        assert_eq!(state.state, FeatureGroupState::Mixed, "{dir}");
        assert_eq!(state.selected_members, [MEMBERS[3]], "{dir}");
        for member in members(dir).iter().filter(|member| **member != MEMBERS[3]) {
            assert!(
                !evaluation.view.control(member).unwrap().selected,
                "{dir} {member}"
            );
        }

        // Picking the recommended potion rule again by hand after switching the group on
        // makes it mixed and touches nothing else.
        let on = apply(
            &base,
            &feature_group_changes(&manifest, &base, GROUP, FeatureGroupTarget::On).unwrap(),
        );
        let mut hand = on.clone();
        hand.set_feature(GENIUS_STACKS, true);
        hand.set_feature(STRICT_POTIONS, false);
        let evaluation = evaluate(&manifest, &hand).unwrap();
        let state = evaluation.group(GROUP).unwrap();
        assert_eq!(state.state, FeatureGroupState::Mixed, "{dir}");
        assert_eq!(state.selected_members, members(dir), "{dir}");
        assert!(!state.variants[0].selected, "{dir}");
    }
}

#[test]
fn without_bg_rebalance_the_potion_variant_is_unavailable_and_never_forced() {
    for (dir, manifest) in profiles() {
        let mut base = recommended_selection(&manifest);
        base.set_feature("mod:chriz-bg-rebalance", false);
        let evaluation = evaluate(&manifest, &base).unwrap();
        let state = evaluation.group(GROUP).unwrap();
        assert!(!state.variants[0].available, "{dir}");
        assert!(state.variants[0].unavailable_reason.is_some(), "{dir}");
        let changes =
            feature_group_changes(&manifest, &base, GROUP, FeatureGroupTarget::On).unwrap();
        assert!(!changes.contains_key(STRICT_POTIONS), "{dir}");
        assert!(!changes.contains_key(GENIUS_STACKS), "{dir}");
        let on = evaluate(&manifest, &apply(&base, &changes)).unwrap();
        assert!(!on.view.control(STRICT_POTIONS).unwrap().selected, "{dir}");
        assert_eq!(
            on.plan.components_for("chriz-bg-rebalance-bg2"),
            None,
            "{dir}"
        );
    }
}

#[test]
fn a_challenge_selection_saved_before_the_potion_variant_reloads_unchanged_as_mixed() {
    for (dir, manifest) in profiles() {
        // Switching the group on stored only the members before the variant existed.
        let mut saved = recommended_selection(&manifest);
        for member in members(dir) {
            saved.set_feature(member, true);
        }
        let first = evaluate(&manifest, &saved).unwrap();
        let reloaded = evaluate(&manifest, &first.normalized_selection.to_selection()).unwrap();
        assert_eq!(
            reloaded.normalized_selection, first.normalized_selection,
            "{dir}"
        );
        assert!(
            reloaded.view.control(GENIUS_STACKS).unwrap().selected,
            "{dir}"
        );
        assert!(
            !reloaded.view.control(STRICT_POTIONS).unwrap().selected,
            "{dir}"
        );
        let state = reloaded.group(GROUP).unwrap();
        assert_eq!(state.state, FeatureGroupState::Mixed, "{dir}");
        assert_eq!(state.selected_members, members(dir), "{dir}");
    }
}

#[test]
fn without_scs_dragon_melee_is_unavailable_and_never_forced_on() {
    for (dir, manifest) in profiles() {
        let mut base = recommended_selection(&manifest);
        base.set_feature("mod:stratagems", false);
        let evaluation = evaluate(&manifest, &base).unwrap();
        let state = evaluation.group(GROUP).unwrap();
        assert_eq!(
            state.available_members.len(),
            members(dir).len() - 1,
            "{dir}"
        );
        assert_eq!(state.unavailable_members.len(), 1, "{dir}");
        assert_eq!(
            state.unavailable_members[0].feature_id, DRAGON_MELEE,
            "{dir}"
        );
        assert_eq!(
            state.unavailable_members[0].reason, "Requires Curated SCS core.",
            "{dir}"
        );

        let changes =
            feature_group_changes(&manifest, &base, GROUP, FeatureGroupTarget::On).unwrap();
        assert!(!changes.contains_key(DRAGON_MELEE), "{dir}");
        let on = evaluate(&manifest, &apply(&base, &changes)).unwrap();
        let state = on.group(GROUP).unwrap();
        assert_eq!(state.state, FeatureGroupState::On, "{dir}");
        assert_eq!(
            state.selected_members.len(),
            members(dir).len() - 1,
            "{dir}"
        );
        assert!(!on.view.control(DRAGON_MELEE).unwrap().selected, "{dir}");
        assert!(!on.normalized_selection.features[DRAGON_MELEE], "{dir}");
    }
}
