use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bg_engine::manifest::{
    Component, ComponentRef, Conflict, Decision, Feature, FeatureInputRef, InputOption, InputSpec,
    InputValue, InvocationMode, ModFile, Phase, Postcondition, PromptAnswer, PromptStep, Readiness,
    Run,
};
use bg_engine::manual_checkpoint::{split_plan, ManualBreakpoint};
use bg_engine::recipe_view::{
    evaluate, evaluate_preset, plan_without_artifacts, render_prompt_script, LeaveOutError,
};
use bg_engine::resolve::Selection;
use bg_engine::validate::check;
use bg_engine::Manifest;

const SOD_COMPONENTS: &[u32] = &[
    100, 110, 120, 130, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 197, 187, 200, 210, 215,
    220, 225, 245, 230, 240, 250, 255, 260, 270, 280, 900,
];
const TEMPUS_COMPONENTS: &[u32] = &[400, 401, 404, 405, 407, 408];

fn fixture() -> Manifest {
    Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
        .unwrap()
}

fn component(id: u32) -> Component {
    Component {
        id,
        name: format!("Component {id}"),
        stdin: None,
        prompts: Vec::new(),
        config_files: Vec::new(),
    }
}

fn add_run(manifest: &mut Manifest, run_id: &str, components: &[u32]) {
    let mut installer = manifest.mods["eefixpack"].clone();
    installer.id = run_id.to_owned();
    installer.name = run_id.to_owned();
    installer.tp2 = format!("{run_id}/{run_id}.tp2");
    installer.invocation_mode = InvocationMode::ExplicitTp2;
    installer.components = components.iter().copied().map(component).collect();
    manifest.mods.insert(run_id.to_owned(), installer);
    let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
    artifact.archive.publish_roots.push(run_id.to_owned());
    artifact
        .archive
        .tp2_paths
        .push(format!("{run_id}/{run_id}.tp2"));
    manifest.collection.runs.push(Run {
        run_id: run_id.to_owned(),
        mod_id: run_id.to_owned(),
        phase: Phase::Bg2Preparation,
        components: components.to_vec(),
        args: Vec::new(),
        postconditions: Vec::new(),
    });
}

fn feature(id: &str, decision: Decision, components: Vec<ComponentRef>) -> Feature {
    Feature {
        id: id.to_owned(),
        title: id.to_owned(),
        description: format!("Description for {id}"),
        category: "rules-abilities".to_owned(),
        source_label: None,
        group_label: None,
        choice_group: None,
        decision,
        readiness: Readiness::Ready,
        unavailable_reason: None,
        parent: None,
        components,
        requires: Vec::new(),
        requires_any: Vec::new(),
        conflicts: Vec::new(),
        inputs: Vec::new(),
    }
}

fn component_refs(run_id: &str, components: &[u32]) -> Vec<ComponentRef> {
    components
        .iter()
        .map(|component| ComponentRef {
            run_id: run_id.to_owned(),
            component: *component,
        })
        .collect()
}

fn grouped_manifest() -> Manifest {
    let mut manifest = fixture();
    manifest.collection.runs.clear();
    add_run(&mut manifest, "stacking", &[1, 2]);
    let mut options = vec![
        feature(
            "unlimited",
            Decision::Default,
            component_refs("stacking", &[1]),
        ),
        feature(
            "forty",
            Decision::Optional,
            component_refs("stacking", &[2]),
        ),
    ];
    for (index, option) in options.iter_mut().enumerate() {
        option.source_label = Some("Tweaks Anthology".to_owned());
        option.group_label = Some("Potion stacking".to_owned());
        option.choice_group = Some("potion-stacking".to_owned());
        option.conflicts.push(Conflict {
            feature_id: if index == 0 { "forty" } else { "unlimited" }.to_owned(),
            reason: "Choose one stack size.".to_owned(),
        });
    }
    manifest.collection.features = options;
    manifest
}

