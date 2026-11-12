//! Steam, GOG and Beamdog installed-game discovery behind injectable providers.
//!
//! Every candidate folder is checked on its own: an unreadable store record or a malformed
//! manifest skips only that record, and a game folder that cannot be checked is listed with
//! its reason. One bad entry never hides the other games.

use std::collections::BTreeMap;
use std::io;
use std::path::{Component, Path, PathBuf};

use super::probe::{
    inspect_profile_paths, role_profiles, unreadable_candidate, FileKind, FileSystemProvider,
    SystemFileSystem,
};
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

/// Where the GOG installer and GOG Galaxy record installed games, one subkey per game with
/// the string values `gameID`, `path` and `language` (among others).
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

/// GOG product ids of the games (GOG database, gogdb.org): Baldur's Gate: Enhanced Edition,
/// whose Siege of Dragonspear is the DLC 1459335293 inside it, and Baldur's Gate II: Enhanced
/// Edition. Recipe profiles may name more ids in `gog_game_ids`.
pub const GOG_GAME_IDS: &[(GameRole, &str)] = &[
    (GameRole::BgeeSod, "1207666353"),
    (GameRole::Bg2ee, "1207666373"),
    (GameRole::Iwdee, "1207666683"),
];

/// Folder names that GOG installs use (GOG database). A folder of that name counts only when
/// it holds the game's `goggame-<id>.info` file.
const GOG_FOLDER_NAMES: &[(GameRole, &str)] = &[
    (GameRole::BgeeSod, "Baldur's Gate Enhanced Edition"),
    (GameRole::Bg2ee, "Baldur's Gate II - Enhanced Edition"),
    (GameRole::Bg2ee, "Baldur's Gate II Enhanced Edition"),
    (GameRole::Iwdee, "Icewind Dale - Enhanced Edition"),
    (GameRole::Iwdee, "Icewind Dale Enhanced Edition"),
];

/// Folder names the Beamdog client gave the games inside its `Games` folder: `00766` for
/// Baldur's Gate: Enhanced Edition, `00806` for it with Siege of Dragonspear, `00783` for
/// Baldur's Gate II: Enhanced Edition, and `00798` for Icewind Dale: Enhanced Edition
/// (Beamdog forums). The client closed in December 2025; installed games stay where it put them.
pub const BEAMDOG_FOLDERS: &[(GameRole, &str)] = &[
    (GameRole::BgeeSod, "00806"),
    (GameRole::BgeeSod, "00766"),
    (GameRole::Bg2ee, "00783"),
    (GameRole::Iwdee, "00798"),
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

/// Folders where stores put games by default, for stores without usable records: the
/// Beamdog client's `Games` folders and GOG's game libraries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoreFolders {
    /// Folders holding Beamdog game folders such as `00806`.
    pub beamdog_games: Vec<PathBuf>,
    /// Folders holding GOG game folders such as `Baldur's Gate Enhanced Edition`.
    pub gog_libraries: Vec<PathBuf>,
}

impl StoreFolders {
    /// The default folders on this PC: `Beamdog\Games` under both Program Files folders,
    /// `GOG Games` on the system drive (the GOG installer's default) and `GOG Galaxy\Games`
    /// under both Program Files folders (GOG Galaxy's default).
    pub fn system() -> Self {
        let program_files = ["ProgramFiles(x86)", "ProgramFiles"]
            .into_iter()
            .filter_map(std::env::var_os)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .collect::<Vec<_>>();
        let system_drive = std::env::var_os("SystemDrive")
            .map(|drive| PathBuf::from(format!("{}\\", drive.to_string_lossy())))
            .filter(|path| path.is_absolute());
        Self {
            beamdog_games: program_files
                .iter()
                .map(|root| root.join("Beamdog").join("Games"))
                .collect(),
            gog_libraries: system_drive
                .into_iter()
                .map(|drive| drive.join("GOG Games"))
                .chain(
                    program_files
                        .iter()
                        .map(|root| root.join("GOG Galaxy").join("Games")),
                )
                .collect(),
        }
    }
}

/// Discovers and inspects installed games using the host read-only providers.
pub fn discover_installed_games(profiles: &GameProfiles) -> Result<Vec<GameCandidate>> {
    discover_games_with(
        profiles,
        &SystemRegistry,
        &SystemFileSystem,
        &StoreFolders::system(),
    )
}

/// Discovers Steam and GOG games from their records, then probes every deduplicated
/// candidate. Default store folders are not searched; see [`discover_games_with`].
pub fn discover_games<R: RegistryProvider, F: FileSystemProvider>(
    profiles: &GameProfiles,
    registry: &R,
    fs_provider: &F,
) -> Result<Vec<GameCandidate>> {
    discover_games_with(profiles, registry, fs_provider, &StoreFolders::default())
}

