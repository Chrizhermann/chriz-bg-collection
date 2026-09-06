use std::path::PathBuf;

use bg_engine::error::{EngineError, Result};
use bg_engine::manifest::{
    AcquisitionPolicy, ComponentRef, Decision, Feature, GameRoot, Phase, Readiness, RunArg,
};
use bg_engine::resolve::{resolve, InstallPlan, Selection};
use bg_engine::Manifest;

fn fixture() -> Manifest {
    let mut manifest =
        Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
            .unwrap();
    manifest.collection.features = vec![Feature {
        id: "eefixpack".to_owned(),
        title: "EE Fixpack".to_owned(),
        description: "Install the curated EE Fixpack components.".to_owned(),
        category: "fixes".to_owned(),
        source_label: None,
        group_label: None,
        choice_group: None,
        decision: Decision::Default,
        readiness: Readiness::Ready,
        unavailable_reason: None,
        parent: None,
        components: vec![
            ComponentRef {
                run_id: "eefixpack-bg1".to_owned(),
                component: 0,
            },
            ComponentRef {
                run_id: "eefixpack-bg1".to_owned(),
                component: 2,
            },
            ComponentRef {
                run_id: "eefixpack-bg2".to_owned(),
                component: 0,
            },
        ],
        requires: Vec::new(),
        conflicts: Vec::new(),
        inputs: Vec::new(),
    }];
    manifest
}

fn invalid_selection_message(result: Result<InstallPlan>) -> String {
    match result {
        Err(EngineError::InvalidSelection(message)) => message,
        Err(other) => panic!("expected invalid selection, got {other:?}"),
        Ok(plan) => panic!("expected invalid selection, got {plan:?}"),
    }
}

#[test]
fn plan_preserves_exact_run_identity_and_component_order() {
    let plan = resolve(&fixture(), &Selection::defaults("windows")).unwrap();

    assert_eq!(plan.runs.len(), 2);
    assert_eq!(plan.runs[0].run_id, "eefixpack-bg1");
    assert_eq!(plan.runs[0].mod_id, "eefixpack");
    assert_eq!(plan.runs[0].target, GameRoot::Bg1);
    assert_eq!(plan.runs[0].phase, Phase::Bg1Preparation);
    assert_eq!(plan.runs[0].components, vec![0, 2]);
    assert_eq!(plan.runs[0].args, vec![RunArg::StagedRoot(GameRoot::Bg1)]);
    assert_eq!(plan.runs[0].artifact_id, "eefixpack");
    assert_eq!(plan.runs[0].weidu_artifact_id, "weidu");

    assert_eq!(plan.runs[1].run_id, "eefixpack-bg2");
    assert_eq!(plan.runs[1].target, GameRoot::Bg2);
    assert_eq!(plan.runs[1].components, vec![0]);
}

#[test]
fn shared_artifact_still_produces_two_distinct_runs() {
    let plan = resolve(&fixture(), &Selection::defaults("windows")).unwrap();

    assert_ne!(plan.runs[0].run_id, plan.runs[1].run_id);
    assert_eq!(plan.runs[0].artifact_id, plan.runs[1].artifact_id);
}

#[test]
fn windows_resolution_never_silently_filters_declared_runs() {
    let manifest = fixture();
    let plan = resolve(&manifest, &Selection::defaults("windows")).unwrap();

    assert_eq!(plan.runs.len(), manifest.collection.runs.len());
}

#[test]
fn unsupported_platform_is_an_error_instead_of_a_filter() {
    let message = invalid_selection_message(resolve(&fixture(), &Selection::defaults("linux")));

    assert!(message.contains("linux"), "{message}");
    assert!(message.contains("windows"), "{message}");
}

#[test]
fn undeclared_semantic_choice_is_rejected() {
    let mut selection = Selection::defaults("windows");
    selection
        .choices
        .insert("future-choice".to_owned(), "on".to_owned());

    let message = invalid_selection_message(resolve(&fixture(), &selection));

    assert!(message.contains("future-choice"), "{message}");
}

#[test]
fn resolution_starts_empty_and_adds_only_effective_features() {
    let manifest = fixture();
    let mut selection = Selection::defaults("windows");
    selection.set_feature("eefixpack", false);

    let plan = resolve(&manifest, &selection).unwrap();

    assert!(plan.runs.is_empty());
}

#[test]
fn invalid_recipe_is_rejected_before_plan_materialization() {
    let mut manifest = fixture();
    manifest.collection.runs[0].components.clear();

    let error = resolve(&manifest, &Selection::defaults("windows")).unwrap_err();

    let EngineError::Validation(findings) = error else {
        panic!("expected manifest validation error");
    };
    assert!(findings
        .iter()
        .any(|finding| finding.rule == "run-components"));
}

#[test]
fn blocked_artifact_cannot_become_a_planned_run() {
    let mut manifest = fixture();
    manifest.artifacts.get_mut("eefixpack").unwrap().acquisition = AcquisitionPolicy::Blocked;

    let error = resolve(&manifest, &Selection::defaults("windows")).unwrap_err();

    let EngineError::Validation(findings) = error else {
        panic!("expected manifest validation error");
    };
    assert!(findings
        .iter()
        .any(|finding| finding.rule == "blocked-artifacts"));
}

#[test]
fn resolution_is_deterministic() {
    let manifest = fixture();
    let selection = Selection::defaults("windows");

    assert_eq!(
        resolve(&manifest, &selection).unwrap(),
        resolve(&manifest, &selection).unwrap()
    );
}
