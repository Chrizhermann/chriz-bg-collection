//! Windows reparse-tag, file-attribute, metadata-noise, and path-normalization helpers.
//!
//! Windows uses reparse points (`FILE_ATTRIBUTE_REPARSE_POINT`, `0x400`) both for real filesystem
//! redirections (symlinks and directory junctions/mount points, which carry Microsoft's
//! `IsReparseTagNameSurrogate` bit `0x2000_0000`) and for benign filter/storage states (OneDrive
//! Cloud Files, Windows Overlay Filter `WOF`, NTFS Deduplication, ProjFS). Only name-surrogate
//! reparse points redirect path lookups or escape containment.

use std::ffi::OsStr;
use std::fs::{self, File, Metadata};
use std::io;
use std::path::{Component, Path, PathBuf};

/// Returns `true` when `name` is harmless OS Explorer/Finder metadata (`desktop.ini`,
/// `Thumbs.db`, `.DS_Store`), case-insensitively.
pub fn is_benign_os_metadata_filename(name: &OsStr) -> bool {
    let Some(text) = name.to_str() else {
        return false;
    };
    text.eq_ignore_ascii_case("desktop.ini")
        || text.eq_ignore_ascii_case("thumbs.db")
        || text.eq_ignore_ascii_case(".ds_store")
}

/// Returns `true` when `tag` is a Microsoft name-surrogate reparse tag (`0x2000_0000`, e.g.
/// `IO_REPARSE_TAG_MOUNT_POINT` `0xA0000003` or `IO_REPARSE_TAG_SYMLINK` `0xA000000C`).
#[inline]
pub const fn is_name_surrogate_reparse_tag(tag: u32) -> bool {
    (tag & 0x2000_0000) != 0
}

/// Returns `true` when `tag` is a known benign Microsoft storage/filter reparse tag
/// (OneDrive Cloud Files `0x9000_001A`..`0x9000_F01A`, Windows Overlay Filter `0x8000_0017`,
/// NTFS Deduplication `0x8000_0013`, or ProjFS `0x9000_001C`). Unknown/third-party reparse tags
/// and all name-surrogate tags return `false` so foreign reparse points stay refused.
#[inline]
pub const fn is_benign_storage_reparse_tag(tag: u32) -> bool {
    !is_name_surrogate_reparse_tag(tag)
        && ((tag & 0xFFFF_0FFF) == 0x9000_001A
            || tag == 0x8000_0017
            || tag == 0x8000_0013
            || tag == 0x9000_001C)
}

/// Inspects an already-open non-following handle on Windows to decide whether its reparse point
/// is unsafe (a symlink, junction/mount point, or unknown third-party reparse tag) rather than a
/// known benign Microsoft storage/filter tag (OneDrive Cloud Files, WOF, Dedup, ProjFS).
#[cfg(windows)]
pub fn handle_is_name_surrogate_reparse(file: &File) -> io::Result<bool> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        FileAttributeTagInfo, GetFileInformationByHandleEx, FILE_ATTRIBUTE_REPARSE_POINT,
        FILE_ATTRIBUTE_TAG_INFO,
    };

    let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
    let ok = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle().cast(),
            FileAttributeTagInfo,
            (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    if (info.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) == 0 {
        return Ok(false);
    }
    Ok(!is_benign_storage_reparse_tag(info.ReparseTag))
}

#[cfg(not(windows))]
pub fn handle_is_name_surrogate_reparse(_file: &File) -> io::Result<bool> {
    Ok(false)
}

/// Opens `path` without following a reparse point and returns `true` when it is a symbolic link or
/// a Windows name-surrogate reparse point (junction, volume mount point, global reparse).
#[cfg(windows)]
pub fn is_name_surrogate_reparse(path: &Path) -> io::Result<bool> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    };

    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(true);
    }
    if (metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) == 0 {
        return Ok(false);
    }
    let file = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)?;
    handle_is_name_surrogate_reparse(&file)
}

#[cfg(not(windows))]
pub fn is_name_surrogate_reparse(path: &Path) -> io::Result<bool> {
    Ok(fs::symlink_metadata(path)?.file_type().is_symlink())
}

