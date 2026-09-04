//! GitHub release discovery and managed installation for BG Radar Overlay.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use sevenz_rust2::{ArchiveReader, Password};
use sha2::{Digest, Sha256};
use thiserror::Error;
use ureq::tls::{RootCerts, TlsConfig};
use ureq::{Agent, Proxy};
use url::{Host, Url};
use walkdir::WalkDir;

use crate::acquire::{ArtifactCache, CacheDisposition, DownloadRequest};
use crate::events::EventSink;

/// Upstream repository whose normal Latest release supplies the add-on.
pub const RADAR_REPOSITORY: &str = "tapahob/BG2RadarOverlay";
/// Directory reserved for the add-on inside the managed BG2 game root.
pub const RADAR_INSTALL_DIRECTORY: &str = "BG Radar Overlay";
/// Executable supplied by every supported Radar Overlay release.
pub const RADAR_EXECUTABLE: &str = "BG Radar Overlay.exe";

const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/tapahob/BG2RadarOverlay/releases/latest";
const MAX_RELEASE_JSON_BYTES: u64 = 1024 * 1024;
const MAX_ASSET_BYTES: u64 = 512 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 256;
const MAX_ENTRY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 512 * 1024 * 1024;
const MAX_PATH_DEPTH: usize = 8;
const RECEIPT_SCHEMA: u32 = 1;
const RECEIPT_RELATIVE_PATH: &str = ".chriz/addons/bg-radar-overlay.json";
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Immutable identity of the current upstream release asset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadarRelease {
    /// Numeric GitHub release identity.
    pub release_id: u64,
    /// Immutable upstream tag.
    pub tag: String,
    /// GitHub publication timestamp.
    pub published_at: String,
    /// Numeric GitHub asset identity.
    pub asset_id: u64,
    /// Upstream asset filename.
    pub asset_name: String,
    /// Versioned GitHub release-download URL.
    pub asset_url: String,
    /// GitHub-advertised exact byte length.
    pub asset_length: u64,
    /// GitHub-advertised lowercase SHA-256.
    pub sha256: String,
}

/// Player-facing integrity/update state for the managed add-on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadarState {
    /// No CEBG-owned overlay receipt exists.
    NotInstalled,
    /// Installed files are intact and match the checked release, when supplied.
    UpToDate,
    /// Installed files are intact but a different Latest tag is available.
    UpdateAvailable,
    /// A receipt or owned file is missing, changed, or unmanaged.
    Modified,
}

/// Small status projection suitable for the launcher bridge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadarStatus {
    /// Coarse state intended for player-facing projection.
    pub state: RadarState,
    /// Receipt tag, when CEBG owns an installation.
    pub installed_version: Option<String>,
    /// Checked Latest tag, when the caller supplied one.
    pub latest_version: Option<String>,
    /// Expected executable path for an owned installation.
    pub executable: Option<PathBuf>,
}

/// Result of installing or updating one verified release.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadarInstallResult {
    /// Installed upstream tag.
    pub version: String,
    /// Executable the launcher may start after the game.
    pub executable: PathBuf,
    /// True when acquisition transferred bytes rather than using the local cache.
    pub downloaded: bool,
}

