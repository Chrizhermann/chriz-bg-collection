//! Folders below a target root, opened one level at a time without following a link.
//!
//! On Windows the target root is opened by path, refusing a link or reparse point, and every
//! folder below it is opened relative to the handle of the folder above it, again without
//! following a link. The whole chain is held without delete sharing, so none of its folders can be
//! renamed, removed or replaced while it is held. Files are opened, created and renamed only
//! through those handles. No path is resolved again once a folder was checked, so swapping a
//! folder for a junction in between cannot redirect a read or a write.
//!
//! Nothing above the target root is inspected. Once the root is held, re-pointing a link above it
//! no longer changes which folders the handles refer to, and a reparse point above it, such as a
//! cloud-files folder or a volume mounted in a folder, is no reason to refuse.

use std::ffi::OsString;
use std::fs::{self, Metadata};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

/// A folder, or an entry in one, that could not be used.
#[derive(Debug)]
pub(crate) struct EntryError {
    /// The entry, spelled as requested.
    pub(crate) path: PathBuf,
    pub(crate) kind: EntryErrorKind,
}

#[derive(Debug)]
pub(crate) enum EntryErrorKind {
    /// Nothing has this name.
    Missing,
    /// A link or other reparse point, or a file where a folder belongs or the other way round.
    NotDirect,
    /// Inspecting it failed otherwise.
    Io(io::Error),
}

impl EntryError {
    fn io(path: PathBuf, error: io::Error) -> Self {
        let kind = if error.kind() == ErrorKind::NotFound {
            EntryErrorKind::Missing
        } else {
            EntryErrorKind::Io(error)
        };
        Self { path, kind }
    }

    fn not_direct(path: PathBuf) -> Self {
        Self {
            path,
            kind: EntryErrorKind::NotDirect,
        }
    }
}

/// A folder below a target root, opened from the root down. On Windows the target root and every
/// folder below it stay open while this is held.
pub(crate) struct HeldFolder {
    /// The target root and every folder below it down to this one, outermost first.
    #[cfg(windows)]
    chain: Vec<fs::File>,
    /// For messages only on Windows, where the handles decide where entries are.
    path: PathBuf,
}

impl HeldFolder {
    /// Open the absolute `target_root`, then each of `folders` inside the one before. Each must be
    /// a direct directory: not a link, another reparse point or a file. No folder is created.
    pub(crate) fn open<'a>(
        target_root: &Path,
        folders: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, EntryError> {
        if !target_root.is_absolute() {
            return Err(EntryError::io(
                target_root.to_path_buf(),
                io::Error::new(ErrorKind::InvalidInput, "the target root must be absolute"),
            ));
        }
        let mut folder = Self::open_root(target_root)?;
        for name in folders {
            folder = folder.descend(name)?;
        }
        Ok(folder)
    }

    /// This folder, spelled as requested.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(windows)]
impl HeldFolder {
    fn open_root(target_root: &Path) -> Result<Self, EntryError> {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        };

        let path = target_root.to_path_buf();
        let root = fs::OpenOptions::new()
            .read(true)
            .share_mode(handles::SHARE_ALL_BUT_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(target_root)
            .map_err(|error| EntryError::io(path.clone(), error))?;
        require_direct_directory(&root, &path)?;
        Ok(Self {
            chain: vec![root],
            path,
        })
    }

    /// Open the folder `name` inside this one and hold it too.
    pub(crate) fn descend(mut self, name: &str) -> Result<Self, EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        let folder = handles::open_folder(self.folder(), name)
            .map_err(|error| EntryError::io(path.clone(), error))?;
        require_direct_directory(&folder, &path)?;
        self.chain.push(folder);
        self.path = path;
        Ok(self)
    }

    fn folder(&self) -> &fs::File {
        self.chain
            .last()
            .expect("the chain starts with the target root")
    }

