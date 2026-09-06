use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use bg_engine::cli::{
    CampaignReport, CampaignStatus, InstallCommandRequest, InstallReviewIdentity,
};
use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::events::{EngineEvent, EventSink, StepOutcome};
use bg_engine::games::{
    Eligibility, FindingKind, GameCandidate, GameFinding, GameRole, Storefront,
};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::registry::{ManagedInstallRecord, ManagedInstallRegistry, REGISTRY_SCHEMA_VERSION};
use bg_engine::resolve::InstallPlan;
use bg_engine::session::{CampaignCreated, SessionEvent, SessionStore, SourceGameFingerprints};
use bg_engine::weidu::runner::{RunnerControl, RunnerControlHandle};
use chriz_bg_app_lib::bridge::{
    display_windows_path, installation_defaults, parse_startup_install_id, BridgeEngine,
    BridgeSystem, NativeBridge, RunEventEnvelope, SequencedEventSink,
};
use chriz_bg_app_lib::error::CommandError;
use chriz_bg_app_lib::shortcut::{shortcut_file_name, ShortcutRequest};
use serde_json::json;
use tempfile::TempDir;

use bg_engine::recipe_envelope::{RecipeEnvelope, RecipeTrustStore, TrustedPublicKey};
use chriz_bg_app_lib::updates::{RecipeUpdateManager, RecipeUpdateState};
use minisign::{sign, KeyPair};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical workspace root")
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create fixture directory");
    let mut entries = fs::read_dir(source)
        .expect("read fixture directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("read fixture entries");
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
    for entry in entries {
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("read fixture entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy fixture file");
        }
    }
}

fn recipe_with_profiles() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("create bridge fixture root");
    let recipe = temp.path().join("recipe");
    copy_tree(
        &workspace_root().join("engine/tests/fixtures/manifest"),
        &recipe,
    );
    copy_tree(
        &workspace_root().join("engine/tests/fixtures/games/profiles"),
        &recipe.join("game-builds"),
    );
    (temp, recipe)
}

fn add_manual_artifact(recipe: &Path, id: &str, filename: &str, bytes: &[u8]) {
    let sha256 = sha256_bytes(bytes);
    let manifest = format!(
        r#"id = "{id}"
name = "Manual fixture"
version = "1.0"
acquisition = "manual-user-supplied"

[source]
kind = "manual"
url = "https://example.invalid/{id}"
reference = "1.0"
expected_filename = "{filename}"
expected_length = {length}
sha256 = "{sha256}"
redirect_hosts = []

[archive]
kind = "zip"
root_rule = "direct"
publish_roots = ["manual-fixture"]
tp2_paths = ["manual-fixture/setup-manual-fixture.tp2"]

[archive.limits]
max_depth = 8
max_entries = 64
max_entry_uncompressed_bytes = 1048576
max_total_uncompressed_bytes = 4194304
max_compression_ratio = 100

[provenance]
homepage = "https://example.invalid/{id}"
license = "User-supplied test fixture"
url = "https://example.invalid/{id}"
reviewed_on = "2026-09-04"
"#,
        length = bytes.len(),
    );
    fs::write(
        recipe.join("artifacts").join(format!("{id}.toml")),
        manifest,
    )
    .expect("write manual artifact fixture");
}

fn set_artifact_archive_kind(recipe: &Path, id: &str, kind: &str) {
    let path = recipe.join("artifacts").join(format!("{id}.toml"));
    let contents = fs::read_to_string(&path)
        .unwrap()
        .replace("kind = \"zip\"", &format!("kind = \"{kind}\""));
    fs::write(path, contents).unwrap();
}

fn make_eefixpack_manual(recipe: &Path, filename: &str, bytes: &[u8]) {
    let path = recipe.join("artifacts/eefixpack.toml");
    let contents = fs::read_to_string(&path)
        .unwrap()
        .replace(
            "acquisition = \"fetch-only\"",
            "acquisition = \"manual-user-supplied\"",
        )
        .replace("kind = \"github-release\"", "kind = \"manual\"")
        .replace(
            "expected_filename = \"ee-fixpack.zip\"",
            &format!("expected_filename = \"{filename}\""),
        )
        .replace(
            "expected_length = 1",
            &format!("expected_length = {}", bytes.len()),
        )
        .replace(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            &sha256_bytes(bytes),
        );
    fs::write(path, contents).unwrap();
    let collection = recipe.join("collection.toml");
    let mut contents = fs::read_to_string(&collection).unwrap();
    contents.push_str(
        r#"

[[features]]
id = "feature:eefixpack:required"
title = "EE Fixpack"
description = "Required fixture components."
category = "core"
decision = "mandatory"
readiness = "ready"
components = [
  { run_id = "eefixpack-bg1", component = 0 },
  { run_id = "eefixpack-bg1", component = 2 },
  { run_id = "eefixpack-bg2", component = 0 },
]
"#,
    );
    fs::write(collection, contents).unwrap();
}

fn candidate(
    role: GameRole,
    storefront: Storefront,
    root: &str,
    eligibility: Eligibility,
    kind: FindingKind,
    message: &str,
) -> GameCandidate {
    GameCandidate {
        role,
        storefront,
        root: PathBuf::from(root),
        build: Some("2.7.3.0".to_owned()),
        eligibility,
        findings: vec![GameFinding {
            kind,
            message: message.to_owned(),
            paths: Vec::new(),
        }],
        fingerprint: Some("11".repeat(32)),
    }
}

struct FakeBridgeEngine {
    bg1: PathBuf,
    bg2: PathBuf,
    installs: Mutex<Vec<(InstallCommandRequest, InstallReviewIdentity)>>,
    resumes: Mutex<Vec<(String, PathBuf)>>,
    controls: Mutex<Vec<RunnerControl>>,
    manual_download_drop_dir: Mutex<Option<String>>,
    review_recipe_digest: Mutex<String>,
    install_release: Condvar,
    install_released: Mutex<bool>,
}

impl FakeBridgeEngine {
    fn new(bg1: PathBuf, bg2: PathBuf) -> Self {
        Self {
            bg1,
            bg2,
            installs: Mutex::new(Vec::new()),
            resumes: Mutex::new(Vec::new()),
            controls: Mutex::new(Vec::new()),
            manual_download_drop_dir: Mutex::new(None),
            review_recipe_digest: Mutex::new("aa".repeat(32)),
            install_release: Condvar::new(),
            install_released: Mutex::new(false),
        }
    }

    fn release_install(&self) {
        *self.install_released.lock().unwrap() = true;
        self.install_release.notify_all();
    }

    fn change_review_identity(&self) {
        *self.review_recipe_digest.lock().unwrap() = "dd".repeat(32);
    }

    fn emit_manual_download_from(&self, drop_dir: &str) {
        *self.manual_download_drop_dir.lock().unwrap() = Some(drop_dir.to_owned());
    }
}

#[derive(Default)]
struct RecordingBridgeSystem {
    launches: Mutex<Vec<(PathBuf, PathBuf)>>,
    folders: Mutex<Vec<PathBuf>>,
    urls: Mutex<Vec<String>>,
    diagnostics: Mutex<Vec<(PathBuf, PathBuf)>>,
    shortcuts: Mutex<Vec<ShortcutRequest>>,
}

impl BridgeSystem for RecordingBridgeSystem {
    fn launch(&self, executable: &Path, working_directory: &Path) -> Result<(), String> {
        self.launches
            .lock()
            .unwrap()
            .push((executable.to_path_buf(), working_directory.to_path_buf()));
        Ok(())
    }

    fn open_folder(&self, path: &Path) -> Result<(), String> {
        self.folders.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }

    fn open_https(&self, url: &str) -> Result<(), String> {
        self.urls.lock().unwrap().push(url.to_owned());
        Ok(())
    }

    fn export_diagnostics(&self, managed_root: &Path, output: &Path) -> Result<PathBuf, String> {
        self.diagnostics
            .lock()
            .unwrap()
            .push((managed_root.to_path_buf(), output.to_path_buf()));
        Ok(output.to_path_buf())
    }

    fn current_exe(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from(
            r"C:\Program Files\Chriz Easy BG\Chriz Easy BG.exe",
        ))
    }

    fn create_desktop_shortcut(&self, request: ShortcutRequest) -> Result<PathBuf, String> {
        self.shortcuts.lock().unwrap().push(request);
        Ok(PathBuf::from(r"C:\Users\Chris\Desktop\Chriz Easy BG.lnk"))
    }
}

fn publish_managed_install(app_data: &Path, managed: &Path, install_id: &str) {
    fs::create_dir_all(managed.join(".chriz")).unwrap();
    fs::create_dir_all(managed.join("game")).unwrap();
    let receipt = managed.join(".chriz/install-receipt.json");
    fs::write(&receipt, b"immutable receipt").unwrap();
    let launch = managed.join("game/InfinityLoader.exe");
    fs::write(&launch, b"verified launcher").unwrap();
    ManagedInstallRegistry::open_or_create(app_data)
        .unwrap()
        .publish(&ManagedInstallRecord {
            schema_version: REGISTRY_SCHEMA_VERSION,
            install_id: install_id.to_owned(),
            display_name: "Task 23 fixture".to_owned(),
            managed_root: managed.canonicalize().unwrap(),
            recipe_version: "0.1.0-alpha.1".to_owned(),
            recipe_sha256: "11".repeat(32),
            engine_name: "task23-fixture".to_owned(),
            managed_save_root: managed.parent().unwrap().join("saves"),
            launch_path: launch.canonicalize().unwrap(),
            receipt_sha256: sha256_bytes(b"immutable receipt"),
            completed_at_millis: 1,
        })
        .unwrap();
}

