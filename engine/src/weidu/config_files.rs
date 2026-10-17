//! Recipe-authored configuration files placed in a staged mod folder before its run.
//!
//! A file is created only while absent. An existing file is accepted only when it already holds
//! the authored bytes, so a retried attempt is idempotent and nothing that a mod archive or a
//! person put there is ever overwritten.
//!
//! The folders from the target root down to the mod folder are opened as a [`HeldFolder`]: one at
//! a time, each relative to the folder above it and without following a link, and held without
//! delete sharing until the file is in place. The file is inspected, created and renamed through
//! those handles, so no path is resolved again once a folder was checked, and no folder can be
//! swapped for a link in between.

use std::io::Read;
use std::path::Path;

use serde::Serialize;

use crate::digest::sha256_bytes;
use crate::held_folder::{EntryError, EntryErrorKind, HeldFolder};
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
    let (folders, name) = file
        .path
        .rsplit_once('/')
        .ok_or_else(|| format!("configuration file {:?} has no mod folder", file.path))?;

    // Only add files to the published folder: never create folders or follow a link out of the
    // staged game.
    let folder = HeldFolder::open(target_root, folders.split('/')).map_err(folder_error)?;
    let path = folder.path().join(name);
    let content = file.content.as_bytes();
    let sha256 = sha256_bytes(content);
    let evidence = |created| ConfigFileEvidence {
        component,
        path: file.path.clone(),
        sha256: sha256.clone(),
        created,
    };
    match folder.open_file(name) {
        Ok((existing, metadata)) => {
            // Compare sizes first, so a large unrelated file is never read in.
            let expected_len = u64::try_from(content.len()).unwrap_or(u64::MAX);
            if metadata.len() != expected_len {
                return Err(format!(
                    "{} already exists with different content ({} bytes instead of {expected_len}); it was left unchanged",
                    path.display(),
                    metadata.len()
                ));
            }
            let mut bytes = Vec::new();
            (&existing)
                .take(expected_len.saturating_add(1))
                .read_to_end(&mut bytes)
                .map_err(|error| format!("read configuration file {}: {error}", path.display()))?;
            if bytes != content {
                return Err(format!(
                    "{} already exists with different content (SHA-256 {}); it was left unchanged",
                    path.display(),
                    sha256_bytes(&bytes)
                ));
            }
            Ok(evidence(false))
        }
        Err(EntryError {
            kind: EntryErrorKind::Missing,
            ..
        }) => {
            folder
                .create_file(name, content)
                .map_err(|error| format!("place configuration file {}: {error}", path.display()))?;
            Ok(evidence(true))
        }
        Err(EntryError {
            kind: EntryErrorKind::NotDirect,
            ..
        }) => Err(format!(
            "configuration file path is not a direct file: {}",
            path.display()
        )),
        Err(EntryError {
            kind: EntryErrorKind::Io(error),
            ..
        }) => Err(format!(
            "inspect configuration file {}: {error}",
            path.display()
        )),
    }
}

fn folder_error(error: EntryError) -> String {
    let path = error.path.display();
    match error.kind {
        EntryErrorKind::Missing => format!(
            "folder {path} is missing; a configuration file goes only into its mod's published folder"
        ),
        EntryErrorKind::NotDirect => {
            format!("configuration file ancestor is not a direct directory: {path}")
        }
        EntryErrorKind::Io(error) => format!("inspect {path}: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::held_folder::tests::entries;
    #[cfg(windows)]
    use crate::held_folder::tests::junction;

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
        assert_eq!(entries(&root.path().join("cdtweaks")), ["cdtweaks.txt"]);
        let expected = |created| ConfigFileEvidence {
            component: 3347,
            path: "cdtweaks/cdtweaks.txt".into(),
            sha256: sha256_bytes(CONTENT.as_bytes()),
            created,
        };
        assert_eq!(first, vec![expected(true)]);
        assert_eq!(second, vec![expected(false)]);
    }

    /// Windows names ignore case, so the recipe's spelling finds the published folder.
    #[cfg(windows)]
    #[test]
    fn finds_the_published_folder_whatever_the_case_of_its_name() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("CDTweaks")).unwrap();

        place(root.path(), &[(3347, &config("cdtweaks/cdtweaks.txt"))]).unwrap();

        assert_eq!(
            fs::read_to_string(root.path().join("CDTweaks/cdtweaks.txt")).unwrap(),
            CONTENT
        );
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
            "cdtweaks/cdtweaks.txt:stream",
        ] {
            assert!(
                place(root.path(), &[(3347, &config(path))]).is_err(),
                "{path} was accepted"
            );
        }
        assert_eq!(entries(&root.path().join("cdtweaks")), Vec::<String>::new());
    }

    #[cfg(windows)]
    #[test]
    fn never_writes_through_a_junction_in_the_staged_game() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("game");
        let outside = root.path().join("outside");
        fs::create_dir_all(outside.join("cdtweaks")).unwrap();
        fs::create_dir_all(game.join("real")).unwrap();
        // The mod folder itself is a junction, and a folder above a real mod folder is one.
        junction(&game.join("cdtweaks"), &outside.join("cdtweaks"));
        junction(&game.join("linked"), &outside);

        for path in ["cdtweaks/cdtweaks.txt", "linked/cdtweaks/cdtweaks.txt"] {
            let error = place(&game, &[(3347, &config(path))]).unwrap_err();
            assert!(error.contains("not a direct directory"), "{path}: {error}");
        }
        // The target root is a junction.
        junction(&root.path().join("linked-game"), &outside);
        let error = place(
            &root.path().join("linked-game"),
            &[(3347, &config("cdtweaks/cdtweaks.txt"))],
        )
        .unwrap_err();
        assert!(error.contains("not a direct directory"), "{error}");
        // The configuration file's own name is a junction.
        junction(
            &game.join("real").join("cdtweaks.txt"),
            &outside.join("cdtweaks"),
        );
        let error = place(&game, &[(3347, &config("real/cdtweaks.txt"))]).unwrap_err();
        assert!(error.contains("not a direct file"), "{error}");

        assert_eq!(entries(&outside.join("cdtweaks")), Vec::<String>::new());
    }

    /// Nothing above the target root is inspected, so a link there, such as a cloud-files folder
    /// or a volume mounted in a folder, is no reason to refuse; the file lands in the real folder.
    #[cfg(windows)]
    #[test]
    fn places_below_a_link_above_the_target_root() {
        let root = tempfile::tempdir().unwrap();
        let real = root.path().join("real");
        fs::create_dir_all(real.join("game/cdtweaks")).unwrap();
        let link = root.path().join("link");
        junction(&link, &real);

        place(
            &link.join("game"),
            &[(3347, &config("cdtweaks/cdtweaks.txt"))],
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(real.join("game/cdtweaks/cdtweaks.txt")).unwrap(),
            CONTENT
        );
    }
}
