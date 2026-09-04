use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::registry::{
    CampaignAvailability, InstallAvailability, ManagedInstallRecord, ManagedInstallRegistry,
    REGISTRY_SCHEMA_VERSION,
};
use bg_engine::resolve::InstallPlan;
use bg_engine::session::{CampaignCreated, SessionEvent, SessionStore, SourceGameFingerprints};
use tempfile::TempDir;

fn record(managed_root: &Path) -> ManagedInstallRecord {
    ManagedInstallRecord {
        schema_version: REGISTRY_SCHEMA_VERSION,
        install_id: "install-001".to_owned(),
        display_name: "Chriz BG Collection Alpha".to_owned(),
        managed_root: managed_root.to_path_buf(),
        recipe_version: "2026.09.03-alpha.1".to_owned(),
        recipe_sha256: "11".repeat(32),
        engine_name: "Chriz BG Collection - abc123".to_owned(),
        managed_save_root: managed_root.join("saves"),
        launch_path: managed_root.join("game/InfinityLoader.exe"),
        receipt_sha256: "22".repeat(32),
        completed_at_millis: 42,
    }
}

fn setup() -> (TempDir, PathBuf, PathBuf, ManagedInstallRegistry) {
    let temp = TempDir::new().unwrap();
    let app_data = temp.path().join("app-data");
    let managed = temp.path().join("managed");
    std::fs::create_dir_all(managed.join(".chriz")).unwrap();
    std::fs::write(
        managed.join(".chriz/install-receipt.json"),
        b"immutable receipt",
    )
    .unwrap();
    std::fs::create_dir_all(managed.join("game")).unwrap();
    std::fs::write(
        managed.join("game/InfinityLoader.exe"),
        b"verified launcher",
    )
    .unwrap();
    let registry = ManagedInstallRegistry::open_or_create(&app_data).unwrap();
    (temp, app_data, managed, registry)
}

fn started_campaign(managed_root: &Path, cache_root: &Path, install_id: &str) -> CampaignCreated {
    let selection = NormalizedSelection {
        platform: "windows".to_owned(),
        features: BTreeMap::new(),
        inputs: BTreeMap::new(),
    };
    let plan = InstallPlan { runs: Vec::new() };
    let payload = b"PK\x03\x04frozen campaign recipe".to_vec();
    let envelope = br#"{"kind":"frozen-campaign"}"#.to_vec();
    CampaignCreated {
        install_id: install_id.to_owned(),
        attempt_id: format!("attempt-{install_id}"),
        managed_root: managed_root.to_path_buf(),
        cache_root: cache_root.to_path_buf(),
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
        staged_bg1: managed_root.join("bg1"),
        staged_bg2: managed_root.join("game"),
    }
}

fn create_started_campaign(root: &Path, install_id: &str) -> (CampaignCreated, SessionStore) {
    let managed_root = root.join(format!("managed-{install_id}"));
    let cache_root = root.join(format!("cache-{install_id}"));
    std::fs::create_dir(&managed_root).unwrap();
    std::fs::create_dir(&cache_root).unwrap();
    let managed_root = managed_root.canonicalize().unwrap();
    let cache_root = cache_root.canonicalize().unwrap();
    let created = started_campaign(&managed_root, &cache_root, install_id);
    let store = SessionStore::create(
        &managed_root,
        SessionEvent::Created(Box::new(created.clone())),
    )
    .unwrap();
    (created, store)
}

#[test]
fn stores_one_immutable_file_per_install_and_lists_live_targets() {
    let (_temp, app_data, managed, registry) = setup();
    let mut expected = record(&managed);
    expected.receipt_sha256 = bg_engine::digest::sha256_bytes(b"immutable receipt");

    let path = registry.publish(&expected).unwrap();

    assert_eq!(path, app_data.join("managed-installs/install-001.json"));
    assert!(path.is_file());
    assert!(!app_data.join("managed-installs.json").exists());
    let listed = registry.list().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].record, expected);
    assert_eq!(listed[0].availability, InstallAvailability::Available);
    assert_eq!(registry.publish(&expected).unwrap(), path);
}

#[test]
fn a_missing_or_moved_target_stays_as_a_stale_card() {
    let (_temp, _app_data, managed, registry) = setup();
    let mut expected = record(&managed);
    expected.receipt_sha256 = bg_engine::digest::sha256_bytes(b"immutable receipt");
    registry.publish(&expected).unwrap();
    let moved = managed.with_file_name("moved-managed");
    std::fs::rename(&managed, moved).unwrap();

    let listed = registry.list().unwrap();

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].record, expected);
    assert_eq!(listed[0].availability, InstallAvailability::Stale);
}

