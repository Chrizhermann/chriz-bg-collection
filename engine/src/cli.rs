//! Deterministic, presentation-neutral operations exposed by the headless installer CLI.
//!
//! Argument parsing stays in the binary. This module owns recipe evaluation, game inspection,
//! managed-state reporting, and diagnostics selection so the desktop bridge can reuse the same
//! behavior without invoking a subprocess.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::acquire::{
    extract_archive, materialize, provide_manual_archive, ArchiveFormat, ArchiveLimits,
    ArchiveMode, ArchiveRequirements, ArtifactCache, CacheDisposition, DownloadRequest,
    ExtractedArtifact, MaterializationRequest,
};
use crate::diagnostics::{export_diagnostics, DiagnosticsBundle, DiagnosticsRequest};
use crate::digest::{plan_digest, selection_digest, sha256_bytes};
use crate::events::{EngineEvent, EventSink};
use crate::games::{
    discover_installed_games, inspect_game_path, Eligibility, GameCandidate, GameProfiles,
    GameRole, Storefront, SystemFileSystem,
};
use crate::lock::LockError;
use crate::manifest::{
    AcquisitionPolicy, ArchiveKind as ManifestArchiveKind, ArchiveRootRule, Artifact, Collection,
    GameRoot, InvocationMode, ModFile, PresetFile,
};
use crate::orchestrator::{
    run_campaign, ArtifactAcquirer, ArtifactKind, ArtifactMaterializer, BuiltInvocation,
    CampaignClock, CampaignOutcome, CampaignPreflight, CampaignRequest, InstallLogVerifier,
    InstallReconciliation, InvocationBuilder, MaterializationOutcome, MaterializationTask,
    MutationCheck, ProcessResult, ProcessRunner, ReceiptDraft, ReceiptWriter, StagingService,
    StepAttempt, StepFailure,
};
use crate::preflight::{
    initial_preflight, recheck_staging_target_before_mutation, recheck_target_before_mutation,
    InitialPreflight, RequiredInput, SpaceRequirement,
};
use crate::receipt::{
    ArtifactCacheOutcome, ArtifactReceipt, FinalLogReceipt, FinalReceiptState, InstallReceipt,
    LogComponentReceipt, LogDiffReceipt, ManagedReceiptWriter, PromptReceipt, ReceiptEvidence,
    ReceiptOutcome, ReceiptStore, ReceiptVersions, RunAttemptReceipt, RunReceipt, RunTiming,
    SourceGameReceipt, WeiDuToolReceipt, RECEIPT_SCHEMA_VERSION,
};
use crate::recipe_view::{evaluate, SelectionEvaluation};
use crate::registry::ManagedInstallRegistry;
use crate::resolve::{InstallPlan, PlannedRun, Selection};
use crate::session::{
    CampaignCreated, FrozenIdentity, SessionReplay, SessionStore, SourceGameFingerprints,
};
use crate::stage::{
    finalize_game_identity, read_engine_name, reserve_save_identity, stage_bgee_sod,
    stage_game_copy, DocumentsLocator, ManagedLayout, ReservedSaveIdentity, SystemDocuments,
};
use crate::validate::{self, Severity};
use crate::weidu::invocation::{
    build as build_invocation, verify_tool_contract, Invocation, InvocationInput, ResolvedPrompt,
    StagedRoots, VerifiedWeidu,
};
use crate::weidu::log::{parse_active_entries, parse_terminal_statuses, DebugStatus, LogEntry};
use crate::weidu::runner::{
    parse_prompt_results, run_controlled, PromptAnswerEvidence, RunOutcome, RunnerControlHandle,
    RunnerRequest, PROMPT_RESULTS_FILE_NAME,
};
use crate::weidu::verify::{reconcile as reconcile_weidu, ExpectedRun, Reconciliation};
use crate::Manifest;

const GAME_PROFILE_DIRECTORY: &str = "game-builds";
const STATE_DIRECTORY: &str = ".chriz";
const ATTEMPTS_DIRECTORY: &str = "attempts";
const ATTEMPT_RECEIPT_FILE: &str = "receipt.json";
const CLI_FROZEN_RECIPE_SCHEMA: u32 = 1;
const APPLICATION_DATA_DIRECTORY: &str = "Chriz BG Collection";
const LOCKS_DIRECTORY: &str = "locks";
const EVIDENCE_DIRECTORY: &str = "evidence";
const BEFORE_LOG_FILE: &str = "before.log";
const AFTER_LOG_FILE: &str = "after.log";
const INVOCATION_FILE: &str = "invocation.json";
const PROCESS_RESULT_FILE: &str = "process-result.json";
const PROCESS_OUTPUT_FILE: &str = "process-output.log";
const STDOUT_FILE: &str = "stdout.log";
const STDERR_FILE: &str = "stderr.log";
const DEBUG_LOG_FILE: &str = "weidu.debug.log";
static CAMPAIGN_SEQUENCE: AtomicU64 = AtomicU64::new(0);

type InterruptHandler = Box<dyn FnMut() + Send + 'static>;

/// Install the process-wide Ctrl+C handler used by one CLI install or resume command.
///
/// The handler requests explicit runner cancellation; the command itself remains blocked until
/// the supervised process tree has terminated and durable attempt evidence has been flushed.
pub fn install_interrupt_handler(controls: &RunnerControlHandle) -> Result<(), CliError> {
    install_interrupt_handler_with(controls, ctrlc::set_handler)
}

fn install_interrupt_handler_with<E>(
    controls: &RunnerControlHandle,
    register: impl FnOnce(InterruptHandler) -> Result<(), E>,
) -> Result<(), CliError>
where
    E: std::fmt::Display,
{
    let controls = controls.clone();
    register(Box::new(move || controls.cancel())).map_err(|error| {
        CliError::new(
            "interrupt_handler_failed",
            format!("could not install the Ctrl+C cancellation handler: {error}"),
        )
    })
}

/// Validation strength requested by the caller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationProfile {
    /// Permit authoring-only warnings while still rejecting structural errors.
    #[default]
    Authoring,
    /// Treat every current warning as release-blocking until Task 19's richer gate exists.
    PublicAlpha,
}

impl ValidationProfile {
    /// Stable CLI spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authoring => "authoring",
            Self::PublicAlpha => "public-alpha",
        }
    }
}

/// Stable finding detached from internal static lifetimes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationFinding {
    /// `warning` or `error`.
    pub severity: String,
    /// Stable validation rule id.
    pub rule: String,
    /// Player/author-facing detail.
    pub message: String,
}

/// Deterministic validation result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationReport {
    /// Canonical recipe directory.
    pub recipe: PathBuf,
    /// Requested validation strength.
    pub profile: ValidationProfile,
    /// Stable validation findings.
    pub findings: Vec<ValidationFinding>,
}

/// Raw semantic overrides parsed from repeatable CLI options.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectionOverrides {
    /// `feature-id=on|off` values.
    pub features: Vec<String>,
    /// `feature-id/input-id=<typed-value>` values.
    pub inputs: Vec<String>,
}

/// Deterministic evaluated-plan result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanReport {
    /// Canonical recipe directory.
    pub recipe: PathBuf,
    /// Preset used as the semantic base selection.
    pub preset: String,
    /// Exact normalized selection, findings, projection, and component plan.
    pub evaluation: SelectionEvaluation,
}

/// One explicit-path inspection result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameInspectionReport {
    /// Canonical candidate selected from the immutable storefront profiles.
    pub candidate: GameCandidate,
}

/// Existing terminal receipt selected for `report` and `diagnostics`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedReport {
    /// Canonical managed root.
    pub managed_root: PathBuf,
    /// Exact latest immutable terminal receipt.
    pub receipt: InstallReceipt,
}

/// Inputs for one new CLI-owned managed installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallCommandRequest {
    /// Executable recipe directory to freeze.
    pub recipe: PathBuf,
    /// Semantic preset used as the selection base.
    pub preset: String,
    /// Target platform passed to recipe evaluation.
    pub platform: String,
    /// Repeatable semantic feature and typed-input overrides.
    pub overrides: SelectionOverrides,
    /// Read-only BGEE plus SoD source.
    pub bg1: PathBuf,
    /// Read-only BG2EE source.
    pub bg2: PathBuf,
    /// New isolated managed installation root.
    pub managed_root: PathBuf,
    /// Explicit shared immutable-artifact cache root.
    pub cache: PathBuf,
}

/// Exact engine-owned identity shown at Review and required again at Start.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallReviewIdentity {
    /// Digest of the complete serialized recipe, profiles, sources, selection, and plan.
    pub recipe_payload_sha256: String,
    /// Canonical normalized semantic selection digest.
    pub selection_sha256: String,
    /// Canonical resolved component-plan digest.
    pub plan_sha256: String,
    /// Exact clean source-game fingerprints.
    pub source_games: SourceGameFingerprints,
    /// Canonical prospective managed-copy root.
    pub managed_root: PathBuf,
    /// Canonical content-addressed cache root.
    pub cache_root: PathBuf,
}

/// Machine-readable result retained even when a campaign stops on a safe guard.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignReport {
    /// Frozen managed-install identity.
    pub install_id: String,
    /// Canonical managed root claimed by the campaign.
    pub managed_root: PathBuf,
    /// Digest of the exact frozen plan used by install or resume.
    pub plan_sha256: String,
    /// Terminal campaign status.
    pub status: CampaignStatus,
}

/// Stable terminal spelling for CLI and desktop consumers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CampaignStatus {
    /// Every frozen step completed and its receipt was published.
    Complete,
    /// One retryable step failed and can be resumed from the frozen campaign.
    Failed {
        /// Stable failed pipeline step.
        step_id: String,
        /// Durable, user-facing reason.
        reason: String,
    },
    /// Existing evidence made this managed copy unsafe to continue.
    FreshCopyRequired {
        /// Stable step at which unsafe evidence was found.
        step_id: String,
        /// Durable, user-facing reason.
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenCliRecipe {
    schema: u32,
    collection: Collection,
    artifacts: BTreeMap<String, Artifact>,
    mods: BTreeMap<String, ModFile>,
    presets: BTreeMap<String, PresetFile>,
    profiles: GameProfiles,
    plan: InstallPlan,
    normalized_selection: crate::recipe_view::NormalizedSelection,
    preset: String,
    bg1: FrozenSource,
    bg2: FrozenSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenSource {
    role: GameRole,
    storefront: Storefront,
    root: PathBuf,
    build: Option<String>,
    fingerprint: String,
}

struct PreparedInstall {
    created: CampaignCreated,
    frozen: FrozenCliRecipe,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct FrozenRecipeEnvelope<'a> {
    schema: u32,
    kind: &'a str,
    payload_sha256: &'a str,
}

/// Stable command failure with a machine-readable category.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct CliError {
    code: &'static str,
    message: String,
    report: Option<Box<CampaignReport>>,
    events: Box<[EngineEvent]>,
}

impl CliError {
    /// Stable machine-readable error code.
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Campaign context retained for deterministic machine-readable failures.
    pub fn campaign_report(&self) -> Option<&CampaignReport> {
        self.report.as_deref()
    }

    /// Buffered engine events attached to a JSON-mode campaign failure.
    pub fn events(&self) -> &[EngineEvent] {
        &self.events
    }

    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            report: None,
            events: Box::default(),
        }
    }

    /// Convert a non-success campaign result into a stable CLI failure.
    pub fn from_campaign(report: CampaignReport) -> Self {
        let (code, message) = match &report.status {
            CampaignStatus::Complete => (
                "invalid_campaign_result",
                "a complete campaign cannot be represented as an error".to_owned(),
            ),
            CampaignStatus::Failed { step_id, reason } => {
                let code = if step_id == "preflight" && reason.contains("source_not_fresh") {
                    "source_not_fresh"
                } else if step_id.starts_with("acquire:")
                    || reason.contains("required artifact")
                    || reason.contains("required tool")
                {
                    "required_input_unavailable"
                } else {
                    "campaign_failed"
                };
                (code, reason.clone())
            }
            CampaignStatus::FreshCopyRequired { reason, .. } => {
                ("fresh_copy_required", reason.clone())
            }
        };
        Self {
            code,
            message,
            report: Some(Box::new(report)),
            events: Box::default(),
        }
    }

    /// Attach deterministically ordered engine events captured by a machine-mode caller.
    pub fn with_events(mut self, events: Vec<EngineEvent>) -> Self {
        self.events = events.into_boxed_slice();
        self
    }
}

