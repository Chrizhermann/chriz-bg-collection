use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use bg_engine::games::discovery::StoreFolders;
use bg_engine::games::{
    detect_storefront, discover_games, discover_games_with, inspect_game_path,
    inspect_game_path_with_language, DirectoryEntry, Eligibility, FileKind, FileSystemProvider,
    FindingKind, GameProfiles, GameRole, RegistryHive, RegistryProvider, Storefront,
    SystemFileSystem,
};
use sha2::{Digest, Sha256};

const FIXTURES: &str = "tests/fixtures/games";
const STEAM_KEY: &str = r"Software\Valve\Steam";
const GOG_KEY: &str = r"SOFTWARE\GOG.com\Games";

#[derive(Default)]
struct FixtureFileSystem {
    versions: BTreeMap<PathBuf, String>,
    read_errors: BTreeMap<PathBuf, io::ErrorKind>,
    directory_errors: BTreeMap<PathBuf, io::ErrorKind>,
    kind_overrides: BTreeMap<PathBuf, FileKind>,
    kind_errors: BTreeMap<PathBuf, io::ErrorKind>,
    directory_overrides: BTreeMap<PathBuf, Vec<DirectoryEntry>>,
}

impl FixtureFileSystem {
    fn add_version(&mut self, executable: PathBuf, version: &str) {
        self.versions
            .insert(fs::canonicalize(executable).unwrap(), version.to_owned());
    }

    fn override_kind(&mut self, path: PathBuf, kind: FileKind) {
        self.kind_overrides
            .insert(fs::canonicalize(path).unwrap(), kind);
    }

    fn override_kind_error(&mut self, path: PathBuf, kind: io::ErrorKind) {
        self.kind_errors
            .insert(fs::canonicalize(path).unwrap(), kind);
    }

    fn override_directory_entries(&mut self, path: PathBuf, entries: Vec<DirectoryEntry>) {
        self.directory_overrides
            .insert(fs::canonicalize(path).unwrap(), entries);
    }
}

impl FileSystemProvider for FixtureFileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        if let Some(kind) = self.read_errors.get(&fs::canonicalize(path)?) {
            return Err(io::Error::new(*kind, "injected read failure"));
        }
        fs::read(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>> {
        let canonical = fs::canonicalize(path)?;
        if let Some(kind) = self.directory_errors.get(&canonical) {
            return Err(io::Error::new(*kind, "injected directory read failure"));
        }
        if let Some(entries) = self.directory_overrides.get(&canonical) {
            return Ok(entries.clone());
        }
        fs::read_dir(path)?
            .map(|entry| {
                let entry = entry?;
                let metadata = fs::symlink_metadata(entry.path())?;
                let kind = if metadata.file_type().is_symlink() {
                    FileKind::Symlink
                } else if metadata.is_file() {
                    FileKind::File
                } else if metadata.is_dir() {
                    FileKind::Directory
                } else {
                    FileKind::Other
                };
                Ok(DirectoryEntry {
                    path: entry.path(),
                    name: entry.file_name(),
                    kind,
                })
            })
            .collect()
    }

    fn file_kind(&self, path: &Path) -> io::Result<FileKind> {
        let canonical = fs::canonicalize(path)?;
        if let Some(kind) = self.kind_errors.get(&canonical) {
            return Err(io::Error::new(*kind, "injected file-kind failure"));
        }
        if let Some(kind) = self.kind_overrides.get(&canonical) {
            return Ok(*kind);
        }
        let metadata = fs::symlink_metadata(path)?;
        Ok(if metadata.file_type().is_symlink() {
            FileKind::Symlink
        } else if metadata.is_file() {
            FileKind::File
        } else if metadata.is_dir() {
            FileKind::Directory
        } else {
            FileKind::Other
        })
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        fs::canonicalize(path)
    }

    fn product_version(&self, executable: &Path) -> io::Result<String> {
        self.versions.get(executable).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("no fixture ProductVersion for {}", executable.display()),
            )
        })
    }
}

#[derive(Default)]
struct FixtureRegistry {
    values: BTreeMap<(RegistryHive, String, String), String>,
    subkeys: BTreeMap<(RegistryHive, String), Vec<String>>,
}

impl FixtureRegistry {
    fn value(&mut self, hive: RegistryHive, key: &str, name: &str, value: impl Into<String>) {
        self.values.insert(
            (hive, key.to_ascii_lowercase(), name.to_ascii_lowercase()),
            value.into(),
        );
    }

    fn set_subkeys(&mut self, hive: RegistryHive, key: &str, values: &[&str]) {
        self.subkeys.insert(
            (hive, key.to_ascii_lowercase()),
            values.iter().map(|value| (*value).to_owned()).collect(),
        );
    }
}

impl RegistryProvider for FixtureRegistry {
    fn read_string(&self, hive: RegistryHive, key: &str, name: &str) -> io::Result<Option<String>> {
        Ok(self
            .values
            .get(&(hive, key.to_ascii_lowercase(), name.to_ascii_lowercase()))
            .cloned())
    }

    fn subkeys(&self, hive: RegistryHive, key: &str) -> io::Result<Vec<String>> {
        Ok(self
            .subkeys
            .get(&(hive, key.to_ascii_lowercase()))
            .cloned()
            .unwrap_or_default())
    }
}

fn profiles() -> GameProfiles {
    GameProfiles::load(Path::new(FIXTURES).join("profiles")).unwrap()
}

