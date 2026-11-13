//! Bounded, fail-closed verification of run postconditions.

use std::fs::{self, File, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::manifest::Postcondition;

/// Largest text file a run postcondition may inspect.
pub const MAX_TEXT_FILE_MARKER_BYTES: u64 = 64 * 1024 * 1024;

/// Returns whether `path` is a canonical forward-slash path below a staged root.
pub(crate) fn is_safe_normalized_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.ends_with('/')
        && !path.contains(['\\', ':'])
        && !path.contains(['<', '>', '"', '|', '?', '*'])
        && !path.chars().any(char::is_control)
        && path.split('/').all(is_safe_portable_component)
}

fn is_safe_portable_component(component: &str) -> bool {
    if component.is_empty() || matches!(component, "." | "..") || component.ends_with(['.', ' ']) {
        return false;
    }

    let stem = component
        .split_once('.')
        .map_or(component, |(stem, _)| stem)
        .to_ascii_lowercase();
    !matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        && !stem.strip_prefix("com").is_some_and(|number| {
            matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
        })
        && !stem.strip_prefix("lpt").is_some_and(|number| {
            matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
        })
}

/// A run postcondition could not be proved from safe, bounded target bytes.
#[derive(Debug, Error)]
pub enum PostconditionError {
    /// The frozen specification is unsafe or outside its supported bounds.
    #[error("unsafe text-marker postcondition for {path:?}: {reason}")]
    InvalidSpecification { path: String, reason: &'static str },
    /// A filesystem operation needed for proof failed.
    #[error("could not inspect postcondition path {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The staged root or selected target is not the required direct file kind.
    #[error("postcondition path {path:?} must be a regular file: {reason}")]
    NotRegularFile { path: PathBuf, reason: &'static str },
    /// Canonical resolution left the staged target.
    #[error("postcondition path {path:?} escapes staged target {root:?}")]
    EscapesTarget { path: PathBuf, root: PathBuf },
    /// A relative directory ancestor could redirect traversal.
    #[error(
        "postcondition path {path:?} has unsafe intermediate ancestor {ancestor:?}: expected a direct non-symlink directory"
    )]
    UnsafeAncestor { path: PathBuf, ancestor: PathBuf },
    /// The file exceeded its frozen byte budget.
    #[error("postcondition file {path:?} exceeds max_bytes {max_bytes}")]
    TooLarge { path: PathBuf, max_bytes: u64 },
    /// A required exact marker was absent.
    #[error("postcondition file {path:?} is missing required marker {marker:?}")]
    MissingRequired { path: PathBuf, marker: String },
    /// A forbidden exact marker was present.
    #[error("postcondition file {path:?} contains forbidden marker {marker:?}")]
    ForbiddenPresent { path: PathBuf, marker: String },
}

/// Verifies every frozen postcondition against one staged game root.
///
/// Marker matching is byte-exact and case-sensitive; file contents need not be UTF-8.
/// Every observed ancestor is checked without following its final link, but `std` does not expose
/// a portable handle-relative no-follow traversal. A hostile actor that can replace path components
/// concurrently could therefore race the check and open; campaign locking prevents installer-owned
/// concurrency, not arbitrary external filesystem mutation.
pub fn verify(root: &Path, postconditions: &[Postcondition]) -> Result<(), PostconditionError> {
    if postconditions.is_empty() {
        return Ok(());
    }

    let root_metadata = metadata_without_link(root)?;
    if !root_metadata.is_dir() || crate::reparse::metadata_is_link(root, &root_metadata) {
        return Err(PostconditionError::NotRegularFile {
            path: root.to_path_buf(),
            reason: "staged target must be a direct non-symlink directory",
        });
    }
    let canonical_root =
        crate::reparse::canonicalize_flexible(root).map_err(|source| PostconditionError::Io {
            path: root.to_path_buf(),
            source,
        })?;

    for postcondition in postconditions {
        verify_one(&canonical_root, postcondition)?;
    }
    Ok(())
}

fn verify_one(
    canonical_root: &Path,
    postcondition: &Postcondition,
) -> Result<(), PostconditionError> {
    match postcondition {
        Postcondition::TextFileMarkers {
            path,
            required,
            forbidden,
            max_bytes,
        } => verify_text_file_markers(canonical_root, path, required, forbidden, *max_bytes),
    }
}

