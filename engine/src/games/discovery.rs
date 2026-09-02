//! Steam and GOG installed-game discovery behind injectable providers.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Component, Path, PathBuf};

use super::probe::{inspect_profile_paths, FileKind, FileSystemProvider, SystemFileSystem};
use super::{GameCandidate, GameError, GameProfile, GameProfiles, GameRole, Result, Storefront};

const STEAM_REGISTRY_LOCATIONS: &[(RegistryHive, &str, &str)] = &[
    (
        RegistryHive::CurrentUser,
        r"Software\Valve\Steam",
        "SteamPath",
    ),
    (
        RegistryHive::LocalMachine,
        r"SOFTWARE\Valve\Steam",
        "InstallPath",
    ),
    (
        RegistryHive::LocalMachine,
        r"SOFTWARE\WOW6432Node\Valve\Steam",
        "InstallPath",
    ),
];

const GOG_REGISTRY_LOCATIONS: &[(RegistryHive, &str)] = &[
    (RegistryHive::LocalMachine, r"SOFTWARE\GOG.com\Games"),
    (
        RegistryHive::LocalMachine,
        r"SOFTWARE\WOW6432Node\GOG.com\Games",
    ),
    (RegistryHive::CurrentUser, r"SOFTWARE\GOG.com\Games"),
    (
        RegistryHive::CurrentUser,
        r"SOFTWARE\WOW6432Node\GOG.com\Games",
    ),
];

/// Supported Windows registry hive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegistryHive {
    /// Per-user installed-store metadata.
    CurrentUser,
    /// Machine-wide installed-store metadata.
    LocalMachine,
}

/// Read-only registry operations needed for installed-game discovery.
pub trait RegistryProvider {
    /// Reads one string value, returning `None` when the key or value is absent.
    fn read_string(&self, hive: RegistryHive, key: &str, name: &str) -> io::Result<Option<String>>;
    /// Lists direct subkey names, returning an empty list when the key is absent.
    fn subkeys(&self, hive: RegistryHive, key: &str) -> io::Result<Vec<String>>;
}

/// Host Windows registry provider. On non-Windows hosts it reports no stores.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemRegistry;