fn real_27_optional_surface_profiles(profile_dir: &Path) -> GameProfiles {
    fs::create_dir_all(profile_dir).unwrap();
    let profile = fs::read_to_string(Path::new(FIXTURES).join("profiles/steam-bg2ee.toml"))
        .unwrap()
        .replace("\r\n", "\n")
        .replace("  \"dialog.tlk\",\n", "")
        .replacen(
            "path = \"dialog.tlk\"\nkind = \"file\"",
            "path = \"dialog.tlk\"\nkind = \"file\"\nallow_missing = true",
            1,
        )
        .replacen(
            "path = \"override\"\nkind = \"tree\"",
            "path = \"override\"\nkind = \"tree\"\nallow_missing = true",
            1,
        )
        .replace(
            r#"[[allowed_clean_variants.inventory]]
path = "dialog.tlk"
kind = "file"
sha256 = "843d6c1d158b59096a2bec14275911d7e2eb4acc956406dd7dea1ddee0f63f9e"

"#,
            "",
        );
    assert!(profile.contains("allow_missing = true"));
    assert!(!profile.contains("  \"dialog.tlk\","));
    fs::write(profile_dir.join("steam-bg2ee.toml"), profile).unwrap();
    GameProfiles::load(profile_dir).unwrap()
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn fixture_game(
    fixture: &str,
    destination: &Path,
    fs_provider: &mut FixtureFileSystem,
    version: &str,
) -> PathBuf {
    copy_tree(
        &Path::new(FIXTURES).join("content").join(fixture),
        destination,
    );
    fs::create_dir_all(destination.join("override")).unwrap();
    fs_provider.add_version(destination.join("Baldur.exe"), version);
    destination.to_path_buf()
}

fn write_steam_metadata(steam_root: &Path, second_library: &Path) {
    let steamapps = steam_root.join("steamapps");
    fs::create_dir_all(&steamapps).unwrap();
    fs::create_dir_all(second_library.join("steamapps")).unwrap();
    let template =
        fs::read_to_string(Path::new(FIXTURES).join("steam/libraryfolders.vdf")).unwrap();
    let escaped = |path: &Path| path.to_string_lossy().replace('\\', "\\\\");
    fs::write(
        steamapps.join("libraryfolders.vdf"),
        template
            .replace("{{STEAM_ROOT}}", &escaped(steam_root))
            .replace("{{SECOND_LIBRARY}}", &escaped(second_library)),
    )
    .unwrap();
    fs::copy(
        Path::new(FIXTURES).join("steam/appmanifest_228280.acf"),
        steamapps.join("appmanifest_228280.acf"),
    )
    .unwrap();
    fs::copy(
        Path::new(FIXTURES).join("steam/appmanifest_257350.acf"),
        second_library
            .join("steamapps")
            .join("appmanifest_257350.acf"),
    )
    .unwrap();
}

fn has_finding(candidate: &bg_engine::games::GameCandidate, kind: FindingKind) -> bool {
    candidate
        .findings
        .iter()
        .any(|finding| finding.kind == kind)
}

#[test]
fn loads_fixture_backed_storefront_profiles() {
    let profiles = profiles();
    assert_eq!(profiles.len(), 4);
    assert!(profiles.iter().all(|profile| profile.locale == "en_US"));
    assert!(profiles
        .iter()
        .all(|profile| profile.product_version == "2.7.3.0"));
    assert_eq!(
        profiles
            .matching(GameRole::Bg2ee, Storefront::Steam)
            .next()
            .unwrap()
            .steam_app_id,
        Some(257350)
    );
}

#[test]
fn underspecified_profile_cannot_bless_an_arbitrary_directory() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join("unsafe.toml"),
        r#"
schema = 1
id = "unsafe"
role = "bg2ee"
storefront = "steam"
locale = "en_US"
executable = "Baldur.exe"
product_version = "2.7.3.0"
steam_app_id = 257350
required_files = []
sod_files = []
fingerprint_files = []
inventory_surfaces = []

[[allowed_clean_variants]]
id = "empty"
fingerprints = []
inventory = []
"#,
    )
    .unwrap();

    let error = GameProfiles::load(temp.path()).unwrap_err();
    assert!(error.to_string().contains("required_files"));
    assert!(error.to_string().contains("safety"));
}

#[test]
fn profile_must_fingerprint_its_product_version_executable() {
    let temp = tempfile::tempdir().unwrap();
    let profile = fs::read_to_string(Path::new(FIXTURES).join("profiles/steam-bg2ee.toml"))
        .unwrap()
        .replace(
            "fingerprint_files = [\"Baldur.exe\", \"data/base.bif\"]",
            "fingerprint_files = [\"data/base.bif\"]",
        );
    fs::write(temp.path().join("unsafe.toml"), profile).unwrap();

    let error = GameProfiles::load(temp.path()).unwrap_err();
    assert!(error.to_string().contains("executable"), "{error}");
    assert!(error.to_string().contains("fingerprint"), "{error}");
}

#[test]
fn profile_root_inventory_must_cover_setup_weidu_and_debug_residue() {
    let temp = tempfile::tempdir().unwrap();
    let profile = fs::read_to_string(Path::new(FIXTURES).join("profiles/steam-bg2ee.toml"))
        .unwrap()
        .replace(
            "patterns = [\"setup-*\", \"weidu*\", \"*.debug\", \"*.debug.log\", \"dlcmerger\", \"eet\", \"stratagems\"]",
            "patterns = [\"unrelated-*\"]",
        );
    fs::write(temp.path().join("unsafe.toml"), profile).unwrap();

    let error = GameProfiles::load(temp.path()).unwrap_err();
    assert!(error.to_string().contains("setup"), "{error}");
    assert!(error.to_string().contains("WeiDU"), "{error}");
    assert!(error.to_string().contains("debug"), "{error}");
}

#[test]
fn matching_inventory_surface_cannot_be_marked_allow_missing() {
    let temp = tempfile::tempdir().unwrap();
    let profile = fs::read_to_string(Path::new(FIXTURES).join("profiles/steam-bg2ee.toml"))
        .unwrap()
        .replace("\r\n", "\n")
        .replace(
            "path = \".\"\nkind = \"matching\"",
            "path = \".\"\nkind = \"matching\"\nallow_missing = true",
        );
    fs::write(temp.path().join("unsafe.toml"), profile).unwrap();

    let error = GameProfiles::load(temp.path()).unwrap_err();
    assert!(error.to_string().contains("matching"), "{error}");
    assert!(error.to_string().contains("allow missing"), "{error}");
}

#[test]
fn discovers_steam_games_from_registry_vdf_and_multiple_libraries() {
    let temp = tempfile::tempdir().unwrap();
    let steam_root = temp.path().join("Steam");
    let second_library = temp.path().join("Library Two");
    write_steam_metadata(&steam_root, &second_library);

    let mut fs_provider = FixtureFileSystem::default();
    let bg1 = fixture_game(
        "steam-bgee-sod",
        &steam_root
            .join("steamapps/common")
            .join("Baldur's Gate Enhanced Edition"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let bg2 = fixture_game(
        "steam-bg2ee",
        &second_library
            .join("steamapps/common")
            .join("Baldur's Gate II Enhanced Edition"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let mut registry = FixtureRegistry::default();
    registry.value(
        RegistryHive::CurrentUser,
        STEAM_KEY,
        "SteamPath",
        steam_root.to_string_lossy(),
    );

    let candidates = discover_games(&profiles(), &registry, &fs_provider).unwrap();
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].role, GameRole::BgeeSod);
    assert_eq!(candidates[0].root, fs::canonicalize(bg1).unwrap());
    assert_eq!(candidates[1].role, GameRole::Bg2ee);
    assert_eq!(candidates[1].root, fs::canonicalize(bg2).unwrap());
    assert!(candidates
        .iter()
        .all(|candidate| candidate.storefront == Storefront::Steam));
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.eligibility == Eligibility::Eligible),
        "{candidates:#?}"
    );
    assert!(candidates
        .iter()
        .all(|candidate| has_finding(candidate, FindingKind::Fresh)));
}

