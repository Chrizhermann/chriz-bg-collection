//! Read-only native adapter over the engine's presentation-neutral CLI operations.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bg_engine::cli::{
    diagnostics_for_managed_install, discover_games as engine_discover_games, inspect_game,
    install_campaign_reviewed, plan_recipe, resume_campaign_controlled_expected, review_install,
    validate_recipe, CampaignReport, CliError, InstallCommandRequest, InstallReviewIdentity,
    SelectionOverrides, ValidationProfile,
};
use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::events::{EngineEvent, EventSink};
use bg_engine::games::{
    Eligibility, FindingKind, GameCandidate, GameProfiles, GameRole, Storefront,
};
use bg_engine::loader::Manifest;
use bg_engine::manifest::{AcquisitionPolicy, ArchiveKind, InputValue, Phase};
use bg_engine::preflight::is_creator_protected_destination;
use bg_engine::recipe_view::{FeatureControl, NormalizedSelection, RecipeView, SelectionFinding};
use bg_engine::registry::{
    CampaignAvailability, InstallAvailability, ManagedCampaignCard, ManagedInstallCard,
    ManagedInstallRegistry,
};
use bg_engine::resolve::InstallPlan;
use bg_engine::weidu::runner::RunnerControlHandle;
use serde::Serialize;

use crate::error::CommandError;
use crate::shortcut::{self, ShortcutRequest};
use crate::updates::{
    ApplicationUpdateResponse, ManagedCopyUpdateResponse, RecipeUpdateResponse, RecipeUpdateState,
    UpdateCenterResponse,
};

type Discoverer = dyn Fn(&Path) -> Result<Vec<GameCandidate>, CliError> + Send + Sync;

fn radar_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "radar_update_failed",
        "BG Radar Overlay could not be installed.",
        "Your game is still ready to play. Retry the overlay from Updates.",
        error.to_string(),
    )
}

const REVIEW_LIFETIME: Duration = Duration::from_secs(10 * 60);
const SNAPSHOT_EVENT_LIMIT: usize = 2_000;
const APPLICATION_DATA_DIRECTORY: &str = "Chriz BG Collection";

/// Engine services used by the native bridge. The test-only seam exercises the same stateful
/// review and worker boundary without launching WeiDU.
#[doc(hidden)]
pub trait BridgeEngine: Send + Sync {
    fn discover(&self, recipe: &Path) -> Result<Vec<GameCandidate>, CliError>;
    fn inspect_explicit(
        &self,
        recipe: &Path,
        role: GameRole,
        path: &Path,
    ) -> Result<GameCandidate, CliError>;
    fn reinspect(
        &self,
        recipe: &Path,
        candidate: &GameCandidate,
    ) -> Result<GameCandidate, CliError>;
    fn review(&self, request: &InstallCommandRequest) -> Result<InstallReviewIdentity, CliError>;
    fn install(
        &self,
        request: &InstallCommandRequest,
        expected: &InstallReviewIdentity,
        sink: &SequencedEventSink,
        controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, CliError>;
    fn resume(
        &self,
        managed_root: &Path,
        expected_install_id: &str,
        sink: &SequencedEventSink,
        controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, CliError>;
}

/// Operating-system actions kept behind the native bridge's validated identity boundary.
#[doc(hidden)]
pub trait BridgeSystem: Send + Sync {
    fn launch(&self, executable: &Path, working_directory: &Path) -> Result<(), String>;
    fn open_folder(&self, path: &Path) -> Result<(), String>;
    fn open_https(&self, url: &str) -> Result<(), String>;
    fn export_diagnostics(&self, managed_root: &Path, output: &Path) -> Result<PathBuf, String>;
    fn current_exe(&self) -> Result<PathBuf, String>;
    fn create_desktop_shortcut(&self, request: ShortcutRequest) -> Result<PathBuf, String>;
}

#[derive(Default)]
struct SystemBridgeSystem;

impl BridgeSystem for SystemBridgeSystem {
    fn launch(&self, executable: &Path, working_directory: &Path) -> Result<(), String> {
        Command::new(executable)
            .current_dir(working_directory)
            .spawn()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn open_folder(&self, path: &Path) -> Result<(), String> {
        open_with_shell(path.as_os_str())
    }

    fn open_https(&self, url: &str) -> Result<(), String> {
        open_with_shell(std::ffi::OsStr::new(url))
    }

    fn export_diagnostics(&self, managed_root: &Path, output: &Path) -> Result<PathBuf, String> {
        diagnostics_for_managed_install(managed_root, output)
            .map(|bundle| bundle.path)
            .map_err(|error| error.to_string())
    }

    fn current_exe(&self) -> Result<PathBuf, String> {
        std::env::current_exe().map_err(|error| error.to_string())
    }

    fn create_desktop_shortcut(&self, request: ShortcutRequest) -> Result<PathBuf, String> {
        shortcut::create_desktop_shortcut(request)
    }
}

#[cfg(windows)]
fn open_with_shell(target: &std::ffi::OsStr) -> Result<(), String> {
    Command::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
fn open_with_shell(target: &std::ffi::OsStr) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[derive(Default)]
struct SystemBridgeEngine;

impl BridgeEngine for SystemBridgeEngine {
    fn discover(&self, recipe: &Path) -> Result<Vec<GameCandidate>, CliError> {
        engine_discover_games(recipe)
    }

    fn inspect_explicit(
        &self,
        recipe: &Path,
        role: GameRole,
        path: &Path,
    ) -> Result<GameCandidate, CliError> {
        inspect_game(recipe, role, path).map(|report| report.candidate)
    }

    fn reinspect(
        &self,
        recipe: &Path,
        candidate: &GameCandidate,
    ) -> Result<GameCandidate, CliError> {
        inspect_game(recipe, candidate.role, &candidate.root).map(|report| report.candidate)
    }

    fn review(&self, request: &InstallCommandRequest) -> Result<InstallReviewIdentity, CliError> {
        review_install(request)
    }

    fn install(
        &self,
        request: &InstallCommandRequest,
        expected: &InstallReviewIdentity,
        sink: &SequencedEventSink,
        controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, CliError> {
        install_campaign_reviewed(request, expected, sink, controls)
    }

    fn resume(
        &self,
        managed_root: &Path,
        expected_install_id: &str,
        sink: &SequencedEventSink,
        controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, CliError> {
        resume_campaign_controlled_expected(managed_root, expected_install_id, sink, controls)
    }
}

struct DiscovererBridgeEngine {
    discoverer: Arc<Discoverer>,
    system: SystemBridgeEngine,
}

impl BridgeEngine for DiscovererBridgeEngine {
    fn discover(&self, recipe: &Path) -> Result<Vec<GameCandidate>, CliError> {
        (self.discoverer)(recipe)
    }

    fn inspect_explicit(
        &self,
        recipe: &Path,
        role: GameRole,
        path: &Path,
    ) -> Result<GameCandidate, CliError> {
        self.system.inspect_explicit(recipe, role, path)
    }

    fn reinspect(
        &self,
        recipe: &Path,
        candidate: &GameCandidate,
    ) -> Result<GameCandidate, CliError> {
        self.system.reinspect(recipe, candidate)
    }

    fn review(&self, request: &InstallCommandRequest) -> Result<InstallReviewIdentity, CliError> {
        self.system.review(request)
    }

    fn install(
        &self,
        request: &InstallCommandRequest,
        expected: &InstallReviewIdentity,
        sink: &SequencedEventSink,
        controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, CliError> {
        self.system.install(request, expected, sink, controls)
    }

    fn resume(
        &self,
        managed_root: &Path,
        expected_install_id: &str,
        sink: &SequencedEventSink,
        controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, CliError> {
        self.system
            .resume(managed_root, expected_install_id, sink, controls)
    }
}

#[derive(Default)]
struct BridgeRuntime {
    app_update_active: bool,
    candidates: HashMap<String, GameCandidate>,
    reviews: HashMap<String, ReviewSnapshot>,
    active_runs: HashMap<String, RunnerControlHandle>,
    run_snapshots: HashMap<String, RunSnapshotResponse>,
    known_installs: HashMap<String, PathBuf>,
}

#[derive(Clone)]
struct ReviewSnapshot {
    expires_at: Instant,
    digest: String,
    recipe_digest: String,
    selection_digest: String,
    plan_digest: String,
    bg1: GameCandidate,
    bg2: GameCandidate,
    request: InstallCommandRequest,
    engine_identity: InstallReviewIdentity,
}

/// Native adapter configuration and process-local review/run registries shared by commands.
#[derive(Clone)]
pub struct NativeBridge {
    resource_root: Option<PathBuf>,
    profile_id: String,
    recipe: PathBuf,
    preset: String,
    cache: PathBuf,
    app_data: Option<PathBuf>,
    engine: Arc<dyn BridgeEngine>,
    system: Arc<dyn BridgeSystem>,
    startup_install_id: Option<String>,
    runtime: Arc<Mutex<BridgeRuntime>>,
    sequence: Arc<AtomicU64>,
}

pub struct AppUpdateGuard(Arc<Mutex<BridgeRuntime>>);

impl Drop for AppUpdateGuard {
    fn drop(&mut self) {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .app_update_active = false;
    }
}

/// One engine event with a JS-lossless sequence and opaque process-local run identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunEventEnvelope {
    pub run_id: String,
    pub sequence_as_string: String,
    pub event: EngineEvent,
}

/// An event sink that sequences delivery and deliberately ignores a dropped listener.
pub struct SequencedEventSink {
    run_id: String,
    sequence: AtomicU64,
    listener: Arc<dyn Fn(RunEventEnvelope) + Send + Sync>,
}

impl SequencedEventSink {
    pub fn new(
        run_id: impl Into<String>,
        listener: impl Fn(RunEventEnvelope) + Send + Sync + 'static,
    ) -> Self {
        Self {
            run_id: run_id.into(),
            sequence: AtomicU64::new(0),
            listener: Arc::new(listener),
        }
    }
}

impl EventSink for SequencedEventSink {
    fn emit(&self, event: EngineEvent) {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed) + 1;
        (self.listener)(RunEventEnvelope {
            run_id: self.run_id.clone(),
            sequence_as_string: sequence.to_string(),
            event,
        });
    }
}

/// Read-only destination safety result shown before Review.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DestinationEvaluationResponse {
    /// Display-only normalized path; freezing the review resolves and validates it again.
    pub path: String,
    pub safe: bool,
    pub title: String,
    pub detail: String,
}

/// Player-editable defaults for a new installation. Computing these never writes to disk.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationDefaultsResponse {
    /// Initial editable installation name.
    pub name: String,
    /// Display-only suggested location; computing it does not create or authorize the folder.
    pub path: String,
}

/// Frozen, server-owned review identity. The opaque token is short-lived and single-use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenReviewResponse {
    pub review_token: String,
    pub digest: String,
    pub display_name: String,
    /// Display-only destination backed by the canonical path frozen in the server-side review.
    pub destination: String,
    pub game_labels: Vec<String>,
    pub evaluation: EvaluateBuildResponse,
}

/// Immediate response from a detached install or resume worker.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StartBuildResponse {
    pub run_id: String,
}

/// Verified manual archive copied into the installer-owned cache.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualArchiveResponse {
    pub artifact_id: String,
    pub filename: String,
    pub sha256: String,
    pub length: u64,
}

/// One selected manual artifact and its exact installer-owned cache readiness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualDownloadRequirementResponse {
    pub artifact_id: String,
    pub mod_ids: Vec<String>,
    pub title: String,
    pub filename: String,
    pub length: u64,
    pub ready: bool,
    pub detail: Option<String>,
}

