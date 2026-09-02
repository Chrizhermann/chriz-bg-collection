//! Engine-owned projection and evaluation of curated recipe features.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, Result};
use crate::manifest::{
    ComponentRef, Decision, Feature, InputSpec, InputValue, PromptAnswer, Readiness,
};
use crate::resolve::{InstallPlan, PlannedRun, Selection};
use crate::Manifest;

/// One typed input and its current normalized value for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputControl {
    /// Authored input definition.
    pub spec: InputSpec,
    /// Current validated value.
    pub value: InputValue,
    /// Whether the current feature state permits editing this input.
    pub interactive: bool,
}

/// One player-facing feature control; it intentionally contains no component numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureControl {
    /// Stable semantic selection id.
    pub id: String,
    /// Player-facing title.
    pub title: String,
    /// Player-facing description.
    pub description: String,
    /// Player-facing category id.
    pub category: String,
    /// Curated decision.
    pub decision: Decision,
    /// Release readiness.
    pub readiness: Readiness,
    /// Parent semantic id, when present.
    pub parent: Option<String>,
    /// Effective selected state after readiness and compatibility evaluation.
    pub selected: bool,
    /// Whether the player can change this feature directly.
    pub interactive: bool,
    /// Authored or engine-derived reason the feature cannot currently resolve.
    pub unavailable_reason: Option<String>,
    /// Typed inputs rendered beneath this feature.
    pub inputs: Vec<InputControl>,
}

/// Ordered engine projection rendered by the frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeView {
    /// Category ids in first-authored-feature order.
    pub categories: Vec<String>,
    /// Visible controls in authored feature order.
    pub controls: Vec<FeatureControl>,
}

impl RecipeView {
    /// Looks up a visible control by stable semantic id.
    pub fn control(&self, id: &str) -> Option<&FeatureControl> {
        self.controls.iter().find(|control| control.id == id)
    }
}

/// Fully typed semantic state retained independently of display order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedSelection {
    /// Requested target platform.
    pub platform: String,
    /// Desired feature state keyed by semantic feature id.
    pub features: BTreeMap<String, bool>,
    /// Validated feature inputs keyed first by feature id and then input id.
    pub inputs: BTreeMap<String, BTreeMap<String, InputValue>>,
}

impl NormalizedSelection {
    /// Converts the normalized state back into an order-independent selection request.
    pub fn to_selection(&self) -> Selection {
        let mut selection = Selection::defaults(&self.platform);
        for (feature_id, selected) in &self.features {
            selection.set_feature(feature_id, *selected);
        }
        for (feature_id, inputs) in &self.inputs {
            for (input_id, value) in inputs {
                selection.set_input(feature_id, input_id, value.clone());
            }
        }
        selection
    }
}

/// A nonfatal consequence of selection evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionFinding {
    /// Stable rule id.
    pub rule: String,
    /// Semantic feature affected by the finding.
    pub feature_id: String,
    /// Player-facing explanation.
    pub message: String,
}

/// Complete engine result consumed by the setup and review screens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionEvaluation {
    /// UI-safe feature projection.
    pub view: RecipeView,
    /// Stable normalized semantic selection.
    pub normalized_selection: NormalizedSelection,
    /// Nonfatal omissions and warnings.
    pub findings: Vec<SelectionFinding>,
    /// Exact component-level installation plan.
    pub plan: InstallPlan,
}

/// One rendered output matcher and answer line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderedPromptStep {
    /// Output text that must be observed before answering.
    pub expected_output: String,
    /// Exact answer text including its terminating newline.
    pub answer: String,
}

/// Ordered prompt steps for one selected component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptScript {
    /// Exact selected component owning these prompts.
    pub component: ComponentRef,
    /// Individual output-gated answers; never an eagerly concatenated stdin stream.
    pub steps: Vec<RenderedPromptStep>,
}

