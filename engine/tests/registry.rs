use std::path::{Path, PathBuf};

use bg_engine::registry::{
    InstallAvailability, ManagedInstallRecord, ManagedInstallRegistry, REGISTRY_SCHEMA_VERSION,
};
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