/// One sanitized diagnostics archive created through a native file choice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticsExportResponse {
    pub path: String,
}

/// Display-only path returned after writing a verified CEBG launcher shortcut.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesktopShortcutResponse {
    pub path: String,
}

/// One immutable registry record projected without exposing executable authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedInstallationResponse {
    pub id: String,
    pub name: String,
    /// Display-only installation directory; actions accept the registry id, not this path.
    pub path: String,
    pub status: String,
    pub receipt_path: Option<String>,
    /// Display-only verified launcher path; actions still accept only the registry id.
    pub launch_path: Option<String>,
    /// Successful receipt completion time used only for launcher display and ordering.
    pub completed_at_millis: Option<u64>,
    pub available: bool,
    pub resumable: bool,
    pub recipe_version: Option<String>,
    pub consistency: Option<crate::consistency::ConsistencySummary>,
    pub radar_version: Option<String>,
}

/// Process-local run snapshot. Durable restart recovery remains an explicit post-v0 slice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunSnapshotResponse {
    pub run_id: String,
    pub status: String,
    pub events: Vec<RunEventEnvelope>,
    pub report: Option<CampaignReport>,
    pub error: Option<CommandError>,
}

/// Native backend identity returned during application startup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapResponse {
    /// Always `native` for this bridge.
    pub mode: String,
    /// Compiled engine crate version.
    pub engine_version: String,
    pub application_version: String,
    pub profiles: Vec<InstallProfileResponse>,
    pub selected_profile: String,
    /// Signed recipe release version; absent until Task 20 supplies it.
    pub recipe_version: Option<String>,
    /// Optional semantic launcher hint; callers must still match it to a current registry record.
    pub startup_install_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstallProfileResponse {
    pub id: String,
    pub label: String,
    pub description: String,
}

/// Parses one launcher-provided install identity without granting it filesystem authority.
pub fn parse_startup_install_id<I, S>(arguments: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut selected = None;
    for argument in arguments {
        let Some(argument) = argument.as_ref().to_str() else {
            continue;
        };
        let Some(install_id) = argument.strip_prefix("--install-id=") else {
            continue;
        };
        if install_id.is_empty() || selected.is_some() {
            return None;
        }
        selected = Some(install_id.to_owned());
    }
    selected
}

/// Player-facing freshness category for one candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CandidateFreshness {
    /// All profiled evidence matches a clean release.
    Fresh,
    /// A mod-sensitive surface differs.
    Modified,
    /// The executable version is unsupported.
    UnsupportedVersion,
    /// Siege of Dragonspear is missing.
    MissingSod,
    /// The storefront has not completed release acceptance.
    UnverifiedStorefront,
    /// Core hashes do not match a known clean variant.
    UnknownFingerprint,
    /// The locale is not supported by the alpha.
    UnsupportedLocale,
}

/// UI-safe projection of an inspected source game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameCandidateResponse {
    /// Stable semantic identity independent of list position.
    pub id: String,
    /// Human-readable game, storefront, and status.
    pub label: String,
    /// Human-readable source directory; later commands use its server-side semantic id.
    pub path: String,
    /// Storefront profile used by the engine.
    pub storefront: String,
    /// Four-part executable product version, when readable.
    pub build: Option<String>,
    /// Highest-priority freshness category.
    pub freshness: CandidateFreshness,
    /// Whether this exact source can be used by the current alpha.
    pub eligible: bool,
    /// Every engine finding, without suppressing simultaneous problems.
    pub findings: Vec<String>,
}

/// Independently grouped BG1 and BG2 discovery results.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameDiscoveryResponse {
    /// BGEE plus SoD candidates.
    pub bg1_candidates: Vec<GameCandidateResponse>,
    /// BG2EE candidates.
    pub bg2_candidates: Vec<GameCandidateResponse>,
    /// Best BG1 candidate identity, or an empty string when none was found.
    pub selected_bg1_id: String,
    /// Best BG2 candidate identity, or an empty string when none was found.
    pub selected_bg2_id: String,
}

/// One phase of the exact resolved plan, without WeiDU component numbers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseSummaryResponse {
    /// Stable phase id.
    pub id: String,
    /// Player-facing title.
    pub title: String,
    /// Short explanation of the phase.
    pub detail: String,
}

/// Safe plan projection used by the setup screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlanSummaryResponse {
    /// Selected phases in execution order.
    pub phases: Vec<PhaseSummaryResponse>,
}

/// Semantic recipe evaluation returned to the frontend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluateBuildResponse {
    /// Engine-owned controls.
    pub view: RecipeView,
    /// Complete normalized semantic selection.
    pub normalized_selection: NormalizedSelection,
    /// Nonfatal compatibility and omission findings.
    pub findings: Vec<SelectionFinding>,
    /// Phase-only plan projection; numeric component identities remain native.
    pub plan: PlanSummaryResponse,
    /// Number of currently selected visible controls.
    pub selected_choice_count: usize,
}

