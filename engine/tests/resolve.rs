use std::collections::BTreeMap;
use std::path::PathBuf;

use bg_engine::error::{EngineError, Result};
use bg_engine::manifest::{
    ChoiceGroup, ChoiceOption, Collection, Component, ComponentRef, ModFile, OrderEntry, Phase,
    Source, SourceKind, Toggle,
};
use bg_engine::resolve::{resolve, InstallPlan, Selection};
use bg_engine::Manifest;

fn component(id: u32) -> Component {
    Component {
        id,
        name: format!("Component {id}"),
        stdin: None,
    }
}

fn component_ref(mod_id: &str, component: u32) -> ComponentRef {
    ComponentRef {
        mod_id: mod_id.to_owned(),
        component,
    }
}

fn mod_file(id: &str, phase: Phase, platforms: &[&str], component_ids: &[u32]) -> ModFile {
    ModFile {
        id: id.to_owned(),
        name: format!("{id} mod"),
        version: "1.0".to_owned(),
        tp2: format!("{id}/{id}.tp2"),
        language: 0,
        weidu: "249.00".to_owned(),
        platforms: platforms
            .iter()
            .map(|platform| (*platform).to_owned())
            .collect(),
        phase,
        source: Source {
            kind: SourceKind::Manual,
            url: "https://example.invalid/mod".to_owned(),
            sha256: "0".repeat(64),
        },
        components: component_ids.iter().copied().map(component).collect(),
    }
}

fn order_entry(id: &str, components: Option<Vec<u32>>) -> OrderEntry {
    OrderEntry {
        id: id.to_owned(),
        components,
    }
}

fn test_manifest() -> Manifest {
    let mods: BTreeMap<_, _> = [
        (
            "core".to_owned(),
            mod_file(
                "core",
                Phase::Bg1PreMerge,
                &["windows", "macos", "linux"],
                &[0, 10, 20, 30, 40],
            ),
        ),
        (
            "addon".to_owned(),
            mod_file(
                "addon",
                Phase::Main,
                &["windows", "macos", "linux"],
                &[0, 10],
            ),
        ),
        (
            "windows-only".to_owned(),
            mod_file("windows-only", Phase::PostEetEnd, &["windows"], &[0]),
        ),
    ]
    .into_iter()
    .collect();

    Manifest {
        root: PathBuf::from("in-memory-manifest"),
        collection: Collection {
            schema: 1,
            game_build: "2.7.3.0".to_owned(),
            order: vec![
                order_entry("core", None),
                order_entry("addon", None),
                order_entry("windows-only", None),
            ],
            toggles: vec![
                Toggle {
                    id: "addon".to_owned(),
                    name: "Addon".to_owned(),
                    default_on: true,
                    removes_mods: vec!["addon".to_owned()],
                    removes_components: vec![component_ref("core", 10)],
                },
                Toggle {
                    id: "optional-core".to_owned(),
                    name: "Optional core component".to_owned(),
                    default_on: false,
                    removes_mods: Vec::new(),
                    removes_components: vec![component_ref("core", 40)],
                },
            ],
            choice_groups: vec![ChoiceGroup {
                id: "flavor".to_owned(),
                name: "Flavor".to_owned(),
                default: "classic".to_owned(),
                options: vec![
                    ChoiceOption {
                        id: "classic".to_owned(),
                        name: "Classic".to_owned(),
                        adds_components: vec![component_ref("core", 20)],
                        removes_components: vec![component_ref("core", 30)],
                    },
                    ChoiceOption {
                        id: "modern".to_owned(),
                        name: "Modern".to_owned(),
                        adds_components: vec![component_ref("core", 30)],
                        removes_components: vec![component_ref("core", 20)],
                    },
                ],
            }],
        },
        mods,
    }
}

fn run_summary(plan: &InstallPlan) -> Vec<(&str, Phase, Vec<u32>)> {
    plan.runs
        .iter()
        .map(|run| {
            (
                run.mod_id.as_str(),
                run.phase,
                run.components
                    .iter()
                    .map(|component| component.id)
                    .collect(),
            )
        })
        .collect()
}

