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
    #[error("manifest validation: {0}")]
    Validation(String),
}

/// Convenience alias for engine results.
pub type Result<T> = std::result::Result<T, EngineError>;