impl NativeBridge {
    /// Creates the production adapter over system game discovery.
    pub fn new(recipe: impl Into<PathBuf>, preset: impl Into<String>) -> Self {
        let recipe = recipe.into();
        let cache = recipe
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".chriz-cache");
        Self::with_configuration(
            recipe,
            preset.into(),
            cache,
            default_application_data_root(),
            Arc::new(SystemBridgeEngine),
            Arc::new(SystemBridgeSystem),
        )
    }

    /// Creates the production adapter from Tauri's trusted packaged-resource directory.
    pub fn from_resource_dir(resource_dir: &Path) -> Self {
        let mut bridge = Self::new(resource_dir.join("manifest"), "chris-recommended");
        bridge.resource_root = Some(resource_dir.to_path_buf());
        bridge.use_packaged_default();
        bridge
    }

    /// Creates the production adapter with a native-owned writable cache directory.
    pub fn from_resource_dir_with_cache(resource_dir: &Path, cache: &Path) -> Self {
        let mut bridge = Self::with_configuration(
            resource_dir.join("manifest"),
            "chris-recommended".to_owned(),
            cache.to_path_buf(),
            default_application_data_root(),
            Arc::new(SystemBridgeEngine),
            Arc::new(SystemBridgeSystem),
        );
        bridge.resource_root = Some(resource_dir.to_path_buf());
        bridge.use_packaged_default();
        bridge
    }

    fn use_packaged_default(&mut self) {
        if let Some(root) = &self.resource_root {
            let curated = root.join("recipes/curated-full-current");
            if curated.join("collection.toml").is_file() {
                self.recipe = curated;
                self.profile_id = "curated-full-current".to_owned();
            }
        }
    }

    pub fn profiles(&self) -> Vec<InstallProfileResponse> {
        // A bundled historical recipe is evidence, not an approved install selection.
        let mut profiles = Vec::new();
        if self.resource_root.as_ref().is_some_and(|root| {
            root.join("recipes/curated-full-current/collection.toml")
                .is_file()
        }) {
            profiles.push(InstallProfileResponse {
                id: "curated-full-current".to_owned(),
                label: "Full curated setup".to_owned(),
                description:
                    "Your curated collection. Some mods need a manually supplied download."
                        .to_owned(),
            });
        }
        profiles.push(InstallProfileResponse {
            id: "public-alpha".to_owned(),
            label: "Smaller downloadable alpha".to_owned(),
            description: "Limited setup using automatically downloadable mods only.".to_owned(),
        });
        profiles
    }

    pub fn select_profile(&self, id: &str) -> Result<Self, CommandError> {
        let mut runtime = self.runtime_lock();
        if !runtime.active_runs.is_empty() || runtime.app_update_active {
            return Err(CommandError::new(
                "build_active",
                "Wait for the installation to finish.",
                "Change setup after the current operation finishes.",
                "profile selection during an active operation",
            ));
        }
        if id == "creator-full-current" {
            return Err(CommandError::new(
                "profile_requires_curation",
                "This historical setup does not match the curated collection.",
                "Choose one of the curated setups included in this release.",
                "legacy WeiDU replay bypasses recorded curation and must not be installed",
            ));
        }
        if !self.profiles().iter().any(|profile| profile.id == id) {
            return Err(CommandError::new(
                "unknown_profile",
                "That setup is unavailable.",
                "Choose one of the included setups.",
                id,
            ));
        }
        let root = self.resource_root.as_ref().ok_or_else(|| {
            CommandError::new(
                "unknown_profile",
                "Setup selection is unavailable.",
                "Use a packaged CEBG release.",
                "no packaged resource root",
            )
        })?;
        let mut next = self.clone();
        next.profile_id = id.to_owned();
        next.recipe = root.join(if id == "curated-full-current" {
            "recipes/curated-full-current"
        } else {
            "manifest"
        });
        next.preset = "chris-recommended".to_owned();
        runtime.reviews.clear();
        Ok(next)
    }

    fn validation_profile(&self) -> ValidationProfile {
        ValidationProfile::PublicAlpha
    }

    pub fn recipe_version(&self) -> Result<Option<String>, CommandError> {
        bg_engine::cli::recipe_release_version(&self.recipe).map_err(CommandError::from_cli)
    }

    /// Attaches a process-start selection hint that remains subject to registry validation.
    pub fn with_startup_install_id(mut self, startup_install_id: Option<String>) -> Self {
        self.startup_install_id = startup_install_id;
        self
    }

    /// Replaces only host discovery, allowing deterministic command-contract tests.
    #[doc(hidden)]
    pub fn with_discoverer<F>(
        recipe: impl Into<PathBuf>,
        preset: impl Into<String>,
        discoverer: F,
    ) -> Self
    where
        F: Fn(&Path) -> Result<Vec<GameCandidate>, CliError> + Send + Sync + 'static,
    {
        let recipe = recipe.into();
        let cache = recipe
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".chriz-cache");
        Self::with_configuration(
            recipe,
            preset.into(),
            cache,
            default_application_data_root(),
            Arc::new(DiscovererBridgeEngine {
                discoverer: Arc::new(discoverer),
                system: SystemBridgeEngine,
            }),
            Arc::new(SystemBridgeSystem),
        )
    }

    /// Injects all engine effects while retaining the production bridge state machine.
    #[doc(hidden)]
    pub fn with_engine(
        recipe: impl Into<PathBuf>,
        preset: impl Into<String>,
        cache: impl Into<PathBuf>,
        engine: Arc<dyn BridgeEngine>,
    ) -> Self {
        let cache = cache.into();
        let app_data = cache
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("app-data");
        Self::with_configuration(
            recipe.into(),
            preset.into(),
            cache,
            Some(app_data),
            engine,
            Arc::new(SystemBridgeSystem),
        )
    }

    /// Injects engine and host services plus the exact managed-install registry root.
    #[doc(hidden)]
    pub fn with_engine_and_system(
        recipe: impl Into<PathBuf>,
        preset: impl Into<String>,
        cache: impl Into<PathBuf>,
        app_data: impl Into<PathBuf>,
        engine: Arc<dyn BridgeEngine>,
        system: Arc<dyn BridgeSystem>,
    ) -> Self {
        Self::with_configuration(
            recipe.into(),
            preset.into(),
            cache.into(),
            Some(app_data.into()),
            engine,
            system,
        )
    }

    /// Validates the complete public-alpha recipe and its configured preset.
    pub fn bootstrap(&self) -> Result<BootstrapResponse, CommandError> {
        validate_recipe(&self.recipe, self.validation_profile()).map_err(CommandError::from_cli)?;
        GameProfiles::load(self.recipe.join("game-builds")).map_err(|error| {
            CommandError::new(
                "game_profiles_failed",
                "Verified game detection data is unavailable.",
                "Install a release that includes verified game profiles.",
                error.to_string(),
            )
        })?;
        plan_recipe(
            &self.recipe,
            &self.preset,
            "windows",
            &SelectionOverrides::default(),
        )
        .map_err(CommandError::from_cli)?;
        Ok(BootstrapResponse {
            mode: "native".to_owned(),
            engine_version: bg_engine::VERSION.to_owned(),
            application_version: env!("CARGO_PKG_VERSION").to_owned(),
            profiles: self.profiles(),
            selected_profile: self.profile_id.clone(),
            recipe_version: self.recipe_version()?,
            startup_install_id: self.startup_install_id.clone(),
        })
    }

    /// Discovers and independently groups installed BG1 and BG2 sources.
    pub fn discover_games(&self) -> Result<GameDiscoveryResponse, CommandError> {
        let candidates = self
            .engine
            .discover(&self.recipe)
            .map_err(CommandError::from_cli)?;
        let mut bg1_candidates = Vec::new();
        let mut bg2_candidates = Vec::new();
        for candidate in candidates {
            let role = candidate.role;
            let response = project_candidate(candidate.clone())?;
            self.runtime_lock()
                .candidates
                .insert(response.id.clone(), candidate);
            match role {
                GameRole::BgeeSod => bg1_candidates.push(response),
                GameRole::Bg2ee => bg2_candidates.push(response),
            }
        }
        sort_candidates(&mut bg1_candidates);
        sort_candidates(&mut bg2_candidates);
        let selected_bg1_id = bg1_candidates
            .first()
            .map(|candidate| candidate.id.clone())
            .unwrap_or_default();
        let selected_bg2_id = bg2_candidates
            .first()
            .map(|candidate| candidate.id.clone())
            .unwrap_or_default();
        Ok(GameDiscoveryResponse {
            bg1_candidates,
            bg2_candidates,
            selected_bg1_id,
            selected_bg2_id,
        })
    }

    /// Validates one explicit path against the configured immutable profiles.
    pub fn inspect_game_path(
        &self,
        role: GameRole,
        path: &Path,
    ) -> Result<GameCandidateResponse, CommandError> {
        let candidate = self
            .engine
            .inspect_explicit(&self.recipe, role, path)
            .map_err(CommandError::from_cli)?;
        let response = project_candidate(candidate.clone())?;
        self.runtime_lock()
            .candidates
            .insert(response.id.clone(), candidate);
        Ok(response)
    }

    /// Validates a native folder choice immediately; cancellation leaves bridge state unchanged.
    pub fn choose_game_folder(
        &self,
        role: GameRole,
        selected: Option<PathBuf>,
    ) -> Result<Option<GameCandidateResponse>, CommandError> {
        selected
            .map(|path| self.inspect_game_path(role, &path))
            .transpose()
    }

    /// Evaluates the configured preset plus a normalized semantic frontend selection.
    pub fn evaluate_build(
        &self,
        selection: &NormalizedSelection,
    ) -> Result<EvaluateBuildResponse, CommandError> {
        let report = plan_recipe(
            &self.recipe,
            &self.preset,
            &selection.platform,
            &selection_overrides(selection),
        )
        .map_err(CommandError::from_cli)?;
        Ok(project_evaluation(report.evaluation))
    }

    /// Lists selected manual artifacts and verifies only their exact installer-owned cache paths.
    pub fn inspect_manual_downloads(
        &self,
        selection: &NormalizedSelection,
    ) -> Result<Vec<ManualDownloadRequirementResponse>, CommandError> {
        let report = plan_recipe(
            &self.recipe,
            &self.preset,
            &selection.platform,
            &selection_overrides(selection),
        )
        .map_err(CommandError::from_cli)?;
        self.manual_download_requirements(&report.evaluation.plan)
    }

    fn manual_download_requirements(
        &self,
        plan: &InstallPlan,
    ) -> Result<Vec<ManualDownloadRequirementResponse>, CommandError> {
        let manifest = Manifest::load(&self.recipe).map_err(|error| {
            CommandError::new(
                "recipe_load_failed",
                "The installer recipe could not be verified.",
                "Repair or replace the installer recipe, then try again.",
                error.to_string(),
            )
        })?;
        let mut owners = BTreeMap::<String, BTreeSet<String>>::new();
        for run in &plan.runs {
            for artifact_id in [&run.artifact_id, &run.weidu_artifact_id] {
                let artifact = manifest.artifacts.get(artifact_id).ok_or_else(|| {
                    invalid_manual_contract(artifact_id, "resolved artifact metadata")
                })?;
                if artifact.acquisition == AcquisitionPolicy::ManualUserSupplied {
                    owners
                        .entry(artifact_id.clone())
                        .or_default()
                        .insert(run.mod_id.clone());
                }
            }
        }

        owners
            .into_iter()
            .map(|(artifact_id, mod_ids)| {
                let artifact = &manifest.artifacts[&artifact_id];
                let filename = artifact
                    .source
                    .expected_filename
                    .as_deref()
                    .filter(|filename| {
                        let mut components = Path::new(filename).components();
                        matches!(components.next(), Some(Component::Normal(_)))
                            && components.next().is_none()
                    })
                    .ok_or_else(|| {
                        invalid_manual_contract(&artifact_id, "safe expected filename")
                    })?;
                let length = artifact.source.expected_length.ok_or_else(|| {
                    invalid_manual_contract(&artifact_id, "expected archive length")
                })?;
                let mod_ids = mod_ids.into_iter().collect::<Vec<_>>();
                let title = mod_ids
                    .iter()
                    .map(|mod_id| {
                        manifest
                            .mods
                            .get(mod_id)
                            .map(|mod_file| mod_file.name.as_str())
                            .unwrap_or(mod_id.as_str())
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let path = self.cache.join("manual").join(filename);
                let (ready, detail) =
                    inspect_manual_cache_file(&path, &artifact.source.sha256, length, filename);
                Ok(ManualDownloadRequirementResponse {
                    artifact_id,
                    mod_ids,
                    title,
                    filename: filename.to_owned(),
                    length,
                    ready,
                    detail,
                })
            })
            .collect()
    }

    fn ensure_manual_downloads_ready(&self, plan: &InstallPlan) -> Result<(), CommandError> {
        let missing = self
            .manual_download_requirements(plan)?
            .into_iter()
            .filter(|requirement| !requirement.ready)
            .collect::<Vec<_>>();
        if missing.is_empty() {
            return Ok(());
        }
        let detail = missing
            .iter()
            .map(|requirement| {
                format!(
                    "{} ({}): {}",
                    requirement.title,
                    requirement.filename,
                    requirement.detail.as_deref().unwrap_or("not ready")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Err(CommandError::new(
            "manual_download_required",
            "A selected mod still needs its official download.",
            "Choose the exact official archive, or skip that mod before starting the installation.",
            detail,
        ))
    }

    /// Validates a prospective managed-copy destination without creating or claiming it.
    pub fn inspect_destination(
        &self,
        destination: &Path,
        bg1_candidate_id: &str,
        bg2_candidate_id: &str,
    ) -> Result<DestinationEvaluationResponse, CommandError> {
        let (bg1, bg2) = self.registered_sources(bg1_candidate_id, bg2_candidate_id)?;
        let mut response =
            inspect_destination_path(destination, &bg1.root, &bg2.root, &self.cache)?;
        response.path = display_windows_path(Path::new(&response.path))?;
        Ok(response)
    }

    /// Validates a native destination choice immediately; cancellation is a no-op.
    pub fn choose_destination_folder(
        &self,
        selected: Option<PathBuf>,
        bg1_candidate_id: &str,
        bg2_candidate_id: &str,
    ) -> Result<Option<DestinationEvaluationResponse>, CommandError> {
        selected
            .map(|path| self.inspect_destination(&path, bg1_candidate_id, bg2_candidate_id))
            .transpose()
    }

    /// Verifies and publishes one user-selected archive using only trusted recipe metadata.
    pub fn supply_manual_archive(
        &self,
        artifact_id: &str,
        selected: Option<PathBuf>,
    ) -> Result<Option<ManualArchiveResponse>, CommandError> {
        let Some(selected) = selected else {
            return Ok(None);
        };
        validate_recipe(&self.recipe, self.validation_profile()).map_err(CommandError::from_cli)?;
        let manifest = Manifest::load(&self.recipe).map_err(|error| {
            CommandError::new(
                "recipe_load_failed",
                "The installer recipe could not be verified.",
                "Repair or replace the installer recipe, then try again.",
                error.to_string(),
            )
        })?;
        let artifact = manifest.artifacts.get(artifact_id).ok_or_else(|| {
            CommandError::new(
                "manual_archive_unknown",
                "That archive is not part of this installer recipe.",
                "Return to the build screen and choose the archive requested there.",
                format!("unknown manual artifact id {artifact_id:?}"),
            )
        })?;
        if artifact.acquisition != AcquisitionPolicy::ManualUserSupplied {
            return Err(CommandError::new(
                "manual_archive_not_allowed",
                "That recipe artifact is not supplied manually.",
                "Return to the build screen and choose the archive requested there.",
                format!("artifact {artifact_id:?} is {:?}", artifact.acquisition),
            ));
        }
        let filename = artifact
            .source
            .expected_filename
            .as_deref()
            .filter(|filename| {
                let mut components = Path::new(filename).components();
                matches!(components.next(), Some(Component::Normal(_)))
                    && components.next().is_none()
            })
            .ok_or_else(|| invalid_manual_contract(artifact_id, "safe expected filename"))?;
        let expected_length = artifact
            .source
            .expected_length
            .ok_or_else(|| invalid_manual_contract(artifact_id, "expected archive length"))?;
        bg_engine::acquire::ArtifactCache::open(&self.cache).map_err(|error| {
            CommandError::new(
                "cache_unavailable",
                "The installer cache is not writable.",
                "Choose a writable local app-data location and try again.",
                error.to_string(),
            )
        })?;
        let destination = self.cache.join("manual").join(filename);
        let published = bg_engine::acquire::publish_manual_archive(
            &selected,
            &destination,
            &artifact.source.sha256,
            expected_length,
        )
        .map_err(|error| {
            CommandError::new(
                "manual_archive_invalid",
                "That file does not match the archive required by this recipe.",
                "Choose the exact published archive named on the build screen.",
                error.to_string(),
            )
        })?;
        Ok(Some(ManualArchiveResponse {
            artifact_id: artifact.id.clone(),
            filename: filename.to_owned(),
            sha256: published.sha256,
            length: published.length,
        }))
    }

    /// Returns the sole extension allowed by this manual artifact's validated archive kind.
    #[doc(hidden)]
    pub fn manual_archive_filter_extensions(
        &self,
        artifact_id: &str,
    ) -> Result<Vec<String>, CommandError> {
        validate_recipe(&self.recipe, self.validation_profile()).map_err(CommandError::from_cli)?;
        let manifest = Manifest::load(&self.recipe).map_err(|error| {
            CommandError::new(
                "recipe_load_failed",
                "The installer recipe could not be verified.",
                "Repair or replace the installer recipe, then try again.",
                error.to_string(),
            )
        })?;
        let artifact = manifest.artifacts.get(artifact_id).ok_or_else(|| {
            CommandError::new(
                "manual_archive_unknown",
                "That file is not part of this installer recipe.",
                "Return to setup and choose the download requested there.",
                format!("unknown manual artifact id {artifact_id:?}"),
            )
        })?;
        if artifact.acquisition != AcquisitionPolicy::ManualUserSupplied {
            return Err(CommandError::new(
                "manual_archive_not_allowed",
                "That recipe file is not supplied manually.",
                "Return to setup and use the requested download action.",
                format!("artifact {artifact_id:?} is {:?}", artifact.acquisition),
            ));
        }
        let extension = match artifact.archive.kind {
            ArchiveKind::Zip => "zip",
            ArchiveKind::Iemod => "iemod",
            ArchiveKind::SelfExtractingRar => "exe",
        };
        Ok(vec![extension.to_owned()])
    }

    /// Opens only the HTTPS manual-download page declared by a verified recipe artifact.
    pub fn open_manual_source(&self, artifact_id: &str) -> Result<(), CommandError> {
        validate_recipe(&self.recipe, self.validation_profile()).map_err(CommandError::from_cli)?;
        let manifest = Manifest::load(&self.recipe).map_err(|error| {
            CommandError::new(
                "recipe_load_failed",
                "The installer recipe could not be verified.",
                "Repair or replace the installer recipe, then try again.",
                error.to_string(),
            )
        })?;
        let artifact = manifest.artifacts.get(artifact_id).ok_or_else(|| {
            CommandError::new(
                "manual_source_unknown",
                "That download page is not part of this installer recipe.",
                "Return to the build screen and use the requested manual download.",
                format!("unknown manual artifact id {artifact_id:?}"),
            )
        })?;
        if artifact.acquisition != AcquisitionPolicy::ManualUserSupplied {
            return Err(CommandError::new(
                "manual_source_not_allowed",
                "That recipe artifact does not use a manual download page.",
                "Return to the build screen and follow the acquisition action shown there.",
                format!("artifact {artifact_id:?} is {:?}", artifact.acquisition),
            ));
        }
        let url = artifact.source.url.trim();
        if !url.starts_with("https://")
            || url.len() == "https://".len()
            || url.chars().any(char::is_whitespace)
        {
            return Err(invalid_manual_contract(
                artifact_id,
                "safe HTTPS source URL",
            ));
        }
        self.system.open_https(url).map_err(|error| {
            CommandError::new(
                "manual_source_open_failed",
                "The manual download page could not be opened.",
                "Open the trusted source shown in the build details and try again.",
                error,
            )
        })
    }

    /// Lists completed managed installs from the immutable app-data registry.
    pub fn list_managed_installations(
        &self,
    ) -> Result<Vec<ManagedInstallationResponse>, CommandError> {
        let completed = self.registry_cards()?;
        let completed_ids = completed
            .iter()
            .map(|card| card.record.install_id.clone())
            .collect::<BTreeSet<_>>();
        let mut responses = completed
            .into_iter()
            .map(project_managed_install)
            .collect::<Result<Vec<_>, _>>()?;
        responses.extend(
            self.campaign_cards()?
                .into_iter()
                .filter(|card| !completed_ids.contains(&card.record.install_id))
                .map(project_managed_campaign)
                .collect::<Result<Vec<_>, _>>()?,
        );
        responses.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(responses)
    }

    /// Launches the exact verified InfinityLoader path owned by one available registry id.
    pub fn launch_install(&self, install_id: &str) -> Result<(), CommandError> {
        if !self.runtime_lock().active_runs.is_empty() {
            return Err(CommandError::new(
                "build_active",
                "The game cannot be launched while an installer build is running.",
                "Wait for the build to finish before launching the campaign.",
                "one or more installer workers are active",
            ));
        }
        let record = self.available_managed_install(install_id)?.record;
        let launch_name = record
            .launch_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if !launch_name.eq_ignore_ascii_case("InfinityLoader.exe") {
            return Err(CommandError::new(
                "managed_launch_invalid",
                "The managed campaign has no verified InfinityLoader executable.",
                "Run diagnostics and rebuild the campaign before launching it.",
                format!("registered launch path is {}", record.launch_path.display()),
            ));
        }
        let working_directory = record.launch_path.parent().ok_or_else(|| {
            CommandError::new(
                "managed_launch_invalid",
                "The managed campaign launch path is incomplete.",
                "Run diagnostics and rebuild the campaign before launching it.",
                format!(
                    "launch path has no parent: {}",
                    record.launch_path.display()
                ),
            )
        })?;
        self.system
            .launch(&record.launch_path, working_directory)
            .map_err(|error| {
                CommandError::new(
                    "managed_launch_failed",
                    "The managed campaign could not be launched.",
                    "Close any running game instance, then try again.",
                    error,
                )
            })?;
        if let Ok(overlay) = bg_engine::radar::status(&record.managed_root, working_directory, None)
        {
            if overlay.state == bg_engine::radar::RadarState::UpToDate {
                if let Some(executable) = overlay.executable {
                    if let Some(directory) = executable.parent() {
                        // The optional overlay must not prevent the game from launching.
                        let _ = self.system.launch(&executable, directory);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn install_radar(
        &self,
        install_id: &str,
    ) -> Result<bg_engine::radar::RadarInstallResult, CommandError> {
        let release = bg_engine::radar::check_latest().map_err(radar_error)?;
        let _guard = self.begin_app_update()?;
        let record = self.available_managed_install(install_id)?.record;
        let game_root = record.managed_root.join("game");
        bg_engine::radar::install(
            &self.cache,
            &record.managed_root,
            &game_root,
            &release,
            &bg_engine::events::ConsoleSink,
        )
        .map_err(radar_error)
    }

    pub fn radar_update(&self) -> crate::updates::RadarUpdateResponse {
        let latest = bg_engine::radar::check_latest();
        let current_version = self.list_managed_installations().ok().and_then(|copies| {
            copies
                .into_iter()
                .filter(|copy| copy.available)
                .max_by_key(|copy| copy.completed_at_millis)
                .and_then(|copy| copy.radar_version)
        });
        match latest {
            Ok(release) => crate::updates::RadarUpdateResponse {
                state: if current_version.as_deref() == Some(release.tag.as_str()) {
                    "up-to-date"
                } else {
                    "available"
                }
                .to_owned(),
                current_version,
                available_version: Some(release.tag.clone()),
                detail: "Optional overlay. Updated separately without changing your mods or saves."
                    .to_owned(),
                release_notes: Some(format!(
                    "BG Radar Overlay {}\nPublished {}\nhttps://github.com/{}/releases/tag/{}",
                    release.tag,
                    release.published_at,
                    bg_engine::radar::RADAR_REPOSITORY,
                    release.tag
                )),
            },
            Err(error) => crate::updates::RadarUpdateResponse {
                state: "offline".to_owned(),
                current_version,
                available_version: None,
                detail: format!("Could not check BG Radar Overlay: {error}"),
                release_notes: None,
            },
        }
    }

    /// Opens only the managed root attached to one available immutable registry id.
    pub fn open_install_folder(&self, install_id: &str) -> Result<(), CommandError> {
        let record = self.available_managed_install(install_id)?.record;
        self.system
            .open_folder(&record.managed_root)
            .map_err(|error| {
                CommandError::new(
                    "managed_folder_open_failed",
                    "The managed campaign folder could not be opened.",
                    "Check that the campaign folder is still available, then try again.",
                    error,
                )
            })
    }

    /// Creates a launcher shortcut only after reloading an available registry identity.
    pub fn create_desktop_shortcut(
        &self,
        install_id: &str,
    ) -> Result<DesktopShortcutResponse, CommandError> {
        let record = self.available_managed_install(install_id)?.record;
        let target = self.system.current_exe().map_err(|error| {
            CommandError::new(
                "application_executable_unavailable",
                "The CEBG application path could not be found.",
                "Open CEBG normally, then try creating the shortcut again.",
                error,
            )
        })?;
        if !target.is_absolute()
            || target
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("InfinityLoader.exe"))
        {
            return Err(CommandError::new(
                "application_executable_invalid",
                "The CEBG application path is not safe for a shortcut.",
                "Open the installed CEBG application, then try again.",
                format!("current executable resolved to {}", target.display()),
            ));
        }
        let working_directory = target
            .parent()
            .ok_or_else(|| {
                CommandError::new(
                    "application_executable_invalid",
                    "The CEBG application folder could not be found.",
                    "Open the installed CEBG application, then try again.",
                    format!("current executable has no parent: {}", target.display()),
                )
            })?
            .to_path_buf();
        let request = ShortcutRequest {
            install_id: record.install_id.clone(),
            display_name: record.display_name,
            target,
            arguments: format!("--install-id={}", record.install_id),
            working_directory,
        };
        let path = self
            .system
            .create_desktop_shortcut(request)
            .map_err(|error| {
                CommandError::new(
                    "desktop_shortcut_failed",
                    "The desktop shortcut could not be created.",
                    "Your installation is ready to play. Retry the shortcut from My installs.",
                    error,
                )
            })?;
        Ok(DesktopShortcutResponse {
            path: display_windows_path(&path)?.to_owned(),
        })
    }

    /// Exports diagnostics for an engine-known install to one native-selected output path.
    pub fn export_diagnostics(
        &self,
        install_id: &str,
        selected_output: Option<PathBuf>,
    ) -> Result<Option<DiagnosticsExportResponse>, CommandError> {
        let Some(output) = selected_output else {
            return Ok(None);
        };
        let managed_root = self.diagnostics_managed_root(install_id)?;
        let path = self
            .system
            .export_diagnostics(&managed_root, &output)
            .map_err(|error| {
                CommandError::new(
                    "diagnostics_failed",
                    "The installer diagnostics could not be exported.",
                    "Choose a new ZIP path outside the managed campaign and try again.",
                    error,
                )
            })?;
        let path = display_windows_path(&path).map_err(|error| {
            CommandError::new(
                "diagnostics_path_invalid",
                "The diagnostics path cannot be displayed safely.",
                "Choose a local path with a Windows-compatible name and try again.",
                error.technical_detail,
            )
        })?;
        Ok(Some(DiagnosticsExportResponse { path }))
    }

    /// Re-inspects exact server-side candidates and freezes a short-lived, single-use review.
    pub fn freeze_review(
        &self,
        display_name: &str,
        selection: &NormalizedSelection,
        destination: &Path,
        bg1_candidate_id: &str,
        bg2_candidate_id: &str,
    ) -> Result<FrozenReviewResponse, CommandError> {
        let overrides = selection_overrides(selection);
        let plan_report = plan_recipe(&self.recipe, &self.preset, &selection.platform, &overrides)
            .map_err(CommandError::from_cli)?;
        self.ensure_manual_downloads_ready(&plan_report.evaluation.plan)?;
        let (registered_bg1, registered_bg2) =
            self.registered_sources(bg1_candidate_id, bg2_candidate_id)?;
        let bg1 = self.reinspect_source(&registered_bg1, GameRole::BgeeSod)?;
        let bg2 = self.reinspect_source(&registered_bg2, GameRole::Bg2ee)?;
        let destination = inspect_destination_path(destination, &bg1.root, &bg2.root, &self.cache)?;
        let destination_path = PathBuf::from(&destination.path);
        bg_engine::acquire::ArtifactCache::open(&self.cache).map_err(|error| {
            CommandError::new(
                "cache_unavailable",
                "The installer cache is not writable.",
                "Choose a writable local app-data location and try again.",
                error.to_string(),
            )
        })?;
        let cache = self.cache.canonicalize().map_err(|error| {
            path_error(
                "cache_unavailable",
                "The installer cache could not be resolved safely.",
                &self.cache,
                error,
            )
        })?;
        let normalized_selection = &plan_report.evaluation.normalized_selection;
        let recipe_digest = recipe_directory_digest(&plan_report.recipe)?;
        let selection_digest = selection_digest(normalized_selection).map_err(digest_error)?;
        let plan_digest = plan_digest(&plan_report.evaluation.plan).map_err(digest_error)?;
        let request = InstallCommandRequest {
            application_version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            display_name: display_name.to_owned(),
            recipe: plan_report.recipe.clone(),
            preset: self.preset.clone(),
            platform: normalized_selection.platform.clone(),
            overrides: selection_overrides(normalized_selection),
            bg1: bg1.root.clone(),
            bg2: bg2.root.clone(),
            managed_root: destination_path,
            cache,
        };
        let engine_identity = self
            .engine
            .review(&request)
            .map_err(CommandError::from_cli)?;
        let frozen_display_name = engine_identity.display_name.clone();
        let digest = review_digest(
            &recipe_digest,
            &selection_digest,
            &plan_digest,
            &bg1,
            &bg2,
            &destination.path,
            &engine_identity,
        )?;
        let token_seed = format!(
            "{digest}\0{}\0{}",
            unix_nanos()?,
            self.sequence.fetch_add(1, Ordering::Relaxed)
        );
        let review_token = sha256_bytes(token_seed.as_bytes());
        self.runtime_lock().reviews.insert(
            review_token.clone(),
            ReviewSnapshot {
                expires_at: Instant::now() + REVIEW_LIFETIME,
                digest: digest.clone(),
                recipe_digest,
                selection_digest,
                plan_digest,
                bg1: bg1.clone(),
                bg2: bg2.clone(),
                request,
                engine_identity,
            },
        );
        let game_labels = vec![project_candidate(bg1)?.label, project_candidate(bg2)?.label];
        Ok(FrozenReviewResponse {
            review_token,
            digest,
            display_name: frozen_display_name,
            destination: display_windows_path(Path::new(&destination.path))?,
            game_labels,
            evaluation: project_evaluation(plan_report.evaluation),
        })
    }

    /// Starts a reviewed install on a retained worker and returns immediately.
    pub fn start_build<F>(
        &self,
        review_token: &str,
        listener: F,
    ) -> Result<StartBuildResponse, CommandError>
    where
        F: Fn(RunEventEnvelope) + Send + Sync + 'static,
    {
        let review = self
            .runtime_lock()
            .reviews
            .remove(review_token)
            .ok_or_else(invalid_review_token)?;
        self.validate_review_snapshot(&review)?;
        let plan = plan_recipe(
            &review.request.recipe,
            &review.request.preset,
            &review.request.platform,
            &review.request.overrides,
        )
        .map_err(CommandError::from_cli)?;
        self.ensure_manual_downloads_ready(&plan.evaluation.plan)?;
        self.spawn_install_worker(review.request, review.engine_identity, listener)
    }

    /// Resumes only an exact application-data identity backed by a verified campaign ledger.
    pub fn resume_build<F>(
        &self,
        install_id: &str,
        listener: F,
    ) -> Result<StartBuildResponse, CommandError>
    where
        F: Fn(RunEventEnvelope) + Send + Sync + 'static,
    {
        let managed_root = self.resumable_campaign(install_id)?;
        self.spawn_resume_worker(install_id.to_owned(), managed_root, listener)
    }

    /// Returns the latest process-local event/result snapshot for one opaque run id.
    pub fn get_run_snapshot(&self, run_id: &str) -> Result<RunSnapshotResponse, CommandError> {
        let mut snapshot = self
            .runtime_lock()
            .run_snapshots
            .get(run_id)
            .cloned()
            .ok_or_else(|| unknown_run(run_id))?;
        if let Some(report) = snapshot.report.as_mut() {
            report.managed_root = PathBuf::from(display_windows_path(&report.managed_root)?);
        }
        Ok(snapshot)
    }

    /// Rearms the silence watchdog for exactly one active worker.
    pub fn continue_waiting(&self, run_id: &str) -> Result<(), CommandError> {
        self.active_controls(run_id)?.continue_waiting();
        Ok(())
    }

    /// Requests cancellation for exactly one active worker.
    pub fn cancel_run(&self, run_id: &str) -> Result<(), CommandError> {
        self.active_controls(run_id)?.cancel();
        Ok(())
    }

    /// Requests a cooperative stop after the active step is durably complete.
    pub fn pause_run(&self, run_id: &str) -> Result<(), CommandError> {
        self.active_controls(run_id)?.pause_after_boundary();
        if let Some(snapshot) = self.runtime_lock().run_snapshots.get_mut(run_id) {
            snapshot.status = "pausing".to_owned();
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn active_run_count(&self) -> usize {
        self.runtime_lock().active_runs.len()
    }

    /// Returns the sole active run id, when this process owns a build worker.
    #[doc(hidden)]
    pub fn active_run_id(&self) -> Option<String> {
        self.runtime_lock().active_runs.keys().next().cloned()
    }

    /// Refuses application or trusted-recipe replacement while this process owns a build.
    /// Download-only checks remain allowed because each campaign already freezes exact bytes.
    #[doc(hidden)]
    pub fn ensure_update_idle(&self) -> Result<(), CommandError> {
        if self.active_run_count() == 0 {
            Ok(())
        } else {
            Err(CommandError::new(
                "update_deferred_build_active",
                "The update is ready, but cannot be installed while a campaign build is running.",
                "Let the current build finish, then install the update.",
                "one or more native build workers retain an immutable recipe snapshot",
            ))
        }
    }

    pub fn begin_app_update(&self) -> Result<AppUpdateGuard, CommandError> {
        let mut state = self.runtime_lock();
        if !state.active_runs.is_empty() || state.app_update_active {
            return Err(CommandError::new(
                "update_busy",
                "An installation or update is already running.",
                "Wait for it to finish.",
                "Update and build workers cannot overlap.",
            ));
        }
        state.app_update_active = true;
        Ok(AppUpdateGuard(Arc::clone(&self.runtime)))
    }

    /// Returns an honest offline-safe projection until release-time trust material is supplied.
    #[doc(hidden)]
    pub fn unconfigured_update_center(
        &self,
        running_app_version: &str,
    ) -> Result<UpdateCenterResponse, CommandError> {
        let managed_copies = self
            .list_managed_installations()?
            .into_iter()
            .map(|copy| {
                let stale = !copy.available && !copy.resumable;
                ManagedCopyUpdateResponse {
                    install_id: copy.id,
                    name: copy.name,
                    path: copy.path,
                    installed_recipe_version: copy.recipe_version,
                    state: if stale { "stale" } else { "unknown" }.to_owned(),
                    detail: if stale {
                        "The registered folder moved or changed; no update action is available."
                            .to_owned()
                    } else {
                        "No signed recipe channel is configured, so update applicability is unknown."
                            .to_owned()
                    },
                }
            })
            .collect();
        Ok(UpdateCenterResponse {
            checked_at: None,
            network_state: "unconfigured".to_owned(),
            application: ApplicationUpdateResponse {
                state: "unavailable".to_owned(),
                current_version: running_app_version.to_owned(),
                available_version: None,
                detail: "The application update key and endpoint must be supplied at release-time."
                    .to_owned(),
                release_notes: None,
            },
            recipe: RecipeUpdateResponse {
                state: RecipeUpdateState::Unavailable,
                current_version: "bundled".to_owned(),
                available_version: None,
                minimum_app_version: None,
                disposition: "unknown-applicability".to_owned(),
                detail: "No signed channel is configured; the bundled trusted recipe was kept."
                    .to_owned(),
                changes: Vec::new(),
            },
            managed_copies,
            radar: None,
        })
    }
}

impl NativeBridge {
    fn with_configuration(
        recipe: PathBuf,
        preset: String,
        cache: PathBuf,
        app_data: Option<PathBuf>,
        engine: Arc<dyn BridgeEngine>,
        system: Arc<dyn BridgeSystem>,
    ) -> Self {
        Self {
            resource_root: None,
            profile_id: "public-alpha".to_owned(),
            recipe,
            preset,
            cache,
            app_data,
            engine,
            system,
            startup_install_id: None,
            runtime: Arc::new(Mutex::new(BridgeRuntime::default())),
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    fn registry_cards(&self) -> Result<Vec<ManagedInstallCard>, CommandError> {
        self.registry()?.list().map_err(registry_error)
    }

    fn campaign_cards(&self) -> Result<Vec<ManagedCampaignCard>, CommandError> {
        self.registry()?.list_campaigns().map_err(registry_error)
    }

    fn registry(&self) -> Result<ManagedInstallRegistry, CommandError> {
        let app_data = self.app_data.as_ref().ok_or_else(|| {
            CommandError::new(
                "application_data_unavailable",
                "Managed campaign records are unavailable.",
                "Restart Windows normally, then open the installer again.",
                "LOCALAPPDATA is missing or is not an absolute path",
            )
        })?;
        ManagedInstallRegistry::open_or_create(app_data).map_err(registry_error)
    }

    fn resumable_campaign(&self, install_id: &str) -> Result<PathBuf, CommandError> {
        if self
            .registry_cards()?
            .iter()
            .any(|card| card.record.install_id == install_id)
        {
            return Err(CommandError::new(
                "managed_campaign_complete",
                "That managed campaign is already complete.",
                "Launch it from Home instead of resuming the installer.",
                format!("completed managed campaign {install_id:?} cannot resume"),
            ));
        }
        let card = self
            .campaign_cards()?
            .into_iter()
            .find(|card| card.record.install_id == install_id)
            .ok_or_else(|| unknown_managed_install(install_id))?;
        match card.availability {
            CampaignAvailability::Resumable => Ok(card.record.managed_root),
            CampaignAvailability::FreshCopyRequired => Err(CommandError::new(
                "managed_campaign_fresh_copy_required",
                "That interrupted campaign cannot be changed safely.",
                "Build a fresh managed campaign copy and keep this folder for diagnostics.",
                format!("managed campaign {install_id:?} has a fresh-copy seal"),
            )),
            CampaignAvailability::Stale => Err(CommandError::new(
                "managed_campaign_stale",
                "That interrupted campaign folder moved or its ledger changed.",
                "Restore the exact folder or build a fresh managed campaign copy.",
                format!("managed campaign {install_id:?} is stale"),
            )),
        }
    }

    fn available_managed_install(
        &self,
        install_id: &str,
    ) -> Result<ManagedInstallCard, CommandError> {
        let card = self
            .registry_cards()?
            .into_iter()
            .find(|card| card.record.install_id == install_id)
            .ok_or_else(|| unknown_managed_install(install_id))?;
        if card.availability != InstallAvailability::Available {
            return Err(CommandError::new(
                "managed_install_stale",
                "That managed campaign folder moved or changed after it was registered.",
                "Restore the exact folder or build a new managed campaign copy.",
                format!("managed install {install_id:?} is stale"),
            ));
        }
        Ok(card)
    }

    fn diagnostics_managed_root(&self, install_id: &str) -> Result<PathBuf, CommandError> {
        if let Some(root) = self.runtime_lock().known_installs.get(install_id).cloned() {
            return Ok(root);
        }
        Ok(self
            .available_managed_install(install_id)?
            .record
            .managed_root)
    }

    fn runtime_lock(&self) -> std::sync::MutexGuard<'_, BridgeRuntime> {
        self.runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn registered_sources(
        &self,
        bg1_candidate_id: &str,
        bg2_candidate_id: &str,
    ) -> Result<(GameCandidate, GameCandidate), CommandError> {
        let runtime = self.runtime_lock();
        let bg1 = runtime
            .candidates
            .get(bg1_candidate_id)
            .cloned()
            .ok_or_else(|| unregistered_candidate(bg1_candidate_id))?;
        let bg2 = runtime
            .candidates
            .get(bg2_candidate_id)
            .cloned()
            .ok_or_else(|| unregistered_candidate(bg2_candidate_id))?;
        if bg1.role != GameRole::BgeeSod || bg2.role != GameRole::Bg2ee {
            return Err(CommandError::new(
                "game_candidate_role_mismatch",
                "The selected game folders do not fill the required BG1 and BG2 roles.",
                "Select one clean BG:EE + SoD folder and one clean BGII:EE folder.",
                format!("BG1 id role {:?}; BG2 id role {:?}", bg1.role, bg2.role),
            ));
        }
        Ok((bg1, bg2))
    }

    fn reinspect_source(
        &self,
        registered: &GameCandidate,
        expected_role: GameRole,
    ) -> Result<GameCandidate, CommandError> {
        let registered = canonical_candidate(registered.clone())?;
        let inspected = self
            .engine
            .reinspect(&self.recipe, &registered)
            .map_err(CommandError::from_cli)?;
        let inspected = canonical_candidate(inspected)?;
        if inspected.role != expected_role
            || inspected.storefront != registered.storefront
            || inspected.root != registered.root
            || inspected.build != registered.build
            || inspected.fingerprint != registered.fingerprint
            || inspected.eligibility != registered.eligibility
        {
            return Err(CommandError::new(
                "game_candidate_changed",
                "A selected game folder changed after it was inspected.",
                "Inspect the game folders again before reviewing the build.",
                format!("registered {registered:?}; re-inspected {inspected:?}"),
            ));
        }
        if inspected.eligibility != Eligibility::Eligible || inspected.fingerprint.is_none() {
            return Err(CommandError::new(
                "source_not_fresh",
                "A selected source is not a verified clean supported game.",
                "Repair or reinstall that game, then run detection again.",
                format!("candidate at {} is {inspected:?}", inspected.root.display()),
            ));
        }
        Ok(inspected)
    }

    fn validate_review_snapshot(&self, review: &ReviewSnapshot) -> Result<(), CommandError> {
        if Instant::now() > review.expires_at {
            return Err(invalid_review_token());
        }
        let recipe_digest = recipe_directory_digest(&review.request.recipe)?;
        if recipe_digest != review.recipe_digest {
            return Err(changed_review("recipe digest changed"));
        }
        let plan = plan_recipe(
            &review.request.recipe,
            &review.request.preset,
            &review.request.platform,
            &review.request.overrides,
        )
        .map_err(CommandError::from_cli)?;
        let current_selection_digest =
            selection_digest(&plan.evaluation.normalized_selection).map_err(digest_error)?;
        let current_plan_digest = plan_digest(&plan.evaluation.plan).map_err(digest_error)?;
        if current_selection_digest != review.selection_digest
            || current_plan_digest != review.plan_digest
        {
            return Err(changed_review("selection or resolved plan changed"));
        }
        let bg1 = self.reinspect_source(&review.bg1, GameRole::BgeeSod)?;
        let bg2 = self.reinspect_source(&review.bg2, GameRole::Bg2ee)?;
        let destination = inspect_destination_path(
            &review.request.managed_root,
            &bg1.root,
            &bg2.root,
            &review.request.cache,
        )?;
        let current_engine_identity = self
            .engine
            .review(&review.request)
            .map_err(CommandError::from_cli)?;
        if current_engine_identity != review.engine_identity {
            return Err(changed_review("engine-owned install identity changed"));
        }
        let current_digest = review_digest(
            &recipe_digest,
            &current_selection_digest,
            &current_plan_digest,
            &bg1,
            &bg2,
            &destination.path,
            &current_engine_identity,
        )?;
        if current_digest != review.digest {
            return Err(changed_review("frozen review identity changed"));
        }
        Ok(())
    }

    fn spawn_install_worker<F>(
        &self,
        request: InstallCommandRequest,
        expected: InstallReviewIdentity,
        listener: F,
    ) -> Result<StartBuildResponse, CommandError>
    where
        F: Fn(RunEventEnvelope) + Send + Sync + 'static,
    {
        let run_id = self.begin_run()?;
        let run_id_for_worker = run_id.clone();
        let engine = Arc::clone(&self.engine);
        let runtime = Arc::clone(&self.runtime);
        let controls = self.active_controls(&run_id)?;
        let worker = std::thread::Builder::new()
            .name(format!("installer-{run_id}"))
            .spawn(move || {
                let sink = run_sink(&run_id_for_worker, Arc::clone(&runtime), listener);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    engine.install(&request, &expected, &sink, &controls)
                }));
                finish_worker(&runtime, &run_id_for_worker, &sink, result);
            });
        if let Err(error) = worker {
            self.runtime_lock().active_runs.remove(&run_id);
            return Err(CommandError::new(
                "build_worker_failed",
                "The installer could not start its background worker.",
                "Close other installer windows and try again.",
                error.to_string(),
            ));
        }
        Ok(StartBuildResponse { run_id })
    }

    fn spawn_resume_worker<F>(
        &self,
        expected_install_id: String,
        managed_root: PathBuf,
        listener: F,
    ) -> Result<StartBuildResponse, CommandError>
    where
        F: Fn(RunEventEnvelope) + Send + Sync + 'static,
    {
        let run_id = self.begin_run()?;
        let run_id_for_worker = run_id.clone();
        let engine = Arc::clone(&self.engine);
        let runtime = Arc::clone(&self.runtime);
        let controls = self.active_controls(&run_id)?;
        let worker = std::thread::Builder::new()
            .name(format!("installer-{run_id}"))
            .spawn(move || {
                let sink = run_sink(&run_id_for_worker, Arc::clone(&runtime), listener);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    engine.resume(&managed_root, &expected_install_id, &sink, &controls)
                }));
                finish_worker(&runtime, &run_id_for_worker, &sink, result);
            });
        if let Err(error) = worker {
            self.runtime_lock().active_runs.remove(&run_id);
            return Err(CommandError::new(
                "build_worker_failed",
                "The installer could not start its background worker.",
                "Close other installer windows and try again.",
                error.to_string(),
            ));
        }
        Ok(StartBuildResponse { run_id })
    }

    fn begin_run(&self) -> Result<String, CommandError> {
        let run_id = format!(
            "run-{:016x}",
            self.sequence.fetch_add(1, Ordering::Relaxed) + 1
        );
        let controls = RunnerControlHandle::new();
        let mut runtime = self.runtime_lock();
        if !runtime.active_runs.is_empty() || runtime.app_update_active {
            return Err(CommandError::new(
                "build_already_running",
                "Another installer build is already running.",
                "Wait for the current build to finish or cancel it first.",
                "the private alpha permits one active WeiDU campaign per process",
            ));
        }
        runtime.active_runs.insert(run_id.clone(), controls);
        runtime.run_snapshots.insert(
            run_id.clone(),
            RunSnapshotResponse {
                run_id: run_id.clone(),
                status: "running".to_owned(),
                events: Vec::new(),
                report: None,
                error: None,
            },
        );
        Ok(run_id)
    }

    fn active_controls(&self, run_id: &str) -> Result<RunnerControlHandle, CommandError> {
        self.runtime_lock()
            .active_runs
            .get(run_id)
            .cloned()
            .ok_or_else(|| unknown_run(run_id))
    }
}

fn run_sink<F>(run_id: &str, runtime: Arc<Mutex<BridgeRuntime>>, listener: F) -> SequencedEventSink
where
    F: Fn(RunEventEnvelope) + Send + Sync + 'static,
{
    SequencedEventSink::new(run_id, move |envelope| {
        let envelope = project_run_event(envelope);
        {
            let mut state = runtime
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(snapshot) = state.run_snapshots.get_mut(&envelope.run_id) {
                snapshot.events.push(envelope.clone());
                if snapshot.events.len() > SNAPSHOT_EVENT_LIMIT {
                    let overflow = snapshot.events.len() - SNAPSHOT_EVENT_LIMIT;
                    snapshot.events.drain(..overflow);
                }
            }
        }
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| listener(envelope)));
    })
}

fn project_run_event(mut envelope: RunEventEnvelope) -> RunEventEnvelope {
    if let EngineEvent::ManualDownloadNeeded { drop_dir, .. } = &mut envelope.event {
        *drop_dir = display_windows_path_text(drop_dir);
    }
    envelope
}

fn finish_worker(
    runtime: &Arc<Mutex<BridgeRuntime>>,
    run_id: &str,
    sink: &SequencedEventSink,
    result: std::thread::Result<Result<CampaignReport, CliError>>,
) {
    let (report, error) = match result {
        Ok(Ok(report)) if matches!(report.status, bg_engine::cli::CampaignStatus::Complete) => {
            (Some(report), None)
        }
        Ok(Ok(report))
            if matches!(report.status, bg_engine::cli::CampaignStatus::Paused { .. }) =>
        {
            (Some(report), None)
        }
        Ok(Ok(report)) => {
            let error = CommandError::from_cli(CliError::from_campaign(report.clone()));
            (Some(report), Some(error))
        }
        Ok(Err(error)) => {
            let report = error.campaign_report().cloned();
            (report, Some(CommandError::from_cli(error)))
        }
        Err(payload) => {
            let detail = payload
                .downcast_ref::<&str>()
                .map(|value| (*value).to_owned())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "background worker panicked without text".to_owned());
            (
                None,
                Some(CommandError::new(
                    "build_worker_panicked",
                    "The installer worker stopped unexpectedly.",
                    "Keep the managed copy untouched and retry from the installer.",
                    detail,
                )),
            )
        }
    };
    let terminal_message = error.as_ref().map(|error| error.message.clone());
    {
        let mut state = runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.active_runs.remove(run_id);
        if let Some(report) = &report {
            state
                .known_installs
                .insert(report.install_id.clone(), report.managed_root.clone());
        }
        if let Some(snapshot) = state.run_snapshots.get_mut(run_id) {
            snapshot.status = match report.as_ref().map(|report| &report.status) {
                Some(bg_engine::cli::CampaignStatus::Paused { .. }) => "paused".to_owned(),
                _ if error.is_some() => "failed".to_owned(),
                _ => "complete".to_owned(),
            };
            snapshot.report = report;
            snapshot.error = error;
        }
    }
    if let Some(message) = terminal_message {
        sink.emit(EngineEvent::Error {
            step_id: None,
            message,
        });
    }
}

fn selection_overrides(selection: &NormalizedSelection) -> SelectionOverrides {
    let mut overrides = SelectionOverrides::default();
    overrides.features.extend(
        selection.features.iter().map(|(feature, selected)| {
            format!("{feature}={}", if *selected { "on" } else { "off" })
        }),
    );
    for (feature, inputs) in &selection.inputs {
        overrides.inputs.extend(
            inputs
                .iter()
                .map(|(input, value)| format!("{feature}/{input}={}", encode_input_value(value))),
        );
    }
    overrides
}

fn default_application_data_root() -> Option<PathBuf> {
    let root = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
    root.is_absolute()
        .then(|| root.join(APPLICATION_DATA_DIRECTORY))
}

/// Suggests the initial player-facing name and location beneath an injected home path.
///
/// This is intentionally a pure projection. The destination is not created or trusted;
/// the existing destination inspection and frozen-review boundaries re-resolve it before use.
pub fn installation_defaults(home: &Path) -> Result<InstallationDefaultsResponse, CommandError> {
    let name = "Chriz Easy BG".to_owned();
    let path = display_windows_path(&home.join("Games").join(&name))?;
    Ok(InstallationDefaultsResponse { name, path })
}

fn project_evaluation(
    evaluation: bg_engine::recipe_view::SelectionEvaluation,
) -> EvaluateBuildResponse {
    let selected_choice_count = evaluation
        .view
        .controls
        .iter()
        .filter(|control: &&FeatureControl| control.selected)
        .count();
    let plan = PlanSummaryResponse {
        phases: phase_summaries(&evaluation.plan.runs),
    };
    EvaluateBuildResponse {
        view: evaluation.view,
        normalized_selection: evaluation.normalized_selection,
        findings: evaluation.findings,
        plan,
        selected_choice_count,
    }
}

fn canonical_candidate(mut candidate: GameCandidate) -> Result<GameCandidate, CommandError> {
    candidate.root = candidate.root.canonicalize().map_err(|error| {
        path_error(
            "game_path_unavailable",
            "A selected game folder is no longer available.",
            &candidate.root,
            error,
        )
    })?;
    Ok(candidate)
}

fn inspect_destination_path(
    destination: &Path,
    bg1: &Path,
    bg2: &Path,
    cache: &Path,
) -> Result<DestinationEvaluationResponse, CommandError> {
    if !destination.is_absolute()
        || destination
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(unsafe_destination(
            destination,
            "the destination must be one absolute normalized folder path",
        ));
    }
    if is_creator_protected_destination(destination) {
        return Err(unsafe_destination(
            destination,
            "the creator's reference installation is permanently read-only",
        ));
    }
    let normalized = normalize_prospective_path(destination)?;
    let bg1 = bg1.canonicalize().map_err(|error| {
        path_error(
            "game_path_unavailable",
            "The selected BG1 source is no longer available.",
            bg1,
            error,
        )
    })?;
    let bg2 = bg2.canonicalize().map_err(|error| {
        path_error(
            "game_path_unavailable",
            "The selected BG2 source is no longer available.",
            bg2,
            error,
        )
    })?;
    let cache = normalize_prospective_path(cache)?;
    for (label, protected) in [
        ("BG1 source", &bg1),
        ("BG2 source", &bg2),
        ("cache", &cache),
    ] {
        if paths_overlap(&normalized, protected) {
            return Err(unsafe_destination(
                destination,
                &format!("the destination overlaps the {label}"),
            ));
        }
    }
    if normalized.try_exists().map_err(|error| {
        path_error(
            "destination_unavailable",
            "The destination could not be inspected safely.",
            &normalized,
            error,
        )
    })? {
        let metadata = fs::symlink_metadata(&normalized).map_err(|error| {
            path_error(
                "destination_unavailable",
                "The destination could not be inspected safely.",
                &normalized,
                error,
            )
        })?;
        if !metadata.is_dir() || metadata_is_reparse(&metadata) {
            return Err(unsafe_destination(
                destination,
                "the destination must be a direct ordinary directory",
            ));
        }
        let mut entries = fs::read_dir(&normalized).map_err(|error| {
            path_error(
                "destination_unavailable",
                "The destination could not be inspected safely.",
                &normalized,
                error,
            )
        })?;
        if entries
            .next()
            .transpose()
            .map_err(|error| {
                path_error(
                    "destination_unavailable",
                    "The destination could not be inspected safely.",
                    &normalized,
                    error,
                )
            })?
            .is_some()
        {
            return Err(unsafe_destination(
                destination,
                "the destination already contains files and is not an empty new managed copy",
            ));
        }
    }
    Ok(DestinationEvaluationResponse {
        path: path_to_string(&normalized)?,
        safe: true,
        title: "Ready for an isolated managed copy".to_owned(),
        detail: "The engine will repeat authoritative path, lock, source, and disk checks when the build starts.".to_owned(),
    })
}

fn normalize_prospective_path(path: &Path) -> Result<PathBuf, CommandError> {
    let mut cursor = path.to_path_buf();
    let mut missing = Vec::<OsString>::new();
    loop {
        match cursor.try_exists() {
            Ok(true) => break,
            Ok(false) => {
                let name = cursor.file_name().ok_or_else(|| {
                    unsafe_destination(path, "the destination has no existing directory ancestor")
                })?;
                missing.push(name.to_os_string());
                cursor = cursor
                    .parent()
                    .ok_or_else(|| {
                        unsafe_destination(
                            path,
                            "the destination has no existing directory ancestor",
                        )
                    })?
                    .to_path_buf();
            }
            Err(error) => {
                return Err(path_error(
                    "destination_unavailable",
                    "The destination could not be inspected safely.",
                    &cursor,
                    error,
                ));
            }
        }
    }
    for ancestor in cursor.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).map_err(|error| {
            path_error(
                "destination_unavailable",
                "A destination ancestor could not be inspected safely.",
                ancestor,
                error,
            )
        })?;
        if metadata_is_reparse(&metadata) {
            return Err(unsafe_destination(
                path,
                &format!(
                    "destination ancestor {} is a reparse point",
                    ancestor.display()
                ),
            ));
        }
    }
    let mut normalized = cursor.canonicalize().map_err(|error| {
        path_error(
            "destination_unavailable",
            "The destination could not be resolved safely.",
            &cursor,
            error,
        )
    })?;
    for component in missing.into_iter().rev() {
        normalized.push(component);
    }
    Ok(normalized)
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
            .all(|(left, right)| component_eq(left.as_os_str(), right.as_os_str()))
}