#[test]
fn steam_app_id_keeps_the_exact_profile_identity_for_same_version_profiles() {
    let temp = tempfile::tempdir().unwrap();
    let profile_dir = temp.path().join("profiles");
    fs::create_dir_all(&profile_dir).unwrap();
    let wrong = fs::read_to_string(Path::new(FIXTURES).join("profiles/steam-bg2ee.toml"))
        .unwrap()
        .replace("steam-bg2ee-2.7.3-en-us", "wrong-same-version-profile")
        .replace("steam_app_id = 257350", "steam_app_id = 111");
    let right = wrong
        .replace("wrong-same-version-profile", "right-same-version-profile")
        .replace("steam_app_id = 111", "steam_app_id = 222")
        .replace(
            "36726d25e91f3b92c26d14b474272ba9a6e5441f996359e15f5ba114385b8fce",
            "2377336b961e3f2898e7dd872401ddb39d18bf996847741fc5f1f421483e821b",
        )
        .replace(
            "fb825b72765a90c2c42953134746642c78648f55fa66c983fbd803e4247d1e2c",
            "a785f0f60393cb490d53854b714e0df92cd9e0941b0cf2b4977b06efeb9afd58",
        )
        .replace(
            "1545d6e8489bd62dd5415f4eed88b810e8db1bf5f88e40e905a9646736924a7a",
            "4c7bb6d4eb9d9d2fa31c824890e3706a12149827c31c473060bd8ed6de85b2ce",
        )
        .replace(
            "843d6c1d158b59096a2bec14275911d7e2eb4acc956406dd7dea1ddee0f63f9e",
            "f3849f851f308b494022d84a51a5b61efbaed667b3928a907a0cf7d60dac019d",
        );
    fs::write(profile_dir.join("00-wrong.toml"), wrong).unwrap();
    fs::write(profile_dir.join("01-right.toml"), right).unwrap();
    let profiles = GameProfiles::load(&profile_dir).unwrap();

    let steam_root = temp.path().join("Steam");
    let steamapps = steam_root.join("steamapps");
    fs::create_dir_all(&steamapps).unwrap();
    fs::write(
        steamapps.join("libraryfolders.vdf"),
        format!(
            "\"libraryfolders\" {{ \"0\" {{ \"path\" \"{}\" }} }}",
            steam_root.to_string_lossy().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    fs::write(
        steamapps.join("appmanifest_222.acf"),
        "\"AppState\" { \"appid\" \"222\" \"installdir\" \"Right Game\" }",
    )
    .unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    fixture_game(
        "gog-bg2ee",
        &steam_root.join("steamapps/common/Right Game"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let mut registry = FixtureRegistry::default();
    registry.value(
        RegistryHive::CurrentUser,
        STEAM_KEY,
        "SteamPath",
        steam_root.to_string_lossy(),
    );

    let candidates = discover_games(&profiles, &registry, &fs_provider).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0].eligibility,
        Eligibility::Eligible,
        "{candidates:#?}"
    );
}

#[test]
fn steam_app_id_selects_the_matching_profile_when_multiple_builds_are_supported() {
    let temp = tempfile::tempdir().unwrap();
    let profile_dir = temp.path().join("profiles");
    fs::create_dir_all(&profile_dir).unwrap();
    let current =
        fs::read_to_string(Path::new(FIXTURES).join("profiles/steam-bg2ee.toml")).unwrap();
    let future = current
        .replace("steam-bg2ee-2.7.3-en-us", "steam-bg2ee-2.8.0-en-us")
        .replace(
            "product_version = \"2.7.3.0\"",
            "product_version = \"2.8.0.0\"",
        );
    fs::write(profile_dir.join("00-future.toml"), future).unwrap();
    fs::write(profile_dir.join("01-current.toml"), current).unwrap();
    let profiles = GameProfiles::load(&profile_dir).unwrap();

    let steam_root = temp.path().join("Steam");
    let steamapps = steam_root.join("steamapps");
    fs::create_dir_all(&steamapps).unwrap();
    fs::write(
        steamapps.join("libraryfolders.vdf"),
        format!(
            "\"libraryfolders\" {{ \"0\" {{ \"path\" \"{}\" }} }}",
            steam_root.to_string_lossy().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    fs::write(
        steamapps.join("appmanifest_257350.acf"),
        "\"AppState\" { \"appid\" \"257350\" \"installdir\" \"BG2EE\" }",
    )
    .unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    fixture_game(
        "steam-bg2ee",
        &steam_root.join("steamapps/common/BG2EE"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let mut registry = FixtureRegistry::default();
    registry.value(
        RegistryHive::CurrentUser,
        STEAM_KEY,
        "SteamPath",
        steam_root.to_string_lossy(),
    );

    let candidates = discover_games(&profiles, &registry, &fs_provider).unwrap();
    assert_eq!(candidates.len(), 1, "{candidates:#?}");
    assert_eq!(candidates[0].build.as_deref(), Some("2.7.3.0"));
    assert_eq!(candidates[0].eligibility, Eligibility::Eligible);
    assert!(has_finding(&candidates[0], FindingKind::Fresh));
}

#[test]
fn discovers_gog_registry_records_but_keeps_them_experimental() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let bg1 = fixture_game(
        "gog-bgee-sod",
        &temp.path().join("GOG BGEE"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let bg2 = fixture_game(
        "gog-bg2ee",
        &temp.path().join("GOG BG2EE"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let mut registry = FixtureRegistry::default();
    registry.set_subkeys(RegistryHive::LocalMachine, GOG_KEY, &["1200", "1300"]);
    registry.value(
        RegistryHive::LocalMachine,
        &format!(r"{GOG_KEY}\1200"),
        "gameID",
        "gog-bgee-sod",
    );
    registry.value(
        RegistryHive::LocalMachine,
        &format!(r"{GOG_KEY}\1200"),
        "path",
        bg1.to_string_lossy(),
    );
    registry.value(
        RegistryHive::LocalMachine,
        &format!(r"{GOG_KEY}\1300"),
        "gameID",
        "gog-bg2ee",
    );
    registry.value(
        RegistryHive::LocalMachine,
        &format!(r"{GOG_KEY}\1300"),
        "path",
        bg2.to_string_lossy(),
    );

    let candidates = discover_games(&profiles(), &registry, &fs_provider).unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.eligibility == Eligibility::Experimental),
        "{candidates:#?}"
    );
    assert!(candidates
        .iter()
        .all(|candidate| has_finding(candidate, FindingKind::UnverifiedStorefront)));
    assert!(candidates
        .iter()
        .all(|candidate| !has_finding(candidate, FindingKind::Fresh)));
}

#[test]
fn modified_gog_candidate_installs_with_modified_and_storefront_warnings() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "gog-bg2ee",
        &temp.path().join("modified-gog"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::write(root.join("chitin.key"), b"modified").unwrap();

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Gog,
        &root,
    )
    .unwrap();
    // Formerly a stop; now the installation continues with both warnings.
    assert_eq!(candidate.eligibility, Eligibility::Experimental);
    assert!(candidate.fingerprint.is_some());
    assert!(has_finding(&candidate, FindingKind::Modified));
    assert!(has_finding(&candidate, FindingKind::UnverifiedStorefront));
    assert!(!has_finding(&candidate, FindingKind::Fresh));
    assert!(candidate
        .findings
        .iter()
        .all(|finding| !finding.kind.blocks_installation()));
}

#[test]
fn browse_inspection_uses_the_explicit_storefront_profile() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("browsed"),
        &mut fs_provider,
        "2.7.3.0",
    );

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(candidate.root, fs::canonicalize(root).unwrap());
    assert_eq!(
        candidate.eligibility,
        Eligibility::Eligible,
        "{candidate:#?}"
    );
    assert_eq!(candidate.build.as_deref(), Some("2.7.3.0"));
    assert!(candidate.fingerprint.is_some());
}

#[test]
fn supported_build_with_modified_core_inventory_installs_with_a_warning() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("modified"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::write(root.join("chitin.key"), b"locally modified core").unwrap();

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(candidate.eligibility, Eligibility::Experimental);
    assert!(candidate.fingerprint.is_some());
    let modified = candidate
        .findings
        .iter()
        .find(|finding| finding.kind == FindingKind::Modified)
        .expect("a modified warning");
    assert!(modified.message.contains("continues"), "{modified:?}");
    assert!(!has_finding(&candidate, FindingKind::Fresh));
}

#[test]
fn missing_sod_is_reported_separately() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bgee-sod",
        &temp.path().join("missing-sod"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::remove_file(root.join("dlc/sod-dlc.zip")).unwrap();

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::BgeeSod,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    // Kept: the collection is built on Siege of Dragonspear.
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&candidate, FindingKind::MissingSod));
    assert!(!has_finding(&candidate, FindingKind::SodMerged));
}

#[test]
fn already_merged_sod_without_its_archive_installs_with_a_warning() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bgee-sod",
        &temp.path().join("merged-sod"),
        &mut fs_provider,
        "2.6.6.0",
    );
    fs::remove_file(root.join("dlc/sod-dlc.zip")).unwrap();
    fs::create_dir_all(root.join("movies")).unwrap();
    fs::write(root.join("movies/sodcin05.wbm"), b"merged SoD movie").unwrap();

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::BgeeSod,
        Storefront::Beamdog,
        &root,
    )
    .unwrap();
    assert_eq!(
        candidate.eligibility,
        Eligibility::Experimental,
        "{candidate:#?}"
    );
    assert_eq!(candidate.storefront, Storefront::Beamdog);
    assert!(has_finding(&candidate, FindingKind::SodMerged));
    assert!(has_finding(&candidate, FindingKind::UnverifiedStorefront));
    assert!(has_finding(&candidate, FindingKind::UnsupportedVersion));
    assert!(!has_finding(&candidate, FindingKind::MissingSod));
    assert!(candidate.fingerprint.is_some());
}