    /// Open the direct regular file `name` in this folder for reading. While it is open, others
    /// can still read it but can neither change, replace nor remove it.
    pub(crate) fn open_file(&self, name: &str) -> Result<(fs::File, Metadata), EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        let file = handles::open_entry(self.folder(), name)
            .map_err(|error| EntryError::io(path.clone(), error))?;
        let metadata = file
            .metadata()
            .map_err(|error| EntryError::io(path.clone(), error))?;
        if !metadata.is_file() || is_link_or_reparse(&metadata) {
            return Err(EntryError::not_direct(path));
        }
        Ok((file, metadata))
    }

    /// Whether anything, a link included, is named `name` in this folder. Names match the way the
    /// file system matches them when a program opens one: ignoring case, unless the folder was
    /// made case-sensitive.
    pub(crate) fn contains(&self, name: &str) -> io::Result<bool> {
        check_name(name)?;
        handles::exists(self.folder(), name)
    }

    /// The names of all entries in this folder, links included, listed through its handle.
    pub(crate) fn names(&self) -> io::Result<Vec<OsString>> {
        handles::list(self.folder())
    }

    /// Create the file `name` with `content`, never replacing anything already there.
    pub(crate) fn create_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        self.publish_file(name, content, false)
    }

    /// Put a file with `content` in place of `name` in one step, or create it. A directory with
    /// that name is not replaced, and a link with that name is never followed.
    pub(crate) fn replace_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        self.publish_file(name, content, true)
    }

    /// Only a complete file is ever published under `name`.
    fn publish_file(&self, name: &str, content: &[u8], replace: bool) -> io::Result<()> {
        check_name(name)?;
        let temporary = self.create_temporary_file()?;
        let published = (&temporary)
            .write_all(content)
            .and_then(|()| temporary.sync_all())
            .and_then(|()| handles::rename_into(&temporary, self.folder(), name, replace));
        match published {
            Ok(()) => Ok(()),
            // Remove the unfinished file through its own handle.
            Err(error) => match handles::delete_on_close(&temporary) {
                Ok(()) => Err(error),
                Err(cleanup) => Err(io::Error::new(
                    error.kind(),
                    format!("{error}; its temporary file could not be removed: {cleanup}"),
                )),
            },
        }
    }

    fn create_temporary_file(&self) -> io::Result<fs::File> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let process = std::process::id();
        let mut attempt = 0_u32;
        loop {
            let name = format!(".chriz-{process}-{nonce}-{attempt}.tmp");
            match handles::create_new_file(self.folder(), &name) {
                Err(error) if error.kind() == ErrorKind::AlreadyExists && attempt < 16 => {
                    attempt += 1;
                }
                result => return result,
            }
        }
    }
}

#[cfg(windows)]
fn require_direct_directory(folder: &fs::File, path: &Path) -> Result<(), EntryError> {
    let metadata = folder
        .metadata()
        .map_err(|error| EntryError::io(path.to_path_buf(), error))?;
    if !metadata.is_dir() || is_link_or_reparse(&metadata) {
        return Err(EntryError::not_direct(path.to_path_buf()));
    }
    Ok(())
}

/// Opening and naming files relative to held folder handles.
#[cfg(windows)]
mod handles {
    use std::ffi::OsString;
    use std::fs::File;
    use std::io;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use std::ptr;

    use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
    use windows_sys::Wdk::Storage::FileSystem::{
        FileRenameInformation, NtCreateFile, NtSetInformationFile, FILE_CREATE,
        FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_REPARSE_POINT, FILE_RENAME_INFORMATION,
        FILE_SYNCHRONOUS_IO_NONALERT,
    };
    use windows_sys::Win32::Foundation::{
        RtlNtStatusToDosError, ERROR_NO_MORE_FILES, HANDLE, NTSTATUS, OBJ_CASE_INSENSITIVE,
        UNICODE_STRING,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        FileDispositionInfo, FileFullDirectoryInfo, FileFullDirectoryRestartInfo,
        GetFileInformationByHandleEx, SetFileInformationByHandle, DELETE, FILE_ATTRIBUTE_NORMAL,
        FILE_DISPOSITION_INFO, FILE_FULL_DIR_INFO, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
        FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, FILE_TRAVERSE, SYNCHRONIZE,
    };
    use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;

