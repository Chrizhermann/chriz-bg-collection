//! Signed update projections owned by the native application.

use std::io::{Cursor, Read};
use std::sync::Mutex;

use bg_engine::recipe_envelope::{
    verify_recipe_package, RecipeTrustError, RecipeTrustStore, VerifiedRecipe,
};
use bg_engine::updates::{
    classify_updates, RecipeChange, RecipeRelease, SaveApplicability, UpdateDisposition, Urgency,
};
use semver::Version;
use serde::Serialize;
use zip::ZipArchive;

/// Trust outcome for the newest recipe advertised by the alpha channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecipeUpdateState {
    UpToDate,
    Available,
    RequiresApp,
    Offline,
    Invalid,
    Replayed,
    Unavailable,
}

/// One authored recipe change. These are generic release notes, never save-inspection results.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeChangeResponse {
    pub title: String,
    pub summary: String,
    pub save_applicability: String,
    pub urgency: String,
    pub condition_note: Option<String>,
}

/// Recipe update status crossing the native command boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeUpdateResponse {
    pub state: RecipeUpdateState,
    pub current_version: String,
    pub available_version: Option<String>,
    pub minimum_app_version: Option<String>,
    pub disposition: String,
    pub detail: String,
    pub changes: Vec<RecipeChangeResponse>,
}

/// Application updater result. The signed Tauri channel is independent from recipe trust.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationUpdateResponse {
    pub state: String,
    pub current_version: String,
    pub available_version: Option<String>,
    pub detail: String,
    pub release_notes: Option<String>,
}

/// Recipe-relative status for one immutable managed campaign record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedCopyUpdateResponse {
    pub install_id: String,
    pub name: String,
    pub path: String,
    pub installed_recipe_version: Option<String>,
    pub state: String,
    pub detail: String,
}

/// Complete native projection for the Updates screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateCenterResponse {
    pub checked_at: Option<String>,
    pub network_state: String,
    pub application: ApplicationUpdateResponse,
    pub recipe: RecipeUpdateResponse,
    pub managed_copies: Vec<ManagedCopyUpdateResponse>,
    pub radar: Option<RadarUpdateResponse>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RadarUpdateResponse {
    pub state: String,
    pub current_version: Option<String>,
    pub available_version: Option<String>,
    pub detail: String,
    pub release_notes: Option<String>,
}

/// In-memory trust boundary for a checked recipe candidate.
///
/// A successful check retains exact verified bytes. Invalid, replayed, and app-incompatible
/// candidates never replace the retained candidate.
pub struct RecipeUpdateManager {
    trust: RecipeTrustStore,
    current_version_text: String,
    running_app_version: String,
    staged: Mutex<Option<VerifiedRecipe>>,
}

impl RecipeUpdateManager {
    /// Creates a verifier for one installed recipe and running application identity.
    pub fn new(
        trust: RecipeTrustStore,
        current_version: impl Into<String>,
        running_app_version: impl Into<String>,
    ) -> Result<Self, String> {
        let current_version_text = current_version.into();
        Version::parse(&current_version_text)
            .map_err(|error| format!("invalid current recipe version: {error}"))?;
        let running_app_version = running_app_version.into();
        Version::parse(&running_app_version)
            .map_err(|error| format!("invalid running app version: {error}"))?;
        Ok(Self {
            trust,
            current_version_text,
            running_app_version,
            staged: Mutex::new(None),
        })
    }

    /// Verifies and classifies exact channel bytes without mutating the active recipe.
    pub fn check_candidate(
        &self,
        payload: &[u8],
        envelope: &[u8],
        signature: &[u8],
    ) -> RecipeUpdateResponse {
        let verified = match verify_recipe_package(
            payload,
            envelope,
            signature,
            &self.trust,
            &self.running_app_version,
            Some(&self.current_version_text),
        ) {
            Ok(verified) => verified,
            Err(RecipeTrustError::MinimumApp { required, .. }) => {
                return RecipeUpdateResponse {
                    state: RecipeUpdateState::RequiresApp,
                    current_version: self.current_version_text.clone(),
                    available_version: signed_envelope_version(envelope),
                    minimum_app_version: Some(required.clone()),
                    disposition: "app-update-required".to_owned(),
                    detail: format!(
                        "Update the application to {required} or newer before using this signed recipe."
                    ),
                    changes: Vec::new(),
                };
            }
            Err(RecipeTrustError::Replay(_)) => {
                return rejected_response(
                    RecipeUpdateState::Replayed,
                    &self.current_version_text,
                    "The recipe channel returned an older or already trusted release. The current trusted recipe was kept.",
                );
            }
            Err(error) => {
                return rejected_response(
                    RecipeUpdateState::Invalid,
                    &self.current_version_text,
                    &format!(
                        "The downloaded recipe could not be verified. The current trusted recipe was kept. ({error})"
                    ),
                );
            }
        };
        let release = match bound_release(&verified.payload_bytes, &verified.envelope.version) {
            Ok(release) => release,
            Err(error) => {
                return rejected_response(
                    RecipeUpdateState::Invalid,
                    &self.current_version_text,
                    &format!(
                        "The signed recipe ledger could not be read. The current trusted recipe was kept. ({error})"
                    ),
                );
            }
        };
        let summary = match classify_updates(
            &self.current_version_text,
            &self.running_app_version,
            std::slice::from_ref(&release),
        ) {
            Ok(summary) => summary,
            Err(error) => {
                return rejected_response(
                    RecipeUpdateState::Invalid,
                    &self.current_version_text,
                    &format!(
                        "The signed recipe update chain is invalid. The current trusted recipe was kept. ({error})"
                    ),
                );
            }
        };
        let response = RecipeUpdateResponse {
            state: RecipeUpdateState::Available,
            current_version: self.current_version_text.clone(),
            available_version: summary.latest_version,
            minimum_app_version: summary.minimum_app_version,
            disposition: disposition_id(summary.disposition).to_owned(),
            detail: disposition_detail(summary.disposition).to_owned(),
            changes: summary.changes.iter().map(project_change).collect(),
        };
        *self
            .staged
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(verified);
        response
    }