/// A bounded release-check or managed-install failure.
#[derive(Debug, Error)]
pub enum RadarError {
    #[error("invalid BG Radar Overlay release: {0}")]
    InvalidRelease(String),
    #[error("BG Radar Overlay release check failed for `{url}`: {message}")]
    ReleaseCheck { url: String, message: String },
    #[error("BG Radar Overlay release check returned HTTP {status} from `{url}`")]
    ReleaseHttpStatus { url: String, status: u16 },
    #[error("invalid managed Radar Overlay paths: {0}")]
    InvalidRoots(String),
    #[error("unmanaged Radar Overlay files already exist at `{path}`")]
    UnmanagedInstall { path: PathBuf },
    #[error("managed Radar Overlay files were changed: {}", paths.join(", "))]
    ModifiedFiles { paths: Vec<String> },
    #[error("Radar Overlay file collides with unowned path `{path}`")]
    UnownedCollision { path: String },
    #[error("invalid Radar Overlay archive `{path}`: {message}")]
    Archive { path: PathBuf, message: String },
    #[error("invalid Radar Overlay receipt `{path}`: {message}")]
    Receipt { path: PathBuf, message: String },
    #[error("could not {action} `{path}`: {source}")]
    Io {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Acquire(#[from] crate::acquire::AcquireError),
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    id: u64,
    tag_name: String,
    published_at: Option<String>,
    draft: bool,
    prerelease: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    id: u64,
    name: String,
    size: u64,
    browser_download_url: String,
    digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RadarReceipt {
    schema: u32,
    repository: String,
    install_directory: String,
    installed_at_unix_seconds: u64,
    release: RadarRelease,
    files: Vec<RadarFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RadarFile {
    path: String,
    length: u64,
    sha256: String,
}

struct ValidatedRoots {
    install: PathBuf,
    receipt: PathBuf,
}

/// Query the official GitHub Latest-release endpoint.
pub fn check_latest() -> Result<RadarRelease, RadarError> {
    check_latest_from(LATEST_RELEASE_API)
}

/// Query a GitHub-compatible Latest-release endpoint.
///
/// The override exists for hermetic verification. Production callers should use
/// [`check_latest`]; only HTTPS or loopback HTTP endpoints are accepted.
pub fn check_latest_from(api_url: &str) -> Result<RadarRelease, RadarError> {
    let endpoint = validate_api_url(api_url)?;
    let agent = release_agent(is_loopback_http(&endpoint));
    let response = agent
        .get(endpoint.as_str())
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "chriz-easy-bg")
        .call()
        .map_err(|source| RadarError::ReleaseCheck {
            url: api_url.to_owned(),
            message: source.to_string(),
        })?;
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        return Err(RadarError::ReleaseHttpStatus {
            url: api_url.to_owned(),
            status,
        });
    }
    let mut body = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(MAX_RELEASE_JSON_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(|source| RadarError::ReleaseCheck {
            url: api_url.to_owned(),
            message: source.to_string(),
        })?;
    if body.len() as u64 > MAX_RELEASE_JSON_BYTES {
        return Err(RadarError::InvalidRelease(format!(
            "metadata exceeds {MAX_RELEASE_JSON_BYTES} bytes"
        )));
    }
    let payload: GitHubRelease = serde_json::from_slice(&body)
        .map_err(|source| RadarError::InvalidRelease(source.to_string()))?;
    release_from_github(payload)
}

fn validate_api_url(value: &str) -> Result<Url, RadarError> {
    let url = Url::parse(value).map_err(|source| {
        RadarError::InvalidRelease(format!("invalid API URL {value:?}: {source}"))
    })?;
    if url.scheme() != "https" && !is_loopback_http(&url) {
        return Err(RadarError::InvalidRelease(
            "release API must use HTTPS (loopback HTTP is allowed for tests)".to_owned(),
        ));
    }
    Ok(url)
}

fn is_loopback_http(url: &Url) -> bool {
    url.scheme() == "http"
        && matches!(url.host(), Some(Host::Ipv4(address)) if address.is_loopback())
}

fn release_agent(loopback: bool) -> Agent {
    Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(15)))
        .proxy(if loopback {
            None
        } else {
            Proxy::try_from_env()
        })
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent()
}

fn release_from_github(payload: GitHubRelease) -> Result<RadarRelease, RadarError> {
    if payload.id == 0 || payload.draft || payload.prerelease {
        return Err(RadarError::InvalidRelease(
            "Latest must be a published, non-prerelease release with a nonzero id".to_owned(),
        ));
    }
    validate_tag(&payload.tag_name)?;
    let published_at = payload.published_at.ok_or_else(|| {
        RadarError::InvalidRelease("Latest release has no publication time".to_owned())
    })?;
    let mut candidates = payload.assets.into_iter().filter(|asset| {
        asset.name.to_ascii_lowercase().ends_with(".7z")
            && asset.name.to_ascii_lowercase().contains("radar")
    });
    let asset = candidates.next().ok_or_else(|| {
        RadarError::InvalidRelease("Latest release has no Radar Overlay 7z asset".to_owned())
    })?;
    if candidates.next().is_some() {
        return Err(RadarError::InvalidRelease(
            "Latest release has more than one Radar Overlay 7z asset".to_owned(),
        ));
    }
    if asset.id == 0 || asset.size == 0 || asset.size > MAX_ASSET_BYTES {
        return Err(RadarError::InvalidRelease(format!(
            "asset id/size is invalid (id {}, size {})",
            asset.id, asset.size
        )));
    }
    let digest = asset.digest.ok_or_else(|| {
        RadarError::InvalidRelease("release asset has no advertised SHA-256 digest".to_owned())
    })?;
    let sha256 = digest.strip_prefix("sha256:").ok_or_else(|| {
        RadarError::InvalidRelease("release asset digest is not SHA-256".to_owned())
    })?;
    if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(RadarError::InvalidRelease(
            "release asset SHA-256 is not 64 hexadecimal characters".to_owned(),
        ));
    }
    validate_asset_url(&asset.browser_download_url, &payload.tag_name, &asset.name)?;
    Ok(RadarRelease {
        release_id: payload.id,
        tag: payload.tag_name,
        published_at,
        asset_id: asset.id,
        asset_name: asset.name,
        asset_url: asset.browser_download_url,
        asset_length: asset.size,
        sha256: sha256.to_ascii_lowercase(),
    })
}

