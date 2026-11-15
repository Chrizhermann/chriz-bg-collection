//! Linux game launch through Steam Proton or Wine with an isolated per-install prefix.
//!
//! Managed campaigns keep their own `STEAM_COMPAT_DATA_PATH` / `WINEPREFIX` under
//! `<managed_root>/.chriz/proton` and point their Wine/Proton `Documents/<engine_name>` save folder
//! at `<managed_save_root>` without mutating `InfinityLoader.ini` (InfinityLoader auto-detects
//! Wine via `wine_get_version`).

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

const MAX_OALINST_BYTES: u64 = 16 * 1024 * 1024;
const MAX_OPENAL_DLL_BYTES: usize = 4 * 1024 * 1024;

#[cfg(any(not(windows), test))]
const APPIMAGE_STRIP_ENV_VARS: &[&str] = &[
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "GIO_MODULE_DIR",
    "GSETTINGS_SCHEMA_DIR",
    "GTK_PATH",
    "GTK_EXE_PREFIX",
    "GTK_IM_MODULE_FILE",
    "QT_PLUGIN_PATH",
    "PYTHONHOME",
    "PYTHONPATH",
    "PERLLIB",
    "APPIMAGE",
    "APPDIR",
    "ARGV0",
    "OWD",
];

#[cfg(any(not(windows), test))]
const APPIMAGE_RESTORE_ENV_VARS: &[(&str, &str)] = &[
    ("LD_LIBRARY_PATH", "APPIMAGE_ORIGINAL_LD_LIBRARY_PATH"),
    ("XDG_DATA_DIRS", "APPIMAGE_ORIGINAL_XDG_DATA_DIRS"),
];

#[cfg(any(not(windows), test))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct HostEnvAdjustments {
    remove: Vec<&'static str>,
    restore: Vec<(&'static str, OsString)>,
}

#[cfg(any(not(windows), test))]
fn host_env_adjustments_from(get_env: impl Fn(&str) -> Option<OsString>) -> HostEnvAdjustments {
    let inside_appimage = get_env("APPIMAGE").is_some_and(|v| !v.is_empty())
        || get_env("APPDIR").is_some_and(|v| !v.is_empty());
    if !inside_appimage {
        return HostEnvAdjustments::default();
    }

    let mut remove = APPIMAGE_STRIP_ENV_VARS.to_vec();
    let mut restore = Vec::new();
    for &(target, original_key) in APPIMAGE_RESTORE_ENV_VARS {
        match get_env(original_key) {
            Some(original) if !original.is_empty() => {
                remove.retain(|&key| key != target);
                restore.push((target, original));
            }
            Some(_) if !remove.contains(&target) => remove.push(target),
            Some(_) | None => {}
        }
    }
    HostEnvAdjustments { remove, restore }
}

/// Clears AppImage runtime overrides before spawning host tools (`proton`, `wine64`, `xdg-open`).
#[cfg(any(not(windows), test))]
pub fn apply_clean_host_env(command: &mut std::process::Command) {
    let adjustments = host_env_adjustments_from(|key| std::env::var_os(key));
    for key in adjustments.remove {
        command.env_remove(key);
    }
    for (key, value) in adjustments.restore {
        command.env(key, value);
    }
}

/// Resolved Linux Windows-compatibility runner for launching a managed game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinuxRunner {
    Proton {
        binary: PathBuf,
        steam_root: PathBuf,
    },
    Wine {
        binary: PathBuf,
    },
}

/// Process invocation plan produced from a discovered [`LinuxRunner`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub envs: Vec<(String, PathBuf)>,
    pub compat_data: PathBuf,
}

/// Starts a managed Windows executable on Linux using Proton or Wine inside an isolated prefix.
#[cfg(not(windows))]
pub fn launch_windows_executable(
    managed_root: &Path,
    managed_save_root: Option<&Path>,
    executable: &Path,
    working_directory: &Path,
) -> Result<(), String> {
    let env_override = std::env::var_os("CHRIZ_PROTON_BIN").map(PathBuf::from);
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let path_dirs = std::env::var_os("PATH")
        .map(|raw| std::env::split_paths(&raw).collect::<Vec<_>>())
        .unwrap_or_default();
    let runner = find_runner_with(env_override.as_deref(), home.as_deref(), &path_dirs)?;
    let spec = build_launch_spec(&runner, managed_root, executable)?;
    let current_user = std::env::var("USER")
        .ok()
        .filter(|name| !name.trim().is_empty())
        .or_else(|| std::env::var("LOGNAME").ok());
    prepare_proton_environment(
        &spec.compat_data,
        working_directory,
        managed_save_root,
        current_user.as_deref(),
    )
    .map_err(|error| format!("prepare isolated Wine/Proton prefix: {error}"))?;

    let mut command = std::process::Command::new(&spec.program);
    command.args(&spec.args).current_dir(working_directory);
    apply_clean_host_env(&mut command);
    for (key, value) in &spec.envs {
        command.env(key, value);
    }
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("start {}: {error}", spec.program.display()))
}