    /// Others may still read and write a held entry, but never rename or delete it.
    pub(super) const SHARE_ALL_BUT_DELETE: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE;

    /// Open the entry `name` directly inside `parent` as a folder to hold.
    pub(super) fn open_folder(parent: &File, name: &str) -> io::Result<File> {
        open_relative(
            parent,
            name,
            FILE_LIST_DIRECTORY | FILE_TRAVERSE | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            SHARE_ALL_BUT_DELETE,
            FILE_OPEN,
            0,
            0,
        )
    }

    /// Open the entry `name` directly inside `folder` for reading.
    pub(super) fn open_entry(folder: &File, name: &str) -> io::Result<File> {
        open_relative(
            folder,
            name,
            FILE_GENERIC_READ,
            FILE_SHARE_READ,
            FILE_OPEN,
            0,
            0,
        )
    }

    /// Whether any entry is named `name` directly inside `folder`. Reading only its attributes
    /// shares everything, so this never gets in anyone's way.
    pub(super) fn exists(folder: &File, name: &str) -> io::Result<bool> {
        match open_relative(
            folder,
            name,
            FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_OPEN,
            0,
            0,
        ) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// The names of all entries directly inside `folder`, read through its handle from the start.
    pub(super) fn list(folder: &File) -> io::Result<Vec<OsString>> {
        // Whole u64 words keep every record in the buffer aligned.
        let mut buffer = vec![0_u64; 8 * 1024];
        let buffer_bytes = buffer.len() * size_of::<u64>();
        let mut names = Vec::new();
        let mut class = FileFullDirectoryRestartInfo;
        loop {
            // SAFETY: the buffer is writable for `buffer_bytes` bytes and outlives the call.
            let listed = unsafe {
                GetFileInformationByHandleEx(
                    folder.as_raw_handle(),
                    class,
                    buffer.as_mut_ptr().cast(),
                    buffer_bytes as u32,
                )
            };
            if listed == 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() == Some(ERROR_NO_MORE_FILES as i32) {
                    return Ok(names);
                }
                return Err(error);
            }
            class = FileFullDirectoryInfo;
            let start = buffer.as_ptr().cast::<u8>();
            let mut offset = 0_usize;
            loop {
                // SAFETY: the call filled the buffer with a chain of complete records, each
                // aligned and starting `NextEntryOffset` bytes after the one before. Fields are
                // read through raw pointers, so no reference outlives a record's bytes.
                let (next, name) = unsafe {
                    let record = start.add(offset).cast::<FILE_FULL_DIR_INFO>();
                    let next = ptr::addr_of!((*record).NextEntryOffset).read();
                    let length = ptr::addr_of!((*record).FileNameLength).read() as usize;
                    let name = std::slice::from_raw_parts(
                        ptr::addr_of!((*record).FileName).cast::<u16>(),
                        length / size_of::<u16>(),
                    );
                    (next, OsString::from_wide(name))
                };
                if name != "." && name != ".." {
                    names.push(name);
                }
                if next == 0 {
                    break;
                }
                offset += next as usize;
                if offset >= buffer_bytes {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "a folder listing record lies outside its buffer",
                    ));
                }
            }
        }
    }

    /// Create the file `name` directly inside `folder`, failing if any entry has that name.
    pub(super) fn create_new_file(folder: &File, name: &str) -> io::Result<File> {
        open_relative(
            folder,
            name,
            FILE_GENERIC_WRITE | DELETE,
            0,
            FILE_CREATE,
            FILE_ATTRIBUTE_NORMAL,
            FILE_NON_DIRECTORY_FILE,
        )
    }

    /// Open one plain `name` relative to the `parent` handle itself. Nothing above `parent` is
    /// resolved again, and a link named `name` is opened as the link, not followed.
    fn open_relative(
        parent: &File,
        name: &str,
        access: u32,
        share: u32,
        disposition: u32,
        attributes: u32,
        options: u32,
    ) -> io::Result<File> {
        let mut wide: Vec<u16> = name.encode_utf16().collect();
        let length = u16::try_from(wide.len() * size_of::<u16>())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "name is too long"))?;
        let object_name = UNICODE_STRING {
            Length: length,
            MaximumLength: length,
            Buffer: wide.as_mut_ptr(),
        };
        let object_attributes = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: parent.as_raw_handle(),
            ObjectName: &object_name,
            // Match names the way Win32 opens do.
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: ptr::null(),
            SecurityQualityOfService: ptr::null(),
        };
        let mut handle: HANDLE = ptr::null_mut();
        let mut status_block = IO_STATUS_BLOCK::default();
        // SAFETY: every pointer refers to a local that outlives the call, and the name buffer
        // holds exactly `length` bytes.
        let status = unsafe {
            NtCreateFile(
                &mut handle,
                access,
                &object_attributes,
                &mut status_block,
                ptr::null(),
                attributes,
                share,
                disposition,
                options | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
                ptr::null(),
                0,
            )
        };
        if status < 0 {
            return Err(status_error(status));
        }
        // SAFETY: the call succeeded, so `handle` is a new handle that nothing else owns.
        Ok(unsafe { File::from_raw_handle(handle) })
    }

    /// Give `file` the plain `name` directly inside `folder`. An existing entry is replaced only
    /// when `replace` is set. The NT call takes the name relative to the folder handle; the Win32
    /// wrapper would resolve a plain name against the current directory instead.
    pub(super) fn rename_into(
        file: &File,
        folder: &File,
        name: &str,
        replace: bool,
    ) -> io::Result<()> {
        let wide: Vec<u16> = name.encode_utf16().collect();
        let name_bytes = wide.len() * size_of::<u16>();
        let size = size_of::<FILE_RENAME_INFORMATION>() + name_bytes;
        // Whole u64 words keep the buffer aligned for FILE_RENAME_INFORMATION.
        let mut buffer = vec![0_u64; size.div_ceil(size_of::<u64>())];
        let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
        // SAFETY: the buffer is aligned and large enough for the header and the whole name.
        unsafe {
            (*info).Anonymous.ReplaceIfExists = replace;
            (*info).RootDirectory = folder.as_raw_handle();
            (*info).FileNameLength = name_bytes as u32;
            ptr::copy_nonoverlapping(
                wide.as_ptr(),
                ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
                wide.len(),
            );
        }
        let mut status_block = IO_STATUS_BLOCK::default();
        // SAFETY: `info` points to `size` initialized bytes, and both handles stay open for the
        // whole call.
        let status = unsafe {
            NtSetInformationFile(
                file.as_raw_handle(),
                &mut status_block,
                info.cast_const().cast(),
                size as u32,
                FileRenameInformation,
            )
        };
        if status < 0 {
            return Err(status_error(status));
        }
        Ok(())
    }

    fn status_error(status: NTSTATUS) -> io::Error {
        // SAFETY: only converts a status code.
        let code = unsafe { RtlNtStatusToDosError(status) };
        io::Error::from_raw_os_error(code as i32)
    }

    /// Delete `file` when its last handle closes.
    pub(super) fn delete_on_close(file: &File) -> io::Result<()> {
        let info = FILE_DISPOSITION_INFO { DeleteFile: true };
        // SAFETY: `info` is a complete FILE_DISPOSITION_INFO that outlives the call.
        let marked = unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle(),
                FileDispositionInfo,
                ptr::from_ref(&info).cast(),
                size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        };
        if marked == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

