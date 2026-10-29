use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipArchive};

#[cfg(windows)]
use widestring::WideCString;

use crate::fs_retry;

use super::damaged::{damaged_entry_reason, entry_is_present, set_aside, CacheRepair};
use super::entry_lock::lock_entry;
use super::{AcquireError, StopSignal};

const BUFFER_SIZE: usize = 64 * 1024;
const EXTRACTION_MARKER: &str = ".chriz-bg-extraction.json";
const EXTRACTION_MARKER_VERSION: u32 = 3;
const EXTRACTION_CACHE_NAMESPACE: &str = "sha256-v3";
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Resource limits applied before any archive entry is written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ArchiveLimits {
    /// Maximum number of path components in one entry.
    pub max_depth: usize,
    /// Maximum central-directory entry count.
    pub max_entries: usize,
    /// Maximum uncompressed size of one regular file.
    pub max_entry_uncompressed_bytes: u64,
    /// Maximum aggregate uncompressed size of all regular files.
    pub max_total_uncompressed_bytes: u64,
    /// Maximum integer ratio of uncompressed to compressed bytes.
    pub max_compression_ratio: u64,
}

impl Default for ArchiveLimits {
    fn default() -> Self {
        Self {
            max_depth: 32,
            max_entries: 100_000,
            max_entry_uncompressed_bytes: 2 * 1024 * 1024 * 1024,
            max_total_uncompressed_bytes: 8 * 1024 * 1024 * 1024,
            max_compression_ratio: 1_000,
        }
    }
}

/// How much a changed archive's entry count and sizes may grow over the reviewed limits.
const SERVED_LIMIT_GROWTH: usize = 16;

impl ArchiveLimits {
    /// Limits for an archive whose download differs from the reviewed one.
    ///
    /// Reviewed limits are tight bounds around the reviewed archive, so a normal update that
    /// adds files would fail them. A changed archive may have sixteen times the reviewed entry
    /// count and sizes, and always at least the general limits (100,000 entries, 2 GiB per file,
    /// 8 GiB in total). Depth and compression ratio keep the larger of the reviewed and the
    /// general limit (32 levels, 1,000:1), so the guard against archive bombs still holds.
    pub fn for_served_archive(&self) -> Self {
        let general = Self::default();
        let growth = SERVED_LIMIT_GROWTH as u64;
        Self {
            max_depth: self.max_depth.max(general.max_depth),
            max_entries: self
                .max_entries
                .saturating_mul(SERVED_LIMIT_GROWTH)
                .max(general.max_entries),
            max_entry_uncompressed_bytes: self
                .max_entry_uncompressed_bytes
                .saturating_mul(growth)
                .max(general.max_entry_uncompressed_bytes),
            max_total_uncompressed_bytes: self
                .max_total_uncompressed_bytes
                .saturating_mul(growth)
                .max(general.max_total_uncompressed_bytes),
            max_compression_ratio: self
                .max_compression_ratio
                .max(general.max_compression_ratio),
        }
    }
}

/// Extraction policy selected by the recipe source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveMode {
    /// A redistributable recipe with a mandatory immutable digest and ZIP-compatible framing.
    Public,
    /// A locally authored recipe. The archive is still subject to all structural limits.
    Authoring,
}

/// ZIP-compatible archive framing declared by the recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveFormat {
    /// A standard `.zip` artifact.
    Zip,
    /// An `.iemod` artifact, which uses ZIP framing.
    Iemod,
    /// A Windows self-extracting RAR archive; its executable stub is never run.
    SelfExtractingRar,
}

/// Exact archive shape and bounds authored by a recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveRequirements {
    /// Expected archive SHA-256, or all zeroes only in authoring mode.
    pub artifact_sha256: String,
    /// Declared archive framing, independent of the content-addressed cache filename.
    pub format: ArchiveFormat,
    /// Payload directory roots that may later be materialized.
    pub expected_roots: Vec<String>,
    /// Exact WeiDU TP2 paths expected after optional wrapper removal.
    pub expected_tp2_paths: Vec<String>,
    /// Bounded extraction limits.
    pub limits: ArchiveLimits,
    /// Public or local authoring policy.
    pub mode: ArchiveMode,
}

/// One fully validated and content-addressed extraction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractedArtifact {
    /// Published extraction root.
    pub root: PathBuf,
    /// Verified archive SHA-256 used as the cache key.
    pub artifact_sha256: String,
    /// Digest of the canonical extraction layout, policy, format, and limits.
    pub extraction_contract_sha256: String,
    /// Single stripped wrapper directory, when present.
    pub wrapper_directory: Option<String>,
    /// Recipe-approved materialization roots.
    pub expected_roots: Vec<String>,
    /// Recipe-approved exact TP2 paths.
    pub expected_tp2_paths: Vec<String>,
}

#[derive(Clone, Debug)]
struct ValidatedEntry {
    index: usize,
    archive_path: String,
    relative_path: String,
    size: u64,
    is_directory: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExtractionMarker {
    version: u32,
    artifact_sha256: String,
    contract_sha256: String,
    wrapper_directory: Option<String>,
    expected_roots: Vec<String>,
    expected_tp2_paths: Vec<String>,
    files: Vec<ExtractedFile>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExtractedFile {
    relative_path: String,
    length: u64,
    sha256: String,
}

#[derive(Serialize)]
struct ExtractionContract<'a> {
    version: u32,
    format: ArchiveFormat,
    mode: ArchiveMode,
    expected_roots: &'a [String],
    expected_tp2_paths: &'a [String],
    limits: &'a ArchiveLimits,
    /// Present only for a changed download, so that reviewed extractions keep their identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    layout: Option<&'static str>,
}

/// Layout rule recorded in the contract of a changed download's extraction.
const SERVED_LAYOUT_RULE: &str = "served-v1";

/// Which payload layout an extraction accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayoutRule {
    /// Exactly the reviewed layout: the payload at the root or below one folder holding
    /// every file, and every TP2 inside the publish roots declared.
    Reviewed,
    /// A changed download: see [`extract_served_archive`].
    Served,
}

struct ExtractionIdentity<'a> {
    digest: &'a str,
    contract_sha256: &'a str,
    expected_roots: &'a [String],
    expected_tp2_paths: &'a [String],
}

/// Validate a complete ZIP central directory, extract only approved payload roots into a
/// private temporary, and publish them under the archive and extraction-contract digests.
pub fn extract_archive(
    archive_path: &Path,
    extraction_cache: &Path,
    requirements: &ArchiveRequirements,
) -> Result<ExtractedArtifact, AcquireError> {
    extract_archive_reporting_repair(archive_path, extraction_cache, requirements)
        .map(|(extracted, _repair)| extracted)
}

/// [`extract_archive`], also reporting a damaged published extraction that was replaced.
///
/// A published extraction that no longer validates (missing or unreadable marker, changed,
/// missing or added files) is never used. It is set aside and the verified archive is
/// extracted again under the same identity. Validation, repair and publication of one
/// identity are serialized across processes by a lock in `<extraction_cache>/locks`.
pub fn extract_archive_reporting_repair(
    archive_path: &Path,
    extraction_cache: &Path,
    requirements: &ArchiveRequirements,
) -> Result<(ExtractedArtifact, Option<CacheRepair>), AcquireError> {
    extract_archive_with_stop(
        archive_path,
        extraction_cache,
        requirements,
        &StopSignal::never(),
    )
}

/// [`extract_archive_reporting_repair`], giving up with [`AcquireError::Stopped`] instead of
/// extracting once `stop` reports that the user stopped, also while another extraction of the
/// same identity holds its lock.
pub fn extract_archive_with_stop(
    archive_path: &Path,
    extraction_cache: &Path,
    requirements: &ArchiveRequirements,
    stop: &StopSignal,
) -> Result<(ExtractedArtifact, Option<CacheRepair>), AcquireError> {
    extract_with_rule(
        archive_path,
        extraction_cache,
        requirements,
        stop,
        LayoutRule::Reviewed,
    )
}

/// [`extract_archive_with_stop`] for an archive whose download differs from the reviewed one;
/// `requirements` name the served archive's own digest and usually
/// [`ArchiveLimits::for_served_archive`].
///
/// Every guard that protects this computer holds as before: entry names must be portable
/// relative paths (no traversal, absolute or drive paths, reserved device names or
/// case-insensitive duplicates), links and other special entries are refused, and the entry
/// count, sizes, depth and compression ratio are bounded before anything is written. What may
/// differ is where the reviewed payload sits. The reviewed rule is tried first; when it finds no
/// single place, the payload may sit below any chain of folders, also beside other files, as
/// long as exactly one place holds every reviewed TP2 path and publish root. Files outside that
/// place are not extracted. A TP2 inside the publish roots that the recipe does not declare is
/// accepted only when its installer name differs from every declared one, so WeiDU cannot pick
/// it instead of the reviewed installer.
pub fn extract_served_archive(
    archive_path: &Path,
    extraction_cache: &Path,
    requirements: &ArchiveRequirements,
    stop: &StopSignal,
) -> Result<(ExtractedArtifact, Option<CacheRepair>), AcquireError> {
    extract_with_rule(
        archive_path,
        extraction_cache,
        requirements,
        stop,
        LayoutRule::Served,
    )
}

