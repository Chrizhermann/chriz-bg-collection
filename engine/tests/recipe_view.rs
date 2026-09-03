use std::collections::BTreeMap;
use std::path::PathBuf;

use bg_engine::manifest::{
    Component, ComponentRef, Conflict, Decision, Feature, FeatureInputRef, InputOption, InputSpec,
    InputValue, InvocationMode, ModFile, Phase, PromptAnswer, PromptStep, Readiness, Run,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset, render_prompt_script};
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
    });
}

fn feature(id: &str, decision: Decision, components: Vec<ComponentRef>) -> Feature {
    Feature {
        id: id.to_owned(),
        title: id.to_owned(),
        description: format!("Description for {id}"),
        category: "rules-abilities".to_owned(),
        decision,
        readiness: Readiness::Ready,
        unavailable_reason: None,
        parent: None,
        components,
        requires: Vec::new(),
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
        },
        PromptStep {
            expected_output: "Confirm".to_owned(),
            answer: PromptAnswer::Literal(InputValue::Boolean(true)),
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
