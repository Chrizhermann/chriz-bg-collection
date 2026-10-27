//! Constrained files and independent copies used by the first patch transaction.
use crate::fs_retry::{self, CheckedRenameError};
use crate::held_folder::{self, EntryError, EntryErrorKind, Existing, HeldFolder, RemovableFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path},
    time::UNIX_EPOCH,
};

pub fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn direct(path: &Path) -> Result<(), String> {
    for part in path.ancestors() {
        let metadata = fs::symlink_metadata(part).map_err(err)?;
        if metadata.file_type().is_symlink() || reparse(&metadata) {
            return Err(format!("Linked path is not supported: {}", part.display()));
        }
    }
    Ok(())
}
pub fn regular(path: &Path) -> Result<(), String> {
    direct(path)?;
    let meta = fs::symlink_metadata(path).map_err(err)?;
    if !meta.is_file() || links(path, &meta)? != 1 {
        return Err(format!(
            "Expected an independent regular file: {}",
            path.display()
        ));
    }
    Ok(())
}
pub fn read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    regular(path)?;
    if fs::metadata(path).map_err(err)?.len() > limit {
        return Err(format!("File exceeds its size limit: {}", path.display()));
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(err)?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(err)?;
    if bytes.len() as u64 > limit {
        return Err("File grew beyond its size limit.".into());
    }
    Ok(bytes)
}
pub fn hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(err)?;
    let mut digest = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(err)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    Ok(hex::encode(digest.finalize()))
}
pub fn safe_relative(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.contains('\\')
        || name.contains(':')
        || Path::new(name)
            .components()
            .any(|p| !matches!(p, Component::Normal(_)))
    {
        return Err("Unsafe relative patch path.".into());
    }
    Ok(())
}
pub fn create(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Missing parent")?;
    // Inspect the nearest existing ancestor before creating any directory.
    let ancestor = parent
        .ancestors()
        .find(|p| p.exists())
        .ok_or("Missing ancestor")?;
    direct(ancestor)?;
    fs::create_dir_all(parent).map_err(err)?;
    direct(parent)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(err)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(err)
}
pub fn json_new(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(err)?;
    let pending = path.with_extension("pending");
    let absent = || {
        if path.exists() {
            return Err("Patch evidence already exists.".to_owned());
        }
        Ok(())
    };
    absent()?;
    if pending.exists() {
        // Preserve a torn unpublished event; recovery may safely publish a new one.
        regular(&pending)?;
        let stamp = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(err)?
            .as_nanos();
        fs_retry::rename(
            &pending,
            &pending.with_extension(format!("interrupted-{stamp}")),
        )
        .map_err(err)?;
    }
    create(&pending, &bytes)?;
    // Evidence is never replaced, also not by one published while a scanner held the pending
    // file: the check runs again before every attempt.
    match fs_retry::rename_checked(&pending, path, absent) {
        Ok(()) => Ok(()),
        Err(CheckedRenameError::Check(error)) => Err(error),
        Err(CheckedRenameError::Rename(error)) => Err(err(error)),
    }
}

/// The game folder of a managed installation and one of its transaction records, both held from
/// the managed root down: every read, write and removal below them stays in the folders that were
/// checked, whatever happens to their paths meanwhile.
pub(crate) struct HeldTransaction {
    pub(crate) game: HeldFolder,
    /// Where the adapter payload and the backups of the affected files are read from. Nothing is
    /// written here: replacements go straight into the held game files.
    pub(crate) record: HeldFolder,
}

