use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::resolve::InstallPlan;
use bg_engine::session::{
    CampaignCreated, FrozenIdentity, SessionEvent, SessionStore, SourceGameFingerprints,
};
use tempfile::TempDir;

const RECIPE_PAYLOAD: &[u8] = b"PK\x03\x04signed recipe payload";
const RECIPE_ENVELOPE: &[u8] = br#"{"recipe_id":"alpha","version":"0.1.0"}"#;
const TRUNCATED_TEMP: &[u8] = include_bytes!("fixtures/session/truncated-record.tmp");

struct CampaignFixture {
    _temp: TempDir,
    managed_root: PathBuf,
    cache_root: PathBuf,
    created: CampaignCreated,
}

type IdentityChange = (&'static str, Box<dyn Fn(&mut CampaignCreated)>);

fn normalized_selection() -> NormalizedSelection {
    NormalizedSelection {
        platform: "windows".to_owned(),
        features: BTreeMap::from([("eet".to_owned(), true), ("optional-npc".to_owned(), false)]),
        inputs: BTreeMap::new(),
    }
}

fn campaign_fixture() -> CampaignFixture {
    let temp = TempDir::new().unwrap();
    let managed_root = temp.path().join("managed");
    let cache_root = temp.path().join("cache");
    std::fs::create_dir(&managed_root).unwrap();
    std::fs::create_dir(&cache_root).unwrap();
    let managed_root = std::fs::canonicalize(managed_root).unwrap();
    let cache_root = std::fs::canonicalize(cache_root).unwrap();
    let selection = normalized_selection();
    let plan = InstallPlan { runs: Vec::new() };
    let created = CampaignCreated {
        install_id: "install-001".to_owned(),
        attempt_id: "attempt-001".to_owned(),
        managed_root: managed_root.clone(),
        cache_root: cache_root.clone(),
        recipe_payload: RECIPE_PAYLOAD.to_vec(),
        recipe_payload_sha256: sha256_bytes(RECIPE_PAYLOAD),
        recipe_envelope: RECIPE_ENVELOPE.to_vec(),
        recipe_envelope_sha256: sha256_bytes(RECIPE_ENVELOPE),
        selection_sha256: selection_digest(&selection).unwrap(),
        normalized_selection: selection,
        plan_sha256: plan_digest(&plan).unwrap(),
        source_games: SourceGameFingerprints {
            bg1: "11".repeat(32),
            bg2: "22".repeat(32),
        },
        artifact_identities: vec![FrozenIdentity {
            id: "eet".to_owned(),
            version: "2.7.3".to_owned(),
            sha256: "33".repeat(32),
            length: 1_024,
        }],
        tool_identities: vec![FrozenIdentity {
            id: "weidu".to_owned(),
            version: "24900".to_owned(),
            sha256: "44".repeat(32),
            length: 2_048,
        }],
        staged_bg1: managed_root.join("bg1"),
        staged_bg2: managed_root.join("game"),
    };
    CampaignFixture {
        _temp: temp,
        managed_root,
        cache_root,
        created,
    }
}

fn create_store(fixture: &CampaignFixture) -> SessionStore {
    SessionStore::create(
        &fixture.managed_root,
        SessionEvent::Created(Box::new(fixture.created.clone())),
    )
    .unwrap()
}

fn ledger(root: &Path) -> PathBuf {
    root.join(".chriz/ledger")
}

#[test]
fn creates_frozen_layout_appends_and_replays_an_unresolved_step() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    store
        .append(SessionEvent::StepStarted {
            step_id: "install:eet".to_owned(),
            attempt: 1,
        })
        .unwrap();

    let replay = SessionStore::open(&fixture.managed_root)
        .unwrap()
        .replay()
        .unwrap();

    assert_eq!(replay.records.len(), 2);
    assert_eq!(replay.unresolved_step(), Some("install:eet"));
    assert_eq!(replay.created(), &fixture.created);
    assert_eq!(
        std::fs::read(fixture.managed_root.join(".chriz/recipe/payload.zip")).unwrap(),
        RECIPE_PAYLOAD
    );
    assert_eq!(
        std::fs::read(fixture.managed_root.join(".chriz/recipe/envelope.json")).unwrap(),
        RECIPE_ENVELOPE
    );
    assert!(fixture
        .managed_root
        .join(".chriz/attempts/attempt-001")
        .is_dir());
    assert!(ledger(&fixture.managed_root)
        .join("0000000000.json")
        .is_file());
    assert!(ledger(&fixture.managed_root)
        .join("0000000001.json")
        .is_file());
}

