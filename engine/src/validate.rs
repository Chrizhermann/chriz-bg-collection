//! Static validation of a loaded executable recipe.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component as PathComponent, Path};

use crate::error::EngineError;
use crate::manifest::{
    AcquisitionPolicy, Artifact, ComponentRef, Decision, GameRoot, InputSpec, InvocationMode,
    PeMachine, Phase, PromptAnswer, Readiness, Source, SourceKind,
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
/// Public artifact identity fields that are incomplete while authoring.
pub const RULE_ARTIFACT_CONTRACT: &str = "artifact-contract";
/// Public artifact URLs that point at a moving branch or latest-release alias.
pub const RULE_MUTABLE_SOURCE: &str = "mutable-source";
/// Redirect destinations that have not been explicitly reviewed.
pub const RULE_UNREVIEWED_REDIRECT: &str = "unreviewed-redirect";
/// Archive layout declarations that are incomplete or ambiguous.
pub const RULE_ARCHIVE_CONTRACT: &str = "archive-contract";
/// Installers or duplicate source declarations that disagree about one artifact.
pub const RULE_SHARED_ARTIFACT: &str = "shared-artifact";
/// WeiDU tool declarations that are absent or not Windows x64.
pub const RULE_TOOL_ARCHITECTURE: &str = "tool-architecture";
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
    check_artifact_contract(manifest, &mut findings);
    check_archive_contract(manifest, &mut findings);
    check_shared_artifacts(manifest, &mut findings);
    check_tool_architecture(manifest, &mut findings);
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

/// Validate one standalone artifact with the same contract rules that gate a public recipe.
///
/// Loopback HTTP remains available solely for hermetic verification tests; every other source
/// route must be HTTPS. Callers must reject every returned finding, including authoring warnings.
pub fn validate_artifact_for_verification(id: &str, artifact: &Artifact) -> Vec<Finding> {
    let mut findings = Vec::new();
    check_artifact_paths(id, artifact, &mut findings);
    check_artifact_acquisition_policy(id, artifact, &mut findings);
    check_artifact_source(id, artifact, true, &mut findings);
    check_one_artifact_contract(id, artifact, &mut findings);
    check_one_archive_contract(id, artifact, &mut findings);
    if artifact.tool.is_some() {
        check_tool_artifact_architecture(id, artifact, &mut findings);
    }
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
        check_artifact_paths(id, artifact, findings);
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

fn check_artifact_paths(id: &str, artifact: &Artifact, findings: &mut Vec<Finding>) {
    for path in artifact
        .archive
        .publish_roots
        .iter()
        .chain(artifact.archive.tp2_paths.iter())
        .chain(artifact.tool.iter().map(|tool| &tool.executable))
    {
        if !is_safe_relative_path(path) {
            error(
                findings,
                RULE_PATHS,
                format!(
                    "artifact {id:?} archive path {path:?} is not a traversal-free relative path"
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
        check_artifact_acquisition_policy(id, artifact, findings);
    }
}

fn check_artifact_acquisition_policy(id: &str, artifact: &Artifact, findings: &mut Vec<Finding>) {
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

fn check_sources(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, artifact) in &manifest.artifacts {
        check_artifact_source(id, artifact, false, findings);
    }
}

fn check_artifact_source(
    id: &str,
    artifact: &Artifact,
    allow_loopback_http: bool,
    findings: &mut Vec<Finding>,
) {
    if artifact.acquisition == AcquisitionPolicy::Blocked {
        return;
    }

    if !is_https_url(&artifact.source.url)
        && !(allow_loopback_http && is_loopback_http_url(&artifact.source.url))
    {
        let message = format!(
            "artifact {id:?} source url {:?} is not https",
            artifact.source.url
        );
        if artifact.acquisition == AcquisitionPolicy::ManualUserSupplied {
            warning(findings, RULE_SOURCES, message);
        } else {
            error(findings, RULE_SOURCES, message);
        }
    }

    let sha256 = artifact.source.sha256.as_str();
    if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        let message = format!("artifact {id:?} source sha256 {sha256:?} is not 64 hex characters");
        if artifact.acquisition == AcquisitionPolicy::ManualUserSupplied {
            warning(findings, RULE_SOURCES, message);
        } else {
            error(findings, RULE_SOURCES, message);
        }
    } else if sha256.eq_ignore_ascii_case(UNPINNED_SHA256) {
        warning(
            findings,
            RULE_UNPINNED_SOURCE,
            format!("artifact {id:?} has an all-zero sha256, so its source is not pinned"),
        );
    }

    if is_moving_source_url(&artifact.source.url)
        || !is_immutable_source_reference(&artifact.source)
    {
        warning(
            findings,
            RULE_MUTABLE_SOURCE,
            format!(
                "artifact {id:?} source url/reference {:?}/{:?} is not a positive immutable identity",
                artifact.source.url, artifact.source.reference
            ),
        );
    }

    if github_url_requires_redirect_review(&artifact.source.url)
        && artifact.source.redirect_hosts.is_empty()
    {
        warning(
            findings,
            RULE_UNREVIEWED_REDIRECT,
            format!(
                "artifact {id:?} uses a GitHub route that redirects but declares no reviewed redirect host"
            ),
        );
    }
}

fn check_artifact_contract(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, artifact) in &manifest.artifacts {
        check_one_artifact_contract(id, artifact, findings);
    }
}

fn check_one_artifact_contract(id: &str, artifact: &Artifact, findings: &mut Vec<Finding>) {
    if artifact.acquisition == AcquisitionPolicy::Blocked {
        return;
    }
    let mut missing = Vec::new();
    if artifact.version.trim().is_empty() {
        missing.push("version");
    }
    if artifact.source.reference.trim().is_empty() {
        missing.push("source reference");
    }
    if artifact
        .source
        .expected_filename
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        missing.push("expected filename");
    } else if !artifact
        .source
        .expected_filename
        .as_deref()
        .is_some_and(|filename| {
            let filename = filename.to_ascii_lowercase();
            match artifact.archive.kind {
                crate::manifest::ArchiveKind::Zip => filename.ends_with(".zip"),
                crate::manifest::ArchiveKind::Iemod => filename.ends_with(".iemod"),
            }
        })
    {
        missing.push("filename/archive kind agreement");
    }
    if artifact
        .source
        .expected_length
        .is_none_or(|length| length == 0)
    {
        missing.push("expected length");
    }
    if !is_review_date(&artifact.provenance.reviewed_on) {
        missing.push("review date");
    }
    if !is_https_url(&artifact.provenance.url) {
        missing.push("HTTPS provenance URL");
    }
    if !missing.is_empty() {
        warning(
            findings,
            RULE_ARTIFACT_CONTRACT,
            format!("artifact {id:?} lacks {}", missing.join(", ")),
        );
    }

    let mut redirect_hosts = BTreeSet::new();
    for host in &artifact.source.redirect_hosts {
        let normalized = host.to_ascii_lowercase();
        if host.is_empty()
            || host.contains(['/', ':'])
            || !host.is_ascii()
            || !redirect_hosts.insert(normalized)
        {
            error(
                findings,
                RULE_UNREVIEWED_REDIRECT,
                format!("artifact {id:?} has invalid or duplicate redirect host {host:?}"),
            );
        }
    }
}

fn check_archive_contract(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (id, artifact) in &manifest.artifacts {
        check_one_archive_contract(id, artifact, findings);
    }
}

fn check_one_archive_contract(id: &str, artifact: &Artifact, findings: &mut Vec<Finding>) {
    let archive = &artifact.archive;
    let mut roots = BTreeSet::new();
    for root in &archive.publish_roots {
        if !roots.insert(casefold_path(root)) {
            error(
                findings,
                RULE_ARCHIVE_CONTRACT,
                format!("artifact {id:?} declares publish root {root:?} more than once"),
            );
        }
    }
    if archive.publish_roots.is_empty() {
        error(
            findings,
            RULE_ARCHIVE_CONTRACT,
            format!("artifact {id:?} declares no publish roots"),
        );
    }
    for (index, left) in archive.publish_roots.iter().enumerate() {
        for right in archive.publish_roots.iter().skip(index + 1) {
            if path_contains(left, right) || path_contains(right, left) {
                error(
                    findings,
                    RULE_ARCHIVE_CONTRACT,
                    format!("artifact {id:?} has overlapping publish roots {left:?} and {right:?}"),
                );
            }
        }
    }

    let mut tp2s = BTreeSet::new();
    for tp2 in &archive.tp2_paths {
        if !tp2.to_ascii_lowercase().ends_with(".tp2") || !tp2s.insert(casefold_path(tp2)) {
            error(
                findings,
                RULE_ARCHIVE_CONTRACT,
                format!("artifact {id:?} has invalid or duplicate TP2 path {tp2:?}"),
            );
        }
        let owners = archive
            .publish_roots
            .iter()
            .filter(|root| path_contains(root, tp2))
            .count();
        if owners != 1 {
            error(
                    findings,
                    RULE_ARCHIVE_CONTRACT,
                    format!(
                        "artifact {id:?} TP2 {tp2:?} belongs to {owners} declared publish roots, expected exactly one"
                    ),
                );
        }
    }
    if artifact.tool.is_none() && archive.tp2_paths.is_empty() {
        error(
            findings,
            RULE_ARCHIVE_CONTRACT,
            format!("payload artifact {id:?} declares no expected TP2 paths"),
        );
    }
    if let Some(tool) = &artifact.tool {
        let owners = archive
            .publish_roots
            .iter()
            .filter(|root| path_contains(root, &tool.executable))
            .count();
        if owners != 1 {
            error(
                    findings,
                    RULE_ARCHIVE_CONTRACT,
                    format!(
                        "tool artifact {id:?} executable {:?} belongs to {owners} declared publish roots, expected exactly one",
                        tool.executable
                    ),
                );
        }
    }
    let limits = &archive.limits;
    if limits.max_depth == 0
        || limits.max_entries == 0
        || limits.max_entry_uncompressed_bytes == 0
        || limits.max_total_uncompressed_bytes == 0
        || limits.max_compression_ratio == 0
        || limits.max_entry_uncompressed_bytes > limits.max_total_uncompressed_bytes
    {
        error(
            findings,
            RULE_ARCHIVE_CONTRACT,
            format!("artifact {id:?} has invalid archive limits"),
        );
    }
}

fn check_shared_artifacts(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for (mod_id, mod_file) in &manifest.mods {
        if !is_safe_relative_path(&mod_file.tp2) {
            continue;
        }
        let Some(artifact) = manifest.artifacts.get(&mod_file.artifact_id) else {
            continue;
        };
        if !artifact
            .archive
            .tp2_paths
            .iter()
            .any(|tp2| tp2.eq_ignore_ascii_case(&mod_file.tp2))
        {
            error(
                findings,
                RULE_SHARED_ARTIFACT,
                format!(
                    "installer {mod_id:?} TP2 {:?} is not declared by shared artifact {:?}",
                    mod_file.tp2, mod_file.artifact_id
                ),
            );
        }
    }

    let mut sources: BTreeMap<String, (&str, &crate::manifest::Artifact)> = BTreeMap::new();
    for (id, artifact) in &manifest.artifacts {
        if artifact.acquisition == AcquisitionPolicy::Blocked {
            continue;
        }
        let key = normalized_source_url(&artifact.source.url);
        if let Some((first_id, first)) = sources.insert(key, (id, artifact)) {
            if !shared_source_contracts_agree(first, artifact) {
                error(
                    findings,
                    RULE_SHARED_ARTIFACT,
                    format!("artifacts {first_id:?} and {id:?} disagree about one byte-source URL"),
                );
            }
        }
    }
}

fn normalized_source_url(value: &str) -> String {
    let Ok(mut url) = url::Url::parse(value) else {
        return value.trim().to_owned();
    };
    url.set_fragment(None);
    url.to_string()
}

fn shared_source_contracts_agree(
    left: &crate::manifest::Artifact,
    right: &crate::manifest::Artifact,
) -> bool {
    left.version == right.version
        && left.source.kind == right.source.kind
        && normalized_source_url(&left.source.url) == normalized_source_url(&right.source.url)
        && left.source.reference == right.source.reference
        && left.source.expected_filename == right.source.expected_filename
        && left.source.expected_length == right.source.expected_length
        && left
            .source
            .sha256
            .eq_ignore_ascii_case(&right.source.sha256)
        && normalized_redirect_hosts(&left.source.redirect_hosts)
            == normalized_redirect_hosts(&right.source.redirect_hosts)
        && left.acquisition == right.acquisition
        && left.archive == right.archive
        && left.tool == right.tool
        && left.provenance == right.provenance
}

fn normalized_redirect_hosts(hosts: &[String]) -> BTreeSet<String> {
    hosts.iter().map(|host| host.to_ascii_lowercase()).collect()
}

fn check_tool_architecture(manifest: &Manifest, findings: &mut Vec<Finding>) {
    let tool_ids = manifest
        .mods
        .values()
        .map(|mod_file| mod_file.weidu_artifact_id.as_str())
        .collect::<BTreeSet<_>>();
    for id in tool_ids {
        let Some(artifact) = manifest.artifacts.get(id) else {
            continue;
        };
        check_tool_artifact_architecture(id, artifact, findings);
    }
}

fn check_tool_artifact_architecture(id: &str, artifact: &Artifact, findings: &mut Vec<Finding>) {
    if artifact.tool.as_ref().map(|tool| tool.pe_machine) != Some(PeMachine::X86_64) {
        warning(
            findings,
            RULE_TOOL_ARCHITECTURE,
            format!("WeiDU artifact {id:?} is not declared as Windows x86-64"),
        );
    }
}

fn casefold_path(value: &str) -> String {
    value.replace('\\', "/").to_ascii_lowercase()
}

fn path_contains(root: &str, path: &str) -> bool {
    let root = casefold_path(root);
    let path = casefold_path(path);
    path == root || path.starts_with(&format!("{root}/"))
}

fn is_review_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit()))
    {
        return false;
    }
    let year = value[..4].parse::<u32>().expect("ASCII digits checked");
    let month = value[5..7].parse::<u32>().expect("ASCII digits checked");
    let day = value[8..].parse::<u32>().expect("ASCII digits checked");
    let leap_year =
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