/// Why a downloaded file that differs from its pin is clearly not an archive of `format`: it
/// does not begin like one, as an error page served in its place does not, or its framing is
/// incomplete, as after a transfer that ended early without announcing its length. `None` when
/// it is framed like a complete archive; unpacking still checks everything else.
///
/// Both checks are the ones unpacking would fail first, so a file refused here could never be
/// installed, while asking the link again may still give the real archive.
pub(super) fn served_container_problem(
    path: &Path,
    format: ArchiveFormat,
) -> Result<Option<String>, AcquireError> {
    let mut start = [0_u8; 32];
    let mut length = 0;
    let mut file = File::open(path).map_err(|source| AcquireError::Io {
        action: "open downloaded file",
        path: path.to_path_buf(),
        source,
    })?;
    while length < start.len() {
        let read = file
            .read(&mut start[length..])
            .map_err(|source| AcquireError::Io {
                action: "read downloaded file",
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        length += read;
    }
    let start = &start[..length];
    match format {
        ArchiveFormat::Zip | ArchiveFormat::Iemod => {
            if !start.starts_with(b"PK\x03\x04") {
                return Ok(Some(format!(
                    "it does not begin like a ZIP archive but with {}",
                    describe_start(start)
                )));
            }
            match ZipArchive::new(file) {
                Ok(_) => Ok(None),
                // Reading past the end or seeking to a place the file does not have is what a
                // directory that the cut-off end describes wrongly leads to; any other failure
                // is this computer's.
                Err(zip::result::ZipError::Io(source))
                    if !matches!(
                        source.kind(),
                        std::io::ErrorKind::UnexpectedEof
                            | std::io::ErrorKind::InvalidInput
                            | std::io::ErrorKind::InvalidData
                    ) =>
                {
                    Err(AcquireError::Io {
                        action: "read downloaded ZIP archive",
                        path: path.to_path_buf(),
                        source,
                    })
                }
                Err(error) => Ok(Some(format!("it is not a complete ZIP archive ({error})"))),
            }
        }
        ArchiveFormat::SelfExtractingRar => {
            if !start.starts_with(b"MZ") {
                return Ok(Some(format!(
                    "it does not begin like a self-extracting RAR archive but with {}",
                    describe_start(start)
                )));
            }
            drop(file);
            Ok(rar_container_problem(path))
        }
    }
}

/// The first bytes of a file, for a message: as text when they are text, else in hex.
fn describe_start(start: &[u8]) -> String {
    if start.is_empty() {
        return "no bytes at all".to_owned();
    }
    if start
        .iter()
        .all(|byte| byte.is_ascii_graphic() || byte.is_ascii_whitespace())
    {
        let text = start
            .iter()
            .map(|byte| {
                if byte.is_ascii_whitespace() {
                    ' '
                } else {
                    char::from(*byte)
                }
            })
            .collect::<String>();
        return format!("the text \"{}\"", text.trim());
    }
    let bytes = start
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>();
    format!("the bytes {}", bytes.join(" "))
}

/// Entries tested before the check of a RAR archive gives up and leaves the rest to
/// unpacking, which bounds the entry count itself.
#[cfg(windows)]
const MAX_RAR_FRAMING_HEADERS: usize = 10_000_000;

/// Whether UnRAR finds every file of a self-extracting RAR archive complete and intact, as
/// `unrar t` tests an archive. Its headers alone do not show a transfer that ended early: the
/// last file is merely cut short, which only reading its data reveals. An archive that
/// unpacking refuses whatever its state, such as a solid or a split one, is left to unpacking,
/// which says why; asking its link again could not help.
#[cfg(windows)]
fn rar_container_problem(path: &Path) -> Option<String> {
    let path = WideCString::from_os_str(path).ok()?;
    let mut open =
        unrar_sys::OpenArchiveDataEx::new(path.as_ptr().cast(), unrar_sys::RAR_OM_EXTRACT);
    let handle = unsafe { unrar_sys::RAROpenArchiveEx(std::ptr::from_mut(&mut open).cast_const()) };
    if handle.is_null() || open.open_result != unrar_sys::ERAR_SUCCESS as u32 {
        return Some(format!(
            "UnRAR cannot open it as an archive (code {})",
            open.open_result
        ));
    }
    struct Guard(*const unrar_sys::Handle);
    impl Drop for Guard {
        fn drop(&mut self) {
            unsafe { unrar_sys::RARCloseArchive(self.0) };
        }
    }
    let _guard = Guard(handle);
    if open.flags
        & (unrar_sys::ROADF_VOLUME
            | unrar_sys::ROADF_FIRSTVOLUME
            | unrar_sys::ROADF_ENCHEADERS
            | unrar_sys::ROADF_SOLID)
        != 0
    {
        return None;
    }
    for _ in 0..MAX_RAR_FRAMING_HEADERS {
        let mut header = unrar_sys::HeaderDataEx::default();
        let code = unsafe {
            unrar_sys::RARReadHeaderEx(handle, std::ptr::from_mut(&mut header).cast_const())
        };
        if code == unrar_sys::ERAR_END_ARCHIVE {
            return None;
        }
        if code != unrar_sys::ERAR_SUCCESS {
            return Some(format!(
                "its archive is damaged or ends early (UnRAR code {code})"
            ));
        }
        if header.flags
            & (unrar_sys::RHDF_SPLITBEFORE
                | unrar_sys::RHDF_SPLITAFTER
                | unrar_sys::RHDF_ENCRYPTED
                | unrar_sys::RHDF_SOLID)
            != 0
            || header.redir_type != 0
        {
            return None;
        }
        let announced = if header.flags & unrar_sys::RHDF_DIRECTORY != 0 {
            0
        } else {
            (u64::from(header.unp_size_high) << 32) | u64::from(header.unp_size)
        };
        let mut tested = RarTestState {
            length: 0,
            announced,
            refused: false,
        };
        unsafe {
            unrar_sys::RARSetCallback(
                handle,
                Some(rar_test_callback),
                std::ptr::from_mut(&mut tested) as unrar_sys::LPARAM,
            )
        };
        let process = unsafe {
            unrar_sys::RARProcessFileW(
                handle,
                unrar_sys::RAR_TEST,
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        unsafe { unrar_sys::RARSetCallback(handle, None, 0) };
        if process != unrar_sys::ERAR_SUCCESS || tested.refused || tested.length != announced {
            let name = unsafe {
                WideCString::from_ptr_truncate(header.filename_w.as_ptr(), header.filename_w.len())
            }
            .to_string_lossy()
            .replace('\\', "/");
            let entry = if name.is_empty() {
                "an entry".to_owned()
            } else {
                format!("`{name}`")
            };
            let detail = if tested.length < announced {
                format!(
                    "{entry} ends after {} of its {announced} bytes",
                    tested.length
                )
            } else if process == unrar_sys::ERAR_BAD_DATA {
                format!("{entry} fails its checksum")
            } else {
                format!("UnRAR stops at {entry}")
            };
            return Some(format!(
                "its archive is damaged or ends early: {detail} (UnRAR code {process})"
            ));
        }
    }
    None
}

/// What testing one RAR entry produced: its length, and whether it gave more than announced
/// or asked for something a test cannot answer.
#[cfg(windows)]
struct RarTestState {
    length: u64,
    announced: u64,
    refused: bool,
}

/// Counts the data UnRAR tests without keeping it; a password or volume request is refused.
#[cfg(windows)]
extern "C" fn rar_test_callback(
    message: unrar_sys::UINT,
    user_data: unrar_sys::LPARAM,
    _data: unrar_sys::LPARAM,
    length: unrar_sys::LPARAM,
) -> std::os::raw::c_int {
    if user_data == 0 {
        return -1;
    }
    let state = unsafe { &mut *(user_data as *mut RarTestState) };
    let next = if message == unrar_sys::UCM_PROCESSDATA {
        u64::try_from(length)
            .ok()
            .and_then(|chunk| state.length.checked_add(chunk))
            .filter(|next| *next <= state.announced)
    } else {
        None
    };
    match next {
        Some(next) => {
            state.length = next;
            0
        }
        None => {
            state.refused = true;
            -1
        }
    }
}

/// Self-extracting RAR archives are unpacked only by the Windows installer, which checks them.
#[cfg(not(windows))]
fn rar_container_problem(_path: &Path) -> Option<String> {
    None
}

fn extract_with_rule(
    archive_path: &Path,
    extraction_cache: &Path,
    requirements: &ArchiveRequirements,
    stop: &StopSignal,
    rule: LayoutRule,
) -> Result<(ExtractedArtifact, Option<CacheRepair>), AcquireError> {
    validate_limits(&requirements.limits)?;
    let expected_roots = normalize_expected_paths(&requirements.expected_roots, "root")?;
    let expected_tp2_paths =
        normalize_expected_paths(&requirements.expected_tp2_paths, "TP2 path")?;
    if expected_roots.is_empty() && expected_tp2_paths.is_empty() {
        return Err(AcquireError::InvalidRequest(
            "archive requirements need at least one publish root or TP2 path".to_owned(),
        ));
    }
    if expected_tp2_paths
        .iter()
        .any(|path| !path.to_ascii_lowercase().ends_with(".tp2"))
    {
        return Err(AcquireError::InvalidRequest(
            "every expected TP2 path must end in .tp2".to_owned(),
        ));
    }
    for path in &expected_tp2_paths {
        let owners = expected_roots
            .iter()
            .filter(|root| path_is_within_root(root, path))
            .count();
        if owners != 1 {
            return Err(AcquireError::InvalidRequest(format!(
                "expected TP2 path `{path}` has {owners} publish root owners; expected exactly one publish root owner"
            )));
        }
    }

    // Both supported kinds deliberately use the same framing. Keeping this match explicit makes
    // a future format addition fail compilation until extraction support is consciously chosen.
    match requirements.format {
        ArchiveFormat::Zip | ArchiveFormat::Iemod | ArchiveFormat::SelfExtractingRar => {}
    }

    let expected_digest = validate_digest(&requirements.artifact_sha256)?;
    if matches!(requirements.mode, ArchiveMode::Public)
        && expected_digest.bytes().all(|byte| byte == b'0')
    {
        return Err(AcquireError::PublicArchiveRejected(
            "an all-zero SHA-256 is not immutable provenance".to_owned(),
        ));
    }
    let snapshot_parent = extraction_cache.join("archive-snapshots");
    fs::create_dir_all(&snapshot_parent).map_err(|source| AcquireError::Io {
        action: "create archive snapshot directory",
        path: snapshot_parent.clone(),
        source,
    })?;
    let (snapshot, archive_length, actual_digest, prefix) =
        snapshot_archive(archive_path, &snapshot_parent)?;
    if expected_digest.bytes().any(|byte| byte != b'0') && actual_digest != expected_digest {
        return Err(AcquireError::HashMismatch {
            expected: expected_digest,
            actual: actual_digest,
        });
    }
    if matches!(
        requirements.format,
        ArchiveFormat::Zip | ArchiveFormat::Iemod
    ) && (archive_length < 4 || prefix != *b"PK\x03\x04")
    {
        let message = if matches!(requirements.mode, ArchiveMode::Public) {
            "self-extracting/prefixed ZIP files are not accepted"
        } else {
            "archive must begin with a ZIP local-file header"
        };
        return Err(AcquireError::PublicArchiveRejected(message.to_owned()));
    }
    if matches!(requirements.format, ArchiveFormat::SelfExtractingRar)
        && (archive_length < 2 || prefix[..2] != *b"MZ")
    {
        return Err(AcquireError::PublicArchiveRejected(
            "self-extracting RAR archive must begin with an MZ executable header".to_owned(),
        ));
    }

    let digest = actual_digest;
    // The same immutable archive can be used by recipes with different publish roots or
    // safety policies. A stricter policy must validate the archive anew, not reuse a tree
    // accepted under broader limits. Legacy digest-only namespaces remain untouched.
    let contract_sha256 =
        extraction_contract_sha256(requirements, &expected_roots, &expected_tp2_paths, rule)?;
    let identity = ExtractionIdentity {
        digest: &digest,
        contract_sha256: &contract_sha256,
        expected_roots: &expected_roots,
        expected_tp2_paths: &expected_tp2_paths,
    };
    let final_root = extraction_cache
        .join(EXTRACTION_CACHE_NAMESPACE)
        .join(&digest[..2])
        .join(&digest)
        .join(&contract_sha256);
    let locks = extraction_cache.join("locks");
    fs::create_dir_all(&locks).map_err(|source| AcquireError::Io {
        action: "create extraction lock directory",
        path: locks.clone(),
        source,
    })?;
    // Held until this identity is validated or republished. A request that finds the entry
    // damaged has validated it under the lock, so it can never set aside a replacement that
    // another request published in the meantime, and identical requests extract only once.
    let _lock = lock_entry(
        &locks.join(format!("{digest}.{contract_sha256}.lock")),
        stop,
    )?;
    let mut repair = None;
    if entry_is_present(&final_root)? {
        match validate_published_extraction(final_root.clone(), &identity) {
            Ok(extracted) => return Ok((extracted, None)),
            Err(error) => {
                let Some(reason) = damaged_entry_reason(&error, &[final_root.as_path()]) else {
                    return Err(error);
                };
                set_aside(extraction_cache, &final_root, &digest)?;
                repair = Some(CacheRepair { reason });
            }
        }
    }

    let (entries, wrapper_directory) = match requirements.format {
        ArchiveFormat::Zip | ArchiveFormat::Iemod => {
            let file = File::open(snapshot.path()).map_err(|source| AcquireError::Io {
                action: "open archive",
                path: archive_path.to_path_buf(),
                source,
            })?;
            let mut archive =
                ZipArchive::new(file).map_err(|source| AcquireError::ArchiveFormat {
                    path: archive_path.to_path_buf(),
                    message: source.to_string(),
                })?;
            validate_central_directory(
                &mut archive,
                snapshot.path(),
                &expected_roots,
                &expected_tp2_paths,
                &requirements.limits,
                rule,
            )?
        }
        ArchiveFormat::SelfExtractingRar => validate_rar_entries(
            snapshot.path(),
            &expected_roots,
            &expected_tp2_paths,
            &requirements.limits,
            rule,
        )?,
    };

    let temporary_parent = extraction_cache.join("temporary");
    fs::create_dir_all(&temporary_parent).map_err(|source| AcquireError::Io {
        action: "create extraction temporary directory",
        path: temporary_parent.clone(),
        source,
    })?;
    let temporary = create_temporary_directory(&temporary_parent, &digest)?;
    let result = match requirements.format {
        ArchiveFormat::Zip | ArchiveFormat::Iemod => extract_zip_entries(
            snapshot.path(),
            &entries,
            &temporary,
            wrapper_directory.clone(),
            &identity,
        ),
        ArchiveFormat::SelfExtractingRar => extract_rar_entries(
            snapshot.path(),
            &entries,
            &temporary,
            wrapper_directory.clone(),
            &identity,
        ),
    };
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }

    let parent = final_root
        .parent()
        .ok_or_else(|| AcquireError::InvalidRequest("extraction path has no parent".to_owned()))?;
    fs::create_dir_all(parent).map_err(|source| AcquireError::Io {
        action: "create extraction digest directory",
        path: parent.to_path_buf(),
        source,
    })?;
    match fs_retry::rename(&temporary, &final_root) {
        Ok(()) => {}
        // Another publisher that does not take the identity lock, such as an older CEBG
        // version sharing this cache, published first; its entry is validated, never repaired.
        Err(_source) if final_root.exists() => {
            let _ = fs::remove_dir_all(&temporary);
            return validate_published_extraction(final_root, &identity)
                .map(|extracted| (extracted, repair));
        }
        Err(source) => {
            let _ = fs::remove_dir_all(&temporary);
            return Err(AcquireError::Io {
                action: "publish validated extraction",
                path: final_root,
                source,
            });
        }
    }

    Ok((
        ExtractedArtifact {
            root: final_root,
            artifact_sha256: digest,
            extraction_contract_sha256: contract_sha256,
            wrapper_directory,
            expected_roots,
            expected_tp2_paths,
        },
        repair,
    ))
}

fn extraction_contract_sha256(
    requirements: &ArchiveRequirements,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
    rule: LayoutRule,
) -> Result<String, AcquireError> {
    // Named, fixed-order fields and JSON string/array framing avoid ambiguous concatenation.
    // Paths have already been validated, case-folded, separator-normalized, and sorted.
    let contract = ExtractionContract {
        version: EXTRACTION_MARKER_VERSION,
        format: requirements.format,
        mode: requirements.mode,
        expected_roots,
        expected_tp2_paths,
        limits: &requirements.limits,
        layout: (rule == LayoutRule::Served).then_some(SERVED_LAYOUT_RULE),
    };
    let bytes = serde_json::to_vec(&contract).map_err(|error| {
        AcquireError::InvalidRequest(format!("could not encode extraction contract: {error}"))
    })?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn validate_limits(limits: &ArchiveLimits) -> Result<(), AcquireError> {
    if limits.max_depth == 0
        || limits.max_entries == 0
        || limits.max_entry_uncompressed_bytes == 0
        || limits.max_total_uncompressed_bytes == 0
        || limits.max_compression_ratio == 0
    {
        return Err(AcquireError::InvalidRequest(
            "every archive limit must be greater than zero".to_owned(),
        ));
    }
    Ok(())
}

fn normalize_expected_paths(values: &[String], label: &str) -> Result<Vec<String>, AcquireError> {
    let mut normalized = Vec::with_capacity(values.len());
    let mut seen = BTreeSet::new();
    for value in values {
        let path = normalize_relative_path(value, false)?;
        let key = casefold(&path);
        if !seen.insert(key.clone()) {
            return Err(AcquireError::InvalidRequest(format!(
                "duplicate case-insensitive expected {label}: {value}"
            )));
        }
        normalized.push(key);
    }
    normalized.sort();
    Ok(normalized)
}

fn validate_digest(value: &str) -> Result<String, AcquireError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AcquireError::InvalidRequest(
            "artifact_sha256 must be exactly 64 hexadecimal characters".to_owned(),
        ));
    }
    Ok(value.to_ascii_lowercase())
}

fn snapshot_archive(
    path: &Path,
    snapshot_parent: &Path,
) -> Result<(tempfile::NamedTempFile, u64, String, [u8; 4]), AcquireError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| AcquireError::Io {
        action: "inspect archive",
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(AcquireError::InvalidRequest(
            "archive path must name a regular non-symlink file".to_owned(),
        ));
    }
    let mut input = File::open(path).map_err(|source| AcquireError::Io {
        action: "open archive for hashing",
        path: path.to_path_buf(),
        source,
    })?;
    let mut snapshot = tempfile::Builder::new()
        .prefix("verified-archive-")
        .tempfile_in(snapshot_parent)
        .map_err(|source| AcquireError::Io {
            action: "create archive snapshot",
            path: snapshot_parent.to_path_buf(),
            source,
        })?;
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut prefix = [0_u8; 4];
    let mut prefix_length = 0_usize;
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = input.read(&mut buffer).map_err(|source| AcquireError::Io {
            action: "read archive for hashing",
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        if prefix_length < prefix.len() {
            let copy = (prefix.len() - prefix_length).min(read);
            prefix[prefix_length..prefix_length + copy].copy_from_slice(&buffer[..copy]);
            prefix_length += copy;
        }
        length = length.checked_add(read as u64).ok_or_else(|| {
            AcquireError::ArchiveLimitExceeded("archive length overflowed u64".to_owned())
        })?;
        hash.update(&buffer[..read]);
        snapshot
            .write_all(&buffer[..read])
            .map_err(|source| AcquireError::Io {
                action: "write archive snapshot",
                path: snapshot.path().to_path_buf(),
                source,
            })?;
    }
    snapshot
        .as_file()
        .sync_all()
        .map_err(|source| AcquireError::Io {
            action: "sync archive snapshot",
            path: snapshot.path().to_path_buf(),
            source,
        })?;
    Ok((snapshot, length, hex::encode(hash.finalize()), prefix))
}

fn validate_central_directory<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    archive_path: &Path,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
    limits: &ArchiveLimits,
    rule: LayoutRule,
) -> Result<(Vec<ValidatedEntry>, Option<String>), AcquireError> {
    if archive.len() > limits.max_entries {
        return Err(AcquireError::ArchiveLimitExceeded(format!(
            "{} entries exceed the maximum of {}",
            archive.len(),
            limits.max_entries
        )));
    }
    let mut entries = Vec::with_capacity(archive.len());
    let mut destination_kinds = BTreeMap::<String, bool>::new();
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index_raw(index)
            .map_err(|source| AcquireError::ArchiveFormat {
                path: archive_path.to_path_buf(),
                message: source.to_string(),
            })?;
        let archive_name = entry.name().to_owned();
        let normalized = normalize_relative_path(&archive_name, entry.is_dir())?;
        let depth = normalized.split('/').count();
        if depth > limits.max_depth {
            return Err(AcquireError::ArchiveLimitExceeded(format!(
                "entry `{archive_name}` has depth {depth}, maximum {}",
                limits.max_depth
            )));
        }
        if entry.encrypted() {
            return Err(AcquireError::ArchiveFormat {
                path: archive_path.to_path_buf(),
                message: format!("encrypted entry `{archive_name}` is unsupported"),
            });
        }
        if entry.is_symlink() || is_unsupported_unix_kind(entry.unix_mode(), entry.is_dir()) {
            return Err(AcquireError::UnsafeArchiveEntry {
                entry: archive_name,
                message: "only regular files and directories are accepted".to_owned(),
            });
        }
        if !matches!(
            entry.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err(AcquireError::ArchiveFormat {
                path: archive_path.to_path_buf(),
                message: format!(
                    "unsupported compression {:?} for `{archive_name}`",
                    entry.compression()
                ),
            });
        }
        if !entry.is_dir() {
            if entry.size() > limits.max_entry_uncompressed_bytes {
                return Err(AcquireError::ArchiveLimitExceeded(format!(
                    "entry `{archive_name}` is {} bytes, maximum {}",
                    entry.size(),
                    limits.max_entry_uncompressed_bytes
                )));
            }
            total = total.checked_add(entry.size()).ok_or_else(|| {
                AcquireError::ArchiveLimitExceeded(
                    "aggregate uncompressed size overflowed u64".to_owned(),
                )
            })?;
            if total > limits.max_total_uncompressed_bytes {
                return Err(AcquireError::ArchiveLimitExceeded(format!(
                    "aggregate uncompressed size {total} exceeds {}",
                    limits.max_total_uncompressed_bytes
                )));
            }
            let compressed = entry.compressed_size().max(1);
            let allowed = compressed.saturating_mul(limits.max_compression_ratio);
            if entry.size() > allowed {
                return Err(AcquireError::ArchiveLimitExceeded(format!(
                    "entry `{archive_name}` exceeds compression ratio {}",
                    limits.max_compression_ratio
                )));
            }
        }
        let key = casefold(&normalized);
        if destination_kinds.insert(key, entry.is_dir()).is_some() {
            return Err(AcquireError::UnsafeArchiveEntry {
                entry: archive_name,
                message: "case-insensitive duplicate destination".to_owned(),
            });
        }
        entries.push(ValidatedEntry {
            index,
            archive_path: normalized.clone(),
            relative_path: normalized,
            size: entry.size(),
            is_directory: entry.is_dir(),
        });
    }

    finalize_entry_inventory(
        entries,
        destination_kinds,
        expected_roots,
        expected_tp2_paths,
        rule,
    )
}

