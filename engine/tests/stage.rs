use std::fs::{self, File, FileTimes};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use bg_engine::games::GameRole;
use bg_engine::stage::{
    finalize_game_identity, propose_save_identity, read_engine_name, reserve_save_identity,
    stage_bgee_sod, stage_game_copy, stage_game_copy_with_observer, stage_initial_games,
    verify_profile_settings, DocumentsLocator, ManagedLayout, ReservedSaveIdentity, StageObserver,
    SystemDocuments,
};

fn source_tree(root: &Path, name: &str) -> PathBuf {
    let source = root.join(name);
    fs::create_dir_all(source.join("data/empty")).unwrap();
    fs::write(source.join("data/base.bif"), b"base payload").unwrap();
    fs::write(
        source.join("engine.lua"),
        b"engine_name = \"Store Game\"\r\nengine_mode = 1\r\n",
    )
    .unwrap();
    source
}

fn layout(temp: &tempfile::TempDir) -> (ManagedLayout, PathBuf, PathBuf) {
    let bg1 = source_tree(temp.path(), "source-bg1");
    let bg2 = source_tree(temp.path(), "source-bg2");
    let layout = ManagedLayout::prepare(temp.path().join("managed"), &bg1, &bg2).unwrap();
    (layout, bg1, bg2)
}

#[test]
fn creates_the_exact_managed_layout_outside_both_sources() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, bg1, bg2) = layout(&temp);

    assert_eq!(layout.bg1_source(), fs::canonicalize(bg1).unwrap());
    assert_eq!(layout.bg2_source(), fs::canonicalize(bg2).unwrap());
    assert_eq!(layout.bg1_root(), layout.root().join("bg1"));
    assert_eq!(layout.game_root(), layout.root().join("game"));
    assert_eq!(layout.state_root(), layout.root().join(".chriz"));
    assert!(layout.bg1_root().is_dir());
    assert!(layout.game_root().is_dir());
    assert!(layout.state_root().is_dir());
}

#[test]
fn rejects_equal_or_nested_source_and_managed_roots() {
    let temp = tempfile::tempdir().unwrap();
    let source = source_tree(temp.path(), "source");
    let other = source_tree(temp.path(), "other");

    for managed in [
        source.clone(),
        source.join("managed"),
        temp.path().to_path_buf(),
    ] {
        let error = ManagedLayout::prepare(&managed, &source, &other).unwrap_err();
        assert!(
            error.to_string().contains("unsafe staging relationship"),
            "{error}"
        );
    }
}

#[test]
fn textual_path_prefixes_are_not_treated_as_containment() {
    let temp = tempfile::tempdir().unwrap();
    let source = source_tree(temp.path(), "game");
    let other = source_tree(temp.path(), "other");
    let managed = temp.path().join("game-copy");

    let layout = ManagedLayout::prepare(&managed, &source, &other).unwrap();
    assert_eq!(layout.root(), fs::canonicalize(managed).unwrap());
}

#[cfg(windows)]
#[test]
fn case_variants_of_the_same_windows_path_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let source = source_tree(temp.path(), "SourceGame");
    let other = source_tree(temp.path(), "OtherGame");
    let variant = PathBuf::from(source.to_string_lossy().to_ascii_uppercase());

    let error = ManagedLayout::prepare(variant, &source, &other).unwrap_err();
    assert!(
        error.to_string().contains("unsafe staging relationship"),
        "{error}"
    );
}

#[test]
fn copies_regular_files_and_empty_directories_without_sharing_file_identity() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, bg1, _) = layout(&temp);

    let report = stage_game_copy(&layout, GameRole::BgeeSod).unwrap();

    assert_eq!(report.copied_files, 2);
    assert_eq!(report.reused_files, 0);
    assert_eq!(
        fs::read(layout.bg1_root().join("data/base.bif")).unwrap(),
        b"base payload"
    );
    assert!(layout.bg1_root().join("data/empty").is_dir());

    fs::write(
        layout.bg1_root().join("data/base.bif"),
        b"stage-only change",
    )
    .unwrap();
    assert_eq!(
        fs::read(bg1.join("data/base.bif")).unwrap(),
        b"base payload"
    );
}

