//! Release-specific gates for the public v0.1 alpha recipe.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component as PathComponent, Path};

use serde::Deserialize;

use crate::error::EngineError;
use crate::manifest::{AcquisitionPolicy, Decision, Feature, Phase, Readiness};
use crate::recipe_view::{evaluate_preset, SelectionEvaluation};
use crate::validate::{Finding, Severity};
use crate::Manifest;

/// Immutable release-record directory currently used by the public alpha profile.
pub const PUBLIC_ALPHA_RELEASE_ID: &str = "v0.1.0-alpha.1";
/// A selected curated outcome has no usable payload/tool artifact.
pub const RULE_RELEASE_SELECTED_ARTIFACT: &str = "release-selected-artifact";
/// A selected curated outcome still identifies a deliberate failing placeholder.
pub const RULE_RELEASE_FAIL_STUB: &str = "release-fail-stub";
/// A resolved run has no accepted static evidence record.
pub const RULE_RELEASE_STATIC_EVIDENCE: &str = "release-static-evidence";
/// A selected post-EET run lacks explicit release-tail approval.
pub const RULE_RELEASE_TAIL: &str = "release-tail-approval";
/// A desired default or mandatory outcome was omitted without a release record.
pub const RULE_RELEASE_OMISSION: &str = "release-omission";
/// An omission record lacks Christopher's approval, a reason, or visible wording.
pub const RULE_RELEASE_OMISSION_APPROVAL: &str = "release-omission-approval";
/// A release evidence/limitation record is malformed or stale.
pub const RULE_RELEASE_RECORD: &str = "release-record";
/// An unavailable optional feature lacks its authored UI explanation.
pub const RULE_RELEASE_OPTIONAL_NOTE: &str = "release-optional-note";

