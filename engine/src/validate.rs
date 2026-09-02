//! Static validation of a loaded executable recipe.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component as PathComponent, Path};

use crate::error::EngineError;
use crate::manifest::{
    AcquisitionPolicy, ComponentRef, Decision, GameRoot, InputSpec, InvocationMode, Phase,
    PromptAnswer, Readiness, SourceKind,
};
use crate::weidu::invocation::setup_executable_name;
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
/// Rule preventing declared runs from using blocked payload or WeiDU artifacts.
pub const RULE_BLOCKED_ARTIFACTS: &str = "blocked-artifacts";
/// Rule requiring authored archive and TP2 paths to remain within staged roots.
pub const RULE_PATHS: &str = "paths";
/// Rule preventing setup-name aliases from selecting the same TP2 implicitly.
pub const RULE_SETUP_NAME_AMBIGUITY: &str = "setup-name-ambiguity";
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
/// Rule requiring semantic feature ids and references to resolve exactly.
pub const RULE_FEATURE_REFERENCES: &str = "feature-references";
/// Rule preventing cycles through feature parents and requirements.
pub const RULE_FEATURE_CYCLES: &str = "feature-cycles";
/// Rule preventing two directly conflicting features from both defaulting on.
pub const RULE_FEATURE_DEFAULT_CONFLICTS: &str = "feature-default-conflicts";
/// Rule requiring one semantic owner for each exact run component.
pub const RULE_FEATURE_COMPONENT_OWNERSHIP: &str = "feature-component-ownership";
/// Rule requiring typed feature inputs to have coherent defaults and bounds.
pub const RULE_FEATURE_INPUTS: &str = "feature-inputs";
/// Rule requiring prompt input references to resolve to declared typed inputs.
pub const RULE_PROMPT_INPUT_REFERENCES: &str = "prompt-input-references";
/// Rule requiring visible blocked features to explain why they are unavailable.
pub const RULE_FEATURE_READINESS: &str = "feature-readiness";

