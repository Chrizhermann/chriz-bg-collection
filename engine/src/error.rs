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
    /// Canonical manifest content could not be serialized for hashing.
    #[error("could not fingerprint manifest at {path}: {source}")]
    ManifestFingerprint {
        /// Root of the manifest being fingerprinted.
        path: std::path::PathBuf,
        /// Underlying JSON serialization error.
        #[source]
        source: serde_json::Error,
    },
    /// A persisted session belongs to different manifest content.
    #[error(
        "resume with changed manifest is forbidden for {path}: expected fingerprint {expected}, found {found}; rebuild instead"
    )]
    ManifestFingerprintMismatch {
        /// Path to the persisted session.
        path: std::path::PathBuf,
        /// Fingerprint of the current manifest.
        expected: String,
        /// Fingerprint recorded in the persisted session.
        found: String,
    },
    /// Manifest validation produced findings; rendered one per line.
    #[error("{}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"))]
    Validation(Vec<crate::validate::Finding>),
    /// A selection contains an unsupported platform or unknown toggle or choice id.
    #[error("invalid selection: {0}")]
    InvalidSelection(String),
}

/// Convenience alias for engine results.
pub type Result<T> = std::result::Result<T, EngineError>;