fn verify_text_file_markers(
    canonical_root: &Path,
    relative_path: &str,
    required: &[String],
    forbidden: &[String],
    max_bytes: u64,
) -> Result<(), PostconditionError> {
    validate_specification(relative_path, required, forbidden, max_bytes)?;
    let candidate = resolve_candidate_path(canonical_root, relative_path)?;
    let candidate_metadata = metadata_without_link(&candidate)?;
    if !candidate_metadata.is_file()
        || crate::reparse::metadata_is_link(&candidate, &candidate_metadata)
    {
        return Err(PostconditionError::NotRegularFile {
            path: candidate,
            reason: "selected path is not a direct non-symlink file",
        });
    }

    let canonical_path = crate::reparse::canonicalize_flexible(&candidate).map_err(|source| {
        PostconditionError::Io {
            path: candidate.clone(),
            source,
        }
    })?;
    if !crate::reparse::windows_path_starts_with(&canonical_path, canonical_root) {
        return Err(PostconditionError::EscapesTarget {
            path: candidate,
            root: canonical_root.to_path_buf(),
        });
    }
    let resolved_relative = candidate
        .strip_prefix(canonical_root)
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| relative_path.to_owned());
    verify_direct_ancestors(canonical_root, &resolved_relative, &candidate)?;

    let file = File::open(&canonical_path).map_err(|source| PostconditionError::Io {
        path: canonical_path.clone(),
        source,
    })?;
    let metadata = file.metadata().map_err(|source| PostconditionError::Io {
        path: canonical_path.clone(),
        source,
    })?;
    if !metadata.is_file() {
        return Err(PostconditionError::NotRegularFile {
            path: canonical_path,
            reason: "opened target is not a regular file",
        });
    }
    if metadata.len() > max_bytes {
        return Err(PostconditionError::TooLarge {
            path: canonical_path,
            max_bytes,
        });
    }

    let read_limit =
        max_bytes
            .checked_add(1)
            .ok_or_else(|| PostconditionError::InvalidSpecification {
                path: relative_path.to_owned(),
                reason: "max_bytes cannot be incremented safely",
            })?;
    let mut bytes = Vec::new();
    file.take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|source| PostconditionError::Io {
            path: canonical_path.clone(),
            source,
        })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(PostconditionError::TooLarge {
            path: canonical_path,
            max_bytes,
        });
    }

    for marker in required {
        if !contains_bytes(&bytes, marker.as_bytes()) {
            return Err(PostconditionError::MissingRequired {
                path: canonical_path,
                marker: marker.clone(),
            });
        }
    }
    for marker in forbidden {
        if contains_bytes(&bytes, marker.as_bytes()) {
            return Err(PostconditionError::ForbiddenPresent {
                path: canonical_path,
                marker: marker.clone(),
            });
        }
    }
    Ok(())
}

fn verify_direct_ancestors(
    canonical_root: &Path,
    relative_path: &str,
    candidate: &Path,
) -> Result<(), PostconditionError> {
    let mut ancestor = canonical_root.to_path_buf();
    let mut components = relative_path.split('/').peekable();
    while let Some(component) = components.next() {
        if components.peek().is_none() {
            break;
        }
        ancestor.push(component);
        let metadata = metadata_without_link(&ancestor)?;
        if !metadata.is_dir() || crate::reparse::metadata_is_link(&ancestor, &metadata) {
            return Err(PostconditionError::UnsafeAncestor {
                path: candidate.to_path_buf(),
                ancestor,
            });
        }
    }
    Ok(())
}