/// Checks from `metadata` (fast path) and `path` (tag lookup when `0x400` is set) whether `path`
/// is a symbolic link or a Windows name-surrogate reparse point. Fails closed (`true`) if a
/// flagged reparse point's tag cannot be read.
pub fn metadata_is_link(path: &Path, metadata: &Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        if (metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) == 0 {
            return false;
        }
        is_name_surrogate_reparse(path).unwrap_or(true)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

/// Checks from an already-open non-following `file` handle and its `metadata` whether it is a
/// symbolic link or name-surrogate reparse point.
pub fn handle_is_link(file: &File, metadata: &Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        if (metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) == 0 {
            return false;
        }
        handle_is_name_surrogate_reparse(file).unwrap_or(true)
    }
    #[cfg(not(windows))]
    {
        let _ = file;
        false
    }
}

/// Verifies every existing ancestor *above* `root` resolves to a directory (allowing parent
/// junctions/mount points and OneDrive folders), while requiring `root` *itself* (if it already
/// exists) to be a direct non-link directory.
pub fn ensure_existing_ancestors_are_directories(root: &Path) -> io::Result<()> {
    for ancestor in root.ancestors().skip(1) {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        if ancestor.components().count() <= 2
            && matches!(ancestor.components().next_back(), Some(Component::RootDir))
        {
            continue;
        }
        match fs::metadata(ancestor) {
            Ok(meta) => {
                if !meta.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "destination ancestor {} is not a direct non-reparse directory",
                            ancestor.display()
                        ),
                    ));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        }
    }
    match fs::symlink_metadata(root) {
        Ok(meta) => {
            if metadata_is_link(root, &meta) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "{} is not a direct non-reparse directory (links and reparse points are not allowed)",
                        root.display()
                    ),
                ));
            }
            if !meta.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{} is not a direct non-reparse directory", root.display()),
                ));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    Ok(())
}

/// Clears `FILE_ATTRIBUTE_READONLY` on an open non-link regular file handle, preserving timestamps.
#[cfg(windows)]
pub fn clear_readonly_handle(file: &File) -> io::Result<()> {
    use std::os::windows::fs::MetadataExt;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        FileBasicInfo, SetFileInformationByHandle, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_READONLY,
        FILE_BASIC_INFO,
    };

    let meta = file.metadata()?;
    let attrs = meta.file_attributes();
    if (attrs & FILE_ATTRIBUTE_READONLY) == 0 {
        return Ok(());
    }
    let next_attrs = attrs & !FILE_ATTRIBUTE_READONLY;
    let basic = FILE_BASIC_INFO {
        CreationTime: 0,
        LastAccessTime: 0,
        LastWriteTime: 0,
        ChangeTime: 0,
        FileAttributes: if next_attrs == 0 {
            FILE_ATTRIBUTE_NORMAL
        } else {
            next_attrs
        },
    };
    let ok = unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle().cast(),
            FileBasicInfo,
            (&basic as *const FILE_BASIC_INFO).cast(),
            std::mem::size_of::<FILE_BASIC_INFO>() as u32,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn clear_readonly_handle(file: &File) -> io::Result<()> {
    let mut perms = file.metadata()?.permissions();
    if perms.readonly() {
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        file.set_permissions(perms)?;
    }
    Ok(())
}

/// Clears `FILE_ATTRIBUTE_READONLY` on `path` without following symlinks or name-surrogate links.
/// No-op if `path` does not exist.
#[cfg(windows)]
pub fn clear_readonly(path: &Path) -> io::Result<()> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_READONLY, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FILE_WRITE_ATTRIBUTES,
    };

    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata_is_link(path, &meta) || (meta.file_attributes() & FILE_ATTRIBUTE_READONLY) == 0 {
        return Ok(());
    }
    let file = match OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    clear_readonly_handle(&file)
}