#[test]
fn resume_reuses_verified_files_and_replaces_a_corrupt_file() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    stage_game_copy(&layout, GameRole::BgeeSod).unwrap();
    fs::write(layout.bg1_root().join("data/base.bif"), b"corrupt").unwrap();

    let report = stage_game_copy(&layout, GameRole::BgeeSod).unwrap();

    assert_eq!(report.copied_files, 1);
    assert_eq!(report.reused_files, 1);
    assert_eq!(
        fs::read(layout.bg1_root().join("data/base.bif")).unwrap(),
        b"base payload"
    );
}

#[test]
fn a_destination_hardlink_to_the_source_is_replaced_with_an_independent_file() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, bg1, _) = layout(&temp);
    fs::create_dir_all(layout.bg1_root().join("data")).unwrap();
    fs::hard_link(
        bg1.join("data/base.bif"),
        layout.bg1_root().join("data/base.bif"),
    )
    .unwrap();

    let report = stage_game_copy(&layout, GameRole::BgeeSod).unwrap();

    assert_eq!(report.copied_files, 2);
    assert_eq!(report.reused_files, 0);
    fs::write(
        layout.bg1_root().join("data/base.bif"),
        b"stage-only change",
    )
    .unwrap();
    assert_eq!(
        fs::read(bg1.join("data/base.bif")).unwrap(),
        b"base payload"
    );
}

#[test]
fn a_destination_hardlink_to_an_unrelated_file_is_replaced_before_reuse() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let outside = temp.path().join("outside.bin");
    fs::write(&outside, b"base payload").unwrap();
    fs::create_dir_all(layout.bg1_root().join("data")).unwrap();
    fs::hard_link(&outside, layout.bg1_root().join("data/base.bif")).unwrap();

    let report = stage_game_copy(&layout, GameRole::BgeeSod).unwrap();

    assert_eq!(report.copied_files, 2);
    assert_eq!(report.reused_files, 0);
    fs::write(layout.bg1_root().join("data/base.bif"), b"stage-only").unwrap();
    assert_eq!(fs::read(outside).unwrap(), b"base payload");
}

#[test]
fn a_stale_scratch_hardlink_is_unlinked_before_copying() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let outside = temp.path().join("outside.bin");
    fs::write(&outside, b"outside payload").unwrap();
    let scratch = layout.state_root().join("staging/bg1/data");
    fs::create_dir_all(&scratch).unwrap();
    fs::hard_link(&outside, scratch.join("base.bif.chriz-part")).unwrap();

    stage_game_copy(&layout, GameRole::BgeeSod).unwrap();

    assert_eq!(fs::read(outside).unwrap(), b"outside payload");
}

#[test]
fn unexpected_destination_entries_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    fs::write(layout.bg1_root().join("not-from-source.txt"), b"unknown").unwrap();

    let error = stage_game_copy(&layout, GameRole::BgeeSod).unwrap_err();
    assert!(error.to_string().contains("not-from-source.txt"), "{error}");
}

#[cfg(any(unix, windows))]
#[test]
fn linked_source_entries_are_rejected_without_being_followed() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, bg1, _) = layout(&temp);
    let outside = temp.path().join("outside.bin");
    fs::write(&outside, b"outside").unwrap();
    let link = bg1.join("linked.bin");
    if let Err(error) = symlink_file(&outside, &link) {
        if cfg!(windows)
            && matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
            )
        {
            return;
        }
        panic!("could not create test symlink: {error}");
    }

    let error = stage_game_copy(&layout, GameRole::BgeeSod).unwrap_err();
    assert!(error.to_string().contains("linked.bin"), "{error}");
    assert!(!layout.bg1_root().join("linked.bin").exists());
}

#[cfg(any(unix, windows))]
#[test]
fn linked_source_directories_are_rejected_without_being_followed() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, bg1, _) = layout(&temp);
    let outside = temp.path().join("outside-directory");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("outside.bin"), b"outside").unwrap();
    let link = bg1.join("linked-directory");
    if let Err(error) = symlink_directory(&outside, &link) {
        if cfg!(windows)
            && matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
            )
        {
            return;
        }
        panic!("could not create test directory symlink: {error}");
    }

    let error = stage_game_copy(&layout, GameRole::BgeeSod).unwrap_err();
    assert!(error.to_string().contains("linked-directory"), "{error}");
    assert!(!layout.bg1_root().join("linked-directory").exists());
}

