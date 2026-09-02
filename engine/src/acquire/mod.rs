//! Immutable artifact acquisition and content-addressed caching.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod archive;
mod cache;
mod http;
mod manual;
mod materialize;

pub use archive::{
    extract_archive, ArchiveLimits, ArchiveMode, ArchiveRequirements, ExtractedArtifact,
};
pub use cache::ArtifactCache;
pub use http::validate_redirect_target;
pub use manual::{provide_manual_archive, VerifiedManualArchive};
pub use materialize::{
    materialize, MaterializationRequest, MaterializationResult, PublicationManifest, PublishedPath,
    SignedCollisionRule,
};

/// One immutable HTTP artifact requested by a recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadRequest {
    /// Stable identifier used for progress events and resumable partial filenames.
    pub request_id: String,
    /// Authored HTTPS download URL. Loopback HTTP is accepted for local verification only.
    pub url: String,
    /// Exact byte length recorded by the recipe.
    pub expected_length: u64,
    /// Exact SHA-256 digest recorded by the recipe.
    pub expected_sha256: String,
    /// Maximum number of transient transport attempts. Must be at least one.
    pub max_attempts: u32,
}

/// Whether acquisition used an existing verified object or published a new one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheDisposition {
    /// A complete cache entry was length-checked and rehashed before use.
    Hit,
    /// A downloaded partial was verified and published.
    Downloaded,
}

/// Durable provenance stored beside a content-addressed archive.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// URL supplied by the immutable recipe.
    pub original_url: String,
    /// URL that returned the artifact after safe redirects.
    pub final_url: String,
    /// Strong or weak HTTP entity tag returned by the final response, when present.
    pub etag: Option<String>,
    /// HTTP Last-Modified validator returned by the final response, when present.
    pub last_modified: Option<String>,
    /// Verified archive length in bytes.
    pub length: u64,
    /// Verified lowercase SHA-256 digest.
    pub sha256: String,
    /// Completion time as seconds since the Unix epoch.
    pub completed_at_unix_seconds: u64,
}

/// A verified cache object ready for extraction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcquiredArtifact {
    /// Content-addressed archive path.
    pub archive_path: PathBuf,
    /// Provenance metadata path beside the archive.
    pub metadata_path: PathBuf,
    /// Verified provenance loaded from or written to the cache.
    pub metadata: ArtifactMetadata,
    /// Whether this call downloaded or reused the object.
    pub disposition: CacheDisposition,
}