fn component_eq(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> bool {
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

#[cfg(windows)]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn recipe_directory_digest(root: &Path) -> Result<String, CommandError> {
    let root = root.canonicalize().map_err(|error| {
        path_error(
            "recipe_load_failed",
            "The installer recipe could not be resolved safely.",
            root,
            error,
        )
    })?;
    let mut files = Vec::new();
    collect_recipe_files(&root, &root, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut bytes = Vec::new();
    for (relative, path) in files {
        bytes.extend_from_slice(relative.as_bytes());
        bytes.push(0);
        let content = fs::read(&path).map_err(|error| {
            path_error(
                "recipe_load_failed",
                "The installer recipe changed while it was being frozen.",
                &path,
                error,
            )
        })?;
        bytes.extend_from_slice(content.len().to_string().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&content);
        bytes.push(0xff);
    }
    Ok(sha256_bytes(&bytes))
}

fn collect_recipe_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<(), CommandError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| {
            path_error(
                "recipe_load_failed",
                "The installer recipe could not be read safely.",
                directory,
                error,
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            path_error(
                "recipe_load_failed",
                "The installer recipe could not be read safely.",
                directory,
                error,
            )
        })?;
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            path_error(
                "recipe_load_failed",
                "The installer recipe contains unreadable filesystem evidence.",
                &path,
                error,
            )
        })?;
        if metadata_is_reparse(&metadata) {
            return Err(CommandError::new(
                "recipe_load_failed",
                "The installer recipe contains an unsafe redirected path.",
                "Repair or replace the installer recipe, then try again.",
                path.display().to_string(),
            ));
        }
        if metadata.is_dir() {
            collect_recipe_files(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = path.strip_prefix(root).expect("walk remains below root");
            let relative = relative.to_str().ok_or_else(|| {
                CommandError::new(
                    "path_encoding_unsupported",
                    "The installer recipe contains a path that cannot be shown safely.",
                    "Move the installer to a folder whose path uses valid Unicode text.",
                    format!("non-Unicode recipe path: {relative:?}"),
                )
            })?;
            files.push((relative.replace('\\', "/"), path));
        }
    }
    Ok(())
}