#[cfg(any(unix, windows))]
#[test]
fn linked_destination_entries_are_rejected_without_being_followed() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let outside = temp.path().join("outside.bin");
    fs::write(&outside, b"outside").unwrap();
    let destination = layout.bg1_root().join("engine.lua");
    if let Err(error) = symlink_file(&outside, &destination) {
        if cfg!(windows)
            && matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
            )
        {
            return;
        }
        panic!("could not create test symlink: {error}");
    }

    let error = stage_game_copy(&layout, GameRole::BgeeSod).unwrap_err();
    assert!(error.to_string().contains("engine.lua"), "{error}");
    assert_eq!(fs::read(outside).unwrap(), b"outside");
}

struct MutateSourceAfterFirstFile {
    path: PathBuf,
    mutated: bool,
}

impl StageObserver for MutateSourceAfterFirstFile {
    fn file_published(&mut self, _relative: &Path) -> io::Result<()> {
        if !self.mutated {
            fs::write(&self.path, b"store update")?;
            self.mutated = true;
        }
        Ok(())
    }
}

#[test]
fn a_source_change_during_copy_discards_the_partial_stage() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, bg1, _) = layout(&temp);
    let mut observer = MutateSourceAfterFirstFile {
        path: bg1.join("engine.lua"),
        mutated: false,
    };

    let error =
        stage_game_copy_with_observer(&layout, GameRole::BgeeSod, &mut observer).unwrap_err();

    assert!(
        error.to_string().contains("changed during staging"),
        "{error}"
    );
    assert!(!layout.bg1_root().exists());
}

struct RetimestampSourceAfterFirstFile {
    path: PathBuf,
    changed: bool,
}

struct RemoveSourceAfterFirstFile {
    path: PathBuf,
    removed: bool,
}

impl StageObserver for RemoveSourceAfterFirstFile {
    fn file_published(&mut self, _relative: &Path) -> io::Result<()> {
        if !self.removed {
            fs::remove_file(&self.path)?;
            self.removed = true;
        }
        Ok(())
    }
}

#[test]
fn a_source_that_disappears_during_copy_discards_the_partial_stage() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, bg1, _) = layout(&temp);
    let mut observer = RemoveSourceAfterFirstFile {
        path: bg1.join("engine.lua"),
        removed: false,
    };

    let error =
        stage_game_copy_with_observer(&layout, GameRole::BgeeSod, &mut observer).unwrap_err();

    assert!(
        error.to_string().contains("changed during staging"),
        "{error}"
    );
    assert!(!layout.bg1_root().exists());
}

impl StageObserver for RetimestampSourceAfterFirstFile {
    fn file_published(&mut self, _relative: &Path) -> io::Result<()> {
        if !self.changed {
            let file = File::options().write(true).open(&self.path)?;
            let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(4_000_000_000);
            file.set_times(FileTimes::new().set_modified(timestamp))?;
            self.changed = true;
        }
        Ok(())
    }
}

#[test]
fn a_source_metadata_change_during_copy_discards_the_partial_stage() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, bg1, _) = layout(&temp);
    let mut observer = RetimestampSourceAfterFirstFile {
        path: bg1.join("engine.lua"),
        changed: false,
    };

    let error =
        stage_game_copy_with_observer(&layout, GameRole::BgeeSod, &mut observer).unwrap_err();

    assert!(
        error.to_string().contains("changed during staging"),
        "{error}"
    );
    assert!(!layout.bg1_root().exists());
}

#[derive(Debug, Clone)]
struct FixedDocuments(PathBuf);

impl DocumentsLocator for FixedDocuments {
    fn documents_dir(&self) -> io::Result<PathBuf> {
        Ok(self.0.clone())
    }
}

