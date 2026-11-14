//! The player's Documents folder, the parent of every installation's saved-game folder.
//!
//! [`SystemDocuments`] is the only code that asks Windows for this folder. Test builds, debug
//! or release, never reach it: they are built with the `test-documents` feature, which only
//! dev-dependencies enable, and then use the isolated folder that a test names in
//! [`TEST_DOCUMENTS_VARIABLE`]. A test that names none stops with a panic, before anything is
//! created. Other builds use the real folder, unless a debug build names one in that variable.

use std::io;
use std::path::{Path, PathBuf};

/// Names an isolated folder that stands in for the Documents folder.
///
/// Test builds require it whenever they look up Documents. Other debug builds honor it, and
/// other release builds ignore it.
pub const TEST_DOCUMENTS_VARIABLE: &str = "CHRIZ_BG_COLLECTION_TEST_DOCUMENTS";

/// Resolves the current user's Documents directory without assuming its location.
pub trait DocumentsLocator {
    /// Returns the user-configured Documents known folder.
    fn documents_dir(&self) -> io::Result<PathBuf>;
}

/// Production Documents resolver.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemDocuments;

impl DocumentsLocator for SystemDocuments {
    fn documents_dir(&self) -> io::Result<PathBuf> {
        #[cfg(any(debug_assertions, feature = "test-documents"))]
        if let Some(folder) = std::env::var_os(TEST_DOCUMENTS_VARIABLE) {
            return isolated_folder(folder);
        }
        #[cfg(feature = "test-documents")]
        refuse_real_folder();
        #[cfg(not(feature = "test-documents"))]
        known_folder()
    }
}

#[cfg(any(debug_assertions, feature = "test-documents", test))]
fn isolated_folder(folder: std::ffi::OsString) -> io::Result<PathBuf> {
    let folder = PathBuf::from(folder);
    if !folder.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{TEST_DOCUMENTS_VARIABLE} is not absolute"),
        ));
    }
    Ok(folder)
}

/// Stops a test that would otherwise create profile folders in the player's real Documents
/// folder, which OneDrive may also sync.
#[cfg(any(feature = "test-documents", test))]
fn refuse_real_folder() -> ! {
    panic!(
        "test builds never use the real Documents folder: name an isolated folder in \
         {TEST_DOCUMENTS_VARIABLE} or pass a DocumentsLocator"
    )
}

// Test builds never call this outside its own unit test.
#[cfg(windows)]
#[cfg_attr(feature = "test-documents", allow(dead_code))]
fn known_folder() -> io::Result<PathBuf> {
    use std::ffi::{c_void, OsString};
    use std::os::windows::ffi::OsStringExt;
    use std::ptr;
    use windows_sys::Win32::Foundation::RPC_E_CHANGED_MODE;
    use windows_sys::Win32::System::Com::{
        CoInitializeEx, CoTaskMemFree, COINIT_APARTMENTTHREADED,
    };
    use windows_sys::Win32::UI::Shell::{FOLDERID_Documents, SHGetKnownFolderPath};

    let initialization = unsafe { CoInitializeEx(ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    let _com_guard = if initialization >= 0 {
        ComInitializationGuard { balance: true }
    } else if initialization == RPC_E_CHANGED_MODE {
        ComInitializationGuard { balance: false }
    } else {
        return Err(super::folder_diagnostics::windows_hresult_error(
            "CoInitializeEx",
            initialization,
        ));
    };

    let mut raw = ptr::null_mut();
    let result = unsafe { SHGetKnownFolderPath(&FOLDERID_Documents, 0, ptr::null_mut(), &mut raw) };
    if result < 0 {
        if !raw.is_null() {
            unsafe { CoTaskMemFree(raw.cast::<c_void>()) };
        }
        return Err(super::folder_diagnostics::windows_hresult_error(
            "SHGetKnownFolderPath(FOLDERID_Documents)",
            result,
        ));
    }
    if raw.is_null() {
        return Err(io::Error::other(
            "SHGetKnownFolderPath(FOLDERID_Documents) returned a null path",
        ));
    }

    let length = unsafe {
        let mut length = 0_usize;
        while *raw.add(length) != 0 {
            length += 1;
        }
        length
    };
    let path = PathBuf::from(OsString::from_wide(unsafe {
        std::slice::from_raw_parts(raw, length)
    }));
    unsafe { CoTaskMemFree(raw.cast::<c_void>()) };
    Ok(path)
}

#[cfg(windows)]
struct ComInitializationGuard {
    balance: bool,
}

#[cfg(windows)]
impl Drop for ComInitializationGuard {
    fn drop(&mut self) {
        if self.balance {
            unsafe { windows_sys::Win32::System::Com::CoUninitialize() };
        }
    }
}

#[cfg(not(windows))]
#[cfg_attr(feature = "test-documents", allow(dead_code))]
#[allow(clippy::redundant_closure)]
fn known_folder() -> io::Result<PathBuf> {
    let dir = resolve_unix_documents_path(
        |name| std::env::var_os(name),
        |path| std::fs::read_to_string(path),
    )?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg_attr(windows, allow(dead_code))]
fn resolve_unix_documents_path(
    env_get: impl Fn(&str) -> Option<std::ffi::OsString>,
    read_file: impl Fn(&Path) -> io::Result<String>,
) -> io::Result<PathBuf> {
    if let Some(raw) = env_get("XDG_DOCUMENTS_DIR").filter(|value| !value.is_empty()) {
        let dir = PathBuf::from(raw);
        if !dir.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "XDG_DOCUMENTS_DIR is not absolute",
            ));
        }
        return Ok(dir);
    }

    let home = env_get("HOME").filter(|value| !value.is_empty());
    let home_dir = home.as_ref().map(PathBuf::from);
    if let Some(ref home_path) = home_dir {
        if !home_path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "HOME is not absolute",
            ));
        }
    }

    let config_home = env_get("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| home_dir.as_ref().map(|home| home.join(".config")));

    if let Some(config_dir) = config_home {
        let user_dirs = config_dir.join("user-dirs.dirs");
        match read_file(&user_dirs) {
            Ok(content) => {
                if let Some(parsed) = parse_xdg_documents_dir(&content, home_dir.as_deref())? {
                    return Ok(parsed);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {}
        }
    }

    let Some(home_path) = home_dir else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is unavailable to resolve the Documents folder",
        ));
    };
    Ok(home_path.join("Documents"))
}

