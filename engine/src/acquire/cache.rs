use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::events::{EngineEvent, EventSink};

use super::http::{validate_download_url, HttpClient, HttpResponse};
use super::{AcquireError, AcquiredArtifact, ArtifactMetadata, CacheDisposition, DownloadRequest};

const BUFFER_SIZE: usize = 64 * 1024;
const MAX_PROGRESS_INTEGER: u64 = (1_u64 << 53) - 1;

/// A filesystem-backed cache of immutable, SHA-256-addressed archives.
#[derive(Clone)]
pub struct ArtifactCache {
    root: PathBuf,
    client: HttpClient,
}

#[derive(Debug, Deserialize, Serialize)]
struct PartialState {
    original_url: String,
    expected_length: u64,
    etag: String,
}

struct ResumeCandidate {
    start: u64,
    etag: String,
}

struct TransferResult {
    final_url: String,
    etag: Option<String>,
    last_modified: Option<String>,
}

enum AttemptFailure {
    Retryable(AcquireError),
    Fatal(AcquireError),
}

struct CachePaths {
    archive: PathBuf,
    metadata: PathBuf,
    partial: PathBuf,
    partial_state: PathBuf,
    lock: PathBuf,
}

struct DigestLock(File);

impl Drop for DigestLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

impl ArtifactCache {
    /// Open or create a cache rooted at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, AcquireError> {
        let root = root.into();
        for directory in [
            root.join("sha256"),
            root.join("partial"),
            root.join("locks"),
        ] {
            fs::create_dir_all(&directory).map_err(|source| AcquireError::Io {
                action: "create cache directory",
                path: directory,
                source,
            })?;
        }
        Ok(Self {
            root,
            client: HttpClient::new(),
        })
    }

    /// Acquire one pinned artifact, returning only a rehashed cache hit or verified download.
    pub fn acquire(
        &self,
        request: &DownloadRequest,
        sink: &dyn EventSink,
    ) -> Result<AcquiredArtifact, AcquireError> {
        let digest = validate_request(request)?;
        validate_download_url(&request.url)?;
        let paths = self.paths(request, &digest);

        {
            let _guard = acquire_digest_lock(&paths.lock)?;
            if let Some(hit) = verified_cache_entry(&paths, &digest, request.expected_length)? {
                sink.emit(EngineEvent::StepProgress {
                    id: request.request_id.clone(),
                    done: request.expected_length,
                    total: request.expected_length,
                });
                return Ok(hit);
            }
        }

        let transfer = self.transfer(request, &paths, sink)?;
        let (actual_length, actual_digest) = hash_file(&paths.partial)?;
        if actual_length != request.expected_length {
            remove_if_exists(&paths.partial)?;
            remove_if_exists(&paths.partial_state)?;
            return Err(AcquireError::LengthMismatch {
                expected: request.expected_length,
                actual: actual_length,
            });
        }
        if actual_digest != digest {
            remove_if_exists(&paths.partial)?;
            remove_if_exists(&paths.partial_state)?;
            return Err(AcquireError::HashMismatch {
                expected: digest,
                actual: actual_digest,
            });
        }
        sync_file(&paths.partial)?;

        let metadata = ArtifactMetadata {
            original_url: request.url.clone(),
            final_url: transfer.final_url,
            etag: transfer.etag,
            last_modified: transfer.last_modified,
            length: request.expected_length,
            sha256: digest.clone(),
            completed_at_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| AcquireError::InvalidClock)?
                .as_secs(),
        };

        let _guard = acquire_digest_lock(&paths.lock)?;
        if let Some(hit) = verified_cache_entry(&paths, &digest, request.expected_length)? {
            remove_if_exists(&paths.partial)?;
            remove_if_exists(&paths.partial_state)?;
            return Ok(hit);
        }
        publish(&paths, request, &metadata)?;
        Ok(AcquiredArtifact {
            archive_path: paths.archive,
            metadata_path: paths.metadata,
            metadata,
            disposition: CacheDisposition::Downloaded,
        })
    }

    fn paths(&self, request: &DownloadRequest, digest: &str) -> CachePaths {
        let digest_directory = self.root.join("sha256").join(&digest[..2]);
        let partial = self
            .root
            .join("partial")
            .join(format!("{}.part", request.request_id));
        CachePaths {
            archive: digest_directory.join(format!("{digest}.archive")),
            metadata: digest_directory.join(format!("{digest}.json")),
            partial_state: partial.with_extension("part.json"),
            partial,
            lock: self.root.join("locks").join(format!("{digest}.lock")),
        }
    }

    fn transfer(
        &self,
        request: &DownloadRequest,
        paths: &CachePaths,
        sink: &dyn EventSink,
    ) -> Result<TransferResult, AcquireError> {
        let mut last_error = String::new();
        for _attempt in 1..=request.max_attempts {
            match self.transfer_once(request, paths, sink) {
                Ok(result) => return Ok(result),
                Err(AttemptFailure::Fatal(error)) => return Err(error),
                Err(AttemptFailure::Retryable(error)) => last_error = error.to_string(),
            }
        }
        Err(AcquireError::RetryExhausted {
            attempts: request.max_attempts,
            last_error,
        })
    }

    fn transfer_once(
        &self,
        request: &DownloadRequest,
        paths: &CachePaths,
        sink: &dyn EventSink,
    ) -> Result<TransferResult, AttemptFailure> {
        let resume = load_resume_candidate(request, paths).map_err(AttemptFailure::Fatal)?;
        let range = resume
            .as_ref()
            .map(|candidate| (candidate.start, candidate.etag.as_str()));
        let response = self
            .client
            .get_with_reviewed_redirects(&request.url, range, &request.redirect_hosts)
            .map_err(classify_transport_error)?;

        if let Some(candidate) = resume {
            match response.response.status().as_u16() {
                206 if exact_resume_response(&response, request, &candidate) => {
                    return stream_response(response, request, paths, sink, candidate.start, false);
                }
                200 => {
                    return stream_response(response, request, paths, sink, 0, true);
                }
                status if is_transient_status(status) => {
                    return Err(AttemptFailure::Retryable(AcquireError::HttpStatus {
                        url: response.final_url,
                        status,
                    }));
                }
                _ => {
                    remove_if_exists(&paths.partial_state).map_err(AttemptFailure::Fatal)?;
                    let fresh = self
                        .client
                        .get_with_reviewed_redirects(&request.url, None, &request.redirect_hosts)
                        .map_err(classify_transport_error)?;
                    return handle_fresh_response(fresh, request, paths, sink);
                }
            }
        }
        handle_fresh_response(response, request, paths, sink)
    }
}

