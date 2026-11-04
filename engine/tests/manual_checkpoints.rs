use bg_engine::manifest::{GameRoot, Phase, Postcondition};
use bg_engine::manual_checkpoint::*;
use bg_engine::resolve::{InstallPlan, PlannedRun};
use bg_engine::Manifest;
use std::fs;
use std::path::Path;

struct Fixture {
    temp: tempfile::TempDir,
    manifest: Manifest,
    plan: InstallPlan,
    checkpoints: Vec<FrozenCheckpoint>,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("bg1")).unwrap();
        fs::create_dir(temp.path().join("game")).unwrap();
        let manifest =
            Manifest::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
                .unwrap();
        let plan = InstallPlan {
            runs: vec![PlannedRun {
                run_id: "eeex".into(),
                mod_id: "eefixpack".into(),
                target: GameRoot::Bg2,
                phase: Phase::Main,
                components: vec![0, 2],
                args: vec![],
                postconditions: vec![],
                artifact_id: "eefixpack".into(),
                weidu_artifact_id: "weidu".into(),
                prompt_scripts: vec![],
            }],
        };
        let (plan, checkpoints) = split_plan(
            &plan,
            &[
                ManualBreakpoint {
                    run_id: "eeex".into(),
                    component: 0,
                },
                ManualBreakpoint {
                    run_id: "eeex".into(),
                    component: 2,
                },
            ],
        )
        .unwrap();
        Self {
            temp,
            manifest,
            plan,
            checkpoints,
        }
    }
    fn root(&self) -> &Path {
        self.temp.path()
    }
    fn begin(&self) {
        begin(
            self.root(),
            &self.checkpoints[0],
            &self.plan,
            &self.manifest.mods,
            &self.manifest.artifacts,
        )
        .unwrap();
    }
    fn review(&self) -> Result<ReviewedCheckpoint, String> {
        review(
            self.root(),
            &self.checkpoints[0],
            &self.plan,
            &self.manifest.mods,
            &self.manifest.artifacts,
        )
    }
    fn add(&self, folder: &str, tp2: &str, component: u32, backup: &str) {
        let path = self.root().join(folder).join(tp2);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("BACKUP ~{backup}~\nAUTHOR ~Fixture~\n")).unwrap();
        let log = self.root().join(folder).join("WeiDU.log");
        let before = fs::read_to_string(&log).unwrap_or_default();
        fs::write(
            log,
            format!("{before}~{tp2}~ #0 #{component} // Manual component: reported version\n"),
        )
        .unwrap();
    }
}

#[test]
fn manual_split_preserves_order_and_defers_postconditions_until_final_segment() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture
            .plan
            .runs
            .iter()
            .map(|r| r.components.clone())
            .collect::<Vec<_>>(),
        vec![vec![0], vec![2]]
    );
    assert_eq!(fixture.checkpoints.len(), 2);
    assert_eq!(fixture.checkpoints[0].next.as_ref().unwrap().component, 2);
    assert!(fixture.checkpoints[1].next.is_none());
    let mut unsplit = fixture.plan.runs[0].clone();
    unsplit.run_id = "group".into();
    unsplit.components = vec![0, 2];
    unsplit.args = vec![bg_engine::manifest::RunArg::StagedRoot(GameRoot::Bg1)];
    unsplit.prompt_scripts = [0, 2]
        .into_iter()
        .map(|component| bg_engine::recipe_view::PromptScript {
            component: bg_engine::manifest::ComponentRef {
                run_id: "group".into(),
                component,
            },
            steps: vec![bg_engine::recipe_view::RenderedPromptStep {
                expected_output: format!("Prompt {component}"),
                answer: format!("{component}\n"),
            }],
        })
        .collect();
    unsplit.postconditions.push(Postcondition::TextFileMarkers {
        path: "file.txt".into(),
        required: vec!["end".into()],
        forbidden: vec![],
        max_bytes: 99,
    });
    let (split, _) = split_plan(
        &InstallPlan {
            runs: vec![unsplit],
        },
        &[ManualBreakpoint {
            run_id: "group".into(),
            component: 0,
        }],
    )
    .unwrap();
    assert!(split.runs[0].postconditions.is_empty());
    assert_eq!(split.runs[1].postconditions.len(), 1);
    for (run, component) in split.runs.iter().zip([0, 2]) {
        assert_eq!(
            run.args,
            vec![bg_engine::manifest::RunArg::StagedRoot(GameRoot::Bg1)]
        );
        assert_eq!(run.prompt_scripts.len(), 1);
        assert_eq!(run.prompt_scripts[0].component.component, component);
        assert_eq!(
            run.prompt_scripts[0].steps[0].answer,
            format!("{component}\n")
        );
    }
}

