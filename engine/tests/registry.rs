use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::registry::{
    inspect_campaign_availability, inspect_install_availability, CampaignAvailability,
    InstallAvailability, ManagedInstallRecord, ManagedInstallRegistry, RegistryError,
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
        recipe_id: None,
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
            bg1: Some("11".repeat(32)),
            bg2: "22".repeat(32),
            standalone_role: None,
        },
        artifact_identities: Vec::new(),
        tool_identities: Vec::new(),
        staged_bg1: Some(managed_root.join("bg1")),
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
    inspect_install_availability(&expected).unwrap();
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
    assert!(matches!(
        inspect_install_availability(&expected),
        Err(RegistryError::Io { path, source })
            if path == managed && source.kind() == std::io::ErrorKind::NotFound
    ));
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
    assert!(matches!(
        inspect_install_availability(&expected),
        Err(RegistryError::Io { path, source })
            if path == expected.launch_path && source.kind() == std::io::ErrorKind::NotFound
    ));
}

#[test]
fn changed_receipt_stays_stale_and_reports_the_exact_receipt_path() {
    let (_temp, _app_data, managed, registry) = setup();
    let mut expected = record(&managed);
    expected.receipt_sha256 = sha256_bytes(b"immutable receipt");
    registry.publish(&expected).unwrap();
    let receipt_path = managed.join(".chriz/install-receipt.json");
    std::fs::write(&receipt_path, b"changed receipt").unwrap();

    assert_eq!(
        registry.list().unwrap()[0].availability,
        InstallAvailability::Stale
    );
    let reason = inspect_install_availability(&expected)
        .unwrap_err()
        .to_string();
    assert!(reason.contains("receipt SHA-256 mismatch"), "{reason}");
    assert!(
        reason.contains(&receipt_path.display().to_string()),
        "{reason}"
    );
    assert!(
        reason.contains(&sha256_bytes(b"changed receipt")),
        "{reason}"
    );
    assert_eq!(std::fs::read(&receipt_path).unwrap(), b"changed receipt");
}

#[test]
fn inspection_distinguishes_a_non_directory_root_and_an_escaping_launcher() {
    let (temp, _app_data, managed, _registry) = setup();
    let mut expected = record(&managed);
    expected.receipt_sha256 = sha256_bytes(b"immutable receipt");
    let outside = temp.path().join("outside.exe");
    std::fs::write(&outside, b"outside launcher").unwrap();
    expected.launch_path = outside.clone();
    assert!(matches!(
        inspect_install_availability(&expected),
        Err(RegistryError::UnsafePath { path, message })
            if path == outside && message.contains("outside managed root")
    ));

    expected.managed_root = outside.clone();
    assert!(matches!(
        inspect_install_availability(&expected),
        Err(RegistryError::UnsafePath { path, message })
            if path == outside && message.contains("direct non-symlink directory")
    ));
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
    assert_eq!(
        inspect_campaign_availability(&listed[0].record).unwrap(),
        None
    );
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
    let corrupt_card = listed
        .iter()
        .find(|card| card.record.install_id == "install-corrupt")
        .unwrap();
    let reason = inspect_campaign_availability(&corrupt_card.record)
        .unwrap_err()
        .to_string();
    assert!(reason.contains("campaign ledger"), "{reason}");
    assert!(
        reason.contains(&corrupt.managed_root.display().to_string()),
        "{reason}"
    );
}

/// A download cache deleted by hand or by the uninstaller does not make an interrupted build
/// look moved or changed: resuming creates it again.
#[test]
fn a_deleted_download_cache_keeps_a_started_campaign_resumable() {
    let temp = TempDir::new().unwrap();
    let registry = ManagedInstallRegistry::open_or_create(&temp.path().join("app-data")).unwrap();
    let (created, _store) = create_started_campaign(temp.path(), "install-no-cache");
    registry.publish_campaign(&created).unwrap();
    std::fs::rename(&created.cache_root, temp.path().join("deleted-cache")).unwrap();

    let listed = registry.list_campaigns().unwrap();

    assert_eq!(listed[0].availability, CampaignAvailability::Resumable);
    assert_eq!(
        inspect_campaign_availability(&listed[0].record).unwrap(),
        None
    );
    assert!(!created.cache_root.exists());
}