#[test]
fn a_verified_terminal_event_resolves_the_started_step() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    store
        .append(SessionEvent::StepStarted {
            step_id: "acquire:eet".to_owned(),
            attempt: 1,
        })
        .unwrap();
    store
        .append(SessionEvent::StepCompleted {
            step_id: "acquire:eet".to_owned(),
            attempt: 1,
        })
        .unwrap();

    let replay = store.replay().unwrap();
    assert_eq!(replay.records.len(), 3);
    assert_eq!(replay.unresolved_step(), None);
}

#[test]
fn create_is_create_once_and_preserves_the_first_campaign() {
    let fixture = campaign_fixture();
    create_store(&fixture);
    let before = std::fs::read(ledger(&fixture.managed_root).join("0000000000.json")).unwrap();

    assert!(SessionStore::create(
        &fixture.managed_root,
        SessionEvent::Created(Box::new(fixture.created.clone())),
    )
    .is_err());
    assert_eq!(
        std::fs::read(ledger(&fixture.managed_root).join("0000000000.json")).unwrap(),
        before
    );
}

#[test]
fn replay_ignores_an_abandoned_truncated_temp_record() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    store
        .append(SessionEvent::StepStarted {
            step_id: "stage:bg1".to_owned(),
            attempt: 1,
        })
        .unwrap();
    std::fs::write(
        ledger(&fixture.managed_root).join("0000000002.json.tmp"),
        TRUNCATED_TEMP,
    )
    .unwrap();

    let replay = SessionStore::open(&fixture.managed_root)
        .unwrap()
        .replay()
        .unwrap();
    assert_eq!(replay.records.len(), 2);
    assert_eq!(replay.unresolved_step(), Some("stage:bg1"));
}

#[test]
fn replay_rejects_a_sequence_gap() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    store
        .append(SessionEvent::StepStarted {
            step_id: "stage:bg2".to_owned(),
            attempt: 1,
        })
        .unwrap();
    std::fs::rename(
        ledger(&fixture.managed_root).join("0000000001.json"),
        ledger(&fixture.managed_root).join("0000000002.json"),
    )
    .unwrap();

    assert!(SessionStore::open(&fixture.managed_root)
        .unwrap()
        .replay()
        .is_err());
}

#[test]
fn replay_rejects_a_duplicate_sequence_inside_a_different_final_name() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    store
        .append(SessionEvent::StepStarted {
            step_id: "stage:bg2".to_owned(),
            attempt: 1,
        })
        .unwrap();
    let path = ledger(&fixture.managed_root).join("0000000001.json");
    let mut record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    record["sequence"] = 0.into();
    std::fs::write(&path, serde_json::to_vec_pretty(&record).unwrap()).unwrap();

    assert!(SessionStore::open(&fixture.managed_root)
        .unwrap()
        .replay()
        .is_err());
}

#[test]
fn replay_rejects_a_changed_previous_record_hash() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    store
        .append(SessionEvent::StepStarted {
            step_id: "stage:bg2".to_owned(),
            attempt: 1,
        })
        .unwrap();
    let path = ledger(&fixture.managed_root).join("0000000001.json");
    let mut record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    record["previous_sha256"] = serde_json::Value::String("55".repeat(32));
    std::fs::write(&path, serde_json::to_vec_pretty(&record).unwrap()).unwrap();

    assert!(SessionStore::open(&fixture.managed_root)
        .unwrap()
        .replay()
        .is_err());
}

#[test]
fn replay_rejects_a_changed_earlier_final_record() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    store
        .append(SessionEvent::StepStarted {
            step_id: "stage:bg2".to_owned(),
            attempt: 1,
        })
        .unwrap();
    let path = ledger(&fixture.managed_root).join("0000000000.json");
    let mut bytes = std::fs::read(&path).unwrap();
    let position = bytes
        .windows(b"install-001".len())
        .position(|window| window == b"install-001")
        .unwrap();
    bytes[position..position + b"install-001".len()].copy_from_slice(b"install-002");
    std::fs::write(&path, bytes).unwrap();

    assert!(SessionStore::open(&fixture.managed_root)
        .unwrap()
        .replay()
        .is_err());
}

#[test]
fn replay_rejects_a_truncated_final_record() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    store
        .append(SessionEvent::StepStarted {
            step_id: "stage:bg2".to_owned(),
            attempt: 1,
        })
        .unwrap();
    std::fs::write(ledger(&fixture.managed_root).join("0000000001.json"), b"{").unwrap();

    assert!(SessionStore::open(&fixture.managed_root)
        .unwrap()
        .replay()
        .is_err());
}

