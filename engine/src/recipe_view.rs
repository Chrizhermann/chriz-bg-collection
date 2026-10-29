//! Engine-owned projection and evaluation of curated recipe features.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, Result};
use crate::manifest::{
    ComponentRef, Decision, Feature, FeatureGroup, FeatureGroupVariant, InputSpec, InputValue,
    PromptAnswer, Readiness,
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
    /// Source mod name for context and searching.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_label: Option<String>,
    /// Shared player-facing context for these options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_label: Option<String>,
    /// Explicit same-purpose alternative group, when authored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice_group: Option<String>,
    /// Whether replacing only this group's siblings would select this option.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice_available: Option<bool>,
    /// Curated decision.
    pub decision: Decision,
    /// Release readiness.
    pub readiness: Readiness,
    /// Parent semantic id, when present.
    pub parent: Option<String>,
    /// Semantic feature ids this feature requires.
    pub requires: Vec<String>,
    /// Alternative prerequisites: at least one must be selected when non-empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires_any: Vec<String>,
    /// Semantic feature ids this feature conflicts with, as authored.
    pub conflicts: Vec<String>,
    /// Effective selected state after readiness and compatibility evaluation.
    pub selected: bool,
    /// Whether the player can change this feature directly.
    pub interactive: bool,
    /// Authored or engine-derived reason the feature cannot currently resolve.
    pub unavailable_reason: Option<String>,
    /// Typed inputs rendered beneath this feature.
    pub inputs: Vec<InputControl>,
}

/// Authored player-facing feature group, projected for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureGroupView {
    /// Stable kebab-case group id.
    pub id: String,
    /// Player-facing title, also used to mark each member control.
    pub title: String,
    /// Player-facing explanation.
    pub description: String,
    /// Member feature ids in authored order.
    pub members: Vec<String>,
    /// Choice-group variants in authored order; omitted when the group has none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<FeatureGroupVariantView>,
}

/// One choice-group alternative that a feature group picks while it is on, for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureGroupVariantView {
    /// Authored choice group the variant switches.
    pub choice_group: String,
    /// Option picked while the group is on.
    pub option: String,
    /// Player-facing name: the authored label, or the option title.
    pub label: String,
    /// Every selectable option of the choice group, in authored feature order.
    pub options: Vec<String>,
    /// Options the recipe selects by default; switching the group off restores exactly these.
    pub recipe_default: Vec<String>,
}

fn group_view(manifest: &Manifest, group: &FeatureGroup) -> FeatureGroupView {
    FeatureGroupView {
        id: group.id.clone(),
        title: group.title.clone(),
        description: group.description.clone(),
        members: group.members.clone(),
        variants: group
            .variants
            .iter()
            .map(|variant| {
                let options = choice_group_options(manifest, &variant.choice_group);
                let title = options
                    .iter()
                    .find(|option| option.id == variant.option)
                    .map(|option| option.title.clone());
                FeatureGroupVariantView {
                    choice_group: variant.choice_group.clone(),
                    option: variant.option.clone(),
                    label: variant
                        .label
                        .clone()
                        .or(title)
                        .unwrap_or_else(|| variant.option.clone()),
                    options: options.iter().map(|option| option.id.clone()).collect(),
                    recipe_default: options
                        .iter()
                        .filter(|option| option.decision == Decision::Default)
                        .map(|option| option.id.clone())
                        .collect(),
                }
            })
            .collect(),
    }
}

/// Selectable options of one authored choice group, in authored feature order.
fn choice_group_options<'a>(manifest: &'a Manifest, choice_group: &str) -> Vec<&'a Feature> {
    manifest
        .collection
        .features
        .iter()
        .filter(|feature| {
            feature.decision != Decision::Excluded
                && feature.choice_group.as_deref() == Some(choice_group)
        })
        .collect()
}

/// Ordered engine projection rendered by the frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeView {
    /// Category ids in first-authored-feature order.
    pub categories: Vec<String>,
    /// Visible controls in authored feature order.
    pub controls: Vec<FeatureControl>,
    /// Authored feature groups in recipe order; omitted when the recipe has none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<FeatureGroupView>,
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

