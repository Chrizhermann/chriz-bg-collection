//! UI provider selection and publication contracts, with no game installation.
use std::path::PathBuf;

use bg_engine::{
    manifest::{AcquisitionPolicy, Decision, InvocationMode, Readiness},
    recipe_view::{evaluate, evaluate_preset},
    weidu::invocation::{setup_executable_name, validate_setup_name_declarations},
    Manifest,
};

const PROVIDERS: &[(&str, &str, u32)] = &[
    ("infinity", "infinity-ui", 0),
    ("eet-sod", "eet-gui", 0),
    ("dragonspear", "dragonspear-ui", 1),
    ("leui-bg2", "leui", 0),
    ("leui-bg1", "leui-bg1ee", 0),
    ("leui-sod", "leui-sod", 0),
];

#[test]
fn all_ui_providers_use_supported_unambiguous_setup_names() {
    let expected_names = [
        ("infinity-ui", "Setup-infinity_ui.exe"),
        ("eet-gui", "Setup-eet_gui.exe"),
        ("dragonspear-ui", "Setup-dragonspear_ui++.exe"),
        ("leui", "Setup-leui.exe"),
        ("leui-bg1ee", "Setup-leui-bg1ee.exe"),
        ("leui-sod", "Setup-leui-sod.exe"),
    ];
    for manifest in recipes() {
        for (id, executable) in expected_names {
            let mod_file = &manifest.mods[id];
            assert_eq!(mod_file.invocation_mode, InvocationMode::SetupName, "{id}");
            assert_eq!(setup_executable_name(&mod_file.tp2).unwrap(), executable);
        }
        validate_setup_name_declarations(
            manifest
                .mods
                .values()
                .filter(|mod_file| mod_file.invocation_mode == InvocationMode::SetupName)
                .map(|mod_file| mod_file.tp2.as_str()),
        )
        .expect("UI setup names must not alias another installer in either recipe");
    }
}

fn recipes() -> impl Iterator<Item = Manifest> {
    ["manifest", "recipes/curated-full-current"]
        .into_iter()
        .map(|path| {
            Manifest::load(
                &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("..")
                    .join(path),
            )
            .unwrap()
        })
}

#[test]
fn providers_are_optional_pinned_and_ordered_before_final_ui_consumers() {
    for manifest in recipes() {
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let positions = |id: &str| {
            manifest
                .collection
                .runs
                .iter()
                .position(|r| r.run_id == id)
                .unwrap()
        };
        for (id, mod_id, component) in PROVIDERS {
            let feature_id = format!("feature:game-interface:{id}");
            let control = baseline.view.control(&feature_id).unwrap();
            assert_eq!(control.decision, Decision::Optional);
            assert!(!control.selected);
            assert_eq!(control.choice_group.as_deref(), Some("game-interface"));
            let mod_file = &manifest.mods[*mod_id];
            assert_eq!(mod_file.language, 0);
            assert_eq!(mod_file.components.len(), 1);
            assert_eq!(mod_file.components[0].id, *component);
            let artifact = &manifest.artifacts[&mod_file.artifact_id];
            assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
            assert_eq!(artifact.source.sha256.len(), 64);
            assert!(artifact.archive.tp2_paths.contains(&mod_file.tp2));
            let run_id = format!("game-interface-{id}-bg2");
            assert!(baseline.plan.components_for(&run_id).is_none());
            assert!(positions("eet-initialize-bg2") < positions("eeex-bg2"));
            assert!(positions("eeex-bg2") < positions(&run_id));
            assert!(positions(&run_id) < positions("chriz-bg-modpack-ui-compat-bg2"));
            assert!(positions("chriz-bg-modpack-ui-compat-bg2") < positions("bubb-spell-menu-bg2"));
            assert!(positions("bubb-spell-menu-bg2") < positions("stratagems-bg2"));
            assert!(positions("eet-end-bg2") < positions("hiddengameplayoptions-bg2"));
            assert!(positions("hiddengameplayoptions-bg2") < positions("buffbot-bg2"));
        }
        assert_eq!(
            manifest.mods["eet-gui"].artifact_id,
            manifest.mods["eet"].artifact_id
        );
        assert!(manifest.artifacts[&manifest.mods["eet"].artifact_id]
            .archive
            .publish_roots
            .contains(&"EET_gui".to_owned()));
    }
}

