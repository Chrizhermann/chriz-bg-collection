use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::events::{EngineEvent, EventSink, Stream};

use super::damaged::{damaged_entry_reason, entry_is_present, set_aside, CacheRepair};
use super::entry_lock::lock_entry;
use super::http::{
    served_filename, validate_download_url, HttpClient, HttpResponse, RedirectPolicy,
};
use super::{
    AcquireError, AcquiredArtifact, ArtifactMetadata, CacheDisposition, DownloadRequest, StopSignal,
};

const BUFFER_SIZE: usize = 64 * 1024;
const MAX_PROGRESS_INTEGER: u64 = (1_u64 << 53) - 1;
/// A download that differs from its pin may be this many times its reviewed length...
const SERVED_GROWTH_FACTOR: u64 = 16;
/// ...and always at least this long, so that small archives can grow as well.
const SERVED_MINIMUM_LIMIT: u64 = 1024 * 1024 * 1024;

/// Content that a campaign already accepted for a request whose download differed from its
/// reviewed pin, so that later steps reuse exactly that archive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedContent {
    /// Accepted archive length in bytes.
    pub length: u64,
    /// Accepted archive SHA-256.
    pub sha256: String,
}

/// What an official download link served: the reviewed archive, or a different file that was
/// accepted and cached under its own digest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServedArtifact {
    /// The cached archive and its provenance.
    pub artifact: AcquiredArtifact,
    /// File name the server gave the download; `None` when no transfer took place.
    pub served_filename: Option<String>,
    /// Redirect hosts that neither the link nor its reviewed redirect hosts name.
    pub unreviewed_hosts: Vec<String>,
}

impl ServedArtifact {
    fn cached(artifact: AcquiredArtifact) -> Self {
        Self {
            artifact,
            served_filename: None,
            unreviewed_hosts: Vec::new(),
        }
    }

    /// Whether the cached bytes are exactly the archive that `request` pins.
    pub fn matches_pin(&self, request: &DownloadRequest) -> bool {
        self.artifact.metadata.length == request.expected_length
            && self
                .artifact
                .metadata
                .sha256
                .eq_ignore_ascii_case(&request.expected_sha256)
    }
}

/// How long a download may be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LengthPolicy {
    /// Exactly the reviewed length, through reviewed redirect hosts only.
    Reviewed,
    /// Whatever the link serves up to `limit` bytes, through any HTTPS redirect.
    Served {
        /// Largest accepted download.
        limit: u64,
    },
}

/// The largest download accepted in place of an archive pinned at `reviewed_length` bytes.
///
/// A grown update must not fail the installation, but a wrong or runaway download must not fill
/// the disk either: sixteen times the reviewed size, and never less than 1 GiB, leaves room for
/// any realistic new version.
pub fn served_length_limit(reviewed_length: u64) -> u64 {
    reviewed_length
        .saturating_mul(SERVED_GROWTH_FACTOR)
        .clamp(SERVED_MINIMUM_LIMIT, MAX_PROGRESS_INTEGER)
}

/// A filesystem-backed cache of immutable, SHA-256-addressed archives.
#[derive(Clone)]
pub struct ArtifactCache {
    root: PathBuf,
    client: HttpClient,
}

#[derive(Debug, Deserialize, Serialize)]
struct PartialState {
    original_url: String,
    /// Total length of the entity being downloaded: the reviewed length, or for a changed
    /// download the length its server announced.
    expected_length: u64,
    etag: String,
}

struct ResumeCandidate {
    start: u64,
    etag: String,
    /// Total length of the entity the partial belongs to.
    total: u64,
}

struct TransferResult {
    final_url: String,
    etag: Option<String>,
    last_modified: Option<String>,
    served_filename: Option<String>,
    unreviewed_hosts: Vec<String>,
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

impl ArtifactCache {
    /// Open or create a cache rooted at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, AcquireError> {
        Self::open_with_client(root.into(), HttpClient::new())
    }

    fn open_with_client(root: PathBuf, client: HttpClient) -> Result<Self, AcquireError> {
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
        Ok(Self { root, client })
    }

    /// Acquire one pinned artifact, returning only a rehashed cache hit or verified download.
    pub fn acquire(
        &self,
        request: &DownloadRequest,
        sink: &dyn EventSink,
    ) -> Result<AcquiredArtifact, AcquireError> {
        self.acquire_with_stop(request, sink, &StopSignal::never())
    }

