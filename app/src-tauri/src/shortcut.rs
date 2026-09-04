use std::path::{Path, PathBuf};

/// All native-owned fields needed to create or verify one CEBG launcher shortcut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutRequest {
    pub install_id: String,
    pub display_name: String,
    pub target: PathBuf,
    pub arguments: String,
    pub working_directory: PathBuf,
}

const MAX_SHORTCUT_STEM_UTF16: usize = 120;

/// Produces a bounded Windows-compatible filename while preserving ordinary Unicode.
pub fn shortcut_file_name(display_name: &str) -> String {
    let mut sanitized = display_name
        .chars()
        .filter(|character| {
            !character.is_control()
                && !matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                )
        })
        .collect::<String>();
    sanitized = sanitized.trim().trim_end_matches(['.', ' ']).to_owned();
    if sanitized.is_empty() || matches!(sanitized.as_str(), "." | "..") {
        sanitized = "Chriz Easy BG".to_owned();
    }
    let device_stem = sanitized
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let reserved = matches!(
        device_stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    );
    if reserved {
        sanitized = format!("CEBG - {sanitized}");
    }
    let mut bounded = String::new();
    let mut units = 0;
    for character in sanitized.chars() {
        let next = character.len_utf16();
        if units + next > MAX_SHORTCUT_STEM_UTF16 {
            break;
        }
        bounded.push(character);
        units += next;
    }
    let bounded = bounded.trim_end_matches(['.', ' ']);
    let bounded = if bounded.is_empty() {
        "Chriz Easy BG"
    } else {
        bounded
    };
    format!("{bounded}.lnk")
}

#[cfg(windows)]
pub fn create_desktop_shortcut(request: ShortcutRequest) -> Result<PathBuf, String> {
    run_sta(move || {
        let desktop = known_desktop_directory()?;
        write_shortcut(&desktop, &request)
    })
}

#[cfg(not(windows))]
pub fn create_desktop_shortcut(_request: ShortcutRequest) -> Result<PathBuf, String> {
    Err("Desktop shortcuts are supported only on Windows.".to_owned())
}

#[cfg(all(windows, test))]
fn create_shortcut_in_directory(
    desktop: &Path,
    request: ShortcutRequest,
) -> Result<PathBuf, String> {
    let desktop = desktop.to_path_buf();
    run_sta(move || write_shortcut(&desktop, &request))
}

#[cfg(windows)]
fn run_sta(
    operation: impl FnOnce() -> Result<PathBuf, String> + Send + 'static,
) -> Result<PathBuf, String> {
    std::thread::Builder::new()
        .name("cebg-shell-link".to_owned())
        .spawn(move || {
            use windows::Win32::System::Com::{
                CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED,
            };

            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
                .ok()
                .map_err(|error| format!("initialize Shell Link COM apartment: {error}"))?;
            struct ComApartment;
            impl Drop for ComApartment {
                fn drop(&mut self) {
                    unsafe { CoUninitialize() };
                }
            }
            let _apartment = ComApartment;
            operation()
        })
        .map_err(|error| format!("start Shell Link worker: {error}"))?
        .join()
        .map_err(|_| "Shell Link worker panicked".to_owned())?
}

#[cfg(windows)]
fn known_desktop_directory() -> Result<PathBuf, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{FOLDERID_Desktop, SHGetKnownFolderPath, KF_FLAG_DEFAULT};

    let raw = unsafe { SHGetKnownFolderPath(&FOLDERID_Desktop, KF_FLAG_DEFAULT, None) }
        .map_err(|error| format!("resolve Windows Desktop folder: {error}"))?;
    let wide = unsafe { raw.as_wide() };
    let path = PathBuf::from(OsString::from_wide(wide));
    unsafe { CoTaskMemFree(Some(raw.as_ptr().cast())) };
    if !path.is_absolute() {
        return Err(format!(
            "Windows returned a non-absolute Desktop folder: {}",
            path.display()
        ));
    }
    Ok(path)
}

