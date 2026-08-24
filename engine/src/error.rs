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
    /// A selection contains an unsupported platform or unknown toggle or choice id.
    #[error("invalid selection: {0}")]
    InvalidSelection(String),
    #[error("manifest validation: {0}")]
    Validation(String),
}

/// Convenience alias for engine results.
pub type Result<T> = std::result::Result<T, EngineError>;