/// Evaluates a named preset using only its semantic feature and input values.
pub fn evaluate_preset(
    manifest: &Manifest,
    preset_id: &str,
    platform: &str,
) -> Result<SelectionEvaluation> {
    let preset = manifest
        .presets
        .get(preset_id)
        .ok_or_else(|| EngineError::InvalidSelection(format!("unknown preset {preset_id:?}")))?;
    let selection = Selection {
        platform: platform.to_owned(),
        choices: preset.selections.clone(),
    };
    evaluate(manifest, &selection)
}

/// Evaluates one semantic selection into a UI projection and exact install plan.
pub fn evaluate(manifest: &Manifest, selection: &Selection) -> Result<SelectionEvaluation> {
    crate::validate::check(manifest)?;

    if selection.platform != "windows" {
        return Err(EngineError::InvalidSelection(format!(
            "unsupported platform {:?}; recipe-v2 alpha supports windows",
            selection.platform
        )));
    }

    let features = manifest
        .collection
        .features
        .iter()
        .map(|feature| (feature.id.as_str(), feature))
        .collect::<BTreeMap<_, _>>();
    let mut desired = manifest
        .collection
        .features
        .iter()
        .filter(|feature| feature.decision != Decision::Excluded)
        .map(|feature| {
            let selected = matches!(feature.decision, Decision::Default | Decision::Mandatory);
            (feature.id.clone(), selected)
        })
        .collect::<BTreeMap<_, _>>();
    let mut inputs = default_inputs(manifest);

    for (key, raw_value) in &selection.choices {
        if let Some(feature) = features.get(key.as_str()) {
            if feature.decision == Decision::Excluded {
                return Err(EngineError::InvalidSelection(format!(
                    "excluded feature {key:?} cannot be selected"
                )));
            }
            let selected = parse_feature_state(key, raw_value)?;
            if feature.decision != Decision::Mandatory {
                desired.insert(key.clone(), selected);
            }
            continue;
        }

        let Some((feature_id, input_id)) = key.split_once('/') else {
            return Err(EngineError::InvalidSelection(format!(
                "unknown semantic selection {key:?}"
            )));
        };
        let feature = features.get(feature_id).ok_or_else(|| {
            EngineError::InvalidSelection(format!(
                "input selection {key:?} names unknown feature {feature_id:?}"
            ))
        })?;
        if feature.decision == Decision::Excluded {
            return Err(EngineError::InvalidSelection(format!(
                "excluded feature {feature_id:?} cannot accept input {input_id:?}"
            )));
        }
        let spec = feature
            .inputs
            .iter()
            .find(|spec| spec.id() == input_id)
            .ok_or_else(|| {
                EngineError::InvalidSelection(format!(
                    "feature {feature_id:?} has no input {input_id:?}"
                ))
            })?;
        let value = parse_input_value(feature_id, spec, raw_value)?;
        if let Some(feature_inputs) = inputs.get_mut(feature_id) {
            feature_inputs.insert(input_id.to_owned(), value);
        }
    }

    let base_effective = base_effective_features(manifest, &desired);
    let mut effective = base_effective.clone();
    loop {
        let previous = effective.clone();
        for feature in manifest
            .collection
            .features
            .iter()
            .filter(|feature| feature.decision != Decision::Excluded)
        {
            let mut selected = base_effective.get(&feature.id).copied().unwrap_or(false);
            if selected {
                if let Some(parent) = &feature.parent {
                    selected &= previous.get(parent).copied().unwrap_or(false);
                }
                selected &= feature
                    .requires
                    .iter()
                    .all(|required| previous.get(required).copied().unwrap_or(false));
                selected &= feature.conflicts.iter().all(|conflict| {
                    !base_effective
                        .get(&conflict.feature_id)
                        .copied()
                        .unwrap_or(false)
                });
            }
            effective.insert(feature.id.clone(), selected);
        }
        if effective == previous {
            break;
        }
    }

    let mut categories = Vec::new();
    let mut seen_categories = BTreeSet::new();
    let mut controls = Vec::new();
    let mut findings = Vec::new();
    for feature in manifest
        .collection
        .features
        .iter()
        .filter(|feature| feature.decision != Decision::Excluded)
    {
        if seen_categories.insert(feature.category.clone()) {
            categories.push(feature.category.clone());
        }
        let reason = unavailable_reason(feature, &features, &base_effective, &effective);
        let selected = effective.get(&feature.id).copied().unwrap_or(false);
        let interactive = feature.decision != Decision::Mandatory && reason.is_none();
        let feature_inputs = inputs.get(&feature.id);
        let input_controls = feature
            .inputs
            .iter()
            .filter_map(|spec| {
                feature_inputs
                    .and_then(|values| values.get(spec.id()))
                    .cloned()
                    .map(|value| InputControl {
                        spec: spec.clone(),
                        value,
                        interactive: interactive && selected,
                    })
            })
            .collect();

        if feature.decision == Decision::Default
            && desired.get(&feature.id).copied().unwrap_or(false)
            && !selected
        {
            let (rule, message) = if feature.readiness == Readiness::Blocked {
                (
                    "blocked-default-omitted",
                    reason
                        .clone()
                        .unwrap_or_else(|| "This default is currently blocked.".to_owned()),
                )
            } else {
                (
                    "default-feature-omitted",
                    reason.clone().unwrap_or_else(|| {
                        "This default is unavailable under the current selection.".to_owned()
                    }),
                )
            };
            findings.push(SelectionFinding {
                rule: rule.to_owned(),
                feature_id: feature.id.clone(),
                message,
            });
        }

        controls.push(FeatureControl {
            id: feature.id.clone(),
            title: feature.title.clone(),
            description: feature.description.clone(),
            category: feature.category.clone(),
            decision: feature.decision,
            readiness: feature.readiness,
            parent: feature.parent.clone(),
            selected,
            interactive,
            unavailable_reason: reason,
            inputs: input_controls,
        });
    }

    let normalized_selection = NormalizedSelection {
        platform: selection.platform.clone(),
        features: desired,
        inputs,
    };
    let plan = build_plan(manifest, &effective, &normalized_selection)?;

    Ok(SelectionEvaluation {
        view: RecipeView {
            categories,
            controls,
        },
        normalized_selection,
        findings,
        plan,
    })
}