/// Only a cache folder that no longer exists is tolerated. Anything else in its place cannot
/// be used or recreated, so the campaign still cannot be resumed.
#[test]
fn a_download_cache_replaced_by_a_file_still_blocks_resuming() {
    let temp = TempDir::new().unwrap();
    let registry = ManagedInstallRegistry::open_or_create(&temp.path().join("app-data")).unwrap();
    let (created, _store) = create_started_campaign(temp.path(), "install-file-cache");
    registry.publish_campaign(&created).unwrap();
    std::fs::rename(&created.cache_root, temp.path().join("moved-cache")).unwrap();
    std::fs::write(&created.cache_root, b"not a folder").unwrap();

    assert_eq!(
        registry.list_campaigns().unwrap()[0].availability,
        CampaignAvailability::Stale
    );
    let reason = inspect_campaign_availability(&registry.list_campaigns().unwrap()[0].record)
        .unwrap_err()
        .to_string();
    assert!(reason.contains("cache root"), "{reason}");
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
    let seal = inspect_campaign_availability(&listed[0].record)
        .unwrap()
        .unwrap();
    assert_eq!(seal.step_id, "stage:bg1");
    assert_eq!(seal.attempt, 1);
    assert_eq!(seal.detail, "target evidence changed");
}

#[test]
fn campaign_inspection_reports_the_mismatched_identity_field() {
    let temp = TempDir::new().unwrap();
    let registry = ManagedInstallRegistry::open_or_create(&temp.path().join("app-data")).unwrap();
    let (created, _store) = create_started_campaign(temp.path(), "install-identity");
    registry.publish_campaign(&created).unwrap();
    let original = registry.list_campaigns().unwrap().remove(0).record;

    let mut changed = original.clone();
    changed.install_id = "install-other".to_owned();
    let reason = inspect_campaign_availability(&changed)
        .unwrap_err()
        .to_string();
    assert!(reason.contains("campaign install id differs"), "{reason}");
    changed = original;
    changed.recipe_sha256 = "ff".repeat(32);
    let reason = inspect_campaign_availability(&changed)
        .unwrap_err()
        .to_string();
    assert!(
        reason.contains("campaign recipe SHA-256 differs"),
        "{reason}"
    );
    assert_eq!(
        registry.list_campaigns().unwrap()[0].availability,
        CampaignAvailability::Resumable
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

fn create_named_campaign(root: &Path, install_id: &str, name: &str) -> CampaignCreated {
    let managed = root.join(format!("named-{install_id}"));
    let cache = root.join(format!("cache-{install_id}"));
    std::fs::create_dir_all(&managed).unwrap();
    std::fs::create_dir_all(&cache).unwrap();
    let mut created = started_campaign(
        &managed.canonicalize().unwrap(),
        &cache.canonicalize().unwrap(),
        install_id,
    );
    created.recipe_payload =
        serde_json::to_vec(&serde_json::json!({ "display_name": name })).unwrap();
    created.recipe_payload_sha256 = sha256_bytes(&created.recipe_payload);
    SessionStore::create(
        &created.managed_root,
        SessionEvent::Created(Box::new(created.clone())),
    )
    .unwrap();
    created
}

#[test]
fn name_defaults_include_failed_missing_and_completed_entries() {
    let (temp, _app_data, managed, registry) = setup();
    assert_eq!(registry.suggested_display_name().unwrap(), "Chriz Easy BG");
    let mut completed = record(&managed);
    completed.display_name = "  chriz EASY bg  ".to_owned();
    completed.receipt_sha256 = sha256_bytes(b"immutable receipt");
    registry.publish(&completed).unwrap();
    assert_eq!(
        registry.suggested_display_name().unwrap(),
        "Chriz Easy BG (2)"
    );
    let campaign = create_named_campaign(temp.path(), "failed-two", "Chriz Easy BG (2)");
    registry
        .publish_campaign_named(&campaign, "Chriz Easy BG (2)")
        .unwrap();
    let store = SessionStore::open(&campaign.managed_root).unwrap();
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
            detail: "fixture failure".to_owned(),
        })
        .unwrap();
    std::fs::rename(&managed, temp.path().join("moved-completed")).unwrap();
    assert_eq!(
        registry.suggested_display_name().unwrap(),
        "Chriz Easy BG (3)"
    );
    assert_eq!(
        registry.display_name("failed-two").unwrap(),
        "Chriz Easy BG (2)"
    );
}

