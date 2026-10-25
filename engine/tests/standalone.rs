//! Standalone single-source collections (Icewind Dale: Enhanced Edition): recipe game
//! selection, standalone validation rules, and the IWD:EE game profile.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use bg_engine::games::{
    discover_games, inspect_game_path, DirectoryEntry, Eligibility, FileKind, FileSystemProvider,
    FindingKind, GameProfiles, GameRole, RegistryHive, RegistryProvider, Storefront,
    SystemFileSystem,
};
use bg_engine::manifest::{Collection, CollectionGame, GameRoot, Phase, RunArg};
use bg_engine::validate::{
    validate, Finding, Severity, RULE_APPLICATION_DATA_DIRECTORY, RULE_EET_ANCHORS,
    RULE_STANDALONE_GAME,
};
use bg_engine::Manifest;

const GAME_FIXTURES: &str = "tests/fixtures/games";
const STEAM_KEY: &str = r"Software\Valve\Steam";
const REAL_IWDEE: &str = r"C:\Games\Icewind Dale Enhanced Edition";

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn eet_manifest() -> Manifest {
    Manifest::load(&fixture("tests/fixtures/manifest")).unwrap()
}

fn iwd_manifest() -> Manifest {
    Manifest::load(&fixture("tests/fixtures/standalone-iwdee")).unwrap()
}

fn errors(findings: &[Finding]) -> Vec<&Finding> {
    findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .collect()
}

#[test]
fn collection_game_defaults_to_eet_and_is_omitted_when_eet() {
    let manifest = eet_manifest();
    assert_eq!(manifest.collection.game, CollectionGame::Eet);
    assert!(!manifest.collection.game.is_standalone());
    assert_eq!(manifest.collection.game.standalone_role(), None);

    let json = serde_json::to_value(&manifest.collection).unwrap();
    assert!(json.get("game").is_none(), "{json:#}");
    let toml_text = toml::to_string(&manifest.collection).unwrap();
    assert!(!toml_text.contains("game ="), "{toml_text}");
    assert_eq!(
        toml::from_str::<Collection>(&toml_text).unwrap(),
        manifest.collection
    );
}

#[test]
fn iwdee_collection_parses_and_round_trips_its_game() {
    let manifest = iwd_manifest();
    let game = manifest.collection.game;
    assert_eq!(game, CollectionGame::Iwdee);
    assert!(game.is_standalone());
    assert_eq!(game.standalone_role(), Some(GameRole::Iwdee));
    assert_eq!(game.display_name(), "Icewind Dale: Enhanced Edition");

    let json = serde_json::to_value(&manifest.collection).unwrap();
    assert_eq!(json["game"], "iwdee");
    assert_eq!(
        serde_json::from_value::<Collection>(json).unwrap(),
        manifest.collection
    );
    let toml_text = toml::to_string(&manifest.collection).unwrap();
    assert!(toml_text.contains("game = \"iwdee\""), "{toml_text}");
    assert_eq!(
        toml::from_str::<Collection>(&toml_text).unwrap(),
        manifest.collection
    );
}

#[test]
fn unknown_collection_games_are_rejected() {
    let text = fs::read_to_string(fixture("tests/fixtures/standalone-iwdee/collection.toml"))
        .unwrap()
        .replace("game = \"iwdee\"", "game = \"pstee\"");
    assert!(toml::from_str::<Collection>(&text).is_err());
}

#[test]
fn standalone_fixture_recipe_validates_without_eet_anchors() {
    let manifest = iwd_manifest();
    let findings = validate(&manifest);
    assert!(errors(&findings).is_empty(), "{findings:#?}");
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule != RULE_EET_ANCHORS),
        "{findings:#?}"
    );
}

