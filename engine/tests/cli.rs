use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Cursor, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use bg_engine::acquire::{ArtifactCache, DownloadRequest};
use bg_engine::cli::{
    install_campaign_reviewed, resume_campaign_controlled_expected, review_install,
    InstallCommandRequest, SelectionOverrides,
};
use bg_engine::digest::sha256_bytes;
use bg_engine::events::ChannelSink;
use bg_engine::games::{FileSystemProvider, SystemFileSystem};
use bg_engine::lock::TargetLock;
use bg_engine::receipt::{InstallReceipt, ReceiptOutcome, RECEIPT_SCHEMA_VERSION};
use bg_engine::registry::ManagedInstallRegistry;
use bg_engine::session::SessionStore;
use bg_engine::weidu::runner::RunnerControlHandle;
use serde_json::Value;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const FIXTURE_RECIPE: &str = "tests/fixtures/manifest";
const GAME_FIXTURES: &str = "tests/fixtures/games";
const MOCK_CHILD: &str = env!("CARGO_BIN_EXE_mock-child");

struct ArtifactServer {
    base_url: String,
    stop: Arc<AtomicBool>,
    address: std::net::SocketAddr,
    thread: Option<JoinHandle<()>>,
}

impl ArtifactServer {
    fn start(artifacts: BTreeMap<String, Vec<u8>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind artifact server");
        let address = listener.local_addr().expect("artifact server address");
        listener
            .set_nonblocking(true)
            .expect("make artifact server nonblocking");
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((_stream, _)) if thread_stop.load(Ordering::Acquire) => break,
                    Ok((stream, _)) => serve_artifact(stream, &artifacts),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept artifact request: {error}"),
                }
            }
        });
        Self {
            base_url: format!("http://{address}"),
            stop,
            address,
            thread: Some(thread),
        }
    }
}

impl Drop for ArtifactServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect(self.address);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("join artifact server");
        }
    }
}

fn serve_artifact(mut stream: TcpStream, artifacts: &BTreeMap<String, Vec<u8>>) {
    stream
        .set_nonblocking(false)
        .expect("make artifact connection blocking");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("set artifact request timeout");
    let mut lines = BufReader::new(&mut stream).lines();
    let request = lines
        .next()
        .expect("artifact request line")
        .expect("read artifact request");
    let path = request
        .split_ascii_whitespace()
        .nth(1)
        .expect("artifact request path");
    for line in lines {
        if line.expect("read artifact header").is_empty() {
            break;
        }
    }
    let (status, body) = artifacts
        .get(path)
        .map(|body| ("200 OK", body.as_slice()))
        .unwrap_or(("404 Not Found", &[]));
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .expect("write artifact headers");
    stream.write_all(body).expect("write artifact body");
    stream.flush().expect("flush artifact response");
    let _ = stream.shutdown(Shutdown::Both);
}

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (path, bytes) in entries {
        writer.start_file(path, options).expect("start ZIP entry");
        writer.write_all(bytes).expect("write ZIP entry");
    }
    writer.finish().expect("finish ZIP").into_inner()
}

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

fn install_request(fixture: &ExecutableFixture) -> InstallCommandRequest {
    InstallCommandRequest {
        display_name: "Chriz Easy BG".to_owned(),
        recipe: fixture.recipe.clone(),
        preset: "recommended".to_owned(),
        platform: "windows".to_owned(),
        overrides: SelectionOverrides::default(),
        bg1: fixture.bg1.clone(),
        bg2: fixture.bg2.clone(),
        managed_root: fixture.managed.clone(),
        cache: fixture.cache.clone(),
    }
}