#[cfg(not(windows))]
fn validate_rar_entries(
    archive_path: &Path,
    _expected_roots: &[String],
    _expected_tp2_paths: &[String],
    _limits: &ArchiveLimits,
    _rule: LayoutRule,
) -> Result<(Vec<ValidatedEntry>, Option<String>), AcquireError> {
    Err(AcquireError::ArchiveFormat {
        path: archive_path.to_path_buf(),
        message: "self-extracting RAR archives are supported only by the Windows installer"
            .to_owned(),
    })
}

#[cfg(windows)]
fn validate_rar_entries(
    archive_path: &Path,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
    limits: &ArchiveLimits,
    rule: LayoutRule,
) -> Result<(Vec<ValidatedEntry>, Option<String>), AcquireError> {
    let path = WideCString::from_os_str(archive_path).map_err(|_| {
        AcquireError::InvalidRequest("RAR archive path contains a NUL character".to_owned())
    })?;
    let mut open =
        unrar_sys::OpenArchiveDataEx::new(path.as_ptr().cast(), unrar_sys::RAR_OM_LIST_INCSPLIT);
    let handle = unsafe { unrar_sys::RAROpenArchiveEx(std::ptr::from_mut(&mut open).cast_const()) };
    if handle.is_null() || open.open_result != unrar_sys::ERAR_SUCCESS as u32 {
        return Err(AcquireError::ArchiveFormat {
            path: archive_path.to_path_buf(),
            message: format!("UnRAR open failed with code {}", open.open_result),
        });
    }
    struct Guard(*const unrar_sys::Handle);
    impl Drop for Guard {
        fn drop(&mut self) {
            unsafe { unrar_sys::RARCloseArchive(self.0) };
        }
    }
    let _guard = Guard(handle);
    if open.flags & (unrar_sys::ROADF_VOLUME | unrar_sys::ROADF_FIRSTVOLUME) != 0 {
        return Err(AcquireError::ArchiveFormat {
            path: archive_path.to_path_buf(),
            message: "multi-volume RAR archives are unsupported".to_owned(),
        });
    }
    if open.flags & unrar_sys::ROADF_ENCHEADERS != 0 {
        return Err(AcquireError::ArchiveFormat {
            path: archive_path.to_path_buf(),
            message: "encrypted RAR headers are unsupported".to_owned(),
        });
    }
    if open.flags & unrar_sys::ROADF_SOLID != 0 {
        return Err(AcquireError::ArchiveFormat {
            path: archive_path.to_path_buf(),
            message: "solid RAR archives are unsupported".to_owned(),
        });
    }

    let mut entries = Vec::new();
    let mut destination_kinds = BTreeMap::<String, bool>::new();
    let mut total = 0_u64;
    loop {
        let mut header = unrar_sys::HeaderDataEx::default();
        let code = unsafe {
            unrar_sys::RARReadHeaderEx(handle, std::ptr::from_mut(&mut header).cast_const())
        };
        if code == unrar_sys::ERAR_END_ARCHIVE {
            break;
        }
        if code != unrar_sys::ERAR_SUCCESS {
            return Err(AcquireError::ArchiveFormat {
                path: archive_path.to_path_buf(),
                message: format!("UnRAR header read failed with code {code}"),
            });
        }
        if entries.len() >= limits.max_entries {
            return Err(AcquireError::ArchiveLimitExceeded(format!(
                "RAR entries exceed the maximum of {}",
                limits.max_entries
            )));
        }
        let name = unsafe {
            WideCString::from_ptr_truncate(header.filename_w.as_ptr(), header.filename_w.len())
        }
        .to_os_string()
        .into_string()
        .map_err(|_| AcquireError::UnsafeArchiveEntry {
            entry: "<non-Unicode RAR path>".to_owned(),
            message: "archive entry path is not Unicode".to_owned(),
        })?;
        let is_directory = header.flags & unrar_sys::RHDF_DIRECTORY != 0;
        let normalized = normalize_relative_path(&name, is_directory)?;
        let depth = normalized.split('/').count();
        if depth > limits.max_depth {
            return Err(AcquireError::ArchiveLimitExceeded(format!(
                "entry `{name}` has depth {depth}, maximum {}",
                limits.max_depth
            )));
        }
        if header.flags
            & (unrar_sys::RHDF_SPLITBEFORE
                | unrar_sys::RHDF_SPLITAFTER
                | unrar_sys::RHDF_ENCRYPTED
                | unrar_sys::RHDF_SOLID)
            != 0
        {
            return Err(AcquireError::ArchiveFormat {
                path: archive_path.to_path_buf(),
                message: format!("split, encrypted, or solid RAR entry `{name}` is unsupported"),
            });
        }
        if header.redir_type != 0 {
            return Err(AcquireError::UnsafeArchiveEntry {
                entry: name,
                message: "RAR links, junctions, references, and redirections are unsupported"
                    .to_owned(),
            });
        }
        if header.host_os > 3 {
            return Err(AcquireError::UnsafeArchiveEntry {
                entry: name,
                message: format!("unknown RAR host OS {}", header.host_os),
            });
        }
        if header.host_os == 3 {
            let unix_kind = header.file_attr & 0xf000;
            let expected_kind = if is_directory { 0x4000 } else { 0x8000 };
            if unix_kind != expected_kind {
                return Err(AcquireError::UnsafeArchiveEntry {
                    entry: name,
                    message: "only regular files and directories are accepted".to_owned(),
                });
            }
        }
        let size = (u64::from(header.unp_size_high) << 32) | u64::from(header.unp_size);
        let packed = (u64::from(header.pack_size_high) << 32) | u64::from(header.pack_size);
        if !is_directory {
            if size > limits.max_entry_uncompressed_bytes {
                return Err(AcquireError::ArchiveLimitExceeded(format!(
                    "entry `{normalized}` is {size} bytes, maximum {}",
                    limits.max_entry_uncompressed_bytes
                )));
            }
            total = total.checked_add(size).ok_or_else(|| {
                AcquireError::ArchiveLimitExceeded(
                    "aggregate uncompressed size overflowed u64".to_owned(),
                )
            })?;
            if total > limits.max_total_uncompressed_bytes {
                return Err(AcquireError::ArchiveLimitExceeded(format!(
                    "aggregate uncompressed size {total} exceeds {}",
                    limits.max_total_uncompressed_bytes
                )));
            }
            if size > packed.max(1).saturating_mul(limits.max_compression_ratio) {
                return Err(AcquireError::ArchiveLimitExceeded(format!(
                    "entry `{normalized}` exceeds compression ratio {}",
                    limits.max_compression_ratio
                )));
            }
        }
        let key = casefold(&normalized);
        if destination_kinds.insert(key, is_directory).is_some() {
            return Err(AcquireError::UnsafeArchiveEntry {
                entry: normalized,
                message: "case-insensitive duplicate destination".to_owned(),
            });
        }
        entries.push(ValidatedEntry {
            index: entries.len(),
            archive_path: normalized.clone(),
            relative_path: normalized,
            size,
            is_directory,
        });
        let process = unsafe {
            unrar_sys::RARProcessFileW(
                handle,
                unrar_sys::RAR_SKIP,
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        if process != unrar_sys::ERAR_SUCCESS {
            return Err(AcquireError::ArchiveFormat {
                path: archive_path.to_path_buf(),
                message: format!("UnRAR skip failed with code {process}"),
            });
        }
    }
    finalize_entry_inventory(
        entries,
        destination_kinds,
        expected_roots,
        expected_tp2_paths,
        rule,
    )
}

fn finalize_entry_inventory(
    mut entries: Vec<ValidatedEntry>,
    destination_kinds: BTreeMap<String, bool>,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
    rule: LayoutRule,
) -> Result<(Vec<ValidatedEntry>, Option<String>), AcquireError> {
    for (key, _) in destination_kinds
        .iter()
        .filter(|(_, is_directory)| !**is_directory)
    {
        // Keys that start with `key/` sort right at or after `key/`, so the first key from
        // there on is one of them whenever any exists.
        let prefix = format!("{key}/");
        if destination_kinds
            .range(prefix.clone()..)
            .next()
            .is_some_and(|(other, _)| other.starts_with(&prefix))
        {
            return Err(AcquireError::UnsafeArchiveEntry {
                entry: key.clone(),
                message: "file/directory prefix collision".to_owned(),
            });
        }
    }
    let files = FileIndex::new(&entries);
    let wrapper = match rule {
        LayoutRule::Reviewed => select_wrapper(&files, expected_roots, expected_tp2_paths)?,
        LayoutRule::Served => select_served_prefix(&files, expected_roots, expected_tp2_paths)?,
    };
    let wrapper_key = wrapper.as_ref().map(|value| casefold(value));
    match (rule, wrapper_key.as_deref()) {
        (_, None) => {}
        (LayoutRule::Reviewed, Some(wrapper_key)) => {
            entries.retain(|entry| casefold(&entry.archive_path) != wrapper_key);
            for entry in &mut entries {
                let prefix = format!("{wrapper_key}/");
                if !casefold(&entry.archive_path).starts_with(&prefix) {
                    return Err(AcquireError::ArchiveLayout(format!(
                        "entry `{}` is outside wrapper `{}`",
                        entry.archive_path,
                        wrapper.as_deref().unwrap_or_default()
                    )));
                }
                let separator = entry.archive_path.find('/').ok_or_else(|| {
                    AcquireError::ArchiveLayout("wrapper directory contains no payload".to_owned())
                })?;
                entry.relative_path = entry.archive_path[separator + 1..].to_owned();
            }
        }
        (LayoutRule::Served, Some(prefix_key)) => {
            // Only what lies below the payload folder is extracted: the folders leading to it
            // and anything beside them stay in the archive.
            let depth = prefix_key.split('/').count();
            let prefix = format!("{prefix_key}/");
            entries.retain_mut(|entry| {
                if !casefold(&entry.archive_path).starts_with(&prefix) {
                    return false;
                }
                match entry.archive_path.splitn(depth + 1, '/').nth(depth) {
                    Some(relative) => {
                        entry.relative_path = relative.to_owned();
                        true
                    }
                    None => false,
                }
            });
        }
    }
    let expected_tp2_keys: BTreeSet<_> = expected_tp2_paths
        .iter()
        .map(|path| casefold(path))
        .collect();
    let declared_installers: BTreeSet<_> = expected_tp2_keys
        .iter()
        .map(|path| installer_name(path))
        .collect();
    for entry in entries.iter().filter(|entry| !entry.is_directory) {
        let key = casefold(&entry.relative_path);
        let inside = expected_roots
            .iter()
            .any(|root| path_is_within_root(root, &entry.relative_path));
        if inside && key.ends_with(".tp2") && !expected_tp2_keys.contains(&key) {
            // A changed archive may bring another installer, but never one that WeiDU could
            // find under the name of a reviewed installer.
            if rule == LayoutRule::Served && !declared_installers.contains(installer_name(&key)) {
                continue;
            }
            return Err(AcquireError::ArchiveLayout(format!(
                "undeclared TP2 `{}`",
                entry.relative_path
            )));
        }
    }
    entries.retain(|entry| {
        expected_roots
            .iter()
            .any(|root| path_is_within_root(root, &entry.relative_path))
    });
    Ok((entries, wrapper))
}

/// An archive's regular files: their paths as spelled in the archive, in entry order, and
/// their case-folded paths sorted, so that layout checks are lookups instead of scans.
struct FileIndex<'a> {
    paths: Vec<&'a str>,
    keys: BTreeSet<String>,
}

impl<'a> FileIndex<'a> {
    fn new(entries: &'a [ValidatedEntry]) -> Self {
        let paths: Vec<&str> = entries
            .iter()
            .filter(|entry| !entry.is_directory)
            .map(|entry| entry.archive_path.as_str())
            .collect();
        let keys = paths.iter().map(|path| casefold(path)).collect();
        Self { paths, keys }
    }

    /// Whether a case-folded file path equals `key`.
    fn contains(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    /// Whether a case-folded file path starts with `prefix`. Such paths sort right at or after
    /// `prefix`, so the first path from there on is one of them whenever any exists.
    fn any_starts_with(&self, prefix: &str) -> bool {
        self.keys
            .range::<str, _>((
                std::ops::Bound::Included(prefix),
                std::ops::Bound::Unbounded,
            ))
            .next()
            .is_some_and(|key| key.starts_with(prefix))
    }

    /// Whether every case-folded file path starts with `prefix`. Those paths form one run in
    /// sorted order, so the first and the last path decide.
    fn all_start_with(&self, prefix: &str) -> bool {
        match (self.keys.first(), self.keys.last()) {
            (Some(first), Some(last)) => first.starts_with(prefix) && last.starts_with(prefix),
            _ => true,
        }
    }
}

fn select_wrapper(
    files: &FileIndex<'_>,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
) -> Result<Option<String>, AcquireError> {
    let mut candidates = Vec::<Option<String>>::new();
    if layout_matches(files, None, expected_roots, expected_tp2_paths) {
        candidates.push(None);
    }
    let mut wrappers = BTreeMap::<String, String>::new();
    for path in &files.paths {
        if let Some((first, _)) = path.split_once('/') {
            wrappers
                .entry(casefold(first))
                .or_insert_with(|| first.to_owned());
        }
    }
    // A wrapper must hold every file, so only a folder that the first file lies below can
    // pass; checking that first keeps this linear in the number of top-level folders.
    let first_file = files.paths.first().map(|path| casefold(path));
    for (key, wrapper) in wrappers {
        if first_file
            .as_deref()
            .is_some_and(|first| !first.starts_with(&format!("{key}/")))
        {
            continue;
        }
        if layout_matches(
            files,
            Some(wrapper.as_str()),
            expected_roots,
            expected_tp2_paths,
        ) {
            candidates.push(Some(wrapper));
        }
    }
    match candidates.len() {
        1 => Ok(candidates.pop().expect("one candidate")),
        0 => Err(AcquireError::ArchiveLayout(
            "expected TP2 paths were not found under an exact payload root".to_owned(),
        )),
        _ => Err(AcquireError::ArchiveLayout(
            "more than one possible TP2 payload root was found".to_owned(),
        )),
    }
}

fn layout_matches(
    files: &FileIndex<'_>,
    wrapper: Option<&str>,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
) -> bool {
    if let Some(wrapper) = wrapper {
        if !files.all_start_with(&format!("{}/", casefold(wrapper))) {
            return false;
        }
    }
    layout_matches_below(files, wrapper, expected_roots, expected_tp2_paths)
}

/// The folder chain in front of the reviewed payload inside a changed archive.
///
/// The reviewed rule comes first. When it finds no single place, every place where the first
/// reviewed TP2 path ends (or, for an archive without TP2 files, where the first publish root
/// occurs) is a candidate, and exactly one candidate must hold every reviewed TP2 path and
/// publish root. No candidate, or more than one, fails instead of guessing.
fn select_served_prefix(
    files: &FileIndex<'_>,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
) -> Result<Option<String>, AcquireError> {
    if let Ok(prefix) = select_wrapper(files, expected_roots, expected_tp2_paths) {
        return Ok(prefix);
    }
    let (anchor, anchor_ends_path) = match expected_tp2_paths.first() {
        Some(tp2) => (tp2, true),
        None => (
            expected_roots.first().ok_or_else(|| {
                AcquireError::ArchiveLayout("no reviewed payload path to look for".to_owned())
            })?,
            false,
        ),
    };
    let anchor: Vec<String> = anchor.split('/').map(casefold).collect();
    let mut candidates = BTreeMap::<String, Option<String>>::new();
    for path in &files.paths {
        let components: Vec<&str> = path.split('/').collect();
        let keys: Vec<String> = components.iter().map(|part| casefold(part)).collect();
        if keys.len() < anchor.len() {
            continue;
        }
        let last_start = keys.len() - anchor.len();
        let starts = if anchor_ends_path {
            last_start..=last_start
        } else {
            0..=last_start
        };
        for start in starts {
            if keys[start..start + anchor.len()] == anchor[..] {
                let prefix = (start > 0).then(|| components[..start].join("/"));
                candidates
                    .entry(prefix.as_deref().map(casefold).unwrap_or_default())
                    .or_insert(prefix);
            }
        }
    }
    let mut matching: Vec<Option<String>> = candidates
        .into_values()
        .filter(|prefix| {
            layout_matches_below(files, prefix.as_deref(), expected_roots, expected_tp2_paths)
        })
        .collect();
    match matching.len() {
        1 => Ok(matching.pop().expect("one candidate")),
        0 => Err(AcquireError::ArchiveLayout(format!(
            "the reviewed `{}` was not found together with every reviewed payload path",
            anchor.join("/")
        ))),
        _ => Err(AcquireError::ArchiveLayout(
            "the reviewed payload was found in more than one place".to_owned(),
        )),
    }
}

/// Whether every reviewed TP2 path and publish root exists below `prefix`; other files may lie
/// beside it.
fn layout_matches_below(
    files: &FileIndex<'_>,
    prefix: Option<&str>,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
) -> bool {
    // A file's path with `prefix/` removed equals `path` exactly when its whole case-folded
    // path equals the case-folded prefix, `/` and the case-folded `path`.
    let prefix = prefix
        .map(|value| format!("{}/", casefold(value)))
        .unwrap_or_default();
    expected_tp2_paths
        .iter()
        .all(|expected| files.contains(&format!("{prefix}{}", casefold(expected))))
        && expected_roots.iter().all(|root| {
            let root = format!("{prefix}{}", casefold(root));
            files.contains(&root) || files.any_starts_with(&format!("{root}/"))
        })
}

/// The installer name WeiDU derives from a TP2 file: its file name without a `setup-` prefix
/// and the `.tp2` extension. `key` is already case-folded.
fn installer_name(key: &str) -> &str {
    let file = key.rsplit('/').next().unwrap_or(key);
    let stem = file.strip_suffix(".tp2").unwrap_or(file);
    stem.strip_prefix("setup-").unwrap_or(stem)
}

fn path_is_within_root(root: &str, path: &str) -> bool {
    let root = casefold(root);
    let path = casefold(path);
    path == root || path.starts_with(&format!("{root}/"))
}

fn extract_zip_entries(
    archive_path: &Path,
    entries: &[ValidatedEntry],
    temporary: &Path,
    wrapper_directory: Option<String>,
    identity: &ExtractionIdentity<'_>,
) -> Result<(), AcquireError> {
    let file = File::open(archive_path).map_err(|source| AcquireError::Io {
        action: "open archive",
        path: archive_path.to_path_buf(),
        source,
    })?;
    let mut archive = ZipArchive::new(file).map_err(|source| AcquireError::ArchiveFormat {
        path: archive_path.to_path_buf(),
        message: source.to_string(),
    })?;
    let mut records = Vec::new();
    for plan in entries.iter().filter(|entry| !entry.is_directory) {
        let output = join_relative(temporary, &plan.relative_path);
        let parent = output.parent().ok_or_else(|| {
            AcquireError::InvalidRequest("archive output has no parent".to_owned())
        })?;
        fs::create_dir_all(parent).map_err(|source| AcquireError::Io {
            action: "create extracted file directory",
            path: parent.to_path_buf(),
            source,
        })?;
        let mut input =
            archive
                .by_index(plan.index)
                .map_err(|source| AcquireError::ArchiveFormat {
                    path: PathBuf::from(&plan.archive_path),
                    message: source.to_string(),
                })?;
        let mut output_file = File::create(&output).map_err(|source| AcquireError::Io {
            action: "create extracted file",
            path: output.clone(),
            source,
        })?;
        let mut hash = Sha256::new();
        let mut length = 0_u64;
        let mut buffer = [0_u8; BUFFER_SIZE];
        loop {
            let read = input.read(&mut buffer).map_err(|source| AcquireError::Io {
                action: "read compressed archive entry",
                path: PathBuf::from(&plan.archive_path),
                source,
            })?;
            if read == 0 {
                break;
            }
            length = length.checked_add(read as u64).ok_or_else(|| {
                AcquireError::ArchiveLimitExceeded("extracted length overflowed u64".to_owned())
            })?;
            if length > plan.size {
                return Err(AcquireError::ArchiveFormat {
                    path: PathBuf::from(&plan.archive_path),
                    message: "entry produced more bytes than its central-directory size".to_owned(),
                });
            }
            output_file
                .write_all(&buffer[..read])
                .map_err(|source| AcquireError::Io {
                    action: "write extracted file",
                    path: output.clone(),
                    source,
                })?;
            hash.update(&buffer[..read]);
        }
        if length != plan.size {
            return Err(AcquireError::ArchiveFormat {
                path: PathBuf::from(&plan.archive_path),
                message: format!("entry announced {} bytes but produced {length}", plan.size),
            });
        }
        output_file.sync_all().map_err(|source| AcquireError::Io {
            action: "sync extracted file",
            path: output.clone(),
            source,
        })?;
        records.push(ExtractedFile {
            relative_path: plan.relative_path.clone(),
            length,
            sha256: hex::encode(hash.finalize()),
        });
    }
    write_extraction_marker(temporary, wrapper_directory, identity, records)
}

#[cfg(not(windows))]
fn extract_rar_entries(
    archive_path: &Path,
    _entries: &[ValidatedEntry],
    _temporary: &Path,
    _wrapper_directory: Option<String>,
    _identity: &ExtractionIdentity<'_>,
) -> Result<(), AcquireError> {
    Err(AcquireError::ArchiveFormat {
        path: archive_path.to_path_buf(),
        message: "self-extracting RAR archives are supported only by the Windows installer"
            .to_owned(),
    })
}

#[cfg(windows)]
struct RarCallbackState {
    output: Option<File>,
    hash: Sha256,
    length: u64,
    expected_length: u64,
    error: Option<String>,
}

#[cfg(windows)]
extern "C" fn rar_callback(
    message: unrar_sys::UINT,
    user_data: unrar_sys::LPARAM,
    data: unrar_sys::LPARAM,
    length: unrar_sys::LPARAM,
) -> std::os::raw::c_int {
    if user_data == 0 {
        return -1;
    }
    let state = unsafe { &mut *(user_data as *mut RarCallbackState) };
    if message != unrar_sys::UCM_PROCESSDATA {
        state.error = Some("RAR requested a password or another volume".to_owned());
        return -1;
    }
    if data == 0 || length < 0 {
        state.error = Some("UnRAR returned an invalid data callback".to_owned());
        return -1;
    }
    let Ok(chunk_length) = usize::try_from(length) else {
        state.error = Some("UnRAR callback length does not fit usize".to_owned());
        return -1;
    };
    let Ok(chunk_length_u64) = u64::try_from(chunk_length) else {
        state.error = Some("UnRAR callback length does not fit u64".to_owned());
        return -1;
    };
    let Some(next_length) = state.length.checked_add(chunk_length_u64) else {
        state.error = Some("RAR extracted length overflowed u64".to_owned());
        return -1;
    };
    if next_length > state.expected_length {
        state.error = Some(format!(
            "RAR entry produced more than its declared {} bytes",
            state.expected_length
        ));
        return -1;
    }
    let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, chunk_length) };
    let Some(output) = state.output.as_mut() else {
        state.error = Some("RAR produced data for an unselected entry".to_owned());
        return -1;
    };
    if let Err(error) = output.write_all(bytes) {
        state.error = Some(format!("write extracted RAR file: {error}"));
        return -1;
    }
    state.hash.update(bytes);
    state.length = next_length;
    0
}