/// Load and evaluate a recipe under one validation profile.
pub fn validate_recipe(
    recipe: &Path,
    profile: ValidationProfile,
) -> Result<ValidationReport, CliError> {
    let manifest = load_recipe(recipe)?;
    let findings = validate::validate(&manifest)
        .into_iter()
        .map(|finding| ValidationFinding {
            severity: match finding.severity {
                Severity::Warning => "warning",
                Severity::Error => "error",
            }
            .to_owned(),
            rule: finding.rule.to_owned(),
            message: finding.message,
        })
        .collect::<Vec<_>>();
    let blocked = findings.iter().any(|finding| {
        finding.severity == "error"
            || (profile == ValidationProfile::PublicAlpha && finding.severity == "warning")
    });
    if blocked {
        let details = findings
            .iter()
            .map(|finding| {
                format!(
                    "{}[{}]: {}",
                    finding.severity, finding.rule, finding.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Err(CliError::new(
            "validation_failed",
            format!(
                "recipe {} failed validation:\n{details}",
                manifest.root.display()
            ),
        ));
    }
    Ok(ValidationReport {
        recipe: manifest.root,
        profile,
        findings,
    })
}

/// Resolve a named preset plus semantic feature/input overrides.
pub fn plan_recipe(
    recipe: &Path,
    preset_id: &str,
    platform: &str,
    overrides: &SelectionOverrides,
) -> Result<PlanReport, CliError> {
    let manifest = load_recipe(recipe)?;
    validate::check(&manifest).map_err(|error| {
        CliError::new(
            "validation_failed",
            format!(
                "recipe {} failed validation:\n{error}",
                manifest.root.display()
            ),
        )
    })?;
    let choices = selection_choices(&manifest, preset_id, overrides)?;
    let evaluation = evaluate(
        &manifest,
        &Selection {
            platform: platform.to_owned(),
            choices,
        },
    )
    .map_err(|error| CliError::new("invalid_selection", error.to_string()))?;
    Ok(PlanReport {
        recipe: manifest.root,
        preset: preset_id.to_owned(),
        evaluation,
    })
}

/// Discover installed games using the recipe's immutable profile directory.
pub fn discover_games(recipe: &Path) -> Result<Vec<GameCandidate>, CliError> {
    let profiles = load_profiles(recipe)?;
    discover_installed_games(&profiles)
        .map_err(|error| CliError::new("game_discovery_failed", error.to_string()))
}

/// Inspect an explicit path against all applicable storefront profiles.
///
/// A raw path does not establish storefront provenance. If more than one storefront profile
/// accepts it, the command refuses ambiguity rather than silently relabeling the source.
pub fn inspect_game(
    recipe: &Path,
    role: GameRole,
    path: &Path,
) -> Result<GameInspectionReport, CliError> {
    let profiles = load_profiles(recipe)?;
    let fs_provider = SystemFileSystem;
    let mut candidates = [Storefront::Steam, Storefront::Gog]
        .into_iter()
        .filter(|storefront| profiles.matching(role, *storefront).next().is_some())
        .map(|storefront| inspect_game_path(&profiles, &fs_provider, role, storefront, path))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| CliError::new("game_inspection_failed", error.to_string()))?;
    if candidates.is_empty() {
        return Err(CliError::new(
            "game_inspection_failed",
            format!("recipe {} has no profile for {role:?}", recipe.display()),
        ));
    }
    let usable = candidates
        .iter()
        .filter(|candidate| candidate.eligibility != Eligibility::Ineligible)
        .count();
    if usable > 1 {
        return Err(CliError::new(
            "ambiguous_storefront",
            format!(
                "{} matches more than one storefront profile; choose a discovered installation instead",
                path.display()
            ),
        ));
    }
    candidates.sort_by(compare_candidates);
    let candidate = candidates
        .into_iter()
        .next()
        .expect("nonempty candidates were checked above");
    Ok(GameInspectionReport { candidate })
}

/// Freeze and start one new managed campaign through the Task-13 state machine.
///
/// The immutable identities are frozen from authored artifact metadata. The production adapter
/// then acquires, stages, materializes, invokes, reconciles, and records only that frozen plan.
pub fn install_campaign<S: EventSink + Sync>(
    request: &InstallCommandRequest,
    sink: &S,
) -> Result<CampaignReport, CliError> {
    install_campaign_controlled(request, sink, &RunnerControlHandle::new())
}

/// Freeze and start a managed campaign controlled by the caller's interrupt handle.
pub fn install_campaign_controlled<S: EventSink + Sync>(
    request: &InstallCommandRequest,
    sink: &S,
    controls: &RunnerControlHandle,
) -> Result<CampaignReport, CliError> {
    let prepared = prepare_install(request)?;
    execute_frozen_campaign(prepared.created, prepared.frozen, sink, controls)
}

/// Resolve the complete engine-owned identity that a UI must show and bind at Review.
pub fn review_install(request: &InstallCommandRequest) -> Result<InstallReviewIdentity, CliError> {
    let prepared = prepare_install(request)?;
    Ok(install_review_identity(&prepared.created))
}

/// Start only when the current inputs still match the identity accepted at Review.
pub fn install_campaign_reviewed<S: EventSink + Sync>(
    request: &InstallCommandRequest,
    expected: &InstallReviewIdentity,
    sink: &S,
    controls: &RunnerControlHandle,
) -> Result<CampaignReport, CliError> {
    let prepared = prepare_install(request)?;
    let current = install_review_identity(&prepared.created);
    if current != *expected {
        return Err(CliError::new(
            "review_changed",
            "the recipe, selection, sources, destination, or cache changed after Review; review the build again",
        ));
    }
    execute_frozen_campaign(prepared.created, prepared.frozen, sink, controls)
}

fn prepare_install(request: &InstallCommandRequest) -> Result<PreparedInstall, CliError> {
    let manifest = load_recipe(&request.recipe)?;
    validate::check(&manifest).map_err(|error| {
        CliError::new(
            "validation_failed",
            format!(
                "recipe {} failed validation:\n{error}",
                manifest.root.display()
            ),
        )
    })?;
    let choices = selection_choices(&manifest, &request.preset, &request.overrides)?;
    let evaluation = evaluate(
        &manifest,
        &Selection {
            platform: request.platform.clone(),
            choices,
        },
    )
    .map_err(|error| CliError::new("invalid_selection", error.to_string()))?;
    let profiles = load_profiles(&manifest.root)?;
    let bg1 = inspect_source_for_campaign(&profiles, GameRole::BgeeSod, &request.bg1, "BGEE+SoD")?;
    let bg2 = inspect_source_for_campaign(&profiles, GameRole::Bg2ee, &request.bg2, "BG2EE")?;
    let frozen_bg1 = FrozenSource::from_candidate(&bg1, "BGEE+SoD")?;
    let frozen_bg2 = FrozenSource::from_candidate(&bg2, "BG2EE")?;
    let managed_root = prospective_direct_path(&request.managed_root, "managed root")?;
    let cache_root = canonical_existing_directory(&request.cache, "cache root")?;
    reject_campaign_path_overlaps(&managed_root, &cache_root, &bg1.root, &bg2.root)?;
    let frozen = FrozenCliRecipe {
        schema: CLI_FROZEN_RECIPE_SCHEMA,
        collection: manifest.collection.clone(),
        artifacts: manifest.artifacts.clone(),
        mods: manifest.mods.clone(),
        presets: manifest.presets.clone(),
        profiles,
        plan: evaluation.plan.clone(),
        normalized_selection: evaluation.normalized_selection.clone(),
        preset: request.preset.clone(),
        bg1: frozen_bg1,
        bg2: frozen_bg2,
    };
    let recipe_payload = serialize_frozen_recipe(&frozen)?;
    let recipe_payload_sha256 = sha256_bytes(&recipe_payload);
    let recipe_envelope = serde_json::to_vec(&FrozenRecipeEnvelope {
        schema: CLI_FROZEN_RECIPE_SCHEMA,
        kind: "local-cli-frozen-recipe",
        payload_sha256: &recipe_payload_sha256,
    })
    .map_err(|error| CliError::new("recipe_freeze_failed", error.to_string()))?;
    let seed = new_campaign_seed(&managed_root, &recipe_payload_sha256)?;
    let created = CampaignCreated {
        install_id: format!("install-{}", &seed[..20]),
        attempt_id: format!("attempt-{}", &seed[20..40]),
        managed_root: managed_root.clone(),
        cache_root,
        recipe_payload,
        recipe_payload_sha256,
        recipe_envelope_sha256: sha256_bytes(&recipe_envelope),
        recipe_envelope,
        selection_sha256: selection_digest(&evaluation.normalized_selection)
            .map_err(|error| CliError::new("recipe_freeze_failed", error.to_string()))?,
        normalized_selection: evaluation.normalized_selection,
        plan_sha256: plan_digest(&evaluation.plan)
            .map_err(|error| CliError::new("recipe_freeze_failed", error.to_string()))?,
        source_games: SourceGameFingerprints {
            bg1: frozen.bg1.fingerprint.clone(),
            bg2: frozen.bg2.fingerprint.clone(),
        },
        artifact_identities: frozen_payload_identities(&frozen)?,
        tool_identities: frozen_tool_identities(&frozen)?,
        staged_bg1: managed_root.join("bg1"),
        staged_bg2: managed_root.join("game"),
    };
    Ok(PreparedInstall { created, frozen })
}

fn install_review_identity(created: &CampaignCreated) -> InstallReviewIdentity {
    InstallReviewIdentity {
        recipe_payload_sha256: created.recipe_payload_sha256.clone(),
        selection_sha256: created.selection_sha256.clone(),
        plan_sha256: created.plan_sha256.clone(),
        source_games: created.source_games.clone(),
        managed_root: created.managed_root.clone(),
        cache_root: created.cache_root.clone(),
    }
}

/// Resume only the campaign identity and recipe payload frozen below `managed_root`.
pub fn resume_campaign<S: EventSink + Sync>(
    managed_root: &Path,
    sink: &S,
) -> Result<CampaignReport, CliError> {
    resume_campaign_controlled(managed_root, sink, &RunnerControlHandle::new())
}

/// Resume a frozen campaign controlled by the caller's interrupt handle.
pub fn resume_campaign_controlled<S: EventSink + Sync>(
    managed_root: &Path,
    sink: &S,
    controls: &RunnerControlHandle,
) -> Result<CampaignReport, CliError> {
    resume_campaign_internal(managed_root, None, sink, controls)
}

/// Resume only when the requested install id owns the frozen campaign ledger at this path.
pub fn resume_campaign_controlled_expected<S: EventSink + Sync>(
    managed_root: &Path,
    expected_install_id: &str,
    sink: &S,
    controls: &RunnerControlHandle,
) -> Result<CampaignReport, CliError> {
    resume_campaign_internal(managed_root, Some(expected_install_id), sink, controls)
}

fn resume_campaign_internal<S: EventSink + Sync>(
    managed_root: &Path,
    expected_install_id: Option<&str>,
    sink: &S,
    controls: &RunnerControlHandle,
) -> Result<CampaignReport, CliError> {
    let managed_root = canonical_existing_directory(managed_root, "managed root")?;
    let store = SessionStore::open(&managed_root).map_err(|error| {
        CliError::new(
            "resume_unavailable",
            format!(
                "could not open frozen campaign {}: {error}",
                managed_root.display()
            ),
        )
    })?;
    let replay = store.replay().map_err(|error| {
        CliError::new(
            "resume_unavailable",
            format!(
                "could not replay frozen campaign {}: {error}",
                managed_root.display()
            ),
        )
    })?;
    let created = replay.created().clone();
    if expected_install_id.is_some_and(|expected| expected != created.install_id) {
        return Err(CliError::new(
            "resume_identity_mismatch",
            format!(
                "requested install id does not own the frozen campaign at {}",
                managed_root.display()
            ),
        ));
    }
    let frozen: FrozenCliRecipe =
        serde_json::from_slice(&created.recipe_payload).map_err(|error| {
            CliError::new(
                "resume_unavailable",
                format!("frozen recipe payload is not a supported CLI snapshot: {error}"),
            )
        })?;
    validate_frozen_cli_recipe(&created, &frozen)?;
    execute_frozen_campaign(created, frozen, sink, controls)
}

fn execute_frozen_campaign<S: EventSink + Sync>(
    created: CampaignCreated,
    frozen: FrozenCliRecipe,
    sink: &S,
    controls: &RunnerControlHandle,
) -> Result<CampaignReport, CliError> {
    let app_data = application_data_root()?;
    let registry_root = app_data.join(LOCKS_DIRECTORY);
    let current_bg1 = inspect_frozen_source(&frozen.profiles, &frozen.bg1)?;
    let current_bg2 = inspect_frozen_source(&frozen.profiles, &frozen.bg2)?;
    let request = CampaignRequest {
        registry_root,
        created: created.clone(),
        plan: frozen.plan.clone(),
    };
    let mut dependencies = GuardedCliDependencies::new(
        &created,
        &frozen,
        current_bg1,
        current_bg2,
        app_data,
        sink,
        controls.clone(),
    );
    let outcome = run_campaign(&request, &mut dependencies, sink)
        .map_err(|error| map_orchestrator_error(error, &created))?;
    Ok(CampaignReport {
        install_id: created.install_id,
        managed_root: created.managed_root,
        plan_sha256: created.plan_sha256,
        status: match outcome {
            CampaignOutcome::Complete => CampaignStatus::Complete,
            CampaignOutcome::Failed { step_id, reason } => {
                CampaignStatus::Failed { step_id, reason }
            }
            CampaignOutcome::FreshCopyRequired { step_id, reason } => {
                CampaignStatus::FreshCopyRequired { step_id, reason }
            }
        },
    })
}

/// Read the newest immutable attempt receipt without trusting mutable summary state.
pub fn report_managed_install(managed_root: &Path) -> Result<ManagedReport, CliError> {
    let managed_root = canonical_direct_directory(managed_root, "managed root")?;
    let store = SessionStore::open(&managed_root).map_err(|error| {
        CliError::new(
            "report_unavailable",
            format!(
                "could not open frozen campaign {}: {error}",
                managed_root.display()
            ),
        )
    })?;
    let replay = store.replay().map_err(|error| {
        CliError::new(
            "report_unavailable",
            format!(
                "could not replay frozen campaign {}: {error}",
                managed_root.display()
            ),
        )
    })?;
    let created = replay.created();
    let frozen: FrozenCliRecipe =
        serde_json::from_slice(&created.recipe_payload).map_err(|error| {
            CliError::new(
                "report_unavailable",
                format!("frozen recipe payload is not a supported CLI snapshot: {error}"),
            )
        })?;
    validate_frozen_cli_recipe(created, &frozen).map_err(|error| {
        CliError::new(
            "report_unavailable",
            format!("frozen campaign identity is invalid: {error}"),
        )
    })?;
    let attempts = managed_root.join(STATE_DIRECTORY).join(ATTEMPTS_DIRECTORY);
    validate_direct_directory(&attempts, "attempt receipt directory")?;
    let mut receipts = Vec::new();
    for entry in fs::read_dir(&attempts).map_err(|source| {
        CliError::new(
            "report_unavailable",
            format!("{}: {source}", attempts.display()),
        )
    })? {
        let entry = entry.map_err(|source| {
            CliError::new(
                "report_unavailable",
                format!("{}: {source}", attempts.display()),
            )
        })?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|source| {
            CliError::new(
                "report_unavailable",
                format!("{}: {source}", entry.path().display()),
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(CliError::new(
                "report_unavailable",
                format!(
                    "attempt entry is not a direct directory: {}",
                    entry.path().display()
                ),
            ));
        }
        let directory_name = entry.file_name().into_string().map_err(|_| {
            CliError::new(
                "report_unavailable",
                format!(
                    "attempt directory name is not valid Unicode: {}",
                    entry.path().display()
                ),
            )
        })?;
        let path = entry.path().join(ATTEMPT_RECEIPT_FILE);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound
                    && directory_name == created.attempt_id =>
            {
                continue;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CliError::new(
                    "report_unavailable",
                    format!(
                        "attempt directory has no receipt: {}",
                        entry.path().display()
                    ),
                ));
            }
            Err(source) => {
                return Err(CliError::new(
                    "report_unavailable",
                    format!("{}: {source}", path.display()),
                ));
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(CliError::new(
                "report_unavailable",
                format!(
                    "attempt receipt is not a direct regular file: {}",
                    path.display()
                ),
            ));
        }
        let bytes = fs::read(&path).map_err(|source| {
            CliError::new(
                "report_unavailable",
                format!("{}: {source}", path.display()),
            )
        })?;
        let receipt: InstallReceipt = serde_json::from_slice(&bytes).map_err(|source| {
            CliError::new(
                "report_unavailable",
                format!("invalid receipt JSON at {}: {source}", path.display()),
            )
        })?;
        validate_report_receipt(&receipt, &directory_name, created, &frozen, &replay).map_err(
            |message| {
                CliError::new(
                    "report_unavailable",
                    format!("invalid receipt at {}: {message}", path.display()),
                )
            },
        )?;
        receipts.push(receipt);
    }
    receipts.sort_by(|left, right| {
        right
            .completed_at_millis
            .cmp(&left.completed_at_millis)
            .then_with(|| right.attempt_id.cmp(&left.attempt_id))
    });
    let receipt = receipts.into_iter().next().ok_or_else(|| {
        CliError::new(
            "report_unavailable",
            format!(
                "no immutable attempt receipt exists under {}",
                attempts.display()
            ),
        )
    })?;
    Ok(ManagedReport {
        managed_root,
        receipt,
    })
}

fn validate_report_receipt(
    receipt: &InstallReceipt,
    directory_name: &str,
    created: &CampaignCreated,
    frozen: &FrozenCliRecipe,
    replay: &SessionReplay,
) -> Result<(), String> {
    for (label, value) in [
        ("install id", receipt.install_id.as_str()),
        ("attempt id", receipt.attempt_id.as_str()),
        ("evidence attempt id", receipt.evidence_attempt_id.as_str()),
    ] {
        validate_cli_identifier(value, label)?;
    }
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION {
        return Err(format!(
            "unsupported schema {}; expected {RECEIPT_SCHEMA_VERSION}",
            receipt.schema_version
        ));
    }
    if receipt.attempt_id != directory_name {
        return Err(format!(
            "attempt id {:?} does not match directory {:?}",
            receipt.attempt_id, directory_name
        ));
    }
    if receipt.install_id != created.install_id {
        return Err("install id differs from the frozen campaign".to_owned());
    }
    if receipt.evidence_attempt_id != created.attempt_id {
        return Err("evidence attempt id differs from the frozen campaign".to_owned());
    }
    if receipt.managed_root != created.managed_root
        || receipt.staged_bg1 != created.staged_bg1
        || receipt.staged_bg2 != created.staged_bg2
    {
        return Err("managed or staged paths differ from the frozen campaign".to_owned());
    }
    let started_at = replay
        .records
        .first()
        .ok_or_else(|| "campaign ledger has no creation record".to_owned())?
        .recorded_at;
    if receipt.started_at_millis != started_at
        || receipt.completed_at_millis < receipt.started_at_millis
    {
        return Err("receipt timestamps contradict the campaign ledger".to_owned());
    }
    validate_receipt_outcome_link(receipt, created, replay)?;

    if receipt.recipe_payload_sha256 != created.recipe_payload_sha256
        || receipt.recipe_envelope_sha256 != created.recipe_envelope_sha256
    {
        return Err("recipe digests differ from the frozen campaign".to_owned());
    }
    if receipt.normalized_selection != created.normalized_selection
        || receipt.selection_sha256 != created.selection_sha256
        || selection_digest(&receipt.normalized_selection).map_err(|error| error.to_string())?
            != created.selection_sha256
    {
        return Err("normalized selection differs from the frozen campaign".to_owned());
    }
    if receipt.plan != frozen.plan
        || receipt.plan_sha256 != created.plan_sha256
        || plan_digest(&receipt.plan).map_err(|error| error.to_string())? != created.plan_sha256
    {
        return Err("resolved plan differs from the frozen campaign".to_owned());
    }

    let expected_sources = vec![frozen.bg1.receipt(), frozen.bg2.receipt()];
    if receipt.source_games != expected_sources {
        return Err("source-game evidence differs from the frozen campaign".to_owned());
    }
    if receipt.versions.application.trim().is_empty()
        || receipt.versions.engine.trim().is_empty()
        || receipt.versions.manifest_schema != frozen.collection.schema
        || receipt.versions.recipe != format!("local-{}", &created.recipe_payload_sha256[..12])
    {
        return Err("receipt versions contradict the frozen recipe".to_owned());
    }

    let succeeded = matches!(receipt.outcome, ReceiptOutcome::Succeeded);
    validate_receipt_identities(
        "artifact",
        receipt
            .artifacts
            .iter()
            .map(|item| (&item.id, &item.version, item.length, &item.sha256)),
        &created.artifact_identities,
        succeeded,
    )?;
    validate_receipt_identities(
        "WeiDU tool",
        receipt
            .weidu_tools
            .iter()
            .map(|item| (&item.id, &item.version, item.length, &item.sha256)),
        &created.tool_identities,
        succeeded,
    )?;
    let mut seen_runs = BTreeSet::new();
    for run in &receipt.runs {
        let planned = receipt
            .plan
            .runs
            .iter()
            .find(|planned| planned.run_id == run.run_id);
        if !seen_runs.insert(run.run_id.as_str())
            || !planned.is_some_and(|planned| {
                run.target == planned.target && run.components == planned.components
            })
        {
            return Err(format!(
                "run evidence for {:?} differs from the frozen plan or is duplicated",
                run.run_id
            ));
        }
        crate::receipt::validate_run_attempt_sequence(
            run,
            planned.expect("matching planned run was established above"),
            succeeded,
        )?;
    }
    if receipt.runs.len() != receipt.plan.runs.len() {
        return Err("receipt does not have exactly one logical row per planned run".to_owned());
    }
    Ok(())
}

fn validate_cli_identifier(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(format!(
            "{label} must contain 1-128 ASCII letters, digits, '-' or '_'"
        ));
    }
    Ok(())
}

fn validate_receipt_outcome_link(
    receipt: &InstallReceipt,
    created: &CampaignCreated,
    replay: &SessionReplay,
) -> Result<(), String> {
    match &receipt.outcome {
        ReceiptOutcome::Succeeded => {
            if receipt.attempt_id != created.attempt_id || receipt.final_state.is_none() {
                return Err(
                    "successful receipt must use the campaign attempt and final state".to_owned(),
                );
            }
            if !replay
                .records
                .iter()
                .any(|record| record.recorded_at == receipt.completed_at_millis)
            {
                return Err("successful receipt timestamp is absent from the ledger".to_owned());
            }
        }
        ReceiptOutcome::Failed { step_id, detail } => {
            if receipt.final_state.is_some() {
                return Err("failed receipt unexpectedly contains final state".to_owned());
            }
            validate_terminal_receipt_link(
                receipt,
                replay,
                &format!("failed\0{step_id}\0{detail}"),
            )?;
        }
        ReceiptOutcome::FreshCopyRequired { step_id, detail } => {
            if receipt.final_state.is_some() {
                return Err("fresh-copy receipt unexpectedly contains final state".to_owned());
            }
            validate_terminal_receipt_link(
                receipt,
                replay,
                &format!("fresh-copy-required\0{step_id}\0{detail}"),
            )?;
        }
    }
    Ok(())
}

fn validate_terminal_receipt_link(
    receipt: &InstallReceipt,
    replay: &SessionReplay,
    outcome_identity: &str,
) -> Result<(), String> {
    let suffix = receipt
        .attempt_id
        .strip_prefix("terminal-")
        .ok_or_else(|| "terminal receipt has no terminal attempt id".to_owned())?;
    let (sequence_text, digest) = suffix
        .split_once('-')
        .ok_or_else(|| "terminal receipt attempt id has invalid shape".to_owned())?;
    if sequence_text.len() != 10
        || !sequence_text.bytes().all(|byte| byte.is_ascii_digit())
        || digest.len() != 16
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("terminal receipt attempt id has invalid shape".to_owned());
    }
    let sequence = sequence_text
        .parse::<usize>()
        .map_err(|_| "terminal receipt sequence does not fit this platform".to_owned())?;
    let expected = format!(
        "terminal-{sequence_text}-{}",
        &sha256_bytes(outcome_identity.as_bytes())[..16]
    );
    if receipt.attempt_id != expected {
        return Err("terminal receipt outcome digest does not match its attempt id".to_owned());
    }
    let record = replay
        .records
        .get(sequence)
        .filter(|record| record.sequence as usize == sequence)
        .ok_or_else(|| "terminal receipt sequence is absent from the ledger".to_owned())?;
    if record.recorded_at != receipt.completed_at_millis {
        return Err("terminal receipt timestamp differs from its ledger record".to_owned());
    }
    Ok(())
}

