//! Engine-wide error type.

use thiserror::Error;

/// Anything that can go wrong inside the engine.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("{path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// The failed filesystem operation, without discarding its original cause.
    #[error("{operation} failed at {path}: {source}")]
    FolderOperation {
        /// Stable, human-readable operation description.
        operation: &'static str,
        /// Folder or ownership marker being accessed.
        path: std::path::PathBuf,
        /// Original I/O or safety error, retained for accurate classification.
        #[source]
        source: Box<EngineError>,
    },
    /// Cleanup failed after an operation had already failed; neither cause is lost.
    #[error("{original}; cleanup also failed: {cleanup}")]
    CleanupFailed {
        /// The operation that actually prevented installation from continuing.
        #[source]
        original: Box<EngineError>,
        /// Secondary cleanup failure, not a replacement for the original cause.
        cleanup: Box<EngineError>,
    },
    #[error("toml parse error in {path}: {source}")]
    ManifestParse {
        path: std::path::PathBuf,
        #[source]
        source: toml::de::Error,
    },
    /// The collection requests a schema version this engine cannot read.
    #[error("unsupported manifest schema {found} in {path}; supported schema is {supported}")]
    UnsupportedSchema {
        /// Path to the collection manifest declaring the unsupported schema.
        path: std::path::PathBuf,
        /// Schema version declared by the collection manifest.
        found: u32,
        /// Schema version supported by this engine.
        supported: u32,
    },
    /// A mod's declared id does not match its TOML file name.
    #[error("mod id {id:?} in {path} does not match file stem {stem:?}")]
    ModIdMismatch {
        /// Path to the mod manifest with the mismatched id.
        path: std::path::PathBuf,
        /// Id declared inside the mod manifest.
        id: String,
        /// File stem required as the mod id.
        stem: String,
    },
    /// A mod manifest path has no file stem representable as UTF-8.
    #[error("mod manifest path does not have a valid UTF-8 file stem: {path}")]
    InvalidModFileStem {
        /// Path to the mod manifest with the invalid file stem.
        path: std::path::PathBuf,
    },
    /// More than one mod manifest declares the same id.
    #[error("duplicate mod id {id:?}: first declared in {first}, then in {second}")]
    DuplicateModId {
        /// Id declared by both mod manifests.
        id: String,
        /// Path to the first mod manifest declaring the id.
        first: std::path::PathBuf,
        /// Path to the second mod manifest declaring the id.
        second: std::path::PathBuf,
    },
    /// An artifact's declared id does not match its TOML file name.
    #[error("artifact id {id:?} in {path} does not match file stem {stem:?}")]
    ArtifactIdMismatch {
        /// Path to the artifact manifest with the mismatched id.
        path: std::path::PathBuf,
        /// Id declared inside the artifact manifest.
        id: String,
        /// File stem required as the artifact id.
        stem: String,
    },
    /// An artifact manifest path has no file stem representable as UTF-8.
    #[error("artifact manifest path does not have a valid UTF-8 file stem: {path}")]
    InvalidArtifactFileStem {
        /// Path to the artifact manifest with the invalid file stem.
        path: std::path::PathBuf,
    },
    /// More than one artifact manifest declares the same id.
    #[error("duplicate artifact id {id:?}: first declared in {first}, then in {second}")]
    DuplicateArtifactId {
        /// Id declared by both artifact manifests.
        id: String,
        /// Path to the first artifact manifest declaring the id.
        first: std::path::PathBuf,
        /// Path to the second artifact manifest declaring the id.
        second: std::path::PathBuf,
    },
    /// A preset's declared id does not match its TOML file name.
    #[error("preset id {id:?} in {path} does not match file stem {stem:?}")]
    PresetIdMismatch {
        /// Path to the preset manifest with the mismatched id.
        path: std::path::PathBuf,
        /// Id declared inside the preset manifest.
        id: String,
        /// File stem required as the preset id.
        stem: String,
    },
    /// A preset manifest path has no file stem representable as UTF-8.
    #[error("preset manifest path does not have a valid UTF-8 file stem: {path}")]
    InvalidPresetFileStem {
        /// Path to the preset manifest with the invalid file stem.
        path: std::path::PathBuf,
    },
    /// More than one preset manifest declares the same id.
    #[error("duplicate preset id {id:?}: first declared in {first}, then in {second}")]
    DuplicatePresetId {
        /// Id declared by both preset manifests.
        id: String,
        /// Path to the first preset manifest declaring the id.
        first: std::path::PathBuf,
        /// Path to the second preset manifest declaring the id.
        second: std::path::PathBuf,
    },
    /// Session JSON could not be serialized or parsed.
    #[error("session JSON error in {path}: {source}")]
    SessionJson {
        /// Path to the session file being serialized or parsed.
        path: std::path::PathBuf,
        /// Underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
    /// A create-once campaign already exists at the requested managed root.
    #[error("campaign state already exists at {path}")]
    SessionAlreadyExists {
        /// Existing campaign state root.
        path: std::path::PathBuf,
    },
    /// An append-only ledger violates its filename, sequence, hash, or event contract.
    #[error("invalid campaign ledger at {path}: {message}")]
    SessionLedger {
        /// Ledger file or directory involved.
        path: std::path::PathBuf,
        /// Validation detail.
        message: String,
    },
    /// Frozen campaign inputs differ from the identity supplied for resume.
    #[error("resume identity mismatch for {field}: expected {expected}, found {found}")]
    SessionIdentityMismatch {
        /// Consequential identity field.
        field: &'static str,
        /// Frozen identity summary.
        expected: String,
        /// Resume identity summary.
        found: String,
    },
    /// The system clock cannot provide a durable record timestamp.
    #[error("system clock is earlier than the Unix epoch")]
    SessionClock,
    /// Canonical JSON could not be produced for a semantic digest.
    #[error("could not digest {context}: {source}")]
    CanonicalDigest {
        /// Semantic object being digested.
        context: &'static str,
        /// Serialization failure.
        #[source]
        source: serde_json::Error,
    },
    /// Manifest validation produced findings; rendered one per line.
    #[error("{}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"))]
    Validation(Vec<crate::validate::Finding>),
    /// A selection contains an unsupported platform or unknown toggle or choice id.
    #[error("invalid selection: {0}")]
    InvalidSelection(String),
    /// A source and managed destination overlap or otherwise alias one another.
    #[error("unsafe staging relationship between {source_root} and {target}: {reason}")]
    UnsafeStagingRelationship {
        /// Read-only source path.
        source_root: std::path::PathBuf,
        /// Proposed managed destination path.
        target: std::path::PathBuf,
        /// Explanation of the containment or aliasing violation.
        reason: String,
    },
    /// Staging encountered a link, reparse point, special file, or unexpected entry.
    #[error("unsafe staging entry {path}: {reason}")]
    UnsafeStagingEntry {
        /// Entry rejected before it could be followed or overwritten.
        path: std::path::PathBuf,
        /// Explanation of the rejected entry kind or ownership conflict.
        reason: String,
    },
    /// Source bytes changed while a regular-file stage was in progress.
    #[error(
        "source {source_root} changed during staging: expected fingerprint {expected}, found {found}"
    )]
    StagingSourceChanged {
        /// Canonical source root.
        source_root: std::path::PathBuf,
        /// Fingerprint captured before copying.
        expected: String,
        /// Fingerprint observed later, or a description when a complete rescan failed.
        found: String,
    },
    /// A completed copy did not reproduce the source tree exactly.
    #[error("staging verification failed at {path}: {reason}")]
    StagingVerification {
        /// Managed path whose content or identity was unexpected.
        path: std::path::PathBuf,
        /// Explanation of the verification failure.
        reason: String,
    },
    /// A derived save directory already belongs to something other than this install.
    #[error("save root collision at {path}: {reason}")]
    SaveRootCollision {
        /// Exact Documents child that cannot safely be claimed.
        path: std::path::PathBuf,
        /// Explanation of the ownership mismatch.
        reason: String,
    },
    /// A game's `engine.lua` cannot safely receive or prove the unique save identity.
    #[error("game identity error at {path}: {reason}")]
    GameIdentity {
        /// `engine.lua` or the identity input that failed validation.
        path: std::path::PathBuf,
        /// Explanation of the validation failure.
        reason: String,
    },
}

