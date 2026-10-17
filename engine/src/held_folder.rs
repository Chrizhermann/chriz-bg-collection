//! Folders below a target root, opened one level at a time without following a link.
//!
//! On Windows the target root is opened by path, refusing a link or reparse point, and every
//! folder below it is opened relative to the handle of the folder above it, again without
//! following a link. The whole chain is held without delete sharing, so none of its folders can be
//! renamed, removed or replaced while it is held. Files are opened, created, renamed and removed
//! only through those handles. No path is resolved again once a folder was checked, so swapping a
//! folder for a junction in between cannot redirect a read, a write or a removal.
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

    /// Like [`Self::io`], for an open that asks for a folder: a file found there is not direct.
    fn folder_io(path: PathBuf, error: io::Error) -> Self {
        if is_not_a_directory(&error) {
            return Self::not_direct(path);
        }
        Self::io(path, error)
    }

    fn not_direct(path: PathBuf) -> Self {
        Self {
            path,
            kind: EntryErrorKind::NotDirect,
        }
    }
}

/// One entry of a held folder, as its listing reports it.
#[derive(Debug)]
pub(crate) struct ListedEntry {
    pub(crate) name: OsString,
    directory: bool,
    /// A link or other reparse point; off Windows also any entry that is neither a plain file
    /// nor a folder.
    reparse_point: bool,
}

/// A folder below a target root, opened from the root down. On Windows the target root and every
/// folder below it stay open while this is held.
pub(crate) struct HeldFolder {
    /// The target root and every folder below it down to this one, outermost first.
    #[cfg(windows)]
    chain: Vec<fs::File>,
    /// For messages only on Windows, where the handles decide where entries are.
    path: PathBuf,
    /// Whether this folder itself was opened so that it can be removed.
    removable: bool,
}

/// A direct regular file opened so that it can be removed; see [`HeldFolder::open_removable_file`].
pub(crate) struct RemovableFile {
    file: fs::File,
    path: PathBuf,
}

impl RemovableFile {
    /// The open file, for reading it before it is removed.
    pub(crate) fn file(&self) -> &fs::File {
        &self.file
    }

    /// For messages: the file, spelled as requested.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Remove the file this was opened for, even when it is read-only, as
    /// `std::fs::remove_file` does.
    pub(crate) fn remove(self) -> io::Result<()> {
        #[cfg(windows)]
        {
            handles::delete(&self.file, true)
        }
        #[cfg(not(windows))]
        {
            fs::remove_file(&self.path)
        }
    }
}

impl HeldFolder {
    /// Open the absolute `target_root`, then each of `folders` inside the one before. Each must be
    /// a direct directory: not a link, another reparse point or a file. No folder is created.
    pub(crate) fn open<'a>(
        target_root: &Path,
        folders: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, EntryError> {
        let mut folder = Self::open_root(require_absolute(target_root)?, false)?;
        for name in folders {
            folder = folder.descend(name)?;
        }
        Ok(folder)
    }

    /// Open the absolute `target_root` alone, like [`Self::open`], so that it can be removed with
    /// [`Self::remove`] once it is empty.
    pub(crate) fn open_for_removal(target_root: &Path) -> Result<Self, EntryError> {
        Self::open_root(require_absolute(target_root)?, true)
    }

    /// This folder, spelled as requested.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Open each of `folders` below this folder, each inside the one before. This folder stays as
    /// it is, and the folders above it stay held for the new one too.
    pub(crate) fn open_below<'a>(
        &self,
        folders: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, EntryError> {
        let mut folder = self.try_clone()?;
        for name in folders {
            folder = folder.descend(name)?;
        }
        Ok(folder)
    }

    /// Like [`Self::open_below`], but a folder that is missing is created. Nothing is created where
    /// a link, another reparse point or a file has the name.
    pub(crate) fn open_or_create_below<'a>(
        &self,
        folders: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, EntryError> {
        let mut folder = self.try_clone()?;
        for name in folders {
            folder = folder.descend_or_create(name)?;
        }
        Ok(folder)
    }

    /// The names of all entries in this folder, links included, listed through its handle.
    pub(crate) fn names(&self) -> io::Result<Vec<OsString>> {
        Ok(self
            .entries()?
            .into_iter()
            .map(|entry| entry.name)
            .collect())
    }

    /// Create the file `name` with `content`, never replacing anything already there.
    pub(crate) fn create_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        self.publish_file(self, name, content, false)
    }

    /// Put a file with `content` in place of `name` in one step, or create it. A directory with
    /// that name is not replaced, and a link with that name is never followed. A replaced file
    /// keeps the spelling it had.
    pub(crate) fn replace_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        self.publish_file(self, name, content, true)
    }

    /// Like [`Self::create_file`], but the file is written in `scratch`, another held folder on the
    /// same volume, and moved here only once it is complete. Nothing unfinished ever appears in
    /// this folder, not even after a crash.
    pub(crate) fn create_file_from(
        &self,
        scratch: &HeldFolder,
        name: &str,
        content: &[u8],
    ) -> io::Result<()> {
        self.publish_file(scratch, name, content, false)
    }

    /// Like [`Self::replace_file`], but the file is written in `scratch`, another held folder on
    /// the same volume, and moved here only once it is complete.
    pub(crate) fn replace_file_from(
        &self,
        scratch: &HeldFolder,
        name: &str,
        content: &[u8],
    ) -> io::Result<()> {
        self.publish_file(scratch, name, content, true)
    }
}

fn require_absolute(target_root: &Path) -> Result<&Path, EntryError> {
    if !target_root.is_absolute() {
        return Err(EntryError::io(
            target_root.to_path_buf(),
            io::Error::new(ErrorKind::InvalidInput, "the target root must be absolute"),
        ));
    }
    Ok(target_root)
}

/// The number of names `file` has; more than one means it shares its bytes with another path.
pub(crate) fn link_count(file: &fs::File) -> io::Result<u64> {
    #[cfg(windows)]
    {
        handles::link_count(file)
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(file.metadata()?.nlink())
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = file;
        Err(io::Error::new(
            ErrorKind::Unsupported,
            "link counts are not supported on this platform",
        ))
    }
}

#[cfg(windows)]
impl HeldFolder {
    fn open_root(target_root: &Path, removable: bool) -> Result<Self, EntryError> {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{
            DELETE, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_GENERIC_READ,
        };

        let path = target_root.to_path_buf();
        let mut options = fs::OpenOptions::new();
        if removable {
            options.access_mode(FILE_GENERIC_READ | DELETE);
        } else {
            options.read(true);
        }
        let root = options
            .share_mode(handles::SHARE_ALL_BUT_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(target_root)
            .map_err(|error| EntryError::io(path.clone(), error))?;
        require_direct_directory(&root, &path)?;
        Ok(Self {
            chain: vec![root],
            path,
            removable,
        })
    }

    /// The same folders, held through duplicated handles.
    fn try_clone(&self) -> Result<Self, EntryError> {
        let chain = self
            .chain
            .iter()
            .map(fs::File::try_clone)
            .collect::<io::Result<Vec<_>>>()
            .map_err(|error| EntryError::io(self.path.clone(), error))?;
        Ok(Self {
            chain,
            path: self.path.clone(),
            removable: self.removable,
        })
    }

    /// Open the folder `name` inside this one and hold it too.
    pub(crate) fn descend(mut self, name: &str) -> Result<Self, EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        let folder = handles::open_folder(self.folder(), &handles::wide(name))
            .map_err(|error| EntryError::io(path.clone(), error))?;
        require_direct_directory(&folder, &path)?;
        self.chain.push(folder);
        self.path = path;
        self.removable = false;
        Ok(self)
    }