fn publish_started_campaign(
    app_data: &Path,
    managed: &Path,
    cache: &Path,
    install_id: &str,
) -> SessionStore {
    fs::create_dir_all(managed).unwrap();
    fs::create_dir_all(cache).unwrap();
    let managed = managed.canonicalize().unwrap();
    let cache = cache.canonicalize().unwrap();
    let selection = NormalizedSelection {
        platform: "windows".to_owned(),
        features: BTreeMap::new(),
        inputs: BTreeMap::new(),
    };
    let plan = InstallPlan { runs: Vec::new() };
    let payload = b"PK\x03\x04restart fixture".to_vec();
    let envelope = br#"{"kind":"restart-fixture"}"#.to_vec();
    let created = CampaignCreated {
        install_id: install_id.to_owned(),
        attempt_id: format!("attempt-{install_id}"),
        managed_root: managed.clone(),
        cache_root: cache,
        recipe_payload_sha256: sha256_bytes(&payload),
        recipe_payload: payload,
        recipe_envelope_sha256: sha256_bytes(&envelope),
        recipe_envelope: envelope,
        selection_sha256: selection_digest(&selection).unwrap(),
        normalized_selection: selection,
        plan_sha256: plan_digest(&plan).unwrap(),
        source_games: SourceGameFingerprints {
            bg1: "11".repeat(32),
            bg2: "22".repeat(32),
        },
        artifact_identities: Vec::new(),
        tool_identities: Vec::new(),
        staged_bg1: managed.join("bg1"),
        staged_bg2: managed.join("game"),
    };
    let store =
        SessionStore::create(&managed, SessionEvent::Created(Box::new(created.clone()))).unwrap();
    ManagedInstallRegistry::open_or_create(app_data)
        .unwrap()
        .publish_campaign(&created)
        .unwrap();
    store
}

impl BridgeEngine for FakeBridgeEngine {
    fn discover(&self, _recipe: &Path) -> Result<Vec<GameCandidate>, bg_engine::cli::CliError> {
        Ok(vec![
            candidate(
                GameRole::BgeeSod,
                Storefront::Steam,
                self.bg1.to_str().expect("Unicode fixture path"),
                Eligibility::Eligible,
                FindingKind::Fresh,
                "Clean supported installation.",
            ),
            candidate(
                GameRole::Bg2ee,
                Storefront::Steam,
                self.bg2.to_str().expect("Unicode fixture path"),
                Eligibility::Eligible,
                FindingKind::Fresh,
                "Clean supported installation.",
            ),
        ])
    }

    fn inspect_explicit(
        &self,
        _recipe: &Path,
        role: GameRole,
        path: &Path,
    ) -> Result<GameCandidate, bg_engine::cli::CliError> {
        Ok(candidate(
            role,
            Storefront::Steam,
            path.to_str().expect("Unicode fixture path"),
            Eligibility::Eligible,
            FindingKind::Fresh,
            "Clean supported installation.",
        ))
    }

    fn reinspect(
        &self,
        _recipe: &Path,
        candidate: &GameCandidate,
    ) -> Result<GameCandidate, bg_engine::cli::CliError> {
        Ok(candidate.clone())
    }

    fn review(
        &self,
        request: &InstallCommandRequest,
    ) -> Result<InstallReviewIdentity, bg_engine::cli::CliError> {
        Ok(InstallReviewIdentity {
            display_name: request.display_name.clone(),
            recipe_payload_sha256: self.review_recipe_digest.lock().unwrap().clone(),
            selection_sha256: "bb".repeat(32),
            plan_sha256: "cc".repeat(32),
            source_games: SourceGameFingerprints {
                bg1: "11".repeat(32),
                bg2: "11".repeat(32),
            },
            managed_root: request.managed_root.clone(),
            cache_root: request.cache.clone(),
        })
    }