fn validate_receipt_identities<'a>(
    label: &str,
    evidence: impl Iterator<Item = (&'a String, &'a String, u64, &'a String)>,
    frozen: &[FrozenIdentity],
    require_complete: bool,
) -> Result<(), String> {
    let evidence = evidence.collect::<Vec<_>>();
    if require_complete && evidence.len() != frozen.len() {
        return Err(format!(
            "successful receipt has incomplete {label} evidence"
        ));
    }
    let mut seen = BTreeSet::new();
    for (id, version, length, sha256) in evidence {
        let matches = frozen.iter().filter(|identity| {
            id == &identity.id
                && version == &identity.version
                && length == identity.length
                && sha256.eq_ignore_ascii_case(&identity.sha256)
        });
        if !seen.insert(id) || matches.count() != 1 {
            return Err(format!(
                "{label} evidence for {id:?} differs from the frozen identity or is duplicated"
            ));
        }
    }
    Ok(())
}

/// Export diagnostics for the newest immutable terminal attempt.
pub fn diagnostics_for_managed_install(
    managed_root: &Path,
    output_path: &Path,
) -> Result<DiagnosticsBundle, CliError> {
    let report = report_managed_install(managed_root)?;
    let redact_roots = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .into_iter()
        .collect();
    export_diagnostics(&DiagnosticsRequest {
        managed_root: report.managed_root,
        attempt_id: report.receipt.attempt_id,
        output_path: output_path.to_path_buf(),
        redact_roots,
    })
    .map_err(|error| CliError::new("diagnostics_failed", error.to_string()))
}

impl FrozenSource {
    fn from_candidate(candidate: &GameCandidate, label: &str) -> Result<Self, CliError> {
        let fingerprint = candidate.fingerprint.clone().ok_or_else(|| {
            CliError::new(
                "source_fingerprint_unavailable",
                format!(
                    "{label} source {} could not produce a complete deterministic fingerprint",
                    candidate.root.display()
                ),
            )
        })?;
        Ok(Self {
            role: candidate.role,
            storefront: candidate.storefront,
            root: candidate.root.clone(),
            build: candidate.build.clone(),
            fingerprint,
        })
    }

    fn receipt(&self) -> SourceGameReceipt {
        SourceGameReceipt {
            role: self.role,
            storefront: self.storefront,
            version: self
                .build
                .clone()
                .unwrap_or_else(|| "unverified".to_owned()),
            fingerprint: self.fingerprint.clone(),
        }
    }
}

fn serialize_frozen_recipe(frozen: &FrozenCliRecipe) -> Result<Vec<u8>, CliError> {
    serde_json::to_vec(frozen)
        .map_err(|error| CliError::new("recipe_freeze_failed", error.to_string()))
}