#[cfg(windows)]
fn extract_rar_entries(
    archive_path: &Path,
    entries: &[ValidatedEntry],
    temporary: &Path,
    wrapper_directory: Option<String>,
    identity: &ExtractionIdentity<'_>,
) -> Result<(), AcquireError> {
    let path = WideCString::from_os_str(archive_path).map_err(|_| {
        AcquireError::InvalidRequest("RAR archive path contains a NUL character".to_owned())
    })?;
    let mut open =
        unrar_sys::OpenArchiveDataEx::new(path.as_ptr().cast(), unrar_sys::RAR_OM_EXTRACT);
    let handle = unsafe { unrar_sys::RAROpenArchiveEx(std::ptr::from_mut(&mut open).cast_const()) };
    if handle.is_null() || open.open_result != unrar_sys::ERAR_SUCCESS as u32 {
        return Err(AcquireError::ArchiveFormat {
            path: archive_path.to_path_buf(),
            message: format!("UnRAR open failed with code {}", open.open_result),
        });
    }
    struct Guard(*const unrar_sys::Handle);
    impl Drop for Guard {
        fn drop(&mut self) {
            unsafe { unrar_sys::RARCloseArchive(self.0) };
        }
    }
    let _guard = Guard(handle);
    if open.flags
        & (unrar_sys::ROADF_VOLUME
            | unrar_sys::ROADF_FIRSTVOLUME
            | unrar_sys::ROADF_ENCHEADERS
            | unrar_sys::ROADF_SOLID)
        != 0
    {
        return Err(AcquireError::ArchiveFormat {
            path: archive_path.to_path_buf(),
            message: "RAR archive flags changed between validation and extraction".to_owned(),
        });
    }
    let selected = entries
        .iter()
        .filter(|entry| !entry.is_directory)
        .map(|entry| (entry.index, entry))
        .collect::<BTreeMap<_, _>>();
    let mut extracted_indices = BTreeSet::new();
    let mut records = Vec::new();
    let mut index = 0_usize;
    loop {
        let mut header = unrar_sys::HeaderDataEx::default();
        let code = unsafe {
            unrar_sys::RARReadHeaderEx(handle, std::ptr::from_mut(&mut header).cast_const())
        };
        if code == unrar_sys::ERAR_END_ARCHIVE {
            break;
        }
        if code != unrar_sys::ERAR_SUCCESS {
            return Err(AcquireError::ArchiveFormat {
                path: archive_path.to_path_buf(),
                message: format!("UnRAR header read failed with code {code}"),
            });
        }
        let current_name = unsafe {
            WideCString::from_ptr_truncate(header.filename_w.as_ptr(), header.filename_w.len())
        }
        .to_string_lossy()
        .replace('\\', "/");
        if let Some(plan) = selected.get(&index) {
            if !current_name.eq_ignore_ascii_case(&plan.archive_path)
                || ((u64::from(header.unp_size_high) << 32) | u64::from(header.unp_size))
                    != plan.size
            {
                return Err(AcquireError::ArchiveFormat {
                    path: archive_path.to_path_buf(),
                    message: format!(
                        "RAR entry metadata changed before extraction at `{current_name}`"
                    ),
                });
            }
            let output = join_relative(temporary, &plan.relative_path);
            let parent = output.parent().ok_or_else(|| {
                AcquireError::InvalidRequest("archive output has no parent".to_owned())
            })?;
            fs::create_dir_all(parent).map_err(|source| AcquireError::Io {
                action: "create extracted file directory",
                path: parent.to_path_buf(),
                source,
            })?;
            let output_file = File::create(&output).map_err(|source| AcquireError::Io {
                action: "create extracted RAR file",
                path: output.clone(),
                source,
            })?;
            let mut state = RarCallbackState {
                output: Some(output_file),
                hash: Sha256::new(),
                length: 0,
                expected_length: plan.size,
                error: None,
            };
            unsafe {
                unrar_sys::RARSetCallback(
                    handle,
                    Some(rar_callback),
                    std::ptr::from_mut(&mut state) as unrar_sys::LPARAM,
                )
            };
            let process = unsafe {
                unrar_sys::RARProcessFileW(
                    handle,
                    unrar_sys::RAR_TEST,
                    std::ptr::null(),
                    std::ptr::null(),
                )
            };
            unsafe { unrar_sys::RARSetCallback(handle, None, 0) };
            if let Some(message) = state.error {
                return Err(AcquireError::ArchiveFormat {
                    path: PathBuf::from(&plan.archive_path),
                    message,
                });
            }
            if process != unrar_sys::ERAR_SUCCESS || state.length != plan.size {
                return Err(AcquireError::ArchiveFormat {
                    path: PathBuf::from(&plan.archive_path),
                    message: format!(
                        "UnRAR returned {process}; entry announced {} bytes but produced {}",
                        plan.size, state.length
                    ),
                });
            }
            let output_file = state
                .output
                .take()
                .ok_or_else(|| AcquireError::ArchiveFormat {
                    path: output.clone(),
                    message: "RAR output file disappeared during extraction".to_owned(),
                })?;
            output_file.sync_all().map_err(|source| AcquireError::Io {
                action: "sync extracted RAR file",
                path: output.clone(),
                source,
            })?;
            let sha256 = hex::encode(state.hash.finalize());
            let metadata = fs::symlink_metadata(&output).map_err(|source| AcquireError::Io {
                action: "inspect extracted RAR file",
                path: output.clone(),
                source,
            })?;
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() != plan.size
            {
                return Err(AcquireError::ArchiveFormat {
                    path: output,
                    message: format!("RAR entry announced {} bytes but did not produce one regular file of that size", plan.size),
                });
            }
            records.push(ExtractedFile {
                relative_path: plan.relative_path.clone(),
                length: state.length,
                sha256,
            });
            extracted_indices.insert(index);
        } else {
            let mut state = RarCallbackState {
                output: None,
                hash: Sha256::new(),
                length: 0,
                expected_length: 0,
                error: None,
            };
            unsafe {
                unrar_sys::RARSetCallback(
                    handle,
                    Some(rar_callback),
                    std::ptr::from_mut(&mut state) as unrar_sys::LPARAM,
                )
            };
            let process = unsafe {
                unrar_sys::RARProcessFileW(
                    handle,
                    unrar_sys::RAR_SKIP,
                    std::ptr::null(),
                    std::ptr::null(),
                )
            };
            unsafe { unrar_sys::RARSetCallback(handle, None, 0) };
            if let Some(message) = state.error {
                return Err(AcquireError::ArchiveFormat {
                    path: archive_path.to_path_buf(),
                    message,
                });
            }
            if process != unrar_sys::ERAR_SUCCESS {
                return Err(AcquireError::ArchiveFormat {
                    path: archive_path.to_path_buf(),
                    message: format!("UnRAR skip failed with code {process}"),
                });
            }
        }
        index = index.checked_add(1).ok_or_else(|| {
            AcquireError::ArchiveLimitExceeded("RAR entry index overflowed usize".to_owned())
        })?;
    }
    if extracted_indices.len() != selected.len() {
        return Err(AcquireError::ArchiveFormat {
            path: archive_path.to_path_buf(),
            message: "not every validated RAR entry was extracted".to_owned(),
        });
    }
    write_extraction_marker(temporary, wrapper_directory, identity, records)
}

