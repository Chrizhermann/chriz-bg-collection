//! The Icewind Dale app's native bridge: one bundled collection, one source game in the `bg2`
//! slot, its own registry folder and texts, and no Baldur's Gate-only features. The test build
//! is the Baldur's Gate app, so these tests switch the bridge to the Icewind Dale flavor.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bg_engine::cli::{
    CampaignReport, CampaignStatus, CliError, InstallCommandRequest, InstallReviewIdentity,
};
use bg_engine::digest::sha256_bytes;
use bg_engine::games::{
    Eligibility, FindingKind, GameCandidate, GameFinding, GameRole, Storefront,
};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::registry::{ManagedInstallRecord, ManagedInstallRegistry, REGISTRY_SCHEMA_VERSION};
use bg_engine::session::SourceGameFingerprints;
use bg_engine::weidu::runner::RunnerControlHandle;
use chriz_bg_app_lib::bridge::{
    display_windows_path, BootstrapResponse, BridgeEngine, BridgeSystem, NativeBridge,
    RadarProcessOutcome, SequencedEventSink,
};
use chriz_bg_app_lib::flavor::{BG, IWD};
use chriz_bg_app_lib::shortcut::ShortcutRequest;
use serde_json::json;
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical workspace root")
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create fixture directory");
    for entry in fs::read_dir(source).expect("read fixture directory") {
        let entry = entry.expect("read fixture entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("read fixture entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy fixture file");
        }
    }
}

/// A packaged-resource folder that holds only the Icewind Dale app's bundled recipe.
fn iwd_resources() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("create resource fixture");
    let resources = temp.path().join("resources");
    copy_tree(
        &workspace_root().join("recipes/iwd-current"),
        &resources.join("recipes/iwd-current"),
    );
    (temp, resources)
}

fn candidate(role: GameRole, root: &Path) -> GameCandidate {
    GameCandidate {
        role,
        storefront: Storefront::Steam,
        root: root.to_path_buf(),
        build: Some("2.7.3.0".to_owned()),
        eligibility: Eligibility::Eligible,
        findings: vec![GameFinding {
            kind: FindingKind::Fresh,
            message: "Clean supported installation.".to_owned(),
            paths: Vec::new(),
        }],
        fingerprint: Some("11".repeat(32)),
    }
}

/// Discovers an Icewind Dale copy beside both Baldur's Gate sources and records installs.
struct FakeEngine {
    games: Vec<GameCandidate>,
    installs: Mutex<Vec<InstallCommandRequest>>,
    profile_repairs: Mutex<usize>,
}

impl BridgeEngine for FakeEngine {
    fn discover(&self, _recipe: &Path) -> Result<Vec<GameCandidate>, CliError> {
        Ok(self.games.clone())
    }

    fn inspect_explicit(
        &self,
        _recipe: &Path,
        role: GameRole,
        path: &Path,
    ) -> Result<GameCandidate, CliError> {
        Ok(candidate(role, path))
    }

    fn reinspect(
        &self,
        _recipe: &Path,
        candidate: &GameCandidate,
    ) -> Result<GameCandidate, CliError> {
        Ok(candidate.clone())
    }

    fn review(&self, request: &InstallCommandRequest) -> Result<InstallReviewIdentity, CliError> {
        Ok(InstallReviewIdentity {
            display_name: request.display_name.clone(),
            recipe_payload_sha256: "aa".repeat(32),
            selection_sha256: "bb".repeat(32),
            plan_sha256: "cc".repeat(32),
            source_games: SourceGameFingerprints {
                bg1: None,
                bg2: "11".repeat(32),
                standalone_role: Some(GameRole::Iwdee),
            },
            managed_root: request.managed_root.clone(),
            cache_root: request.cache.clone(),
        })
    }