/// Discovers Proton or Wine using `CHRIZ_PROTON_BIN` -> Steam `compatibilitytools.d` ->
/// Steam `steamapps/common` -> `wine64`/`wine` on `PATH`.
pub fn find_runner_with(
    env_override: Option<&Path>,
    home: Option<&Path>,
    path_dirs: &[PathBuf],
) -> Result<LinuxRunner, String> {
    let steam_roots = home.map(collect_steam_roots).unwrap_or_default();

    if let Some(explicit) = env_override.filter(|path| !path.as_os_str().is_empty()) {
        if !explicit.is_file() {
            return Err(format!(
                "CHRIZ_PROTON_BIN points to {:?}, which does not exist or is not a file.",
                explicit.display()
            ));
        }
        if is_proton_binary_name(explicit.file_name()) {
            let steam_root = steam_root_for_proton(explicit, &steam_roots)
                .unwrap_or_else(|| explicit.parent().unwrap_or(explicit).to_path_buf());
            return Ok(LinuxRunner::Proton {
                binary: explicit.to_path_buf(),
                steam_root,
            });
        }
        return Ok(LinuxRunner::Wine {
            binary: explicit.to_path_buf(),
        });
    }

    for steam_root in &steam_roots {
        if let Some(binary) = find_custom_proton(&steam_root.join("compatibilitytools.d")) {
            return Ok(LinuxRunner::Proton {
                binary,
                steam_root: steam_root.clone(),
            });
        }
    }

    for steam_root in &steam_roots {
        if let Some(binary) = find_steam_proton(&steam_root.join("steamapps").join("common")) {
            return Ok(LinuxRunner::Proton {
                binary,
                steam_root: steam_root.clone(),
            });
        }
    }

    for candidate_name in ["wine64", "wine"] {
        for dir in path_dirs {
            let candidate = dir.join(candidate_name);
            if candidate.is_file() {
                return Ok(LinuxRunner::Wine { binary: candidate });
            }
        }
    }

    Err(
        "Could not find Steam Proton or Wine to launch the Windows game executable. Install Proton via Steam (Settings -> Compatibility) or install wine/wine64 via your package manager, or set CHRIZ_PROTON_BIN to a Proton script or Wine binary."
            .to_owned(),
    )
}

/// Builds the isolated command, arguments, and environment for launching `executable`.
pub fn build_launch_spec(
    runner: &LinuxRunner,
    managed_root: &Path,
    executable: &Path,
) -> Result<LaunchSpec, String> {
    let executable_str = executable.to_str().ok_or_else(|| {
        format!(
            "executable path is not valid UTF-8: {}",
            executable.display()
        )
    })?;
    let compat_data = managed_root.join(".chriz").join("proton");
    match runner {
        LinuxRunner::Proton { binary, steam_root } => Ok(LaunchSpec {
            program: binary.clone(),
            args: vec!["waitforexitandrun".to_owned(), executable_str.to_owned()],
            envs: vec![
                ("STEAM_COMPAT_DATA_PATH".to_owned(), compat_data.clone()),
                (
                    "STEAM_COMPAT_CLIENT_INSTALL_PATH".to_owned(),
                    steam_root.clone(),
                ),
            ],
            compat_data,
        }),
        LinuxRunner::Wine { binary } => {
            let wine_prefix = compat_data.join("pfx");
            Ok(LaunchSpec {
                program: binary.clone(),
                args: vec![executable_str.to_owned()],
                envs: vec![("WINEPREFIX".to_owned(), wine_prefix)],
                compat_data,
            })
        }
    }
}

/// Ensures the isolated Proton/Wine prefix directory exists, provisions `OpenAL32.dll` and
/// `wrap_oal.dll` from the game's `oalinst.exe` when absent, and links `Documents/<engine_name>`
/// to `managed_save_root` without mutating non-empty existing save directories.
pub fn prepare_proton_environment(
    compat_data: &Path,
    working_directory: &Path,
    managed_save_root: Option<&Path>,
    current_user: Option<&str>,
) -> io::Result<()> {
    let pfx = compat_data.join("pfx");
    fs::create_dir_all(&pfx)?;
    provision_openal_dlls(&pfx, working_directory)?;

    let Some(save_root) = managed_save_root else {
        return Ok(());
    };
    let Some(engine_name) = read_engine_name(working_directory) else {
        return Ok(());
    };

    fs::create_dir_all(save_root)?;
    prepare_user_save_directory(&pfx, "steamuser", &engine_name, save_root)?;

    if let Some(user) = valid_prefix_user_name(current_user) {
        prepare_user_save_directory(&pfx, user, &engine_name, save_root)?;
    }
    Ok(())
}

fn valid_prefix_user_name(raw: Option<&str>) -> Option<&str> {
    let trimmed = raw?.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("steamuser")
        || trimmed.contains(['/', '\\', '\0'])
    {
        return None;
    }
    let mut components = Path::new(trimmed).components();
    matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    )
    .then_some(trimmed)
}

fn prepare_user_save_directory(
    pfx: &Path,
    user: &str,
    engine_name: &str,
    save_root: &Path,
) -> io::Result<()> {
    let docs_dir = pfx
        .join("drive_c")
        .join("users")
        .join(user)
        .join("Documents");
    if let Ok(meta) = fs::symlink_metadata(&docs_dir) {
        if meta.file_type().is_symlink() {
            fs::remove_file(&docs_dir)?;
        }
    }
    fs::create_dir_all(&docs_dir)?;
    link_save_directory(&docs_dir.join(engine_name), save_root)
}