#[test]
fn duplicate_ids_with_different_roots_are_rejected_without_data_loss() {
    let (_temp, _app_data, managed, registry) = setup();
    let mut original = record(&managed);
    original.receipt_sha256 = bg_engine::digest::sha256_bytes(b"immutable receipt");
    let path = registry.publish(&original).unwrap();
    let before = std::fs::read(&path).unwrap();
    let other = managed.with_file_name("other-managed");
    std::fs::create_dir_all(other.join(".chriz")).unwrap();
    std::fs::write(
        other.join(".chriz/install-receipt.json"),
        b"immutable receipt",
    )
    .unwrap();
    std::fs::create_dir_all(other.join("game")).unwrap();
    std::fs::write(other.join("game/InfinityLoader.exe"), b"verified launcher").unwrap();
    let mut duplicate = original.clone();
    duplicate.managed_root = other.clone();
    duplicate.launch_path = other.join("game/InfinityLoader.exe");

    let error = registry.publish(&duplicate).unwrap_err();

    assert!(
        error.to_string().contains("different managed root"),
        "{error}"
    );
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[test]
fn missing_launch_executable_marks_an_install_stale() {
    let (_temp, _app_data, managed, registry) = setup();
    let mut expected = record(&managed);
    expected.receipt_sha256 = bg_engine::digest::sha256_bytes(b"immutable receipt");
    registry.publish(&expected).unwrap();
    std::fs::remove_file(&expected.launch_path).unwrap();

    let listed = registry.list().unwrap();

    assert_eq!(listed[0].availability, InstallAvailability::Stale);
}

#[test]
fn launch_path_cannot_escape_the_managed_root_with_parent_segments() {
    let (_temp, _app_data, managed, registry) = setup();
    let mut unsafe_record = record(&managed);
    unsafe_record.receipt_sha256 = bg_engine::digest::sha256_bytes(b"immutable receipt");
    unsafe_record.launch_path = managed.join("game/../../outside.exe");

    let error = registry.publish(&unsafe_record).unwrap_err();

    assert!(error.to_string().contains("launch path"), "{error}");
}

#[test]
fn install_identifier_cannot_be_a_relative_path_component() {
    let (_temp, _app_data, managed, registry) = setup();
    let mut unsafe_record = record(&managed);
    unsafe_record.install_id = "..".to_owned();
    unsafe_record.receipt_sha256 = bg_engine::digest::sha256_bytes(b"immutable receipt");

    let error = registry.publish(&unsafe_record).unwrap_err();

    assert!(error.to_string().contains("path-safe"), "{error}");
}

#[test]
fn publishes_a_create_once_started_campaign_and_replays_it_as_resumable() {
    let temp = TempDir::new().unwrap();
    let app_data = temp.path().join("app-data");
    let registry = ManagedInstallRegistry::open_or_create(&app_data).unwrap();
    let (created, _store) = create_started_campaign(temp.path(), "install-started");

    let path = registry.publish_campaign(&created).unwrap();

    assert_eq!(
        path,
        app_data.join("managed-campaigns/install-started.json")
    );
    assert_eq!(registry.publish_campaign(&created).unwrap(), path);
    let listed = registry.list_campaigns().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].record.install_id, "install-started");
    assert_eq!(listed[0].record.managed_root, created.managed_root);
    assert_eq!(
        listed[0].record.recipe_sha256,
        created.recipe_payload_sha256
    );
    assert_eq!(listed[0].availability, CampaignAvailability::Resumable);
}

#[test]
fn moved_or_corrupt_started_campaigns_remain_visible_but_stale() {
    let temp = TempDir::new().unwrap();
    let registry = ManagedInstallRegistry::open_or_create(&temp.path().join("app-data")).unwrap();
    let (moved, _store) = create_started_campaign(temp.path(), "install-moved");
    let (corrupt, _store) = create_started_campaign(temp.path(), "install-corrupt");
    registry.publish_campaign(&moved).unwrap();
    registry.publish_campaign(&corrupt).unwrap();
    std::fs::rename(&moved.managed_root, temp.path().join("moved-away")).unwrap();
    std::fs::write(
        corrupt.managed_root.join(".chriz/ledger/0000000000.json"),
        b"not a ledger record",
    )
    .unwrap();

    let listed = registry.list_campaigns().unwrap();

    assert_eq!(listed.len(), 2);
    assert!(listed
        .iter()
        .all(|card| card.availability == CampaignAvailability::Stale));
}

#[test]
fn a_fresh_copy_seal_is_visible_but_never_resumable() {
    let temp = TempDir::new().unwrap();
    let registry = ManagedInstallRegistry::open_or_create(&temp.path().join("app-data")).unwrap();
    let (created, store) = create_started_campaign(temp.path(), "install-sealed");
    store
        .append(SessionEvent::StepStarted {
            step_id: "stage:bg1".to_owned(),
            attempt: 1,
        })
        .unwrap();
    store
        .append(SessionEvent::FreshCopyRequired {
            step_id: "stage:bg1".to_owned(),
            attempt: 1,
            detail: "target evidence changed".to_owned(),
        })
        .unwrap();
    registry.publish_campaign(&created).unwrap();

    let listed = registry.list_campaigns().unwrap();

    assert_eq!(
        listed[0].availability,
        CampaignAvailability::FreshCopyRequired
    );
}

#[test]
fn a_started_install_id_cannot_be_reassigned_to_another_valid_ledger() {
    let temp = TempDir::new().unwrap();
    let registry = ManagedInstallRegistry::open_or_create(&temp.path().join("app-data")).unwrap();
    let first_root = temp.path().join("first");
    let second_root = temp.path().join("second");
    std::fs::create_dir(&first_root).unwrap();
    std::fs::create_dir(&second_root).unwrap();
    let cache = temp.path().join("cache");
    std::fs::create_dir(&cache).unwrap();
    let first = started_campaign(
        &first_root.canonicalize().unwrap(),
        &cache.canonicalize().unwrap(),
        "install-conflict",
    );
    let second = started_campaign(
        &second_root.canonicalize().unwrap(),
        &cache.canonicalize().unwrap(),
        "install-conflict",
    );
    SessionStore::create(
        &first.managed_root,
        SessionEvent::Created(Box::new(first.clone())),
    )
    .unwrap();
    SessionStore::create(
        &second.managed_root,
        SessionEvent::Created(Box::new(second.clone())),
    )
    .unwrap();
    registry.publish_campaign(&first).unwrap();

    let error = registry.publish_campaign(&second).unwrap_err();

    assert!(
        error.to_string().contains("different managed root"),
        "{error}"
    );
}
