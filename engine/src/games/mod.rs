//! Read-only source-game discovery and profile-driven freshness inspection.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod discovery;
pub mod probe;
pub mod profile;

pub use discovery::{
    discover_games, discover_games_with, discover_installed_games, RegistryHive, RegistryProvider,
    SystemRegistry, BEAMDOG_FOLDERS, GOG_GAME_IDS,
};
pub use probe::{
    detect_storefront, inspect_game_path, inspect_game_path_with_language, left_out_root_patterns,
    resolve_game_executable, DirectoryEntry, FileKind, FileSystemProvider, SystemFileSystem,
};
pub use profile::{
    AllowedCleanVariant, ExpectedInventoryEntry, FileFingerprint, ForbiddenResiduePattern,
    GameProfile, GameProfiles, InventoryKind, InventorySurface,
};

/// Logical source-game role: one of the two EET sources, or a standalone game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameRole {
    /// Baldur's Gate: Enhanced Edition with Siege of Dragonspear.
    BgeeSod,
    /// Baldur's Gate II: Enhanced Edition.
    Bg2ee,
    /// Icewind Dale: Enhanced Edition, installed as a standalone single-source collection.
    Iwdee,
}

impl GameRole {
    /// Returns the full player-facing game name.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::BgeeSod => "Baldur's Gate: Enhanced Edition",
            Self::Bg2ee => "Baldur's Gate II: Enhanced Edition",
            Self::Iwdee => "Icewind Dale: Enhanced Edition",
        }
    }

    /// Returns a short player-facing game label.
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::BgeeSod => "BGEE+SoD",
            Self::Bg2ee => "BG2EE",
            Self::Iwdee => "IWD:EE",
        }
    }

    /// Returns `true` for a role that is installed alone instead of as one half of EET.
    pub const fn is_standalone(self) -> bool {
        matches!(self, Self::Iwdee)
    }

    /// Canonical Windows executable name expected by mods, InfinityLoader and overlays.
    pub const fn canonical_executable(self) -> &'static str {
        match self {
            Self::BgeeSod | Self::Bg2ee => "Baldur.exe",
            Self::Iwdee => "icewind.exe",
        }
    }

    /// Known storefront executable names for this game role, canonical name first.
    ///
    /// GOG's 2.7.3.0 Baldur's Gate II: Enhanced Edition build ships `BaldurII.exe` instead of
    /// `Baldur.exe`; Siege of Dragonspear and older/repackaged releases have also used
    /// role-specific names.
    pub const fn known_executables(self) -> &'static [&'static str] {
        match self {
            Self::BgeeSod => &[
                "Baldur.exe",
                "SiegeOfDragonspear.exe",
                "BaldursGate.exe",
                "BGEE.exe",
            ],
            Self::Bg2ee => &[
                "Baldur.exe",
                "BaldurII.exe",
                "BaldursGateII.exe",
                "Baldur2.exe",
                "BG2EE.exe",
            ],
            Self::Iwdee => &["icewind.exe", "IcewindDale.exe", "IWDEE.exe", "Baldur.exe"],
        }
    }
}

/// Storefront that owns a discovered source installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Storefront {
    /// Valve Steam installation.
    Steam,
    /// GOG installation, from the GOG installer or GOG Galaxy.
    Gog,
    /// Beamdog client installation (the client closed in December 2025; installed games stay).
    Beamdog,
    /// Microsoft Store or Xbox app installation.
    Microsoft,
    /// A folder whose store CEBG cannot tell, such as a copied or moved game.
    Other,
}

impl Storefront {
    /// Player-facing store name.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Steam => "Steam",
            Self::Gog => "GOG",
            Self::Beamdog => "Beamdog",
            Self::Microsoft => "Microsoft Store",
            Self::Other => "Other",
        }
    }
}

/// Overall source eligibility derived after all findings have been accumulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Eligibility {
    /// Matches the tested clean game (store, version and files) and is suitable for staging.
    Eligible,
    /// Usable as a source, but it differs from the tested game: another store, version,
    /// language or changed files. Its findings say how; the installation continues.
    Experimental,
    /// Has one or more findings that make an installation from it impossible.
    Ineligible,
}

impl Eligibility {
    /// Whether an installation may use this source: tested, or usable with warnings.
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Eligible | Self::Experimental)
    }
}

/// Stable category for one source-game inspection finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    /// The source matches one complete verified clean variant.
    Fresh,
    /// A mod-sensitive surface differs from its clean variant. A warning.
    Modified,
    /// The executable ProductVersion is not the tested one. A warning.
    UnsupportedVersion,
    /// A required Siege of Dragonspear payload is absent or unsafe. Blocks.
    MissingSod,
    /// The store is not the tested one (Steam). A warning.
    UnverifiedStorefront,
    /// Core fingerprints do not match the tested clean game. A warning.
    UnknownFingerprint,
    /// The game is set to a language other than English. A warning.
    UnsupportedLocale,
    /// Leftovers of earlier mod installs, which the copy leaves out. A warning.
    LeftOut,
    /// Siege of Dragonspear is already merged into the folder, as in Beamdog copies. A warning.
    SodMerged,
    /// Files every installation needs are missing or are links. Blocks.
    MissingFiles,
    /// Game files could not be read, so the source cannot be checked or copied. Blocks.
    Unreadable,
    /// The folder is protected by Windows (Microsoft Store), so it cannot be copied. Blocks.
    ProtectedFolder,
}

impl FindingKind {
    /// Whether this finding makes an installation from the source impossible. Every other
    /// finding is a warning: the installation continues.
    pub const fn blocks_installation(self) -> bool {
        matches!(
            self,
            Self::MissingSod | Self::MissingFiles | Self::Unreadable | Self::ProtectedFolder
        )
    }
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
