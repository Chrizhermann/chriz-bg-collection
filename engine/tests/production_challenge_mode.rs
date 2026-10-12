//! Challenge Mode in both production profiles: the bundled public-alpha `manifest/` and the
//! curated `recipes/curated-full-current/`. Static recipe evaluation only; no game install.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

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
fn both_profiles_define_challenge_mode_with_exactly_the_four_optional_members() {
    for (dir, manifest) in profiles() {
        let groups = &manifest.collection.feature_groups;
        assert_eq!(groups.len(), 1, "{dir}");
        let group = &groups[0];
        assert_eq!(group.id, GROUP, "{dir}");
        assert_eq!(group.title, "Challenge Mode", "{dir}");
        assert_eq!(group.members, MEMBERS, "{dir}");
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
            for member in MEMBERS {
                // Chris's Recommended may name a member only to keep it off (Emotion 301).
                assert!(
                    preset
                        .selections
                        .get(member)
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
        assert_eq!(state.available_members, MEMBERS, "{dir}");
        assert!(state.unavailable_members.is_empty(), "{dir}");
        assert_eq!(state.disabled_reason, None, "{dir}");
        for feature in NORMAL_DEFAULTS {
            assert!(
                evaluation.view.control(feature).unwrap().selected,
                "{dir} {feature}"
            );
        }
    }
}

#[test]
fn turning_challenge_mode_on_selects_exactly_the_members_and_off_restores_recommended() {
    for (dir, manifest) in profiles() {
        let base = recommended_selection(&manifest);
        let recommended = evaluate(&manifest, &base).unwrap();

        let on_changes =
            feature_group_changes(&manifest, &base, GROUP, FeatureGroupTarget::On).unwrap();
        assert_eq!(
            on_changes,
            MEMBERS.iter().map(|id| ((*id).to_owned(), true)).collect(),
            "{dir}"
        );
        let on_selection = apply(&base, &on_changes);
        let on = evaluate(&manifest, &on_selection).unwrap();
        assert_eq!(
            on.group(GROUP).unwrap().state,
            FeatureGroupState::On,
            "{dir}"
        );

        // Only the members change: every other control and every non-member component stays.
        let before = selected(&recommended);
        let after = selected(&on);
        for (id, was_selected) in &before {
            if MEMBERS.contains(&id.as_str()) {
                assert!(!was_selected && after[id], "{dir} {id}");
            } else {
                assert_eq!(after[id], *was_selected, "{dir} {id} changed");
            }
        }
        let added = installed_components(&on)
            .difference(&installed_components(&recommended))
            .cloned()
            .collect::<BTreeSet<_>>();
        assert!(
            installed_components(&recommended).is_subset(&installed_components(&on)),
            "{dir}"
        );
        assert_eq!(added, member_components(&manifest, &MEMBERS), "{dir}");
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
        let members = member_components(&manifest, &MEMBERS);
        for run in on
            .plan
            .runs
            .iter()
            .filter(|run| !recommended_runs.contains(&run.run_id.as_str()))
        {
            assert!(
                run.components
                    .iter()
                    .all(|component| members.contains(&(run.run_id.clone(), *component))),
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
        for member in &MEMBERS[..3] {
            assert!(
                !evaluation.view.control(member).unwrap().selected,
                "{dir} {member}"
            );
        }
    }
}

#[test]
fn without_scs_dragon_melee_is_unavailable_and_never_forced_on() {
    for (dir, manifest) in profiles() {
        let mut base = recommended_selection(&manifest);
        base.set_feature("mod:stratagems", false);
        let evaluation = evaluate(&manifest, &base).unwrap();
        let state = evaluation.group(GROUP).unwrap();
        assert_eq!(state.available_members.len(), 3, "{dir}");
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
        assert_eq!(state.selected_members.len(), 3, "{dir}");
        assert!(!on.view.control(DRAGON_MELEE).unwrap().selected, "{dir}");
        assert!(!on.normalized_selection.features[DRAGON_MELEE], "{dir}");
    }
}