#[test]
fn any_requirement_uses_either_encounter_and_respects_sr_and_cascades() {
    let mut manifest = fixture();
    manifest.collection.runs.clear();
    add_run(&mut manifest, "compat", &[0]);
    let mut compat_json = serde_json::to_value(feature(
        "compat",
        Decision::Mandatory,
        component_refs("compat", &[0]),
    ))
    .unwrap();
    compat_json["requires"] = serde_json::json!(["sr"]);
    compat_json["requires_any"] = serde_json::json!(["rr11", "rr12"]);
    let compat: Feature = serde_json::from_value(compat_json).unwrap();
    let mut child = feature("dependent", Decision::Default, vec![]);
    child.requires.push("compat".to_owned());
    manifest.collection.features = vec![
        feature("sr", Decision::Default, vec![]),
        feature("rr11", Decision::Default, vec![]),
        feature("rr12", Decision::Default, vec![]),
        compat,
        child,
    ];
    check(&manifest).unwrap();
    for (sr, rr11, rr12, expected) in [
        (true, true, true, true),
        (true, true, false, true),
        (true, false, true, true),
        (true, false, false, false),
        (false, true, true, false),
    ] {
        let mut selection = Selection::defaults("windows");
        selection.set_feature("sr", sr);
        selection.set_feature("rr11", rr11);
        selection.set_feature("rr12", rr12);
        let evaluation = evaluate(&manifest, &selection).unwrap();
        assert_eq!(evaluation.plan.components_for("compat").is_some(), expected);
        assert_eq!(
            evaluation.view.control("compat").unwrap().selected,
            expected
        );
        assert_eq!(
            evaluation.view.control("dependent").unwrap().selected,
            expected
        );
        if sr && !rr11 && !rr12 {
            assert_eq!(
                evaluation
                    .view
                    .control("compat")
                    .unwrap()
                    .unavailable_reason
                    .as_deref(),
                Some("Requires at least one of: rr11, rr12.")
            );
        }
        // Disabled dependencies retain their preferences and reappear when available.
        let mut retry = evaluation.normalized_selection.to_selection();
        retry.set_feature("sr", true);
        retry.set_feature("rr12", true);
        assert!(
            evaluate(&manifest, &retry)
                .unwrap()
                .view
                .control("dependent")
                .unwrap()
                .selected
        );
    }
}

#[test]
fn choice_projection_previews_replacement_without_changing_defaults() {
    let manifest = grouped_manifest();
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let alternate = evaluation.view.control("forty").unwrap();
    assert_eq!(alternate.source_label.as_deref(), Some("Tweaks Anthology"));
    assert_eq!(alternate.group_label.as_deref(), Some("Potion stacking"));
    assert_eq!(alternate.choice_available, Some(true));
    assert!(alternate.unavailable_reason.is_none());
    assert!(!alternate.selected);
    assert!(evaluation.normalized_selection.features["unlimited"]);
    assert!(!evaluation.normalized_selection.features["forty"]);
    assert_eq!(evaluation.plan.runs[0].components, vec![1]);

    let mut switched = evaluation.normalized_selection.to_selection();
    switched.set_feature("unlimited", false);
    switched.set_feature("forty", true);
    let result = evaluate(&manifest, &switched).unwrap();
    assert_eq!(result.plan.runs[0].components, vec![2]);
}

#[test]
fn choice_replacement_keeps_external_conflicts_and_dependencies() {
    let mut manifest = grouped_manifest();
    manifest
        .collection
        .features
        .push(feature("spell-revisions", Decision::Default, vec![]));
    manifest.collection.features[1].conflicts.push(Conflict {
        feature_id: "spell-revisions".to_owned(),
        reason: "Unavailable with Spell Revisions.".to_owned(),
    });
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let alternate = evaluation.view.control("forty").unwrap();
    assert_eq!(alternate.choice_available, Some(false));
    assert_eq!(
        alternate.unavailable_reason.as_deref(),
        Some("Unavailable with Spell Revisions.")
    );
    assert!(evaluation.normalized_selection.features["spell-revisions"]);

    manifest.collection.features[1].conflicts.pop();
    manifest.collection.features[1]
        .requires
        .push("unlimited".to_owned());
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    assert_eq!(
        evaluation.view.control("forty").unwrap().choice_available,
        Some(false)
    );
}

#[test]
fn malformed_or_mandatory_choice_groups_never_relax_controls() {
    let mut manifest = grouped_manifest();
    manifest.collection.features[1].conflicts.clear();
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    assert_eq!(
        evaluation.view.control("forty").unwrap().choice_available,
        None
    );
    let mut manifest = grouped_manifest();
    manifest.collection.features[0].decision = Decision::Mandatory;
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    assert_eq!(
        evaluation.view.control("forty").unwrap().choice_available,
        None
    );
}