/// Runs the currently defined recipe validation rules in stable order.
pub fn validate(manifest: &Manifest) -> Vec<Finding> {
    let mut findings = Vec::new();
    check_run_ids(manifest, &mut findings);
    check_nonempty(manifest, &mut findings);
    check_run_components(manifest, &mut findings);
    check_references(manifest, &mut findings);
    check_blocked_artifacts(manifest, &mut findings);
    check_paths(manifest, &mut findings);
    check_setup_name_ambiguity(manifest, &mut findings);
    check_component_ids(manifest, &mut findings);
    check_component_placement(manifest, &mut findings);
    check_acquisition_policy(manifest, &mut findings);
    check_sources(manifest, &mut findings);
    check_phase_order(manifest, &mut findings);
    check_eet_anchors(manifest, &mut findings);
    check_stdin(manifest, &mut findings);
    check_feature_references(manifest, &mut findings);
    check_feature_cycles(manifest, &mut findings);
    check_feature_default_conflicts(manifest, &mut findings);
    check_feature_component_ownership(manifest, &mut findings);
    check_feature_inputs(manifest, &mut findings);
    check_prompt_input_references(manifest, &mut findings);
    check_feature_readiness(manifest, &mut findings);
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

fn check_blocked_artifacts(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (index, run) in manifest.collection.runs.iter().enumerate() {
        let Some(mod_file) = manifest.mods.get(&run.mod_id) else {
            continue;
        };

        for (role, artifact_id) in [
            ("payload", mod_file.artifact_id.as_str()),
            ("WeiDU", mod_file.weidu_artifact_id.as_str()),
        ] {
            let Some(artifact) = manifest.artifacts.get(artifact_id) else {
                continue;
            };
            if artifact.acquisition == AcquisitionPolicy::Blocked {
                error(
                    findings,
                    RULE_BLOCKED_ARTIFACTS,
                    format!(
                        "run {index} ({:?}) uses blocked {role} artifact {artifact_id:?}",
                        run.run_id
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

fn check_setup_name_ambiguity(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (id, mod_file) in &manifest.mods {
        if mod_file.invocation_mode != InvocationMode::SetupName {
            continue;
        }
        let Ok(executable) = setup_executable_name(&mod_file.tp2) else {
            // The path rule reports malformed TP2 paths independently.
            continue;
        };
        let key = executable.to_ascii_lowercase();
        if let Some(first) = seen.insert(key, id) {
            error(
                findings,
                RULE_SETUP_NAME_AMBIGUITY,
                format!(
                    "installers {first:?} and {id:?} both map to setup executable {executable:?}"
                ),
            );
        }
    }
}

fn is_safe_relative_path(value: &str) -> bool {
    if value.is_empty() || value.starts_with('/') || value.starts_with('\\') {
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
        && !value
            .split(['/', '\\'])
            .any(|component| component == ".." || component.contains(':'))
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

fn check_feature_references(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let mut feature_ids = BTreeSet::new();
    for feature in &manifest.collection.features {
        if feature.id.is_empty() || feature.id.contains('/') {
            error(
                findings,
                RULE_FEATURE_REFERENCES,
                format!(
                    "feature id {:?} must be nonempty and cannot contain '/'",
                    feature.id
                ),
            );
        }
        if !feature_ids.insert(feature.id.as_str()) {
            error(
                findings,
                RULE_FEATURE_REFERENCES,
                format!("feature id {:?} is declared more than once", feature.id),
            );
        }
    }

    let runs = manifest
        .collection
        .runs
        .iter()
        .map(|run| (run.run_id.as_str(), run))
        .collect::<BTreeMap<_, _>>();
    for feature in &manifest.collection.features {
        if feature.decision == Decision::Mandatory && feature.parent.is_none() {
            error(
                findings,
                RULE_FEATURE_REFERENCES,
                format!("mandatory feature {:?} must declare a parent", feature.id),
            );
        }
        for (relation, target) in feature
            .parent
            .iter()
            .map(|target| ("parent", target))
            .chain(
                feature
                    .requires
                    .iter()
                    .map(|target| ("requirement", target)),
            )
            .chain(
                feature
                    .conflicts
                    .iter()
                    .map(|conflict| ("conflict", &conflict.feature_id)),
            )
        {
            if !feature_ids.contains(target.as_str()) {
                error(
                    findings,
                    RULE_FEATURE_REFERENCES,
                    format!(
                        "feature {:?} has unknown {relation} feature {:?}",
                        feature.id, target
                    ),
                );
            }
        }
        for component in &feature.components {
            let valid = runs
                .get(component.run_id.as_str())
                .is_some_and(|run| run.components.contains(&component.component));
            if !valid {
                error(
                    findings,
                    RULE_FEATURE_REFERENCES,
                    format!(
                        "feature {:?} references component {} outside declared run {:?}",
                        feature.id, component.component, component.run_id
                    ),
                );
            }
        }
    }
}

fn check_feature_cycles(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let graph = manifest
        .collection
        .features
        .iter()
        .map(|feature| {
            let edges = feature
                .parent
                .iter()
                .chain(feature.requires.iter())
                .map(String::as_str)
                .collect::<Vec<_>>();
            (feature.id.as_str(), edges)
        })
        .collect::<BTreeMap<_, _>>();
    let mut complete = BTreeSet::new();
    let mut visiting = BTreeSet::new();

    fn visit<'a>(
        node: &'a str,
        graph: &BTreeMap<&'a str, Vec<&'a str>>,
        visiting: &mut BTreeSet<&'a str>,
        complete: &mut BTreeSet<&'a str>,
    ) -> bool {
        if complete.contains(node) {
            return false;
        }
        if !visiting.insert(node) {
            return true;
        }
        let cyclic = graph.get(node).is_some_and(|edges| {
            edges
                .iter()
                .any(|edge| graph.contains_key(edge) && visit(edge, graph, visiting, complete))
        });
        visiting.remove(node);
        complete.insert(node);
        cyclic
    }

    for feature in &manifest.collection.features {
        if visit(feature.id.as_str(), &graph, &mut visiting, &mut complete) {
            error(
                findings,
                RULE_FEATURE_CYCLES,
                format!(
                    "feature {:?} participates in a parent or requirement cycle",
                    feature.id
                ),
            );
            break;
        }
    }
}

fn check_feature_default_conflicts(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let decisions = manifest
        .collection
        .features
        .iter()
        .map(|feature| (feature.id.as_str(), feature.decision))
        .collect::<BTreeMap<_, _>>();
    for feature in &manifest.collection.features {
        if feature.decision != Decision::Default {
            continue;
        }
        for conflict in &feature.conflicts {
            if decisions.get(conflict.feature_id.as_str()) == Some(&Decision::Default) {
                error(
                    findings,
                    RULE_FEATURE_DEFAULT_CONFLICTS,
                    format!(
                        "default feature {:?} conflicts with default feature {:?}",
                        feature.id, conflict.feature_id
                    ),
                );
            }
        }
    }
}

fn check_feature_component_ownership(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let mut owners: BTreeMap<&ComponentRef, &str> = BTreeMap::new();
    for feature in &manifest.collection.features {
        for component in &feature.components {
            if let Some(first_owner) = owners.insert(component, feature.id.as_str()) {
                error(
                    findings,
                    RULE_FEATURE_COMPONENT_OWNERSHIP,
                    format!(
                        "run {:?} component {} is owned by both {:?} and {:?}",
                        component.run_id, component.component, first_owner, feature.id
                    ),
                );
            }
        }
    }
}

fn check_feature_inputs(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for feature in &manifest.collection.features {
        let mut input_ids = BTreeSet::new();
        for input in &feature.inputs {
            if input.id().is_empty() || input.id().contains('/') || !input_ids.insert(input.id()) {
                error(
                    findings,
                    RULE_FEATURE_INPUTS,
                    format!(
                        "feature {:?} has an invalid or duplicate input id {:?}",
                        feature.id,
                        input.id()
                    ),
                );
            }
            match input {
                InputSpec::Boolean { .. } => {}
                InputSpec::Choice {
                    id,
                    default,
                    options,
                } => {
                    let mut option_ids = BTreeSet::new();
                    let options_valid = !options.is_empty()
                        && options.iter().all(|option| {
                            !option.id.is_empty() && option_ids.insert(option.id.as_str())
                        });
                    if !options_valid || !option_ids.contains(default.as_str()) {
                        error(
                            findings,
                            RULE_FEATURE_INPUTS,
                            format!(
                                "feature {:?} choice input {id:?} has invalid options or default {default:?}",
                                feature.id
                            ),
                        );
                    }
                }
                InputSpec::Integer {
                    id,
                    default,
                    min,
                    max,
                } => {
                    if min > max || !(*min..=*max).contains(default) {
                        error(
                            findings,
                            RULE_FEATURE_INPUTS,
                            format!(
                                "feature {:?} integer input {id:?} default {default} is outside {min}..={max}",
                                feature.id
                            ),
                        );
                    }
                }
            }
        }
    }
}

fn check_prompt_input_references(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let inputs = manifest
        .collection
        .features
        .iter()
        .map(|feature| {
            (
                feature.id.as_str(),
                feature
                    .inputs
                    .iter()
                    .map(InputSpec::id)
                    .collect::<BTreeSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (mod_id, mod_file) in &manifest.mods {
        for component in &mod_file.components {
            for prompt in &component.prompts {
                let PromptAnswer::Input(input_ref) = &prompt.answer else {
                    continue;
                };
                let valid =
                    inputs
                        .get(input_ref.feature_id.as_str())
                        .is_some_and(|feature_inputs| {
                            feature_inputs.contains(input_ref.input_id.as_str())
                        });
                if !valid {
                    error(
                        findings,
                        RULE_PROMPT_INPUT_REFERENCES,
                        format!(
                            "installer {mod_id:?} component {} prompt references unknown input {:?}/{:?}",
                            component.id, input_ref.feature_id, input_ref.input_id
                        ),
                    );
                }
            }
        }
    }
}

fn check_feature_readiness(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for feature in &manifest.collection.features {
        if feature.readiness == Readiness::Blocked
            && feature.decision != Decision::Excluded
            && feature
                .unavailable_reason
                .as_deref()
                .is_none_or(str::is_empty)
        {
            error(
                findings,
                RULE_FEATURE_READINESS,
                format!(
                    "blocked visible feature {:?} must provide an unavailable reason",
                    feature.id
                ),
            );
        }
        for conflict in &feature.conflicts {
            if conflict.reason.is_empty() {
                error(
                    findings,
                    RULE_FEATURE_READINESS,
                    format!(
                        "feature {:?} conflict with {:?} must provide an unavailable reason",
                        feature.id, conflict.feature_id
                    ),
                );
            }
        }
    }
}