fn review_digest(
    recipe_digest: &str,
    selection_digest: &str,
    plan_digest: &str,
    bg1: &GameCandidate,
    bg2: &GameCandidate,
    destination: &str,
    engine_identity: &InstallReviewIdentity,
) -> Result<String, CommandError> {
    let bg1_path = path_to_string(&bg1.root)?;
    let bg2_path = path_to_string(&bg2.root)?;
    let engine_managed_root = path_to_string(&engine_identity.managed_root)?;
    let engine_cache_root = path_to_string(&engine_identity.cache_root)?;
    let identity = format!(
        "review-v3\0{recipe_digest}\0{selection_digest}\0{plan_digest}\0{:?}\0{:?}\0{}\0{}\0{:?}\0{:?}\0{}\0{}\0{destination}\0{}\0{}\0{}\0{}\0{}\0{}\0{engine_managed_root}\0{engine_cache_root}",
        bg1.role,
        bg1.storefront,
        bg1_path,
        bg1.fingerprint.as_deref().unwrap_or_default(),
        bg2.role,
        bg2.storefront,
        bg2_path,
        bg2.fingerprint.as_deref().unwrap_or_default(),
        engine_identity.recipe_payload_sha256,
        engine_identity.selection_sha256,
        engine_identity.plan_sha256,
        engine_identity.source_games.bg1,
        engine_identity.source_games.bg2,
        engine_identity.display_name,
    );
    Ok(sha256_bytes(identity.as_bytes()))
}