    /// [`Self::acquire`], giving up with [`AcquireError::Stopped`] instead of downloading or
    /// publishing once `stop` reports that the user stopped, also while another download of the
    /// same artifact holds its cache entry.
    pub fn acquire_with_stop(
        &self,
        request: &DownloadRequest,
        sink: &dyn EventSink,
        stop: &StopSignal,
    ) -> Result<AcquiredArtifact, AcquireError> {
        let digest = validate_request(request)?;
        validate_download_url(&request.url)?;
        request.route.check_request(&request.url)?;
        let paths = self.paths(request, &digest);

        let mut repaired = None;
        {
            let _guard = lock_entry(&paths.lock, stop)?;
            if let Some(hit) =
                self.usable_cache_entry(&paths, &digest, request.expected_length, &mut repaired)?
            {
                sink.emit(EngineEvent::StepProgress {
                    id: request.request_id.clone(),
                    done: request.expected_length,
                    total: request.expected_length,
                });
                return Ok(hit);
            }
        }

        let transfer = self.transfer(request, &paths, sink, LengthPolicy::Reviewed)?;
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

        let _guard = lock_entry(&paths.lock, stop)?;
        if let Some(hit) =
            self.usable_cache_entry(&paths, &digest, request.expected_length, &mut repaired)?
        {
            remove_if_exists(&paths.partial)?;
            remove_if_exists(&paths.partial_state)?;
            return Ok(AcquiredArtifact { repaired, ..hit });
        }
        publish(&paths, request, &metadata)?;
        Ok(AcquiredArtifact {
            archive_path: paths.archive,
            metadata_path: paths.metadata,
            metadata,
            disposition: CacheDisposition::Downloaded,
            repaired,
        })
    }

    /// Acquire what the official link serves, for an installation that must not fail because a
    /// download changed after it was reviewed.
    ///
    /// The cache is asked first: for `accepted`, the changed content this campaign accepted
    /// earlier, so that a resumed campaign keeps exactly the archive it started with; otherwise
    /// for the reviewed archive. Only then is the link asked, and whatever it serves is
    /// accepted: HTTPS redirects may lead to any host (the hosts outside the reviewed ones are
    /// reported), a partial download resumes when the server proves the same entity, and the
    /// file is cached under its own digest. A plain-HTTP downgrade is still refused, and the
    /// download may be at most [`served_length_limit`] bytes long.
    pub fn acquire_as_served(
        &self,
        request: &DownloadRequest,
        accepted: Option<&AcceptedContent>,
        sink: &dyn EventSink,
        stop: &StopSignal,
    ) -> Result<ServedArtifact, AcquireError> {
        let digest = validate_request(request)?;
        validate_download_url(&request.url)?;
        request.route.check_request(&request.url)?;
        let reviewed = self.paths(request, &digest);

        // A campaign that accepted a changed file never switches to the reviewed one, even when
        // another installation put that one into the shared cache.
        let (cached_digest, cached_length) = match accepted {
            Some(accepted) => (
                validate_sha256(&accepted.sha256, "accepted sha256")?,
                accepted.length,
            ),
            None => (digest.clone(), request.expected_length),
        };
        let mut repaired = None;
        {
            let paths = self.paths(request, &cached_digest);
            let _guard = lock_entry(&paths.lock, stop)?;
            if let Some(hit) =
                self.usable_cache_entry(&paths, &cached_digest, cached_length, &mut repaired)?
            {
                sink.emit(EngineEvent::StepProgress {
                    id: request.request_id.clone(),
                    done: cached_length,
                    total: cached_length,
                });
                return Ok(ServedArtifact::cached(hit));
            }
        }

        let policy = LengthPolicy::Served {
            limit: served_length_limit(request.expected_length),
        };
        let transfer = self.transfer(request, &reviewed, sink, policy)?;
        let (actual_length, actual_digest) = hash_file(&reviewed.partial)?;
        sync_file(&reviewed.partial)?;
        // The partial is named by the request, so the paths below share it with `reviewed`.
        let paths = self.paths(request, &actual_digest);
        let metadata = ArtifactMetadata {
            original_url: request.url.clone(),
            final_url: transfer.final_url,
            etag: transfer.etag,
            last_modified: transfer.last_modified,
            length: actual_length,
            sha256: actual_digest.clone(),
            completed_at_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| AcquireError::InvalidClock)?
                .as_secs(),
        };

