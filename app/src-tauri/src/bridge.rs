//! Read-only native adapter over the engine's presentation-neutral CLI operations.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bg_engine::cli::{
    discover_games as engine_discover_games, inspect_game, plan_recipe, validate_recipe, CliError,
    SelectionOverrides, ValidationProfile,
};
use bg_engine::digest::sha256_bytes;
use bg_engine::games::{Eligibility, FindingKind, GameCandidate, GameRole, Storefront};
use bg_engine::manifest::{InputValue, Phase};
use bg_engine::recipe_view::{FeatureControl, NormalizedSelection, RecipeView, SelectionFinding};
use serde::Serialize;

use crate::error::CommandError;

type Discoverer = dyn Fn(&Path) -> Result<Vec<GameCandidate>, CliError> + Send + Sync;

/// Immutable native adapter configuration shared by Tauri commands.
#[derive(Clone)]
pub struct NativeBridge {
    recipe: PathBuf,
    preset: String,
    discoverer: Arc<Discoverer>,
}

/// Native backend identity returned during application startup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapResponse {
    /// Always `native` for this bridge.
    pub mode: String,
    /// Compiled engine crate version.
    pub engine_version: String,
    /// Signed recipe release version; absent until Task 20 supplies it.
    pub recipe_version: Option<String>,
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
    /// Canonical source directory.
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
        Self::with_discoverer(recipe, preset, engine_discover_games)
    }

    /// Creates the production adapter from Tauri's trusted packaged-resource directory.
    pub fn from_resource_dir(resource_dir: &Path) -> Self {
        Self::new(resource_dir.join("manifest"), "chris-recommended")
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
        Self {
            recipe: recipe.into(),
            preset: preset.into(),
            discoverer: Arc::new(discoverer),
        }
    }

    /// Validates the complete public-alpha recipe and its configured preset.
    pub fn bootstrap(&self) -> Result<BootstrapResponse, CommandError> {
        validate_recipe(&self.recipe, ValidationProfile::PublicAlpha)
            .map_err(CommandError::from_cli)?;
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
            recipe_version: None,
        })
    }

    /// Discovers and independently groups installed BG1 and BG2 sources.
    pub fn discover_games(&self) -> Result<GameDiscoveryResponse, CommandError> {
        let candidates = (self.discoverer)(&self.recipe).map_err(CommandError::from_cli)?;
        let mut bg1_candidates = Vec::new();
        let mut bg2_candidates = Vec::new();
        for candidate in candidates {
            let role = candidate.role;
            let response = project_candidate(candidate)?;
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
        let report = inspect_game(&self.recipe, role, path).map_err(CommandError::from_cli)?;
        project_candidate(report.candidate)
    }

    /// Evaluates the configured preset plus a normalized semantic frontend selection.
    pub fn evaluate_build(
        &self,
        selection: &NormalizedSelection,
    ) -> Result<EvaluateBuildResponse, CommandError> {
        let mut overrides = SelectionOverrides::default();
        overrides
            .features
            .extend(selection.features.iter().map(|(feature, selected)| {
                format!("{feature}={}", if *selected { "on" } else { "off" })
            }));
        for (feature, inputs) in &selection.inputs {
            overrides.inputs.extend(
                inputs.iter().map(|(input, value)| {
                    format!("{feature}/{input}={}", encode_input_value(value))
                }),
            );
        }
        let evaluation = plan_recipe(&self.recipe, &self.preset, &selection.platform, &overrides)
            .map_err(CommandError::from_cli)?
            .evaluation;
        let selected_choice_count = evaluation
            .view
            .controls
            .iter()
            .filter(|control: &&FeatureControl| control.selected)
            .count();
        let plan = PlanSummaryResponse {
            phases: phase_summaries(&evaluation.plan.runs),
        };
        Ok(EvaluateBuildResponse {
            view: evaluation.view,
            normalized_selection: evaluation.normalized_selection,
            findings: evaluation.findings,
            plan,
            selected_choice_count,
        })
    }
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
    let path = candidate.root.to_str().ok_or_else(|| {
        CommandError::new(
            "path_encoding_unsupported",
            "A detected game path cannot be represented safely in the interface.",
            "Choose a game folder whose path uses valid Unicode text.",
            format!("non-Unicode native path: {:?}", candidate.root),
        )
    })?;
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
    let identity = format!("{:?}\0{:?}\0{path}", candidate.role, candidate.storefront);
    Ok(GameCandidateResponse {
        id: format!("game-{}", &sha256_bytes(identity.as_bytes())[..20]),
        label: format!("{role} — {storefront} — {status}"),
        path: path.to_owned(),
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