#[test]
fn name_validation_trims_compares_case_insensitively_and_rejects_invalid_input() {
    use bg_engine::registry::names::normalize_display_name;
    let (temp, _app_data, _managed, registry) = setup();
    let campaign = create_named_campaign(temp.path(), "custom", "My adventure");
    registry
        .publish_campaign_named(&campaign, "  My adventure  ")
        .unwrap();
    assert_eq!(registry.display_name("custom").unwrap(), "My adventure");
    assert_eq!(
        registry
            .validate_display_name("  Different  ", None)
            .unwrap(),
        "Different"
    );
    assert!(
        matches!(registry.validate_display_name(" MY ADVENTURE ", None), Err(RegistryError::DuplicateName { install_id, .. }) if install_id == "custom")
    );
    assert_eq!(
        registry
            .validate_display_name(" MY ADVENTURE ", Some("custom"))
            .unwrap(),
        "MY ADVENTURE"
    );
    for invalid in [
        "",
        "   ",
        ".",
        "..",
        "line\nbreak",
        "Bad/name",
        "Bad\\name",
        "Name?",
        "Name:",
        "<name>",
    ] {
        assert!(
            matches!(
                normalize_display_name(invalid),
                Err(RegistryError::InvalidName(_))
            ),
            "{invalid:?}"
        );
    }
    assert!(normalize_display_name(&"a".repeat(81)).is_err());
    assert_eq!(
        normalize_display_name(&"界".repeat(80))
            .unwrap()
            .chars()
            .count(),
        80
    );
    assert!(registry
        .validate_display_name("Name", Some("unknown"))
        .is_err());
}

#[test]
fn name_comparison_handles_non_ascii_case() {
    let (temp, _app_data, _managed, registry) = setup();
    let campaign = create_named_campaign(temp.path(), "unicode", "Älven adventure");
    registry
        .publish_campaign_named(&campaign, "Älven adventure")
        .unwrap();
    assert!(matches!(
        registry.validate_display_name("älven ADVENTURE", None),
        Err(RegistryError::DuplicateName { .. })
    ));
}

#[test]
fn name_survives_failure_reopen_completion_and_display_only_rename() {
    let (temp, app_data, _managed, registry) = setup();
    let campaign = create_named_campaign(temp.path(), "lifecycle", "First name");
    let pointer = registry
        .publish_campaign_named(&campaign, "First name")
        .unwrap();
    let pointer_before = std::fs::read(&pointer).unwrap();
    let ledger = campaign.managed_root.join(".chriz/ledger/0000000000.json");
    let ledger_before = std::fs::read(&ledger).unwrap();
    assert_eq!(
        registry
            .rename_display_name("lifecycle", "  Better name  ")
            .unwrap(),
        "Better name"
    );
    assert_eq!(
        ManagedInstallRegistry::open_or_create(&app_data)
            .unwrap()
            .display_name("lifecycle")
            .unwrap(),
        "Better name"
    );
    registry
        .publish_campaign_named(&campaign, "First name")
        .unwrap();
    assert_eq!(registry.display_name("lifecycle").unwrap(), "Better name");
    assert_eq!(std::fs::read(&pointer).unwrap(), pointer_before);
    assert_eq!(std::fs::read(&ledger).unwrap(), ledger_before);

    std::fs::create_dir_all(campaign.managed_root.join("game")).unwrap();
    std::fs::write(
        campaign.managed_root.join("game/InfinityLoader.exe"),
        b"launcher",
    )
    .unwrap();
    std::fs::write(
        campaign.managed_root.join(".chriz/install-receipt.json"),
        b"receipt",
    )
    .unwrap();
    let profile = temp.path().join("saved-profile");
    std::fs::create_dir_all(&profile).unwrap();
    std::fs::write(profile.join("save.sav"), b"untouched save").unwrap();
    let mut completed = record(&campaign.managed_root);
    completed.install_id = campaign.install_id.clone();
    completed.display_name = "First name".to_owned();
    completed.managed_save_root = profile.clone();
    completed.receipt_sha256 = sha256_bytes(b"receipt");
    let completed_path = registry.publish(&completed).unwrap();
    let completed_before = std::fs::read(&completed_path).unwrap();
    registry
        .rename_display_name("lifecycle", "Final name")
        .unwrap();
    assert_eq!(registry.display_name("lifecycle").unwrap(), "Final name");
    assert_eq!(registry.list().unwrap()[0].record, completed);
    assert_eq!(std::fs::read(&completed_path).unwrap(), completed_before);
    assert_eq!(
        std::fs::read(campaign.managed_root.join(".chriz/install-receipt.json")).unwrap(),
        b"receipt"
    );
    assert_eq!(
        std::fs::read(profile.join("save.sav")).unwrap(),
        b"untouched save"
    );
    assert!(campaign.managed_root.is_dir());
    assert!(!campaign.managed_root.with_file_name("Final name").exists());
}