fn path_to_string(path: &Path) -> Result<String, CommandError> {
    path.to_str().map(str::to_owned).ok_or_else(|| {
        CommandError::new(
            "path_encoding_unsupported",
            "A native path cannot be represented safely in the interface.",
            "Choose a folder whose path uses valid Unicode text.",
            format!("non-Unicode native path: {path:?}"),
        )
    })
}

/// Formats a native path for display without changing the canonical path used by the engine.
pub fn display_windows_path(path: &Path) -> Result<String, CommandError> {
    let path = path_to_string(path)?;
    Ok(display_windows_path_text(&path))
}

fn display_windows_path_text(path: &str) -> String {
    if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{unc}");
    }
    path.strip_prefix(r"\\?\")
        .map_or_else(|| path.to_owned(), str::to_owned)
}

fn unix_nanos() -> Result<u128, CommandError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| {
            CommandError::new(
                "system_clock_invalid",
                "The system clock cannot create a review token.",
                "Correct the system clock and review the build again.",
                error.to_string(),
            )
        })
}

fn digest_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "review_digest_failed",
        "The installer could not freeze the exact reviewed build.",
        "Review the selected options and try again.",
        error.to_string(),
    )
}

fn path_error(
    code: &str,
    message: &str,
    path: &Path,
    error: impl std::fmt::Display,
) -> CommandError {
    CommandError::new(
        code,
        message,
        "Choose an accessible local folder and try again.",
        format!("{}: {error}", path.display()),
    )
}

