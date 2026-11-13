//! Linux game launch through Steam Proton or Wine with an isolated per-install prefix.
//!
//! Managed campaigns keep their own `STEAM_COMPAT_DATA_PATH` / `WINEPREFIX` under
//! `<managed_root>/.chriz/proton` and point their Wine/Proton `Documents/<engine_name>` save folder
//! at `<managed_save_root>` without mutating `InfinityLoader.ini` (InfinityLoader auto-detects
//! Wine via `wine_get_version`).

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
    let current_user = std::env::var("USER").ok();
    prepare_proton_environment(
        &spec.compat_data,
        working_directory,
        managed_save_root,
        current_user.as_deref(),
    )
    .map_err(|error| format!("prepare isolated Wine/Proton prefix: {error}"))?;

    let mut command = std::process::Command::new(&spec.program);
    command.args(&spec.args).current_dir(working_directory);
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

/// Ensures the isolated Proton/Wine prefix directory exists and links its `Documents/<engine_name>`
/// save folder to `managed_save_root` without mutating non-empty existing save directories.
pub fn prepare_proton_environment(
    compat_data: &Path,
    working_directory: &Path,
    managed_save_root: Option<&Path>,
    current_user: Option<&str>,
) -> io::Result<()> {
    let pfx = compat_data.join("pfx");
    fs::create_dir_all(&pfx)?;

    let Some(save_root) = managed_save_root else {
        return Ok(());
    };
    let Some(engine_name) = read_engine_name(working_directory) else {
        return Ok(());
    };

    fs::create_dir_all(save_root)?;
    let steamuser_docs = pfx
        .join("drive_c")
        .join("users")
        .join("steamuser")
        .join("Documents");
    fs::create_dir_all(&steamuser_docs)?;
    link_save_directory(&steamuser_docs.join(&engine_name), save_root)?;

    if let Some(user) = current_user
        .map(str::trim)
        .filter(|name| !name.is_empty() && !name.eq_ignore_ascii_case("steamuser"))
    {
        let host_user_root = pfx.join("drive_c").join("users").join(user);
        if host_user_root.is_dir() {
            let host_docs = host_user_root.join("Documents");
            fs::create_dir_all(&host_docs)?;
            link_save_directory(&host_docs.join(&engine_name), save_root)?;
        }
    }
    Ok(())
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
            prepare_proton_environment(&compat, &game, Some(&save_root), Some("deck")).unwrap();
            prepare_proton_environment(&compat, &game, Some(&save_root), Some("deck")).unwrap();
            let link = compat.join(
                "pfx/drive_c/users/steamuser/Documents/Baldur's Gate - Enhanced Edition Trilogy",
            );
            assert_eq!(fs::read_link(link).unwrap(), save_root);
        }
    }
}
