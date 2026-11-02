//! Read-only comparisons for an installation's preserved choices, not an Apply contract.
mod selection;
pub use selection::{preserve_installed_selection, SelectionMigrationIssue};

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    manifest::GameRoot,
    recipe_view::{evaluate, NormalizedSelection},
    resolve::{InstallPlan, PlannedRun},
    session::FrozenIdentity,
    Manifest,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComparisonState {
    ComparisonReady,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstalledRunState {
    Unchanged,
    Unassessed,
    RequiresMigration,
}

/// A change to one actually selected installer run. This is never an execution plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledRunChange {
    pub run_id: String,
    pub mod_name: String,
    pub installed_version: String,
    pub target_version: Option<String>,
    pub components: Vec<u32>,
    pub state: InstalledRunState,
    pub detail: String,
}

/// An honest comparison: no Apply token and no inference of save compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledUpdatePlan {
    pub install_id: String,
    pub base_recipe_version: String,
    pub target_recipe_version: String,
    pub state: ComparisonState,
    pub detail: String,
    pub choices_preserved: bool,
    pub issues: Vec<SelectionMigrationIssue>,
    pub runs: Vec<InstalledRunChange>,
}

/// Caller supplies a verified frozen recipe/selection/plan and verified recovery replacements.
/// Live receipt, log and history checks belong to the ID-based native/CLI boundary.
pub struct InstalledRecipeBaseline<'a> {
    pub install_id: &'a str,
    pub recipe_version: &'a str,
    pub manifest: &'a Manifest,
    pub selection: &'a NormalizedSelection,
    pub plan: &'a InstallPlan,
    pub recovered_artifacts: &'a BTreeMap<String, FrozenIdentity>,
}

pub fn compare_installed_recipe(
    baseline: &InstalledRecipeBaseline<'_>,
    target: &Manifest,
    target_version: &str,
) -> InstalledUpdatePlan {
    let mut result = InstalledUpdatePlan {
        install_id: baseline.install_id.to_owned(),
        base_recipe_version: baseline.recipe_version.to_owned(),
        target_recipe_version: target_version.to_owned(),
        state: ComparisonState::Blocked,
        detail: "This installation needs a closer check before its mods can be compared.".into(),
        choices_preserved: false,
        issues: Vec::new(),
        runs: Vec::new(),
    };
    let problem = |rule: &str, message: &str| SelectionMigrationIssue {
        rule: rule.into(),
        feature_id: None,
        message: message.into(),
    };
    match evaluate(baseline.manifest, &baseline.selection.to_selection()) {
        Ok(old) if old.plan == *baseline.plan => {}
        _ => {
            result.issues.push(problem(
                "baseline-plan-mismatch",
                "The recorded mod order does not match this installation's original choices.",
            ));
            return result;
        }
    }
    let preserved =
        match preserve_installed_selection(baseline.manifest, target, baseline.selection) {
            Ok(value) => value,
            Err(issues) => {
                result.issues = issues;
                return result;
            }
        };
    result.choices_preserved = true;
    if baseline.manifest.collection.game_build != target.collection.game_build {
        result.issues.push(problem("game-build-changed", "The new collection requires a different game version. A mod update alone cannot change the underlying game."));
    }
    for run in &baseline.plan.runs {
        let old_mod = &baseline.manifest.mods[&run.mod_id];
        let old_artifact = &baseline.manifest.artifacts[&run.artifact_id];
        let recovered = baseline.recovered_artifacts.get(&run.run_id);
        let old_version = recovered.map_or(old_artifact.version.as_str(), |value| &value.version);
        let old_hash = recovered.map_or(old_artifact.source.sha256.as_str(), |value| &value.sha256);
        let next = preserved
            .plan
            .runs
            .iter()
            .find(|candidate| candidate.run_id == run.run_id);
        let mut entry = InstalledRunChange {
            run_id: run.run_id.clone(), mod_name: old_mod.name.clone(),
            installed_version: old_version.to_owned(), target_version: None,
            components: run.components.clone(), state: InstalledRunState::RequiresMigration,
            detail: "This installed mod no longer has the same place in the new setup. Nothing will be removed automatically.".into(),
        };
        if let Some(next) = next {
            let next_mod = &target.mods[&next.mod_id];
            let next_artifact = &target.artifacts[&next.artifact_id];
            entry.target_version = Some(next_artifact.version.clone());
            let same_structure = run.mod_id == next.mod_id
                && run.target == next.target
                && run.phase == next.phase
                && run.components == next.components
                && old_mod.language == next_mod.language
                && old_mod.tp2.eq_ignore_ascii_case(&next_mod.tp2)
                && run_order(baseline.plan, run.target) == run_order(&preserved.plan, run.target);
            if !same_structure {
                entry.detail = "Component choices, language or install order changed. An explicit upgrade route is needed; nothing will be added or removed automatically.".into();
            } else {
                let same_settings = run.args == next.args
                    && run.prompt_scripts == next.prompt_scripts
                    && run.postconditions == next.postconditions
                    && old_mod.invocation_mode == next_mod.invocation_mode
                    && run.components.iter().all(|id| {
                        old_mod
                            .components
                            .iter()
                            .find(|c| c.id == *id)
                            .map(|c| &c.stdin)
                            == next_mod
                                .components
                                .iter()
                                .find(|c| c.id == *id)
                                .map(|c| &c.stdin)
                    });
                let same_tool = baseline.manifest.artifacts[&run.weidu_artifact_id].tool
                    == target.artifacts[&next.weidu_artifact_id].tool;
                let same_payload = old_hash.eq_ignore_ascii_case(&next_artifact.source.sha256)
                    && old_artifact.archive == next_artifact.archive;
                if same_settings && same_tool && same_payload {
                    entry.state = InstalledRunState::Unchanged;
                    entry.detail = "The selected components use the same mod files and install settings. This is not a scan of every game file.".into();
                } else {
                    entry.state = InstalledRunState::Unassessed;
                    entry.detail = "The mod files or install settings changed. Updating this folder and continuing existing saves have not yet been approved for this change.".into();
                }
            }
        }
        result.runs.push(entry);
    }
    // An existing semantic feature can gain components; never silently adopt that expansion.
    if preserved.plan.runs.iter().any(|run| {
        !baseline
            .plan
            .runs
            .iter()
            .any(|old| old.run_id == run.run_id)
    }) {
        result.issues.push(problem("new-installed-run", "An existing choice now includes an additional mod run. Adding it requires an explicit decision."));
    }
    if result.issues.is_empty() {
        result.state = ComparisonState::ComparisonReady;
        result.detail = "Your choices were kept. This comparison does not change game files or establish that a mod can be updated safely.".into();
    }
    result
}

fn run_order(plan: &InstallPlan, target: GameRoot) -> Vec<&str> {
    plan.runs
        .iter()
        .filter(|run: &&PlannedRun| run.target == target)
        .map(|run| run.run_id.as_str())
        .collect()
}