    fn install(
        &self,
        request: &InstallCommandRequest,
        expected: &InstallReviewIdentity,
        sink: &SequencedEventSink,
        controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, bg_engine::cli::CliError> {
        let registration = controls.register().expect("register fake runner controls");
        self.installs
            .lock()
            .expect("install fixture lock")
            .push((request.clone(), expected.clone()));
        sink.emit(EngineEvent::CampaignStarted {
            install_id: "install-fixture".to_owned(),
            resumed: false,
        });
        if let Some(drop_dir) = self.manual_download_drop_dir.lock().unwrap().clone() {
            sink.emit(EngineEvent::ManualDownloadNeeded {
                mod_id: "manual-fixture".to_owned(),
                page: "https://example.invalid/manual-fixture".to_owned(),
                expected_sha256: "11".repeat(32),
                drop_dir,
            });
        }
        let released = self.install_released.lock().unwrap();
        let _released = self
            .install_release
            .wait_while(released, |released| !*released)
            .unwrap();
        self.controls
            .lock()
            .expect("control fixture lock")
            .extend(registration.receiver().try_iter());
        sink.emit(EngineEvent::StepFinished {
            id: "install:fixture".to_owned(),
            outcome: StepOutcome::Succeeded,
        });
        sink.emit(EngineEvent::CampaignFinished {
            install_id: "install-fixture".to_owned(),
        });
        Ok(CampaignReport {
            install_id: "install-fixture".to_owned(),
            managed_root: request.managed_root.clone(),
            plan_sha256: "11".repeat(32),
            status: CampaignStatus::Complete,
        })
    }

    fn resume(
        &self,
        managed_root: &Path,
        expected_install_id: &str,
        sink: &SequencedEventSink,
        _controls: &RunnerControlHandle,
    ) -> Result<CampaignReport, bg_engine::cli::CliError> {
        self.resumes
            .lock()
            .expect("resume fixture lock")
            .push((expected_install_id.to_owned(), managed_root.to_path_buf()));
        sink.emit(EngineEvent::CampaignStarted {
            install_id: "install-fixture".to_owned(),
            resumed: true,
        });
        Ok(CampaignReport {
            install_id: "install-fixture".to_owned(),
            managed_root: managed_root.to_path_buf(),
            plan_sha256: "11".repeat(32),
            status: CampaignStatus::Complete,
        })
    }
}

#[test]
fn installation_defaults_use_the_injected_home_without_creating_directories() {
    let temp = tempfile::tempdir().expect("create defaults fixture root");
    let home = temp.path().join("Chris");
    let expected = home.join("Games").join("Chriz Easy BG");

    let defaults = installation_defaults(&home).expect("project installation defaults");

    assert_eq!(defaults.name, "Chriz Easy BG");
    assert_eq!(defaults.path, display_windows_path(&expected).unwrap());
    assert!(
        !home.exists(),
        "reading defaults must not create the home path"
    );

    let windows_defaults = installation_defaults(Path::new(r"C:\Users\Chris"))
        .expect("project Windows installation defaults");
    assert_eq!(windows_defaults.path, r"C:\Users\Chris\Games\Chriz Easy BG");
}

#[test]
fn display_paths_hide_only_windows_verbatim_prefixes() {
    assert_eq!(
        display_windows_path(Path::new(r"\\?\C:\Games\Example")).unwrap(),
        r"C:\Games\Example"
    );
    assert_eq!(
        display_windows_path(Path::new(r"\\?\UNC\server\share\Example")).unwrap(),
        r"\\server\share\Example"
    );
    assert_eq!(
        display_windows_path(Path::new(r"D:\Games\Example")).unwrap(),
        r"D:\Games\Example"
    );
}

#[test]
fn manual_download_paths_are_prettified_in_live_events_and_snapshots_only() {
    let (_temp, recipe) = recipe_with_profiles();
    let root = recipe.parent().unwrap().to_path_buf();
    let cache = root.join("cache");
    let bg1 = root.join("clean-bg1");
    let bg2 = root.join("clean-bg2");
    fs::create_dir(&cache).unwrap();
    fs::create_dir(&bg1).unwrap();
    fs::create_dir(&bg2).unwrap();
    let engine = Arc::new(FakeBridgeEngine::new(bg1, bg2));
    let canonical_drop_dir = r"\\?\C:\Installer Cache\manual";
    engine.emit_manual_download_from(canonical_drop_dir);
    let bridge = NativeBridge::with_engine(recipe, "recommended", cache, engine.clone());
    let discovery = bridge.discover_games().unwrap();
    let review = bridge
        .freeze_review(
            "Chriz Easy BG",
            &NormalizedSelection {
                platform: "windows".to_owned(),
                features: Default::default(),
                inputs: Default::default(),
            },
            &root.join("installation"),
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .unwrap();
    let (event_tx, event_rx) = mpsc::channel::<RunEventEnvelope>();
    let started = bridge
        .start_build(&review.review_token, move |event| {
            let _ = event_tx.send(event);
        })
        .unwrap();

    let _started_event = event_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let live = event_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let EngineEvent::ManualDownloadNeeded { drop_dir, .. } = live.event else {
        panic!("expected live manual-download event");
    };
    assert_eq!(drop_dir, r"C:\Installer Cache\manual");

    let snapshot = bridge.get_run_snapshot(&started.run_id).unwrap();
    let stored_drop_dir = snapshot.events.iter().find_map(|event| match &event.event {
        EngineEvent::ManualDownloadNeeded { drop_dir, .. } => Some(drop_dir.as_str()),
        _ => None,
    });
    assert_eq!(stored_drop_dir, Some(r"C:\Installer Cache\manual"));
    assert_eq!(
        engine.manual_download_drop_dir.lock().unwrap().as_deref(),
        Some(canonical_drop_dir)
    );

    engine.release_install();
    while !matches!(
        event_rx.recv_timeout(Duration::from_secs(2)).unwrap().event,
        EngineEvent::CampaignFinished { .. }
    ) {}
}

#[test]
fn display_name_is_forwarded_and_changes_the_frozen_review_identity() {
    let (_temp, recipe) = recipe_with_profiles();
    let cache = recipe.parent().unwrap().join("cache");
    fs::create_dir(&cache).expect("create cache fixture");
    let bg1 = recipe.parent().unwrap().join("clean-bg1");
    let bg2 = recipe.parent().unwrap().join("clean-bg2");
    fs::create_dir(&bg1).expect("create BG1 fixture");
    fs::create_dir(&bg2).expect("create BG2 fixture");
    let destination = recipe.parent().unwrap().join("installation");
    let engine = Arc::new(FakeBridgeEngine::new(bg1, bg2));
    let bridge = NativeBridge::with_engine(recipe, "recommended", &cache, engine);
    let discovery = bridge.discover_games().expect("register discovered games");
    let selection = NormalizedSelection {
        platform: "windows".to_owned(),
        features: Default::default(),
        inputs: Default::default(),
    };

    let first = bridge
        .freeze_review(
            "Chriz Easy BG",
            &selection,
            &destination,
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .expect("freeze default display name");
    let renamed = bridge
        .freeze_review(
            "My Baldur's Gate",
            &selection,
            &destination,
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .expect("freeze renamed display name");

    assert_eq!(first.display_name, "Chriz Easy BG");
    assert_eq!(renamed.display_name, "My Baldur's Gate");
    assert_ne!(first.digest, renamed.digest);
}

#[test]
fn command_errors_have_one_stable_serialized_shape() {
    let error = CommandError::new(
        "invalid_selection",
        "That combination is unavailable.",
        "Review the highlighted options and try again.",
        "engine detail",
    );

    assert_eq!(
        serde_json::to_value(error).expect("serialize command error"),
        json!({
            "code": "invalid_selection",
            "message": "That combination is unavailable.",
            "recovery_action": "Review the highlighted options and try again.",
            "technical_detail": "engine detail"
        })
    );
}

#[test]
fn bootstrap_validates_the_recipe_without_claiming_an_unsigned_version() {
    let (_temp, recipe) = recipe_with_profiles();
    let bridge = NativeBridge::new(recipe, "recommended");

    let status = bridge.bootstrap().expect("bootstrap production recipe");

    assert_eq!(status.mode, "native");
    assert_eq!(status.engine_version, "0.1.0");
    assert_eq!(status.recipe_version, None);
    assert_eq!(status.startup_install_id, None);
}

#[test]
fn desktop_shortcut_startup_hint_accepts_only_one_exact_nonempty_argument() {
    assert_eq!(
        parse_startup_install_id(["cebg.exe", "--install-id=install-ready"]),
        Some("install-ready".to_owned())
    );
    assert_eq!(
        parse_startup_install_id(["cebg.exe", "--other=value"]),
        None
    );
    assert_eq!(parse_startup_install_id(["cebg.exe", "--install-id"]), None);
    assert_eq!(
        parse_startup_install_id(["cebg.exe", "--install-id="]),
        None
    );
    assert_eq!(
        parse_startup_install_id([
            "cebg.exe",
            "--install-id=install-one",
            "--install-id=install-two",
        ]),
        None
    );
}

#[test]
fn desktop_shortcut_filename_is_unicode_safe_and_windows_compatible() {
    assert_eq!(
        shortcut_file_name(" Ordinary — 이름<>:\"/\\|?*... "),
        "Ordinary — 이름.lnk"
    );
    assert_eq!(shortcut_file_name("CON"), "CEBG - CON.lnk");
    assert_eq!(shortcut_file_name(" ... "), "Chriz Easy BG.lnk");
    let long = shortcut_file_name(&"이".repeat(200));
    assert!(long.ends_with(".lnk"));
    assert!(long.encode_utf16().count() <= 124);
}

#[test]
fn production_bridge_loads_only_the_recipe_bundled_below_the_resource_directory() {
    let resources = tempfile::tempdir().expect("create packaged resource fixture");
    copy_tree(
        &workspace_root().join("manifest"),
        &resources.path().join("manifest"),
    );
    let bridge = NativeBridge::from_resource_dir(resources.path());

    let status = bridge.bootstrap().expect("bootstrap packaged recipe");

    assert_eq!(status.mode, "native");
    assert_eq!(status.recipe_version.as_deref(), Some("0.1.0-alpha.2"));
}

#[test]
fn packaged_profiles_are_selected_by_known_identity_only() {
    let resources = tempfile::tempdir().unwrap();
    copy_tree(
        &workspace_root().join("manifest"),
        &resources.path().join("manifest"),
    );
    let bridge = NativeBridge::from_resource_dir(resources.path());
    let status = bridge.bootstrap().unwrap();
    assert_eq!(status.profiles.len(), 1);
    assert_eq!(status.profiles[0].id, "public-alpha");
    let selected = bridge.select_profile("public-alpha").unwrap();
    assert_eq!(
        selected.bootstrap().unwrap().selected_profile,
        "public-alpha"
    );
    assert!(bridge.select_profile("../manifest").is_err());
}

#[test]
fn curated_full_profile_is_default_when_packaged_and_keeps_release_validation() {
    let resources = tempfile::tempdir().unwrap();
    copy_tree(
        &workspace_root().join("manifest"),
        &resources.path().join("manifest"),
    );
    let full = resources.path().join("recipes/curated-full-current");
    copy_tree(&workspace_root().join("manifest"), &full);
    let bridge = NativeBridge::from_resource_dir(resources.path());
    let status = bridge.bootstrap().unwrap();
    assert_eq!(status.selected_profile, "curated-full-current");
    assert_eq!(status.profiles[0].id, "curated-full-current");
    assert_eq!(status.profiles.len(), 2);
    assert_eq!(
        bridge
            .select_profile("public-alpha")
            .unwrap()
            .bootstrap()
            .unwrap()
            .selected_profile,
        "public-alpha"
    );
    let cached = NativeBridge::from_resource_dir_with_cache(
        resources.path(),
        &resources.path().join("cache"),
    );
    assert_eq!(
        cached.bootstrap().unwrap().selected_profile,
        "curated-full-current"
    );
    fs::remove_file(full.join("releases/v0.1.0-alpha.1/acceptance.toml")).unwrap();
    assert_eq!(bridge.bootstrap().unwrap_err().code, "validation_failed");
}

#[test]
fn historical_full_recipe_is_not_offered_or_selectable_as_curated() {
    // Keep the historical recipe as evidence, but its presence is not approval.
    assert!(workspace_root()
        .join("recipes/creator-full-current/collection.toml")
        .is_file());
    let bridge = NativeBridge::from_resource_dir(&workspace_root());
    assert!(bridge
        .profiles()
        .iter()
        .all(|profile| profile.id != "creator-full-current"));
    let error = match bridge.select_profile("creator-full-current") {
        Ok(_) => panic!("historical WeiDU replay must not bypass curation"),
        Err(error) => error,
    };
    assert_eq!(error.code, "profile_requires_curation");
}

#[test]
fn production_recipe_bootstraps_and_reaches_game_discovery() {
    let bridge = NativeBridge::with_discoverer(
        workspace_root().join("manifest"),
        "chris-recommended",
        |_| {
            Ok(vec![
                candidate(
                    GameRole::BgeeSod,
                    Storefront::Steam,
                    r"C:\Fixture\BGEE",
                    Eligibility::Eligible,
                    FindingKind::Fresh,
                    "Clean supported installation.",
                ),
                candidate(
                    GameRole::Bg2ee,
                    Storefront::Steam,
                    r"C:\Fixture\BG2EE",
                    Eligibility::Eligible,
                    FindingKind::Fresh,
                    "Clean supported installation.",
                ),
            ])
        },
    );

    let status = bridge.bootstrap().expect("bootstrap production recipe");
    let discovery = bridge
        .discover_games()
        .expect("reach production game discovery");

    assert_eq!(status.mode, "native");
    assert_eq!(discovery.bg1_candidates.len(), 1);
    assert_eq!(discovery.bg2_candidates.len(), 1);
    assert!(!discovery.selected_bg1_id.is_empty());
    assert!(!discovery.selected_bg2_id.is_empty());
}

#[test]
fn discovery_groups_roles_and_selects_each_candidate_independently() {
    let bridge = NativeBridge::with_discoverer(
        workspace_root().join("engine/tests/fixtures/manifest"),
        "recommended",
        |_| {
            Ok(vec![
                candidate(
                    GameRole::Bg2ee,
                    Storefront::Gog,
                    r"C:\Fixture\BG2EE",
                    Eligibility::Experimental,
                    FindingKind::UnverifiedStorefront,
                    "GOG still needs release acceptance.",
                ),
                candidate(
                    GameRole::BgeeSod,
                    Storefront::Steam,
                    r"C:\Fixture\BGEE",
                    Eligibility::Eligible,
                    FindingKind::Fresh,
                    "Clean supported installation.",
                ),
            ])
        },
    );

    let discovery = bridge.discover_games().expect("project discovery");

    assert_eq!(discovery.bg1_candidates.len(), 1);
    assert_eq!(discovery.bg2_candidates.len(), 1);
    assert_eq!(discovery.selected_bg1_id, discovery.bg1_candidates[0].id);
    assert_eq!(discovery.selected_bg2_id, discovery.bg2_candidates[0].id);
    assert_ne!(discovery.selected_bg1_id, discovery.selected_bg2_id);
    assert!(discovery.bg1_candidates[0].eligible);
    assert_eq!(discovery.bg1_candidates[0].storefront, "steam");
    assert_eq!(
        discovery.bg1_candidates[0].build.as_deref(),
        Some("2.7.3.0")
    );
    assert!(!discovery.bg2_candidates[0].eligible);
}

#[test]
fn explicit_game_paths_are_inspected_by_the_engine_before_projection() {
    let (_temp, recipe) = recipe_with_profiles();
    let bridge = NativeBridge::new(recipe, "recommended");
    let game = workspace_root().join("engine/tests/fixtures/games/content/steam-bgee-sod");

    let inspected = bridge
        .inspect_game_path(GameRole::BgeeSod, &game)
        .expect("inspect fixture path");

    assert_eq!(
        Path::new(&inspected.path).canonicalize().unwrap(),
        game.canonicalize().unwrap()
    );
    assert!(!inspected.path.starts_with(r"\\?\"));
    assert!(!inspected.eligible);
    assert!(!inspected.findings.is_empty());
}

#[test]
fn chosen_game_folders_are_validated_and_cancelled_choices_are_noops() {
    let (_temp, recipe) = recipe_with_profiles();
    let bridge = NativeBridge::new(recipe, "recommended");
    let game = workspace_root().join("engine/tests/fixtures/games/content/steam-bgee-sod");

    assert_eq!(
        bridge
            .choose_game_folder(GameRole::BgeeSod, None)
            .expect("cancel folder choice"),
        None
    );
    let inspected = bridge
        .choose_game_folder(GameRole::BgeeSod, Some(game.clone()))
        .expect("inspect chosen folder")
        .expect("chosen folder response");

    assert_eq!(
        Path::new(&inspected.path).canonicalize().unwrap(),
        game.canonicalize().unwrap()
    );
    assert!(!inspected.path.starts_with(r"\\?\"));
}

#[test]
fn evaluate_build_returns_only_semantic_ui_projection_and_phase_summaries() {
    let bridge = NativeBridge::new(workspace_root().join("manifest"), "chris-recommended");

    let evaluation = bridge
        .evaluate_build(&NormalizedSelection {
            platform: "windows".to_owned(),
            features: Default::default(),
            inputs: Default::default(),
        })
        .expect("evaluate recommended build");

    assert!(!evaluation.view.controls.is_empty());
    assert!(evaluation.selected_choice_count > 0);
    assert!(evaluation
        .plan
        .phases
        .iter()
        .any(|phase| phase.id == "main"));
    let serialized = serde_json::to_string(&evaluation).expect("serialize evaluation");
    assert!(!serialized.contains("\"components\":"));
}

#[test]
fn destination_review_freezes_exact_server_side_inputs_and_is_single_use() {
    let (_temp, recipe) = recipe_with_profiles();
    let cache = recipe.parent().unwrap().join("cache");
    fs::create_dir(&cache).expect("create cache fixture");
    let bg1 = recipe.parent().unwrap().join("clean-bg1");
    let bg2 = recipe.parent().unwrap().join("clean-bg2");
    fs::create_dir(&bg1).expect("create BG1 fixture");
    fs::create_dir(&bg2).expect("create BG2 fixture");
    let destination = recipe.parent().unwrap().join("campaign");
    let canonical_destination = recipe
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap()
        .join("campaign");
    let engine = Arc::new(FakeBridgeEngine::new(bg1.clone(), bg2.clone()));
    let bridge = NativeBridge::with_engine(recipe, "recommended", &cache, engine.clone());
    let selection = NormalizedSelection {
        platform: "windows".to_owned(),
        features: Default::default(),
        inputs: Default::default(),
    };

    let discovery = bridge.discover_games().expect("register discovered games");
    let checked = bridge
        .inspect_destination(
            &destination,
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .expect("inspect destination");
    assert!(checked.safe);
    assert_eq!(
        checked.path,
        display_windows_path(&canonical_destination).unwrap()
    );

    let review = bridge
        .freeze_review(
            "Chriz Easy BG",
            &selection,
            &destination,
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .expect("freeze review");
    assert_eq!(
        review.destination,
        display_windows_path(&canonical_destination).unwrap()
    );
    assert_eq!(review.game_labels.len(), 2);
    assert_eq!(review.review_token.len(), 64);
    assert_eq!(review.digest.len(), 64);

    let (event_tx, event_rx) = mpsc::channel::<RunEventEnvelope>();
    let started = bridge
        .start_build(&review.review_token, move |event| {
            let _ = event_tx.send(event);
        })
        .expect("start reviewed build");

    let first = event_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("campaign started event");
    assert!(matches!(first.event, EngineEvent::CampaignStarted { .. }));
    assert_eq!(engine.installs.lock().unwrap().len(), 1);
    let installed = &engine.installs.lock().unwrap()[0];
    assert_eq!(installed.0.bg1, bg1.canonicalize().unwrap());
    assert_eq!(installed.0.bg2, bg2.canonicalize().unwrap());
    assert_eq!(installed.0.managed_root, canonical_destination);
    assert_eq!(installed.0.cache, cache.canonicalize().unwrap());
    assert_eq!(installed.1.managed_root, installed.0.managed_root);
    assert_eq!(installed.1.cache_root, installed.0.cache);
    engine.release_install();
    let mut events = vec![first];
    while events
        .last()
        .is_none_or(|event| !matches!(event.event, EngineEvent::CampaignFinished { .. }))
    {
        events.push(
            event_rx
                .recv_timeout(Duration::from_secs(2))
                .expect("next campaign event"),
        );
    }
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].sequence_as_string, "1");
    assert_eq!(events[2].sequence_as_string, "3");
    assert!(events.iter().all(|event| event.run_id == started.run_id));
    let deadline = Instant::now() + Duration::from_secs(2);
    let snapshot = loop {
        let snapshot = bridge
            .get_run_snapshot(&started.run_id)
            .expect("read completed run snapshot");
        if snapshot.report.is_some() {
            break snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "worker did not publish its report"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(
        snapshot.report.expect("completed report").managed_root,
        PathBuf::from(display_windows_path(&canonical_destination).unwrap())
    );

    let reused = bridge.start_build(&review.review_token, |_| {});
    assert_eq!(reused.unwrap_err().code, "review_token_invalid");
}

#[test]
fn start_rejects_an_engine_identity_that_changed_after_review() {
    let (_temp, recipe) = recipe_with_profiles();
    let cache = recipe.parent().unwrap().join("cache");
    fs::create_dir(&cache).expect("create cache fixture");
    let bg1 = recipe.parent().unwrap().join("clean-bg1");
    let bg2 = recipe.parent().unwrap().join("clean-bg2");
    fs::create_dir(&bg1).expect("create BG1 fixture");
    fs::create_dir(&bg2).expect("create BG2 fixture");
    let destination = recipe.parent().unwrap().join("campaign");
    let engine = Arc::new(FakeBridgeEngine::new(bg1, bg2));
    let bridge = NativeBridge::with_engine(recipe, "recommended", &cache, engine.clone());
    let discovery = bridge.discover_games().expect("register discovered games");
    let review = bridge
        .freeze_review(
            "Chriz Easy BG",
            &NormalizedSelection {
                platform: "windows".to_owned(),
                features: Default::default(),
                inputs: Default::default(),
            },
            &destination,
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .expect("freeze review");

    engine.change_review_identity();
    let error = bridge
        .start_build(&review.review_token, |_| {})
        .expect_err("changed engine identity must not start");

    assert_eq!(error.code, "review_changed");
    assert!(engine.installs.lock().unwrap().is_empty());
    assert_eq!(bridge.active_run_count(), 0);
}

#[test]
fn sequenced_event_sink_uses_string_sequences_and_ignores_a_dropped_listener() {
    let received = Arc::new(Mutex::new(Vec::<RunEventEnvelope>::new()));
    let copy = Arc::clone(&received);
    let sink = SequencedEventSink::new("run-fixture", move |event| {
        copy.lock().expect("event fixture lock").push(event);
    });

    sink.emit(EngineEvent::PhaseStarted {
        name: "main".to_owned(),
    });
    sink.emit(EngineEvent::CampaignFinished {
        install_id: "install-fixture".to_owned(),
    });

    let received = received.lock().unwrap();
    assert_eq!(received[0].run_id, "run-fixture");
    assert_eq!(received[0].sequence_as_string, "1");
    assert_eq!(received[1].sequence_as_string, "2");

    let dropped = SequencedEventSink::new("run-dropped", |_| {});
    dropped.emit(EngineEvent::Error {
        step_id: None,
        message: "renderer gone".to_owned(),
    });
}

#[test]
fn restart_lists_and_resumes_only_a_verified_indexed_campaign() {
    let (_temp, recipe) = recipe_with_profiles();
    let root = recipe.parent().unwrap().to_path_buf();
    let cache = recipe.parent().unwrap().join("cache");
    fs::create_dir(&cache).expect("create cache fixture");
    let app_data = root.join("app-data");
    let destination = root.join("campaign");
    publish_started_campaign(&app_data, &destination, &cache, "install-restart");
    let bg1 = root.join("clean-bg1");
    let bg2 = root.join("clean-bg2");
    fs::create_dir(&bg1).expect("create BG1 fixture");
    fs::create_dir(&bg2).expect("create BG2 fixture");
    let canonical_destination = destination.canonicalize().unwrap();
    let engine = Arc::new(FakeBridgeEngine::new(bg1, bg2));
    // This bridge is constructed only after the campaign/index already exist.
    let bridge = NativeBridge::with_engine_and_system(
        recipe,
        "recommended",
        &cache,
        app_data,
        engine.clone(),
        Arc::new(RecordingBridgeSystem::default()),
    );

    let cards = bridge.list_managed_installations().unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].id, "install-restart");
    assert_eq!(cards[0].name, "Chriz Easy BG — incomplete installation");
    assert!(!cards[0].available);
    assert!(cards[0].resumable);
    assert_eq!(cards[0].receipt_path, None);
    assert_eq!(cards[0].launch_path, None);
    assert_eq!(cards[0].completed_at_millis, None);

    let (resume_tx, resume_rx) = mpsc::channel();
    let resumed = bridge
        .resume_build("install-restart", move |event| {
            let _ = resume_tx.send(event);
        })
        .expect("queue resume");
    let resumed_event = resume_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("resumed campaign event");
    assert_eq!(resumed_event.run_id, resumed.run_id);
    assert!(matches!(
        resumed_event.event,
        EngineEvent::CampaignStarted { resumed: true, .. }
    ));
    assert_eq!(
        engine.resumes.lock().unwrap().as_slice(),
        &[("install-restart".to_owned(), canonical_destination)]
    );

    let unknown = bridge.resume_build("install-unknown", |_| {});
    assert_eq!(unknown.unwrap_err().code, "managed_install_unknown");
}

#[test]
fn restart_keeps_stale_and_fresh_copy_campaigns_visible_but_not_resumable() {
    let (_temp, recipe) = recipe_with_profiles();
    let root = recipe.parent().unwrap();
    let cache = root.join("cache");
    let app_data = root.join("app-data");
    let stale_root = root.join("stale-campaign");
    let sealed_root = root.join("sealed-campaign");
    fs::create_dir(&cache).unwrap();
    publish_started_campaign(&app_data, &stale_root, &cache, "install-stale-start");
    let sealed = publish_started_campaign(&app_data, &sealed_root, &cache, "install-sealed");
    sealed
        .append(SessionEvent::StepStarted {
            step_id: "stage:bg1".to_owned(),
            attempt: 1,
        })
        .unwrap();
    sealed
        .append(SessionEvent::FreshCopyRequired {
            step_id: "stage:bg1".to_owned(),
            attempt: 1,
            detail: "staged bytes changed".to_owned(),
        })
        .unwrap();
    fs::rename(&stale_root, root.join("stale-moved-away")).unwrap();
    let engine = Arc::new(FakeBridgeEngine::new(root.join("bg1"), root.join("bg2")));
    let bridge = NativeBridge::with_engine_and_system(
        recipe,
        "recommended",
        cache,
        app_data,
        engine,
        Arc::new(RecordingBridgeSystem::default()),
    );

    let cards = bridge.list_managed_installations().unwrap();

    assert_eq!(cards.len(), 2);
    assert!(cards.iter().all(|card| !card.available && !card.resumable));
    assert!(cards
        .iter()
        .any(|card| card.id == "install-stale-start" && card.status.contains("unavailable")));
    assert!(cards
        .iter()
        .any(|card| card.id == "install-sealed"
            && card.status == "Needs attention — automatic resume unavailable"));
    assert_eq!(
        bridge
            .resume_build("install-stale-start", |_| {})
            .unwrap_err()
            .code,
        "managed_campaign_stale"
    );
    let sealed_error = bridge.resume_build("install-sealed", |_| {}).unwrap_err();
    assert_eq!(sealed_error.code, "managed_campaign_fresh_copy_required");
    assert_eq!(
        sealed_error.message,
        "That installation needs attention; automatic resume is unavailable."
    );
    assert_eq!(
        sealed_error.recovery_action,
        "Keep this folder unchanged and export diagnostics. A supervised targeted repair may be possible after the underlying problem is assessed and fixed."
    );
}

#[test]
fn start_returns_before_the_worker_finishes() {
    let (_temp, recipe) = recipe_with_profiles();
    let cache = recipe.parent().unwrap().join("cache");
    fs::create_dir(&cache).expect("create cache fixture");
    let bg1 = recipe.parent().unwrap().join("clean-bg1");
    let bg2 = recipe.parent().unwrap().join("clean-bg2");
    fs::create_dir(&bg1).expect("create BG1 fixture");
    fs::create_dir(&bg2).expect("create BG2 fixture");
    let destination = recipe.parent().unwrap().join("campaign");
    let engine = Arc::new(FakeBridgeEngine::new(bg1, bg2));
    let bridge = NativeBridge::with_engine(recipe, "recommended", &cache, engine.clone());
    let discovery = bridge.discover_games().expect("register discovered games");
    let review = bridge
        .freeze_review(
            "Chriz Easy BG",
            &NormalizedSelection {
                platform: "windows".to_owned(),
                features: Default::default(),
                inputs: Default::default(),
            },
            &destination,
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .expect("freeze review");

    let started = bridge
        .start_build(&review.review_token, |_| {})
        .expect("queue build without waiting for completion");
    assert!(started.run_id.starts_with("run-"));
    assert_eq!(bridge.active_run_count(), 1);
    engine.release_install();
}

#[test]
fn destination_checks_use_registered_candidates_and_never_create_the_target() {
    let (_temp, recipe) = recipe_with_profiles();
    let cache = recipe.parent().unwrap().join("cache");
    fs::create_dir(&cache).expect("create cache fixture");
    let bg1 = recipe.parent().unwrap().join("clean-bg1");
    let bg2 = recipe.parent().unwrap().join("clean-bg2");
    fs::create_dir(&bg1).expect("create BG1 fixture");
    fs::create_dir(&bg2).expect("create BG2 fixture");
    let engine = Arc::new(FakeBridgeEngine::new(bg1.clone(), bg2));
    let bridge = NativeBridge::with_engine(recipe, "recommended", &cache, engine);
    let discovery = bridge.discover_games().expect("register discovered games");
    let untouched = cache.parent().unwrap().join("never-created");

    let unknown =
        bridge.inspect_destination(&untouched, "unregistered-bg1", &discovery.selected_bg2_id);
    assert_eq!(unknown.unwrap_err().code, "game_candidate_unknown");
    assert!(!untouched.exists());

    let overlap = bridge.inspect_destination(
        &bg1.join("managed-copy"),
        &discovery.selected_bg1_id,
        &discovery.selected_bg2_id,
    );
    assert_eq!(overlap.unwrap_err().code, "destination_unsafe");
    assert!(!bg1.join("managed-copy").exists());

    let occupied = cache.parent().unwrap().join("occupied");
    fs::create_dir(&occupied).expect("create occupied destination");
    fs::write(occupied.join("existing.txt"), b"owned by somebody else")
        .expect("write occupied fixture");
    let occupied_error = bridge.inspect_destination(
        &occupied,
        &discovery.selected_bg1_id,
        &discovery.selected_bg2_id,
    );
    assert_eq!(occupied_error.unwrap_err().code, "destination_unsafe");
}

#[test]
fn chosen_destinations_are_validated_and_cancelled_choices_are_noops() {
    let (_temp, recipe) = recipe_with_profiles();
    let cache = recipe.parent().unwrap().join("cache");
    fs::create_dir(&cache).expect("create cache fixture");
    let bg1 = recipe.parent().unwrap().join("clean-bg1");
    let bg2 = recipe.parent().unwrap().join("clean-bg2");
    fs::create_dir(&bg1).expect("create BG1 fixture");
    fs::create_dir(&bg2).expect("create BG2 fixture");
    let engine = Arc::new(FakeBridgeEngine::new(bg1, bg2));
    let bridge = NativeBridge::with_engine(recipe, "recommended", &cache, engine);
    let discovery = bridge.discover_games().expect("register discovered games");
    let destination = cache.parent().unwrap().join("chosen-campaign");
    let canonical_destination = cache
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap()
        .join("chosen-campaign");

    assert_eq!(
        bridge
            .choose_destination_folder(
                None,
                "not-consulted-after-cancel",
                "not-consulted-after-cancel",
            )
            .expect("cancel folder choice"),
        None
    );
    let inspected = bridge
        .choose_destination_folder(
            Some(destination.clone()),
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .expect("inspect chosen destination")
        .expect("chosen destination response");

    assert!(inspected.safe);
    assert_eq!(
        inspected.path,
        display_windows_path(&canonical_destination).unwrap()
    );
}

#[test]
fn manual_archive_selection_uses_only_the_trusted_recipe_identity_and_owned_cache_path() {
    let (temp, recipe) = recipe_with_profiles();
    let bytes = b"manual fixture archive";
    add_manual_artifact(&recipe, "manual-fixture", "manual-fixture.zip", bytes);
    let selected = temp.path().join("downloaded-under-an-arbitrary-name.zip");
    fs::write(&selected, bytes).expect("write selected archive");
    let bridge = NativeBridge::new(&recipe, "recommended");

    assert_eq!(
        bridge
            .supply_manual_archive("not-consulted-after-cancel", None)
            .expect("cancel manual archive choice"),
        None
    );
    let supplied = bridge
        .supply_manual_archive("manual-fixture", Some(selected.clone()))
        .expect("supply valid manual archive")
        .expect("manual archive response");

    assert_eq!(supplied.artifact_id, "manual-fixture");
    assert_eq!(supplied.filename, "manual-fixture.zip");
    assert_eq!(supplied.length, bytes.len() as u64);
    assert_eq!(
        fs::read(temp.path().join(".chriz-cache/manual/manual-fixture.zip"))
            .expect("read published manual archive"),
        bytes
    );
    assert_eq!(fs::read(selected).expect("read original selection"), bytes);
    assert_eq!(
        bridge
            .supply_manual_archive("not-a-recipe-artifact", Some(temp.path().join("missing")))
            .expect_err("unknown artifact must fail")
            .code,
        "manual_archive_unknown"
    );
}

#[test]
fn manual_picker_filter_accepts_each_declared_archive_kind_without_executing_sfx_bytes() {
    let (temp, recipe) = recipe_with_profiles();
    let sfx_bytes = b"synthetic executable-framed fixture; never execute";
    add_manual_artifact(&recipe, "manual-zip", "manual.zip", b"zip");
    add_manual_artifact(&recipe, "manual-iemod", "manual.iemod", b"iemod");
    set_artifact_archive_kind(&recipe, "manual-iemod", "iemod");
    add_manual_artifact(&recipe, "manual-sfx", "manual.exe", sfx_bytes);
    set_artifact_archive_kind(&recipe, "manual-sfx", "self-extracting-rar");
    let selected = temp.path().join("official-windows-sfx.exe");
    fs::write(&selected, sfx_bytes).unwrap();
    let bridge = NativeBridge::new(&recipe, "recommended");

    assert_eq!(
        bridge
            .manual_archive_filter_extensions("manual-zip")
            .unwrap(),
        vec!["zip"]
    );
    assert_eq!(
        bridge
            .manual_archive_filter_extensions("manual-iemod")
            .unwrap(),
        vec!["iemod"]
    );
    assert_eq!(
        bridge
            .manual_archive_filter_extensions("manual-sfx")
            .unwrap(),
        vec!["exe"]
    );
    let supplied = bridge
        .supply_manual_archive("manual-sfx", Some(selected))
        .unwrap()
        .unwrap();
    assert_eq!(supplied.filename, "manual.exe");
    assert_eq!(supplied.length, sfx_bytes.len() as u64);
    assert_eq!(
        fs::read(temp.path().join(".chriz-cache/manual/manual.exe")).unwrap(),
        sfx_bytes
    );
}

#[test]
fn manual_download_inspection_is_selection_scoped_deduped_read_only_and_exact() {
    let (temp, recipe) = recipe_with_profiles();
    let expected = b"exact manual fixture";
    make_eefixpack_manual(&recipe, "eefix-official.zip", expected);
    add_manual_artifact(&recipe, "unused-manual", "unused.zip", b"unused");
    let cache = temp.path().join("manual-cache");
    let bridge = NativeBridge::with_engine(
        &recipe,
        "recommended",
        &cache,
        Arc::new(FakeBridgeEngine::new(
            temp.path().join("bg1"),
            temp.path().join("bg2"),
        )),
    );
    let selection = NormalizedSelection {
        platform: "windows".to_owned(),
        features: Default::default(),
        inputs: Default::default(),
    };

    let missing = bridge.inspect_manual_downloads(&selection).unwrap();
    assert_eq!(missing.len(), 1, "two resolved runs share one artifact");
    assert_eq!(missing[0].artifact_id, "eefixpack");
    assert_eq!(missing[0].mod_ids, vec!["eefixpack"]);
    assert_eq!(missing[0].title, "EE Fixpack");
    assert_eq!(missing[0].filename, "eefix-official.zip");
    assert_eq!(missing[0].length, expected.len() as u64);
    assert!(!missing[0].ready);
    assert!(missing[0]
        .detail
        .as_deref()
        .unwrap()
        .contains("not been supplied"));
    assert!(!cache.exists(), "inspection must not create the cache");

    let cached = cache.join("manual/eefix-official.zip");
    fs::create_dir_all(cached.parent().unwrap()).unwrap();
    fs::write(&cached, b"short").unwrap();
    let wrong_length = bridge.inspect_manual_downloads(&selection).unwrap();
    assert!(!wrong_length[0].ready);
    assert!(wrong_length[0]
        .detail
        .as_deref()
        .unwrap()
        .contains("length"));

    fs::write(&cached, vec![b'x'; expected.len()]).unwrap();
    let wrong_hash = bridge.inspect_manual_downloads(&selection).unwrap();
    assert!(!wrong_hash[0].ready);
    assert!(wrong_hash[0]
        .detail
        .as_deref()
        .unwrap()
        .contains("does not match"));

    fs::write(&cached, expected).unwrap();
    let ready = bridge.inspect_manual_downloads(&selection).unwrap();
    assert!(ready[0].ready);
    assert_eq!(ready[0].detail, None);
    assert_eq!(fs::read(&cached).unwrap(), expected);
}

#[test]
fn freeze_and_start_recheck_manual_readiness_before_a_worker_can_start() {
    let (temp, recipe) = recipe_with_profiles();
    let expected = b"exact manual fixture";
    make_eefixpack_manual(&recipe, "eefix-official.zip", expected);
    let cache = temp.path().join("cache");
    let bg1 = temp.path().join("clean-bg1");
    let bg2 = temp.path().join("clean-bg2");
    fs::create_dir(&bg1).unwrap();
    fs::create_dir(&bg2).unwrap();
    let engine = Arc::new(FakeBridgeEngine::new(bg1, bg2));
    let bridge = NativeBridge::with_engine(&recipe, "recommended", &cache, engine.clone());
    let selection = NormalizedSelection {
        platform: "windows".to_owned(),
        features: Default::default(),
        inputs: Default::default(),
    };

    let error = bridge
        .freeze_review(
            "CEBG",
            &selection,
            &temp.path().join("campaign"),
            "unused-bg1",
            "unused-bg2",
        )
        .unwrap_err();
    assert_eq!(error.code, "manual_download_required");
    assert_eq!(bridge.active_run_count(), 0);
    assert!(engine.installs.lock().unwrap().is_empty());
    assert!(
        !cache.exists(),
        "missing-manual freeze must not initialize cache directories"
    );

    let selected = temp.path().join("selected.zip");
    fs::write(&selected, expected).unwrap();
    bridge
        .supply_manual_archive("eefixpack", Some(selected))
        .unwrap();
    let discovery = bridge.discover_games().unwrap();
    let review = bridge
        .freeze_review(
            "CEBG",
            &selection,
            &temp.path().join("campaign"),
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .unwrap();
    fs::remove_file(cache.join("manual/eefix-official.zip")).unwrap();

    let error = bridge
        .start_build(&review.review_token, |_| {})
        .unwrap_err();
    assert_eq!(error.code, "manual_download_required");
    assert_eq!(bridge.active_run_count(), 0);
    assert!(engine.installs.lock().unwrap().is_empty());
}

#[test]
fn build_controls_are_scoped_to_the_named_active_run() {
    let (_temp, recipe) = recipe_with_profiles();
    let cache = recipe.parent().unwrap().join("cache");
    fs::create_dir(&cache).expect("create cache fixture");
    let bg1 = recipe.parent().unwrap().join("clean-bg1");
    let bg2 = recipe.parent().unwrap().join("clean-bg2");
    fs::create_dir(&bg1).expect("create BG1 fixture");
    fs::create_dir(&bg2).expect("create BG2 fixture");
    let destination = recipe.parent().unwrap().join("campaign");
    let engine = Arc::new(FakeBridgeEngine::new(bg1, bg2));
    let bridge = NativeBridge::with_engine(recipe, "recommended", &cache, engine.clone());
    let discovery = bridge.discover_games().expect("register discovered games");
    let review = bridge
        .freeze_review(
            "Chriz Easy BG",
            &NormalizedSelection {
                platform: "windows".to_owned(),
                features: Default::default(),
                inputs: Default::default(),
            },
            &destination,
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .expect("freeze review");
    let (tx, rx) = mpsc::channel();
    let started = bridge
        .start_build(&review.review_token, move |event| {
            let _ = tx.send(event);
        })
        .expect("start build");
    rx.recv_timeout(Duration::from_secs(2))
        .expect("campaign started");

    bridge
        .continue_waiting(&started.run_id)
        .expect("continue named run");
    assert_eq!(
        bridge.pause_run("run-not-active").unwrap_err().code,
        "run_unknown"
    );
    bridge.pause_run(&started.run_id).expect("pause named run");
    assert_eq!(
        bridge
            .get_run_snapshot(&started.run_id)
            .expect("read pausing snapshot")
            .status,
        "pausing"
    );
    assert_eq!(
        bridge.cancel_run("run-not-active").unwrap_err().code,
        "run_unknown"
    );
    bridge
        .cancel_run(&started.run_id)
        .expect("cancel named run");
    engine.release_install();
    while bridge.active_run_count() != 0 {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        engine.controls.lock().unwrap().as_slice(),
        &[RunnerControl::ContinueWaiting, RunnerControl::Cancel]
    );
}

#[test]
fn managed_actions_reload_only_the_exact_available_registry_identity() {
    let (_temp, recipe) = recipe_with_profiles();
    let root = recipe.parent().unwrap();
    let cache = root.join("cache");
    let app_data = root.join("app-data");
    let managed = root.join("managed-campaign");
    fs::create_dir(&cache).unwrap();
    publish_started_campaign(&app_data, &managed, &cache, "install-task23");
    publish_managed_install(&app_data, &managed, "install-task23");
    let engine = Arc::new(FakeBridgeEngine::new(root.join("bg1"), root.join("bg2")));
    let system = Arc::new(RecordingBridgeSystem::default());

    // A fresh bridge instance simulates reopening the application after installation.
    let bridge = NativeBridge::with_engine_and_system(
        recipe,
        "recommended",
        cache,
        app_data,
        engine,
        system.clone(),
    );
    let cards = bridge.list_managed_installations().unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].id, "install-task23");
    assert!(cards[0].available);
    assert!(!cards[0].resumable);
    assert!(cards[0].receipt_path.is_some());
    assert_eq!(cards[0].status, "Ready to play");

    let canonical_managed = managed.canonicalize().unwrap();
    assert_eq!(
        cards[0].path,
        display_windows_path(&canonical_managed).unwrap()
    );
    let expected_receipt =
        display_windows_path(&canonical_managed.join(".chriz/install-receipt.json")).unwrap();
    assert_eq!(
        cards[0].receipt_path.as_deref(),
        Some(expected_receipt.as_str())
    );
    let expected_launch =
        display_windows_path(&canonical_managed.join("game/InfinityLoader.exe")).unwrap();
    assert_eq!(
        cards[0].launch_path.as_deref(),
        Some(expected_launch.as_str())
    );
    assert!(!cards[0]
        .launch_path
        .as_deref()
        .unwrap()
        .starts_with(r"\\?\"));
    assert_eq!(cards[0].completed_at_millis, Some(1));

    bridge.launch_install("install-task23").unwrap();
    bridge.open_install_folder("install-task23").unwrap();
    assert_eq!(
        system.launches.lock().unwrap().as_slice(),
        &[(
            canonical_managed.join("game/InfinityLoader.exe"),
            canonical_managed.join("game"),
        )]
    );
    assert_eq!(
        system.folders.lock().unwrap().as_slice(),
        &[canonical_managed]
    );
    assert_eq!(
        bridge.launch_install("not-a-registry-id").unwrap_err().code,
        "managed_install_unknown"
    );
}

#[test]
fn desktop_shortcut_uses_only_the_available_registry_identity_and_current_app() {
    let (_temp, recipe) = recipe_with_profiles();
    let root = recipe.parent().unwrap();
    let cache = root.join("cache");
    let app_data = root.join("app-data");
    let managed = root.join("managed-campaign");
    fs::create_dir(&cache).unwrap();
    publish_managed_install(&app_data, &managed, "install-shortcut");
    let engine = Arc::new(FakeBridgeEngine::new(root.join("bg1"), root.join("bg2")));
    let system = Arc::new(RecordingBridgeSystem::default());
    let bridge = NativeBridge::with_engine_and_system(
        recipe,
        "recommended",
        cache,
        app_data,
        engine,
        system.clone(),
    );

    let response = bridge
        .create_desktop_shortcut("install-shortcut")
        .expect("create shortcut from available install");
    assert_eq!(response.path, r"C:\Users\Chris\Desktop\Chriz Easy BG.lnk");
    assert_eq!(
        system.shortcuts.lock().unwrap().as_slice(),
        &[ShortcutRequest {
            install_id: "install-shortcut".to_owned(),
            display_name: "Task 23 fixture".to_owned(),
            target: PathBuf::from(r"C:\Program Files\Chriz Easy BG\Chriz Easy BG.exe"),
            arguments: "--install-id=install-shortcut".to_owned(),
            working_directory: PathBuf::from(r"C:\Program Files\Chriz Easy BG"),
        }]
    );
    assert_eq!(
        bridge
            .create_desktop_shortcut("not-a-registry-id")
            .unwrap_err()
            .code,
        "managed_install_unknown"
    );
}

#[test]
fn desktop_shortcut_rejects_stale_and_incomplete_installations() {
    let (_temp, recipe) = recipe_with_profiles();
    let root = recipe.parent().unwrap().to_path_buf();
    let cache = root.join("cache");
    let app_data = root.join("app-data");
    let stale = root.join("stale-campaign");
    let incomplete = root.join("incomplete-campaign");
    fs::create_dir(&cache).unwrap();
    publish_managed_install(&app_data, &stale, "install-stale-shortcut");
    fs::write(stale.join(".chriz/install-receipt.json"), b"changed").unwrap();
    publish_started_campaign(
        &app_data,
        &incomplete,
        &cache,
        "install-incomplete-shortcut",
    );
    let bridge = NativeBridge::with_engine_and_system(
        recipe,
        "recommended",
        cache,
        app_data,
        Arc::new(FakeBridgeEngine::new(root.join("bg1"), root.join("bg2"))),
        Arc::new(RecordingBridgeSystem::default()),
    );

    assert_eq!(
        bridge
            .create_desktop_shortcut("install-stale-shortcut")
            .unwrap_err()
            .code,
        "managed_install_stale"
    );
    assert_eq!(
        bridge
            .create_desktop_shortcut("install-incomplete-shortcut")
            .unwrap_err()
            .code,
        "managed_install_unknown"
    );
}

#[test]
fn stale_managed_install_is_listed_but_cannot_be_launched_or_opened() {
    let (_temp, recipe) = recipe_with_profiles();
    let root = recipe.parent().unwrap();
    let cache = root.join("cache");
    let app_data = root.join("app-data");
    let managed = root.join("managed-campaign");
    fs::create_dir(&cache).unwrap();
    publish_managed_install(&app_data, &managed, "install-stale");
    fs::write(
        managed.join(".chriz/install-receipt.json"),
        b"changed receipt",
    )
    .unwrap();
    let engine = Arc::new(FakeBridgeEngine::new(root.join("bg1"), root.join("bg2")));
    let system = Arc::new(RecordingBridgeSystem::default());
    let bridge = NativeBridge::with_engine_and_system(
        recipe,
        "recommended",
        cache,
        app_data,
        engine,
        system,
    );

    let cards = bridge.list_managed_installations().unwrap();
    assert!(!cards[0].available);
    assert_eq!(cards[0].status, "Unavailable — folder moved or changed");
    assert_eq!(
        bridge.launch_install("install-stale").unwrap_err().code,
        "managed_install_stale"
    );
    assert_eq!(
        bridge
            .open_install_folder("install-stale")
            .unwrap_err()
            .code,
        "managed_install_stale"
    );
}

#[test]
fn diagnostics_and_manual_page_use_native_choices_plus_trusted_recipe_identity() {
    let (_temp, recipe) = recipe_with_profiles();
    add_manual_artifact(&recipe, "manual-fixture", "manual-fixture.zip", b"fixture");
    let root = recipe.parent().unwrap().to_path_buf();
    let cache = root.join("cache");
    let app_data = root.join("app-data");
    let managed = root.join("managed-campaign");
    let incomplete = root.join("incomplete-campaign");
    let output = root.join("exports/task23-diagnostics.zip");
    let incomplete_output = root.join("exports/incomplete-diagnostics.zip");
    let stale_output = root.join("exports/stale-diagnostics.zip");
    fs::create_dir(&cache).unwrap();
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    publish_managed_install(&app_data, &managed, "install-task23");
    publish_started_campaign(&app_data, &incomplete, &cache, "install-incomplete-diagnostics");
    let canonical_managed = managed.canonicalize().unwrap();
    let engine = Arc::new(FakeBridgeEngine::new(root.join("bg1"), root.join("bg2")));
    let system = Arc::new(RecordingBridgeSystem::default());
    let bridge = NativeBridge::with_engine_and_system(
        recipe,
        "recommended",
        cache,
        app_data,
        engine,
        system.clone(),
    );

    assert_eq!(
        bridge
            .export_diagnostics("not-consulted-after-cancel", None)
            .unwrap(),
        None
    );
    let exported = bridge
        .export_diagnostics("install-task23", Some(output.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(PathBuf::from(exported.path), output);
    bridge
        .export_diagnostics(
            "install-incomplete-diagnostics",
            Some(incomplete_output.clone()),
        )
        .unwrap()
        .unwrap();
    fs::remove_dir_all(&managed).unwrap();
    bridge
        .export_diagnostics("install-task23", Some(stale_output.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(
        bridge
            .export_diagnostics("unknown-install", Some(root.join("exports/unknown.zip")))
            .unwrap_err()
            .code,
        "managed_install_unknown"
    );
    assert_eq!(
        system.diagnostics.lock().unwrap().as_slice(),
        &[
            (canonical_managed.clone(), output),
            (incomplete.canonicalize().unwrap(), incomplete_output),
            (canonical_managed, stale_output),
        ]
    );

    bridge.open_manual_source("manual-fixture").unwrap();
    assert_eq!(
        system.urls.lock().unwrap().as_slice(),
        &["https://example.invalid/manual-fixture"]
    );
    assert_eq!(
        bridge
            .open_manual_source("not-a-recipe-artifact")
            .unwrap_err()
            .code,
        "manual_source_unknown"
    );
}

fn signed_recipe_candidate(
    version: &str,
    supersedes: &str,
    minimum_app_version: &str,
) -> (Vec<u8>, Vec<u8>, Vec<u8>, TrustedPublicKey) {
    let keys = KeyPair::generate_unencrypted_keypair().unwrap();
    let ledger = format!(
        "schema = 1\nrecipe_id = \"chriz-bg-collection\"\nversion = \"{version}\"\npublished_at = \"2026-09-04T00:00:00Z\"\nminimum_app_version = \"{minimum_app_version}\"\nsupersedes = \"{supersedes}\"\n\n[[changes]]\nid = \"npc-kit\"\ntitle = \"NPC kit correction\"\nsummary = \"Uses the corrected kit in a newly built campaign.\"\nsave_applicability = \"before-npc-join\"\nurgency = \"recommended\"\ncondition_note = \"This is authored guidance; the installer did not inspect the save.\"\ncovers = [\"feature:npc-kit\"]\n"
    );
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::default());
    writer.start_file("collection.toml", options).unwrap();
    writer
        .write_all(b"schema=2\ngame_build='2.7.3.0'\n")
        .unwrap();
    writer
        .start_file(format!("releases/v{version}/ledger.toml"), options)
        .unwrap();
    writer.write_all(ledger.as_bytes()).unwrap();
    let payload = writer.finish().unwrap().into_inner();
    let envelope = serde_json::to_vec(&RecipeEnvelope {
        recipe_id: "chriz-bg-collection".to_owned(),
        version: version.to_owned(),
        payload_sha256: sha256_bytes(&payload),
        key_id: "test-update-key".to_owned(),
        minimum_app_version: minimum_app_version.to_owned(),
        published_at: "2026-09-04T00:00:00Z".to_owned(),
    })
    .unwrap();
    let signature = sign(
        None,
        &keys.sk,
        Cursor::new(&envelope),
        Some("task 24 update fixture"),
        None,
    )
    .unwrap()
    .into_string()
    .into_bytes();
    let public = TrustedPublicKey {
        key_id: "test-update-key".to_owned(),
        minisign_public_key: keys.pk.to_box().unwrap().to_string(),
    };
    (payload, envelope, signature, public)
}

#[test]
fn signed_recipe_update_projection_preserves_authored_guidance_and_app_prerequisite() {
    let (payload, envelope, signature, public) =
        signed_recipe_candidate("0.1.0-alpha.2", "0.1.0-alpha.1", "0.1.0-alpha.1");
    let manager = RecipeUpdateManager::new(
        RecipeTrustStore::new([public]).unwrap(),
        "0.1.0-alpha.1",
        "0.1.0-alpha.1",
    )
    .unwrap();
    let response = manager.check_candidate(&payload, &envelope, &signature);
    assert_eq!(response.state, RecipeUpdateState::Available);
    assert_eq!(response.available_version.as_deref(), Some("0.1.0-alpha.2"));
    assert_eq!(response.changes.len(), 1);
    assert!(response.changes[0]
        .condition_note
        .as_deref()
        .unwrap()
        .contains("did not inspect the save"));

    let (payload, envelope, signature, public) =
        signed_recipe_candidate("0.1.0-alpha.2", "0.1.0-alpha.1", "0.2.0");
    let manager = RecipeUpdateManager::new(
        RecipeTrustStore::new([public]).unwrap(),
        "0.1.0-alpha.1",
        "0.1.0-alpha.1",
    )
    .unwrap();
    let response = manager.check_candidate(&payload, &envelope, &signature);
    assert_eq!(response.state, RecipeUpdateState::RequiresApp);
    assert_eq!(response.minimum_app_version.as_deref(), Some("0.2.0"));
}

#[test]
fn invalid_and_replayed_recipe_updates_keep_the_trusted_recipe() {
    let (payload, envelope, mut signature, public) =
        signed_recipe_candidate("0.1.0-alpha.2", "0.1.0-alpha.1", "0.1.0-alpha.1");
    let manager = RecipeUpdateManager::new(
        RecipeTrustStore::new([public]).unwrap(),
        "0.1.0-alpha.1",
        "0.1.0-alpha.1",
    )
    .unwrap();
    let signed_line = signature.iter().position(|byte| *byte == b'\n').unwrap() + 5;
    signature[signed_line] = if signature[signed_line] == b'A' {
        b'B'
    } else {
        b'A'
    };
    let invalid = manager.check_candidate(&payload, &envelope, &signature);
    assert_eq!(invalid.state, RecipeUpdateState::Invalid);
    assert_eq!(manager.staged_version(), None);

    let (payload, envelope, signature, public) =
        signed_recipe_candidate("0.1.0-alpha.1", "0.1.0-alpha.0", "0.1.0-alpha.1");
    let manager = RecipeUpdateManager::new(
        RecipeTrustStore::new([public]).unwrap(),
        "0.1.0-alpha.1",
        "0.1.0-alpha.1",
    )
    .unwrap();
    let replayed = manager.check_candidate(&payload, &envelope, &signature);
    assert_eq!(replayed.state, RecipeUpdateState::Replayed);
    assert_eq!(manager.staged_version(), None);
}

#[test]
fn update_replacement_is_deferred_while_a_build_is_active() {
    let (_temp, recipe) = recipe_with_profiles();
    let cache = recipe.parent().unwrap().join("cache");
    fs::create_dir(&cache).unwrap();
    let bg1 = recipe.parent().unwrap().join("clean-bg1");
    let bg2 = recipe.parent().unwrap().join("clean-bg2");
    fs::create_dir(&bg1).unwrap();
    fs::create_dir(&bg2).unwrap();
    let destination = recipe.parent().unwrap().join("campaign");
    let engine = Arc::new(FakeBridgeEngine::new(bg1, bg2));
    let bridge = NativeBridge::with_engine(recipe, "recommended", &cache, engine.clone());
    let discovery = bridge.discover_games().unwrap();
    let review = bridge
        .freeze_review(
            "Chriz Easy BG",
            &NormalizedSelection {
                platform: "windows".to_owned(),
                features: Default::default(),
                inputs: Default::default(),
            },
            &destination,
            &discovery.selected_bg1_id,
            &discovery.selected_bg2_id,
        )
        .unwrap();
    let (_tx, rx) = mpsc::channel::<RunEventEnvelope>();
    bridge.start_build(&review.review_token, |_| {}).unwrap();
    while bridge.active_run_count() == 0 {
        std::thread::yield_now();
    }

    let error = bridge.ensure_update_idle().unwrap_err();
    assert_eq!(error.code, "update_deferred_build_active");
    engine.release_install();
    drop(rx);
}

#[test]
fn missing_release_keys_and_endpoints_are_clear_non_destructive_update_state() {
    let (_temp, recipe) = recipe_with_profiles();
    let root = recipe.parent().unwrap().to_path_buf();
    let cache = root.join("cache");
    let app_data = root.join("app-data");
    let managed = root.join("managed-campaign");
    fs::create_dir(&cache).unwrap();
    publish_managed_install(&app_data, &managed, "install-update-state");
    let bridge = NativeBridge::with_engine_and_system(
        recipe,
        "recommended",
        cache,
        app_data,
        Arc::new(FakeBridgeEngine::new(root.join("bg1"), root.join("bg2"))),
        Arc::new(RecordingBridgeSystem::default()),
    );

    let update = bridge.unconfigured_update_center("0.1.0-alpha.1").unwrap();
    assert_eq!(update.network_state, "unconfigured");
    assert_eq!(update.application.state, "unavailable");
    assert_eq!(update.recipe.state, RecipeUpdateState::Unavailable);
    assert!(update.application.detail.contains("release-time"));
    assert!(update.recipe.detail.contains("trusted recipe"));
    assert_eq!(update.managed_copies.len(), 1);
    assert_eq!(update.managed_copies[0].install_id, "install-update-state");
    assert_eq!(update.managed_copies[0].state, "unknown");
}