/// Failure while validating, transferring, or publishing an immutable artifact.
#[derive(Debug, Error)]
pub enum AcquireError {
    /// A request field violates the acquisition contract.
    #[error("invalid download request: {0}")]
    InvalidRequest(String),
    /// An archive could not be parsed or did not satisfy the supported ZIP profile.
    #[error("invalid archive `{path}`: {message}")]
    ArchiveFormat {
        /// Archive being inspected.
        path: PathBuf,
        /// Parser or policy detail.
        message: String,
    },
    /// An archive entry could escape or alias another destination path.
    #[error("unsafe archive entry `{entry}`: {message}")]
    UnsafeArchiveEntry {
        /// Rejected archive entry name.
        entry: String,
        /// Validation detail.
        message: String,
    },
    /// An archive exceeded a recipe-authored extraction bound.
    #[error("archive limit exceeded: {0}")]
    ArchiveLimitExceeded(String),
    /// The archive did not contain one exact recipe-declared payload layout.
    #[error("archive payload layout is invalid: {0}")]
    ArchiveLayout(String),
    /// Public recipes may not use this archive form or digest.
    #[error("public archive rejected: {0}")]
    PublicArchiveRejected(String),
    /// A URL could not be parsed.
    #[error("invalid download URL `{url}`: {message}")]
    InvalidUrl {
        /// Rejected URL.
        url: String,
        /// Parser or policy detail.
        message: String,
    },
    /// A non-loopback HTTP URL was rejected.
    #[error("insecure artifact URL is not allowed: {url}")]
    InsecureUrl {
        /// Rejected URL.
        url: String,
    },
    /// A redirect attempted to downgrade a trusted HTTPS transfer to HTTP.
    #[error("HTTPS-to-HTTP redirect rejected: {from} -> {to}")]
    RedirectDowngrade {
        /// URL that returned the redirect.
        from: String,
        /// Rejected redirect target.
        to: String,
    },
    /// A redirect omitted its target.
    #[error("redirect from `{url}` did not include a valid Location header")]
    MissingRedirectLocation {
        /// URL that returned the redirect.
        url: String,
    },
    /// The bounded redirect limit was exhausted.
    #[error("too many redirects while downloading from `{url}`")]
    TooManyRedirects {
        /// Original URL.
        url: String,
    },
    /// The HTTP transport failed.
    #[error("HTTP request to `{url}` failed: {message}")]
    Http {
        /// URL being requested.
        url: String,
        /// Transport detail.
        message: String,
    },
    /// The server returned a terminal status.
    #[error("HTTP request to `{url}` returned status {status}")]
    HttpStatus {
        /// Final response URL.
        url: String,
        /// Numeric HTTP status.
        status: u16,
    },
    /// Transient transport failures exhausted the authored attempt limit.
    #[error("download failed after {attempts} attempts: {last_error}")]
    RetryExhausted {
        /// Number of attempts made.
        attempts: u32,
        /// Last transient error.
        last_error: String,
    },
    /// A partial-content response did not identify the exact requested range.
    #[error("invalid partial response from `{url}`: {message}")]
    InvalidRange {
        /// Final response URL.
        url: String,
        /// Validation detail.
        message: String,
    },
    /// Downloaded bytes did not have the pinned length.
    #[error("artifact length mismatch: expected {expected}, got {actual}")]
    LengthMismatch {
        /// Pinned byte length.
        expected: u64,
        /// Observed byte length.
        actual: u64,
    },
    /// Downloaded bytes did not have the pinned digest.
    #[error("artifact SHA-256 mismatch: expected {expected}, got {actual}")]
    HashMismatch {
        /// Pinned lowercase digest.
        expected: String,
        /// Observed lowercase digest.
        actual: String,
    },
    /// A destination file exists with bytes that no signed rule permits replacing.
    #[error("undeclared overwrite of `{path}`")]
    UndeclaredOverwrite {
        /// Destination-relative path.
        path: String,
    },
    /// A collision rule exists but does not match both owners and exact byte hashes.
    #[error("signed collision rule rejected for `{path}`: {message}")]
    CollisionRuleRejected {
        /// Destination-relative path.
        path: String,
        /// Validation detail.
        message: String,
    },
    /// Previously published bytes no longer match their durable manifest.
    #[error("published payload mismatch at `{path}`: {message}")]
    PublicationMismatch {
        /// Destination-relative path.
        path: String,
        /// Validation detail.
        message: String,
    },
    /// A materialization request or destination violates the publication contract.
    #[error("invalid materialization: {0}")]
    InvalidMaterialization(String),
    /// An existing cache object or its metadata was internally inconsistent.
    #[error("corrupt cache entry for {digest}: {message}")]
    CorruptCache {
        /// Content digest naming the entry.
        digest: String,
        /// Validation detail.
        message: String,
    },
    /// A filesystem operation failed.
    #[error("could not {action} `{path}`: {source}")]
    Io {
        /// Operation being attempted.
        action: &'static str,
        /// Path involved.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Cache metadata could not be serialized or parsed.
    #[error("invalid cache metadata at `{path}`: {message}")]
    Metadata {
        /// Metadata file involved.
        path: PathBuf,
        /// Serialization detail.
        message: String,
    },
    /// The system clock is earlier than the Unix epoch.
    #[error("system clock is earlier than the Unix epoch")]
    InvalidClock,
}
