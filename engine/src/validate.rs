//! Static validation of a loaded executable recipe.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component as PathComponent, Path};

use crate::error::EngineError;
use crate::manifest::{AcquisitionPolicy, GameRoot, Phase, SourceKind};
use crate::Manifest;

/// Placeholder digest used only while authoring an artifact entry.
const UNPINNED_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// How serious a [`Finding`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The recipe is usable, but something looks wrong.
    Warning,
    /// The recipe cannot be used as-is.
    Error,
}

/// One stable validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Finding severity.
    pub severity: Severity,
    /// Stable kebab-case rule id.
    pub rule: &'static str,
    /// Human-readable explanation naming the offending recipe element.
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(formatter, "{severity}[{}]: {}", self.rule, self.message)
    }
}

/// Rule requiring component ids to be unique within one installer.
pub const RULE_COMPONENT_IDS_UNIQUE: &str = "component-ids-unique";
/// Rule requiring every logical run id to be globally unique.
pub const RULE_RUN_IDS: &str = "run-ids";
/// Rule requiring the recipe and every installer to contain executable work.
pub const RULE_NONEMPTY: &str = "nonempty";
/// Rule requiring every run to list at least one component explicitly.
pub const RULE_RUN_COMPONENTS: &str = "run-components";
/// Rule requiring all artifact, installer, tool, and component references to exist.
pub const RULE_REFERENCES: &str = "references";
/// Rule requiring authored archive and TP2 paths to remain within staged roots.
pub const RULE_PATHS: &str = "paths";
/// Rule preventing one installer component from running twice against one game root.
pub const RULE_COMPONENT_PLACEMENT: &str = "component-placement";
/// Rule requiring acquisition policy and source kind to describe one coherent flow.
pub const RULE_ACQUISITION_POLICY: &str = "acquisition-policy";
/// Rule requiring fetchable source identity to be well formed.
pub const RULE_SOURCES: &str = "sources";
/// Warning emitted for an all-zero authoring digest.
pub const RULE_UNPINNED_SOURCE: &str = "unpinned-source";
/// Rule requiring recipe phases to be monotonic.
pub const RULE_PHASE_ORDER: &str = "phase-order";
/// Rule enforcing the EET initialization and finalization boundaries.
pub const RULE_EET_ANCHORS: &str = "eet-anchors";
/// Warning emitted for legacy stdin that lacks a terminating newline.
pub const RULE_STDIN_NEWLINE: &str = "stdin-newline";

/// Runs the currently defined recipe validation rules in stable order.
pub fn validate(manifest: &Manifest) -> Vec<Finding> {
    let mut findings = Vec::new();
    check_run_ids(manifest, &mut findings);
    check_nonempty(manifest, &mut findings);
    check_run_components(manifest, &mut findings);
    check_references(manifest, &mut findings);
    check_paths(manifest, &mut findings);
    check_component_ids(manifest, &mut findings);
    check_component_placement(manifest, &mut findings);
    check_acquisition_policy(manifest, &mut findings);
    check_sources(manifest, &mut findings);
    check_phase_order(manifest, &mut findings);
    check_eet_anchors(manifest, &mut findings);
    check_stdin(manifest, &mut findings);
    findings
}

/// Rejects a recipe when any validation finding has error severity.
pub fn check(manifest: &Manifest) -> crate::error::Result<()> {
    let findings = validate(manifest);
    if findings
        .iter()
        .any(|finding| finding.severity == Severity::Error)
    {
        return Err(EngineError::Validation(findings));
    }
    Ok(())
}

fn error(findings: &mut Vec<Finding>, rule: &'static str, message: String) {
    findings.push(Finding {
        severity: Severity::Error,
        rule,
        message,
    });
}

fn warning(findings: &mut Vec<Finding>, rule: &'static str, message: String) {
    findings.push(Finding {
        severity: Severity::Warning,
        rule,
        message,
    });
}

fn check_run_ids(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, run) in manifest.collection.runs.iter().enumerate() {
        if let Some(first) = seen.insert(run.run_id.as_str(), index) {
            error(
                findings,
                RULE_RUN_IDS,
                format!(
                    "run id {:?} is declared by both run {first} and run {index}",
                    run.run_id
                ),
            );
        }
    }
}