fn write_extraction_marker(
    temporary: &Path,
    wrapper_directory: Option<String>,
    identity: &ExtractionIdentity<'_>,
    mut records: Vec<ExtractedFile>,
) -> Result<(), AcquireError> {
    records.sort_by_key(|record| casefold(&record.relative_path));
    let marker = ExtractionMarker {
        version: EXTRACTION_MARKER_VERSION,
        artifact_sha256: identity.digest.to_owned(),
        contract_sha256: identity.contract_sha256.to_owned(),
        wrapper_directory,
        expected_roots: identity.expected_roots.to_vec(),
        expected_tp2_paths: identity.expected_tp2_paths.to_vec(),
        files: records,
    };
    let marker_path = temporary.join(EXTRACTION_MARKER);
    let mut bytes =
        serde_json::to_vec_pretty(&marker).map_err(|source| AcquireError::Metadata {
            path: marker_path.clone(),
            message: source.to_string(),
        })?;
    bytes.push(b'\n');
    let mut file = File::create(&marker_path).map_err(|source| AcquireError::Io {
        action: "create extraction marker",
        path: marker_path.clone(),
        source,
    })?;
    file.write_all(&bytes).map_err(|source| AcquireError::Io {
        action: "write extraction marker",
        path: marker_path.clone(),
        source,
    })?;
    file.sync_all().map_err(|source| AcquireError::Io {
        action: "sync extraction marker",
        path: marker_path,
        source,
    })
}