fn new_campaign_seed(managed_root: &Path, recipe_sha256: &str) -> Result<String, CliError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CliError::new("campaign_identity_failed", "system clock is before epoch"))?;
    let sequence = CAMPAIGN_SEQUENCE.fetch_add(1, AtomicOrdering::Relaxed);
    Ok(sha256_bytes(
        format!(
            "{}\0{}\0{}\0{}\0{}",
            managed_root.display(),
            recipe_sha256,
            now.as_nanos(),
            std::process::id(),
            sequence
        )
        .as_bytes(),
    ))
}

fn validate_frozen_cli_recipe(
    created: &CampaignCreated,
    frozen: &FrozenCliRecipe,
) -> Result<(), CliError> {
    if frozen.schema != CLI_FROZEN_RECIPE_SCHEMA {
        return Err(CliError::new(
            "resume_unavailable",
            format!(
                "unsupported frozen CLI recipe schema {}; expected {CLI_FROZEN_RECIPE_SCHEMA}",
                frozen.schema
            ),
        ));
    }
    let actual_plan = plan_digest(&frozen.plan)
        .map_err(|error| CliError::new("resume_unavailable", error.to_string()))?;
    if actual_plan != created.plan_sha256 {
        return Err(CliError::new(
            "resume_unavailable",
            "frozen CLI plan differs from the campaign plan digest",
        ));
    }
    let actual_selection = selection_digest(&frozen.normalized_selection)
        .map_err(|error| CliError::new("resume_unavailable", error.to_string()))?;
    if actual_selection != created.selection_sha256 {
        return Err(CliError::new(
            "resume_unavailable",
            "frozen CLI selection differs from the campaign selection digest",
        ));
    }
    if frozen.bg1.fingerprint != created.source_games.bg1
        || frozen.bg2.fingerprint != created.source_games.bg2
    {
        return Err(CliError::new(
            "resume_unavailable",
            "frozen CLI source identities differ from the campaign fingerprints",
        ));
    }
    if frozen.bg1.role != GameRole::BgeeSod || frozen.bg2.role != GameRole::Bg2ee {
        return Err(CliError::new(
            "resume_unavailable",
            "frozen CLI sources do not have the required BGEE+SoD/BG2EE roles",
        ));
    }
    Ok(())
}

fn frozen_payload_identities(frozen: &FrozenCliRecipe) -> Result<Vec<FrozenIdentity>, CliError> {
    frozen
        .plan
        .runs
        .iter()
        .flat_map(|run| [run.artifact_id.as_str(), run.weidu_artifact_id.as_str()])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|id| frozen_identity(frozen, id))
        .collect()
}

fn frozen_tool_identities(frozen: &FrozenCliRecipe) -> Result<Vec<FrozenIdentity>, CliError> {
    selected_tool_ids(&frozen.plan)
        .into_iter()
        .map(|id| frozen_tool_identity(frozen, id))
        .collect()
}

fn selected_tool_ids(plan: &InstallPlan) -> BTreeSet<&str> {
    plan.runs
        .iter()
        .map(|run| run.weidu_artifact_id.as_str())
        .collect()
}

fn frozen_identity(frozen: &FrozenCliRecipe, id: &str) -> Result<FrozenIdentity, CliError> {
    let artifact = frozen.artifacts.get(id).ok_or_else(|| {
        CliError::new(
            "validation_failed",
            format!("selected plan references missing artifact {id:?}"),
        )
    })?;
    Ok(FrozenIdentity {
        id: artifact.id.clone(),
        version: artifact.version.clone(),
        sha256: artifact.source.sha256.to_ascii_lowercase(),
        length: artifact.source.expected_length.ok_or_else(|| {
            CliError::new(
                "validation_failed",
                format!("selected artifact {id:?} has no expected length"),
            )
        })?,
    })
}

fn frozen_tool_identity(frozen: &FrozenCliRecipe, id: &str) -> Result<FrozenIdentity, CliError> {
    let artifact = frozen.artifacts.get(id).ok_or_else(|| {
        CliError::new(
            "validation_failed",
            format!("selected plan references missing WeiDU artifact {id:?}"),
        )
    })?;
    let tool = artifact.tool.as_ref().ok_or_else(|| {
        CliError::new(
            "validation_failed",
            format!("selected WeiDU artifact {id:?} has no executable identity"),
        )
    })?;
    Ok(FrozenIdentity {
        id: artifact.id.clone(),
        version: tool.weidu_version.clone(),
        sha256: tool.sha256.to_ascii_lowercase(),
        length: tool.expected_length,
    })
}

fn inspect_source_for_campaign(
    profiles: &GameProfiles,
    role: GameRole,
    path: &Path,
    label: &str,
) -> Result<GameCandidate, CliError> {
    let mut candidates = [Storefront::Steam, Storefront::Gog]
        .into_iter()
        .filter(|storefront| profiles.matching(role, *storefront).next().is_some())
        .map(|storefront| inspect_game_path(profiles, &SystemFileSystem, role, storefront, path))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            CliError::new(
                "game_inspection_failed",
                format!(
                    "could not inspect {label} source {}: {error}",
                    path.display()
                ),
            )
        })?;
    candidates.sort_by(compare_candidates);
    let eligible = candidates
        .iter()
        .filter(|candidate| candidate.eligibility == Eligibility::Eligible)
        .count();
    if eligible > 1 {
        return Err(CliError::new(
            "ambiguous_storefront",
            format!(
                "{label} source {} matches more than one verified storefront profile",
                path.display()
            ),
        ));
    }
    candidates.into_iter().next().ok_or_else(|| {
        CliError::new(
            "game_inspection_failed",
            format!("recipe has no source profile for {label}"),
        )
    })
}

fn inspect_frozen_source(
    profiles: &GameProfiles,
    source: &FrozenSource,
) -> Result<GameCandidate, CliError> {
    inspect_game_path(
        profiles,
        &SystemFileSystem,
        source.role,
        source.storefront,
        &source.root,
    )
    .map_err(|error| {
        CliError::new(
            "game_inspection_failed",
            format!(
                "could not re-inspect frozen source {}: {error}",
                source.root.display()
            ),
        )
    })
}

fn application_data_root() -> Result<PathBuf, CliError> {
    let root = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
        CliError::new(
            "application_data_unavailable",
            "LOCALAPPDATA is unavailable; target locking cannot fail closed",
        )
    })?;
    let root = PathBuf::from(root);
    if !root.is_absolute() {
        return Err(CliError::new(
            "application_data_unavailable",
            format!("LOCALAPPDATA is not absolute: {}", root.display()),
        ));
    }
    Ok(root.join(APPLICATION_DATA_DIRECTORY))
}

fn canonical_existing_directory(path: &Path, label: &str) -> Result<PathBuf, CliError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        CliError::new(
            "invalid_path",
            format!("could not inspect {label} {}: {error}", path.display()),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(CliError::new(
            "invalid_path",
            format!("{label} is not a direct directory: {}", path.display()),
        ));
    }
    fs::canonicalize(path).map_err(|error| {
        CliError::new(
            "invalid_path",
            format!("could not canonicalize {label} {}: {error}", path.display()),
        )
    })
}

fn prospective_direct_path(path: &Path, label: &str) -> Result<PathBuf, CliError> {
    let absolute = std::path::absolute(path).map_err(|error| {
        CliError::new(
            "invalid_path",
            format!("could not resolve {label} {}: {error}", path.display()),
        )
    })?;
    let mut existing = absolute.as_path();
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(existing) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(CliError::new(
                        "invalid_path",
                        format!(
                            "{label} ancestor is not a direct directory: {}",
                            existing.display()
                        ),
                    ));
                }
                let mut canonical = fs::canonicalize(existing).map_err(|error| {
                    CliError::new(
                        "invalid_path",
                        format!(
                            "could not canonicalize {label} ancestor {}: {error}",
                            existing.display()
                        ),
                    )
                })?;
                for component in suffix.iter().rev() {
                    canonical.push(component);
                }
                return Ok(canonical);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let component = existing.file_name().ok_or_else(|| {
                    CliError::new(
                        "invalid_path",
                        format!(
                            "{label} has no existing directory ancestor: {}",
                            path.display()
                        ),
                    )
                })?;
                suffix.push(component.to_os_string());
                existing = existing.parent().ok_or_else(|| {
                    CliError::new(
                        "invalid_path",
                        format!(
                            "{label} has no existing directory ancestor: {}",
                            path.display()
                        ),
                    )
                })?;
            }
            Err(error) => {
                return Err(CliError::new(
                    "invalid_path",
                    format!("could not inspect {label} {}: {error}", existing.display()),
                ));
            }
        }
    }
}

fn reject_campaign_path_overlaps(
    managed_root: &Path,
    cache_root: &Path,
    bg1_root: &Path,
    bg2_root: &Path,
) -> Result<(), CliError> {
    for (label, source_root) in [("BGEE+SoD", bg1_root), ("BG2EE", bg2_root)] {
        if paths_overlap(managed_root, source_root) {
            return Err(CliError::new(
                "source_target_overlap",
                format!(
                    "managed root {} overlaps the read-only {label} source {}",
                    managed_root.display(),
                    source_root.display()
                ),
            ));
        }
        if paths_overlap(cache_root, source_root) {
            return Err(CliError::new(
                "cache_source_overlap",
                format!(
                    "cache root {} overlaps the read-only {label} source {}",
                    cache_root.display(),
                    source_root.display()
                ),
            ));
        }
    }
    if paths_overlap(managed_root, cache_root) {
        return Err(CliError::new(
            "cache_target_overlap",
            format!(
                "cache root {} overlaps managed root {}",
                cache_root.display(),
                managed_root.display()
            ),
        ));
    }
    if paths_overlap(bg1_root, bg2_root) {
        return Err(CliError::new(
            "source_overlap",
            format!(
                "BGEE+SoD source {} overlaps BG2EE source {}",
                bg1_root.display(),
                bg2_root.display()
            ),
        ));
    }
    Ok(())
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    path_is_within(left, right) || path_is_within(right, left)
}

fn path_is_within(path: &Path, base: &Path) -> bool {
    let path = path.components().collect::<Vec<_>>();
    let base = base.components().collect::<Vec<_>>();
    path.len() >= base.len()
        && path
            .iter()
            .zip(base.iter())
            .all(|(left, right)| path_component_eq(left.as_os_str(), right.as_os_str()))
}

fn path_component_eq(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> bool {
    #[cfg(windows)]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn map_orchestrator_error(
    error: crate::orchestrator::OrchestratorError,
    created: &CampaignCreated,
) -> CliError {
    let code = match &error {
        crate::orchestrator::OrchestratorError::Lock(LockError::Contended { .. }) => {
            "target_locked"
        }
        crate::orchestrator::OrchestratorError::UnsafeTarget { .. } => "unsafe_target",
        crate::orchestrator::OrchestratorError::Engine(
            crate::error::EngineError::SessionIdentityMismatch { .. }
            | crate::error::EngineError::SessionAlreadyExists { .. },
        ) => "unsafe_target",
        _ => "campaign_error",
    };
    CliError::new(
        code,
        format!(
            "managed campaign {} at {}: {error}",
            created.install_id,
            created.managed_root.display()
        ),
    )
}

struct GuardedCliDependencies<'a, S> {
    created: &'a CampaignCreated,
    frozen: &'a FrozenCliRecipe,
    current_bg1: GameCandidate,
    current_bg2: GameCandidate,
    app_data: PathBuf,
    sink: &'a S,
    controls: RunnerControlHandle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PreparedInvocationEvidence {
    run_id: String,
    attempt: u32,
    components: Vec<u32>,
    prompts: Vec<PromptReceipt>,
    prompt_identities: Vec<PromptAnswerEvidence>,
    invocation_sha256: String,
    prepared_at_millis: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProcessTerminal {
    Exited,
    Cancelled,
    SpawnFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProcessEvidence {
    started_at_millis: u64,
    completed_at_millis: u64,
    exit_code: i32,
    terminal: ProcessTerminal,
}

struct ReconciledAttemptEvidence<'a> {
    process: &'a ProcessEvidence,
    before: &'a [u8],
    after: &'a [u8],
    debug: &'a [u8],
    prompts: Vec<PromptReceipt>,
}

#[derive(Clone, Copy, Debug, Default)]
struct CliDocumentsLocator;

impl DocumentsLocator for CliDocumentsLocator {
    fn documents_dir(&self) -> std::io::Result<PathBuf> {
        #[cfg(debug_assertions)]
        if let Some(path) = std::env::var_os("CHRIZ_BG_COLLECTION_TEST_DOCUMENTS") {
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "CHRIZ_BG_COLLECTION_TEST_DOCUMENTS is not absolute",
                ));
            }
            return Ok(path);
        }
        SystemDocuments.documents_dir()
    }
}

impl<'a, S: EventSink> GuardedCliDependencies<'a, S> {
    fn new(
        created: &'a CampaignCreated,
        frozen: &'a FrozenCliRecipe,
        current_bg1: GameCandidate,
        current_bg2: GameCandidate,
        app_data: PathBuf,
        sink: &'a S,
        controls: RunnerControlHandle,
    ) -> Self {
        Self {
            created,
            frozen,
            current_bg1,
            current_bg2,
            app_data,
            sink,
            controls,
        }
    }