#[test]
fn missing_documents_reports_parent_check_without_creating_a_guessed_location() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, bg1, bg2) = layout(&temp);
    let documents = temp.path().join("offline-user-volume/Documents");
    let source_before = fs::read(bg1.join("engine.lua")).unwrap();

    let error = reserve_save_identity(
        &FixedDocuments(documents.clone()),
        &layout,
        "Fixture",
        "install-one",
    )
    .unwrap_err();

    assert_eq!(
        error.folder_operation(),
        Some(("check Documents folder", documents.as_path()))
    );
    let bg_engine::error::EngineError::Io { path, source } = error.primary_cause() else {
        panic!("expected original missing-path error: {error}");
    };
    assert_eq!(path, &documents);
    assert_eq!(source.kind(), io::ErrorKind::NotFound);
    assert_eq!(error.folder_failure_code(), Some("folder_unavailable"));
    assert!(!documents.exists());
    assert!(!documents.parent().unwrap().exists());
    assert_eq!(fs::read_dir(layout.bg1_root()).unwrap().count(), 0);
    assert_eq!(fs::read_dir(layout.game_root()).unwrap().count(), 0);
    assert_eq!(fs::read(bg1.join("engine.lua")).unwrap(), source_before);
    assert_eq!(fs::read(bg2.join("engine.lua")).unwrap(), source_before);
}

#[test]
fn denied_documents_lookup_is_distinct_from_a_missing_folder() {
    struct DeniedDocuments;
    impl DocumentsLocator for DeniedDocuments {
        fn documents_dir(&self) -> io::Result<PathBuf> {
            #[cfg(windows)]
            let code = 5;
            #[cfg(not(windows))]
            let code = 13;
            Err(io::Error::from_raw_os_error(code))
        }
    }

    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let error =
        reserve_save_identity(&DeniedDocuments, &layout, "Fixture", "install-one").unwrap_err();
    assert_eq!(
        error.folder_operation(),
        Some((
            "resolve Documents folder",
            Path::new("<Windows Documents folder>")
        ))
    );
    let bg_engine::error::EngineError::Io { source, .. } = error.primary_cause() else {
        panic!("expected original denied-access error: {error}");
    };
    assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(error.folder_failure_code(), Some("folder_access_denied"));
    assert!(source.raw_os_error().is_some());
    assert_eq!(fs::read_dir(layout.bg1_root()).unwrap().count(), 0);
}

#[test]
fn documents_path_that_is_a_file_is_not_reported_as_missing() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("Documents");
    fs::write(&documents, b"keep this unrelated file").unwrap();

    let error = reserve_save_identity(
        &FixedDocuments(documents.clone()),
        &layout,
        "Fixture",
        "install-one",
    )
    .unwrap_err();
    assert_eq!(
        error.folder_operation(),
        Some(("check Documents folder", documents.as_path()))
    );
    assert!(matches!(
        error.primary_cause(),
        bg_engine::error::EngineError::GameIdentity { .. }
    ));
    assert!(error.to_string().contains("not a directory"));
    assert_eq!(error.folder_failure_code(), Some("folder_conflict"));
    assert_eq!(fs::read(documents).unwrap(), b"keep this unrelated file");
}

#[test]
fn relocated_documents_reservation_uses_only_the_configured_parent() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("relocated-user-data/profile-documents");
    fs::create_dir_all(&documents).unwrap();
    let identity = reserve_save_identity(
        &FixedDocuments(documents.clone()),
        &layout,
        "Fixture",
        "install-one",
    )
    .unwrap();
    assert_eq!(identity.save_root.parent(), Some(documents.as_path()));
    assert!(identity.owner_marker().is_file());
    assert!(!temp.path().join("Documents").exists());
    assert_eq!(
        reserve_save_identity(
            &FixedDocuments(documents),
            &layout,
            "Fixture",
            "install-one"
        )
        .unwrap(),
        identity
    );
}

#[test]
fn disappearing_reserved_child_reports_revalidation_without_staging_the_games() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let identity = reserve_save_identity(
        &FixedDocuments(documents),
        &layout,
        "Fixture",
        "install-one",
    )
    .unwrap();
    fs::remove_file(identity.owner_marker()).unwrap();
    fs::remove_dir(&identity.save_root).unwrap();

    let error = stage_bgee_sod(&layout, &identity).unwrap_err();
    assert_eq!(
        error.folder_operation(),
        Some(("revalidate save folder", identity.save_root.as_path()))
    );
    let bg_engine::error::EngineError::Io { path, source } = error.primary_cause() else {
        panic!("expected original missing-child error: {error}");
    };
    assert_eq!(path, &identity.save_root);
    assert_eq!(source.kind(), io::ErrorKind::NotFound);
    assert_eq!(fs::read_dir(layout.bg1_root()).unwrap().count(), 0);
    assert_eq!(fs::read_dir(layout.game_root()).unwrap().count(), 0);
}

