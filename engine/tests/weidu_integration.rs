#![cfg(windows)]

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use bg_engine::events::{ChannelSink, EngineEvent};
use bg_engine::manifest::{GameRoot, InvocationMode};
use bg_engine::weidu::invocation::{
    build, Invocation, InvocationInput, ResolvedPrompt, StagedRoots, VerifiedWeidu,
};
use bg_engine::weidu::log::parse_active_entries;
use bg_engine::weidu::runner::{run, RunOutcome, RunnerControl, RunnerRequest};
use bg_engine::weidu::verify::{reconcile, ExpectedRun, Reconciliation};
use pelite::image::IMAGE_FILE_MACHINE_AMD64;
use pelite::pe64::{Pe, PeFile};
use sha2::{Digest, Sha256};
use support::fakegame::FakeGame;
use tempfile::TempDir;
use walkdir::WalkDir;

const PINNED_WEIDU_SHA256: &str =
    "ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a";
const TESTMOD_TP2: &str = "setup-testmod.tp2";

#[derive(Debug, PartialEq, Eq)]
struct ProtectedSnapshot {
    key: String,
    bif: String,
    root_tlk: String,
    language_tlk: String,
    override_tree: BTreeMap<String, String>,
}

struct Harness {
    _temp: TempDir,
    game: FakeGame,
    roots: StagedRoots,
    tool: VerifiedWeidu,
    source_digest: String,
    protected_before: ProtectedSnapshot,
}

struct RunEvidence {
    outcome: RunOutcome,
    before_log: String,
    after_log: String,
    debug: String,
    output: String,
}

impl Harness {
    fn new() -> Self {
        let temp = TempDir::new().expect("create isolated real-WeiDU test root");
        let source = pinned_weidu(temp.path());
        let source_digest = sha256_file(source.source_path());

        let game_root = temp.path().join("fake-bg2ee");
        let game = FakeGame::build(&game_root).expect("build fake game");
        copy_fixture(&fixture_root(), &game.root);
        let protected_before = snapshot_protected(&game);
        let roots = StagedRoots {
            bg1: game.root.clone(),
            bg2: game.root.clone(),
        };

        Self {
            _temp: temp,
            game,
            roots,
            tool: source,
            source_digest,
            protected_before,
        }
    }

    fn invocation(
        &self,
        attempt_name: &str,
        mode: InvocationMode,
        components: &[u32],
        prompts: Vec<ResolvedPrompt>,
    ) -> Invocation {
        self.invocation_with_args(attempt_name, mode, components, Vec::new(), prompts)
    }

    fn invocation_with_args(
        &self,
        attempt_name: &str,
        mode: InvocationMode,
        components: &[u32],
        run_args: Vec<bg_engine::manifest::RunArg>,
        prompts: Vec<ResolvedPrompt>,
    ) -> Invocation {
        let attempt = self._temp.path().join(attempt_name);
        fs::create_dir(&attempt).expect("create invocation attempt directory");
        build(
            &InvocationInput {
                target: GameRoot::Bg2,
                tp2: TESTMOD_TP2.to_owned(),
                mode,
                explicit_tp2_tested: mode == InvocationMode::ExplicitTp2,
                language: 0,
                remaining_components: components.to_vec(),
                run_args,
                prompts,
            },
            &self.roots,
            &self.tool,
            &attempt,
        )
        .expect("build production WeiDU invocation")
    }

    fn run_sync(&self, invocation: Invocation, attempt_name: &str) -> RunEvidence {
        let before_log = read_optional(&self.game.root.join("WeiDU.log"));
        let debug_path = invocation.debug_path.clone();
        let output_log = self
            ._temp
            .path()
            .join(attempt_name)
            .join("process-output.log");
        let (sink, _events) = ChannelSink::unbounded();
        let (_control_tx, control_rx) = crossbeam_channel::unbounded();

        let outcome = run(
            RunnerRequest {
                invocation,
                step_id: format!("real-weidu:{attempt_name}"),
                output_log: output_log.clone(),
                silence_threshold: Duration::from_secs(10),
            },
            control_rx,
            sink,
        );

        RunEvidence {
            outcome,
            before_log,
            after_log: read_optional(&self.game.root.join("WeiDU.log")),
            debug: fs::read_to_string(debug_path).expect("read real WeiDU debug log"),
            output: fs::read_to_string(output_log).expect("read real WeiDU process log"),
        }
    }

    fn assert_protected_unchanged(&self) {
        let after = snapshot_protected(&self.game);
        assert_eq!(after.key, self.protected_before.key, "chitin.key changed");
        assert_eq!(after.bif, self.protected_before.bif, "fake BIF changed");
        assert_eq!(
            after.root_tlk, self.protected_before.root_tlk,
            "root dialog.tlk changed"
        );
        assert_eq!(
            after.language_tlk, self.protected_before.language_tlk,
            "language dialog.tlk changed"
        );
        assert_eq!(
            sha256_file(self.tool.source_path()),
            self.source_digest,
            "read-only pinned WeiDU source changed"
        );
    }