        let _guard = lock_entry(&paths.lock, stop)?;
        if let Some(hit) =
            self.usable_cache_entry(&paths, &actual_digest, actual_length, &mut repaired)?
        {
            remove_if_exists(&paths.partial)?;
            remove_if_exists(&paths.partial_state)?;
            return Ok(ServedArtifact {
                artifact: AcquiredArtifact { repaired, ..hit },
                served_filename: transfer.served_filename,
                unreviewed_hosts: transfer.unreviewed_hosts,
            });
        }
        publish(&paths, request, &metadata)?;
        Ok(ServedArtifact {
            artifact: AcquiredArtifact {
                archive_path: paths.archive,
                metadata_path: paths.metadata,
                metadata,
                disposition: CacheDisposition::Downloaded,
                repaired,
            },
            served_filename: transfer.served_filename,
            unreviewed_hosts: transfer.unreviewed_hosts,
        })
    }

    /// A verified cache hit, or `None` when the object must be downloaded. Called with the
    /// digest lock held. A damaged entry is set aside so the download can replace it; the
    /// first repair of this call is reported.
    fn usable_cache_entry(
        &self,
        paths: &CachePaths,
        digest: &str,
        expected_length: u64,
        repaired: &mut Option<CacheRepair>,
    ) -> Result<Option<AcquiredArtifact>, AcquireError> {
        match verified_cache_entry(paths, digest, expected_length) {
            Ok(hit) => Ok(hit),
            Err(error) => {
                let owned = [paths.archive.as_path(), paths.metadata.as_path()];
                let Some(reason) = damaged_entry_reason(&error, &owned) else {
                    return Err(error);
                };
                for entry in owned {
                    if entry_is_present(entry)? {
                        set_aside(&self.root, entry, digest)?;
                    }
                }
                repaired.get_or_insert(CacheRepair { reason });
                Ok(None)
            }
        }
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
        policy: LengthPolicy,
    ) -> Result<TransferResult, AcquireError> {
        retry_transfer(
            request,
            sink,
            || self.transfer_once(request, paths, sink, policy),
            std::thread::sleep,
        )
    }

    fn request(
        &self,
        request: &DownloadRequest,
        range: Option<(u64, &str)>,
        policy: LengthPolicy,
    ) -> Result<HttpResponse, AttemptFailure> {
        let redirects = match policy {
            LengthPolicy::Reviewed => request.route.redirect_policy(&request.redirect_hosts),
            // Whatever the official link serves is installed, so its redirects may lead to any
            // HTTPS host, whichever route published it; the hosts are reported.
            LengthPolicy::Served { .. } => RedirectPolicy::NoteUnreviewed(&request.redirect_hosts),
        };
        self.client
            .get_with_redirect_policy(&request.url, range, redirects)
            .map_err(classify_transport_error)
    }

    fn transfer_once(
        &self,
        request: &DownloadRequest,
        paths: &CachePaths,
        sink: &dyn EventSink,
        policy: LengthPolicy,
    ) -> Result<TransferResult, AttemptFailure> {
        let resume =
            load_resume_candidate(request, paths, policy).map_err(AttemptFailure::Fatal)?;
        let range = resume
            .as_ref()
            .map(|candidate| (candidate.start, candidate.etag.as_str()));
        let response = self.request(request, range, policy)?;

        if let Some(candidate) = resume {
            match response.response.status().as_u16() {
                206 if exact_resume_response(&response, &candidate) => {
                    return stream_response(
                        response,
                        request,
                        paths,
                        sink,
                        Some(&candidate),
                        policy,
                    );
                }
                200 => {
                    return stream_response(response, request, paths, sink, None, policy);
                }
                status if is_transient_status(status) => {
                    return Err(AttemptFailure::Retryable(AcquireError::HttpStatus {
                        url: response.final_url,
                        status,
                    }));
                }
                _ => {
                    remove_if_exists(&paths.partial_state).map_err(AttemptFailure::Fatal)?;
                    let fresh = self.request(request, None, policy)?;
                    return handle_fresh_response(fresh, request, paths, sink, policy);
                }
            }
        }
        handle_fresh_response(response, request, paths, sink, policy)
    }
}