#[test]
fn unavailable_source_is_not_mistaken_for_a_destination_creation_failure() {
    let temp = tempfile::tempdir().unwrap();
    let missing_bg1 = temp.path().join("disconnected-source");
    let bg2 = source_tree(temp.path(), "source-bg2");
    let managed = temp.path().join("managed");

    let error = ManagedLayout::prepare(&managed, &missing_bg1, &bg2).unwrap_err();
    assert_eq!(
        error.folder_operation(),
        Some(("check BG1 source folder", missing_bg1.as_path()))
    );
    assert!(matches!(
        error.primary_cause(),
        bg_engine::error::EngineError::Io { source, .. }
            if source.kind() == io::ErrorKind::NotFound
    ));
    assert!(!managed.exists());
}

#[cfg(windows)]
#[test]
fn locked_ownership_marker_preserves_the_actual_windows_read_failure() {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;

    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let locator = FixedDocuments(documents);
    let identity = reserve_save_identity(&locator, &layout, "Fixture", "install-one").unwrap();
    let original_marker = fs::read(identity.owner_marker()).unwrap();
    let lock = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(identity.owner_marker())
        .unwrap();

    let error = reserve_save_identity(&locator, &layout, "Fixture", "install-one").unwrap_err();
    assert_eq!(
        error.folder_operation(),
        Some((
            "read save-folder ownership marker",
            identity.owner_marker().as_path()
        ))
    );
    let bg_engine::error::EngineError::Io { source, .. } = error.primary_cause() else {
        panic!("expected original file-sharing failure: {error}");
    };
    assert_eq!(source.raw_os_error(), Some(32));
    assert_eq!(error.folder_failure_code(), Some("folder_in_use"));
    drop(lock);
    assert_eq!(fs::read(identity.owner_marker()).unwrap(), original_marker);
    assert_eq!(
        reserve_save_identity(&locator, &layout, "Fixture", "install-one").unwrap(),
        identity
    );
}

#[test]
fn save_identity_is_normalized_unique_and_reserved_for_one_managed_install() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let locator = FixedDocuments(documents.clone());

    let first =
        reserve_save_identity(&locator, &layout, "  Chriz:EET / 2.7 🎲 ", "install-one").unwrap();
    let resumed =
        reserve_save_identity(&locator, &layout, "  Chriz:EET / 2.7 🎲 ", "install-one").unwrap();
    let second =
        propose_save_identity(&documents, &layout, "  Chriz:EET / 2.7 🎲 ", "install-two").unwrap();

    assert_eq!(first, resumed);
    assert!(first.engine_name.starts_with("Chriz EET 2.7 - "));
    assert!(first.engine_name.is_ascii());
    assert!(!first
        .engine_name
        .chars()
        .any(|character| "<>:\"/\\|?*".contains(character)));
    assert_eq!(first.save_root, documents.join(&first.engine_name));
    assert!(first.save_root.is_dir());
    assert!(first.owner_marker().is_file());
    assert_ne!(first.engine_name, second.engine_name);
    assert_ne!(first.save_root, documents.join("Store Game"));
}

#[test]
fn unmanaged_or_differently_owned_save_root_collision_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let proposed = propose_save_identity(&documents, &layout, "Chriz EET", "install-one").unwrap();
    fs::create_dir(&proposed.save_root).unwrap();

    let error = reserve_save_identity(
        &FixedDocuments(documents.clone()),
        &layout,
        "Chriz EET",
        "install-one",
    )
    .unwrap_err();
    assert!(error.to_string().contains("save root collision"), "{error}");

    fs::remove_dir(&proposed.save_root).unwrap();
    let reserved = reserve_save_identity(
        &FixedDocuments(documents.clone()),
        &layout,
        "Chriz EET",
        "install-one",
    )
    .unwrap();
    fs::write(
        reserved.owner_marker(),
        b"{\"schema\":1,\"install_id\":\"someone-else\"}",
    )
    .unwrap();
    let error = reserve_save_identity(
        &FixedDocuments(documents),
        &layout,
        "Chriz EET",
        "install-one",
    )
    .unwrap_err();
    assert!(error.to_string().contains("save root collision"), "{error}");
}