#[test]
fn leftovers_of_earlier_mod_installs_are_left_out_and_the_clean_game_stays_tested() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bgee-sod",
        &temp.path().join("stale-dlcmerger"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::create_dir_all(root.join("dlcmerger")).unwrap();
    fs::write(root.join("setup-dlcmerger.exe"), b"stale tool").unwrap();
    fs::write(
        root.join("WeiDU.log"),
        b"// Log of Currently Installed WeiDU Mods\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("stratagems")).unwrap();
    fs::write(root.join("stratagems/readme.txt"), b"mod folder").unwrap();

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::BgeeSod,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    // Formerly "modified" and a stop: the leftovers are left out of the copy, which is then
    // the tested clean game.
    assert_eq!(
        candidate.eligibility,
        Eligibility::Eligible,
        "{candidate:#?}"
    );
    assert!(has_finding(&candidate, FindingKind::Fresh));
    let left_out = candidate
        .findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::LeftOut)
        .collect::<Vec<_>>();
    assert!(left_out
        .iter()
        .any(|finding| finding.message.contains("DLC Merger 1.7")
            && finding.message.contains("leaves them out")));
    let generic = left_out
        .iter()
        .find(|finding| finding.message.contains("Leftovers"))
        .expect("one finding for the other leftovers");
    assert!(generic.message.contains("WeiDU.log"), "{generic:?}");
    assert!(generic.message.contains("stratagems"), "{generic:?}");
    assert!(!has_finding(&candidate, FindingKind::Modified));

    // The leftovers still belong to the source's identity: a change to one is noticed.
    let before = candidate.fingerprint.clone();
    fs::write(root.join("WeiDU.log"), b"changed").unwrap();
    let after = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::BgeeSod,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_ne!(before, after.fingerprint);
}

#[test]
fn clean_inventory_with_an_unknown_core_fingerprint_installs_with_a_warning() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("unknown-fingerprint"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::write(root.join("data/base.bif"), b"unrecognized store payload").unwrap();

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(candidate.eligibility, Eligibility::Experimental);
    assert!(has_finding(&candidate, FindingKind::UnknownFingerprint));
    assert!(!has_finding(&candidate, FindingKind::Fresh));
}

#[test]
fn another_game_version_installs_with_a_version_warning() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("older"),
        &mut fs_provider,
        "2.6.6.0",
    );

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(candidate.eligibility, Eligibility::Experimental);
    assert_eq!(candidate.build.as_deref(), Some("2.6.6.0"));
    let version = candidate
        .findings
        .iter()
        .find(|finding| finding.kind == FindingKind::UnsupportedVersion)
        .expect("a version warning");
    assert!(version.message.contains("2.6.6.0") && version.message.contains("2.7.3.0"));
    // Its clean files are not known, so a file difference is not a finding of its own.
    assert!(!has_finding(&candidate, FindingKind::UnknownFingerprint));
    assert!(candidate.fingerprint.is_some());
}

#[test]
fn a_game_set_to_another_language_installs_with_a_language_warning() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("german"),
        &mut fs_provider,
        "2.7.3.0",
    );

    let candidate = inspect_game_path_with_language(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
        Some("german"),
    )
    .unwrap();
    assert_eq!(candidate.eligibility, Eligibility::Experimental);
    let language = candidate
        .findings
        .iter()
        .find(|finding| finding.kind == FindingKind::UnsupportedLocale)
        .expect("a language warning");
    assert!(language.message.contains("German"), "{language:?}");

    for english in ["english", "en-US", "en_US"] {
        let candidate = inspect_game_path_with_language(
            &profiles(),
            &fs_provider,
            GameRole::Bg2ee,
            Storefront::Steam,
            &root,
            Some(english),
        )
        .unwrap();
        assert_eq!(candidate.eligibility, Eligibility::Eligible, "{english}");
    }
}

#[test]
fn inspection_accumulates_unsupported_version_and_missing_sod() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bgee-sod",
        &temp.path().join("two-findings"),
        &mut fs_provider,
        "2.6.6.0",
    );
    fs::remove_file(root.join("dlc/sod-dlc.zip")).unwrap();

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::BgeeSod,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&candidate, FindingKind::UnsupportedVersion));
    assert!(has_finding(&candidate, FindingKind::MissingSod));
}

#[test]
fn unexpected_override_content_installs_with_a_warning() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("override-content"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::write(root.join("override/leftover.spl"), b"mod residue").unwrap();

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(candidate.eligibility, Eligibility::Experimental);
    assert!(has_finding(&candidate, FindingKind::Modified));
    assert!(!has_finding(&candidate, FindingKind::Fresh));
}

