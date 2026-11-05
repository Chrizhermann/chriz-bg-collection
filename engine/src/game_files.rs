//! Every file of a finished game folder, recorded once by final verification, so that a later
//! game update can tell whether anything in the folder changed after the installer finished.
//!
//! Only content counts: a file keeps its identity while its length and SHA-256 stay the same,
//! whatever its timestamps. Folders count as well, so an added empty folder is a change too.
//! Names are relative to the game folder, lowercase and separated by `/`, as Windows treats
//! them; two names that differ only in case are refused.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::patches::files;

/// Format of [`GameFiles`].
pub const SCHEMA: u32 = 1;
/// Where final verification keeps the finished game's files, below the managed root.
pub const COMPLETED_RECORD: &str = ".chriz/evidence/final/game-files.json";
/// Largest record that is read back.
pub const RECORD_LIMIT: u64 = 256 * 1024 * 1024;
/// Most threads that hash at once; the disk, not the processor, limits more of them.
const HASHING_THREADS: usize = 4;

/// Length and content digest of one file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileIdentity {
    pub length: u64,
    pub sha256: String,
}

/// The files and folders below one game folder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameFiles {
    pub schema: u32,
    pub files: BTreeMap<String, FileIdentity>,
    pub folders: BTreeSet<String>,
}

/// How one name differs from an earlier record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Difference {
    Added,
    Removed,
    Changed,
}

impl GameFiles {
    /// Every name whose file or folder in `now` differs from this record, and how. Names that
    /// `ignored` accepts are left out, as are the folders above them.
    pub fn differences(
        &self,
        now: &GameFiles,
        ignored: impl Fn(&str) -> bool,
    ) -> BTreeMap<String, Difference> {
        let mut out = BTreeMap::new();
        for (name, identity) in &self.files {
            if ignored(name) {
                continue;
            }
            match now.files.get(name) {
                Some(current) if current == identity => {}
                Some(_) => {
                    out.insert(name.clone(), Difference::Changed);
                }
                None if now.folders.contains(name) => {
                    out.insert(name.clone(), Difference::Changed);
                }
                None => {
                    out.insert(name.clone(), Difference::Removed);
                }
            }
        }
        for name in now.files.keys() {
            if !ignored(name) && !self.files.contains_key(name) {
                out.entry(name.clone())
                    .or_insert(if self.folders.contains(name) {
                        Difference::Changed
                    } else {
                        Difference::Added
                    });
            }
        }
        // A folder that only holds ignored names is the runtime's, not a change.
        let holds_only_ignored = |record: &GameFiles, folder: &str| {
            let prefix = format!("{folder}/");
            let below = record
                .files
                .keys()
                .chain(record.folders.iter())
                .filter(|name| name.starts_with(&prefix))
                .collect::<Vec<_>>();
            !below.is_empty() && below.iter().all(|name| ignored(name))
        };
        for name in self.folders.symmetric_difference(&now.folders) {
            if out.contains_key(name) || ignored(name) {
                continue;
            }
            let (record, difference) = if self.folders.contains(name) {
                (self, Difference::Removed)
            } else {
                (now, Difference::Added)
            };
            if !holds_only_ignored(record, name) {
                out.insert(name.clone(), difference);
            }
        }
        out
    }

    /// Check that every recorded name is a safe lowercase relative name.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SCHEMA {
            return Err("This record of the game's files has an unknown format.".into());
        }
        for name in self.files.keys().chain(self.folders.iter()) {
            files::safe_relative(name)?;
            if name.to_ascii_lowercase() != *name {
                return Err("The record of the game's files is not normalized.".into());
            }
        }
        if self.files.keys().any(|name| self.folders.contains(name)) {
            return Err("The record of the game's files names one entry twice.".into());
        }
        Ok(())
    }
}