#[cfg(not(windows))]
pub fn clear_readonly(path: &Path) -> io::Result<()> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    let mut perms = meta.permissions();
    if perms.readonly() {
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// Canonicalizes `path`, falling back to `std::path::absolute` on non-MountMgr Windows volumes
/// (e.g. virtual/RAM disks) where `GetFinalPathNameByHandleW` fails with `ERROR_INVALID_FUNCTION`
/// (`os error 1`).
pub fn canonicalize_flexible(path: &Path) -> io::Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(canonical) => Ok(canonical),
        Err(error) if is_invalid_function(&error) => {
            fs::metadata(path)?;
            std::path::absolute(path)
        }
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
fn is_invalid_function(error: &io::Error) -> bool {
    error.raw_os_error() == Some(1)
}

#[cfg(not(windows))]
fn is_invalid_function(_error: &io::Error) -> bool {
    false
}

/// Canonicalizes a path that may not exist yet by canonicalizing its deepest existing ancestor and
/// appending the remaining normal components.
pub fn canonicalize_prospective(path: &Path) -> io::Result<PathBuf> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::path::absolute(path)?
    };
    let mut tail = Vec::new();
    let mut cursor = abs.as_path();
    loop {
        if cursor.exists() {
            let mut resolved = canonicalize_flexible(cursor)?;
            for component in tail.iter().rev() {
                resolved.push(component);
            }
            return Ok(resolved);
        }
        match cursor.file_name() {
            Some(name) => {
                tail.push(name.to_os_string());
                match cursor.parent() {
                    Some(parent) => cursor = parent,
                    None => break,
                }
            }
            None => break,
        }
    }
    Ok(abs)
}

/// Strips the Windows verbatim `\\?\` prefix (`\\?\UNC\...` -> `\\...`, `\\?\C:\...` -> `C:\...`)
/// when doing so preserves identity, for child processes (`WeiDU`, game launch) and Explorer/UI.
pub fn display_windows_path(path: &Path) -> PathBuf {
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
    #[cfg(windows)]
    {
        let text = path.to_string_lossy();
        let stripped = if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
            PathBuf::from(format!(r"\\{unc}"))
        } else if let Some(local) = text.strip_prefix(r"\\?\") {
            PathBuf::from(local)
        } else {
            return path.to_path_buf();
        };
        if path.exists() {
            if let (Ok(orig), Ok(cand)) = (
                canonicalize_flexible(path),
                canonicalize_flexible(&stripped),
            ) {
                if same_normalized_string(&orig, &cand) {
                    return stripped;
                }
            }
            return path.to_path_buf();
        }
        if let (Some(parent), Some(name)) = (path.parent(), path.file_name()) {
            return display_windows_path(parent).join(name);
        }
        stripped
    }
}

