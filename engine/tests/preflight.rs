use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::games::{
    Eligibility, FindingKind, GameCandidate, GameFinding, GameRole, Storefront,
};
use bg_engine::preflight::{
    initial_preflight_with, is_creator_protected_destination, recheck_target_before_mutation_with,
    ExclusiveFileError, InitialPreflight, PreflightError, PreflightHost, RequiredInput,
    SpaceRequirement, SystemPreflight,
};
use bg_engine::recipe_view::NormalizedSelection;
use bg_engine::resolve::InstallPlan;
use bg_engine::session::{
    CampaignCreated, FrozenIdentity, SessionEvent, SessionReplay, SessionStore,
    SourceGameFingerprints,
};

const RECIPE_PAYLOAD: &[u8] = b"PK\x03\x04signed recipe payload";
const RECIPE_ENVELOPE: &[u8] = br#"{"recipe_id":"alpha","version":"0.1.0"}"#;

#[derive(Clone)]
struct FakeHost {
    free_space: u64,
    cache_free_space: u64,
    separate_cache_volume: bool,
    running: Vec<PathBuf>,
    unwritable_directory_name: Option<String>,
    tlk_error: Option<PathBuf>,
}

impl Default for FakeHost {
    fn default() -> Self {
        Self {
            free_space: u64::MAX,
            cache_free_space: u64::MAX,
            separate_cache_volume: false,
            running: Vec::new(),
            unwritable_directory_name: None,
            tlk_error: None,
        }
    }
}

impl PreflightHost for FakeHost {
    fn available_space(&self, path: &Path) -> io::Result<u64> {
        if path.file_name().and_then(|name| name.to_str()) == Some("cache") {
            Ok(self.cache_free_space)
        } else {
            Ok(self.free_space)
        }
    }

    fn volume_key(&self, path: &Path) -> io::Result<std::ffi::OsString> {
        if self.separate_cache_volume
            && path.file_name().and_then(|name| name.to_str()) == Some("cache")
        {
            Ok("cache-volume".into())
        } else {
            Ok("managed-volume".into())
        }
    }

    fn probe_directory_writable(&self, path: &Path) -> io::Result<()> {
        if self.unwritable_directory_name.as_deref()
            == path.file_name().and_then(|name| name.to_str())
        {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        } else {
            Ok(())
        }
    }

    fn running_executable_paths(&self) -> io::Result<Vec<PathBuf>> {
        Ok(self.running.clone())
    }

    fn probe_exclusive_writable_files(&self, paths: &[PathBuf]) -> Result<(), ExclusiveFileError> {
        if let Some(path) = paths
            .iter()
            .find(|path| self.tlk_error.as_deref() == Some(path.as_path()))
        {
            return Err(ExclusiveFileError {
                path: path.clone(),
                source: io::Error::from(io::ErrorKind::PermissionDenied),
            });
        }
        for path in paths {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(|source| ExclusiveFileError {
                    path: path.clone(),
                    source,
                })?;
        }
        Ok(())
    }
}

struct Fixture {
    _temp: tempfile::TempDir,
    bg1: GameCandidate,
    bg2: GameCandidate,
    destination: PathBuf,
    replay: SessionReplay,
    plan: InstallPlan,
}

impl Fixture {
    fn new() -> Self {
        Self::with_frozen_plan_sha256(None)
    }

    fn with_frozen_plan_sha256(plan_sha256: Option<String>) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let bg1_root = temp.path().join("clean-bg1");
        let bg2_root = temp.path().join("clean-bg2");
        let destination = temp.path().join("managed");
        let cache = temp.path().join("cache");
        for root in [&bg1_root, &bg2_root, &destination, &cache] {
            fs::create_dir(root).unwrap();
        }
        fs::write(bg1_root.join("chitin.key"), b"bg1").unwrap();
        fs::write(bg2_root.join("chitin.key"), b"bg2").unwrap();
        let bg1_root = fs::canonicalize(bg1_root).unwrap();
        let bg2_root = fs::canonicalize(bg2_root).unwrap();
        let destination = fs::canonicalize(destination).unwrap();
        let cache = fs::canonicalize(cache).unwrap();

