//! Deterministic, presentation-neutral operations exposed by the headless installer CLI.
//!
//! Argument parsing stays in the binary. This module owns recipe evaluation, game inspection,
//! managed-state reporting, and diagnostics selection so the desktop bridge can reuse the same
//! behavior without invoking a subprocess.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::acquire::{
    extract_archive_with_stop, materialize, provide_manual_archive, ArchiveFormat, ArchiveLimits,
    ArchiveMode, ArchiveRequirements, ArtifactCache, CacheDisposition, CacheRepair,
    DownloadRequest, DownloadRoute, ExtractedArtifact, MaterializationRequest, StopSignal,
    DEFAULT_DOWNLOAD_ATTEMPTS,
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
    run_campaign_controlled, ArtifactAcquirer, ArtifactKind, ArtifactMaterializer, BuiltInvocation,
    CampaignClock, CampaignOutcome, CampaignPreflight, CampaignRecorder, CampaignRequest,
    InstallLogVerifier, InstallReconciliation, InvocationBuilder, MaterializationOutcome,
    MaterializationTask, MutationCheck, MutationKind, ProcessResult, ProcessRunner, ReceiptDraft,
    ReceiptWriter, StagingService, StepAttempt, StepFailure,
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
use crate::recipe_view::{
    evaluate, feature_group_changes, FeatureGroupState, FeatureGroupTarget, SelectionEvaluation,
};
use crate::registry::ManagedInstallRegistry;
use crate::resolve::{InstallPlan, PlannedRun, Selection};
use crate::session::{
    CampaignCreated, FrozenIdentity, SessionEvent, SessionReplay, SessionStore,
    SourceGameFingerprints,
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
use crate::weidu::log::{
    has_new_uninstall_comment, normalize_tp2_key, parse_active_entries, parse_terminal_statuses,
    DebugStatus, LogEntry,
};
use crate::weidu::runner::{
    parse_prompt_results, run_controlled, PromptAnswerEvidence, RunOutcome, RunnerControlHandle,
    RunnerRequest, PROMPT_RESULTS_FILE_NAME,
};
use crate::weidu::verify::{reconcile as reconcile_weidu, ExpectedRun, Reconciliation};
use crate::Manifest;

mod installed_update;
mod recovery;
pub use installed_update::inspect_installed_update;
pub(crate) use installed_update::{verify_installed_baseline, VerifiedInstalledBaseline};
pub use recovery::{
    accept_supervised_recovery, inspect_supervised_recovery, SupervisedRecoveryRequest,
};

const GAME_PROFILE_DIRECTORY: &str = "game-builds";
const STATE_DIRECTORY: &str = ".chriz";
const ATTEMPTS_DIRECTORY: &str = "attempts";
const ATTEMPT_RECEIPT_FILE: &str = "receipt.json";
const LEGACY_CLI_FROZEN_RECIPE_SCHEMA: u32 = 1;
const CLI_FROZEN_RECIPE_SCHEMA: u32 = 2;
const LEGACY_DISPLAY_NAME: &str = "Chriz BG Collection";
const APPLICATION_DATA_DIRECTORY: &str = "Chriz BG Collection";
const EVIDENCE_DIRECTORY: &str = "evidence";
/// Later, accepted acquisitions of an artifact whose final URL differs from its write-once first
/// acquisition evidence, or that were downloaded anew. Stored as
/// `<category>/<artifact-id>/<digest>.json`.
const ARTIFACT_REACQUISITIONS_EVIDENCE: &str = "artifact-reacquisitions";
/// Damaged download or extraction cache entries that were set aside and rebuilt. Stored as
/// `<category>/<artifact-id>/<digest>.json`.
const CACHE_REPAIRS_EVIDENCE: &str = "cache-repairs";
const BEFORE_LOG_FILE: &str = "before.log";
const AFTER_LOG_FILE: &str = "after.log";
const INVOCATION_FILE: &str = "invocation.json";
/// Recipe-authored configuration files placed before the attempt's WeiDU run.
const CONFIG_FILES_EVIDENCE_FILE: &str = "config-files.json";
const PRE_SPAWN_FAILURE_FILE: &str = "pre-spawn-failure.json";
const PROCESS_RESULT_FILE: &str = "process-result.json";
const PROCESS_OUTPUT_FILE: &str = "process-output.log";
const STDOUT_FILE: &str = "stdout.log";
const STDERR_FILE: &str = "stderr.log";
const DEBUG_LOG_FILE: &str = "weidu.debug.log";
const WEIDU_SILENCE_THRESHOLD: Duration = Duration::from_secs(5 * 60);
static CAMPAIGN_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static APPLICATION_DATA_DIRECTORY_OVERRIDE: std::sync::OnceLock<String> =
    std::sync::OnceLock::new();

/// Which derived cache held a damaged entry.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum CacheRepairKind {
    Download,
    Extraction,
}

#[derive(Serialize)]
struct CacheRepairRecord<'a> {
    cache: CacheRepairKind,
    artifact_id: &'a str,
    artifact_sha256: &'a str,
    reason: &'a str,
}

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
    /// Enforce static evidence, omission, and approved-tail records for the public alpha.
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
    /// `group-id=on|off` values, expanded into member features after `features` apply; a group
    /// never overwrites a feature chosen explicitly in `features`, and leaves a variant's choice
    /// group to `features` when an explicit choice there disagrees with the group.
    pub groups: Vec<String>,
    /// `feature-id=on|off` values.
    pub features: Vec<String>,
    /// `feature-id/input-id=<typed-value>` values.
    pub inputs: Vec<String>,
    /// Advanced, opt-in component boundaries frozen with the reviewed recipe.
    pub manual_breakpoints: Vec<crate::manual_checkpoint::ManualBreakpoint>,
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
    /// Separate completion lineage after an explicitly supervised repair. The original
    /// terminal receipt above retains its actual failed outcome.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<crate::recovery_receipt::RecoveryReceipt>,
}

/// Inputs for one new CLI-owned managed installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallCommandRequest {
    /// Desktop application version, when this request originates from the launcher.
    pub application_version: Option<String>,
    /// Validated player-facing installation name frozen for review and resume.
    pub display_name: String,
    /// Executable recipe directory to freeze.
    pub recipe: PathBuf,
    /// Semantic preset used as the selection base.
    pub preset: String,
    /// Target platform passed to recipe evaluation.
    pub platform: String,
    /// Repeatable semantic feature and typed-input overrides.
    pub overrides: SelectionOverrides,
    /// Read-only BGEE plus SoD source; required for an EET recipe and `None` for a standalone
    /// recipe such as Icewind Dale: Enhanced Edition.
    pub bg1: Option<PathBuf>,
    /// Read-only primary source: BG2EE for an EET recipe, or the one game of a standalone
    /// recipe.
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
    /// Exact player-facing installation name included in the frozen recipe identity.
    pub display_name: String,
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
    /// The current step completed durably and the frozen campaign can be resumed.
    Paused {
        /// Last pipeline step proven complete before orchestration stopped.
        after_step_id: String,
    },
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    release_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    application_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    collection: Collection,
    artifacts: BTreeMap<String, Artifact>,
    mods: BTreeMap<String, ModFile>,
    presets: BTreeMap<String, PresetFile>,
    profiles: GameProfiles,
    plan: InstallPlan,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    manual_checkpoints: Vec<crate::manual_checkpoint::FrozenCheckpoint>,
    normalized_selection: crate::recipe_view::NormalizedSelection,
    preset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bg1: Option<FrozenSource>,
    /// Primary source: BG2EE for EET, or the one standalone game.
    bg2: FrozenSource,
    /// Stable recipe-family id from the release ledger; frozen only for standalone recipes so
    /// every EET payload keeps its exact bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recipe_id: Option<String>,
}

impl FrozenCliRecipe {
    /// Every frozen source in staging order: BGEE+SoD (EET only), then the primary game.
    fn sources(&self) -> Vec<&FrozenSource> {
        self.bg1.iter().chain(std::iter::once(&self.bg2)).collect()
    }

    /// Every staged root in staging order: BG1 (EET only), then the primary game root.
    fn targets(&self) -> Vec<GameRoot> {
        if self.bg1.is_some() {
            vec![GameRoot::Bg1, GameRoot::Bg2]
        } else {
            vec![GameRoot::Bg2]
        }
    }

    /// Executable launched when no EEex loader applies, from the frozen primary-source profile.
    fn fallback_executable(&self) -> Result<String, String> {
        if self.collection.game.standalone_role().is_none() {
            return Ok("Baldur.exe".to_owned());
        }
        let source = &self.bg2;
        self.profiles
            .matching(source.role, source.storefront)
            .find(|profile| Some(&profile.product_version) == source.build.as_ref())
            .map(|profile| profile.executable.clone())
            .ok_or_else(|| {
                format!(
                    "the frozen recipe has no {} profile for build {}",
                    source.role.display_name(),
                    source.build.as_deref().unwrap_or("unknown")
                )
            })
    }

    fn recipe_version(&self, digest: &str) -> String {
        self.release_version
            .clone()
            .unwrap_or_else(|| format!("local-{}", &digest[..12]))
    }

    fn effective_display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(LEGACY_DISPLAY_NAME)
    }
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

/// Reads optional release metadata bundled with the recipe. The exact version is frozen
/// into the campaign payload; older local recipes retain their content-hash identity.
pub fn recipe_release_version(root: &Path) -> Result<Option<String>, CliError> {
    let path = root.join("release.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CliError::new("recipe_release_invalid", error.to_string())),
    };
    if bytes.len() > 16 * 1024 {
        return Err(CliError::new(
            "recipe_release_invalid",
            "release metadata exceeds 16 KiB",
        ));
    }
    #[derive(Deserialize)]
    struct Release {
        version: String,
    }
    let release: Release = serde_json::from_slice(&bytes)
        .map_err(|error| CliError::new("recipe_release_invalid", error.to_string()))?;
    semver::Version::parse(&release.version)
        .map_err(|error| CliError::new("recipe_release_invalid", error.to_string()))?;
    Ok(Some(release.version))
}

/// Reads the stable recipe-family id from the release ledger of `version`, when the recipe
/// ships release metadata. A recipe without `release.json` has no recipe id.
fn recipe_release_id(root: &Path, version: Option<&str>) -> Result<Option<String>, CliError> {
    let Some(version) = version else {
        return Ok(None);
    };
    let path = root
        .join("releases")
        .join(format!("v{version}"))
        .join("ledger.toml");
    let release = crate::updates::load_release(&path)
        .map_err(|error| CliError::new("recipe_release_invalid", error.to_string()))?;
    if release.version != version {
        return Err(CliError::new(
            "recipe_release_invalid",
            format!(
                "release ledger {} records version {}, but release.json names {version}",
                path.display(),
                release.version
            ),
        ));
    }
    Ok(Some(release.recipe_id))
}