#[cfg_attr(windows, allow(dead_code))]
fn parse_xdg_documents_dir(content: &str, home: Option<&Path>) -> io::Result<Option<PathBuf>> {
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").map_or(line, str::trim_start);
        let Some(value) = line.strip_prefix("XDG_DOCUMENTS_DIR=") else {
            continue;
        };
        let value = value.trim();
        let unquoted = if (value.starts_with('"') && value.ends_with('"') && value.len() >= 2)
            || (value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2)
        {
            &value[1..value.len() - 1]
        } else {
            value
        };
        let expanded = if let Some(rest) = unquoted
            .strip_prefix("${HOME}")
            .or_else(|| unquoted.strip_prefix("$HOME"))
        {
            let Some(home) = home else {
                continue;
            };
            let trimmed = rest.trim_start_matches(['/', '\\']);
            if trimmed.is_empty() {
                home.to_path_buf()
            } else {
                home.join(trimmed)
            }
        } else {
            PathBuf::from(unquoted)
        };
        if expanded.is_absolute() {
            return Ok(Some(expanded));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_isolated_folder_must_be_absolute() {
        let folder = std::env::temp_dir().join("isolated-documents");
        assert_eq!(isolated_folder(folder.clone().into()).unwrap(), folder);
        let error = isolated_folder("relative/Documents".into()).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    #[should_panic(expected = "test builds never use the real Documents folder")]
    fn a_test_without_an_isolated_folder_stops() {
        refuse_real_folder();
    }

    /// The one test that resolves the real folder. It compares the path with the shell's
    /// older lookup of the same folder and never opens, lists or changes the folder itself.
    #[cfg(windows)]
    #[test]
    fn known_folder_is_the_shell_documents_folder() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows_sys::Win32::UI::Shell::{SHGetFolderPathW, CSIDL_PERSONAL, SHGFP_TYPE_CURRENT};

        let documents = known_folder().unwrap();
        assert!(documents.is_absolute(), "{}", documents.display());

        let mut buffer = [0_u16; 260];
        let result = unsafe {
            SHGetFolderPathW(
                std::ptr::null_mut(),
                CSIDL_PERSONAL as i32,
                std::ptr::null_mut(),
                SHGFP_TYPE_CURRENT as u32,
                buffer.as_mut_ptr(),
            )
        };
        assert_eq!(result, 0, "SHGetFolderPathW(CSIDL_PERSONAL) failed");
        let length = buffer.iter().position(|unit| *unit == 0).unwrap();
        assert_eq!(
            documents,
            PathBuf::from(OsString::from_wide(&buffer[..length]))
        );
    }

    #[test]
    fn unix_documents_resolver_uses_env_user_dirs_and_home_fallback() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let custom = root.path().join("custom-docs");
        let from_xdg_file = home.join("Dokumente");

        let resolved_env = resolve_unix_documents_path(
            |name| match name {
                "XDG_DOCUMENTS_DIR" => Some(custom.clone().into_os_string()),
                "HOME" => Some(home.clone().into_os_string()),
                _ => None,
            },
            |_| unreachable!("env takes precedence over user-dirs.dirs"),
        )
        .unwrap();
        assert_eq!(resolved_env, custom);

        let user_dirs = "# comment\nXDG_DOCUMENTS_DIR=\"$HOME/Dokumente\"\n";
        let resolved_user_dirs = resolve_unix_documents_path(
            |name| match name {
                "HOME" => Some(home.clone().into_os_string()),
                _ => None,
            },
            |path| {
                assert_eq!(path, home.join(".config").join("user-dirs.dirs"));
                Ok(user_dirs.to_owned())
            },
        )
        .unwrap();
        assert_eq!(resolved_user_dirs, from_xdg_file);

        let fallback = resolve_unix_documents_path(
            |name| match name {
                "HOME" => Some(home.clone().into_os_string()),
                _ => None,
            },
            |_| Err(io::Error::from(io::ErrorKind::NotFound)),
        )
        .unwrap();
        assert_eq!(fallback, home.join("Documents"));
    }
}