        let bg1_fingerprint = "11".repeat(32);
        let bg2_fingerprint = "22".repeat(32);
        let plan = InstallPlan { runs: Vec::new() };
        let selection = NormalizedSelection {
            platform: "windows".to_owned(),
            features: BTreeMap::new(),
            inputs: BTreeMap::new(),
        };
        let artifacts = artifact_inputs();
        let tools = tool_inputs();
        let created = CampaignCreated {
            install_id: "install-001".to_owned(),
            attempt_id: "attempt-001".to_owned(),
            managed_root: destination.clone(),
            cache_root: cache,
            recipe_payload: RECIPE_PAYLOAD.to_vec(),
            recipe_payload_sha256: sha256_bytes(RECIPE_PAYLOAD),
            recipe_envelope: RECIPE_ENVELOPE.to_vec(),
            recipe_envelope_sha256: sha256_bytes(RECIPE_ENVELOPE),
            selection_sha256: selection_digest(&selection).unwrap(),
            normalized_selection: selection,
            plan_sha256: plan_sha256.unwrap_or_else(|| plan_digest(&plan).unwrap()),
            source_games: SourceGameFingerprints {
                bg1: bg1_fingerprint.clone(),
                bg2: bg2_fingerprint.clone(),
            },
            artifact_identities: identities(&artifacts),
            tool_identities: identities(&tools),
            staged_bg1: destination.join("bg1"),
            staged_bg2: destination.join("game"),
        };
        let store =
            SessionStore::create(&destination, SessionEvent::Created(Box::new(created))).unwrap();
        let replay = store.replay().unwrap();

        Self {
            _temp: temp,
            bg1: candidate(GameRole::BgeeSod, bg1_root, bg1_fingerprint),
            bg2: candidate(GameRole::Bg2ee, bg2_root, bg2_fingerprint),
            destination,
            replay,
            plan,
        }
    }
}

fn candidate(role: GameRole, root: PathBuf, fingerprint: String) -> GameCandidate {
    GameCandidate {
        role,
        storefront: Storefront::Steam,
        root: root.clone(),
        build: Some("2.7.3.0".to_owned()),
        eligibility: Eligibility::Eligible,
        findings: vec![GameFinding {
            kind: FindingKind::Fresh,
            message: "fixture is fresh".to_owned(),
            paths: vec![root],
        }],
        fingerprint: Some(fingerprint),
    }
}

fn artifact_inputs() -> Vec<RequiredInput> {
    vec![RequiredInput {
        id: "eefixpack".to_owned(),
        version: "1.0.0".to_owned(),
        sha256: "aa".repeat(32),
        length: 42,
        obtainable: true,
    }]
}

fn tool_inputs() -> Vec<RequiredInput> {
    vec![RequiredInput {
        id: "weidu".to_owned(),
        version: "24900".to_owned(),
        sha256: "bb".repeat(32),
        length: 84,
        obtainable: true,
    }]
}

fn identities(inputs: &[RequiredInput]) -> Vec<FrozenIdentity> {
    inputs
        .iter()
        .map(|input| FrozenIdentity {
            id: input.id.clone(),
            version: input.version.clone(),
            sha256: input.sha256.clone(),
            length: input.length,
        })
        .collect()
}

fn request<'a>(
    fixture: &'a Fixture,
    artifacts: &'a [RequiredInput],
    tools: &'a [RequiredInput],
) -> InitialPreflight<'a> {
    InitialPreflight {
        bg1_source: &fixture.bg1,
        bg2_source: &fixture.bg2,
        destination: &fixture.destination,
        space: SpaceRequirement {
            staged_copies: 40,
            downloads: 20,
            extraction: 30,
            safety_margin: 10,
        },
        frozen_campaign: &fixture.replay,
        current_plan: &fixture.plan,
        expected_review_token: "same-token",
        presented_review_token: "same-token",
        required_artifacts: artifacts,
        required_tools: tools,
    }
}

fn create_target(root: &Path) {
    fs::create_dir_all(root.join("lang/en_US")).unwrap();
    fs::write(root.join("dialog.tlk"), b"root tlk").unwrap();
    fs::write(root.join("lang/en_US/dialog.tlk"), b"language tlk").unwrap();
}

#[test]
fn insufficient_total_space_is_rejected() {
    let fixture = Fixture::new();
    let artifacts = artifact_inputs();
    let tools = tool_inputs();
    let host = FakeHost {
        free_space: 99,
        ..FakeHost::default()
    };

    let error = initial_preflight_with(&request(&fixture, &artifacts, &tools), &host).unwrap_err();
    assert!(matches!(
        error,
        PreflightError::InsufficientSpace {
            required: 100,
            available: 99
        }
    ));
}