#[test]
fn a_reserved_identity_cannot_be_used_for_a_different_managed_layout() {
    let temp = tempfile::tempdir().unwrap();
    let bg1 = source_tree(temp.path(), "source-bg1");
    let bg2 = source_tree(temp.path(), "source-bg2");
    let first_layout = ManagedLayout::prepare(temp.path().join("managed-one"), &bg1, &bg2).unwrap();
    let second_layout =
        ManagedLayout::prepare(temp.path().join("managed-two"), &bg1, &bg2).unwrap();
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let identity = reserve_save_identity(
        &FixedDocuments(documents),
        &first_layout,
        "Chriz EET",
        "install-one",
    )
    .unwrap();

    let error = stage_initial_games(&second_layout, &identity).unwrap_err();

    assert!(
        error.to_string().contains("different managed layout"),
        "{error}"
    );
    assert_eq!(fs::read_dir(second_layout.bg1_root()).unwrap().count(), 0);
    assert_eq!(fs::read_dir(second_layout.game_root()).unwrap().count(), 0);
}

#[test]
fn initial_staging_patches_only_bg1_and_finalization_repatches_bg2() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let identity = reserve_save_identity(
        &FixedDocuments(documents),
        &layout,
        "Chriz EET 2.7 RC",
        "install-one",
    )
    .unwrap();

    let reports = stage_initial_games(&layout, &identity).unwrap();

    assert_eq!(reports.bg1.copied_files, 2);
    assert_eq!(reports.bg2.copied_files, 2);
    assert_eq!(
        read_engine_name(layout.bg1_root()).unwrap(),
        identity.engine_name
    );
    assert_eq!(read_engine_name(layout.game_root()).unwrap(), "Store Game");

    fs::write(
        layout.game_root().join("engine.lua"),
        b"engine_name = \"EET overwrote this\"\r\nengine_mode = 1\r\n",
    )
    .unwrap();
    finalize_game_identity(&layout, &identity).unwrap();
    assert_eq!(
        read_engine_name(layout.game_root()).unwrap(),
        identity.engine_name
    );
}

#[test]
fn single_role_bg1_staging_patches_identity_without_touching_bg2() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let identity = reserve_save_identity(
        &FixedDocuments(documents),
        &layout,
        "Chriz EET 2.7 RC",
        "install-one",
    )
    .unwrap();

    let report = stage_bgee_sod(&layout, &identity).unwrap();

    assert_eq!(report.copied_files, 2);
    assert_eq!(
        read_engine_name(layout.bg1_root()).unwrap(),
        identity.engine_name
    );
    assert_eq!(fs::read_dir(layout.game_root()).unwrap().count(), 0);
}

#[test]
fn a_bg1_identity_patch_failure_discards_the_unisolated_stage() {
    let temp = tempfile::tempdir().unwrap();
    let bg1 = source_tree(temp.path(), "source-bg1");
    let bg2 = source_tree(temp.path(), "source-bg2");
    fs::write(
        bg1.join("engine.lua"),
        b"engine_name = \"One\"\r\nengine_name = \"Two\"\r\n",
    )
    .unwrap();
    let layout = ManagedLayout::prepare(temp.path().join("managed"), &bg1, &bg2).unwrap();
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let identity = reserve_save_identity(
        &FixedDocuments(documents),
        &layout,
        "Chriz EET",
        "install-one",
    )
    .unwrap();

    let error = stage_initial_games(&layout, &identity).unwrap_err();

    assert!(error.to_string().contains("exactly one"), "{error}");
    assert!(!layout.bg1_root().exists());
    assert_eq!(fs::read_dir(layout.game_root()).unwrap().count(), 0);
}