fn semantic_manifest() -> Manifest {
    let mut manifest = fixture();
    manifest.collection.runs.clear();

    add_run(&mut manifest, "meaning", &[1, 2, 3, 4, 5, 6]);
    add_run(&mut manifest, "sod-remix", SOD_COMPONENTS);
    add_run(&mut manifest, "tempus-rework", TEMPUS_COMPONENTS);
    add_run(&mut manifest, "spell-revisions", &[0]);
    add_run(&mut manifest, "scs", &[4240, 8040]);
    add_run(&mut manifest, "randomiser", &[10300]);

    let mut mandatory_child = feature(
        "mandatory-child",
        Decision::Mandatory,
        component_refs("meaning", &[4]),
    );
    mandatory_child.parent = Some("default-feature".to_owned());

    let mut blocked = feature(
        "blocked-default",
        Decision::Default,
        component_refs("meaning", &[5]),
    );
    blocked.readiness = Readiness::Blocked;
    blocked.unavailable_reason = Some("Awaiting a release-ready implementation.".to_owned());

    let mut choice_host = feature("choice-host", Decision::Default, Vec::new());
    choice_host.inputs = (1..=7)
        .map(|index| InputSpec::Choice {
            id: format!("optional-group-{index}"),
            default: "none".to_owned(),
            options: vec![
                InputOption {
                    id: "none".to_owned(),
                    title: "None".to_owned(),
                    answer: String::new(),
                },
                InputOption {
                    id: "enabled".to_owned(),
                    title: "Enabled".to_owned(),
                    answer: "1".to_owned(),
                },
            ],
        })
        .collect();

    let mut scs_4240 = feature(
        "scs-4240",
        Decision::Optional,
        component_refs("scs", &[4240]),
    );
    scs_4240.conflicts.push(Conflict {
        feature_id: "spell-revisions".to_owned(),
        reason: "Unavailable while Spell Revisions is selected.".to_owned(),
    });

    let mut scs_8040 = feature(
        "scs-8040",
        Decision::Mandatory,
        component_refs("scs", &[8040]),
    );
    scs_8040.parent = Some("scs-core".to_owned());

    let mut randomiser_10300 = feature(
        "randomiser-10300",
        Decision::Default,
        component_refs("randomiser", &[10300]),
    );
    randomiser_10300.conflicts.push(Conflict {
        feature_id: "scs-8040".to_owned(),
        reason: "Already provided by SCS component 8040 (Improved random spawns).".to_owned(),
    });
    randomiser_10300.requires.push("scs-core".to_owned());

    manifest.collection.features = vec![
        feature(
            "blank-component",
            Decision::Excluded,
            component_refs("meaning", &[1]),
        ),
        feature(
            "optional-feature",
            Decision::Optional,
            component_refs("meaning", &[2]),
        ),
        feature(
            "default-feature",
            Decision::Default,
            component_refs("meaning", &[3]),
        ),
        mandatory_child,
        feature(
            "root-mandatory",
            Decision::Mandatory,
            component_refs("meaning", &[6]),
        ),
        blocked,
        feature(
            "sod-remix",
            Decision::Default,
            component_refs("sod-remix", SOD_COMPONENTS),
        ),
        feature(
            "tempus-rework",
            Decision::Default,
            component_refs("tempus-rework", TEMPUS_COMPONENTS),
        ),
        feature(
            "spell-revisions",
            Decision::Default,
            component_refs("spell-revisions", &[0]),
        ),
        feature("scs-core", Decision::Default, Vec::new()),
        scs_4240,
        scs_8040,
        randomiser_10300,
        choice_host,
    ];

    manifest
}

#[test]
fn root_mandatory_is_valid_selected_and_noninteractive() {
    let manifest = semantic_manifest();
    check(&manifest).unwrap();

    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let control = evaluation.view.control("root-mandatory").unwrap();

    assert!(control.selected);
    assert!(!control.interactive);
    assert!(evaluation
        .plan
        .components_for("meaning")
        .unwrap()
        .contains(&6));
}

#[test]
fn explicit_off_cannot_disable_root_mandatory() {
    let manifest = semantic_manifest();
    check(&manifest).unwrap();
    let mut selection = Selection::defaults("windows");
    selection.set_feature("root-mandatory", false);

    let evaluation = evaluate(&manifest, &selection).unwrap();

    assert!(evaluation.view.control("root-mandatory").unwrap().selected);
    assert_eq!(
        evaluation
            .normalized_selection
            .features
            .get("root-mandatory"),
        Some(&true)
    );
    assert!(evaluation
        .plan
        .components_for("meaning")
        .unwrap()
        .contains(&6));
}

#[test]
fn resolved_plan_carries_run_postconditions_unchanged() {
    let mut manifest = semantic_manifest();
    let expected = bg_engine::manifest::Postcondition::TextFileMarkers {
        path: "weidu.conf".to_owned(),
        required: vec!["lang_dir = en_US".to_owned()],
        forbidden: Vec::new(),
        max_bytes: 4096,
    };
    manifest
        .collection
        .runs
        .iter_mut()
        .find(|run| run.run_id == "meaning")
        .unwrap()
        .postconditions
        .push(expected.clone());

    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let run = evaluation
        .plan
        .runs
        .iter()
        .find(|run| run.run_id == "meaning")
        .unwrap();

    assert_eq!(run.postconditions, vec![expected]);
}

#[test]
fn projects_settled_decisions_and_exact_atomic_expansions() {
    let evaluation = evaluate(&semantic_manifest(), &Selection::defaults("windows")).unwrap();
    let view = &evaluation.view;

    assert!(view.control("blank-component").is_none());
    assert!(!view.control("optional-feature").unwrap().selected);
    assert!(view.control("default-feature").unwrap().selected);
    assert!(!view.control("mandatory-child").unwrap().interactive);
    assert_eq!(
        evaluation.plan.components_for("sod-remix").unwrap(),
        SOD_COMPONENTS
    );
    assert_eq!(
        evaluation.plan.components_for("tempus-rework").unwrap(),
        TEMPUS_COMPONENTS
    );
}