#[test]
fn optional_missing_root_tlk_and_override_match_a_real_27_clean_variant() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = real_27_optional_surface_profiles(&temp.path().join("profiles"));
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("real-27-layout"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::remove_file(root.join("dialog.tlk")).unwrap();
    fs::remove_dir(root.join("override")).unwrap();

    let candidate = inspect_game_path(
        &profiles,
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();

    assert_eq!(
        candidate.eligibility,
        Eligibility::Eligible,
        "{candidate:#?}"
    );
    assert!(has_finding(&candidate, FindingKind::Fresh));
}

#[test]
fn missing_english_texts_stay_a_stop_with_a_plain_reason() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = real_27_optional_surface_profiles(&temp.path().join("profiles"));
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("missing-language-tlk"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::remove_file(root.join("dialog.tlk")).unwrap();
    fs::remove_file(root.join("lang/en_US/dialog.tlk")).unwrap();
    fs::remove_dir(root.join("override")).unwrap();

    let candidate = inspect_game_path(
        &profiles,
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();

    // Kept: every WeiDU run writes the English texts, so they must exist.
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    let missing = candidate
        .findings
        .iter()
        .find(|finding| finding.kind == FindingKind::MissingFiles)
        .expect("a missing-files finding");
    assert!(missing.message.contains("English"), "{missing:?}");
    assert!(!has_finding(&candidate, FindingKind::Fresh));
}

#[test]
fn linked_optional_override_is_not_treated_as_an_absent_clean_surface() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = real_27_optional_surface_profiles(&temp.path().join("profiles"));
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("linked-optional-override"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::remove_file(root.join("dialog.tlk")).unwrap();
    fs_provider.override_kind(root.join("override"), FileKind::Symlink);

    let candidate = inspect_game_path(
        &profiles,
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();

    // Kept: the copy refuses links, so a linked folder cannot be copied.
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&candidate, FindingKind::Unreadable));
    assert!(!has_finding(&candidate, FindingKind::Fresh));
}

#[test]
fn linked_override_surface_is_never_treated_as_an_empty_clean_tree() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("linked-override"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs_provider.override_kind(root.join("override"), FileKind::Symlink);

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(
        candidate.eligibility,
        Eligibility::Ineligible,
        "{candidate:#?}"
    );
    assert!(has_finding(&candidate, FindingKind::Unreadable));
    assert!(candidate.fingerprint.is_none());
}

#[test]
fn linked_ancestor_cannot_supply_an_otherwise_clean_fingerprinted_file() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("linked-data-ancestor"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs_provider.override_kind(root.join("data"), FileKind::Symlink);

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(
        candidate.eligibility,
        Eligibility::Ineligible,
        "{candidate:#?}"
    );
    assert!(has_finding(&candidate, FindingKind::MissingFiles));
    assert!(has_finding(&candidate, FindingKind::Unreadable));
    assert!(!has_finding(&candidate, FindingKind::Fresh));
}

#[test]
fn colliding_inventory_identity_is_unreadable_instead_of_overwritten() {
    let temp = tempfile::tempdir().unwrap();
    let profile_dir = temp.path().join("profiles");
    fs::create_dir_all(&profile_dir).unwrap();
    let digest = hex::encode(Sha256::digest(b"expected"));
    let profile = format!(
        "{}\n[[allowed_clean_variants.inventory]]\npath = \"override/foo.spl\"\nkind = \"file\"\nsha256 = \"{digest}\"\n",
        fs::read_to_string(Path::new(FIXTURES).join("profiles/steam-bg2ee.toml")).unwrap()
    );
    fs::write(profile_dir.join("profile.toml"), profile).unwrap();
    let profiles = GameProfiles::load(&profile_dir).unwrap();

    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("colliding-inventory"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let file = root.join("override/foo.spl");
    fs::write(&file, b"expected").unwrap();
    let canonical_file = fs::canonicalize(&file).unwrap();
    fs_provider.override_directory_entries(
        root.join("override"),
        vec![
            DirectoryEntry {
                path: canonical_file.clone(),
                name: "foo.spl".into(),
                kind: FileKind::File,
            },
            DirectoryEntry {
                path: canonical_file,
                name: "FOO.SPL".into(),
                kind: FileKind::File,
            },
        ],
    );

    let candidate = inspect_game_path(
        &profiles,
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(
        candidate.eligibility,
        Eligibility::Ineligible,
        "{candidate:#?}"
    );
    assert!(has_finding(&candidate, FindingKind::Unreadable));
    assert!(!has_finding(&candidate, FindingKind::Fresh));
}

/// A Steam root with the BG:EE manifest of the fixtures and a game folder for it.
fn steam_with_bgee(temp: &Path, fs_provider: &mut FixtureFileSystem) -> (PathBuf, PathBuf) {
    let steam_root = temp.join("Steam");
    fs::create_dir_all(steam_root.join("steamapps")).unwrap();
    fs::copy(
        Path::new(FIXTURES).join("steam/appmanifest_228280.acf"),
        steam_root.join("steamapps/appmanifest_228280.acf"),
    )
    .unwrap();
    let manifest = fs::read_to_string(steam_root.join("steamapps/appmanifest_228280.acf")).unwrap();
    let install_dir = manifest
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("\"installdir\"")
                .map(|rest| rest.trim().trim_matches('"').to_owned())
        })
        .unwrap();
    let game = fixture_game(
        "steam-bgee-sod",
        &steam_root.join("steamapps/common").join(install_dir),
        fs_provider,
        "2.7.3.0",
    );
    (steam_root, game)
}

fn steam_registry(steam_root: &Path) -> FixtureRegistry {
    let mut registry = FixtureRegistry::default();
    registry.value(
        RegistryHive::CurrentUser,
        STEAM_KEY,
        "SteamPath",
        steam_root.to_string_lossy(),
    );
    registry
}

#[test]
fn an_unsafe_steam_manifest_is_skipped_without_hiding_the_other_game() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let (steam_root, bgee) = steam_with_bgee(temp.path(), &mut fs_provider);
    fs::write(
        steam_root.join("steamapps/appmanifest_257350.acf"),
        "\"AppState\" { \"appid\" \"257350\" \"installdir\" \"../escape\" }",
    )
    .unwrap();

    // Formerly the whole search failed; now only the unsafe record is skipped.
    let candidates =
        discover_games(&profiles(), &steam_registry(&steam_root), &fs_provider).unwrap();
    assert_eq!(candidates.len(), 1, "{candidates:#?}");
    assert_eq!(candidates[0].role, GameRole::BgeeSod);
    assert_eq!(candidates[0].root, fs::canonicalize(bgee).unwrap());
    assert!(candidates
        .iter()
        .all(|candidate| !candidate.root.to_string_lossy().contains("escape")));
}

#[test]
fn a_malformed_steam_manifest_is_skipped_without_hiding_the_other_game() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let (steam_root, _) = steam_with_bgee(temp.path(), &mut fs_provider);
    fs::write(
        steam_root.join("steamapps/appmanifest_257350.acf"),
        "\"AppState\" { \"appid\" ",
    )
    .unwrap();

    let candidates =
        discover_games(&profiles(), &steam_registry(&steam_root), &fs_provider).unwrap();
    assert_eq!(candidates.len(), 1, "{candidates:#?}");
    assert_eq!(candidates[0].eligibility, Eligibility::Eligible);
}

#[test]
fn a_relative_steam_library_is_skipped_and_the_steam_folder_still_searched() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let (steam_root, _) = steam_with_bgee(temp.path(), &mut fs_provider);
    fs::write(
        steam_root.join("steamapps/libraryfolders.vdf"),
        "\"libraryfolders\" { \"1\" { \"path\" \"relative-library\" } }",
    )
    .unwrap();

    let candidates =
        discover_games(&profiles(), &steam_registry(&steam_root), &fs_provider).unwrap();
    assert_eq!(candidates.len(), 1, "{candidates:#?}");
    assert_eq!(candidates[0].storefront, Storefront::Steam);
}

#[test]
fn steam_language_setting_reaches_the_game_check() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let (steam_root, _) = steam_with_bgee(temp.path(), &mut fs_provider);
    let manifest = steam_root.join("steamapps/appmanifest_228280.acf");
    let text = fs::read_to_string(&manifest).unwrap();
    let closing = text.rfind('}').unwrap();
    fs::write(
        &manifest,
        format!(
            "{}\"UserConfig\" {{ \"language\" \"french\" }}\n}}\n",
            &text[..closing]
        ),
    )
    .unwrap();

    let candidates =
        discover_games(&profiles(), &steam_registry(&steam_root), &fs_provider).unwrap();
    assert_eq!(candidates.len(), 1, "{candidates:#?}");
    assert_eq!(candidates[0].eligibility, Eligibility::Experimental);
    assert!(candidates[0].findings.iter().any(|finding| {
        finding.kind == FindingKind::UnsupportedLocale && finding.message.contains("French")
    }));
}