    fn assert_override_publications(&self, expected_new: &[&str]) {
        let after = snapshot_tree(&self.game.override_dir());
        for (path, digest) in &self.protected_before.override_tree {
            assert_eq!(
                after.get(path),
                Some(digest),
                "initial override file changed: {path}"
            );
        }

        let expected = expected_new
            .iter()
            .map(|path| path.to_ascii_lowercase())
            .collect::<BTreeSet<_>>();
        let actual = after
            .keys()
            .filter(|path| !self.protected_before.override_tree.contains_key(*path))
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected, "unexpected override publication set");
    }
}

#[test]
#[ignore = "requires CHRIZ_WEIDU_EXE pointing to the pinned WeiDU 249 Windows binary"]
fn staged_root_argument_survives_eet_style_path_sanitization() {
    use bg_engine::manifest::RunArg;

    let harness = Harness::new();
    fs::create_dir_all(harness.game.root.join("movies")).expect("create sentinel directory");
    fs::write(
        harness.game.root.join("movies/sodcin05.wbm"),
        b"synthetic SoD movie sentinel",
    )
    .expect("write SoD movie sentinel");
    let components = [50];
    let invocation = harness.invocation_with_args(
        "eet-path",
        InvocationMode::SetupName,
        &components,
        vec![
            RunArg::Literal("--args-list".to_owned()),
            RunArg::Literal("p".to_owned()),
            RunArg::StagedRoot(GameRoot::Bg1),
        ],
        Vec::new(),
    );

    let evidence = harness.run_sync(invocation, "eet-path");

    assert_eq!(
        evidence.outcome,
        RunOutcome::Exited { code: 0 },
        "real WeiDU output:\n{}\ndebug log:\n{}",
        evidence.output,
        evidence.debug
    );
    assert_eq!(
        reconcile(
            &evidence.before_log,
            &evidence.after_log,
            &evidence.output,
            &expected_run(&components, 0),
        ),
        Reconciliation::ProvenDone
    );
    assert_eq!(
        fs::read(harness.game.override_dir().join("tmpathok.txt")).unwrap(),
        b"EET-style path accepted\n"
    );
}

#[test]
#[ignore = "requires CHRIZ_WEIDU_EXE pointing to the pinned WeiDU 249 Windows binary"]
fn setup_name_installs_multiple_components_and_reconciles_real_logs() {
    let harness = Harness::new();
    let components = [0, 10, 20];
    let invocation = harness.invocation(
        "success",
        InvocationMode::SetupName,
        &components,
        vec![ResolvedPrompt {
            expected_output: b"TESTMOD_PROMPT: type alpha".to_vec(),
            answer: b"alpha\n".to_vec(),
        }],
    );
    assert_eq!(
        invocation.program.file_name(),
        Some(OsStr::new("Setup-testmod.exe")),
        "setup-name mode must remain the production default"
    );

    let evidence = harness.run_sync(invocation, "success");
    assert_eq!(
        evidence.outcome,
        RunOutcome::Exited { code: 0 },
        "real WeiDU output:\n{}\ndebug log:\n{}",
        evidence.output,
        evidence.debug
    );
    let reconciliation = reconcile(
        &evidence.before_log,
        &evidence.after_log,
        &evidence.output,
        &expected_run(&components, 0),
    );
    assert_eq!(
        reconciliation,
        Reconciliation::ProvenDone,
        "before:\n{}\nafter:\n{}\noutput:\n{}\ndebug:\n{}",
        evidence.before_log,
        evidence.after_log,
        evidence.output,
        evidence.debug
    );

    let entries = parse_active_entries(&evidence.after_log).expect("parse real WeiDU.log");
    let appended = &entries[entries.len() - components.len()..];
    assert_eq!(
        appended
            .iter()
            .map(|entry| entry.annotation.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("Copy an existing BIFF resource: v1.0.0-test"),
            Some("Record setup-name mod folder: v1.0.0-test"),
            Some("Answer one authored prompt: v1.0.0-test"),
        ]
    );

    harness.assert_protected_unchanged();
    harness.assert_override_publications(&["tmcopy.ids", "tmfolder.txt", "tmprompt.txt"]);
    assert_eq!(
        fs::read(harness.game.override_dir().join("tmcopy.ids")).unwrap(),
        b"IDS V1.0\r\n0 NONE\r\n"
    );
    assert_eq!(
        fs::read_to_string(harness.game.override_dir().join("tmfolder.txt"))
            .unwrap()
            .trim(),
        "testmod",
        "%MOD_FOLDER% must follow the setup-testmod executable identity"
    );
    assert_eq!(
        fs::read_to_string(harness.game.override_dir().join("tmprompt.txt"))
            .unwrap()
            .trim(),
        "alpha"
    );
}