#[test]
fn authors_compatibility_reasons_and_omits_conflicting_components() {
    let evaluation = evaluate(&semantic_manifest(), &Selection::defaults("windows")).unwrap();

    assert_eq!(
        evaluation
            .view
            .control("scs-4240")
            .unwrap()
            .unavailable_reason
            .as_deref(),
        Some("Unavailable while Spell Revisions is selected.")
    );
    assert_eq!(
        evaluation
            .view
            .control("randomiser-10300")
            .unwrap()
            .unavailable_reason
            .as_deref(),
        Some("Already provided by SCS component 8040 (Improved random spawns).")
    );
    assert!(evaluation.plan.components_for("randomiser").is_none());
}

#[test]
fn feature_controls_expose_authored_constraints_without_inventing_reciprocals() {
    let evaluation = evaluate(&semantic_manifest(), &Selection::defaults("windows")).unwrap();

    let randomiser = evaluation.view.control("randomiser-10300").unwrap();
    assert_eq!(randomiser.requires, ["scs-core"]);
    assert_eq!(randomiser.conflicts, ["scs-8040"]);

    let scs_8040 = evaluation.view.control("scs-8040").unwrap();
    assert!(scs_8040.requires.is_empty());
    assert!(scs_8040.conflicts.is_empty());

    let scs_4240 = evaluation.view.control("scs-4240").unwrap();
    assert_eq!(scs_4240.conflicts, ["spell-revisions"]);
    assert!(evaluation
        .view
        .control("spell-revisions")
        .unwrap()
        .conflicts
        .is_empty());
}

#[test]
fn blocked_default_keeps_identity_reason_and_an_omission_finding() {
    let evaluation = evaluate(&semantic_manifest(), &Selection::defaults("windows")).unwrap();
    let control = evaluation.view.control("blocked-default").unwrap();

    assert!(!control.selected);
    assert!(!control.interactive);
    assert_eq!(
        control.unavailable_reason.as_deref(),
        Some("Awaiting a release-ready implementation.")
    );
    assert_eq!(
        evaluation
            .normalized_selection
            .features
            .get("blocked-default"),
        Some(&true)
    );
    assert!(evaluation.findings.iter().any(|finding| {
        finding.rule == "blocked-default-omitted" && finding.feature_id == "blocked-default"
    }));
}

#[test]
fn semantic_selection_identity_survives_reordered_reevaluation() {
    let mut selection = Selection::defaults("windows");
    selection.set_feature("optional-feature", true);
    let manifest = semantic_manifest();
    let first = evaluate(&manifest, &selection).unwrap();

    let mut reordered = manifest;
    reordered.collection.features.reverse();
    let second = evaluate(&reordered, &first.normalized_selection.to_selection()).unwrap();

    assert!(first.view.control("optional-feature").unwrap().selected);
    assert!(second.view.control("optional-feature").unwrap().selected);
}

#[test]
fn optional_only_choice_groups_have_explicit_none_defaults() {
    let evaluation = evaluate(&semantic_manifest(), &Selection::defaults("windows")).unwrap();
    let inputs = &evaluation.normalized_selection.inputs["choice-host"];

    assert_eq!(inputs.len(), 7);
    assert!(inputs
        .values()
        .all(|value| value == &InputValue::Choice("none".to_owned())));
}

#[test]
fn boolean_choice_and_integer_inputs_normalize_to_typed_values() {
    let mut manifest = semantic_manifest();
    let configured = manifest
        .collection
        .features
        .iter_mut()
        .find(|feature| feature.id == "choice-host")
        .unwrap();
    configured.inputs.extend([
        InputSpec::Boolean {
            id: "enabled".to_owned(),
            default: false,
        },
        InputSpec::Integer {
            id: "count".to_owned(),
            default: 2,
            min: 1,
            max: 3,
        },
    ]);
    let mut selection = Selection::defaults("windows");
    selection.set_input("choice-host", "enabled", InputValue::Boolean(true));
    selection.set_input("choice-host", "count", InputValue::Integer(3));

    let evaluation = evaluate(&manifest, &selection).unwrap();

    assert_eq!(
        evaluation.normalized_selection.inputs["choice-host"]["enabled"],
        InputValue::Boolean(true)
    );
    assert_eq!(
        evaluation.normalized_selection.inputs["choice-host"]["count"],
        InputValue::Integer(3)
    );
}

