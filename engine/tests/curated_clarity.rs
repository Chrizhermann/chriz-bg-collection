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
    // The October 13 intake moves Modpack scroll policy 430 into its own final run.
    assert_eq!(result.plan.runs.len(), 53);
    assert_eq!(
        result
            .plan
            .runs
            .iter()
            .map(|run| run.components.len())
            .sum::<usize>(),
        // Next-release policy removes Emotion 301 from October 7's defaults;
        // October 13 default bard ceilings add 420/421 and Bardic 3010;
        // EET's desktop shortcut (EET 100) becomes an unchecked option.
        // The October 13 intake adds Modpack 237/238/660 and SoD 177/198/285.
        469
    );
    // Scroll policy 430 hashes the final scrolls and spells, so it follows BuffBot.
    let (tail, rest) = result.plan.runs.split_last().unwrap();
    assert_eq!(tail.run_id, "chriz-bg-modpack-scrolls-bg2");
    assert_eq!(tail.components, vec![430]);
    let buffbot = rest.last().unwrap();
    assert_eq!(buffbot.mod_id, "buffbot");
    assert_eq!(buffbot.components, vec![1, 0]);
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
fn emotion_exclusivity_is_a_ready_optional_choice_with_an_independent_opt_in() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let feature_id = "feature:chriz-bg-rebalance:component-301";
    let control = baseline.view.control(feature_id).unwrap();
    assert_eq!(control.decision, Decision::Optional);
    assert_eq!(control.readiness, Readiness::Ready);
    assert!(!control.selected && control.interactive);
    let baeloth = baseline
        .view
        .control("feature:chriz-bg-modpack:component-235")
        .unwrap();
    assert_eq!(baeloth.decision, Decision::Optional);
    assert!(!baeloth.selected && baeloth.interactive);
    let feature = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == feature_id)
        .unwrap();
    assert_eq!(feature.parent.as_deref(), Some("mod:chriz-bg-rebalance"));
    assert_eq!(
        manifest.mods["chriz-bg-rebalance"].artifact_id,
        "chriz-bg-rebalance-0.8.2"
    );
    assert_eq!(
        baseline.plan.components_for("chriz-bg-rebalance-bg2"),
        Some(
            &[
                100, 101, 111, 120, 121, 200, 210, 211, 400, 401, 404, 405, 407, 408, 302, 420,
                421, 310, 130, 220, 320,
            ][..]
        )
    );

    let mut selection = baseline.normalized_selection.to_selection();
    selection.set_feature(feature_id, true);
    let enabled = evaluate(&manifest, &selection).unwrap();
    let mut expected = baseline.plan.clone();
    expected
        .runs
        .iter_mut()
        .find(|run| run.run_id == "chriz-bg-rebalance-bg2")
        .unwrap()
        .components
        .insert(8, 301);
    assert_eq!(enabled.plan, expected);

    let mut selection = enabled.normalized_selection.to_selection();
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
fn stronghold_choices_cannot_send_native_incompatible_components_to_weidu() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let feature = |n| format!("feature:cdtweaks:component-{n}");
    for n in 1340..=1347 {
        let control = baseline.view.control(&feature(n)).unwrap();
        assert!(!control.selected && !control.interactive);
        assert!(control
            .unavailable_reason
            .as_deref()
            .unwrap()
            .contains("Multiple Strongholds"));
    }

    // Both supported Multiple Strongholds choices remain usable on their own.
    for multiple in [1160, 1161] {
        let mut selection = baseline.normalized_selection.to_selection();
        for n in [1160, 1161] {
            selection.set_feature(&feature(n), n == multiple);
        }
        let result = evaluate(&manifest, &selection).unwrap();
        assert!(result
            .plan
            .components_for("cdtweaks-bg2")
            .unwrap()
            .contains(&multiple));
        // Even a stale/imported selection cannot slip an incompatible pair through.
        for individual in 1340..=1347 {
            let mut invalid = selection.clone();
            invalid.set_feature(&feature(individual), true);
            let result = evaluate(&manifest, &invalid).unwrap();
            let components = result.plan.components_for("cdtweaks-bg2").unwrap();
            assert!(!components.contains(&multiple));
            assert!(!components.contains(&individual));
        }
    }

    // All eight individual choices are mutually compatible once Multiple is off.
    let mut selection = baseline.normalized_selection.to_selection();
    for n in [1160, 1161] {
        selection.set_feature(&feature(n), false);
    }
    for n in 1340..=1347 {
        selection.set_feature(&feature(n), true);
    }
    let result = evaluate(&manifest, &selection).unwrap();
    let mut expected = baseline.plan.clone();
    let components = &mut expected
        .runs
        .iter_mut()
        .find(|run| run.run_id == "cdtweaks-bg2")
        .unwrap()
        .components;
    // These eight components immediately follow 1161 in the native TP2 order,
    // which is not globally numeric. Preserve every other component's position.
    let at = components.iter().position(|n| *n == 1161).unwrap();
    components.splice(at..=at, 1340..=1347);
    assert_eq!(result.plan, expected);
    for n in 1340..=1347 {
        let control = result.view.control(&feature(n)).unwrap();
        assert!(control.selected && control.interactive);
    }
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
fn custom_bg1npc_choices_require_the_core_even_with_the_parent_enabled() {
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let mut selection = baseline.normalized_selection.to_selection();
    selection.set_feature("feature:bg1npc:component-0", false);
    for feature in &manifest.collection.features {
        if feature.id.starts_with("feature:bg1npc:") && feature.id != "feature:bg1npc:component-0" {
            let mut requested = selection.clone();
            requested.set_feature(&feature.id, true);
            let result = evaluate(&manifest, &requested).unwrap();
            assert!(result.view.control("mod:bg1npc").unwrap().selected);
            assert!(result.normalized_selection.features[&feature.id]);
            assert!(
                !result.view.control(&feature.id).unwrap().selected,
                "{}",
                feature.id
            );
            assert_eq!(
                result.plan.components_for("bg1npc-bg1"),
                None,
                "{}",
                feature.id
            );
        }
    }
    selection.set_feature("feature:bg1npc:component-0", true);
    assert_eq!(evaluate(&manifest, &selection).unwrap().plan, baseline.plan);
}