// Inject the wait operation so schedule/termination tests do not sleep or download.
// Success and fatal failures leave immediately; only transient failures use the budget.
fn retry_transfer<T>(
    request: &DownloadRequest,
    sink: &dyn EventSink,
    mut transfer: impl FnMut() -> Result<T, AttemptFailure>,
    mut wait: impl FnMut(Duration),
) -> Result<T, AcquireError> {
    let mut last_error = String::new();
    for attempt in 1..=request.max_attempts {
        match transfer() {
            Ok(result) => return Ok(result),
            Err(AttemptFailure::Fatal(error)) => return Err(error),
            Err(AttemptFailure::Retryable(error)) => last_error = error.to_string(),
        }
        if attempt == request.max_attempts {
            break;
        }

        let seconds = match attempt {
            1 => 0,
            2 => 2,
            3 => 10,
            _ => 30,
        };
        let when = if seconds == 0 {
            "immediately".to_owned()
        } else {
            format!("in {seconds} seconds")
        };
        sink.emit(EngineEvent::ConsoleLine {
            step_id: format!("acquire:{}", request.request_id),
            stream: Stream::Stdout,
            line: format!(
                "Download interrupted for {}. Retrying {when} (attempt {}/{}).",
                request.request_id,
                attempt + 1,
                request.max_attempts,
            ),
        });
        if seconds != 0 {
            wait(Duration::from_secs(seconds));
        }
    }
    Err(AcquireError::RetryExhausted {
        attempts: request.max_attempts,
        last_error,
    })
}

fn validate_request(request: &DownloadRequest) -> Result<String, AcquireError> {
    if request.request_id.is_empty()
        || request.request_id.len() > 128
        || !request
            .request_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(AcquireError::InvalidRequest(
            "request_id must contain 1-128 ASCII letters, digits, '-', '_' or '.'".to_owned(),
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

fn validate_sha256(value: &str, label: &str) -> Result<String, AcquireError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AcquireError::InvalidRequest(format!(
            "{label} must be exactly 64 hexadecimal characters"
        )));
    }
    Ok(value.to_ascii_lowercase())
}