fn validate_request(request: &DownloadRequest) -> Result<String, AcquireError> {
    if request.request_id.is_empty()
        || request.request_id.len() > 128
        || !request
            .request_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(AcquireError::InvalidRequest(
            "request_id must contain 1-128 ASCII letters, digits, '-' or '_'".to_owned(),
        ));
    }
    if request.max_attempts == 0 {
        return Err(AcquireError::InvalidRequest(
            "max_attempts must be at least one".to_owned(),
        ));
    }
    if request.expected_length > MAX_PROGRESS_INTEGER {
        return Err(AcquireError::InvalidRequest(
            "expected_length exceeds the progress event integer limit".to_owned(),
        ));
    }
    if request.expected_sha256.len() != 64
        || !request
            .expected_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(AcquireError::InvalidRequest(
            "expected_sha256 must be exactly 64 hexadecimal characters".to_owned(),
        ));
    }
    if request
        .redirect_hosts
        .iter()
        .any(|host| host.is_empty() || !host.is_ascii() || host.contains('/') || host.contains(':'))
    {
        return Err(AcquireError::InvalidRequest(
            "redirect_hosts must contain bare ASCII host names".to_owned(),
        ));
    }
    Ok(request.expected_sha256.to_ascii_lowercase())
}

fn handle_fresh_response(
    response: HttpResponse,
    request: &DownloadRequest,
    paths: &CachePaths,
    sink: &dyn EventSink,
) -> Result<TransferResult, AttemptFailure> {
    let status = response.response.status().as_u16();
    if status == 200 {
        return stream_response(response, request, paths, sink, 0, true);
    }
    if is_transient_status(status) {
        return Err(AttemptFailure::Retryable(AcquireError::HttpStatus {
            url: response.final_url,
            status,
        }));
    }
    if status == 206 {
        return Err(AttemptFailure::Fatal(AcquireError::InvalidRange {
            url: response.final_url,
            message: "server returned 206 without a resumable partial".to_owned(),
        }));
    }
    Err(AttemptFailure::Fatal(AcquireError::HttpStatus {
        url: response.final_url,
        status,
    }))
}