#[test]
fn scs_bundle_requires_the_actual_ascension_finale_in_both_profiles() {
    for directory in ["manifest", "recipes/curated-full-current"] {
        let manifest = Manifest::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(directory),
        )
        .unwrap();
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let mut selection = baseline.normalized_selection.to_selection();
        selection.set_feature("feature:ascension:component-0", false);
        let result = evaluate(&manifest, &selection).unwrap();
        assert!(result.view.control("mod:ascension").unwrap().selected);
        let scs = result.view.control("mod:stratagems").unwrap();
        assert!(!scs.selected && !scs.interactive);
        assert!(scs
            .unavailable_reason
            .as_deref()
            .unwrap()
            .contains("Rewritten Final Chapter"));
        assert!(result.normalized_selection.features["mod:stratagems"]);
        assert_eq!(result.plan.components_for("stratagems-bg2"), None);
        selection.set_feature("feature:ascension:component-0", true);
        assert_eq!(evaluate(&manifest, &selection).unwrap().plan, baseline.plan);
    }
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
    // Klatu runs right before BuffBot; only the Modpack scroll policy follows BuffBot.
    let klatu = expected.remove(expected.len() - 3);
    assert_eq!(klatu.run_id, "klatu-armor-thieving-bg2");
    assert_eq!(klatu.components, vec![2150]);
    assert_eq!(changed.plan.runs, expected);
    assert_eq!(
        changed.plan.runs[changed.plan.runs.len() - 2].mod_id,
        "buffbot"
    );
    assert_eq!(
        changed.plan.runs.last().unwrap().run_id,
        "chriz-bg-modpack-scrolls-bg2"
    );

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