fn validate_specification(
    path: &str,
    required: &[String],
    forbidden: &[String],
    max_bytes: u64,
) -> Result<(), PostconditionError> {
    if !is_safe_normalized_relative_path(path) {
        return Err(PostconditionError::InvalidSpecification {
            path: path.to_owned(),
            reason: "path is not normalized and target-relative",
        });
    }
    if required.is_empty() && forbidden.is_empty() {
        return Err(PostconditionError::InvalidSpecification {
            path: path.to_owned(),
            reason: "at least one required or forbidden marker is required",
        });
    }
    if required.iter().chain(forbidden).any(String::is_empty) {
        return Err(PostconditionError::InvalidSpecification {
            path: path.to_owned(),
            reason: "markers cannot be empty",
        });
    }
    if max_bytes == 0 || max_bytes > MAX_TEXT_FILE_MARKER_BYTES {
        return Err(PostconditionError::InvalidSpecification {
            path: path.to_owned(),
            reason: "max_bytes is outside the supported bounds",
        });
    }
    if required
        .iter()
        .any(|marker| marker_len_exceeds(marker, max_bytes))
    {
        return Err(PostconditionError::InvalidSpecification {
            path: path.to_owned(),
            reason: "a required marker is larger than max_bytes",
        });
    }
    if required.iter().any(|marker| forbidden.contains(marker)) {
        return Err(PostconditionError::InvalidSpecification {
            path: path.to_owned(),
            reason: "the same marker cannot be both required and forbidden",
        });
    }
    Ok(())
}

pub(crate) fn marker_len_exceeds(marker: &str, max_bytes: u64) -> bool {
    u64::try_from(marker.len()).map_or(true, |length| length > max_bytes)
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn resolve_candidate_path(
    canonical_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, PostconditionError> {
    let exact = canonical_root.join(relative_path);
    #[cfg(windows)]
    {
        let _ = metadata_without_link(&exact)?;
        Ok(exact)
    }
    #[cfg(not(windows))]
    {
        match metadata_without_link(&exact) {
            Ok(_) => return Ok(exact),
            Err(PostconditionError::Io { ref source, .. })
                if source.kind() == io::ErrorKind::NotFound => {}
            Err(other) => return Err(other),
        }
        let mut current = canonical_root.to_path_buf();
        let mut components = relative_path.split('/').peekable();
        while let Some(part) = components.next() {
            let is_leaf = components.peek().is_none();
            let direct = current.join(part);
            let next_path = match fs::symlink_metadata(&direct) {
                Ok(meta) => {
                    if !is_leaf
                        && (!meta.is_dir() || crate::reparse::metadata_is_link(&direct, &meta))
                    {
                        return Err(PostconditionError::UnsafeAncestor {
                            path: exact,
                            ancestor: direct,
                        });
                    }
                    direct
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    let mut matched: Option<(PathBuf, Metadata)> = None;
                    for entry in
                        fs::read_dir(&current).map_err(|source| PostconditionError::Io {
                            path: current.clone(),
                            source,
                        })?
                    {
                        let entry = entry.map_err(|source| PostconditionError::Io {
                            path: current.clone(),
                            source,
                        })?;
                        if entry
                            .file_name()
                            .to_str()
                            .is_some_and(|name| name.eq_ignore_ascii_case(part))
                        {
                            let entry_path = entry.path();
                            let entry_meta = metadata_without_link(&entry_path)?;
                            if matched.is_some() {
                                return Err(PostconditionError::NotRegularFile {
                                    path: exact,
                                    reason: "ambiguous case-colliding postcondition path",
                                });
                            }
                            matched = Some((entry_path, entry_meta));
                        }
                    }
                    let Some((found_path, found_meta)) = matched else {
                        return Err(PostconditionError::Io {
                            path: direct,
                            source: io::Error::from(io::ErrorKind::NotFound),
                        });
                    };
                    if !is_leaf
                        && (!found_meta.is_dir()
                            || crate::reparse::metadata_is_link(&found_path, &found_meta))
                    {
                        return Err(PostconditionError::UnsafeAncestor {
                            path: exact,
                            ancestor: found_path,
                        });
                    }
                    found_path
                }
                Err(source) => {
                    return Err(PostconditionError::Io {
                        path: direct,
                        source,
                    });
                }
            };
            current = next_path;
        }
        Ok(current)
    }
}

fn metadata_without_link(path: &Path) -> Result<Metadata, PostconditionError> {
    fs::symlink_metadata(path).map_err(|source| PostconditionError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_candidate_path_handles_mixed_case_segments() {
        let temp = tempfile::tempdir().unwrap();
        let root = crate::reparse::canonicalize_flexible(temp.path()).unwrap();
        fs::create_dir_all(root.join("Override")).unwrap();
        fs::write(root.join("Override").join("KITLIST.2DA"), b"test").unwrap();

        let resolved = resolve_candidate_path(&root, "override/kitlist.2da").unwrap();
        assert_eq!(fs::read(resolved).unwrap(), b"test");
    }
}
