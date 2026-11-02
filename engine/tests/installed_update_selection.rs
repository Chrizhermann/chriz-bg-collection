use std::path::PathBuf;

use bg_engine::installed_updates::preserve_installed_selection;
use bg_engine::manifest::{
    ComponentRef, Conflict, Decision, Feature, InputOption, InputSpec, InputValue, Readiness,
};
use bg_engine::recipe_view::{evaluate, NormalizedSelection};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

fn feature(id: &str, decision: Decision) -> Feature {
    Feature {
        id: id.into(),
        title: id.into(),
        description: format!("Description for {id}"),
        category: "settings".into(),
        source_label: None,
        group_label: None,
        choice_group: None,
        decision,
        readiness: Readiness::Ready,
        unavailable_reason: None,
        parent: None,
        components: vec![],
        requires: vec![],
        requires_any: vec![],
        conflicts: vec![],
        inputs: vec![],
    }
}

fn fixture() -> Manifest {
    let mut manifest =
        Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
            .unwrap();
    let mut sr = feature("sr", Decision::Default);
    sr.components.push(ComponentRef {
        run_id: "eefixpack-bg1".into(),
        component: 0,
    });
    let mut child = feature("sr-child", Decision::Mandatory);
    child.parent = Some("sr".into());
    child.components.push(ComponentRef {
        run_id: "eefixpack-bg1".into(),
        component: 2,
    });
    let mut standard = feature("standard", Decision::Default);
    standard.conflicts.push(Conflict {
        feature_id: "alternate".into(),
        reason: "Choose one alternative.".into(),
    });
    let mut alternate = feature("alternate", Decision::Optional);
    alternate.conflicts.push(Conflict {
        feature_id: "standard".into(),
        reason: "Choose one alternative.".into(),
    });
    let mut settings = feature("settings", Decision::Default);
    settings.inputs = vec![
        InputSpec::Boolean {
            id: "enabled".into(),
            default: true,
        },
        InputSpec::Integer {
            id: "amount".into(),
            default: 10,
            min: 1,
            max: 100,
        },
        InputSpec::Choice {
            id: "mode".into(),
            default: "normal".into(),
            options: vec![
                InputOption {
                    id: "normal".into(),
                    title: "Normal".into(),
                    answer: "1".into(),
                },
                InputOption {
                    id: "custom".into(),
                    title: "Custom".into(),
                    answer: "2".into(),
                },
            ],
        },
    ];
    manifest.collection.features = vec![sr, child, standard, alternate, settings];
    manifest
}

fn installed(manifest: &Manifest) -> NormalizedSelection {
    evaluate(manifest, &Selection::defaults("windows"))
        .unwrap()
        .normalized_selection
}

fn mutate<'a>(manifest: &'a mut Manifest, id: &str) -> &'a mut Feature {
    manifest
        .collection
        .features
        .iter_mut()
        .find(|feature| feature.id == id)
        .unwrap()
}

fn assert_issue(old: &Manifest, target: &Manifest, selection: &NormalizedSelection, rule: &str) {
    let issues = preserve_installed_selection(old, target, selection).unwrap_err();
    assert!(
        issues.iter().any(|issue| issue.rule == rule),
        "expected {rule}, got {issues:?}"
    );
}

#[test]
fn new_defaults_and_newly_available_excluded_features_stay_off() {
    let mut old = fixture();
    old.collection
        .features
        .push(feature("previously-excluded", Decision::Excluded));
    let selection = installed(&old);
    let mut target = old.clone();
    target
        .collection
        .features
        .push(feature("new-default", Decision::Default));
    target
        .collection
        .features
        .push(feature("new-optional", Decision::Optional));
    mutate(&mut target, "previously-excluded").decision = Decision::Default;
    mutate(&mut target, "alternate").decision = Decision::Default;
    mutate(&mut target, "standard").decision = Decision::Optional;
    let result = preserve_installed_selection(&old, &target, &selection).unwrap();
    for id in [
        "new-default",
        "new-optional",
        "previously-excluded",
        "alternate",
    ] {
        assert!(!result.normalized_selection.features[id]);
        assert!(!result.view.control(id).unwrap().selected);
    }
    assert!(result.view.control("standard").unwrap().selected);
}