    /// Open the folder `name` inside this one, creating it when nothing has that name, and hold it
    /// too.
    fn descend_or_create(mut self, name: &str) -> Result<Self, EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        let folder = handles::open_or_create_folder(self.folder(), &handles::wide(name))
            .map_err(|error| EntryError::folder_io(path.clone(), error))?;
        require_direct_directory(&folder, &path)?;
        self.chain.push(folder);
        self.path = path;
        self.removable = false;
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
        let file = handles::open_entry(self.folder(), &handles::wide(name))
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
        handles::exists(self.folder(), &handles::wide(name))
    }

    /// All entries in this folder, links included, listed through its handle.
    pub(crate) fn entries(&self) -> io::Result<Vec<ListedEntry>> {
        handles::list(self.folder())
    }

    /// Open the direct regular file `name` in this folder for reading, so that it can then be
    /// removed. While it is open, others can still read it but can neither change, rename nor
    /// remove it.
    pub(crate) fn open_removable_file(&self, name: &str) -> Result<RemovableFile, EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        let file = handles::open_removable_entry(self.folder(), &handles::wide(name), true)
            .map_err(|error| EntryError::io(path.clone(), error))?;
        let metadata = file
            .metadata()
            .map_err(|error| EntryError::io(path.clone(), error))?;
        if !metadata.is_file() || is_link_or_reparse(&metadata) {
            return Err(EntryError::not_direct(path));
        }
        Ok(RemovableFile { file, path })
    }

    /// Remove the empty direct folder `name` in this folder. A read-only folder is not removed, as
    /// with `std::fs::remove_dir`.
    pub(crate) fn remove_empty_folder(&self, name: &str) -> Result<(), EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        let folder = handles::open_removable_folder(self.folder(), &handles::wide(name))
            .map_err(|error| EntryError::folder_io(path.clone(), error))?;
        require_direct_directory(&folder, &path)?;
        handles::delete(&folder, false).map_err(|error| EntryError::io(path, error))
    }

    /// Remove this folder, which must be empty and opened with [`Self::open_for_removal`].
    pub(crate) fn remove(self) -> io::Result<()> {
        if !self.removable {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "the folder was not opened for removal",
            ));
        }
        handles::delete(self.folder(), false)
    }

    /// Remove everything inside this folder through the held handles, depth first. A link or other
    /// reparse point is refused, neither followed nor removed, and stops the removal. Entries named
    /// in `last` are removed after their siblings, in every folder. `before_remove` is called with
    /// the path of each file just before it is removed and of each folder once it is empty; an
    /// error from it stops the removal.
    pub(crate) fn remove_contents(
        &self,
        last: &[&str],
        before_remove: &mut dyn FnMut(&Path) -> io::Result<()>,
    ) -> Result<(), EntryError> {
        remove_contents_of(self.folder(), &self.path, last, before_remove)
    }

    /// The first link or other reparse point anywhere below this folder, found through the held
    /// handles without following one.
    pub(crate) fn find_link_below(&self) -> Result<Option<PathBuf>, EntryError> {
        find_link_below_of(self.folder(), &self.path)
    }

    /// The file identity of this folder, read from its own handle.
    pub(crate) fn identity(&self) -> io::Result<same_file::Handle> {
        same_file::Handle::from_file(self.folder().try_clone()?)
    }

    /// Only a complete file is ever published under `name`.
    fn publish_file(
        &self,
        scratch: &HeldFolder,
        name: &str,
        content: &[u8],
        replace: bool,
    ) -> io::Result<()> {
        check_name(name)?;
        let temporary = scratch.create_temporary_file()?;
        let published = (&temporary)
            .write_all(content)
            .and_then(|()| temporary.sync_all())
            .and_then(|()| {
                // A replacement keeps the spelling of the entry it replaces.
                let existing = if replace {
                    handles::existing_name(self.folder(), &handles::wide(name))?
                } else {
                    None
                };
                let target = existing.unwrap_or_else(|| handles::wide(name));
                handles::rename_into(&temporary, self.folder(), &target, replace)
            });
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
            match handles::create_new_file(self.folder(), &handles::wide(&name)) {
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

/// Entries named in `last` sort after their siblings, the others by name.
fn removal_order(entries: &mut [ListedEntry], last: &[&str]) {
    entries.sort_by(|left, right| {
        let is_last = |entry: &ListedEntry| last.iter().any(|name| entry.name == *name);
        (is_last(left), &left.name).cmp(&(is_last(right), &right.name))
    });
}

#[cfg(windows)]
fn remove_contents_of(
    folder: &fs::File,
    path: &Path,
    last: &[&str],
    before_remove: &mut dyn FnMut(&Path) -> io::Result<()>,
) -> Result<(), EntryError> {
    let mut entries =
        handles::list(folder).map_err(|error| EntryError::io(path.to_path_buf(), error))?;
    removal_order(&mut entries, last);
    for entry in entries {
        let child = path.join(&entry.name);
        if entry.reparse_point {
            return Err(EntryError::not_direct(child));
        }
        let name = handles::wide(&entry.name);
        if entry.directory {
            let inner = handles::open_removable_folder(folder, &name)
                .map_err(|error| EntryError::folder_io(child.clone(), error))?;
            require_direct_directory(&inner, &child)?;
            remove_contents_of(&inner, &child, last, before_remove)?;
            before_remove(&child).map_err(|error| EntryError::io(child.clone(), error))?;
            handles::delete(&inner, false).map_err(|error| EntryError::io(child, error))?;
        } else {
            let file = handles::open_removable_entry(folder, &name, false)
                .map_err(|error| EntryError::io(child.clone(), error))?;
            let metadata = file
                .metadata()
                .map_err(|error| EntryError::io(child.clone(), error))?;
            if !metadata.is_file() || is_link_or_reparse(&metadata) {
                return Err(EntryError::not_direct(child));
            }
            before_remove(&child).map_err(|error| EntryError::io(child.clone(), error))?;
            handles::delete(&file, true).map_err(|error| EntryError::io(child, error))?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn find_link_below_of(folder: &fs::File, path: &Path) -> Result<Option<PathBuf>, EntryError> {
    let mut entries =
        handles::list(folder).map_err(|error| EntryError::io(path.to_path_buf(), error))?;
    removal_order(&mut entries, &[]);
    for entry in entries {
        let child = path.join(&entry.name);
        if entry.reparse_point {
            return Ok(Some(child));
        }
        if !entry.directory {
            continue;
        }
        let inner = handles::open_folder(folder, &handles::wide(&entry.name))
            .map_err(|error| EntryError::io(child.clone(), error))?;
        let metadata = inner
            .metadata()
            .map_err(|error| EntryError::io(child.clone(), error))?;
        // It was replaced since it was listed.
        if is_link_or_reparse(&metadata) {
            return Ok(Some(child));
        }
        if metadata.is_dir() {
            if let Some(found) = find_link_below_of(&inner, &child)? {
                return Ok(Some(found));
            }
        }
    }
    Ok(None)
}

#[cfg(windows)]
fn is_not_a_directory(error: &io::Error) -> bool {
    use windows_sys::Win32::Foundation::ERROR_DIRECTORY;
    error.raw_os_error() == Some(ERROR_DIRECTORY as i32)
}

/// Opening and naming files relative to held folder handles.
#[cfg(windows)]
mod handles {
    use std::ffi::{OsStr, OsString};
    use std::fs::File;
    use std::io;
    use std::mem::size_of;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use std::ptr;

    use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
    use windows_sys::Wdk::Storage::FileSystem::{
        FileRenameInformation, NtCreateFile, NtSetInformationFile, FILE_CREATE,
        FILE_DIRECTORY_FILE, FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_IF,
        FILE_OPEN_REPARSE_POINT, FILE_RENAME_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT,
    };
    use windows_sys::Win32::Foundation::{
        RtlNtStatusToDosError, ERROR_INVALID_FUNCTION, ERROR_INVALID_PARAMETER,
        ERROR_NOT_SUPPORTED, ERROR_NO_MORE_FILES, HANDLE, NTSTATUS, OBJ_CASE_INSENSITIVE,
        UNICODE_STRING,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        FileDispositionInfo, FileDispositionInfoEx, FileFullDirectoryInfo,
        FileFullDirectoryRestartInfo, FileNameInfo, GetFileInformationByHandle,
        GetFileInformationByHandleEx, SetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT,
        FILE_DISPOSITION_FLAG_DELETE, FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE,
        FILE_DISPOSITION_FLAG_POSIX_SEMANTICS, FILE_DISPOSITION_INFO, FILE_DISPOSITION_INFO_EX,
        FILE_FULL_DIR_INFO, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_LIST_DIRECTORY,
        FILE_NAME_INFO, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FILE_TRAVERSE, SYNCHRONIZE,
    };
    use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;

    use super::ListedEntry;

    /// Others may still read and write a held entry, but never rename or delete it.
    pub(super) const SHARE_ALL_BUT_DELETE: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE;
    const SHARE_ALL: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    /// What a held folder is opened for: listing it and opening entries in it.
    const FOLDER_ACCESS: u32 =
        FILE_LIST_DIRECTORY | FILE_TRAVERSE | FILE_READ_ATTRIBUTES | SYNCHRONIZE;

    /// A name as the NT calls take it.
    pub(super) fn wide(name: impl AsRef<OsStr>) -> Vec<u16> {
        name.as_ref().encode_wide().collect()
    }

    /// Open the entry `name` directly inside `parent` as a folder to hold.
    pub(super) fn open_folder(parent: &File, name: &[u16]) -> io::Result<File> {
        open_relative(
            parent,
            name,
            FOLDER_ACCESS,
            SHARE_ALL_BUT_DELETE,
            FILE_OPEN,
            0,
            0,
        )
    }

    /// Open the folder `name` directly inside `parent` to hold it, creating it when nothing has
    /// that name. A file with the name fails the open instead.
    pub(super) fn open_or_create_folder(parent: &File, name: &[u16]) -> io::Result<File> {
        open_relative(
            parent,
            name,
            FOLDER_ACCESS,
            SHARE_ALL_BUT_DELETE,
            FILE_OPEN_IF,
            0,
            FILE_DIRECTORY_FILE,
        )
    }

    /// Open the folder `name` directly inside `parent` to hold it, so that it can be removed.
    pub(super) fn open_removable_folder(parent: &File, name: &[u16]) -> io::Result<File> {
        open_relative(
            parent,
            name,
            FOLDER_ACCESS | DELETE,
            SHARE_ALL_BUT_DELETE,
            FILE_OPEN,
            0,
            FILE_DIRECTORY_FILE,
        )
    }

    /// Open the entry `name` directly inside `folder` for reading.
    pub(super) fn open_entry(folder: &File, name: &[u16]) -> io::Result<File> {
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

    /// Open the entry `name` directly inside `folder` so that it can be removed. When it is read
    /// first, nobody else may change, rename or remove it meanwhile. Otherwise others may keep it
    /// open in any way, as with a plain delete.
    pub(super) fn open_removable_entry(
        folder: &File,
        name: &[u16],
        read: bool,
    ) -> io::Result<File> {
        let (access, share) = if read {
            (FILE_GENERIC_READ | DELETE, FILE_SHARE_READ)
        } else {
            (DELETE | FILE_READ_ATTRIBUTES | SYNCHRONIZE, SHARE_ALL)
        };
        open_relative(folder, name, access, share, FILE_OPEN, 0, 0)
    }

    /// Whether any entry is named `name` directly inside `folder`. Reading only its attributes
    /// shares everything, so this never gets in anyone's way.
    pub(super) fn exists(folder: &File, name: &[u16]) -> io::Result<bool> {
        match open_relative(
            folder,
            name,
            FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            SHARE_ALL,
            FILE_OPEN,
            0,
            0,
        ) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// The stored name of the entry that the file system finds for `name` directly inside
    /// `folder`, which can differ from `name` in case, or `None` if nothing has that name.
    pub(super) fn existing_name(folder: &File, name: &[u16]) -> io::Result<Option<Vec<u16>>> {
        let entry = match open_relative(
            folder,
            name,
            FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            SHARE_ALL,
            FILE_OPEN,
            0,
            0,
        ) {
            Ok(entry) => entry,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        // Room for the header and the longest path the file system allows.
        let buffer_bytes = size_of::<FILE_NAME_INFO>() + 32_768 * size_of::<u16>();
        // Whole u64 words keep the buffer aligned for FILE_NAME_INFO.
        let mut buffer = vec![0_u64; buffer_bytes.div_ceil(size_of::<u64>())];
        let buffer_bytes = buffer.len() * size_of::<u64>();
        // SAFETY: the buffer is writable for `buffer_bytes` bytes and outlives the call.
        let queried = unsafe {
            GetFileInformationByHandleEx(
                entry.as_raw_handle(),
                FileNameInfo,
                buffer.as_mut_ptr().cast(),
                buffer_bytes as u32,
            )
        };
        if queried == 0 {
            return Err(io::Error::last_os_error());
        }
        let info = buffer.as_ptr().cast::<FILE_NAME_INFO>();
        // SAFETY: the call filled in a complete FILE_NAME_INFO at the start of the buffer. The
        // name is read only as far as it lies inside the buffer.
        let path = unsafe {
            let name = ptr::addr_of!((*info).FileName).cast::<u16>();
            let available =
                (buffer_bytes - (name as usize - buffer.as_ptr() as usize)) / size_of::<u16>();
            let length = ptr::addr_of!((*info).FileNameLength).read() as usize / size_of::<u16>();
            if length > available {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "a file name lies outside its buffer",
                ));
            }
            std::slice::from_raw_parts(name, length).to_vec()
        };
        // The path runs from the root of the volume; its last part is the entry's own name.
        let start = path
            .iter()
            .rposition(|&unit| unit == u16::from(b'\\'))
            .map_or(0, |separator| separator + 1);
        Ok(Some(path[start..].to_vec()).filter(|stored| !stored.is_empty()))
    }

    /// All entries directly inside `folder`, read through its handle from the start.
    pub(super) fn list(folder: &File) -> io::Result<Vec<ListedEntry>> {
        // Whole u64 words keep every record in the buffer aligned.
        let mut buffer = vec![0_u64; 8 * 1024];
        let buffer_bytes = buffer.len() * size_of::<u64>();
        let mut entries = Vec::new();
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
                    return Ok(entries);
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
                let (next, attributes, name) = unsafe {
                    let record = start.add(offset).cast::<FILE_FULL_DIR_INFO>();
                    let next = ptr::addr_of!((*record).NextEntryOffset).read();
                    let attributes = ptr::addr_of!((*record).FileAttributes).read();
                    let length = ptr::addr_of!((*record).FileNameLength).read() as usize;
                    let name = std::slice::from_raw_parts(
                        ptr::addr_of!((*record).FileName).cast::<u16>(),
                        length / size_of::<u16>(),
                    );
                    (next, attributes, OsString::from_wide(name))
                };
                if name != "." && name != ".." {
                    entries.push(ListedEntry {
                        name,
                        directory: attributes & FILE_ATTRIBUTE_DIRECTORY != 0,
                        reparse_point: attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0,
                    });
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
    pub(super) fn create_new_file(folder: &File, name: &[u16]) -> io::Result<File> {
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
        name: &[u16],
        access: u32,
        share: u32,
        disposition: u32,
        attributes: u32,
        options: u32,
    ) -> io::Result<File> {
        // The call takes a mutable buffer, though it does not write to it.
        let mut wide = name.to_vec();
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
        name: &[u16],
        replace: bool,
    ) -> io::Result<()> {
        let name_bytes = std::mem::size_of_val(name);
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
                name.as_ptr(),
                ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
                name.len(),
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

    /// Delete the entry `file` was opened as, which must have been opened with delete access. Its
    /// name goes away at once, even while others still have it open. A read-only file is deleted
    /// only with `ignore_read_only`, which `std::fs::remove_file` also does; `std::fs::remove_dir`
    /// does not remove a read-only folder.
    pub(super) fn delete(file: &File, ignore_read_only: bool) -> io::Result<()> {
        let mut flags = FILE_DISPOSITION_FLAG_DELETE | FILE_DISPOSITION_FLAG_POSIX_SEMANTICS;
        if ignore_read_only {
            flags |= FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE;
        }
        let info = FILE_DISPOSITION_INFO_EX { Flags: flags };
        // SAFETY: `info` is a complete FILE_DISPOSITION_INFO_EX that outlives the call.
        let marked = unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle(),
                FileDispositionInfoEx,
                ptr::from_ref(&info).cast(),
                size_of::<FILE_DISPOSITION_INFO_EX>() as u32,
            )
        };
        if marked != 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        // A file system without these deletes, such as FAT, takes only the plain request: the name
        // goes away once the last handle is closed.
        let unsupported = [
            ERROR_INVALID_PARAMETER,
            ERROR_NOT_SUPPORTED,
            ERROR_INVALID_FUNCTION,
        ]
        .iter()
        .any(|code| error.raw_os_error() == Some(*code as i32));
        if unsupported {
            return delete_on_close(file);
        }
        Err(error)
    }

    /// The number of names the file `file` refers to has.
    pub(super) fn link_count(file: &File) -> io::Result<u64> {
        let mut information = BY_HANDLE_FILE_INFORMATION::default();
        // SAFETY: `information` is a complete BY_HANDLE_FILE_INFORMATION that outlives the call.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(u64::from(information.nNumberOfLinks))
    }
}

/// Off Windows, which the installer does not support, links are refused by path.
#[cfg(not(windows))]
impl HeldFolder {
    fn open_root(target_root: &Path, removable: bool) -> Result<Self, EntryError> {
        require_direct_directory(target_root)?;
        Ok(Self {
            path: target_root.to_path_buf(),
            removable,
        })
    }

    fn try_clone(&self) -> Result<Self, EntryError> {
        Ok(Self {
            path: self.path.clone(),
            removable: self.removable,
        })
    }

    pub(crate) fn descend(mut self, name: &str) -> Result<Self, EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        require_direct_directory(&path)?;
        self.path = path;
        self.removable = false;
        Ok(self)
    }

    fn descend_or_create(mut self, name: &str) -> Result<Self, EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        match fs::create_dir(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(EntryError::folder_io(path, error)),
        }
        require_direct_directory(&path)?;
        self.path = path;
        self.removable = false;
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

    pub(crate) fn entries(&self) -> io::Result<Vec<ListedEntry>> {
        fs::read_dir(&self.path)?
            .map(|entry| {
                let entry = entry?;
                let file_type = entry.file_type()?;
                Ok(ListedEntry {
                    name: entry.file_name(),
                    directory: file_type.is_dir(),
                    // A link, or anything else that is neither a plain file nor a folder.
                    reparse_point: !file_type.is_dir() && !file_type.is_file(),
                })
            })
            .collect()
    }

    pub(crate) fn open_removable_file(&self, name: &str) -> Result<RemovableFile, EntryError> {
        let (file, _) = self.open_file(name)?;
        Ok(RemovableFile {
            file,
            path: self.path.join(name),
        })
    }

    pub(crate) fn remove_empty_folder(&self, name: &str) -> Result<(), EntryError> {
        let path = self.path.join(name);
        check_name(name).map_err(|error| EntryError::io(path.clone(), error))?;
        require_direct_directory(&path)?;
        fs::remove_dir(&path).map_err(|error| EntryError::io(path, error))
    }

    pub(crate) fn remove(self) -> io::Result<()> {
        if !self.removable {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "the folder was not opened for removal",
            ));
        }
        fs::remove_dir(&self.path)
    }

    pub(crate) fn remove_contents(
        &self,
        last: &[&str],
        before_remove: &mut dyn FnMut(&Path) -> io::Result<()>,
    ) -> Result<(), EntryError> {
        let mut entries = self
            .entries()
            .map_err(|error| EntryError::io(self.path.clone(), error))?;
        removal_order(&mut entries, last);
        for entry in entries {
            let child = self.path.join(&entry.name);
            if entry.reparse_point {
                return Err(EntryError::not_direct(child));
            }
            if entry.directory {
                require_direct_directory(&child)?;
                let inner = Self {
                    path: child.clone(),
                    removable: false,
                };
                inner.remove_contents(last, before_remove)?;
                before_remove(&child).map_err(|error| EntryError::io(child.clone(), error))?;
                fs::remove_dir(&child).map_err(|error| EntryError::io(child, error))?;
            } else {
                before_remove(&child).map_err(|error| EntryError::io(child.clone(), error))?;
                fs::remove_file(&child).map_err(|error| EntryError::io(child, error))?;
            }
        }
        Ok(())
    }

    pub(crate) fn find_link_below(&self) -> Result<Option<PathBuf>, EntryError> {
        let mut entries = self
            .entries()
            .map_err(|error| EntryError::io(self.path.clone(), error))?;
        removal_order(&mut entries, &[]);
        for entry in entries {
            let child = self.path.join(&entry.name);
            if entry.reparse_point {
                return Ok(Some(child));
            }
            if entry.directory {
                let inner = Self {
                    path: child,
                    removable: false,
                };
                if let Some(found) = inner.find_link_below()? {
                    return Ok(Some(found));
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn identity(&self) -> io::Result<same_file::Handle> {
        same_file::Handle::from_path(&self.path)
    }

    fn publish_file(
        &self,
        scratch: &HeldFolder,
        name: &str,
        content: &[u8],
        replace: bool,
    ) -> io::Result<()> {
        check_name(name)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&scratch.path)?;
        temporary.write_all(content)?;
        temporary.as_file().sync_all()?;
        let target = self.path.join(name);
        if replace {
            temporary.persist(target).map_err(|error| error.error)?;
        } else {
            temporary
                .persist_noclobber(target)
                .map_err(|error| error.error)?;
        }
        Ok(())
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

#[cfg(not(windows))]
fn is_not_a_directory(error: &io::Error) -> bool {
    error.kind() == ErrorKind::NotADirectory
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

    /// Make `folder` a new empty folder that is a reparse point of a tag no filter handles, as a
    /// third-party storage product might leave one: not a link, but no plain folder either.
    #[cfg(windows)]
    pub(crate) fn foreign_reparse_point(folder: &Path) {
        use std::os::windows::fs::OpenOptionsExt;
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_WRITE_ATTRIBUTES,
            FILE_WRITE_DATA,
        };
        use windows_sys::Win32::System::IO::DeviceIoControl;
        const FSCTL_SET_REPARSE_POINT: u32 = 0x0009_00A4;
        // A tag without the Microsoft bit, which needs no privilege to set.
        const TAG: u32 = 0x0000_7A11;
        const GUID: [u8; 16] = [
            0x5c, 0x2b, 0x1f, 0x3e, 0x9a, 0x47, 0x4d, 0x11, 0x8e, 0x21, 0x6a, 0x50, 0x0c, 0x77,
            0x13, 0x42,
        ];

        fs::create_dir(folder).unwrap();
        let handle = fs::OpenOptions::new()
            .access_mode(FILE_WRITE_DATA | FILE_WRITE_ATTRIBUTES)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(folder)
            .unwrap();
        // REPARSE_GUID_DATA_BUFFER without data: tag, data length, reserved, GUID.
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&TAG.to_le_bytes());
        buffer.extend_from_slice(&[0; 4]);
        buffer.extend_from_slice(&GUID);
        let mut returned = 0_u32;
        // SAFETY: the input buffer is valid for its length, no output buffer is passed, and the
        // handle stays open for the call.
        let set = unsafe {
            DeviceIoControl(
                handle.as_raw_handle(),
                FSCTL_SET_REPARSE_POINT,
                buffer.as_ptr().cast(),
                buffer.len() as u32,
                std::ptr::null_mut(),
                0,
                &mut returned,
                std::ptr::null_mut(),
            )
        };
        assert_ne!(set, 0, "{}", io::Error::last_os_error());
    }

    /// Copy the folder `from` and everything in it to the new folder `to`.
    pub(crate) fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), &target).unwrap();
            }
        }
    }

    /// Every file below `root` with its bytes, by path relative to `root` with `/` separators.
    pub(crate) fn tree(root: &Path) -> Vec<(String, Vec<u8>)> {
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(root).follow_links(false) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let name = entry
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((name, fs::read(entry.path()).unwrap()));
            }
        }
        files.sort();
        files
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

    /// A file replaced under another spelling keeps the one it had, as an overwrite in place
    /// would; a new file gets the requested spelling.
    #[cfg(windows)]
    #[test]
    fn replacing_keeps_the_spelling_of_the_replaced_file() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("BGCLATXT.2DA"), OTHER).unwrap();
        let folder = HeldFolder::open(root.path(), []).unwrap();

        folder.replace_file("bgclatxt.2da", CONTENT).unwrap();
        folder.replace_file("New.Txt", OTHER).unwrap();
        drop(folder);

        assert_eq!(entries(root.path()), ["BGCLATXT.2DA", "New.Txt"]);
        assert_eq!(fs::read(root.path().join("BGCLATXT.2DA")).unwrap(), CONTENT);
    }

    /// The file is written in the scratch folder and appears in its folder only once complete.
    /// Neither a publication nor a refused one leaves anything behind in either folder.
    #[test]
    fn files_written_in_a_scratch_folder_appear_complete_and_leave_nothing_behind() {
        let root = tempfile::tempdir().unwrap();
        let scratch_path = root.path().join(".chriz/record");
        let target = root.path().join("game/override");
        fs::create_dir_all(&scratch_path).unwrap();
        fs::create_dir_all(&target).unwrap();
        let held = HeldFolder::open(root.path(), []).unwrap();
        let scratch = held.open_below([".chriz", "record"]).unwrap();
        let folder = held.open_below(["game", "override"]).unwrap();

        folder
            .create_file_from(&scratch, "new.2da", CONTENT)
            .unwrap();
        let error = folder
            .create_file_from(&scratch, "new.2da", OTHER)
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::AlreadyExists, "{error}");
        folder
            .replace_file_from(&scratch, "new.2da", OTHER)
            .unwrap();
        folder
            .replace_file_from(&scratch, "weidu.log", CONTENT)
            .unwrap();
        assert!(folder
            .create_file_from(&scratch, "bad/name", CONTENT)
            .is_err());
        drop((scratch, folder, held));

        assert_eq!(entries(&scratch_path), Vec::<String>::new());
        assert_eq!(entries(&target), ["new.2da", "weidu.log"]);
        assert_eq!(fs::read(target.join("new.2da")).unwrap(), OTHER);
        assert_eq!(fs::read(target.join("weidu.log")).unwrap(), CONTENT);
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
        assert_eq!(link_count(&file).unwrap(), 1);
        let error = unwrap_error(folder.open_file("folder"));
        assert_eq!(kind(&error), "not direct");
        assert_eq!(error.path, root.path().join("folder"));
        let error = unwrap_error(folder.open_file("absent.txt"));
        assert_eq!(kind(&error), "missing");
        assert_eq!(error.path, root.path().join("absent.txt"));
    }

    #[test]
    fn link_count_counts_every_name() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("one.txt"), CONTENT).unwrap();
        fs::hard_link(root.path().join("one.txt"), root.path().join("two.txt")).unwrap();
        let folder = HeldFolder::open(root.path(), []).unwrap();

        let (file, _) = folder.open_file("two.txt").unwrap();
        assert_eq!(link_count(&file).unwrap(), 2);
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

    /// The listing tells folders, files and links apart without opening or following any.
    #[cfg(windows)]
    #[test]
    fn entries_tell_folders_files_and_links_apart() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("game");
        fs::create_dir_all(game.join("folder")).unwrap();
        fs::write(game.join("file.txt"), CONTENT).unwrap();
        junction(&game.join("link"), root.path());
        let folder = HeldFolder::open(&game, []).unwrap();

        let mut listed = folder
            .entries()
            .unwrap()
            .into_iter()
            .map(|entry| {
                (
                    entry.name.to_string_lossy().into_owned(),
                    entry.directory && !entry.reparse_point,
                    entry.reparse_point,
                )
            })
            .collect::<Vec<_>>();
        listed.sort();
        assert_eq!(
            listed,
            [
                ("file.txt".to_owned(), false, false),
                ("folder".to_owned(), true, false),
                ("link".to_owned(), false, true),
            ]
        );
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
        let error = unwrap_error(HeldFolder::open_for_removal(Path::new("relative")));
        assert_eq!(kind(&error), "io");
        let held = HeldFolder::open(root.path(), []).unwrap();
        let error = unwrap_error(held.open_below(["mods", "missing"]));
        assert_eq!(kind(&error), "missing");
        assert_eq!(error.path, root.path().join("mods/missing"));

        assert_eq!(entries(root.path()), ["mods"]);
        assert_eq!(entries(&root.path().join("mods")), ["file.txt"]);
    }

    /// Missing folders are created, existing ones opened, and a file where a folder belongs is
    /// refused without being replaced.
    #[test]
    fn opening_or_creating_creates_only_missing_folders() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("mods")).unwrap();
        fs::write(root.path().join("mods/existing.txt"), CONTENT).unwrap();
        fs::write(root.path().join("mods/blocker"), OTHER).unwrap();
        let held = HeldFolder::open(root.path(), []).unwrap();

        let created = held
            .open_or_create_below(["mods", "new", "deeper"])
            .unwrap();
        assert_eq!(created.path(), root.path().join("mods/new/deeper"));
        created.create_file("file.txt", CONTENT).unwrap();
        let reopened = held.open_or_create_below(["mods", "new"]).unwrap();
        assert_eq!(reopened.names().unwrap(), ["deeper"]);
        let error = unwrap_error(held.open_or_create_below(["mods", "blocker", "inside"]));
        assert_eq!(kind(&error), "not direct");
        assert_eq!(error.path, root.path().join("mods/blocker"));
        drop((created, reopened, held));

        assert_eq!(
            entries(&root.path().join("mods")),
            ["blocker", "existing.txt", "new"]
        );
        assert_eq!(fs::read(root.path().join("mods/blocker")).unwrap(), OTHER);
        assert_eq!(
            fs::read(root.path().join("mods/new/deeper/file.txt")).unwrap(),
            CONTENT
        );
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
            for result in [
                folder.open_file(name).map(|_| ()),
                folder.open_removable_file(name).map(|_| ()),
                folder.remove_empty_folder(name),
                folder.open_below([name]).map(|_| ()),
                folder.open_or_create_below([name]).map(|_| ()),
            ] {
                let error = unwrap_error(result);
                assert!(
                    matches!(&error.kind, EntryErrorKind::Io(io) if io.kind() == ErrorKind::InvalidInput),
                    "{name:?}: {error:?}"
                );
            }
            for result in [
                folder.contains(name).map(|_| ()),
                folder.create_file(name, CONTENT),
                folder.replace_file(name, CONTENT),
                folder.create_file_from(&folder, name, CONTENT),
                folder.replace_file_from(&folder, name, CONTENT),
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
            let error = unwrap_error(HeldFolder::open(&target_root, folders.clone()));
            assert_eq!(kind(&error), "not direct", "{}", refused.display());
            assert_eq!(error.path, refused);
            if target_root == game {
                let held = HeldFolder::open(&game, []).unwrap();
                for result in [
                    held.open_below(folders.clone()).map(|_| ()),
                    held.open_or_create_below(folders.clone()).map(|_| ()),
                    held.open_or_create_below(folders.iter().copied().chain(["new"]))
                        .map(|_| ()),
                ] {
                    let error = unwrap_error(result);
                    assert_eq!(kind(&error), "not direct", "{}", refused.display());
                    assert_eq!(error.path, refused);
                }
            }
        }
        // A target root that is a junction is not opened for removal either.
        for linked_root in [root.path().join("linked-game"), game.join("linked")] {
            let error = unwrap_error(HeldFolder::open_for_removal(&linked_root));
            assert_eq!(kind(&error), "not direct", "{}", linked_root.display());
            assert_eq!(error.path, linked_root);
        }
        // An entry named like the file is a junction: it is neither opened nor written through.
        let folder = HeldFolder::open(&game, ["real"]).unwrap();
        let error = unwrap_error(folder.open_file("linked-lib"));
        assert_eq!(kind(&error), "not direct");
        let error = unwrap_error(folder.open_removable_file("linked-lib"));
        assert_eq!(kind(&error), "not direct");
        let error = unwrap_error(folder.remove_empty_folder("linked-lib"));
        assert_eq!(kind(&error), "not direct");
        assert!(folder.contains("linked-lib").unwrap());
        assert!(folder.replace_file("linked-lib", CONTENT).is_err());
        assert!(folder.create_file("linked-lib", CONTENT).is_err());
        assert!(folder
            .replace_file_from(&folder, "linked-lib", CONTENT)
            .is_err());
        drop(folder);

        assert_eq!(entries(&outside), ["lib"]);
        assert_eq!(entries(&outside.join("lib")), Vec::<String>::new());
        assert_eq!(entries(&game.join("real")), ["lib", "linked-lib"]);
    }

    /// A reparse point that is no link, such as one a third-party storage product leaves, is no
    /// plain folder either: it is refused wherever a held folder is opened, and never removed.
    #[cfg(windows)]
    #[test]
    fn refuses_a_reparse_point_that_is_no_link_at_every_level() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("game");
        fs::create_dir_all(game.join("real")).unwrap();
        foreign_reparse_point(&root.path().join("tagged-root"));
        foreign_reparse_point(&game.join("tagged"));
        foreign_reparse_point(&game.join("real").join("tagged"));
        // Plain metadata calls it a folder, not a link.
        let metadata = fs::symlink_metadata(game.join("tagged")).unwrap();
        assert!(metadata.is_dir() && !metadata.file_type().is_symlink());

        for (target_root, folders, refused) in [
            (
                root.path().join("tagged-root"),
                vec![],
                root.path().join("tagged-root"),
            ),
            (game.clone(), vec!["tagged"], game.join("tagged")),
            (
                game.clone(),
                vec!["real", "tagged"],
                game.join("real").join("tagged"),
            ),
        ] {
            let error = unwrap_error(HeldFolder::open(&target_root, folders.clone()));
            assert_eq!(kind(&error), "not direct", "{}", refused.display());
            assert_eq!(error.path, refused);
        }
        let error = unwrap_error(HeldFolder::open_for_removal(
            &root.path().join("tagged-root"),
        ));
        assert_eq!(kind(&error), "not direct");
        let held = HeldFolder::open(&game, []).unwrap();
        for result in [
            held.open_below(["tagged"]).map(|_| ()),
            held.open_or_create_below(["tagged", "new"]).map(|_| ()),
            held.open_file("tagged").map(|_| ()),
            held.open_removable_file("tagged").map(|_| ()),
            held.remove_empty_folder("tagged"),
        ] {
            let error = unwrap_error(result);
            assert_eq!(kind(&error), "not direct");
            assert_eq!(error.path, game.join("tagged"));
        }
        assert!(held.replace_file("tagged", CONTENT).is_err());
        let real = held.open_below(["real"]).unwrap();
        assert_eq!(
            held.find_link_below().unwrap(),
            Some(game.join("real").join("tagged"))
        );
        let error = unwrap_error(real.remove_contents(&[], &mut |_| Ok(())));
        assert_eq!(kind(&error), "not direct");
        drop((real, held));

        assert_eq!(entries(&game), ["real", "tagged"]);
        assert_eq!(entries(&game.join("real")), ["tagged"]);
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

    /// The file checked is the file removed: until then, nobody else can change, rename or remove
    /// it. A read-only file is removed as `std::fs::remove_file` removes it.
    #[cfg(windows)]
    #[test]
    fn a_file_opened_for_removal_cannot_change_until_it_is_removed() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("owned.tpa");
        fs::write(&path, CONTENT).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions).unwrap();
        fs::write(root.path().join("kept.tpa"), OTHER).unwrap();
        let folder = HeldFolder::open(root.path(), []).unwrap();

        let owned = folder.open_removable_file("owned.tpa").unwrap();
        assert_eq!(owned.path(), path);
        assert!(fs::rename(&path, path.with_file_name("moved.tpa")).is_err());
        assert!(fs::remove_file(&path).is_err());
        assert_eq!(
            crate::digest::sha256_reader(owned.file()).unwrap(),
            crate::digest::sha256_bytes(CONTENT)
        );
        owned.remove().unwrap();
        let error = unwrap_error(folder.open_removable_file("owned.tpa"));
        assert_eq!(kind(&error), "missing");
        drop(folder);

        assert_eq!(entries(root.path()), ["kept.tpa"]);
    }

    /// Only an empty direct folder is removed; a read-only one stays, as with
    /// `std::fs::remove_dir`.
    #[cfg(windows)]
    #[test]
    fn removes_only_empty_direct_folders() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("full/inner")).unwrap();
        fs::create_dir(root.path().join("empty")).unwrap();
        let read_only = root.path().join("read-only");
        fs::create_dir(&read_only).unwrap();
        let writable = fs::metadata(&read_only).unwrap().permissions();
        let mut permissions = writable.clone();
        permissions.set_readonly(true);
        fs::set_permissions(&read_only, permissions).unwrap();
        fs::write(root.path().join("file.txt"), CONTENT).unwrap();
        let folder = HeldFolder::open(root.path(), []).unwrap();

        assert_eq!(
            kind(&unwrap_error(folder.remove_empty_folder("full"))),
            "io"
        );
        assert_eq!(
            kind(&unwrap_error(folder.remove_empty_folder("file.txt"))),
            "not direct"
        );
        assert_eq!(
            kind(&unwrap_error(folder.remove_empty_folder("absent"))),
            "missing"
        );
        assert_eq!(
            kind(&unwrap_error(folder.remove_empty_folder("read-only"))),
            "io"
        );
        let error = unwrap_error(folder.open_removable_file("empty"));
        assert_eq!(kind(&error), "not direct");
        folder.remove_empty_folder("empty").unwrap();
        drop(folder);

        assert_eq!(entries(root.path()), ["file.txt", "full", "read-only"]);
        // Its original attributes again, so that the temporary folder can be cleaned up.
        fs::set_permissions(&read_only, writable).unwrap();
    }

    #[test]
    fn only_a_folder_opened_for_removal_can_be_removed() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("managed");
        fs::create_dir_all(target.join("game")).unwrap();

        let plain = HeldFolder::open(&target, []).unwrap();
        assert_eq!(plain.remove().unwrap_err().kind(), ErrorKind::InvalidInput);
        let below = HeldFolder::open_for_removal(&target)
            .unwrap()
            .open_below(["game"])
            .unwrap();
        assert_eq!(below.remove().unwrap_err().kind(), ErrorKind::InvalidInput);
        assert!(HeldFolder::open_for_removal(&target)
            .unwrap()
            .remove()
            .is_err());
        assert_eq!(entries(&target), ["game"]);
        fs::remove_dir(target.join("game")).unwrap();
        HeldFolder::open_for_removal(&target)
            .unwrap()
            .remove()
            .unwrap();

        assert!(!target.exists());
    }

    #[test]
    fn identity_is_the_identity_of_the_held_folder() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("game")).unwrap();
        let held = HeldFolder::open(root.path(), ["game"]).unwrap();

        assert_eq!(
            held.identity().unwrap(),
            same_file::Handle::from_path(root.path().join("game")).unwrap()
        );
        assert_ne!(
            held.identity().unwrap(),
            same_file::Handle::from_path(root.path()).unwrap()
        );
    }

    /// Everything inside is removed depth first: files just after `before_remove` saw them,
    /// folders once empty, and entries named last after their siblings in every folder.
    #[test]
    fn removing_contents_goes_depth_first_and_removes_named_entries_last() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("managed");
        for file in [
            ".chriz/ledger/0.json",
            "game/a/b.txt",
            "game/c.txt",
            "z.txt",
        ] {
            fs::create_dir_all(target.join(file).parent().unwrap()).unwrap();
            fs::write(target.join(file), CONTENT).unwrap();
        }
        let held = HeldFolder::open_for_removal(&target).unwrap();
        let mut seen = Vec::new();

        held.remove_contents(&[".chriz"], &mut |path| {
            seen.push(
                path.strip_prefix(&target)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
            Ok(())
        })
        .unwrap();
        held.remove().unwrap();

        assert_eq!(
            seen,
            [
                "game/a/b.txt",
                "game/a",
                "game/c.txt",
                "game",
                "z.txt",
                ".chriz/ledger/0.json",
                ".chriz/ledger",
                ".chriz",
            ]
        );
        assert!(!target.exists());
    }

    /// An error from `before_remove` stops the removal before that entry is removed.
    #[test]
    fn removing_contents_stops_at_an_error_from_before_remove() {
        let root = tempfile::tempdir().unwrap();
        for file in ["a.txt", "b.txt"] {
            fs::write(root.path().join(file), CONTENT).unwrap();
        }
        let held = HeldFolder::open(root.path(), []).unwrap();

        let error = unwrap_error(held.remove_contents(&[], &mut |path| {
            if path.ends_with("b.txt") {
                return Err(io::Error::other("locked"));
            }
            Ok(())
        }));
        drop(held);

        assert_eq!(kind(&error), "io");
        assert_eq!(error.path, root.path().join("b.txt"));
        assert_eq!(entries(root.path()), ["b.txt"]);
    }

    /// A link inside is refused: nothing is removed through it, the link itself stays, and so
    /// does everything after it.
    #[cfg(windows)]
    #[test]
    fn removing_contents_refuses_a_link_without_following_it() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("managed");
        let outside = root.path().join("outside");
        fs::create_dir_all(target.join("game/override")).unwrap();
        fs::write(target.join("game/a.txt"), CONTENT).unwrap();
        fs::write(target.join("game/override/z.txt"), CONTENT).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("precious.txt"), OTHER).unwrap();
        junction(&target.join("game").join("linked"), &outside);
        let held = HeldFolder::open_for_removal(&target).unwrap();

        assert_eq!(
            held.find_link_below().unwrap(),
            Some(target.join("game/linked"))
        );
        let error = unwrap_error(held.remove_contents(&[], &mut |_| Ok(())));
        drop(held);

        assert_eq!(kind(&error), "not direct");
        assert_eq!(error.path, target.join("game/linked"));
        assert_eq!(entries(&outside), ["precious.txt"]);
        assert_eq!(fs::read(outside.join("precious.txt")).unwrap(), OTHER);
        assert_eq!(entries(&target.join("game")), ["linked", "override"]);
    }

    /// A link that replaces a folder after it was checked is found, and stops the removal there.
    #[cfg(windows)]
    #[test]
    fn removing_contents_refuses_a_folder_replaced_by_a_link_while_removing() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("managed");
        let outside = root.path().join("outside");
        fs::create_dir_all(target.join("b-folder")).unwrap();
        fs::write(target.join("a.txt"), CONTENT).unwrap();
        fs::write(target.join("b-folder/inside.txt"), CONTENT).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("inside.txt"), OTHER).unwrap();
        let held = HeldFolder::open_for_removal(&target).unwrap();
        assert_eq!(held.find_link_below().unwrap(), None);

        let error = unwrap_error(held.remove_contents(&[], &mut |path| {
            if path.ends_with("a.txt") {
                fs::rename(target.join("b-folder"), root.path().join("moved")).unwrap();
                junction(&target.join("b-folder"), &outside);
            }
            Ok(())
        }));
        drop(held);

        assert_eq!(kind(&error), "not direct");
        assert_eq!(error.path, target.join("b-folder"));
        assert_eq!(fs::read(outside.join("inside.txt")).unwrap(), OTHER);
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
            fs::create_dir_all(tree.join("game/.chriz")).unwrap();
            fs::write(tree.join("game/cdtweaks/existing.txt"), OTHER).unwrap();
            fs::write(tree.join("game/cdtweaks/owned.txt"), OTHER).unwrap();
            fs::create_dir(tree.join("game/cdtweaks/empty")).unwrap();
        }
        fs::write(other.join("game/cdtweaks/only-elsewhere.txt"), OTHER).unwrap();
        let link = root.path().join("link");
        junction(&link, &checked);

        let held = HeldFolder::open(&link.join("game"), []).unwrap();
        let folder = held.open_below(["cdtweaks"]).unwrap();
        fs::remove_dir(&link).unwrap();
        junction(&link, &other);
        let scratch = held.open_below([".chriz"]).unwrap();
        let created = held.open_or_create_below(["cdtweaks", "new"]).unwrap();
        assert!(!folder.contains("only-elsewhere.txt").unwrap());
        let mut names = folder.names().unwrap();
        names.sort();
        assert_eq!(names, ["empty", "existing.txt", "new", "owned.txt"]);
        folder.create_file("cdtweaks.txt", CONTENT).unwrap();
        folder.replace_file("existing.txt", CONTENT).unwrap();
        created
            .create_file_from(&scratch, "made.txt", CONTENT)
            .unwrap();
        let (existing, _) = folder.open_file("existing.txt").unwrap();
        assert_eq!(
            crate::digest::sha256_reader(&existing).unwrap(),
            crate::digest::sha256_bytes(CONTENT)
        );
        drop(existing);
        folder
            .open_removable_file("owned.txt")
            .unwrap()
            .remove()
            .unwrap();
        folder.remove_empty_folder("empty").unwrap();
        drop((folder, created, scratch, held));

        let checked_mod = checked.join("game/cdtweaks");
        assert_eq!(
            entries(&checked_mod),
            ["cdtweaks.txt", "existing.txt", "new"]
        );
        assert_eq!(fs::read(checked_mod.join("existing.txt")).unwrap(), CONTENT);
        assert_eq!(fs::read(checked_mod.join("new/made.txt")).unwrap(), CONTENT);
        let other_mod = other.join("game/cdtweaks");
        assert_eq!(
            entries(&other_mod),
            ["empty", "existing.txt", "only-elsewhere.txt", "owned.txt"]
        );
        assert_eq!(fs::read(other_mod.join("existing.txt")).unwrap(), OTHER);
        assert_eq!(fs::read(other_mod.join("owned.txt")).unwrap(), OTHER);
    }

    /// Removing a whole tree stays in the checked tree after a link above it is re-pointed.
    #[cfg(windows)]
    #[test]
    fn removes_the_held_tree_after_a_link_above_the_root_is_repointed() {
        let root = tempfile::tempdir().unwrap();
        let checked = root.path().join("checked");
        let other = root.path().join("other");
        for tree in [&checked, &other] {
            fs::create_dir_all(tree.join("managed/game/override")).unwrap();
            fs::write(tree.join("managed/game/override/a.2da"), CONTENT).unwrap();
            fs::write(tree.join("managed/game/weidu.log"), CONTENT).unwrap();
        }
        let before = tree(&other);
        let link = root.path().join("link");
        junction(&link, &checked);

        let held = HeldFolder::open_for_removal(&link.join("managed")).unwrap();
        fs::remove_dir(&link).unwrap();
        junction(&link, &other);
        assert_eq!(held.find_link_below().unwrap(), None);
        held.remove_contents(&[], &mut |_| Ok(())).unwrap();
        held.remove().unwrap();

        assert_eq!(entries(&checked), Vec::<String>::new());
        assert_eq!(tree(&other), before);
    }
}