#[test]
fn manual_removed_anchor_and_indivisible_run_are_rejected() {
    let fixture = Fixture::new();
    assert!(split_plan(
        &fixture.plan,
        &[ManualBreakpoint {
            run_id: "missing".into(),
            component: 0
        }]
    )
    .is_err());
    let mut run = fixture.plan.runs[0].clone();
    run.mod_id = "stratagems".into();
    run.components = vec![1, 2];
    assert!(split_plan(
        &InstallPlan {
            runs: vec![run.clone()]
        },
        &[ManualBreakpoint {
            run_id: run.run_id,
            component: 1
        }]
    )
    .unwrap_err()
    .contains("coordinated"));
}

#[test]
fn manual_unchanged_continuation_survives_reopen_without_customization() {
    let fixture = Fixture::new();
    fixture.begin();
    let saved = paused_status(fixture.root(), &fixture.checkpoints[0]).unwrap();
    assert!(saved.review_token.is_empty());
    fixture.begin();
    let reviewed = fixture.review().unwrap();
    assert!(reviewed.status.additions.is_empty());
    accept(fixture.root(), &fixture.checkpoints[0], &reviewed).unwrap();
    verify_accepted_current(
        fixture.root(),
        &fixture.checkpoints[0],
        &fixture.plan,
        &fixture.manifest.mods,
        &fixture.manifest.artifacts,
    )
    .unwrap();
    assert!(!is_customized(fixture.root()).unwrap());
    game_update_guard(fixture.root()).unwrap();
}

#[test]
fn manual_dragonspear_addition_is_interleaved_and_permanently_blocks_game_updates() {
    let fixture = Fixture::new();
    fixture.begin();
    fixture.add(
        "game",
        "DragonspearUI/setup-DragonspearUI.tp2",
        0,
        "DragonspearUI/backup",
    );
    let reviewed = fixture.review().unwrap();
    assert_eq!(
        reviewed.status.additions[0].description.as_deref(),
        Some("Manual component: reported version")
    );
    accept(fixture.root(), &fixture.checkpoints[0], &reviewed).unwrap();
    let expected = expected_components(
        fixture.root(),
        &fixture.plan,
        &fixture.plan,
        &fixture.manifest.mods,
        &fixture.checkpoints,
        GameRoot::Bg2,
    )
    .unwrap();
    assert_eq!(expected.len(), 3);
    assert_eq!(expected[1].tp2, "dragonspearui/setup-dragonspearui.tp2");
    assert_eq!(expected[2].component, 2);
    // When the run before the pause is left out, the accepted additions keep their place
    // between the installed runs around it.
    let without_first = InstallPlan {
        runs: fixture.plan.runs[1..].to_vec(),
    };
    let expected = expected_components(
        fixture.root(),
        &fixture.plan,
        &without_first,
        &fixture.manifest.mods,
        &fixture.checkpoints,
        GameRoot::Bg2,
    )
    .unwrap();
    assert_eq!(expected.len(), 2);
    assert_eq!(expected[0].tp2, "dragonspearui/setup-dragonspearui.tp2");
    assert_eq!(expected[1].component, 2);
    fs::write(fixture.root().join("game/WeiDU.log"), "").unwrap();
    assert!(is_customized(fixture.root()).unwrap());
    assert!(game_update_guard(fixture.root())
        .unwrap_err()
        .contains("app updates remain available"));
    assert!(verify_accepted_current(
        fixture.root(),
        &fixture.checkpoints[0],
        &fixture.plan,
        &fixture.manifest.mods,
        &fixture.manifest.artifacts
    )
    .is_err());
}

#[test]
fn manual_addition_to_a_standalone_game_needs_no_bg1_folder() {
    // A standalone game such as IWD:EE is staged in `game` alone.
    let fixture = Fixture::new();
    fs::remove_dir(fixture.root().join("bg1")).unwrap();
    fixture.begin();
    let paused = paused_status(fixture.root(), &fixture.checkpoints[0]).unwrap();
    assert_eq!(paused.target, GameRoot::Bg2);
    assert_eq!(paused.target_path, fixture.root().join("game"));
    fixture.add("game", "Other/setup-Other.tp2", 0, "Other/backup");
    let reviewed = fixture.review().unwrap();
    assert_eq!(reviewed.status.additions.len(), 1);
    accept(fixture.root(), &fixture.checkpoints[0], &reviewed).unwrap();
    verify_accepted_current(
        fixture.root(),
        &fixture.checkpoints[0],
        &fixture.plan,
        &fixture.manifest.mods,
        &fixture.manifest.artifacts,
    )
    .unwrap();
    assert!(is_customized(fixture.root()).unwrap());
    assert!(!fixture.root().join("bg1").exists());

    // A BG1 folder that appears during such a pause is a change outside the shown target.
    let fixture = Fixture::new();
    fs::remove_dir(fixture.root().join("bg1")).unwrap();
    fixture.begin();
    fs::create_dir(fixture.root().join("bg1")).unwrap();
    assert!(fixture
        .review()
        .err()
        .unwrap()
        .contains("other game folder"));
}