    fn install(
        &self,
        request: &InstallCommandRequest,
        _expected: &InstallReviewIdentity,
        _sink: &SequencedEventSink,
        _controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, CliError> {
        self.installs.lock().unwrap().push(request.clone());
        Ok(CampaignReport {
            install_id: "install-iwd-fixture".to_owned(),
            managed_root: request.managed_root.clone(),
            plan_sha256: "cc".repeat(32),
            status: CampaignStatus::Complete,
        })
    }

    fn resume(
        &self,
        _managed_root: &Path,
        _expected_install_id: &str,
        _sink: &SequencedEventSink,
        _controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, CliError> {
        unreachable!("no test resumes an installation")
    }

    fn repair_profile_settings(&self, _record: &ManagedInstallRecord) -> Result<bool, String> {
        *self.profile_repairs.lock().unwrap() += 1;
        Ok(true)
    }
}

#[derive(Default)]
struct RecordingSystem {
    launches: Mutex<Vec<PathBuf>>,
}

impl BridgeSystem for RecordingSystem {
    fn launch(&self, executable: &Path, _working_directory: &Path) -> Result<(), String> {
        self.launches.lock().unwrap().push(executable.to_path_buf());
        Ok(())
    }

    fn launch_radar(
        &self,
        _executable: &Path,
        _working_directory: &Path,
    ) -> Result<RadarProcessOutcome, String> {
        Err("the Icewind Dale app never starts BG Radar Overlay".to_owned())
    }

    fn open_folder(&self, _path: &Path) -> Result<(), String> {
        Ok(())
    }

    fn open_https(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }

    fn export_diagnostics(&self, _managed_root: &Path, output: &Path) -> Result<PathBuf, String> {
        Ok(output.to_path_buf())
    }

    fn current_exe(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from(
            r"C:\Program Files\Chriz Easy IWD\chriz-easy-iwd.exe",
        ))
    }

    fn create_desktop_shortcut(&self, _request: ShortcutRequest) -> Result<PathBuf, String> {
        Ok(PathBuf::from(
            r"C:\Users\Player\Desktop\Play Chriz Easy IWD.lnk",
        ))
    }
}

struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    app_data: PathBuf,
    iwd: PathBuf,
    engine: Arc<FakeEngine>,
    system: Arc<RecordingSystem>,
    bridge: NativeBridge,
}

/// An Icewind Dale bridge over a copy of the real IWD recipe, with injected engine and host.
fn iwd_fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("create bridge fixture");
    let root = temp.path().canonicalize().unwrap();
    let recipe = root.join("recipe");
    copy_tree(&workspace_root().join("recipes/iwd-current"), &recipe);
    let cache = root.join("cache");
    let app_data = root.join("local").join(IWD.application_data_directory);
    let (bg1, bg2, iwd) = (root.join("bgee"), root.join("bg2ee"), root.join("iwdee"));
    for folder in [&cache, &app_data, &bg1, &bg2, &iwd] {
        fs::create_dir_all(folder).unwrap();
    }
    let engine = Arc::new(FakeEngine {
        games: vec![
            candidate(GameRole::BgeeSod, &bg1),
            candidate(GameRole::Bg2ee, &bg2),
            candidate(GameRole::Iwdee, &iwd),
        ],
        installs: Mutex::new(Vec::new()),
        profile_repairs: Mutex::new(0),
    });
    let system = Arc::new(RecordingSystem::default());
    let bridge = NativeBridge::with_engine_and_system(
        recipe,
        "chris-recommended",
        cache,
        app_data.clone(),
        engine.clone(),
        system.clone(),
    )
    .with_flavor(&IWD);
    Fixture {
        _temp: temp,
        root,
        app_data,
        iwd,
        engine,
        system,
        bridge,
    }
}

fn recommended() -> NormalizedSelection {
    NormalizedSelection {
        platform: "windows".to_owned(),
        features: Default::default(),
        inputs: Default::default(),
    }
}

/// Publishes a completed installation whose verified launch path is `launcher` below `game`.
fn publish_install(app_data: &Path, managed: &Path, launcher: &str) -> String {
    fs::create_dir_all(managed.join(".chriz")).unwrap();
    fs::create_dir_all(managed.join("game")).unwrap();
    fs::write(managed.join(".chriz/install-receipt.json"), b"receipt").unwrap();
    let launch = managed.join("game").join(launcher);
    fs::write(&launch, b"verified launcher").unwrap();
    let install_id = "install-iwd-launch".to_owned();
    ManagedInstallRegistry::open_or_create(app_data)
        .unwrap()
        .publish(&ManagedInstallRecord {
            schema_version: REGISTRY_SCHEMA_VERSION,
            install_id: install_id.clone(),
            display_name: "Chriz Easy IWD".to_owned(),
            managed_root: managed.canonicalize().unwrap(),
            recipe_version: "0.1.0-alpha.1".to_owned(),
            recipe_sha256: "11".repeat(32),
            engine_name: "iwd-fixture".to_owned(),
            managed_save_root: managed.parent().unwrap().join("saves"),
            launch_path: launch.canonicalize().unwrap(),
            receipt_sha256: sha256_bytes(b"receipt"),
            completed_at_millis: 1,
            recipe_id: Some("chriz-easy-iwd".to_owned()),
        })
        .unwrap();
    install_id
}