#[test]
fn legacy_campaign_names_are_read_from_frozen_payload_without_a_migration() {
    let (temp, app_data, _managed, registry) = setup();
    let campaign = create_named_campaign(temp.path(), "legacy", "Legacy chosen name");
    registry.publish_campaign(&campaign).unwrap();
    assert_eq!(
        registry.display_name("legacy").unwrap(),
        "Legacy chosen name"
    );
    assert!(!app_data.join("managed-install-names").exists());
    assert!(matches!(
        registry.validate_display_name("LEGACY CHOSEN NAME", None),
        Err(RegistryError::DuplicateName { .. })
    ));
}

#[test]
fn legacy_duplicate_names_remain_readable_but_new_duplicates_are_rejected() {
    let (temp, _app_data, _managed, registry) = setup();
    let first = create_named_campaign(temp.path(), "duplicate-first", "Repeated");
    let second = create_named_campaign(temp.path(), "duplicate-second", "Repeated");
    registry.publish_campaign(&first).unwrap();
    registry.publish_campaign(&second).unwrap();
    assert_eq!(
        registry.display_name(&first.install_id).unwrap(),
        "Repeated"
    );
    assert_eq!(
        registry.display_name(&second.install_id).unwrap(),
        "Repeated"
    );
    // Legacy resume does not force a rename or break the campaign.
    registry.publish_campaign_named(&first, "Repeated").unwrap();
    let third = create_named_campaign(temp.path(), "duplicate-third", "Repeated");
    assert!(matches!(
        registry.publish_campaign_named(&third, "Repeated"),
        Err(RegistryError::DuplicateName { .. })
    ));
    assert!(matches!(
        registry.rename_display_name(&first.install_id, "repeated"),
        Err(RegistryError::DuplicateName { .. })
    ));
    registry
        .rename_display_name(&first.install_id, "Unique now")
        .unwrap();
    assert_eq!(
        registry.display_name(&second.install_id).unwrap(),
        "Repeated"
    );
}

#[test]
fn unavailable_name_persists_and_forgetting_releases_it_without_deleting_other_metadata() {
    let (temp, app_data, _managed, registry) = setup();
    let campaign = create_named_campaign(temp.path(), "missing-named", "Chriz Easy BG");
    registry
        .publish_campaign_named(&campaign, "Chriz Easy BG")
        .unwrap();
    std::fs::rename(&campaign.managed_root, temp.path().join("moved-named")).unwrap();
    registry
        .rename_display_name(&campaign.install_id, "Unavailable but named")
        .unwrap();
    assert_eq!(
        registry.display_name(&campaign.install_id).unwrap(),
        "Unavailable but named"
    );
    registry
        .forget_missing(&campaign.install_id, &campaign.managed_root)
        .unwrap();
    assert!(registry.display_name(&campaign.install_id).is_err());
    assert_eq!(
        registry
            .validate_display_name("Unavailable but named", None)
            .unwrap(),
        "Unavailable but named"
    );
    assert!(app_data
        .join("managed-install-names/missing-named.json")
        .is_file());
}

#[test]
fn competing_campaign_names_have_exactly_one_winner() {
    use std::sync::{Arc, Barrier};
    let (temp, _app_data, _managed, registry) = setup();
    let first = create_named_campaign(temp.path(), "race-first", "Same name");
    let second = create_named_campaign(temp.path(), "race-second", "same NAME");
    let start = Arc::new(Barrier::new(2));
    let jobs: Vec<_> = [(first, "Same name"), (second, "same NAME")]
        .into_iter()
        .map(|(created, name)| {
            let registry = registry.clone();
            let start = Arc::clone(&start);
            std::thread::spawn(move || {
                start.wait();
                registry.publish_campaign_named(&created, name)
            })
        })
        .collect();
    let results: Vec<_> = jobs.into_iter().map(|job| job.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(RegistryError::DuplicateName { .. })))
            .count(),
        1
    );
    assert_eq!(registry.list_campaigns().unwrap().len(), 1);
}