#[test]
#[ignore = "requires CHRIZ_WEIDU_EXE pointing to the pinned WeiDU 249 Windows binary"]
fn failed_component_rolls_back_while_successful_prefix_reconciles() {
    let harness = Harness::new();
    let components = [0, 30];
    let invocation = harness.invocation(
        "controlled-failure",
        InvocationMode::SetupName,
        &components,
        Vec::new(),
    );
    let evidence = harness.run_sync(invocation, "controlled-failure");
    let exit_code = match evidence.outcome {
        RunOutcome::Exited { code } => code,
        other => panic!("real WeiDU did not exit normally: {other:?}"),
    };

    let reconciliation = reconcile(
        &evidence.before_log,
        &evidence.after_log,
        &evidence.output,
        &expected_run(&components, exit_code),
    );
    assert_eq!(
        reconciliation,
        Reconciliation::PartialPrefix {
            remaining: vec![30]
        },
        "before:\n{}\nafter:\n{}\noutput:\n{}\ndebug:\n{}",
        evidence.before_log,
        evidence.after_log,
        evidence.output,
        evidence.debug
    );
    let entries = parse_active_entries(&evidence.after_log).expect("parse real WeiDU.log");
    assert_eq!(
        entries.last().and_then(|entry| entry.annotation.as_deref()),
        Some("Copy an existing BIFF resource: v1.0.0-test")
    );
    assert!(
        !harness.game.override_dir().join("tmfail.txt").exists(),
        "the failing component's file survived WeiDU rollback"
    );
    harness.assert_protected_unchanged();
    harness.assert_override_publications(&["tmcopy.ids"]);
}

#[test]
#[ignore = "requires CHRIZ_WEIDU_EXE pointing to the pinned WeiDU 249 Windows binary"]
fn unexpected_prompt_requires_attention_and_remains_alive_until_cancelled() {
    let harness = Harness::new();
    let invocation = harness.invocation(
        "unexpected-prompt",
        InvocationMode::SetupName,
        &[40],
        vec![ResolvedPrompt {
            expected_output: b"THIS_PROMPT_IS_NOT_AUTHORED_BY_THE_FIXTURE".to_vec(),
            answer: b"beta\n".to_vec(),
        }],
    );
    let before_log = read_optional(&harness.game.root.join("WeiDU.log"));
    let output_log = harness
        ._temp
        .path()
        .join("unexpected-prompt")
        .join("process-output.log");
    let recorded_output_log = output_log.clone();
    let (sink, events) = ChannelSink::unbounded();
    let (control_tx, control_rx) = crossbeam_channel::unbounded();
    let (outcome_tx, outcome_rx) = crossbeam_channel::bounded(1);
    let runner = thread::spawn(move || {
        let outcome = run(
            RunnerRequest {
                invocation,
                step_id: "real-weidu:unexpected-prompt".to_owned(),
                output_log,
                silence_threshold: Duration::from_millis(250),
            },
            control_rx,
            sink,
        );
        let _ = outcome_tx.send(outcome);
    });

    let deadline = Instant::now() + Duration::from_secs(10);
    let attention = loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match events.recv_timeout(remaining) {
            Ok(event @ EngineEvent::AttentionRequired { .. }) => break event,
            Ok(_) => {}
            Err(error) => {
                let _ = control_tx.send(RunnerControl::Cancel);
                let _ = runner.join();
                panic!("real WeiDU did not request attention: {error}");
            }
        }
    };
    match attention {
        EngineEvent::AttentionRequired {
            step_id,
            last_output,
            ..
        } => {
            assert_eq!(step_id, "real-weidu:unexpected-prompt");
            assert!(last_output.contains("TESTMOD_UNEXPECTED_PROMPT: type beta"));
        }
        _ => unreachable!(),
    }
    assert!(
        matches!(
            outcome_rx.try_recv(),
            Err(crossbeam_channel::TryRecvError::Empty)
        ),
        "silence watchdog terminated real WeiDU without authorization"
    );

    control_tx.send(RunnerControl::Cancel).unwrap();
    assert_eq!(
        outcome_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        RunOutcome::Cancelled
    );
    runner.join().unwrap();

    let after_log = read_optional(&harness.game.root.join("WeiDU.log"));
    let process_output = fs::read_to_string(recorded_output_log).unwrap();
    assert_eq!(
        reconcile(
            &before_log,
            &after_log,
            &process_output,
            &expected_run(&[40], 1),
        ),
        Reconciliation::Ambiguous,
        "an in-progress --safe-exit log row must not prove a cancelled component"
    );
    harness.assert_protected_unchanged();
    harness.assert_override_publications(&[]);
}