fn validate_tag(tag: &str) -> Result<(), RadarError> {
    if tag.is_empty()
        || tag.len() > 128
        || !tag
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(RadarError::InvalidRelease(format!(
            "unsafe release tag {tag:?}"
        )));
    }
    Ok(())
}

fn validate_asset_url(url: &str, tag: &str, name: &str) -> Result<(), RadarError> {
    let parsed = Url::parse(url)
        .map_err(|source| RadarError::InvalidRelease(format!("invalid asset URL: {source}")))?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("github.com") {
        return Err(RadarError::InvalidRelease(
            "asset URL is not an HTTPS github.com release download".to_owned(),
        ));
    }
    let segments = parsed
        .path_segments()
        .map(|segments| segments.collect::<Vec<_>>())
        .unwrap_or_default();
    let expected = [
        "tapahob",
        "BG2RadarOverlay",
        "releases",
        "download",
        tag,
        name,
    ];
    if segments != expected {
        return Err(RadarError::InvalidRelease(
            "asset URL does not match the selected repository, tag, and filename".to_owned(),
        ));
    }
    Ok(())
}

/// Inspect the installed receipt and owned files.
pub fn status(
    managed_root: impl AsRef<Path>,
    game_root: impl AsRef<Path>,
    latest: Option<&RadarRelease>,
) -> Result<RadarStatus, RadarError> {
    let roots = validate_roots(managed_root.as_ref(), game_root.as_ref())?;
    validate_optional_install_root(&roots.install)?;
    let latest_version = latest.map(|release| release.tag.clone());
    let Some(receipt) = load_receipt(&roots.receipt)? else {
        let unmanaged = directory_has_entries(&roots.install)?;
        return Ok(RadarStatus {
            state: if unmanaged {
                RadarState::Modified
            } else {
                RadarState::NotInstalled
            },
            installed_version: None,
            latest_version,
            executable: None,
        });
    };
    let modified = modified_files(&roots.install, &receipt.files)?;
    let state = if !modified.is_empty() {
        RadarState::Modified
    } else if latest.is_some_and(|release| {
        release.tag != receipt.release.tag
            || release.release_id != receipt.release.release_id
            || release.asset_id != receipt.release.asset_id
            || !release.sha256.eq_ignore_ascii_case(&receipt.release.sha256)
    }) {
        RadarState::UpdateAvailable
    } else {
        RadarState::UpToDate
    };
    Ok(RadarStatus {
        state,
        installed_version: Some(receipt.release.tag),
        latest_version,
        executable: Some(roots.install.join(RADAR_EXECUTABLE)),
    })
}

