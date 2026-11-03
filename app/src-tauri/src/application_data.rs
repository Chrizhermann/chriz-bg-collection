//! Removal of the app's own data folders for the Windows uninstaller.
//!
//! When "Delete the application data" is checked, Tauri's uninstaller deletes
//! `%APPDATA%\<identifier>` and `%LOCALAPPDATA%\<identifier>` with NSIS `RMDir /r`. That stops at
//! paths longer than 260 characters, such as mods unpacked deep in the download cache, and it
//! follows directory junctions out of these folders. The hook in `windows/hooks.nsh` therefore
//! runs the app with [`REMOVE_APPLICATION_DATA`] first: `remove_dir_all` handles long paths and
//! removes links without following them.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Argument with which the uninstaller asks the app to remove its data, followed by the bundle
/// identifier whose folders the uninstaller would delete.
pub const REMOVE_APPLICATION_DATA: &str = "--remove-application-data";
/// The bundle identifier in `tauri.conf.json`, which names the application data folders.
pub const IDENTIFIER: &str = "dev.chrizhermann.bgcollection";
/// Exit code: no application data folder remains.
pub const REMOVED: i32 = 0;
/// Exit code: some application data could not be removed; it is reported on standard error.
pub const NOT_REMOVED: i32 = 1;
/// Exit code: the request was not exactly the uninstaller's, so nothing was touched.
pub const REFUSED: i32 = 2;

/// A program that has just closed, such as the app's web view, or a virus scanner can hold a
/// file for a moment, so a failed removal is tried again for about five seconds.
const ATTEMPTS: u32 = 10;
const RETRY_DELAY: Duration = Duration::from_millis(500);

/// What the command line asks the app to do.
#[derive(Debug, PartialEq, Eq)]
enum Request {
    /// Start the app normally.
    Start,
    /// Remove the application data folders.
    Remove,
    /// A malformed removal request; nothing is touched.
    Refuse(String),
}

/// Handles an uninstaller request in `arguments`, which include the program, and returns the
/// exit code, or `None` when the app should start normally.
pub fn uninstaller_request(arguments: impl IntoIterator<Item = OsString>) -> Option<i32> {
    let arguments = arguments.into_iter().skip(1).collect::<Vec<_>>();
    match parse(&arguments) {
        Request::Start => None,
        Request::Refuse(reason) => {
            eprintln!("application data was not removed: {reason}");
            Some(REFUSED)
        }
        Request::Remove => {
            let folders = match application_data_folders() {
                Ok(folders) => folders,
                Err(reason) => {
                    eprintln!("application data was not removed: {reason}");
                    return Some(REFUSED);
                }
            };
            match remove_folders(&folders, ATTEMPTS, RETRY_DELAY) {
                Ok(()) => Some(REMOVED),
                Err(failures) => {
                    for (folder, error) in failures {
                        eprintln!("could not remove {}: {error}", folder.display());
                    }
                    Some(NOT_REMOVED)
                }
            }
        }
    }
}

fn parse(arguments: &[OsString]) -> Request {
    if !arguments
        .iter()
        .any(|argument| argument == REMOVE_APPLICATION_DATA)
    {
        return Request::Start;
    }
    // Deletion follows only the exact request of this app's own uninstaller.
    match arguments {
        [request, identifier] if request == REMOVE_APPLICATION_DATA && identifier == IDENTIFIER => {
            Request::Remove
        }
        [request, identifier] if request == REMOVE_APPLICATION_DATA => Request::Refuse(format!(
            "the uninstaller names {identifier:?}, but this app is {IDENTIFIER:?}"
        )),
        _ => Request::Refuse(format!(
            "expected `{REMOVE_APPLICATION_DATA} {IDENTIFIER}` and nothing else"
        )),
    }
}

/// The folders the uninstaller deletes: `<identifier>` in the roaming and local application
/// data folders.
fn application_data_folders() -> Result<Vec<PathBuf>, String> {
    // Development and test builds remove only folders that a test declares, never real app data.
    #[cfg(any(debug_assertions, feature = "test-application-data"))]
    let parents = std::env::split_paths(
        &std::env::var_os("CHRIZ_BG_COLLECTION_TEST_APPLICATION_DATA_PARENTS")
            .ok_or("development and test builds remove only test folders")?,
    )
    .collect::<Vec<_>>();
    #[cfg(not(any(debug_assertions, feature = "test-application-data")))]
    let parents = known_application_data_parents()?;
    parents
        .into_iter()
        .map(|parent| {
            // A relative folder would resolve against the working directory.
            if parent.is_absolute() {
                Ok(parent.join(IDENTIFIER))
            } else {
                Err(format!(
                    "the application data folder {} is not absolute",
                    parent.display()
                ))
            }
        })
        .collect()
}