#[cfg(windows)]
#[test]
fn display_name_is_validated_and_frozen_into_the_review_identity() {
    let fixture = executable_fixture();
    let first = install_request(&fixture);
    let mut renamed = first.clone();
    renamed.display_name = "My Baldur's Gate".to_owned();

    let first_review = review_install(&first).expect("review the default display name");
    let renamed_review = review_install(&renamed).expect("review the renamed installation");

    assert_ne!(first_review, renamed_review);
    assert_ne!(
        first_review.recipe_payload_sha256,
        renamed_review.recipe_payload_sha256
    );
    assert_eq!(renamed_review.display_name, "My Baldur's Gate");

    for invalid in [
        "",
        "   ",
        "Line\nBreak",
        "Bad<Name",
        "Bad>Name",
        "Bad:Name",
        "Bad\"Name",
        "Bad/Name",
        "Bad\\Name",
        "Bad|Name",
        "Bad?Name",
        "Bad*Name",
    ] {
        let mut request = first.clone();
        request.display_name = invalid.to_owned();
        let error = review_install(&request).expect_err("invalid display name must fail closed");
        assert_eq!(error.code(), "invalid_display_name", "accepted {invalid:?}");
    }

    let mut unicode = first;
    unicode.display_name = "Éowyn’s gemütliches BG! (한글)".to_owned();
    let review = review_install(&unicode).expect("ordinary Unicode and punctuation are valid");
    assert_eq!(review.display_name, unicode.display_name);
}

#[cfg(windows)]
#[test]
fn display_name_cli_default_is_exact_and_custom_name_reaches_the_registry() {
    let help = run(&["install", "--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    assert!(
        stdout(&help).contains("Chriz Easy BG"),
        "install help did not expose the exact default name:\n{}",
        stdout(&help)
    );

    let fixture = executable_fixture();
    let mut command = executable_command(&fixture);
    command
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .args(["--name", "My Baldur's Gate"]);

    let output = command.output().expect("execute renamed synthetic install");
    assert!(output.status.success(), "{}", stderr(&output));
    let replay = SessionStore::open(&fixture.managed)
        .expect("open renamed installation ledger")
        .replay()
        .expect("replay renamed installation ledger");
    let frozen: Value = serde_json::from_slice(&replay.created().recipe_payload)
        .expect("parse frozen CLI identity");
    assert_eq!(frozen["schema"], 2);
    assert_eq!(frozen["display_name"], "My Baldur's Gate");
    let cards =
        ManagedInstallRegistry::open_or_create(&fixture.app_data.join("Chriz BG Collection"))
            .expect("open completed-install registry")
            .list()
            .expect("list completed installations");
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].record.display_name, "My Baldur's Gate");
    assert!(
        cards[0]
            .record
            .engine_name
            .starts_with("My Baldur s Gate - "),
        "save identity did not derive from the frozen display name: {}",
        cards[0].record.engine_name
    );
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

struct ExecutableFixture {
    _temp: TempDir,
    _server: ArtifactServer,
    recipe: PathBuf,
    bg1: PathBuf,
    bg2: PathBuf,
    managed: PathBuf,
    cache: PathBuf,
    app_data: PathBuf,
    documents: PathBuf,
}