#[test]
fn out_of_bounds_selected_integer_is_rejected() {
    let mut manifest = semantic_manifest();
    manifest
        .collection
        .features
        .iter_mut()
        .find(|feature| feature.id == "choice-host")
        .unwrap()
        .inputs
        .push(InputSpec::Integer {
            id: "count".to_owned(),
            default: 2,
            min: 1,
            max: 3,
        });
    let mut selection = Selection::defaults("windows");
    selection.set_input("choice-host", "count", InputValue::Integer(4));

    let error = evaluate(&manifest, &selection).unwrap_err();

    assert!(error.to_string().contains("count"), "{error}");
    assert!(error.to_string().contains("1..=3"), "{error}");
}

#[test]
fn preset_values_are_semantic_and_do_not_expose_component_numbers() {
    let mut manifest = semantic_manifest();
    manifest.presets.get_mut("recommended").unwrap().selections = BTreeMap::from([
        ("optional-feature".to_owned(), "on".to_owned()),
        (
            "choice-host/optional-group-1".to_owned(),
            "enabled".to_owned(),
        ),
    ]);

    let evaluation = evaluate_preset(&manifest, "recommended", "windows").unwrap();

    assert!(
        evaluation
            .view
            .control("optional-feature")
            .unwrap()
            .selected
    );
    assert_eq!(
        evaluation.normalized_selection.inputs["choice-host"]["optional-group-1"],
        InputValue::Choice("enabled".to_owned())
    );
}

#[test]
fn prompt_answers_are_rendered_stepwise_from_literals_and_validated_input_refs() {
    let mut manifest = semantic_manifest();
    let prompt_run = "prompt-mod";
    add_run(&mut manifest, prompt_run, &[10]);
    let ModFile { components, .. } = manifest.mods.get_mut(prompt_run).unwrap();
    components[0].prompts = vec![
        PromptStep {
            expected_output: "Choose mode".to_owned(),
            answer: PromptAnswer::Input(FeatureInputRef {
                feature_id: "prompt-feature".to_owned(),
                input_id: "mode".to_owned(),
            }),
            when_any_features: Vec::new(),
        },
        PromptStep {
            expected_output: "Confirm".to_owned(),
            answer: PromptAnswer::Literal(InputValue::Boolean(true)),
            when_any_features: Vec::new(),
        },
    ];
    let mut prompt_feature = feature(
        "prompt-feature",
        Decision::Default,
        component_refs(prompt_run, &[10]),
    );
    prompt_feature.inputs = vec![InputSpec::Choice {
        id: "mode".to_owned(),
        default: "safe".to_owned(),
        options: vec![InputOption {
            id: "safe".to_owned(),
            title: "Safe mode".to_owned(),
            answer: "1".to_owned(),
        }],
    }];
    manifest.collection.features.push(prompt_feature);

    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let script = render_prompt_script(
        &manifest,
        &ComponentRef {
            run_id: prompt_run.to_owned(),
            component: 10,
        },
        &evaluation.normalized_selection,
    )
    .unwrap();

    assert_eq!(script.steps.len(), 2);
    assert_eq!(script.steps[0].expected_output, "Choose mode");
    assert_eq!(script.steps[0].answer, "1\n");
    assert_eq!(script.steps[1].answer, "true\n");
}

fn conditional_prompt_manifest() -> Manifest {
    let mut manifest = semantic_manifest();
    let prompt_run = "conditional-prompt-mod";
    add_run(&mut manifest, prompt_run, &[10]);
    manifest.mods.get_mut(prompt_run).unwrap().components[0].prompts = vec![PromptStep {
        expected_output: "Install compatibility patch?".to_owned(),
        answer: PromptAnswer::Literal(InputValue::Boolean(true)),
        when_any_features: vec!["condition-a".to_owned(), "condition-b".to_owned()],
    }];
    manifest.collection.features.extend([
        feature(
            "conditional-prompt-owner",
            Decision::Default,
            component_refs(prompt_run, &[10]),
        ),
        feature("condition-a", Decision::Optional, Vec::new()),
        feature("condition-b", Decision::Optional, Vec::new()),
    ]);
    manifest
}

#[test]
fn conditional_prompt_is_omitted_when_all_conditions_are_effectively_off() {
    let manifest = conditional_prompt_manifest();

    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let run = evaluation
        .plan
        .runs
        .iter()
        .find(|run| run.run_id == "conditional-prompt-mod")
        .unwrap();

    assert_eq!(run.components, [10]);
    assert!(run.prompt_scripts.is_empty());
}