#[cfg(windows)]
#[test]
fn only_the_two_creator_reference_roots_are_denylisted_under_c_games() {
    for protected in [
        r"C:\Games\Baldur's Gate II Enhanced Edition modded",
        r"C:\Games\Baldurs Gate 1 and 2 mods",
    ] {
        assert!(is_creator_protected_destination(Path::new(protected)));
        assert!(is_creator_protected_destination(
            &Path::new(protected).join("nested")
        ));
    }
    assert!(is_creator_protected_destination(Path::new(
        r"C:\Games\scratch\..\Baldur's Gate II Enhanced Edition modded"
    )));
    assert!(!is_creator_protected_destination(Path::new(
        r"C:\Games\My New Managed EET"
    )));
}

#[test]
fn non_writable_destination_is_rejected() {
    let fixture = Fixture::new();
    let artifacts = artifact_inputs();
    let tools = tool_inputs();
    let host = FakeHost {
        unwritable_directory_name: Some("managed".to_owned()),
        ..FakeHost::default()
    };

    let error = initial_preflight_with(&request(&fixture, &artifacts, &tools), &host).unwrap_err();
    assert!(matches!(
        error,
        PreflightError::DestinationNotWritable { .. }
    ));
}

#[test]
fn frozen_cache_root_must_be_direct_and_writable() {
    let fixture = Fixture::new();
    let artifacts = artifact_inputs();
    let tools = tool_inputs();
    let host = FakeHost {
        unwritable_directory_name: Some("cache".to_owned()),
        ..FakeHost::default()
    };

    let error = initial_preflight_with(&request(&fixture, &artifacts, &tools), &host).unwrap_err();
    assert!(matches!(error, PreflightError::CacheNotWritable { .. }));
}

#[test]
fn a_separate_cache_volume_gets_its_own_space_check() {
    let fixture = Fixture::new();
    let artifacts = artifact_inputs();
    let tools = tool_inputs();
    let host = FakeHost {
        free_space: 50,
        cache_free_space: 49,
        separate_cache_volume: true,
        ..FakeHost::default()
    };

    let error = initial_preflight_with(&request(&fixture, &artifacts, &tools), &host).unwrap_err();
    assert!(matches!(
        error,
        PreflightError::InsufficientCacheSpace {
            required: 50,
            available: 49
        }
    ));
}

#[test]
fn process_matching_uses_canonical_paths_under_the_target_not_names() {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.destination.join("game")).unwrap();
    let inside = fixture.destination.join("game/Baldur.exe");
    let outside = fixture._temp.path().join("other/Baldur.exe");
    fs::create_dir_all(outside.parent().unwrap()).unwrap();
    fs::write(&inside, b"inside").unwrap();
    fs::write(&outside, b"outside").unwrap();
    let artifacts = artifact_inputs();
    let tools = tool_inputs();

    let outside_only = FakeHost {
        running: vec![outside],
        ..FakeHost::default()
    };
    initial_preflight_with(&request(&fixture, &artifacts, &tools), &outside_only).unwrap();

    let inside_running = FakeHost {
        running: vec![inside.clone()],
        ..FakeHost::default()
    };
    let error = initial_preflight_with(&request(&fixture, &artifacts, &tools), &inside_running)
        .unwrap_err();
    assert!(matches!(
        error,
        PreflightError::TargetProcessRunning { executable } if executable == fs::canonicalize(inside).unwrap()
    ));
}

#[test]
fn locked_root_tlk_is_rejected_before_mutation() {
    let fixture = Fixture::new();
    let target = fixture.destination.join("game");
    create_target(&target);
    let root_tlk = fs::canonicalize(target.join("dialog.tlk")).unwrap();
    let host = FakeHost {
        tlk_error: Some(root_tlk.clone()),
        ..FakeHost::default()
    };

    let error = recheck_target_before_mutation_with(&target, "en_US", &host).unwrap_err();
    assert!(matches!(
        error,
        PreflightError::TlkUnavailable { path, .. } if path == root_tlk
    ));
}

#[test]
fn locked_language_tlk_is_rejected_before_mutation() {
    let fixture = Fixture::new();
    let target = fixture.destination.join("game");
    create_target(&target);
    let language_tlk = fs::canonicalize(target.join("lang/en_US/dialog.tlk")).unwrap();
    let host = FakeHost {
        tlk_error: Some(language_tlk.clone()),
        ..FakeHost::default()
    };

    let error = recheck_target_before_mutation_with(&target, "en_US", &host).unwrap_err();
    assert!(matches!(
        error,
        PreflightError::TlkUnavailable { path, .. } if path == language_tlk
    ));
}