    /// Version of the exact verified candidate retained for a later new-copy action.
    pub fn staged_version(&self) -> Option<String> {
        self.staged
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|recipe| recipe.envelope.version.clone())
    }

    /// Copies the retained exact candidate only when the caller names its signed version.
    pub fn staged_recipe(&self, version: &str) -> Option<VerifiedRecipe> {
        self.staged
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .filter(|recipe| recipe.envelope.version == version)
            .cloned()
    }
}

fn rejected_response(
    state: RecipeUpdateState,
    current_version: &str,
    detail: &str,
) -> RecipeUpdateResponse {
    RecipeUpdateResponse {
        state,
        current_version: current_version.to_owned(),
        available_version: None,
        minimum_app_version: None,
        disposition: "unknown-applicability".to_owned(),
        detail: detail.to_owned(),
        changes: Vec::new(),
    }
}

fn signed_envelope_version(bytes: &[u8]) -> Option<String> {
    serde_json::from_slice::<bg_engine::recipe_envelope::RecipeEnvelope>(bytes)
        .ok()
        .map(|envelope| envelope.version)
}

fn bound_release(payload: &[u8], version: &str) -> Result<RecipeRelease, String> {
    let mut archive = ZipArchive::new(Cursor::new(payload)).map_err(|error| error.to_string())?;
    let path = format!("releases/v{version}/ledger.toml");
    let mut entry = archive.by_name(&path).map_err(|error| error.to_string())?;
    if entry.size() > 1024 * 1024 {
        return Err("release ledger exceeds 1 MiB".to_owned());
    }
    let mut text = String::new();
    entry
        .read_to_string(&mut text)
        .map_err(|error| error.to_string())?;
    toml::from_str(&text).map_err(|error| error.to_string())
}

fn project_change(change: &RecipeChange) -> RecipeChangeResponse {
    RecipeChangeResponse {
        title: change.title.clone(),
        summary: change.summary.clone(),
        save_applicability: applicability_id(change.save_applicability).to_owned(),
        urgency: urgency_id(change.urgency).to_owned(),
        condition_note: change.condition_note.clone(),
    }
}

fn applicability_id(value: SaveApplicability) -> &'static str {
    match value {
        SaveApplicability::CurrentSave => "current-save",
        SaveApplicability::BeforeNpcJoin => "before-npc-join",
        SaveApplicability::BeforeAreaVisit => "before-area-visit",
        SaveApplicability::BeforeEvent => "before-event",
        SaveApplicability::NextPlaythrough => "next-playthrough",
        SaveApplicability::NewGameOnly => "new-game-only",
        SaveApplicability::Unknown => "unknown",
    }
}

fn urgency_id(value: Urgency) -> &'static str {
    match value {
        Urgency::Critical => "critical",
        Urgency::Recommended => "recommended",
        Urgency::Optional => "optional",
        Urgency::Informational => "informational",
    }
}

fn disposition_id(value: UpdateDisposition) -> &'static str {
    match value {
        UpdateDisposition::UpToDate => "up-to-date",
        UpdateDisposition::DeferredForNextPlaythrough => "deferred-for-next-playthrough",
        UpdateDisposition::MayAffectCurrentPlaythrough => "may-affect-current-playthrough",
        UpdateDisposition::UnknownApplicability => "unknown-applicability",
        UpdateDisposition::AppUpdateRequired => "app-update-required",
    }
}

fn disposition_detail(value: UpdateDisposition) -> &'static str {
    match value {
        UpdateDisposition::UpToDate => "The trusted recipe is current.",
        UpdateDisposition::DeferredForNextPlaythrough => {
            "Useful for a future playthrough. Existing campaigns and saves stay unchanged."
        }
        UpdateDisposition::MayAffectCurrentPlaythrough => {
            "The author marked changes that may matter to an existing playthrough. The installer did not inspect any save."
        }
        UpdateDisposition::UnknownApplicability => {
            "Save applicability is unknown. The installer did not inspect any save."
        }
        UpdateDisposition::AppUpdateRequired => {
            "A newer application is required before this recipe can be used."
        }
    }
}