fn handle_fresh_response(
    response: HttpResponse,
    request: &DownloadRequest,
    paths: &CachePaths,
    sink: &dyn EventSink,
    policy: LengthPolicy,
) -> Result<TransferResult, AttemptFailure> {
    let status = response.response.status().as_u16();
    if status == 200 {
        return stream_response(response, request, paths, sink, None, policy);
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

/// Streams `response` into the partial file: from byte zero, or after the bytes of `resume`.
fn stream_response(
    response: HttpResponse,
    request: &DownloadRequest,
    paths: &CachePaths,
    sink: &dyn EventSink,
    resume: Option<&ResumeCandidate>,
    policy: LengthPolicy,
) -> Result<TransferResult, AttemptFailure> {
    // Before anything is written: a page instead of the reviewed archive is not a partial
    // download. A changed download is taken as it is served: its route's checks of the reviewed
    // response (such as the attachment name) cannot hold for another file, and whether the file
    // can be used is decided by the archive checks, which protect this computer either way.
    if matches!(policy, LengthPolicy::Reviewed) {
        request
            .route
            .check_response(&response)
            .map_err(AttemptFailure::Fatal)?;
    }
    let start = resume.map_or(0, |candidate| candidate.start);
    let truncate = resume.is_none();
    let response_length = header_u64(&response.response, "content-length");
    // Total length of the entity, when known; a changed download may announce none.
    let total = match policy {
        LengthPolicy::Reviewed => {
            let expected_response_length = request.expected_length - start;
            if let Some(actual) = response_length {
                if actual != expected_response_length {
                    return Err(AttemptFailure::Fatal(AcquireError::LengthMismatch {
                        expected: expected_response_length,
                        actual,
                    }));
                }
            }
            Some(request.expected_length)
        }
        LengthPolicy::Served { limit } => {
            let total = match resume {
                Some(candidate) => {
                    if response_length.is_some_and(|length| length != candidate.total - start) {
                        return Err(AttemptFailure::Fatal(AcquireError::InvalidRange {
                            url: response.final_url,
                            message: "partial response length differs from its content range"
                                .to_owned(),
                        }));
                    }
                    Some(candidate.total)
                }
                None => response_length,
            };
            if let Some(total) = total.filter(|total| *total > limit) {
                return Err(AttemptFailure::Fatal(
                    AcquireError::ServedDownloadTooLarge {
                        length: total,
                        limit,
                    },
                ));
            }
            total
        }
    };
    let etag = header_string(&response.response, "etag");
    let last_modified = header_string(&response.response, "last-modified");
    let filename = served_filename(&response);
    let unreviewed_hosts = response.unreviewed_hosts.clone();

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

    // Only an entity of known length can be resumed: the partial response must name it.
    match (etag.as_deref().filter(|value| is_strong_etag(value)), total) {
        (Some(strong), Some(total)) => write_partial_state(
            &paths.partial_state,
            &PartialState {
                original_url: request.url.clone(),
                expected_length: total,
                etag: strong.to_owned(),
            },
        )
        .map_err(AttemptFailure::Fatal)?,
        _ => remove_if_exists(&paths.partial_state).map_err(AttemptFailure::Fatal)?,
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
        if let Some(total) = total.filter(|total| done > *total) {
            return Err(AttemptFailure::Fatal(AcquireError::LengthMismatch {
                expected: total,
                actual: done,
            }));
        }
        if let LengthPolicy::Served { limit } = policy {
            if done > limit {
                return Err(AttemptFailure::Fatal(
                    AcquireError::ServedDownloadTooLarge {
                        length: done,
                        limit,
                    },
                ));
            }
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
            total: total.unwrap_or_else(|| done.max(request.expected_length)),
        });
    }
    if let Some(total) = total.filter(|total| done != *total) {
        let _ = file.sync_all();
        return Err(AttemptFailure::Retryable(AcquireError::LengthMismatch {
            expected: total,
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
        served_filename: filename,
        unreviewed_hosts,
    })
}

fn exact_resume_response(response: &HttpResponse, candidate: &ResumeCandidate) -> bool {
    let Some(etag) = header_string(&response.response, "etag") else {
        return false;
    };
    if etag != candidate.etag || !is_strong_etag(&etag) || candidate.total == 0 {
        return false;
    }
    let expected = format!(
        "bytes {}-{}/{}",
        candidate.start,
        candidate.total - 1,
        candidate.total
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
    policy: LengthPolicy,
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
    if length == 0 || (policy == LengthPolicy::Reviewed && length >= request.expected_length) {
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
    let total_acceptable = match policy {
        LengthPolicy::Reviewed => state.expected_length == request.expected_length,
        // A changed download resumes only toward the same entity, which the server must still
        // confirm by its strong entity tag and exact content range.
        LengthPolicy::Served { limit } => state.expected_length <= limit,
    };
    if state.original_url != request.url
        || !total_acceptable
        || length >= state.expected_length
        || !is_strong_etag(&state.etag)
    {
        return Ok(None);
    }
    Ok(Some(ResumeCandidate {
        start: length,
        etag: state.etag,
        total: state.expected_length,
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

fn verified_cache_entry(
    paths: &CachePaths,
    digest: &str,
    expected_length: u64,
) -> Result<Option<AcquiredArtifact>, AcquireError> {
    let archive_exists = entry_is_present(&paths.archive)?;
    let metadata_exists = entry_is_present(&paths.metadata)?;
    if !archive_exists && !metadata_exists {
        return Ok(None);
    }
    if archive_exists != metadata_exists {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: "archive and metadata must both exist".to_owned(),
        });
    }
    // Read before parsing so that a read failure keeps its I/O kind: only a record that was
    // read completely and does not parse proves damage.
    let bytes = fs::read(&paths.metadata).map_err(|source| AcquireError::Io {
        action: "read cache metadata",
        path: paths.metadata.clone(),
        source,
    })?;
    let metadata: ArtifactMetadata =
        serde_json::from_slice(&bytes).map_err(|source| AcquireError::Metadata {
            path: paths.metadata.clone(),
            message: source.to_string(),
        })?;
    let (actual_length, actual_digest) = hash_file(&paths.archive)?;
    if metadata.sha256 != digest || actual_digest != digest || metadata.length != actual_length {
        return Err(AcquireError::CorruptCache {
            digest: digest.to_owned(),
            message: format!(
                "metadata/file mismatch (metadata {} bytes {}, file {} bytes {})",
                metadata.sha256, metadata.length, actual_digest, actual_length
            ),
        });
    }
    // The entry is consistent and holds the requested digest, so a different length is an
    // error in the request. The shared entry stays for requests that pin it correctly.
    if actual_length != expected_length {
        return Err(AcquireError::LengthMismatch {
            expected: expected_length,
            actual: actual_length,
        });
    }
    Ok(Some(AcquiredArtifact {
        archive_path: paths.archive.clone(),
        metadata_path: paths.metadata.clone(),
        metadata,
        disposition: CacheDisposition::Hit,
        repaired: None,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};

    use sha2::{Digest, Sha256};
    use tempfile::TempDir;

    use super::{retry_transfer, validate_request, ArtifactCache, AttemptFailure};
    use crate::acquire::http::{HttpClient, HttpTimeouts};
    use crate::acquire::{AcquireError, CacheDisposition, DownloadRequest, DownloadRoute};
    use crate::events::{ChannelSink, EngineEvent, Stream};

    fn retry_request(max_attempts: u32) -> DownloadRequest {
        DownloadRequest {
            request_id: "xan-19".to_owned(),
            url: "https://example.invalid/archive.zip".to_owned(),
            expected_length: 1,
            expected_sha256: "0".repeat(64),
            redirect_hosts: Vec::new(),
            max_attempts,
            route: DownloadRoute::Generic,
        }
    }

    fn temporary_failure() -> AttemptFailure {
        AttemptFailure::Retryable(AcquireError::HttpStatus {
            url: "https://example.invalid/archive.zip".to_owned(),
            status: 500,
        })
    }

    #[test]
    fn progressive_retries_wait_only_before_remaining_attempts_and_report_them() {
        let request = retry_request(5);
        let (sink, events) = ChannelSink::unbounded();
        let mut calls = 0;
        let mut waits = Vec::new();
        let result: Result<(), _> = retry_transfer(
            &request,
            &sink,
            || {
                calls += 1;
                Err(temporary_failure())
            },
            |delay| waits.push(delay.as_secs()),
        );

        assert_eq!(calls, 5);
        assert_eq!(waits, [2, 10, 30]);
        assert!(
            matches!(result, Err(AcquireError::RetryExhausted { attempts: 5, last_error }) if last_error.contains("status 500"))
        );
        let lines = events
            .try_iter()
            .map(|event| match event {
                EngineEvent::ConsoleLine {
                    step_id,
                    stream,
                    line,
                } => {
                    assert_eq!(step_id, "acquire:xan-19");
                    assert_eq!(stream, Stream::Stdout);
                    line
                }
                other => panic!("Retry must not signal failure/attention: {other:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 4);
        assert!(lines[0].contains("immediately") && lines[0].contains("2/5"));
        for (line, delay) in lines[1..].iter().zip([2, 10, 30]) {
            assert!(line.contains(&format!("in {delay} seconds")), "{line}");
        }
        assert!(lines[3].contains("5/5"));
    }

    #[test]
    fn progressive_retries_stop_immediately_on_success_at_any_attempt() {
        for succeed_on in 1..=5 {
            let (sink, _) = ChannelSink::unbounded();
            let mut calls = 0;
            let mut waits = Vec::new();
            let result = retry_transfer(
                &retry_request(5),
                &sink,
                || {
                    calls += 1;
                    if calls == succeed_on {
                        Ok(42)
                    } else {
                        Err(temporary_failure())
                    }
                },
                |delay| waits.push(delay.as_secs()),
            )
            .unwrap();
            assert_eq!(result, 42);
            assert_eq!(calls, succeed_on);
            let expected: Vec<_> = [2, 10, 30]
                .into_iter()
                .take((succeed_on as usize).saturating_sub(2))
                .collect();
            assert_eq!(waits, expected);
        }
    }

    #[test]
    fn progressive_retries_do_not_retry_a_permanent_failure() {
        let (sink, events) = ChannelSink::unbounded();
        let result: Result<(), _> = retry_transfer(
            &retry_request(5),
            &sink,
            || {
                Err(AttemptFailure::Fatal(AcquireError::HttpStatus {
                    url: "https://example.invalid/archive.zip".to_owned(),
                    status: 404,
                }))
            },
            |_| panic!("Permanent errors must not wait"),
        );
        assert!(matches!(
            result,
            Err(AcquireError::HttpStatus { status: 404, .. })
        ));
        assert!(events.try_iter().next().is_none());
    }

    #[test]
    fn progressive_retries_respect_smaller_limits_and_cap_longer_waits() {
        for (limit, expected) in [
            (1, vec![]),
            (2, vec![]),
            (3, vec![2]),
            (6, vec![2, 10, 30, 30]),
        ] {
            let (sink, events) = ChannelSink::unbounded();
            let mut calls = 0;
            let mut waits = Vec::new();
            let result: Result<(), _> = retry_transfer(
                &retry_request(limit),
                &sink,
                || {
                    calls += 1;
                    Err(temporary_failure())
                },
                |delay: Duration| waits.push(delay.as_secs()),
            );
            assert!(
                matches!(result, Err(AcquireError::RetryExhausted { attempts, .. }) if attempts == limit)
            );
            assert_eq!(calls, limit);
            assert_eq!(waits, expected);
            assert_eq!(events.try_iter().count(), (limit - 1) as usize);
        }
    }

    #[test]
    fn accepts_versioned_artifact_ids_with_dots() {
        let request = DownloadRequest {
            request_id: "bg1ub-17.1".to_owned(),
            url: "https://example.invalid/bg1ub.iemod".to_owned(),
            expected_length: 1,
            expected_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                .to_owned(),
            redirect_hosts: Vec::new(),
            max_attempts: 1,
            route: DownloadRoute::Generic,
        };

        assert!(validate_request(&request).is_ok());
    }

    // Loopback transfers with millisecond limits. Each connection has its own server thread,
    // so a connection held open by a stalled reply does not delay the next attempt.

    const TEST_TIMEOUTS: HttpTimeouts = HttpTimeouts {
        resolve: Duration::from_millis(300),
        connect: Duration::from_millis(300),
        send_request: Duration::from_millis(300),
        body_stall: Duration::from_millis(300),
    };
    // Generous for a loaded test machine; still far below a production limit.
    const DETECTION_SLACK: Duration = Duration::from_millis(1500);

    struct Connection {
        stream: TcpStream,
        headers: BTreeMap<String, String>,
    }

    impl Connection {
        fn reply_headers(&mut self, status: &str, headers: &[(&str, String)], length: usize) {
            let mut head = format!("HTTP/1.1 {status}\r\n");
            for (name, value) in headers {
                head.push_str(&format!("{name}: {value}\r\n"));
            }
            head.push_str(&format!(
                "Content-Length: {length}\r\nConnection: close\r\n\r\n"
            ));
            self.send(head.as_bytes());
        }

        fn send(&mut self, bytes: &[u8]) {
            self.stream.write_all(bytes).expect("write test response");
            self.stream.flush().expect("flush test response");
        }

        /// Keep the connection open without sending anything until the client gives up.
        fn hold_until_client_closes(&mut self) {
            self.stream
                .set_read_timeout(Some(Duration::from_millis(10)))
                .expect("set hold read timeout");
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut byte = [0_u8; 1];
            while Instant::now() < deadline {
                match self.stream.read(&mut byte) {
                    Ok(0) => return,
                    Ok(_) => {}
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) => {}
                    Err(_) => return,
                }
            }
            panic!("client kept a stalled test connection open for 10 seconds");
        }
    }

    type Handler = Box<dyn FnOnce(&mut Connection) + Send>;

    /// Serve one connection per handler, in order. Joining returns each request's headers.
    fn serve(handlers: Vec<Handler>) -> (String, JoinHandle<Vec<BTreeMap<String, String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback test server");
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let server = thread::spawn(move || {
            let mut connections = Vec::new();
            for handler in handlers {
                let deadline = Instant::now() + Duration::from_secs(10);
                let stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < deadline, "test request did not arrive");
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("accept test request: {error}"),
                    }
                };
                connections.push(thread::spawn(move || {
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .unwrap();
                    let mut headers = BTreeMap::new();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    reader.read_line(&mut line).expect("read request line");
                    loop {
                        line.clear();
                        reader.read_line(&mut line).expect("read request header");
                        let Some((name, value)) = line.trim_end().split_once(':') else {
                            break;
                        };
                        headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
                    }
                    let mut connection = Connection { stream, headers };
                    handler(&mut connection);
                    connection.headers
                }));
            }
            connections
                .into_iter()
                .map(|connection| connection.join().expect("test connection thread"))
                .collect()
        });
        (base_url, server)
    }

    fn archive_bytes(length: usize) -> Vec<u8> {
        (0..length).map(|index| (index % 251) as u8).collect()
    }

    fn download(id: &str, url: String, bytes: &[u8]) -> DownloadRequest {
        DownloadRequest {
            request_id: id.to_owned(),
            url,
            expected_length: bytes.len() as u64,
            expected_sha256: hex::encode(Sha256::digest(bytes)),
            redirect_hosts: Vec::new(),
            max_attempts: 1,
            route: DownloadRoute::Generic,
        }
    }

    fn test_cache(temp: &TempDir) -> ArtifactCache {
        ArtifactCache::open_with_client(
            temp.path().join("cache"),
            HttpClient::with_timeouts(TEST_TIMEOUTS),
        )
        .unwrap()
    }

    fn longest_test_timeout() -> Duration {
        [
            TEST_TIMEOUTS.resolve,
            TEST_TIMEOUTS.connect,
            TEST_TIMEOUTS.send_request,
            TEST_TIMEOUTS.body_stall,
        ]
        .into_iter()
        .max()
        .unwrap()
    }

    #[test]
    fn a_steady_download_longer_than_every_timeout_completes() {
        let bytes = archive_bytes(64_000);
        let body = bytes.clone();
        let (url, server) = serve(vec![Box::new(move |connection: &mut Connection| {
            connection.reply_headers("200 OK", &[("ETag", "\"steady\"".to_owned())], body.len());
            // ureq gives a read that starts after a passed deadline one more second, so a
            // total limit only fails a wait it interrupts. Chunks 120 ms apart put the
            // 300 ms limits in the middle of a wait, while staying well inside the stall limit.
            for chunk in body.chunks(6_400) {
                thread::sleep(Duration::from_millis(120));
                connection.send(chunk);
            }
        })]);
        let temp = TempDir::new().unwrap();
        let (sink, _) = ChannelSink::unbounded();

        let started = Instant::now();
        let acquired = test_cache(&temp)
            .acquire(&download("steady", url, &bytes), &sink)
            .unwrap();

        assert!(
            started.elapsed() > 3 * longest_test_timeout(),
            "the transfer must outlast every limit, took {:?}",
            started.elapsed()
        );
        assert_eq!(acquired.disposition, CacheDisposition::Downloaded);
        assert_eq!(fs::read(&acquired.archive_path).unwrap(), bytes);
        server.join().unwrap();
    }

    #[test]
    fn a_stalled_body_fails_after_the_stall_window_and_the_next_attempt_resumes() {
        let bytes = archive_bytes(32 * 1024);
        let prefix_len = 12_000;
        let total = bytes.len();
        let (first, rest) = (bytes[..prefix_len].to_vec(), bytes[prefix_len..].to_vec());
        let (url, server) = serve(vec![
            Box::new(move |connection: &mut Connection| {
                connection.reply_headers("200 OK", &[("ETag", "\"stall-v1\"".to_owned())], total);
                connection.send(&first);
                connection.hold_until_client_closes();
            }),
            Box::new(move |connection: &mut Connection| {
                connection.reply_headers(
                    "206 Partial Content",
                    &[
                        ("ETag", "\"stall-v1\"".to_owned()),
                        (
                            "Content-Range",
                            format!("bytes {prefix_len}-{}/{total}", total - 1),
                        ),
                    ],
                    rest.len(),
                );
                connection.send(&rest);
            }),
        ]);
        let temp = TempDir::new().unwrap();
        let cache = test_cache(&temp);
        let (sink, _) = ChannelSink::unbounded();
        let request = download("stall", url, &bytes);

        let started = Instant::now();
        let error = cache.acquire(&request, &sink).unwrap_err();
        let elapsed = started.elapsed();

        // Exhausting the budget, instead of returning the error itself, proves that the stall
        // was classified as retryable.
        assert!(
            matches!(&error, AcquireError::RetryExhausted { attempts: 1, last_error } if last_error.contains("timeout: receive body")),
            "{error}"
        );
        assert!(
            elapsed >= TEST_TIMEOUTS.body_stall
                && elapsed < TEST_TIMEOUTS.body_stall + DETECTION_SLACK,
            "stall detected after {elapsed:?}"
        );
        assert_eq!(
            fs::metadata(temp.path().join("cache/partial/stall.part"))
                .unwrap()
                .len(),
            prefix_len as u64
        );

        let acquired = cache.acquire(&request, &sink).unwrap();

        assert_eq!(acquired.disposition, CacheDisposition::Downloaded);
        assert_eq!(fs::read(&acquired.archive_path).unwrap(), bytes);
        let seen = server.join().unwrap();
        assert!(!seen[0].contains_key("range"));
        assert_eq!(seen[1]["range"], format!("bytes={prefix_len}-"));
        assert_eq!(seen[1]["if-range"], "\"stall-v1\"");
    }

    #[test]
    fn a_server_that_never_sends_headers_fails_within_the_header_bound() {
        let bytes = archive_bytes(1024);
        let (url, server) = serve(vec![Box::new(|connection: &mut Connection| {
            connection.hold_until_client_closes();
        })]);
        let temp = TempDir::new().unwrap();
        let (sink, _) = ChannelSink::unbounded();

        let started = Instant::now();
        let error = test_cache(&temp)
            .acquire(&download("silent", url, &bytes), &sink)
            .unwrap_err();
        let elapsed = started.elapsed();

        assert!(
            matches!(&error, AcquireError::RetryExhausted { attempts: 1, last_error } if last_error.contains("timeout: send request")),
            "{error}"
        );
        assert!(
            elapsed >= TEST_TIMEOUTS.send_request
                && elapsed < TEST_TIMEOUTS.send_request + DETECTION_SLACK,
            "missing headers detected after {elapsed:?}"
        );
        server.join().unwrap();
    }
}