fn validate_published_extraction(
    root: PathBuf,
    identity: &ExtractionIdentity<'_>,
) -> Result<ExtractedArtifact, AcquireError> {
    let ExtractionIdentity {
        digest,
        contract_sha256,
        expected_roots,
        expected_tp2_paths,
    } = *identity;
    let metadata = fs::symlink_metadata(&root).map_err(|source| AcquireError::Io {
        action: "inspect published extraction",
        path: root.clone(),
        source,
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: "published extraction root is not a regular directory".to_owned(),
        });
    }
    let marker_path = root.join(EXTRACTION_MARKER);
    let marker_metadata =
        fs::symlink_metadata(&marker_path).map_err(|source| AcquireError::Io {
            action: "inspect extraction marker",
            path: marker_path.clone(),
            source,
        })?;
    if !marker_metadata.is_file() || marker_metadata.file_type().is_symlink() {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: "extraction marker is not a regular file".to_owned(),
        });
    }
    // Read before parsing so that a read failure keeps its I/O kind: only a marker that was
    // read completely and does not parse proves damage.
    let marker_bytes = fs::read(&marker_path).map_err(|source| AcquireError::Io {
        action: "read extraction marker",
        path: marker_path.clone(),
        source,
    })?;
    let marker: ExtractionMarker =
        serde_json::from_slice(&marker_bytes).map_err(|source| AcquireError::Metadata {
            path: marker_path,
            message: source.to_string(),
        })?;
    if marker.version != EXTRACTION_MARKER_VERSION
        || marker.artifact_sha256 != digest
        || marker.contract_sha256 != contract_sha256
        || marker.expected_roots != expected_roots
        || marker.expected_tp2_paths != expected_tp2_paths
    {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: "extraction marker does not match the requested archive layout".to_owned(),
        });
    }
    let mut expected_files = BTreeMap::new();
    let mut missing_tp2s = expected_tp2_paths
        .iter()
        .map(|path| (casefold(path), path))
        .collect::<BTreeMap<_, _>>();
    let mut root_has_file = vec![false; expected_roots.len()];
    for record in &marker.files {
        let relative_path =
            normalize_relative_path(&record.relative_path, false).map_err(|error| {
                AcquireError::CorruptCache {
                    digest: digest.to_owned(),
                    message: format!("invalid marker path `{}`: {error}", record.relative_path),
                }
            })?;
        let mut owned = false;
        for (index, root) in expected_roots.iter().enumerate() {
            if path_is_within_root(root, &relative_path) {
                root_has_file[index] = true;
                owned = true;
            }
        }
        if !owned {
            return Err(AcquireError::CorruptCache {
                digest: digest.to_owned(),
                message: format!(
                    "extraction marker path `{relative_path}` is outside declared publish roots"
                ),
            });
        }
        missing_tp2s.remove(&casefold(&relative_path));
        if record.sha256.len() != 64 || !record.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(AcquireError::CorruptCache {
                digest: digest.to_owned(),
                message: format!("invalid marker digest for `{relative_path}`"),
            });
        }
        if expected_files
            .insert(casefold(&relative_path), (relative_path, record))
            .is_some()
        {
            return Err(AcquireError::CorruptCache {
                digest: digest.to_owned(),
                message: "duplicate case-insensitive extraction marker path".to_owned(),
            });
        }
    }
    if let Some((_key, path)) = missing_tp2s.into_iter().next() {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: format!("expected TP2 `{path}` is missing from the extraction marker"),
        });
    }
    if let Some((index, _)) = root_has_file
        .iter()
        .enumerate()
        .find(|(_index, has_file)| !**has_file)
    {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: format!(
                "publish root `{}` has no recorded regular file",
                expected_roots[index]
            ),
        });
    }
    for entry in WalkDir::new(&root).min_depth(1).follow_links(false) {
        let entry = entry.map_err(|error| listing_error(&root, digest, error))?;
        if entry.file_type().is_symlink()
            || (!entry.file_type().is_file() && !entry.file_type().is_dir())
        {
            return Err(AcquireError::CorruptCache {
                digest: digest.to_owned(),
                message: format!(
                    "non-regular extracted path `{}`",
                    display_relative(&root, entry.path())
                ),
            });
        }
        if entry.file_type().is_dir() {
            continue;
        }
        let relative =
            entry
                .path()
                .strip_prefix(&root)
                .map_err(|_| AcquireError::CorruptCache {
                    digest: digest.to_owned(),
                    message: "extracted path escaped its cache root".to_owned(),
                })?;
        let relative = portable_path(relative).map_err(|message| AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message,
        })?;
        if relative == EXTRACTION_MARKER {
            continue;
        }
        let Some((expected_path, record)) = expected_files.remove(&casefold(&relative)) else {
            return Err(AcquireError::CorruptCache {
                digest: digest.to_owned(),
                message: format!("unrecorded extracted file `{relative}`"),
            });
        };
        if relative != expected_path {
            return Err(AcquireError::CorruptCache {
                digest: digest.to_owned(),
                message: format!(
                    "extracted path case changed from `{expected_path}` to `{relative}`"
                ),
            });
        }
        let (length, actual) = hash_regular_file(entry.path())?;
        if length != record.length || actual != record.sha256.to_ascii_lowercase() {
            return Err(AcquireError::CorruptCache {
                digest: digest.to_owned(),
                message: format!("extracted file `{relative}` no longer matches"),
            });
        }
    }
    if let Some((_key, (relative, _record))) = expected_files.into_iter().next() {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: format!("recorded extracted file `{relative}` is missing"),
        });
    }
    Ok(ExtractedArtifact {
        root,
        artifact_sha256: digest.to_owned(),
        extraction_contract_sha256: contract_sha256.to_owned(),
        wrapper_directory: marker.wrapper_directory,
        expected_roots: expected_roots.to_vec(),
        expected_tp2_paths: expected_tp2_paths.to_vec(),
    })
}

pub(super) fn verify_extracted_artifact(artifact: &ExtractedArtifact) -> Result<(), AcquireError> {
    let verified = validate_published_extraction(
        artifact.root.clone(),
        &ExtractionIdentity {
            digest: &artifact.artifact_sha256,
            contract_sha256: &artifact.extraction_contract_sha256,
            expected_roots: &artifact.expected_roots,
            expected_tp2_paths: &artifact.expected_tp2_paths,
        },
    )?;
    if verified.wrapper_directory != artifact.wrapper_directory {
        return Err(AcquireError::CorruptCache {
            digest: artifact.artifact_sha256.clone(),
            message: "extracted artifact wrapper identity changed".to_owned(),
        });
    }
    Ok(())
}

/// A failure to list a published extraction keeps its I/O kind, so that only a path of the
/// extraction that no longer exists can prove damage. Without link following, walkdir has no
/// other kind of failure.
fn listing_error(root: &Path, digest: &str, error: walkdir::Error) -> AcquireError {
    let path = error
        .path()
        .map_or_else(|| root.to_path_buf(), Path::to_path_buf);
    match error.into_io_error() {
        Some(source) => AcquireError::Io {
            action: "list published extraction",
            path,
            source,
        },
        None => AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: "published extraction contains a link loop".to_owned(),
        },
    }
}

/// `path` relative to the extraction root, for messages that must not reveal the cache location.
fn display_relative(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(relative) => portable_path(relative)
            .unwrap_or_else(|_| relative.to_string_lossy().replace('\\', "/")),
        Err(_) => "a path outside the extraction".to_owned(),
    }
}

fn portable_path(path: &Path) -> Result<String, String> {
    let mut components = Vec::new();
    for component in path.components() {
        let component = component
            .as_os_str()
            .to_str()
            .ok_or_else(|| "extracted path is not valid Unicode".to_owned())?;
        components.push(component);
    }
    normalize_relative_path(&components.join("/"), false).map_err(|error| error.to_string())
}

fn hash_regular_file(path: &Path) -> Result<(u64, String), AcquireError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| AcquireError::Io {
        action: "inspect extracted file",
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(AcquireError::InvalidMaterialization(format!(
            "`{}` is not a regular file",
            path.display()
        )));
    }
    let mut file = File::open(path).map_err(|source| AcquireError::Io {
        action: "open file for hashing",
        path: path.to_path_buf(),
        source,
    })?;
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).map_err(|source| AcquireError::Io {
            action: "read file for hashing",
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        length = length.checked_add(read as u64).ok_or_else(|| {
            AcquireError::ArchiveLimitExceeded("file length overflowed u64".to_owned())
        })?;
        hash.update(&buffer[..read]);
    }
    Ok((length, hex::encode(hash.finalize())))
}

fn create_temporary_directory(parent: &Path, digest: &str) -> Result<PathBuf, AcquireError> {
    for _ in 0..32 {
        let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".{digest}.{}.{}.tmp", std::process::id(), sequence));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(AcquireError::Io {
                    action: "create unique extraction temporary",
                    path,
                    source,
                });
            }
        }
    }
    Err(AcquireError::InvalidRequest(
        "could not allocate a unique extraction temporary".to_owned(),
    ))
}

fn is_unsupported_unix_kind(mode: Option<u32>, is_directory: bool) -> bool {
    let Some(mode) = mode else {
        return false;
    };
    let kind = mode & 0o170000;
    if is_directory {
        !matches!(kind, 0 | 0o040000)
    } else {
        !matches!(kind, 0 | 0o100000)
    }
}