fn unsafe_destination(path: &Path, reason: &str) -> CommandError {
    CommandError::new(
        "destination_unsafe",
        "That folder cannot be used for a new isolated installation.",
        "Choose a new empty local folder outside both source games and the installer cache.",
        format!("{}: {reason}", path.display()),
    )
}

fn unregistered_candidate(candidate_id: &str) -> CommandError {
    CommandError::new(
        "game_candidate_unknown",
        "That game selection is no longer available.",
        "Run game detection or inspect the folder again.",
        format!("unregistered candidate id {candidate_id:?}"),
    )
}

fn invalid_review_token() -> CommandError {
    CommandError::new(
        "review_token_invalid",
        "That reviewed build is no longer available to start.",
        "Return to Review and freeze the current build again.",
        "review token is unknown, expired, or was already used",
    )
}

fn invalid_manual_contract(artifact_id: &str, missing: &str) -> CommandError {
    CommandError::new(
        "manual_archive_contract_invalid",
        "The requested manual archive is not frozen completely in this recipe.",
        "Install a corrected recipe release before continuing.",
        format!("manual artifact {artifact_id:?} has no valid {missing}"),
    )
}

fn inspect_manual_cache_file(
    path: &Path,
    expected_sha256: &str,
    expected_length: u64,
    filename: &str,
) -> (bool, Option<String>) {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return (false, Some(format!("{filename} has not been supplied yet")));
        }
        Err(error) => {
            return (
                false,
                Some(format!("could not inspect {filename}: {error}")),
            );
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return (
            false,
            Some(format!("{filename} is not a regular cache file")),
        );
    }
    if metadata.len() != expected_length {
        return (
            false,
            Some(format!(
                "{filename} has length {}; expected {expected_length}",
                metadata.len()
            )),
        );
    }
    match bg_engine::acquire::provide_manual_archive(path, expected_sha256) {
        Ok(verified) if verified.length == expected_length => (true, None),
        Ok(verified) => (
            false,
            Some(format!(
                "{filename} has length {}; expected {expected_length}",
                verified.length
            )),
        ),
        Err(_) => (
            false,
            Some(format!(
                "{filename} does not match the exact archive required by this recipe"
            )),
        ),
    }
}