    fn required_inputs(&self, identities: &[FrozenIdentity], tools: bool) -> Vec<RequiredInput> {
        identities
            .iter()
            .map(|identity| RequiredInput {
                id: identity.id.clone(),
                version: identity.version.clone(),
                sha256: identity.sha256.clone(),
                length: identity.length,
                obtainable: self
                    .frozen
                    .artifacts
                    .get(&identity.id)
                    .is_some_and(|artifact| {
                        artifact.acquisition != AcquisitionPolicy::Blocked
                            && if tools {
                                artifact.tool.as_ref().is_some_and(|tool| {
                                    tool.weidu_version == identity.version
                                        && tool.expected_length == identity.length
                                        && tool.sha256.eq_ignore_ascii_case(&identity.sha256)
                                })
                            } else {
                                artifact.source.expected_length == Some(identity.length)
                                    && artifact
                                        .source
                                        .sha256
                                        .eq_ignore_ascii_case(&identity.sha256)
                            }
                    }),
            })
            .collect()
    }

    fn ensure_source_fresh(
        candidate: &GameCandidate,
        frozen: &FrozenSource,
        label: &str,
    ) -> Result<(), StepFailure> {
        let findings = candidate
            .findings
            .iter()
            .map(|finding| finding.message.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if candidate.eligibility != Eligibility::Eligible
            || candidate.fingerprint.as_deref() != Some(frozen.fingerprint.as_str())
        {
            return Err(StepFailure::new(format!(
                "source_not_fresh: {label} source {} is modified, unsupported, or differs from the frozen identity. {findings}",
                candidate.root.display()
            )));
        }
        Ok(())
    }

    fn layout(&self) -> Result<ManagedLayout, StepFailure> {
        ManagedLayout::prepare(
            &self.created.managed_root,
            &self.frozen.bg1.root,
            &self.frozen.bg2.root,
        )
        .map_err(step_error)
    }

    fn save_identity(&self, layout: &ManagedLayout) -> Result<ReservedSaveIdentity, StepFailure> {
        reserve_save_identity(
            &CliDocumentsLocator,
            layout,
            "Chriz BG Collection",
            &self.created.install_id,
        )
        .map_err(step_error)
    }

    fn artifact(&self, id: &str) -> Result<&Artifact, StepFailure> {
        self.frozen
            .artifacts
            .get(id)
            .ok_or_else(|| StepFailure::new(format!("frozen artifact {id:?} is missing")))
    }

    fn acquire_archive(
        &self,
        artifact: &Artifact,
    ) -> Result<(PathBuf, ArtifactReceipt), StepFailure> {
        let expected_length = artifact.source.expected_length.ok_or_else(|| {
            StepFailure::new(format!(
                "frozen artifact {:?} has no expected archive length",
                artifact.id
            ))
        })?;
        let (archive_path, original_url, final_url, length, sha256, cache_outcome) = match artifact
            .acquisition
        {
            AcquisitionPolicy::FetchOnly | AcquisitionPolicy::BundlePermitted => {
                let acquired = ArtifactCache::open(&self.created.cache_root)
                    .and_then(|cache| {
                        cache.acquire(
                            &DownloadRequest {
                                request_id: artifact.id.clone(),
                                url: artifact.source.url.clone(),
                                expected_length,
                                expected_sha256: artifact.source.sha256.clone(),
                                redirect_hosts: artifact.source.redirect_hosts.clone(),
                                max_attempts: 3,
                            },
                            self.sink,
                        )
                    })
                    .map_err(step_error)?;
                let outcome = match acquired.disposition {
                    CacheDisposition::Hit => ArtifactCacheOutcome::Hit,
                    CacheDisposition::Downloaded => ArtifactCacheOutcome::Downloaded,
                };
                (
                    acquired.archive_path,
                    acquired.metadata.original_url,
                    acquired.metadata.final_url,
                    acquired.metadata.length,
                    acquired.metadata.sha256,
                    outcome,
                )
            }
            AcquisitionPolicy::ManualUserSupplied => {
                let filename = artifact
                    .source
                    .expected_filename
                    .as_deref()
                    .ok_or_else(|| {
                        StepFailure::new(format!(
                            "manual artifact {:?} has no exact expected filename",
                            artifact.id
                        ))
                    })?;
                if Path::new(filename).components().count() != 1 {
                    return Err(StepFailure::new(format!(
                        "manual artifact {:?} has an unsafe expected filename",
                        artifact.id
                    )));
                }
                let drop_dir = self.created.cache_root.join("manual");
                ensure_directories(&drop_dir)?;
                let path = drop_dir.join(filename);
                if !path.is_file() {
                    self.sink.emit(EngineEvent::ManualDownloadNeeded {
                        mod_id: artifact.id.clone(),
                        page: artifact.source.url.clone(),
                        expected_sha256: artifact.source.sha256.clone(),
                        drop_dir: drop_dir.display().to_string(),
                    });
                }
                let verified =
                    provide_manual_archive(&path, &artifact.source.sha256).map_err(step_error)?;
                if verified.length != expected_length {
                    return Err(StepFailure::new(format!(
                        "manual artifact {:?} length mismatch: expected {expected_length}, got {}",
                        artifact.id, verified.length
                    )));
                }
                (
                    verified.path,
                    artifact.source.url.clone(),
                    artifact.source.url.clone(),
                    verified.length,
                    verified.sha256,
                    ArtifactCacheOutcome::Manual,
                )
            }
            AcquisitionPolicy::Blocked => {
                return Err(StepFailure::new(format!(
                    "frozen artifact {:?} is blocked and cannot be acquired",
                    artifact.id
                )))
            }
        };
        let observed_receipt = ArtifactReceipt {
            id: artifact.id.clone(),
            version: artifact.version.clone(),
            original_url,
            final_url,
            length,
            sha256,
            cache_outcome,
        };
        let receipt = match self.load_named_evidence("artifacts", &artifact.id)? {
            Some(first) => retain_first_acquisition_receipt(&first, &observed_receipt)?,
            None => {
                self.persist_named_evidence("artifacts", &artifact.id, &observed_receipt)?;
                observed_receipt
            }
        };
        Ok((archive_path, receipt))
    }

    fn extract(&self, artifact: &Artifact) -> Result<ExtractedArtifact, StepFailure> {
        let (archive, _) = self.acquire_archive(artifact)?;
        let extracted = extract_archive(
            &archive,
            &self.created.cache_root.join("extracted"),
            &ArchiveRequirements {
                artifact_sha256: artifact.source.sha256.clone(),
                format: match artifact.archive.kind {
                    ManifestArchiveKind::Zip => ArchiveFormat::Zip,
                    ManifestArchiveKind::Iemod => ArchiveFormat::Iemod,
                },
                expected_roots: artifact.archive.publish_roots.clone(),
                expected_tp2_paths: artifact.archive.tp2_paths.clone(),
                limits: ArchiveLimits {
                    max_depth: artifact.archive.limits.max_depth,
                    max_entries: artifact.archive.limits.max_entries,
                    max_entry_uncompressed_bytes: artifact
                        .archive
                        .limits
                        .max_entry_uncompressed_bytes,
                    max_total_uncompressed_bytes: artifact
                        .archive
                        .limits
                        .max_total_uncompressed_bytes,
                    max_compression_ratio: artifact.archive.limits.max_compression_ratio,
                },
                mode: ArchiveMode::Public,
            },
        )
        .map_err(step_error)?;
        let wrapper_ok = match artifact.archive.root_rule {
            ArchiveRootRule::Direct => extracted.wrapper_directory.is_none(),
            ArchiveRootRule::SingleWrapper => extracted.wrapper_directory.is_some(),
            ArchiveRootRule::DirectOrSingleWrapper => true,
        };
        if !wrapper_ok {
            return Err(StepFailure::new(format!(
                "artifact {:?} archive root differs from its frozen root rule",
                artifact.id
            )));
        }
        Ok(extracted)
    }

    fn verified_tool(&self, id: &str) -> Result<VerifiedWeidu, StepFailure> {
        let artifact = self.artifact(id)?;
        let contract = artifact.tool.as_ref().ok_or_else(|| {
            StepFailure::new(format!("artifact {id:?} has no WeiDU executable contract"))
        })?;
        let extracted = self.extract(artifact)?;
        let executable = extracted.root.join(&contract.executable);
        let (verified, observed) =
            verify_tool_contract(&executable, contract).map_err(step_error)?;
        let receipt = WeiDuToolReceipt {
            id: artifact.id.clone(),
            version: observed.weidu_version,
            length: observed.length,
            sha256: observed.sha256,
        };
        self.persist_named_evidence("tools", &artifact.id, &receipt)?;
        Ok(verified)
    }

    fn target_root(&self, target: GameRoot) -> &Path {
        match target {
            GameRoot::Bg1 => &self.created.staged_bg1,
            GameRoot::Bg2 => &self.created.staged_bg2,
        }
    }

    fn evidence_root(&self) -> PathBuf {
        self.created
            .managed_root
            .join(STATE_DIRECTORY)
            .join(EVIDENCE_DIRECTORY)
    }

    fn named_evidence_path(&self, category: &str, id: &str) -> Result<PathBuf, StepFailure> {
        validate_cli_identifier(id, "evidence id").map_err(StepFailure::new)?;
        let directory = self.evidence_root().join(category);
        ensure_directories(&directory)?;
        Ok(directory.join(format!("{id}.json")))
    }

    fn persist_named_evidence<T: Serialize>(
        &self,
        category: &str,
        id: &str,
        value: &T,
    ) -> Result<(), StepFailure> {
        write_json_once(&self.named_evidence_path(category, id)?, value)
    }

    fn load_named_evidence<T: for<'de> Deserialize<'de>>(
        &self,
        category: &str,
        id: &str,
    ) -> Result<Option<T>, StepFailure> {
        read_optional_json(&self.named_evidence_path(category, id)?)
    }

    fn run_attempt_path(&self, run_id: &str, attempt: u32) -> Result<PathBuf, StepFailure> {
        validate_cli_identifier(run_id, "run id").map_err(StepFailure::new)?;
        if attempt == 0 {
            return Err(StepFailure::new("run attempt number must be positive"));
        }
        let directory = self.evidence_root().join("runs").join(run_id);
        ensure_directories(&directory)?;
        Ok(directory.join(format!("attempt-{attempt:04}.json")))
    }

    fn persist_run_attempt(
        &self,
        run_id: &str,
        receipt: &RunAttemptReceipt,
    ) -> Result<(), StepFailure> {
        write_json_once(&self.run_attempt_path(run_id, receipt.attempt)?, receipt)
    }

    fn load_run_receipt(&self, run: &PlannedRun) -> Result<RunReceipt, StepFailure> {
        validate_cli_identifier(&run.run_id, "run id").map_err(StepFailure::new)?;
        let directory = self.evidence_root().join("runs").join(&run.run_id);
        let metadata = match fs::symlink_metadata(&directory) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(RunReceipt {
                    run_id: run.run_id.clone(),
                    target: run.target,
                    components: run.components.clone(),
                    attempts: Vec::new(),
                })
            }
            Err(error) => return Err(step_error(error)),
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(StepFailure::new(format!(
                "run evidence path is not a direct directory: {}",
                directory.display()
            )));
        }
        let mut paths = fs::read_dir(&directory)
            .map_err(step_error)?
            .map(|entry| entry.map(|entry| entry.path()).map_err(step_error))
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort();
        let mut attempts = Vec::new();
        for path in paths {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                return Err(StepFailure::new(format!(
                    "run evidence filename is not Unicode: {}",
                    path.display()
                )));
            };
            if !name.starts_with("attempt-") || !name.ends_with(".json") {
                return Err(StepFailure::new(format!(
                    "unexpected run evidence file: {}",
                    path.display()
                )));
            }
            attempts.push(read_required_json(&path)?);
        }
        Ok(RunReceipt {
            run_id: run.run_id.clone(),
            target: run.target,
            components: run.components.clone(),
            attempts,
        })
    }
}

impl<S: EventSink> CampaignPreflight for GuardedCliDependencies<'_, S> {
    fn initial(
        &mut self,
        request: &CampaignRequest,
        replay: &SessionReplay,
    ) -> Result<(), StepFailure> {
        Self::ensure_source_fresh(&self.current_bg1, &self.frozen.bg1, "BGEE+SoD")?;
        Self::ensure_source_fresh(&self.current_bg2, &self.frozen.bg2, "BG2EE")?;
        let required_artifacts = self.required_inputs(&request.created.artifact_identities, false);
        let required_tools = self.required_inputs(&request.created.tool_identities, true);
        let staged_copies = directory_bytes(&self.frozen.bg1.root)?
            .checked_add(directory_bytes(&self.frozen.bg2.root)?)
            .ok_or_else(|| StepFailure::new("source game byte count overflowed u64"))?;
        let downloads = request
            .created
            .artifact_identities
            .iter()
            .try_fold(0_u64, |sum, identity| sum.checked_add(identity.length))
            .ok_or_else(|| StepFailure::new("artifact byte count overflowed u64"))?;
        let extraction = request
            .created
            .artifact_identities
            .iter()
            .try_fold(0_u64, |sum, identity| {
                self.frozen
                    .artifacts
                    .get(&identity.id)
                    .and_then(|artifact| {
                        sum.checked_add(artifact.archive.limits.max_total_uncompressed_bytes)
                    })
            })
            .ok_or_else(|| StepFailure::new("archive extraction byte count overflowed u64"))?;
        let subtotal = staged_copies
            .checked_add(downloads)
            .and_then(|sum| sum.checked_add(extraction))
            .ok_or_else(|| StepFailure::new("preflight byte count overflowed u64"))?;
        initial_preflight(&InitialPreflight {
            bg1_source: &self.current_bg1,
            bg2_source: &self.current_bg2,
            destination: &request.created.managed_root,
            space: SpaceRequirement {
                staged_copies,
                downloads,
                extraction,
                safety_margin: subtotal / 10,
            },
            frozen_campaign: replay,
            current_plan: &request.plan,
            expected_review_token: "cli-frozen-review",
            presented_review_token: "cli-frozen-review",
            required_artifacts: &required_artifacts,
            required_tools: &required_tools,
        })
        .map(|_| ())
        .map_err(|error| StepFailure::new(error.to_string()))
    }

    fn recheck_before_mutation(&mut self, check: &MutationCheck) -> Result<(), StepFailure> {
        if matches!(check.kind, crate::orchestrator::MutationKind::Stage) {
            let bg1 = inspect_frozen_source(&self.frozen.profiles, &self.frozen.bg1)
                .map_err(step_error)?;
            let bg2 = inspect_frozen_source(&self.frozen.profiles, &self.frozen.bg2)
                .map_err(step_error)?;
            Self::ensure_source_fresh(&bg1, &self.frozen.bg1, "BGEE+SoD")?;
            Self::ensure_source_fresh(&bg2, &self.frozen.bg2, "BG2EE")?;
            recheck_staging_target_before_mutation(self.target_root(check.target), "en_US")
                .map_err(step_error)?;
            return Ok(());
        }
        recheck_target_before_mutation(self.target_root(check.target), "en_US").map_err(step_error)
    }
}