/// Discovers Steam and GOG games from their records and Beamdog and GOG games in `folders`,
/// then probes every deduplicated candidate. Each candidate is checked on its own; the
/// result lists every game found, usable or not, and never fails because of one of them.
pub fn discover_games_with<R: RegistryProvider, F: FileSystemProvider>(
    profiles: &GameProfiles,
    registry: &R,
    fs_provider: &F,
    folders: &StoreFolders,
) -> Result<Vec<GameCandidate>> {
    let mut discovered = Vec::new();
    discover_steam(profiles, registry, fs_provider, &mut discovered);
    discover_gog(profiles, registry, fs_provider, &mut discovered);
    discover_gog_folders(
        profiles,
        fs_provider,
        &folders.gog_libraries,
        &mut discovered,
    );
    discover_beamdog(
        profiles,
        fs_provider,
        &folders.beamdog_games,
        &mut discovered,
    );

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

/// Inspects one discovered folder; a folder that cannot be checked is listed with the reason.
fn inspect_discovered<F: FileSystemProvider>(
    fs_provider: &F,
    profiles: &[&GameProfile],
    root: &Path,
    storefront: Storefront,
    language: Option<&str>,
    output: &mut Vec<GameCandidate>,
) {
    let Some(role) = profiles.first().map(|profile| profile.role) else {
        return;
    };
    match is_candidate_directory(fs_provider, root) {
        Ok(true) => {}
        Ok(false) => return,
        Err(error) => {
            output.push(unreadable_candidate(role, storefront, root, &error));
            return;
        }
    }
    output.push(
        match inspect_profile_paths(fs_provider, profiles, root, storefront, language) {
            Ok(candidate) => candidate,
            Err(error) => unreadable_candidate(role, storefront, root, &error),
        },
    );
}

fn discover_steam<R: RegistryProvider, F: FileSystemProvider>(
    profiles: &GameProfiles,
    registry: &R,
    fs_provider: &F,
    output: &mut Vec<GameCandidate>,
) {
    let mut steam_roots = BTreeMap::<String, PathBuf>::new();
    for &(hive, key, value_name) in STEAM_REGISTRY_LOCATIONS {
        // A record that cannot be read or names no usable folder is skipped; the other
        // records may still find Steam.
        let Ok(Some(value)) = registry.read_string(hive, key, value_name) else {
            continue;
        };
        let root = PathBuf::from(value.trim());
        if root.as_os_str().is_empty() || !root.is_absolute() {
            continue;
        }
        steam_roots.insert(folded_path(&root), root);
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
    for steam_root in steam_roots.into_values() {
        for library in steam_libraries(fs_provider, &steam_root) {
            for (&(app_id, _), profile_group) in &steam_profiles {
                let manifest_path = library
                    .join("steamapps")
                    .join(format!("appmanifest_{app_id}.acf"));
                let Some(app) = steam_app(fs_provider, &manifest_path, app_id) else {
                    continue;
                };
                let root = library.join("steamapps/common").join(&app.install_dir);
                inspect_discovered(
                    fs_provider,
                    profile_group,
                    &root,
                    Storefront::Steam,
                    app.language.as_deref(),
                    output,
                );
            }
        }
    }
}

/// What one Steam app manifest says about an installed game.
struct SteamApp {
    install_dir: String,
    language: Option<String>,
}

/// Reads one app manifest. An absent, unreadable, malformed or unsafe manifest gives `None`:
/// its game cannot be located, and the other manifests are read on.
fn steam_app<F: FileSystemProvider>(
    fs_provider: &F,
    manifest_path: &Path,
    app_id: u32,
) -> Option<SteamApp> {
    let bytes = fs_provider.read(manifest_path).ok()?;
    let document = parse_vdf(manifest_path, &bytes).ok()?;
    let app_state = object(&document, "AppState")?;
    if text_value(app_state, "appid")? != app_id.to_string() {
        return None;
    }
    let install_dir = text_value(app_state, "installdir")?;
    if !safe_install_dir(install_dir) {
        return None;
    }
    // Steam keeps the game language the player picked in `UserConfig`.
    let language = ["UserConfig", "MountedConfig"].iter().find_map(|section| {
        object(app_state, section)
            .and_then(|config| text_value(config, "language"))
            .filter(|language| !language.trim().is_empty())
            .map(str::to_owned)
    });
    Some(SteamApp {
        install_dir: install_dir.to_owned(),
        language,
    })
}

/// The Steam root and every library `libraryfolders.vdf` names. A missing or malformed file,
/// or a library entry without a usable absolute path, leaves just the libraries it can name.
fn steam_libraries<F: FileSystemProvider>(fs_provider: &F, steam_root: &Path) -> Vec<PathBuf> {
    let mut libraries = BTreeMap::<String, PathBuf>::new();
    libraries.insert(folded_path(steam_root), steam_root.to_path_buf());
    let path = steam_root.join("steamapps/libraryfolders.vdf");
    let Some(document) = fs_provider
        .read(&path)
        .ok()
        .and_then(|bytes| parse_vdf(&path, &bytes).ok())
    else {
        return libraries.into_values().collect();
    };
    if let Some(folders) = object(&document, "libraryfolders") {
        for (_, value) in folders {
            let VdfValue::Object(entry) = value else {
                continue;
            };
            let Some(library_path) = text_value(entry, "path") else {
                continue;
            };
            let library = PathBuf::from(library_path.trim());
            if library.as_os_str().is_empty() || !library.is_absolute() {
                continue;
            }
            libraries.insert(folded_path(&library), library);
        }
    }
    libraries.into_values().collect()
}

/// The roles a GOG product id stands for: the known ids, and the ids recipe profiles name.
fn gog_roles(profiles: &GameProfiles, game_id: &str) -> Vec<GameRole> {
    let mut roles = GOG_GAME_IDS
        .iter()
        .filter(|(_, id)| id.eq_ignore_ascii_case(game_id.trim()))
        .map(|(role, _)| *role)
        .collect::<Vec<_>>();
    for profile in profiles
        .iter()
        .filter(|profile| profile.storefront == Storefront::Gog)
    {
        if profile
            .gog_game_ids
            .iter()
            .any(|id| id.eq_ignore_ascii_case(game_id.trim()))
            && !roles.contains(&profile.role)
        {
            roles.push(profile.role);
        }
    }
    roles.sort();
    roles
}

fn discover_gog<R: RegistryProvider, F: FileSystemProvider>(
    profiles: &GameProfiles,
    registry: &R,
    fs_provider: &F,
    output: &mut Vec<GameCandidate>,
) {
    for &(hive, root_key) in GOG_REGISTRY_LOCATIONS {
        let Ok(mut records) = registry.subkeys(hive, root_key) else {
            continue;
        };
        records.sort_by_key(|value| value.to_ascii_lowercase());
        for record in records {
            let key = format!(r"{root_key}\{record}");
            let game_id = match registry.read_string(hive, &key, "gameID") {
                Ok(Some(game_id)) => game_id,
                Ok(None) => record.clone(),
                Err(_) => continue,
            };
            let roles = gog_roles(profiles, &game_id);
            if roles.is_empty() {
                continue;
            }
            let Ok(Some(value)) = registry.read_string(hive, &key, "path") else {
                continue;
            };
            let root = PathBuf::from(value.trim());
            if root.as_os_str().is_empty() || !root.is_absolute() {
                continue;
            }
            let language = registry.read_string(hive, &key, "language").ok().flatten();
            for role in roles {
                let group = role_profiles(profiles, role, Storefront::Gog);
                inspect_discovered(
                    fs_provider,
                    &group,
                    &root,
                    Storefront::Gog,
                    language.as_deref(),
                    output,
                );
            }
        }
    }
}

/// GOG games in the default GOG folders, for installs whose registry record is missing. A
/// folder counts only when it holds the game's `goggame-<id>.info` file.
fn discover_gog_folders<F: FileSystemProvider>(
    profiles: &GameProfiles,
    fs_provider: &F,
    libraries: &[PathBuf],
    output: &mut Vec<GameCandidate>,
) {
    for library in libraries {
        for &(role, name) in GOG_FOLDER_NAMES {
            let root = library.join(name);
            let Some(info) = GOG_GAME_IDS
                .iter()
                .filter(|(id_role, _)| *id_role == role)
                .map(|(_, id)| root.join(format!("goggame-{id}.info")))
                .find(|info| matches!(fs_provider.file_kind(info), Ok(FileKind::File)))
            else {
                continue;
            };
            let language = gog_info_language(fs_provider, &info);
            let group = role_profiles(profiles, role, Storefront::Gog);
            inspect_discovered(
                fs_provider,
                &group,
                &root,
                Storefront::Gog,
                language.as_deref(),
                output,
            );
        }
    }
}

/// The `language` a GOG game's `goggame-<id>.info` file names, when it can be read.
fn gog_info_language<F: FileSystemProvider>(fs_provider: &F, info: &Path) -> Option<String> {
    let bytes = fs_provider.read(info).ok()?;
    let document = serde_json::from_slice::<serde_json::Value>(&bytes).ok()?;
    document
        .get("language")
        .and_then(serde_json::Value::as_str)
        .filter(|language| !language.trim().is_empty())
        .map(str::to_owned)
}

/// Beamdog client games in its default `Games` folders.
fn discover_beamdog<F: FileSystemProvider>(
    profiles: &GameProfiles,
    fs_provider: &F,
    games_folders: &[PathBuf],
    output: &mut Vec<GameCandidate>,
) {
    for games in games_folders {
        for &(role, folder) in BEAMDOG_FOLDERS {
            let group = role_profiles(profiles, role, Storefront::Beamdog);
            inspect_discovered(
                fs_provider,
                &group,
                &games.join(folder),
                Storefront::Beamdog,
                None,
                output,
            );
        }
    }
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