/// Derived state of one feature group; never stored, always computed from its members.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureGroupState {
    /// Every currently available member is selected, and at least one member is available.
    On,
    /// No member is selected.
    Off,
    /// Some, but not all, available members are selected.
    Mixed,
}

/// Requested group switch, expanded into ordinary member feature changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureGroupTarget {
    /// Select every currently available member.
    On,
    /// Deselect every member.
    Off,
}

/// Derived state of one group variant under the current selection; never stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureGroupVariantState {
    /// Choice group the variant switches.
    pub choice_group: String,
    /// Option picked while the group is on.
    pub option: String,
    /// Whether the option is currently effective.
    pub selected: bool,
    /// Whether picking the option, and deselecting its alternatives, would make it effective.
    pub available: bool,
    /// Ordinary reason the option cannot be picked; present only when unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
}

/// A group member that cannot currently be selected, with the ordinary feature reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnavailableGroupMember {
    /// Member feature id.
    pub feature_id: String,
    /// Player-facing reason, identical to the member control's unavailable reason.
    pub reason: String,
}

/// Evaluated state of one authored feature group under the current selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureGroupEvaluation {
    /// Stable group id.
    pub id: String,
    /// Derived on/off/mixed state.
    pub state: FeatureGroupState,
    /// Members whose effective state is selected, in authored order.
    pub selected_members: Vec<String>,
    /// Members that can currently be selected, in authored order.
    pub available_members: Vec<String>,
    /// Members whose requirements are unmet, in authored order.
    pub unavailable_members: Vec<UnavailableGroupMember>,
    /// Variant states in authored order; omitted when the group has none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<FeatureGroupVariantState>,
    /// Why the group control is disabled; present only when no member or variant is available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
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
    /// Derived feature-group states in recipe order; omitted when the recipe has none.
    /// Deliberately separate from the normalized selection so groups never alter its digest.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<FeatureGroupEvaluation>,
}

impl SelectionEvaluation {
    /// Looks up one evaluated feature group by id.
    pub fn group(&self, id: &str) -> Option<&FeatureGroupEvaluation> {
        self.groups.iter().find(|group| group.id == id)
    }
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
    let effective = effective_features(manifest, &base_effective);

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
        let choice_preview = preview_choice(manifest, feature, &features, &desired);
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
            && !under_unrequested_optional_ancestor(&feature.id, &features, &desired)
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
            source_label: feature.source_label.clone(),
            group_label: feature.group_label.clone(),
            choice_group: feature.choice_group.clone(),
            choice_available: choice_preview.as_ref().map(|(available, _)| *available),
            decision: feature.decision,
            readiness: feature.readiness,
            parent: feature.parent.clone(),
            requires: feature.requires.clone(),
            requires_any: feature.requires_any.clone(),
            conflicts: feature
                .conflicts
                .iter()
                .map(|conflict| conflict.feature_id.clone())
                .collect(),
            selected,
            interactive,
            unavailable_reason: choice_preview.map_or(reason, |(_, reason)| reason),
            inputs: input_controls,
        });
    }

    let groups = manifest
        .collection
        .feature_groups
        .iter()
        .map(|group| {
            let variants = group
                .variants
                .iter()
                .map(|variant| evaluate_variant(manifest, variant, &features, &desired, &effective))
                .collect();
            evaluate_group(group, &controls, variants)
        })
        .collect();
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
            groups: manifest
                .collection
                .feature_groups
                .iter()
                .map(|group| group_view(manifest, group))
                .collect(),
        },
        normalized_selection,
        findings,
        plan,
        groups,
    })
}