fn executable_fixture() -> ExecutableFixture {
    let temp = tempfile::tempdir().expect("create executable CLI fixture root");
    let recipe = recipe_with_profiles(&temp);
    let bg1 = copied_game(&temp, "steam-bgee-sod", "bg1");
    let bg2 = copied_game(&temp, "steam-bg2ee", "bg2");
    install_versioned_fixture_executable(&recipe, &bg1, "steam-bgee-sod.toml");
    install_versioned_fixture_executable(&recipe, &bg2, "steam-bg2ee.toml");
    let managed = temp.path().join("managed");
    let cache = temp.path().join("cache");
    let app_data = temp.path().join("app-data");
    let documents = temp.path().join("Documents");
    for directory in [&cache, &app_data, &documents] {
        fs::create_dir(directory).expect("create executable fixture directory");
    }

    let payload = zip_bytes(&[(
        "EE_Fixpack/EE_Fixpack.tp2",
        b"BEGIN ~Synthetic EE Fixpack~\nDESIGNATED 0\n",
    )]);
    let tool_bytes = fs::read(MOCK_CHILD).expect("read pinned synthetic WeiDU executable");
    let tool_archive = zip_bytes(&[("weidu.exe", &tool_bytes)]);
    let server = ArtifactServer::start(BTreeMap::from([
        ("/eefixpack.zip".to_owned(), payload.clone()),
        ("/weidu.zip".to_owned(), tool_archive.clone()),
    ]));
    let artifact_cache = ArtifactCache::open(&cache).expect("open fixture artifact cache");
    let (sink, _events) = ChannelSink::unbounded();
    for (id, path, bytes) in [
        ("eefixpack", "/eefixpack.zip", payload.as_slice()),
        ("weidu", "/weidu.zip", tool_archive.as_slice()),
    ] {
        artifact_cache
            .acquire(
                &DownloadRequest {
                    request_id: format!("fixture-{id}"),
                    url: format!("{}{path}", server.base_url),
                    expected_length: bytes.len() as u64,
                    expected_sha256: sha256_bytes(bytes),
                    redirect_hosts: Vec::new(),
                    max_attempts: 1,
                },
                &sink,
            )
            .expect("prepopulate verified fixture cache");
    }

    fs::write(
        recipe.join("artifacts/eefixpack.toml"),
        artifact_toml(
            "eefixpack",
            "EE Fixpack",
            "2026.08",
            "https://example.invalid/eefixpack.zip",
            "ee-fixpack.zip",
            &payload,
            &["EE_Fixpack"],
            &["EE_Fixpack/EE_Fixpack.tp2"],
            None,
        ),
    )
    .expect("write synthetic payload contract");
    fs::write(
        recipe.join("artifacts/weidu.toml"),
        artifact_toml(
            "weidu",
            "WeiDU",
            "249.00",
            "https://example.invalid/weidu.zip",
            "weidu.zip",
            &tool_archive,
            &["weidu.exe"],
            &[],
            Some((&tool_bytes, "24900")),
        ),
    )
    .expect("write synthetic tool contract");
    let mod_file = recipe.join("mods/eefixpack.toml");
    let text = fs::read_to_string(&mod_file)
        .expect("read fixture mod")
        .replace(
            "invocation_mode = \"explicit-tp2\"",
            "invocation_mode = \"setup-name\"",
        );
    fs::write(mod_file, text).expect("select setup-name fixture mode");

    ExecutableFixture {
        _temp: temp,
        _server: server,
        recipe,
        bg1,
        bg2,
        managed,
        cache,
        app_data,
        documents,
    }
}

fn install_versioned_fixture_executable(recipe: &Path, game: &Path, profile_name: &str) {
    let system_root = std::env::var_os("SystemRoot").expect("SystemRoot for Windows fixture");
    let executable = PathBuf::from(system_root).join("System32/notepad.exe");
    let version = SystemFileSystem
        .product_version(&executable)
        .expect("read fixture PE ProductVersion");
    let old_bytes = fs::read(game.join("Baldur.exe")).expect("read old fixture executable");
    let new_bytes = fs::read(&executable).expect("read versioned fixture executable");
    fs::write(game.join("Baldur.exe"), &new_bytes).expect("install versioned fixture executable");
    let profile = recipe.join("game-builds").join(profile_name);
    let text = fs::read_to_string(&profile)
        .expect("read fixture profile")
        .replace(
            "product_version = \"2.7.3.0\"",
            &format!("product_version = \"{version}\""),
        )
        .replace(&sha256_bytes(&old_bytes), &sha256_bytes(&new_bytes));
    fs::write(profile, text).expect("write executable fixture profile");
}

