use std::path::PathBuf;

use bg_engine::manifest::{Decision, Phase, Readiness};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const HGO_AUTHORED: &[u32] = &[
    10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 22, 23, 24, 25, 27, 28, 29, 30, 32, 33, 34, 35, 36, 37,
    38, 39, 40, 200, 300, 301, 302, 304, 101, 102, 103,
];
const HGO_RECOMMENDED: &[u32] = &[
    10, 11, 12, 13, 14, 16, 18, 19, 20, 22, 23, 24, 25, 27, 28, 29, 30, 32, 33, 34, 35, 36, 37, 39,
    300, 301, 103,
];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn installs_hgo_late_with_the_reviewed_optional_defaults() {
    let manifest = recipe();
    let runs = &manifest.collection.runs;
    let position = |id: &str| {
        runs.iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing run {id}"))
    };
    assert!(position("eet-end-bg2") < position("hiddengameplayoptions-bg2"));
    assert!(position("chriz-sod-remix-bg2") < position("hiddengameplayoptions-bg2"));
    assert!(position("hiddengameplayoptions-bg2") < position("buffbot-bg2"));

    let run = &runs[position("hiddengameplayoptions-bg2")];
    assert_eq!(run.phase, Phase::PostEetEnd);
    assert_eq!(run.components, HGO_AUTHORED);
    assert!(manifest.mods["hidden-gameplay-options"]
        .components
        .iter()
        .all(|component| component.prompts.is_empty()));

    let parent = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "mod:hidden-gameplay-options")
        .expect("HGO parent control");
    assert_eq!(parent.decision, Decision::Optional);
    assert_eq!(parent.readiness, Readiness::Ready);

    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        evaluation.plan.components_for("hiddengameplayoptions-bg2"),
        Some(HGO_RECOMMENDED)
    );

    let defaults = HGO_RECOMMENDED.to_vec();
    for component in HGO_AUTHORED {
        let id = format!("feature:hiddengameplayoptions:component-{component}");
        let control = evaluation
            .view
            .control(&id)
            .unwrap_or_else(|| panic!("missing HGO control {id}"));
        assert_eq!(
            control.decision,
            if defaults.contains(component) {
                Decision::Default
            } else {
                Decision::Optional
            },
            "{id}"
        );
        if [38, 40].contains(component) {
            assert_eq!(control.readiness, Readiness::Blocked, "{id}");
            assert!(!control.interactive, "{id}");
        } else {
            assert_eq!(control.readiness, Readiness::Ready, "{id}");
        }
    }
}

#[test]
fn hgo_can_be_disabled_or_reconfigured_without_bypassing_exclusivity() {
    let manifest = recipe();
    let defaults = Selection::defaults("windows");
    assert_eq!(
        evaluate(&manifest, &defaults)
            .unwrap()
            .plan
            .components_for("hiddengameplayoptions-bg2"),
        None
    );

    let mut enabled = defaults.clone();
    enabled.set_feature("mod:hidden-gameplay-options", true);
    assert_eq!(
        evaluate(&manifest, &enabled)
            .unwrap()
            .plan
            .components_for("hiddengameplayoptions-bg2"),
        Some(HGO_RECOMMENDED)
    );

    enabled.set_feature("feature:hiddengameplayoptions:component-103", false);
    enabled.set_feature("feature:hiddengameplayoptions:component-101", true);
    enabled.set_feature("feature:hiddengameplayoptions:component-200", true);
    let customized = evaluate(&manifest, &enabled).unwrap();
    let actual = customized
        .plan
        .components_for("hiddengameplayoptions-bg2")
        .unwrap();
    assert_eq!(
        actual,
        &[
            10, 11, 12, 13, 14, 16, 18, 19, 20, 22, 23, 24, 25, 27, 28, 29, 30, 32, 33, 34, 35, 36,
            37, 39, 200, 300, 301, 101,
        ]
    );

    let key_binding_ids = [101, 102, 103]
        .map(|component| format!("feature:hiddengameplayoptions:component-{component}"));
    for id in &key_binding_ids {
        let feature = manifest
            .collection
            .features
            .iter()
            .find(|feature| feature.id == *id)
            .unwrap();
        assert_eq!(feature.conflicts.len(), 2, "{id}");
        assert!(feature
            .conflicts
            .iter()
            .all(|conflict| key_binding_ids.contains(&conflict.feature_id)));
    }
}