/// Download, verify, extract, and safely publish one Radar Overlay release.
pub fn install(
    cache_root: impl AsRef<Path>,
    managed_root: impl AsRef<Path>,
    game_root: impl AsRef<Path>,
    release: &RadarRelease,
    sink: &dyn EventSink,
) -> Result<RadarInstallResult, RadarError> {
    validate_release_for_install(release)?;
    let roots = validate_roots(managed_root.as_ref(), game_root.as_ref())?;
    validate_optional_install_root(&roots.install)?;
    let addons = roots
        .receipt
        .parent()
        .ok_or_else(|| RadarError::InvalidRoots("receipt has no parent".to_owned()))?;
    create_directories(addons, "create add-on state directory")?;
    reject_link(addons, "add-on state directory")?;
    let lock_path = addons.join("bg-radar-overlay.lock");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|source| io_error("open add-on lock", &lock_path, source))?;
    lock.lock_exclusive()
        .map_err(|source| io_error("acquire add-on lock", &lock_path, source))?;

    let previous = load_receipt(&roots.receipt)?;
    match &previous {
        Some(receipt) => {
            let modified = modified_files(&roots.install, &receipt.files)?;
            if !modified.is_empty() {
                return Err(RadarError::ModifiedFiles { paths: modified });
            }
            if receipt.release.tag == release.tag
                && receipt.release.asset_id == release.asset_id
                && receipt.release.sha256.eq_ignore_ascii_case(&release.sha256)
            {
                return Ok(RadarInstallResult {
                    version: release.tag.clone(),
                    executable: roots.install.join(RADAR_EXECUTABLE),
                    downloaded: false,
                });
            }
        }
        None if directory_has_entries(&roots.install)? => {
            return Err(RadarError::UnmanagedInstall {
                path: roots.install,
            });
        }
        None => {}
    }

    let cache = ArtifactCache::open(cache_root.as_ref())?;
    let acquired = cache.acquire(
        &DownloadRequest {
            request_id: format!("bg-radar-overlay-{}", release.asset_id),
            url: release.asset_url.clone(),
            expected_length: release.asset_length,
            expected_sha256: release.sha256.clone(),
            redirect_hosts: if release.asset_url.starts_with("https://github.com/") {
                vec!["release-assets.githubusercontent.com".to_owned()]
            } else {
                Vec::new()
            },
            max_attempts: 2,
        },
        sink,
    )?;

    let extraction_parent = cache_root.as_ref().join("radar-extraction");
    create_directories(&extraction_parent, "create Radar extraction directory")?;
    let extracted = tempfile::Builder::new()
        .prefix("verified-")
        .tempdir_in(&extraction_parent)
        .map_err(|source| {
            io_error(
                "create Radar extraction temporary",
                &extraction_parent,
                source,
            )
        })?;
    let files = extract_archive(&acquired.archive_path, extracted.path())?;
    let executable_file = files
        .iter()
        .find(|file| file.path.eq_ignore_ascii_case(RADAR_EXECUTABLE))
        .ok_or_else(|| RadarError::Archive {
            path: acquired.archive_path.clone(),
            message: format!("archive does not contain {RADAR_EXECUTABLE}"),
        })?;
    if executable_file.path != RADAR_EXECUTABLE {
        return Err(RadarError::Archive {
            path: acquired.archive_path,
            message: format!("executable must use exact path {RADAR_EXECUTABLE}"),
        });
    }

    let publish = unique_sibling(&roots.install, "publish")?;
    fs::create_dir(&publish)
        .map_err(|source| io_error("create Radar publication temporary", &publish, source))?;
    let publication_result = (|| {
        copy_inventory(extracted.path(), &publish, &files)?;
        if let Some(receipt) = &previous {
            preserve_unowned(&roots.install, &publish, &receipt.files)?;
        }
        let receipt = RadarReceipt {
            schema: RECEIPT_SCHEMA,
            repository: RADAR_REPOSITORY.to_owned(),
            install_directory: RADAR_INSTALL_DIRECTORY.to_owned(),
            installed_at_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| {
                    RadarError::InvalidRoots("system clock is before Unix epoch".to_owned())
                })?
                .as_secs(),
            release: release.clone(),
            files,
        };
        publish_transaction(&roots, &publish, previous.is_some(), &receipt)
    })();
    if publication_result.is_err() {
        remove_directory_if_present(&publish);
    }
    publication_result?;

    Ok(RadarInstallResult {
        version: release.tag.clone(),
        executable: roots.install.join(RADAR_EXECUTABLE),
        downloaded: acquired.disposition == CacheDisposition::Downloaded,
    })
}

fn validate_release_for_install(release: &RadarRelease) -> Result<(), RadarError> {
    if release.release_id == 0 || release.asset_id == 0 {
        return Err(RadarError::InvalidRelease(
            "release and asset ids must be nonzero".to_owned(),
        ));
    }
    validate_tag(&release.tag)?;
    if release.published_at.is_empty()
        || release.asset_length == 0
        || release.asset_length > MAX_ASSET_BYTES
        || !release.asset_name.to_ascii_lowercase().ends_with(".7z")
        || release.sha256.len() != 64
        || !release.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(RadarError::InvalidRelease(
            "release fields do not describe a bounded SHA-256-pinned 7z asset".to_owned(),
        ));
    }
    let url = Url::parse(&release.asset_url)
        .map_err(|source| RadarError::InvalidRelease(format!("invalid asset URL: {source}")))?;
    if is_loopback_http(&url) {
        return Ok(());
    }
    validate_asset_url(&release.asset_url, &release.tag, &release.asset_name)
}

fn validate_roots(managed_root: &Path, game_root: &Path) -> Result<ValidatedRoots, RadarError> {
    for (label, path) in [("managed root", managed_root), ("game root", game_root)] {
        if !path.is_absolute() {
            return Err(RadarError::InvalidRoots(format!(
                "{label} must be absolute: {}",
                path.display()
            )));
        }
        reject_link(path, label)?;
        if is_protected_games_path(path) {
            return Err(RadarError::InvalidRoots(format!(
                "{label} is beneath the read-only C:\\Games reference tree"
            )));
        }
    }
    let managed = fs::canonicalize(managed_root)
        .map_err(|source| io_error("canonicalize managed root", managed_root, source))?;
    let game = fs::canonicalize(game_root)
        .map_err(|source| io_error("canonicalize game root", game_root, source))?;
    if managed == game || game.strip_prefix(&managed).is_err() {
        return Err(RadarError::InvalidRoots(
            "game root must be a child of the managed root".to_owned(),
        ));
    }
    if is_protected_games_path(&managed) || is_protected_games_path(&game) {
        return Err(RadarError::InvalidRoots(
            "canonical roots are beneath the read-only C:\\Games reference tree".to_owned(),
        ));
    }
    Ok(ValidatedRoots {
        install: game_root.join(RADAR_INSTALL_DIRECTORY),
        receipt: managed_root.join(RECEIPT_RELATIVE_PATH),
    })
}

