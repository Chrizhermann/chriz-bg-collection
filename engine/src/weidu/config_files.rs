//! Recipe-authored configuration files placed in a staged mod folder before its run.
//!
//! A file is created only while absent. An existing file is accepted only when it already holds
//! the authored bytes, so a retried attempt is idempotent and nothing that a mod archive or a
//! person put there is ever overwritten.

use std::fs::{self, Metadata};
use std::io::{self, ErrorKind, Read, Write};
use std::path::Path;

use serde::Serialize;

use crate::digest::sha256_bytes;
use crate::manifest::{ConfigFile, MAX_CONFIG_FILE_BYTES};
use crate::postcondition::is_safe_normalized_relative_path;

/// Written into the evidence directory of every attempt that places configuration files.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ConfigFileEvidence {
    component: u32,
    path: String,
    sha256: String,
    created: bool,
}

/// Place the configuration files of the components one attempt installs, in recipe order.
pub(crate) fn place(
    target_root: &Path,
    files: &[(u32, &ConfigFile)],
) -> Result<Vec<ConfigFileEvidence>, String> {
    files
        .iter()
        .map(|(component, file)| place_one(target_root, *component, file))
        .collect()
}

fn place_one(
    target_root: &Path,
    component: u32,
    file: &ConfigFile,
) -> Result<ConfigFileEvidence, String> {
    // Recipe validation enforces these too; the frozen plan is checked again before writing.
    if !is_safe_normalized_relative_path(&file.path)
        || file.content.is_empty()
        || file.content.len() > MAX_CONFIG_FILE_BYTES
    {
        return Err(format!(
            "configuration file {:?} is not a valid recipe entry",
            file.path
        ));
    }
    if !target_root.is_absolute() {
        return Err("configuration file target must be absolute".into());
    }
    let (parent_segments, name) = file
        .path
        .rsplit_once('/')
        .ok_or_else(|| format!("configuration file {:?} has no mod folder", file.path))?;
    let mut parent = target_root.to_path_buf();
    parent.extend(parent_segments.split('/'));

    // Hold the mod folder before checking the path to it. While the pin lasts, Windows refuses
    // to rename or delete that folder or any folder above it, so no checked folder can be
    // swapped for a link before the file is in place. Only add files to the published folder:
    // never create folders or follow a link out of the staged game.
    let _pin = pin_directory(&parent)?;
    let mut ancestor = target_root.to_path_buf();
    require_direct_directory(&ancestor)?;
    for segment in parent_segments.split('/') {
        ancestor.push(segment);
        require_direct_directory(&ancestor)?;
    }

    let path = parent.join(name);
    let content = file.content.as_bytes();
    let sha256 = sha256_bytes(content);
    let evidence = |created| ConfigFileEvidence {
        component,
        path: file.path.clone(),
        sha256: sha256.clone(),
        created,
    };
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if !metadata.is_file() || is_link_or_reparse(&metadata) {
                return Err(format!(
                    "configuration file path is not a direct file: {}",
                    path.display()
                ));
            }
            // Compare sizes first, so a large unrelated file is never read into memory.
            let expected_len = u64::try_from(content.len()).unwrap_or(u64::MAX);
            if metadata.len() != expected_len {
                return Err(format!(
                    "{} already exists with different content ({} bytes instead of {expected_len}); it was left unchanged",
                    path.display(),
                    metadata.len()
                ));
            }
            let existing = read_at_most(&path, expected_len.saturating_add(1))
                .map_err(|error| format!("read configuration file {}: {error}", path.display()))?;
            if existing != content {
                return Err(format!(
                    "{} already exists with different content (SHA-256 {}); it was left unchanged",
                    path.display(),
                    sha256_bytes(&existing)
                ));
            }
            Ok(evidence(false))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            write_new(&parent, &path, content)?;
            Ok(evidence(true))
        }
        Err(error) => Err(format!(
            "inspect configuration file {}: {error}",
            path.display()
        )),
    }
}

fn read_at_most(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(limit).read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn write_new(parent: &Path, path: &Path, content: &[u8]) -> Result<(), String> {
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| format!("create a temporary file in {}: {error}", parent.display()))?;
    temporary
        .write_all(content)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| format!("write configuration file {}: {error}", path.display()))?;
    // Never replace a file that appeared since the check above.
    temporary.persist_noclobber(path).map_err(|error| {
        format!(
            "place configuration file {}: {}",
            path.display(),
            error.error
        )
    })?;
    Ok(())
}

fn require_direct_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| folder_error(path, &error))?;
    if !metadata.is_dir() || is_link_or_reparse(&metadata) {
        return Err(not_a_direct_directory(path));
    }
    Ok(())
}

/// Open `path` itself, not a link target, without delete sharing.
#[cfg(windows)]
fn pin_directory(path: &Path) -> Result<fs::File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE,
    };

    let directory = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|error| folder_error(path, &error))?;
    let metadata = directory
        .metadata()
        .map_err(|error| folder_error(path, &error))?;
    if !metadata.is_dir() || is_link_or_reparse(&metadata) {
        return Err(not_a_direct_directory(path));
    }
    Ok(directory)
}

#[cfg(not(windows))]
fn pin_directory(path: &Path) -> Result<(), String> {
    require_direct_directory(path)
}

fn folder_error(path: &Path, error: &io::Error) -> String {
    if error.kind() == ErrorKind::NotFound {
        format!(
            "folder {} is missing; a configuration file goes only into its mod's published folder",
            path.display()
        )
    } else {
        format!("inspect {}: {error}", path.display())
    }
}