fn check_nonempty(manifest: &Manifest, findings: &mut Vec<Finding>) {
    if manifest.collection.runs.is_empty() {
        error(
            findings,
            RULE_NONEMPTY,
            "the collection contains no installer runs".to_owned(),
        );
    }

    for (id, mod_file) in &manifest.mods {
        if mod_file.components.is_empty() {
            error(
                findings,
                RULE_NONEMPTY,
                format!("installer {id:?} declares no components"),
            );
        }
    }
}

fn check_run_components(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (index, run) in manifest.collection.runs.iter().enumerate() {
        if run.components.is_empty() {
            error(
                findings,
                RULE_RUN_COMPONENTS,
                format!("run {index} ({:?}) has no explicit components", run.run_id),
            );
        }
    }
}

fn check_references(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, mod_file) in &manifest.mods {
        if !manifest.artifacts.contains_key(&mod_file.artifact_id) {
            error(
                findings,
                RULE_REFERENCES,
                format!(
                    "installer {id:?} references unknown payload artifact {:?}",
                    mod_file.artifact_id
                ),
            );
        }
        if !manifest.artifacts.contains_key(&mod_file.weidu_artifact_id) {
            error(
                findings,
                RULE_REFERENCES,
                format!(
                    "installer {id:?} references unknown WeiDU artifact {:?}",
                    mod_file.weidu_artifact_id
                ),
            );
        }
    }

    for (index, run) in manifest.collection.runs.iter().enumerate() {
        let Some(mod_file) = manifest.mods.get(&run.mod_id) else {
            error(
                findings,
                RULE_REFERENCES,
                format!(
                    "run {index} ({:?}) references unknown installer {:?}",
                    run.run_id, run.mod_id
                ),
            );
            continue;
        };

        for component in &run.components {
            if !mod_file
                .components
                .iter()
                .any(|declared| declared.id == *component)
            {
                error(
                    findings,
                    RULE_REFERENCES,
                    format!(
                        "run {index} ({:?}) references component {component}, which installer {:?} does not declare",
                        run.run_id, run.mod_id
                    ),
                );
            }
        }
    }
}

fn check_paths(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, artifact) in &manifest.artifacts {
        if !is_safe_relative_path(&artifact.archive.path) {
            error(
                findings,
                RULE_PATHS,
                format!(
                    "artifact {id:?} archive path {:?} is not a traversal-free relative path",
                    artifact.archive.path
                ),
            );
        }
    }

    for (id, mod_file) in &manifest.mods {
        if !is_safe_relative_path(&mod_file.tp2) {
            error(
                findings,
                RULE_PATHS,
                format!(
                    "installer {id:?} TP2 path {:?} is not a traversal-free relative path",
                    mod_file.tp2
                ),
            );
        }
    }
}

fn is_safe_relative_path(value: &str) -> bool {
    if value.is_empty() || value.starts_with('/') || value.starts_with('\\') {
        return false;
    }

    let first_separator = value.find(['/', '\\']).unwrap_or(value.len());
    if value[..first_separator].contains(':') {
        return false;
    }

    let path = Path::new(value);
    !path.is_absolute()
        && path.components().all(|component| {
            !matches!(
                component,
                PathComponent::ParentDir | PathComponent::RootDir | PathComponent::Prefix(_)
            )
        })
        && !value.split(['/', '\\']).any(|component| component == "..")
}

fn check_component_ids(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, mod_file) in &manifest.mods {
        let mut seen = BTreeSet::new();
        for component in &mod_file.components {
            if !seen.insert(component.id) {
                error(
                    findings,
                    RULE_COMPONENT_IDS_UNIQUE,
                    format!("installer {id:?} declares component {} twice", component.id),
                );
            }
        }
    }
}

fn check_component_placement(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let mut seen: BTreeMap<(GameRoot, &str, u32), &str> = BTreeMap::new();
    for run in &manifest.collection.runs {
        let target = run.phase.game_root();
        for component in &run.components {
            let key = (target, run.mod_id.as_str(), *component);
            if let Some(first_run) = seen.insert(key, run.run_id.as_str()) {
                error(
                    findings,
                    RULE_COMPONENT_PLACEMENT,
                    format!(
                        "installer {:?} component {component} targets {target:?} in both run {first_run:?} and run {:?}",
                        run.mod_id, run.run_id
                    ),
                );
            }
        }
    }
}

fn check_acquisition_policy(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, artifact) in &manifest.artifacts {
        let inconsistent = match artifact.acquisition {
            AcquisitionPolicy::FetchOnly | AcquisitionPolicy::BundlePermitted => {
                artifact.source.kind == SourceKind::Manual
            }
            AcquisitionPolicy::ManualUserSupplied | AcquisitionPolicy::Blocked => false,
        };

        if inconsistent {
            error(
                findings,
                RULE_ACQUISITION_POLICY,
                format!(
                    "artifact {id:?} acquisition {:?} is inconsistent with source kind {:?}",
                    artifact.acquisition, artifact.source.kind
                ),
            );
        }
    }
}