fn is_https_url(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| url.scheme() == "https" && url.host_str().is_some())
}

fn is_loopback_http_url(value: &str) -> bool {
    let Ok(url) = url::Url::parse(value) else {
        return false;
    };
    url.scheme() == "http"
        && match url.host() {
            Some(url::Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
            Some(url::Host::Ipv4(address)) => address.is_loopback(),
            Some(url::Host::Ipv6(address)) => address.is_loopback(),
            None => false,
        }
}

fn is_moving_source_url(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "/refs/heads/",
        "/archive/main",
        "/archive/master",
        "/zipball/main",
        "/zipball/master",
        "/releases/latest",
        "/latest/download/",
        "/main.zip",
        "/master.zip",
    ]
    .iter()
    .any(|pattern| value.contains(pattern))
}

fn is_immutable_source_reference(source: &Source) -> bool {
    let reference = source.reference.trim();
    match source.kind {
        SourceKind::GithubCommitZip => {
            reference.len() == 40
                && reference.bytes().all(|byte| byte.is_ascii_hexdigit())
                && github_route_matches_reference(&source.url, source.kind, reference)
        }
        SourceKind::GithubRelease | SourceKind::GithubTagArchive => {
            looks_like_version_tag(reference)
                && github_route_matches_reference(&source.url, source.kind, reference)
        }
        SourceKind::Manual => {
            !reference.is_empty()
                && !matches!(
                    reference.to_ascii_lowercase().as_str(),
                    "head" | "main" | "master" | "latest"
                )
                && !reference.to_ascii_lowercase().starts_with("refs/heads/")
        }
    }
}