#[test]
fn manual_wrong_target_changed_prefix_and_failed_installer_are_rejected() {
    let fixture = Fixture::new();
    fixture.begin();
    fixture.add("bg1", "Other/setup-Other.tp2", 0, "Other/backup");
    assert!(fixture
        .review()
        .err()
        .unwrap()
        .contains("other game folder"));
    let fixture = Fixture::new();
    fs::write(
        fixture.root().join("game/WeiDU.log"),
        "~Managed/a.tp2~ #0 #1 // old\n",
    )
    .unwrap();
    fixture.begin();
    fs::write(
        fixture.root().join("game/WeiDU.log"),
        "~Managed/a.tp2~ #0 #1 // new\n",
    )
    .unwrap();
    assert!(fixture
        .review()
        .err()
        .unwrap()
        .contains("removed, reordered or changed"));
    let fixture = Fixture::new();
    fixture.begin();
    fixture.add("game", "Other/setup-Other.tp2", 0, "Other/backup");
    fs::write(
        fixture.root().join("game/SETUP-OTHER.DEBUG"),
        "NOT INSTALLED DUE TO ERRORS Manual\n",
    )
    .unwrap();
    assert!(fixture.review().err().unwrap().contains("failure"));
}

#[test]
fn manual_future_managed_components_and_shared_backup_namespaces_are_rejected() {
    let fixture = Fixture::new();
    fixture.begin();
    fixture.add("game", "EE_Fixpack/EE_Fixpack.tp2", 99, "EE_Fixpack/backup");
    assert!(fixture.review().is_err());
    let fixture = Fixture::new();
    let managed = fixture.root().join("game/EE_Fixpack/EE_Fixpack.tp2");
    fs::create_dir_all(managed.parent().unwrap()).unwrap();
    fs::write(managed, "BACKUP ~weidu_external/backup/EE_Fixpack~\n").unwrap();
    fixture.begin();
    fixture.add(
        "game",
        "Other/setup-Other.tp2",
        0,
        "weidu_external/backup/EE_Fixpack",
    );
    assert!(fixture.review().err().unwrap().contains("BACKUP"));
}

#[test]
fn manual_distinct_shared_backup_subfolder_is_allowed_but_source_edit_is_rejected() {
    let fixture = Fixture::new();
    let managed = fixture.root().join("game/EE_Fixpack/EE_Fixpack.tp2");
    fs::create_dir_all(managed.parent().unwrap()).unwrap();
    fs::write(&managed, "BACKUP ~weidu_external/backup/EE_Fixpack~\n").unwrap();
    fixture.begin();
    fixture.add(
        "game",
        "Other/setup-Other.tp2",
        0,
        "weidu_external/backup/Other",
    );
    assert!(fixture.review().is_ok());
    fs::write(
        &managed,
        "BACKUP ~weidu_external/backup/EE_Fixpack~\nchanged managed source\n",
    )
    .unwrap();
    assert!(fixture
        .review()
        .err()
        .unwrap()
        .contains("source or backup files changed"));
}

#[test]
fn manual_acceptance_uses_exact_reviewed_bytes_and_later_mutation_blocks_resume() {
    let fixture = Fixture::new();
    fixture.begin();
    fixture.add("game", "Other/setup-Other.tp2", 0, "Other/backup");
    let reviewed = fixture.review().unwrap();
    let reviewed_log = fs::read_to_string(fixture.root().join("game/WeiDU.log")).unwrap();
    fs::write(
        fixture.root().join("game/WeiDU.log"),
        "changed after review\n",
    )
    .unwrap();
    accept(fixture.root(), &fixture.checkpoints[0], &reviewed).unwrap();
    assert_eq!(
        acceptance(fixture.root(), &fixture.checkpoints[0])
            .unwrap()
            .unwrap()
            .bg2_log,
        reviewed_log
    );
    assert!(verify_accepted_current(
        fixture.root(),
        &fixture.checkpoints[0],
        &fixture.plan,
        &fixture.manifest.mods,
        &fixture.manifest.artifacts
    )
    .is_err());
}

#[test]
fn manual_file_copy_without_weidu_additions_and_wrong_target_resources_are_rejected() {
    let fixture = Fixture::new();
    fixture.begin();
    fs::create_dir(fixture.root().join("game/override")).unwrap();
    fs::write(
        fixture.root().join("game/override/new.spl"),
        b"copy-only mod",
    )
    .unwrap();
    assert!(fixture.review().err().unwrap().contains("File-copy mods"));
    let fixture = Fixture::new();
    fixture.begin();
    fixture.add("game", "Other/setup-Other.tp2", 0, "Other/backup");
    fs::write(
        fixture.root().join("bg1/dialog.tlk"),
        b"wrong target changed",
    )
    .unwrap();
    assert!(fixture
        .review()
        .err()
        .unwrap()
        .contains("other game folder"));
}