#[test]
fn conditional_prompt_is_emitted_once_when_either_condition_is_effective() {
    let manifest = conditional_prompt_manifest();

    for feature_ids in [
        &["condition-a"][..],
        &["condition-b"][..],
        &["condition-a", "condition-b"][..],
    ] {
        let mut selection = Selection::defaults("windows");
        for feature_id in feature_ids {
            selection.set_feature(feature_id, true);
        }

        let evaluation = evaluate(&manifest, &selection).unwrap();
        let run = evaluation
            .plan
            .runs
            .iter()
            .find(|run| run.run_id == "conditional-prompt-mod")
            .unwrap();

        assert_eq!(run.components, [10]);
        assert_eq!(run.prompt_scripts.len(), 1);
        assert_eq!(run.prompt_scripts[0].steps.len(), 1);
        assert_eq!(
            run.prompt_scripts[0].steps[0].expected_output,
            "Install compatibility patch?"
        );
    }
}

#[test]
fn conditional_prompt_uses_effective_state_when_selected_child_has_disabled_parent() {
    let mut manifest = conditional_prompt_manifest();
    let condition = manifest
        .collection
        .features
        .iter_mut()
        .find(|feature| feature.id == "condition-a")
        .unwrap();
    condition.parent = Some("condition-parent".to_owned());
    manifest
        .collection
        .features
        .push(feature("condition-parent", Decision::Optional, Vec::new()));
    let mut selection = Selection::defaults("windows");
    selection.set_feature("condition-a", true);

    let evaluation = evaluate(&manifest, &selection).unwrap();
    let run = evaluation
        .plan
        .runs
        .iter()
        .find(|run| run.run_id == "conditional-prompt-mod")
        .unwrap();

    assert!(evaluation
        .normalized_selection
        .features
        .get("condition-a")
        .copied()
        .unwrap());
    assert!(!evaluation.view.control("condition-a").unwrap().selected);
    assert!(run.prompt_scripts.is_empty());
}

/// Adds a run whose installer comes from its own download, `{run_id}-archive`.
fn add_run_with_own_download(manifest: &mut Manifest, run_id: &str, components: &[u32]) {
    add_run(manifest, run_id, components);
    let tp2 = format!("{run_id}/{run_id}.tp2");
    let shared = manifest.artifacts.get_mut("eefixpack").unwrap();
    shared.archive.publish_roots.retain(|root| root != run_id);
    shared.archive.tp2_paths.retain(|path| *path != tp2);
    let artifact_id = format!("{run_id}-archive");
    let mut artifact = manifest.artifacts["eefixpack"].clone();
    artifact.id = artifact_id.clone();
    artifact.name = format!("{run_id} download");
    artifact.source.url = format!("https://example.invalid/{run_id}.zip");
    artifact.source.expected_filename = Some(format!("{run_id}.zip"));
    artifact.archive.publish_roots = vec![run_id.to_owned()];
    artifact.archive.tp2_paths = vec![tp2];
    manifest.artifacts.insert(artifact_id.clone(), artifact);
    manifest.mods.get_mut(run_id).unwrap().artifact_id = artifact_id;
}

fn literal_prompt(expected_output: &str, when_any_features: &[&str]) -> PromptStep {
    PromptStep {
        expected_output: expected_output.to_owned(),
        answer: PromptAnswer::Literal(InputValue::Boolean(true)),
        when_any_features: when_any_features
            .iter()
            .map(|feature| (*feature).to_owned())
            .collect(),
    }
}

fn marker_check() -> Postcondition {
    Postcondition::TextFileMarkers {
        path: "weidu.conf".to_owned(),
        required: vec!["lang_dir = en_US".to_owned()],
        forbidden: Vec::new(),
        max_bytes: 4096,
    }
}