#[test]
fn competing_renames_preserve_the_loser_and_have_exactly_one_winner() {
    use std::sync::{Arc, Barrier};
    let (temp, _app_data, _managed, registry) = setup();
    for (id, name) in [("rename-first", "First"), ("rename-second", "Second")] {
        let created = create_named_campaign(temp.path(), id, name);
        registry.publish_campaign_named(&created, name).unwrap();
    }
    let start = Arc::new(Barrier::new(2));
    let jobs: Vec<_> = ["rename-first", "rename-second"]
        .into_iter()
        .map(|id| {
            let registry = registry.clone();
            let start = Arc::clone(&start);
            std::thread::spawn(move || {
                start.wait();
                registry.rename_display_name(id, "New shared name")
            })
        })
        .collect();
    let results: Vec<_> = jobs.into_iter().map(|job| job.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(RegistryError::DuplicateName { .. })))
            .count(),
        1
    );
    let names: Vec<_> = ["rename-first", "rename-second"]
        .into_iter()
        .map(|id| registry.display_name(id).unwrap())
        .collect();
    assert_eq!(
        names
            .iter()
            .filter(|name| *name == "New shared name")
            .count(),
        1
    );
    assert_eq!(
        names
            .iter()
            .filter(|name| matches!(name.as_str(), "First" | "Second"))
            .count(),
        1
    );
}

#[test]
fn name_metadata_rejects_foreign_identity_corruption_and_path_traversal() {
    let (temp, app_data, _managed, registry) = setup();
    let campaign = create_named_campaign(temp.path(), "guarded-name", "Guarded name");
    registry
        .publish_campaign_named(&campaign, "Guarded name")
        .unwrap();
    let path = app_data.join("managed-install-names/guarded-name.json");
    let before = std::fs::read(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&before).unwrap();
    value["managed_root"] = serde_json::json!(temp.path().join("foreign-root"));
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(registry
        .display_name("guarded-name")
        .unwrap_err()
        .to_string()
        .contains("does not match"));
    assert!(registry
        .rename_display_name("guarded-name", "Overwrite?")
        .is_err());
    std::fs::write(&path, b"malformed JSON").unwrap();
    assert!(matches!(
        registry.display_name("guarded-name"),
        Err(RegistryError::Json { .. })
    ));
    assert!(registry
        .rename_display_name("../outside", "Safe name")
        .is_err());
    assert!(registry.display_name("../outside").is_err());
}

#[test]
fn name_publication_rejects_an_id_reassigned_to_a_different_root() {
    let (temp, _app_data, _managed, registry) = setup();
    let first = create_named_campaign(temp.path(), "bound-name", "Bound name");
    registry
        .publish_campaign_named(&first, "Bound name")
        .unwrap();
    let second_parent = temp.path().join("other");
    std::fs::create_dir(&second_parent).unwrap();
    let second = create_named_campaign(&second_parent, "bound-name", "Other name");
    assert!(matches!(
        registry.publish_campaign_named(&second, "Other name"),
        Err(RegistryError::DuplicateRoot { .. })
    ));
    assert_eq!(registry.display_name("bound-name").unwrap(), "Bound name");
}

#[test]
fn recipe_id_is_written_only_when_present_and_round_trips() {
    let (_temp, _app_data, managed, registry) = setup();
    let mut legacy = record(&managed);
    legacy.receipt_sha256 = bg_engine::digest::sha256_bytes(b"immutable receipt");
    let legacy_path = registry.publish(&legacy).unwrap();
    let legacy_text = std::fs::read_to_string(&legacy_path).unwrap();
    assert!(!legacy_text.contains("recipe_id"), "{legacy_text}");

    let mut standalone = legacy.clone();
    standalone.install_id = "install-iwd".to_owned();
    standalone.recipe_id = Some("chriz-easy-iwd".to_owned());
    let path = registry.publish(&standalone).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"recipe_id\": \"chriz-easy-iwd\""), "{text}");
    let listed = registry.list().unwrap();
    assert!(listed.iter().any(|card| card.record == standalone));
    assert!(listed.iter().any(|card| card.record == legacy));
}

#[test]
fn an_unsafe_recipe_id_is_rejected() {
    let (_temp, _app_data, managed, registry) = setup();
    for recipe_id in ["", "..", "chriz/iwd", "chriz iwd"] {
        let mut invalid = record(&managed);
        invalid.receipt_sha256 = bg_engine::digest::sha256_bytes(b"immutable receipt");
        invalid.recipe_id = Some(recipe_id.to_owned());

        assert!(registry.publish(&invalid).is_err(), "{recipe_id:?}");
    }
}