#[test]
fn bootstrap_reports_the_bundled_iwd_collection_and_its_one_source() {
    let (temp, resources) = iwd_resources();
    let bridge =
        NativeBridge::from_resource_dir_for_flavor(&resources, &temp.path().join("cache"), &IWD);

    let status = bridge
        .bootstrap()
        .expect("the bundled IWD recipe passes its startup checks");

    assert_eq!(status.selected_profile, "iwd-current");
    assert_eq!(status.recipe_version.as_deref(), Some("0.1.0-alpha.1"));
    assert_eq!(status.profiles.len(), 1);
    assert_eq!(status.profiles[0].id, "iwd-current");
    let wire = serde_json::to_value(&status).unwrap();
    assert_eq!(
        wire["collection"],
        json!({
            "flavor": "iwd",
            "game": "iwdee",
            "product_name": "Chriz Easy IWD",
            "sources": [{"slot": "bg2", "role": "iwdee", "label": "Icewind Dale: Enhanced Edition"}],
        })
    );
}

#[test]
fn the_baldurs_gate_bootstrap_has_no_collection_field() {
    let status = BootstrapResponse {
        mode: "native".to_owned(),
        engine_version: "0".to_owned(),
        application_version: "0".to_owned(),
        profiles: Vec::new(),
        selected_profile: "curated-full-current".to_owned(),
        recipe_version: None,
        startup_install_id: None,
        collection: None,
    };
    assert!(serde_json::to_value(status)
        .unwrap()
        .get("collection")
        .is_none());
}

#[test]
fn each_app_refuses_the_other_apps_recipe() {
    let fixture = iwd_fixture();
    let bg = fixture.bridge.clone().with_flavor(&BG);
    let error = bg
        .bootstrap()
        .expect_err("the BG app must not install the IWD recipe");
    assert_eq!(error.code, "collection_mismatch");
    assert!(error.message.contains("Chriz Easy BG"), "{error:?}");

    let bg_recipe = fixture.root.join("bg-recipe");
    copy_tree(&workspace_root().join("manifest"), &bg_recipe);
    let iwd = NativeBridge::with_engine_and_system(
        bg_recipe,
        "chris-recommended",
        fixture.root.join("cache"),
        fixture.app_data.clone(),
        fixture.engine.clone(),
        fixture.system.clone(),
    )
    .with_flavor(&IWD);
    let error = iwd
        .bootstrap()
        .expect_err("the IWD app must not install the EET recipe");
    assert_eq!(error.code, "collection_mismatch");
    assert!(error.message.contains("Chriz Easy IWD"), "{error:?}");
}

#[test]
fn only_the_bundled_collection_can_be_selected() {
    let (temp, resources) = iwd_resources();
    let bridge =
        NativeBridge::from_resource_dir_for_flavor(&resources, &temp.path().join("cache"), &IWD);
    let selected = bridge
        .select_profile("iwd-current")
        .expect("select the bundled collection");
    assert_eq!(
        selected.bootstrap().unwrap().selected_profile,
        "iwd-current"
    );
    for other in [
        "curated-full-current",
        "public-alpha",
        "creator-full-current",
    ] {
        assert!(bridge.select_profile(other).is_err(), "{other}");
    }
}

#[test]
fn discovery_offers_only_icewind_dale_in_the_bg2_slot() {
    let fixture = iwd_fixture();
    let discovery = fixture.bridge.discover_games().unwrap();
    assert!(discovery.bg1_candidates.is_empty());
    assert_eq!(discovery.selected_bg1_id, "");
    assert_eq!(discovery.bg2_candidates.len(), 1);
    assert!(discovery.bg2_candidates[0]
        .label
        .starts_with("Icewind Dale: Enhanced Edition"));
    assert_eq!(discovery.selected_bg2_id, discovery.bg2_candidates[0].id);
}

#[test]
fn review_and_install_use_one_source_game() {
    let fixture = iwd_fixture();
    let discovery = fixture.bridge.discover_games().unwrap();
    let game = discovery.selected_bg2_id.clone();
    let destination = fixture.root.join("Games").join("Chriz Easy IWD");

    let review = fixture
        .bridge
        .freeze_review("Chriz Easy IWD", &recommended(), &destination, "", &game)
        .expect("freeze an IWD review with one source");
    assert_eq!(review.game_labels.len(), 1);
    assert!(review.game_labels[0].starts_with("Icewind Dale: Enhanced Edition"));

    let started = fixture
        .bridge
        .start_build(&review.review_token, |_| {})
        .expect("start the reviewed IWD build");
    let deadline = Instant::now() + Duration::from_secs(10);
    while fixture
        .bridge
        .get_run_snapshot(&started.run_id)
        .unwrap()
        .status
        == "running"
    {
        assert!(Instant::now() < deadline, "the fake install never finished");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        fixture
            .bridge
            .get_run_snapshot(&started.run_id)
            .unwrap()
            .status,
        "complete"
    );
    let installs = fixture.engine.installs.lock().unwrap();
    assert_eq!(installs.len(), 1);
    assert_eq!(installs[0].bg1, None);
    assert_eq!(installs[0].bg2, fixture.iwd.canonicalize().unwrap());
}

