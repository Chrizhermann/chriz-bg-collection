//! Deterministic, presentation-neutral operations exposed by the headless installer CLI.
//!
//! Argument parsing stays in the binary. This module owns recipe evaluation, game inspection,
//! managed-state reporting, and diagnostics selection so the desktop bridge can reuse the same
//! behavior without invoking a subprocess.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::diagnostics::{export_diagnostics, DiagnosticsBundle, DiagnosticsRequest};
use crate::digest::{plan_digest, selection_digest, sha256_bytes};
use crate::events::{EngineEvent, EventSink};
use crate::games::{
    discover_installed_games, inspect_game_path, Eligibility, GameCandidate, GameProfiles,
    GameRole, Storefront, SystemFileSystem,
};
use crate::lock::LockError;
use crate::manifest::{AcquisitionPolicy, Artifact, Collection, ModFile, PresetFile};
use crate::orchestrator::{
    run_campaign, ArtifactAcquirer, ArtifactKind, ArtifactMaterializer, BuiltInvocation,
    CampaignClock, CampaignOutcome, CampaignPreflight, CampaignRequest, InstallLogVerifier,
    InstallReconciliation, InvocationBuilder, MaterializationOutcome, MaterializationTask,
    MutationCheck, ProcessResult, ProcessRunner, ReceiptDraft, ReceiptWriter, StagingService,
    StepAttempt, StepFailure,
};
use crate::preflight::{initial_preflight, InitialPreflight, RequiredInput, SpaceRequirement};
use crate::receipt::{
    InstallReceipt, ManagedReceiptWriter, ReceiptEvidence, ReceiptOutcome, ReceiptStore,
    ReceiptVersions, SourceGameReceipt, RECEIPT_SCHEMA_VERSION,
};
use crate::recipe_view::{evaluate, SelectionEvaluation};
use crate::registry::ManagedInstallRegistry;
use crate::resolve::{InstallPlan, PlannedRun, Selection};
use crate::session::{
    CampaignCreated, FrozenIdentity, SessionReplay, SessionStore, SourceGameFingerprints,
};
use crate::validate::{self, Severity};
use crate::Manifest;

const GAME_PROFILE_DIRECTORY: &str = "game-builds";
const STATE_DIRECTORY: &str = ".chriz";
const ATTEMPTS_DIRECTORY: &str = "attempts";
const ATTEMPT_RECEIPT_FILE: &str = "receipt.json";
const CLI_FROZEN_RECIPE_SCHEMA: u32 = 1;
const APPLICATION_DATA_DIRECTORY: &str = "Chriz BG Collection";
const LOCKS_DIRECTORY: &str = "locks";
static CAMPAIGN_SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
/// The immutable identities are frozen from authored artifact metadata. Execution remains
/// fail-closed at required-input preflight until the production materialization slice lands.
pub fn install_campaign<S: EventSink>(
    request: &InstallCommandRequest,
    sink: &S,
) -> Result<CampaignReport, CliError> {
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
    execute_frozen_campaign(created, frozen, sink)
}

/// Resume only the campaign identity and recipe payload frozen below `managed_root`.
pub fn resume_campaign<S: EventSink>(
    managed_root: &Path,
    sink: &S,
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
    let frozen: FrozenCliRecipe =
        serde_json::from_slice(&created.recipe_payload).map_err(|error| {
            CliError::new(
                "resume_unavailable",
                format!("frozen recipe payload is not a supported CLI snapshot: {error}"),
            )
        })?;
    validate_frozen_cli_recipe(&created, &frozen)?;
    execute_frozen_campaign(created, frozen, sink)
}

fn execute_frozen_campaign<S: EventSink>(
    created: CampaignCreated,
    frozen: FrozenCliRecipe,
    sink: &S,
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
    let mut dependencies =
        GuardedCliDependencies::new(&created, &frozen, current_bg1, current_bg2, app_data, sink);
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
    }
    if succeeded && receipt.runs.len() != receipt.plan.runs.len() {
        return Err("successful receipt has incomplete run evidence".to_owned());
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
        version: artifact.version.clone(),
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
}

impl<'a, S: EventSink> GuardedCliDependencies<'a, S> {
    fn new(
        created: &'a CampaignCreated,
        frozen: &'a FrozenCliRecipe,
        current_bg1: GameCandidate,
        current_bg2: GameCandidate,
        app_data: PathBuf,
        sink: &'a S,
    ) -> Self {
        Self {
            created,
            frozen,
            current_bg1,
            current_bg2,
            app_data,
            sink,
        }
    }