fn looks_like_version_tag(reference: &str) -> bool {
    !reference.is_empty()
        && reference.len() <= 128
        && reference.bytes().any(|byte| byte.is_ascii_digit())
        && reference
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+'))
}

fn github_route_matches_reference(url: &str, kind: SourceKind, reference: &str) -> bool {
    let Ok(url) = url::Url::parse(url) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    if !host.eq_ignore_ascii_case("github.com") && !host.eq_ignore_ascii_case("codeload.github.com")
    {
        return true;
    }
    let segments = url
        .path_segments()
        .map(|segments| segments.collect::<Vec<_>>())
        .unwrap_or_default();
    match kind {
        SourceKind::GithubRelease => segments.windows(3).any(|window| {
            window[0].eq_ignore_ascii_case("releases")
                && window[1].eq_ignore_ascii_case("download")
                && window[2] == reference
        }),
        SourceKind::GithubTagArchive => segments.windows(4).any(|window| {
            (window[0].eq_ignore_ascii_case("archive") || window[0].eq_ignore_ascii_case("zip"))
                && window[1].eq_ignore_ascii_case("refs")
                && window[2].eq_ignore_ascii_case("tags")
                && window[3].strip_suffix(".zip").unwrap_or(window[3]) == reference
        }),
        SourceKind::GithubCommitZip => segments
            .iter()
            .any(|segment| segment.strip_suffix(".zip").unwrap_or(segment) == reference),
        SourceKind::Manual => true,
    }
}

fn github_url_requires_redirect_review(value: &str) -> bool {
    let Ok(url) = url::Url::parse(value) else {
        return false;
    };
    url.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("github.com")
            && (url.path().contains("/releases/download/") || url.path().contains("/archive/"))
    })
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
