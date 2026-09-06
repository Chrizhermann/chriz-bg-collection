//! Source-only acceptance against the actual curated recipe, without game I/O.
use std::path::PathBuf;

use bg_engine::{
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
    assert_eq!(result.plan.runs.len(), 43);
    assert_eq!(
        result
            .plan
            .runs
            .iter()
            .map(|run| run.components.len())
            .sum::<usize>(),
        434
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