#[test]
fn engine_name_patch_preserves_other_lines_and_rejects_ambiguous_assignments() {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let identity = reserve_save_identity(
        &FixedDocuments(documents),
        &layout,
        "Chriz EET",
        "install-one",
    )
    .unwrap();
    stage_game_copy(&layout, GameRole::Bg2ee).unwrap();
    fs::write(
        layout.game_root().join("engine.lua"),
        b"engine_name = \"One\"\r\nengine_mode = 1\r\nengine_name = \"Two\"\r\n",
    )
    .unwrap();

    let error = finalize_game_identity(&layout, &identity).unwrap_err();
    assert!(error.to_string().contains("exactly one"), "{error}");
    assert_eq!(
        fs::read(layout.game_root().join("engine.lua")).unwrap(),
        b"engine_name = \"One\"\r\nengine_mode = 1\r\nengine_name = \"Two\"\r\n"
    );
}

#[cfg(windows)]
#[test]
fn system_documents_uses_the_windows_known_folder() {
    let documents = SystemDocuments.documents_dir().unwrap();
    assert!(documents.is_absolute());
    assert!(documents.is_dir());
}

fn interval_profile_fixture(
    component: u32,
) -> (tempfile::TempDir, ManagedLayout, ReservedSaveIdentity) {
    let temp = tempfile::tempdir().unwrap();
    let (layout, _, _) = layout(&temp);
    let documents = temp.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let identity = reserve_save_identity(
        &FixedDocuments(documents),
        &layout,
        "Interval regression",
        "interval-test",
    )
    .unwrap();
    stage_initial_games(&layout, &identity).unwrap();
    fs::write(
        layout.game_root().join("WeiDU.log"),
        format!("~CDTWEAKS/SETUP-CDTWEAKS.TP2~ #0 #{component} // Installed interval saves\n"),
    )
    .unwrap();
    (temp, layout, identity)
}

#[test]
fn final_identity_seeds_interval_settings_in_isolated_profile_not_shared_eet_profile() {
    let (temp, layout, identity) = interval_profile_fixture(3354);
    let shared = temp
        .path()
        .join("Documents/Baldur's Gate - Enhanced Edition Trilogy");
    fs::create_dir(&shared).unwrap();
    let unrelated = b"SetPrivateProfileString('Graphics','Width','1234')\n";
    fs::write(shared.join("Baldur.lua"), unrelated).unwrap();
    fs::write(
        layout.game_root().join("engine.lua"),
        "engine_name = \"Baldur's Gate - Enhanced Edition Trilogy\"\n",
    )
    .unwrap();
    let log_before = fs::read(layout.game_root().join("WeiDU.log")).unwrap();
    assert!(verify_profile_settings(&layout, &identity).is_err());

    finalize_game_identity(&layout, &identity).unwrap();

    assert_eq!(
        read_engine_name(layout.game_root()).unwrap(),
        identity.engine_name
    );
    assert_eq!(fs::read_to_string(identity.save_root.join("Baldur.lua")).unwrap(),
        "SetPrivateProfileString('Script','IntervalSaveEnabled','1')\nSetPrivateProfileString('Script','IntervalSaveCombat','0')\n");
    assert_eq!(fs::read(shared.join("Baldur.lua")).unwrap(), unrelated);
    assert_eq!(
        fs::read(layout.game_root().join("WeiDU.log")).unwrap(),
        log_before
    );
    verify_profile_settings(&layout, &identity).unwrap();
}

#[test]
fn interval_defaults_preserve_preferences_bom_crlf_and_repeated_finalization() {
    let (_temp, layout, identity) = interval_profile_fixture(3354);
    let path = identity.save_root.join("Baldur.lua");
    let original = "\u{feff}SetPrivateProfileString('Graphics','Width','1234')\r\nSetPrivateProfileString( \"Script\", \"IntervalSaveEnabled\", \"0\" ); -- intentional\r\n";
    fs::write(&path, original).unwrap();
    finalize_game_identity(&layout, &identity).unwrap();
    let expected =
        format!("{original}SetPrivateProfileString('Script','IntervalSaveCombat','0')\r\n");
    assert_eq!(fs::read(&path).unwrap(), expected.as_bytes());
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    finalize_game_identity(&layout, &identity).unwrap();
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert_eq!(fs::read(&path).unwrap(), expected.as_bytes());
}

