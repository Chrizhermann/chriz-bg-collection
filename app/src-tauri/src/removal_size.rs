//! Best-effort logical file size for the removal dialog, never removal authority.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

const SCAN_BUDGET: Duration = Duration::from_secs(5);
const ENTRY_LIMIT: usize = 500_000;

/// Run only on a background worker, without the bridge runtime lock. File contents
/// are never read. Any incomplete scan returns unknown, not a misleading partial sum.
/// The time budget is checked between filesystem calls; it cannot interrupt OS I/O.
pub(crate) fn estimate(root: &Path) -> Option<u64> {
    estimate_with_limits(root, Instant::now() + SCAN_BUDGET, ENTRY_LIMIT)
}

fn estimate_with_limits(root: &Path, deadline: Instant, mut entries_left: usize) -> Option<u64> {
    let mut total = 0_u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        if Instant::now() >= deadline {
            return None;
        }
        // Recheck queued directories before opening them, including the root.
        let metadata = fs::symlink_metadata(&directory).ok()?;
        if linked(&metadata) || !metadata.is_dir() {
            return None;
        }
        for entry in fs::read_dir(&directory).ok()? {
            if Instant::now() >= deadline {
                return None;
            }
            entries_left = entries_left.checked_sub(1)?;
            let entry = entry.ok()?;
            // DirEntry metadata does not follow links and reuses enumeration metadata
            // on Windows, avoiding a separate path lookup for every game resource.
            let metadata = entry.metadata().ok()?;
            if linked(&metadata) {
                return None;
            }
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                total = total.checked_add(metadata.len())?;
            } else {
                return None;
            }
        }
    }
    (Instant::now() < deadline).then_some(total)
}

fn linked(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_all_nested_files_but_nothing_outside_the_folder() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        fs::create_dir_all(root.join("game/save")).unwrap();
        fs::create_dir(root.join(".chriz")).unwrap();
        fs::write(root.join("game/game.exe"), [0; 100]).unwrap();
        fs::write(root.join("game/save/user-save"), [0; 23]).unwrap();
        fs::write(root.join(".chriz/record"), [0; 7]).unwrap();
        fs::write(temp.path().join("outside-save"), [0; 500]).unwrap();
        assert_eq!(estimate(&root), Some(130));
    }

    #[test]
    fn empty_is_zero_but_missing_or_non_directory_is_unknown() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(estimate(temp.path()), Some(0));
        assert_eq!(estimate(&temp.path().join("missing")), None);
        fs::write(temp.path().join("file"), [0; 1]).unwrap();
        assert_eq!(estimate(&temp.path().join("file")), None);
    }

    #[test]
    fn exhausted_budgets_return_unknown_instead_of_partial_bytes() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("one"), [0; 1]).unwrap();
        fs::write(temp.path().join("two"), [0; 2]).unwrap();
        assert_eq!(estimate_with_limits(temp.path(), Instant::now(), 10), None);
        assert_eq!(
            estimate_with_limits(temp.path(), Instant::now() + SCAN_BUDGET, 1),
            None
        );
        assert_eq!(
            estimate_with_limits(temp.path(), Instant::now() + SCAN_BUDGET, 2),
            Some(3)
        );
    }

    #[test]
    fn linked_root_and_nested_link_return_unknown_without_following_target() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        let outside = temp.path().join("outside");
        let link = root.join("link");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("private-save"), [0; 500]).unwrap();
        #[cfg(windows)]
        {
            let output = std::process::Command::new("cmd.exe")
                .args(["/c", "mklink", "/J"])
                .arg(&link)
                .arg(&outside)
                .output()
                .unwrap();
            assert!(output.status.success(), "{:?}", output);
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        assert_eq!(estimate(&link), None);
        assert_eq!(estimate(&root), None);
        assert_eq!(
            fs::metadata(outside.join("private-save")).unwrap().len(),
            500
        );
    }
}