#[test]
fn append_never_replaces_a_preexisting_final_record() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    let path = ledger(&fixture.managed_root).join("0000000001.json");
    let sentinel = b"preexisting final must survive";
    std::fs::write(&path, sentinel).unwrap();

    assert!(store
        .append(SessionEvent::StepStarted {
            step_id: "stage:bg2".to_owned(),
            attempt: 1,
        })
        .is_err());
    assert_eq!(std::fs::read(path).unwrap(), sentinel);
}

#[test]
fn replay_rejects_recipe_files_that_no_longer_match_the_created_event() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    std::fs::write(
        fixture.managed_root.join(".chriz/recipe/payload.zip"),
        b"changed",
    )
    .unwrap();

    assert!(store.replay().is_err());
}

#[test]
fn resume_rejects_every_consequential_identity_change() {
    let fixture = campaign_fixture();
    let store = create_store(&fixture);
    let replay = store.replay().unwrap();

    let mut changes: Vec<IdentityChange> = vec![
        (
            "recipe payload",
            Box::new(|created| {
                created.recipe_payload.push(0);
                created.recipe_payload_sha256 = sha256_bytes(&created.recipe_payload);
            }),
        ),
        (
            "recipe envelope",
            Box::new(|created| {
                created.recipe_envelope.push(b' ');
                created.recipe_envelope_sha256 = sha256_bytes(&created.recipe_envelope);
            }),
        ),
        (
            "selection",
            Box::new(|created| {
                created
                    .normalized_selection
                    .features
                    .insert("optional-npc".to_owned(), true);
                created.selection_sha256 = selection_digest(&created.normalized_selection).unwrap();
            }),
        ),
        (
            "plan",
            Box::new(|created| created.plan_sha256 = "66".repeat(32)),
        ),
        (
            "source game",
            Box::new(|created| created.source_games.bg1 = "77".repeat(32)),
        ),
        (
            "artifact",
            Box::new(|created| created.artifact_identities[0].sha256 = "88".repeat(32)),
        ),
        (
            "tool",
            Box::new(|created| created.tool_identities[0].sha256 = "99".repeat(32)),
        ),
        (
            "staged path",
            Box::new(|created| created.staged_bg2 = created.managed_root.join("other-game")),
        ),
    ];
    for (label, change) in changes.drain(..) {
        let mut changed = fixture.created.clone();
        change(&mut changed);
        assert!(
            replay.validate_resume(&changed).is_err(),
            "{label} change was accepted"
        );
    }

    let other_cache = fixture.managed_root.join("other-cache");
    std::fs::create_dir(&other_cache).unwrap();
    let mut changed_cache = fixture.created.clone();
    changed_cache.cache_root = std::fs::canonicalize(other_cache).unwrap();
    assert!(replay.validate_resume(&changed_cache).is_err());
    assert_eq!(replay.created().cache_root, fixture.cache_root);
}

#[test]
fn pure_selection_and_plan_digests_are_canonical_and_sensitive() {
    let selection = normalized_selection();
    let mut reordered = NormalizedSelection {
        platform: selection.platform.clone(),
        features: BTreeMap::new(),
        inputs: BTreeMap::new(),
    };
    reordered.features.insert("optional-npc".to_owned(), false);
    reordered.features.insert("eet".to_owned(), true);
    assert_eq!(
        selection_digest(&selection).unwrap(),
        selection_digest(&reordered).unwrap()
    );

    reordered.features.insert("optional-npc".to_owned(), true);
    assert_ne!(
        selection_digest(&selection).unwrap(),
        selection_digest(&reordered).unwrap()
    );

    let empty = InstallPlan { runs: Vec::new() };
    let changed = InstallPlan {
        runs: vec![bg_engine::resolve::PlannedRun {
            run_id: "eet".to_owned(),
            mod_id: "eet".to_owned(),
            target: bg_engine::manifest::GameRoot::Bg2,
            phase: bg_engine::manifest::Phase::EetInitialization,
            components: vec![0],
            args: Vec::new(),
            artifact_id: "eet".to_owned(),
            weidu_artifact_id: "weidu".to_owned(),
            prompt_scripts: Vec::new(),
        }],
    };
    assert_ne!(plan_digest(&empty).unwrap(), plan_digest(&changed).unwrap());
}