/// Parses `engine_name = "..."` from `engine.lua` inside `game_root`, returning `None` when absent
/// or containing path separators.
pub fn read_engine_name(game_root: &Path) -> Option<String> {
    let lua = fs::read_to_string(game_root.join("engine.lua")).ok()?;
    for line in lua.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            continue;
        }
        let Some((lhs, rhs)) = trimmed.split_once('=') else {
            continue;
        };
        if lhs.trim() != "engine_name" {
            continue;
        }
        let rhs = rhs.trim().trim_end_matches(';').trim();
        let unquoted = rhs
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .or_else(|| rhs.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))?;
        if unquoted.is_empty()
            || unquoted == "."
            || unquoted == ".."
            || unquoted.contains(['/', '\\', '\0'])
        {
            return None;
        }
        return Some(unquoted.to_owned());
    }
    None
}

fn is_proton_binary_name(name: Option<&OsStr>) -> bool {
    name.and_then(OsStr::to_str)
        .is_some_and(|text| text.eq_ignore_ascii_case("proton"))
}

fn collect_steam_roots(home: &Path) -> Vec<PathBuf> {
    let candidates = [
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
        home.join(".steam/root"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        home.join("snap/steam/common/.local/share/Steam"),
    ];
    let mut roots = Vec::new();
    for candidate in candidates {
        if candidate.is_dir() && !roots.contains(&candidate) {
            roots.push(candidate);
        }
    }
    roots
}

fn steam_root_for_proton(proton_binary: &Path, steam_roots: &[PathBuf]) -> Option<PathBuf> {
    steam_roots
        .iter()
        .find(|root| proton_binary.starts_with(root))
        .cloned()
        .or_else(|| steam_roots.first().cloned())
}

fn find_custom_proton(compat_tools_dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(compat_tools_dir).ok()?;
    let mut named_candidates = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let proton = dir.join("proton");
        if proton.is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            named_candidates.push((name, proton));
        }
    }
    named_candidates.sort_by(|left, right| right.0.cmp(&left.0));
    named_candidates.into_iter().next().map(|(_, path)| path)
}

fn find_steam_proton(common_dir: &Path) -> Option<PathBuf> {
    let experimental = common_dir.join("Proton - Experimental").join("proton");
    if experimental.is_file() {
        return Some(experimental);
    }
    let entries = fs::read_dir(common_dir).ok()?;
    let mut candidates = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let proton = dir.join("proton");
        if !proton.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("Proton") {
            continue;
        }
        let tier = if name.starts_with("Proton 9.0") {
            0_u8
        } else if name.starts_with("Proton 8.0") {
            1_u8
        } else {
            2_u8
        };
        candidates.push((tier, name, proton));
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
    candidates.into_iter().next().map(|(_, _, path)| path)
}

fn link_save_directory(link_path: &Path, target: &Path) -> io::Result<()> {
    match fs::symlink_metadata(link_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                if fs::read_link(link_path).ok().as_deref() == Some(target) {
                    return Ok(());
                }
                fs::remove_file(link_path)?;
            } else if metadata.is_dir() {
                if fs::read_dir(link_path)?.next().is_some() {
                    return Ok(());
                }
                fs::remove_dir(link_path)?;
            } else {
                return Ok(());
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    create_dir_symlink(target, link_path)
}

#[cfg(unix)]
fn create_dir_symlink(target: &Path, link_path: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link_path)
}

#[cfg(not(unix))]
fn create_dir_symlink(_target: &Path, _link_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Proton save symlinks are only supported on Unix hosts",
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpenAlArch {
    X64,
    X86,
}

impl OpenAlArch {
    fn prefix_subdir(self) -> &'static str {
        match self {
            Self::X64 => "system32",
            Self::X86 => "syswow64",
        }
    }
}

struct ExtractedOpenAlDll {
    arch: OpenAlArch,
    file_name: &'static str,
    bytes: Vec<u8>,
}

/// Provisions x64 and x86 `OpenAL32.dll` / `wrap_oal.dll` from `oalinst.exe` into
/// `pfx/drive_c/windows/{system32,syswow64}` without altering `working_directory` or overwriting
/// existing prefix DLLs. Fails soft when `oalinst.exe` is absent, symlinked, or unreadable.
fn provision_openal_dlls(pfx: &Path, working_directory: &Path) -> io::Result<()> {
    let system32 = pfx.join("drive_c").join("windows").join("system32");
    let syswow64 = pfx.join("drive_c").join("windows").join("syswow64");
    if dir_has_openal_dlls(&system32) && dir_has_openal_dlls(&syswow64) {
        return Ok(());
    }

    let Some(installer_path) = find_case_insensitive_regular_file(working_directory, "oalinst.exe")
    else {
        return Ok(());
    };
    let Ok(meta) = fs::symlink_metadata(&installer_path) else {
        return Ok(());
    };
    if !meta.file_type().is_file() || meta.len() > MAX_OALINST_BYTES {
        return Ok(());
    }
    let Ok(installer_bytes) = fs::read(&installer_path) else {
        return Ok(());
    };

    let extracted = extract_openal_dlls_from_installer(&installer_bytes);
    if extracted.is_empty() {
        return Ok(());
    }

    for dll in extracted {
        let Some(target_dir) =
            ensure_real_prefix_subdir(pfx, &["drive_c", "windows", dll.arch.prefix_subdir()])?
        else {
            continue;
        };
        if dir_has_entry_case_insensitive(&target_dir, dll.file_name) {
            continue;
        }
        let dest = target_dir.join(dll.file_name);
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&dest)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        file.write_all(&dll.bytes)?;
    }

    Ok(())
}