#[test]
fn interval_defaults_cover_all_fixed_presets_but_not_unselected_or_custom_components() {
    for component in 3350..=3357 {
        let (_temp, layout, identity) = interval_profile_fixture(component);
        finalize_game_identity(&layout, &identity).unwrap();
        verify_profile_settings(&layout, &identity).unwrap();
        assert!(identity.save_root.join("Baldur.lua").is_file());
    }
    for component in [40, 3358] {
        let (_temp, layout, identity) = interval_profile_fixture(component);
        finalize_game_identity(&layout, &identity).unwrap();
        verify_profile_settings(&layout, &identity).unwrap();
        assert!(!identity.save_root.join("Baldur.lua").exists());
    }
}

#[test]
fn interval_defaults_ignore_uninstalled_comments_and_other_mod_component_numbers() {
    let (_temp, layout, identity) = interval_profile_fixture(3354);
    fs::write(layout.game_root().join("WeiDU.log"),
        "// Recently Uninstalled: ~CDTWEAKS/SETUP-CDTWEAKS.TP2~ #0 #3354\n~other/other.tp2~ #0 #3354\n").unwrap();
    finalize_game_identity(&layout, &identity).unwrap();
    assert!(!identity.save_root.join("Baldur.lua").exists());
}

#[test]
fn interval_defaults_do_not_treat_commented_or_other_section_values_as_enabled() {
    let (_temp, layout, identity) = interval_profile_fixture(3354);
    let path = identity.save_root.join("Baldur.lua");
    let original = "-- SetPrivateProfileString('Script','IntervalSaveEnabled','0')\nSetPrivateProfileString('Other','IntervalSaveCombat','1')";
    fs::write(&path, original).unwrap();
    finalize_game_identity(&layout, &identity).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), format!("{original}\nSetPrivateProfileString('Script','IntervalSaveEnabled','1')\nSetPrivateProfileString('Script','IntervalSaveCombat','0')\n"));
}

#[test]
fn interval_defaults_report_unrecognized_config_without_overwriting_it() {
    for original in [
        "SetPrivateProfileString('Script','IntervalSaveEnabled', unknownValue)\n",
        "SetPrivateProfileString('Script','IntervalSaveEnabled','maybe')\n",
    ] {
        let (_temp, layout, identity) = interval_profile_fixture(3354);
        let path = identity.save_root.join("Baldur.lua");
        fs::write(&path, original).unwrap();
        let error = finalize_game_identity(&layout, &identity).unwrap_err();
        assert!(
            error.to_string().contains("interval-save settings"),
            "{error}"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    }
}

#[test]
fn final_profile_verification_detects_missing_settings_without_rewriting_preferences() {
    let (_temp, layout, identity) = interval_profile_fixture(3354);
    finalize_game_identity(&layout, &identity).unwrap();
    let path = identity.save_root.join("Baldur.lua");
    let changed = "SetPrivateProfileString('Script','IntervalSaveEnabled','0')\n";
    fs::write(&path, changed).unwrap();
    assert!(verify_profile_settings(&layout, &identity).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), changed);
}

#[test]
fn interval_defaults_reject_linked_settings_without_changing_the_other_file() {
    let (temp, layout, identity) = interval_profile_fixture(3354);
    let other = temp.path().join("other-profile.lua");
    fs::write(&other, "-- unrelated profile\n").unwrap();
    fs::hard_link(&other, identity.save_root.join("Baldur.lua")).unwrap();
    assert!(finalize_game_identity(&layout, &identity).is_err());
    assert_eq!(
        fs::read_to_string(&other).unwrap(),
        "-- unrelated profile\n"
    );
}

#[cfg(unix)]
fn symlink_file(source: &Path, destination: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(source, destination)
}

#[cfg(unix)]
fn symlink_directory(source: &Path, destination: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(source, destination)
}

#[cfg(windows)]
fn symlink_file(source: &Path, destination: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(source, destination)
}

#[cfg(windows)]
fn symlink_directory(source: &Path, destination: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_dir(source, destination)
}
