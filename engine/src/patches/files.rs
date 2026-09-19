//! Constrained files and independent copies used by the first patch transaction.
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
    if path.exists() {
        return Err("Patch evidence already exists.".into());
    }
    if pending.exists() {
        // Preserve a torn unpublished event; recovery may safely publish a new one.
        regular(&pending)?;
        let stamp = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(err)?
            .as_nanos();
        fs::rename(
            &pending,
            pending.with_extension(format!("interrupted-{stamp}")),
        )
        .map_err(err)?;
    }
    create(&pending, &bytes)?;
    fs::rename(&pending, path).map_err(err)
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
pub fn copy_tree(source: &Path, destination: &Path, profile: bool) -> Result<(), String> {
    if destination.exists() {
        return Err("Backup destination already exists.".into());
    }
    let before = inventory(source, profile)?;
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
        }
    }
    if inventory(source, profile)? != before {
        return Err("Backup source changed during copying.".into());
    }
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