#[test]
fn a_steam_game_folder_that_cannot_be_read_is_listed_with_its_reason() {
    let temp = tempfile::tempdir().unwrap();
    let steam_root = temp.path().join("Steam");
    fs::create_dir_all(steam_root.join("steamapps")).unwrap();
    fs::write(
        steam_root.join("steamapps/libraryfolders.vdf"),
        format!(
            "\"libraryfolders\" {{ \"0\" {{ \"path\" \"{}\" }} }}",
            steam_root.to_string_lossy().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    fs::write(
        steam_root.join("steamapps/appmanifest_257350.acf"),
        "\"AppState\" { \"appid\" \"257350\" \"installdir\" \"BG2EE\" }",
    )
    .unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &steam_root.join("steamapps/common/BG2EE"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs_provider.override_kind_error(root.clone(), io::ErrorKind::PermissionDenied);

    // Formerly the whole search failed; now the folder is listed, unusable, with the reason.
    let candidates =
        discover_games(&profiles(), &steam_registry(&steam_root), &fs_provider).unwrap();
    assert_eq!(candidates.len(), 1, "{candidates:#?}");
    let candidate = &candidates[0];
    assert_eq!(candidate.root, root);
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    let finding = candidate
        .findings
        .iter()
        .find(|finding| finding.kind == FindingKind::Unreadable)
        .expect("an unreadable finding");
    assert_eq!(
        finding.message,
        "CEBG cannot read this game folder. Check that it can be read, then try again."
    );
}

/// The fixture profiles without their GOG ones, like the shipped recipes, which have only
/// Steam profiles.
fn steam_only_profiles(directory: &Path) -> GameProfiles {
    fs::create_dir_all(directory).unwrap();
    for name in ["steam-bgee-sod.toml", "steam-bg2ee.toml"] {
        fs::copy(
            Path::new(FIXTURES).join("profiles").join(name),
            directory.join(name),
        )
        .unwrap();
    }
    GameProfiles::load(directory).unwrap()
}

#[test]
fn gog_records_with_the_real_product_ids_are_found_without_a_gog_profile() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = steam_only_profiles(&temp.path().join("profiles"));
    let mut fs_provider = FixtureFileSystem::default();
    let bg1 = fixture_game(
        "steam-bgee-sod",
        &temp.path().join("GOG Games/Baldur's Gate Enhanced Edition"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let bg2 = fixture_game(
        "steam-bg2ee",
        &temp
            .path()
            .join("GOG Games/Baldur's Gate II - Enhanced Edition"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let wow = r"SOFTWARE\WOW6432Node\GOG.com\Games";
    let mut registry = FixtureRegistry::default();
    registry.set_subkeys(
        RegistryHive::LocalMachine,
        wow,
        &["1207666353", "1207666373", "1459335293", "999"],
    );
    for (id, root, language) in [
        ("1207666353", &bg1, "english"),
        ("1207666373", &bg2, "German"),
    ] {
        let key = format!(r"{wow}\{id}");
        registry.value(RegistryHive::LocalMachine, &key, "gameID", id);
        registry.value(
            RegistryHive::LocalMachine,
            &key,
            "path",
            root.to_string_lossy(),
        );
        registry.value(RegistryHive::LocalMachine, &key, "language", language);
    }
    // The SoD DLC record has no path of its own, and an unrelated game is ignored.
    registry.value(
        RegistryHive::LocalMachine,
        &format!(r"{wow}\999"),
        "gameID",
        "999",
    );

    let candidates = discover_games(&profiles, &registry, &fs_provider).unwrap();
    assert_eq!(candidates.len(), 2, "{candidates:#?}");
    for candidate in &candidates {
        assert_eq!(candidate.storefront, Storefront::Gog);
        // Formerly no candidate at all; now usable, with the "less tested" warning.
        assert_eq!(
            candidate.eligibility,
            Eligibility::Experimental,
            "{candidate:#?}"
        );
        assert!(has_finding(candidate, FindingKind::UnverifiedStorefront));
        assert!(candidate.fingerprint.is_some());
    }
    let bg2 = candidates
        .iter()
        .find(|candidate| candidate.role == GameRole::Bg2ee)
        .unwrap();
    assert!(has_finding(bg2, FindingKind::UnsupportedLocale));
}

#[test]
fn gog_and_beamdog_games_in_their_default_folders_are_found() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = steam_only_profiles(&temp.path().join("profiles"));
    let mut fs_provider = FixtureFileSystem::default();
    let gog_library = temp.path().join("GOG Games");
    let gog_bg2 = fixture_game(
        "steam-bg2ee",
        &gog_library.join("Baldur's Gate II - Enhanced Edition"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::write(
        gog_bg2.join("goggame-1207666373.info"),
        br#"{ "gameId": "1207666373", "language": "French" }"#,
    )
    .unwrap();
    // A folder of the right name without GOG's info file is not taken for a GOG game.
    fixture_game(
        "steam-bgee-sod",
        &gog_library.join("Baldur's Gate Enhanced Edition"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let beamdog_games = temp.path().join("Beamdog/Games");
    let beamdog_bg1 = fixture_game(
        "steam-bgee-sod",
        &beamdog_games.join("00806"),
        &mut fs_provider,
        "2.6.6.0",
    );
    fs::remove_file(beamdog_bg1.join("dlc/sod-dlc.zip")).unwrap();
    fs::create_dir_all(beamdog_bg1.join("movies")).unwrap();
    fs::write(beamdog_bg1.join("movies/sodcin05.wbm"), b"merged").unwrap();

    let candidates = discover_games_with(
        &profiles,
        &FixtureRegistry::default(),
        &fs_provider,
        &StoreFolders {
            beamdog_games: vec![beamdog_games, temp.path().join("missing")],
            gog_libraries: vec![gog_library],
        },
    )
    .unwrap();
    assert_eq!(candidates.len(), 2, "{candidates:#?}");
    let gog = candidates
        .iter()
        .find(|candidate| candidate.storefront == Storefront::Gog)
        .unwrap();
    assert_eq!(gog.role, GameRole::Bg2ee);
    assert_eq!(gog.eligibility, Eligibility::Experimental);
    assert!(has_finding(gog, FindingKind::UnsupportedLocale));
    let beamdog = candidates
        .iter()
        .find(|candidate| candidate.storefront == Storefront::Beamdog)
        .unwrap();
    assert_eq!(beamdog.role, GameRole::BgeeSod);
    assert_eq!(
        beamdog.eligibility,
        Eligibility::Experimental,
        "{beamdog:#?}"
    );
    assert!(has_finding(beamdog, FindingKind::SodMerged));
    assert!(has_finding(beamdog, FindingKind::UnverifiedStorefront));
}

#[test]
fn source_read_access_is_tested_instead_of_assumed_from_windowsapps() {
    for parent in ["WindowsApps", "ordinary-games"] {
        let temp = tempfile::tempdir().unwrap();
        let mut provider = FixtureFileSystem::default();
        let root = fixture_game(
            "steam-bg2ee",
            &temp.path().join(parent).join("BG2"),
            &mut provider,
            "2.7.3.0",
        );
        let store = detect_storefront(&provider, &root);
        let inspect = |provider: &FixtureFileSystem| {
            inspect_game_path(&profiles(), provider, GameRole::Bg2ee, store, &root).unwrap()
        };
        let readable = inspect(&provider);
        assert!(readable.eligibility.is_usable());
        if parent == "WindowsApps" {
            assert_eq!(readable.storefront, Storefront::Microsoft);
            assert_eq!(readable.eligibility, Eligibility::Experimental);
        } else {
            assert_eq!(readable.eligibility, Eligibility::Eligible);
        }
        assert!(readable.fingerprint.is_some());
        assert!(!has_finding(&readable, FindingKind::ProtectedFolder));
        provider.directory_errors.insert(
            fs::canonicalize(&root).unwrap(),
            io::ErrorKind::PermissionDenied,
        );
        let denied = inspect(&provider);
        assert_eq!(denied.eligibility, Eligibility::Ineligible);
        let (kind, message) = if parent == "WindowsApps" {
            (FindingKind::ProtectedFolder, "This copy comes from the Microsoft Store or the Xbox app. Windows protects its folder, so CEBG cannot copy the game from it. Use a Steam or GOG copy of the game instead.")
        } else {
            (
                FindingKind::Unreadable,
                "CEBG cannot read this game folder. Check that it can be read, then try again.",
            )
        };
        assert_eq!(denied.findings[0].kind, kind);
        assert_eq!(denied.findings[0].message, message);
        provider.directory_errors.clear();
        provider.read_errors.insert(
            fs::canonicalize(root.join("chitin.key")).unwrap(),
            io::ErrorKind::PermissionDenied,
        );
        let denied_file = inspect(&provider);
        assert_eq!(denied_file.eligibility, Eligibility::Ineligible);
        let finding = denied_file
            .findings
            .iter()
            .find(|finding| finding.kind == kind)
            .unwrap();
        if parent == "WindowsApps" {
            assert_eq!(finding.message, message);
        } else {
            assert!(!has_finding(&denied_file, FindingKind::ProtectedFolder));
        }
    }
}

#[test]
fn a_browsed_folder_names_its_store_by_what_it_holds_and_where_it_is() {
    let temp = tempfile::tempdir().unwrap();
    let fs_provider = FixtureFileSystem::default();
    let gog = temp.path().join("Games/BG2");
    fs::create_dir_all(&gog).unwrap();
    fs::write(gog.join("goggame-1207666373.info"), b"{}").unwrap();
    let steam = temp.path().join("SteamLibrary/steamapps/common/BG2");
    let beamdog = temp.path().join("BeamDog/Games/00783");
    let xbox = temp.path().join("XboxGames/Baldur's Gate/Content");
    let other = temp.path().join("Copies/BG2");
    for folder in [&steam, &beamdog, &xbox, &other] {
        fs::create_dir_all(folder).unwrap();
    }
    assert_eq!(detect_storefront(&fs_provider, &gog), Storefront::Gog);
    assert_eq!(detect_storefront(&fs_provider, &steam), Storefront::Steam);
    assert_eq!(
        detect_storefront(&fs_provider, &beamdog),
        Storefront::Beamdog
    );
    assert_eq!(
        detect_storefront(&fs_provider, &xbox),
        Storefront::Microsoft
    );
    assert_eq!(detect_storefront(&fs_provider, &other), Storefront::Other);
}

#[test]
fn a_copied_steam_game_of_unknown_store_is_the_tested_setup_when_its_files_match() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("Copies/BG2"),
        &mut fs_provider,
        "2.7.3.0",
    );
    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Other,
        &root,
    )
    .unwrap();
    assert_eq!(candidate.eligibility, Eligibility::Eligible);
    assert_eq!(candidate.storefront, Storefront::Steam);

    fs::write(root.join("chitin.key"), b"changed").unwrap();
    let changed = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Other,
        &root,
    )
    .unwrap();
    assert_eq!(changed.eligibility, Eligibility::Experimental);
    assert_eq!(changed.storefront, Storefront::Other);
    assert!(has_finding(&changed, FindingKind::UnverifiedStorefront));
}

#[cfg(windows)]
#[test]
#[ignore = "read-only local probe; configure CHRIZ_GAME_PROFILE_DIR, CHRIZ_PROBE_BGEE_SOD, and CHRIZ_PROBE_BG2EE explicitly"]
fn configured_local_probe_reproduces_known_source_evidence_without_writes() {
    let profile_dir = std::env::var_os("CHRIZ_GAME_PROFILE_DIR")
        .map(PathBuf::from)
        .expect("set CHRIZ_GAME_PROFILE_DIR to independently authored profiles");
    let bg1 = std::env::var_os("CHRIZ_PROBE_BGEE_SOD")
        .map(PathBuf::from)
        .expect("set CHRIZ_PROBE_BGEE_SOD to the configured read-only source");
    let bg2 = std::env::var_os("CHRIZ_PROBE_BG2EE")
        .map(PathBuf::from)
        .expect("set CHRIZ_PROBE_BG2EE to the configured read-only source");
    let profiles = GameProfiles::load(profile_dir).unwrap();
    let fs_provider = SystemFileSystem;

    let bg1 = inspect_game_path(
        &profiles,
        &fs_provider,
        GameRole::BgeeSod,
        Storefront::Steam,
        &bg1,
    )
    .unwrap();
    let bg2 = inspect_game_path(
        &profiles,
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &bg2,
    )
    .unwrap();

    assert_eq!(bg2.build.as_deref(), Some("2.7.3.0"));
    assert_eq!(bg2.eligibility, Eligibility::Eligible);
    assert_eq!(bg1.eligibility, Eligibility::Ineligible);
    assert!(bg1.findings.iter().any(|finding| {
        finding.kind == FindingKind::Modified && finding.message.contains("DLC Merger 1.7")
    }));
}

#[test]
fn gog_bg2ee_2_7_3_with_baldur_ii_exe_is_usable_without_missing_files_or_version_error() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("gog-bg2ee-273"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::rename(root.join("Baldur.exe"), root.join("BaldurII.exe")).unwrap();
    fs::write(root.join("goggame-1207666373.info"), b"{}").unwrap();
    fs_provider.add_version(root.join("BaldurII.exe"), "2.7.3.0");

    let real_profiles =
        GameProfiles::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("../manifest/game-builds"))
            .unwrap();
    let candidate = inspect_game_path(
        &real_profiles,
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Gog,
        &root,
    )
    .unwrap();

    assert_eq!(
        candidate.eligibility,
        Eligibility::Experimental,
        "{candidate:#?}"
    );
    assert_eq!(candidate.build.as_deref(), Some("2.7.3.0"));
    assert!(candidate.fingerprint.is_some());
    assert!(!has_finding(&candidate, FindingKind::MissingFiles));
    assert!(!has_finding(&candidate, FindingKind::UnsupportedVersion));
    assert!(has_finding(&candidate, FindingKind::UnverifiedStorefront));
}

#[test]
fn custom_root_game_executable_is_detected_while_helper_tools_are_ignored() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = fixture_game(
        "steam-bg2ee",
        &temp.path().join("repackaged-bg2ee"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::rename(root.join("Baldur.exe"), root.join("BG2Enhanced.exe")).unwrap();
    fs::write(root.join("unins000.exe"), b"uninstaller").unwrap();
    fs::write(root.join("oalinst.exe"), b"openal").unwrap();
    fs_provider.add_version(root.join("BG2Enhanced.exe"), "2.7.3.0");

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Other,
        &root,
    )
    .unwrap();

    assert_eq!(
        candidate.eligibility,
        Eligibility::Experimental,
        "{candidate:#?}"
    );
    assert_eq!(candidate.build.as_deref(), Some("2.7.3.0"));
    assert!(candidate.fingerprint.is_some());
    assert!(!has_finding(&candidate, FindingKind::MissingFiles));

    // When no game executable exists at all (only helper binaries), MissingFiles still blocks.
    fs::remove_file(root.join("BG2Enhanced.exe")).unwrap();
    let missing = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Other,
        &root,
    )
    .unwrap();
    assert_eq!(missing.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&missing, FindingKind::MissingFiles));
}

#[test]
fn older_iwdee_build_without_patch27_bif_is_usable_when_data_has_regular_bifs() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();
    let root = temp.path().join("beamdog-iwdee-266");
    fs::create_dir_all(root.join("data")).unwrap();
    fs::create_dir_all(root.join("lang/en_US")).unwrap();
    fs::write(root.join("icewind.exe"), b"iwdee 2.6").unwrap();
    fs::write(root.join("chitin.key"), b"key").unwrap();
    fs::write(root.join("engine.lua"), b"engine_name = \"IWD\"\n").unwrap();
    fs::write(root.join("lang/en_US/dialog.tlk"), b"tlk").unwrap();
    fs::write(root.join("data/DEFAULT.BIF"), b"bif").unwrap();
    fs_provider.add_version(root.join("icewind.exe"), "2.6.6.0");

    let iwd_profiles = GameProfiles::load(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../recipes/iwd-current/game-builds"),
    )
    .unwrap();
    let candidate = inspect_game_path(
        &iwd_profiles,
        &fs_provider,
        GameRole::Iwdee,
        Storefront::Beamdog,
        &root,
    )
    .unwrap();

    assert_eq!(
        candidate.eligibility,
        Eligibility::Experimental,
        "{candidate:#?}"
    );
    assert_eq!(candidate.build.as_deref(), Some("2.6.6.0"));
    assert!(candidate.fingerprint.is_some());
    assert!(!has_finding(&candidate, FindingKind::MissingFiles));
    assert!(has_finding(&candidate, FindingKind::UnsupportedVersion));

    // Removing the remaining BIF archive still blocks with MissingFiles.
    fs::remove_file(root.join("data/DEFAULT.BIF")).unwrap();
    let broken = inspect_game_path(
        &iwd_profiles,
        &fs_provider,
        GameRole::Iwdee,
        Storefront::Beamdog,
        &root,
    )
    .unwrap();
    assert_eq!(broken.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&broken, FindingKind::MissingFiles));
}