fn stream_response(
    response: HttpResponse,
    request: &DownloadRequest,
    paths: &CachePaths,
    sink: &dyn EventSink,
    start: u64,
    truncate: bool,
) -> Result<TransferResult, AttemptFailure> {
    let response_length = header_u64(&response.response, "content-length");
    let expected_response_length = request.expected_length - start;
    if let Some(actual) = response_length {
        if actual != expected_response_length {
            return Err(AttemptFailure::Fatal(AcquireError::LengthMismatch {
                expected: expected_response_length,
                actual,
            }));
        }
    }
    let etag = header_string(&response.response, "etag");
    let last_modified = header_string(&response.response, "last-modified");

    if truncate {
        remove_if_exists(&paths.partial_state).map_err(AttemptFailure::Fatal)?;
    }
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    if truncate {
        options.truncate(true);
    } else {
        options.append(true);
    }
    let mut file = options.open(&paths.partial).map_err(|source| {
        AttemptFailure::Fatal(AcquireError::Io {
            action: "open partial artifact",
            path: paths.partial.clone(),
            source,
        })
    })?;
    if !truncate {
        file.seek(SeekFrom::End(0)).map_err(|source| {
            AttemptFailure::Fatal(AcquireError::Io {
                action: "seek partial artifact",
                path: paths.partial.clone(),
                source,
            })
        })?;
    }

    if let Some(strong) = etag.as_deref().filter(|value| is_strong_etag(value)) {
        write_partial_state(
            &paths.partial_state,
            &PartialState {
                original_url: request.url.clone(),
                expected_length: request.expected_length,
                etag: strong.to_owned(),
            },
        )
        .map_err(AttemptFailure::Fatal)?;
    } else {
        remove_if_exists(&paths.partial_state).map_err(AttemptFailure::Fatal)?;
    }

    let final_url = response.final_url;
    let mut body = response.response.into_body().into_reader();
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut done = start;
    loop {
        let read = match body.read(&mut buffer) {
            Ok(read) => read,
            Err(source) => {
                let _ = file.sync_all();
                return Err(AttemptFailure::Retryable(AcquireError::Http {
                    url: final_url,
                    message: source.to_string(),
                }));
            }
        };
        if read == 0 {
            break;
        }
        done = done.saturating_add(read as u64);
        if done > request.expected_length {
            return Err(AttemptFailure::Fatal(AcquireError::LengthMismatch {
                expected: request.expected_length,
                actual: done,
            }));
        }
        file.write_all(&buffer[..read]).map_err(|source| {
            AttemptFailure::Fatal(AcquireError::Io {
                action: "write partial artifact",
                path: paths.partial.clone(),
                source,
            })
        })?;
        sink.emit(EngineEvent::StepProgress {
            id: request.request_id.clone(),
            done,
            total: request.expected_length,
        });
    }
    if done != request.expected_length {
        let _ = file.sync_all();
        return Err(AttemptFailure::Retryable(AcquireError::LengthMismatch {
            expected: request.expected_length,
            actual: done,
        }));
    }
    file.sync_all().map_err(|source| {
        AttemptFailure::Fatal(AcquireError::Io {
            action: "sync partial artifact",
            path: paths.partial.clone(),
            source,
        })
    })?;
    Ok(TransferResult {
        final_url,
        etag,
        last_modified,
    })
}