/// Expands a group switch into member feature changes under `selection`.
///
/// `on` selects every member that is available under the current selection and, for each
/// available variant, picks its option and deselects the choice group's other options; members
/// and variants whose requirements are unmet are left unchanged and never forced on. `off`
/// deselects every member and restores each variant's choice group to its recipe default.
/// No feature outside the members and the variants' choice groups is ever changed; the caller
/// applies the result as ordinary feature choices, so later individual feature choices still
/// take precedence.
pub fn feature_group_changes(
    manifest: &Manifest,
    selection: &Selection,
    group_id: &str,
    target: FeatureGroupTarget,
) -> Result<BTreeMap<String, bool>> {
    let Some(index) = manifest
        .collection
        .feature_groups
        .iter()
        .position(|group| group.id == group_id)
    else {
        let valid = manifest
            .collection
            .feature_groups
            .iter()
            .map(|group| group.id.as_str())
            .collect::<Vec<_>>();
        return Err(EngineError::InvalidSelection(if valid.is_empty() {
            format!("unknown feature group {group_id:?}; this recipe defines no feature groups")
        } else {
            format!(
                "unknown feature group {group_id:?}; valid groups: {}",
                valid.join(", ")
            )
        }));
    };
    let group = &manifest.collection.feature_groups[index];
    match target {
        FeatureGroupTarget::Off => {
            let mut changes = group
                .members
                .iter()
                .map(|member| (member.clone(), false))
                .collect::<BTreeMap<_, _>>();
            for variant in &group.variants {
                for option in choice_group_options(manifest, &variant.choice_group) {
                    changes.insert(option.id.clone(), option.decision == Decision::Default);
                }
            }
            Ok(changes)
        }
        FeatureGroupTarget::On => {
            let evaluation = evaluate(manifest, selection)?;
            let state = &evaluation.groups[index];
            if let Some(reason) = &state.disabled_reason {
                return Err(EngineError::InvalidSelection(format!(
                    "feature group {group_id:?} cannot be turned on: {reason}"
                )));
            }
            let mut changes = state
                .available_members
                .iter()
                .map(|member| (member.clone(), true))
                .collect::<BTreeMap<_, _>>();
            for variant in state.variants.iter().filter(|variant| variant.available) {
                for option in choice_group_options(manifest, &variant.choice_group) {
                    changes.insert(option.id.clone(), option.id == variant.option);
                }
            }
            Ok(changes)
        }
    }
}

/// Derives one variant's state: picked when its option is effective, available when picking
/// it with the ordinary evaluator would make it effective.
fn evaluate_variant(
    manifest: &Manifest,
    variant: &FeatureGroupVariant,
    features: &BTreeMap<&str, &Feature>,
    desired: &BTreeMap<String, bool>,
    effective: &BTreeMap<String, bool>,
) -> FeatureGroupVariantState {
    const FALLBACK: &str = "Unavailable with the current choices.";
    let selected = effective.get(&variant.option).copied().unwrap_or(false);
    // Validation guarantees the option is an optional feature of its choice group.
    let (available, reason) = match features.get(variant.option.as_str()) {
        Some(option) => preview_switch(
            manifest,
            option,
            features,
            desired,
            &choice_group_options(manifest, &variant.choice_group),
        ),
        None => (false, None),
    };
    FeatureGroupVariantState {
        choice_group: variant.choice_group.clone(),
        option: variant.option.clone(),
        selected,
        available,
        unavailable_reason: (!available).then(|| reason.unwrap_or_else(|| FALLBACK.to_owned())),
    }
}