/// Renders one component's prompt steps without writing to stdin or starting a process.
pub fn render_prompt_script(
    manifest: &Manifest,
    component_ref: &ComponentRef,
    selection: &NormalizedSelection,
) -> Result<PromptScript> {
    let run = manifest
        .collection
        .runs
        .iter()
        .find(|run| run.run_id == component_ref.run_id)
        .ok_or_else(|| {
            EngineError::InvalidSelection(format!(
                "prompt component references unknown run {:?}",
                component_ref.run_id
            ))
        })?;
    let installer = manifest.mods.get(&run.mod_id).ok_or_else(|| {
        EngineError::InvalidSelection(format!(
            "prompt run {:?} references unknown installer {:?}",
            run.run_id, run.mod_id
        ))
    })?;
    let component = installer
        .components
        .iter()
        .find(|component| component.id == component_ref.component)
        .ok_or_else(|| {
            EngineError::InvalidSelection(format!(
                "run {:?} has no declared component {}",
                run.run_id, component_ref.component
            ))
        })?;

    let mut steps = Vec::with_capacity(component.prompts.len());
    for prompt in &component.prompts {
        let answer = match &prompt.answer {
            PromptAnswer::Literal(value) => render_literal(value),
            PromptAnswer::Input(input_ref) => {
                let feature = manifest
                    .collection
                    .features
                    .iter()
                    .find(|feature| feature.id == input_ref.feature_id)
                    .ok_or_else(|| {
                        EngineError::InvalidSelection(format!(
                            "prompt references unknown feature {:?}",
                            input_ref.feature_id
                        ))
                    })?;
                let spec = feature
                    .inputs
                    .iter()
                    .find(|spec| spec.id() == input_ref.input_id)
                    .ok_or_else(|| {
                        EngineError::InvalidSelection(format!(
                            "prompt references unknown input {:?}/{:?}",
                            input_ref.feature_id, input_ref.input_id
                        ))
                    })?;
                let value = selection
                    .inputs
                    .get(&input_ref.feature_id)
                    .and_then(|inputs| inputs.get(&input_ref.input_id))
                    .ok_or_else(|| {
                        EngineError::InvalidSelection(format!(
                            "normalized selection has no input {:?}/{:?}",
                            input_ref.feature_id, input_ref.input_id
                        ))
                    })?;
                render_input(spec, value)?
            }
        };
        steps.push(RenderedPromptStep {
            expected_output: prompt.expected_output.clone(),
            answer: format!("{answer}\n"),
        });
    }

    Ok(PromptScript {
        component: component_ref.clone(),
        steps,
    })
}

