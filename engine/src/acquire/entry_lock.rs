//! Blocking cross-process locks for one content-addressed cache entry.

use std::fs::{File, OpenOptions};
use std::path::Path;

use fs4::fs_std::FileExt;

use super::AcquireError;

/// Exclusive operating-system lock on one cache entry. Dropping the guard releases it, and the
/// operating system releases it when the process exits, so a lock is never left stale.
pub(super) struct EntryLock(File);

impl Drop for EntryLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

/// Waits for the exclusive lock at `path`, creating the lock file when needed. The lock file
/// stays after release; ownership is the operating-system lock, never the file's existence.
pub(super) fn lock_entry(path: &Path) -> Result<EntryLock, AcquireError> {
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
    Ok(EntryLock(file))
}