    fn required_inputs(identities: &[FrozenIdentity]) -> Vec<RequiredInput> {
        identities
            .iter()
            .map(|identity| RequiredInput {
                id: identity.id.clone(),
                version: identity.version.clone(),
                sha256: identity.sha256.clone(),
                length: identity.length,
                obtainable: false,
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
}

impl<S: EventSink> CampaignPreflight for GuardedCliDependencies<'_, S> {
    fn initial(
        &mut self,
        request: &CampaignRequest,
        replay: &SessionReplay,
    ) -> Result<(), StepFailure> {
        Self::ensure_source_fresh(&self.current_bg1, &self.frozen.bg1, "BGEE+SoD")?;
        Self::ensure_source_fresh(&self.current_bg2, &self.frozen.bg2, "BG2EE")?;
        let required_artifacts = Self::required_inputs(&request.created.artifact_identities);
        let required_tools = Self::required_inputs(&request.created.tool_identities);
        initial_preflight(&InitialPreflight {
            bg1_source: &self.current_bg1,
            bg2_source: &self.current_bg2,
            destination: &request.created.managed_root,
            space: SpaceRequirement::default(),
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
        Err(StepFailure::new(format!(
            "the frozen recipe has no executable mutation metadata for {} ({})",
            check.step_id,
            check.kind.as_str()
        )))
    }
}

impl<S: EventSink> ArtifactAcquirer for GuardedCliDependencies<'_, S> {
    fn acquire(
        &mut self,
        identity: &FrozenIdentity,
        _kind: ArtifactKind,
    ) -> Result<(), StepFailure> {
        if let Some(artifact) = self.frozen.artifacts.get(&identity.id) {
            if artifact.acquisition == AcquisitionPolicy::ManualUserSupplied {
                let drop_dir = self.created.cache_root.join("manual");
                self.sink.emit(EngineEvent::ManualDownloadNeeded {
                    mod_id: artifact.id.clone(),
                    page: artifact.source.url.clone(),
                    expected_sha256: artifact.source.sha256.clone(),
                    drop_dir: drop_dir.display().to_string(),
                });
            }
        }
        Err(StepFailure::new(format!(
            "required artifact {:?} has no signed expected length/publication contract",
            identity.id
        )))
    }
}

impl<S: EventSink> StagingService for GuardedCliDependencies<'_, S> {
    fn stage(&mut self, role: GameRole) -> Result<(), StepFailure> {
        Err(StepFailure::new(format!(
            "staging {role:?} is unavailable before executable artifact metadata is frozen"
        )))
    }

    fn finalize_identity(&mut self) -> Result<(), StepFailure> {
        Err(StepFailure::new(
            "final identity is unavailable before a complete staged build",
        ))
    }
}

impl<S: EventSink> ArtifactMaterializer for GuardedCliDependencies<'_, S> {
    fn materialize(
        &mut self,
        task: &MaterializationTask,
    ) -> Result<MaterializationOutcome, StepFailure> {
        Err(StepFailure::new(format!(
            "materialization metadata is unavailable for {:?}",
            task.artifact_id
        )))
    }
}

impl<S: EventSink> InvocationBuilder for GuardedCliDependencies<'_, S> {
    type Invocation = ();

    fn build(
        &mut self,
        run: &PlannedRun,
        _components: &[u32],
        _attempt: &StepAttempt,
    ) -> Result<BuiltInvocation<Self::Invocation>, StepFailure> {
        Err(StepFailure::new(format!(
            "WeiDU invocation metadata is unavailable for {:?}",
            run.run_id
        )))
    }
}

impl<S: EventSink> ProcessRunner<()> for GuardedCliDependencies<'_, S> {
    fn run(
        &mut self,
        _invocation: (),
        _attempt: &StepAttempt,
    ) -> Result<ProcessResult, StepFailure> {
        Err(StepFailure::new(
            "no process may start without a verified invocation",
        ))
    }
}

impl<S: EventSink> InstallLogVerifier for GuardedCliDependencies<'_, S> {
    fn recover(
        &mut self,
        _run: &PlannedRun,
        _attempt: &StepAttempt,
    ) -> Result<InstallReconciliation, StepFailure> {
        Err(StepFailure::new(
            "no WeiDU evidence exists before executable recipe metadata",
        ))
    }

    fn snapshot_before(
        &mut self,
        _run: &PlannedRun,
        _attempt: &StepAttempt,
    ) -> Result<(), StepFailure> {
        Err(StepFailure::new(
            "cannot snapshot an unstaged WeiDU installation",
        ))
    }

    fn record_invocation(
        &mut self,
        _run: &PlannedRun,
        _attempt: &StepAttempt,
        _identity_digest: &str,
    ) -> Result<(), StepFailure> {
        Err(StepFailure::new(
            "cannot record an unavailable WeiDU invocation",
        ))
    }

    fn sync_and_reconcile(
        &mut self,
        _run: &PlannedRun,
        _attempt: &StepAttempt,
        _result: ProcessResult,
    ) -> Result<InstallReconciliation, StepFailure> {
        Err(StepFailure::new(
            "cannot reconcile an unavailable WeiDU invocation",
        ))
    }

    fn verify_final(&mut self, _plan: &InstallPlan) -> Result<(), StepFailure> {
        Err(StepFailure::new(
            "final verification requires a complete executable recipe",
        ))
    }
}

impl<S: EventSink> ReceiptWriter for GuardedCliDependencies<'_, S> {
    fn write(&mut self, draft: &ReceiptDraft) -> Result<(), StepFailure> {
        let store = ReceiptStore::open(&self.created.managed_root, &self.created.install_id)
            .map_err(|error| StepFailure::new(error.to_string()))?;
        let registry = ManagedInstallRegistry::open_or_create(&self.app_data)
            .map_err(|error| StepFailure::new(error.to_string()))?;
        let evidence = ReceiptEvidence {
            versions: ReceiptVersions {
                application: env!("CARGO_PKG_VERSION").to_owned(),
                engine: env!("CARGO_PKG_VERSION").to_owned(),
                manifest_schema: self.frozen.collection.schema,
                recipe: format!("local-{}", &self.created.recipe_payload_sha256[..12]),
            },
            source_games: vec![self.frozen.bg1.receipt(), self.frozen.bg2.receipt()],
            artifacts: Vec::new(),
            weidu_tools: Vec::new(),
            runs: Vec::new(),
            final_state: None,
        };
        ManagedReceiptWriter::new(store, registry, "Chriz BG Collection".to_owned(), evidence)
            .write(draft)
    }
}

impl<S: EventSink> CampaignClock for GuardedCliDependencies<'_, S> {
    fn now_millis(&mut self) -> Result<u64, StepFailure> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| StepFailure::new("system clock is earlier than the Unix epoch"))?
            .as_millis();
        u64::try_from(millis).map_err(|_| StepFailure::new("system timestamp does not fit u64"))
    }
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
