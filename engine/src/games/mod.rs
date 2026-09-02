//! Read-only source-game discovery and profile-driven freshness inspection.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod discovery;
pub mod probe;
pub mod profile;

pub use discovery::{
    discover_games, discover_installed_games, RegistryHive, RegistryProvider, SystemRegistry,
};
pub use probe::{
    inspect_game_path, DirectoryEntry, FileKind, FileSystemProvider, SystemFileSystem,
};
pub use profile::{
    AllowedCleanVariant, ExpectedInventoryEntry, FileFingerprint, ForbiddenResiduePattern,
    GameProfile, GameProfiles, InventoryKind, InventorySurface,
};

/// Logical source-game role in an EET build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameRole {
    /// Baldur's Gate: Enhanced Edition with Siege of Dragonspear.
    BgeeSod,
    /// Baldur's Gate II: Enhanced Edition.
    Bg2ee,
}

/// Storefront that owns a discovered source installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Storefront {
    /// Valve Steam installation.
    Steam,
    /// GOG installation.
    Gog,
}

/// Overall source eligibility derived after all findings have been accumulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Eligibility {
    /// Matches a verified Steam build profile and is suitable for staging.
    Eligible,
    /// Matches a clean profile but its storefront has not completed release rehearsal.
    Experimental,
    /// Has one or more findings that forbid use as an installation source.
    Ineligible,
}

/// Stable category for one source-game inspection finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    /// The source matches one complete verified clean variant.
    Fresh,
    /// A mod-sensitive surface differs from its clean variant.
    Modified,
    /// The executable ProductVersion is not represented by a supported profile.
    UnsupportedVersion,
    /// A required Siege of Dragonspear payload is absent or unsafe.
    MissingSod,
    /// The storefront is discoverable but has not completed release acceptance.
    UnverifiedStorefront,
    /// Core fingerprints do not match any independently authored clean variant.
    UnknownFingerprint,
    /// The candidate cannot prove the sole alpha locale, `en_US`.
    UnsupportedLocale,
}

/// One user-visible reason contributing to source eligibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameFinding {
    /// Stable machine-readable finding category.
    pub kind: FindingKind,
    /// Human-readable explanation which never hides simultaneous findings.
    pub message: String,
    /// Relative or absolute paths that supplied the evidence, when applicable.
    pub paths: Vec<PathBuf>,
}

/// One discovered or browsed source-game candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameCandidate {
    /// Role this source can fill in the EET build.
    pub role: GameRole,
    /// Storefront-specific profile used for inspection.
    pub storefront: Storefront,
    /// Canonical read-only source root.
    pub root: PathBuf,
    /// Four-part executable ProductVersion, when it could be read.
    pub build: Option<String>,
    /// Overall eligibility derived from all findings.
    pub eligibility: Eligibility,
    /// Every applicable finding, in deterministic safety-first order.
    pub findings: Vec<GameFinding>,
    /// Deterministic digest of the observed profiled files, when complete.
    pub fingerprint: Option<String>,
}

/// Failure while loading profiles, discovering stores, or inspecting a candidate.
#[derive(Debug, Error)]
pub enum GameError {
    /// A filesystem operation failed at a named path.
    #[error("{path}: {source}")]
    Io {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A profile file was not valid TOML.
    #[error("game profile parse error in {path}: {source}")]
    ProfileParse {
        /// Profile path that could not be parsed.
        path: PathBuf,
        /// Underlying TOML error.
        #[source]
        source: toml::de::Error,
    },
    /// Parsed profile content violated a safety invariant.
    #[error("invalid game profile {path}: {message}")]
    InvalidProfile {
        /// Profile path containing the invalid value.
        path: PathBuf,
        /// Explanation of the rejected invariant.
        message: String,
    },
    /// Valve KeyValues metadata was malformed or unsafe.
    #[error("invalid Steam metadata {path}: {message}")]
    SteamMetadata {
        /// VDF or ACF path containing the invalid value.
        path: PathBuf,
        /// Explanation of the parse or containment failure.
        message: String,
    },
    /// Registry access failed at a named logical key/value.
    #[error("registry {location}: {source}")]
    Registry {
        /// Hive/key/value description.
        location: String,
        /// Underlying registry-provider error.
        #[source]
        source: std::io::Error,
    },
    /// No profile exists for an explicitly requested role/storefront pair.
    #[error("no game profile for {role:?} on {storefront:?}")]
    MissingProfile {
        /// Requested game role.
        role: GameRole,
        /// Requested storefront.
        storefront: Storefront,
    },
}

/// Result type for source-game discovery and inspection.
pub type Result<T> = std::result::Result<T, GameError>;