impl<S: EventSink> ArtifactAcquirer for GuardedCliDependencies<'_, S> {
    fn acquire(
        &mut self,
        identity: &FrozenIdentity,
        kind: ArtifactKind,
    ) -> Result<(), StepFailure> {
        let artifact = self.artifact(&identity.id)?;
        if artifact.source.expected_length != Some(identity.length)
            || !artifact
                .source
                .sha256
                .eq_ignore_ascii_case(&identity.sha256)
        {
            return Err(StepFailure::new(format!(
                "frozen archive identity for {:?} differs from its executable recipe",
                identity.id
            )));
        }
        if matches!(kind, ArtifactKind::Tool) && artifact.tool.is_none() {
            return Err(StepFailure::new(format!(
                "frozen tool archive {:?} has no executable contract",
                identity.id
            )));
        }
        self.acquire_archive(artifact).map(|_| ())
    }
}

impl<S: EventSink> StagingService for GuardedCliDependencies<'_, S> {
    fn stage(&mut self, role: GameRole) -> Result<(), StepFailure> {
        let layout = self.layout()?;
        match role {
            GameRole::BgeeSod => {
                let identity = self.save_identity(&layout)?;
                stage_bgee_sod(&layout, &identity)
                    .map(|_| ())
                    .map_err(step_error)
            }
            GameRole::Bg2ee => stage_game_copy(&layout, GameRole::Bg2ee)
                .map(|_| ())
                .map_err(step_error),
        }
    }

    fn finalize_identity(&mut self) -> Result<(), StepFailure> {
        let layout = self.layout()?;
        let identity = self.save_identity(&layout)?;
        finalize_game_identity(&layout, &identity).map_err(step_error)
    }
}

impl<S: EventSink> ArtifactMaterializer for GuardedCliDependencies<'_, S> {
    fn materialize(
        &mut self,
        task: &MaterializationTask,
    ) -> Result<MaterializationOutcome, StepFailure> {
        let artifact = self.artifact(&task.artifact_id)?;
        let extracted = self.extract(artifact)?;
        let result = materialize(
            &extracted,
            self.target_root(task.target),
            &MaterializationRequest {
                materialization_id: format!(
                    "materialize-{}",
                    &sha256_bytes(task.step_id().as_bytes())[..16]
                ),
                owner: artifact.id.clone(),
                roots: artifact.archive.publish_roots.clone(),
                tp2_paths: artifact.archive.tp2_paths.clone(),
                collision_rules: Vec::new(),
            },
        );
        match result {
            Ok(_) => Ok(MaterializationOutcome::Complete),
            Err(
                error @ (crate::acquire::AcquireError::UndeclaredOverwrite { .. }
                | crate::acquire::AcquireError::CollisionRuleRejected { .. }
                | crate::acquire::AcquireError::PublicationMismatch { .. }),
            ) => Ok(MaterializationOutcome::FreshCopyRequired {
                reason: error.to_string(),
            }),
            Err(error) => Err(step_error(error)),
        }
    }
}

impl<S: EventSink> InvocationBuilder for GuardedCliDependencies<'_, S> {
    type Invocation = Invocation;

    fn build(
        &mut self,
        run: &PlannedRun,
        components: &[u32],
        attempt: &StepAttempt,
    ) -> Result<BuiltInvocation<Self::Invocation>, StepFailure> {
        ensure_directories(&attempt.evidence_root)?;
        let mod_file = self.frozen.mods.get(&run.mod_id).ok_or_else(|| {
            StepFailure::new(format!("frozen installer {:?} is missing", run.mod_id))
        })?;
        if mod_file.artifact_id != run.artifact_id
            || mod_file.weidu_artifact_id != run.weidu_artifact_id
        {
            return Err(StepFailure::new(format!(
                "frozen installer metadata for {:?} differs from its plan",
                run.run_id
            )));
        }
        // The frozen recipe currently carries the invocation mode but no signed evidence that a
        // particular explicit relative TP2 path was compatibility-tested. Keep production
        // execution limited to setup-name mode until that schema contract is separately reviewed.
        if mod_file.invocation_mode == InvocationMode::ExplicitTp2 {
            return Err(StepFailure::new(
                "explicit-relative-TP2 invocation is unavailable until the signed recipe schema records separately reviewed compatibility evidence; use setup-name mode",
            ));
        }
        let authored_prompts = run
            .prompt_scripts
            .iter()
            .filter(|script| components.contains(&script.component.component))
            .flat_map(|script| script.steps.iter())
            .collect::<Vec<_>>();
        let prompts = authored_prompts
            .iter()
            .map(|prompt| ResolvedPrompt {
                expected_output: prompt.expected_output.as_bytes().to_vec(),
                answer: prompt.answer.as_bytes().to_vec(),
            })
            .collect();
        let prompt_receipts = authored_prompts
            .iter()
            .map(|prompt| PromptReceipt {
                expected_output: prompt.expected_output.clone(),
                answer: sanitize_prompt_answer(
                    &prompt.answer,
                    &self.created.staged_bg1,
                    &self.created.staged_bg2,
                ),
                matched: false,
            })
            .collect();
        let prompt_identities = authored_prompts
            .iter()
            .enumerate()
            .map(|(index, prompt)| PromptAnswerEvidence {
                index,
                expected_output_sha256: sha256_bytes(prompt.expected_output.as_bytes()),
                answer_sha256: sha256_bytes(prompt.answer.as_bytes()),
            })
            .collect();
        let tool = self.verified_tool(&run.weidu_artifact_id)?;
        let invocation = build_invocation(
            &InvocationInput {
                target: run.target,
                tp2: mod_file.tp2.clone(),
                mode: mod_file.invocation_mode,
                explicit_tp2_tested: false,
                language: mod_file.language,
                remaining_components: components.to_vec(),
                run_args: run.args.clone(),
                prompts,
            },
            &StagedRoots {
                bg1: self.created.staged_bg1.clone(),
                bg2: self.created.staged_bg2.clone(),
            },
            &tool,
            &attempt.evidence_root,
        )
        .map_err(step_error)?;
        write_json_once(
            &attempt.evidence_root.join(INVOCATION_FILE),
            &PreparedInvocationEvidence {
                run_id: run.run_id.clone(),
                attempt: attempt.attempt,
                components: components.to_vec(),
                prompts: prompt_receipts,
                prompt_identities,
                invocation_sha256: invocation.identity_digest.clone(),
                prepared_at_millis: unix_millis()?,
            },
        )?;
        Ok(BuiltInvocation {
            identity_digest: invocation.identity_digest.clone(),
            invocation,
        })
    }
}

impl<S: EventSink + Sync> ProcessRunner<Invocation> for GuardedCliDependencies<'_, S> {
    fn run(
        &mut self,
        invocation: Invocation,
        attempt: &StepAttempt,
    ) -> Result<ProcessResult, StepFailure> {
        ensure_directories(&attempt.evidence_root)?;
        let debug_log = invocation.debug_path.clone();
        let expected_debug_log = attempt.evidence_root.join(DEBUG_LOG_FILE);
        if debug_log != expected_debug_log {
            return Err(StepFailure::new(format!(
                "invocation debug path {} differs from its attempt evidence path {}",
                debug_log.display(),
                expected_debug_log.display()
            )));
        }
        let output_log = attempt.evidence_root.join(PROCESS_OUTPUT_FILE);
        let started_at_millis = unix_millis()?;
        let result = run_controlled(
            RunnerRequest {
                invocation,
                step_id: attempt.step_id.clone(),
                output_log: output_log.clone(),
                silence_threshold: Duration::from_secs(30),
            },
            &self.controls,
            BorrowedSink(self.sink),
        );
        let completed_at_millis = unix_millis()?;
        for path in [
            output_log,
            attempt.evidence_root.join(STDOUT_FILE),
            attempt.evidence_root.join(STDERR_FILE),
            attempt.evidence_root.join(PROMPT_RESULTS_FILE_NAME),
            debug_log,
        ] {
            ensure_empty_file(&path)?;
            sync_direct_file(&path)?;
        }
        #[cfg(debug_assertions)]
        if std::env::var_os("CHRIZ_BG_COLLECTION_TEST_INTERRUPT_AFTER_WEIDU").is_some() {
            return Err(StepFailure::new(
                "simulated interruption after WeiDU exit and durable log synchronization",
            ));
        }
        let (exit_code, terminal) = match result {
            RunOutcome::Exited { code } => (code, ProcessTerminal::Exited),
            RunOutcome::Cancelled => (-1, ProcessTerminal::Cancelled),
            RunOutcome::SpawnFailed => (-1, ProcessTerminal::SpawnFailed),
        };
        write_json_once(
            &attempt.evidence_root.join(PROCESS_RESULT_FILE),
            &ProcessEvidence {
                started_at_millis,
                completed_at_millis,
                exit_code,
                terminal,
            },
        )?;
        match terminal {
            ProcessTerminal::Exited => Ok(ProcessResult { exit_code }),
            ProcessTerminal::Cancelled => Err(StepFailure::new(
                "installation cancelled by user after the WeiDU process tree terminated",
            )),
            ProcessTerminal::SpawnFailed => Err(StepFailure::new(
                "WeiDU could not be supervised; inspect the durable attempt diagnostics",
            )),
        }
    }
}

