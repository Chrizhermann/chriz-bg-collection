//! Process-scoped locks for managed targets and immutable cache objects.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use fs4::fs_std::FileExt;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// A failure to validate or acquire an installer lock.
#[derive(Debug, Error)]
pub enum LockError {
    /// Cache keys must be canonical SHA-256 values before they become path components.
    #[error("invalid cache digest {digest:?}; expected 64 hexadecimal characters")]
    InvalidDigest { digest: String },
    /// An existing path in the lock surface is not a direct regular file/directory.
    #[error("unsafe lock path {path}: {reason}")]
    UnsafePath { path: PathBuf, reason: String },
    /// Another live handle owns the requested operating-system lock.
    #[error("{kind} lock is already held at {path}")]
    Contended { kind: &'static str, path: PathBuf },
    /// Filesystem work required to open or lock the handle failed.
    #[error("could not {action} at {path}: {source}")]
    Io {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// Exclusive operating-system lock for one managed installation root.
#[derive(Debug)]
pub struct TargetLock {
    file: File,
    path: PathBuf,
    target: PathBuf,
}

impl TargetLock {
    /// Attempts to lock one prospective managed target without waiting.
    ///
    /// Locks live in `<registry_root>/targets/<canonical-target-digest>.lock`, so lock-first
    /// orchestration does not create `.chriz` before the campaign ledger claims it. Keeping
    /// the lock file after this handle closes is intentional: ownership is represented by
    /// the OS lock, never by sentinel-file existence.
    pub fn try_acquire(
        registry_root: impl AsRef<Path>,
        managed_root: impl AsRef<Path>,
    ) -> Result<Self, LockError> {
        let target = prospective_target_identity(managed_root.as_ref())?;
        let locks = registry_root.as_ref().join("targets");
        create_or_validate_direct_directory(&locks)?;
        let identity = target_identity_digest(&target);
        let path = locks.join(format!("{identity}.lock"));
        let file = open_lock_file(&path, "target")?;
        Ok(Self { file, path, target })
    }

    /// Stable lock-file path retained for diagnostics.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Prospective canonical managed root represented by this lock.
    pub fn target(&self) -> &Path {
        &self.target
    }
}

impl Drop for TargetLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

/// Exclusive operating-system lock for one content-addressed cache digest.
#[derive(Debug)]
pub struct CacheDigestLock {
    file: File,
    path: PathBuf,
}

impl CacheDigestLock {
    /// Attempts to lock `<cache_root>/locks/<sha256>.lock` without waiting.
    pub fn try_acquire(cache_root: impl AsRef<Path>, digest: &str) -> Result<Self, LockError> {
        let digest = normalize_digest(digest)?;
        let locks = cache_root.as_ref().join("locks");
        create_or_validate_direct_directory(&locks)?;
        let path = locks.join(format!("{digest}.lock"));
        let file = open_lock_file(&path, "cache digest")?;
        Ok(Self { file, path })
    }

    /// Stable lock-file path retained for diagnostics.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for CacheDigestLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

/// Acquires the target lock for the complete app or recipe replacement operation.
///
/// The caller must retain the returned guard until replacement is fully committed; returning
/// a guard rather than a boolean avoids a check-then-update race with a new build.
pub fn acquire_target_for_update(
    registry_root: impl AsRef<Path>,
    managed_root: impl AsRef<Path>,
) -> Result<TargetLock, LockError> {
    TargetLock::try_acquire(registry_root, managed_root)
}

fn normalize_digest(digest: &str) -> Result<String, LockError> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(LockError::InvalidDigest {
            digest: digest.to_owned(),
        });
    }
    Ok(digest.to_ascii_lowercase())
}

fn prospective_target_identity(path: &Path) -> Result<PathBuf, LockError> {
    let absolute = std::path::absolute(path).map_err(|source| LockError::Io {
        action: "resolve absolute target path",
        path: path.to_path_buf(),
        source,
    })?;
    let mut existing = absolute.as_path();
    while !existing.exists() {
        existing = existing.parent().ok_or_else(|| LockError::UnsafePath {
            path: absolute.clone(),
            reason: "target has no existing ancestor".to_owned(),
        })?;
    }
    let canonical = fs::canonicalize(existing).map_err(|source| LockError::Io {
        action: "canonicalize target ancestor",
        path: existing.to_path_buf(),
        source,
    })?;
    let suffix = absolute
        .strip_prefix(existing)
        .map_err(|_| LockError::UnsafePath {
            path: absolute.clone(),
            reason: "target could not be resolved below its existing ancestor".to_owned(),
        })?;
    Ok(canonical.join(suffix))
}

fn target_identity_digest(target: &Path) -> String {
    let display = target.to_string_lossy();
    #[cfg(windows)]
    let display = display.to_lowercase();
    hex::encode(Sha256::digest(display.as_bytes()))
}

fn open_lock_file(path: &Path, kind: &'static str) -> Result<File, LockError> {
    validate_direct_regular_file_if_present(path)?;
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)
        .map_err(|source| LockError::Io {
            action: "open lock file",
            path: path.to_path_buf(),
            source,
        })?;
    match file.try_lock_exclusive() {
        Ok(true) => Ok(file),
        Ok(false) => Err(LockError::Contended {
            kind,
            path: path.to_path_buf(),
        }),
        Err(source) => Err(LockError::Io {
            action: "acquire operating-system lock",
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn create_or_validate_direct_directory(path: &Path) -> Result<(), LockError> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|source| LockError::Io {
            action: "create lock directory",
            path: path.to_path_buf(),
            source,
        })?;
    }
    validate_direct_directory(path)
}

fn validate_direct_directory(path: &Path) -> Result<(), LockError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| LockError::Io {
        action: "inspect lock directory",
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || is_windows_reparse_point(&metadata) {
        return Err(LockError::UnsafePath {
            path: path.to_path_buf(),
            reason: "links and reparse points are not allowed".to_owned(),
        });
    }
    if !metadata.is_dir() {
        return Err(LockError::UnsafePath {
            path: path.to_path_buf(),
            reason: "expected a directory".to_owned(),
        });
    }
    Ok(())
}

fn validate_direct_regular_file_if_present(path: &Path) -> Result<(), LockError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(LockError::Io {
                action: "inspect lock file",
                path: path.to_path_buf(),
                source,
            });
        }
    };
    if metadata.file_type().is_symlink() || is_windows_reparse_point(&metadata) {
        return Err(LockError::UnsafePath {
            path: path.to_path_buf(),
            reason: "links and reparse points are not allowed".to_owned(),
        });
    }
    if !metadata.is_file() {
        return Err(LockError::UnsafePath {
            path: path.to_path_buf(),
            reason: "expected a regular file".to_owned(),
        });
    }
    Ok(())
}

#[cfg(windows)]
fn is_windows_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_windows_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}
