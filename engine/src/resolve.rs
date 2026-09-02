//! Resolve a validated recipe into exact ordered installer runs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, Result};
use crate::manifest::{GameRoot, Phase, RunArg};
use crate::Manifest;

/// User overrides applied to a recipe.
///
/// Recipe-v2 has no component selectors yet. The platform remains explicit so
/// unsupported targets are rejected rather than silently filtering runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Requested target platform; recipe-v2 alpha supports Windows only.
    pub platform: String,
    /// Reserved semantic choices, empty until curated selection is introduced.
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
    /// Artifact containing the installer payload.
    pub artifact_id: String,
    /// Artifact containing the WeiDU executable.
    pub weidu_artifact_id: String,
}

/// Fully resolved install work in recipe order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallPlan {
    /// Exact installer runs in recipe order.
    pub runs: Vec<PlannedRun>,
}

/// Resolves a validated recipe and semantic selection into an install plan.
///
/// Recipe-v2 alpha is Windows-only. Other target values fail explicitly; no
/// installer run is removed because of platform metadata.
pub fn resolve(manifest: &Manifest, selection: &Selection) -> Result<InstallPlan> {
    crate::validate::check(manifest)?;

    if selection.platform != "windows" {
        return Err(EngineError::InvalidSelection(format!(
            "unsupported platform {:?}; recipe-v2 alpha supports windows",
            selection.platform
        )));
    }

    if !selection.choices.is_empty() {
        return Err(EngineError::InvalidSelection(
            "recipe-v2 does not define semantic choices yet".to_owned(),
        ));
    }

    let mut runs = Vec::with_capacity(manifest.collection.runs.len());
    for run in &manifest.collection.runs {
        let Some(mod_file) = manifest.mods.get(&run.mod_id) else {
            return Err(EngineError::InvalidSelection(format!(
                "run {:?} references unknown installer {:?}",
                run.run_id, run.mod_id
            )));
        };
        runs.push(PlannedRun {
            run_id: run.run_id.clone(),
            mod_id: run.mod_id.clone(),
            target: run.phase.game_root(),
            phase: run.phase,
            components: run.components.clone(),
            args: run.args.clone(),
            artifact_id: mod_file.artifact_id.clone(),
            weidu_artifact_id: mod_file.weidu_artifact_id.clone(),
        });
    }

    Ok(InstallPlan { runs })
}