const RELEASE_SCHEMA: u32 = 1;
const PUBLIC_ALPHA_PRESET: &str = "chris-recommended";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KnownLimitations {
    schema: u32,
    release: String,
    preset: String,
    approved_tail_runs: Vec<String>,
    #[serde(default)]
    omissions: Vec<Omission>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Omission {
    feature_id: String,
    approved_by: String,
    approved_on: String,
    reason: String,
    user_facing_limitation: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Acceptance {
    schema: u32,
    release: String,
    preset: String,
    #[serde(default)]
    evidence: Vec<AcceptanceRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum EvidenceSubjectKind {
    Feature,
    Run,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum EvidenceKind {
    StaticTest,
    RuntimeTest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum EvidenceStatus {
    Accepted,
    Pending,
    Failed,
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptanceRecord {
    subject_kind: EvidenceSubjectKind,
    subject_id: String,
    kind: EvidenceKind,
    repository: Option<String>,
    commit: Option<String>,
    test_artifact: Option<String>,
    date: String,
    status: EvidenceStatus,
    scope: String,
}

/// Validate the release records stored beneath the current public-alpha recipe.
pub fn validate_public_alpha(manifest: &Manifest) -> crate::error::Result<Vec<Finding>> {
    validate_public_alpha_at(
        manifest,
        &manifest.root.join("releases").join(PUBLIC_ALPHA_RELEASE_ID),
    )
}

/// Validate public-alpha release records from an explicit directory.
///
/// The explicit path exists for hermetic tests and authoring tools. Production callers use
/// [`validate_public_alpha`], which binds the records to the loaded recipe directory.
pub fn validate_public_alpha_at(
    manifest: &Manifest,
    release_dir: &Path,
) -> crate::error::Result<Vec<Finding>> {
    // Small synthetic recipes used by engine/app contract tests are not the named public
    // release candidate. A real candidate either carries the canonical preset or its release
    // directory, in which case missing/malformed records remain release-blocking below.
    if !manifest.presets.contains_key(PUBLIC_ALPHA_PRESET) && !release_dir.exists() {
        return Ok(Vec::new());
    }
    let limitations: KnownLimitations = read_toml(&release_dir.join("known-limitations.toml"))?;
    let acceptance: Acceptance = read_toml(&release_dir.join("acceptance.toml"))?;
    let mut findings = Vec::new();

    check_release_metadata(&limitations, &acceptance, &mut findings);
    if !manifest.presets.contains_key(PUBLIC_ALPHA_PRESET) {
        release_error(
            &mut findings,
            RULE_RELEASE_RECORD,
            format!("public alpha preset {PUBLIC_ALPHA_PRESET:?} is missing"),
        );
        return Ok(findings);
    }

    check_optional_notes(manifest, &mut findings);
    check_acceptance_records(manifest, &acceptance, &mut findings);
    check_limitation_records(manifest, &limitations, &mut findings);

    match evaluate_preset(manifest, PUBLIC_ALPHA_PRESET, "windows") {
        Ok(evaluation) => {
            check_selected_artifacts(manifest, &evaluation, &mut findings);
            check_static_evidence(&acceptance, &evaluation, &mut findings);
            check_tail_approval(&limitations, &evaluation, &mut findings);
            check_omission_coverage(manifest, &limitations, &evaluation, &mut findings);
        }
        Err(error) => release_error(
            &mut findings,
            RULE_RELEASE_RECORD,
            format!("public alpha preset cannot be evaluated: {error}"),
        ),
    }

    Ok(findings)
}

fn read_toml<T>(path: &Path) -> crate::error::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let text = std::fs::read_to_string(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&text).map_err(|source| EngineError::ManifestParse {
        path: path.to_path_buf(),
        source,
    })
}

fn check_release_metadata(
    limitations: &KnownLimitations,
    acceptance: &Acceptance,
    findings: &mut Vec<Finding>,
) {
    for (name, schema, release, preset) in [
        (
            "known limitations",
            limitations.schema,
            limitations.release.as_str(),
            limitations.preset.as_str(),
        ),
        (
            "acceptance",
            acceptance.schema,
            acceptance.release.as_str(),
            acceptance.preset.as_str(),
        ),
    ] {
        if schema != RELEASE_SCHEMA {
            release_error(
                findings,
                RULE_RELEASE_RECORD,
                format!("{name} schema {schema} is not {RELEASE_SCHEMA}"),
            );
        }
        if release != PUBLIC_ALPHA_RELEASE_ID {
            release_error(
                findings,
                RULE_RELEASE_RECORD,
                format!("{name} release {release:?} is not {PUBLIC_ALPHA_RELEASE_ID:?}"),
            );
        }
        if preset != PUBLIC_ALPHA_PRESET {
            release_error(
                findings,
                RULE_RELEASE_RECORD,
                format!("{name} preset {preset:?} is not {PUBLIC_ALPHA_PRESET:?}"),
            );
        }
    }
}

fn check_selected_artifacts(
    manifest: &Manifest,
    evaluation: &SelectionEvaluation,
    findings: &mut Vec<Finding>,
) {
    let selected = evaluation
        .view
        .controls
        .iter()
        .filter(|control| {
            control.selected && matches!(control.decision, Decision::Default | Decision::Mandatory)
        })
        .map(|control| control.id.as_str())
        .collect::<BTreeSet<_>>();
    for feature in manifest
        .collection
        .features
        .iter()
        .filter(|feature| selected.contains(feature.id.as_str()))
    {
        for component_ref in &feature.components {
            let Some(run) = manifest
                .collection
                .runs
                .iter()
                .find(|run| run.run_id == component_ref.run_id)
            else {
                release_error(
                    findings,
                    RULE_RELEASE_SELECTED_ARTIFACT,
                    format!(
                        "selected outcome {:?} references missing run {:?}",
                        feature.id, component_ref.run_id
                    ),
                );
                continue;
            };
            let Some(mod_file) = manifest.mods.get(&run.mod_id) else {
                release_error(
                    findings,
                    RULE_RELEASE_SELECTED_ARTIFACT,
                    format!(
                        "selected outcome {:?} references run {:?} with missing installer {:?}",
                        feature.id, run.run_id, run.mod_id
                    ),
                );
                continue;
            };
            let payload = manifest.artifacts.get(&mod_file.artifact_id);
            let tool = manifest.artifacts.get(&mod_file.weidu_artifact_id);
            if payload.is_none()
                || payload
                    .is_some_and(|artifact| artifact.acquisition == AcquisitionPolicy::Blocked)
                || tool.is_none()
                || tool.is_some_and(|artifact| artifact.acquisition == AcquisitionPolicy::Blocked)
            {
                release_error(
                    findings,
                    RULE_RELEASE_SELECTED_ARTIFACT,
                    format!(
                        "selected outcome {:?} does not have usable payload {:?} and tool {:?}",
                        feature.id, mod_file.artifact_id, mod_file.weidu_artifact_id
                    ),
                );
            }

            let component = mod_file
                .components
                .iter()
                .find(|component| component.id == component_ref.component);
            let mut fail_candidates = vec![
                feature.title.as_str(),
                feature.description.as_str(),
                mod_file.name.as_str(),
                mod_file.tp2.as_str(),
            ];
            if let Some(component) = component {
                fail_candidates.push(component.name.as_str());
            }
            if let Some(artifact) = payload {
                fail_candidates.extend([
                    artifact.id.as_str(),
                    artifact.name.as_str(),
                    artifact.version.as_str(),
                    artifact.source.url.as_str(),
                    artifact.source.reference.as_str(),
                ]);
            }
            if fail_candidates.into_iter().any(is_fail_stub) {
                release_error(
                    findings,
                    RULE_RELEASE_FAIL_STUB,
                    format!(
                        "selected outcome {:?} component {} still identifies a FAIL stub",
                        feature.id, component_ref.component
                    ),
                );
            }
        }
    }
}

fn is_fail_stub(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value == "fail" || value.contains("fail stub") || value.contains("fail placeholder")
}

fn check_optional_notes(manifest: &Manifest, findings: &mut Vec<Finding>) {
    for feature in &manifest.collection.features {
        if feature.decision == Decision::Optional
            && feature.readiness == Readiness::Blocked
            && feature
                .unavailable_reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
        {
            release_error(
                findings,
                RULE_RELEASE_OPTIONAL_NOTE,
                format!(
                    "blocked optional feature {:?} has no user-facing unavailable note",
                    feature.id
                ),
            );
        }
    }
}

fn check_acceptance_records(
    manifest: &Manifest,
    acceptance: &Acceptance,
    findings: &mut Vec<Finding>,
) {
    let mut identities = BTreeSet::new();
    for record in &acceptance.evidence {
        let identity = (record.subject_kind, record.subject_id.as_str(), record.kind);
        if !identities.insert(identity) {
            release_error(
                findings,
                RULE_RELEASE_RECORD,
                format!(
                    "duplicate {:?} evidence for {:?}",
                    record.kind, record.subject_id
                ),
            );
        }
        let repository_commit = record
            .repository
            .as_deref()
            .zip(record.commit.as_deref())
            .is_some_and(|(repository, commit)| !repository.trim().is_empty() && is_commit(commit));
        let test_artifact = record
            .test_artifact
            .as_deref()
            .filter(|path| !path.trim().is_empty());
        if !repository_commit && test_artifact.is_none() {
            release_error(
                findings,
                RULE_RELEASE_RECORD,
                format!(
                    "evidence for {:?} has neither repository/commit nor test artifact",
                    record.subject_id
                ),
            );
        }
        if !is_date(&record.date) || record.scope.trim().is_empty() {
            release_error(
                findings,
                RULE_RELEASE_RECORD,
                format!(
                    "evidence for {:?} lacks a valid date or scope",
                    record.subject_id
                ),
            );
        }
        match record.subject_kind {
            EvidenceSubjectKind::Run
                if !manifest
                    .collection
                    .runs
                    .iter()
                    .any(|run| run.run_id == record.subject_id) =>
            {
                release_error(
                    findings,
                    RULE_RELEASE_RECORD,
                    format!("evidence names unknown run {:?}", record.subject_id),
                );
            }
            EvidenceSubjectKind::Feature
                if !manifest
                    .collection
                    .features
                    .iter()
                    .any(|feature| feature.id == record.subject_id) =>
            {
                release_error(
                    findings,
                    RULE_RELEASE_RECORD,
                    format!("evidence names unknown feature {:?}", record.subject_id),
                );
            }
            EvidenceSubjectKind::Release if record.subject_id != PUBLIC_ALPHA_RELEASE_ID => {
                release_error(
                    findings,
                    RULE_RELEASE_RECORD,
                    format!("evidence names unknown release {:?}", record.subject_id),
                );
            }
            _ => {}
        }

        if record.kind == EvidenceKind::StaticTest && record.status == EvidenceStatus::Accepted {
            let Some(path) = test_artifact else {
                release_error(
                    findings,
                    RULE_RELEASE_RECORD,
                    format!(
                        "accepted static evidence for {:?} has no test artifact",
                        record.subject_id
                    ),
                );
                continue;
            };
            if !repository_commit || !safe_relative_path(path) {
                release_error(
                    findings,
                    RULE_RELEASE_RECORD,
                    format!(
                        "accepted static evidence for {:?} must name an immutable repository commit and safe test artifact {path:?}",
                        record.subject_id
                    ),
                );
            }
        }
    }
}

fn check_static_evidence(
    acceptance: &Acceptance,
    evaluation: &SelectionEvaluation,
    findings: &mut Vec<Finding>,
) {
    let accepted = acceptance
        .evidence
        .iter()
        .filter(|record| {
            record.subject_kind == EvidenceSubjectKind::Run
                && record.kind == EvidenceKind::StaticTest
                && record.status == EvidenceStatus::Accepted
                && record
                    .repository
                    .as_deref()
                    .zip(record.commit.as_deref())
                    .is_some_and(|(repository, commit)| {
                        !repository.trim().is_empty() && is_commit(commit)
                    })
                && record
                    .test_artifact
                    .as_deref()
                    .is_some_and(safe_relative_path)
        })
        .map(|record| record.subject_id.as_str())
        .collect::<BTreeSet<_>>();

    for run in &evaluation.plan.runs {
        if !accepted.contains(run.run_id.as_str()) {
            release_error(
                findings,
                RULE_RELEASE_STATIC_EVIDENCE,
                format!(
                    "resolved run {:?} has no accepted static-test evidence",
                    run.run_id
                ),
            );
        }
    }
}

fn check_tail_approval(
    limitations: &KnownLimitations,
    evaluation: &SelectionEvaluation,
    findings: &mut Vec<Finding>,
) {
    let approved = limitations
        .approved_tail_runs
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if approved.len() != limitations.approved_tail_runs.len() {
        release_error(
            findings,
            RULE_RELEASE_RECORD,
            "approved_tail_runs contains a duplicate".to_owned(),
        );
    }
    for run in evaluation
        .plan
        .runs
        .iter()
        .filter(|run| run.phase == Phase::PostEetEnd)
    {
        if !approved.contains(run.run_id.as_str()) {
            release_error(
                findings,
                RULE_RELEASE_TAIL,
                format!(
                    "selected post-EET run {:?} has no explicit tail approval",
                    run.run_id
                ),
            );
        }
    }
}

fn check_limitation_records(
    manifest: &Manifest,
    limitations: &KnownLimitations,
    findings: &mut Vec<Finding>,
) {
    let mut ids = BTreeSet::new();
    for omission in &limitations.omissions {
        if !ids.insert(omission.feature_id.as_str()) {
            release_error(
                findings,
                RULE_RELEASE_RECORD,
                format!("duplicate omission for {:?}", omission.feature_id),
            );
        }
        if !manifest
            .collection
            .features
            .iter()
            .any(|feature| feature.id == omission.feature_id)
        {
            release_error(
                findings,
                RULE_RELEASE_RECORD,
                format!("omission names unknown feature {:?}", omission.feature_id),
            );
        }
        if !valid_omission(omission) {
            release_error(
                findings,
                RULE_RELEASE_OMISSION_APPROVAL,
                format!(
                    "omission {:?} requires Christopher approval, date, reason, and user-facing limitation",
                    omission.feature_id
                ),
            );
        }
    }
}

fn check_omission_coverage(
    manifest: &Manifest,
    limitations: &KnownLimitations,
    evaluation: &SelectionEvaluation,
    findings: &mut Vec<Finding>,
) {
    let omissions = limitations
        .omissions
        .iter()
        .filter(|omission| valid_omission(omission))
        .map(|omission| (omission.feature_id.as_str(), omission))
        .collect::<BTreeMap<_, _>>();
    let features = manifest
        .collection
        .features
        .iter()
        .map(|feature| (feature.id.as_str(), feature))
        .collect::<BTreeMap<_, _>>();

    for control in &evaluation.view.controls {
        let desired = evaluation
            .normalized_selection
            .features
            .get(&control.id)
            .copied()
            .unwrap_or(false);
        if desired
            && !control.selected
            && matches!(control.decision, Decision::Default | Decision::Mandatory)
            && !omission_or_ancestor(&control.id, &features, &omissions)
        {
            release_error(
                findings,
                RULE_RELEASE_OMISSION,
                format!(
                    "desired {:?} outcome {:?} is omitted without an approved visible limitation",
                    control.decision, control.id
                ),
            );
        }
    }
}

fn omission_or_ancestor(
    feature_id: &str,
    features: &BTreeMap<&str, &Feature>,
    omissions: &BTreeMap<&str, &Omission>,
) -> bool {
    let mut current = Some(feature_id);
    for _ in 0..=features.len() {
        let Some(id) = current else {
            return false;
        };
        if omissions.contains_key(id) {
            return true;
        }
        current = features
            .get(id)
            .and_then(|feature| feature.parent.as_deref());
    }
    false
}

fn valid_omission(omission: &Omission) -> bool {
    omission.approved_by.trim() == "Christopher"
        && is_date(&omission.approved_on)
        && !omission.reason.trim().is_empty()
        && !omission.user_facing_limitation.trim().is_empty()
}

fn is_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

fn is_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.contains(':')
        && !value.contains('\\')
        && path
            .components()
            .all(|component| matches!(component, PathComponent::Normal(_)))
}

fn release_error(findings: &mut Vec<Finding>, rule: &'static str, message: String) {
    findings.push(Finding {
        severity: Severity::Error,
        rule,
        message,
    });
}