/// Off Windows, which the installer does not support, links are refused by path.
#[cfg(not(windows))]
impl HeldFolder {
    fn open_root(target_root: &Path) -> Result<Self, EntryError> {
        require_direct_directory(target_root)?;
        Ok(Self {
            path: target_root.to_path_buf(),
        })
    }

    pub(crate) fn descend(mut self, name: &str) -> Result<Self, EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        require_direct_directory(&path)?;
        self.path = path;
        Ok(self)
    }

    pub(crate) fn open_file(&self, name: &str) -> Result<(fs::File, Metadata), EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        // Opening a link, or a FIFO that would block, is never needed.
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| EntryError::io(path.clone(), error))?;
        if !metadata.is_file() {
            return Err(EntryError::not_direct(path));
        }
        let file = fs::File::open(&path).map_err(|error| EntryError::io(path.clone(), error))?;
        Ok((file, metadata))
    }

    pub(crate) fn contains(&self, name: &str) -> io::Result<bool> {
        check_name(name)?;
        match fs::symlink_metadata(self.path.join(name)) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn names(&self) -> io::Result<Vec<OsString>> {
        fs::read_dir(&self.path)?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect()
    }

    pub(crate) fn create_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        check_name(name)?;
        self.write_temporary_file(content)?
            .persist_noclobber(self.path.join(name))
            .map_err(|error| error.error)?;
        Ok(())
    }

    pub(crate) fn replace_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        check_name(name)?;
        self.write_temporary_file(content)?
            .persist(self.path.join(name))
            .map_err(|error| error.error)?;
        Ok(())
    }

    fn write_temporary_file(&self, content: &[u8]) -> io::Result<tempfile::NamedTempFile> {
        let mut temporary = tempfile::NamedTempFile::new_in(&self.path)?;
        temporary.write_all(content)?;
        temporary.as_file().sync_all()?;
        Ok(temporary)
    }
}