impl EngineError {
    /// Returns the original operation's cause, never a secondary cleanup error.
    ///
    /// Callers must still distinguish I/O errors from ownership and safety failures.
    /// In particular, a collision is not made retryable by a cleanup I/O failure.
    pub fn primary_cause(&self) -> &Self {
        match self {
            Self::FolderOperation { source, .. } => source.primary_cause(),
            Self::CleanupFailed { original, .. } => original.primary_cause(),
            _ => self,
        }
    }

    /// Returns the failed folder operation, ignoring secondary cleanup context.
    pub fn folder_operation(&self) -> Option<(&'static str, &std::path::Path)> {
        match self {
            Self::FolderOperation {
                operation, path, ..
            } => Some((*operation, path)),
            Self::CleanupFailed { original, .. } => original.folder_operation(),
            _ => None,
        }
    }

    /// Stable presentation code for a contextual folder failure.
    ///
    /// This classifies the cause for display only. It does not grant retry,
    /// overwrite, cleanup or resume permission, and never classifies a secondary
    /// cleanup error in place of the original failure.
    pub fn folder_failure_code(&self) -> Option<&'static str> {
        use std::io::ErrorKind;

        self.folder_operation()?;
        Some(match self.primary_cause() {
            Self::Io { source, .. } => {
                #[cfg(windows)]
                if matches!(source.raw_os_error(), Some(32 | 33)) {
                    return Some("folder_in_use");
                }
                match source.kind() {
                    ErrorKind::PermissionDenied => "folder_access_denied",
                    ErrorKind::NotFound | ErrorKind::NotConnected => "folder_unavailable",
                    ErrorKind::AlreadyExists
                    | ErrorKind::NotADirectory
                    | ErrorKind::IsADirectory => "folder_conflict",
                    _ => "folder_failed",
                }
            }
            Self::SaveRootCollision { .. }
            | Self::UnsafeStagingEntry { .. }
            | Self::UnsafeStagingRelationship { .. }
            | Self::GameIdentity { .. } => "folder_conflict",
            _ => "folder_failed",
        })
    }
}

/// Convenience alias for engine results.
pub type Result<T> = std::result::Result<T, EngineError>;