#[cfg(windows)]
fn write_shortcut(desktop: &Path, request: &ShortcutRequest) -> Result<PathBuf, String> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::{ffi::OsString, fs, ptr};
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    fn wide(value: &std::ffi::OsStr) -> Result<Vec<u16>, String> {
        let mut value = value.encode_wide().collect::<Vec<_>>();
        if value.contains(&0) {
            return Err("Shell Link fields cannot contain a null character.".to_owned());
        }
        value.push(0);
        Ok(value)
    }

    fn read_wide(buffer: &[u16]) -> OsString {
        let end = buffer
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(buffer.len());
        OsString::from_wide(&buffer[..end])
    }

    fn owned(link: &IShellLinkW, request: &ShortcutRequest, marker: &str) -> Result<bool, String> {
        let mut target = vec![0_u16; 32_768];
        let mut arguments = vec![0_u16; 512];
        let mut description = vec![0_u16; 512];
        unsafe {
            link.GetPath(&mut target, ptr::null_mut(), 0)
                .map_err(|error| format!("read existing shortcut target: {error}"))?;
            link.GetArguments(&mut arguments)
                .map_err(|error| format!("read existing shortcut arguments: {error}"))?;
            link.GetDescription(&mut description)
                .map_err(|error| format!("read existing shortcut marker: {error}"))?;
        }
        let stored_target = PathBuf::from(read_wide(&target));
        let same_target = fs::canonicalize(&stored_target)
            .ok()
            .zip(fs::canonicalize(&request.target).ok())
            .map(|(left, right)| left == right)
            .unwrap_or(stored_target == request.target);
        Ok(same_target
            && read_wide(&arguments) == std::ffi::OsStr::new(&request.arguments)
            && read_wide(&description) == std::ffi::OsStr::new(marker))
    }

    if !desktop.is_absolute()
        || !request.target.is_absolute()
        || !request.working_directory.is_absolute()
    {
        return Err("Desktop shortcut paths must be absolute.".to_owned());
    }
    let shortcut_path = desktop.join(shortcut_file_name(&request.display_name));
    let marker = format!("Chriz Easy BG launcher; install-id={}", request.install_id);
    let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
        .map_err(|error| format!("create Shell Link object: {error}"))?;
    let persist: IPersistFile = link
        .cast()
        .map_err(|error| format!("open Shell Link persistence: {error}"))?;

    let existing = match fs::symlink_metadata(&shortcut_path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(format!("inspect existing shortcut: {error}")),
    };
    if let Some(metadata) = existing {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "A different item already exists at {}.",
                shortcut_path.display()
            ));
        }
        let shortcut_wide = wide(shortcut_path.as_os_str())?;
        if unsafe { persist.Load(PCWSTR::from_raw(shortcut_wide.as_ptr()), STGM_READ) }.is_err()
            || !owned(&link, request, &marker)?
        {
            return Err(format!(
                "A shortcut already exists at {} and is not owned by this CEBG installation.",
                shortcut_path.display()
            ));
        }
    }

    let target = wide(request.target.as_os_str())?;
    let arguments = wide(std::ffi::OsStr::new(&request.arguments))?;
    let working_directory = wide(request.working_directory.as_os_str())?;
    let marker = wide(std::ffi::OsStr::new(&marker))?;
    let shortcut = wide(shortcut_path.as_os_str())?;
    unsafe {
        link.SetPath(PCWSTR::from_raw(target.as_ptr()))
            .map_err(|error| format!("set shortcut target: {error}"))?;
        link.SetArguments(PCWSTR::from_raw(arguments.as_ptr()))
            .map_err(|error| format!("set shortcut arguments: {error}"))?;
        link.SetWorkingDirectory(PCWSTR::from_raw(working_directory.as_ptr()))
            .map_err(|error| format!("set shortcut working folder: {error}"))?;
        link.SetDescription(PCWSTR::from_raw(marker.as_ptr()))
            .map_err(|error| format!("set shortcut ownership marker: {error}"))?;
        link.SetIconLocation(PCWSTR::from_raw(target.as_ptr()), 0)
            .map_err(|error| format!("set shortcut icon: {error}"))?;
        persist
            .Save(PCWSTR::from_raw(shortcut.as_ptr()), true)
            .map_err(|error| format!("save desktop shortcut: {error}"))?;
    }
    Ok(shortcut_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn shortcut_filename_sanitizes_windows_names_without_losing_unicode() {
        assert_eq!(
            shortcut_file_name(" Ordinary — 이름<>:\"/\\|?*... "),
            "Ordinary — 이름.lnk"
        );
        assert_eq!(shortcut_file_name("CON"), "CEBG - CON.lnk");
        assert_eq!(shortcut_file_name(" ... "), "Chriz Easy BG.lnk");
        assert!(shortcut_file_name(&"이".repeat(200)).encode_utf16().count() <= 124);
    }

    #[cfg(windows)]
    #[test]
    fn owned_shortcut_is_idempotent_and_foreign_shortcut_is_preserved() {
        let desktop = tempfile::tempdir().expect("create desktop fixture");
        let app = desktop.path().join("CEBG 테스트.exe");
        fs::write(&app, b"fixture executable").expect("write app fixture");
        let request = ShortcutRequest {
            install_id: "install-one".to_owned(),
            display_name: "Chriz Easy BG".to_owned(),
            target: app.clone(),
            arguments: "--install-id=install-one".to_owned(),
            working_directory: desktop.path().to_path_buf(),
        };

        let created = create_shortcut_in_directory(desktop.path(), request.clone())
            .expect("create owned shortcut");
        let first = fs::read(&created).expect("read first shortcut");
        assert_eq!(
            create_shortcut_in_directory(desktop.path(), request).expect("refresh owned shortcut"),
            created
        );

        let conflict = create_shortcut_in_directory(
            desktop.path(),
            ShortcutRequest {
                install_id: "install-two".to_owned(),
                display_name: "Chriz Easy BG".to_owned(),
                target: app,
                arguments: "--install-id=install-two".to_owned(),
                working_directory: desktop.path().to_path_buf(),
            },
        )
        .expect_err("refuse foreign shortcut");
        assert!(conflict.contains("already exists"));
        assert_eq!(fs::read(created).expect("read preserved shortcut"), first);
    }
}