fn exact_resume_response(
    response: &HttpResponse,
    request: &DownloadRequest,
    candidate: &ResumeCandidate,
) -> bool {
    let Some(etag) = header_string(&response.response, "etag") else {
        return false;
    };
    if etag != candidate.etag || !is_strong_etag(&etag) || request.expected_length == 0 {
        return false;
    }
    let expected = format!(
        "bytes {}-{}/{}",
        candidate.start,
        request.expected_length - 1,
        request.expected_length
    );
    header_string(&response.response, "content-range").as_deref() == Some(expected.as_str())
}

fn header_string(response: &ureq::http::Response<ureq::Body>, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn header_u64(response: &ureq::http::Response<ureq::Body>, name: &str) -> Option<u64> {
    header_string(response, name).and_then(|value| value.parse().ok())
}

fn is_strong_etag(value: &str) -> bool {
    value.len() >= 2 && value.starts_with('"') && value.ends_with('"') && !value.starts_with("W/")
}

fn is_transient_status(status: u16) -> bool {
    matches!(status, 408 | 425 | 429 | 500..=599)
}

fn classify_transport_error(error: AcquireError) -> AttemptFailure {
    match error {
        AcquireError::Http { .. } => AttemptFailure::Retryable(error),
        _ => AttemptFailure::Fatal(error),
    }
}

fn load_resume_candidate(
    request: &DownloadRequest,
    paths: &CachePaths,
) -> Result<Option<ResumeCandidate>, AcquireError> {
    let length = match fs::metadata(&paths.partial) {
        Ok(metadata) => metadata.len(),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(AcquireError::Io {
                action: "inspect partial artifact",
                path: paths.partial.clone(),
                source,
            });
        }
    };
    if length == 0 || length >= request.expected_length {
        return Ok(None);
    }
    let state_file = match File::open(&paths.partial_state) {
        Ok(file) => file,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(AcquireError::Io {
                action: "open partial metadata",
                path: paths.partial_state.clone(),
                source,
            });
        }
    };
    let state: PartialState = match serde_json::from_reader(state_file) {
        Ok(state) => state,
        Err(_) => return Ok(None),
    };
    if state.original_url != request.url
        || state.expected_length != request.expected_length
        || !is_strong_etag(&state.etag)
    {
        return Ok(None);
    }
    Ok(Some(ResumeCandidate {
        start: length,
        etag: state.etag,
    }))
}

fn write_partial_state(path: &Path, state: &PartialState) -> Result<(), AcquireError> {
    let bytes = serde_json::to_vec(state).map_err(|source| AcquireError::Metadata {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;
    let mut file = File::create(path).map_err(|source| AcquireError::Io {
        action: "create partial metadata",
        path: path.to_path_buf(),
        source,
    })?;
    file.write_all(&bytes).map_err(|source| AcquireError::Io {
        action: "write partial metadata",
        path: path.to_path_buf(),
        source,
    })?;
    file.sync_all().map_err(|source| AcquireError::Io {
        action: "sync partial metadata",
        path: path.to_path_buf(),
        source,
    })
}

fn hash_file(path: &Path) -> Result<(u64, String), AcquireError> {
    let mut file = File::open(path).map_err(|source| AcquireError::Io {
        action: "open artifact for hashing",
        path: path.to_path_buf(),
        source,
    })?;
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).map_err(|source| AcquireError::Io {
            action: "read artifact for hashing",
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        length = length.saturating_add(read as u64);
        hash.update(&buffer[..read]);
    }
    Ok((length, hex::encode(hash.finalize())))
}

fn sync_file(path: &Path) -> Result<(), AcquireError> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| AcquireError::Io {
            action: "sync verified artifact",
            path: path.to_path_buf(),
            source,
        })
}