impl<S: EventSink> GuardedCliDependencies<'_, S> {
    fn reconcile_install_attempt(
        &self,
        run: &PlannedRun,
        attempt: &StepAttempt,
        observed_result: Option<ProcessResult>,
    ) -> Result<InstallReconciliation, StepFailure> {
        let prepared: PreparedInvocationEvidence =
            read_required_json(&attempt.evidence_root.join(INVOCATION_FILE))?;
        if prepared.run_id != run.run_id
            || prepared.attempt != attempt.attempt
            || prepared.components.is_empty()
            || !run.components.ends_with(&prepared.components)
        {
            return Err(StepFailure::new(format!(
                "invocation evidence for {:?} differs from the frozen run",
                run.run_id
            )));
        }
        let before = read_direct_file(&attempt.evidence_root.join(BEFORE_LOG_FILE))?;
        let after = read_optional_weidu_log(self.target_root(run.target))?;
        write_bytes_once(&attempt.evidence_root.join(AFTER_LOG_FILE), &after)?;
        let debug = read_direct_file(&attempt.evidence_root.join(DEBUG_LOG_FILE))?;
        let prompts = self.verified_prompt_receipts(attempt, &prepared)?;
        let process: Option<ProcessEvidence> =
            read_optional_json(&attempt.evidence_root.join(PROCESS_RESULT_FILE))?;
        if let (Some(observed), Some(process)) = (observed_result, process.as_ref()) {
            if process.terminal != ProcessTerminal::Exited
                || process.exit_code != observed.exit_code
            {
                return Err(StepFailure::new(
                    "runner result differs from its synchronized process evidence",
                ));
            }
        }

        let statuses = parse_terminal_statuses(&String::from_utf8_lossy(&debug));
        let proven_no_process_change = before == after
            && statuses.is_empty()
            && process
                .as_ref()
                .is_some_and(|process| process.terminal != ProcessTerminal::Exited);
        let reconciliation =
            if prompts.iter().any(|prompt| !prompt.matched) && !proven_no_process_change {
                InstallReconciliation::FreshCopyRequired {
                    reason: "WeiDU did not durably receive every authored prompt answer".to_owned(),
                }
            } else if proven_no_process_change {
                InstallReconciliation::Retry {
                    remaining: prepared.components.clone(),
                }
            } else {
                let exit_code = process
                    .as_ref()
                    .map(|evidence| evidence.exit_code)
                    .or_else(|| observed_result.map(|result| result.exit_code))
                    .or_else(|| inferred_success_exit_code(&statuses))
                    .unwrap_or(-1);
                match reconcile_weidu(
                    &String::from_utf8_lossy(&before),
                    &String::from_utf8_lossy(&after),
                    &String::from_utf8_lossy(&debug),
                    &ExpectedRun {
                        tp2: self
                            .frozen
                            .mods
                            .get(&run.mod_id)
                            .ok_or_else(|| {
                                StepFailure::new(format!(
                                    "frozen installer {:?} is missing",
                                    run.mod_id
                                ))
                            })?
                            .tp2
                            .clone(),
                        language: self.frozen.mods[&run.mod_id].language,
                        components: prepared.components.clone(),
                        exit_code,
                    },
                ) {
                    Reconciliation::ProvenDone => InstallReconciliation::ProvenDone,
                    Reconciliation::PartialPrefix { remaining } => {
                        InstallReconciliation::PartialPrefix { remaining }
                    }
                    Reconciliation::UnchangedRetryable => InstallReconciliation::Retry {
                        remaining: prepared.components.clone(),
                    },
                    Reconciliation::StackDisturbed => InstallReconciliation::FreshCopyRequired {
                        reason: "WeiDU.log changed outside the exact frozen component suffix"
                            .to_owned(),
                    },
                    Reconciliation::Ambiguous => InstallReconciliation::FreshCopyRequired {
                        reason: "WeiDU debug/log/exit evidence is incomplete or contradictory"
                            .to_owned(),
                    },
                }
            };

        if !matches!(
            reconciliation,
            InstallReconciliation::FreshCopyRequired { .. }
        ) {
            let recovered_process = if process.is_none() && !statuses.is_empty() {
                Some(ProcessEvidence {
                    started_at_millis: prepared.prepared_at_millis,
                    completed_at_millis: unix_millis()?,
                    exit_code: observed_result.map_or(-1, |result| result.exit_code),
                    // This in-memory value supplies conservative timing/unknown-exit receipt
                    // evidence only; it is never persisted as process-result.json.
                    terminal: ProcessTerminal::SpawnFailed,
                })
            } else {
                None
            };
            if let Some(process) = process.as_ref().or(recovered_process.as_ref()) {
                let receipt = self.build_attempt_receipt(
                    attempt,
                    &prepared,
                    ReconciledAttemptEvidence {
                        process,
                        before: &before,
                        after: &after,
                        debug: &debug,
                        prompts,
                    },
                )?;
                self.persist_run_attempt(&run.run_id, &receipt)?;
            }
        }
        Ok(reconciliation)
    }

    fn build_attempt_receipt(
        &self,
        attempt: &StepAttempt,
        prepared: &PreparedInvocationEvidence,
        evidence: ReconciledAttemptEvidence<'_>,
    ) -> Result<RunAttemptReceipt, StepFailure> {
        let ReconciledAttemptEvidence {
            process,
            before,
            after,
            debug,
            prompts,
        } = evidence;
        let before_entries =
            parse_active_entries(&String::from_utf8_lossy(before)).map_err(step_error)?;
        let after_entries =
            parse_active_entries(&String::from_utf8_lossy(after)).map_err(step_error)?;
        if after_entries.len() < before_entries.len()
            || !before_entries
                .iter()
                .zip(&after_entries)
                .all(|(left, right)| same_log_entry(left, right))
        {
            return Err(StepFailure::new(
                "cannot create run evidence for a disturbed WeiDU.log",
            ));
        }
        let stdout = read_direct_file(&attempt.evidence_root.join(STDOUT_FILE))?;
        let stderr = read_direct_file(&attempt.evidence_root.join(STDERR_FILE))?;
        let statuses = parse_terminal_statuses(&String::from_utf8_lossy(debug));
        let warnings = statuses
            .iter()
            .enumerate()
            .filter(|(_, status)| **status == DebugStatus::InstalledWithWarnings)
            .map(|(index, _)| {
                format!(
                    "component {} installed with warnings",
                    prepared.components[index]
                )
            })
            .collect();
        Ok(RunAttemptReceipt {
            attempt: prepared.attempt,
            components: prepared.components.clone(),
            prompts,
            timing: RunTiming {
                started_at_millis: process.started_at_millis,
                completed_at_millis: process.completed_at_millis,
            },
            exit_code: process.exit_code,
            warnings,
            invocation_sha256: prepared.invocation_sha256.clone(),
            stdout_sha256: sha256_bytes(&stdout),
            stderr_sha256: sha256_bytes(&stderr),
            debug_sha256: sha256_bytes(debug),
            log_diff: LogDiffReceipt {
                before_sha256: sha256_bytes(before),
                after_sha256: sha256_bytes(after),
                added: after_entries[before_entries.len()..]
                    .iter()
                    .map(log_component_receipt)
                    .collect(),
                removed: Vec::new(),
            },
        })
    }

    fn verified_prompt_receipts(
        &self,
        attempt: &StepAttempt,
        prepared: &PreparedInvocationEvidence,
    ) -> Result<Vec<PromptReceipt>, StepFailure> {
        if prepared.prompts.len() != prepared.prompt_identities.len()
            || prepared
                .prompt_identities
                .iter()
                .enumerate()
                .any(|(index, identity)| identity.index != index)
        {
            return Err(StepFailure::new(
                "prepared prompt identities differ from the authored prompt sequence",
            ));
        }
        let bytes = read_direct_file(&attempt.evidence_root.join(PROMPT_RESULTS_FILE_NAME))?;
        let answered = parse_prompt_results(&bytes).map_err(step_error)?;
        if answered.len() > prepared.prompt_identities.len()
            || answered.iter().enumerate().any(|(index, found)| {
                found.index != index || found != &prepared.prompt_identities[index]
            })
        {
            return Err(StepFailure::new(
                "runner prompt evidence differs from the prepared authored sequence",
            ));
        }
        Ok(prepared
            .prompts
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, mut prompt)| {
                prompt.matched = index < answered.len();
                prompt
            })
            .collect())
    }
}

struct BorrowedSink<'a, S>(&'a S);

impl<S: EventSink + Sync> EventSink for BorrowedSink<'_, S> {
    fn emit(&self, event: EngineEvent) {
        self.0.emit(event);
    }
}

impl<S: EventSink> InstallLogVerifier for GuardedCliDependencies<'_, S> {
    fn recover(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
    ) -> Result<InstallReconciliation, StepFailure> {
        self.reconcile_install_attempt(run, attempt, None)
    }

    fn snapshot_before(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
    ) -> Result<(), StepFailure> {
        ensure_directories(&attempt.evidence_root)?;
        let before = read_optional_weidu_log(self.target_root(run.target))?;
        write_bytes_once(&attempt.evidence_root.join(BEFORE_LOG_FILE), &before)
    }

    fn record_invocation(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
        identity_digest: &str,
    ) -> Result<(), StepFailure> {
        let evidence: PreparedInvocationEvidence =
            read_required_json(&attempt.evidence_root.join(INVOCATION_FILE))?;
        if evidence.run_id != run.run_id
            || evidence.attempt != attempt.attempt
            || evidence.invocation_sha256 != identity_digest
        {
            return Err(StepFailure::new(
                "prepared invocation differs from its durable intent evidence",
            ));
        }
        sync_direct_file(&attempt.evidence_root.join(INVOCATION_FILE))
    }

    fn sync_and_reconcile(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
        result: ProcessResult,
    ) -> Result<InstallReconciliation, StepFailure> {
        for name in [
            PROCESS_OUTPUT_FILE,
            STDOUT_FILE,
            STDERR_FILE,
            PROMPT_RESULTS_FILE_NAME,
            DEBUG_LOG_FILE,
        ] {
            sync_direct_file(&attempt.evidence_root.join(name))?;
        }
        self.reconcile_install_attempt(run, attempt, Some(result))
    }

    fn verify_final(&mut self, plan: &InstallPlan) -> Result<(), StepFailure> {
        let layout = self.layout()?;
        let identity = self.save_identity(&layout)?;
        let bg1_engine_name = read_engine_name(layout.bg1_root()).map_err(step_error)?;
        let bg2_engine_name = read_engine_name(layout.game_root()).map_err(step_error)?;
        if bg1_engine_name != identity.engine_name || bg2_engine_name != identity.engine_name {
            return Err(StepFailure::new(
                "staged game identities differ from the reserved managed save identity",
            ));
        }
        let mut logs = Vec::new();
        for target in [GameRoot::Bg1, GameRoot::Bg2] {
            let bytes = read_optional_weidu_log(self.target_root(target))?;
            let entries =
                parse_active_entries(&String::from_utf8_lossy(&bytes)).map_err(step_error)?;
            let expected = expected_log_components(plan, &self.frozen.mods, target)?;
            if entries.len() != expected.len()
                || !entries.iter().zip(&expected).all(|(actual, expected)| {
                    actual.tp2_key == expected.tp2.replace('\\', "/").to_ascii_lowercase()
                        && actual.language == expected.language
                        && actual.component == expected.component
                })
            {
                return Err(StepFailure::new(format!(
                    "final {:?} WeiDU.log does not exactly match the frozen plan",
                    target
                )));
            }
            logs.push(FinalLogReceipt {
                target,
                sha256: sha256_bytes(&bytes),
                components: entries.iter().map(log_component_receipt).collect(),
            });
        }
        let launch_path = verified_launch_path(plan, layout.game_root())?;
        let final_state = FinalReceiptState {
            logs,
            bg1_engine_name,
            bg2_engine_name,
            managed_save_root: identity.save_root,
            launch_path,
            verification_summary: format!(
                "exact frozen stack matched across {} planned WeiDU runs",
                plan.runs.len()
            ),
        };
        self.persist_named_evidence("final", "state", &final_state)
    }
}

impl<S: EventSink> ReceiptWriter for GuardedCliDependencies<'_, S> {
    fn write(&mut self, draft: &ReceiptDraft) -> Result<(), StepFailure> {
        let store = ReceiptStore::open(&self.created.managed_root, &self.created.install_id)
            .map_err(|error| StepFailure::new(error.to_string()))?;
        let registry = ManagedInstallRegistry::open_or_create(&self.app_data)
            .map_err(|error| StepFailure::new(error.to_string()))?;
        let artifacts = self
            .created
            .artifact_identities
            .iter()
            .map(|identity| self.load_named_evidence("artifacts", &identity.id))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        let weidu_tools = self
            .created
            .tool_identities
            .iter()
            .map(|identity| self.load_named_evidence("tools", &identity.id))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        let runs = draft
            .plan
            .runs
            .iter()
            .map(|run| self.load_run_receipt(run))
            .collect::<Result<Vec<_>, _>>()?;
        let final_state = self.load_named_evidence("final", "state")?;
        let evidence = ReceiptEvidence {
            versions: ReceiptVersions {
                application: env!("CARGO_PKG_VERSION").to_owned(),
                engine: env!("CARGO_PKG_VERSION").to_owned(),
                manifest_schema: self.frozen.collection.schema,
                recipe: format!("local-{}", &self.created.recipe_payload_sha256[..12]),
            },
            source_games: vec![self.frozen.bg1.receipt(), self.frozen.bg2.receipt()],
            artifacts,
            weidu_tools,
            runs,
            final_state,
        };
        ManagedReceiptWriter::new(store, registry, "Chriz BG Collection".to_owned(), evidence)
            .write(draft)
    }
}

impl<S: EventSink> CampaignClock for GuardedCliDependencies<'_, S> {
    fn now_millis(&mut self) -> Result<u64, StepFailure> {
        unix_millis()
    }
}

fn step_error(error: impl std::fmt::Display) -> StepFailure {
    StepFailure::new(error.to_string())
}

fn unix_millis() -> Result<u64, StepFailure> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StepFailure::new("system clock is earlier than the Unix epoch"))?
        .as_millis();
    u64::try_from(millis).map_err(|_| StepFailure::new("system timestamp does not fit u64"))
}

fn ensure_directories(path: &Path) -> Result<(), StepFailure> {
    let absolute = std::path::absolute(path).map_err(step_error)?;
    let mut missing = Vec::new();
    let mut cursor = absolute.as_path();
    loop {
        match fs::symlink_metadata(cursor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink()
                    || metadata_is_reparse(&metadata)
                    || !metadata.is_dir()
                {
                    return Err(StepFailure::new(format!(
                        "path ancestor is not a direct directory: {}",
                        cursor.display()
                    )));
                }
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(cursor.to_path_buf());
                cursor = cursor.parent().ok_or_else(|| {
                    StepFailure::new(format!(
                        "path has no existing ancestor: {}",
                        absolute.display()
                    ))
                })?;
            }
            Err(error) => return Err(step_error(error)),
        }
    }
    for directory in missing.into_iter().rev() {
        match fs::create_dir(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(step_error(error)),
        }
        let metadata = fs::symlink_metadata(&directory).map_err(step_error)?;
        if metadata.file_type().is_symlink() || metadata_is_reparse(&metadata) || !metadata.is_dir()
        {
            return Err(StepFailure::new(format!(
                "created path is not a direct directory: {}",
                directory.display()
            )));
        }
    }
    Ok(())
}

fn write_json_once<T: Serialize>(path: &Path, value: &T) -> Result<(), StepFailure> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(step_error)?;
    bytes.push(b'\n');
    write_bytes_once(path, &bytes)
}

fn write_bytes_once(path: &Path, bytes: &[u8]) -> Result<(), StepFailure> {
    let parent = path
        .parent()
        .ok_or_else(|| StepFailure::new(format!("path has no parent: {}", path.display())))?;
    ensure_directories(parent)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink()
                || metadata_is_reparse(&metadata)
                || !metadata.is_file()
            {
                return Err(StepFailure::new(format!(
                    "evidence path is not a direct file: {}",
                    path.display()
                )));
            }
            let existing = fs::read(path).map_err(step_error)?;
            if existing != bytes {
                return Err(StepFailure::new(format!(
                    "create-once evidence differs at {}",
                    path.display()
                )));
            }
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(step_error(error)),
    }
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(step_error)?;
    file.write_all(bytes).map_err(step_error)?;
    file.sync_all().map_err(step_error)
}