/// The roaming and local application data folders, which NSIS calls `$APPDATA` and
/// `$LOCALAPPDATA` for the current user.
#[cfg(windows)]
#[cfg_attr(
    any(debug_assertions, feature = "test-application-data"),
    allow(dead_code)
)]
fn known_application_data_parents() -> Result<Vec<PathBuf>, String> {
    use windows::Win32::UI::Shell::{FOLDERID_LocalAppData, FOLDERID_RoamingAppData};

    Ok(vec![
        known_folder(&FOLDERID_RoamingAppData, "roaming application data")?,
        known_folder(&FOLDERID_LocalAppData, "local application data")?,
    ])
}

#[cfg(not(windows))]
#[cfg_attr(
    any(debug_assertions, feature = "test-application-data"),
    allow(dead_code)
)]
fn known_application_data_parents() -> Result<Vec<PathBuf>, String> {
    Err("only the Windows uninstaller removes application data".to_owned())
}

#[cfg(windows)]
#[cfg_attr(
    any(debug_assertions, feature = "test-application-data"),
    allow(dead_code)
)]
fn known_folder(id: &windows::core::GUID, label: &str) -> Result<PathBuf, String> {
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT};

    let raw = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }
        .map_err(|error| format!("could not find the {label} folder: {error}"))?;
    let path = PathBuf::from(OsString::from_wide(unsafe { raw.as_wide() }));
    unsafe { CoTaskMemFree(Some(raw.as_ptr().cast())) };
    Ok(path)
}