fn dir_has_openal_dlls(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    let mut has_openal32 = false;
    let mut has_wrap_oal = false;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(text) = name.to_str() else {
            continue;
        };
        if text.eq_ignore_ascii_case("OpenAL32.dll") {
            has_openal32 = true;
        } else if text.eq_ignore_ascii_case("wrap_oal.dll") {
            has_wrap_oal = true;
        }
        if has_openal32 && has_wrap_oal {
            return true;
        }
    }
    false
}

fn dir_has_entry_case_insensitive(dir: &Path, expected_name: &str) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(expected_name))
    })
}

fn find_case_insensitive_regular_file(dir: &Path, expected_name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(expected_name))
        {
            return Some(entry.path());
        }
    }
    None
}

fn ensure_real_prefix_subdir(pfx: &Path, rel_parts: &[&str]) -> io::Result<Option<PathBuf>> {
    let mut current = pfx.to_path_buf();
    for part in rel_parts {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) => {
                if meta.file_type().is_symlink() || !meta.is_dir() {
                    return Ok(None);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                match fs::create_dir(&current) {
                    Ok(()) => {}
                    Err(create_err) if create_err.kind() == io::ErrorKind::AlreadyExists => {
                        let meta = fs::symlink_metadata(&current)?;
                        if meta.file_type().is_symlink() || !meta.is_dir() {
                            return Ok(None);
                        }
                    }
                    Err(create_err) => return Err(create_err),
                }
            }
            Err(error) => return Err(error),
        }
    }
    Ok(Some(current))
}

fn extract_openal_dlls_from_installer(installer_bytes: &[u8]) -> Vec<ExtractedOpenAlDll> {
    let Ok(pe) = pelite::PeFile::from_bytes(installer_bytes) else {
        return Vec::new();
    };
    let Ok(resources) = pe.resources() else {
        return Vec::new();
    };
    let Ok(root) = resources.root() else {
        return Vec::new();
    };
    let Ok(dll_dir) = root.get_dir(pelite::resources::Name::Str("DLL")) else {
        return Vec::new();
    };

    let mut extracted = Vec::new();
    for entry in dll_dir.entries() {
        let Some(sub) = entry.entry().ok().and_then(pelite::resources::Entry::dir) else {
            continue;
        };
        let Ok(data_entry) = sub.first_data() else {
            continue;
        };
        let Ok(compressed) = data_entry.bytes() else {
            continue;
        };
        if let Some(dll) = extract_openal_dll_payload(compressed) {
            extracted.push(dll);
        }
    }
    extracted
}