#[test]
fn bard_spell_ceilings_are_default_and_gate_the_bardic_kit_adapter() {
    const BASE: &str = "feature:chriz-bg-rebalance:bard-spell-ceilings";
    const ADAPTER: &str = "feature:bardicwonders:component-3010";
    const REBALANCE: &str = "chriz-bg-rebalance-bg2";
    const BARDIC: &str = "bardicwonders-progression-bg2";
    let manifest = recipe();
    let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let has = |plan: &bg_engine::resolve::InstallPlan, run: &str, component: u32| {
        plan.components_for(run)
            .is_some_and(|components| components.contains(&component))
    };

    // Default (Christopher's decision, 2026-10-13): both checked and deselectable.
    for id in [BASE, ADAPTER] {
        let control = baseline.view.control(id).unwrap();
        assert_eq!(control.decision, Decision::Default, "{id}");
        assert_eq!(control.readiness, Readiness::Ready, "{id}");
        assert!(control.selected && control.interactive, "{id}");
    }
    assert!(has(&baseline.plan, REBALANCE, 420) && has(&baseline.plan, REBALANCE, 421));

    // Opting out of both removes exactly 420/421 and the 3010 run.
    let mut neither = baseline.normalized_selection.to_selection();
    neither.set_feature(BASE, false);
    neither.set_feature(ADAPTER, false);
    let neither = evaluate(&manifest, &neither).unwrap();
    let mut expected = baseline.plan.clone();
    expected.runs.retain(|run| run.run_id != BARDIC);
    expected
        .runs
        .iter_mut()
        .find(|run| run.run_id == REBALANCE)
        .unwrap()
        .components
        .retain(|c| ![420, 421].contains(c));
    assert_eq!(neither.plan, expected);

    // A stale selection of only the adapter cannot install 3010 without 420/421.
    let mut adapter_only = neither.normalized_selection.to_selection();
    adapter_only.set_feature(ADAPTER, true);
    let adapter_only = evaluate(&manifest, &adapter_only).unwrap();
    assert_eq!(adapter_only.plan, neither.plan);
    let adapter = adapter_only.view.control(ADAPTER).unwrap();
    assert!(!adapter.selected && !adapter.interactive);
    assert_eq!(
        adapter.unavailable_reason.as_deref(),
        Some("Requires Bard spell-level ceilings.")
    );

    // The base choice alone adds exactly 420 then 421 after Flail 302.
    let mut base_only = neither.normalized_selection.to_selection();
    base_only.set_feature(BASE, true);
    let base_only = evaluate(&manifest, &base_only).unwrap();
    let mut expected = neither.plan.clone();
    let rebalance = &mut expected
        .runs
        .iter_mut()
        .find(|run| run.run_id == REBALANCE)
        .unwrap()
        .components;
    let after_flail = rebalance.iter().position(|c| *c == 302).unwrap() + 1;
    rebalance.splice(after_flail..after_flail, [420, 421]);
    assert_eq!(base_only.plan, expected);
    assert!(base_only.view.control(ADAPTER).unwrap().interactive);

    // Both add 3010 in its own late run immediately after the Rebalance run.
    let both = baseline;
    assert_eq!(both.plan.components_for(BARDIC), Some(&[3010][..]));
    let position = |id: &str| {
        both.plan
            .runs
            .iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing selected run {id}"))
    };
    assert_eq!(position(BARDIC), position(REBALANCE) + 1);
    for earlier in [
        "eeex-bg2",
        "bardicwonders-bg2",
        "bardicwonders-garrick-bg2",
        "bardicwonders-dialogue-patch-bg2",
        "cdtweaks-bg2",
    ] {
        assert!(position(earlier) < position(BARDIC), "{earlier}");
    }
    let mut without_adapter = both.plan.clone();
    without_adapter.runs.retain(|run| run.run_id != BARDIC);
    assert_eq!(without_adapter, base_only.plan);

    // Deselecting Bardic Wonders keeps 420/421 and drops every Bardic run.
    let mut no_bardic = both.normalized_selection.to_selection();
    no_bardic.set_feature("mod:bardicwonders", false);
    let no_bardic = evaluate(&manifest, &no_bardic).unwrap();
    assert!(has(&no_bardic.plan, REBALANCE, 420) && has(&no_bardic.plan, REBALANCE, 421));
    assert!(no_bardic.view.control(BASE).unwrap().selected);
    assert!(!no_bardic
        .plan
        .runs
        .iter()
        .any(|run| run.mod_id == "bardicwonders"));

    // Removing the base choice, Rebalance or EEex removes the adapter too.
    for disabled in [BASE, "mod:chriz-bg-rebalance", "mod:eeex"] {
        let mut selection = both.normalized_selection.to_selection();
        selection.set_feature(disabled, false);
        let result = evaluate(&manifest, &selection).unwrap();
        assert_eq!(result.plan.components_for(BARDIC), None, "{disabled}");
        assert!(!has(&result.plan, REBALANCE, 420), "{disabled}");
        assert!(
            !result.view.control(ADAPTER).unwrap().selected,
            "{disabled}"
        );
    }
}