#[test]
fn sr_off_retains_parent_gated_mandatory_desire_without_enabling_it() {
    let old = fixture();
    let mut choices = Selection::defaults("windows");
    choices.set_feature("sr", false);
    let selection = evaluate(&old, &choices).unwrap().normalized_selection;
    let result = preserve_installed_selection(&old, &old, &selection).unwrap();
    assert_eq!(result.normalized_selection, selection);
    assert!(result.normalized_selection.features["sr-child"]);
    assert!(!result.view.control("sr-child").unwrap().selected);
    assert!(result.plan.components_for("eefixpack-bg1").is_none());
}

#[test]
fn explicit_alternative_and_exclusion_survive_target_defaults() {
    let old = fixture();
    let mut choices = Selection::defaults("windows");
    choices.set_feature("standard", false);
    choices.set_feature("alternate", true);
    let selection = evaluate(&old, &choices).unwrap().normalized_selection;
    let mut target = old.clone();
    mutate(&mut target, "alternate").decision = Decision::Default;
    mutate(&mut target, "standard").decision = Decision::Optional;
    let result = preserve_installed_selection(&old, &target, &selection).unwrap();
    assert_eq!(result.normalized_selection, selection);
    assert!(result.view.control("alternate").unwrap().selected);
    assert!(!result.view.control("standard").unwrap().selected);
}

#[test]
fn typed_values_survive_defaults_labels_and_valid_bound_changes() {
    let old = fixture();
    let mut choices = Selection::defaults("windows");
    choices.set_input("settings", "enabled", InputValue::Boolean(false));
    choices.set_input("settings", "amount", InputValue::Integer(37));
    choices.set_input("settings", "mode", InputValue::Choice("custom".into()));
    let selection = evaluate(&old, &choices).unwrap().normalized_selection;
    let mut target = old.clone();
    let settings = mutate(&mut target, "settings");
    settings.title = "New label".into();
    settings.description = "New explanatory text".into();
    for input in &mut settings.inputs {
        match input {
            InputSpec::Boolean { default, .. } => *default = false,
            InputSpec::Integer {
                default, min, max, ..
            } => {
                *default = 5;
                *min = 5;
                *max = 50;
            }
            InputSpec::Choice {
                default, options, ..
            } => {
                *default = "custom".into();
                options[1].title = "Updated label".into();
                options[0].answer = "unchosen-answer-change".into();
                options.reverse();
            }
        }
    }
    let result = preserve_installed_selection(&old, &target, &selection).unwrap();
    assert_eq!(result.normalized_selection, selection);
}

#[test]
fn new_mandatory_and_previously_deselected_mandatory_features_require_migration() {
    let old = fixture();
    let selection = installed(&old);
    let mut target = old.clone();
    let mut new_mandatory = feature("new-mandatory", Decision::Mandatory);
    new_mandatory.parent = Some("alternate".into());
    target.collection.features.push(new_mandatory);
    assert_issue(&old, &target, &selection, "new-mandatory-feature");
    target = old.clone();
    mutate(&mut target, "alternate").decision = Decision::Mandatory;
    assert_issue(&old, &target, &selection, "newly-mandatory-feature");
}

#[test]
fn removed_or_now_excluded_ids_require_migration_even_when_deselected() {
    let old = fixture();
    let mut choices = Selection::defaults("windows");
    choices.set_feature("settings", false);
    let selection = evaluate(&old, &choices).unwrap().normalized_selection;
    let mut target = old.clone();
    target
        .collection
        .features
        .retain(|feature| feature.id != "settings");
    assert_issue(&old, &target, &selection, "feature-removed");
    target = old.clone();
    mutate(&mut target, "settings").decision = Decision::Excluded;
    assert_issue(&old, &target, &selection, "feature-excluded");
}