#[cfg(not(windows))]
fn require_direct_directory(path: &Path) -> Result<(), EntryError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| EntryError::io(path.to_path_buf(), error))?;
    if !metadata.is_dir() || is_link_or_reparse(&metadata) {
        return Err(EntryError::not_direct(path.to_path_buf()));
    }
    Ok(())
}

/// Only a single plain name is opened relative to a folder: anything else would resolve further
/// names, following links in them, or name a stream.
fn check_name(name: &str) -> io::Result<()> {
    if name.is_empty() || name.contains(['/', '\\', ':', '\0']) || name.ends_with(['.', ' ']) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{name:?} is not a single plain file or folder name"),
        ));
    }
    Ok(())
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
pub(crate) mod tests {
    use super::*;

    const CONTENT: &[u8] = b"OUTER_SET party_movement_speed_value = 125\n";
    const OTHER: &[u8] = b"OUTER_SET party_movement_speed_value = 200\n";

    /// The names in `folder`, sorted.
    pub(crate) fn entries(folder: &Path) -> Vec<String> {
        let mut names = fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Make `link` a junction to the folder `target`, as an unprivileged user can.
    #[cfg(windows)]
    pub(crate) fn junction(link: &Path, target: &Path) {
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

    fn kind(error: &EntryError) -> &'static str {
        match error.kind {
            EntryErrorKind::Missing => "missing",
            EntryErrorKind::NotDirect => "not direct",
            EntryErrorKind::Io(_) => "io",
        }
    }

    fn unwrap_error<T>(result: Result<T, EntryError>) -> EntryError {
        match result {
            Ok(_) => panic!("expected an error"),
            Err(error) => error,
        }
    }

    #[test]
    fn creating_never_replaces_an_entry_and_leaves_no_temporary_file() {
        let root = tempfile::tempdir().unwrap();
        let folder_path = root.path().join("mods");
        fs::create_dir(&folder_path).unwrap();
        let folder = HeldFolder::open(root.path(), ["mods"]).unwrap();

        folder.create_file("config.txt", CONTENT).unwrap();
        let error = folder.create_file("config.txt", OTHER).unwrap_err();
        drop(folder);

        assert_eq!(error.kind(), ErrorKind::AlreadyExists, "{error}");
        assert_eq!(entries(&folder_path), ["config.txt"]);
        assert_eq!(fs::read(folder_path.join("config.txt")).unwrap(), CONTENT);
    }

    #[test]
    fn replacing_swaps_the_file_in_one_step_and_leaves_no_temporary_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("macros.tph");
        let folder = HeldFolder::open(root.path(), []).unwrap();

        // Without a file it is created; with one it is replaced.
        folder.replace_file("macros.tph", OTHER).unwrap();
        assert_eq!(fs::read(&path).unwrap(), OTHER);
        folder.replace_file("macros.tph", CONTENT).unwrap();
        drop(folder);

        assert_eq!(fs::read(&path).unwrap(), CONTENT);
        assert_eq!(entries(root.path()), ["macros.tph"]);
    }

