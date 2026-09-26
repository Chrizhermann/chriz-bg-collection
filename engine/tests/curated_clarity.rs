//! Source-only acceptance against the actual curated recipe, without game I/O.
use std::path::PathBuf;

use bg_engine::{
    manifest::{Decision, Readiness},
    recipe_view::{evaluate, evaluate_preset},
    Manifest,
};

fn recipe() -> Manifest {
    Manifest::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../recipes/curated-full-current"),
    )
    .expect("curated recipe loads with presentation metadata")
}

#[test]
fn clarity_keeps_the_accepted_default_install_and_buffbot_tail() {
    let manifest = recipe();
    let result = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(result.plan.runs.len(), 50);
    assert_eq!(
        result
            .plan
            .runs
            .iter()
            .map(|run| run.components.len())
            .sum::<usize>(),
        450 // September 26 approval: Emotion exclusivity (BG Rebalance 301) now default.
    );
    let tail = result.plan.runs.last().unwrap();
    assert_eq!(tail.mod_id, "buffbot");
    assert_eq!(tail.components, vec![1, 0]);
    assert!(result
        .view
        .controls
        .iter()
        .all(|control| control.source_label.is_some() && control.group_label.is_some()));
    let potion = result
        .view
        .control("feature:cdtweaks:component-1142")
        .unwrap();
    assert_eq!(potion.title, "Potions require identification");
}

#[test]
fn emotion_exclusivity_is_a_ready_default_with_an_independent_opt_out() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let feature_id = "feature:chriz-bg-rebalance:component-301";
    let control = baseline.view.control(feature_id).unwrap();
    assert_eq!(control.decision, Decision::Default);
    assert_eq!(control.readiness, Readiness::Ready);
    assert!(control.selected && control.interactive);
    let feature = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == feature_id)
        .unwrap();
    assert_eq!(feature.parent.as_deref(), Some("mod:chriz-bg-rebalance"));
    assert_eq!(
        manifest.mods["chriz-bg-rebalance"].artifact_id,
        "chriz-bg-rebalance-0.6.0"
    );
    assert_eq!(
        baseline.plan.components_for("chriz-bg-rebalance-bg2"),
        Some(&[100, 101, 120, 121, 301, 400, 401, 404, 405, 407, 408][..])
    );

    let mut selection = baseline.normalized_selection.to_selection();
    selection.set_feature(feature_id, false);
    let disabled = evaluate(&manifest, &selection).unwrap();
    let mut expected = baseline.plan.clone();
    expected
        .runs
        .iter_mut()
        .find(|run| run.run_id == "chriz-bg-rebalance-bg2")
        .unwrap()
        .components
        .retain(|component| *component != 301);
    assert_eq!(disabled.plan, expected);

    let mut selection = baseline.normalized_selection.to_selection();
    selection.set_feature("mod:chriz-bg-rebalance", false);
    let parent_disabled = evaluate(&manifest, &selection).unwrap();
    assert!(parent_disabled.normalized_selection.features[feature_id]);
    assert!(!parent_disabled.view.control(feature_id).unwrap().selected);
    assert_eq!(
        parent_disabled
            .plan
            .components_for("chriz-bg-rebalance-bg2"),
        None
    );
}

#[test]
fn emotion_exclusivity_keeps_the_reviewed_provider_and_later_tweak_order() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let position = |id: &str| {
        baseline
            .plan
            .runs
            .iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing selected run {id}"))
    };
    let rebalance = position("chriz-bg-rebalance-bg2");
    for provider in ["iwdification-bg2", "spell-rev-core-bg2", "stratagems-bg2"] {
        assert!(position(provider) < rebalance, "{provider}");
    }
    assert!(baseline
        .plan
        .components_for("stratagems-bg2")
        .unwrap()
        .contains(&5900));
    for later in ["chriz-bg-modpack-bg2", "cdtweaks-spell-save-penalties-bg2"] {
        assert!(rebalance < position(later), "{later}");
    }
    assert!(baseline
        .plan
        .components_for("cdtweaks-spell-save-penalties-bg2")
        .unwrap()
        .contains(&2312));
}

