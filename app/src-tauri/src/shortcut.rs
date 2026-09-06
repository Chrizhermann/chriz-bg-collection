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
    format!(
        "Play {}.lnk",
        shortcut_stem(display_name, MAX_SHORTCUT_STEM_UTF16 - "Play ".len())
    )
}

#[cfg(windows)]
fn collision_file_name(request: &ShortcutRequest) -> String {
    let identity = bg_engine::digest::sha256_bytes(request.install_id.as_bytes());
    let suffix = format!(" ({})", &identity[..12]);
    format!(
        "Play {}{suffix}.lnk",
        shortcut_stem(
            &request.display_name,
            MAX_SHORTCUT_STEM_UTF16 - "Play ".len() - suffix.len()
        )
    )
}

fn shortcut_stem(display_name: &str, max_utf16: usize) -> String {
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
        if units + next > max_utf16 {
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
    bounded.to_owned()
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
    use std::os::windows::fs::MetadataExt;
    use std::{ffi::OsString, fs, ptr};
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
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

    #[derive(PartialEq)]
    enum ExistingShortcut {
        Missing,
        Owned,
        Occupied,
    }

    fn inspect(
        path: &Path,
        request: &ShortcutRequest,
        marker: &str,
    ) -> Result<ExistingShortcut, String> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ExistingShortcut::Missing);
            }
            Err(error) => {
                return Err(format!(
                    "inspect existing shortcut {}: {error}",
                    path.display()
                ))
            }
        };
        if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 {
            return Ok(ExistingShortcut::Occupied);
        }
        let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
            .map_err(|error| format!("create Shell Link inspection object: {error}"))?;
        let persist: IPersistFile = link
            .cast()
            .map_err(|error| format!("open Shell Link inspection: {error}"))?;
        let path_wide = wide(path.as_os_str())?;
        if unsafe { persist.Load(PCWSTR::from_raw(path_wide.as_ptr()), STGM_READ) }.is_err()
            || !owned(&link, request, marker)?
        {
            return Ok(ExistingShortcut::Occupied);
        }
        Ok(ExistingShortcut::Owned)
    }

    if !desktop.is_absolute()
        || !request.target.is_absolute()
        || !request.working_directory.is_absolute()
    {
        return Err("Desktop shortcut paths must be absolute.".to_owned());
    }
    let marker = format!("Chriz Easy BG launcher; install-id={}", request.install_id);
    let legacy = desktop.join(format!(
        "{}.lnk",
        shortcut_stem(&request.display_name, MAX_SHORTCUT_STEM_UTF16)
    ));
    let primary = desktop.join(shortcut_file_name(&request.display_name));
    let fallback = desktop.join(collision_file_name(request));
    let candidates = [legacy, primary, fallback];
    let states = candidates
        .iter()
        .map(|path| inspect(path, request, &marker))
        .collect::<Result<Vec<_>, _>>()?;
    // Reuse an owned link before taking a vacant name, including links made by
    // earlier app versions and fallback links whose original collision is gone.
    let selected = states
        .iter()
        .position(|state| *state == ExistingShortcut::Owned)
        .or_else(|| {
            states
                .iter()
                .enumerate()
                .skip(1)
                .find(|(_, state)| **state == ExistingShortcut::Missing)
                .map(|(index, _)| index)
        })
        .ok_or_else(|| {
            format!(
                "A different item already exists at both {} and {}. Neither is owned by this CEBG installation.",
                candidates[1].display(), candidates[2].display()
            )
        })?;
    let shortcut_path = &candidates[selected];
    // Use a fresh COM object so inspecting a foreign link cannot copy its other
    // shell properties into a newly created game shortcut.
    let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
        .map_err(|error| format!("create Shell Link object: {error}"))?;
    let persist: IPersistFile = link
        .cast()
        .map_err(|error| format!("open Shell Link persistence: {error}"))?;

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
    Ok(shortcut_path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn shortcut_filename_sanitizes_windows_names_without_losing_unicode() {
        assert_eq!(
            shortcut_file_name(" Ordinary — 이름<>:\"/\\|?*... "),
            "Play Ordinary — 이름.lnk"
        );
        assert_eq!(shortcut_file_name("CON"), "Play CEBG - CON.lnk");
        assert_eq!(shortcut_file_name(" ... "), "Play Chriz Easy BG.lnk");
        assert!(shortcut_file_name(&"이".repeat(200)).encode_utf16().count() <= 124);
        assert!(shortcut_file_name(&"🎲".repeat(200)).encode_utf16().count() <= 124);
    }

    #[cfg(windows)]
    #[test]
    fn game_shortcuts_with_the_same_name_are_separate_and_idempotent() {
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
        assert_eq!(
            create_shortcut_in_directory(desktop.path(), request).expect("refresh owned shortcut"),
            created
        );
        let first = fs::read(&created).expect("read refreshed first shortcut");

        let second_request = ShortcutRequest {
            install_id: "install-two".to_owned(),
            display_name: "Chriz Easy BG".to_owned(),
            target: app,
            arguments: "--install-id=install-two".to_owned(),
            working_directory: desktop.path().to_path_buf(),
        };
        let second = create_shortcut_in_directory(desktop.path(), second_request.clone())
            .expect("create shortcut for a second installation with the same name");
        assert_ne!(created, second);
        assert_eq!(fs::read(&created).expect("read preserved shortcut"), first);
        assert_eq!(
            create_shortcut_in_directory(desktop.path(), second_request.clone())
                .expect("refresh second shortcut"),
            second
        );

        fs::remove_file(created).expect("remove first fixture shortcut");
        assert_eq!(
            create_shortcut_in_directory(desktop.path(), second_request)
                .expect("keep the second shortcut path when the default name becomes available"),
            second
        );
    }

    #[cfg(windows)]
    fn request_fixture(desktop: &Path) -> ShortcutRequest {
        let target = desktop.join("CEBG 테스트.exe");
        fs::write(&target, b"fixture executable").expect("write app fixture");
        ShortcutRequest {
            install_id: "install-one".to_owned(),
            display_name: "Chriz Easy BG".to_owned(),
            target,
            arguments: "--install-id=install-one".to_owned(),
            working_directory: desktop.to_path_buf(),
        }
    }

    #[cfg(windows)]
    #[test]
    fn app_shortcut_without_game_ownership_is_preserved() {
        use std::os::windows::ffi::OsStrExt;
        use windows::core::{Interface, PCWSTR};
        use windows::Win32::System::Com::{CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER};
        use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

        let desktop = tempfile::tempdir().expect("create desktop fixture");
        let request = request_fixture(desktop.path());
        let app_shortcut = desktop.path().join("Chriz Easy BG.lnk");
        let path = app_shortcut.clone();
        let target = request.target.clone();
        run_sta(move || {
            let target = target
                .as_os_str()
                .encode_wide()
                .chain([0])
                .collect::<Vec<_>>();
            let destination = path
                .as_os_str()
                .encode_wide()
                .chain([0])
                .collect::<Vec<_>>();
            unsafe {
                let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                    .expect("create app shortcut fixture");
                link.SetPath(PCWSTR::from_raw(target.as_ptr()))
                    .expect("set app target");
                let persist: IPersistFile = link.cast().expect("open app shortcut persistence");
                persist
                    .Save(PCWSTR::from_raw(destination.as_ptr()), true)
                    .expect("save app shortcut with no install arguments or marker");
            }
            Ok(path)
        })
        .expect("write native app shortcut fixture");
        let original = fs::read(&app_shortcut).expect("read app shortcut");

        let created = create_shortcut_in_directory(desktop.path(), request)
            .expect("create game shortcut beside the app shortcut");
        assert_eq!(created, desktop.path().join("Play Chriz Easy BG.lnk"));
        assert_eq!(
            fs::read(app_shortcut).expect("read preserved app shortcut"),
            original
        );
    }

    #[cfg(windows)]
    #[test]
    fn owned_legacy_shortcut_is_reused_without_creating_a_duplicate() {
        let desktop = tempfile::tempdir().expect("create desktop fixture");
        let request = request_fixture(desktop.path());
        let created = create_shortcut_in_directory(desktop.path(), request.clone())
            .expect("create owned shortcut fixture");
        let legacy = desktop.path().join("Chriz Easy BG.lnk");
        fs::rename(created, &legacy).expect("place owned fixture at the legacy filename");

        assert_eq!(
            create_shortcut_in_directory(desktop.path(), request).expect("reuse legacy shortcut"),
            legacy
        );
        assert!(!desktop.path().join("Play Chriz Easy BG.lnk").exists());
    }

    #[cfg(windows)]
    #[test]
    fn foreign_collision_uses_a_stable_fallback_and_never_overwrites_occupied_paths() {
        let desktop = tempfile::tempdir().expect("create desktop fixture");
        let request = request_fixture(desktop.path());
        let primary = desktop.path().join("Play Chriz Easy BG.lnk");
        fs::write(&primary, b"foreign primary file").expect("write foreign primary fixture");
        let fallback = create_shortcut_in_directory(desktop.path(), request.clone())
            .expect("create fallback alongside foreign primary");
        assert_ne!(primary, fallback);
        assert!(fallback
            .file_name()
            .expect("fallback filename")
            .to_string_lossy()
            .starts_with("Play Chriz Easy BG ("));
        fs::write(&fallback, b"foreign fallback file").expect("replace test fallback fixture");

        let error = create_shortcut_in_directory(desktop.path(), request.clone())
            .expect_err("refuse to overwrite either occupied filename");
        assert!(error.contains("already exists"));
        assert_eq!(
            fs::read(&primary).expect("read primary fixture"),
            b"foreign primary file"
        );
        assert_eq!(
            fs::read(&fallback).expect("read fallback fixture"),
            b"foreign fallback file"
        );

        fs::remove_file(&fallback).expect("remove fallback fixture file");
        fs::create_dir(&fallback).expect("create occupied fallback directory");
        create_shortcut_in_directory(desktop.path(), request)
            .expect_err("refuse to overwrite fallback directory");
        assert!(fallback.is_dir());
    }

    #[cfg(windows)]
    #[test]
    fn unicode_game_shortcut_collision_keeps_distinct_bounded_names() {
        let desktop = tempfile::tempdir().expect("create desktop fixture");
        let mut request = request_fixture(desktop.path());
        request.display_name = format!("이름<>:\"/\\|?* {}", "🎲".repeat(200));
        let first = create_shortcut_in_directory(desktop.path(), request.clone())
            .expect("create Unicode game shortcut");
        request.install_id = "install-two".to_owned();
        request.arguments = "--install-id=install-two".to_owned();
        let second = create_shortcut_in_directory(desktop.path(), request)
            .expect("create separate bounded Unicode game shortcut");
        assert_ne!(first, second);
        for path in [first, second] {
            let name = path
                .file_name()
                .expect("shortcut filename")
                .to_string_lossy();
            assert!(name.starts_with("Play 이름 "));
            assert!(name.encode_utf16().count() <= 124);
        }
    }

    #[cfg(windows)]
    #[test]
    fn shortcut_reparse_collision_and_its_target_are_preserved() {
        let desktop = tempfile::tempdir().expect("create desktop fixture");
        let request = request_fixture(desktop.path());
        let original = create_shortcut_in_directory(desktop.path(), request.clone())
            .expect("create owned shortcut fixture");
        let protected = desktop.path().join("protected.lnk");
        fs::rename(&original, &protected).expect("move shortcut fixture out of candidate paths");
        let bytes = fs::read(&protected).expect("read protected shortcut fixture");
        if let Err(error) = std::os::windows::fs::symlink_file(&protected, &original) {
            if error.kind() == std::io::ErrorKind::PermissionDenied
                || error.kind() == std::io::ErrorKind::Unsupported
                || error.raw_os_error() == Some(1314)
            {
                eprintln!("shortcut reparse fixture unavailable: {error}");
                return;
            }
            panic!("create shortcut reparse fixture: {error}");
        }

        let created = create_shortcut_in_directory(desktop.path(), request)
            .expect("create safe fallback for reparse collision");
        assert_ne!(created, original);
        assert!(fs::symlink_metadata(&original)
            .expect("inspect preserved shortcut reparse fixture")
            .file_type()
            .is_symlink());
        assert_eq!(fs::read(protected).expect("read protected target"), bytes);
    }
}
