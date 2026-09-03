use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use url::Url;

use super::http::HttpClient;
use super::AcquireError;

const BUFFER_SIZE: usize = 64 * 1024;
const MAX_INSPECTION_BYTES: u64 = 8 * 1024 * 1024 * 1024;
static INSPECTION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Untrusted bytes retained beneath the authoring-only quarantine namespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuarantinedDownload {
    /// URL supplied by the author.
    pub original_url: String,
    /// URL that returned the bytes after transport-safe redirects.
    pub final_url: String,
    /// Content-addressed path beneath `authoring-quarantine`.
    pub path: PathBuf,
    /// Exact downloaded length.
    pub length: u64,
    /// Lowercase SHA-256 of the downloaded bytes.
    pub sha256: String,
}

/// Download an unpinned candidate into an authoring-only quarantine.
///
/// This deliberately does not publish into [`super::ArtifactCache`], because the bytes have
/// no trusted identity until a curator reviews and records this report.
pub fn download_for_inspection(
    url: &str,
    cache_root: &Path,
) -> Result<QuarantinedDownload, AcquireError> {
    let quarantine = cache_root.join("authoring-quarantine");
    let incoming = quarantine.join("incoming");
    fs::create_dir_all(&incoming).map_err(|source| AcquireError::Io {
        action: "create authoring quarantine",
        path: incoming.clone(),
        source,
    })?;
    let temporary = incoming.join(format!(
        "inspection-{}-{}-{}.part",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AcquireError::InvalidClock)?
            .as_nanos(),
        INSPECTION_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|source| AcquireError::Io {
            action: "create quarantined inspection download",
            path: temporary.clone(),
            source,
        })?;

    let response = HttpClient::new().get(url, None)?;
    let status = response.response.status().as_u16();
    if !(200..300).contains(&status) {
        remove_file_if_present(&temporary)?;
        return Err(AcquireError::HttpStatus {
            url: response.final_url,
            status,
        });
    }
    let final_url = response.final_url;
    let mut input = response.response.into_body().into_reader();
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = input
            .read(&mut buffer)
            .map_err(|source| AcquireError::Http {
                url: final_url.clone(),
                message: source.to_string(),
            })?;
        if read == 0 {
            break;
        }
        length = length.checked_add(read as u64).ok_or_else(|| {
            AcquireError::ArchiveLimitExceeded("inspection length overflowed u64".to_owned())
        })?;
        if length > MAX_INSPECTION_BYTES {
            remove_file_if_present(&temporary)?;
            return Err(AcquireError::ArchiveLimitExceeded(format!(
                "inspection download exceeds {MAX_INSPECTION_BYTES} bytes"
            )));
        }
        output
            .write_all(&buffer[..read])
            .map_err(|source| AcquireError::Io {
                action: "write quarantined inspection download",
                path: temporary.clone(),
                source,
            })?;
        hash.update(&buffer[..read]);
    }
    output.sync_all().map_err(|source| AcquireError::Io {
        action: "sync quarantined inspection download",
        path: temporary.clone(),
        source,
    })?;
    drop(output);

    let sha256 = hex::encode(hash.finalize());
    let directory = quarantine.join("sha256").join(&sha256[..2]).join(&sha256);
    fs::create_dir_all(&directory).map_err(|source| AcquireError::Io {
        action: "create quarantined content directory",
        path: directory.clone(),
        source,
    })?;
    let filename = candidate_filename(&final_url);
    let destination = directory.join(filename);
    if destination.exists() {
        let (existing_length, existing_hash) = hash_file(&destination)?;
        if existing_length != length || existing_hash != sha256 {
            remove_file_if_present(&temporary)?;
            return Err(AcquireError::CorruptCache {
                digest: sha256,
                message: "authoring quarantine destination does not match its name".to_owned(),
            });
        }
        remove_file_if_present(&temporary)?;
    } else {
        fs::rename(&temporary, &destination).map_err(|source| AcquireError::Io {
            action: "publish quarantined inspection download",
            path: destination.clone(),
            source,
        })?;
    }

    Ok(QuarantinedDownload {
        original_url: url.to_owned(),
        final_url,
        path: destination,
        length,
        sha256,
    })
}

fn candidate_filename(final_url: &str) -> String {
    let candidate = Url::parse(final_url)
        .ok()
        .and_then(|url| {
            url.path_segments()
                .and_then(|mut segments| segments.next_back().map(str::to_owned))
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "artifact.zip".to_owned());
    let sanitized = candidate
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "artifact.zip".to_owned()
    } else {
        sanitized
    }
}

fn hash_file(path: &Path) -> Result<(u64, String), AcquireError> {
    let mut file = File::open(path).map_err(|source| AcquireError::Io {
        action: "open quarantined inspection download",
        path: path.to_path_buf(),
        source,
    })?;
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).map_err(|source| AcquireError::Io {
            action: "read quarantined inspection download",
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

fn remove_file_if_present(path: &Path) -> Result<(), AcquireError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(AcquireError::Io {
            action: "remove quarantined inspection partial",
            path: path.to_path_buf(),
            source,
        }),
    }
}