fn evaluate_group(
    group: &FeatureGroup,
    controls: &[FeatureControl],
    variants: Vec<FeatureGroupVariantState>,
) -> FeatureGroupEvaluation {
    let mut selected_members = Vec::new();
    let mut available_members = Vec::new();
    let mut unavailable_members = Vec::new();
    for member in &group.members {
        // Validation guarantees that every member is a visible optional feature.
        let Some(control) = controls.iter().find(|control| control.id == *member) else {
            continue;
        };
        if control.selected {
            selected_members.push(member.clone());
        }
        if control.interactive && control.unavailable_reason.is_none() {
            available_members.push(member.clone());
        } else {
            unavailable_members.push(UnavailableGroupMember {
                feature_id: member.clone(),
                reason: control
                    .unavailable_reason
                    .clone()
                    .unwrap_or_else(|| "Unavailable with the current choices.".to_owned()),
            });
        }
    }
    // A variant counts like a member: picked when its option is effective, and an unavailable
    // variant does not hold the group in mixed.
    let state = if selected_members.is_empty() && !variants.iter().any(|variant| variant.selected) {
        FeatureGroupState::Off
    } else if available_members
        .iter()
        .all(|member| selected_members.contains(member))
        && variants
            .iter()
            .all(|variant| variant.selected || !variant.available)
    {
        FeatureGroupState::On
    } else {
        FeatureGroupState::Mixed
    };
    let disabled_reason = (available_members.is_empty()
        && !variants.iter().any(|variant| variant.available))
    .then(|| {
        format!(
            "No {} change is available with the current choices.",
            group.title
        )
    });
    FeatureGroupEvaluation {
        id: group.id.clone(),
        state,
        selected_members,
        available_members,
        unavailable_members,
        variants,
        disabled_reason,
    }
}

/// Renders one component's prompt steps without writing to stdin or starting a process.
pub fn render_prompt_script(
    manifest: &Manifest,
    component_ref: &ComponentRef,
    selection: &NormalizedSelection,
) -> Result<PromptScript> {
    let base_effective = base_effective_features(manifest, &selection.features);
    let effective = effective_features(manifest, &base_effective);
    render_prompt_script_for_effective(manifest, component_ref, selection, &effective)
}

/// Preview an atomic switch with the ordinary evaluator. Metadata alone must
/// never waive a prerequisite or a conflict with another mod.
fn preview_choice(
    manifest: &Manifest,
    feature: &Feature,
    features: &BTreeMap<&str, &Feature>,
    desired: &BTreeMap<String, bool>,
) -> Option<(bool, Option<String>)> {
    let group = feature
        .choice_group
        .as_deref()
        .filter(|id| !id.trim().is_empty())?;
    feature
        .group_label
        .as_deref()
        .filter(|label| !label.trim().is_empty())?;
    let peers = manifest
        .collection
        .features
        .iter()
        .filter(|peer| peer.choice_group.as_deref() == Some(group))
        .collect::<Vec<_>>();
    if peers.len() < 2
        || peers.iter().any(|peer| {
            matches!(peer.decision, Decision::Excluded | Decision::Mandatory)
                || peer.category != feature.category
                || peer.parent != feature.parent
                || peer.group_label != feature.group_label
                || peers.iter().any(|other| {
                    peer.id != other.id
                        && !peer
                            .conflicts
                            .iter()
                            .any(|conflict| conflict.feature_id == other.id)
                })
        })
    {
        return None;
    }
    Some(preview_switch(manifest, feature, features, desired, &peers))
}

/// Evaluates selecting `feature` while deselecting the other `peers`, with the ordinary
/// evaluator, and returns whether it would be effective and its reason otherwise.
fn preview_switch(
    manifest: &Manifest,
    feature: &Feature,
    features: &BTreeMap<&str, &Feature>,
    desired: &BTreeMap<String, bool>,
    peers: &[&Feature],
) -> (bool, Option<String>) {
    let mut candidate = desired.clone();
    for peer in peers {
        candidate.insert(peer.id.clone(), peer.id == feature.id);
    }
    candidate.insert(feature.id.clone(), true);
    let base = base_effective_features(manifest, &candidate);
    let effective = effective_features(manifest, &base);
    (
        effective.get(&feature.id).copied().unwrap_or(false),
        unavailable_reason(feature, features, &base, &effective),
    )
}

fn render_prompt_script_for_effective(
    manifest: &Manifest,
    component_ref: &ComponentRef,
    selection: &NormalizedSelection,
    effective: &BTreeMap<String, bool>,
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
        if !prompt.when_any_features.is_empty()
            && !prompt
                .when_any_features
                .iter()
                .any(|feature_id| effective.get(feature_id).copied().unwrap_or(false))
        {
            continue;
        }
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
            let any_requirement_selected = feature.requires_any.is_empty()
                || feature
                    .requires_any
                    .iter()
                    .any(|required| previous.get(required).copied().unwrap_or(false));
            selected.insert(
                feature.id.clone(),
                requested
                    && feature.readiness != Readiness::Blocked
                    && parent_selected
                    && requirements_selected
                    && any_requirement_selected,
            );
        }
        if selected == previous {
            break;
        }
    }
    selected
}