fn extract_openal_dll_payload(compressed: &[u8]) -> Option<ExtractedOpenAlDll> {
    let mut dll_bytes = Vec::new();
    let decoder = flate2::read::ZlibDecoder::new(compressed);
    if decoder
        .take((MAX_OPENAL_DLL_BYTES + 1) as u64)
        .read_to_end(&mut dll_bytes)
        .is_err()
    {
        return None;
    }
    if dll_bytes.len() > MAX_OPENAL_DLL_BYTES {
        return None;
    }

    let dll_pe = pelite::PeFile::from_bytes(&dll_bytes).ok()?;
    let arch = match dll_pe.file_header().Machine {
        pelite::image::IMAGE_FILE_MACHINE_AMD64 => OpenAlArch::X64,
        pelite::image::IMAGE_FILE_MACHINE_I386 => OpenAlArch::X86,
        _ => return None,
    };
    let exports = dll_pe.exports().ok()?;
    let raw_name = exports.dll_name().ok()?;
    let dll_name = raw_name.to_str().ok()?;
    let file_name = if dll_name.eq_ignore_ascii_case("OpenAL32.dll") {
        "OpenAL32.dll"
    } else if dll_name.eq_ignore_ascii_case("wrap_oal.dll") {
        "wrap_oal.dll"
    } else {
        return None;
    };

    Some(ExtractedOpenAlDll {
        arch,
        file_name,
        bytes: dll_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn runner_discovery_prefers_env_then_compat_tools_then_steam_then_wine() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("home");
        let steam = home.join(".local/share/Steam");
        let compat = steam.join("compatibilitytools.d");
        let common = steam.join("steamapps/common");
        let bin = temp.path().join("bin");

        let custom_old = compat.join("GE-Proton9-10");
        let custom_new = compat.join("GE-Proton9-25");
        let proton8 = common.join("Proton 8.0");
        let proton9 = common.join("Proton 9.0 (Beta)");
        let experimental = common.join("Proton - Experimental");
        for dir in [
            &custom_old,
            &custom_new,
            &proton8,
            &proton9,
            &experimental,
            &bin,
        ] {
            fs::create_dir_all(dir).unwrap();
            if dir != &bin {
                fs::write(dir.join("proton"), b"#!/bin/sh\n").unwrap();
            }
        }
        let wine = bin.join("wine");
        let wine64 = bin.join("wine64");
        fs::write(&wine, b"elf").unwrap();
        fs::write(&wine64, b"elf").unwrap();
        let override_proton = temp.path().join("custom/proton");
        fs::create_dir_all(override_proton.parent().unwrap()).unwrap();
        fs::write(&override_proton, b"#!/bin/sh\n").unwrap();

        // 1. Explicit CHRIZ_PROTON_BIN wins
        assert_eq!(
            find_runner_with(
                Some(&override_proton),
                Some(&home),
                std::slice::from_ref(&bin)
            )
            .unwrap(),
            LinuxRunner::Proton {
                binary: override_proton.clone(),
                steam_root: steam.clone(),
            }
        );

        // 2. Latest GE-Proton in compatibilitytools.d beats stock Steam Protons
        assert_eq!(
            find_runner_with(None, Some(&home), std::slice::from_ref(&bin)).unwrap(),
            LinuxRunner::Proton {
                binary: custom_new.join("proton"),
                steam_root: steam.clone(),
            }
        );

        // 3. Remove compatibilitytools.d -> Proton - Experimental wins over Proton 9.0 / 8.0
        fs::remove_dir_all(&compat).unwrap();
        assert_eq!(
            find_runner_with(None, Some(&home), std::slice::from_ref(&bin)).unwrap(),
            LinuxRunner::Proton {
                binary: experimental.join("proton"),
                steam_root: steam.clone(),
            }
        );

        // 4. Remove Experimental -> Proton 9.0 wins over Proton 8.0
        fs::remove_dir_all(&experimental).unwrap();
        assert_eq!(
            find_runner_with(None, Some(&home), std::slice::from_ref(&bin)).unwrap(),
            LinuxRunner::Proton {
                binary: proton9.join("proton"),
                steam_root: steam.clone(),
            }
        );

        // 5. Remove Steam common -> wine64 on PATH wins over wine
        fs::remove_dir_all(&common).unwrap();
        assert_eq!(
            find_runner_with(None, Some(&home), std::slice::from_ref(&bin)).unwrap(),
            LinuxRunner::Wine {
                binary: wine64.clone()
            }
        );

        // 6. Invalid explicit CHRIZ_PROTON_BIN fails clearly instead of falling back
        let missing = temp.path().join("does-not-exist/proton");
        assert!(find_runner_with(Some(&missing), Some(&home), &[bin]).is_err());

        // 7. No runner anywhere -> actionable error
        let empty = temp.path().join("empty");
        fs::create_dir_all(&empty).unwrap();
        let error = find_runner_with(None, Some(&empty), &[]).unwrap_err();
        assert!(error.contains("CHRIZ_PROTON_BIN"));
    }

    #[test]
    fn build_launch_spec_sets_isolated_proton_and_wine_envs() {
        let managed = Path::new("/home/deck/Games/CEBG");
        let exe = Path::new("/home/deck/Games/CEBG/game/InfinityLoader.exe");

        let proton_spec = build_launch_spec(
            &LinuxRunner::Proton {
                binary: PathBuf::from("/steam/proton"),
                steam_root: PathBuf::from("/steam/root"),
            },
            managed,
            exe,
        )
        .unwrap();
        assert_eq!(proton_spec.program, PathBuf::from("/steam/proton"));
        assert_eq!(
            proton_spec.args,
            vec!["waitforexitandrun", exe.to_str().unwrap()]
        );
        assert_eq!(
            proton_spec.envs,
            vec![
                (
                    "STEAM_COMPAT_DATA_PATH".to_owned(),
                    managed.join(".chriz/proton")
                ),
                (
                    "STEAM_COMPAT_CLIENT_INSTALL_PATH".to_owned(),
                    PathBuf::from("/steam/root")
                ),
            ]
        );

        let wine_spec = build_launch_spec(
            &LinuxRunner::Wine {
                binary: PathBuf::from("/usr/bin/wine64"),
            },
            managed,
            exe,
        )
        .unwrap();
        assert_eq!(wine_spec.args, vec![exe.to_str().unwrap()]);
        assert_eq!(
            wine_spec.envs,
            vec![(
                "WINEPREFIX".to_owned(),
                managed.join(".chriz/proton").join("pfx")
            )]
        );
    }

    #[test]
    fn parses_engine_name_safely_and_prepares_prefix() {
        let temp = TempDir::new().unwrap();
        let game = temp.path().join("game");
        fs::create_dir_all(&game).unwrap();
        fs::write(
            game.join("engine.lua"),
            "-- comment\nengine_name = \"Baldur's Gate II - Enhanced Edition\"\n",
        )
        .unwrap();
        assert_eq!(
            read_engine_name(&game).as_deref(),
            Some("Baldur's Gate II - Enhanced Edition")
        );

        fs::write(game.join("engine.lua"), "engine_name = \"../escape\"\n").unwrap();
        assert_eq!(read_engine_name(&game), None);

        let compat = temp.path().join("managed/.chriz/proton");
        prepare_proton_environment(&compat, &game, None, Some("deck")).unwrap();
        assert!(compat.join("pfx").is_dir());

        #[cfg(unix)]
        {
            fs::write(
                game.join("engine.lua"),
                "engine_name = \"Baldur's Gate - Enhanced Edition Trilogy\"\n",
            )
            .unwrap();
            let save_root = temp.path().join("Documents/Chriz Easy BG/campaign-1");
            // Fresh prefix: both `steamuser` and `deck` must have Documents/<engine_name>
            // pre-linked before first `wine` / `wineboot` spawn.
            prepare_proton_environment(&compat, &game, Some(&save_root), Some("deck")).unwrap();
            prepare_proton_environment(&compat, &game, Some(&save_root), Some("deck")).unwrap();
            let steamuser_link = compat.join(
                "pfx/drive_c/users/steamuser/Documents/Baldur's Gate - Enhanced Edition Trilogy",
            );
            let user_link = compat
                .join("pfx/drive_c/users/deck/Documents/Baldur's Gate - Enhanced Edition Trilogy");
            assert_eq!(fs::read_link(steamuser_link).unwrap(), save_root);
            assert_eq!(fs::read_link(user_link).unwrap(), save_root);

            // Healing: an existing `wineboot` Documents symlink to external `$HOME/Documents` must
            // be unlinked inside the prefix without mutating the external directory.
            let real_home_docs = temp.path().join("real-home/Documents");
            let sentinel_dir = real_home_docs.join("Baldur's Gate - Enhanced Edition Trilogy");
            fs::create_dir_all(&sentinel_dir).unwrap();
            let sentinel_file = sentinel_dir.join("baldur.lua");
            fs::write(&sentinel_file, b"untouched").unwrap();

            let healed_compat = temp.path().join("healed/.chriz/proton");
            let healed_user_root = healed_compat.join("pfx/drive_c/users/alice");
            fs::create_dir_all(&healed_user_root).unwrap();
            std::os::unix::fs::symlink(&real_home_docs, healed_user_root.join("Documents"))
                .unwrap();

            prepare_proton_environment(&healed_compat, &game, Some(&save_root), Some("alice"))
                .unwrap();
            let healed_docs = healed_user_root.join("Documents");
            assert!(
                !fs::symlink_metadata(&healed_docs)
                    .unwrap()
                    .file_type()
                    .is_symlink(),
                "prefix Documents symlink must be replaced with a local prefix directory"
            );
            assert_eq!(
                fs::read_link(healed_docs.join("Baldur's Gate - Enhanced Edition Trilogy"))
                    .unwrap(),
                save_root
            );
            assert_eq!(fs::read(&sentinel_file).unwrap(), b"untouched");

            // Traversal/path-separator usernames must be ignored safely.
            prepare_proton_environment(&healed_compat, &game, Some(&save_root), Some("../escape"))
                .unwrap();
            assert!(!healed_compat.join("pfx/drive_c/users/../escape").exists());
        }
    }

    #[test]
    fn host_env_adjustments_strip_appimage_vars_and_restore_originals() {
        assert_eq!(
            host_env_adjustments_from(|key| match key {
                "LD_LIBRARY_PATH" => Some(OsString::from("/usr/local/lib")),
                _ => None,
            }),
            HostEnvAdjustments::default(),
            "non-AppImage environments must not strip user overrides"
        );

        let inside = host_env_adjustments_from(|key| match key {
            "APPIMAGE" => Some(OsString::from("/home/deck/CEBG.AppImage")),
            "LD_LIBRARY_PATH" => Some(OsString::from("/tmp/.mount_CEBG/usr/lib")),
            "APPIMAGE_ORIGINAL_LD_LIBRARY_PATH" => Some(OsString::from("/opt/gpu/lib")),
            "APPIMAGE_ORIGINAL_XDG_DATA_DIRS" => Some(OsString::from("")),
            _ => None,
        });
        assert!(inside.remove.contains(&"PYTHONHOME"));
        assert!(inside.remove.contains(&"APPIMAGE"));
        assert!(inside.remove.contains(&"XDG_DATA_DIRS"));
        assert!(!inside.remove.contains(&"LD_LIBRARY_PATH"));
        assert_eq!(
            inside.restore,
            vec![("LD_LIBRARY_PATH", OsString::from("/opt/gpu/lib"))]
        );
    }

    fn write_u16(buf: &mut [u8], offset: usize, value: u16) {
        buf[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(buf: &mut [u8], offset: usize, value: u32) {
        buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn zlib_compress(data: &[u8]) -> Vec<u8> {
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    fn build_minimal_export_dll(machine: u16, export_dll_name: &[u8]) -> Vec<u8> {
        let is_64 = machine == pelite::image::IMAGE_FILE_MACHINE_AMD64;
        let opt_size: usize = if is_64 { 0xF0 } else { 0xE0 };
        let section_raw_offset = 0x200_usize;
        let section_rva = 0x1000_u32;
        let section_content_len = 40 + export_dll_name.len() + 1;
        let section_aligned_len = section_content_len.div_ceil(0x200) * 0x200;
        let mut pe = vec![0_u8; section_raw_offset + section_aligned_len];

        write_u16(&mut pe, 0x00, 0x5A4D);
        write_u32(&mut pe, 0x3C, 0x80);
        write_u32(&mut pe, 0x80, 0x0000_4550);
        write_u16(&mut pe, 0x84, machine);
        write_u16(&mut pe, 0x86, 1);
        write_u16(&mut pe, 0x94, opt_size as u16);

        let opt = 0x98_usize;
        write_u16(&mut pe, opt, if is_64 { 0x20B } else { 0x10B });
        write_u32(&mut pe, opt + 0x20, 0x1000);
        write_u32(&mut pe, opt + 0x24, 0x200);
        write_u32(
            &mut pe,
            opt + 0x38,
            0x1000 + (section_aligned_len as u32).div_ceil(0x1000) * 0x1000,
        );
        write_u32(&mut pe, opt + 0x3C, 0x200);
        let dd_count_off = opt + if is_64 { 0x6C } else { 0x5C };
        write_u32(&mut pe, dd_count_off, 16);
        let data_dir = opt + if is_64 { 0x70 } else { 0x60 };
        // DataDirectory[0] = Exports
        write_u32(&mut pe, data_dir, section_rva);
        write_u32(&mut pe, data_dir + 4, section_content_len as u32);

        let sec = opt + opt_size;
        pe[sec..sec + 8].copy_from_slice(b".edata\0\0");
        write_u32(&mut pe, sec + 8, section_content_len as u32);
        write_u32(&mut pe, sec + 12, section_rva);
        write_u32(&mut pe, sec + 16, section_aligned_len as u32);
        write_u32(&mut pe, sec + 20, section_raw_offset as u32);

        // IMAGE_EXPORT_DIRECTORY.Name at +12 points right after the 40-byte export dir header.
        write_u32(&mut pe, section_raw_offset + 12, section_rva + 40);
        write_u32(&mut pe, section_raw_offset + 16, 1);
        let name_start = section_raw_offset + 40;
        pe[name_start..name_start + export_dll_name.len()].copy_from_slice(export_dll_name);
        pe[name_start + export_dll_name.len()] = 0;

        pe
    }

    fn build_minimal_oalinst_pe(blobs: &[(u16, &[u8])]) -> Vec<u8> {
        let count = blobs.len();
        let type_name_offset = 0x18_usize;
        let dll_dir_offset = 0x20_usize;
        let dll_entries_offset = dll_dir_offset + 16;
        let id_dirs_start = dll_entries_offset + count * 8;
        let data_entries_start = id_dirs_start + count * 24;
        let blobs_start = data_entries_start + count * 16;
        let total_blobs_len: usize = blobs.iter().map(|(_, bytes)| bytes.len()).sum();
        let rsrc_len = blobs_start + total_blobs_len;
        let mut rsrc = vec![0_u8; rsrc_len];

        // Root directory: 1 named entry ("DLL")
        write_u16(&mut rsrc, 12, 1);
        write_u32(&mut rsrc, 16, 0x8000_0000 | (type_name_offset as u32));
        write_u32(&mut rsrc, 20, 0x8000_0000 | (dll_dir_offset as u32));

        // UTF-16 length-prefixed resource type string "DLL" (8 bytes -> keeps 0x20 4-byte aligned)
        write_u16(&mut rsrc, type_name_offset, 3);
        write_u16(&mut rsrc, type_name_offset + 2, u16::from(b'D'));
        write_u16(&mut rsrc, type_name_offset + 4, u16::from(b'L'));
        write_u16(&mut rsrc, type_name_offset + 6, u16::from(b'L'));

        // "DLL" directory: `count` ID entries
        write_u16(&mut rsrc, dll_dir_offset + 14, count as u16);

        let section_rva = 0x1000_u32;
        let mut cursor = blobs_start;
        for (idx, &(res_id, payload)) in blobs.iter().enumerate() {
            let id_dir_off = id_dirs_start + idx * 24;
            let data_entry_off = data_entries_start + idx * 16;

            let dll_entry_off = dll_entries_offset + idx * 8;
            write_u32(&mut rsrc, dll_entry_off, u32::from(res_id));
            write_u32(
                &mut rsrc,
                dll_entry_off + 4,
                0x8000_0000 | (id_dir_off as u32),
            );

            write_u16(&mut rsrc, id_dir_off + 14, 1);
            write_u32(&mut rsrc, id_dir_off + 16, 1033);
            write_u32(&mut rsrc, id_dir_off + 20, data_entry_off as u32);

            write_u32(&mut rsrc, data_entry_off, section_rva + cursor as u32);
            write_u32(&mut rsrc, data_entry_off + 4, payload.len() as u32);
            rsrc[cursor..cursor + payload.len()].copy_from_slice(payload);
            cursor += payload.len();
        }

        let section_raw_offset = 0x200_usize;
        let section_aligned_len = rsrc.len().div_ceil(0x200) * 0x200;
        let mut pe = vec![0_u8; section_raw_offset + section_aligned_len];

        write_u16(&mut pe, 0x00, 0x5A4D);
        write_u32(&mut pe, 0x3C, 0x80);
        write_u32(&mut pe, 0x80, 0x0000_4550);
        write_u16(&mut pe, 0x84, pelite::image::IMAGE_FILE_MACHINE_I386);
        write_u16(&mut pe, 0x86, 1);
        write_u16(&mut pe, 0x94, 0xE0);

        let opt = 0x98_usize;
        write_u16(&mut pe, opt, 0x10B);
        write_u32(&mut pe, opt + 0x20, 0x1000);
        write_u32(&mut pe, opt + 0x24, 0x200);
        write_u32(
            &mut pe,
            opt + 0x38,
            0x1000 + (section_aligned_len as u32).div_ceil(0x1000) * 0x1000,
        );
        write_u32(&mut pe, opt + 0x3C, 0x200);
        write_u32(&mut pe, opt + 0x5C, 16);
        // DataDirectory[2] = Resources (at opt + 0x60 + 2 * 8 = opt + 0x70)
        write_u32(&mut pe, opt + 0x70, section_rva);
        write_u32(&mut pe, opt + 0x74, rsrc.len() as u32);

        let sec = opt + 0xE0;
        pe[sec..sec + 8].copy_from_slice(b".rsrc\0\0\0");
        write_u32(&mut pe, sec + 8, rsrc.len() as u32);
        write_u32(&mut pe, sec + 12, section_rva);
        write_u32(&mut pe, sec + 16, section_aligned_len as u32);
        write_u32(&mut pe, sec + 20, section_raw_offset as u32);

        pe[section_raw_offset..section_raw_offset + rsrc.len()].copy_from_slice(&rsrc);
        pe
    }

    #[test]
    fn provisions_openal_dlls_from_oalinst_without_overwriting_or_blocking() {
        let temp = TempDir::new().unwrap();
        let game = temp.path().join("game");
        fs::create_dir_all(&game).unwrap();
        let compat = temp.path().join("managed/.chriz/proton");

        // Missing oalinst.exe succeeds cleanly as a no-op.
        prepare_proton_environment(&compat, &game, None, None).unwrap();
        assert!(!compat.join("pfx/drive_c/windows/system32").exists());

        let x64_openal =
            build_minimal_export_dll(pelite::image::IMAGE_FILE_MACHINE_AMD64, b"OpenAL32.dll");
        let x86_wrap =
            build_minimal_export_dll(pelite::image::IMAGE_FILE_MACHINE_I386, b"wrap_oal.dll");
        let unexpected =
            build_minimal_export_dll(pelite::image::IMAGE_FILE_MACHINE_AMD64, b"evil.dll");

        let z_x64_openal = zlib_compress(&x64_openal);
        let z_x86_wrap = zlib_compress(&x86_wrap);
        let z_unexpected = zlib_compress(&unexpected);

        let oalinst = build_minimal_oalinst_pe(&[
            (104, &z_x86_wrap),
            (114, &z_x64_openal),
            (199, &z_unexpected),
        ]);
        fs::write(game.join("OALINST.EXE"), oalinst).unwrap();

        // Works even when managed_save_root is None.
        prepare_proton_environment(&compat, &game, None, None).unwrap();

        let sys32 = compat.join("pfx/drive_c/windows/system32");
        let wow64 = compat.join("pfx/drive_c/windows/syswow64");
        assert_eq!(fs::read(sys32.join("OpenAL32.dll")).unwrap(), x64_openal);
        assert_eq!(fs::read(wow64.join("wrap_oal.dll")).unwrap(), x86_wrap);
        assert!(!sys32.join("evil.dll").exists());
        assert!(!wow64.join("evil.dll").exists());

        // Existing DLLs are never overwritten on subsequent launches.
        let sentinel = b"custom-openal".to_vec();
        fs::write(sys32.join("OpenAL32.dll"), &sentinel).unwrap();
        prepare_proton_environment(&compat, &game, None, None).unwrap();
        assert_eq!(fs::read(sys32.join("OpenAL32.dll")).unwrap(), sentinel);
    }

    #[cfg(unix)]
    #[test]
    fn skips_openal_write_when_prefix_system32_is_symlinked() {
        let temp = TempDir::new().unwrap();
        let game = temp.path().join("game");
        fs::create_dir_all(&game).unwrap();
        let x64_openal =
            build_minimal_export_dll(pelite::image::IMAGE_FILE_MACHINE_AMD64, b"OpenAL32.dll");
        let z_x64_openal = zlib_compress(&x64_openal);
        fs::write(
            game.join("oalinst.exe"),
            build_minimal_oalinst_pe(&[(114, &z_x64_openal)]),
        )
        .unwrap();

        let outside = temp.path().join("outside-system32");
        fs::create_dir_all(&outside).unwrap();

        let compat = temp.path().join("managed/.chriz/proton");
        let windows_dir = compat.join("pfx/drive_c/windows");
        fs::create_dir_all(&windows_dir).unwrap();
        std::os::unix::fs::symlink(&outside, windows_dir.join("system32")).unwrap();

        prepare_proton_environment(&compat, &game, None, None).unwrap();
        assert!(!outside.join("OpenAL32.dll").exists());
    }
}