fn default_inputs(manifest: &Manifest) -> BTreeMap<String, BTreeMap<String, InputValue>> {
    manifest
        .collection
        .features
        .iter()
        .filter(|feature| feature.decision != Decision::Excluded)
        .map(|feature| {
            let values = feature
                .inputs
                .iter()
                .map(|spec| (spec.id().to_owned(), spec.default_value()))
                .collect();
            (feature.id.clone(), values)
        })
        .collect()
}

fn parse_feature_state(feature_id: &str, raw_value: &str) -> Result<bool> {
    match raw_value {
        "on" | "true" => Ok(true),
        "off" | "false" => Ok(false),
        _ => Err(EngineError::InvalidSelection(format!(
            "feature {feature_id:?} expects on or off, got {raw_value:?}"
        ))),
    }
}

fn parse_input_value(feature_id: &str, spec: &InputSpec, raw_value: &str) -> Result<InputValue> {
    match spec {
        InputSpec::Boolean { id, .. } => {
            let value = raw_value.strip_prefix("boolean:").unwrap_or(raw_value);
            match value {
                "true" => Ok(InputValue::Boolean(true)),
                "false" => Ok(InputValue::Boolean(false)),
                _ => Err(EngineError::InvalidSelection(format!(
                    "input {feature_id:?}/{id:?} expects true or false, got {raw_value:?}"
                ))),
            }
        }
        InputSpec::Choice { id, options, .. } => {
            let value = raw_value.strip_prefix("choice:").unwrap_or(raw_value);
            if options.iter().any(|option| option.id == value) {
                Ok(InputValue::Choice(value.to_owned()))
            } else {
                Err(EngineError::InvalidSelection(format!(
                    "input {feature_id:?}/{id:?} has no option {value:?}"
                )))
            }
        }
        InputSpec::Integer { id, min, max, .. } => {
            let value = raw_value.strip_prefix("integer:").unwrap_or(raw_value);
            let value = value.parse::<i64>().map_err(|_| {
                EngineError::InvalidSelection(format!(
                    "input {feature_id:?}/{id:?} expects an integer, got {raw_value:?}"
                ))
            })?;
            if !(*min..=*max).contains(&value) {
                return Err(EngineError::InvalidSelection(format!(
                    "input {feature_id:?}/{id:?} must be in {min}..={max}, got {value}"
                )));
            }
            Ok(InputValue::Integer(value))
        }
    }
}

fn base_effective_features(
    manifest: &Manifest,
    desired: &BTreeMap<String, bool>,
) -> BTreeMap<String, bool> {
    let mut selected = desired.clone();
    for _ in 0..manifest.collection.features.len().saturating_add(1) {
        let previous = selected.clone();
        for feature in manifest
            .collection
            .features
            .iter()
            .filter(|feature| feature.decision != Decision::Excluded)
        {
            let requested = match feature.decision {
                Decision::Mandatory => true,
                _ => desired.get(&feature.id).copied().unwrap_or(false),
            };
            let parent_selected = feature
                .parent
                .as_ref()
                .is_none_or(|parent| previous.get(parent).copied().unwrap_or(false));
            let requirements_selected = feature
                .requires
                .iter()
                .all(|required| previous.get(required).copied().unwrap_or(false));
            selected.insert(
                feature.id.clone(),
                requested
                    && feature.readiness != Readiness::Blocked
                    && parent_selected
                    && requirements_selected,
            );
        }
        if selected == previous {
            break;
        }
    }
    selected
}

