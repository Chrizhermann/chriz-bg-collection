use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use bg_engine::lock::TargetLock;
use serde_json::Value;
use tempfile::TempDir;

const FIXTURE_RECIPE: &str = "tests/fixtures/manifest";
const GAME_FIXTURES: &str = "tests/fixtures/games";

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_chriz-bg-install"))
}

fn run(args: &[&str]) -> Output {
    cli().args(args).output().expect("run chriz-bg-install")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("CLI stdout is UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("CLI stderr is UTF-8")
}

fn json_stdout(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout was not one JSON value: {error}\nstdout:\n{}\nstderr:\n{}",
            stdout(output),
            stderr(output)
        )
    })
}

fn fixture_recipe() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_RECIPE)
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create copied directory");
    let mut entries = fs::read_dir(source)
        .expect("read copied directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("read copied entries");
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
    for entry in entries {
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("read entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy fixture file");
        }
    }
}

fn recipe_with_profiles(temp: &TempDir) -> PathBuf {
    let recipe = temp.path().join("recipe");
    copy_tree(&fixture_recipe(), &recipe);
    add_selected_fixture_features(&recipe);
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(GAME_FIXTURES)
            .join("profiles"),
        &recipe.join("game-builds"),
    );
    recipe
}

fn add_selected_fixture_features(recipe: &Path) {
    let collection = recipe.join("collection.toml");
    let mut text = fs::read_to_string(&collection).unwrap();
    text.push_str(
        r#"

[[features]]
id = "eefix-bg1"
title = "BG1 fixes"
description = "Fixture BG1 fixes"
category = "foundation"
decision = "default"
readiness = "ready"
components = [
  { run_id = "eefixpack-bg1", component = 0 },
  { run_id = "eefixpack-bg1", component = 2 },
]

[[features]]
id = "eefix-bg2"
title = "BG2 fixes"
description = "Fixture BG2 fixes"
category = "foundation"
decision = "default"
readiness = "ready"
components = [{ run_id = "eefixpack-bg2", component = 0 }]
"#,
    );
    fs::write(collection, text).unwrap();
}

fn recipe_with_selected_runs(temp: &TempDir) -> PathBuf {
    let recipe = temp.path().join("recipe");
    copy_tree(&fixture_recipe(), &recipe);
    add_selected_fixture_features(&recipe);
    recipe
}

fn copied_game(temp: &TempDir, fixture: &str, name: &str) -> PathBuf {
    let root = temp.path().join(name);
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(GAME_FIXTURES)
            .join("content")
            .join(fixture),
        &root,
    );
    fs::create_dir_all(root.join("override")).expect("create fixture override");
    root
}

fn install_args<'a>(
    recipe: &'a Path,
    bg1: &'a Path,
    bg2: &'a Path,
    managed: &'a Path,
    cache: &'a Path,
) -> Vec<&'a str> {
    vec![
        "--json",
        "install",
        recipe.to_str().unwrap(),
        "--preset",
        "recommended",
        "--bg1",
        bg1.to_str().unwrap(),
        "--bg2",
        bg2.to_str().unwrap(),
        "--managed-root",
        managed.to_str().unwrap(),
        "--cache",
        cache.to_str().unwrap(),
    ]
}

fn failed_install_fixture() -> (TempDir, PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    let temp = tempfile::tempdir().expect("create CLI fixture root");
    let recipe = recipe_with_profiles(&temp);
    let bg1 = copied_game(&temp, "steam-bgee-sod", "bg1");
    let bg2 = copied_game(&temp, "steam-bg2ee", "bg2");
    fs::write(bg1.join("setup-dirty.exe"), b"modified")
        .expect("add explicit modified-game residue");
    let managed = temp.path().join("managed");
    let cache = temp.path().join("cache");
    let app_data = temp.path().join("app-data");
    fs::create_dir(&cache).expect("create cache root");
    fs::create_dir(&app_data).expect("create app-data root");
    (temp, recipe, bg1, bg2, managed, cache)
}