pub(super) fn normalize_relative_path(
    value: &str,
    allow_directory_suffix: bool,
) -> Result<String, AcquireError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.chars().any(char::is_control)
        || value.contains(':')
    {
        return Err(AcquireError::UnsafeArchiveEntry {
            entry: value.to_owned(),
            message: "path must be a non-empty portable relative path".to_owned(),
        });
    }
    let replaced = value.replace('\\', "/");
    if replaced.starts_with('/') || replaced.starts_with("//") {
        return Err(AcquireError::UnsafeArchiveEntry {
            entry: value.to_owned(),
            message: "absolute and UNC paths are forbidden".to_owned(),
        });
    }
    let trimmed = if allow_directory_suffix {
        replaced.strip_suffix('/').unwrap_or(&replaced)
    } else {
        replaced.as_str()
    };
    if trimmed.is_empty() {
        return Err(AcquireError::UnsafeArchiveEntry {
            entry: value.to_owned(),
            message: "empty paths are forbidden".to_owned(),
        });
    }
    let mut components = Vec::new();
    for component in trimmed.split('/') {
        if component.is_empty() || matches!(component, "." | "..") {
            return Err(AcquireError::UnsafeArchiveEntry {
                entry: value.to_owned(),
                message: "empty, current, and parent components are forbidden".to_owned(),
            });
        }
        if component.ends_with('.') || component.ends_with(' ') {
            return Err(AcquireError::UnsafeArchiveEntry {
                entry: value.to_owned(),
                message: "components may not end in a dot or space".to_owned(),
            });
        }
        let stem = component.split('.').next().unwrap_or(component);
        let stem = stem.to_ascii_lowercase();
        if matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
            || stem.strip_prefix("com").is_some_and(|number| {
                matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
            || stem.strip_prefix("lpt").is_some_and(|number| {
                matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
        {
            return Err(AcquireError::UnsafeArchiveEntry {
                entry: value.to_owned(),
                message: "Windows device names are forbidden".to_owned(),
            });
        }
        components.push(component);
    }
    Ok(components.join("/"))
}

pub(super) fn join_relative(root: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

pub(super) fn casefold(value: &str) -> String {
    value.to_lowercase()
}

#[cfg(test)]
mod layout_scale_tests {
    use super::*;

    /// Entries in archive order; a path ending in `/` is a directory entry.
    fn entries(paths: &[String]) -> (Vec<ValidatedEntry>, BTreeMap<String, bool>) {
        let mut entries = Vec::with_capacity(paths.len());
        let mut kinds = BTreeMap::new();
        for (index, path) in paths.iter().enumerate() {
            let (path, is_directory) = match path.strip_suffix('/') {
                Some(directory) => (directory, true),
                None => (path.as_str(), false),
            };
            assert!(
                kinds.insert(casefold(path), is_directory).is_none(),
                "duplicate test entry {path}"
            );
            entries.push(ValidatedEntry {
                index,
                archive_path: path.to_owned(),
                relative_path: path.to_owned(),
                size: 1,
                is_directory,
            });
        }
        (entries, kinds)
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn inventory(
        paths: &[String],
        roots: &[&str],
        tp2s: &[&str],
        rule: LayoutRule,
    ) -> Result<(Vec<ValidatedEntry>, Option<String>), AcquireError> {
        let (entries, kinds) = entries(paths);
        finalize_entry_inventory(entries, kinds, &strings(roots), &strings(tp2s), rule)
    }

    /// A mod payload of `count` files below `prefix` (empty for the archive root), spread over
    /// a few hundred folders, with its installer.
    fn payload(prefix: &str, count: usize) -> Vec<String> {
        let mut paths = vec![format!("{prefix}mymod/setup-mymod.tp2")];
        paths.extend((1..count).map(|n| format!("{prefix}mymod/data{}/file{n}.bin", n % 300)));
        paths
    }

    fn layout_message(
        result: Result<(Vec<ValidatedEntry>, Option<String>), AcquireError>,
    ) -> String {
        match result {
            Err(AcquireError::ArchiveLayout(message)) => message,
            other => panic!("expected a layout error, got {other:?}"),
        }
    }

    #[test]
    fn reviewed_wrapper_is_found_among_150000_entries() {
        let mut paths = vec!["Mod-V2/".to_owned()];
        paths.extend(payload("Mod-V2/", 150_000));
        // A differently spelled folder name later on keeps the first spelling.
        paths.push("MOD-v2/mymod/readme.txt".to_owned());

        let (entries, wrapper) = inventory(
            &paths,
            &["mymod"],
            &["mymod/setup-mymod.tp2"],
            LayoutRule::Reviewed,
        )
        .unwrap();

        assert_eq!(wrapper.as_deref(), Some("Mod-V2"));
        assert_eq!(entries.len(), 150_001);
        assert!(entries
            .iter()
            .any(|entry| entry.relative_path == "mymod/setup-mymod.tp2"));
    }

    #[test]
    fn reviewed_root_payload_beside_many_top_level_folders() {
        let mut paths = payload("", 100_000);
        paths.extend((0..50_000).map(|n| format!("extra{n}/notes.txt")));

        let (entries, wrapper) = inventory(
            &paths,
            &["mymod"],
            &["mymod/setup-mymod.tp2"],
            LayoutRule::Reviewed,
        )
        .unwrap();

        assert_eq!(wrapper, None);
        assert_eq!(entries.len(), 100_000);
    }

    #[test]
    fn reviewed_payload_both_at_the_root_and_below_a_wrapper_stays_refused() {
        // Everything lies below `mymod/`, and `mymod/setup-mymod.tp2` exists both with and
        // without that folder stripped.
        let mut paths = payload("mymod/", 150_000);
        paths.push("mymod/setup-mymod.tp2".to_owned());

        let message = layout_message(inventory(
            &paths,
            &["mymod"],
            &["mymod/setup-mymod.tp2"],
            LayoutRule::Reviewed,
        ));

        assert_eq!(message, "more than one possible TP2 payload root was found");
    }

    #[test]
    fn reviewed_payload_beside_many_top_level_folders_without_a_root_payload_is_refused() {
        let mut paths: Vec<String> = (0..50_000).map(|n| format!("extra{n}/notes.txt")).collect();
        paths.extend(payload("Wrapped/", 100_000));

        let message = layout_message(inventory(
            &paths,
            &["mymod"],
            &["mymod/setup-mymod.tp2"],
            LayoutRule::Reviewed,
        ));

        assert_eq!(
            message,
            "expected TP2 paths were not found under an exact payload root"
        );
    }

    /// 25,000 places that end in the first reviewed TP2 path, each a candidate prefix (50,000
    /// files), and 50,000 payload files in each place that also holds the second installer.
    fn served_archive(complete_places: &[&str]) -> Vec<String> {
        let mut paths = Vec::new();
        for n in 0..25_000 {
            paths.push(format!("copies/{n}/mymod/setup-mymod.tp2"));
            paths.push(format!("copies/{n}/mymod/readme.txt"));
        }
        for place in complete_places {
            paths.extend(payload(&format!("{place}/"), 50_000));
            paths.push(format!("{place}/other/setup-other.tp2"));
        }
        paths
    }

    const SERVED_ROOTS: &[&str] = &["mymod", "other"];
    const SERVED_TP2S: &[&str] = &["mymod/setup-mymod.tp2", "other/setup-other.tp2"];

    #[test]
    fn served_prefix_is_found_among_many_candidates() {
        let paths = served_archive(&["Release/Final Build"]);
        assert!(paths.len() > 100_000);

        let (entries, prefix) =
            inventory(&paths, SERVED_ROOTS, SERVED_TP2S, LayoutRule::Served).unwrap();

        assert_eq!(prefix.as_deref(), Some("Release/Final Build"));
        assert_eq!(entries.len(), 50_001);
        assert!(entries
            .iter()
            .any(|entry| entry.relative_path == "other/setup-other.tp2"));
    }

    #[test]
    fn served_payload_in_two_places_among_many_candidates_stays_refused() {
        let paths = served_archive(&["Release/Final Build", "Release/Old Build"]);

        let message = layout_message(inventory(
            &paths,
            SERVED_ROOTS,
            SERVED_TP2S,
            LayoutRule::Served,
        ));

        assert_eq!(
            message,
            "the reviewed payload was found in more than one place"
        );
    }

    #[test]
    fn prefix_collision_is_found_among_150000_entries() {
        let mut paths = payload("", 150_000);
        // Siblings that sort between `mymod/data7` and `mymod/data7/` must not hide the clash.
        paths.push("mymod/data7-old/file.bin".to_owned());
        paths.push("mymod/data7".to_owned());

        let result = inventory(
            &paths,
            &["mymod"],
            &["mymod/setup-mymod.tp2"],
            LayoutRule::Reviewed,
        );

        match result {
            Err(AcquireError::UnsafeArchiveEntry { entry, message }) => {
                assert_eq!(entry, "mymod/data7");
                assert_eq!(message, "file/directory prefix collision");
            }
            other => panic!("expected a prefix collision, got {other:?}"),
        }
    }

    #[test]
    fn root_lookups_ignore_folders_that_only_share_the_name_start() {
        let paths = strings(&["mymod-old/a.txt", "mymod0/b.txt", "setup-mymod.tp2"]);

        let message = layout_message(inventory(
            &paths,
            &["mymod"],
            &["setup-mymod.tp2"],
            LayoutRule::Reviewed,
        ));

        assert_eq!(
            message,
            "expected TP2 paths were not found under an exact payload root"
        );
    }

    /// The scanning implementation these lookups replaced, kept as the reference.
    mod reference {
        use super::super::*;

        pub(super) fn prefix_collision(kinds: &BTreeMap<String, bool>) -> Option<String> {
            let keys: Vec<_> = kinds.keys().cloned().collect();
            keys.iter()
                .find(|key| {
                    kinds.get(*key) == Some(&false) && {
                        let prefix = format!("{key}/");
                        keys.iter().any(|other| other.starts_with(&prefix))
                    }
                })
                .cloned()
        }

        fn files(entries: &[ValidatedEntry]) -> Vec<&str> {
            entries
                .iter()
                .filter(|entry| !entry.is_directory)
                .map(|entry| entry.archive_path.as_str())
                .collect()
        }

        pub(super) fn select_wrapper(
            entries: &[ValidatedEntry],
            roots: &[String],
            tp2s: &[String],
        ) -> Result<Option<String>, AcquireError> {
            let files = files(entries);
            let mut candidates = Vec::<Option<String>>::new();
            if layout_matches(&files, None, roots, tp2s, true) {
                candidates.push(None);
            }
            let mut wrappers = BTreeMap::<String, String>::new();
            for path in &files {
                if let Some((first, _)) = path.split_once('/') {
                    wrappers
                        .entry(casefold(first))
                        .or_insert_with(|| first.to_owned());
                }
            }
            for wrapper in wrappers.into_values() {
                if layout_matches(&files, Some(wrapper.as_str()), roots, tp2s, true) {
                    candidates.push(Some(wrapper));
                }
            }
            match candidates.len() {
                1 => Ok(candidates.pop().unwrap()),
                0 => Err(AcquireError::ArchiveLayout(
                    "expected TP2 paths were not found under an exact payload root".to_owned(),
                )),
                _ => Err(AcquireError::ArchiveLayout(
                    "more than one possible TP2 payload root was found".to_owned(),
                )),
            }
        }

        /// `layout_matches` with `whole`, `layout_matches_below` without.
        fn layout_matches(
            files: &[&str],
            wrapper: Option<&str>,
            roots: &[String],
            tp2s: &[String],
            whole: bool,
        ) -> bool {
            let prefix = wrapper.map(|value| format!("{}/", casefold(value)));
            let stripped: Vec<String> = files
                .iter()
                .filter_map(|path| {
                    let key = casefold(path);
                    match prefix.as_deref() {
                        Some(prefix) => key.strip_prefix(prefix).map(str::to_owned),
                        None => Some(key),
                    }
                })
                .collect();
            if whole && wrapper.is_some() && stripped.len() != files.len() {
                return false;
            }
            tp2s.iter()
                .all(|expected| stripped.iter().any(|path| path == &casefold(expected)))
                && roots.iter().all(|root| {
                    let root = casefold(root);
                    let prefix = format!("{root}/");
                    stripped
                        .iter()
                        .any(|path| path == &root || path.starts_with(&prefix))
                })
        }

        pub(super) fn select_served_prefix(
            entries: &[ValidatedEntry],
            roots: &[String],
            tp2s: &[String],
        ) -> Result<Option<String>, AcquireError> {
            if let Ok(prefix) = select_wrapper(entries, roots, tp2s) {
                return Ok(prefix);
            }
            let files = files(entries);
            let (anchor, anchor_ends_path) = match tp2s.first() {
                Some(tp2) => (tp2, true),
                None => (
                    roots.first().ok_or_else(|| {
                        AcquireError::ArchiveLayout(
                            "no reviewed payload path to look for".to_owned(),
                        )
                    })?,
                    false,
                ),
            };
            let anchor: Vec<String> = anchor.split('/').map(casefold).collect();
            let mut candidates = BTreeMap::<String, Option<String>>::new();
            for path in &files {
                let components: Vec<&str> = path.split('/').collect();
                let keys: Vec<String> = components.iter().map(|part| casefold(part)).collect();
                if keys.len() < anchor.len() {
                    continue;
                }
                let last_start = keys.len() - anchor.len();
                let starts = if anchor_ends_path {
                    last_start..=last_start
                } else {
                    0..=last_start
                };
                for start in starts {
                    if keys[start..start + anchor.len()] == anchor[..] {
                        let prefix = (start > 0).then(|| components[..start].join("/"));
                        candidates
                            .entry(prefix.as_deref().map(casefold).unwrap_or_default())
                            .or_insert(prefix);
                    }
                }
            }
            let mut matching: Vec<Option<String>> = candidates
                .into_values()
                .filter(|prefix| layout_matches(&files, prefix.as_deref(), roots, tp2s, false))
                .collect();
            match matching.len() {
                1 => Ok(matching.pop().unwrap()),
                0 => Err(AcquireError::ArchiveLayout(format!(
                    "the reviewed `{}` was not found together with every reviewed payload path",
                    anchor.join("/")
                ))),
                _ => Err(AcquireError::ArchiveLayout(
                    "the reviewed payload was found in more than one place".to_owned(),
                )),
            }
        }
    }

    /// Deterministic pseudo-random numbers, enough to vary small synthetic archives.
    struct Lcg(u64);

    impl Lcg {
        fn below(&mut self, bound: usize) -> usize {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) % bound as u64) as usize
        }
    }

    #[test]
    fn lookups_decide_exactly_like_the_scans_they_replace() {
        // Folder names that differ only in case, sort right beside one another (`-`, `.` and
        // `0` around `/`), or change under whole-string lowercasing (final sigma).
        const PARTS: &[&str] = &[
            "a", "A", "a-b", "a.b", "a0", "mymod", "MyMod", "other", "ΑΣ", "ας", "wrap", "Wrap",
        ];
        const FILES: &[&str] = &[
            "setup-mymod.tp2",
            "setup-other.tp2",
            "x.txt",
            "Σ",
            "a",
            "mymod",
        ];
        const ROOT_SETS: &[&[&str]] = &[&["mymod"], &["mymod", "other"], &["a"], &["ΑΣ"], &[]];
        const TP2_SETS: &[&[&str]] = &[
            &["mymod/setup-mymod.tp2"],
            &["mymod/setup-mymod.tp2", "other/setup-other.tp2"],
            &["setup-mymod.tp2"],
            &["ΑΣ/Σ"],
            &[],
        ];
        let mut random = Lcg(0x5eed);
        let mut outcomes = BTreeMap::<&str, usize>::new();
        for _ in 0..10_000 {
            let mut paths = Vec::new();
            let mut seen = BTreeSet::new();
            for _ in 0..1 + random.below(12) {
                let depth = random.below(4);
                let mut path: Vec<&str> = (0..depth)
                    .map(|_| PARTS[random.below(PARTS.len())])
                    .collect();
                let is_directory = depth > 0 && random.below(6) == 0;
                if !is_directory {
                    path.push(FILES[random.below(FILES.len())]);
                }
                let path = path.join("/");
                if seen.insert(casefold(&path)) {
                    paths.push(if is_directory {
                        format!("{path}/")
                    } else {
                        path
                    });
                }
            }
            let roots = strings(ROOT_SETS[random.below(ROOT_SETS.len())]);
            let tp2s = strings(TP2_SETS[random.below(TP2_SETS.len())]);
            let (entries, kinds) = entries(&paths);
            let context = format!("{paths:?} roots {roots:?} tp2s {tp2s:?}");

            let expected_collision = reference::prefix_collision(&kinds);
            for rule in [LayoutRule::Reviewed, LayoutRule::Served] {
                let collision = match finalize_entry_inventory(
                    entries.clone(),
                    kinds.clone(),
                    &roots,
                    &tp2s,
                    rule,
                ) {
                    Err(AcquireError::UnsafeArchiveEntry { entry, message }) => {
                        assert_eq!(message, "file/directory prefix collision", "{context}");
                        Some(entry)
                    }
                    _ => None,
                };
                assert_eq!(collision, expected_collision, "{context}");
            }

            let files = FileIndex::new(&entries);
            let wrapper = format!("{:?}", select_wrapper(&files, &roots, &tp2s));
            assert_eq!(
                wrapper,
                format!("{:?}", reference::select_wrapper(&entries, &roots, &tp2s)),
                "{context}"
            );
            let served = format!("{:?}", select_served_prefix(&files, &roots, &tp2s));
            assert_eq!(
                served,
                format!(
                    "{:?}",
                    reference::select_served_prefix(&entries, &roots, &tp2s)
                ),
                "{context}"
            );
            if expected_collision.is_some() {
                *outcomes.entry("collision").or_default() += 1;
            }
            let wrapper_outcome = if wrapper == "Ok(None)" {
                "root payload"
            } else if wrapper.starts_with("Ok(Some") {
                "wrapper"
            } else if wrapper.contains("more than one") {
                "ambiguous wrapper"
            } else {
                "no wrapper"
            };
            *outcomes.entry(wrapper_outcome).or_default() += 1;
            if wrapper.starts_with("Err") {
                let served_outcome = if served.starts_with("Ok(Some") {
                    "served prefix"
                } else if served.contains("more than one place") {
                    "ambiguous served prefix"
                } else {
                    "no served prefix"
                };
                *outcomes.entry(served_outcome).or_default() += 1;
            }
        }
        // Every decision occurs, so the comparison covers each branch.
        assert_eq!(outcomes.len(), 8, "{outcomes:?}");
        assert!(outcomes.values().all(|count| *count >= 10), "{outcomes:?}");
    }
}

#[cfg(all(test, windows))]
mod rar_safety_tests {
    use super::*;

    #[test]
    fn callback_rejects_overproduction_before_writing_the_chunk() {
        let temporary = tempfile::NamedTempFile::new().unwrap();
        let output = temporary.reopen().unwrap();
        let mut state = RarCallbackState {
            output: Some(output),
            hash: Sha256::new(),
            length: 0,
            expected_length: 3,
            error: None,
        };
        let bytes = [1_u8, 2, 3, 4];
        let result = rar_callback(
            unrar_sys::UCM_PROCESSDATA,
            std::ptr::from_mut(&mut state) as unrar_sys::LPARAM,
            bytes.as_ptr() as unrar_sys::LPARAM,
            bytes.len() as unrar_sys::LPARAM,
        );

        assert_eq!(result, -1);
        assert_eq!(state.length, 0);
        assert_eq!(fs::read(temporary.path()).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn engine_owned_snapshot_is_stable_after_source_changes() {
        let temporary = tempfile::TempDir::new().unwrap();
        let source = temporary.path().join("source.exe");
        fs::write(&source, b"MZoriginal").unwrap();
        let (snapshot, _, digest, _) = snapshot_archive(&source, temporary.path()).unwrap();

        fs::write(&source, b"MZreplacement").unwrap();

        assert_eq!(fs::read(snapshot.path()).unwrap(), b"MZoriginal");
        assert_eq!(digest, hex::encode(Sha256::digest(b"MZoriginal")));
    }
}

#[cfg(test)]
mod served_container_tests {
    use std::io::Write;

    use super::*;

    fn problem(bytes: &[u8], format: ArchiveFormat) -> Option<String> {
        let temporary = tempfile::NamedTempFile::new().unwrap();
        fs::write(temporary.path(), bytes).unwrap();
        served_container_problem(temporary.path(), format).unwrap()
    }

    fn zip(text: &[u8]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file("Mod/readme.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(text).unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn a_complete_zip_or_iemod_is_framed_like_one() {
        let archive = zip(b"a release");
        assert_eq!(problem(&archive, ArchiveFormat::Zip), None);
        assert_eq!(problem(&archive, ArchiveFormat::Iemod), None);
    }

    #[test]
    fn a_page_empty_file_or_program_is_no_zip() {
        assert_eq!(
            problem(b"<!DOCTYPE html>\r\n<html>\t<body>", ArchiveFormat::Zip).as_deref(),
            Some("it does not begin like a ZIP archive but with the text \"<!DOCTYPE html>  <html> <body>\"")
        );
        assert_eq!(
            problem(b"", ArchiveFormat::Iemod).as_deref(),
            Some("it does not begin like a ZIP archive but with no bytes at all")
        );
        assert_eq!(
            problem(b"MZ\x90\x00\x03\x00\x00\x00\x04\x00", ArchiveFormat::Zip).as_deref(),
            Some("it does not begin like a ZIP archive but with the bytes 4d 5a 90 00 03 00 00 00")
        );
    }

    #[test]
    fn a_zip_that_ends_early_is_not_complete() {
        let archive = zip(b"a release whose transfer ended early");
        let found = problem(&archive[..archive.len() - 30], ArchiveFormat::Zip).unwrap();
        assert!(
            found.starts_with("it is not a complete ZIP archive ("),
            "{found}"
        );
    }

    #[test]
    fn a_self_extractor_must_begin_like_a_program() {
        assert_eq!(
            problem(b"PK\x03\x04", ArchiveFormat::SelfExtractingRar).as_deref(),
            Some("it does not begin like a self-extracting RAR archive but with the bytes 50 4b 03 04")
        );
    }

    #[cfg(windows)]
    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = !0_u32;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
            }
        }
        !crc
    }

    #[cfg(windows)]
    fn rar4_header(kind: u8, flags: u16, body: &[u8]) -> Vec<u8> {
        let size = u16::try_from(7 + body.len()).unwrap();
        let mut protected = vec![kind];
        protected.extend_from_slice(&flags.to_le_bytes());
        protected.extend_from_slice(&size.to_le_bytes());
        protected.extend_from_slice(body);
        let mut header = (crc32(&protected) as u16).to_le_bytes().to_vec();
        header.extend(protected);
        header
    }

    /// A stored RAR4 self-extractor with one file, as `tests/rar_archive.rs` builds them.
    #[cfg(windows)]
    fn rar4_sfx(name: &str, payload: &[u8]) -> Vec<u8> {
        let mut bytes = vec![b'M', b'Z'];
        bytes.resize(128, 0);
        bytes.extend_from_slice(b"Rar!\x1a\x07\x00");
        bytes.extend(rar4_header(0x73, 0, &[0; 6]));
        let mut body = Vec::new();
        body.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        body.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        body.push(2);
        body.extend_from_slice(&crc32(payload).to_le_bytes());
        body.extend_from_slice(&0_u32.to_le_bytes());
        body.push(20);
        body.push(0x30);
        body.extend_from_slice(&(name.len() as u16).to_le_bytes());
        body.extend_from_slice(&0x20_u32.to_le_bytes());
        body.extend_from_slice(name.as_bytes());
        bytes.extend(rar4_header(0x74, 0x8000, &body));
        bytes.extend_from_slice(payload);
        bytes.extend(rar4_header(0x7b, 0, &[]));
        bytes
    }

    #[cfg(windows)]
    #[test]
    fn a_self_extractor_is_framed_like_one_only_when_complete() {
        let archive = rar4_sfx("evandra/setup-evandra.tp2", b"BACKUP ~evandra/backup~");
        assert_eq!(problem(&archive, ArchiveFormat::SelfExtractingRar), None);

        // A program without an archive, and an archive whose file ends early.
        let program = problem(
            b"MZ\x90\x00 just a program",
            ArchiveFormat::SelfExtractingRar,
        );
        assert!(
            program
                .as_deref()
                .is_some_and(|found| found.starts_with("UnRAR cannot open it as an archive")),
            "{program:?}"
        );
        let cut = problem(
            &archive[..archive.len() - 14],
            ArchiveFormat::SelfExtractingRar,
        );
        assert_eq!(
            cut.as_deref(),
            Some(
                "its archive is damaged or ends early: `evandra/setup-evandra.tp2` ends after \
                 16 of its 23 bytes (UnRAR code 12)"
            )
        );

        // A file whose bytes changed on the way fails its checksum.
        let mut damaged = archive.clone();
        let payload_start = archive.len() - 7 - 23;
        damaged[payload_start + 3] ^= 0x20;
        assert_eq!(
            problem(&damaged, ArchiveFormat::SelfExtractingRar).as_deref(),
            Some(
                "its archive is damaged or ends early: `evandra/setup-evandra.tp2` fails its \
                 checksum (UnRAR code 12)"
            )
        );
    }

    /// The fixture above stores its file; real packages compress theirs. Run with
    /// `CHRIZ_TEST_RAR_SFX` naming a published self-extracting RAR mod package.
    #[cfg(windows)]
    #[test]
    fn a_real_self_extractor_is_complete_and_cut_copies_are_not_when_explicitly_supplied() {
        let Some(path) = std::env::var_os("CHRIZ_TEST_RAR_SFX") else {
            return;
        };
        let path = PathBuf::from(path);
        assert_eq!(
            served_container_problem(&path, ArchiveFormat::SelfExtractingRar).unwrap(),
            None
        );
        let bytes = fs::read(&path).unwrap();
        let half = problem(&bytes[..bytes.len() / 2], ArchiveFormat::SelfExtractingRar);
        let mut flipped = bytes.clone();
        flipped[bytes.len() * 3 / 4] ^= 0x5a;
        let flipped = problem(&flipped, ArchiveFormat::SelfExtractingRar);
        for found in [&half, &flipped] {
            assert!(
                found
                    .as_deref()
                    .is_some_and(|found| found.starts_with("its archive is damaged or ends early")),
                "{found:?}"
            );
        }
        println!("half: {half:?}; one byte changed: {flipped:?}");
    }
}