#[test]
fn provider_surface_is_read_only_by_construction() {
    fn assert_read_only_shape<T: FileSystemProvider>(_provider: &T) {}
    assert_read_only_shape(&SystemFileSystem);
}

#[test]
fn discovers_gog_and_beamdog_iwdee_and_unhyphenated_bg2ee_and_ignores_os_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = FixtureFileSystem::default();

    let gog_lib = temp.path().join("GOG Games");
    let gog_iwd = gog_lib.join("Icewind Dale Enhanced Edition");
    fs::create_dir_all(gog_iwd.join("data")).unwrap();
    fs::create_dir_all(gog_iwd.join("lang/en_US")).unwrap();
    fs::write(gog_iwd.join("icewind.exe"), b"gog iwd").unwrap();
    fs::write(gog_iwd.join("chitin.key"), b"key").unwrap();
    fs::write(gog_iwd.join("engine.lua"), b"engine_name = \"IWD\"\n").unwrap();
    fs::write(gog_iwd.join("lang/en_US/dialog.tlk"), b"tlk").unwrap();
    fs::write(gog_iwd.join("data/DEFAULT.BIF"), b"bif").unwrap();
    fs::write(
        gog_iwd.join("goggame-1207666683.info"),
        br#"{"language":"English"}"#,
    )
    .unwrap();
    fs_provider.add_version(gog_iwd.join("icewind.exe"), "2.6.6.0");

    let gog_bg2 = gog_lib.join("Baldur's Gate II Enhanced Edition");
    fixture_game("steam-bg2ee", &gog_bg2, &mut fs_provider, "2.7.3.0");
    fs::write(
        gog_bg2.join("goggame-1207666373.info"),
        br#"{"language":"English"}"#,
    )
    .unwrap();

    let beamdog_lib = temp.path().join("Beamdog/Games");
    let beamdog_iwd = beamdog_lib.join("00798");
    fs::create_dir_all(beamdog_iwd.join("data")).unwrap();
    fs::create_dir_all(beamdog_iwd.join("lang/en_US")).unwrap();
    fs::write(beamdog_iwd.join("icewind.exe"), b"bd iwd").unwrap();
    fs::write(beamdog_iwd.join("chitin.key"), b"key").unwrap();
    fs::write(beamdog_iwd.join("engine.lua"), b"engine_name = \"IWD\"\n").unwrap();
    fs::write(beamdog_iwd.join("lang/en_US/dialog.tlk"), b"tlk").unwrap();
    fs::write(beamdog_iwd.join("data/DEFAULT.BIF"), b"bif").unwrap();
    fs_provider.add_version(beamdog_iwd.join("icewind.exe"), "2.6.6.0");

    let iwd_profiles = GameProfiles::load(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../recipes/iwd-current/game-builds"),
    )
    .unwrap();
    let folders = StoreFolders {
        beamdog_games: vec![beamdog_lib],
        gog_libraries: vec![gog_lib],
    };
    let iwd_found = discover_games_with(
        &iwd_profiles,
        &FixtureRegistry::default(),
        &fs_provider,
        &folders,
    )
    .unwrap();
    assert!(
        iwd_found.iter().any(|c| c.role == GameRole::Iwdee
            && c.storefront == Storefront::Gog
            && c.eligibility.is_usable()),
        "{iwd_found:#?}"
    );
    assert!(
        iwd_found.iter().any(|c| c.role == GameRole::Iwdee
            && c.storefront == Storefront::Beamdog
            && c.eligibility.is_usable()),
        "{iwd_found:#?}"
    );

    let bg_found = discover_games_with(
        &profiles(),
        &FixtureRegistry::default(),
        &fs_provider,
        &folders,
    )
    .unwrap();
    assert!(
        bg_found.iter().any(|c| c.role == GameRole::Bg2ee
            && c.storefront == Storefront::Gog
            && c.eligibility.is_usable()),
        "{bg_found:#?}"
    );

    // desktop.ini and Thumbs.db in override/ do not dirty clean Steam BG2EE fingerprints.
    let clean_bg2 = fixture_game(
        "steam-bg2ee",
        &temp.path().join("clean-bg2-meta"),
        &mut fs_provider,
        "2.7.3.0",
    );
    fs::write(
        clean_bg2.join("override/desktop.ini"),
        b"[.ShellClassInfo]\n",
    )
    .unwrap();
    fs::write(clean_bg2.join("override/Thumbs.db"), b"db").unwrap();
    let clean_cand = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &clean_bg2,
    )
    .unwrap();
    assert_eq!(
        clean_cand.eligibility,
        Eligibility::Eligible,
        "{clean_cand:#?}"
    );
}