fn invalid_selection_message(result: Result<InstallPlan>) -> String {
    match result {
        Err(EngineError::InvalidSelection(message)) => message,
        Err(other) => panic!("expected invalid selection, got {other:?}"),
        Ok(plan) => panic!("expected invalid selection, got plan {plan:?}"),
    }
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest")
}

#[test]
fn default_selection_includes_everything_on_by_default() {
    let plan = resolve(&test_manifest(), &Selection::defaults("windows")).unwrap();

    assert_eq!(
        run_summary(&plan),
        vec![
            ("core", Phase::Bg1PreMerge, vec![0, 10, 20]),
            ("addon", Phase::Main, vec![0, 10]),
            ("windows-only", Phase::PostEetEnd, vec![0]),
        ]
    );
}

#[test]
fn toggle_off_removes_mod_and_knock_ons() {
    let mut selection = Selection::defaults("windows");
    selection.toggles_off.push("addon".to_owned());

    let plan = resolve(&test_manifest(), &selection).unwrap();

    assert_eq!(
        run_summary(&plan),
        vec![
            ("core", Phase::Bg1PreMerge, vec![0, 20]),
            ("windows-only", Phase::PostEetEnd, vec![0]),
        ]
    );
}

#[test]
fn default_off_toggle_can_be_turned_on() {
    let mut selection = Selection::defaults("windows");
    selection.toggles_on.push("optional-core".to_owned());

    let plan = resolve(&test_manifest(), &selection).unwrap();

    assert_eq!(
        plan.runs[0]
            .components
            .iter()
            .map(|component| component.id)
            .collect::<Vec<_>>(),
        vec![0, 10, 20, 40]
    );
}

#[test]
fn choice_group_swaps_components() {
    let mut selection = Selection::defaults("windows");
    selection
        .choices
        .insert("flavor".to_owned(), "modern".to_owned());

    let plan = resolve(&test_manifest(), &selection).unwrap();

    assert_eq!(
        plan.runs[0]
            .components
            .iter()
            .map(|component| component.id)
            .collect::<Vec<_>>(),
        vec![0, 10, 30]
    );
}

#[test]
fn option_added_components_are_excluded_from_baseline() {
    let manifest = Manifest::load(&fixture_dir()).unwrap();

    let plan = resolve(&manifest, &Selection::defaults("windows")).unwrap();

    assert_eq!(
        run_summary(&plan),
        vec![
            ("eet", Phase::Main, vec![0, 100]),
            ("testmod", Phase::Main, vec![0]),
        ]
    );
}

#[test]
fn plan_preserves_manifest_order_and_groups_by_mod_run() {
    let mut manifest = test_manifest();
    manifest.collection.order = vec![
        order_entry("core", Some(vec![10, 0])),
        order_entry("addon", Some(vec![10, 0])),
        order_entry("core", Some(vec![40, 20])),
    ];
    manifest.collection.toggles.clear();
    manifest.collection.choice_groups.clear();

    let plan = resolve(&manifest, &Selection::defaults("linux")).unwrap();

    assert_eq!(
        run_summary(&plan),
        vec![
            ("core", Phase::Bg1PreMerge, vec![0, 10]),
            ("addon", Phase::Main, vec![0, 10]),
            ("core", Phase::Bg1PreMerge, vec![20, 40]),
        ]
    );
}

#[test]
fn platform_filter_drops_incompatible_mods() {
    let plan = resolve(&test_manifest(), &Selection::defaults("linux")).unwrap();

    assert_eq!(
        run_summary(&plan),
        vec![
            ("core", Phase::Bg1PreMerge, vec![0, 10, 20]),
            ("addon", Phase::Main, vec![0, 10]),
        ]
    );
}

#[test]
fn empty_runs_are_dropped() {
    let mut manifest = test_manifest();
    manifest.collection.order = vec![
        order_entry("core", Some(vec![40])),
        order_entry("addon", Some(vec![0])),
    ];
    manifest.collection.choice_groups.clear();

    let plan = resolve(&manifest, &Selection::defaults("linux")).unwrap();

    assert_eq!(run_summary(&plan), vec![("addon", Phase::Main, vec![0])]);
}