fn unavailable_reason(
    feature: &Feature,
    features: &BTreeMap<&str, &Feature>,
    base_effective: &BTreeMap<String, bool>,
    effective: &BTreeMap<String, bool>,
) -> Option<String> {
    if feature.readiness == Readiness::Blocked {
        return feature.unavailable_reason.clone();
    }
    if let Some(parent) = &feature.parent {
        if !effective.get(parent).copied().unwrap_or(false) {
            let title = features
                .get(parent.as_str())
                .map_or(parent.as_str(), |parent| parent.title.as_str());
            return Some(format!("Available when {title} is selected."));
        }
    }
    for required in &feature.requires {
        if !effective.get(required).copied().unwrap_or(false) {
            let title = features
                .get(required.as_str())
                .map_or(required.as_str(), |required| required.title.as_str());
            return Some(format!("Requires {title}."));
        }
    }
    feature
        .conflicts
        .iter()
        .find(|conflict| {
            base_effective
                .get(&conflict.feature_id)
                .copied()
                .unwrap_or(false)
        })
        .map(|conflict| conflict.reason.clone())
}

fn build_plan(
    manifest: &Manifest,
    effective: &BTreeMap<String, bool>,
    normalized: &NormalizedSelection,
) -> Result<InstallPlan> {
    let selected_components = manifest
        .collection
        .features
        .iter()
        .filter(|feature| effective.get(&feature.id).copied().unwrap_or(false))
        .flat_map(|feature| feature.components.iter().cloned())
        .collect::<BTreeSet<_>>();

    let mut runs = Vec::new();
    for run in &manifest.collection.runs {
        let components = run
            .components
            .iter()
            .copied()
            .filter(|component| {
                selected_components.contains(&ComponentRef {
                    run_id: run.run_id.clone(),
                    component: *component,
                })
            })
            .collect::<Vec<_>>();
        if components.is_empty() {
            continue;
        }
        let mod_file = manifest.mods.get(&run.mod_id).ok_or_else(|| {
            EngineError::InvalidSelection(format!(
                "run {:?} references unknown installer {:?}",
                run.run_id, run.mod_id
            ))
        })?;
        let mut prompt_scripts = Vec::new();
        for component in &components {
            let script = render_prompt_script(
                manifest,
                &ComponentRef {
                    run_id: run.run_id.clone(),
                    component: *component,
                },
                normalized,
            )?;
            if !script.steps.is_empty() {
                prompt_scripts.push(script);
            }
        }
        runs.push(PlannedRun {
            run_id: run.run_id.clone(),
            mod_id: run.mod_id.clone(),
            target: run.phase.game_root(),
            phase: run.phase,
            components,
            args: run.args.clone(),
            artifact_id: mod_file.artifact_id.clone(),
            weidu_artifact_id: mod_file.weidu_artifact_id.clone(),
            prompt_scripts,
        });
    }

    Ok(InstallPlan { runs })
}

fn render_literal(value: &InputValue) -> String {
    match value {
        InputValue::Boolean(value) => value.to_string(),
        InputValue::Choice(value) => value.clone(),
        InputValue::Integer(value) => value.to_string(),
    }
}

fn render_input(spec: &InputSpec, value: &InputValue) -> Result<String> {
    match (spec, value) {
        (InputSpec::Boolean { .. }, InputValue::Boolean(value)) => Ok(value.to_string()),
        (InputSpec::Integer { min, max, .. }, InputValue::Integer(value))
            if (*min..=*max).contains(value) =>
        {
            Ok(value.to_string())
        }
        (InputSpec::Choice { options, .. }, InputValue::Choice(value)) => options
            .iter()
            .find(|option| option.id == *value)
            .map(|option| option.answer.clone())
            .ok_or_else(|| {
                EngineError::InvalidSelection(format!(
                    "normalized choice value {value:?} is not an authored option"
                ))
            }),
        (spec, value) => Err(EngineError::InvalidSelection(format!(
            "normalized value {value:?} does not match input {:?}",
            spec.id()
        ))),
    }
}