fn is_protected_games_path(path: &Path) -> bool {
    let normalized = path
        .to_string_lossy()
        .replace('/', "\\")
        .trim_start_matches("\\\\?\\")
        .to_ascii_lowercase();
    normalized == "c:\\games" || normalized.starts_with("c:\\games\\")
}

fn reject_link(path: &Path, label: &str) -> Result<(), RadarError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|source| io_error("inspect managed directory", path, source))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(RadarError::InvalidRoots(format!(
            "{label} is not a direct directory: {}",
            path.display()
        )));
    }
    Ok(())
}

fn validate_optional_install_root(path: &Path) -> Result<(), RadarError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(io_error("inspect Radar install directory", path, source)),
    };
    #[cfg(windows)]
    let is_reparse_point = {
        use std::os::windows::fs::MetadataExt as _;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    };
    #[cfg(not(windows))]
    let is_reparse_point = false;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || is_reparse_point {
        return Err(RadarError::InvalidRoots(format!(
            "Radar install path is not a direct directory: {}",
            path.display()
        )));
    }
    Ok(())
}

fn load_receipt(path: &Path) -> Result<Option<RadarReceipt>, RadarError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(io_error("read Radar receipt", path, source)),
    };
    let receipt: RadarReceipt =
        serde_json::from_slice(&bytes).map_err(|source| RadarError::Receipt {
            path: path.to_path_buf(),
            message: source.to_string(),
        })?;
    validate_receipt(path, &receipt)?;
    Ok(Some(receipt))
}