fn normalized_key(path: &Path) -> String {
    let text = path.to_string_lossy();
    let no_prefix = if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else if let Some(local) = text.strip_prefix(r"\\?\") {
        local.to_string()
    } else {
        text.into_owned()
    };
    let unified = no_prefix.replace('/', r"\");
    let mut trimmed = unified.as_str();
    while trimmed.len() > 3 && trimmed.ends_with('\\') {
        trimmed = &trimmed[..trimmed.len() - 1];
    }
    #[cfg(windows)]
    {
        trimmed.to_lowercase()
    }
    #[cfg(not(windows))]
    {
        trimmed.to_string()
    }
}

fn same_normalized_string(a: &Path, b: &Path) -> bool {
    normalized_key(a) == normalized_key(b)
}

/// Lexical Windows path equality (case-insensitive, `/` == `\`, ignores `\\?\` prefix and
/// trailing separators) WITHOUT following symlinks/junctions on disk. Use when comparing an
/// already-canonicalized path against a recorded path to detect link redirection.
pub fn same_normalized_windows_path(a: &Path, b: &Path) -> bool {
    same_normalized_string(a, b)
}

/// Returns `true` when `a` and `b` refer to the same path (case-insensitively on Windows, ignoring
/// `\\?\` prefix differences and trailing separators, and using file identity when both exist).
pub fn same_windows_path(a: &Path, b: &Path) -> bool {
    if same_normalized_string(a, b) {
        return true;
    }
    if let Ok(true) = same_file::is_same_file(a, b) {
        return true;
    }
    match (canonicalize_prospective(a), canonicalize_prospective(b)) {
        (Ok(ca), Ok(cb)) => same_normalized_string(&ca, &cb),
        _ => false,
    }
}

/// Returns `true` when `path` equals `base` or lies underneath `base`, comparing components
/// case-insensitively on Windows and ignoring `\\?\` prefix differences.
pub fn windows_path_starts_with(path: &Path, base: &Path) -> bool {
    let p = normalized_key(path);
    let b = normalized_key(base);
    if p == b {
        return true;
    }
    if b.ends_with('\\') {
        return p.starts_with(&b);
    }
    p.strip_prefix(&b)
        .is_some_and(|rest| rest.starts_with('\\'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benign_os_metadata_names_match_case_insensitively() {
        assert!(is_benign_os_metadata_filename(OsStr::new("desktop.ini")));
        assert!(is_benign_os_metadata_filename(OsStr::new("Desktop.INI")));
        assert!(is_benign_os_metadata_filename(OsStr::new("Thumbs.db")));
        assert!(is_benign_os_metadata_filename(OsStr::new(".DS_Store")));
        assert!(!is_benign_os_metadata_filename(OsStr::new("Baldur.lua")));
        assert!(!is_benign_os_metadata_filename(OsStr::new("install.json")));
    }

    #[test]
    fn name_surrogate_bit_distinguishes_links_and_foreign_tags_from_cloud_and_filter_tags() {
        assert!(is_name_surrogate_reparse_tag(0xA000_0003)); // IO_REPARSE_TAG_MOUNT_POINT
        assert!(is_name_surrogate_reparse_tag(0xA000_000C)); // IO_REPARSE_TAG_SYMLINK
        assert!(!is_name_surrogate_reparse_tag(0x9000_001A)); // IO_REPARSE_TAG_CLOUD
        assert!(!is_name_surrogate_reparse_tag(0x9000_601A)); // IO_REPARSE_TAG_CLOUD_6
        assert!(!is_name_surrogate_reparse_tag(0x8000_0013)); // IO_REPARSE_TAG_DEDUP
        assert!(!is_name_surrogate_reparse_tag(0x8000_0017)); // IO_REPARSE_TAG_WOF

        assert!(is_benign_storage_reparse_tag(0x9000_001A));
        assert!(is_benign_storage_reparse_tag(0x9000_601A));
        assert!(is_benign_storage_reparse_tag(0x8000_0013));
        assert!(is_benign_storage_reparse_tag(0x8000_0017));
        assert!(is_benign_storage_reparse_tag(0x9000_001C));
        assert!(!is_benign_storage_reparse_tag(0xA000_0003));
        assert!(!is_benign_storage_reparse_tag(0xA000_000C));
        assert!(!is_benign_storage_reparse_tag(0x0000_7A11)); // Foreign/unknown tag
    }

    #[test]
    fn clear_readonly_makes_read_only_file_writable() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("readonly.txt");
        fs::write(&file, b"original").unwrap();
        let mut perms = fs::metadata(&file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file, perms).unwrap();

        clear_readonly(&file).unwrap();
        assert!(!fs::metadata(&file).unwrap().permissions().readonly());
        fs::write(&file, b"overwritten").unwrap();
        assert_eq!(fs::read(&file).unwrap(), b"overwritten");
    }

    #[test]
    fn windows_path_comparison_and_display_strip_verbatim_and_ignore_case() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("SubDir");
        fs::create_dir(&dir).unwrap();
        let canonical = canonicalize_flexible(&dir).unwrap();
        let displayed = display_windows_path(&canonical);
        #[cfg(windows)]
        {
            assert!(!displayed.to_string_lossy().starts_with(r"\\?\"));
            let upper = PathBuf::from(displayed.to_string_lossy().to_uppercase());
            assert!(same_windows_path(&canonical, &upper));
            assert!(windows_path_starts_with(
                &upper.join("child.txt"),
                &canonical
            ));
            assert!(!windows_path_starts_with(
                &PathBuf::from(format!("{}Other\\child.txt", displayed.display())),
                &displayed
            ));
        }
        #[cfg(not(windows))]
        {
            assert_eq!(displayed, canonical);
        }
    }

    #[test]
    fn prospective_canonicalize_handles_non_existent_tail() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("new-parent").join("leaf");
        let resolved = canonicalize_prospective(&target).unwrap();
        assert!(resolved.ends_with(Path::new("new-parent").join("leaf")));
    }
}