/// Every source profile of a recipe must describe a game this collection installs: the two EET
/// sources for EET, or exactly the standalone game otherwise.
fn check_profile_roles(
    game: crate::manifest::CollectionGame,
    profiles: &GameProfiles,
) -> Result<(), CliError> {
    for profile in profiles.iter() {
        let allowed = match game.standalone_role() {
            Some(role) => profile.role == role,
            None => matches!(profile.role, GameRole::BgeeSod | GameRole::Bg2ee),
        };
        if !allowed {
            return Err(CliError::new(
                "validation_failed",
                format!(
                    "game profile {:?} is for {}, which this {} recipe does not install",
                    profile.id,
                    profile.role.display_name(),
                    game.display_name()
                ),
            ));
        }
    }
    Ok(())
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
            CampaignStatus::Paused { .. } => (
                "invalid_campaign_result",
                "a paused campaign is resumable and cannot be represented as an error".to_owned(),
            ),
            CampaignStatus::Failed { step_id, reason } => {
                let code = if let Some(code) = folder_step_code(reason) {
                    code
                } else if reason == crate::weidu::ui_compat::LEGACY_FAILURE {
                    "installer_launch_unsupported"
                } else if step_id == "preflight" && reason.contains("source_not_fresh") {
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

    /// A command-line usage error detected after argument parsing.
    pub fn invalid_arguments(message: impl Into<String>) -> Self {
        Self::new("invalid_arguments", message)
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
    let mut raw_findings = validate::validate(&manifest);
    if profile == ValidationProfile::PublicAlpha {
        raw_findings.extend(
            crate::release_validate::validate_public_alpha(&manifest).map_err(|error| {
                CliError::new(
                    "validation_failed",
                    format!(
                        "recipe {} could not load public-alpha release records: {error}",
                        manifest.root.display()
                    ),
                )
            })?,
        );
    }
    let findings = raw_findings
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
    let choices = selection_choices(&manifest, preset_id, platform, overrides)?;
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
    Ok(install_review_identity(
        &prepared.created,
        prepared.frozen.effective_display_name(),
    ))
}

/// Start only when the current inputs still match the identity accepted at Review.
pub fn install_campaign_reviewed<S: EventSink + Sync>(
    request: &InstallCommandRequest,
    expected: &InstallReviewIdentity,
    sink: &S,
    controls: &RunnerControlHandle,
) -> Result<CampaignReport, CliError> {
    let prepared = prepare_install(request)?;
    let current =
        install_review_identity(&prepared.created, prepared.frozen.effective_display_name());
    if current != *expected {
        return Err(CliError::new(
            "review_changed",
            "the installation name, recipe, selection, sources, destination, or cache changed after Review; review the build again",
        ));
    }
    execute_frozen_campaign(prepared.created, prepared.frozen, sink, controls)
}

fn prepare_install(request: &InstallCommandRequest) -> Result<PreparedInstall, CliError> {
    let display_name = crate::registry::names::normalize_display_name(&request.display_name)
        .map_err(|error| CliError::new("invalid_display_name", error.to_string()))?;
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
    // The folder must agree with this process before anything else is checked.
    collection_application_data_directory(&manifest.collection)?;
    let unsupported = crate::release_validate::validate_production_invocations(&manifest);
    if !unsupported.is_empty() {
        return Err(CliError::new(
            "installer_launch_unsupported",
            unsupported
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }
    let choices = selection_choices(
        &manifest,
        &request.preset,
        &request.platform,
        &request.overrides,
    )?;
    let evaluation = evaluate(
        &manifest,
        &Selection {
            platform: request.platform.clone(),
            choices,
        },
    )
    .map_err(|error| CliError::new("invalid_selection", error.to_string()))?;
    let profiles = load_profiles(&manifest.root)?;
    let game = manifest.collection.game;
    check_profile_roles(game, &profiles)?;
    let standalone_role = game.standalone_role();
    let bg1 = match (standalone_role, &request.bg1) {
        (None, Some(path)) => Some(inspect_source_for_campaign(
            &profiles,
            GameRole::BgeeSod,
            path,
            "BGEE+SoD",
        )?),
        (None, None) => {
            return Err(CliError::new(
                "invalid_sources",
                "this Enhanced Edition Trilogy recipe needs a Baldur's Gate: Enhanced Edition source",
            ));
        }
        (Some(role), Some(_)) => {
            return Err(CliError::new(
                "invalid_sources",
                format!(
                    "this {} recipe installs one game; do not supply a Baldur's Gate: Enhanced Edition source",
                    role.display_name()
                ),
            ));
        }
        (Some(_), None) => None,
    };
    let primary_role = standalone_role.unwrap_or(GameRole::Bg2ee);
    let primary_label = primary_role.short_label();
    let bg2 = inspect_source_for_campaign(&profiles, primary_role, &request.bg2, primary_label)?;
    let frozen_bg1 = bg1
        .as_ref()
        .map(|bg1| FrozenSource::from_candidate(bg1, "BGEE+SoD"))
        .transpose()?;
    let frozen_bg2 = FrozenSource::from_candidate(&bg2, primary_label)?;
    let managed_root = prospective_direct_path(&request.managed_root, "managed root")?;
    let cache_root = canonical_existing_directory(&request.cache, "cache root")?;
    let mut source_roots = Vec::new();
    if let Some(bg1) = &bg1 {
        source_roots.push(("BGEE+SoD", bg1.root.as_path()));
    }
    source_roots.push((primary_label, bg2.root.as_path()));
    reject_campaign_path_overlaps(&managed_root, &cache_root, &source_roots)?;
    checked_application_data_root(&manifest.collection, &managed_root)?;
    let release_version = recipe_release_version(&manifest.root)?;
    let recipe_id = if game.is_standalone() {
        recipe_release_id(&manifest.root, release_version.as_deref())?
    } else {
        None
    };
    let (execution_plan, mut manual_checkpoints) = crate::manual_checkpoint::split_plan(
        &evaluation.plan,
        &request.overrides.manual_breakpoints,
    )
    .map_err(|error| CliError::new("invalid_manual_breakpoint", error))?;
    for checkpoint in &mut manual_checkpoints {
        crate::manual_checkpoint::label_component(&mut checkpoint.after, &manifest.mods);
        if let Some(next) = &mut checkpoint.next {
            crate::manual_checkpoint::label_component(next, &manifest.mods);
        }
    }
    let frozen = FrozenCliRecipe {
        schema: CLI_FROZEN_RECIPE_SCHEMA,
        release_version,
        application_version: request.application_version.clone(),
        display_name: Some(display_name),
        collection: manifest.collection.clone(),
        artifacts: manifest.artifacts.clone(),
        mods: manifest.mods.clone(),
        presets: manifest.presets.clone(),
        profiles,
        plan: execution_plan,
        manual_checkpoints,
        normalized_selection: evaluation.normalized_selection.clone(),
        preset: request.preset.clone(),
        bg1: frozen_bg1,
        bg2: frozen_bg2,
        recipe_id,
    };
    let recipe_payload = serialize_frozen_recipe(&frozen)?;
    let recipe_payload_sha256 = sha256_bytes(&recipe_payload);
    let recipe_envelope = frozen_recipe_envelope(&recipe_payload_sha256)?;
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
        plan_sha256: plan_digest(&frozen.plan)
            .map_err(|error| CliError::new("recipe_freeze_failed", error.to_string()))?,
        source_games: SourceGameFingerprints {
            bg1: frozen.bg1.as_ref().map(|bg1| bg1.fingerprint.clone()),
            bg2: frozen.bg2.fingerprint.clone(),
            standalone_role,
        },
        artifact_identities: frozen_payload_identities(&frozen)?,
        tool_identities: frozen_tool_identities(&frozen)?,
        staged_bg1: frozen.bg1.as_ref().map(|_| managed_root.join("bg1")),
        staged_bg2: managed_root.join("game"),
    };
    Ok(PreparedInstall { created, frozen })
}

fn install_review_identity(created: &CampaignCreated, display_name: &str) -> InstallReviewIdentity {
    InstallReviewIdentity {
        display_name: display_name.to_owned(),
        recipe_payload_sha256: created.recipe_payload_sha256.clone(),
        selection_sha256: created.selection_sha256.clone(),
        plan_sha256: created.plan_sha256.clone(),
        source_games: created.source_games.clone(),
        managed_root: created.managed_root.clone(),
        cache_root: created.cache_root.clone(),
    }
}

fn validate_display_name(display_name: &str) -> Result<(), CliError> {
    if display_name.trim().is_empty() {
        return Err(CliError::new(
            "invalid_display_name",
            "installation name must not be blank",
        ));
    }
    if display_name.chars().any(char::is_control) {
        return Err(CliError::new(
            "invalid_display_name",
            "installation name must not contain control characters",
        ));
    }
    if display_name.chars().any(|character| {
        matches!(
            character,
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
        )
    }) {
        return Err(CliError::new(
            "invalid_display_name",
            "installation name must not contain Windows filename characters < > : \" / \\ | ? *",
        ));
    }
    Ok(())
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
    recreate_deleted_cache_root(&store, &managed_root, expected_install_id)?;
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
    let frozen = resumable_frozen_recipe(&created, &managed_root, expected_install_id)?;
    execute_frozen_campaign(created, frozen, sink, controls)
}

/// The frozen recipe of a campaign that this request may resume.
fn resumable_frozen_recipe(
    created: &CampaignCreated,
    managed_root: &Path,
    expected_install_id: Option<&str>,
) -> Result<FrozenCliRecipe, CliError> {
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
    validate_frozen_cli_recipe(created, &frozen)?;
    // A campaign is resumed only into its own, safely placed application data folder; this runs
    // before the deleted-cache repair or anything else writes.
    checked_application_data_root(&frozen.collection, managed_root)?;
    Ok(frozen)
}

/// Creates the campaign's download cache again at its recorded path when that folder no longer
/// exists, for example after it was deleted by hand or by the uninstaller, so that resuming
/// downloads again whatever is missing. Anything else at that path is left for the usual
/// campaign checks to report, and nothing is ever removed.
///
/// Only a campaign that this request may resume gets its cache back, and nothing is created
/// unless the recorded path still leads through direct folders to itself, so that no link that
/// replaced one of its folders is followed, its source games can be inspected, and the campaign
/// checks would accept a cache there.
fn recreate_deleted_cache_root(
    store: &SessionStore,
    managed_root: &Path,
    expected_install_id: Option<&str>,
) -> Result<(), CliError> {
    // A campaign that cannot be read is reported by the full replay that follows.
    let Ok(replay) = store.replay_for_inspection() else {
        return Ok(());
    };
    let created = replay.created();
    let cache_root = &created.cache_root;
    match fs::symlink_metadata(cache_root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        _ => return Ok(()),
    }
    let frozen = resumable_frozen_recipe(created, managed_root, expected_install_id)?;
    let unavailable = |reason: String| {
        CliError::new(
            "resume_unavailable",
            format!(
                "the download cache {} was deleted and cannot be created there again: {reason}",
                cache_root.display()
            ),
        )
    };
    if !cache_root.is_absolute()
        || cache_root
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(unavailable(
            "its recorded path is not a full path".to_owned(),
        ));
    }
    let resolved = prospective_direct_path(cache_root, "download cache")
        .map_err(|error| unavailable(error.to_string()))?;
    if !crate::preflight::paths_equal(&resolved, cache_root) {
        return Err(unavailable(format!(
            "its path now leads to {}",
            resolved.display()
        )));
    }
    // The source games are checked as recorded and where they are now, as resuming checks them
    // next: a source folder that moved behind a link must not receive the cache either.
    let recorded = frozen
        .sources()
        .into_iter()
        .map(|source| source.root.clone())
        .collect::<Vec<_>>();
    let current = frozen
        .sources()
        .into_iter()
        .map(|source| inspect_frozen_source(&frozen.profiles, source).map(|found| found.root))
        .collect::<Result<Vec<_>, _>>()?;
    for roots in [&recorded, &current] {
        crate::preflight::check_cache_root_placement(&resolved, roots, managed_root)
            .map_err(|error| unavailable(error.to_string()))?;
    }
    fs::create_dir_all(cache_root).map_err(|error| {
        CliError::new(
            "resume_unavailable",
            format!(
                "could not create the download cache {} again: {error}",
                cache_root.display()
            ),
        )
    })
}

fn open_manual_campaign(
    managed_root: &Path,
    expected_install_id: &str,
) -> Result<(SessionReplay, FrozenCliRecipe), CliError> {
    // A paused campaign can be checked without its downloads; resuming recreates the cache.
    let replay = SessionStore::open(managed_root)
        .and_then(|store| store.replay_for_inspection())
        .map_err(|error| CliError::new("manual_checkpoint_unavailable", error.to_string()))?;
    if replay.created().install_id != expected_install_id || replay.fresh_copy_required().is_some()
    {
        return Err(CliError::new(
            "manual_checkpoint_unavailable",
            "The saved campaign identity is unavailable or requires attention.",
        ));
    }
    let frozen: FrozenCliRecipe = serde_json::from_slice(&replay.created().recipe_payload)
        .map_err(|error| CliError::new("manual_checkpoint_unavailable", error.to_string()))?;
    validate_frozen_cli_recipe(replay.created(), &frozen)?;
    Ok((replay, frozen))
}

fn pending_manual<'a>(
    replay: &SessionReplay,
    frozen: &'a FrozenCliRecipe,
) -> Option<&'a crate::manual_checkpoint::FrozenCheckpoint> {
    let step = replay.unresolved_step()?;
    frozen
        .manual_checkpoints
        .iter()
        .find(|checkpoint| step == format!("checkpoint:{}", checkpoint.checkpoint_id))
}

/// Read a saved manual pause; fresh changes are inspected only on an explicit check.
pub fn inspect_manual_checkpoint(
    managed_root: &Path,
    expected_install_id: &str,
    check_changes: bool,
) -> Result<Option<crate::manual_checkpoint::ManualCheckpointStatus>, CliError> {
    let lock_root = target_lock_root()?;
    let _lock = crate::lock::TargetLock::try_acquire(&lock_root, managed_root)
        .map_err(|error| CliError::new("manual_checkpoint_busy", error.to_string()))?;
    let (replay, frozen) = open_manual_campaign(managed_root, expected_install_id)?;
    let Some(checkpoint) = pending_manual(&replay, &frozen) else {
        return Ok(None);
    };
    if check_changes {
        crate::manual_checkpoint::check_manual_processes().map_err(manual_error)?;
        recheck_staged_roots(replay.created())?;
        crate::manual_checkpoint::review(
            managed_root,
            checkpoint,
            &frozen.plan,
            &frozen.mods,
            &frozen.artifacts,
        )
        .map(|reviewed| Some(reviewed.status))
        .map_err(manual_error)
    } else {
        crate::manual_checkpoint::paused_status(managed_root, checkpoint)
            .map(Some)
            .map_err(manual_error)
    }
}

/// Bind explicit confirmation to the exact reviewed history under the campaign mutation lock.
pub fn accept_manual_checkpoint(
    managed_root: &Path,
    expected_install_id: &str,
    review_token: &str,
    accept_additions: bool,
) -> Result<(), CliError> {
    let lock_root = target_lock_root()?;
    let _lock = crate::lock::TargetLock::try_acquire(&lock_root, managed_root)
        .map_err(|error| CliError::new("manual_checkpoint_busy", error.to_string()))?;
    let (replay, frozen) = open_manual_campaign(managed_root, expected_install_id)?;
    let checkpoint = pending_manual(&replay, &frozen)
        .ok_or_else(|| manual_error("This campaign is not paused for manual mods."))?;
    crate::manual_checkpoint::check_manual_processes().map_err(manual_error)?;
    recheck_staged_roots(replay.created())?;
    let reviewed = crate::manual_checkpoint::review(
        managed_root,
        checkpoint,
        &frozen.plan,
        &frozen.mods,
        &frozen.artifacts,
    )
    .map_err(manual_error)?;
    if reviewed.status.review_token != review_token || review_token.is_empty() {
        return Err(manual_error("Manual installation evidence changed since review. Check changes again before accepting."));
    }
    if !accept_additions && !reviewed.status.additions.is_empty() {
        return Err(manual_error("Manual additions were found. Review and explicitly accept them, or restore the checkpoint backup."));
    }
    crate::manual_checkpoint::accept(managed_root, checkpoint, &reviewed).map_err(manual_error)
}

fn manual_error(error: impl std::fmt::Display) -> CliError {
    CliError::new("manual_checkpoint_needs_attention", error.to_string())
}

/// Rechecks every staged root of the campaign (BG1 only for EET) before manual-mod review.
fn recheck_staged_roots(created: &CampaignCreated) -> Result<(), CliError> {
    for root in created
        .staged_bg1
        .iter()
        .chain(std::iter::once(&created.staged_bg2))
    {
        recheck_target_before_mutation(root, "en_US").map_err(manual_error)?;
    }
    Ok(())
}

fn verify_manual_prefix(
    root: &Path,
    plan: &InstallPlan,
    frozen: &FrozenCliRecipe,
) -> Result<(), String> {
    let (bg1, bg2) = crate::manual_checkpoint::read_logs(root)?;
    for (target, log) in [(GameRoot::Bg1, bg1), (GameRoot::Bg2, bg2)] {
        let entries = parse_active_entries(&log).map_err(|e| e.to_string())?;
        let expected = crate::manual_checkpoint::expected_components(
            root,
            plan,
            &frozen.mods,
            &frozen.manual_checkpoints,
            target,
        )?;
        if entries.len() != expected.len()
            || !entries.iter().zip(&expected).all(|(a, b)| {
                a.tp2_key == b.tp2 && a.language == b.language && a.component == b.component
            })
        {
            return Err("Game history changed outside a declared manual checkpoint. Preserve WeiDU.log and the installation folder.".into());
        }
    }
    Ok(())
}

fn execute_frozen_campaign<S: EventSink + Sync>(
    created: CampaignCreated,
    frozen: FrozenCliRecipe,
    sink: &S,
    controls: &RunnerControlHandle,
) -> Result<CampaignReport, CliError> {
    let app_data = checked_application_data_root(&frozen.collection, &created.managed_root)?;
    let registry_root = target_lock_root()?;
    let current_bg1 = frozen
        .bg1
        .as_ref()
        .map(|bg1| inspect_frozen_source(&frozen.profiles, bg1))
        .transpose()?;
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
    let outcome =
        run_campaign_controlled(&request, &mut dependencies, sink, controls).map_err(|error| {
            dependencies
                .placement_error
                .take()
                .unwrap_or_else(|| map_orchestrator_error(error, &created))
        })?;
    Ok(CampaignReport {
        install_id: created.install_id,
        managed_root: created.managed_root,
        plan_sha256: created.plan_sha256,
        status: match outcome {
            CampaignOutcome::Complete => CampaignStatus::Complete,
            CampaignOutcome::Paused { after_step_id } => CampaignStatus::Paused { after_step_id },
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
    let recovery = recovery::report_recovery(&managed_root, &receipt, &frozen)?;
    Ok(ManagedReport {
        managed_root,
        receipt,
        recovery,
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

    let expected_sources = frozen
        .sources()
        .into_iter()
        .map(FrozenSource::receipt)
        .collect::<Vec<_>>();
    if receipt.source_games != expected_sources {
        return Err("source-game evidence differs from the frozen campaign".to_owned());
    }
    if receipt.versions.application.trim().is_empty()
        || receipt.versions.engine.trim().is_empty()
        || receipt.versions.manifest_schema != frozen.collection.schema
        || receipt.versions.recipe != frozen.recipe_version(&created.recipe_payload_sha256)
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
        || matches!(value, "." | "..")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(format!(
            "{label} must contain 1-128 ASCII letters, digits, '-', '_' or '.' and cannot be a dot path"
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

/// What the `hotpatch` command does to one registered installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotpatchAction {
    /// Show what Updates shows; never writes the game.
    Check,
    /// Apply the next hotpatch that a check offered with `review_token`.
    Apply {
        review_token: String,
        full_backup: bool,
        save_backup: bool,
    },
    /// Undo the latest applied hotpatch.
    Undo,
    /// Finish restoring a hotpatch that did not finish.
    Restore,
}

/// Run `action` on the registered installation at `managed_root` with the hotpatch catalog
/// folder `catalog`, as the app's Updates screen does. The registry is the one of this
/// process's application data folder; downloads go to `cache`.
pub fn hotpatch_managed_install(
    managed_root: &Path,
    catalog: &Path,
    cache: &Path,
    action: &HotpatchAction,
) -> Result<crate::hotpatch::HotpatchPreview, CliError> {
    let failed = |error: String| CliError::new("hotpatch_failed", error);
    let catalog = crate::hotpatch::catalog::load(catalog)
        .map_err(|error| CliError::new("hotpatch_catalog_invalid", error))?;
    let local = local_application_data()?;
    let app_data = local.join(application_data_directory());
    let root = fs::canonicalize(managed_root).map_err(|error| {
        CliError::new(
            "managed_install_unavailable",
            format!("{}: {error}", managed_root.display()),
        )
    })?;
    let record = ManagedInstallRegistry::open_or_create(&app_data)
        .and_then(|registry| registry.list())
        .map_err(|error| CliError::new("registry_unavailable", error.to_string()))?
        .into_iter()
        .map(|card| card.record)
        .find(|record| record.managed_root == root)
        .ok_or_else(|| {
            CliError::new(
                "managed_install_unavailable",
                format!(
                    "{} is not a completed installation in the {:?} registry",
                    root.display(),
                    application_data_directory()
                ),
            )
        })?;
    let locks = crate::lock::TargetLockNamespace::in_local_application_data(local)
        .map_err(|error| CliError::new("application_data_unavailable", error.to_string()))?;
    let ctx = crate::hotpatch::Context {
        record: &record,
        app_data: &app_data,
        locks: &locks,
        cache,
    };
    match action {
        HotpatchAction::Check => {
            let _lock = locks
                .try_acquire(&record.managed_root)
                .map_err(|error| failed(error.to_string()))?;
            crate::hotpatch::inspect(&ctx, &catalog)
        }
        HotpatchAction::Apply {
            review_token,
            full_backup,
            save_backup,
        } => crate::hotpatch::apply(&ctx, &catalog, review_token, *full_backup, *save_backup),
        HotpatchAction::Undo => crate::hotpatch::restore(&ctx, &catalog, true),
        HotpatchAction::Restore => crate::hotpatch::restore(&ctx, &catalog, false),
    }
    .map_err(failed)
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

/// The envelope bytes recorded beside a frozen payload with digest `payload_sha256`.
fn frozen_recipe_envelope(payload_sha256: &str) -> Result<Vec<u8>, CliError> {
    serde_json::to_vec(&FrozenRecipeEnvelope {
        schema: CLI_FROZEN_RECIPE_SCHEMA,
        kind: "local-cli-frozen-recipe",
        payload_sha256,
    })
    .map_err(|error| CliError::new("recipe_freeze_failed", error.to_string()))
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
    if !matches!(
        frozen.schema,
        LEGACY_CLI_FROZEN_RECIPE_SCHEMA | CLI_FROZEN_RECIPE_SCHEMA
    ) {
        return Err(CliError::new(
            "resume_unavailable",
            format!(
                "unsupported frozen CLI recipe schema {}; expected {LEGACY_CLI_FROZEN_RECIPE_SCHEMA} or {CLI_FROZEN_RECIPE_SCHEMA}",
                frozen.schema
            ),
        ));
    }
    if frozen.schema == CLI_FROZEN_RECIPE_SCHEMA && frozen.display_name.is_none() {
        return Err(CliError::new(
            "resume_unavailable",
            "schema-two frozen installation is missing its display name",
        ));
    }
    validate_display_name(frozen.effective_display_name()).map_err(|error| {
        CliError::new(
            "resume_unavailable",
            format!("frozen installation name is invalid: {error}"),
        )
    })?;
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
    if frozen.bg1.as_ref().map(|bg1| &bg1.fingerprint) != created.source_games.bg1.as_ref()
        || frozen.bg2.fingerprint != created.source_games.bg2
    {
        return Err(CliError::new(
            "resume_unavailable",
            "frozen CLI source identities differ from the campaign fingerprints",
        ));
    }
    let standalone_role = frozen.collection.game.standalone_role();
    if created.source_games.standalone_role != standalone_role {
        return Err(CliError::new(
            "resume_unavailable",
            "frozen CLI collection game differs from the campaign source roles",
        ));
    }
    let roles_match = match standalone_role {
        None => {
            frozen.bg1.as_ref().map(|bg1| bg1.role) == Some(GameRole::BgeeSod)
                && frozen.bg2.role == GameRole::Bg2ee
        }
        Some(role) => frozen.bg1.is_none() && frozen.bg2.role == role,
    };
    if !roles_match {
        return Err(CliError::new(
            "resume_unavailable",
            match standalone_role {
                None => {
                    "frozen CLI sources do not have the required BGEE+SoD/BG2EE roles".to_owned()
                }
                Some(role) => format!(
                    "frozen CLI source is not the one {} source this recipe requires",
                    role.display_name()
                ),
            },
        ));
    }
    if standalone_role.is_none() && frozen.recipe_id.is_some() {
        return Err(CliError::new(
            "resume_unavailable",
            "an Enhanced Edition Trilogy frozen recipe unexpectedly records a recipe id",
        ));
    }
    if standalone_role.is_some() != frozen.collection.application_data_directory.is_some() {
        return Err(CliError::new(
            "resume_unavailable",
            "a frozen recipe must name its own application data folder exactly when it installs a standalone game",
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

/// Declares the folder under `LOCALAPPDATA` that this process expects to hold the installation
/// registry and evidence, instead of the default `Chriz BG Collection`.
///
/// The recipe decides the folder (`application_data_directory` in a standalone collection, the
/// default for EET); this declaration only guards it: installing, resuming or recovering a
/// collection whose folder differs fails with `application_data_mismatch` before anything is
/// written. Target locks never depend on it. The name must be one plain folder name. It can be
/// set once per process; setting the same name again is accepted.
pub fn set_application_data_directory(name: &str) -> Result<(), CliError> {
    validate_application_data_directory(name)?;
    let stored = APPLICATION_DATA_DIRECTORY_OVERRIDE.get_or_init(|| name.to_owned());
    if stored != name {
        return Err(CliError::new(
            "application_data_unavailable",
            format!("the application data folder is already set to {stored:?}"),
        ));
    }
    Ok(())
}

/// Folder name under `LOCALAPPDATA` this process was declared for: the value given to
/// [`set_application_data_directory`], or `Chriz BG Collection`. Each installation still uses
/// the folder its recipe requires.
pub fn application_data_directory() -> &'static str {
    APPLICATION_DATA_DIRECTORY_OVERRIDE
        .get()
        .map_or(APPLICATION_DATA_DIRECTORY, String::as_str)
}

fn validate_application_data_directory(name: &str) -> Result<(), CliError> {
    crate::registry::names::check_application_data_directory(name).map_err(|reason| {
        CliError::new(
            "application_data_unavailable",
            format!("application data folder {name:?} {reason}"),
        )
    })
}

fn local_application_data() -> Result<PathBuf, CliError> {
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
    Ok(root)
}

/// Target locks of every application and collection: the legacy Baldur's Gate `locks` folder,
/// whatever application data folder this process or recipe uses for its registry.
fn target_lock_root() -> Result<PathBuf, CliError> {
    crate::lock::TargetLockNamespace::in_local_application_data(local_application_data()?)
        .map(|locks| locks.root().to_path_buf())
        .map_err(|error| CliError::new("application_data_unavailable", error.to_string()))
}

/// The application data folder that holds the registry and evidence of `collection`: its own
/// folder for a standalone game, the Baldur's Gate folder for EET.
///
/// The recipe decides. A folder chosen explicitly for this process (`--app-data-directory` or
/// [`set_application_data_directory`]) must agree, otherwise this fails before anything is
/// written, so a standalone installation can never land in another application's registry.
fn collection_application_data_directory(collection: &Collection) -> Result<&str, CliError> {
    let required = collection
        .application_data_directory
        .as_deref()
        .unwrap_or(APPLICATION_DATA_DIRECTORY);
    match APPLICATION_DATA_DIRECTORY_OVERRIDE.get() {
        Some(chosen) if chosen != required => Err(CliError::new(
            "application_data_mismatch",
            format!(
                "this {} installation keeps its records in the application data folder {required:?}, but this process was started for {chosen:?}",
                collection.game.display_name()
            ),
        )),
        _ => Ok(required),
    }
}

/// The registry folder of `collection` below `LOCALAPPDATA`, after proving that writing it is
/// safe for the campaign at `managed_root`:
///
/// - A standalone collection's folder must not be, contain or lie inside the Baldur's Gate
///   folder under any name. Its final path is compared, so a DOS 8.3 alias (`CHRIZB~1`), a
///   junction or a symbolic link is caught, not only the literal name.
/// - Neither the registry folder nor the shared target-lock folder may overlap the managed
///   root: the registry would otherwise be written into the campaign before staging, which then
///   refuses those entries as unowned, and removal refuses the overlap, leaving a registered
///   campaign that needs manual cleanup.
///
/// Every operation that creates campaign state or publishes a registry record calls this first.
fn checked_application_data_root(
    collection: &Collection,
    managed_root: &Path,
) -> Result<PathBuf, CliError> {
    let directory = collection_application_data_directory(collection)?;
    let local = local_application_data()?;
    let registry = local.join(directory);
    if collection.game.is_standalone() {
        let shared = resolved_path_identity(&local.join(APPLICATION_DATA_DIRECTORY))?;
        let own = resolved_path_identity(&registry)?;
        if paths_overlap(&own, &shared) {
            return Err(CliError::new(
                "application_data_shared",
                format!(
                    "the {} application data folder {} resolves to {}, which is or overlaps the Baldur's Gate application data folder {}",
                    collection.game.display_name(),
                    registry.display(),
                    own.display(),
                    shared.display()
                ),
            ));
        }
    }
    let managed = resolved_path_identity(managed_root)?;
    for (label, folder) in [
        ("application data", registry.clone()),
        ("target lock", target_lock_root()?),
    ] {
        let resolved = resolved_path_identity(&folder)?;
        if paths_overlap(&managed, &resolved) {
            return Err(CliError::new(
                "application_data_overlap",
                format!(
                    "managed root {} overlaps the {label} folder {}; choose a destination outside {}",
                    managed_root.display(),
                    folder.display(),
                    local.display()
                ),
            ));
        }
    }
    Ok(registry)
}

/// The final path of `path`: its deepest existing ancestor resolved by the file system (long
/// names for 8.3 aliases, junction and link targets), followed by the components that do not
/// exist yet.
fn resolved_path_identity(path: &Path) -> Result<PathBuf, CliError> {
    let unavailable = |error: std::io::Error| {
        CliError::new(
            "application_data_unavailable",
            format!("could not resolve {}: {error}", path.display()),
        )
    };
    let absolute = std::path::absolute(path).map_err(unavailable)?;
    let mut existing = absolute.as_path();
    let mut missing = Vec::new();
    loop {
        match fs::canonicalize(existing) {
            Ok(mut resolved) => {
                for component in missing.iter().rev() {
                    resolved.push(component);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let (Some(name), Some(parent)) = (existing.file_name(), existing.parent()) else {
                    return Err(unavailable(error));
                };
                missing.push(name.to_os_string());
                existing = parent;
            }
            Err(error) => return Err(unavailable(error)),
        }
    }
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
    sources: &[(&str, &Path)],
) -> Result<(), CliError> {
    for &(label, source_root) in sources {
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
    for (index, &(left_label, left)) in sources.iter().enumerate() {
        for &(right_label, right) in &sources[index + 1..] {
            if paths_overlap(left, right) {
                return Err(CliError::new(
                    "source_overlap",
                    format!(
                        "{left_label} source {} overlaps {right_label} source {}",
                        left.display(),
                        right.display()
                    ),
                ));
            }
        }
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
    current_bg1: Option<GameCandidate>,
    current_bg2: GameCandidate,
    app_data: PathBuf,
    sink: &'a S,
    controls: RunnerControlHandle,
    /// The placement refusal behind a failed registry recheck, reported with its own code.
    placement_error: Option<CliError>,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PreSpawnFailureEvidence {
    run_id: String,
    attempt: u32,
    components: Vec<u32>,
    guard: String,
    failure: String,
    files_sha256: BTreeMap<String, String>,
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
    terminal_evidence: &'a [u8],
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
    fn reviewed_ui_correction(&self, run: &PlannedRun) -> bool {
        let (Some(mod_file), Some(artifact), Some(tool)) = (
            self.frozen.mods.get(&run.mod_id),
            self.frozen.artifacts.get(&run.artifact_id),
            self.frozen.artifacts.get(&run.weidu_artifact_id),
        ) else {
            return false;
        };
        crate::weidu::ui_compat::reviewed(run, mod_file, artifact, tool)
    }

    fn new(
        created: &'a CampaignCreated,
        frozen: &'a FrozenCliRecipe,
        current_bg1: Option<GameCandidate>,
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
            placement_error: None,
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
        frozen_layout(&self.created.managed_root, self.frozen).map_err(stage_error)
    }

    fn primary_label(&self) -> &'static str {
        self.frozen.bg2.role.short_label()
    }

    /// Player-facing name of one staged root. Kept as the historical `Bg1`/`Bg2` spelling for
    /// EET so existing failure texts are unchanged.
    fn target_label(&self, target: GameRoot) -> String {
        if self.frozen.bg1.is_some() {
            format!("{target:?}")
        } else {
            self.frozen.bg2.role.display_name().to_owned()
        }
    }

    fn save_identity(&self, layout: &ManagedLayout) -> Result<ReservedSaveIdentity, StepFailure> {
        reserve_save_identity(
            &CliDocumentsLocator,
            layout,
            self.frozen.effective_display_name(),
            &self.created.install_id,
        )
        .map_err(stage_error)
    }

    /// Stop ends waits for another download or extraction of the same cache entry, which may
    /// belong to a process that hangs or is suspended.
    fn stop_signal(&self) -> StopSignal {
        let controls = self.controls.clone();
        StopSignal::new(move || controls.cancel_requested())
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
                let acquired = DownloadRoute::for_source(&artifact.source)
                    .and_then(|route| {
                        ArtifactCache::open(&self.created.cache_root).map(|cache| (cache, route))
                    })
                    .and_then(|(cache, route)| {
                        cache.acquire_with_stop(
                            &DownloadRequest {
                                request_id: artifact.id.clone(),
                                url: artifact.source.url.clone(),
                                expected_length,
                                expected_sha256: artifact.source.sha256.clone(),
                                redirect_hosts: artifact.source.redirect_hosts.clone(),
                                max_attempts: DEFAULT_DOWNLOAD_ATTEMPTS,
                                route,
                            },
                            self.sink,
                            &self.stop_signal(),
                        )
                    })
                    .map_err(step_error)?;
                if let Some(repair) = &acquired.repaired {
                    self.record_cache_repair(
                        CacheRepairKind::Download,
                        &artifact.id,
                        &acquired.metadata.sha256,
                        repair,
                    )?;
                }
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
            Some(first) => {
                match retain_first_acquisition_receipt(&first, &observed_receipt)? {
                    LaterAcquisition::Identical => {}
                    LaterAcquisition::Reacquired => {
                        self.record_artifact_reacquisition(&observed_receipt)?;
                    }
                }
                first
            }
            None => {
                self.persist_named_evidence("artifacts", &artifact.id, &observed_receipt)?;
                observed_receipt
            }
        };
        Ok((archive_path, receipt))
    }

    fn extract(&self, artifact: &Artifact) -> Result<ExtractedArtifact, StepFailure> {
        let (archive, _) = self.acquire_archive(artifact)?;
        let (extracted, repair) = extract_archive_with_stop(
            &archive,
            &self.created.cache_root.join("extracted"),
            &ArchiveRequirements {
                artifact_sha256: artifact.source.sha256.clone(),
                format: match artifact.archive.kind {
                    ManifestArchiveKind::Zip => ArchiveFormat::Zip,
                    ManifestArchiveKind::Iemod => ArchiveFormat::Iemod,
                    ManifestArchiveKind::SelfExtractingRar => ArchiveFormat::SelfExtractingRar,
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
            &self.stop_signal(),
        )
        .map_err(step_error)?;
        if let Some(repair) = &repair {
            self.record_cache_repair(
                CacheRepairKind::Extraction,
                &artifact.id,
                &extracted.artifact_sha256,
                repair,
            )?;
        }
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

    fn run_tool(&self, run: &PlannedRun) -> Result<VerifiedWeidu, StepFailure> {
        #[cfg(debug_assertions)]
        if std::env::var("CHRIZ_BG_COLLECTION_TEST_FAIL_TOOL")
            .ok()
            .as_deref()
            == Some(run.run_id.as_str())
        {
            return Err(StepFailure::new(
                "simulated failure to get the WeiDU executable",
            ));
        }
        self.verified_tool(&run.weidu_artifact_id)
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

    fn target_root(&self, target: GameRoot) -> Result<&Path, StepFailure> {
        match target {
            GameRoot::Bg1 => self.created.staged_bg1.as_deref().ok_or_else(|| {
                StepFailure::new("this installation stages no Baldur's Gate: Enhanced Edition copy")
            }),
            GameRoot::Bg2 => Ok(&self.created.staged_bg2),
        }
    }

    fn evidence_root(&self) -> PathBuf {
        self.created
            .managed_root
            .join(STATE_DIRECTORY)
            .join(EVIDENCE_DIRECTORY)
    }

    fn named_evidence_path(
        &self,
        category: impl AsRef<Path>,
        id: &str,
    ) -> Result<PathBuf, StepFailure> {
        validate_cli_identifier(id, "evidence id").map_err(StepFailure::new)?;
        let directory = self.evidence_root().join(category);
        ensure_directories(&directory)?;
        Ok(directory.join(format!("{id}.json")))
    }

    fn persist_named_evidence<T: Serialize>(
        &self,
        category: impl AsRef<Path>,
        id: &str,
        value: &T,
    ) -> Result<(), StepFailure> {
        write_json_once(&self.named_evidence_path(category, id)?, value)
    }

    fn load_named_evidence<T: for<'de> Deserialize<'de>>(
        &self,
        category: impl AsRef<Path>,
        id: &str,
    ) -> Result<Option<T>, StepFailure> {
        read_optional_json(&self.named_evidence_path(category, id)?)
    }

    /// Keep the provenance of a later acquisition that was accepted against, but differs
    /// from, the write-once first acquisition evidence (for example a new single-use signed
    /// redirect URL after the download cache was lost). The first evidence stays the receipt
    /// of record; each distinct observation is stored once, named by its own digest, so
    /// repeated identical observations are idempotent.
    fn record_artifact_reacquisition(&self, observed: &ArtifactReceipt) -> Result<(), StepFailure> {
        validate_cli_identifier(&observed.id, "artifact id").map_err(StepFailure::new)?;
        let digest = sha256_bytes(&serde_json::to_vec(observed).map_err(step_error)?);
        self.persist_named_evidence(
            Path::new(ARTIFACT_REACQUISITIONS_EVIDENCE).join(&observed.id),
            &digest,
            observed,
        )
    }

    /// Keep a record of a damaged download or extraction cache entry that was set aside and
    /// rebuilt from the verified archive, so a later diagnosis can see it happened. Each
    /// distinct record is stored once, named by its own digest.
    fn record_cache_repair(
        &self,
        cache: CacheRepairKind,
        artifact_id: &str,
        artifact_sha256: &str,
        repair: &CacheRepair,
    ) -> Result<(), StepFailure> {
        validate_cli_identifier(artifact_id, "artifact id").map_err(StepFailure::new)?;
        let record = CacheRepairRecord {
            cache,
            artifact_id,
            artifact_sha256,
            reason: &repair.reason,
        };
        let digest = sha256_bytes(&serde_json::to_vec(&record).map_err(step_error)?);
        self.persist_named_evidence(
            Path::new(CACHE_REPAIRS_EVIDENCE).join(artifact_id),
            &digest,
            &record,
        )
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

impl<S: EventSink> GuardedCliDependencies<'_, S> {
    /// Proves again, right before a registry write, that the registry folder is still this
    /// collection's own and outside the campaign: a folder replaced by an alias or link while
    /// the campaign ran must not receive its record.
    fn recheck_application_data(&mut self) -> Result<(), StepFailure> {
        let current = match checked_application_data_root(
            &self.frozen.collection,
            &self.created.managed_root,
        ) {
            Ok(current) => current,
            Err(error) => {
                let failure = StepFailure::new(error.to_string());
                // Reported with its own code rather than as a generic campaign error.
                self.placement_error = Some(error);
                return Err(failure);
            }
        };
        if current != self.app_data {
            return Err(StepFailure::new(format!(
                "the application data folder changed from {} to {} during the campaign",
                self.app_data.display(),
                current.display()
            )));
        }
        Ok(())
    }
}

impl<S: EventSink> CampaignRecorder for GuardedCliDependencies<'_, S> {
    fn before_campaign_state(&mut self, _created: &CampaignCreated) -> Result<(), StepFailure> {
        self.recheck_application_data()
    }

    fn record_campaign(&mut self, created: &CampaignCreated) -> Result<(), StepFailure> {
        self.recheck_application_data()?;
        ManagedInstallRegistry::open_or_create(&self.app_data)
            .and_then(|registry| {
                registry.publish_campaign_named(created, self.frozen.effective_display_name())
            })
            .map(|_| ())
            .map_err(step_error)
    }
}

impl<S: EventSink> CampaignPreflight for GuardedCliDependencies<'_, S> {
    fn initial(
        &mut self,
        request: &CampaignRequest,
        replay: &SessionReplay,
    ) -> Result<(), StepFailure> {
        match (&self.current_bg1, &self.frozen.bg1) {
            (Some(current), Some(frozen)) => {
                Self::ensure_source_fresh(current, frozen, "BGEE+SoD")?;
            }
            (None, None) => {}
            _ => {
                return Err(StepFailure::new(
                    "the current BGEE+SoD source does not match the frozen campaign",
                ));
            }
        }
        Self::ensure_source_fresh(&self.current_bg2, &self.frozen.bg2, self.primary_label())?;
        let required_artifacts = self.required_inputs(&request.created.artifact_identities, false);
        let required_tools = self.required_inputs(&request.created.tool_identities, true);
        let staged_copies = self
            .frozen
            .sources()
            .into_iter()
            .try_fold(0_u64, |sum, source| {
                directory_bytes(&source.root).and_then(|bytes| {
                    sum.checked_add(bytes)
                        .ok_or_else(|| StepFailure::new("source game byte count overflowed u64"))
                })
            })?;
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
            bg1_source: self.current_bg1.as_ref(),
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
        #[cfg(debug_assertions)]
        if std::env::var("CHRIZ_BG_COLLECTION_TEST_FAIL_RECHECK")
            .ok()
            .as_deref()
            == Some(format!("{}:{}", check.kind.as_str(), check.step_id).as_str())
        {
            let executable = self
                .frozen
                .fallback_executable()
                .map_err(StepFailure::new)?;
            return Err(step_error(
                crate::preflight::PreflightError::TargetProcessRunning {
                    executable: self.target_root(check.target)?.join(executable),
                },
            ));
        }
        if matches!(check.kind, crate::orchestrator::MutationKind::Stage) {
            for source in self.frozen.sources() {
                let current =
                    inspect_frozen_source(&self.frozen.profiles, source).map_err(step_error)?;
                let label = if source.role == GameRole::BgeeSod {
                    "BGEE+SoD"
                } else {
                    source.role.short_label()
                };
                Self::ensure_source_fresh(&current, source, label)?;
            }
            recheck_staging_target_before_mutation(self.target_root(check.target)?, "en_US")
                .map_err(step_error)?;
            return Ok(());
        }
        recheck_target_before_mutation(self.target_root(check.target)?, "en_US").map_err(step_error)
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
                    .map_err(stage_error)
            }
            GameRole::Bg2ee => stage_game_copy(&layout, GameRole::Bg2ee)
                .map(|_| ())
                .map_err(stage_error),
            GameRole::Iwdee => {
                // Claim the isolated save folder before copying, as BG1 staging does, so a
                // collision is reported before the copy; the engine_name is patched once all
                // runs are complete, at final identity.
                self.save_identity(&layout)?;
                stage_game_copy(&layout, role)
                    .map(|_| ())
                    .map_err(stage_error)
            }
        }
    }

    fn finalize_identity(&mut self) -> Result<(), StepFailure> {
        let layout = self.layout()?;
        let identity = self.save_identity(&layout)?;
        finalize_game_identity(&layout, &identity).map_err(stage_error)
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
            self.target_root(task.target)?,
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
        // An old, exact Infinity recipe is corrected to the already-supported setup-name
        // invocation. No explicit-TP2 execution is enabled and its frozen plan is unchanged.
        let ui_correction = self.reviewed_ui_correction(run) && components == [0];
        if mod_file.invocation_mode == InvocationMode::ExplicitTp2 && !ui_correction {
            return Err(StepFailure::new(crate::weidu::ui_compat::LEGACY_FAILURE));
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
                    self.created.staged_bg1.as_deref(),
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
        // Nothing for this attempt is prepared before its WeiDU executable, so failing to get it,
        // for example after a failed download or a Stop while another program unpacks it,
        // leaves the run retryable.
        let tool = self
            .run_tool(run)
            .map_err(StepFailure::before_preparation)?;
        // Placing configuration files comes next and is retryable as well: it only creates
        // complete files and accepts identical ones, and its evidence is written after the last
        // file, so a failed placement leaves the attempt with its before.log alone.
        let config_files = mod_file
            .components
            .iter()
            .filter(|component| components.contains(&component.id))
            .flat_map(|component| {
                component
                    .config_files
                    .iter()
                    .map(move |file| (component.id, file))
            })
            .collect::<Vec<_>>();
        if !config_files.is_empty() {
            let evidence =
                crate::weidu::config_files::place(self.target_root(run.target)?, &config_files)
                    .map_err(|error| step_error(error).before_preparation())?;
            write_json_once(
                &attempt.evidence_root.join(CONFIG_FILES_EVIDENCE_FILE),
                &evidence,
            )?;
        }
        if ui_correction {
            crate::weidu::invocation::validate_setup_name_declarations(
                self.frozen
                    .mods
                    .values()
                    .filter(|item| {
                        item.invocation_mode == InvocationMode::SetupName || item.id == run.mod_id
                    })
                    .map(|item| item.tp2.as_str()),
            )
            .map_err(step_error)?;
            let evidence = crate::weidu::ui_compat::prepare(self.target_root(run.target)?)
                .map_err(step_error)?;
            write_json_once(
                &attempt
                    .evidence_root
                    .join("ui-invocation-compatibility.json"),
                &evidence,
            )?;
        }
        // Run here, not only during materialization, so retrying an older frozen EET
        // attempt receives the same narrowly hash-guarded Windows compatibility fix.
        let artifact = self.artifact(&run.artifact_id)?;
        if let Some(compatibility) =
            crate::weidu::eet_compat::prepare(crate::weidu::eet_compat::EetCompatibilityInput {
                mod_id: &run.mod_id,
                artifact_id: &run.artifact_id,
                artifact_sha256: &artifact.source.sha256,
                target: run.target,
                tp2: &mod_file.tp2,
                components,
                target_root: self.target_root(run.target)?,
            })
            .map_err(step_error)?
        {
            write_json_once(
                &attempt.evidence_root.join("eet-compatibility.json"),
                &compatibility,
            )?;
        }
        let invocation = build_invocation(
            &InvocationInput {
                target: run.target,
                tp2: mod_file.tp2.clone(),
                mode: if ui_correction {
                    InvocationMode::SetupName
                } else {
                    mod_file.invocation_mode
                },
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
                silence_threshold: WEIDU_SILENCE_THRESHOLD,
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
    fn recover_before_spawn(
        &self,
        run: &PlannedRun,
        attempt: &StepAttempt,
    ) -> Result<Option<InstallReconciliation>, StepFailure> {
        let marker_path = attempt.evidence_root.join(PRE_SPAWN_FAILURE_FILE);
        let marker: Option<PreSpawnFailureEvidence> = read_optional_json(&marker_path)?;
        if marker.is_none()
            && fs::symlink_metadata(attempt.evidence_root.join(INVOCATION_FILE)).is_ok()
        {
            // A prepared invocation without an explicit guard record follows the existing
            // strict process/rollback reconciliation, even when execution files are missing.
            return Ok(None);
        }
        let replay = SessionStore::open(&self.created.managed_root)
            .and_then(|store| store.replay())
            .map_err(step_error)?;
        let detail = match replay.records.last().map(|record| &record.event) {
            Some(SessionEvent::StepFailed {
                step_id,
                attempt: failed_attempt,
                detail,
            }) if step_id == &attempt.step_id
                && *failed_attempt == attempt.attempt
                && attempt.step_id == format!("install:{}", run.run_id) =>
            {
                detail
            }
            _ => {
                return Err(StepFailure::new(
                    "unstarted-run recovery requires its matching terminal ledger failure",
                ))
            }
        };

        let components = if let Some(marker) = marker {
            if marker.run_id != run.run_id
                || marker.attempt != attempt.attempt
                || marker.components.is_empty()
                || !run.components.ends_with(&marker.components)
                || detail
                    != &format!(
                        "{}\nPre-spawn evidence SHA-256: {}",
                        marker.failure,
                        sha256_bytes(&read_direct_file(&marker_path)?)
                    )
            {
                return Err(StepFailure::new(
                    "pre-spawn failure evidence differs from the terminal ledger or frozen run",
                ));
            }
            let kind = match marker.guard.as_str() {
                "invocation-build" => MutationKind::InvocationBuild,
                "process-spawn" => MutationKind::ProcessSpawn,
                _ => return Err(StepFailure::new("unknown pre-spawn safety guard")),
            };
            if pre_spawn_file_digests(attempt, kind, true)? != marker.files_sha256 {
                return Err(StepFailure::new(
                    "pre-spawn attempt files differ from the durable guard evidence",
                ));
            }
            if kind == MutationKind::ProcessSpawn {
                let prepared: PreparedInvocationEvidence =
                    read_required_json(&attempt.evidence_root.join(INVOCATION_FILE))?;
                if prepared.run_id != run.run_id
                    || prepared.attempt != attempt.attempt
                    || prepared.components != marker.components
                {
                    return Err(StepFailure::new(
                        "pre-spawn invocation differs from its guarded remaining suffix",
                    ));
                }
            }
            marker.components
        } else if attempt.attempt == 1
            && detail == crate::weidu::ui_compat::LEGACY_FAILURE
            && self.reviewed_ui_correction(run)
        {
            // The shipped alpha.19 guard returned before tool preparation/invocation.json.
            // Require its exact failure, first attempt, before.log alone and (below) an
            // unchanged live log. Missing evidence for any other failure remains unsafe.
            pre_spawn_file_digests(attempt, MutationKind::InvocationBuild, false)?;
            run.components.clone()
        } else {
            // Alpha.13 predates the structured guard record. Its synchronized invocation
            // file is a mandatory write barrier before runner entry. Accept only its first
            // attempt, before.log alone, and the exact terminal target-process diagnostic.
            // Missing evidence after launch, arbitrary build errors and interrupted starts
            // must never enter this compatibility path.
            let executable = detail
                .strip_prefix("a process executable under the managed target is active: ")
                .filter(|_| attempt.attempt == 1)
                .ok_or_else(|| {
                    StepFailure::new("missing invocation is not proof that WeiDU never started")
                })?;
            let executable = Path::new(executable);
            if !executable.is_absolute()
                || !path_is_within(
                    &executable.canonicalize().map_err(step_error)?,
                    &self
                        .target_root(run.target)?
                        .canonicalize()
                        .map_err(step_error)?,
                )
            {
                return Err(StepFailure::new(
                    "legacy pre-spawn process diagnostic does not identify this target",
                ));
            }
            pre_spawn_file_digests(attempt, MutationKind::InvocationBuild, false)?;
            run.components.clone()
        };
        let before = read_direct_file(&attempt.evidence_root.join(BEFORE_LOG_FILE))?;
        if before != read_optional_weidu_log(self.target_root(run.target)?)? {
            return Err(StepFailure::new(
                "WeiDU.log changed after the pre-spawn safety guard failed",
            ));
        }
        Ok(Some(InstallReconciliation::Retry {
            remaining: components,
        }))
    }

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
        let after = read_optional_weidu_log(self.target_root(run.target)?)?;
        write_bytes_once(&attempt.evidence_root.join(AFTER_LOG_FILE), &after)?;
        let debug = read_direct_file(&attempt.evidence_root.join(DEBUG_LOG_FILE))?;
        let stdout = read_direct_file(&attempt.evidence_root.join(STDOUT_FILE))?;
        let terminal_evidence = select_terminal_evidence(&debug, &stdout);
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

        let statuses = parse_terminal_statuses(&String::from_utf8_lossy(terminal_evidence));
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
                    &String::from_utf8_lossy(terminal_evidence),
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
                    Reconciliation::StackDisturbed => stack_disturbed_reconciliation(
                        &String::from_utf8_lossy(&before),
                        &String::from_utf8_lossy(&after),
                        &run.run_id,
                        &run.mod_id,
                        &self.frozen.mods[&run.mod_id].tp2,
                        self.frozen.mods[&run.mod_id].language,
                        &prepared.components,
                    ),
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
                        terminal_evidence,
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
            terminal_evidence,
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
        let statuses = parse_terminal_statuses(&String::from_utf8_lossy(terminal_evidence));
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
    fn manual_checkpoints(&self) -> Vec<crate::manual_checkpoint::FrozenCheckpoint> {
        self.frozen.manual_checkpoints.clone()
    }

    fn manual_checkpoint(
        &mut self,
        checkpoint: &crate::manual_checkpoint::FrozenCheckpoint,
    ) -> Result<bool, StepFailure> {
        let root = &self.created.managed_root;
        if crate::manual_checkpoint::acceptance(root, checkpoint)
            .map_err(step_error)?
            .is_some()
        {
            crate::manual_checkpoint::check_manual_processes().map_err(step_error)?;
            for target in self.frozen.targets() {
                recheck_target_before_mutation(self.target_root(target)?, "en_US")
                    .map_err(step_error)?;
            }
            crate::manual_checkpoint::verify_accepted_current(
                root,
                checkpoint,
                &self.frozen.plan,
                &self.frozen.mods,
                &self.frozen.artifacts,
            )
            .map_err(step_error)?;
            Ok(true)
        } else {
            // Validate the complete managed prefix before freezing a hand-over baseline.
            let through = self
                .frozen
                .plan
                .runs
                .iter()
                .position(|run| run.run_id == checkpoint.after_run_id)
                .ok_or_else(|| StepFailure::new("Frozen checkpoint run is absent."))?;
            let prefix = InstallPlan {
                runs: self.frozen.plan.runs[..=through].to_vec(),
            };
            if !crate::manual_checkpoint::has_baseline(root, checkpoint).map_err(step_error)? {
                verify_manual_prefix(root, &prefix, self.frozen).map_err(step_error)?;
            }
            crate::manual_checkpoint::begin(
                root,
                checkpoint,
                &self.frozen.plan,
                &self.frozen.mods,
                &self.frozen.artifacts,
            )
            .map_err(step_error)?;
            Ok(false)
        }
    }

    fn recover(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
    ) -> Result<InstallReconciliation, StepFailure> {
        match self.recover_before_spawn(run, attempt)? {
            Some(reconciliation) => Ok(reconciliation),
            None => self.reconcile_install_attempt(run, attempt, None),
        }
    }

    fn snapshot_before(
        &mut self,
        run: &PlannedRun,
        attempt: &StepAttempt,
    ) -> Result<(), StepFailure> {
        ensure_directories(&attempt.evidence_root)?;
        let before = read_optional_weidu_log(self.target_root(run.target)?)?;
        write_bytes_once(&attempt.evidence_root.join(BEFORE_LOG_FILE), &before)
    }

    fn record_pre_spawn_failure(
        &mut self,
        run: &PlannedRun,
        components: &[u32],
        attempt: &StepAttempt,
        kind: MutationKind,
        failure: &StepFailure,
    ) -> Result<String, StepFailure> {
        let evidence = PreSpawnFailureEvidence {
            run_id: run.run_id.clone(),
            attempt: attempt.attempt,
            components: components.to_vec(),
            guard: kind.as_str().to_owned(),
            failure: failure.to_string(),
            files_sha256: pre_spawn_file_digests(attempt, kind, false)?,
        };
        let path = attempt.evidence_root.join(PRE_SPAWN_FAILURE_FILE);
        write_json_once(&path, &evidence)?;
        Ok(sha256_bytes(&read_direct_file(&path)?))
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
        let bg1_engine_name = if layout.is_standalone() {
            None
        } else {
            Some(read_engine_name(layout.bg1_root()).map_err(step_error)?)
        };
        let bg2_engine_name = read_engine_name(layout.game_root()).map_err(step_error)?;
        crate::stage::verify_profile_settings(&layout, &identity).map_err(step_error)?;
        if bg1_engine_name
            .as_ref()
            .is_some_and(|name| *name != identity.engine_name)
            || bg2_engine_name != identity.engine_name
        {
            return Err(StepFailure::new(
                "staged game identities differ from the reserved managed save identity",
            ));
        }
        let mut logs = Vec::new();
        for target in self.frozen.targets() {
            let bytes = read_optional_weidu_log(self.target_root(target)?)?;
            let entries =
                parse_active_entries(&String::from_utf8_lossy(&bytes)).map_err(step_error)?;
            let expected = crate::manual_checkpoint::expected_components(
                &self.created.managed_root,
                plan,
                &self.frozen.mods,
                &self.frozen.manual_checkpoints,
                target,
            )
            .map_err(step_error)?;
            if entries.len() != expected.len()
                || !entries.iter().zip(&expected).all(|(actual, expected)| {
                    actual.tp2_key == expected.tp2.replace('\\', "/").to_ascii_lowercase()
                        && actual.language == expected.language
                        && actual.component == expected.component
                })
            {
                return Err(StepFailure::new(format!(
                    "final {} WeiDU.log does not exactly match the frozen plan",
                    self.target_label(target)
                )));
            }
            logs.push(FinalLogReceipt {
                target,
                sha256: sha256_bytes(&bytes),
                components: entries.iter().map(log_component_receipt).collect(),
            });
        }
        let fallback = self
            .frozen
            .fallback_executable()
            .map_err(StepFailure::new)?;
        let root_label = if self.frozen.bg1.is_some() {
            "staged BG2 root".to_owned()
        } else {
            format!("staged {} folder", self.frozen.bg2.role.display_name())
        };
        let launch_path = verified_launch_path(plan, layout.game_root(), &fallback, &root_label)?;
        let final_state = FinalReceiptState {
            logs,
            bg1_engine_name,
            bg2_engine_name,
            managed_save_root: identity.save_root,
            launch_path,
            verification_summary: if crate::manual_checkpoint::is_customized(
                &self.created.managed_root,
            )
            .map_err(step_error)?
            {
                format!("Frozen managed stack and explicitly accepted manual WeiDU additions matched across {} planned runs. Manual compatibility is unverified; game updates are disabled.", plan.runs.len())
            } else {
                format!(
                    "exact frozen stack matched across {} planned WeiDU runs",
                    plan.runs.len()
                )
            },
        };
        // Game updates later compare the folder with what the installer finished.
        let game_files =
            crate::game_files::capture(layout.game_root()).map_err(StepFailure::new)?;
        self.persist_named_evidence("final", "game-files", &game_files)?;
        self.persist_named_evidence("final", "state", &final_state)
    }
}

impl<S: EventSink> ReceiptWriter for GuardedCliDependencies<'_, S> {
    fn write(&mut self, draft: &ReceiptDraft) -> Result<(), StepFailure> {
        let store = ReceiptStore::open(&self.created.managed_root, &self.created.install_id)
            .map_err(|error| StepFailure::new(error.to_string()))?;
        self.recheck_application_data()?;
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
                application: self
                    .frozen
                    .application_version
                    .clone()
                    .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_owned()),
                engine: env!("CARGO_PKG_VERSION").to_owned(),
                manifest_schema: self.frozen.collection.schema,
                recipe: self
                    .frozen
                    .recipe_version(&self.created.recipe_payload_sha256),
            },
            source_games: self
                .frozen
                .sources()
                .into_iter()
                .map(FrozenSource::receipt)
                .collect(),
            artifacts,
            weidu_tools,
            runs,
            final_state,
        };
        ManagedReceiptWriter::new(
            store,
            registry,
            self.frozen.effective_display_name().to_owned(),
            evidence,
        )
        .with_recipe_id(self.frozen.recipe_id.clone())
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

// Preserve a stable folder category through the existing string-only campaign ledger.
// This affects explanation only; retry eligibility still comes from the durable checks.
fn stage_error(error: crate::error::EngineError) -> StepFailure {
    match error.folder_failure_code() {
        Some(code) => StepFailure::new(format!("[{code}] {error}")),
        None => step_error(error),
    }
}

fn folder_step_code(reason: &str) -> Option<&'static str> {
    [
        "folder_access_denied",
        "folder_unavailable",
        "folder_in_use",
        "folder_conflict",
        "folder_failed",
    ]
    .into_iter()
    .find(|code| reason.starts_with(&format!("[{code}] ")))
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

fn pre_spawn_file_digests(
    attempt: &StepAttempt,
    kind: MutationKind,
    has_marker: bool,
) -> Result<BTreeMap<String, String>, StepFailure> {
    for directory in attempt.evidence_root.ancestors() {
        let metadata = fs::symlink_metadata(directory).map_err(step_error)?;
        if metadata.file_type().is_symlink() || metadata_is_reparse(&metadata) || !metadata.is_dir()
        {
            return Err(StepFailure::new(format!(
                "pre-spawn evidence directory is not direct: {}",
                directory.display()
            )));
        }
    }
    let prepared = match kind {
        MutationKind::InvocationBuild => false,
        MutationKind::ProcessSpawn => true,
        _ => {
            return Err(StepFailure::new(
                "cannot record a non-install guard as pre-spawn evidence",
            ))
        }
    };
    let mut files = BTreeMap::new();
    let mut marker_found = false;
    for entry in fs::read_dir(&attempt.evidence_root).map_err(step_error)? {
        let entry = entry.map_err(step_error)?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| StepFailure::new("pre-spawn evidence filename is not Unicode"))?;
        if name == PRE_SPAWN_FAILURE_FILE && has_marker {
            read_direct_file(&entry.path())?;
            marker_found = true;
            continue;
        }
        if name != BEFORE_LOG_FILE
            && !(prepared
                && matches!(
                    name.as_str(),
                    INVOCATION_FILE
                        | DEBUG_LOG_FILE
                        | CONFIG_FILES_EVIDENCE_FILE
                        | "eet-compatibility.json"
                        | "ui-invocation-compatibility.json"
                ))
        {
            return Err(StepFailure::new(format!(
                "unexpected evidence in an unstarted WeiDU attempt: {name}"
            )));
        }
        let bytes = read_direct_file(&entry.path())?;
        if name == DEBUG_LOG_FILE && !bytes.is_empty() {
            return Err(StepFailure::new(
                "pre-spawn debug evidence contains possible WeiDU output",
            ));
        }
        files.insert(name, sha256_bytes(&bytes));
    }
    if !files.contains_key(BEFORE_LOG_FILE)
        || (prepared
            && (!files.contains_key(INVOCATION_FILE) || !files.contains_key(DEBUG_LOG_FILE)))
        || has_marker != marker_found
    {
        return Err(StepFailure::new(
            "required pre-spawn attempt evidence is missing",
        ));
    }
    Ok(files)
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

fn non_prefix_component_diagnostic(
    before: &str,
    after: &str,
    run_id: &str,
    mod_id: &str,
    tp2: &str,
    language: u32,
    requested: &[u32],
) -> Option<String> {
    if requested.is_empty() || has_new_uninstall_comment(before, after) {
        return None;
    }
    let before_entries = parse_active_entries(before).ok()?;
    let after_entries = parse_active_entries(after).ok()?;
    if after_entries.len() < before_entries.len()
        || !before_entries
            .iter()
            .zip(&after_entries)
            .all(|(left, right)| same_log_entry(left, right))
    {
        return None;
    }

    let tp2_key = normalize_tp2_key(tp2);
    let appended = &after_entries[before_entries.len()..];
    if appended.is_empty()
        || appended
            .iter()
            .any(|entry| entry.tp2_key != tp2_key || entry.language != language)
    {
        return None;
    }

    // This is explanatory evidence only, never permission to resume. A missing
    // component and a different order can occur together; do not lose either
    // diagnosis by requiring the recorded subset to be an ordered subsequence.
    let requested_set = requested.iter().copied().collect::<BTreeSet<_>>();
    if requested_set.len() != requested.len() {
        return None;
    }
    let mut installed = Vec::with_capacity(appended.len());
    let mut installed_set = BTreeSet::new();
    for entry in appended {
        if !requested_set.contains(&entry.component)
            || !installed_set.insert(entry.component)
            || before_entries
                .iter()
                .any(|previous| same_log_entry(previous, entry))
        {
            return None;
        }
        installed.push(entry.component);
    }
    let missing = requested
        .iter()
        .copied()
        .filter(|component| !installed_set.contains(component))
        .collect::<Vec<_>>();
    let expected_installed = requested
        .iter()
        .copied()
        .filter(|component| installed_set.contains(component))
        .collect::<Vec<_>>();
    let order_differs = installed != expected_installed;
    if missing.is_empty() && !order_differs {
        return None;
    }

    let mut detail = format!(
        "For run {run_id:?} (mod {mod_id:?}), {} of {} requested components are recorded. Previously installed components are still recorded in the same order.",
        installed.len(),
        requested.len(),
    );
    if !missing.is_empty() {
        let missing = missing
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        detail.push_str(&format!(
            " Components {missing} are not recorded as installed. Missing rows do not identify the cause; see the attempt diagnostics."
        ));
    }
    if order_differs {
        detail.push_str(&format!(
            " Component order differs from the recipe: expected the recorded components in order {expected_installed:?}, found {installed:?}."
        ));
    }
    Some(detail)
}

fn stack_disturbed_reconciliation(
    before: &str,
    after: &str,
    run_id: &str,
    mod_id: &str,
    tp2: &str,
    language: u32,
    requested: &[u32],
) -> InstallReconciliation {
    let reason = non_prefix_component_diagnostic(
        before, after, run_id, mod_id, tp2, language, requested,
    )
    .map_or_else(
        || "WeiDU.log changed outside the exact frozen component suffix".to_owned(),
        |detail| {
            format!("The mod's recorded result does not match the installation recipe. {detail}")
        },
    );
    InstallReconciliation::FreshCopyRequired { reason }
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

pub(crate) fn select_terminal_evidence<'a>(debug: &'a [u8], stdout: &'a [u8]) -> &'a [u8] {
    // Setup-name WeiDU can redirect its detailed log to SETUP-<MOD>.DEBUG beside the
    // executable even when --log names our attempt path. In that case the requested
    // debug file contains only a header, while the runner's synchronized per-attempt
    // stdout still contains WeiDU's terminal component results. Never replace partial
    // debug evidence: a single recognized marker keeps debug authoritative so missing
    // or contradictory component results continue to fail closed.
    if parse_terminal_statuses(&String::from_utf8_lossy(debug)).is_empty() {
        stdout
    } else {
        debug
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

/// How a later acquisition of an artifact relates to its accepted first acquisition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LaterAcquisition {
    /// The first acquisition evidence itself, or a cache hit on that same download: routine
    /// reuse that adds no provenance and is not recorded.
    Identical,
    /// The same pinned archive from the same authored source, observed with a different final
    /// URL (signed single-use redirects) or acquired anew (a re-download after the download
    /// cache was lost).
    Reacquired,
}

/// Decide whether `observed` may stand in for the write-once `first` acquisition.
///
/// The first acquisition always remains the receipt of record. A later acquisition is
/// accepted when it names the same artifact, version and authored source URL and its
/// verified bytes have the same length and SHA-256; its final URL and Hit/Downloaded outcome
/// are transport details. Manual import versus download is not: the acquisition policy is
/// frozen, so such a switch means the recipe changed.
fn retain_first_acquisition_receipt(
    first: &ArtifactReceipt,
    observed: &ArtifactReceipt,
) -> Result<LaterAcquisition, StepFailure> {
    let cache_hit_on_first_download = first.cache_outcome == ArtifactCacheOutcome::Downloaded
        && observed.cache_outcome == ArtifactCacheOutcome::Hit
        && ArtifactReceipt {
            cache_outcome: ArtifactCacheOutcome::Downloaded,
            ..observed.clone()
        } == *first;
    if first == observed || cache_hit_on_first_download {
        return Ok(LaterAcquisition::Identical);
    }
    let same_archive = first.id == observed.id
        && first.version == observed.version
        && first.original_url == observed.original_url
        && first.length == observed.length
        && first.sha256 == observed.sha256;
    let same_policy = (first.cache_outcome == ArtifactCacheOutcome::Manual)
        == (observed.cache_outcome == ArtifactCacheOutcome::Manual);
    if same_archive && same_policy {
        return Ok(LaterAcquisition::Reacquired);
    }
    Err(StepFailure::new(format!(
        "artifact {:?} differs from its first acquisition evidence",
        first.id
    )))
}

/// Prepares the managed layout recorded by a frozen recipe: EET with both sources, or one
/// standalone game.
fn frozen_layout(
    managed_root: &Path,
    frozen: &FrozenCliRecipe,
) -> crate::error::Result<ManagedLayout> {
    match &frozen.bg1 {
        Some(bg1) => ManagedLayout::prepare(managed_root, &bg1.root, &frozen.bg2.root),
        None => ManagedLayout::prepare_standalone(managed_root, frozen.bg2.role, &frozen.bg2.root),
    }
}

fn sanitize_prompt_answer(answer: &str, bg1: Option<&Path>, bg2: &Path) -> String {
    match bg1 {
        Some(bg1) => answer
            .replace(&bg1.display().to_string(), "<staged-bg1>")
            .replace(&bg2.display().to_string(), "<staged-bg2>"),
        None => answer.replace(&bg2.display().to_string(), "<staged-game>"),
    }
}

/// The verified launch executable: InfinityLoader when EEex is selected or present, otherwise
/// the game's own executable named by `fallback` (`Baldur.exe` for EET, or the standalone
/// game's profile executable such as `icewind.exe`). `root_label` names the folder in errors.
fn verified_launch_path(
    plan: &InstallPlan,
    game_root: &Path,
    fallback: &str,
    root_label: &str,
) -> Result<PathBuf, StepFailure> {
    let infinity_loader = game_root.join("InfinityLoader.exe");
    if plan
        .runs
        .iter()
        .any(|run| run.mod_id.eq_ignore_ascii_case("eeex"))
    {
        return direct_regular_file(&infinity_loader)
            .then_some(infinity_loader)
            .ok_or_else(|| {
                StepFailure::new(format!(
                    "EEex is selected but InfinityLoader.exe is unavailable in the {root_label}"
                ))
            });
    }
    if direct_regular_file(&infinity_loader) {
        return Ok(infinity_loader);
    }
    let vanilla = game_root.join(fallback);
    direct_regular_file(&vanilla)
        .then_some(vanilla)
        .ok_or_else(|| StepFailure::new(format!("{root_label} has no verified launch executable")))
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
    let groups = report.evaluation.groups.iter().map(|group| {
        let title = report
            .evaluation
            .view
            .groups
            .iter()
            .find(|view| view.id == group.id)
            .map_or(group.id.as_str(), |view| view.title.as_str());
        let state = match group.state {
            FeatureGroupState::On => "on",
            FeatureGroupState::Off => "off",
            FeatureGroupState::Mixed => "mixed",
        };
        let variants = if group.variants.is_empty() {
            String::new()
        } else {
            format!(
                ", {} of {} available variants picked",
                group
                    .variants
                    .iter()
                    .filter(|variant| variant.selected)
                    .count(),
                group
                    .variants
                    .iter()
                    .filter(|variant| variant.available)
                    .count()
            )
        };
        format!(
            "Group {} ({title}): {state}, {} of {} available selected{variants}",
            group.id,
            group.selected_members.len(),
            group.available_members.len()
        )
    });
    let runs = report
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
        });
    groups.chain(runs).collect()
}

/// Return preset choices with CLI overrides applied; useful to freeze new campaigns.
fn selection_choices(
    manifest: &Manifest,
    preset_id: &str,
    platform: &str,
    overrides: &SelectionOverrides,
) -> Result<BTreeMap<String, String>, CliError> {
    let preset = manifest.presets.get(preset_id).ok_or_else(|| {
        CliError::new("invalid_selection", format!("unknown preset {preset_id:?}"))
    })?;
    let mut choices = preset.selections.clone();
    // Explicit `--feature` overrides apply first, so group expansion sees them: a prerequisite
    // switched on by `--feature` makes a dependent member available to `--group ...=on`.
    let mut explicit_features = BTreeMap::new();
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
        explicit_features.insert(id.to_owned(), matches!(value, "on" | "true"));
    }
    // Groups then expand in command-line order against that state plus earlier groups. A group
    // never overwrites a feature chosen explicitly, so `--feature` always wins for a member. In a
    // variant's choice group, an explicit choice that disagrees with the group leaves the whole
    // choice to the explicit choices; one that agrees, such as the variant option itself, still
    // lets the group deselect the other options, so they never both stay requested.
    let choice_group_of = manifest
        .collection
        .features
        .iter()
        .filter_map(|feature| Some((feature.id.as_str(), feature.choice_group.as_deref()?)))
        .collect::<BTreeMap<_, _>>();
    for authored in &overrides.groups {
        let (id, value) = split_assignment(authored, "group override")?;
        let target = match value {
            "on" => FeatureGroupTarget::On,
            "off" => FeatureGroupTarget::Off,
            _ => {
                return Err(CliError::new(
                    "invalid_selection",
                    format!("group override {authored:?} must use on or off"),
                ))
            }
        };
        let changes = feature_group_changes(
            manifest,
            &Selection {
                platform: platform.to_owned(),
                choices: choices.clone(),
            },
            id,
            target,
        )
        .map_err(|error| CliError::new("invalid_selection", error.to_string()))?;
        let overridden_choices = changes
            .iter()
            .filter(|(feature_id, selected)| {
                explicit_features
                    .get(feature_id.as_str())
                    .is_some_and(|explicit| explicit != *selected)
            })
            .filter_map(|(feature_id, _)| choice_group_of.get(feature_id.as_str()).copied())
            .collect::<BTreeSet<_>>();
        for (feature_id, selected) in changes {
            let left_to_explicit = explicit_features.contains_key(&feature_id)
                || choice_group_of
                    .get(feature_id.as_str())
                    .is_some_and(|choice| overridden_choices.contains(choice));
            if !left_to_explicit {
                choices.insert(feature_id, if selected { "on" } else { "off" }.to_owned());
            }
        }
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
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::digest::{plan_digest, selection_digest, sha256_bytes};
    use crate::games::{GameProfiles, GameRole, Storefront};
    use crate::receipt::{ArtifactCacheOutcome, ArtifactReceipt};
    use crate::recipe_view::evaluate;
    use crate::resolve::Selection;
    use crate::session::{CampaignCreated, SourceGameFingerprints};
    use crate::weidu::runner::{RunnerControl, RunnerControlHandle};
    use crate::Manifest;

    use super::{
        install_interrupt_handler_with, non_prefix_component_diagnostic,
        retain_first_acquisition_receipt, select_terminal_evidence, selection_choices,
        stack_disturbed_reconciliation, validate_frozen_cli_recipe, FrozenCliRecipe, FrozenSource,
        InstallReconciliation, LaterAcquisition, SelectionOverrides, CLI_FROZEN_RECIPE_SCHEMA,
    };

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
    fn frozen_infinity_retry_requires_exact_terminal_failure_and_unchanged_unstarted_evidence() {
        use super::{GuardedCliDependencies, StepAttempt};
        use crate::events::ChannelSink;
        use crate::games::{Eligibility, GameCandidate};
        use crate::manifest::InvocationMode;
        use crate::session::{SessionEvent, SessionStore};

        for case in [
            "valid",
            "wrong-reason",
            "wrong-pin",
            "wrong-run",
            "changed-log",
            "runner-output",
            "debug-output",
            "prepared-invocation",
            "wrong-attempt",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            let manifest =
                Manifest::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../manifest")).unwrap();
            let authored = manifest
                .collection
                .runs
                .iter()
                .find(|run| run.run_id == "game-interface-infinity-bg2")
                .unwrap();
            let mut run = crate::resolve::PlannedRun {
                run_id: authored.run_id.clone(),
                mod_id: authored.mod_id.clone(),
                target: authored.phase.game_root(),
                phase: authored.phase,
                components: authored.components.clone(),
                args: authored.args.clone(),
                postconditions: authored.postconditions.clone(),
                artifact_id: "infinity-ui-1.28".into(),
                weidu_artifact_id: "weidu-249-amd64".into(),
                prompt_scripts: vec![],
            };
            if case == "wrong-run" {
                run.run_id = "other-run".into();
            }
            let mut frozen = frozen_recipe_fixture();
            frozen.mods = manifest.mods;
            frozen.artifacts = manifest.artifacts;
            frozen.mods.get_mut("infinity-ui").unwrap().invocation_mode =
                InvocationMode::ExplicitTp2;
            if case == "wrong-pin" {
                frozen
                    .artifacts
                    .get_mut("infinity-ui-1.28")
                    .unwrap()
                    .source
                    .sha256 = "00".repeat(32);
            }
            frozen.plan.runs = vec![run.clone()];
            let mut created =
                created_for_frozen_recipe(&frozen, serde_json::to_vec(&frozen).unwrap());
            created.managed_root = root.clone();
            created.cache_root = root.join("cache");
            created.staged_bg1 = Some(root.join("bg1"));
            created.staged_bg2 = root.join("game");
            for path in [
                &created.cache_root,
                created.staged_bg1.as_ref().unwrap(),
                &created.staged_bg2,
            ] {
                std::fs::create_dir(path).unwrap();
            }
            let store =
                SessionStore::create(&root, SessionEvent::Created(Box::new(created.clone())))
                    .unwrap();
            let step_id = format!("install:{}", run.run_id);
            store
                .append(SessionEvent::StepStarted {
                    step_id: step_id.clone(),
                    attempt: 1,
                })
                .unwrap();
            store
                .append(SessionEvent::StepFailed {
                    step_id: step_id.clone(),
                    attempt: 1,
                    detail: if case == "wrong-reason" {
                        "some unrelated build error".into()
                    } else {
                        crate::weidu::ui_compat::LEGACY_FAILURE.into()
                    },
                })
                .unwrap();
            let evidence_root = root.join("evidence");
            std::fs::create_dir(&evidence_root).unwrap();
            std::fs::write(evidence_root.join("before.log"), b"").unwrap();
            if case == "changed-log" {
                std::fs::write(created.staged_bg2.join("WeiDU.log"), b"changed").unwrap();
            }
            if case == "runner-output" {
                std::fs::write(evidence_root.join("stdout.log"), b"possible execution").unwrap();
            }
            if case == "debug-output" {
                std::fs::write(evidence_root.join("weidu.debug.log"), b"possible execution")
                    .unwrap();
            }
            if case == "prepared-invocation" {
                std::fs::write(evidence_root.join("invocation.json"), b"{}").unwrap();
            }
            let attempt = StepAttempt {
                step_id,
                attempt: if case == "wrong-attempt" { 2 } else { 1 },
                evidence_root,
            };
            let candidate = GameCandidate {
                role: GameRole::Bg2ee,
                storefront: Storefront::Steam,
                root: root.clone(),
                build: None,
                eligibility: Eligibility::Eligible,
                findings: vec![],
                fingerprint: None,
            };
            let (sink, _events) = ChannelSink::unbounded();
            let dependencies = GuardedCliDependencies::new(
                &created,
                &frozen,
                Some(candidate.clone()),
                candidate,
                root.join("app-data"),
                &sink,
                RunnerControlHandle::default(),
            );
            let result = dependencies.recover_before_spawn(&run, &attempt);
            if case == "valid" {
                assert_eq!(
                    result.unwrap(),
                    Some(InstallReconciliation::Retry { remaining: vec![0] })
                );
            } else {
                assert!(
                    !matches!(result, Ok(Some(InstallReconciliation::Retry { .. }))),
                    "unsafe retry for {case}"
                );
            }
            assert_eq!(
                store.replay().unwrap().records.len(),
                3,
                "inspection must not rewrite history"
            );
        }
    }

    fn frozen_recipe_fixture() -> FrozenCliRecipe {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest");
        let manifest = Manifest::load(&root).expect("load frozen-recipe manifest fixture");
        let choices = selection_choices(
            &manifest,
            "recommended",
            "windows",
            &SelectionOverrides::default(),
        )
        .expect("select frozen-recipe fixture");
        let evaluation = evaluate(
            &manifest,
            &Selection {
                platform: "windows".to_owned(),
                choices,
            },
        )
        .expect("evaluate frozen-recipe fixture");
        let profiles = GameProfiles::load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/games/profiles"),
        )
        .expect("load game-profile fixtures");
        FrozenCliRecipe {
            release_version: None,
            manual_checkpoints: Vec::new(),
            application_version: None,
            schema: CLI_FROZEN_RECIPE_SCHEMA,
            display_name: Some("Chriz Easy BG".to_owned()),
            collection: manifest.collection,
            artifacts: manifest.artifacts,
            mods: manifest.mods,
            presets: manifest.presets,
            profiles,
            plan: evaluation.plan,
            normalized_selection: evaluation.normalized_selection,
            preset: "recommended".to_owned(),
            bg1: Some(FrozenSource {
                role: GameRole::BgeeSod,
                storefront: Storefront::Steam,
                root: PathBuf::from(r"C:\Legacy BG1"),
                build: Some("2.7.3.0".to_owned()),
                fingerprint: "11".repeat(32),
            }),
            bg2: FrozenSource {
                role: GameRole::Bg2ee,
                storefront: Storefront::Steam,
                root: PathBuf::from(r"C:\Legacy BG2"),
                build: Some("2.7.3.0".to_owned()),
                fingerprint: "22".repeat(32),
            },
            recipe_id: None,
        }
    }

    fn created_for_frozen_recipe(frozen: &FrozenCliRecipe, payload: Vec<u8>) -> CampaignCreated {
        let managed = PathBuf::from(r"C:\Legacy Managed Install");
        let envelope = b"legacy-envelope".to_vec();
        CampaignCreated {
            install_id: "install-legacy".to_owned(),
            attempt_id: "attempt-legacy".to_owned(),
            managed_root: managed.clone(),
            cache_root: PathBuf::from(r"C:\Legacy Cache"),
            recipe_payload_sha256: sha256_bytes(&payload),
            recipe_payload: payload,
            recipe_envelope_sha256: sha256_bytes(&envelope),
            recipe_envelope: envelope,
            selection_sha256: selection_digest(&frozen.normalized_selection)
                .expect("digest frozen selection"),
            normalized_selection: frozen.normalized_selection.clone(),
            plan_sha256: plan_digest(&frozen.plan).expect("digest frozen plan"),
            source_games: SourceGameFingerprints {
                bg1: frozen.bg1.as_ref().map(|bg1| bg1.fingerprint.clone()),
                bg2: frozen.bg2.fingerprint.clone(),
                standalone_role: None,
            },
            artifact_identities: Vec::new(),
            tool_identities: Vec::new(),
            staged_bg1: frozen.bg1.as_ref().map(|_| managed.join("bg1")),
            staged_bg2: managed.join("game"),
        }
    }

    #[test]
    fn schema_one_frozen_recipe_defaults_the_legacy_display_name_and_still_validates() {
        let current = frozen_recipe_fixture();
        let mut legacy = serde_json::to_value(&current).expect("serialize current frozen recipe");
        legacy["schema"] = serde_json::json!(1);
        legacy
            .as_object_mut()
            .expect("frozen recipe is an object")
            .remove("display_name");
        let payload = serde_json::to_vec(&legacy).expect("serialize legacy frozen recipe");

        let frozen: FrozenCliRecipe =
            serde_json::from_slice(&payload).expect("read schema-one frozen recipe");
        assert_eq!(frozen.schema, 1);
        assert!(frozen.display_name.is_none());
        assert_eq!(frozen.effective_display_name(), "Chriz BG Collection");

        let created = created_for_frozen_recipe(&frozen, payload);
        validate_frozen_cli_recipe(&created, &frozen)
            .expect("schema-one frozen recipe remains resumable and reportable");

        for unsupported in [0, 3] {
            let mut rejected = frozen.clone();
            rejected.schema = unsupported;
            let error = validate_frozen_cli_recipe(&created, &rejected)
                .expect_err("unsupported frozen schema must fail closed");
            assert_eq!(error.code(), "resume_unavailable");
        }
    }

    const LEGACY_EET_GOLDENS: &str = "tests/fixtures/compat/legacy-eet";

    fn legacy_eet_golden(name: &str) -> Vec<u8> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(LEGACY_EET_GOLDENS)
            .join(name);
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
    }

    fn pretty_record<T: serde::Serialize>(value: &T) -> Vec<u8> {
        let mut bytes = serde_json::to_vec_pretty(value).expect("serialize record");
        bytes.push(b'\n');
        bytes
    }

    fn reserialized_pretty<T: serde::Serialize + serde::de::DeserializeOwned>(
        bytes: &[u8],
    ) -> Vec<u8> {
        pretty_record(&serde_json::from_slice::<T>(bytes).expect("read legacy record"))
    }

    /// The records an Enhanced Edition Trilogy installation of the fixture recipe writes, built
    /// the same way the goldens in `tests/fixtures/compat/legacy-eet` were produced by the engine
    /// before standalone games existed (commit 5106b53). Those goldens are never regenerated
    /// from current code: they are the compatibility contract.
    fn legacy_eet_records() -> Vec<(&'static str, Vec<u8>)> {
        use crate::manifest::GameRoot;
        use crate::receipt::{
            FinalLogReceipt, FinalReceiptState, InstallReceipt, LogComponentReceipt,
            ReceiptOutcome, ReceiptVersions, RECEIPT_SCHEMA_VERSION,
        };
        use crate::registry::{ManagedInstallRecord, REGISTRY_SCHEMA_VERSION};
        use crate::session::{LedgerRecord, SessionEvent};

        let frozen = frozen_recipe_fixture();
        let payload = super::serialize_frozen_recipe(&frozen).expect("freeze fixture recipe");
        let created = created_for_frozen_recipe(&frozen, payload.clone());
        let ledger = pretty_record(&LedgerRecord {
            sequence: 0,
            previous_sha256: "0".repeat(64),
            recorded_at: 1_761_400_000_000,
            event: SessionEvent::Created(Box::new(created.clone())),
        });
        let review = serde_json::to_vec(&super::install_review_identity(
            &created,
            frozen.effective_display_name(),
        ))
        .expect("serialize review identity");
        let engine_name = "Chriz Easy BG - abc123def456".to_owned();
        let save_root = PathBuf::from(r"C:\Users\Player\Documents\Chriz Easy BG - abc123def456");
        let launch_path = created.staged_bg2.join("Baldur.exe");
        let component = |component| LogComponentReceipt {
            tp2: "ee_fixpack/ee_fixpack.tp2".to_owned(),
            language: 0,
            component,
        };
        let receipt = pretty_record(&InstallReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            install_id: created.install_id.clone(),
            attempt_id: created.attempt_id.clone(),
            evidence_attempt_id: created.attempt_id.clone(),
            managed_root: created.managed_root.clone(),
            staged_bg1: created.staged_bg1.clone(),
            staged_bg2: created.staged_bg2.clone(),
            started_at_millis: 1_761_400_000_000,
            completed_at_millis: 1_761_400_900_000,
            outcome: ReceiptOutcome::Succeeded,
            versions: ReceiptVersions {
                application: "0.1.0-alpha.20".to_owned(),
                engine: "0.1.0".to_owned(),
                manifest_schema: frozen.collection.schema,
                recipe: "0.1.0-alpha.20".to_owned(),
            },
            source_games: frozen
                .sources()
                .into_iter()
                .map(FrozenSource::receipt)
                .collect(),
            recipe_payload_sha256: created.recipe_payload_sha256.clone(),
            recipe_envelope_sha256: created.recipe_envelope_sha256.clone(),
            selection_sha256: created.selection_sha256.clone(),
            normalized_selection: created.normalized_selection.clone(),
            plan_sha256: created.plan_sha256.clone(),
            plan: frozen.plan.clone(),
            artifacts: Vec::new(),
            weidu_tools: Vec::new(),
            runs: Vec::new(),
            final_state: Some(FinalReceiptState {
                logs: vec![
                    FinalLogReceipt {
                        target: GameRoot::Bg1,
                        sha256: "33".repeat(32),
                        components: vec![component(0)],
                    },
                    FinalLogReceipt {
                        target: GameRoot::Bg2,
                        sha256: "44".repeat(32),
                        components: vec![component(0), component(1)],
                    },
                ],
                bg1_engine_name: Some(engine_name.clone()),
                bg2_engine_name: engine_name.clone(),
                managed_save_root: save_root.clone(),
                launch_path: launch_path.clone(),
                verification_summary: "exact frozen stack matched across 2 planned WeiDU runs"
                    .to_owned(),
            }),
        });
        let registry = pretty_record(&ManagedInstallRecord {
            schema_version: REGISTRY_SCHEMA_VERSION,
            install_id: created.install_id.clone(),
            display_name: "Chriz Easy BG".to_owned(),
            managed_root: created.managed_root.clone(),
            recipe_version: "0.1.0-alpha.20".to_owned(),
            recipe_sha256: created.recipe_payload_sha256.clone(),
            engine_name,
            managed_save_root: save_root,
            launch_path,
            receipt_sha256: "55".repeat(32),
            completed_at_millis: 1_761_400_900_000,
            recipe_id: None,
        });
        // The ledger fixture above records a literal envelope; the production envelope of the
        // same payload has its own golden.
        let envelope =
            super::frozen_recipe_envelope(&sha256_bytes(&payload)).expect("build envelope");
        vec![
            ("recipe-envelope.json", envelope),
            ("frozen-payload.json", payload),
            ("ledger-record-0.json", ledger),
            ("review-identity.json", review),
            ("install-receipt.json", receipt),
            ("registry-record.json", registry),
        ]
    }

    #[test]
    fn eet_installations_still_write_their_legacy_bytes_exactly() {
        for (name, current) in legacy_eet_records() {
            let golden = legacy_eet_golden(name);
            assert!(
                current == golden,
                "{name} changed for an Enhanced Edition Trilogy installation:\n--- legacy\n{}\n--- current\n{}",
                String::from_utf8_lossy(&golden),
                String::from_utf8_lossy(&current)
            );
        }
    }

    #[test]
    fn legacy_eet_records_read_back_and_reserialize_byte_for_byte() {
        use crate::receipt::InstallReceipt;
        use crate::registry::ManagedInstallRecord;
        use crate::session::{LedgerRecord, SessionEvent};

        let payload = legacy_eet_golden("frozen-payload.json");
        let frozen: FrozenCliRecipe =
            serde_json::from_slice(&payload).expect("read legacy frozen payload");
        assert!(frozen.bg1.is_some());
        assert!(frozen.recipe_id.is_none());
        assert_eq!(frozen.collection.game, crate::manifest::CollectionGame::Eet);
        assert_eq!(serde_json::to_vec(&frozen).unwrap(), payload);

        let ledger = legacy_eet_golden("ledger-record-0.json");
        assert_eq!(reserialized_pretty::<LedgerRecord>(&ledger), ledger);
        let record: LedgerRecord = serde_json::from_slice(&ledger).unwrap();
        let SessionEvent::Created(created) = record.event else {
            panic!("legacy ledger record 0 is not a campaign creation");
        };
        assert!(!created.is_standalone());
        assert_eq!(created.staged_bg1, Some(created.managed_root.join("bg1")));
        validate_frozen_cli_recipe(&created, &frozen).expect("legacy campaign stays resumable");

        let review = legacy_eet_golden("review-identity.json");
        let identity: super::InstallReviewIdentity = serde_json::from_slice(&review).unwrap();
        assert_eq!(serde_json::to_vec(&identity).unwrap(), review);

        let receipt = legacy_eet_golden("install-receipt.json");
        assert_eq!(reserialized_pretty::<InstallReceipt>(&receipt), receipt);
        let registry = legacy_eet_golden("registry-record.json");
        assert_eq!(
            reserialized_pretty::<ManagedInstallRecord>(&registry),
            registry
        );
    }

    /// The shipped recipe freezes without any standalone-only key, so its payload digest is
    /// unchanged by standalone support.
    #[test]
    fn shipped_eet_recipe_freezes_without_standalone_keys() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../recipes/curated-full-current");
        let manifest = Manifest::load(&root).expect("load the shipped recipe");
        assert_eq!(
            manifest.collection.game,
            crate::manifest::CollectionGame::Eet
        );
        let choices = selection_choices(
            &manifest,
            "chris-recommended",
            "windows",
            &SelectionOverrides::default(),
        )
        .expect("select the shipped preset");
        let evaluation = evaluate(
            &manifest,
            &Selection {
                platform: "windows".to_owned(),
                choices,
            },
        )
        .expect("evaluate the shipped preset");
        let mut frozen = frozen_recipe_fixture();
        frozen.collection = manifest.collection.clone();
        frozen.artifacts = manifest.artifacts.clone();
        frozen.mods = manifest.mods.clone();
        frozen.presets = manifest.presets.clone();
        frozen.profiles = super::load_profiles(&manifest.root).expect("load shipped profiles");
        frozen.plan = evaluation.plan;
        frozen.normalized_selection = evaluation.normalized_selection;
        frozen.preset = "chris-recommended".to_owned();

        let payload = super::serialize_frozen_recipe(&frozen).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert!(value.get("recipe_id").is_none());
        assert!(value["collection"].get("game").is_none());
        assert!(value["bg1"].is_object());
        let text = String::from_utf8(payload.clone()).unwrap();
        assert!(!text.contains("\"standalone_role\""));
        assert!(!text.contains("\"iwdee\""));
        let reread: FrozenCliRecipe = serde_json::from_slice(&payload).unwrap();
        assert_eq!(reread, frozen);
        assert_eq!(serde_json::to_vec(&reread).unwrap(), payload);
    }

    fn standalone_frozen_recipe() -> FrozenCliRecipe {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/standalone-iwdee");
        let manifest = Manifest::load(&root).expect("load standalone recipe fixture");
        let choices = selection_choices(
            &manifest,
            "recommended",
            "windows",
            &SelectionOverrides::default(),
        )
        .expect("select standalone fixture");
        let evaluation = evaluate(
            &manifest,
            &Selection {
                platform: "windows".to_owned(),
                choices,
            },
        )
        .expect("evaluate standalone fixture");
        FrozenCliRecipe {
            release_version: Some("0.1.0".to_owned()),
            manual_checkpoints: Vec::new(),
            application_version: None,
            schema: CLI_FROZEN_RECIPE_SCHEMA,
            display_name: Some("Chriz Easy IWD".to_owned()),
            collection: manifest.collection,
            artifacts: manifest.artifacts,
            mods: manifest.mods,
            presets: manifest.presets,
            profiles: GameProfiles::load(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/games/iwdee-profiles"),
            )
            .expect("load IWD:EE profile fixture"),
            plan: evaluation.plan,
            normalized_selection: evaluation.normalized_selection,
            preset: "recommended".to_owned(),
            bg1: None,
            bg2: FrozenSource {
                role: GameRole::Iwdee,
                storefront: Storefront::Steam,
                root: PathBuf::from(r"C:\Legacy IWD"),
                build: Some("2.7.3.0".to_owned()),
                fingerprint: "33".repeat(32),
            },
            recipe_id: Some("chriz-easy-iwd".to_owned()),
        }
    }

    fn standalone_created(frozen: &FrozenCliRecipe) -> CampaignCreated {
        let mut created =
            created_for_frozen_recipe(frozen, super::serialize_frozen_recipe(frozen).unwrap());
        created.source_games.standalone_role = Some(GameRole::Iwdee);
        created
    }

    #[test]
    fn standalone_frozen_recipe_round_trips_and_validates() {
        let frozen = standalone_frozen_recipe();
        let payload = super::serialize_frozen_recipe(&frozen).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert!(value.get("bg1").is_none());
        assert_eq!(value["recipe_id"], "chriz-easy-iwd");
        assert_eq!(value["collection"]["game"], "iwdee");
        assert_eq!(value["bg2"]["role"], "iwdee");
        let reread: FrozenCliRecipe = serde_json::from_slice(&payload).unwrap();
        assert_eq!(reread, frozen);
        assert_eq!(reread.targets(), vec![crate::manifest::GameRoot::Bg2]);
        assert_eq!(reread.sources().len(), 1);

        let created = standalone_created(&frozen);
        assert!(created.staged_bg1.is_none());
        assert!(created.source_games.bg1.is_none());
        validate_frozen_cli_recipe(&created, &frozen).expect("standalone campaign validates");
    }

    #[test]
    fn standalone_frozen_recipe_rejects_mixed_source_identities() {
        let frozen = standalone_frozen_recipe();
        let created = standalone_created(&frozen);

        let mut eet_campaign = created.clone();
        eet_campaign.source_games.standalone_role = None;
        let mut with_bg1 = frozen.clone();
        with_bg1.bg1 = frozen_recipe_fixture().bg1;
        let mut with_bg1_campaign = created.clone();
        with_bg1_campaign.source_games.bg1 = Some("11".repeat(32));
        let mut wrong_role = frozen.clone();
        wrong_role.bg2.role = GameRole::Bg2ee;

        for (case, campaign, recipe) in [
            (
                "campaign without the standalone role",
                &eet_campaign,
                &frozen,
            ),
            ("recipe with a BG1 source", &with_bg1_campaign, &with_bg1),
            ("source with a Baldur's Gate role", &created, &wrong_role),
        ] {
            let error = validate_frozen_cli_recipe(campaign, recipe)
                .expect_err(&format!("{case} must not resume"));
            assert_eq!(error.code(), "resume_unavailable", "{case}");
        }

        let eet = frozen_recipe_fixture();
        let eet_created =
            created_for_frozen_recipe(&eet, super::serialize_frozen_recipe(&eet).unwrap());
        let mut eet_with_id = eet.clone();
        eet_with_id.recipe_id = Some("chriz-easy-bg".to_owned());
        let error = validate_frozen_cli_recipe(&eet_created, &eet_with_id)
            .expect_err("an EET recipe never carries a recipe id");
        assert!(error.message.contains("recipe id"), "{error:?}");
    }

    #[test]
    fn launch_falls_back_to_the_frozen_game_executable() {
        assert_eq!(
            frozen_recipe_fixture().fallback_executable().unwrap(),
            "Baldur.exe"
        );
        let frozen = standalone_frozen_recipe();
        assert_eq!(frozen.fallback_executable().unwrap(), "icewind.exe");

        let mut unknown_build = frozen.clone();
        unknown_build.bg2.build = Some("2.6.6.0".to_owned());
        let error = unknown_build.fallback_executable().unwrap_err();
        assert!(error.contains("Icewind Dale: Enhanced Edition"), "{error}");
        assert!(error.contains("2.6.6.0"), "{error}");
    }

    #[test]
    fn application_data_directory_defaults_and_accepts_only_plain_folder_names() {
        // The override is process-global, so library tests only read the default here; the CLI
        // end-to-end test sets it through `--app-data-directory` in its own process.
        assert_eq!(super::application_data_directory(), "Chriz BG Collection");
        for accepted in [
            "Chriz Easy IWD",
            "Chriz BG Collection",
            "IWD.v1",
            "COMX",
            "CON1",
            "A\u{20ac}",
            // Only superscript one to three are reserved, and only right after COM or LPT.
            "COM\u{2074}",
            "COM\u{b9}x",
            "\u{b9}COM",
            "Ich\u{b9}",
            "Eisw\u{fc}ste",
        ] {
            super::validate_application_data_directory(accepted)
                .unwrap_or_else(|error| panic!("{accepted:?} rejected: {error:?}"));
        }
        for rejected in [
            "",
            " ",
            ".",
            "..",
            "trailing.",
            "trailing ",
            " leading",
            r"nested\folder",
            "nested/folder",
            "drive:",
            "CON",
            "nul.txt",
            "COM1",
            "lpt9",
            "COM0",
            "LPT0.dat",
            // Windows also reserves COM and LPT with superscript digits one to three.
            "COM\u{b9}",
            "com\u{b2}",
            "COM\u{b3}.txt",
            "LPT\u{b9}",
            "lpt\u{b2}.log",
            "LPT\u{b3} ",
            "CONIN$",
            "conout$",
            "CONIN$.txt",
            "tab\there",
            &"x".repeat(65),
        ] {
            let error = super::validate_application_data_directory(rejected)
                .expect_err(&format!("{rejected:?} must be rejected"));
            assert_eq!(error.code(), "application_data_unavailable", "{rejected:?}");
        }
    }

    /// A deleted download cache is created again only for a campaign that this request may
    /// resume, only where its recorded path still leads to itself, and only where the campaign
    /// checks accept a cache, also next to source games that moved behind a link. Refusals
    /// create nothing, also not where a link points.
    #[cfg(windows)]
    #[test]
    fn a_deleted_download_cache_is_created_again_only_where_the_campaign_may_use_it() {
        use crate::session::{SessionEvent, SessionStore};

        let link = |link: &Path, target: &Path| {
            let output = std::process::Command::new("cmd.exe")
                .args(["/c", "mklink", "/J"])
                .arg(link)
                .arg(target)
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
        };
        for case in [
            "valid",
            "linked-parent",
            "linked-ancestor",
            "other-install",
            "inside-source",
            "inside-managed",
            "linked-source",
            "source-behind-link",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let root = std::fs::canonicalize(temp.path()).unwrap();
            let managed = root.join("managed");
            let elsewhere = root.join("elsewhere");
            let mut frozen = frozen_recipe_fixture();
            frozen.bg1.as_mut().unwrap().root = match case {
                "linked-source" => root.join("old-bg1"),
                "source-behind-link" => root.join("games").join("bg1"),
                _ => root.join("bg1-source"),
            };
            frozen.bg2.root = root.join("bg2-source");
            let parent = match case {
                "inside-source" => frozen.bg1.as_ref().unwrap().root.clone(),
                "inside-managed" => managed.clone(),
                "linked-ancestor" => root.join("downloads").join("nested"),
                "source-behind-link" => root.join("downloads").join("bg1"),
                _ => root.join("downloads"),
            };
            std::fs::create_dir_all(&managed).unwrap();
            std::fs::create_dir_all(&frozen.bg2.root).unwrap();
            if !matches!(case, "linked-source" | "source-behind-link") {
                std::fs::create_dir_all(&frozen.bg1.as_ref().unwrap().root).unwrap();
            }
            std::fs::create_dir_all(parent.join("cache")).unwrap();
            let mut created =
                created_for_frozen_recipe(&frozen, serde_json::to_vec(&frozen).unwrap());
            created.managed_root = managed.clone();
            created.cache_root = parent.join("cache");
            created.staged_bg1 = Some(managed.join("bg1"));
            created.staged_bg2 = managed.join("game");
            let store =
                SessionStore::create(&managed, SessionEvent::Created(Box::new(created.clone())))
                    .unwrap();
            // The cache is deleted; this test moves it away instead.
            std::fs::rename(&created.cache_root, root.join("deleted-cache")).unwrap();
            match case {
                "linked-parent" | "linked-ancestor" => {
                    // A folder on the cache's path is replaced with a link that leads elsewhere.
                    std::fs::create_dir_all(elsewhere.join("nested")).unwrap();
                    std::fs::rename(root.join("downloads"), root.join("old-downloads")).unwrap();
                    link(&root.join("downloads"), &elsewhere);
                }
                // The BG1 game moved to the folder that held the cache, and its recorded folder
                // now links there.
                "linked-source" => {
                    link(&frozen.bg1.as_ref().unwrap().root, &root.join("downloads"))
                }
                // A folder above the BG1 game links to the folder that holds it and the cache.
                "source-behind-link" => link(&root.join("games"), &root.join("downloads")),
                _ => {}
            }
            let expected_install_id = if case == "other-install" {
                "install-other"
            } else {
                created.install_id.as_str()
            };

            let result =
                super::recreate_deleted_cache_root(&store, &managed, Some(expected_install_id));

            if case == "valid" {
                result.expect("create the deleted cache again");
                assert!(created.cache_root.is_dir());
                continue;
            }
            let error = result.expect_err(case);
            let code = match case {
                "other-install" => "resume_identity_mismatch",
                "linked-source" => "game_inspection_failed",
                _ => "resume_unavailable",
            };
            assert_eq!(error.code(), code, "{case}: {error}");
            assert!(!created.cache_root.exists(), "{case}");
            assert!(!elsewhere.join("cache").exists(), "{case}");
            assert!(!elsewhere.join("nested").join("cache").exists(), "{case}");
        }
    }

    #[test]
    fn schema_two_frozen_recipe_rejects_a_missing_display_name() {
        let current = frozen_recipe_fixture();
        let mut missing = serde_json::to_value(&current).expect("serialize current frozen recipe");
        missing
            .as_object_mut()
            .expect("frozen recipe is an object")
            .remove("display_name");
        let payload = serde_json::to_vec(&missing).expect("serialize incomplete schema-two recipe");
        let frozen: FrozenCliRecipe =
            serde_json::from_slice(&payload).expect("parse incomplete schema-two recipe");
        let created = created_for_frozen_recipe(&frozen, payload);

        let error = validate_frozen_cli_recipe(&created, &frozen)
            .expect_err("schema two must require an explicit display name");

        assert_eq!(error.code(), "resume_unavailable");
        assert!(error.to_string().contains("display name"));
    }

    #[test]
    fn identical_later_acquisition_is_not_a_reacquisition() {
        for outcome in [
            ArtifactCacheOutcome::Downloaded,
            ArtifactCacheOutcome::Hit,
            ArtifactCacheOutcome::Manual,
        ] {
            let first = ArtifactReceipt {
                cache_outcome: outcome,
                ..downloaded_artifact_receipt()
            };

            assert_eq!(
                retain_first_acquisition_receipt(&first, &first.clone()).unwrap(),
                LaterAcquisition::Identical
            );
        }
    }

    #[test]
    fn cache_hit_on_the_first_download_is_routine_reuse() {
        let first = downloaded_artifact_receipt();
        let reused = ArtifactReceipt {
            cache_outcome: ArtifactCacheOutcome::Hit,
            ..first.clone()
        };

        assert_eq!(
            retain_first_acquisition_receipt(&first, &reused).unwrap(),
            LaterAcquisition::Identical
        );
    }

    #[test]
    fn same_archive_from_the_same_source_is_accepted_whatever_its_final_url_or_cache_outcome() {
        use ArtifactCacheOutcome::{Downloaded, Hit};
        let signed = "https://release-assets.githubusercontent.invalid/test-mod.zip?sig=2";
        for (first_outcome, later_outcome, later_final_url) in [
            // Cache lost and re-downloaded through a new single-use signed redirect.
            (Downloaded, Downloaded, Some(signed)),
            (Hit, Downloaded, None),
            (Hit, Downloaded, Some(signed)),
            // Cache entry re-populated through another redirect, then reused.
            (Downloaded, Hit, Some(signed)),
            (Hit, Hit, Some(signed)),
        ] {
            let first = ArtifactReceipt {
                cache_outcome: first_outcome,
                ..downloaded_artifact_receipt()
            };
            let mut later = first.clone();
            later.cache_outcome = later_outcome;
            if let Some(final_url) = later_final_url {
                later.final_url = final_url.to_owned();
            }

            assert_eq!(
                retain_first_acquisition_receipt(&first, &later).unwrap_or_else(|error| panic!(
                    "{first_outcome:?} -> {later_outcome:?} via {later_final_url:?}: {error}"
                )),
                LaterAcquisition::Reacquired,
                "{first_outcome:?} -> {later_outcome:?} via {later_final_url:?}"
            );
        }
    }

    #[test]
    fn later_acquisition_cannot_change_archive_identity_source_or_acquisition_policy() {
        type Change = fn(&mut ArtifactReceipt);
        let changes: [(&str, Change); 6] = [
            ("sha256", |receipt| receipt.sha256 = "cd".repeat(32)),
            ("length", |receipt| receipt.length += 1),
            ("original_url", |receipt| {
                receipt.original_url = "https://example.invalid/other-test-mod.zip".to_owned();
            }),
            ("version", |receipt| receipt.version = "1.1".to_owned()),
            ("id", |receipt| receipt.id = "other-mod".to_owned()),
            ("final_url and sha256", |receipt| {
                receipt.final_url = "https://other.invalid/test-mod.zip".to_owned();
                receipt.sha256 = "cd".repeat(32);
            }),
        ];
        for (outcome, later_outcome) in [
            (
                ArtifactCacheOutcome::Downloaded,
                ArtifactCacheOutcome::Downloaded,
            ),
            (ArtifactCacheOutcome::Downloaded, ArtifactCacheOutcome::Hit),
            (ArtifactCacheOutcome::Hit, ArtifactCacheOutcome::Downloaded),
            (ArtifactCacheOutcome::Manual, ArtifactCacheOutcome::Manual),
        ] {
            for (field, change) in changes {
                let first = ArtifactReceipt {
                    cache_outcome: outcome,
                    ..downloaded_artifact_receipt()
                };
                let mut changed = first.clone();
                changed.cache_outcome = later_outcome;
                change(&mut changed);

                let error = retain_first_acquisition_receipt(&first, &changed)
                    .expect_err(&format!("changed {field} must be refused"));

                assert_eq!(
                    error.to_string(),
                    "artifact \"test-mod\" differs from its first acquisition evidence",
                    "{field}"
                );
            }
        }
    }

    #[test]
    fn later_acquisition_cannot_switch_between_manual_import_and_download() {
        use ArtifactCacheOutcome::{Downloaded, Hit, Manual};
        for (first_outcome, later_outcome) in [
            (Manual, Downloaded),
            (Manual, Hit),
            (Downloaded, Manual),
            (Hit, Manual),
        ] {
            let first = ArtifactReceipt {
                cache_outcome: first_outcome,
                ..downloaded_artifact_receipt()
            };
            let mut later = first.clone();
            later.cache_outcome = later_outcome;

            let error = retain_first_acquisition_receipt(&first, &later)
                .expect_err("acquisition policy is frozen");

            assert!(
                error
                    .to_string()
                    .contains("differs from its first acquisition evidence"),
                "{first_outcome:?} -> {later_outcome:?}"
            );
        }
    }

    #[test]
    fn terminal_evidence_uses_stdout_only_for_a_marker_free_debug_stub() {
        let debug = b"WeiDU v 24900 Log\ncommand line only\n";
        let stdout = b"SUCCESSFULLY INSTALLED component 1\n";

        assert!(std::ptr::eq(
            select_terminal_evidence(debug, stdout),
            stdout
        ));
    }

    #[test]
    fn partial_debug_terminal_evidence_cannot_be_replaced_by_stdout() {
        let debug = b"SUCCESSFULLY INSTALLED component 1\n";
        let stdout = b"SUCCESSFULLY INSTALLED component 1\nSUCCESSFULLY INSTALLED component 2\n";

        assert!(std::ptr::eq(select_terminal_evidence(debug, stdout), debug));
    }

    #[test]
    fn non_prefix_modpack_completion_names_unrecorded_requested_components() {
        let requested = [
            110, 130, 140, 170, 190, 192, 193, 194, 195, 196, 197, 198, 410, 430, 440, 450,
        ];
        let installed = [
            110, 130, 140, 190, 193, 194, 195, 196, 197, 198, 410, 430, 440, 450,
        ];
        let before = "~EARLIER/SETUP-EARLIER.TP2~ #0 #1 // earlier\n";
        let mut after = before.to_owned();
        for component in installed {
            after.push_str(&format!(
                "~CHRIZ-BG-MODPACK/SETUP-CHRIZ-BG-MODPACK.TP2~ #0 #{component} // selected\n"
            ));
        }

        let detail = non_prefix_component_diagnostic(
            before,
            &after,
            "modpack-main",
            "chriz-bg-modpack",
            "chriz-bg-modpack/setup-chriz-bg-modpack.tp2",
            0,
            &requested,
        )
        .expect("ordered non-prefix completion should produce bounded context");

        assert!(detail.contains("run \"modpack-main\" (mod \"chriz-bg-modpack\")"));
        assert!(detail.contains("14 of 16 requested components"));
        assert!(detail.contains("170, 192"));
        assert!(detail.contains("not recorded as installed"));
        assert!(detail.contains("Missing rows do not identify the cause"));

        assert!(matches!(
            stack_disturbed_reconciliation(
                before,
                &after,
                "modpack-main",
                "chriz-bg-modpack",
                "chriz-bg-modpack/setup-chriz-bg-modpack.tp2",
                0,
                &requested,
            ),
            InstallReconciliation::FreshCopyRequired { reason }
                if reason.contains("170, 192")
        ));
    }

    fn diagnostic_log(before: &str, components: &[u32]) -> String {
        let mut after = before.to_owned();
        for component in components {
            after.push_str(&format!(
                "~SETUP-CHRIZ-BG-REBALANCE.TP2~ #0 #{component} // selected\n"
            ));
        }
        after
    }

    #[test]
    fn non_prefix_diagnostic_reports_missing_components_and_reordered_subset_separately() {
        let requested = [
            100, 101, 110, 111, 120, 121, 301, 302, 400, 401, 404, 405, 407, 408,
        ];
        let installed = [
            100, 101, 111, 120, 121, 301, 400, 401, 404, 405, 407, 408, 302,
        ];
        let before = "~EARLIER/SETUP-EARLIER.TP2~ #0 #1 // earlier\n";
        let after = diagnostic_log(before, &installed);

        let detail = non_prefix_component_diagnostic(
            before,
            &after,
            "chriz-bg-rebalance-bg2",
            "chriz-bg-rebalance",
            "setup-chriz-bg-rebalance.tp2",
            0,
            &requested,
        )
        .expect("missing component and reordered tail should both be explained");

        assert!(detail.contains("13 of 14 requested components"));
        assert!(detail.contains("Components 110 are not recorded as installed"));
        assert!(
            detail.contains("Previously installed components are still recorded in the same order")
        );
        assert!(detail.contains("Component order differs from the recipe"));
        assert!(
            detail.contains("[100, 101, 111, 120, 121, 301, 302, 400, 401, 404, 405, 407, 408]")
        );
        assert!(
            detail.contains("[100, 101, 111, 120, 121, 301, 400, 401, 404, 405, 407, 408, 302]")
        );
        assert!(matches!(
            stack_disturbed_reconciliation(
                before,
                &after,
                "chriz-bg-rebalance-bg2",
                "chriz-bg-rebalance",
                "setup-chriz-bg-rebalance.tp2",
                0,
                &requested,
            ),
            InstallReconciliation::FreshCopyRequired { reason }
                if reason.contains("Components 110")
                    && reason.contains("Component order differs")
                    && !reason.contains("changed outside")
        ));
    }

    #[test]
    fn non_prefix_diagnostic_explains_complete_but_reordered_run_without_permitting_resume() {
        use crate::weidu::verify::{reconcile, ExpectedRun, Reconciliation};

        let requested = [301, 302, 400];
        let before = "~EARLIER/SETUP-EARLIER.TP2~ #0 #1 // earlier\n";
        let after = diagnostic_log(before, &[301, 400, 302]);
        let detail = non_prefix_component_diagnostic(
            before,
            &after,
            "rebalance",
            "chriz-bg-rebalance",
            "setup-chriz-bg-rebalance.tp2",
            0,
            &requested,
        )
        .expect("full but reordered result should explain the order discrepancy");

        assert!(detail.contains("3 of 3 requested components"));
        assert!(detail.contains("Component order differs from the recipe"));
        assert!(!detail.contains("not recorded as installed"));
        let debug = "SUCCESSFULLY INSTALLED component\n".repeat(3);
        assert_eq!(
            reconcile(
                before,
                &after,
                &debug,
                &ExpectedRun {
                    tp2: "setup-chriz-bg-rebalance.tp2".into(),
                    language: 0,
                    components: requested.into(),
                    exit_code: 0,
                },
            ),
            Reconciliation::StackDisturbed,
        );
        assert!(matches!(
            stack_disturbed_reconciliation(
                before,
                &after,
                "rebalance",
                "chriz-bg-rebalance",
                "setup-chriz-bg-rebalance.tp2",
                0,
                &requested,
            ),
            InstallReconciliation::FreshCopyRequired { .. }
        ));
    }

    #[test]
    fn non_prefix_diagnostic_does_not_explain_untrusted_or_unchanged_results_as_partial_success() {
        let before = "~EARLIER/SETUP-EARLIER.TP2~ #0 #1 // earlier\n";
        let tail = diagnostic_log("", &[301, 400]);
        let changed_prefix = "~EARLIER/SETUP-EARLIER.TP2~ #0 #2 // changed\n";
        let earlier_same_component = diagnostic_log(before, &[301]);
        let cases = [
            (
                "empty result",
                before.to_owned(),
                before.to_owned(),
                vec![301, 302, 400],
            ),
            (
                "exact result",
                before.to_owned(),
                diagnostic_log(before, &[301, 302, 400]),
                vec![301, 302, 400],
            ),
            (
                "duplicate requested",
                before.to_owned(),
                diagnostic_log(before, &[301, 400]),
                vec![301, 302, 400, 301],
            ),
            (
                "duplicate recorded",
                before.to_owned(),
                diagnostic_log(before, &[301, 400, 301]),
                vec![301, 302, 400],
            ),
            (
                "duplicate earlier component",
                earlier_same_component.clone(),
                diagnostic_log(&earlier_same_component, &[301, 400]),
                vec![301, 302, 400],
            ),
            (
                "unrequested component",
                before.to_owned(),
                diagnostic_log(before, &[301, 999]),
                vec![301, 302, 400],
            ),
            (
                "changed earlier component",
                before.to_owned(),
                format!("{changed_prefix}{tail}"),
                vec![301, 302, 400],
            ),
            (
                "removed earlier component",
                before.to_owned(),
                tail.clone(),
                vec![301, 302, 400],
            ),
            (
                "wrong mod",
                before.to_owned(),
                format!(
                    "{before}{}",
                    tail.replace("SETUP-CHRIZ-BG-REBALANCE", "SETUP-OTHER")
                ),
                vec![301, 302, 400],
            ),
            (
                "wrong path",
                before.to_owned(),
                format!("{before}{}", tail.replace("~SETUP-", "~OTHER/SETUP-")),
                vec![301, 302, 400],
            ),
            (
                "wrong language",
                before.to_owned(),
                format!("{before}{}", tail.replace("#0", "#1")),
                vec![301, 302, 400],
            ),
            (
                "new uninstall",
                before.to_owned(),
                format!(
                    "{before}{tail}// Recently Uninstalled: ~EARLIER/SETUP-EARLIER.TP2~ #0 #1\n"
                ),
                vec![301, 302, 400],
            ),
            (
                "malformed result",
                before.to_owned(),
                format!("{before}{tail}~BROKEN\n"),
                vec![301, 302, 400],
            ),
            (
                "malformed before",
                "~BROKEN\n".to_owned(),
                format!("{before}{tail}"),
                vec![301, 302, 400],
            ),
            (
                "empty request",
                before.to_owned(),
                format!("{before}{tail}"),
                vec![],
            ),
        ];

        for (name, before, after, requested) in cases {
            assert!(
                non_prefix_component_diagnostic(
                    &before,
                    &after,
                    "rebalance",
                    "chriz-bg-rebalance",
                    "setup-chriz-bg-rebalance.tp2",
                    0,
                    &requested,
                )
                .is_none(),
                "unexpected bounded diagnosis for {name}"
            );
        }
    }

    #[test]
    fn unrelated_stack_addition_is_not_attributed_to_missing_requested_components() {
        let before = "~EARLIER/SETUP-EARLIER.TP2~ #0 #1 // earlier\n";
        let after = concat!(
            "~EARLIER/SETUP-EARLIER.TP2~ #0 #1 // earlier\n",
            "~OTHER/SETUP-OTHER.TP2~ #0 #7 // unrelated\n"
        );

        assert!(non_prefix_component_diagnostic(
            before,
            after,
            "modpack-main",
            "chriz-bg-modpack",
            "chriz-bg-modpack/setup-chriz-bg-modpack.tp2",
            0,
            &[110, 170],
        )
        .is_none());
        assert_eq!(
            stack_disturbed_reconciliation(
                before,
                after,
                "modpack-main",
                "chriz-bg-modpack",
                "chriz-bg-modpack/setup-chriz-bg-modpack.tp2",
                0,
                &[110, 170],
            ),
            InstallReconciliation::FreshCopyRequired {
                reason: "WeiDU.log changed outside the exact frozen component suffix".to_owned(),
            }
        );
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

    /// A one-entry ZIP holding `mod/setup-mod.tp2`, and a fetch-only artifact pinned to it.
    fn pinned_test_archive(frozen: &FrozenCliRecipe) -> (Vec<u8>, crate::manifest::Artifact) {
        use std::io::Write;

        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file(
                "mod/setup-mod.tp2",
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(b"BACKUP ~mod/backup~\n").unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let mut artifact = frozen.artifacts["eefixpack"].clone();
        artifact.acquisition = crate::manifest::AcquisitionPolicy::FetchOnly;
        artifact.source.url = "https://example.invalid/stop-test.zip".to_owned();
        artifact.source.expected_length = Some(bytes.len() as u64);
        artifact.source.sha256 = sha256_bytes(&bytes);
        artifact.archive.kind = crate::manifest::ArchiveKind::Zip;
        artifact.archive.root_rule = crate::manifest::ArchiveRootRule::Direct;
        artifact.archive.publish_roots = vec!["mod".to_owned()];
        artifact.archive.tp2_paths = vec!["mod/setup-mod.tp2".to_owned()];
        artifact.archive.limits = crate::manifest::ArchiveLimitsSpec {
            max_depth: 8,
            max_entries: 8,
            max_entry_uncompressed_bytes: 1024,
            max_total_uncompressed_bytes: 1024,
            max_compression_ratio: 100,
        };
        (bytes, artifact)
    }

    /// Stop ends a wait for a download or unpacked mod whose cache lock another process holds
    /// and never releases, for example because it hangs or is suspended. Without a stop, the
    /// wait continues, because the other process may still be working.
    #[test]
    fn stop_ends_waits_for_cache_entries_another_process_never_releases() {
        use fs4::fs_std::FileExt;

        use super::GuardedCliDependencies;
        use crate::acquire::ArtifactMetadata;
        use crate::events::ChannelSink;
        use crate::games::{Eligibility, GameCandidate};

        let temp = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(temp.path()).unwrap();
        let frozen = frozen_recipe_fixture();
        let (bytes, artifact) = pinned_test_archive(&frozen);
        let digest = sha256_bytes(&bytes);
        // The verified download, published the way a finished download publishes it.
        let downloads = root.join("cache/sha256").join(&digest[..2]);
        std::fs::create_dir_all(&downloads).unwrap();
        std::fs::write(downloads.join(format!("{digest}.archive")), &bytes).unwrap();
        let metadata = ArtifactMetadata {
            original_url: artifact.source.url.clone(),
            final_url: artifact.source.url.clone(),
            etag: None,
            last_modified: None,
            length: bytes.len() as u64,
            sha256: digest.clone(),
            completed_at_unix_seconds: 1,
        };
        std::fs::write(
            downloads.join(format!("{digest}.json")),
            serde_json::to_vec(&metadata).unwrap(),
        )
        .unwrap();
        let mut created = created_for_frozen_recipe(&frozen, Vec::new());
        created.cache_root = root.join("cache");
        created.managed_root = root.join("managed");
        std::fs::create_dir(&created.managed_root).unwrap();
        let candidate = GameCandidate {
            role: GameRole::Bg2ee,
            storefront: Storefront::Steam,
            root: root.join("game"),
            build: None,
            eligibility: Eligibility::Eligible,
            findings: vec![],
            fingerprint: None,
        };
        let (sink, _events) = ChannelSink::unbounded();
        let dependencies = |controls: &RunnerControlHandle| {
            GuardedCliDependencies::new(
                &created,
                &frozen,
                Some(candidate.clone()),
                candidate.clone(),
                root.join("app-data"),
                &sink,
                controls.clone(),
            )
        };
        let contract = dependencies(&RunnerControlHandle::new())
            .extract(&artifact)
            .expect("unpack the verified download once")
            .extraction_contract_sha256;

        for lock in [
            root.join("cache/locks").join(format!("{digest}.lock")),
            root.join("cache/extracted/locks")
                .join(format!("{digest}.{contract}.lock")),
        ] {
            let holder = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&lock)
                .unwrap();
            holder.lock_exclusive().unwrap();
            let controls = RunnerControlHandle::new();
            let worker = dependencies(&controls);
            std::thread::scope(|scope| {
                let waiting = scope.spawn(|| worker.extract(&artifact));
                std::thread::sleep(Duration::from_millis(400));
                assert!(
                    !waiting.is_finished(),
                    "{} must be waited for",
                    lock.display()
                );

                controls.cancel();
                let deadline = std::time::Instant::now() + Duration::from_secs(5);
                while !waiting.is_finished() && std::time::Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(20));
                }
                let stopped = waiting.is_finished();
                // Release the lock so that a wait that ignored Stop still ends this test.
                FileExt::unlock(&holder).unwrap();
                assert!(stopped, "Stop did not end the wait for {}", lock.display());
                let error = waiting.join().unwrap().unwrap_err().to_string();
                assert!(error.contains("stopped by the user"), "{error}");
            });
        }
        let extracted = root
            .join("cache/extracted/sha256-v3")
            .join(&digest[..2])
            .join(&digest)
            .join(&contract);
        assert!(extracted.join("mod/setup-mod.tp2").is_file());
        assert!(!root.join("cache/damaged").exists());
    }
}