#[test]
fn infinity_omits_only_specific_controls_and_restores_requested_preferences() {
    for manifest in recipes() {
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let mut selection = baseline.normalized_selection.to_selection();
        selection.set_feature("feature:game-interface:infinity", true);
        let result = evaluate(&manifest, &selection).unwrap();
        assert_eq!(
            result.plan.components_for("game-interface-infinity-bg2"),
            Some(&[0][..])
        );
        for component in [29, 30, 300, 301] {
            let id = format!("feature:hiddengameplayoptions:component-{component}");
            let control = result.view.control(&id).unwrap();
            assert!(!control.selected);
            assert!(control
                .unavailable_reason
                .as_ref()
                .unwrap()
                .contains("Hidden Gameplay Options"));
        }
        assert!(!result
            .plan
            .components_for("stratagems-bg2")
            .unwrap()
            .contains(&4115));
        for component in [10, 28, 39, 103] {
            assert!(result
                .plan
                .components_for("hiddengameplayoptions-bg2")
                .unwrap()
                .contains(&component));
        }
        assert_eq!(
            result.plan.components_for("eeex-bg2"),
            baseline.plan.components_for("eeex-bg2")
        );
        assert_eq!(
            result.plan.components_for("bubb-spell-menu-bg2"),
            Some(&[0][..])
        );
        assert_eq!(result.plan.components_for("buffbot-bg2"), Some(&[1, 0][..]));
        let mut restored = result.normalized_selection.to_selection();
        restored.set_feature("feature:game-interface:infinity", false);
        assert_eq!(evaluate(&manifest, &restored).unwrap().plan, baseline.plan);

        selection.set_feature("mod:hidden-gameplay-options", false);
        let no_hgo = evaluate(&manifest, &selection).unwrap();
        assert_eq!(
            no_hgo.plan.components_for("game-interface-infinity-bg2"),
            Some(&[0][..])
        );
        assert!(no_hgo
            .plan
            .components_for("hiddengameplayoptions-bg2")
            .is_none());
    }
}

#[test]
fn eet_sod_style_adds_only_its_ui_run() {
    for manifest in recipes() {
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let mut selection = baseline.normalized_selection.to_selection();
        selection.set_feature("feature:game-interface:eet-sod", true);
        let mut selected = evaluate(&manifest, &selection).unwrap().plan;
        assert_eq!(
            selected.components_for("game-interface-eet-sod-bg2"),
            Some(&[0][..])
        );
        selected
            .runs
            .retain(|run| run.run_id != "game-interface-eet-sod-bg2");
        assert_eq!(selected, baseline.plan);
    }
}

#[test]
fn classic_choices_always_include_the_released_helper_without_optional_dependencies() {
    for manifest in recipes() {
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let helper = "feature:chriz-bg-modpack:component-640";
        let helper_run = "chriz-bg-modpack-ui-compat-bg2";
        assert!(baseline.plan.components_for(helper_run).is_none());
        for id in ["dragonspear", "leui-bg2", "leui-bg1", "leui-sod"] {
            let feature_id = format!("feature:game-interface:{id}");
            let control = baseline.view.control(&feature_id).unwrap();
            assert_eq!(control.readiness, Readiness::Experimental);
            let mut selection = baseline.normalized_selection.to_selection();
            selection.set_feature(&feature_id, true);
            selection.set_feature(helper, false);
            selection.set_feature("mod:hidden-gameplay-options", false);
            selection.set_feature("mod:chriz-bg-modpack", false);
            selection.set_feature("mod:eeex", false);
            let selected = evaluate(&manifest, &selection).unwrap();
            assert!(selected.view.control(&feature_id).unwrap().selected);
            assert_eq!(selected.plan.components_for(helper_run), Some(&[640][..]));
            assert!(selected.view.control(helper).unwrap().selected);
            assert!(selected
                .plan
                .components_for("hiddengameplayoptions-bg2")
                .is_none());
            assert!(selected
                .plan
                .components_for("chriz-bg-modpack-bg2")
                .is_none());
            assert!(selected.plan.components_for("eeex-bg2").is_none());
        }
        for id in ["infinity", "eet-sod"] {
            let mut selection = baseline.normalized_selection.to_selection();
            selection.set_feature(&format!("feature:game-interface:{id}"), true);
            assert!(evaluate(&manifest, &selection)
                .unwrap()
                .plan
                .components_for(helper_run)
                .is_none());
        }
    }
}