fn effective_features(
    manifest: &Manifest,
    base_effective: &BTreeMap<String, bool>,
) -> BTreeMap<String, bool> {
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
                selected &= feature.requires_any.is_empty()
                    || feature
                        .requires_any
                        .iter()
                        .any(|required| previous.get(required).copied().unwrap_or(false));
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
    effective
}

/// Returns whether an optional ancestor of `feature_id` is not requested by the selection.
///
/// A default or mandatory feature beneath such a parent only describes what ticking the
/// parent would add; it is not a requested outcome, so leaving it out is not an omission. A
/// default or mandatory ancestor never qualifies, so a feature that drops out while its
/// parents are requested is still reported.
pub(crate) fn under_unrequested_optional_ancestor(
    feature_id: &str,
    features: &BTreeMap<&str, &Feature>,
    desired: &BTreeMap<String, bool>,
) -> bool {
    let mut current = features
        .get(feature_id)
        .and_then(|feature| feature.parent.as_deref());
    // Validation rejects parent cycles; the bound only keeps a malformed chain finite.
    for _ in 0..=features.len() {
        let Some(ancestor) = current.and_then(|id| features.get(id)) else {
            return false;
        };
        if ancestor.decision == Decision::Optional
            && !desired.get(&ancestor.id).copied().unwrap_or(false)
        {
            return true;
        }
        current = ancestor.parent.as_deref();
    }
    false
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
    if !feature.requires_any.is_empty()
        && !feature
            .requires_any
            .iter()
            .any(|required| effective.get(required).copied().unwrap_or(false))
    {
        let titles = feature
            .requires_any
            .iter()
            .map(|required| {
                features
                    .get(required.as_str())
                    .map_or(required.as_str(), |required| required.title.as_str())
            })
            .collect::<Vec<_>>();
        return Some(format!("Requires at least one of: {}.", titles.join(", ")));
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
            let script = render_prompt_script_for_effective(
                manifest,
                &ComponentRef {
                    run_id: run.run_id.clone(),
                    component: *component,
                },
                normalized,
                effective,
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
            postconditions: run.postconditions.clone(),
            artifact_id: mod_file.artifact_id.clone(),
            weidu_artifact_id: mod_file.weidu_artifact_id.clone(),
            prompt_scripts,
        });
    }

    Ok(InstallPlan { runs })
}

/// Why an installation cannot continue without some mods.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LeaveOutError {
    /// The recipe always installs a part of a mod that would have to be left out.
    #[error(
        "{title} is a fixed part of this collection, so the installation cannot continue without it"
    )]
    Required {
        /// Feature that the recipe always installs.
        feature_id: String,
        /// Its display title.
        title: String,
    },
    /// The frozen recipe, selection and plan do not fit together.
    #[error("the remaining plan cannot be built: {0}")]
    Invalid(String),
}