#[test]
fn manual_backup_parser_ignores_comment_decoys_and_local_source_change_invalidates_review() {
    let fixture = Fixture::new();
    fixture.begin();
    fixture.add("game", "Other/setup-Other.tp2", 0, "Other/backup");
    let path = fixture.root().join("game/Other/setup-Other.tp2");
    fs::write(&path, "/*\n BACKUP ~EE_Fixpack/backup~\n*/\n// BACKUP ~EE_Fixpack/backup~\nBACKUP ~Other/backup~\n").unwrap();
    let first = fixture.review().unwrap().status.review_token;
    fs::write(&path, "BACKUP ~Other/backup~\nAUTHOR ~Changed~\n").unwrap();
    assert_ne!(first, fixture.review().unwrap().status.review_token);
    fs::write(
        path,
        "/* BACKUP ~Other/backup~ */ BACKUP ~EE_Fixpack/backup~\n",
    )
    .unwrap();
    assert!(fixture.review().err().unwrap().contains("BACKUP"));
}

#[test]
fn manual_diagnostics_provenance_is_bounded_and_unchanged_old_errors_are_not_new_failures() {
    let fixture = Fixture::new();
    let debug = fixture.root().join("game/OLD-MANAGED.DEBUG");
    fs::write(
        &debug,
        format!(
            "ERROR: old history\n{}",
            "long old diagnostic line\n".repeat(100_000)
        ),
    )
    .unwrap();
    fixture.begin();
    assert!(
        fs::metadata(
            fixture
                .root()
                .join(".chriz/manual-checkpoints/manual-1/baseline.json")
        )
        .unwrap()
        .len()
            < 12_000
    );
    assert!(fixture.review().is_ok());
    fs::write(debug, "ERROR: new failure\n").unwrap();
    assert!(fixture.review().err().unwrap().contains("failure"));
}

/// Reports `0` as the programs running on the PC. The manual-installer check uses nothing else.
struct RunningPrograms(Result<Vec<std::path::PathBuf>, std::io::ErrorKind>);

impl bg_engine::preflight::PreflightHost for RunningPrograms {
    fn available_space(&self, _: &Path) -> std::io::Result<u64> {
        unreachable!("the manual-installer check reads only running programs")
    }

    fn volume_key(&self, _: &Path) -> std::io::Result<std::ffi::OsString> {
        unreachable!("the manual-installer check reads only running programs")
    }

    fn probe_directory_writable(&self, _: &Path) -> std::io::Result<()> {
        unreachable!("the manual-installer check reads only running programs")
    }

    fn running_executable_paths(&self) -> std::io::Result<Vec<std::path::PathBuf>> {
        self.0.clone().map_err(std::io::Error::from)
    }

    fn probe_exclusive_writable_files(
        &self,
        _: &[std::path::PathBuf],
    ) -> Result<(), bg_engine::preflight::ExclusiveFileError> {
        unreachable!("the manual-installer check reads only running programs")
    }
}

#[test]
fn manual_installers_block_checking_wherever_they_run() {
    for installer in [
        Path::new("Downloads").join("weidu.exe"),
        Path::new("Mods").join("ItemRev").join("SETUP-ItemRev.EXE"),
        Path::new("game").join("WeiDU.exe"),
    ] {
        let error = check_manual_processes_with(&RunningPrograms(Ok(vec![
            Path::new("Tools").join("notepad.exe"),
            installer.clone(),
        ])))
        .unwrap_err();
        assert!(
            error.contains("Close the manual installer")
                && error.contains(installer.to_str().unwrap()),
            "{error}"
        );
    }
}

#[test]
fn other_programs_do_not_block_checking() {
    let running = [
        "notepad.exe",
        "Baldur.exe",
        "setup.exe",
        "weidu-helper.exe",
        "mysetup-mod.exe",
        "setup-notes.txt",
    ]
    .map(|name| Path::new("Tools").join(name))
    .to_vec();
    assert_eq!(
        check_manual_processes_with(&RunningPrograms(Ok(running))),
        Ok(())
    );
    assert_eq!(
        check_manual_processes_with(&RunningPrograms(Ok(Vec::new()))),
        Ok(())
    );
}

#[test]
fn a_process_list_that_cannot_be_read_blocks_checking() {
    assert!(check_manual_processes_with(&RunningPrograms(Err(
        std::io::ErrorKind::PermissionDenied
    )))
    .is_err());
}