/// Hold `managed_root/game` and the transaction record folder `record` inside `managed_root`.
pub(crate) fn hold(managed_root: &Path, record: &Path) -> Result<HeldTransaction, String> {
    let names = record
        .strip_prefix(managed_root)
        .map_err(|_| "Transaction record is outside this installation.")?
        .components()
        .map(|part| match part {
            Component::Normal(name) => name.to_str().ok_or("Unsupported transaction record name."),
            _ => Err("Unsupported transaction record name."),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let root = HeldFolder::open(managed_root, []).map_err(held)?;
    Ok(HeldTransaction {
        game: root.open_below(["game"]).map_err(held)?,
        record: root.open_below(names).map_err(held)?,
    })
}

/// A held folder or entry that could not be used, as a message.
pub(crate) fn held(error: EntryError) -> String {
    let path = error.path.display();
    match error.kind {
        EntryErrorKind::Missing => format!("Missing file or folder: {path}"),
        EntryErrorKind::NotDirect => format!("Linked or unexpected entry is not supported: {path}"),
        EntryErrorKind::Io(error) => format!("{path}: {error}"),
    }
}

/// The folders and the last name of a safe relative name such as `override/bgclatxt.2da`.
pub(crate) fn split_relative(name: &str) -> Result<(Vec<&str>, &str), String> {
    safe_relative(name)?;
    let mut parts = name.split('/').collect::<Vec<_>>();
    let last = parts.pop().ok_or("Unsafe relative patch path.")?;
    Ok((parts, last))
}

/// `None` for an entry that is not there.
fn present<T>(result: Result<T, EntryError>) -> Result<Option<T>, String> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(EntryError {
            kind: EntryErrorKind::Missing,
            ..
        }) => Ok(None),
        Err(error) => Err(held(error)),
    }
}

/// A file with another name is changed through either name.
fn independent(file: &File, path: &Path) -> Result<(), String> {
    if held_folder::link_count(file).map_err(err)? != 1 {
        return Err(format!(
            "Expected an independent regular file: {}",
            path.display()
        ));
    }
    Ok(())
}

/// Read the independent regular file `name` below `folder`, refusing more than `limit` bytes.
pub(crate) fn read_held(folder: &HeldFolder, name: &str, limit: u64) -> Result<Vec<u8>, String> {
    let (folders, last) = split_relative(name)?;
    let parent = folder.open_below(folders).map_err(held)?;
    let (file, metadata) = parent.open_file(last).map_err(held)?;
    let path = parent.path().join(last);
    independent(&file, &path)?;
    if metadata.len() > limit {
        return Err(format!("File exceeds its size limit: {}", path.display()));
    }
    let mut bytes = Vec::new();
    (&file)
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(err)?;
    if bytes.len() as u64 > limit {
        return Err("File grew beyond its size limit.".into());
    }
    Ok(bytes)
}

/// The hash of the independent regular file `name` below `folder`.
pub(crate) fn hash_held(folder: &HeldFolder, name: &str) -> Result<String, String> {
    let (folders, last) = split_relative(name)?;
    let parent = folder.open_below(folders).map_err(held)?;
    let (file, _) = parent.open_file(last).map_err(held)?;
    independent(&file, &parent.path().join(last))?;
    crate::digest::sha256_reader(&file).map_err(err)
}

/// Check that each named file below `folder` still has its hash.
pub(crate) fn verify_held_hashes(
    folder: &HeldFolder,
    hashes: &BTreeMap<String, String>,
) -> Result<(), String> {
    for (name, expected) in hashes {
        if hash_held(folder, name)? != *expected {
            return Err(format!("File changed: {name}"));
        }
    }
    Ok(())
}

/// Write `bytes` to the file `name` directly in `folder` as a truncating write by path did: an
/// existing file, which must be a direct regular file without another name, keeps its identity
/// and gets the new content; a missing one is created in `folder`.
pub(crate) fn write_held(folder: &HeldFolder, name: &str, bytes: &[u8]) -> Result<(), String> {
    folder
        .write_file(name, bytes, Existing::Overwrite)
        .map_err(held)
}

/// The file `name` below `folder` that a transaction created, opened so that it can be removed,
/// or `None` if it is gone. It must still be an independent regular file with `hash`; otherwise
/// this fails with `changed`. Until the returned file is closed or removed, nobody can change it.
pub(crate) fn created_file(
    folder: &HeldFolder,
    name: &str,
    hash: &str,
    changed: &str,
) -> Result<Option<RemovableFile>, String> {
    let (folders, last) = split_relative(name)?;
    let Some(parent) = present(folder.open_below(folders))? else {
        return Ok(None);
    };
    let Some(file) = present(parent.open_removable_file(last))? else {
        return Ok(None);
    };
    independent(file.file(), file.path())?;
    if crate::digest::sha256_reader(file.file()).map_err(err)? != hash {
        return Err(changed.into());
    }
    Ok(Some(file))
}