#[test]
fn unknown_toggle_is_invalid_selection() {
    let manifest = test_manifest();
    let mut off = Selection::defaults("windows");
    off.toggles_off.push("missing-off".to_owned());
    let mut on = Selection::defaults("windows");
    on.toggles_on.push("missing-on".to_owned());

    let off_message = invalid_selection_message(resolve(&manifest, &off));
    let on_message = invalid_selection_message(resolve(&manifest, &on));

    assert!(off_message.contains("missing-off"), "{off_message}");
    assert!(on_message.contains("missing-on"), "{on_message}");
}

#[test]
fn unknown_choice_or_option_is_invalid_selection() {
    let manifest = test_manifest();
    let mut unknown_group = Selection::defaults("windows");
    unknown_group
        .choices
        .insert("missing-group".to_owned(), "classic".to_owned());
    let mut unknown_option = Selection::defaults("windows");
    unknown_option
        .choices
        .insert("flavor".to_owned(), "missing-option".to_owned());

    let group_message = invalid_selection_message(resolve(&manifest, &unknown_group));
    let option_message = invalid_selection_message(resolve(&manifest, &unknown_option));

    assert!(group_message.contains("missing-group"), "{group_message}");
    assert!(option_message.contains("flavor"), "{option_message}");
    assert!(
        option_message.contains("missing-option"),
        "{option_message}"
    );
}

#[test]
fn option_targeting_removed_mod_is_invalid_selection() {
    let mut manifest = test_manifest();
    manifest.collection.choice_groups[0]
        .options
        .push(ChoiceOption {
            id: "addon-flavor".to_owned(),
            name: "Addon flavor".to_owned(),
            adds_components: vec![component_ref("addon", 10)],
            removes_components: Vec::new(),
        });
    let mut selection = Selection::defaults("windows");
    selection.toggles_off.push("addon".to_owned());
    selection
        .choices
        .insert("flavor".to_owned(), "addon-flavor".to_owned());

    let message = invalid_selection_message(resolve(&manifest, &selection));

    assert!(message.contains("addon-flavor"), "{message}");
    assert!(message.contains("addon"), "{message}");
}

#[test]
fn resolve_is_deterministic() {
    let manifest = test_manifest();
    let mut selection = Selection::defaults("windows");
    selection.toggles_on.push("optional-core".to_owned());
    selection
        .choices
        .insert("flavor".to_owned(), "modern".to_owned());

    let first = resolve(&manifest, &selection).unwrap();
    let second = resolve(&manifest, &selection).unwrap();

    assert_eq!(first, second);
}

#[test]
fn toggles_on_wins_when_toggle_is_in_both_override_lists() {
    let manifest = test_manifest();
    let mut selection = Selection::defaults("windows");
    selection.toggles_off.push("addon".to_owned());
    selection.toggles_on.push("addon".to_owned());

    let plan = resolve(&manifest, &selection).unwrap();

    assert!(plan.runs.iter().any(|run| run.mod_id == "addon"));
    assert!(plan.runs[0]
        .components
        .iter()
        .any(|component| component.id == 10));
}

#[test]
fn choice_addition_can_restore_an_emptied_split_run() {
    let manifest = Manifest::load(&fixture_dir()).unwrap();
    let mut selection = Selection::defaults("windows");
    selection
        .choices
        .insert("flavor".to_owned(), "spicy".to_owned());

    let plan = resolve(&manifest, &selection).unwrap();

    assert_eq!(
        run_summary(&plan),
        vec![
            ("eet", Phase::Main, vec![0, 100]),
            ("testmod", Phase::Main, vec![10]),
        ]
    );
}

#[test]
fn invalid_platform_is_invalid_selection() {
    let message =
        invalid_selection_message(resolve(&test_manifest(), &Selection::defaults("solaris")));

    assert!(message.contains("solaris"), "{message}");
}