fn validate_receipt(path: &Path, receipt: &RadarReceipt) -> Result<(), RadarError> {
    if receipt.schema != RECEIPT_SCHEMA
        || receipt.repository != RADAR_REPOSITORY
        || receipt.install_directory != RADAR_INSTALL_DIRECTORY
        || receipt.files.is_empty()
    {
        return Err(RadarError::Receipt {
            path: path.to_path_buf(),
            message: "receipt identity or schema is invalid".to_owned(),
        });
    }
    validate_release_for_install(&receipt.release).map_err(|error| RadarError::Receipt {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let mut seen = BTreeSet::new();
    let mut executable = false;
    for file in &receipt.files {
        let normalized =
            normalize_relative_path(&file.path).map_err(|message| RadarError::Receipt {
                path: path.to_path_buf(),
                message,
            })?;
        if normalized != file.path
            || file.sha256.len() != 64
            || !file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !seen.insert(normalized.to_ascii_lowercase())
        {
            return Err(RadarError::Receipt {
                path: path.to_path_buf(),
                message: format!("invalid or duplicate owned file {}", file.path),
            });
        }
        executable |= file.path == RADAR_EXECUTABLE;
    }
    if !executable {
        return Err(RadarError::Receipt {
            path: path.to_path_buf(),
            message: format!("receipt does not own {RADAR_EXECUTABLE}"),
        });
    }
    Ok(())
}

fn modified_files(install_root: &Path, files: &[RadarFile]) -> Result<Vec<String>, RadarError> {
    let mut modified = Vec::new();
    for file in files {
        let path = install_root.join(native_relative_path(&file.path));
        match hash_regular_file(&path) {
            Ok((length, sha256)) if length == file.length && sha256 == file.sha256 => {}
            Ok(_) => modified.push(file.path.clone()),
            Err(RadarError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
                modified.push(file.path.clone());
            }
            Err(error) => return Err(error),
        }
    }
    modified.sort_by_key(|path| path.to_ascii_lowercase());
    Ok(modified)
}

fn directory_has_entries(path: &Path) -> Result<bool, RadarError> {
    match fs::read_dir(path) {
        Ok(mut entries) => Ok(entries
            .next()
            .transpose()
            .map_err(|source| io_error("inspect Radar install directory", path, source))?
            .is_some()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(io_error("inspect Radar install directory", path, source)),
    }
}

fn extract_archive(archive_path: &Path, destination: &Path) -> Result<Vec<RadarFile>, RadarError> {
    let reader = ArchiveReader::open(archive_path, Password::empty()).map_err(|source| {
        RadarError::Archive {
            path: archive_path.to_path_buf(),
            message: source.to_string(),
        }
    })?;
    let entries = &reader.archive().files;
    if entries.is_empty() || entries.len() > MAX_ARCHIVE_ENTRIES {
        return Err(RadarError::Archive {
            path: archive_path.to_path_buf(),
            message: format!(
                "archive has {} entries; supported range is 1..={MAX_ARCHIVE_ENTRIES}",
                entries.len()
            ),
        });
    }
    let mut declared_files = BTreeMap::<String, (String, u64)>::new();
    let mut declared_paths = BTreeSet::new();
    let mut total = 0_u64;
    for entry in entries {
        if entry.is_anti_item {
            return Err(RadarError::Archive {
                path: archive_path.to_path_buf(),
                message: format!("anti-item entry is unsupported: {}", entry.name),
            });
        }
        let normalized = normalize_relative_path(entry.name.trim_end_matches(['/', '\\']))
            .map_err(|message| RadarError::Archive {
                path: archive_path.to_path_buf(),
                message,
            })?;
        let key = normalized.to_ascii_lowercase();
        if !declared_paths.insert(key.clone()) {
            return Err(RadarError::Archive {
                path: archive_path.to_path_buf(),
                message: format!("case-insensitive duplicate path {normalized}"),
            });
        }
        if !entry.is_directory {
            if entry.size > MAX_ENTRY_BYTES {
                return Err(RadarError::Archive {
                    path: archive_path.to_path_buf(),
                    message: format!("entry {normalized} exceeds {MAX_ENTRY_BYTES} bytes"),
                });
            }
            total = total
                .checked_add(entry.size)
                .ok_or_else(|| RadarError::Archive {
                    path: archive_path.to_path_buf(),
                    message: "aggregate extraction size overflowed u64".to_owned(),
                })?;
            if total > MAX_EXTRACTED_BYTES {
                return Err(RadarError::Archive {
                    path: archive_path.to_path_buf(),
                    message: format!("archive expands beyond {MAX_EXTRACTED_BYTES} bytes"),
                });
            }
            declared_files.insert(key, (normalized, entry.size));
        }
    }
    if !declared_files.contains_key(&RADAR_EXECUTABLE.to_ascii_lowercase()) {
        return Err(RadarError::Archive {
            path: archive_path.to_path_buf(),
            message: format!("archive does not contain {RADAR_EXECUTABLE}"),
        });
    }
    for (key, (path, _)) in &declared_files {
        let prefix = format!("{key}/");
        if declared_files
            .keys()
            .any(|other| other.starts_with(&prefix))
        {
            return Err(RadarError::Archive {
                path: archive_path.to_path_buf(),
                message: format!("file/directory prefix collision at {path}"),
            });
        }
    }

    sevenz_rust2::decompress_file(archive_path, destination).map_err(|source| {
        RadarError::Archive {
            path: archive_path.to_path_buf(),
            message: source.to_string(),
        }
    })?;
    let mut files = Vec::with_capacity(declared_files.len());
    for entry in WalkDir::new(destination).follow_links(false) {
        let entry = entry.map_err(|source| RadarError::Archive {
            path: archive_path.to_path_buf(),
            message: source.to_string(),
        })?;
        if entry.path() == destination || entry.file_type().is_dir() {
            continue;
        }
        if !entry.file_type().is_file() || entry.file_type().is_symlink() {
            return Err(RadarError::Archive {
                path: archive_path.to_path_buf(),
                message: format!("extracted non-regular entry {}", entry.path().display()),
            });
        }
        let relative =
            entry
                .path()
                .strip_prefix(destination)
                .map_err(|source| RadarError::Archive {
                    path: archive_path.to_path_buf(),
                    message: source.to_string(),
                })?;
        let normalized = relative_to_slash(relative)?;
        let key = normalized.to_ascii_lowercase();
        let Some((declared, declared_length)) = declared_files.remove(&key) else {
            return Err(RadarError::Archive {
                path: archive_path.to_path_buf(),
                message: format!("extracted undeclared file {normalized}"),
            });
        };
        if declared != normalized {
            return Err(RadarError::Archive {
                path: archive_path.to_path_buf(),
                message: format!(
                    "entry casing changed during extraction: {declared} -> {normalized}"
                ),
            });
        }
        let (length, sha256) = hash_regular_file(entry.path())?;
        if length != declared_length {
            return Err(RadarError::Archive {
                path: archive_path.to_path_buf(),
                message: format!("entry length changed during extraction: {normalized}"),
            });
        }
        files.push(RadarFile {
            path: normalized,
            length,
            sha256,
        });
    }
    if !declared_files.is_empty() {
        return Err(RadarError::Archive {
            path: archive_path.to_path_buf(),
            message: "not every declared file was extracted".to_owned(),
        });
    }
    files.sort_by_key(|file| file.path.to_ascii_lowercase());
    Ok(files)
}

fn normalize_relative_path(value: &str) -> Result<String, String> {
    if value.is_empty() || value.contains('\0') || value.contains(':') {
        return Err(format!("unsafe empty, NUL, or colon path {value:?}"));
    }
    let value = value.replace('\\', "/");
    if value.starts_with('/') || value.ends_with('/') {
        return Err(format!(
            "path is not a regular relative file path: {value:?}"
        ));
    }
    let components = value.split('/').collect::<Vec<_>>();
    if components.len() > MAX_PATH_DEPTH
        || components
            .iter()
            .any(|component| !safe_windows_component(component))
        || components[0].eq_ignore_ascii_case(".chriz")
    {
        return Err(format!("unsafe archive path {value:?}"));
    }
    Ok(components.join("/"))
}

fn safe_windows_component(component: &str) -> bool {
    if component.is_empty()
        || matches!(component, "." | "..")
        || component.ends_with([' ', '.'])
        || component.chars().any(|character| {
            character.is_control() || matches!(character, '<' | '>' | '"' | '|' | '?' | '*')
        })
    {
        return false;
    }
    let stem = component
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    !matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        && !(stem.len() == 4
            && (stem.starts_with("com") || stem.starts_with("lpt"))
            && stem.as_bytes()[3].is_ascii_digit()
            && stem.as_bytes()[3] != b'0')
}

fn copy_inventory(
    source: &Path,
    destination: &Path,
    files: &[RadarFile],
) -> Result<(), RadarError> {
    for file in files {
        let relative = native_relative_path(&file.path);
        let from = source.join(&relative);
        let to = destination.join(&relative);
        if let Some(parent) = to.parent() {
            create_directories(parent, "create Radar publication directory")?;
        }
        fs::copy(&from, &to).map_err(|source| io_error("copy verified Radar file", &to, source))?;
        let (length, sha256) = hash_regular_file(&to)?;
        if length != file.length || sha256 != file.sha256 {
            return Err(RadarError::Archive {
                path: from,
                message: "file changed while preparing publication".to_owned(),
            });
        }
    }
    Ok(())
}

fn preserve_unowned(
    current: &Path,
    publication: &Path,
    owned_files: &[RadarFile],
) -> Result<(), RadarError> {
    let owned = owned_files
        .iter()
        .map(|file| file.path.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    for entry in WalkDir::new(current).follow_links(false) {
        let entry = entry.map_err(|source| RadarError::InvalidRoots(source.to_string()))?;
        if entry.path() == current || entry.file_type().is_dir() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(current)
            .map_err(|source| RadarError::InvalidRoots(source.to_string()))?;
        let normalized = relative_to_slash(relative)?;
        if owned.contains(&normalized.to_ascii_lowercase()) {
            continue;
        }
        if !entry.file_type().is_file() || entry.file_type().is_symlink() {
            return Err(RadarError::ModifiedFiles {
                paths: vec![normalized],
            });
        }
        let target = publication.join(native_relative_path(&normalized));
        if target.exists() {
            return Err(RadarError::UnownedCollision { path: normalized });
        }
        if let Some(parent) = target.parent() {
            create_directories(parent, "create preserved Radar directory")?;
        }
        fs::copy(entry.path(), &target)
            .map_err(|source| io_error("preserve unowned Radar file", &target, source))?;
    }
    Ok(())
}

fn publish_transaction(
    roots: &ValidatedRoots,
    publication: &Path,
    updating: bool,
    receipt: &RadarReceipt,
) -> Result<(), RadarError> {
    let receipt_temp = unique_sibling(&roots.receipt, "new")?;
    write_receipt(&receipt_temp, receipt)?;
    if !updating {
        if roots.install.exists() {
            fs::remove_dir(&roots.install).map_err(|source| {
                io_error("remove empty Radar directory", &roots.install, source)
            })?;
        }
        if let Err(source) = fs::rename(publication, &roots.install) {
            remove_file_if_present(&receipt_temp);
            return Err(io_error(
                "publish Radar install directory",
                &roots.install,
                source,
            ));
        }
        if let Err(source) = fs::rename(&receipt_temp, &roots.receipt) {
            let _ = fs::rename(&roots.install, publication);
            remove_directory_if_present(publication);
            remove_file_if_present(&receipt_temp);
            return Err(io_error("publish Radar receipt", &roots.receipt, source));
        }
        return Ok(());
    }

    let backup = unique_sibling(&roots.install, "backup")?;
    let receipt_backup = unique_sibling(&roots.receipt, "backup")?;
    if let Err(source) = fs::rename(&roots.receipt, &receipt_backup) {
        remove_file_if_present(&receipt_temp);
        return Err(io_error("back up Radar receipt", &roots.receipt, source));
    }
    if let Err(source) = fs::rename(&roots.install, &backup) {
        let _ = fs::rename(&receipt_backup, &roots.receipt);
        remove_file_if_present(&receipt_temp);
        return Err(io_error(
            "back up Radar install directory",
            &roots.install,
            source,
        ));
    }
    if let Err(source) = fs::rename(publication, &roots.install) {
        let _ = fs::rename(&backup, &roots.install);
        let _ = fs::rename(&receipt_backup, &roots.receipt);
        remove_file_if_present(&receipt_temp);
        return Err(io_error(
            "publish Radar update directory",
            &roots.install,
            source,
        ));
    }
    if let Err(source) = fs::rename(&receipt_temp, &roots.receipt) {
        let failed = unique_sibling(&roots.install, "failed")?;
        let _ = fs::rename(&roots.install, &failed);
        let _ = fs::rename(&backup, &roots.install);
        let _ = fs::rename(&receipt_backup, &roots.receipt);
        remove_directory_if_present(&failed);
        remove_file_if_present(&receipt_temp);
        return Err(io_error(
            "publish Radar update receipt",
            &roots.receipt,
            source,
        ));
    }
    remove_directory_if_present(&backup);
    remove_file_if_present(&receipt_backup);
    Ok(())
}

fn write_receipt(path: &Path, receipt: &RadarReceipt) -> Result<(), RadarError> {
    let mut bytes = serde_json::to_vec_pretty(receipt).map_err(|source| RadarError::Receipt {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;
    bytes.push(b'\n');
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|source| io_error("create Radar receipt temporary", path, source))?;
    output
        .write_all(&bytes)
        .map_err(|source| io_error("write Radar receipt temporary", path, source))?;
    output
        .sync_all()
        .map_err(|source| io_error("sync Radar receipt temporary", path, source))
}

fn hash_regular_file(path: &Path) -> Result<(u64, String), RadarError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|source| io_error("inspect Radar file", path, source))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(RadarError::ModifiedFiles {
            paths: vec![path.to_string_lossy().into_owned()],
        });
    }
    let mut input = File::open(path).map_err(|source| io_error("open Radar file", path, source))?;
    let mut digest = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = input
            .read(&mut buffer)
            .map_err(|source| io_error("read Radar file", path, source))?;
        if read == 0 {
            break;
        }
        length = length.checked_add(read as u64).ok_or_else(|| {
            RadarError::InvalidRoots("Radar file length overflowed u64".to_owned())
        })?;
        digest.update(&buffer[..read]);
    }
    Ok((length, hex::encode(digest.finalize())))
}

fn relative_to_slash(path: &Path) -> Result<String, RadarError> {
    let text = path.to_str().ok_or_else(|| {
        RadarError::InvalidRoots(format!("Radar path is not Unicode: {}", path.display()))
    })?;
    normalize_relative_path(&text.replace('\\', "/")).map_err(RadarError::InvalidRoots)
}

fn native_relative_path(path: &str) -> PathBuf {
    path.split('/').collect()
}

fn create_directories(path: &Path, action: &'static str) -> Result<(), RadarError> {
    fs::create_dir_all(path).map_err(|source| io_error(action, path, source))
}

fn unique_sibling(path: &Path, purpose: &str) -> Result<PathBuf, RadarError> {
    let parent = path
        .parent()
        .ok_or_else(|| RadarError::InvalidRoots("temporary path has no parent".to_owned()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| RadarError::InvalidRoots("temporary filename is not Unicode".to_owned()))?;
    for _ in 0..32 {
        let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{name}.chriz-{purpose}-{}-{sequence}",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(RadarError::InvalidRoots(
        "could not allocate a unique Radar temporary path".to_owned(),
    ))
}

fn remove_directory_if_present(path: &Path) {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            let _ = fs::remove_dir_all(path);
        }
        _ => {}
    }
}

fn remove_file_if_present(path: &Path) {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            let _ = fs::remove_file(path);
        }
        _ => {}
    }
}

fn io_error(action: &'static str, path: &Path, source: std::io::Error) -> RadarError {
    RadarError::Io {
        action,
        path: path.to_path_buf(),
        source,
    }
}
