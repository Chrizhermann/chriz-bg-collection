//! Reading and publishing evidence files only inside one installation's own folder.
//!
//! A path check before a file is used leaves a window in which a folder on the way could be
//! replaced by a junction to somewhere else. The functions here therefore check the handle
//! they actually read or write through: it is opened without following a reparse point at the
//! file itself, and its real path, as the system resolves it, must lie inside the expected
//! folder. This is proportionate protection for a mod installer, not a hostile-filesystem
//! framework: a replaced folder makes the operation fail instead of reading or writing
//! elsewhere.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// The temporary file of a record being published is named `.cebg-record-<12 letters or
/// digits>.partial`, a format reserved for it, so that a reader can tell the leftover of an
/// interrupted publication from every other entry.
const TEMPORARY_PREFIX: &str = ".cebg-record-";
const TEMPORARY_SUFFIX: &str = ".partial";
const TEMPORARY_RANDOM: usize = 12;

/// Whether the entry `name` in `folder` is the leftover temporary file of an interrupted
/// publication: a direct regular file, not a link, folder or other reparse point, whose name
/// has exactly the reserved format.
pub(crate) fn is_interrupted_publication(folder: &Path, name: &str) -> bool {
    let Some(random) = name
        .strip_prefix(TEMPORARY_PREFIX)
        .and_then(|rest| rest.strip_suffix(TEMPORARY_SUFFIX))
    else {
        return false;
    };
    if random.len() != TEMPORARY_RANDOM || !random.bytes().all(|byte| byte.is_ascii_alphanumeric())
    {
        return false;
    }
    let candidate = folder.join(name);
    fs::symlink_metadata(&candidate).is_ok_and(|metadata| {
        metadata.is_file() && !crate::reparse::metadata_is_link(&candidate, &metadata)
    })
}
/// The real path of an open file or folder.
#[cfg(windows)]
fn final_path(file: &File, opened: &Path) -> io::Result<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::ERROR_INVALID_FUNCTION;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFinalPathNameByHandleW, FILE_NAME_NORMALIZED, VOLUME_NAME_DOS,
    };

    let handle = file.as_raw_handle();
    let mut buffer = vec![0_u16; 1024];
    loop {
        let length = unsafe {
            GetFinalPathNameByHandleW(
                handle,
                buffer.as_mut_ptr(),
                u32::try_from(buffer.len()).unwrap_or(u32::MAX),
                FILE_NAME_NORMALIZED | VOLUME_NAME_DOS,
            )
        } as usize;
        if length == 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(ERROR_INVALID_FUNCTION as i32) {
                for ancestor in opened.ancestors() {
                    if ancestor.as_os_str().is_empty() {
                        continue;
                    }
                    let metadata = fs::symlink_metadata(ancestor)?;
                    if crate::reparse::metadata_is_link(ancestor, &metadata) {
                        return Err(outside(ancestor, "is a link or another reparse point"));
                    }
                }
                return crate::reparse::canonicalize_flexible(opened);
            }
            return Err(error);
        }
        if length < buffer.len() {
            return Ok(PathBuf::from(std::ffi::OsString::from_wide(
                &buffer[..length],
            )));
        }
        buffer.resize(length + 1, 0);
    }
}

#[cfg(not(windows))]
fn final_path(_file: &File, opened: &Path) -> io::Result<PathBuf> {
    fs::canonicalize(opened)
}

/// Whether `path` is `root` or lies below it, comparing components as the platform does.
fn is_within(path: &Path, root: &Path) -> bool {
    crate::reparse::windows_path_starts_with(path, root)
}

/// Checks that the open `file`, opened as `opened`, is no link/surrogate reparse point and
/// really lies inside `root`.
fn confirm_inside(file: &File, opened: &Path, root: &Path) -> io::Result<()> {
    let metadata = file.metadata()?;
    if crate::reparse::handle_is_link(file, &metadata) {
        return Err(outside(opened, "is a link or another reparse point"));
    }
    let real = final_path(file, opened)?;
    if !is_within(&real, root) {
        return Err(outside(
            opened,
            &format!("resolves to {}, outside {}", real.display(), root.display()),
        ));
    }
    Ok(())
}

