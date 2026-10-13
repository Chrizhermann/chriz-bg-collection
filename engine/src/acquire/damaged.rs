//! Recovery of derived cache entries that no longer validate.
//!
//! Download and extraction cache entries are rebuilt from pinned, hash-verified sources, so a
//! damaged entry (for example after a partial manual deletion or an antivirus quarantine) is
//! never used and never blocks a build. It is first moved out of its published path with one
//! atomic rename, so a removal that fails part-way can never leave a damaged published entry
//! behind, and then removed on a best-effort basis.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use super::AcquireError;

/// Directory beneath a cache root that receives entries set aside for removal.
const DAMAGED_DIRECTORY: &str = "damaged";
static DAMAGED_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A damaged derived cache entry that was set aside and rebuilt from its verified source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheRepair {
    /// Validation failure that made the previous entry unusable.
    pub reason: String,
}

/// Whether anything, including a dangling link, exists at `path`.
pub(super) fn path_is_present(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// Validation failures that prove a published entry is damaged rather than temporarily
/// unreadable. `owned` lists the files or directories that make up the entry. Other failures,
/// such as a file locked by another program, keep failing the step so that nothing is
/// discarded while it may still be in use.
pub(super) fn damaged_entry_reason(error: &AcquireError, owned: &[&Path]) -> Option<String> {
    match error {
        AcquireError::CorruptCache { .. } => Some(error.to_string()),
        AcquireError::Metadata { path, message } => {
            let shown = display_within(path, owned)?;
            Some(format!("unreadable cache record `{shown}`: {message}"))
        }
        AcquireError::Io {
            action,
            path,
            source,
        } if source.kind() == std::io::ErrorKind::NotFound => {
            let shown = display_within(path, owned)?;
            Some(format!("could not {action} `{shown}`: {source}"))
        }
        _ => None,
    }
}

/// Moves `entry` (a file, directory or link) beneath `<cache_root>/damaged` and removes it
/// there on a best-effort basis. An entry that another process already set aside is accepted.
pub(super) fn set_aside(cache_root: &Path, entry: &Path, digest: &str) -> Result<(), AcquireError> {
    let damaged = cache_root.join(DAMAGED_DIRECTORY);
    fs::create_dir_all(&damaged).map_err(|source| AcquireError::Io {
        action: "create damaged cache directory",
        path: damaged.clone(),
        source,
    })?;
    for _ in 0..32 {
        let sequence = DAMAGED_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let destination = damaged.join(format!(
            ".{digest}.{}.{}.damaged",
            std::process::id(),
            sequence
        ));
        if path_is_present(&destination) {
            continue;
        }
        match fs::rename(entry, &destination) {
            Ok(()) => {
                remove_best_effort(&destination);
                return Ok(());
            }
            Err(_source) if !path_is_present(entry) => return Ok(()),
            Err(source) => {
                return Err(AcquireError::Io {
                    action: "set aside damaged cache entry",
                    path: entry.to_path_buf(),
                    source,
                })
            }
        }
    }
    Err(AcquireError::InvalidRequest(
        "could not allocate a unique name for a damaged cache entry".to_owned(),
    ))
}

/// `remove_dir_all` removes a link itself instead of following it; a regular file needs
/// `remove_file`. A leftover is harmless because it is outside every published path.
fn remove_best_effort(path: &Path) {
    let _ = fs::remove_dir_all(path).or_else(|_| fs::remove_file(path));
}

/// The failing path relative to the entry that owns it, without the user's cache location.
/// `None` when no listed entry owns the path.
fn display_within(path: &Path, owned: &[&Path]) -> Option<String> {
    owned.iter().find_map(|owner| {
        let relative = path.strip_prefix(owner).ok()?;
        if relative.as_os_str().is_empty() {
            Some(path.file_name()?.to_string_lossy().into_owned())
        } else {
            Some(relative.display().to_string())
        }
    })
}
