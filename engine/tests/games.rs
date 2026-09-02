use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use bg_engine::games::{
    discover_games, inspect_game_path, DirectoryEntry, Eligibility, FileKind, FileSystemProvider,
    FindingKind, GameError, GameProfiles, GameRole, RegistryHive, RegistryProvider, Storefront,
    SystemFileSystem,
};
use sha2::{Digest, Sha256};

const FIXTURES: &str = "tests/fixtures/games";
const STEAM_KEY: &str = r"Software\Valve\Steam";
const GOG_KEY: &str = r"SOFTWARE\GOG.com\Games";

#[derive(Default)]
struct FixtureFileSystem {
    versions: BTreeMap<PathBuf, String>,
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
        fs::read(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>> {
        let canonical = fs::canonicalize(path)?;
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
fn modified_gog_candidate_stays_ineligible_and_retains_storefront_warning() {
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
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&candidate, FindingKind::Modified));
    assert!(has_finding(&candidate, FindingKind::UnverifiedStorefront));
    assert!(!has_finding(&candidate, FindingKind::Fresh));
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
fn supported_build_with_modified_core_inventory_is_ineligible() {
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
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&candidate, FindingKind::Modified));
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
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&candidate, FindingKind::MissingSod));
}

#[test]
fn stale_dlc_merger_residue_is_modified_even_after_store_core_update() {
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

    let candidate = inspect_game_path(
        &profiles(),
        &fs_provider,
        GameRole::BgeeSod,
        Storefront::Steam,
        &root,
    )
    .unwrap();
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    assert!(candidate.findings.iter().any(|finding| {
        finding.kind == FindingKind::Modified && finding.message.contains("DLC Merger 1.7")
    }));
}

#[test]
fn clean_inventory_with_an_unknown_core_fingerprint_fails_closed() {
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
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&candidate, FindingKind::UnknownFingerprint));
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
fn unexpected_override_content_prevents_freshness() {
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
    assert_eq!(candidate.eligibility, Eligibility::Ineligible);
    assert!(has_finding(&candidate, FindingKind::Modified));
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
    assert!(has_finding(&candidate, FindingKind::Modified));
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
    assert!(has_finding(&candidate, FindingKind::Modified));
    assert!(!has_finding(&candidate, FindingKind::Fresh));
}

#[test]
fn colliding_inventory_identity_is_modified_instead_of_overwritten() {
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
    assert!(has_finding(&candidate, FindingKind::Modified));
    assert!(!has_finding(&candidate, FindingKind::Fresh));
}

#[test]
fn steam_manifest_traversal_is_rejected_pathfully() {
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
        "\"AppState\" { \"appid\" \"257350\" \"installdir\" \"../escape\" }",
    )
    .unwrap();
    let mut registry = FixtureRegistry::default();
    registry.value(
        RegistryHive::CurrentUser,
        STEAM_KEY,
        "SteamPath",
        steam_root.to_string_lossy(),
    );

    let error = discover_games(&profiles(), &registry, &FixtureFileSystem::default()).unwrap_err();
    assert!(error.to_string().contains("appmanifest_257350.acf"));
    assert!(error.to_string().contains("installdir"));
}

#[test]
fn relative_steam_library_path_is_rejected_pathfully() {
    let temp = tempfile::tempdir().unwrap();
    let steam_root = temp.path().join("Steam");
    fs::create_dir_all(steam_root.join("steamapps")).unwrap();
    fs::write(
        steam_root.join("steamapps/libraryfolders.vdf"),
        "\"libraryfolders\" { \"1\" { \"path\" \"relative-library\" } }",
    )
    .unwrap();
    let mut registry = FixtureRegistry::default();
    registry.value(
        RegistryHive::CurrentUser,
        STEAM_KEY,
        "SteamPath",
        steam_root.to_string_lossy(),
    );

    let error = discover_games(&profiles(), &registry, &FixtureFileSystem::default()).unwrap_err();
    assert!(error.to_string().contains("libraryfolders.vdf"), "{error}");
    assert!(error.to_string().contains("absolute"), "{error}");
}

#[test]
fn steam_candidate_permission_error_is_not_hidden_as_an_absent_game() {
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
    let mut registry = FixtureRegistry::default();
    registry.value(
        RegistryHive::CurrentUser,
        STEAM_KEY,
        "SteamPath",
        steam_root.to_string_lossy(),
    );

    let error = discover_games(&profiles(), &registry, &fs_provider).unwrap_err();
    let GameError::Io { path, .. } = &error else {
        panic!("expected a pathful I/O error, got {error}");
    };
    assert_eq!(path, &root);
    assert!(error.to_string().contains("injected"), "{error}");
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
fn provider_surface_is_read_only_by_construction() {
    fn assert_read_only_shape<T: FileSystemProvider>(_provider: &T) {}
    assert_read_only_shape(&SystemFileSystem);
}