/// Record every file and folder below `root`, refusing links and other special entries.
pub fn capture(root: &Path) -> Result<GameFiles, String> {
    files::direct(root)?;
    let mut folders = BTreeSet::new();
    let mut pending: Vec<(String, PathBuf, u64)> = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(files::err)?;
        let path = entry.path();
        if path == root {
            continue;
        }
        let metadata = fs::symlink_metadata(path).map_err(files::err)?;
        if metadata.file_type().is_symlink() || reparse(&metadata) {
            return Err(format!("Linked path is not supported: {}", path.display()));
        }
        let name = path
            .strip_prefix(root)
            .map_err(files::err)?
            .to_string_lossy()
            .replace('\\', "/");
        files::safe_relative(&name)?;
        let name = name.to_ascii_lowercase();
        if !seen.insert(name.clone()) {
            return Err(format!(
                "Names that differ only in case are not supported: {name}"
            ));
        }
        if metadata.is_dir() {
            folders.insert(name);
        } else if metadata.is_file() {
            pending.push((name, path.to_path_buf(), metadata.len()));
        } else {
            return Err(format!("Unsupported special file: {}", path.display()));
        }
    }
    let next = AtomicUsize::new(0);
    let hashed = Mutex::new(BTreeMap::new());
    let failure = Mutex::new(None::<String>);
    let threads = std::thread::available_parallelism()
        .map_or(1, |count| count.get())
        .clamp(1, HASHING_THREADS);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some((name, path, length)) = pending.get(index) else {
                    break;
                };
                if failure.lock().map_or(true, |failed| failed.is_some()) {
                    break;
                }
                match identity(path, *length) {
                    Ok(identity) => {
                        if let Ok(mut hashed) = hashed.lock() {
                            hashed.insert(name.clone(), identity);
                        }
                    }
                    Err(error) => {
                        if let Ok(mut failed) = failure.lock() {
                            failed.get_or_insert(error);
                        }
                        break;
                    }
                }
            });
        }
    });
    if let Some(error) = failure.into_inner().map_err(files::err)? {
        return Err(error);
    }
    let files = hashed.into_inner().map_err(files::err)?;
    if files.len() != pending.len() {
        return Err("The game's files could not all be read.".into());
    }
    Ok(GameFiles {
        schema: SCHEMA,
        files,
        folders,
    })
}

fn identity(path: &Path, length: u64) -> Result<FileIdentity, String> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let sha256 = crate::digest::sha256_reader(&file)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if file.metadata().map_err(files::err)?.len() != length {
        return Err(format!(
            "File changed while it was read: {}",
            path.display()
        ));
    }
    Ok(FileIdentity { length, sha256 })
}

/// The files the installer finished below `managed_root/game`, as final verification recorded
/// them. Installations finished before this record existed have none.
pub fn read_completed(managed_root: &Path) -> Result<GameFiles, String> {
    let path = managed_root.join(COMPLETED_RECORD);
    if !path.exists() {
        return Err("This installation has no record of its finished game files; it was made before game updates could check them.".into());
    }
    let record: GameFiles =
        serde_json::from_slice(&files::read(&path, RECORD_LIMIT)?).map_err(files::err)?;
    record.validate()?;
    Ok(record)
}