#[test]
fn standalone_recipe_rejects_every_phase_but_main() {
    for (phase, mentions_bg1_root) in [
        (Phase::Bg1Preparation, true),
        (Phase::Bg2Preparation, false),
        (Phase::EetInitialization, false),
    ] {
        let mut manifest = iwd_manifest();
        manifest.collection.runs[0].phase = phase;
        let findings = validate(&manifest);
        let standalone = errors(&findings)
            .into_iter()
            .filter(|finding| finding.rule == RULE_STANDALONE_GAME)
            .map(|finding| finding.message.as_str())
            .collect::<Vec<_>>();
        assert!(
            standalone.iter().any(|message| message.contains(&format!(
                "has phase {phase:?}, but a standalone Icewind Dale: Enhanced Edition collection supports only the main phase"
            ))),
            "{phase:?}: {findings:#?}"
        );
        assert_eq!(
            standalone
                .iter()
                .any(|message| message.contains("targets the Baldur's Gate root")),
            mentions_bg1_root,
            "{phase:?}: {findings:#?}"
        );
        assert!(
            findings
                .iter()
                .all(|finding| finding.rule != RULE_EET_ANCHORS),
            "{phase:?}: {findings:#?}"
        );
    }
}

#[test]
fn standalone_recipe_rejects_a_bg1_root_argument_but_accepts_its_own_root() {
    let mut manifest = iwd_manifest();
    manifest.collection.runs[0].args = vec![RunArg::StagedRoot(GameRoot::Bg2)];
    assert!(errors(&validate(&manifest)).is_empty());

    manifest.collection.runs[0].args = vec![RunArg::StagedRoot(GameRoot::Bg1)];
    let findings = validate(&manifest);
    assert!(
        errors(&findings).iter().any(|finding| finding.rule == RULE_STANDALONE_GAME
            && finding.message.contains(
                "passes the Baldur's Gate root as an argument, which a standalone Icewind Dale: Enhanced Edition collection does not stage"
            )),
        "{findings:#?}"
    );
}

#[test]
fn eet_recipe_still_requires_its_anchors_and_ignores_standalone_rules() {
    let eet_findings = validate(&eet_manifest());
    assert!(errors(&eet_findings).is_empty(), "{eet_findings:#?}");

    // The same main-phase-only runs that a standalone recipe accepts are not a valid EET
    // recipe: without an explicit game the EET anchor rules apply again.
    let mut manifest = iwd_manifest();
    manifest.collection.game = CollectionGame::Eet;
    let findings = validate(&manifest);
    assert!(
        errors(&findings)
            .iter()
            .any(|finding| finding.rule == RULE_EET_ANCHORS),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule != RULE_STANDALONE_GAME),
        "{findings:#?}"
    );
}

#[test]
fn standalone_recipe_must_name_its_own_application_data_folder() {
    let manifest = iwd_manifest();
    assert_eq!(
        manifest.collection.application_data_directory.as_deref(),
        Some("Chriz Easy IWD")
    );
    let json = serde_json::to_value(&manifest.collection).unwrap();
    assert_eq!(json["application_data_directory"], "Chriz Easy IWD");
    assert!(serde_json::to_value(&eet_manifest().collection)
        .unwrap()
        .get("application_data_directory")
        .is_none());

    for (value, expected) in [
        (None, "must name its own application_data_directory"),
        (
            Some("Chriz BG Collection"),
            "must not share the Baldur's Gate application data folder",
        ),
        (
            Some("chriz bg collection"),
            "must not share the Baldur's Gate application data folder",
        ),
        (Some("COM\u{b9}"), "is a reserved Windows device name"),
        (Some(r"Chriz\IWD"), "must be one folder name"),
    ] {
        let mut manifest = iwd_manifest();
        manifest.collection.application_data_directory = value.map(str::to_owned);
        let findings = validate(&manifest);
        assert!(
            errors(&findings)
                .iter()
                .any(|finding| finding.rule == RULE_APPLICATION_DATA_DIRECTORY
                    && finding.message.contains(expected)),
            "{value:?}: {findings:#?}"
        );
    }
}

#[test]
fn eet_recipe_never_names_an_application_data_folder() {
    // EET installations always keep their records in the Baldur's Gate folder; naming it,
    // even as the default, would give the same recipe a second payload digest.
    for value in ["Chriz BG Collection", "Chriz Easy IWD"] {
        let mut manifest = eet_manifest();
        manifest.collection.application_data_directory = Some(value.to_owned());
        let findings = validate(&manifest);
        assert!(
            errors(&findings)
                .iter()
                .any(|finding| finding.rule == RULE_APPLICATION_DATA_DIRECTORY
                    && finding.message.contains("Enhanced Edition Trilogy")),
            "{value:?}: {findings:#?}"
        );
    }
}
#[derive(Default)]
struct VersionedFileSystem {
    versions: BTreeMap<PathBuf, String>,
}