#[cfg(windows)]
impl RegistryProvider for SystemRegistry {
    fn read_string(&self, hive: RegistryHive, key: &str, name: &str) -> io::Result<Option<String>> {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
        use winreg::RegKey;

        let predefined = match hive {
            RegistryHive::CurrentUser => RegKey::predef(HKEY_CURRENT_USER),
            RegistryHive::LocalMachine => RegKey::predef(HKEY_LOCAL_MACHINE),
        };
        let opened = match predefined.open_subkey_with_flags(key, KEY_READ) {
            Ok(opened) => opened,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        match opened.get_value(name) {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn subkeys(&self, hive: RegistryHive, key: &str) -> io::Result<Vec<String>> {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
        use winreg::RegKey;

        let predefined = match hive {
            RegistryHive::CurrentUser => RegKey::predef(HKEY_CURRENT_USER),
            RegistryHive::LocalMachine => RegKey::predef(HKEY_LOCAL_MACHINE),
        };
        let opened = match predefined.open_subkey_with_flags(key, KEY_READ) {
            Ok(opened) => opened,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        opened.enum_keys().collect()
    }
}

#[cfg(not(windows))]
impl RegistryProvider for SystemRegistry {
    fn read_string(
        &self,
        _hive: RegistryHive,
        _key: &str,
        _name: &str,
    ) -> io::Result<Option<String>> {
        Ok(None)
    }

    fn subkeys(&self, _hive: RegistryHive, _key: &str) -> io::Result<Vec<String>> {
        Ok(Vec::new())
    }
}

/// Discovers and inspects installed games using the host read-only providers.
pub fn discover_installed_games(profiles: &GameProfiles) -> Result<Vec<GameCandidate>> {
    discover_games(profiles, &SystemRegistry, &SystemFileSystem)
}

/// Discovers Steam first and GOG second, then probes every deduplicated candidate.
pub fn discover_games<R: RegistryProvider, F: FileSystemProvider>(
    profiles: &GameProfiles,
    registry: &R,
    fs_provider: &F,
) -> Result<Vec<GameCandidate>> {
    let mut discovered = Vec::new();
    discover_steam(profiles, registry, fs_provider, &mut discovered)?;
    discover_gog(profiles, registry, fs_provider, &mut discovered)?;

    discovered.sort_by(|left, right| {
        (left.storefront, left.role, folded_path(&left.root)).cmp(&(
            right.storefront,
            right.role,
            folded_path(&right.root),
        ))
    });
    discovered.dedup_by(|left, right| {
        left.role == right.role
            && left.storefront == right.storefront
            && folded_path(&left.root) == folded_path(&right.root)
    });
    Ok(discovered)
}

fn discover_steam<R: RegistryProvider, F: FileSystemProvider>(
    profiles: &GameProfiles,
    registry: &R,
    fs_provider: &F,
    output: &mut Vec<GameCandidate>,
) -> Result<()> {
    let mut steam_roots = BTreeSet::new();
    for &(hive, key, value_name) in STEAM_REGISTRY_LOCATIONS {
        let value = registry
            .read_string(hive, key, value_name)
            .map_err(|source| GameError::Registry {
                location: format!("{hive:?}\\{key}::{value_name}"),
                source,
            })?;
        if let Some(value) = value {
            let root = PathBuf::from(value.trim());
            if root.as_os_str().is_empty() || !root.is_absolute() {
                return Err(GameError::Registry {
                    location: format!("{hive:?}\\{key}::{value_name}"),
                    source: io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Steam root is empty or not absolute",
                    ),
                });
            }
            steam_roots.insert(root);
        }
    }

    let mut steam_profiles = BTreeMap::<(u32, GameRole), Vec<&GameProfile>>::new();
    for profile in profiles
        .iter()
        .filter(|profile| profile.storefront == Storefront::Steam)
    {
        if let Some(app_id) = profile.steam_app_id {
            steam_profiles
                .entry((app_id, profile.role))
                .or_default()
                .push(profile);
        }
    }
    for group in steam_profiles.values_mut() {
        group.sort_by_key(|profile| (&profile.product_version, &profile.id));
    }
    for steam_root in steam_roots {
        let libraries = steam_libraries(fs_provider, &steam_root)?;
        for library in libraries {
            for (&(app_id, _), profile_group) in &steam_profiles {
                let manifest_path = library
                    .join("steamapps")
                    .join(format!("appmanifest_{app_id}.acf"));
                let bytes = match fs_provider.read(&manifest_path) {
                    Ok(bytes) => bytes,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(source) => {
                        return Err(GameError::Io {
                            path: manifest_path,
                            source,
                        })
                    }
                };
                let document = parse_vdf(&manifest_path, &bytes)?;
                let app_state =
                    object(&document, "AppState").ok_or_else(|| GameError::SteamMetadata {
                        path: manifest_path.clone(),
                        message: "missing AppState object".to_owned(),
                    })?;
                let declared_app_id =
                    text_value(app_state, "appid").ok_or_else(|| GameError::SteamMetadata {
                        path: manifest_path.clone(),
                        message: "missing appid".to_owned(),
                    })?;
                if declared_app_id != app_id.to_string() {
                    return Err(GameError::SteamMetadata {
                        path: manifest_path,
                        message: format!(
                            "appid {declared_app_id:?} does not match expected {app_id}"
                        ),
                    });
                }
                let install_dir = text_value(app_state, "installdir").ok_or_else(|| {
                    GameError::SteamMetadata {
                        path: manifest_path.clone(),
                        message: "missing installdir".to_owned(),
                    }
                })?;
                if !safe_install_dir(install_dir) {
                    return Err(GameError::SteamMetadata {
                        path: manifest_path,
                        message: format!("unsafe installdir {install_dir:?}"),
                    });
                }
                let root = library.join("steamapps/common").join(install_dir);
                if !is_candidate_directory(fs_provider, &root)? {
                    continue;
                }
                output.push(inspect_profile_paths(fs_provider, profile_group, &root)?);
            }
        }
    }
    Ok(())
}

fn steam_libraries<F: FileSystemProvider>(
    fs_provider: &F,
    steam_root: &Path,
) -> Result<Vec<PathBuf>> {
    let mut libraries = BTreeMap::<String, PathBuf>::new();
    libraries.insert(folded_path(steam_root), steam_root.to_path_buf());
    let path = steam_root.join("steamapps/libraryfolders.vdf");
    let bytes = match fs_provider.read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(libraries.into_values().collect())
        }
        Err(source) => return Err(GameError::Io { path, source }),
    };
    let document = parse_vdf(&path, &bytes)?;
    let folders = object(&document, "libraryfolders").ok_or_else(|| GameError::SteamMetadata {
        path: path.clone(),
        message: "missing libraryfolders object".to_owned(),
    })?;
    for (_, value) in folders {
        let VdfValue::Object(entry) = value else {
            continue;
        };
        let Some(library_path) = text_value(entry, "path") else {
            continue;
        };
        if library_path.trim().is_empty() {
            return Err(GameError::SteamMetadata {
                path,
                message: "library path is empty".to_owned(),
            });
        }
        let library = PathBuf::from(library_path.trim());
        if !library.is_absolute() {
            return Err(GameError::SteamMetadata {
                path,
                message: format!("library path {library_path:?} is not absolute"),
            });
        }
        libraries.insert(folded_path(&library), library);
    }
    Ok(libraries.into_values().collect())
}

fn discover_gog<R: RegistryProvider, F: FileSystemProvider>(
    profiles: &GameProfiles,
    registry: &R,
    fs_provider: &F,
    output: &mut Vec<GameCandidate>,
) -> Result<()> {
    for &(hive, root_key) in GOG_REGISTRY_LOCATIONS {
        let mut records =
            registry
                .subkeys(hive, root_key)
                .map_err(|source| GameError::Registry {
                    location: format!("{hive:?}\\{root_key}"),
                    source,
                })?;
        records.sort_by_key(|value| value.to_ascii_lowercase());
        for record in records {
            let key = format!(r"{root_key}\{record}");
            let game_id = registry
                .read_string(hive, &key, "gameID")
                .map_err(|source| GameError::Registry {
                    location: format!("{hive:?}\\{key}::gameID"),
                    source,
                })?
                .unwrap_or_else(|| record.clone());
            let matched_profiles = profiles
                .iter()
                .filter(|profile| profile.storefront == Storefront::Gog)
                .filter(|profile| {
                    profile
                        .gog_game_ids
                        .iter()
                        .any(|id| id.eq_ignore_ascii_case(&game_id))
                })
                .collect::<Vec<_>>();
            if matched_profiles.is_empty() {
                continue;
            }
            let value = registry
                .read_string(hive, &key, "path")
                .map_err(|source| GameError::Registry {
                    location: format!("{hive:?}\\{key}::path"),
                    source,
                })?
                .ok_or_else(|| GameError::Registry {
                    location: format!("{hive:?}\\{key}::path"),
                    source: io::Error::new(
                        io::ErrorKind::InvalidData,
                        "recognized GOG record has no path",
                    ),
                })?;
            let root = PathBuf::from(value);
            if root.as_os_str().is_empty() || !root.is_absolute() {
                return Err(GameError::Registry {
                    location: format!("{hive:?}\\{key}::path"),
                    source: io::Error::new(
                        io::ErrorKind::InvalidData,
                        "recognized GOG path is empty or not absolute",
                    ),
                });
            }
            if !is_candidate_directory(fs_provider, &root)? {
                continue;
            }
            let mut role_groups = BTreeMap::<GameRole, Vec<&GameProfile>>::new();
            for profile in matched_profiles {
                role_groups.entry(profile.role).or_default().push(profile);
            }
            for profile_group in role_groups.values_mut() {
                profile_group.sort_by_key(|profile| (&profile.product_version, &profile.id));
                output.push(inspect_profile_paths(fs_provider, profile_group, &root)?);
            }
        }
    }
    Ok(())
}

fn is_candidate_directory<F: FileSystemProvider>(fs_provider: &F, root: &Path) -> Result<bool> {
    match fs_provider.file_kind(root) {
        Ok(FileKind::Directory) => Ok(true),
        Ok(_) => Ok(false),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(GameError::Io {
            path: root.to_path_buf(),
            source,
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VdfValue {
    Text(String),
    Object(Vec<(String, VdfValue)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Text(String),
    Open,
    Close,
}

fn parse_vdf(path: &Path, bytes: &[u8]) -> Result<Vec<(String, VdfValue)>> {
    let text = std::str::from_utf8(bytes).map_err(|error| GameError::SteamMetadata {
        path: path.to_path_buf(),
        message: format!("metadata is not UTF-8: {error}"),
    })?;
    let tokens = lex_vdf(path, text)?;
    let mut position = 0;
    let document = parse_object(path, &tokens, &mut position, false)?;
    if position != tokens.len() {
        return Err(GameError::SteamMetadata {
            path: path.to_path_buf(),
            message: "unexpected trailing tokens".to_owned(),
        });
    }
    Ok(document)
}

fn lex_vdf(path: &Path, text: &str) -> Result<Vec<Token>> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_whitespace() => index += 1,
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'{' => {
                tokens.push(Token::Open);
                index += 1;
            }
            b'}' => {
                tokens.push(Token::Close);
                index += 1;
            }
            b'"' => {
                index += 1;
                let mut value = Vec::new();
                let mut closed = false;
                while index < bytes.len() {
                    match bytes[index] {
                        b'"' => {
                            index += 1;
                            closed = true;
                            break;
                        }
                        b'\\' => {
                            let Some(&next) = bytes.get(index + 1) else {
                                break;
                            };
                            match next {
                                b'\\' | b'"' => value.push(next),
                                _ => {
                                    value.push(b'\\');
                                    value.push(next);
                                }
                            }
                            index += 2;
                        }
                        byte => {
                            value.push(byte);
                            index += 1;
                        }
                    }
                }
                if !closed {
                    return Err(GameError::SteamMetadata {
                        path: path.to_path_buf(),
                        message: "unterminated quoted string".to_owned(),
                    });
                }
                let value = String::from_utf8(value).map_err(|error| GameError::SteamMetadata {
                    path: path.to_path_buf(),
                    message: format!("quoted value is not UTF-8: {error}"),
                })?;
                tokens.push(Token::Text(value));
            }
            other => {
                return Err(GameError::SteamMetadata {
                    path: path.to_path_buf(),
                    message: format!("unexpected byte {other:#x} outside a quoted string"),
                })
            }
        }
    }
    Ok(tokens)
}

fn parse_object(
    path: &Path,
    tokens: &[Token],
    position: &mut usize,
    expect_close: bool,
) -> Result<Vec<(String, VdfValue)>> {
    let mut values = Vec::new();
    loop {
        match tokens.get(*position) {
            Some(Token::Close) if expect_close => {
                *position += 1;
                return Ok(values);
            }
            None if !expect_close => return Ok(values),
            None => {
                return Err(GameError::SteamMetadata {
                    path: path.to_path_buf(),
                    message: "object is missing a closing brace".to_owned(),
                })
            }
            Some(Token::Text(key)) => {
                let key = key.clone();
                *position += 1;
                let value = match tokens.get(*position) {
                    Some(Token::Text(value)) => {
                        *position += 1;
                        VdfValue::Text(value.clone())
                    }
                    Some(Token::Open) => {
                        *position += 1;
                        VdfValue::Object(parse_object(path, tokens, position, true)?)
                    }
                    _ => {
                        return Err(GameError::SteamMetadata {
                            path: path.to_path_buf(),
                            message: format!("key {key:?} has no string or object value"),
                        })
                    }
                };
                values.push((key, value));
            }
            Some(_) => {
                return Err(GameError::SteamMetadata {
                    path: path.to_path_buf(),
                    message: "expected a quoted key".to_owned(),
                })
            }
        }
    }
}

fn object<'a>(values: &'a [(String, VdfValue)], key: &str) -> Option<&'a [(String, VdfValue)]> {
    values.iter().find_map(|(candidate, value)| {
        if candidate.eq_ignore_ascii_case(key) {
            if let VdfValue::Object(object) = value {
                return Some(object.as_slice());
            }
        }
        None
    })
}

fn text_value<'a>(values: &'a [(String, VdfValue)], key: &str) -> Option<&'a str> {
    values.iter().find_map(|(candidate, value)| {
        if candidate.eq_ignore_ascii_case(key) {
            if let VdfValue::Text(value) = value {
                return Some(value.as_str());
            }
        }
        None
    })
}

fn safe_install_dir(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(['/', '\\', ':'])
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn folded_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{object, parse_vdf, text_value};

    #[test]
    fn parses_nested_keyvalues_and_escaped_paths() {
        let parsed = parse_vdf(
            Path::new("fixture.vdf"),
            br#""libraryfolders" { "1" { "path" "D:\\Steam Library" "apps" { "1" "1" } } }"#,
        )
        .unwrap();
        let folders = object(&parsed, "libraryfolders").unwrap();
        let one = object(folders, "1").unwrap();
        assert_eq!(text_value(one, "path"), Some(r"D:\Steam Library"));
    }
}
