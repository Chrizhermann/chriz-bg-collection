use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipArchive};

use super::AcquireError;

const BUFFER_SIZE: usize = 64 * 1024;
const EXTRACTION_MARKER: &str = ".chriz-bg-extraction.json";
const EXTRACTION_MARKER_VERSION: u32 = 2;
const EXTRACTION_CACHE_NAMESPACE: &str = "sha256-v2";
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Resource limits applied before any archive entry is written.
#[derive(Clone, Debug, PartialEq, Eq)]
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

/// Extraction policy selected by the recipe source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveMode {
    /// A redistributable recipe with a mandatory immutable digest and ZIP-compatible framing.
    Public,
    /// A locally authored recipe. The archive is still subject to all structural limits.
    Authoring,
}

/// ZIP-compatible archive framing declared by the recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// A standard `.zip` artifact.
    Zip,
    /// An `.iemod` artifact, which uses ZIP framing.
    Iemod,
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

/// Validate a complete ZIP central directory, extract only approved payload roots into a
/// private temporary, and publish them under the verified archive digest.
pub fn extract_archive(
    archive_path: &Path,
    extraction_cache: &Path,
    requirements: &ArchiveRequirements,
) -> Result<ExtractedArtifact, AcquireError> {
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
        ArchiveFormat::Zip | ArchiveFormat::Iemod => {}
    }

    let expected_digest = validate_digest(&requirements.artifact_sha256)?;
    if matches!(requirements.mode, ArchiveMode::Public)
        && expected_digest.bytes().all(|byte| byte == b'0')
    {
        return Err(AcquireError::PublicArchiveRejected(
            "an all-zero SHA-256 is not immutable provenance".to_owned(),
        ));
    }
    let (archive_length, actual_digest, prefix) = hash_archive(archive_path)?;
    if expected_digest.bytes().any(|byte| byte != b'0') && actual_digest != expected_digest {
        return Err(AcquireError::HashMismatch {
            expected: expected_digest,
            actual: actual_digest,
        });
    }
    if archive_length < 4 || prefix != *b"PK\x03\x04" {
        let message = if matches!(requirements.mode, ArchiveMode::Public) {
            "self-extracting/prefixed ZIP files are not accepted"
        } else {
            "archive must begin with a ZIP local-file header"
        };
        return Err(AcquireError::PublicArchiveRejected(message.to_owned()));
    }

    let digest = actual_digest;
    let final_root = extraction_cache
        .join(EXTRACTION_CACHE_NAMESPACE)
        .join(&digest[..2])
        .join(&digest);
    if final_root.exists() {
        return validate_published_extraction(
            final_root,
            &digest,
            &expected_roots,
            &expected_tp2_paths,
        );
    }

    let file = File::open(archive_path).map_err(|source| AcquireError::Io {
        action: "open archive",
        path: archive_path.to_path_buf(),
        source,
    })?;
    let mut archive = ZipArchive::new(file).map_err(|source| AcquireError::ArchiveFormat {
        path: archive_path.to_path_buf(),
        message: source.to_string(),
    })?;
    let (entries, wrapper_directory) = validate_central_directory(
        &mut archive,
        archive_path,
        &expected_roots,
        &expected_tp2_paths,
        &requirements.limits,
    )?;

    let temporary_parent = extraction_cache.join("temporary");
    fs::create_dir_all(&temporary_parent).map_err(|source| AcquireError::Io {
        action: "create extraction temporary directory",
        path: temporary_parent.clone(),
        source,
    })?;
    let temporary = create_temporary_directory(&temporary_parent, &digest)?;
    let result = extract_validated_entries(
        archive,
        &entries,
        &temporary,
        &digest,
        wrapper_directory.clone(),
        &expected_roots,
        &expected_tp2_paths,
    );
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
    match fs::rename(&temporary, &final_root) {
        Ok(()) => {}
        Err(_source) if final_root.exists() => {
            let _ = fs::remove_dir_all(&temporary);
            return validate_published_extraction(
                final_root,
                &digest,
                &expected_roots,
                &expected_tp2_paths,
            );
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

    Ok(ExtractedArtifact {
        root: final_root,
        artifact_sha256: digest,
        wrapper_directory,
        expected_roots,
        expected_tp2_paths,
    })
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
        if !seen.insert(key) {
            return Err(AcquireError::InvalidRequest(format!(
                "duplicate case-insensitive expected {label}: {value}"
            )));
        }
        normalized.push(path);
    }
    normalized.sort_by_key(|path| casefold(path));
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

fn hash_archive(path: &Path) -> Result<(u64, String, [u8; 4]), AcquireError> {
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
    let mut file = File::open(path).map_err(|source| AcquireError::Io {
        action: "open archive for hashing",
        path: path.to_path_buf(),
        source,
    })?;
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut prefix = [0_u8; 4];
    let mut prefix_length = 0_usize;
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).map_err(|source| AcquireError::Io {
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
    }
    Ok((length, hex::encode(hash.finalize()), prefix))
}

fn validate_central_directory<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    archive_path: &Path,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
    limits: &ArchiveLimits,
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

    let keys: Vec<_> = destination_kinds.keys().cloned().collect();
    for key in &keys {
        if destination_kinds.get(key) == Some(&false) {
            let prefix = format!("{key}/");
            if keys.iter().any(|other| other.starts_with(&prefix)) {
                return Err(AcquireError::UnsafeArchiveEntry {
                    entry: key.clone(),
                    message: "file/directory prefix collision".to_owned(),
                });
            }
        }
    }

    let wrapper = select_wrapper(&entries, expected_roots, expected_tp2_paths)?;
    let wrapper_key = wrapper.as_ref().map(|value| casefold(value));
    if let Some(wrapper_key) = wrapper_key.as_deref() {
        entries.retain(|entry| casefold(&entry.archive_path) != wrapper_key);
    }
    for entry in &mut entries {
        if let Some(wrapper_key) = wrapper_key.as_deref() {
            let archive_key = casefold(&entry.archive_path);
            let prefix = format!("{wrapper_key}/");
            if !archive_key.starts_with(&prefix) {
                return Err(AcquireError::ArchiveLayout(format!(
                    "entry `{}` is outside wrapper `{}`",
                    entry.archive_path,
                    wrapper.as_deref().unwrap_or_default()
                )));
            }
            let first_separator = entry.archive_path.find('/').ok_or_else(|| {
                AcquireError::ArchiveLayout("wrapper directory contains no payload".to_owned())
            })?;
            entry.relative_path = entry.archive_path[first_separator + 1..].to_owned();
        }
    }

    let expected_tp2_keys: BTreeSet<_> = expected_tp2_paths
        .iter()
        .map(|path| casefold(path))
        .collect();
    for entry in entries.iter().filter(|entry| !entry.is_directory) {
        let key = casefold(&entry.relative_path);
        let inside_publish_root = expected_roots
            .iter()
            .any(|root| path_is_within_root(root, &entry.relative_path));
        if inside_publish_root && key.ends_with(".tp2") && !expected_tp2_keys.contains(&key) {
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

fn select_wrapper(
    entries: &[ValidatedEntry],
    expected_roots: &[String],
    expected_tp2_paths: &[String],
) -> Result<Option<String>, AcquireError> {
    let files: Vec<_> = entries
        .iter()
        .filter(|entry| !entry.is_directory)
        .map(|entry| entry.archive_path.as_str())
        .collect();
    let mut candidates = Vec::<Option<String>>::new();
    if layout_matches(&files, None, expected_roots, expected_tp2_paths) {
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
        if layout_matches(
            &files,
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
    files: &[&str],
    wrapper: Option<&str>,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
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
    if wrapper.is_some() && stripped.len() != files.len() {
        return false;
    }
    expected_tp2_paths
        .iter()
        .all(|expected| stripped.iter().any(|path| path == &casefold(expected)))
        && expected_roots.iter().all(|root| {
            let root = casefold(root);
            let prefix = format!("{root}/");
            stripped
                .iter()
                .any(|path| path == &root || path.starts_with(&prefix))
        })
}

fn path_is_within_root(root: &str, path: &str) -> bool {
    let root = casefold(root);
    let path = casefold(path);
    path == root || path.starts_with(&format!("{root}/"))
}

fn extract_validated_entries<R: Read + std::io::Seek>(
    mut archive: ZipArchive<R>,
    entries: &[ValidatedEntry],
    temporary: &Path,
    digest: &str,
    wrapper_directory: Option<String>,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
) -> Result<(), AcquireError> {
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
    records.sort_by_key(|record| casefold(&record.relative_path));
    let marker = ExtractionMarker {
        version: EXTRACTION_MARKER_VERSION,
        artifact_sha256: digest.to_owned(),
        wrapper_directory,
        expected_roots: expected_roots.to_vec(),
        expected_tp2_paths: expected_tp2_paths.to_vec(),
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
    digest: &str,
    expected_roots: &[String],
    expected_tp2_paths: &[String],
) -> Result<ExtractedArtifact, AcquireError> {
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
    let marker_file = File::open(&marker_path).map_err(|source| AcquireError::Io {
        action: "open extraction marker",
        path: marker_path.clone(),
        source,
    })?;
    let marker: ExtractionMarker =
        serde_json::from_reader(marker_file).map_err(|source| AcquireError::Metadata {
            path: marker_path,
            message: source.to_string(),
        })?;
    if marker.version != EXTRACTION_MARKER_VERSION
        || marker.artifact_sha256 != digest
        || marker.expected_roots != expected_roots
        || marker.expected_tp2_paths != expected_tp2_paths
    {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: "extraction marker does not match the requested archive layout".to_owned(),
        });
    }
    let mut expected_files = BTreeMap::new();
    for record in &marker.files {
        let relative_path =
            normalize_relative_path(&record.relative_path, false).map_err(|error| {
                AcquireError::CorruptCache {
                    digest: digest.to_owned(),
                    message: format!("invalid marker path `{}`: {error}", record.relative_path),
                }
            })?;
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
    for entry in WalkDir::new(&root).min_depth(1).follow_links(false) {
        let entry = entry.map_err(|error| AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: error.to_string(),
        })?;
        if entry.file_type().is_symlink()
            || (!entry.file_type().is_file() && !entry.file_type().is_dir())
        {
            return Err(AcquireError::CorruptCache {
                digest: digest.to_owned(),
                message: format!("non-regular extracted path `{}`", entry.path().display()),
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
        wrapper_directory: marker.wrapper_directory,
        expected_roots: expected_roots.to_vec(),
        expected_tp2_paths: expected_tp2_paths.to_vec(),
    })
}

pub(super) fn verify_extracted_artifact(artifact: &ExtractedArtifact) -> Result<(), AcquireError> {
    let verified = validate_published_extraction(
        artifact.root.clone(),
        &artifact.artifact_sha256,
        &artifact.expected_roots,
        &artifact.expected_tp2_paths,
    )?;
    if verified.wrapper_directory != artifact.wrapper_directory {
        return Err(AcquireError::CorruptCache {
            digest: artifact.artifact_sha256.clone(),
            message: "extracted artifact wrapper identity changed".to_owned(),
        });
    }
    Ok(())
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