/// Removes every folder, deleting links inside without following them. A folder that does not
/// exist counts as removed.
fn remove_folders(
    folders: &[PathBuf],
    attempts: u32,
    retry_delay: Duration,
) -> Result<(), Vec<(PathBuf, io::Error)>> {
    let failures = folders
        .iter()
        .filter_map(|folder| {
            remove_folder(folder, attempts, retry_delay)
                .err()
                .map(|error| (folder.clone(), error))
        })
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

fn remove_folder(folder: &Path, attempts: u32, retry_delay: Duration) -> io::Result<()> {
    let mut attempt = 1;
    loop {
        let error = match std::fs::remove_dir_all(folder) {
            Ok(()) => return Ok(()),
            Err(error) => error,
        };
        if matches!(
            std::fs::symlink_metadata(folder),
            Err(missing) if missing.kind() == io::ErrorKind::NotFound
        ) {
            return Ok(());
        }
        if attempt >= attempts {
            return Err(error);
        }
        attempt += 1;
        std::thread::sleep(retry_delay);
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use super::{parse, remove_folders, Request, IDENTIFIER, REMOVE_APPLICATION_DATA};

    fn arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn only_the_uninstallers_exact_request_removes_application_data() {
        assert_eq!(parse(&[]), Request::Start);
        assert_eq!(parse(&arguments(&["--install-id", "abc"])), Request::Start);
        assert_eq!(
            parse(&arguments(&[REMOVE_APPLICATION_DATA, IDENTIFIER])),
            Request::Remove
        );
        for refused in [
            vec![REMOVE_APPLICATION_DATA],
            vec![REMOVE_APPLICATION_DATA, "dev.example.other"],
            vec![REMOVE_APPLICATION_DATA, ".."],
            vec![REMOVE_APPLICATION_DATA, ""],
            vec![REMOVE_APPLICATION_DATA, IDENTIFIER, "extra"],
            vec![IDENTIFIER, REMOVE_APPLICATION_DATA],
            vec!["--install-id", "abc", REMOVE_APPLICATION_DATA, IDENTIFIER],
        ] {
            assert!(
                matches!(parse(&arguments(&refused)), Request::Refuse(_)),
                "{refused:?}"
            );
        }
    }

    #[test]
    fn the_identifier_is_the_bundle_identifier() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["identifier"], IDENTIFIER);
    }

    /// Release builds for players remove `<identifier>` in the folders NSIS calls `$APPDATA` and
    /// `$LOCALAPPDATA`. Reading them touches nothing.
    #[cfg(windows)]
    #[test]
    fn release_builds_use_the_roaming_and_local_application_data_folders() {
        assert_eq!(
            super::known_application_data_parents().unwrap(),
            [
                PathBuf::from(std::env::var_os("APPDATA").unwrap()),
                PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap()),
            ]
        );
    }

    #[cfg(windows)]
    fn junction(link: &Path, target: &Path) {
        let output = std::process::Command::new("cmd.exe")
            .args(["/c", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }

    /// Creates `folder` with a file nested deeper than the 260-character Windows path limit.
    fn deep_tree(folder: &Path) -> PathBuf {
        let mut deepest = folder
            .join("cache/extracted/sha256-v3/ab")
            .join("a".repeat(64));
        while deepest.as_os_str().len() < 400 {
            deepest = deepest.join("nested-directory-level");
        }
        fs::create_dir_all(&deepest).unwrap();
        let file = deepest.join("setup-mod.tp2");
        fs::write(&file, b"BACKUP ~mod/backup~\n").unwrap();
        file
    }

    #[test]
    fn removes_paths_longer_than_the_windows_path_limit() {
        let temp = tempfile::tempdir().unwrap();
        let folder = temp.path().join(IDENTIFIER);
        let deepest = deep_tree(&folder);
        assert!(deepest.is_file());

        remove_folders(std::slice::from_ref(&folder), 1, Duration::ZERO).unwrap();

        assert!(!folder.exists());
    }

    #[cfg(windows)]
    #[test]
    fn removes_junctions_without_following_them() {
        let temp = tempfile::tempdir().unwrap();
        let outside = temp.path().join("game");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("baldur.sav"), b"keep").unwrap();
        let folder = temp.path().join(IDENTIFIER);
        deep_tree(&folder);
        junction(&folder.join("cache").join("linked"), &outside);
        // A folder that is itself a junction is unlinked, not emptied.
        let linked_folder = temp.path().join("roaming").join(IDENTIFIER);
        fs::create_dir(temp.path().join("roaming")).unwrap();
        junction(&linked_folder, &outside);

        remove_folders(&[folder.clone(), linked_folder.clone()], 1, Duration::ZERO).unwrap();

        assert!(!folder.exists());
        assert!(fs::symlink_metadata(&linked_folder).is_err());
        assert_eq!(fs::read(outside.join("baldur.sav")).unwrap(), b"keep");
    }

    #[test]
    fn missing_folders_count_as_removed() {
        let temp = tempfile::tempdir().unwrap();
        remove_folders(&[temp.path().join(IDENTIFIER)], 1, Duration::ZERO).unwrap();
    }

    /// Opens `path` so that it cannot be deleted until the handle is closed.
    #[cfg(windows)]
    fn hold(path: &Path) -> fs::File {
        use std::os::windows::fs::OpenOptionsExt;

        const FILE_SHARE_READ: u32 = 1;
        fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(path)
            .unwrap()
    }

    #[cfg(windows)]
    #[test]
    fn a_file_released_while_retrying_is_removed() {
        let temp = tempfile::tempdir().unwrap();
        let folder = temp.path().join(IDENTIFIER);
        let held = hold(&deep_tree(&folder));
        let releasing = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            drop(held);
        });

        let result = remove_folders(
            std::slice::from_ref(&folder),
            20,
            Duration::from_millis(100),
        );

        releasing.join().unwrap();
        result.unwrap();
        assert!(!folder.exists());
    }

    #[cfg(windows)]
    #[test]
    fn a_file_that_stays_held_is_reported_and_other_folders_are_still_removed() {
        let temp = tempfile::tempdir().unwrap();
        let held_folder = temp.path().join("local").join(IDENTIFIER);
        let other_folder = temp.path().join("roaming").join(IDENTIFIER);
        let held_file = deep_tree(&held_folder);
        deep_tree(&other_folder);
        let held = hold(&held_file);

        let failures = remove_folders(
            &[held_folder.clone(), other_folder.clone()],
            2,
            Duration::ZERO,
        )
        .unwrap_err();

        drop(held);
        assert_eq!(
            failures
                .iter()
                .map(|(folder, _)| folder.clone())
                .collect::<Vec<_>>(),
            vec![held_folder]
        );
        assert!(held_file.is_file());
        assert!(!other_folder.exists());
    }
}
