//! Resolve a validated manifest and user selection into ordered install runs.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, Result};
use crate::manifest::{Component, ComponentRef, ModFile, Phase};
use crate::Manifest;

/// User overrides applied on top of manifest toggle and choice defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Default-on toggle ids to turn off.
    pub toggles_off: Vec<String>,
    /// Default-off toggle ids to turn on.
    pub toggles_on: Vec<String>,
    /// Choice group id to selected option id.
    pub choices: BTreeMap<String, String>,
    /// Target platform: "windows", "macos", or "linux".
    pub platform: String,
}

impl Selection {
    /// Creates a selection that uses every manifest default for the requested platform.
    pub fn defaults(platform: &str) -> Selection {
        Selection {
            toggles_off: Vec::new(),
            toggles_on: Vec::new(),
            choices: BTreeMap::new(),
            platform: platform.to_owned(),
        }
    }
}

/// One ordered invocation of a mod installer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedRun {
    /// Id of the mod installed by this run.
    pub mod_id: String,
    /// Install phase declared by the mod.
    pub phase: Phase,
    /// Resolved components in the mod's declared component order.
    pub components: Vec<Component>,
}

/// Fully resolved install work in collection order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallPlan {
    /// Installer runs, preserving separate entries for split mods.
    pub runs: Vec<PlannedRun>,
}

struct CandidateRun<'a> {
    mod_file: &'a ModFile,
    eligible_components: BTreeSet<u32>,
    enabled_components: BTreeSet<u32>,
}

/// Resolves a valid manifest and selection into an ordered install plan.
///
/// # Errors
///
/// Returns EngineError::InvalidSelection when the selection names an
/// unsupported platform, unknown toggle, unknown choice group or option, or a
/// selected option adds a component to a mod removed by platform or toggle
/// filtering.
pub fn resolve(m: &Manifest, sel: &Selection) -> Result<InstallPlan> {
    validate_selection(m, sel)?;

    let baseline_exclusions = baseline_exclusions(m);
    let mut candidates = build_candidates(m, sel.platform.as_str(), &baseline_exclusions);

    apply_toggles(m, sel, &mut candidates);
    apply_choices(m, sel, &mut candidates)?;

    Ok(InstallPlan {
        runs: materialize_runs(candidates),
    })
}

fn validate_selection(m: &Manifest, sel: &Selection) -> Result<()> {
    if !matches!(sel.platform.as_str(), "windows" | "macos" | "linux") {
        return Err(EngineError::InvalidSelection(format!(
            "unsupported platform {:?}; expected windows, macos, or linux",
            sel.platform
        )));
    }

    for toggle_id in sel.toggles_off.iter().chain(&sel.toggles_on) {
        if !m
            .collection
            .toggles
            .iter()
            .any(|toggle| toggle.id == *toggle_id)
        {
            return Err(EngineError::InvalidSelection(format!(
                "unknown toggle id {toggle_id:?}"
            )));
        }
    }

    for (group_id, option_id) in &sel.choices {
        let Some(group) = m
            .collection
            .choice_groups
            .iter()
            .find(|group| group.id == *group_id)
        else {
            return Err(EngineError::InvalidSelection(format!(
                "unknown choice group id {group_id:?}"
            )));
        };

        if !group.options.iter().any(|option| option.id == *option_id) {
            return Err(EngineError::InvalidSelection(format!(
                "unknown option id {option_id:?} for choice group {group_id:?}"
            )));
        }
    }

    Ok(())
}

fn baseline_exclusions(m: &Manifest) -> BTreeMap<&str, BTreeSet<u32>> {
    let mut exclusions = BTreeMap::new();

    for group in &m.collection.choice_groups {
        for option in &group.options {
            for component in &option.adds_components {
                exclusions
                    .entry(component.mod_id.as_str())
                    .or_insert_with(BTreeSet::new)
                    .insert(component.component);
            }
        }
    }

    exclusions
}

fn build_candidates<'a>(
    m: &'a Manifest,
    platform: &str,
    baseline_exclusions: &BTreeMap<&str, BTreeSet<u32>>,
) -> Vec<CandidateRun<'a>> {
    let mut candidates = Vec::new();

    for entry in &m.collection.order {
        let Some(mod_file) = m.mods.get(&entry.id) else {
            continue;
        };
        if !mod_file
            .platforms
            .iter()
            .any(|supported| supported == platform)
        {
            continue;
        }

        let eligible_components: BTreeSet<u32> = mod_file
            .components
            .iter()
            .filter(|component| match &entry.components {
                Some(explicit) => explicit.contains(&component.id),
                None => true,
            })
            .map(|component| component.id)
            .collect();

        let excluded_for_mod = baseline_exclusions.get(mod_file.id.as_str());
        let enabled_components = eligible_components
            .iter()
            .copied()
            .filter(|component| match excluded_for_mod {
                Some(excluded) => !excluded.contains(component),
                None => true,
            })
            .collect();

        candidates.push(CandidateRun {
            mod_file,
            eligible_components,
            enabled_components,
        });
    }

    candidates
}

fn apply_toggles<'a>(m: &'a Manifest, sel: &Selection, candidates: &mut Vec<CandidateRun<'a>>) {
    for toggle in &m.collection.toggles {
        let effective_on = (toggle.default_on && !sel.toggles_off.contains(&toggle.id))
            || sel.toggles_on.contains(&toggle.id);
        if effective_on {
            continue;
        }

        candidates.retain(|candidate| {
            !toggle
                .removes_mods
                .iter()
                .any(|mod_id| mod_id == &candidate.mod_file.id)
        });
        remove_components(candidates, &toggle.removes_components);
    }
}

fn apply_choices<'a>(
    m: &'a Manifest,
    sel: &Selection,
    candidates: &mut [CandidateRun<'a>],
) -> Result<()> {
    for group in &m.collection.choice_groups {
        let option_id = match sel.choices.get(&group.id) {
            Some(option_id) => option_id,
            None => &group.default,
        };
        let Some(option) = group.options.iter().find(|option| option.id == *option_id) else {
            continue;
        };

        remove_components(candidates, &option.removes_components);

        for component in &option.adds_components {
            let mut target_mod_survives = false;

            for candidate in candidates
                .iter_mut()
                .filter(|candidate| candidate.mod_file.id == component.mod_id)
            {
                target_mod_survives = true;
                if candidate.eligible_components.contains(&component.component) {
                    candidate.enabled_components.insert(component.component);
                }
            }

            if !target_mod_survives {
                return Err(EngineError::InvalidSelection(format!(
                    "choice option {:?} targets unavailable mod {:?}",
                    option.id, component.mod_id
                )));
            }
        }
    }

    Ok(())
}

fn remove_components(candidates: &mut [CandidateRun<'_>], components: &[ComponentRef]) {
    for component in components {
        for candidate in candidates
            .iter_mut()
            .filter(|candidate| candidate.mod_file.id == component.mod_id)
        {
            candidate.enabled_components.remove(&component.component);
        }
    }
}

fn materialize_runs(candidates: Vec<CandidateRun<'_>>) -> Vec<PlannedRun> {
    candidates
        .into_iter()
        .filter_map(|candidate| {
            let components: Vec<Component> = candidate
                .mod_file
                .components
                .iter()
                .filter(|component| {
                    candidate.eligible_components.contains(&component.id)
                        && candidate.enabled_components.contains(&component.id)
                })
                .cloned()
                .collect();

            if components.is_empty() {
                None
            } else {
                Some(PlannedRun {
                    mod_id: candidate.mod_file.id.clone(),
                    phase: candidate.mod_file.phase,
                    components,
                })
            }
        })
        .collect()
}