fn not_a_direct_directory(path: &Path) -> String {
    format!(
        "configuration file ancestor is not a direct directory: {}",
        path.display()
    )
}

#[cfg(not(windows))]
fn is_link_or_reparse(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
fn is_link_or_reparse(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

    metadata.file_type().is_symlink()
        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENT: &str = "OUTER_SET party_movement_speed_value = 125\n";

    fn config(path: &str) -> ConfigFile {
        ConfigFile {
            path: path.to_owned(),
            content: CONTENT.to_owned(),
        }
    }

    fn staged_mod() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("cdtweaks")).unwrap();
        root
    }

    #[test]
    fn creates_an_absent_file_and_accepts_it_again_on_retry() {
        let root = staged_mod();
        let file = config("cdtweaks/cdtweaks.txt");

        let first = place(root.path(), &[(3347, &file)]).unwrap();
        let second = place(root.path(), &[(3347, &file)]).unwrap();

        assert_eq!(
            fs::read_to_string(root.path().join("cdtweaks/cdtweaks.txt")).unwrap(),
            CONTENT
        );
        let expected = |created| ConfigFileEvidence {
            component: 3347,
            path: "cdtweaks/cdtweaks.txt".into(),
            sha256: sha256_bytes(CONTENT.as_bytes()),
            created,
        };
        assert_eq!(first, vec![expected(true)]);
        assert_eq!(second, vec![expected(false)]);
    }

    #[test]
    fn never_overwrites_a_different_existing_file() {
        let root = staged_mod();
        let existing = root.path().join("cdtweaks/cdtweaks.txt");
        let other = "OUTER_SET party_movement_speed_value = 200\n";
        fs::write(&existing, other).unwrap();

        let error = place(root.path(), &[(3347, &config("cdtweaks/cdtweaks.txt"))]).unwrap_err();

        assert!(
            error.contains("already exists with different content (SHA-256"),
            "{error}"
        );
        assert_eq!(fs::read_to_string(existing).unwrap(), other);
    }

    #[test]
    fn rejects_a_larger_existing_file_by_its_size() {
        let root = staged_mod();
        let existing = root.path().join("cdtweaks/cdtweaks.txt");
        fs::write(&existing, vec![b'x'; 1024 * 1024]).unwrap();

        let error = place(root.path(), &[(3347, &config("cdtweaks/cdtweaks.txt"))]).unwrap_err();

        assert!(error.contains("1048576 bytes instead of 43"), "{error}");
        assert_eq!(fs::metadata(existing).unwrap().len(), 1024 * 1024);
    }

    #[test]
    fn requires_the_published_mod_folder_and_creates_no_folders() {
        let root = tempfile::tempdir().unwrap();

        let error = place(root.path(), &[(3347, &config("cdtweaks/cdtweaks.txt"))]).unwrap_err();

        assert!(error.contains("is missing"), "{error}");
        assert!(!root.path().join("cdtweaks").exists());
    }

    #[test]
    fn rejects_a_directory_where_the_file_belongs() {
        let root = staged_mod();
        fs::create_dir(root.path().join("cdtweaks/cdtweaks.txt")).unwrap();

        let error = place(root.path(), &[(3347, &config("cdtweaks/cdtweaks.txt"))]).unwrap_err();

        assert!(error.contains("not a direct file"), "{error}");
    }

    #[test]
    fn rejects_unsafe_or_folderless_paths_before_touching_the_target() {
        let root = staged_mod();
        for path in [
            "cdtweaks.txt",
            "../cdtweaks.txt",
            "cdtweaks/../x.txt",
            "/cdtweaks/x.txt",
        ] {
            assert!(
                place(root.path(), &[(3347, &config(path))]).is_err(),
                "{path} was accepted"
            );
        }
        assert_eq!(
            fs::read_dir(root.path().join("cdtweaks")).unwrap().count(),
            0
        );
    }

    #[cfg(windows)]
    fn junction(link: &Path, target: &Path) {
        let status = std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(link)
            .arg(target)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "mklink /J failed for {}", link.display());
    }

    #[cfg(windows)]
    #[test]
    fn never_writes_through_a_junction_in_the_mod_path() {
        let root = tempfile::tempdir().unwrap();
        let outside = root.path().join("outside");
        fs::create_dir_all(outside.join("cdtweaks")).unwrap();
        // The mod folder itself is a junction, and a folder above a real mod folder is one.
        junction(&root.path().join("cdtweaks"), &outside.join("cdtweaks"));
        junction(&root.path().join("linked"), &outside);

        for path in ["cdtweaks/cdtweaks.txt", "linked/cdtweaks/cdtweaks.txt"] {
            let error = place(root.path(), &[(3347, &config(path))]).unwrap_err();
            assert!(error.contains("not a direct directory"), "{path}: {error}");
        }
        assert_eq!(fs::read_dir(outside.join("cdtweaks")).unwrap().count(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn a_pinned_mod_folder_and_the_folders_above_it_cannot_be_moved() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("game");
        let folder = game.join("cdtweaks");
        fs::create_dir_all(&folder).unwrap();

        let pin = pin_directory(&folder).unwrap();
        assert!(fs::rename(&folder, game.join("moved")).is_err());
        assert!(fs::rename(&game, root.path().join("moved")).is_err());
        // Adding the configuration file inside the pinned folder still works.
        write_new(&folder, &folder.join("cdtweaks.txt"), CONTENT.as_bytes()).unwrap();
        drop(pin);

        fs::rename(&folder, game.join("moved")).unwrap();
    }
}