/// Remove what a transaction created at `name` below `folder`, if it is still there: a file that
/// still has `hash`, failing with `changed` otherwise, or with no hash an empty folder.
pub(crate) fn remove_created(
    folder: &HeldFolder,
    name: &str,
    hash: Option<&str>,
    changed: &str,
) -> Result<(), String> {
    match hash {
        Some(hash) => match created_file(folder, name, hash, changed)? {
            Some(file) => {
                let path = file.path().to_path_buf();
                file.remove()
                    .map_err(|error| format!("{}: {error}", path.display()))
            }
            None => Ok(()),
        },
        None => {
            let (folders, last) = split_relative(name)?;
            let Some(parent) = present(folder.open_below(folders))? else {
                return Ok(());
            };
            present(parent.remove_empty_folder(last)).map(|_| ())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stamp {
    pub length: u64,
    pub modified_nanos: u128,
    pub directory: bool,
}
pub type Inventory = BTreeMap<String, Stamp>;
pub fn inventory(root: &Path, profile: bool) -> Result<Inventory, String> {
    if !profile {
        direct(root)?;
    }
    let mut out = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(err)?;
        let path = entry.path();
        let meta = fs::symlink_metadata(path).map_err(err)?;
        if meta.file_type().is_symlink() || (reparse(&meta) && !(profile && cloud(path)?)) {
            return Err(format!("Unsupported linked path: {}", path.display()));
        }
        if !meta.is_file() && !meta.is_dir() {
            return Err("Unsupported special file.".into());
        }
        if path == root {
            continue;
        }
        let name = path
            .strip_prefix(root)
            .map_err(err)?
            .to_string_lossy()
            .replace('\\', "/");
        safe_relative(&name)?;
        let stamp = Stamp {
            length: if meta.is_dir() { 0 } else { meta.len() },
            modified_nanos: if meta.is_dir() {
                0
            } else {
                meta.modified()
                    .map_err(err)?
                    .duration_since(UNIX_EPOCH)
                    .map_err(err)?
                    .as_nanos()
            },
            directory: meta.is_dir(),
        };
        if out.insert(name.to_ascii_lowercase(), stamp).is_some() {
            return Err("Case-colliding files are not supported.".into());
        }
    }
    Ok(out)
}
pub fn total(items: &Inventory) -> u64 {
    items.values().map(|v| v.length).sum()
}
#[cfg(test)]
pub fn copy_tree(source: &Path, destination: &Path, profile: bool) -> Result<(), String> {
    copy_tree_with_progress(source, destination, profile, "backup-game", &|_| {})
}

pub fn copy_tree_with_progress(
    source: &Path,
    destination: &Path,
    profile: bool,
    stage: &str,
    progress: crate::operation_progress::ProgressSink<'_>,
) -> Result<(), String> {
    crate::operation_progress::stage(progress, stage);
    if destination.exists() {
        return Err("Backup destination already exists.".into());
    }
    let before = inventory(source, profile)?;
    let total = before
        .values()
        .filter(|stamp| !stamp.directory)
        .map(|stamp| stamp.length)
        .sum();
    let mut completed = 0;
    let mut reported = std::time::Instant::now();
    crate::operation_progress::bytes(progress, stage, completed, total);
    direct(destination.parent().ok_or("Backup parent missing")?)?;
    fs::create_dir(destination).map_err(err)?;
    for (name, stamp) in &before {
        let to = destination.join(name);
        if stamp.directory {
            fs::create_dir_all(&to).map_err(err)?;
        } else {
            // Inventory path casing is canonicalized only on Windows, the supported platform.
            let from = source.join(name);
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).map_err(err)?;
            }
            let before_hash = hash(&from)?;
            let mut input = File::open(&from).map_err(err)?;
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&to)
                .map_err(err)?;
            std::io::copy(&mut input, &mut output).map_err(err)?;
            output.sync_all().map_err(err)?;
            if hash(&to)? != before_hash || hash(&from)? != before_hash {
                return Err(format!("Backup verification failed: {name}"));
            }
            completed += stamp.length;
            if completed < total && reported.elapsed() >= std::time::Duration::from_millis(250) {
                crate::operation_progress::bytes(progress, stage, completed, total);
                reported = std::time::Instant::now();
            }
        }
    }
    if inventory(source, profile)? != before {
        return Err("Backup source changed during copying.".into());
    }
    crate::operation_progress::bytes(progress, stage, completed, total);
    Ok(())
}

