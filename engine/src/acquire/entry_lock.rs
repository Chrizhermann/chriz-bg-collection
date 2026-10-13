//! Cross-process locks for one content-addressed cache entry, waited for until they are free
//! or the user stops.

use std::fs::{File, OpenOptions};
use std::path::Path;
use std::time::Duration;

use fs4::fs_std::FileExt;

use super::{AcquireError, StopSignal};

/// How often a wait checks whether the lock is free and whether the user stopped.
const WAIT_INTERVAL: Duration = Duration::from_millis(100);

/// Exclusive operating-system lock on one cache entry. Dropping the guard releases it, and the
/// operating system releases it when the process exits, so a lock is never left stale.
pub(super) struct EntryLock(File);

impl Drop for EntryLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

/// Takes the exclusive lock at `path`, creating the lock file when needed. The lock file stays
/// after release; ownership is the operating-system lock, never the file's existence.
///
/// While another download or extraction holds the lock, this waits for as long as it takes,
/// because the holder may be making progress, but gives up with [`AcquireError::Stopped`] once
/// `stop` reports that the user stopped: a holder that hangs or is suspended must not keep the
/// build from stopping.
pub(super) fn lock_entry(path: &Path, stop: &StopSignal) -> Result<EntryLock, AcquireError> {
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
    loop {
        match file.try_lock_exclusive() {
            Ok(true) => return Ok(EntryLock(file)),
            Ok(false) => {}
            Err(source) => {
                return Err(AcquireError::Io {
                    action: "acquire digest lock",
                    path: path.to_path_buf(),
                    source,
                })
            }
        }
        if stop.requested() {
            return Err(AcquireError::Stopped {
                lock: path.to_path_buf(),
            });
        }
        std::thread::sleep(WAIT_INTERVAL);
    }
}
