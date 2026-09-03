//! Resolve a validated recipe into exact ordered installer runs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::manifest::{GameRoot, InputValue, Phase, Postcondition, RunArg};
use crate::recipe_view::PromptScript;
use crate::Manifest;

/// User overrides applied to a recipe.
///
/// Values are keyed by stable feature ids or `feature-id/input-id`; component
/// numbers never appear in user selections. The platform remains explicit so
/// unsupported targets are rejected rather than silently filtering runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Requested target platform; recipe-v2 alpha supports Windows only.
    pub platform: String,
    /// Semantic feature states (`on`/`off`) and typed input values.
    #[serde(default)]
    pub choices: BTreeMap<String, String>,
}

impl Selection {
    /// Creates an empty semantic selection for `platform`.
    pub fn defaults(platform: &str) -> Selection {
        Selection {
            platform: platform.to_owned(),
            choices: BTreeMap::new(),
        }
    }

    /// Sets one feature by semantic id.
    pub fn set_feature(&mut self, feature_id: &str, selected: bool) {
        self.choices.insert(
            feature_id.to_owned(),
            if selected { "on" } else { "off" }.to_owned(),
        );
    }

    /// Sets one typed feature input by semantic ids.
    pub fn set_input(&mut self, feature_id: &str, input_id: &str, value: InputValue) {
        let value = match value {
            InputValue::Boolean(value) => format!("boolean:{value}"),
            InputValue::Choice(value) => format!("choice:{value}"),
            InputValue::Integer(value) => format!("integer:{value}"),
        };
        self.choices
            .insert(format!("{feature_id}/{input_id}"), value);
    }
}

/// One exact ordered invocation of a WeiDU installer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedRun {
    /// Globally unique logical run id.
    pub run_id: String,
    /// Installer id executed by this run.
    pub mod_id: String,
    /// Derived staged game root targeted by this run.
    pub target: GameRoot,
    /// Install phase declared by the run.
    pub phase: Phase,
    /// Exact ordered WeiDU component numbers.
    pub components: Vec<u32>,
    /// Typed extra invocation arguments.
    pub args: Vec<RunArg>,
    /// Assertions checked after install-log reconciliation succeeds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub postconditions: Vec<Postcondition>,
    /// Artifact containing the installer payload.
    pub artifact_id: String,
    /// Artifact containing the WeiDU executable.
    pub weidu_artifact_id: String,
    /// Output-gated prompts grouped by their selected component.
    pub prompt_scripts: Vec<PromptScript>,
}

/// Fully resolved install work in recipe order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallPlan {
    /// Exact installer runs in recipe order.
    pub runs: Vec<PlannedRun>,
}

impl InstallPlan {
    /// Returns the exact ordered components for `run_id` when that run resolves.
    pub fn components_for(&self, run_id: &str) -> Option<&[u32]> {
        self.runs
            .iter()
            .find(|run| run.run_id == run_id)
            .map(|run| run.components.as_slice())
    }
}

/// Resolves a validated recipe and semantic selection into an install plan.
///
/// Recipe-v2 alpha is Windows-only. Other target values fail explicitly; no
/// installer run is removed because of platform metadata.
pub fn resolve(manifest: &Manifest, selection: &Selection) -> Result<InstallPlan> {
    Ok(crate::recipe_view::evaluate(manifest, selection)?.plan)
}