#[test]
fn a_baldurs_gate_source_never_fills_the_iwd_slots() {
    let fixture = iwd_fixture();
    let discovery = fixture.bridge.discover_games().unwrap();
    let game = discovery.selected_bg2_id.clone();
    let bg2 = fixture
        .bridge
        .inspect_game_path(GameRole::Bg2ee, &fixture.root.join("bg2ee"))
        .unwrap()
        .id;
    let destination = fixture.root.join("Games").join("Chriz Easy IWD");

    for (bg1_id, bg2_id) in [(game.as_str(), game.as_str()), ("", bg2.as_str())] {
        let error = fixture
            .bridge
            .freeze_review(
                "Chriz Easy IWD",
                &recommended(),
                &destination,
                bg1_id,
                bg2_id,
            )
            .expect_err("an IWD review takes exactly one IWD source");
        assert_eq!(error.code, "game_candidate_role_mismatch", "{error:?}");
    }
}

#[test]
fn destination_checks_name_the_game_source_and_the_app() {
    let fixture = iwd_fixture();
    let game = fixture.bridge.discover_games().unwrap().selected_bg2_id;

    let ready = fixture
        .bridge
        .inspect_destination(
            &fixture.root.join("Games").join("Chriz Easy IWD"),
            "",
            &game,
        )
        .unwrap();
    assert!(ready.safe);
    assert!(
        ready
            .detail
            .starts_with("Chriz Easy IWD will create this folder"),
        "{ready:?}"
    );

    let inside = fixture
        .bridge
        .inspect_destination(&fixture.iwd.join("copy"), "", &game)
        .expect_err("a copy inside the source game is unsafe");
    assert_eq!(inside.code, "destination_unsafe");
    assert!(
        inside.technical_detail.contains("overlaps the game source"),
        "{inside:?}"
    );
}

#[test]
fn launch_accepts_the_games_own_executable_and_skips_eet_settings() {
    let fixture = iwd_fixture();
    let managed = fixture.root.join("Games").join("Chriz Easy IWD");
    let install_id = publish_install(&fixture.app_data, &managed, "icewind.exe");

    let launched = fixture
        .bridge
        .launch_install(&install_id, false)
        .expect("launch the IWD installation without EEex");

    assert_eq!(
        serde_json::to_value(&launched.game_settings).unwrap()["state"],
        "unchanged"
    );
    assert_eq!(*fixture.engine.profile_repairs.lock().unwrap(), 0);
    let launches = fixture.system.launches.lock().unwrap().clone();
    assert_eq!(launches.len(), 1);
    assert!(launches[0].ends_with("game/icewind.exe"), "{launches:?}");

    // The Baldur's Gate app launches only InfinityLoader.
    let bg = fixture.bridge.clone().with_flavor(&BG);
    assert_eq!(
        bg.launch_install(&install_id, false).unwrap_err().code,
        "managed_launch_invalid"
    );
}

#[test]
fn baldurs_gate_only_features_are_refused_before_anything_is_read() {
    let fixture = iwd_fixture();
    let checks = [
        fixture
            .bridge
            .launch_install("install-unknown", true)
            .map(|_| ()),
        fixture.bridge.install_radar("install-unknown").map(|_| ()),
        fixture
            .bridge
            .inspect_install_patches("install-unknown")
            .map(|_| ()),
        fixture
            .bridge
            .apply_install_patch("install-unknown", "token", false, false)
            .map(|_| ()),
        fixture
            .bridge
            .inspect_mod_upgrade("install-unknown")
            .map(|_| ()),
        fixture
            .bridge
            .restore_mod_upgrade("install-unknown", true)
            .map(|_| ()),
    ];
    for result in checks {
        let error = result.expect_err("a BG-only feature must be refused");
        assert_eq!(error.code, "feature_unavailable", "{error:?}");
        assert!(error.message.contains("Chriz Easy IWD"), "{error:?}");
    }
}

#[test]
fn new_installations_are_named_after_the_iwd_app() {
    let fixture = iwd_fixture();
    let home = fixture.root.join("Player");
    let defaults = fixture.bridge.suggest_installation_defaults(&home).unwrap();
    assert_eq!(defaults.name, "Chriz Easy IWD");
    assert_eq!(
        defaults.path,
        display_windows_path(&home.join("Games").join("Chriz Easy IWD")).unwrap()
    );
    assert!(!home.exists(), "suggesting defaults creates nothing");
}