#[allow(clippy::too_many_arguments)]
fn artifact_toml(
    id: &str,
    name: &str,
    version: &str,
    url: &str,
    filename: &str,
    archive: &[u8],
    roots: &[&str],
    tp2_paths: &[&str],
    tool: Option<(&[u8], &str)>,
) -> String {
    let roots = roots
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let tp2_paths = tp2_paths
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let tool = tool.map_or_else(String::new, |(bytes, version)| {
        format!(
            "\n[tool]\nexecutable = \"weidu.exe\"\nexpected_length = {}\nsha256 = \"{}\"\nweidu_version = \"{version}\"\npe_machine = \"x86-64\"\n",
            bytes.len(),
            sha256_bytes(bytes)
        )
    });
    format!(
        r#"id = "{id}"
name = "{name}"
version = "{version}"
acquisition = "fetch-only"

[source]
kind = "github-release"
url = "{url}"
reference = "{version}"
expected_filename = "{filename}"
expected_length = {archive_length}
sha256 = "{archive_sha256}"
redirect_hosts = []

[archive]
kind = "zip"
root_rule = "direct"
publish_roots = [{roots}]
tp2_paths = [{tp2_paths}]

[archive.limits]
max_depth = 8
max_entries = 16
max_entry_uncompressed_bytes = {max_entry}
max_total_uncompressed_bytes = {max_total}
max_compression_ratio = 100
{tool}
[provenance]
homepage = "https://example.invalid/{id}"
license = "MIT"
url = "https://example.invalid/{id}/{version}"
reviewed_on = "2026-09-04"
"#,
        archive_length = archive.len(),
        archive_sha256 = sha256_bytes(archive),
        max_entry = archive.len().max(1),
        max_total = archive.len().max(1),
    )
}

fn executable_command(fixture: &ExecutableFixture) -> Command {
    let mut command = cli();
    command
        .env("LOCALAPPDATA", &fixture.app_data)
        .env("CHRIZ_BG_COLLECTION_TEST_DOCUMENTS", &fixture.documents)
        .env("CHRIZ_TEST_MOCK_WEIDU_TP2", "EE_Fixpack/EE_Fixpack.tp2");
    command
}