    #[test]
    fn replacing_never_replaces_a_directory() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("macros.tph/inside")).unwrap();
        let folder = HeldFolder::open(root.path(), []).unwrap();

        assert!(folder.replace_file("macros.tph", CONTENT).is_err());
        drop(folder);

        assert_eq!(entries(root.path()), ["macros.tph"]);
        assert_eq!(entries(&root.path().join("macros.tph")), ["inside"]);
    }

    #[test]
    fn opens_only_direct_regular_files() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("file.txt"), CONTENT).unwrap();
        fs::create_dir(root.path().join("folder")).unwrap();
        let folder = HeldFolder::open(root.path(), []).unwrap();

        let (file, metadata) = folder.open_file("file.txt").unwrap();
        assert_eq!(metadata.len(), CONTENT.len() as u64);
        assert_eq!(
            crate::digest::sha256_reader(&file).unwrap(),
            crate::digest::sha256_bytes(CONTENT)
        );
        let error = unwrap_error(folder.open_file("folder"));
        assert_eq!(kind(&error), "not direct");
        assert_eq!(error.path, root.path().join("folder"));
        let error = unwrap_error(folder.open_file("absent.txt"));
        assert_eq!(kind(&error), "missing");
        assert_eq!(error.path, root.path().join("absent.txt"));
    }

    #[test]
    fn contains_reports_any_entry() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("file.txt"), CONTENT).unwrap();
        fs::create_dir(root.path().join("folder")).unwrap();
        let folder = HeldFolder::open(root.path(), []).unwrap();

        assert!(folder.contains("file.txt").unwrap());
        assert!(folder.contains("folder").unwrap());
        assert!(!folder.contains("absent.txt").unwrap());
        // Windows matches names without case, as programs opening them there see it.
        #[cfg(windows)]
        assert!(folder.contains("FILE.TXT").unwrap());
    }

    #[test]
    fn names_lists_every_entry_as_spelled() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("File.TXT"), CONTENT).unwrap();
        fs::create_dir(root.path().join("Folder")).unwrap();
        let folder = HeldFolder::open(root.path(), []).unwrap();

        let mut names = folder.names().unwrap();
        names.sort();
        assert_eq!(names, ["File.TXT", "Folder"]);
        // Listing again starts from the beginning.
        assert_eq!(folder.names().unwrap().len(), 2);
        let empty = HeldFolder::open(root.path(), ["Folder"]).unwrap();
        assert_eq!(empty.names().unwrap(), Vec::<OsString>::new());
    }

    /// More entries than one listing buffer holds are all returned.
    #[test]
    fn names_lists_a_folder_larger_than_one_buffer() {
        let root = tempfile::tempdir().unwrap();
        let expected = (0..1500)
            .map(|index| format!("entry-with-a-longer-name-{index:04}.txt"))
            .collect::<Vec<_>>();
        for name in &expected {
            fs::write(root.path().join(name), b"").unwrap();
        }
        let folder = HeldFolder::open(root.path(), []).unwrap();

        let mut names = folder
            .names()
            .unwrap()
            .into_iter()
            .map(|name| name.into_string().unwrap())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(names, expected);
    }

    #[test]
    fn requires_every_folder_and_creates_none() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("mods")).unwrap();
        fs::write(root.path().join("mods/file.txt"), CONTENT).unwrap();

        let error = unwrap_error(HeldFolder::open(root.path(), ["mods", "missing", "deeper"]));
        assert_eq!(kind(&error), "missing");
        assert_eq!(error.path, root.path().join("mods/missing"));
        let error = unwrap_error(HeldFolder::open(root.path(), ["mods", "file.txt"]));
        assert_eq!(kind(&error), "not direct");
        assert_eq!(error.path, root.path().join("mods/file.txt"));
        let error = unwrap_error(HeldFolder::open(&root.path().join("absent"), []));
        assert_eq!(kind(&error), "missing");
        let error = unwrap_error(HeldFolder::open(Path::new("relative"), []));
        assert_eq!(kind(&error), "io");

        assert_eq!(entries(root.path()), ["mods"]);
        assert_eq!(entries(&root.path().join("mods")), ["file.txt"]);
    }

    #[test]
    fn refuses_names_that_are_not_single_plain_names() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("mods/inner")).unwrap();
        fs::write(root.path().join("mods/inner/file.txt"), CONTENT).unwrap();

        for name in [
            "",
            ".",
            "..",
            "mods/inner",
            "mods\\inner",
            "file.txt:stream",
            "C:",
            "trailing.",
            "trailing ",
            "nul\0byte",
        ] {
            let error = unwrap_error(HeldFolder::open(root.path(), [name]));
            assert!(
                matches!(&error.kind, EntryErrorKind::Io(io) if io.kind() == ErrorKind::InvalidInput),
                "{name:?}: {error:?}"
            );
            let folder = HeldFolder::open(root.path(), ["mods"]).unwrap();
            let error = unwrap_error(folder.open_file(name));
            assert!(
                matches!(&error.kind, EntryErrorKind::Io(io) if io.kind() == ErrorKind::InvalidInput),
                "{name:?}: {error:?}"
            );
            for result in [
                folder.contains(name).map(|_| ()),
                folder.create_file(name, CONTENT),
                folder.replace_file(name, CONTENT),
            ] {
                assert_eq!(
                    result.unwrap_err().kind(),
                    ErrorKind::InvalidInput,
                    "{name:?}"
                );
            }
        }
        assert_eq!(entries(&root.path().join("mods")), ["inner"]);
        assert_eq!(entries(&root.path().join("mods/inner")), ["file.txt"]);
    }

    #[cfg(windows)]
    #[test]
    fn refuses_a_link_at_every_level_and_never_writes_through_one() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("game");
        let outside = root.path().join("outside");
        fs::create_dir_all(outside.join("lib")).unwrap();
        fs::create_dir_all(game.join("real/lib")).unwrap();
        junction(&root.path().join("linked-game"), &game);
        junction(&game.join("linked"), &outside);
        junction(&game.join("real").join("linked-lib"), &outside.join("lib"));

        // The target root, a folder in the middle and the last folder are junctions.
        for (target_root, folders, refused) in [
            (
                root.path().join("linked-game"),
                vec!["real", "lib"],
                root.path().join("linked-game"),
            ),
            (game.clone(), vec!["linked", "lib"], game.join("linked")),
            (
                game.clone(),
                vec!["real", "linked-lib"],
                game.join("real").join("linked-lib"),
            ),
        ] {
            let error = unwrap_error(HeldFolder::open(&target_root, folders));
            assert_eq!(kind(&error), "not direct", "{}", refused.display());
            assert_eq!(error.path, refused);
        }
        // An entry named like the file is a junction: it is neither opened nor written through.
        let folder = HeldFolder::open(&game, ["real"]).unwrap();
        let error = unwrap_error(folder.open_file("linked-lib"));
        assert_eq!(kind(&error), "not direct");
        assert!(folder.contains("linked-lib").unwrap());
        assert!(folder.replace_file("linked-lib", CONTENT).is_err());
        assert!(folder.create_file("linked-lib", CONTENT).is_err());
        drop(folder);

        assert_eq!(entries(&outside.join("lib")), Vec::<String>::new());
        assert_eq!(entries(&game.join("real")), ["lib", "linked-lib"]);
    }

    #[cfg(windows)]
    #[test]
    fn held_folders_cannot_be_moved_while_held() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("game");
        let outer = game.join("mods");
        let mod_folder = outer.join("cdtweaks");
        fs::create_dir_all(&mod_folder).unwrap();

        let folder = HeldFolder::open(&game, ["mods", "cdtweaks"]).unwrap();
        for held in [&mod_folder, &outer, &game] {
            let moved = held.with_file_name("moved");
            assert!(fs::rename(held, &moved).is_err(), "{}", held.display());
        }
        // Adding and replacing files inside the held folder still works.
        folder.create_file("cdtweaks.txt", OTHER).unwrap();
        folder.replace_file("cdtweaks.txt", CONTENT).unwrap();
        drop(folder);

        assert_eq!(entries(&mod_folder), ["cdtweaks.txt"]);
        assert_eq!(fs::read(mod_folder.join("cdtweaks.txt")).unwrap(), CONTENT);
        fs::rename(&mod_folder, outer.join("moved")).unwrap();
    }

    /// The bytes read are the bytes in place: while open, the file cannot be changed, replaced
    /// or removed.
    #[cfg(windows)]
    #[test]
    fn an_open_file_cannot_change_while_it_is_read() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("cdtweaks.txt");
        fs::write(&path, CONTENT).unwrap();
        let folder = HeldFolder::open(root.path(), []).unwrap();

        let (existing, _) = folder.open_file("cdtweaks.txt").unwrap();
        assert!(fs::write(&path, OTHER).is_err());
        assert!(fs::remove_file(&path).is_err());
        assert!(fs::rename(&path, path.with_file_name("moved.txt")).is_err());
        drop(existing);

        assert_eq!(fs::read(&path).unwrap(), CONTENT);
        fs::remove_file(&path).unwrap();
    }

    /// A link above the target root, such as a cloud-files folder or a volume mounted in a
    /// folder, is not inspected. Once the root is held, re-pointing such a link redirects every
    /// path through it, but every read and write stays in the folders that were checked.
    #[cfg(windows)]
    #[test]
    fn works_in_the_held_folders_after_a_link_above_the_root_is_repointed() {
        let root = tempfile::tempdir().unwrap();
        let checked = root.path().join("checked");
        let other = root.path().join("other");
        for tree in [&checked, &other] {
            fs::create_dir_all(tree.join("game/cdtweaks")).unwrap();
            fs::write(tree.join("game/cdtweaks/existing.txt"), OTHER).unwrap();
        }
        fs::write(other.join("game/cdtweaks/only-elsewhere.txt"), OTHER).unwrap();
        let link = root.path().join("link");
        junction(&link, &checked);

        let folder = HeldFolder::open(&link.join("game"), ["cdtweaks"]).unwrap();
        fs::remove_dir(&link).unwrap();
        junction(&link, &other);
        assert!(!folder.contains("only-elsewhere.txt").unwrap());
        assert_eq!(folder.names().unwrap(), ["existing.txt"]);
        folder.create_file("cdtweaks.txt", CONTENT).unwrap();
        folder.replace_file("existing.txt", CONTENT).unwrap();
        let (existing, _) = folder.open_file("existing.txt").unwrap();
        assert_eq!(
            crate::digest::sha256_reader(&existing).unwrap(),
            crate::digest::sha256_bytes(CONTENT)
        );
        drop(existing);
        drop(folder);

        let checked_mod = checked.join("game/cdtweaks");
        assert_eq!(entries(&checked_mod), ["cdtweaks.txt", "existing.txt"]);
        assert_eq!(fs::read(checked_mod.join("existing.txt")).unwrap(), CONTENT);
        let other_mod = other.join("game/cdtweaks");
        assert_eq!(entries(&other_mod), ["existing.txt", "only-elsewhere.txt"]);
        assert_eq!(fs::read(other_mod.join("existing.txt")).unwrap(), OTHER);
    }
}