fn ensure_empty_file(path: &Path) -> Result<(), StepFailure> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if !metadata.file_type().is_symlink()
                && !metadata_is_reparse(&metadata)
                && metadata.is_file() =>
        {
            Ok(())
        }
        Ok(_) => Err(StepFailure::new(format!(
            "process evidence is not a direct file: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => write_bytes_once(path, b""),
        Err(error) => Err(step_error(error)),
    }
}

fn read_required_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, StepFailure> {
    serde_json::from_slice(&read_direct_file(path)?).map_err(step_error)
}

fn read_optional_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, StepFailure> {
    match fs::symlink_metadata(path) {
        Ok(_) => read_required_json(path).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(step_error(error)),
    }
}

fn read_direct_file(path: &Path) -> Result<Vec<u8>, StepFailure> {
    let metadata = fs::symlink_metadata(path).map_err(step_error)?;
    if metadata.file_type().is_symlink() || metadata_is_reparse(&metadata) || !metadata.is_file() {
        return Err(StepFailure::new(format!(
            "evidence is not a direct regular file: {}",
            path.display()
        )));
    }
    fs::read(path).map_err(step_error)
}

fn sync_direct_file(path: &Path) -> Result<(), StepFailure> {
    let metadata = fs::symlink_metadata(path).map_err(step_error)?;
    if metadata.file_type().is_symlink() || metadata_is_reparse(&metadata) || !metadata.is_file() {
        return Err(StepFailure::new(format!(
            "cannot sync non-direct evidence file: {}",
            path.display()
        )));
    }
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(step_error)
}

fn read_optional_weidu_log(root: &Path) -> Result<Vec<u8>, StepFailure> {
    let path = root.join("WeiDU.log");
    match fs::symlink_metadata(&path) {
        Ok(_) => read_direct_file(&path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(step_error(error)),
    }
}

fn directory_bytes(root: &Path) -> Result<u64, StepFailure> {
    let metadata = fs::symlink_metadata(root).map_err(step_error)?;
    if metadata.file_type().is_symlink() || metadata_is_reparse(&metadata) || !metadata.is_dir() {
        return Err(StepFailure::new(format!(
            "source tree root is not a direct directory: {}",
            root.display()
        )));
    }
    let mut total = 0_u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(step_error)? {
            let entry = entry.map_err(step_error)?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(step_error)?;
            if metadata.file_type().is_symlink() || metadata_is_reparse(&metadata) {
                return Err(StepFailure::new(format!(
                    "source tree contains a link or reparse point: {}",
                    path.display()
                )));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                total = total
                    .checked_add(metadata.len())
                    .ok_or_else(|| StepFailure::new("source tree byte count overflowed u64"))?;
            } else {
                return Err(StepFailure::new(format!(
                    "source tree contains an unsupported filesystem entry: {}",
                    path.display()
                )));
            }
        }
    }
    Ok(total)
}

fn expected_log_components(
    plan: &InstallPlan,
    mods: &BTreeMap<String, ModFile>,
    target: GameRoot,
) -> Result<Vec<LogComponentReceipt>, StepFailure> {
    let mut expected = Vec::new();
    for run in plan.runs.iter().filter(|run| run.target == target) {
        let mod_file = mods.get(&run.mod_id).ok_or_else(|| {
            StepFailure::new(format!("frozen installer {:?} is missing", run.mod_id))
        })?;
        expected.extend(run.components.iter().map(|component| LogComponentReceipt {
            tp2: mod_file.tp2.replace('\\', "/").to_ascii_lowercase(),
            language: mod_file.language,
            component: *component,
        }));
    }
    Ok(expected)
}

fn same_log_entry(left: &LogEntry, right: &LogEntry) -> bool {
    left.tp2_key == right.tp2_key
        && left.language == right.language
        && left.component == right.component
}

fn log_component_receipt(entry: &LogEntry) -> LogComponentReceipt {
    LogComponentReceipt {
        tp2: entry.tp2_key.clone(),
        language: entry.language,
        component: entry.component,
    }
}

fn inferred_success_exit_code(statuses: &[DebugStatus]) -> Option<i32> {
    if statuses.is_empty()
        || statuses.iter().any(|status| {
            matches!(
                status,
                DebugStatus::NotInstalledDueToErrors | DebugStatus::Skipped
            )
        })
    {
        return None;
    }
    if statuses.contains(&DebugStatus::InstalledWithWarnings) {
        Some(3)
    } else {
        Some(0)
    }
}

fn retain_first_acquisition_receipt(
    first: &ArtifactReceipt,
    observed: &ArtifactReceipt,
) -> Result<ArtifactReceipt, StepFailure> {
    if first == observed {
        return Ok(first.clone());
    }
    let mut later_hit = observed.clone();
    let downloaded_then_hit = first.cache_outcome == ArtifactCacheOutcome::Downloaded
        && later_hit.cache_outcome == ArtifactCacheOutcome::Hit;
    later_hit.cache_outcome = first.cache_outcome;
    if downloaded_then_hit && &later_hit == first {
        return Ok(first.clone());
    }
    Err(StepFailure::new(format!(
        "artifact {:?} differs from its first acquisition evidence",
        first.id
    )))
}

fn sanitize_prompt_answer(answer: &str, bg1: &Path, bg2: &Path) -> String {
    answer
        .replace(&bg1.display().to_string(), "<staged-bg1>")
        .replace(&bg2.display().to_string(), "<staged-bg2>")
}

fn verified_launch_path(plan: &InstallPlan, game_root: &Path) -> Result<PathBuf, StepFailure> {
    let infinity_loader = game_root.join("InfinityLoader.exe");
    if plan
        .runs
        .iter()
        .any(|run| run.mod_id.eq_ignore_ascii_case("eeex"))
    {
        return direct_regular_file(&infinity_loader)
            .then_some(infinity_loader)
            .ok_or_else(|| {
                StepFailure::new(
                    "EEex is selected but InfinityLoader.exe is unavailable in the staged BG2 root",
                )
            });
    }
    if direct_regular_file(&infinity_loader) {
        return Ok(infinity_loader);
    }
    let vanilla = game_root.join("Baldur.exe");
    direct_regular_file(&vanilla)
        .then_some(vanilla)
        .ok_or_else(|| StepFailure::new("staged BG2 root has no verified launch executable"))
}

fn direct_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        !metadata.file_type().is_symlink() && !metadata_is_reparse(&metadata) && metadata.is_file()
    })
}

#[cfg(windows)]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes()
        & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
        != 0
}

#[cfg(not(windows))]
fn metadata_is_reparse(_metadata: &fs::Metadata) -> bool {
    false
}

fn load_recipe(recipe: &Path) -> Result<Manifest, CliError> {
    Manifest::load(recipe).map_err(|error| {
        CliError::new(
            "recipe_load_failed",
            format!("could not load recipe {}: {error}", recipe.display()),
        )
    })
}

fn load_profiles(recipe: &Path) -> Result<GameProfiles, CliError> {
    let path = recipe.join(GAME_PROFILE_DIRECTORY);
    GameProfiles::load(&path).map_err(|error| {
        CliError::new(
            "game_profiles_failed",
            format!(
                "could not load game profiles from {}: {error}",
                path.display()
            ),
        )
    })
}

fn split_assignment<'a>(
    authored: &'a str,
    label: &'static str,
) -> Result<(&'a str, &'a str), CliError> {
    let Some((key, value)) = authored.split_once('=') else {
        return Err(CliError::new(
            "invalid_selection",
            format!("{label} {authored:?} must use key=value"),
        ));
    };
    if key.is_empty() || value.is_empty() {
        return Err(CliError::new(
            "invalid_selection",
            format!("{label} {authored:?} must use nonempty key=value"),
        ));
    }
    Ok((key, value))
}

fn compare_candidates(left: &GameCandidate, right: &GameCandidate) -> Ordering {
    candidate_rank(right)
        .cmp(&candidate_rank(left))
        .then_with(|| left.findings.len().cmp(&right.findings.len()))
        .then_with(|| left.storefront.cmp(&right.storefront))
}

fn candidate_rank(candidate: &GameCandidate) -> u8 {
    match candidate.eligibility {
        Eligibility::Eligible => 2,
        Eligibility::Experimental => 1,
        Eligibility::Ineligible => 0,
    }
}

fn canonical_direct_directory(path: &Path, label: &str) -> Result<PathBuf, CliError> {
    validate_direct_directory(path, label)?;
    fs::canonicalize(path).map_err(|source| {
        CliError::new(
            "report_unavailable",
            format!(
                "could not canonicalize {label} {}: {source}",
                path.display()
            ),
        )
    })
}

fn validate_direct_directory(path: &Path, label: &str) -> Result<(), CliError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| {
        CliError::new(
            "report_unavailable",
            format!("could not inspect {label} {}: {source}", path.display()),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(CliError::new(
            "report_unavailable",
            format!("{label} is not a direct directory: {}", path.display()),
        ));
    }
    Ok(())
}

/// Stable phase labels used by human CLI output.
pub fn human_plan_lines(report: &PlanReport) -> Vec<String> {
    report
        .evaluation
        .plan
        .runs
        .iter()
        .enumerate()
        .map(|(index, run)| {
            let components = run
                .components
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{:03} {:?} {:?} {} {} [{}]",
                index + 1,
                run.phase,
                run.target,
                run.run_id,
                run.mod_id,
                components
            )
        })
        .collect()
}

/// Return preset choices with CLI overrides applied; useful to freeze new campaigns.
fn selection_choices(
    manifest: &Manifest,
    preset_id: &str,
    overrides: &SelectionOverrides,
) -> Result<BTreeMap<String, String>, CliError> {
    let preset = manifest.presets.get(preset_id).ok_or_else(|| {
        CliError::new("invalid_selection", format!("unknown preset {preset_id:?}"))
    })?;
    let mut choices = preset.selections.clone();
    for authored in &overrides.features {
        let (id, value) = split_assignment(authored, "feature override")?;
        if !matches!(value, "on" | "off" | "true" | "false") {
            return Err(CliError::new(
                "invalid_selection",
                format!(
                    "feature override {authored:?} must use on/off (true/false are accepted aliases)"
                ),
            ));
        }
        choices.insert(id.to_owned(), value.to_owned());
    }
    for authored in &overrides.inputs {
        let (id, value) = split_assignment(authored, "input override")?;
        let Some((feature_id, input_id)) = id.split_once('/') else {
            return Err(CliError::new(
                "invalid_selection",
                format!("input override {authored:?} must name feature-id/input-id"),
            ));
        };
        if feature_id.is_empty() || input_id.is_empty() || input_id.contains('/') {
            return Err(CliError::new(
                "invalid_selection",
                format!("input override {authored:?} must name feature-id/input-id"),
            ));
        }
        let _ = value;
        choices.insert(id.to_owned(), value.to_owned());
    }
    Ok(choices)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::receipt::{ArtifactCacheOutcome, ArtifactReceipt};
    use crate::weidu::runner::{RunnerControl, RunnerControlHandle};

    use super::{install_interrupt_handler_with, retain_first_acquisition_receipt};

    fn downloaded_artifact_receipt() -> ArtifactReceipt {
        ArtifactReceipt {
            id: "test-mod".to_owned(),
            version: "1.0".to_owned(),
            original_url: "https://example.invalid/test-mod.zip".to_owned(),
            final_url: "https://cdn.example.invalid/test-mod.zip".to_owned(),
            length: 42,
            sha256: "ab".repeat(32),
            cache_outcome: ArtifactCacheOutcome::Downloaded,
        }
    }

    #[test]
    fn later_cache_hit_retains_the_first_download_acquisition_receipt() {
        let first = downloaded_artifact_receipt();
        let mut revalidated = first.clone();
        revalidated.cache_outcome = ArtifactCacheOutcome::Hit;

        let retained = retain_first_acquisition_receipt(&first, &revalidated)
            .expect("same immutable archive may become a cache hit later in the campaign");

        assert_eq!(retained, first);
    }

    #[test]
    fn later_cache_observation_cannot_change_acquisition_provenance() {
        let first = downloaded_artifact_receipt();
        let mut changed = first.clone();
        changed.cache_outcome = ArtifactCacheOutcome::Hit;
        changed.final_url = "https://other.invalid/test-mod.zip".to_owned();

        let error = retain_first_acquisition_receipt(&first, &changed).unwrap_err();

        assert!(error
            .to_string()
            .contains("differs from its first acquisition evidence"));
    }

    #[test]
    fn interrupt_handler_forwards_cancel_to_the_active_runner_channel() {
        type InterruptHandler = Box<dyn FnMut() + Send + 'static>;

        let controls = RunnerControlHandle::new();
        let captured = Arc::new(Mutex::new(None::<InterruptHandler>));
        let captured_for_registration = Arc::clone(&captured);
        install_interrupt_handler_with(&controls, move |handler| {
            *captured_for_registration
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(handler);
            Ok::<_, &'static str>(())
        })
        .expect("register interrupt handler");

        let registration = controls.register().expect("register active runner");
        let mut handler = captured
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
            .expect("interrupt handler was captured");
        handler();

        assert_eq!(
            registration
                .receiver()
                .recv_timeout(Duration::from_secs(1))
                .expect("cancel was not forwarded"),
            RunnerControl::Cancel
        );
    }
}