fn check_sources(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, artifact) in &manifest.artifacts {
        if matches!(
            artifact.acquisition,
            AcquisitionPolicy::ManualUserSupplied | AcquisitionPolicy::Blocked
        ) {
            continue;
        }

        if !artifact.source.url.starts_with("https://") {
            error(
                findings,
                RULE_SOURCES,
                format!(
                    "artifact {id:?} source url {:?} is not https",
                    artifact.source.url
                ),
            );
        }

        let sha256 = artifact.source.sha256.as_str();
        if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            error(
                findings,
                RULE_SOURCES,
                format!("artifact {id:?} source sha256 {sha256:?} is not 64 hex characters"),
            );
        } else if sha256 == UNPINNED_SHA256 {
            warning(
                findings,
                RULE_UNPINNED_SOURCE,
                format!("artifact {id:?} has an all-zero sha256, so its source is not pinned"),
            );
        }
    }
}

fn check_phase_order(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let mut highest: Option<(usize, &str, Phase)> = None;
    for (index, run) in manifest.collection.runs.iter().enumerate() {
        if let Some((high_index, high_id, high_phase)) = highest {
            if run.phase < high_phase {
                error(
                    findings,
                    RULE_PHASE_ORDER,
                    format!(
                        "run {index} ({:?}) has phase {:?}, but run {high_index} ({high_id:?}) already has later phase {high_phase:?}",
                        run.run_id, run.phase
                    ),
                );
                continue;
            }
        }
        highest = Some((index, run.run_id.as_str(), run.phase));
    }
}

fn check_eet_anchors(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let runs = &manifest.collection.runs;
    let has_eet_sequence = runs.iter().any(|run| {
        matches!(
            run.phase,
            Phase::EetInitialization | Phase::Main | Phase::EetFinalization | Phase::PostEetEnd
        )
    });
    if !has_eet_sequence {
        return;
    }

    let initialization_indices = runs
        .iter()
        .enumerate()
        .filter_map(|(index, run)| (run.phase == Phase::EetInitialization).then_some(index))
        .collect::<Vec<_>>();
    if initialization_indices.len() != 1 {
        error(
            findings,
            RULE_EET_ANCHORS,
            format!(
                "the EET sequence requires exactly one initialization run, found {}",
                initialization_indices.len()
            ),
        );
    }

    let first_after_preparation = runs
        .iter()
        .position(|run| !matches!(run.phase, Phase::Bg1Preparation | Phase::Bg2Preparation));
    if first_after_preparation.is_some()
        && first_after_preparation != initialization_indices.first().copied()
    {
        let index = first_after_preparation.unwrap_or_default();
        error(
            findings,
            RULE_EET_ANCHORS,
            format!(
                "run {index} ({:?}) is first after preparation but is not the EET initialization",
                runs[index].run_id
            ),
        );
    }

    let finalization_indices = runs
        .iter()
        .enumerate()
        .filter_map(|(index, run)| (run.phase == Phase::EetFinalization).then_some(index))
        .collect::<Vec<_>>();
    if finalization_indices.len() != 1 {
        error(
            findings,
            RULE_EET_ANCHORS,
            format!(
                "the EET sequence requires exactly one finalization run, found {}",
                finalization_indices.len()
            ),
        );
    }

    let last_before_tail = runs.iter().rposition(|run| run.phase != Phase::PostEetEnd);
    if last_before_tail.is_some() && last_before_tail != finalization_indices.first().copied() {
        let index = last_before_tail.unwrap_or_default();
        error(
            findings,
            RULE_EET_ANCHORS,
            format!(
                "run {index} ({:?}) is last before the post-EET tail but is not EET finalization",
                runs[index].run_id
            ),
        );
    }
}

fn check_stdin(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, mod_file) in &manifest.mods {
        for component in &mod_file.components {
            if component
                .stdin
                .as_deref()
                .is_some_and(|stdin| !stdin.ends_with('\n'))
            {
                warning(
                    findings,
                    RULE_STDIN_NEWLINE,
                    format!(
                        "installer {id:?} component {} stdin does not end with a newline",
                        component.id
                    ),
                );
            }
        }
    }
}