#[cfg(windows)]
fn reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn reparse(_: &fs::Metadata) -> bool {
    false
}
#[cfg(windows)]
fn links(path: &Path, _: &fs::Metadata) -> Result<u64, String> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };
    let file = File::open(path).map_err(err)?;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    if unsafe { GetFileInformationByHandle(file.as_raw_handle().cast(), &mut info) } == 0 {
        return Err(err(std::io::Error::last_os_error()));
    }
    Ok(info.nNumberOfLinks.into())
}
#[cfg(unix)]
fn links(_: &Path, meta: &fs::Metadata) -> Result<u64, String> {
    use std::os::unix::fs::MetadataExt;
    Ok(meta.nlink())
}
#[cfg(not(any(unix, windows)))]
fn links(_: &Path, _: &fs::Metadata) -> Result<u64, String> {
    Err("Platform not supported.".into())
}
#[cfg(windows)]
fn cloud(path: &Path) -> Result<bool, String> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        FileAttributeTagInfo, GetFileInformationByHandleEx, FILE_ATTRIBUTE_TAG_INFO,
    };
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(0x02000000 | 0x00200000)
        .open(path)
        .map_err(err)?;
    let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle().cast(),
            FileAttributeTagInfo,
            (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    } == 0
    {
        return Err(err(std::io::Error::last_os_error()));
    }
    Ok(info.ReparseTag & 0xFFFF0FFF == 0x9000001A)
}
#[cfg(not(windows))]
fn cloud(_: &Path) -> Result<bool, String> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_copy_and_create_once_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("a"), b"original").unwrap();
        let dst = temp.path().join("backup");
        copy_tree(&src, &dst, false).unwrap();
        fs::write(src.join("a"), b"new").unwrap();
        assert_eq!(fs::read(dst.join("a")).unwrap(), b"original");
        assert!(copy_tree(&src, &dst, false).is_err());
        json_new(&temp.path().join("event.json"), &1).unwrap();
        assert!(json_new(&temp.path().join("event.json"), &2).is_err());
    }
    /// A torn pending event that a scanner briefly holds is still preserved, and the new event
    /// is published.
    #[cfg(windows)]
    #[test]
    fn a_held_torn_event_is_set_aside_once_the_holder_lets_go() {
        use crate::fs_retry::test_hook::hold_until_first_wait;

        let temp = tempfile::tempdir().unwrap();
        let event = temp.path().join("event.json");
        let torn = temp.path().join("event.pending");
        fs::write(&torn, b"{\"tor").unwrap();

        let (result, waits) = hold_until_first_wait(&torn, || json_new(&event, &1));

        result.unwrap();
        assert!(
            waits >= 1,
            "the held pending event must fail the first attempt"
        );
        assert_eq!(fs::read(&event).unwrap(), b"1");
        let preserved = fs::read_dir(temp.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path != &event)
            .collect::<Vec<_>>();
        assert_eq!(preserved.len(), 1, "{preserved:?}");
        assert!(preserved[0]
            .extension()
            .unwrap()
            .to_string_lossy()
            .starts_with("interrupted-"));
        assert_eq!(fs::read(&preserved[0]).unwrap(), b"{\"tor");
    }
    #[test]
    fn hardlinks_and_path_escapes_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let one = temp.path().join("one");
        fs::write(&one, b"x").unwrap();
        fs::hard_link(&one, temp.path().join("two")).unwrap();
        assert!(regular(&one).is_err());
        for name in ["../other", "/root", "x:stream", "a\\b", ""] {
            assert!(safe_relative(name).is_err());
        }
    }
}
