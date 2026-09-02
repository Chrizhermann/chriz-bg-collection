use std::collections::BTreeMap;
use std::path::PathBuf;

use bg_engine::error::EngineError;
use bg_engine::resolve::Selection;
use bg_engine::session::{manifest_fingerprint, Session, StepRecord, StepStatus};
use bg_engine::Manifest;

fn selection() -> Selection {
    Selection {
        choices: BTreeMap::from([("difficulty".to_owned(), "tactical".to_owned())]),
        platform: "windows".to_owned(),
    }
}

fn step(id: &str, status: StepStatus) -> StepRecord {
    StepRecord {
        id: id.to_owned(),
        status,
        detail: None,
    }
}

fn session() -> Session {
    Session {
        manifest_fingerprint: "manifest-sha256".to_owned(),
        selection: selection(),
        steps: vec![
            step("preflight", StepStatus::Done),
            step("acquire:testmod", StepStatus::Pending),
        ],
    }
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest")
}

#[test]
fn round_trips_through_session_json() {
    let target = tempfile::tempdir().unwrap();
    let original = session();

    original.save(target.path()).unwrap();
    let loaded = Session::load(target.path(), "manifest-sha256").unwrap();

    assert_eq!(loaded, original);
    assert!(target.path().join("session.json").is_file());
}

#[test]
fn next_pending_skips_done_and_returns_failed_work_for_retry() {
    let mut session = session();
    session.steps = vec![
        step("preflight", StepStatus::Done),
        step("install:eet", StepStatus::Failed),
        step("install:testmod", StepStatus::Pending),
    ];

    assert_eq!(session.next_pending(), Some(1));

    session.steps[1].status = StepStatus::Done;
    assert_eq!(session.next_pending(), Some(2));

    session.steps[2].status = StepStatus::Done;
    assert!(session.next_pending().is_none());
}

#[test]
fn load_marks_running_steps_failed_after_an_interrupted_run() {
    let target = tempfile::tempdir().unwrap();
    let mut interrupted = session();
    interrupted.steps = vec![
        StepRecord {
            id: "acquire:testmod".to_owned(),
            status: StepStatus::Running,
            detail: Some("download started".to_owned()),
        },
        step("stage:copy-bg2", StepStatus::Done),
        StepRecord {
            id: "install:testmod".to_owned(),
            status: StepStatus::Running,
            detail: Some("WeiDU process started".to_owned()),
        },
    ];
    interrupted.save(target.path()).unwrap();

    let loaded = Session::load(target.path(), "manifest-sha256").unwrap();

    assert_eq!(loaded.steps[0].status, StepStatus::Failed);
    assert_eq!(loaded.steps[2].status, StepStatus::Failed);
    assert_eq!(loaded.steps[0].detail.as_deref(), Some("download started"));
    assert_eq!(
        loaded.steps[2].detail.as_deref(),
        Some("WeiDU process started")
    );
    assert_eq!(loaded.next_pending(), Some(0));
}

#[test]
fn load_rejects_a_changed_manifest_fingerprint() {
    let target = tempfile::tempdir().unwrap();
    session().save(target.path()).unwrap();
    let session_path = target.path().join("session.json");

    let error = Session::load(target.path(), "different-sha256").unwrap_err();
    assert!(error.to_string().contains("rebuild"), "{error}");

    match error {
        EngineError::ManifestFingerprintMismatch {
            path,
            expected,
            found,
        } => {
            assert_eq!(path, session_path);
            assert_eq!(expected, "different-sha256");
            assert_eq!(found, "manifest-sha256");
        }
        other => panic!("expected a fingerprint mismatch, got {other:?}"),
    }
}

#[test]
fn save_atomically_replaces_session_json_without_leaving_a_temp_file() {
    let target = tempfile::tempdir().unwrap();
    let session_path = target.path().join("session.json");
    std::fs::write(&session_path, "incomplete old session").unwrap();
    let replacement = session();

    replacement.save(target.path()).unwrap();

    let loaded = Session::load(target.path(), "manifest-sha256").unwrap();
    assert_eq!(loaded, replacement);
    let mut names: Vec<String> = std::fs::read_dir(target.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["session.json"]);
}

#[test]
fn malformed_session_json_carries_its_path() {
    let target = tempfile::tempdir().unwrap();
    let session_path = target.path().join("session.json");
    std::fs::write(&session_path, "{").unwrap();

    let error = Session::load(target.path(), "manifest-sha256").unwrap_err();

    match error {
        EngineError::SessionJson { path, .. } => assert_eq!(path, session_path),
        other => panic!("expected a session JSON error, got {other:?}"),
    }
}

#[test]
fn failed_publish_cleans_up_the_temp_file_and_preserves_the_destination() {
    let target = tempfile::tempdir().unwrap();
    let session_path = target.path().join("session.json");
    std::fs::create_dir(&session_path).unwrap();

    let error = session().save(target.path()).unwrap_err();

    match error {
        EngineError::Io { path, .. } => assert_eq!(path, session_path),
        other => panic!("expected an I/O error, got {other:?}"),
    }
    assert!(session_path.is_dir());
    let names: Vec<String> = std::fs::read_dir(target.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["session.json"]);
}

#[test]
fn manifest_fingerprint_is_a_root_independent_lowercase_sha256() {
    let manifest = Manifest::load(&fixture_dir()).unwrap();
    let mut relocated = manifest.clone();
    relocated.root = PathBuf::from("a/different/manifest/root");

    let fingerprint = manifest_fingerprint(&manifest).unwrap();
    let relocated_fingerprint = manifest_fingerprint(&relocated).unwrap();

    assert_eq!(fingerprint, relocated_fingerprint);
    assert_eq!(fingerprint.len(), 64);
    assert!(
        fingerprint
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "not lowercase hexadecimal: {fingerprint}"
    );
}

#[test]
fn manifest_fingerprint_changes_with_semantic_manifest_content() {
    let manifest = Manifest::load(&fixture_dir()).unwrap();
    let mut changed = manifest.clone();
    changed.collection.game_build = "different-build".to_owned();

    assert_ne!(
        manifest_fingerprint(&manifest).unwrap(),
        manifest_fingerprint(&changed).unwrap()
    );
}