#[cfg(windows)]
#[test]
fn system_tlk_probe_holds_both_non_truncating_share_mode_zero_handles_together() {
    use std::os::windows::fs::OpenOptionsExt;

    let fixture = Fixture::new();
    let target = fixture.destination.join("game");
    create_target(&target);
    let tlks = [
        fs::canonicalize(target.join("dialog.tlk")).unwrap(),
        fs::canonicalize(target.join("lang/en_US/dialog.tlk")).unwrap(),
    ];
    let originals = tlks.clone().map(|path| fs::read(path).unwrap());

    for tlk in &tlks {
        let held = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(tlk)
            .unwrap();
        let error =
            recheck_target_before_mutation_with(&target, "en_US", &SystemPreflight).unwrap_err();
        assert!(matches!(
            error,
            PreflightError::TlkUnavailable { path, .. } if path == *tlk
        ));
        drop(held);
    }

    recheck_target_before_mutation_with(&target, "en_US", &SystemPreflight).unwrap();
    assert_eq!(fs::read(&tlks[0]).unwrap(), originals[0]);
    assert_eq!(fs::read(&tlks[1]).unwrap(), originals[1]);
}

#[test]
fn changed_review_token_is_rejected() {
    let fixture = Fixture::new();
    let artifacts = artifact_inputs();
    let tools = tool_inputs();
    let mut changed = request(&fixture, &artifacts, &tools);
    changed.presented_review_token = "different-token";

    let error = initial_preflight_with(&changed, &FakeHost::default()).unwrap_err();
    assert!(matches!(error, PreflightError::ReviewMismatch));
}

#[test]
fn current_plan_must_match_the_frozen_plan_digest() {
    let fixture = Fixture::with_frozen_plan_sha256(Some("99".repeat(32)));
    let artifacts = artifact_inputs();
    let tools = tool_inputs();

    let error =
        initial_preflight_with(&request(&fixture, &artifacts, &tools), &FakeHost::default())
            .unwrap_err();
    assert!(matches!(
        error,
        PreflightError::FrozenCampaignMismatch {
            field: "resolved plan",
            ..
        }
    ));
}

#[test]
fn current_source_fingerprint_must_match_the_frozen_campaign() {
    let fixture = Fixture::new();
    let artifacts = artifact_inputs();
    let tools = tool_inputs();
    let mut changed_bg1 = fixture.bg1.clone();
    changed_bg1.fingerprint = Some("77".repeat(32));
    let mut changed = request(&fixture, &artifacts, &tools);
    changed.bg1_source = &changed_bg1;

    let error = initial_preflight_with(&changed, &FakeHost::default()).unwrap_err();
    assert!(matches!(
        error,
        PreflightError::SourceNotFresh {
            label: "BGEE+SoD",
            ..
        }
    ));
}

#[test]
fn required_artifact_and_tool_sets_must_match_frozen_nonzero_pins() {
    let fixture = Fixture::new();
    let mut artifacts = artifact_inputs();
    let tools = tool_inputs();
    artifacts[0].version = "different".to_owned();

    let error =
        initial_preflight_with(&request(&fixture, &artifacts, &tools), &FakeHost::default())
            .unwrap_err();
    assert!(matches!(
        error,
        PreflightError::FrozenCampaignMismatch {
            field: "artifact identities",
            ..
        }
    ));

    artifacts = artifact_inputs();
    artifacts[0].sha256 = "00".repeat(32);
    let error =
        initial_preflight_with(&request(&fixture, &artifacts, &tools), &FakeHost::default())
            .unwrap_err();
    assert!(matches!(
        error,
        PreflightError::RequiredInputUnavailable { ref id, .. } if id == "eefixpack"
    ));
}

#[test]
fn unobtainable_required_input_is_rejected() {
    let fixture = Fixture::new();
    let mut artifacts = artifact_inputs();
    let tools = tool_inputs();
    artifacts[0].obtainable = false;

    let error =
        initial_preflight_with(&request(&fixture, &artifacts, &tools), &FakeHost::default())
            .unwrap_err();
    assert!(matches!(
        error,
        PreflightError::RequiredInputUnavailable { ref id, .. } if id == "eefixpack"
    ));
}

#[test]
fn clean_initial_and_per_run_preflights_pass() {
    let fixture = Fixture::new();
    let artifacts = artifact_inputs();
    let tools = tool_inputs();
    let host = FakeHost {
        free_space: 100,
        ..FakeHost::default()
    };

    let report = initial_preflight_with(&request(&fixture, &artifacts, &tools), &host).unwrap();
    assert_eq!(report.required_space, 100);
    assert_eq!(report.available_space, 100);

    let target = fixture.destination.join("game");
    create_target(&target);
    recheck_target_before_mutation_with(&target, "en_US", &host).unwrap();
}
