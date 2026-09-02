use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::AcquireError;

const BUFFER_SIZE: usize = 64 * 1024;

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

fn validate_digest(value: &str) -> Result<String, AcquireError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AcquireError::InvalidRequest(
            "expected manual SHA-256 must be exactly 64 hexadecimal characters".to_owned(),
        ));
    }
    Ok(value.to_ascii_lowercase())
}