/// A collection around one mod, `snowy`, whose download may become unusable: a fixed core,
/// mods and tweaks that need `snowy` through each kind of relation, and unrelated parts.
fn leave_out_manifest() -> Manifest {
    let mut manifest = fixture();
    manifest.collection.runs.clear();
    for (run_id, components) in [
        ("core", &[0][..]),
        ("snowy", &[0, 1][..]),
        ("needs-snowy", &[0][..]),
        ("either", &[0][..]),
        ("other", &[0][..]),
        ("alt", &[0][..]),
        ("snowy-patch", &[0][..]),
        ("tweaks", &[1, 2, 3, 4][..]),
    ] {
        add_run_with_own_download(&mut manifest, run_id, components);
    }
    for run in manifest
        .collection
        .runs
        .iter_mut()
        .filter(|run| run.run_id == "tweaks" || run.run_id == "other")
    {
        run.postconditions.push(marker_check());
    }
    let tweaks = manifest.mods.get_mut("tweaks").unwrap();
    tweaks.components[0].prompts = vec![literal_prompt("Always asked", &[])];
    tweaks.components[2].prompts = vec![
        literal_prompt("Patch the snowy areas?", &["snowy"]),
        literal_prompt("Confirm", &[]),
    ];

    let mut snowy_extra = feature(
        "snowy-extra",
        Decision::Mandatory,
        component_refs("snowy", &[1]),
    );
    snowy_extra.parent = Some("snowy".to_owned());
    let mut snowy_patch = feature(
        "snowy-patch",
        Decision::Mandatory,
        component_refs("snowy-patch", &[0]),
    );
    snowy_patch.parent = Some("snowy".to_owned());
    let mut needs_snowy = feature(
        "needs-snowy",
        Decision::Default,
        component_refs("needs-snowy", &[0]),
    );
    needs_snowy.requires.push("snowy".to_owned());
    let mut either = feature("either", Decision::Default, component_refs("either", &[0]));
    either.requires_any = vec!["snowy".to_owned(), "other".to_owned()];
    let mut alt = feature("alt", Decision::Optional, component_refs("alt", &[0]));
    alt.conflicts.push(Conflict {
        feature_id: "snowy".to_owned(),
        reason: "Choose either snowy or its alternative.".to_owned(),
    });
    let mut tweak_snowy = feature(
        "tweak-snowy",
        Decision::Default,
        component_refs("tweaks", &[2]),
    );
    tweak_snowy.requires.push("snowy".to_owned());
    let mut tweak_under_snowy = feature(
        "tweak-under-snowy",
        Decision::Default,
        component_refs("tweaks", &[4]),
    );
    tweak_under_snowy.parent = Some("snowy".to_owned());
    manifest.collection.features = vec![
        feature("core", Decision::Mandatory, component_refs("core", &[0])),
        feature("snowy", Decision::Default, component_refs("snowy", &[0])),
        snowy_extra,
        snowy_patch,
        needs_snowy,
        either,
        feature("other", Decision::Default, component_refs("other", &[0])),
        alt,
        feature(
            "tweak-plain",
            Decision::Default,
            component_refs("tweaks", &[1]),
        ),
        tweak_snowy,
        feature(
            "tweak-prompted",
            Decision::Default,
            component_refs("tweaks", &[3]),
        ),
        tweak_under_snowy,
    ];
    manifest
}

fn unusable(artifact_ids: &[&str]) -> BTreeSet<String> {
    artifact_ids.iter().map(|id| (*id).to_owned()).collect()
}

fn run_ids(plan: &bg_engine::resolve::InstallPlan) -> Vec<&str> {
    plan.runs.iter().map(|run| run.run_id.as_str()).collect()
}

fn planned_run<'a>(
    plan: &'a bg_engine::resolve::InstallPlan,
    run_id: &str,
) -> &'a bg_engine::resolve::PlannedRun {
    plan.runs
        .iter()
        .find(|run| run.run_id == run_id)
        .unwrap_or_else(|| panic!("run {run_id} is not planned"))
}