#[test]
fn incomplete_or_invalid_installed_values_are_not_rebuilt_from_old_defaults() {
    let old = fixture();
    let selection = installed(&old);
    let mut incomplete = selection.clone();
    incomplete.features.remove("alternate");
    assert_issue(&old, &old, &incomplete, "installed-selection-incomplete");
    incomplete = selection.clone();
    incomplete
        .inputs
        .get_mut("settings")
        .unwrap()
        .remove("mode");
    assert_issue(&old, &old, &incomplete, "installed-selection-incomplete");
    incomplete = selection.clone();
    incomplete.inputs.remove("sr");
    assert_issue(&old, &old, &incomplete, "installed-selection-incomplete");
    let mut invalid = selection.clone();
    invalid
        .inputs
        .get_mut("settings")
        .unwrap()
        .insert("amount".into(), InputValue::Integer(0));
    assert_issue(&old, &old, &invalid, "installed-selection-invalid");
    invalid = selection;
    invalid.features.insert("sr-child".into(), false);
    assert_issue(&old, &old, &invalid, "installed-selection-incomplete");
}

#[test]
fn changed_input_type_missing_input_and_invalid_retained_value_require_migration() {
    let old = fixture();
    let selection = installed(&old);
    let mut target = old.clone();
    mutate(&mut target, "settings").inputs[0] = InputSpec::Integer {
        id: "enabled".into(),
        default: 1,
        min: 0,
        max: 1,
    };
    assert_issue(&old, &target, &selection, "input-schema-changed");
    target = old.clone();
    mutate(&mut target, "settings").inputs.remove(0);
    assert_issue(&old, &target, &selection, "input-schema-changed");
    target = old.clone();
    mutate(&mut target, "settings")
        .inputs
        .push(InputSpec::Boolean {
            id: "new-input".into(),
            default: true,
        });
    assert_issue(&old, &target, &selection, "input-schema-changed");
    target = old.clone();
    mutate(&mut target, "settings").inputs[1] = InputSpec::Integer {
        id: "amount".into(),
        default: 20,
        min: 20,
        max: 100,
    };
    assert_issue(&old, &target, &selection, "input-value-invalid");
}

#[test]
fn changed_answer_or_removed_choice_requires_migration_even_with_the_same_option_id() {
    let old = fixture();
    let selection = installed(&old);
    let mut target = old.clone();
    if let InputSpec::Choice { options, .. } = &mut mutate(&mut target, "settings").inputs[2] {
        options[0].answer = "different-operation".into();
    }
    assert_issue(&old, &target, &selection, "input-meaning-changed");
    if let InputSpec::Choice {
        default, options, ..
    } = &mut mutate(&mut target, "settings").inputs[2]
    {
        options.remove(0);
        *default = "custom".into();
    }
    assert_issue(&old, &target, &selection, "input-value-invalid");
}

#[test]
fn conflict_removal_must_not_activate_previously_suppressed_defaults() {
    let mut old = fixture();
    mutate(&mut old, "sr").decision = Decision::Mandatory;
    mutate(&mut old, "settings").conflicts.push(Conflict {
        feature_id: "sr".into(),
        reason: "Unavailable with SR.".into(),
    });
    let selection = installed(&old);
    assert!(selection.features["settings"]);
    let mut target = old.clone();
    mutate(&mut target, "settings").conflicts.clear();
    assert_issue(&old, &target, &selection, "effective-selection-changed");
}

#[test]
fn conflicts_and_dependency_changes_cannot_silently_drop_installed_features() {
    let mut old = fixture();
    mutate(&mut old, "sr").decision = Decision::Mandatory;
    let selection = installed(&old);
    let mut target = old.clone();
    mutate(&mut target, "settings").conflicts.push(Conflict {
        feature_id: "sr".into(),
        reason: "New incompatibility.".into(),
    });
    assert_issue(&old, &target, &selection, "effective-selection-changed");
    target = old.clone();
    mutate(&mut target, "settings")
        .requires
        .push("alternate".into());
    assert_issue(&old, &target, &selection, "new-feature-dependency");
    target = old.clone();
    mutate(&mut target, "settings").requires.push("sr".into());
    assert_issue(&old, &target, &selection, "new-feature-dependency");
}

#[test]
fn migration_issue_uses_camel_case_field_names() {
    let old = fixture();
    let mut target = old.clone();
    mutate(&mut target, "settings").decision = Decision::Excluded;
    let issues = preserve_installed_selection(&old, &target, &installed(&old)).unwrap_err();
    let value = serde_json::to_value(&issues[0]).unwrap();
    assert_eq!(value["featureId"], "settings");
    assert!(value.get("feature_id").is_none());
}
