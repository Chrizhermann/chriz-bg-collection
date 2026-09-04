use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use super::AcquireError;

const BUFFER_SIZE: usize = 64 * 1024;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A user-selected local archive whose bytes match immutable recipe provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedManualArchive {
    /// Exact path explicitly chosen by the user.
    pub path: PathBuf,
    /// Verified lowercase SHA-256.
    pub sha256: String,
    /// Verified archive byte length.
    pub length: u64,
}

/// Verify one explicit user-selected file. This function never scans or polls a folder.
pub fn provide_manual_archive(
    path: &Path,
    expected_sha256: &str,
) -> Result<VerifiedManualArchive, AcquireError> {
    let expected = validate_digest(expected_sha256)?;
    if expected.bytes().all(|byte| byte == b'0') {
        return Err(AcquireError::InvalidRequest(
            "manual archive SHA-256 may not be all zeroes".to_owned(),
        ));
    }
    let metadata = fs::symlink_metadata(path).map_err(|source| AcquireError::Io {
        action: "inspect manual archive",
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(AcquireError::InvalidRequest(
            "manual archive path must name a regular non-symlink file".to_owned(),
        ));
    }
    let mut file = File::open(path).map_err(|source| AcquireError::Io {
        action: "open manual archive",
        path: path.to_path_buf(),
        source,
    })?;
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).map_err(|source| AcquireError::Io {
            action: "read manual archive",
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        length = length.checked_add(read as u64).ok_or_else(|| {
            AcquireError::InvalidRequest("manual archive length overflowed u64".to_owned())
        })?;
        hash.update(&buffer[..read]);
    }
    let actual = hex::encode(hash.finalize());
    if actual != expected {
        return Err(AcquireError::HashMismatch { expected, actual });
    }
    Ok(VerifiedManualArchive {
        path: path.to_path_buf(),
        sha256: actual,
        length,
    })
}

/// Verify one explicit selection and publish an independent copy at an installer-owned path.
///
/// Invalid selected bytes never create or replace the destination. A valid existing destination
/// is reused, while an invalid regular cache file is replaced only after the selection has passed
/// both digest and length checks.
pub fn publish_manual_archive(
    selected: &Path,
    destination: &Path,
    expected_sha256: &str,
    expected_length: u64,
) -> Result<VerifiedManualArchive, AcquireError> {
    let verified = provide_manual_archive(selected, expected_sha256)?;
    if verified.length != expected_length {
        return Err(AcquireError::LengthMismatch {
            expected: expected_length,
            actual: verified.length,
        });
    }

    if selected == destination {
        return Ok(verified);
    }
    let parent = destination.parent().ok_or_else(|| {
        AcquireError::InvalidRequest(
            "manual archive destination must have a parent directory".to_owned(),
        )
    })?;
    fs::create_dir_all(parent).map_err(|source| AcquireError::Io {
        action: "create manual archive cache directory",
        path: parent.to_path_buf(),
        source,
    })?;
    let parent_metadata = fs::symlink_metadata(parent).map_err(|source| AcquireError::Io {
        action: "inspect manual archive cache directory",
        path: parent.to_path_buf(),
        source,
    })?;
    if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
        return Err(AcquireError::InvalidRequest(
            "manual archive cache directory must be a direct directory".to_owned(),
        ));
    }

    if let Some(existing) = inspect_existing_destination(destination)? {
        if existing.sha256 == verified.sha256 && existing.length == verified.length {
            return Ok(existing);
        }
    }

    let temporary = temporary_sibling(destination)?;
    let copy_result = copy_and_verify(selected, &temporary, expected_sha256, expected_length);
    if let Err(error) = copy_result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    if destination.exists() {
        fs::remove_file(destination).map_err(|source| AcquireError::Io {
            action: "remove invalid manual archive cache file",
            path: destination.to_path_buf(),
            source,
        })?;
    }
    if let Err(source) = fs::rename(&temporary, destination) {
        let _ = fs::remove_file(&temporary);
        return Err(AcquireError::Io {
            action: "publish verified manual archive",
            path: destination.to_path_buf(),
            source,
        });
    }
    provide_manual_archive(destination, expected_sha256)
}

fn inspect_existing_destination(
    destination: &Path,
) -> Result<Option<VerifiedManualArchive>, AcquireError> {
    let metadata = match fs::symlink_metadata(destination) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(AcquireError::Io {
                action: "inspect manual archive cache file",
                path: destination.to_path_buf(),
                source,
            })
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AcquireError::InvalidRequest(
            "manual archive cache path must be a regular non-symlink file".to_owned(),
        ));
    }
    match provide_manual_archive(destination, &hash_file(destination)?) {
        Ok(verified) => Ok(Some(verified)),
        Err(error) => Err(error),
    }
}

fn copy_and_verify(
    selected: &Path,
    temporary: &Path,
    expected_sha256: &str,
    expected_length: u64,
) -> Result<(), AcquireError> {
    let mut input = File::open(selected).map_err(|source| AcquireError::Io {
        action: "open selected manual archive",
        path: selected.to_path_buf(),
        source,
    })?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .map_err(|source| AcquireError::Io {
            action: "create manual archive cache temporary",
            path: temporary.to_path_buf(),
            source,
        })?;
    io::copy(&mut input, &mut output).map_err(|source| AcquireError::Io {
        action: "copy selected manual archive",
        path: temporary.to_path_buf(),
        source,
    })?;
    output.sync_all().map_err(|source| AcquireError::Io {
        action: "sync selected manual archive",
        path: temporary.to_path_buf(),
        source,
    })?;
    let copied = provide_manual_archive(temporary, expected_sha256)?;
    if copied.length != expected_length {
        return Err(AcquireError::LengthMismatch {
            expected: expected_length,
            actual: copied.length,
        });
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<String, AcquireError> {
    let mut file = File::open(path).map_err(|source| AcquireError::Io {
        action: "open manual archive cache file",
        path: path.to_path_buf(),
        source,
    })?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).map_err(|source| AcquireError::Io {
            action: "read manual archive cache file",
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(hex::encode(hash.finalize()))
}

fn temporary_sibling(destination: &Path) -> Result<PathBuf, AcquireError> {
    let filename = destination.file_name().ok_or_else(|| {
        AcquireError::InvalidRequest("manual archive destination has no filename".to_owned())
    })?;
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut temporary_name = filename.to_os_string();
    temporary_name.push(format!(".manual.tmp.{}.{}", std::process::id(), sequence));
    Ok(destination.with_file_name(temporary_name))
}

fn validate_digest(value: &str) -> Result<String, AcquireError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AcquireError::InvalidRequest(
            "expected manual SHA-256 must be exactly 64 hexadecimal characters".to_owned(),
        ));
    }
    Ok(value.to_ascii_lowercase())
}
