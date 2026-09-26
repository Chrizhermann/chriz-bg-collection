//! Progress is observational: a byte count must not replace backup verification.
use crate::{operation_progress, patches::files};
use std::{cell::RefCell, fs};

fn byte_counts(events: &[operation_progress::OperationProgress]) -> Vec<(u64, u64)> {
    events
        .iter()
        .filter_map(|event| match (&event.completed, &event.total) {
            (Some(completed), Some(total)) => Some((
                completed.parse().expect("decimal completed bytes"),
                total.parse().expect("decimal total bytes"),
            )),
            (None, None) => None,
            _ => panic!("progress count has only one bound"),
        })
        .collect()
}

#[test]
fn byte_events_keep_full_u64_precision_and_stage_events_have_no_counts() {
    let events = RefCell::new(Vec::new());
    let sink = |event| events.borrow_mut().push(event);
    operation_progress::stage(&sink, "checking");
    operation_progress::bytes(&sink, "backup-game", u64::MAX - 1, u64::MAX);
    let events = events.borrow();
    assert_eq!(events[0].stage, "checking");
    assert!(events[0].completed.is_none());
    assert!(events[0].total.is_none());
    assert!(events[0].unit.is_none());
    assert_eq!(events[1].completed.as_deref(), Some("18446744073709551614"));
    assert_eq!(events[1].total.as_deref(), Some("18446744073709551615"));
    assert_eq!(events[1].unit.as_deref(), Some("bytes"));
    let serialized = serde_json::to_value(&events[1]).unwrap();
    assert_eq!(serialized["total"], "18446744073709551615");
}

#[test]
fn successful_backup_reports_exact_monotonic_bytes_and_preserves_contents() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let backup = temp.path().join("backup");
    fs::create_dir_all(source.join("nested/empty-dir")).unwrap();
    fs::write(source.join("readme.txt"), b"hello").unwrap();
    fs::write(source.join("empty.dat"), []).unwrap();
    let large = vec![0xa7; 65_537];
    fs::write(source.join("nested/data.bin"), &large).unwrap();
    let original_inventory = files::inventory(&source, false).unwrap();
    let events = RefCell::new(Vec::new());

    files::copy_tree_with_progress(&source, &backup, false, "backup-game", &|event| {
        events.borrow_mut().push(event);
    })
    .unwrap();

    let events = events.borrow();
    assert!(events.iter().all(|event| event.stage == "backup-game"));
    assert!(events[0].completed.is_none());
    let counts = byte_counts(&events);
    assert_eq!(counts.first(), Some(&(0, 65_542)));
    assert_eq!(counts.last(), Some(&(65_542, 65_542)));
    assert!(counts
        .iter()
        .all(|(done, total)| *total == 65_542 && done <= total));
    assert!(counts.windows(2).all(|pair| pair[0].0 <= pair[1].0));
    assert_eq!(fs::read(backup.join("readme.txt")).unwrap(), b"hello");
    assert_eq!(
        fs::read(backup.join("empty.dat")).unwrap(),
        Vec::<u8>::new()
    );
    assert_eq!(fs::read(backup.join("nested/data.bin")).unwrap(), large);
    assert!(backup.join("nested/empty-dir").is_dir());
    assert_eq!(
        files::inventory(&source, false).unwrap(),
        original_inventory
    );
    fs::write(backup.join("readme.txt"), b"backup-only edit").unwrap();
    assert_eq!(fs::read(source.join("readme.txt")).unwrap(), b"hello");
}

#[test]
fn empty_profile_backup_has_zero_real_bytes_not_invented_work() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("profile");
    let backup = temp.path().join("profile-backup");
    fs::create_dir_all(source.join("empty-save-directory")).unwrap();
    let events = RefCell::new(Vec::new());
    files::copy_tree_with_progress(&source, &backup, true, "backup-saves", &|event| {
        events.borrow_mut().push(event);
    })
    .unwrap();
    assert!(backup.join("empty-save-directory").is_dir());
    let events = events.borrow();
    assert!(events.iter().all(|event| event.stage == "backup-saves"));
    let counts = byte_counts(&events);
    assert!(!counts.is_empty());
    assert!(counts.iter().all(|counts| *counts == (0, 0)));
}

#[test]
fn existing_destination_is_not_overwritten_or_reported_complete() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let backup = temp.path().join("backup");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&backup).unwrap();
    fs::write(source.join("file"), b"source").unwrap();
    fs::write(backup.join("file"), b"existing backup").unwrap();
    let events = RefCell::new(Vec::new());
    let result = files::copy_tree_with_progress(&source, &backup, false, "backup-game", &|event| {
        events.borrow_mut().push(event)
    });
    assert!(result.unwrap_err().contains("already exists"));
    assert!(byte_counts(&events.borrow()).is_empty());
    assert_eq!(fs::read(backup.join("file")).unwrap(), b"existing backup");
    assert_eq!(fs::read(source.join("file")).unwrap(), b"source");
}

#[test]
fn absent_source_fails_without_a_backup_or_complete_byte_event() {
    let temp = tempfile::tempdir().unwrap();
    let backup = temp.path().join("backup");
    let events = RefCell::new(Vec::new());
    assert!(files::copy_tree_with_progress(
        &temp.path().join("missing"),
        &backup,
        false,
        "backup-game",
        &|event| events.borrow_mut().push(event),
    )
    .is_err());
    assert!(!backup.exists());
    assert!(byte_counts(&events.borrow()).is_empty());
}

#[test]
fn final_source_verification_failure_never_reports_all_bytes_complete() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let backup = temp.path().join("backup");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("file"), b"original").unwrap();
    let events = RefCell::new(Vec::new());
    let result = files::copy_tree_with_progress(&source, &backup, false, "backup-game", &|event| {
        if event.completed.as_deref() == Some("0") && event.total.as_deref() == Some("8") {
            // Change the source after its inventory was captured. Pass the reporting
            // throttle deliberately: a final timed sample must wait for verification.
            fs::write(source.join("added-during-backup"), b"external change").unwrap();
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        events.borrow_mut().push(event);
    });
    assert!(result.unwrap_err().contains("source changed"));
    let counts = byte_counts(&events.borrow());
    assert_eq!(counts.first(), Some(&(0, 8)));
    assert!(counts.iter().all(|(completed, total)| completed < total));
    assert_eq!(fs::read(source.join("file")).unwrap(), b"original");
    assert_eq!(fs::read(backup.join("file")).unwrap(), b"original");
    assert!(!backup.join("added-during-backup").exists());
}
