//! Recipe-authored configuration files placed in a staged mod folder before its run.
//!
//! A file is created only while absent. An existing file is accepted only when it already holds
//! the authored bytes, so a retried attempt is idempotent and nothing that a mod archive or a
//! person put there is ever overwritten.
//!
//! On Windows the folders from the target root down to the mod folder are opened one at a time,
//! each relative to the folder above it and without following a link, and held without delete
//! sharing until the file is in place. The file is inspected, created and renamed through those
//! handles, so no path is resolved again once a folder was checked, and no folder can be swapped
//! for a link in between.

use std::fs::{self, Metadata};
use std::io::{self, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

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
    let (folders, name) = file
        .path
        .rsplit_once('/')
        .ok_or_else(|| format!("configuration file {:?} has no mod folder", file.path))?;

    // Only add files to the published folder: never create folders or follow a link out of the
    // staged game.
    let folder = ModFolder::open(target_root, folders)?;
    let path = folder.path.join(name);
    let content = file.content.as_bytes();
    let sha256 = sha256_bytes(content);
    let evidence = |created| ConfigFileEvidence {
        component,
        path: file.path.clone(),
        sha256: sha256.clone(),
        created,
    };
    match folder.open_file(name) {
        Ok(existing) => {
            let metadata = existing
                .metadata()
                .map_err(|error| inspect_error(&path, &error))?;
            if !metadata.is_file() || is_link_or_reparse(&metadata) {
                return Err(format!(
                    "configuration file path is not a direct file: {}",
                    path.display()
                ));
            }
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
        Err(error) if error.kind() == ErrorKind::NotFound => {
            folder
                .create_file(name, content)
                .map_err(|error| format!("place configuration file {}: {error}", path.display()))?;
            Ok(evidence(true))
        }
        Err(error) => Err(inspect_error(&path, &error)),
    }
}

/// The published mod folder a configuration file goes into, opened from the target root down.
#[cfg(windows)]
struct ModFolder {
    /// The target root and every folder below it down to the mod folder, outermost first. While
    /// they are held, none of them can be renamed, removed or replaced.
    chain: Vec<fs::File>,
    /// For messages only; the handles decide where the file goes.
    path: PathBuf,
}

#[cfg(windows)]
impl ModFolder {
    fn open(target_root: &Path, folders: &str) -> Result<Self, String> {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        };

        let root = fs::OpenOptions::new()
            .read(true)
            .share_mode(handles::SHARE_ALL_BUT_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(target_root)
            .map_err(|error| folder_error(target_root, &error))?;
        require_direct_directory(&root, target_root)?;
        let mut chain = vec![root];
        let mut path = target_root.to_path_buf();
        for name in folders.split('/') {
            path.push(name);
            let parent = chain.last().expect("the chain starts with the target root");
            let folder =
                handles::open_folder(parent, name).map_err(|error| folder_error(&path, &error))?;
            require_direct_directory(&folder, &path)?;
            chain.push(folder);
        }
        Ok(Self { chain, path })
    }

    fn folder(&self) -> &fs::File {
        self.chain
            .last()
            .expect("the chain starts with the target root")
    }

    /// Open the entry `name` in the mod folder for reading, without following a link.
    fn open_file(&self, name: &str) -> io::Result<fs::File> {
        handles::open_entry(self.folder(), name)
    }

    /// Create `name` in the mod folder with `content`, never replacing anything already there.
    fn create_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        let temporary = self.create_temporary_file()?;
        let placed = (&temporary)
            .write_all(content)
            .and_then(|()| temporary.sync_all())
            .and_then(|()| handles::rename_into(&temporary, self.folder(), name));
        match placed {
            Ok(()) => Ok(()),
            // Remove the unfinished file through its own handle: only a complete file is ever
            // published under the configuration file's name.
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
            let name = format!(".chriz-config-{process}-{nonce}-{attempt}.tmp");
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
fn require_direct_directory(folder: &fs::File, path: &Path) -> Result<(), String> {
    let metadata = folder
        .metadata()
        .map_err(|error| folder_error(path, &error))?;
    if !metadata.is_dir() || is_link_or_reparse(&metadata) {
        return Err(not_a_direct_directory(path));
    }
    Ok(())
}

/// Opening and naming files relative to held folder handles.
#[cfg(windows)]
mod handles {
    use std::fs::File;
    use std::io;
    use std::mem::size_of;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use std::ptr;

    use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
    use windows_sys::Wdk::Storage::FileSystem::{
        FileRenameInformation, NtCreateFile, NtSetInformationFile, FILE_CREATE,
        FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_REPARSE_POINT, FILE_RENAME_INFORMATION,
        FILE_SYNCHRONOUS_IO_NONALERT,
    };
    use windows_sys::Win32::Foundation::{
        RtlNtStatusToDosError, HANDLE, NTSTATUS, OBJ_CASE_INSENSITIVE, UNICODE_STRING,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        FileDispositionInfo, SetFileInformationByHandle, DELETE, FILE_ATTRIBUTE_NORMAL,
        FILE_DISPOSITION_INFO, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_LIST_DIRECTORY,
        FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_TRAVERSE, SYNCHRONIZE,
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

    /// Give `file` the plain `name` directly inside `folder`, never replacing an entry. The NT call
    /// takes the name relative to the folder handle; the Win32 wrapper would resolve a plain name
    /// against the current directory instead.
    pub(super) fn rename_into(file: &File, folder: &File, name: &str) -> io::Result<()> {
        let wide: Vec<u16> = name.encode_utf16().collect();
        let name_bytes = wide.len() * size_of::<u16>();
        let size = size_of::<FILE_RENAME_INFORMATION>() + name_bytes;
        // Whole u64 words keep the buffer aligned for FILE_RENAME_INFORMATION. Its zeroed flags
        // mean that an existing entry is never replaced.
        let mut buffer = vec![0_u64; size.div_ceil(size_of::<u64>())];
        let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
        // SAFETY: the buffer is aligned and large enough for the header and the whole name.
        unsafe {
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

/// The published mod folder a configuration file goes into. Off Windows, which the installer
/// does not support, links are refused by path.
#[cfg(not(windows))]
struct ModFolder {
    path: PathBuf,
}

#[cfg(not(windows))]
impl ModFolder {
    fn open(target_root: &Path, folders: &str) -> Result<Self, String> {
        let mut path = target_root.to_path_buf();
        require_direct_directory(&path)?;
        for name in folders.split('/') {
            path.push(name);
            require_direct_directory(&path)?;
        }
        Ok(Self { path })
    }

    fn open_file(&self, name: &str) -> io::Result<fs::File> {
        let path = self.path.join(name);
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "a link is not a direct file",
            ));
        }
        fs::File::open(path)
    }

    fn create_file(&self, name: &str, content: &[u8]) -> io::Result<()> {
        let mut temporary = tempfile::NamedTempFile::new_in(&self.path)?;
        temporary.write_all(content)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist_noclobber(self.path.join(name))
            .map_err(|error| error.error)?;
        Ok(())
    }
}

#[cfg(not(windows))]
fn require_direct_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| folder_error(path, &error))?;
    if !metadata.is_dir() || is_link_or_reparse(&metadata) {
        return Err(not_a_direct_directory(path));
    }
    Ok(())
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

fn inspect_error(path: &Path, error: &io::Error) -> String {
    format!("inspect configuration file {}: {error}", path.display())
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

    fn entries(folder: &Path) -> Vec<String> {
        let mut names = fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        names.sort();
        names
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
    fn creating_never_replaces_an_entry_and_leaves_no_temporary_file() {
        let root = staged_mod();
        let folder_path = root.path().join("cdtweaks");
        let other = "OUTER_SET party_movement_speed_value = 200\n";
        fs::write(folder_path.join("cdtweaks.txt"), other).unwrap();
        let folder = ModFolder::open(root.path(), "cdtweaks").unwrap();

        // As if the file appeared after placement found the name free.
        let error = folder
            .create_file("cdtweaks.txt", CONTENT.as_bytes())
            .unwrap_err();
        drop(folder);

        assert_eq!(error.kind(), ErrorKind::AlreadyExists, "{error}");
        assert_eq!(entries(&folder_path), ["cdtweaks.txt"]);
        assert_eq!(
            fs::read_to_string(folder_path.join("cdtweaks.txt")).unwrap(),
            other
        );
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

    #[cfg(windows)]
    #[test]
    fn held_folders_cannot_be_moved_while_the_file_is_placed() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("game");
        let mod_folder = game.join("cdtweaks");
        fs::create_dir_all(&mod_folder).unwrap();

        let folder = ModFolder::open(&game, "cdtweaks").unwrap();
        assert!(fs::rename(&mod_folder, game.join("moved")).is_err());
        assert!(fs::rename(&game, root.path().join("moved")).is_err());
        // Adding the configuration file inside the held folder still works.
        folder
            .create_file("cdtweaks.txt", CONTENT.as_bytes())
            .unwrap();
        drop(folder);

        assert_eq!(entries(&mod_folder), ["cdtweaks.txt"]);
        fs::rename(&mod_folder, game.join("moved")).unwrap();
    }

    /// A link above the target root can still be re-pointed while the folders are held, which
    /// redirects every path through it. The file must go into the folder that was checked.
    #[cfg(windows)]
    #[test]
    fn writes_into_the_held_folder_even_when_its_path_is_redirected() {
        let root = tempfile::tempdir().unwrap();
        let checked = root.path().join("checked");
        let other = root.path().join("other");
        fs::create_dir_all(checked.join("game/cdtweaks")).unwrap();
        fs::create_dir_all(other.join("game/cdtweaks")).unwrap();
        let link = root.path().join("link");
        junction(&link, &checked);

        let folder = ModFolder::open(&link.join("game"), "cdtweaks").unwrap();
        fs::remove_dir(&link).unwrap();
        junction(&link, &other);
        folder
            .create_file("cdtweaks.txt", CONTENT.as_bytes())
            .unwrap();
        drop(folder);

        assert_eq!(
            fs::read_to_string(checked.join("game/cdtweaks/cdtweaks.txt")).unwrap(),
            CONTENT
        );
        assert_eq!(entries(&other.join("game/cdtweaks")), Vec::<String>::new());
    }
}