/// The part of `plan` that remains when the mods installed from the `unusable` artifacts are
/// left out, together with every selected feature that depends on them through a parent,
/// `requires` or `requires_any` relation.
///
/// `plan` is the frozen execution plan, possibly split at manual pauses. The result only
/// removes runs, components and the prompt answers that go with them: a feature the selection
/// did not install is never added, even when it was off only because it conflicted with a
/// removed feature. Prompt answers are rendered again because they may depend on removed
/// features, and a run that lost components drops its recipe checks, which describe the
/// complete run. A mandatory part of an optional feature removes that whole feature; a
/// feature the recipe always installs (mandatory, with only mandatory parents) cannot be
/// removed, so then this fails.
pub fn plan_without_artifacts(
    manifest: &Manifest,
    selection: &NormalizedSelection,
    plan: &InstallPlan,
    unusable: &BTreeSet<String>,
) -> std::result::Result<InstallPlan, LeaveOutError> {
    let features = manifest
        .collection
        .features
        .iter()
        .map(|feature| (feature.id.as_str(), feature))
        .collect::<BTreeMap<_, _>>();
    let effective = effective_features(
        manifest,
        &base_effective_features(manifest, &selection.features),
    );
    let blocked_runs = manifest
        .collection
        .runs
        .iter()
        .filter(|run| {
            manifest
                .mods
                .get(&run.mod_id)
                .is_some_and(|installer| unusable.contains(&installer.artifact_id))
        })
        .map(|run| run.run_id.as_str())
        .collect::<BTreeSet<_>>();
    let uses_blocked = |feature: &Feature| {
        feature
            .components
            .iter()
            .any(|component| blocked_runs.contains(component.run_id.as_str()))
    };
    let is_on = |map: &BTreeMap<String, bool>, id: &str| map.get(id).copied().unwrap_or(false);

    let mut desired = selection.features.clone();
    for feature in manifest
        .collection
        .features
        .iter()
        .filter(|feature| is_on(&effective, &feature.id) && uses_blocked(feature))
    {
        // Switch off the feature itself, or its nearest ancestor the selection can switch off.
        let mut current = feature;
        let mut depth = 0_usize;
        while current.decision == Decision::Mandatory {
            match current.parent.as_deref().and_then(|id| features.get(id)) {
                Some(parent) if depth <= features.len() => {
                    current = parent;
                    depth += 1;
                }
                _ => {
                    return Err(LeaveOutError::Required {
                        feature_id: feature.id.clone(),
                        title: feature.title.clone(),
                    })
                }
            }
        }
        desired.insert(current.id.clone(), false);
    }
    let reduced = effective_features(manifest, &base_effective_features(manifest, &desired));
    let kept = effective
        .iter()
        .map(|(id, selected)| (id.clone(), *selected && is_on(&reduced, id)))
        .collect::<BTreeMap<_, _>>();
    if let Some(feature) = manifest
        .collection
        .features
        .iter()
        .find(|feature| is_on(&kept, &feature.id) && uses_blocked(feature))
    {
        return Err(LeaveOutError::Invalid(format!(
            "feature {:?} still installs a left-out mod",
            feature.id
        )));
    }
    let selected_components = manifest
        .collection
        .features
        .iter()
        .filter(|feature| is_on(&kept, &feature.id))
        .flat_map(|feature| feature.components.iter().cloned())
        .collect::<BTreeSet<_>>();

    let mut runs = Vec::new();
    for run in &plan.runs {
        let base = manifest
            .collection
            .runs
            .iter()
            .find(|candidate| candidate.run_id == run.run_id)
            .or_else(|| {
                manifest.collection.runs.iter().find(|candidate| {
                    run.run_id
                        .strip_prefix(candidate.run_id.as_str())
                        .is_some_and(|rest| rest.starts_with("--manual-"))
                })
            })
            .ok_or_else(|| {
                LeaveOutError::Invalid(format!("planned run {:?} is not in the recipe", run.run_id))
            })?;
        let component_ref = |component: u32| ComponentRef {
            run_id: base.run_id.clone(),
            component,
        };
        let components = run
            .components
            .iter()
            .copied()
            .filter(|component| selected_components.contains(&component_ref(*component)))
            .collect::<Vec<_>>();
        if components.is_empty() {
            continue;
        }
        let mut prompt_scripts = Vec::new();
        for component in &components {
            let script = render_prompt_script_for_effective(
                manifest,
                &component_ref(*component),
                selection,
                &kept,
            )
            .map_err(|error| LeaveOutError::Invalid(error.to_string()))?;
            if !script.steps.is_empty() {
                prompt_scripts.push(script);
            }
        }
        let mut remaining = run.clone();
        if components != run.components {
            remaining.postconditions.clear();
        }
        remaining.components = components;
        remaining.prompt_scripts = prompt_scripts;
        runs.push(remaining);
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