fn changed_review(reason: &str) -> CommandError {
    CommandError::new(
        "review_changed",
        "The build changed after Review and was not started.",
        "Inspect the sources and destination, then review the build again.",
        reason,
    )
}

fn unknown_run(run_id: &str) -> CommandError {
    CommandError::new(
        "run_unknown",
        "That installer run is not active in this session.",
        "Return to the current Build screen and try again.",
        format!("unknown process-local run id {run_id:?}"),
    )
}

fn unknown_managed_install(install_id: &str) -> CommandError {
    CommandError::new(
        "managed_install_unknown",
        "That managed campaign is not registered on this computer.",
        "Return to Home and select a campaign discovered by the installer.",
        format!("unknown managed install id {install_id:?}"),
    )
}

fn registry_error(error: bg_engine::registry::RegistryError) -> CommandError {
    CommandError::new(
        "managed_install_registry_failed",
        "Managed campaign records could not be verified.",
        "Keep the campaign folders unchanged and retain this error for diagnosis.",
        error.to_string(),
    )
}

fn encode_input_value(value: &InputValue) -> String {
    match value {
        InputValue::Boolean(value) => format!("boolean:{value}"),
        InputValue::Choice(value) => format!("choice:{value}"),
        InputValue::Integer(value) => format!("integer:{value}"),
    }
}

fn phase_summaries(runs: &[bg_engine::resolve::PlannedRun]) -> Vec<PhaseSummaryResponse> {
    let mut seen = BTreeSet::new();
    runs.iter()
        .filter(|run| seen.insert(run.phase))
        .map(|run| phase_summary(run.phase))
        .collect()
}

fn phase_summary(phase: Phase) -> PhaseSummaryResponse {
    let (id, title, detail) = match phase {
        Phase::Bg1Preparation => (
            "bg1-preparation",
            "Prepare the BG1 campaign",
            "Apply the pinned pre-merge recipe to the separate BG1 copy.",
        ),
        Phase::Bg2Preparation => (
            "bg2-preparation",
            "Prepare the BG2 campaign",
            "Apply the pinned pre-merge recipe to the separate BG2 copy.",
        ),
        Phase::EetInitialization => (
            "eet-initialization",
            "Merge with EET",
            "Import the prepared BG1 campaign into the BG2 campaign.",
        ),
        Phase::Main => (
            "main",
            "Build the main campaign",
            "Install the selected curated content and rules in order.",
        ),
        Phase::EetFinalization => (
            "eet-finalization",
            "Finalize EET",
            "Finalize the merged campaign after the main recipe.",
        ),
        Phase::PostEetEnd => (
            "post-eet-end",
            "Apply the reviewed tail",
            "Install the explicitly audited post-finalization additions.",
        ),
    };
    PhaseSummaryResponse {
        id: id.to_owned(),
        title: title.to_owned(),
        detail: detail.to_owned(),
    }
}

fn project_candidate(candidate: GameCandidate) -> Result<GameCandidateResponse, CommandError> {
    let canonical_path = path_to_string(&candidate.root)?;
    let display_path = display_windows_path(&candidate.root)?;
    let freshness = candidate_freshness(&candidate);
    let role = match candidate.role {
        GameRole::BgeeSod => "BG:EE + SoD",
        GameRole::Bg2ee => "BGII:EE",
    };
    let (storefront_id, storefront) = match candidate.storefront {
        Storefront::Steam => ("steam", "Steam"),
        Storefront::Gog => ("gog", "GOG"),
    };
    let status = match freshness {
        CandidateFreshness::Fresh => "clean",
        CandidateFreshness::Modified => "modified",
        CandidateFreshness::UnsupportedVersion => "unsupported version",
        CandidateFreshness::MissingSod => "SoD missing",
        CandidateFreshness::UnverifiedStorefront => "verify storefront",
        CandidateFreshness::UnknownFingerprint => "unknown files",
        CandidateFreshness::UnsupportedLocale => "unsupported language",
    };
    let identity = format!(
        "{:?}\0{:?}\0{canonical_path}",
        candidate.role, candidate.storefront
    );
    Ok(GameCandidateResponse {
        id: format!("game-{}", &sha256_bytes(identity.as_bytes())[..20]),
        label: format!("{role} — {storefront} — {status}"),
        path: display_path,
        storefront: storefront_id.to_owned(),
        build: candidate.build,
        freshness,
        eligible: candidate.eligibility == Eligibility::Eligible,
        findings: candidate
            .findings
            .into_iter()
            .map(|finding| finding.message)
            .collect(),
    })
}

fn project_managed_install(
    card: ManagedInstallCard,
) -> Result<ManagedInstallationResponse, CommandError> {
    let available = card.availability == InstallAvailability::Available;
    let record = card.record;
    let consistency = available.then(|| crate::consistency::inspect(&record.managed_root));
    let radar_version = if available {
        bg_engine::radar::status(&record.managed_root, record.managed_root.join("game"), None)
            .ok()
            .and_then(|status| status.installed_version)
    } else {
        None
    };
    let path = display_windows_path(&record.managed_root)?;
    let receipt = record.managed_root.join(".chriz/install-receipt.json");
    let receipt_path = display_windows_path(&receipt)?;
    let launch_path = display_windows_path(&record.launch_path)?;
    Ok(ManagedInstallationResponse {
        id: record.install_id,
        name: record.display_name,
        path,
        status: if available {
            "Ready to play".to_owned()
        } else {
            "Unavailable — folder moved or changed".to_owned()
        },
        receipt_path: Some(receipt_path),
        launch_path: Some(launch_path),
        completed_at_millis: Some(record.completed_at_millis),
        available,
        resumable: false,
        recipe_version: Some(record.recipe_version),
        consistency,
        radar_version,
    })
}

fn project_managed_campaign(
    card: ManagedCampaignCard,
) -> Result<ManagedInstallationResponse, CommandError> {
    let path = display_windows_path(&card.record.managed_root)?;
    let (status, resumable) = match card.availability {
        CampaignAvailability::Resumable => ("Build interrupted — ready to resume", true),
        CampaignAvailability::FreshCopyRequired => ("Fresh copy required", false),
        CampaignAvailability::Stale => ("Build unavailable — folder moved or changed", false),
    };
    Ok(ManagedInstallationResponse {
        id: card.record.install_id,
        name: "Chriz Easy BG — incomplete installation".to_owned(),
        path,
        status: status.to_owned(),
        receipt_path: None,
        launch_path: None,
        completed_at_millis: None,
        consistency: None,
        radar_version: None,
        available: false,
        resumable,
        recipe_version: None,
    })
}

fn candidate_freshness(candidate: &GameCandidate) -> CandidateFreshness {
    for (kind, freshness) in [
        (FindingKind::Modified, CandidateFreshness::Modified),
        (
            FindingKind::UnsupportedVersion,
            CandidateFreshness::UnsupportedVersion,
        ),
        (FindingKind::MissingSod, CandidateFreshness::MissingSod),
        (
            FindingKind::UnsupportedLocale,
            CandidateFreshness::UnsupportedLocale,
        ),
        (
            FindingKind::UnknownFingerprint,
            CandidateFreshness::UnknownFingerprint,
        ),
        (
            FindingKind::UnverifiedStorefront,
            CandidateFreshness::UnverifiedStorefront,
        ),
    ] {
        if candidate
            .findings
            .iter()
            .any(|finding| finding.kind == kind)
        {
            return freshness;
        }
    }
    CandidateFreshness::Fresh
}

fn sort_candidates(candidates: &mut [GameCandidateResponse]) {
    candidates.sort_by(|left, right| {
        right
            .eligible
            .cmp(&left.eligible)
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.path.cmp(&right.path))
    });
}