fn outside(path: &Path, why: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!(
            "{} is not inside this installation's evidence: it {why}",
            path.display()
        ),
    )
}

fn open_without_following(path: &Path, directory: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        };
        let mut flags = FILE_FLAG_OPEN_REPARSE_POINT;
        if directory {
            flags |= FILE_FLAG_BACKUP_SEMANTICS;
        }
        options.custom_flags(flags);
    }
    #[cfg(not(windows))]
    let _ = directory;
    options.open(path)
}

/// The bytes of the file at `path`, read through a handle whose real path lies inside `root`;
/// `None` when there is no such file.
pub(crate) fn read_inside(path: &Path, root: &Path) -> io::Result<Option<Vec<u8>>> {
    #[cfg(any(debug_assertions, feature = "test-hooks"))]
    before_evidence_use_for_test();
    let mut file = match open_without_following(path, false) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    confirm_inside(&file, path, root)?;
    if !file.metadata()?.is_file() {
        return Err(outside(path, "is not a regular file"));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(Some(bytes))
}

/// What publishing a file inside an installation's evidence found.
pub(crate) enum Published {
    /// The file is now there, complete.
    Written,
    /// The same bytes were there already.
    AlreadyThere,
    /// Different bytes are there; nothing was changed.
    Differs,
}

/// Publishes `bytes` as the new file `path`, complete or not at all, inside `root`. The folder
/// and the temporary file are each checked through their open handles to lie inside `root`
/// before anything is written; the temporary file is written and synced through its own handle
/// and only then renamed into place without replacing anything, and the published file is read
/// back through a checked handle. `interrupt` lets test builds stop halfway, as a crash would.
pub(crate) fn publish_inside(
    path: &Path,
    bytes: &[u8],
    root: &Path,
    interrupt: impl FnOnce(tempfile::NamedTempFile) -> io::Result<tempfile::NamedTempFile>,
) -> io::Result<Published> {
    let folder = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no folder"))?;
    #[cfg(any(debug_assertions, feature = "test-hooks"))]
    before_evidence_use_for_test();
    let directory = open_without_following(folder, true)?;
    confirm_inside(&directory, folder, root)?;
    if let Some(existing) = read_inside(path, root)? {
        return Ok(if existing == bytes {
            Published::AlreadyThere
        } else {
            Published::Differs
        });
    }
    let temporary = tempfile::Builder::new()
        .prefix(TEMPORARY_PREFIX)
        .suffix(TEMPORARY_SUFFIX)
        .rand_bytes(TEMPORARY_RANDOM)
        .tempfile_in(folder)?;
    confirm_inside(temporary.as_file(), temporary.path(), root)?;
    let mut temporary = interrupt(temporary)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    drop(directory);
    match crate::fs_retry::persist_noclobber(temporary, path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    match read_inside(path, root)? {
        Some(written) if written == bytes => Ok(Published::Written),
        Some(_) => Ok(Published::Differs),
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "{} disappeared right after it was published",
                path.display()
            ),
        )),
    }
}

/// Test builds run, once, the script that `CHRIZ_BG_COLLECTION_TEST_BEFORE_EVIDENCE_USE` names,
/// right before an evidence file is opened: a test uses it to replace a folder on the way after
/// every path check, as another program could.
#[cfg(any(debug_assertions, feature = "test-hooks"))]
fn before_evidence_use_for_test() {
    let Some(script) = std::env::var_os("CHRIZ_BG_COLLECTION_TEST_BEFORE_EVIDENCE_USE") else {
        return;
    };
    let script = PathBuf::from(script);
    if !script.is_file() {
        return;
    }
    let pending = script.with_extension("running.cmd");
    if fs::rename(&script, &pending).is_err() {
        return;
    }
    let _ = std::process::Command::new("cmd")
        .arg("/C")
        .arg(&pending)
        .status();
}