#[cfg(windows)]
fn reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn reparse(_: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, bytes: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn content_not_timestamps_decides_and_every_kind_of_change_is_named() {
        let temp = tempfile::tempdir().unwrap();
        let game = temp.path().join("game");
        write(&game.join("override/A.CRE"), b"one");
        write(&game.join("WeiDU.log"), b"log");
        fs::create_dir_all(game.join("empty")).unwrap();
        let before = capture(&game).unwrap();
        assert_eq!(
            before.files.keys().collect::<Vec<_>>(),
            ["override/a.cre", "weidu.log"]
        );
        assert!(before.folders.contains("override") && before.folders.contains("empty"));
        // Rewriting the same bytes is no change.
        write(&game.join("override/A.CRE"), b"one");
        assert!(before
            .differences(&capture(&game).unwrap(), |_| false)
            .is_empty());

        write(&game.join("override/A.CRE"), b"two");
        write(&game.join("override/new.spl"), b"new");
        fs::remove_file(game.join("WeiDU.log")).unwrap();
        fs::remove_dir(game.join("empty")).unwrap();
        fs::create_dir_all(game.join("added")).unwrap();
        let now = capture(&game).unwrap();
        let differences = before.differences(&now, |_| false);
        assert_eq!(
            differences,
            BTreeMap::from([
                ("added".to_owned(), Difference::Added),
                ("empty".to_owned(), Difference::Removed),
                ("override/a.cre".to_owned(), Difference::Changed),
                ("override/new.spl".to_owned(), Difference::Added),
                ("weidu.log".to_owned(), Difference::Removed),
            ])
        );
    }

    #[test]
    fn ignored_names_and_folders_holding_only_them_are_no_change() {
        let temp = tempfile::tempdir().unwrap();
        let game = temp.path().join("game");
        write(&game.join("override/a.cre"), b"one");
        let before = capture(&game).unwrap();
        write(&game.join("InfinityLoader.log"), b"runtime");
        write(&game.join("override/bfbot_presets/Ilauna.lua"), b"preset");
        let now = capture(&game).unwrap();
        let ignored = |name: &str| {
            name == "infinityloader.log" || name.starts_with("override/bfbot_presets/")
        };
        assert!(before.differences(&now, ignored).is_empty());
        // The same folder with anything else in it is a change.
        write(&game.join("override/bfbot_presets/other.txt"), b"x");
        let ignored_presets = |name: &str| name.ends_with(".lua");
        assert_eq!(
            before.differences(&capture(&game).unwrap(), ignored_presets),
            BTreeMap::from([
                ("infinityloader.log".to_owned(), Difference::Added),
                ("override/bfbot_presets".to_owned(), Difference::Added),
                (
                    "override/bfbot_presets/other.txt".to_owned(),
                    Difference::Added
                ),
            ])
        );
    }

    #[test]
    fn a_folder_replaced_by_a_file_is_a_change_and_records_validate() {
        let temp = tempfile::tempdir().unwrap();
        let game = temp.path().join("game");
        write(&game.join("thing/inner"), b"x");
        let before = capture(&game).unwrap();
        fs::remove_dir_all(game.join("thing")).unwrap();
        write(&game.join("thing"), b"file now");
        let differences = before.differences(&capture(&game).unwrap(), |_| false);
        assert_eq!(differences.get("thing"), Some(&Difference::Changed));
        assert_eq!(differences.get("thing/inner"), Some(&Difference::Removed));
        before.validate().unwrap();
        let mut bad = before.clone();
        bad.files
            .insert("Upper".into(), bad.files["thing/inner"].clone());
        assert!(bad.validate().is_err());
        let mut escape = before;
        escape.folders.insert("../x".into());
        assert!(escape.validate().is_err());
    }

    #[test]
    fn missing_record_explains_that_older_installations_cannot_be_checked() {
        let temp = tempfile::tempdir().unwrap();
        let error = read_completed(temp.path()).unwrap_err();
        assert!(error.contains("made before game updates could check them"));
        let game = temp.path().join("game");
        write(&game.join("a"), b"x");
        let record = capture(&game).unwrap();
        write(
            &temp.path().join(COMPLETED_RECORD),
            &serde_json::to_vec_pretty(&record).unwrap(),
        );
        assert_eq!(read_completed(temp.path()).unwrap(), record);
    }

    #[cfg(windows)]
    #[test]
    fn linked_folders_are_refused() {
        let temp = tempfile::tempdir().unwrap();
        let game = temp.path().join("game");
        write(&game.join("a"), b"x");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        crate::held_folder::tests::junction(&game.join("linked"), &outside);
        assert!(capture(&game).unwrap_err().contains("Linked path"));
    }
}