#[test]
fn help_exposes_the_complete_safe_command_surface_without_bypasses() {
    let output = run(&["--help"]);
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
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
    let text = fs::read_to_string(&collection)
        .unwrap()
        .replace("\r\n", "\n");
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
    assert_eq!(response["events"][0]["type"], "step_started");
    assert_eq!(response["events"][0]["id"], "lock");
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
fn campaign_identity_freezes_payload_archives_and_extracted_tool_bytes_separately() {
    let (_temp, recipe, bg1, bg2, managed, cache) = failed_install_fixture();
    let app_data = managed.parent().unwrap().join("app-data");
    let mut command = cli();
    command
        .env("LOCALAPPDATA", &app_data)
        .args(install_args(&recipe, &bg1, &bg2, &managed, &cache));
    let output = command.output().expect("start guarded campaign");
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));

    let replay = SessionStore::open(&managed).unwrap().replay().unwrap();
    let created = replay.created();
    assert!(
        created
            .artifact_identities
            .iter()
            .all(|identity| identity.length == 1),
        "payload identities must retain archive lengths: {created:#?}"
    );
    let tool_archive = created
        .artifact_identities
        .iter()
        .find(|identity| identity.id == "weidu")
        .expect("WeiDU archive identity must remain frozen for acquisition");
    assert_eq!(tool_archive.version, "249.00");
    assert_eq!(
        tool_archive.sha256,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(created.tool_identities.len(), 1, "{created:#?}");
    let tool = &created.tool_identities[0];
    assert_eq!(tool.id, "weidu");
    assert_eq!(tool.version, "24900");
    assert_eq!(tool.length, 1_364_992);
    assert_eq!(
        tool.sha256,
        "ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a"
    );
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

#[test]
fn report_rejects_a_failed_receipt_missing_a_planned_run_row() {
    let (_temp, recipe, bg1, bg2, managed, cache) = failed_install_fixture();
    let app_data = managed.parent().unwrap().join("app-data");
    let mut install = cli();
    install
        .env("LOCALAPPDATA", &app_data)
        .args(install_args(&recipe, &bg1, &bg2, &managed, &cache));
    let installed = install.output().expect("start guarded campaign");
    assert_eq!(installed.status.code(), Some(1), "{}", stderr(&installed));

    let receipt_path = fs::read_dir(managed.join(".chriz/attempts"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("receipt.json"))
        .find(|path| path.is_file())
        .expect("terminal failure receipt");
    let mut receipt: Value = serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    receipt["runs"]
        .as_array_mut()
        .expect("receipt runs")
        .pop()
        .expect("at least one planned run row");
    fs::write(receipt_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();

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

#[cfg(windows)]
#[test]
fn reviewed_install_rejects_a_recipe_changed_after_review_before_creating_state() {
    let fixture = executable_fixture();
    let request = install_request(&fixture);
    let review = review_install(&request).expect("freeze install review identity");
    let artifact = fixture.recipe.join("artifacts/eefixpack.toml");
    let text = fs::read_to_string(&artifact).expect("read reviewed artifact");
    fs::write(
        &artifact,
        text.replace("name = \"EE Fixpack\"", "name = \"Changed after Review\""),
    )
    .expect("change valid recipe metadata after Review");
    let (sink, _events) = ChannelSink::unbounded();

    let error = install_campaign_reviewed(&request, &review, &sink, &RunnerControlHandle::new())
        .expect_err("changed reviewed recipe must not execute");

    assert_eq!(error.code(), "review_changed");
    assert!(!fixture.managed.exists());
}

#[cfg(windows)]
#[test]
fn resume_rejects_a_requested_install_id_that_does_not_match_the_ledger() {
    let fixture = executable_fixture();
    let mut command = executable_command(&fixture);
    command.args(install_args(
        &fixture.recipe,
        &fixture.bg1,
        &fixture.bg2,
        &fixture.managed,
        &fixture.cache,
    ));
    let installed = command.output().expect("execute synthetic install");
    assert!(installed.status.success(), "{}", stderr(&installed));
    let (sink, _events) = ChannelSink::unbounded();

    let error = resume_campaign_controlled_expected(
        &fixture.managed,
        "install-not-the-ledger-owner",
        &sink,
        &RunnerControlHandle::new(),
    )
    .expect_err("wrong install id must not resume this managed path");

    assert_eq!(error.code(), "resume_identity_mismatch");
}

#[cfg(windows)]
#[test]
fn install_executes_the_frozen_recipe_and_publishes_complete_durable_evidence() {
    let fixture = executable_fixture();
    let mut command = executable_command(&fixture);
    command.args(install_args(
        &fixture.recipe,
        &fixture.bg1,
        &fixture.bg2,
        &fixture.managed,
        &fixture.cache,
    ));

    let output = command.output().expect("execute synthetic install");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let response = json_stdout(&output);
    assert_eq!(response["ok"], true);
    assert_eq!(response["status"]["status"], "complete");
    assert!(response["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["type"] == "console_line"));

    let receipt_path = fixture.managed.join(".chriz/install-receipt.json");
    let receipt: InstallReceipt =
        serde_json::from_slice(&fs::read(&receipt_path).expect("durable success receipt"))
            .expect("parse durable success receipt");
    assert_eq!(receipt.schema_version, RECEIPT_SCHEMA_VERSION);
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    assert_eq!(receipt.plan.runs.len(), 2);
    assert_eq!(receipt.runs.len(), receipt.plan.runs.len());
    assert_eq!(receipt.artifacts.len(), 2);
    assert_eq!(receipt.weidu_tools.len(), 1);
    assert!(receipt.final_state.is_some());
    assert_eq!(receipt.runs[0].components, vec![0, 2]);
    assert_eq!(receipt.runs[0].attempts.len(), 1);
    assert_eq!(receipt.runs[0].attempts[0].components, vec![0, 2]);
    assert_eq!(
        receipt.runs[0].attempts[0]
            .log_diff
            .added
            .iter()
            .map(|entry| entry.component)
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    assert!(fixture
        .managed
        .join("bg1/EE_Fixpack/EE_Fixpack.tp2")
        .is_file());
    assert!(fixture
        .managed
        .join("game/EE_Fixpack/EE_Fixpack.tp2")
        .is_file());
    for run in &receipt.runs {
        for attempt in &run.attempts {
            for digest in [
                &attempt.invocation_sha256,
                &attempt.stdout_sha256,
                &attempt.stderr_sha256,
                &attempt.debug_sha256,
                &attempt.log_diff.before_sha256,
                &attempt.log_diff.after_sha256,
            ] {
                assert_eq!(digest.len(), 64, "missing digest in {attempt:#?}");
            }
        }
    }
}

#[cfg(windows)]
#[test]
fn display_name_and_recipe_are_both_frozen_across_a_failed_install_resume() {
    let fixture = executable_fixture();
    let marker = fixture._temp.path().join("mock-weidu-failed-once");
    let mut install = executable_command(&fixture);
    install
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .args(["--name", "Resumed BG"]);

    let failed = install.output().expect("execute first synthetic attempt");

    assert_eq!(failed.status.code(), Some(1), "{}", stderr(&failed));
    let failed = json_stdout(&failed);
    assert_eq!(failed["error"]["code"], "campaign_failed");
    assert_eq!(failed["status"]["step_id"], "install:eefixpack-bg1");
    assert!(failed["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["type"] == "console_line"));
    let frozen_plan = failed["plan_sha256"].as_str().unwrap().to_owned();
    let failed_receipt = fs::read_dir(fixture.managed.join(".chriz/attempts"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("receipt.json"))
        .find(|path| path.is_file())
        .expect("failed-attempt receipt");
    let failed_receipt: InstallReceipt =
        serde_json::from_slice(&fs::read(failed_receipt).unwrap()).unwrap();
    assert_eq!(failed_receipt.runs.len(), failed_receipt.plan.runs.len());
    assert_eq!(failed_receipt.runs[0].attempts.len(), 1);
    assert!(failed_receipt.runs[0].attempts[0].log_diff.added.is_empty());
    assert!(failed_receipt.runs[1].attempts.is_empty());

    fs::write(
        fixture.recipe.join("collection.toml"),
        "this current recipe is deliberately invalid\n",
    )
    .expect("mutate current recipe after campaign freeze");
    let mut resume = executable_command(&fixture);
    resume
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(["--json", "resume", fixture.managed.to_str().unwrap()]);

    let resumed = resume.output().expect("resume synthetic campaign");

    assert!(
        resumed.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&resumed),
        stderr(&resumed)
    );
    let resumed = json_stdout(&resumed);
    assert_eq!(resumed["plan_sha256"], frozen_plan);
    assert!(resumed["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["type"] == "campaign_started" && event["resumed"] == true));
    let receipt: InstallReceipt = serde_json::from_slice(
        &fs::read(fixture.managed.join(".chriz/install-receipt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt.plan_sha256, frozen_plan);
    assert_eq!(receipt.runs[0].attempts.len(), 2);
    assert_eq!(receipt.runs[0].attempts[0].attempt, 1);
    assert_eq!(receipt.runs[0].attempts[1].attempt, 2);
    assert_eq!(receipt.runs[0].attempts[0].components, vec![0, 2]);
    assert_eq!(receipt.runs[0].attempts[1].components, vec![0, 2]);
    assert!(receipt.runs[0].attempts[0].log_diff.added.is_empty());
    assert_eq!(
        receipt.runs[0].attempts[1]
            .log_diff
            .added
            .iter()
            .map(|entry| entry.component)
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    let cards =
        ManagedInstallRegistry::open_or_create(&fixture.app_data.join("Chriz BG Collection"))
            .expect("open resumed-install registry")
            .list()
            .expect("list resumed installation");
    assert_eq!(cards[0].record.display_name, "Resumed BG");
}

#[cfg(windows)]
#[test]
fn completed_mock_weidu_without_process_result_is_proven_on_resume_without_rerunning() {
    let fixture = executable_fixture();
    let mut install = executable_command(&fixture);
    install
        .env("CHRIZ_BG_COLLECTION_TEST_INTERRUPT_AFTER_WEIDU", "1")
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ));

    let interrupted = install
        .output()
        .expect("execute interrupted synthetic attempt");

    assert_eq!(
        interrupted.status.code(),
        Some(1),
        "{}",
        stderr(&interrupted)
    );
    let interrupted = json_stdout(&interrupted);
    assert_eq!(interrupted["error"]["code"], "campaign_failed");
    assert!(stdout_from_value(&interrupted).contains("simulated interruption after WeiDU exit"));
    let replay = SessionStore::open(&fixture.managed)
        .expect("open interrupted campaign")
        .replay()
        .expect("replay interrupted campaign");
    let steps_root = fixture
        .managed
        .join(".chriz/attempts")
        .join(&replay.created().attempt_id)
        .join("steps");
    let attempt_root = fs::read_dir(&steps_root)
        .expect("read interrupted step evidence")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("attempt-0001"))
        .find(|path| path.join("weidu.debug.log").is_file())
        .expect("find interrupted WeiDU attempt evidence");
    assert!(attempt_root.join("weidu.debug.log").is_file());
    assert!(!attempt_root.join("process-result.json").exists());
    assert_eq!(
        fs::read_to_string(fixture.managed.join("bg1/WeiDU.log"))
            .expect("committed synthetic WeiDU log")
            .lines()
            .count(),
        2
    );

    fs::write(
        fixture.recipe.join("collection.toml"),
        "the current recipe is invalid after the frozen campaign\n",
    )
    .expect("mutate current recipe after interruption");
    let mut resume = executable_command(&fixture);
    resume.args(["--json", "resume", fixture.managed.to_str().unwrap()]);

    let resumed = resume
        .output()
        .expect("resume interrupted synthetic campaign");

    assert!(
        resumed.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&resumed),
        stderr(&resumed)
    );
    let receipt: InstallReceipt = serde_json::from_slice(
        &fs::read(fixture.managed.join(".chriz/install-receipt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt.runs.len(), 2);
    assert_eq!(receipt.runs[0].attempts.len(), 1);
    assert_eq!(receipt.runs[0].attempts[0].exit_code, -1);
    assert_eq!(
        receipt.runs[0].attempts[0]
            .log_diff
            .added
            .iter()
            .map(|entry| entry.component)
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
}

#[cfg(windows)]
#[test]
fn explicit_relative_tp2_stays_unavailable_without_signed_recipe_evidence() {
    let fixture = executable_fixture();
    let mod_file = fixture.recipe.join("mods/eefixpack.toml");
    let text = fs::read_to_string(&mod_file)
        .expect("read setup-name fixture mod")
        .replace(
            "invocation_mode = \"setup-name\"",
            "invocation_mode = \"explicit-tp2\"",
        );
    fs::write(mod_file, text).expect("select unsupported explicit TP2 fixture mode");
    let mut command = executable_command(&fixture);
    command.args(install_args(
        &fixture.recipe,
        &fixture.bg1,
        &fixture.bg2,
        &fixture.managed,
        &fixture.cache,
    ));

    let output = command.output().expect("execute explicit TP2 campaign");

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let response = json_stdout(&output);
    assert_eq!(response["error"]["code"], "campaign_failed");
    assert!(stdout_from_value(&response).contains(
        "explicit-relative-TP2 invocation is unavailable until the signed recipe schema records separately reviewed compatibility evidence"
    ));
}

#[cfg(windows)]
#[test]
fn eeex_selected_install_cannot_complete_without_infinity_loader() {
    let fixture = executable_fixture();
    let old_artifact_file = fixture.recipe.join("artifacts/eefixpack.toml");
    let new_artifact_file = fixture.recipe.join("artifacts/eeex.toml");
    let artifact_text = fs::read_to_string(&old_artifact_file)
        .expect("read synthetic artifact")
        .replace("id = \"eefixpack\"", "id = \"eeex\"");
    fs::write(&new_artifact_file, artifact_text).expect("write synthetic EEex artifact");
    fs::remove_file(old_artifact_file).expect("remove old synthetic artifact id");
    let old_mod_file = fixture.recipe.join("mods/eefixpack.toml");
    let new_mod_file = fixture.recipe.join("mods/eeex.toml");
    let mod_text = fs::read_to_string(&old_mod_file)
        .expect("read synthetic installer")
        .replace("id = \"eefixpack\"", "id = \"eeex\"");
    fs::write(&new_mod_file, mod_text).expect("write synthetic EEex installer");
    fs::remove_file(old_mod_file).expect("remove old synthetic installer id");
    let collection = fixture.recipe.join("collection.toml");
    let collection_text = fs::read_to_string(&collection)
        .expect("read synthetic collection")
        .replace("mod_id = \"eefixpack\"", "mod_id = \"eeex\"");
    fs::write(collection, collection_text).expect("select synthetic EEex installer");
    let mut command = executable_command(&fixture);
    command.args(install_args(
        &fixture.recipe,
        &fixture.bg1,
        &fixture.bg2,
        &fixture.managed,
        &fixture.cache,
    ));

    let output = command.output().expect("execute synthetic EEex campaign");

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let response = json_stdout(&output);
    assert_eq!(
        response["status"]["step_id"], "verify:final",
        "{response:#}"
    );
    assert!(stdout_from_value(&response)
        .contains("EEex is selected but InfinityLoader.exe is unavailable"));
}

#[cfg(windows)]
#[test]
fn missing_authored_prompt_cannot_publish_a_successful_install() {
    let fixture = executable_fixture();
    let mod_file = fixture.recipe.join("mods/eefixpack.toml");
    let text = fs::read_to_string(&mod_file)
        .expect("read synthetic installer")
        .replace(
            "name = \"Core fixes\"",
            "name = \"Core fixes\"\nprompts = [{ expected_output = \"Never emitted: \", answer = { kind = \"literal\", value = { kind = \"boolean\", value = true } } }]",
        );
    fs::write(&mod_file, text).expect("author synthetic missing prompt");
    let run = || {
        let mut command = executable_command(&fixture);
        command.args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ));
        command.output().expect("execute missing-prompt campaign")
    };

    let output = run();

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let response = json_stdout(&output);
    assert_eq!(
        response["status"]["step_id"], "install:eefixpack-bg1",
        "{response:#}"
    );
    assert!(stdout_from_value(&response).contains("WeiDU could not be supervised"));
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    let steps = fixture
        .managed
        .join(".chriz/attempts")
        .join(&replay.created().attempt_id)
        .join("steps");
    let prompt_files = fs::read_dir(steps)
        .unwrap()
        .filter_map(Result::ok)
        .flat_map(|step| {
            fs::read_dir(step.path())
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(|attempt| attempt.path().join("prompt-results.jsonl"))
        })
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    assert_eq!(prompt_files.len(), 1);
    assert_eq!(fs::metadata(&prompt_files[0]).unwrap().len(), 0);

    let mut resume = executable_command(&fixture);
    resume.args(["--json", "resume", fixture.managed.to_str().unwrap()]);
    let output = resume.output().expect("resume missing-prompt campaign");
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let response = json_stdout(&output);
    assert_eq!(
        response["status"]["status"], "fresh_copy_required",
        "{response:#}"
    );
    assert!(stdout_from_value(&response)
        .contains("did not durably receive every authored prompt answer"));
}

#[cfg(windows)]
#[test]
fn missing_manual_archive_is_reported_as_a_persistent_machine_readable_action() {
    let fixture = executable_fixture();
    let artifact = fixture.recipe.join("artifacts/eefixpack.toml");
    let text = fs::read_to_string(&artifact)
        .unwrap()
        .replace(
            "acquisition = \"fetch-only\"",
            "acquisition = \"manual-user-supplied\"",
        )
        .replace("kind = \"github-release\"", "kind = \"manual\"");
    fs::write(artifact, text).unwrap();
    let mut command = executable_command(&fixture);
    command.args(install_args(
        &fixture.recipe,
        &fixture.bg1,
        &fixture.bg2,
        &fixture.managed,
        &fixture.cache,
    ));

    let output = command.output().expect("execute manual-artifact campaign");

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let response = json_stdout(&output);
    let manual = response["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["type"] == "manual_download_needed")
        .expect("manual download event retained in JSON failure");
    assert_eq!(manual["mod_id"], "eefixpack");
    assert!(manual["drop_dir"]
        .as_str()
        .unwrap()
        .ends_with("cache\\manual"));
}
