//! Recovery of derived cache entries that no longer validate.
//!
//! Download and extraction cache entries are rebuilt from pinned, hash-verified sources, so a
//! damaged entry (for example after a partial manual deletion or an antivirus quarantine) is
//! never used and never blocks a build. It is first moved out of its published path with one
//! atomic rename, so a removal that fails part-way can never leave a damaged published entry
//! behind, and then removed on a best-effort basis.
//!
//! Callers hold the entry's lock from the validation that proved the damage until the
//! replacement is published. A request therefore never sets aside a replacement that another
//! request published after this one validated.

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path};
use std::sync::atomic::{AtomicU64, Ordering};

use super::AcquireError;

/// Directory beneath a cache root that receives entries set aside for removal.
const DAMAGED_DIRECTORY: &str = "damaged";
static DAMAGED_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A damaged derived cache entry that was set aside and rebuilt from its verified source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheRepair {
    /// Validation failure that made the previous entry unusable, relative to the entry.
    pub reason: String,
}

/// Whether anything, including a dangling link, exists at `path`. Only a missing path is
/// absence: any other failure to inspect it is an error, never a reason to rebuild.
pub(super) fn entry_is_present(path: &Path) -> Result<bool, AcquireError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(false),
        Err(source) => Err(AcquireError::Io {
            action: "inspect cache entry",
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Validation failures that prove a published entry is damaged: a definite content or format
/// failure, or a path of the entry that no longer exists. `owned` lists the files or
/// directories that make up the entry. Any other failure, such as a file another program holds
/// open or locks, or a permission or device error, keeps failing the step so that nothing is
/// discarded while it may still be usable.
///
/// The validators report content and format failures without the cache location, and this
/// shows other paths relative to their entry, so a reason never reveals where the cache is.
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
        } if source.kind() == ErrorKind::NotFound => {
            let shown = display_within(path, owned)?;
            Some(format!("could not {action} `{shown}`: {source}"))
        }
        _ => None,
    }
}

/// Moves `entry` (a file, directory or link) beneath `<cache_root>/damaged` and removes it
/// there on a best-effort basis. An entry that no longer exists needs no setting aside.
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
        if entry_is_present(&destination)? {
            continue;
        }
        match fs::rename(entry, &destination) {
            Ok(()) => {
                remove_best_effort(&destination);
                return Ok(());
            }
            Err(source) => {
                return match entry_is_present(entry) {
                    Ok(false) => Ok(()),
                    _ => Err(AcquireError::Io {
                        action: "set aside damaged cache entry",
                        path: entry.to_path_buf(),
                        source,
                    }),
                }
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

/// The failing path relative to the entry that owns it, with `/` separators and without the
/// user's cache location. `None` when no listed entry owns the path.
fn display_within(path: &Path, owned: &[&Path]) -> Option<String> {
    owned.iter().find_map(|owner| {
        let relative = path.strip_prefix(owner).ok()?;
        if relative.as_os_str().is_empty() {
            return Some(path.file_name()?.to_string_lossy().into_owned());
        }
        let parts = relative
            .components()
            .map(|component| match component {
                Component::Normal(part) => Some(part.to_string_lossy()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        Some(parts.join("/"))
    })
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::{Path, PathBuf};

    use super::damaged_entry_reason;
    use crate::acquire::AcquireError;

    fn io_error(kind: io::ErrorKind, path: PathBuf) -> AcquireError {
        AcquireError::Io {
            action: "read file for hashing",
            path,
            source: io::Error::from(kind),
        }
    }

    #[test]
    fn only_definite_damage_inside_the_entry_proves_it_damaged() {
        let cache = Path::new("users-cache-location");
        let entry = cache.join("entry");
        let owned = [entry.as_path()];
        let inside = entry.join("mod").join("setup.tra");

        let missing =
            damaged_entry_reason(&io_error(io::ErrorKind::NotFound, inside.clone()), &owned)
                .expect("a missing file of the entry is damage");
        assert!(
            missing.starts_with("could not read file for hashing `mod/setup.tra`: "),
            "{missing}"
        );
        assert!(!missing.contains("users-cache-location"), "{missing}");
        let unreadable = damaged_entry_reason(
            &AcquireError::Metadata {
                path: entry.join("record.json"),
                message: "EOF while parsing".to_owned(),
            },
            &owned,
        );
        assert_eq!(
            unreadable.as_deref(),
            Some("unreadable cache record `record.json`: EOF while parsing")
        );
        let corrupt = damaged_entry_reason(
            &AcquireError::CorruptCache {
                digest: "ab".repeat(32),
                message: "recorded extracted file `mod/setup.tra` is missing".to_owned(),
            },
            &owned,
        );
        assert!(corrupt.is_some_and(|reason| reason.contains("mod/setup.tra")));

        for kind in [
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::Interrupted,
            io::ErrorKind::Other,
        ] {
            assert_eq!(
                damaged_entry_reason(&io_error(kind, inside.clone()), &owned),
                None,
                "{kind:?} must not prove damage"
            );
        }
        let elsewhere = cache.join("other").join("setup.tra");
        assert_eq!(
            damaged_entry_reason(
                &io_error(io::ErrorKind::NotFound, elsewhere.clone()),
                &owned
            ),
            None
        );
        assert_eq!(
            damaged_entry_reason(
                &AcquireError::Metadata {
                    path: elsewhere,
                    message: "EOF while parsing".to_owned(),
                },
                &owned,
            ),
            None
        );
        assert_eq!(
            damaged_entry_reason(
                &AcquireError::LengthMismatch {
                    expected: 2,
                    actual: 1,
                },
                &owned,
            ),
            None
        );
    }
}