#[test]
#[ignore = "requires CHRIZ_WEIDU_EXE pointing to the pinned WeiDU 249 Windows binary"]
fn tested_explicit_relative_tp2_mode_runs_without_becoming_the_default() {
    let harness = Harness::new();
    let components = [10];
    let invocation = harness.invocation(
        "explicit-relative",
        InvocationMode::ExplicitTp2,
        &components,
        Vec::new(),
    );
    assert_eq!(
        invocation.program.file_name(),
        Some(OsStr::new("WeiDU.exe"))
    );
    assert_eq!(
        invocation.args.first().map(|arg| arg.as_os_str()),
        Some(OsStr::new(TESTMOD_TP2)),
        "explicit compatibility mode must retain a root-relative TP2 argument"
    );

    let evidence = harness.run_sync(invocation, "explicit-relative");
    assert_eq!(
        evidence.outcome,
        RunOutcome::Exited { code: 0 },
        "real WeiDU output:\n{}\ndebug log:\n{}",
        evidence.output,
        evidence.debug
    );
    assert_eq!(
        reconcile(
            &evidence.before_log,
            &evidence.after_log,
            &evidence.output,
            &expected_run(&components, 0),
        ),
        Reconciliation::ProvenDone
    );
    harness.assert_protected_unchanged();
    harness.assert_override_publications(&["tmfolder.txt"]);
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("testmod")
}

fn copy_fixture(source: &Path, target_root: &Path) {
    assert!(
        source.is_dir(),
        "real-WeiDU integration fixture is missing: {}",
        source.display()
    );
    for entry in WalkDir::new(source) {
        let entry = entry.expect("walk integration fixture");
        let relative = entry.path().strip_prefix(source).unwrap();
        let target = target_root.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).expect("create fixture directory");
        } else if entry.file_type().is_file() {
            fs::copy(entry.path(), &target).expect("copy fixture file");
        } else {
            panic!(
                "fixture contains a non-file entry: {}",
                entry.path().display()
            );
        }
    }
}

fn pinned_weidu(sandbox: &Path) -> VerifiedWeidu {
    let source_path = env::var_os("CHRIZ_WEIDU_EXE")
        .map(PathBuf::from)
        .expect("set CHRIZ_WEIDU_EXE to the pinned WeiDU 249 binary");
    let source_bytes = fs::read(&source_path).expect("read pinned WeiDU binary");
    let pe = PeFile::from_bytes(&source_bytes).expect("pinned WeiDU is a valid PE32+ file");
    assert_eq!(
        pe.file_header().Machine,
        IMAGE_FILE_MACHINE_AMD64,
        "pinned WeiDU must target x64 Windows"
    );

    let version = Command::new(&source_path)
        .arg("--version")
        .current_dir(sandbox)
        .output()
        .expect("query pinned WeiDU version in isolated directory");
    assert!(
        version.status.success(),
        "WeiDU --version failed: {version:?}"
    );
    assert!(
        String::from_utf8_lossy(&version.stdout).contains("WeiDU version 24900"),
        "integration tests require WeiDU 24900, got {:?}",
        String::from_utf8_lossy(&version.stdout)
    );

    VerifiedWeidu::verify(&source_path, PINNED_WEIDU_SHA256)
        .expect("verify the recipe-pinned WeiDU digest")
}

fn expected_run(components: &[u32], exit_code: i32) -> ExpectedRun {
    ExpectedRun {
        tp2: TESTMOD_TP2.to_owned(),
        language: 0,
        components: components.to_vec(),
        exit_code,
    }
}

fn snapshot_protected(game: &FakeGame) -> ProtectedSnapshot {
    ProtectedSnapshot {
        key: sha256_file(&game.key_path()),
        bif: sha256_file(&game.bif_path()),
        root_tlk: sha256_file(&game.root.join("dialog.tlk")),
        language_tlk: sha256_file(&game.root.join("lang/en_US/dialog.tlk")),
        override_tree: snapshot_tree(&game.override_dir()),
    }
}

fn snapshot_tree(root: &Path) -> BTreeMap<String, String> {
    WalkDir::new(root)
        .into_iter()
        .map(|entry| entry.expect("walk protected tree"))
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            let relative = entry.path().strip_prefix(root).unwrap();
            let key = relative
                .to_string_lossy()
                .replace('\\', "/")
                .to_ascii_lowercase();
            (key, sha256_file(entry.path()))
        })
        .collect()
}

fn sha256_file(path: &Path) -> String {
    hex::encode(Sha256::digest(
        fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    ))
}

fn read_optional(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => panic!("read {}: {error}", path.display()),
    }
}