fn acquire_digest_lock(path: &Path) -> Result<DigestLock, AcquireError> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .map_err(|source| AcquireError::Io {
            action: "open digest lock",
            path: path.to_path_buf(),
            source,
        })?;
    file.lock_exclusive().map_err(|source| AcquireError::Io {
        action: "acquire digest lock",
        path: path.to_path_buf(),
        source,
    })?;
    Ok(DigestLock(file))
}

fn verified_cache_entry(
    paths: &CachePaths,
    digest: &str,
    expected_length: u64,
) -> Result<Option<AcquiredArtifact>, AcquireError> {
    let archive_exists = paths.archive.exists();
    let metadata_exists = paths.metadata.exists();
    if !archive_exists && !metadata_exists {
        return Ok(None);
    }
    if archive_exists != metadata_exists {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: "archive and metadata must both exist".to_owned(),
        });
    }
    let file = File::open(&paths.metadata).map_err(|source| AcquireError::Io {
        action: "open cache metadata",
        path: paths.metadata.clone(),
        source,
    })?;
    let metadata: ArtifactMetadata =
        serde_json::from_reader(file).map_err(|source| AcquireError::Metadata {
            path: paths.metadata.clone(),
            message: source.to_string(),
        })?;
    let (actual_length, actual_digest) = hash_file(&paths.archive)?;
    if metadata.sha256 != digest
        || metadata.length != expected_length
        || actual_digest != digest
        || actual_length != expected_length
    {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: format!(
                "metadata/file mismatch (metadata {} bytes {}, file {} bytes {})",
                metadata.sha256, metadata.length, actual_digest, actual_length
            ),
        });
    }
    Ok(Some(AcquiredArtifact {
        archive_path: paths.archive.clone(),
        metadata_path: paths.metadata.clone(),
        metadata,
        disposition: CacheDisposition::Hit,
    }))
}

fn publish(
    paths: &CachePaths,
    request: &DownloadRequest,
    metadata: &ArtifactMetadata,
) -> Result<(), AcquireError> {
    let parent = paths
        .archive
        .parent()
        .ok_or_else(|| AcquireError::InvalidRequest("cache archive has no parent".to_owned()))?;
    fs::create_dir_all(parent).map_err(|source| AcquireError::Io {
        action: "create digest directory",
        path: parent.to_path_buf(),
        source,
    })?;
    let metadata_temp = parent.join(format!(
        ".{}.{}.json.tmp",
        metadata.sha256, request.request_id
    ));
    let bytes = serde_json::to_vec_pretty(metadata).map_err(|source| AcquireError::Metadata {
        path: metadata_temp.clone(),
        message: source.to_string(),
    })?;
    let mut file = File::create(&metadata_temp).map_err(|source| AcquireError::Io {
        action: "create cache metadata temporary",
        path: metadata_temp.clone(),
        source,
    })?;
    file.write_all(&bytes).map_err(|source| AcquireError::Io {
        action: "write cache metadata temporary",
        path: metadata_temp.clone(),
        source,
    })?;
    file.write_all(b"\n").map_err(|source| AcquireError::Io {
        action: "write cache metadata temporary",
        path: metadata_temp.clone(),
        source,
    })?;
    file.sync_all().map_err(|source| AcquireError::Io {
        action: "sync cache metadata temporary",
        path: metadata_temp.clone(),
        source,
    })?;
    drop(file);

    fs::rename(&paths.partial, &paths.archive).map_err(|source| AcquireError::Io {
        action: "publish verified archive",
        path: paths.archive.clone(),
        source,
    })?;
    fs::rename(&metadata_temp, &paths.metadata).map_err(|source| AcquireError::Io {
        action: "publish cache metadata",
        path: paths.metadata.clone(),
        source,
    })?;
    remove_if_exists(&paths.partial_state)
}

fn remove_if_exists(path: &Path) -> Result<(), AcquireError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(AcquireError::Io {
            action: "remove temporary artifact file",
            path: path.to_path_buf(),
            source,
        }),
    }
}