#[test]
fn help_exposes_the_complete_safe_command_surface_without_bypasses() {
    let output = run(&["--help"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    for command in [
        "validate",
        "plan",
        "games",
        "install",
        "resume",
        "report",
        "diagnostics",
    ] {
        assert!(text.contains(command), "missing {command} in:\n{text}");
    }
    assert!(!text.contains("assume-build"), "{text}");
    assert!(!text.contains("skip"), "{text}");

    let games = run(&["games", "--help"]);
    assert!(games.status.success(), "{}", stderr(&games));
    let games = stdout(&games);
    assert!(games.contains("discover"), "{games}");
    assert!(games.contains("inspect"), "{games}");
}

#[test]
fn validate_has_deterministic_human_and_json_output() {
    let recipe = fixture_recipe();
    let recipe = recipe.to_str().unwrap();

    let first = run(&["validate", recipe, "--profile", "authoring"]);
    let second = run(&["validate", recipe, "--profile", "authoring"]);
    assert!(first.status.success(), "{}", stderr(&first));
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    assert!(stdout(&first).contains("Recipe is valid"));

    let first = run(&["--json", "validate", recipe, "--profile", "authoring"]);
    let second = run(&["--json", "validate", recipe, "--profile", "authoring"]);
    assert!(first.status.success(), "{}", stderr(&first));
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(json_stdout(&first)["command"], "validate");
    assert_eq!(json_stdout(&first)["ok"], true);
}

#[test]
fn validation_errors_exit_one_and_include_the_recipe_path() {
    let temp = tempfile::tempdir().unwrap();
    let recipe = temp.path().join("broken-recipe");
    copy_tree(&fixture_recipe(), &recipe);
    let collection = recipe.join("collection.toml");
    let text = fs::read_to_string(&collection).unwrap();
    fs::write(
        &collection,
        text.replace(
            "components = [0]\nargs = []",
            "components = [999]\nargs = []",
        ),
    )
    .unwrap();

    let output = run(&["validate", recipe.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let error = stderr(&output);
    assert!(error.contains("references"), "{error}");
    assert!(error.contains(&recipe.display().to_string()), "{error}");

    let missing = temp.path().join("missing-recipe");
    let output = run(&["--json", "validate", missing.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let response = json_stdout(&output);
    assert_eq!(response["ok"], false);
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains(&missing.display().to_string()));
}

#[test]
fn plan_has_deterministic_human_and_json_output() {
    let temp = tempfile::tempdir().unwrap();
    let recipe = recipe_with_selected_runs(&temp);
    let recipe = recipe.to_str().unwrap();
    let args = ["plan", recipe, "--preset", "recommended"];
    let first = run(&args);
    let second = run(&args);
    assert!(first.status.success(), "{}", stderr(&first));
    assert_eq!(first.stdout, second.stdout);
    assert!(stdout(&first).contains("eefixpack-bg1"));
    assert!(stdout(&first).contains("eefixpack-bg2"));

    let output = run(&["--json", "plan", recipe, "--preset", "recommended"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let response = json_stdout(&output);
    assert_eq!(response["command"], "plan");
    assert_eq!(response["ok"], true);
    assert_eq!(response["plan"]["runs"].as_array().unwrap().len(), 2);
}

#[test]
fn unknown_feature_and_input_overrides_are_rejected_by_semantic_id() {
    let recipe = fixture_recipe();
    let recipe = recipe.to_str().unwrap();
    for (flag, value, expected) in [
        ("--feature", "missing-feature=off", "missing-feature"),
        (
            "--input",
            "missing-feature/value=integer:2",
            "missing-feature/value",
        ),
    ] {
        let output = run(&["plan", recipe, "--preset", "recommended", flag, value]);
        assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
        assert!(stderr(&output).contains(expected), "{}", stderr(&output));
    }
}

#[test]
fn path_errors_and_modified_games_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let recipe = recipe_with_profiles(&temp);
    let missing = temp.path().join("not-a-game");
    let inspect = run(&[
        "games",
        "inspect",
        recipe.to_str().unwrap(),
        "bg2ee",
        missing.to_str().unwrap(),
    ]);
    assert_eq!(inspect.status.code(), Some(1), "{}", stderr(&inspect));
    assert!(
        stderr(&inspect).contains(&missing.display().to_string()),
        "{}",
        stderr(&inspect)
    );

    let (_temp, recipe, bg1, bg2, managed, cache) = failed_install_fixture();
    let app_data = managed.parent().unwrap().join("app-data");
    let mut command = cli();
    command
        .env("LOCALAPPDATA", &app_data)
        .args(install_args(&recipe, &bg1, &bg2, &managed, &cache));
    let output = command.output().expect("run guarded install");
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let response = json_stdout(&output);
    assert_eq!(response["error"]["code"], "source_not_fresh");
    let message = response["error"]["message"].as_str().unwrap();
    assert!(message.contains(&bg1.display().to_string()), "{message}");
    assert!(
        message.to_ascii_lowercase().contains("modified"),
        "{message}"
    );
    assert!(!managed.join("bg1").exists());
    assert!(!managed.join("game").exists());
}

#[test]
fn target_lock_contention_is_a_distinct_failure() {
    let (_temp, recipe, bg1, bg2, managed, cache) = failed_install_fixture();
    let app_data = managed.parent().unwrap().join("app-data");
    let lock_root = app_data.join("Chriz BG Collection").join("locks");
    let _lock = TargetLock::try_acquire(&lock_root, &managed).expect("hold target lock");

    let mut command = cli();
    command
        .env("LOCALAPPDATA", &app_data)
        .args(install_args(&recipe, &bg1, &bg2, &managed, &cache));
    let output = command.output().expect("run contending install");
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let response = json_stdout(&output);
    assert_eq!(response["error"]["code"], "target_locked");
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains(&managed.display().to_string()));
}

#[test]
fn source_target_overlap_is_rejected_before_campaign_state_is_created() {
    let (_temp, recipe, bg1, bg2, _managed, cache) = failed_install_fixture();
    let managed = bg1.join("must-not-be-created");
    let app_data = bg1.parent().unwrap().join("app-data");
    let mut command = cli();
    command
        .env("LOCALAPPDATA", &app_data)
        .args(install_args(&recipe, &bg1, &bg2, &managed, &cache));
    let output = command.output().expect("run overlapping install");
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let response = json_stdout(&output);
    assert_eq!(response["error"]["code"], "source_target_overlap");
    assert!(!managed.exists(), "overlap guard wrote into source game");
}

#[test]
fn install_against_an_existing_campaign_is_not_silently_treated_as_resume() {
    let (_temp, recipe, bg1, bg2, managed, cache) = failed_install_fixture();
    let app_data = managed.parent().unwrap().join("app-data");
    let run_install = || {
        let mut command = cli();
        command
            .env("LOCALAPPDATA", &app_data)
            .args(install_args(&recipe, &bg1, &bg2, &managed, &cache));
        command.output().expect("run guarded install")
    };

    let first = run_install();
    assert_eq!(first.status.code(), Some(1), "{}", stderr(&first));
    assert_eq!(json_stdout(&first)["error"]["code"], "source_not_fresh");
    let ledger_before = fs::read_dir(managed.join(".chriz/ledger")).unwrap().count();

    let second = run_install();
    assert_eq!(second.status.code(), Some(1), "{}", stderr(&second));
    assert_eq!(json_stdout(&second)["error"]["code"], "unsafe_target");
    assert_eq!(
        fs::read_dir(managed.join(".chriz/ledger")).unwrap().count(),
        ledger_before,
        "a second install command appended to the existing campaign"
    );
}

#[test]
fn resume_uses_the_frozen_recipe_and_report_and_diagnostics_remain_available() {
    let (_temp, recipe, bg1, bg2, managed, cache) = failed_install_fixture();
    let app_data = managed.parent().unwrap().join("app-data");
    let mut install = cli();
    install
        .env("LOCALAPPDATA", &app_data)
        .args(install_args(&recipe, &bg1, &bg2, &managed, &cache));
    let installed = install.output().expect("start guarded campaign");
    assert_eq!(installed.status.code(), Some(1), "{}", stderr(&installed));
    let installed = json_stdout(&installed);
    let frozen_plan = installed["plan_sha256"].as_str().unwrap().to_owned();

    let collection = recipe.join("collection.toml");
    let newest = fs::read_to_string(&collection)
        .unwrap()
        .replace("eefixpack-bg1", "newest-recipe-run");
    fs::write(&collection, newest).unwrap();

    let mut resume = cli();
    resume
        .env("LOCALAPPDATA", &app_data)
        .args(["--json", "resume", managed.to_str().unwrap()]);
    let resumed = resume.output().expect("resume guarded campaign");
    assert_eq!(resumed.status.code(), Some(1), "{}", stderr(&resumed));
    let resumed = json_stdout(&resumed);
    assert_eq!(resumed["plan_sha256"], frozen_plan);
    assert!(!stdout_from_value(&resumed).contains("newest-recipe-run"));

    let report = run(&["--json", "report", managed.to_str().unwrap()]);
    assert!(report.status.success(), "{}", stderr(&report));
    let report = json_stdout(&report);
    assert_eq!(report["command"], "report");
    assert_eq!(report["receipt"]["plan_sha256"], frozen_plan);

    let output = managed.parent().unwrap().join("diagnostics.zip");
    let diagnostics = run(&[
        "--json",
        "diagnostics",
        managed.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(diagnostics.status.success(), "{}", stderr(&diagnostics));
    assert!(output.is_file());
    assert_eq!(json_stdout(&diagnostics)["command"], "diagnostics");
}

#[test]
fn report_rejects_a_receipt_whose_attempt_directory_or_campaign_identity_is_forged() {
    let (_temp, recipe, bg1, bg2, managed, cache) = failed_install_fixture();
    let app_data = managed.parent().unwrap().join("app-data");
    let mut install = cli();
    install
        .env("LOCALAPPDATA", &app_data)
        .args(install_args(&recipe, &bg1, &bg2, &managed, &cache));
    let installed = install.output().expect("start guarded campaign");
    assert_eq!(installed.status.code(), Some(1), "{}", stderr(&installed));

    let attempts = managed.join(".chriz/attempts");
    let receipt_path = fs::read_dir(&attempts)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("receipt.json"))
        .find(|path| path.is_file())
        .expect("terminal failure receipt");
    let mut receipt: Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).expect("parse real receipt");
    receipt["attempt_id"] = Value::String("different-attempt".to_owned());
    receipt["install_id"] = Value::String("different-install".to_owned());
    receipt["completed_at_millis"] = Value::from(
        receipt["completed_at_millis"]
            .as_u64()
            .unwrap()
            .saturating_add(1_000),
    );
    let forged = attempts.join("forged-attempt");
    fs::create_dir(&forged).unwrap();
    fs::write(
        forged.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();

    let report = run(&["--json", "report", managed.to_str().unwrap()]);
    assert_eq!(report.status.code(), Some(1), "{}", stderr(&report));
    assert_eq!(json_stdout(&report)["error"]["code"], "report_unavailable");
}

fn stdout_from_value(value: &Value) -> String {
    serde_json::to_string(value).expect("serialize parsed CLI response")
}

#[test]
fn clap_rejects_unsafe_or_unknown_bypass_flags_with_usage_exit_code() {
    let recipe = fixture_recipe();
    for flag in ["--assume-build", "--skip", "--skip-failed"] {
        let output = run(&[
            "install",
            recipe.to_str().unwrap(),
            "--preset",
            "recommended",
            "--bg1",
            "x",
            "--bg2",
            "y",
            "--managed-root",
            "z",
            "--cache",
            "c",
            flag,
        ]);
        assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
        assert!(stderr(&output).contains(flag), "{}", stderr(&output));
    }
}