fn prompt_outputs(run: &bg_engine::resolve::PlannedRun) -> Vec<(u32, Vec<&str>)> {
    run.prompt_scripts
        .iter()
        .map(|script| {
            (
                script.component.component,
                script
                    .steps
                    .iter()
                    .map(|step| step.expected_output.as_str())
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn leaving_out_a_mod_drops_its_runs_and_everything_that_needs_it() {
    let manifest = leave_out_manifest();
    check(&manifest).unwrap();
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let plan = &evaluation.plan;
    assert_eq!(
        run_ids(plan),
        [
            "core",
            "snowy",
            "needs-snowy",
            "either",
            "other",
            "snowy-patch",
            "tweaks"
        ]
    );
    assert_eq!(planned_run(plan, "tweaks").components, [1, 2, 3, 4]);
    assert_eq!(
        prompt_outputs(planned_run(plan, "tweaks")),
        [
            (1, vec!["Always asked"]),
            (3, vec!["Patch the snowy areas?", "Confirm"])
        ]
    );

    let remaining = plan_without_artifacts(
        &manifest,
        &evaluation.normalized_selection,
        plan,
        &unusable(&["snowy-archive"]),
    )
    .unwrap();

    // `needs-snowy` (requires), `snowy-patch` (mandatory child) and the tweaks that need
    // snowy (requires, parent) go; `either` keeps `other`, which it may use instead.
    assert_eq!(run_ids(&remaining), ["core", "either", "other", "tweaks"]);
    assert_eq!(planned_run(&remaining, "core"), planned_run(plan, "core"));
    assert_eq!(
        planned_run(&remaining, "either"),
        planned_run(plan, "either")
    );
    assert_eq!(planned_run(&remaining, "other"), planned_run(plan, "other"));
    assert_eq!(
        planned_run(&remaining, "other").postconditions,
        [marker_check()]
    );
    let tweaks = planned_run(&remaining, "tweaks");
    assert_eq!(tweaks.components, [1, 3]);
    // The recipe checks describe the complete run, so a run with fewer parts drops them.
    assert!(tweaks.postconditions.is_empty());
    // The answer that only applies with snowy is no longer given.
    assert_eq!(
        prompt_outputs(tweaks),
        [(1, vec!["Always asked"]), (3, vec!["Confirm"])]
    );
}

#[test]
fn leaving_out_a_mod_never_adds_a_part_the_player_did_not_get() {
    let manifest = leave_out_manifest();
    let mut selection = Selection::defaults("windows");
    // `alt` is ticked but loses to snowy; `either` then only works through snowy.
    selection.set_feature("alt", true);
    selection.set_feature("other", false);
    let evaluation = evaluate(&manifest, &selection).unwrap();
    assert!(!run_ids(&evaluation.plan).contains(&"alt"));
    assert!(run_ids(&evaluation.plan).contains(&"either"));

    let remaining = plan_without_artifacts(
        &manifest,
        &evaluation.normalized_selection,
        &evaluation.plan,
        &unusable(&["snowy-archive"]),
    )
    .unwrap();

    assert_eq!(run_ids(&remaining), ["core", "tweaks"]);
    assert_eq!(planned_run(&remaining, "tweaks").components, [1, 3]);
}

#[test]
fn a_required_part_of_a_selectable_mod_leaves_out_that_whole_mod() {
    let manifest = leave_out_manifest();
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let without_snowy = plan_without_artifacts(
        &manifest,
        &evaluation.normalized_selection,
        &evaluation.plan,
        &unusable(&["snowy-archive"]),
    )
    .unwrap();

    // `snowy-patch` is always installed with snowy, so losing its own download means
    // leaving out snowy and everything that needs it.
    let without_patch = plan_without_artifacts(
        &manifest,
        &evaluation.normalized_selection,
        &evaluation.plan,
        &unusable(&["snowy-patch-archive"]),
    )
    .unwrap();

    assert_eq!(without_patch, without_snowy);
}

#[test]
fn a_fixed_part_of_the_collection_is_never_left_out() {
    let manifest = leave_out_manifest();
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();

    for artifacts in [
        &["core-archive"][..],
        &["snowy-archive", "core-archive"][..],
    ] {
        let error = plan_without_artifacts(
            &manifest,
            &evaluation.normalized_selection,
            &evaluation.plan,
            &unusable(artifacts),
        )
        .unwrap_err();
        assert_eq!(
            error,
            LeaveOutError::Required {
                feature_id: "core".to_owned(),
                title: "core".to_owned(),
            }
        );
        assert_eq!(
            error.to_string(),
            "core is a fixed part of this collection, so the installation cannot continue without it"
        );
    }
}

#[test]
fn an_unused_or_empty_leave_out_keeps_the_plan_exactly() {
    let manifest = leave_out_manifest();
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();

    for artifacts in [&[][..], &["alt-archive"][..]] {
        let remaining = plan_without_artifacts(
            &manifest,
            &evaluation.normalized_selection,
            &evaluation.plan,
            &unusable(artifacts),
        )
        .unwrap();
        assert_eq!(remaining, evaluation.plan);
    }
}

#[test]
fn manual_pause_segments_keep_their_identity_when_mods_are_left_out() {
    let manifest = leave_out_manifest();
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let (split, _checkpoints) = split_plan(
        &evaluation.plan,
        &[
            ManualBreakpoint {
                run_id: "tweaks".to_owned(),
                component: 1,
            },
            ManualBreakpoint {
                run_id: "snowy".to_owned(),
                component: 0,
            },
        ],
    )
    .unwrap();
    assert_eq!(
        run_ids(&split),
        [
            "core",
            "snowy--manual-0",
            "snowy",
            "needs-snowy",
            "either",
            "other",
            "snowy-patch",
            "tweaks--manual-1",
            "tweaks"
        ]
    );

    let remaining = plan_without_artifacts(
        &manifest,
        &evaluation.normalized_selection,
        &split,
        &unusable(&["snowy-archive"]),
    )
    .unwrap();

    assert_eq!(
        run_ids(&remaining),
        ["core", "either", "other", "tweaks--manual-1", "tweaks"]
    );
    assert_eq!(
        planned_run(&remaining, "tweaks--manual-1"),
        planned_run(&split, "tweaks--manual-1")
    );
    let tail = planned_run(&remaining, "tweaks");
    assert_eq!(tail.components, [3]);
    assert!(tail.postconditions.is_empty());
    assert_eq!(prompt_outputs(tail), [(3, vec!["Confirm"])]);
}

#[test]
fn a_planned_run_outside_the_recipe_is_refused() {
    let manifest = leave_out_manifest();
    let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let mut plan = evaluation.plan.clone();
    plan.runs[0].run_id = "core-renamed".to_owned();

    let error = plan_without_artifacts(
        &manifest,
        &evaluation.normalized_selection,
        &plan,
        &unusable(&["snowy-archive"]),
    )
    .unwrap_err();

    assert!(matches!(error, LeaveOutError::Invalid(_)), "{error:?}");
}
