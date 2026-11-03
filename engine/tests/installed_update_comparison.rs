use bg_engine::{
    installed_updates::{
        compare_installed_recipe, ComparisonState, InstalledRecipeBaseline, InstalledRunState,
    },
    manifest::{ComponentRef, Decision, Feature, Readiness, RunArg},
    recipe_view::evaluate,
    resolve::Selection,
    session::FrozenIdentity,
    Manifest,
};
use std::{collections::BTreeMap, path::PathBuf};

fn fixture() -> Manifest {
    let mut manifest =
        Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
            .unwrap();
    manifest.collection.features = manifest
        .collection
        .runs
        .iter()
        .map(|run| Feature {
            id: run.run_id.clone(),
            title: run.run_id.clone(),
            description: "Test feature".into(),
            category: "test".into(),
            source_label: None,
            group_label: None,
            choice_group: None,
            decision: Decision::Default,
            readiness: Readiness::Ready,
            unavailable_reason: None,
            parent: None,
            components: run
                .components
                .iter()
                .map(|component| ComponentRef {
                    run_id: run.run_id.clone(),
                    component: *component,
                })
                .collect(),
            requires: vec![],
            requires_any: vec![],
            conflicts: vec![],
            inputs: vec![],
        })
        .collect();
    manifest
}

fn compare(
    old: &Manifest,
    target: &Manifest,
    selection: &Selection,
    replacements: &BTreeMap<String, FrozenIdentity>,
) -> bg_engine::installed_updates::InstalledUpdatePlan {
    let evaluation = evaluate(old, selection).unwrap();
    compare_installed_recipe(
        &InstalledRecipeBaseline {
            install_id: "test",
            recipe_version: "0.1.0-alpha.1",
            manifest: old,
            selection: &evaluation.normalized_selection,
            plan: &evaluation.plan,
            recovered_artifacts: replacements,
        },
        target,
        "0.1.0-alpha.2",
    )
}

#[test]
fn identical_selected_content_and_cosmetic_versions_do_not_offer_upgrade() {
    let old = fixture();
    let mut target = old.clone();
    target.artifacts.get_mut("eefixpack").unwrap().version = "Relabeled same bytes".into();
    target.collection.features[0].title = "Clearer title".into();
    let preview = compare(
        &old,
        &target,
        &Selection::defaults("windows"),
        &BTreeMap::new(),
    );
    assert_eq!(preview.state, ComparisonState::ComparisonReady);
    assert!(preview.choices_preserved);
    assert_eq!(preview.runs.len(), 2);
    assert!(preview
        .runs
        .iter()
        .all(|r| r.state == InstalledRunState::Unchanged));
    let json = serde_json::to_value(preview).unwrap();
    assert!(json.get("reviewToken").is_none());
    assert!(json.get("canApply").is_none());
}

#[test]
fn changed_payload_only_compares_selected_runs() {
    let old = fixture();
    let mut target = old.clone();
    target.artifacts.get_mut("eefixpack").unwrap().source.sha256 = "ab".repeat(32);
    let mut selection = Selection::defaults("windows");
    selection.set_feature("eefixpack-bg1", false);
    let preview = compare(&old, &target, &selection, &BTreeMap::new());
    assert_eq!(preview.runs.len(), 1);
    assert_eq!(preview.runs[0].run_id, "eefixpack-bg2");
    assert_eq!(preview.runs[0].state, InstalledRunState::Unassessed);
}

#[test]
fn same_archive_with_changed_arguments_is_not_current() {
    let old = fixture();
    let mut target = old.clone();
    target.collection.runs[1]
        .args
        .push(RunArg::Literal("changed-setting".into()));
    let preview = compare(
        &old,
        &target,
        &Selection::defaults("windows"),
        &BTreeMap::new(),
    );
    assert_eq!(preview.runs[0].state, InstalledRunState::Unchanged);
    assert_eq!(preview.runs[1].state, InstalledRunState::Unassessed);
}

#[test]
fn language_change_requires_explicit_migration() {
    let old = fixture();
    let mut target = old.clone();
    target.mods.get_mut("eefixpack").unwrap().language = 1;
    let preview = compare(
        &old,
        &target,
        &Selection::defaults("windows"),
        &BTreeMap::new(),
    );
    assert!(preview
        .runs
        .iter()
        .all(|r| r.state == InstalledRunState::RequiresMigration));
}

#[test]
fn changed_component_expansion_is_not_silently_installed() {
    let old = fixture();
    let mut target = old.clone();
    target.collection.features[0].components.pop();
    let preview = compare(
        &old,
        &target,
        &Selection::defaults("windows"),
        &BTreeMap::new(),
    );
    assert_eq!(preview.runs[0].state, InstalledRunState::RequiresMigration);
    assert_eq!(preview.runs[0].components, vec![0, 2]);
}

#[test]
fn verified_recovery_payload_counts_as_already_delivered_for_its_run_only() {
    let old = fixture();
    let mut target = old.clone();
    target.artifacts.get_mut("eefixpack").unwrap().source.sha256 = "ab".repeat(32);
    target.artifacts.get_mut("eefixpack").unwrap().version = "repaired".into();
    let replacements = BTreeMap::from([(
        "eefixpack-bg2".into(),
        FrozenIdentity {
            id: "eefixpack".into(),
            version: "repaired".into(),
            sha256: "ab".repeat(32),
            length: 1,
        },
    )]);
    let preview = compare(
        &old,
        &target,
        &Selection::defaults("windows"),
        &replacements,
    );
    assert_eq!(preview.runs[0].state, InstalledRunState::Unassessed);
    assert_eq!(preview.runs[1].state, InstalledRunState::Unchanged);
    assert_eq!(preview.runs[1].installed_version, "repaired");
}

#[test]
fn target_build_change_is_visible_and_blocks_comparison() {
    let old = fixture();
    let mut target = old.clone();
    target.collection.game_build = "2.8.0.0".into();
    let preview = compare(
        &old,
        &target,
        &Selection::defaults("windows"),
        &BTreeMap::new(),
    );
    assert_eq!(preview.state, ComparisonState::Blocked);
    assert_eq!(preview.issues[0].rule, "game-build-changed");
}

#[test]
fn frozen_plan_mismatch_cannot_be_treated_as_an_update() {
    let old = fixture();
    let evaluated = evaluate(&old, &Selection::defaults("windows")).unwrap();
    let mut plan = evaluated.plan.clone();
    plan.runs.pop();
    let result = compare_installed_recipe(
        &InstalledRecipeBaseline {
            install_id: "test",
            recipe_version: "old",
            manifest: &old,
            selection: &evaluated.normalized_selection,
            plan: &plan,
            recovered_artifacts: &BTreeMap::new(),
        },
        &old,
        "target",
    );
    assert_eq!(result.state, ComparisonState::Blocked);
    assert_eq!(result.issues[0].rule, "baseline-plan-mismatch");
}