#[test]
fn interval_save_controls_follow_the_selected_interval_without_forcing_it_on() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let hgo = "feature:hiddengameplayoptions:component-38";
    assert!(baseline.view.control(hgo).unwrap().selected);
    assert!(baseline
        .plan
        .runs
        .iter()
        .find(|run| run.mod_id == "hidden-gameplay-options")
        .unwrap()
        .components
        .contains(&38));
    let mut selection = baseline.normalized_selection.to_selection();
    selection.set_feature("feature:cdtweaks:component-3354", false);
    selection.set_feature("feature:cdtweaks:component-3355", true);
    assert!(
        evaluate(&manifest, &selection)
            .unwrap()
            .view
            .control(hgo)
            .unwrap()
            .selected
    );
    selection.set_feature("feature:cdtweaks:component-3355", false);
    let disabled = evaluate(&manifest, &selection).unwrap();
    assert!(!disabled.view.control(hgo).unwrap().selected);
    assert!(disabled
        .plan
        .runs
        .iter()
        .find(|run| run.mod_id == "cdtweaks")
        .unwrap()
        .components
        .iter()
        .all(|id| !(3350..=3357).contains(id)));
    assert!(!disabled
        .plan
        .runs
        .iter()
        .find(|run| run.mod_id == "hidden-gameplay-options")
        .unwrap()
        .components
        .contains(&38));
}

#[test]
fn armor_qol_is_optional_and_keeps_the_original_install_when_disabled() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let control = baseline
        .view
        .control("feature:klatu:component-2150")
        .expect("armor QoL has its own visible control");
    assert!(control.selected);
    assert_eq!(control.title, "Use thief skills in armor");
    let mut selection = baseline.normalized_selection.to_selection();
    selection.set_feature("feature:klatu:component-2150", false);
    let changed = evaluate(&manifest, &selection).unwrap();
    let mut expected = baseline.plan.runs;
    let klatu = expected.remove(expected.len() - 2);
    assert_eq!(klatu.run_id, "klatu-armor-thieving-bg2");
    assert_eq!(klatu.components, vec![2150]);
    assert_eq!(changed.plan.runs, expected);
    assert_eq!(changed.plan.runs.last().unwrap().mod_id, "buffbot");

    let findings = bg_engine::release_validate::validate_public_alpha(&manifest).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn real_potion_choice_switches_without_changing_any_other_run() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let option = baseline
        .view
        .control("feature:cdtweaks:component-3101")
        .unwrap();
    assert_eq!(option.choice_available, Some(true));
    let mut selection = baseline.normalized_selection.to_selection();
    for control in &baseline.view.controls {
        if control.choice_group == option.choice_group {
            selection.set_feature(&control.id, control.id == option.id);
        }
    }
    let changed = evaluate(&manifest, &selection).unwrap();
    let mut expected = baseline.plan;
    let run = expected
        .runs
        .iter_mut()
        .find(|run| run.mod_id == "cdtweaks")
        .unwrap();
    *run.components
        .iter_mut()
        .find(|component| **component == 3100)
        .unwrap() = 3101;
    assert_eq!(changed.plan, expected);
}

#[test]
fn disabling_eeex_explains_and_omits_buffbot_without_changing_requested_preference() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let mut selection = baseline.normalized_selection.to_selection();
    selection.set_feature("mod:eeex", false);
    let changed = evaluate(&manifest, &selection).unwrap();
    assert!(changed.normalized_selection.features["mod:buffbot"]);
    let buffbot = changed.view.control("mod:buffbot").unwrap();
    assert!(!buffbot.selected);
    assert!(buffbot
        .unavailable_reason
        .as_deref()
        .unwrap()
        .contains("EEex"));
    assert!(!changed.plan.runs.iter().any(|run| run.mod_id == "buffbot"));
}

#[test]
fn repaired_bg1npc_groups_replace_only_the_authored_alternative() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    for (old, new) in [(111, 110), (240, 241), (130, 131)] {
        let option = baseline
            .view
            .control(&format!("feature:bg1npc:component-{new}"))
            .unwrap();
        assert_eq!(option.choice_available, Some(true));
        let mut selection = baseline.normalized_selection.to_selection();
        for control in &baseline.view.controls {
            if control.choice_group == option.choice_group {
                selection.set_feature(&control.id, control.id == option.id);
            }
        }
        let changed = evaluate(&manifest, &selection).unwrap();
        let mut expected = baseline.plan.clone();
        let run = expected
            .runs
            .iter_mut()
            .find(|run| run.mod_id == "bg1npc")
            .unwrap();
        *run.components
            .iter_mut()
            .find(|component| **component == old)
            .unwrap() = new;
        assert_eq!(changed.plan, expected);
    }
}