impl VersionedFileSystem {
    fn add_version(&mut self, executable: &Path, version: &str) {
        self.versions
            .insert(fs::canonicalize(executable).unwrap(), version.to_owned());
    }
}

impl FileSystemProvider for VersionedFileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        SystemFileSystem.read(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>> {
        SystemFileSystem.read_dir(path)
    }

    fn file_kind(&self, path: &Path) -> io::Result<FileKind> {
        SystemFileSystem.file_kind(path)
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
}

impl RegistryProvider for FixtureRegistry {
    fn read_string(&self, hive: RegistryHive, key: &str, name: &str) -> io::Result<Option<String>> {
        Ok(self
            .values
            .get(&(hive, key.to_ascii_lowercase(), name.to_ascii_lowercase()))
            .cloned())
    }

    fn subkeys(&self, _hive: RegistryHive, _key: &str) -> io::Result<Vec<String>> {
        Ok(Vec::new())
    }
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

fn fixture_iwdee(destination: &Path, fs_provider: &mut VersionedFileSystem) -> PathBuf {
    copy_tree(
        &fixture(GAME_FIXTURES).join("content/steam-iwdee"),
        destination,
    );
    fs_provider.add_version(&destination.join("icewind.exe"), "2.7.3.0");
    destination.to_path_buf()
}

fn fixture_profiles() -> GameProfiles {
    GameProfiles::load(fixture(GAME_FIXTURES).join("iwdee-profiles")).unwrap()
}

#[test]
fn release_iwdee_profile_loads_with_its_steam_identity() {
    let profiles =
        GameProfiles::load(fixture(GAME_FIXTURES).join("iwdee-release-profile")).unwrap();
    assert_eq!(profiles.len(), 1);
    let profile = profiles
        .matching(GameRole::Iwdee, Storefront::Steam)
        .next()
        .unwrap();
    assert_eq!(profile.id, "steam-iwdee-2.7.3-en-us");
    assert_eq!(profile.executable, "icewind.exe");
    assert_eq!(profile.product_version, "2.7.3.0");
    assert_eq!(profile.locale, "en_US");
    assert_eq!(profile.steam_app_id, Some(321800));
    assert!(profile.sod_files.is_empty());
    assert_eq!(
        profiles
            .matching(GameRole::Bg2ee, Storefront::Steam)
            .count(),
        0
    );
}

#[test]
fn steam_discovery_finds_iwdee_beside_eet_games() {
    let temp = tempfile::tempdir().unwrap();
    let profile_dir = temp.path().join("profiles");
    fs::create_dir_all(&profile_dir).unwrap();
    for source in [
        fixture(GAME_FIXTURES).join("profiles/steam-bg2ee.toml"),
        fixture(GAME_FIXTURES).join("iwdee-profiles/steam-iwdee.toml"),
    ] {
        fs::copy(&source, profile_dir.join(source.file_name().unwrap())).unwrap();
    }
    let profiles = GameProfiles::load(&profile_dir).unwrap();

    let steam_root = temp.path().join("Steam");
    let steamapps = steam_root.join("steamapps");
    fs::create_dir_all(&steamapps).unwrap();
    fs::write(
        steamapps.join("libraryfolders.vdf"),
        format!(
            "\"libraryfolders\"\n{{\n  \"0\"\n  {{\n    \"path\"  \"{}\"\n    \"apps\"\n    {{\n      \"321800\"  \"1\"\n    }}\n  }}\n}}\n",
            steam_root.to_string_lossy().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    fs::write(
        steamapps.join("appmanifest_321800.acf"),
        "\"AppState\"\n{\n  \"appid\"  \"321800\"\n  \"installdir\"  \"Icewind Dale Enhanced Edition\"\n}\n",
    )
    .unwrap();
    let mut fs_provider = VersionedFileSystem::default();
    let game = fixture_iwdee(
        &steamapps.join("common/Icewind Dale Enhanced Edition"),
        &mut fs_provider,
    );
    let mut registry = FixtureRegistry::default();
    registry.values.insert(
        (
            RegistryHive::CurrentUser,
            STEAM_KEY.to_ascii_lowercase(),
            "steampath".to_owned(),
        ),
        steam_root.to_string_lossy().into_owned(),
    );

    let candidates = discover_games(&profiles, &registry, &fs_provider).unwrap();
    assert_eq!(candidates.len(), 1, "{candidates:#?}");
    let candidate = &candidates[0];
    assert_eq!(candidate.role, GameRole::Iwdee);
    assert_eq!(candidate.storefront, Storefront::Steam);
    assert_eq!(candidate.root, fs::canonicalize(game).unwrap());
    assert_eq!(
        candidate.eligibility,
        Eligibility::Eligible,
        "{candidate:#?}"
    );
    assert!(candidate
        .findings
        .iter()
        .any(|finding| finding.kind == FindingKind::Fresh));
}

#[test]
fn clean_iwdee_with_stock_steam_files_is_eligible() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = VersionedFileSystem::default();
    let game = fixture_iwdee(&temp.path().join("IWD"), &mut fs_provider);
    assert!(game.join("weidu.conf").is_file());
    assert!(game.join("steam_appid.txt").is_file());

    let candidate = inspect_game_path(
        &fixture_profiles(),
        &fs_provider,
        GameRole::Iwdee,
        Storefront::Steam,
        &game,
    )
    .unwrap();
    assert_eq!(
        candidate.eligibility,
        Eligibility::Eligible,
        "{candidate:#?}"
    );
    assert_eq!(candidate.role, GameRole::Iwdee);
}

#[test]
fn modded_iwdee_residue_makes_the_source_ineligible() {
    for residue in ["setup-iwdfix.exe", "WeiDU.log", "override/spwi101.spl"] {
        let temp = tempfile::tempdir().unwrap();
        let mut fs_provider = VersionedFileSystem::default();
        let game = fixture_iwdee(&temp.path().join("IWD"), &mut fs_provider);
        let path = game.join(residue);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"modded").unwrap();

        let candidate = inspect_game_path(
            &fixture_profiles(),
            &fs_provider,
            GameRole::Iwdee,
            Storefront::Steam,
            &game,
        )
        .unwrap();
        assert_ne!(
            candidate.eligibility,
            Eligibility::Eligible,
            "{residue}: {candidate:#?}"
        );
        assert!(
            candidate
                .findings
                .iter()
                .any(|finding| finding.kind == FindingKind::Modified),
            "{residue}: {candidate:#?}"
        );
    }
}

#[test]
fn an_iwdee_folder_is_not_accepted_as_a_baldurs_gate_source() {
    let temp = tempfile::tempdir().unwrap();
    let mut fs_provider = VersionedFileSystem::default();
    let game = fixture_iwdee(&temp.path().join("IWD"), &mut fs_provider);
    let profiles = GameProfiles::load(fixture(GAME_FIXTURES).join("profiles")).unwrap();
    let candidate = inspect_game_path(
        &profiles,
        &fs_provider,
        GameRole::Bg2ee,
        Storefront::Steam,
        &game,
    )
    .unwrap();
    assert_ne!(
        candidate.eligibility,
        Eligibility::Eligible,
        "{candidate:#?}"
    );
}

/// Read-only check of the locally installed IWD:EE against the release profile. It only
/// reads files; run it explicitly with `--ignored` on a machine with that install.
#[test]
#[ignore = "reads the local Steam IWD:EE install read-only; run explicitly"]
fn local_steam_iwdee_matches_the_release_profile() {
    let root = Path::new(REAL_IWDEE);
    let profiles =
        GameProfiles::load(fixture(GAME_FIXTURES).join("iwdee-release-profile")).unwrap();
    let candidate = inspect_game_path(
        &profiles,
        &SystemFileSystem,
        GameRole::Iwdee,
        Storefront::Steam,
        root,
    )
    .unwrap();
    assert_eq!(
        candidate.eligibility,
        Eligibility::Eligible,
        "{candidate:#?}"
    );
    assert!(candidate
        .findings
        .iter()
        .any(|finding| finding.kind == FindingKind::Fresh));
}
