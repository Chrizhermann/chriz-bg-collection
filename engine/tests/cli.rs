use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Cursor, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
use bg_engine::receipt::{
    ArtifactCacheOutcome, ArtifactReceipt, InstallReceipt, ReceiptOutcome, RECEIPT_SCHEMA_VERSION,
};
use bg_engine::registry::{CampaignAvailability, ManagedInstallRegistry};
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
        let signatures = AtomicU64::new(0);
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((_stream, _)) if thread_stop.load(Ordering::Acquire) => break,
                    Ok((stream, _)) => serve_artifact(stream, &artifacts, &signatures),
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

/// Serves exact artifact paths. `/signed/<path>` mimics a GitHub release download: it
/// redirects to `<path>` with a fresh single-use signature, so every transfer of the same
/// bytes reports a different final URL.
fn serve_artifact(
    mut stream: TcpStream,
    artifacts: &BTreeMap<String, Vec<u8>>,
    signatures: &AtomicU64,
) {
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
    if let Some(signed) = path.strip_prefix("/signed") {
        let signature = signatures.fetch_add(1, Ordering::AcqRel) + 1;
        write!(
            stream,
            "HTTP/1.1 302 Found
Location: {signed}?signature={signature}
Content-Length: 0
Connection: close

"
        )
        .expect("write artifact redirect");
        stream.flush().expect("flush artifact redirect");
        let _ = stream.shutdown(Shutdown::Both);
        return;
    }
    let path = path.split_once('?').map_or(path, |(path, _query)| path);
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

fn fixture_tempdir() -> TempDir {
    // CLI errors report canonical paths. Resolve the existing parent first so
    // temporary-path aliases and separators also agree for not-yet-created children.
    let parent = fs::canonicalize(std::env::temp_dir()).expect("canonicalize CLI temp parent");
    tempfile::tempdir_in(parent).expect("create CLI fixture root")
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
        application_version: None,
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
    let temp = fixture_tempdir();
    let recipe = recipe_with_profiles(&temp);
    use_production_invocation_mode(&recipe);
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
    payload: Vec<u8>,
    tool_archive: Vec<u8>,
}

fn executable_fixture() -> ExecutableFixture {
    let temp = fixture_tempdir();
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
    use_production_invocation_mode(&recipe);

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
        payload,
        tool_archive,
    }
}

fn use_production_invocation_mode(recipe: &Path) {
    // The shared authoring fixture deliberately exercises explicit-TP2 parsing. Tests
    // entering the production installer need its supported setup-name mode instead.
    let mod_file = recipe.join("mods/eefixpack.toml");
    let text = fs::read_to_string(&mod_file).expect("read fixture mod");
    assert!(text.contains("invocation_mode = \"explicit-tp2\""));
    fs::write(
        mod_file,
        text.replace(
            "invocation_mode = \"explicit-tp2\"",
            "invocation_mode = \"setup-name\"",
        ),
    )
    .expect("select supported production fixture mode");
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

#[cfg(windows)]
#[test]
fn manual_checkpoints_resume_two_boundaries_and_finalize_with_dragonspear_history_intact() {
    let fixture = executable_fixture();
    let mut install = executable_command(&fixture);
    install
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .args([
            "--manual-pause-after",
            "eefixpack-bg1:0",
            "--manual-pause-after",
            "eefixpack-bg2:0",
        ]);
    let first = install.output().unwrap();
    assert!(
        first.status.success(),
        "{}\n{}",
        stdout(&first),
        stderr(&first)
    );
    let first = json_stdout(&first);
    assert_eq!(first["status"]["status"], "paused");
    let install_id = first["install_id"].as_str().unwrap();
    let frozen_path = fixture.managed.join(".chriz/recipe/payload.zip");
    let original_recipe = fs::read(&frozen_path).unwrap();
    assert_eq!(
        fs::read_to_string(fixture.managed.join("bg1/WeiDU.log"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    assert!(!fixture.managed.join(".chriz/install-receipt.json").exists());

    // Reopening/resuming without explicit acceptance must preserve the pending pause.
    let reopened = executable_command(&fixture)
        .args(["--json", "resume", fixture.managed.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(reopened.status.success(), "{}", stdout(&reopened));
    assert_eq!(json_stdout(&reopened)["status"]["status"], "paused");
    for index in 0..2 {
        if index == 1 {
            let directory = fixture.managed.join("game/DragonspearUI");
            fs::create_dir(&directory).unwrap();
            fs::write(
                directory.join("setup-DragonspearUI.tp2"),
                "BACKUP ~DragonspearUI/backup~\nAUTHOR ~Fixture~\n",
            )
            .unwrap();
            let path = fixture.managed.join("game/WeiDU.log");
            let before = fs::read_to_string(&path).unwrap();
            fs::write(path, format!("{before}~DragonspearUI/setup-DragonspearUI.tp2~ #0 #0 // Dragonspear UI++: reported version\n")).unwrap();
            fs::write(
                fixture.managed.join("game/SETUP-DRAGONSPEARUI.DEBUG"),
                "SUCCESSFULLY INSTALLED Dragonspear UI++\n",
            )
            .unwrap();
        }
        let checked = executable_command(&fixture)
            .args([
                "--json",
                "check-manual-checkpoint",
                fixture.managed.to_str().unwrap(),
                "--install-id",
                install_id,
            ])
            .output()
            .unwrap();
        assert!(
            checked.status.success(),
            "{}\n{}",
            stdout(&checked),
            stderr(&checked)
        );
        let checkpoint = json_stdout(&checked)["checkpoint"].clone();
        assert_eq!(checkpoint["target"], if index == 0 { "bg1" } else { "bg2" });
        assert_eq!(checkpoint["additions"].as_array().unwrap().len(), index);
        // A stale/forged review token never records acceptance.
        let denied = executable_command(&fixture)
            .args([
                "--json",
                "accept-manual-checkpoint",
                fixture.managed.to_str().unwrap(),
                "--install-id",
                install_id,
                "--review-token",
                "stale",
                "--accept-additions",
            ])
            .output()
            .unwrap();
        assert!(!denied.status.success());
        let mut accept = executable_command(&fixture);
        accept.args([
            "--json",
            "accept-manual-checkpoint",
            fixture.managed.to_str().unwrap(),
            "--install-id",
            install_id,
            "--review-token",
            checkpoint["reviewToken"].as_str().unwrap(),
        ]);
        if index == 1 {
            accept.arg("--accept-additions");
        }
        let accepted = accept.output().unwrap();
        assert!(
            accepted.status.success(),
            "{}\n{}",
            stdout(&accepted),
            stderr(&accepted)
        );
        let resumed = executable_command(&fixture)
            .args(["--json", "resume", fixture.managed.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            resumed.status.success(),
            "{}\n{}",
            stdout(&resumed),
            stderr(&resumed)
        );
        assert_eq!(
            json_stdout(&resumed)["status"]["status"],
            if index == 0 { "paused" } else { "complete" }
        );
    }
    assert_eq!(fs::read(&frozen_path).unwrap(), original_recipe);
    let receipt: InstallReceipt = serde_json::from_slice(
        &fs::read(fixture.managed.join(".chriz/install-receipt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    assert_eq!(receipt.runs.len(), 3, "only the requested batch was split");
    assert!(receipt.final_state.unwrap().logs.iter().any(|log| log
        .components
        .iter()
        .any(|entry| entry.tp2.contains("dragonspearui"))));
    assert!(fs::read_to_string(fixture.managed.join("game/WeiDU.log"))
        .unwrap()
        .contains("DragonspearUI"));
    assert!(bg_engine::manual_checkpoint::game_update_guard(&fixture.managed).is_err());
    let report = executable_command(&fixture)
        .args(["--json", "report", fixture.managed.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        report.status.success(),
        "{}\n{}",
        stdout(&report),
        stderr(&report)
    );
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
    let temp = fixture_tempdir();
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
    let temp = fixture_tempdir();
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
        ("--group", "missing-group=on", "missing-group"),
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

/// Fixture recipe with three optional `challenge-mode` members owning every fixture component.
fn recipe_with_feature_group(temp: &TempDir) -> PathBuf {
    let recipe = temp.path().join("recipe");
    copy_tree(&fixture_recipe(), &recipe);
    let collection = recipe.join("collection.toml");
    let mut text = fs::read_to_string(&collection).unwrap();
    text.push_str(
        r#"

[[feature_groups]]
id = "challenge-mode"
title = "Challenge Mode"
description = "Fixture challenge rules."
members = ["challenge-a", "challenge-b", "challenge-c"]

[[features]]
id = "challenge-a"
title = "Challenge A"
description = "Fixture challenge A"
category = "difficulty"
decision = "optional"
readiness = "ready"
components = [{ run_id = "eefixpack-bg1", component = 0 }]

[[features]]
id = "challenge-b"
title = "Challenge B"
description = "Fixture challenge B"
category = "difficulty"
decision = "optional"
readiness = "ready"
components = [{ run_id = "eefixpack-bg1", component = 2 }]

[[features]]
id = "challenge-c"
title = "Challenge C"
description = "Fixture challenge C"
category = "difficulty"
decision = "optional"
readiness = "ready"
components = [{ run_id = "eefixpack-bg2", component = 0 }]
"#,
    );
    fs::write(collection, text).unwrap();
    recipe
}

fn planned_components(response: &Value) -> Vec<(String, Vec<u64>)> {
    response["plan"]["runs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|run| {
            (
                run["run_id"].as_str().unwrap().to_owned(),
                run["components"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|component| component.as_u64().unwrap())
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn plan_group_switch_expands_members_before_feature_overrides() {
    let temp = fixture_tempdir();
    let recipe = recipe_with_feature_group(&temp);
    let recipe = recipe.to_str().unwrap();
    let plan = |extra: &[&str]| {
        let mut args = vec!["--json", "plan", recipe, "--preset", "recommended"];
        args.extend_from_slice(extra);
        let output = run(&args);
        assert!(output.status.success(), "{extra:?}: {}", stderr(&output));
        json_stdout(&output)
    };

    let off = plan(&[]);
    assert!(planned_components(&off).is_empty());
    assert_eq!(off["groups"][0]["id"], "challenge-mode");
    assert_eq!(off["groups"][0]["state"], "off");

    let on = plan(&["--group", "challenge-mode=on"]);
    assert_eq!(
        planned_components(&on),
        vec![
            ("eefixpack-bg1".to_owned(), vec![0, 2]),
            ("eefixpack-bg2".to_owned(), vec![0]),
        ]
    );
    assert_eq!(on["groups"][0]["state"], "on");
    assert_eq!(on["normalized_selection"]["features"]["challenge-a"], true);
    assert!(on["normalized_selection"]["features"]
        .get("challenge-mode")
        .is_none());

    // `--feature` wins over the group for one member, wherever it appears on the command line.
    for args in [
        [
            "--group",
            "challenge-mode=on",
            "--feature",
            "challenge-b=off",
        ],
        [
            "--feature",
            "challenge-b=off",
            "--group",
            "challenge-mode=on",
        ],
    ] {
        let mixed = plan(&args);
        assert_eq!(
            planned_components(&mixed),
            vec![
                ("eefixpack-bg1".to_owned(), vec![0]),
                ("eefixpack-bg2".to_owned(), vec![0]),
            ],
            "{args:?}"
        );
        assert_eq!(mixed["groups"][0]["state"], "mixed");
        assert_eq!(
            mixed["groups"][0]["selected_members"],
            serde_json::json!(["challenge-a", "challenge-c"])
        );
    }

    let kept = plan(&[
        "--group",
        "challenge-mode=off",
        "--feature",
        "challenge-c=on",
    ]);
    assert_eq!(
        planned_components(&kept),
        vec![("eefixpack-bg2".to_owned(), vec![0])]
    );

    // Later group switches see earlier ones.
    let toggled = plan(&[
        "--group",
        "challenge-mode=on",
        "--group",
        "challenge-mode=off",
    ]);
    assert!(planned_components(&toggled).is_empty());

    let human = run(&[
        "plan",
        recipe,
        "--preset",
        "recommended",
        "--group",
        "challenge-mode=on",
    ]);
    assert!(human.status.success(), "{}", stderr(&human));
    assert!(
        stdout(&human)
            .contains("Group challenge-mode (Challenge Mode): on, 3 of 3 available selected"),
        "{}",
        stdout(&human)
    );
}

/// Fixture recipe whose preset switches the default `gate` off, with `challenge-c` requiring
/// `gate`; `members` names the `challenge-mode` members among `challenge-a` and `challenge-c`.
fn recipe_with_gated_group(temp: &TempDir, members: &[&str]) -> PathBuf {
    let recipe = temp.path().join("recipe");
    copy_tree(&fixture_recipe(), &recipe);
    let collection = recipe.join("collection.toml");
    let mut text = fs::read_to_string(&collection).unwrap();
    let members = members
        .iter()
        .map(|member| format!("{member:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    text.push_str(&format!(
        r#"

[[feature_groups]]
id = "challenge-mode"
title = "Challenge Mode"
description = "Fixture challenge rules."
members = [{members}]

[[features]]
id = "gate"
title = "Gate"
description = "Fixture prerequisite"
category = "difficulty"
decision = "default"
readiness = "ready"
components = [{{ run_id = "eefixpack-bg1", component = 0 }}]

[[features]]
id = "challenge-a"
title = "Challenge A"
description = "Fixture challenge A"
category = "difficulty"
decision = "optional"
readiness = "ready"
components = [{{ run_id = "eefixpack-bg1", component = 2 }}]

[[features]]
id = "challenge-c"
title = "Challenge C"
description = "Fixture challenge C"
category = "difficulty"
decision = "optional"
readiness = "ready"
requires = ["gate"]
components = [{{ run_id = "eefixpack-bg2", component = 0 }}]
"#
    ));
    fs::write(collection, text).unwrap();
    let preset = recipe.join("presets/recommended.toml");
    let mut text = fs::read_to_string(&preset).unwrap();
    text.push_str("\n[selections]\ngate = \"off\"\n");
    fs::write(preset, text).unwrap();
    recipe
}

#[test]
fn plan_group_switch_sees_an_explicit_prerequisite_override() {
    let temp = fixture_tempdir();
    let recipe = recipe_with_gated_group(&temp, &["challenge-a", "challenge-c"]);
    let recipe = recipe.to_str().unwrap();
    let plan = |extra: &[&str]| {
        let mut args = vec!["--json", "plan", recipe, "--preset", "recommended"];
        args.extend_from_slice(extra);
        let output = run(&args);
        assert!(output.status.success(), "{extra:?}: {}", stderr(&output));
        json_stdout(&output)
    };

    // Without the prerequisite the group selects only the available member.
    let gated = plan(&["--group", "challenge-mode=on"]);
    assert_eq!(
        planned_components(&gated),
        vec![("eefixpack-bg1".to_owned(), vec![2])]
    );

    // An explicit prerequisite override is visible to the group, wherever it appears.
    for args in [
        ["--group", "challenge-mode=on", "--feature", "gate=on"],
        ["--feature", "gate=on", "--group", "challenge-mode=on"],
    ] {
        let on = plan(&args);
        assert_eq!(
            planned_components(&on),
            vec![
                ("eefixpack-bg1".to_owned(), vec![0, 2]),
                ("eefixpack-bg2".to_owned(), vec![0]),
            ],
            "{args:?}"
        );
        assert_eq!(on["groups"][0]["state"], "on", "{args:?}");
        assert_eq!(
            on["normalized_selection"]["features"]["challenge-c"], true,
            "{args:?}"
        );
    }

    // An explicit member choice still wins over the group.
    let member_off = plan(&[
        "--group",
        "challenge-mode=on",
        "--feature",
        "gate=on",
        "--feature",
        "challenge-c=off",
    ]);
    assert_eq!(
        planned_components(&member_off),
        vec![("eefixpack-bg1".to_owned(), vec![0, 2])]
    );
    assert_eq!(member_off["groups"][0]["state"], "mixed");
}

#[test]
fn plan_group_whose_only_member_needs_an_explicit_prerequisite_turns_on() {
    let temp = fixture_tempdir();
    let recipe = recipe_with_gated_group(&temp, &["challenge-c"]);
    let recipe = recipe.to_str().unwrap();

    let disabled = run(&[
        "--json",
        "plan",
        recipe,
        "--preset",
        "recommended",
        "--group",
        "challenge-mode=on",
    ]);
    assert_eq!(disabled.status.code(), Some(1), "{}", stderr(&disabled));
    assert_eq!(json_stdout(&disabled)["error"]["code"], "invalid_selection");

    let output = run(&[
        "--json",
        "plan",
        recipe,
        "--preset",
        "recommended",
        "--group",
        "challenge-mode=on",
        "--feature",
        "gate=on",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let on = json_stdout(&output);
    assert_eq!(
        planned_components(&on),
        vec![
            ("eefixpack-bg1".to_owned(), vec![0]),
            ("eefixpack-bg2".to_owned(), vec![0]),
        ]
    );
    assert_eq!(on["groups"][0]["state"], "on");
}

#[test]
fn install_group_switch_sees_an_explicit_prerequisite_override() {
    // `install` shares the plan's selection expansion. With the prerequisite switched on, the
    // group's only member is available, so selection and planning succeed and install reaches
    // the source-game freshness guard, which the deliberately modified BG1 copy then trips.
    let temp = fixture_tempdir();
    let recipe = recipe_with_gated_group(&temp, &["challenge-c"]);
    use_production_invocation_mode(&recipe);
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(GAME_FIXTURES)
            .join("profiles"),
        &recipe.join("game-builds"),
    );
    let bg1 = copied_game(&temp, "steam-bgee-sod", "bg1");
    let bg2 = copied_game(&temp, "steam-bg2ee", "bg2");
    fs::write(bg1.join("setup-dirty.exe"), b"modified").expect("add modified-game residue");
    let cache = temp.path().join("cache");
    fs::create_dir(&cache).expect("create cache root");
    // Each attempt gets its own managed root and app data, so one guarded attempt's campaign
    // state or registry entry cannot affect another.
    let install = |attempt: &str, extra: &[&str]| {
        let managed = temp.path().join(format!("managed-{attempt}"));
        let app_data = temp.path().join(format!("app-data-{attempt}"));
        fs::create_dir(&app_data).expect("create app-data root");
        let mut args = install_args(&recipe, &bg1, &bg2, &managed, &cache);
        args.extend_from_slice(extra);
        let output = cli()
            .env("LOCALAPPDATA", &app_data)
            .args(&args)
            .output()
            .expect("run guarded install");
        assert_eq!(
            output.status.code(),
            Some(1),
            "{extra:?}: {}",
            stderr(&output)
        );
        assert!(!managed.join("bg1").exists(), "{extra:?}");
        assert!(!managed.join("game").exists(), "{extra:?}");
        json_stdout(&output)
    };

    let disabled = install("gated", &["--group", "challenge-mode=on"]);
    assert_eq!(disabled["error"]["code"], "invalid_selection", "{disabled}");
    assert!(
        disabled["error"]["message"]
            .as_str()
            .unwrap()
            .contains("cannot be turned on"),
        "{disabled}"
    );

    for (attempt, args) in [
        (
            "group-first",
            ["--group", "challenge-mode=on", "--feature", "gate=on"],
        ),
        (
            "feature-first",
            ["--feature", "gate=on", "--group", "challenge-mode=on"],
        ),
    ] {
        let enabled = install(attempt, &args);
        assert_eq!(
            enabled["error"]["code"], "source_not_fresh",
            "{args:?}: {enabled}"
        );
    }
}

#[test]
fn unknown_or_malformed_group_switches_are_rejected() {
    let temp = fixture_tempdir();
    let recipe = recipe_with_feature_group(&temp);
    let recipe = recipe.to_str().unwrap();
    let output = run(&[
        "plan",
        recipe,
        "--preset",
        "recommended",
        "--group",
        "hard-mode=on",
    ]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let error = stderr(&output);
    assert!(
        error.contains("unknown feature group \"hard-mode\""),
        "{error}"
    );
    assert!(error.contains("valid groups: challenge-mode"), "{error}");

    for value in [
        "challenge-mode=true",
        "challenge-mode=mixed",
        "challenge-mode",
    ] {
        let output = run(&["plan", recipe, "--preset", "recommended", "--group", value]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{value}: {}",
            stderr(&output)
        );
        assert!(
            stderr(&output).contains("group override"),
            "{}",
            stderr(&output)
        );
    }

    let output = run(&[
        "--json",
        "plan",
        recipe,
        "--preset",
        "recommended",
        "--group",
        "hard-mode=on",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let response = json_stdout(&output);
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "invalid_selection");
}

#[test]
fn path_errors_and_modified_games_fail_closed() {
    let temp = fixture_tempdir();
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
    fs::write(
        fixture.recipe.join("release.json"),
        r#"{"version":"0.1.0-alpha.2","label":"Test collection"}"#,
    )
    .unwrap();
    // Production artifact IDs carry dotted versions; exercise them through the entire run.
    let artifact_path = fixture.recipe.join("artifacts/eefixpack.toml");
    let artifact = fs::read_to_string(&artifact_path)
        .unwrap()
        .replace("id = \"eefixpack\"", "id = \"eefixpack-2026.08\"");
    fs::write(
        fixture.recipe.join("artifacts/eefixpack-2026.08.toml"),
        artifact,
    )
    .unwrap();
    fs::remove_file(artifact_path).unwrap();
    let mod_path = fixture.recipe.join("mods/eefixpack.toml");
    let mod_text = fs::read_to_string(&mod_path).unwrap().replace(
        "artifact_id = \"eefixpack\"",
        "artifact_id = \"eefixpack-2026.08\"",
    );
    fs::write(mod_path, mod_text).unwrap();
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
    assert_eq!(receipt.versions.recipe, "0.1.0-alpha.2");
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
fn setup_name_log_override_reconciles_from_attempt_owned_stdout() {
    let fixture = executable_fixture();
    let mut command = executable_command(&fixture);
    command
        .env("CHRIZ_TEST_MOCK_WEIDU_SETUP_LOG_OVERRIDE", "1")
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ));

    let output = command
        .output()
        .expect("execute setup-name log override campaign");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let replay = SessionStore::open(&fixture.managed)
        .expect("open completed campaign")
        .replay()
        .expect("replay completed campaign");
    let steps_root = fixture
        .managed
        .join(".chriz/attempts")
        .join(&replay.created().attempt_id)
        .join("steps");
    let attempt_root = fs::read_dir(steps_root)
        .expect("read completed step evidence")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("attempt-0001"))
        .find(|path| path.join("weidu.debug.log").is_file())
        .expect("find completed WeiDU attempt evidence");
    let debug = fs::read_to_string(attempt_root.join("weidu.debug.log"))
        .expect("read overridden debug stub");
    let stdout =
        fs::read_to_string(attempt_root.join("stdout.log")).expect("read attempt-owned stdout");
    assert!(!debug.contains("SUCCESSFULLY INSTALLED"));
    assert!(stdout.contains("SUCCESSFULLY INSTALLED"));
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

/// Replace a fixture artifact's verified cache entry with a fresh transfer through the
/// signed-redirect route, as happens when the download cache is lost and a GitHub release
/// asset is fetched again: same original URL and bytes, new single-use final URL.
#[cfg(windows)]
fn reacquire_through_signed_redirect(fixture: &ExecutableFixture, path: &str, bytes: &[u8]) {
    let digest = sha256_bytes(bytes);
    let entry = fixture.cache.join("sha256").join(&digest[..2]);
    for extension in ["archive", "json"] {
        let file = entry.join(format!("{digest}.{extension}"));
        if file.exists() {
            fs::remove_file(&file).expect("remove fixture cache entry");
        }
    }
    let (sink, _events) = ChannelSink::unbounded();
    let acquired = ArtifactCache::open(&fixture.cache)
        .expect("open fixture artifact cache")
        .acquire(
            &DownloadRequest {
                request_id: format!("fixture-signed{}", path.replace(['/', '.'], "-")),
                url: format!("{}/signed{path}", fixture._server.base_url),
                expected_length: bytes.len() as u64,
                expected_sha256: digest,
                redirect_hosts: Vec::new(),
                max_attempts: 1,
            },
            &sink,
        )
        .expect("download verified fixture archive through a signed redirect");
    assert!(acquired.metadata.final_url.contains("?signature="));
}

#[cfg(windows)]
fn read_artifact_receipt(path: &Path) -> ArtifactReceipt {
    serde_json::from_slice(&fs::read(path).expect("read artifact evidence"))
        .expect("parse artifact evidence")
}

#[cfg(windows)]
#[test]
fn resume_retains_first_acquisition_after_a_lost_cache_is_redownloaded_via_a_new_signed_url() {
    let fixture = executable_fixture();
    let archives = [
        ("eefixpack", "/eefixpack.zip", fixture.payload.clone()),
        ("weidu", "/weidu.zip", fixture.tool_archive.clone()),
    ];
    for (_, path, bytes) in &archives {
        reacquire_through_signed_redirect(&fixture, path, bytes);
    }
    let marker = fixture._temp.path().join("mock-weidu-failed-once");
    let failed = executable_command(&fixture)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .output()
        .expect("execute first synthetic attempt");
    assert_eq!(failed.status.code(), Some(1), "{}", stderr(&failed));
    assert_eq!(
        json_stdout(&failed)["status"]["step_id"],
        "install:eefixpack-bg1"
    );

    let evidence = fixture.managed.join(".chriz/evidence");
    let first_evidence: BTreeMap<_, _> = archives
        .iter()
        .map(|(id, _, _)| {
            let bytes = fs::read(evidence.join("artifacts").join(format!("{id}.json")))
                .expect("read first acquisition evidence");
            (*id, bytes)
        })
        .collect();
    let first: BTreeMap<_, _> = archives
        .iter()
        .map(|(id, _, _)| {
            (
                *id,
                read_artifact_receipt(&evidence.join("artifacts").join(format!("{id}.json"))),
            )
        })
        .collect();
    assert!(!evidence.join("artifact-reacquisitions").exists());

    // The download cache is separate from the install folder and can disappear between
    // attempts; the next transfer of the same bytes gets a different signed final URL.
    for (_, path, bytes) in &archives {
        reacquire_through_signed_redirect(&fixture, path, bytes);
    }
    let resumed = executable_command(&fixture)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(["--json", "resume", fixture.managed.to_str().unwrap()])
        .output()
        .expect("resume synthetic campaign");
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
    for (id, _, _) in &archives {
        assert_eq!(
            fs::read(evidence.join("artifacts").join(format!("{id}.json"))).unwrap(),
            first_evidence[id],
            "first acquisition evidence for {id} must stay write-once"
        );
        assert_eq!(
            receipt.artifacts.iter().find(|artifact| artifact.id == *id),
            Some(&first[id]),
            "the install receipt must carry the first acquisition of {id}"
        );
    }

    // Payload materialization finished before the failure; WeiDU is re-verified by both
    // resumed install steps, so the same reacquisition is observed twice but recorded once.
    let reacquired = fs::read_dir(evidence.join("artifact-reacquisitions"))
        .expect("read recorded reacquisitions")
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(reacquired, ["weidu"]);
    let records = fs::read_dir(evidence.join("artifact-reacquisitions/weidu"))
        .expect("read recorded WeiDU reacquisitions")
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1, "{records:?}");
    let later = read_artifact_receipt(&records[0]);
    assert_ne!(later.final_url, first["weidu"].final_url);
    assert!(later.final_url.contains("?signature="));
    assert_eq!(later.cache_outcome, ArtifactCacheOutcome::Hit);
    assert_eq!(
        ArtifactReceipt {
            final_url: first["weidu"].final_url.clone(),
            cache_outcome: first["weidu"].cache_outcome,
            ..later.clone()
        },
        first["weidu"]
    );
    assert_eq!(
        records[0].file_stem().and_then(|stem| stem.to_str()),
        Some(sha256_bytes(&serde_json::to_vec(&later).unwrap()).as_str())
    );
}

/// The whole download cache was deleted between attempts, by hand or by the uninstaller.
/// The interrupted build stays resumable, and resuming creates the cache again at its recorded
/// path instead of stopping before the build continues.
#[cfg(windows)]
#[test]
fn resume_recreates_a_deleted_download_cache() {
    let fixture = executable_fixture();
    let marker = fixture._temp.path().join("mock-weidu-failed-once");
    let failed = executable_command(&fixture)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .output()
        .expect("execute first synthetic attempt");
    assert_eq!(failed.status.code(), Some(1), "{}", stderr(&failed));
    fs::rename(&fixture.cache, fixture._temp.path().join("deleted-cache"))
        .expect("move the download cache away");
    let registry =
        ManagedInstallRegistry::open_or_create(&fixture.app_data.join("Chriz BG Collection"))
            .expect("open campaign registry");
    assert_eq!(
        registry.list_campaigns().unwrap()[0].availability,
        CampaignAvailability::Resumable
    );

    // The fixture's download addresses are unreachable, so stop the resumed step at its first
    // check before it needs the tool archive: everything before it ran with the cache missing.
    let stopped = executable_command(&fixture)
        .env(
            "CHRIZ_BG_COLLECTION_TEST_FAIL_RECHECK",
            "invocation-build:install:eefixpack-bg1",
        )
        .args(["--json", "resume", fixture.managed.to_str().unwrap()])
        .output()
        .expect("resume without a download cache");
    assert_eq!(stopped.status.code(), Some(1), "{}", stdout(&stopped));
    let stopped = json_stdout(&stopped);
    assert_eq!(stopped["status"]["step_id"], "install:eefixpack-bg1");
    assert!(stopped["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["type"] == "campaign_started" && event["resumed"] == true));
    assert!(fixture.cache.is_dir());

    // What a resume downloads again, fetched here from the fixture server.
    let cache = ArtifactCache::open(&fixture.cache).expect("open recreated cache");
    let (sink, _events) = ChannelSink::unbounded();
    for (path, bytes) in [
        ("/eefixpack.zip", &fixture.payload),
        ("/weidu.zip", &fixture.tool_archive),
    ] {
        cache
            .acquire(
                &DownloadRequest {
                    request_id: format!("refetch{}", path.replace(['/', '.'], "-")),
                    url: format!("{}{path}", fixture._server.base_url),
                    expected_length: bytes.len() as u64,
                    expected_sha256: sha256_bytes(bytes),
                    redirect_hosts: Vec::new(),
                    max_attempts: 1,
                },
                &sink,
            )
            .expect("download fixture archive again");
    }
    let resumed = executable_command(&fixture)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(["--json", "resume", fixture.managed.to_str().unwrap()])
        .output()
        .expect("resume synthetic campaign");

    assert!(
        resumed.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&resumed),
        stderr(&resumed)
    );
    assert!(fixture
        .managed
        .join(".chriz/install-receipt.json")
        .is_file());
    assert_eq!(registry.list().unwrap().len(), 1);
}

/// The published extraction of `bytes` beneath the fixture's extraction cache.
#[cfg(windows)]
fn published_extraction(fixture: &ExecutableFixture, bytes: &[u8]) -> PathBuf {
    let digest = sha256_bytes(bytes);
    let parent = fixture
        .cache
        .join("extracted/sha256-v3")
        .join(&digest[..2])
        .join(&digest);
    let entries = fs::read_dir(&parent)
        .expect("read published extraction contracts")
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1, "{entries:?}");
    entries.into_iter().next().unwrap()
}

/// The reported failure: a player deleted CEBG's cache folders by hand between attempts and
/// a published extraction survived without its marker. Resuming must rebuild it from the
/// verified archive and record the repair, not stop at "could not inspect extraction marker".
#[cfg(windows)]
#[test]
fn resume_rebuilds_a_partially_deleted_extraction_and_records_the_repair() {
    let fixture = executable_fixture();
    let marker = fixture._temp.path().join("mock-weidu-failed-once");
    let failed = executable_command(&fixture)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .output()
        .expect("execute first synthetic attempt");
    assert_eq!(failed.status.code(), Some(1), "{}", stderr(&failed));
    assert_eq!(
        json_stdout(&failed)["status"]["step_id"],
        "install:eefixpack-bg1"
    );

    // Both resumed install steps extract the WeiDU tool again before running it.
    let tool = published_extraction(&fixture, &fixture.tool_archive);
    let extraction_marker = tool.join(".chriz-bg-extraction.json");
    fs::remove_file(&extraction_marker).expect("remove extraction marker");
    assert!(tool.join("weidu.exe").is_file());

    let resumed = executable_command(&fixture)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(["--json", "resume", fixture.managed.to_str().unwrap()])
        .output()
        .expect("resume synthetic campaign");
    assert!(
        resumed.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&resumed),
        stderr(&resumed)
    );
    assert!(extraction_marker.is_file());

    let repairs = fixture.managed.join(".chriz/evidence/cache-repairs");
    let repaired = fs::read_dir(&repairs)
        .expect("read recorded cache repairs")
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(repaired, ["weidu"]);
    let records = fs::read_dir(repairs.join("weidu"))
        .expect("read recorded WeiDU cache repairs")
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1, "{records:?}");
    let bytes = fs::read(&records[0]).expect("read cache repair record");
    let record: Value = serde_json::from_slice(&bytes).expect("parse cache repair record");
    assert_eq!(record["cache"], "extraction");
    assert_eq!(record["artifact_id"], "weidu");
    assert_eq!(
        record["artifact_sha256"],
        sha256_bytes(&fixture.tool_archive)
    );
    let reason = record["reason"].as_str().unwrap();
    assert!(
        reason.contains("inspect extraction marker")
            && reason.contains(".chriz-bg-extraction.json"),
        "{reason}"
    );
    assert!(
        !reason.contains(&*fixture.cache.to_string_lossy()),
        "{reason}"
    );
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
fn blocked_before_spawn_fixture(kind: &str) -> (ExecutableFixture, PathBuf) {
    let fixture = executable_fixture();
    let mut install = executable_command(&fixture);
    install
        .env(
            "CHRIZ_BG_COLLECTION_TEST_FAIL_RECHECK",
            format!("{kind}:install:eefixpack-bg2"),
        )
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ));
    let output = install
        .output()
        .expect("block synthetic guard before spawn");
    assert_eq!(output.status.code(), Some(1), "{}", stdout(&output));
    let response = json_stdout(&output);
    assert_eq!(response["status"]["step_id"], "install:eefixpack-bg2");
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    let step_key = &sha256_bytes(b"install:eefixpack-bg2")[..16];
    let step = fs::read_dir(
        fixture
            .managed
            .join(".chriz/attempts")
            .join(&replay.created().attempt_id)
            .join("steps"),
    )
    .unwrap()
    .filter_map(Result::ok)
    .find(|entry| entry.file_name().to_string_lossy().ends_with(step_key))
    .unwrap();
    let attempt = step.path().join("attempt-0001");
    assert!(attempt.join("before.log").is_file());
    assert!(!attempt.join("process-result.json").exists());
    assert!(!attempt.join("stdout.log").exists());
    assert!(!fixture.managed.join("game/WeiDU.log").exists());
    assert_eq!(
        fs::read_to_string(fixture.managed.join("bg1/WeiDU.log"))
            .unwrap()
            .lines()
            .count(),
        2
    );
    (fixture, attempt)
}

#[cfg(windows)]
fn use_legacy_pre_spawn_evidence(fixture: &ExecutableFixture, attempt: &Path) {
    let marker_path = attempt.join("pre-spawn-failure.json");
    let marker: Value = serde_json::from_slice(&fs::read(&marker_path).unwrap()).unwrap();
    fs::remove_file(marker_path).unwrap();
    // This disposable fixture models alpha.13's original terminal detail and files.
    // The terminal record has no successor, so its previous-record hash stays valid.
    let ledger = fs::read_dir(fixture.managed.join(".chriz/ledger"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .max()
        .unwrap();
    let mut record: Value = serde_json::from_slice(&fs::read(&ledger).unwrap()).unwrap();
    record["event"]["data"]["detail"] = marker["failure"].clone();
    fs::write(ledger, serde_json::to_vec_pretty(&record).unwrap()).unwrap();
}

#[cfg(windows)]
#[test]
fn pre_spawn_guard_failures_resume_only_the_unstarted_run() {
    for (kind, legacy) in [
        ("invocation-build", false),
        ("process-spawn", false),
        ("invocation-build", true),
    ] {
        let (fixture, attempt) = blocked_before_spawn_fixture(kind);
        if legacy {
            use_legacy_pre_spawn_evidence(&fixture, &attempt);
        }
        let prior_log = fs::read(fixture.managed.join("bg1/WeiDU.log")).unwrap();
        let mut resume = executable_command(&fixture);
        resume.args(["--json", "resume", fixture.managed.to_str().unwrap()]);
        let output = resume.output().expect("resume never-spawned synthetic run");
        assert!(
            output.status.success(),
            "{kind} legacy={legacy}: {}\n{}",
            stdout(&output),
            stderr(&output)
        );
        assert_eq!(
            fs::read(fixture.managed.join("bg1/WeiDU.log")).unwrap(),
            prior_log
        );
        let receipt: InstallReceipt = serde_json::from_slice(
            &fs::read(fixture.managed.join(".chriz/install-receipt.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(receipt.runs[0].attempts.len(), 1);
        assert_eq!(receipt.runs[0].attempts[0].attempt, 1);
        assert_eq!(receipt.runs[1].attempts.len(), 1);
        assert_eq!(receipt.runs[1].attempts[0].attempt, 2);
        assert_eq!(receipt.runs[1].attempts[0].components, vec![0]);
        assert!(!attempt.join("stdout.log").exists());
    }
}

#[cfg(windows)]
#[test]
fn pre_spawn_recovery_rejects_missing_changed_and_execution_evidence() {
    for change in [
        "missing-before",
        "changed-before",
        "changed-current",
        "unexpected-output",
        "changed-marker",
        "missing-marker",
        "missing-invocation",
        "changed-invocation",
        "nonempty-debug",
        "legacy-output",
        "unresolved-ledger",
    ] {
        let kind = if matches!(
            change,
            "missing-invocation" | "changed-invocation" | "nonempty-debug"
        ) {
            "process-spawn"
        } else {
            "invocation-build"
        };
        let (fixture, attempt) = blocked_before_spawn_fixture(kind);
        match change {
            "missing-before" => fs::remove_file(attempt.join("before.log")).unwrap(),
            "changed-before" => fs::write(attempt.join("before.log"), b"changed\n").unwrap(),
            "changed-current" => {
                fs::write(fixture.managed.join("game/WeiDU.log"), b"changed\n").unwrap()
            }
            "unexpected-output" => fs::write(attempt.join("stdout.log"), b"").unwrap(),
            "changed-marker" => {
                let path = attempt.join("pre-spawn-failure.json");
                let mut marker: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                marker["attempt"] = Value::from(2);
                fs::write(path, serde_json::to_vec(&marker).unwrap()).unwrap();
            }
            "missing-invocation" => fs::remove_file(attempt.join("invocation.json")).unwrap(),
            "missing-marker" => fs::remove_file(attempt.join("pre-spawn-failure.json")).unwrap(),
            "changed-invocation" => fs::write(attempt.join("invocation.json"), b"{}\n").unwrap(),
            "nonempty-debug" => {
                fs::write(attempt.join("weidu.debug.log"), b"some execution\n").unwrap()
            }
            "legacy-output" => {
                use_legacy_pre_spawn_evidence(&fixture, &attempt);
                fs::write(attempt.join("stdout.log"), b"").unwrap();
            }
            "unresolved-ledger" => {
                let terminal = fs::read_dir(fixture.managed.join(".chriz/ledger"))
                    .unwrap()
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .max()
                    .unwrap();
                fs::remove_file(terminal).unwrap();
            }
            _ => unreachable!(),
        }
        let mut resume = executable_command(&fixture);
        resume.args(["--json", "resume", fixture.managed.to_str().unwrap()]);
        let output = resume
            .output()
            .expect("reject compromised pre-spawn evidence");
        assert_eq!(
            output.status.code(),
            Some(1),
            "{change}: {}",
            stdout(&output)
        );
        assert_eq!(
            json_stdout(&output)["status"]["status"],
            "fresh_copy_required",
            "{change}: {}",
            stdout(&output)
        );
        assert!(!attempt.parent().unwrap().join("attempt-0002").exists());
    }
}

#[cfg(windows)]
#[test]
fn launched_run_with_missing_invocation_is_never_classified_as_unstarted() {
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
    let output = install.output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    let attempt = fs::read_dir(
        fixture
            .managed
            .join(".chriz/attempts")
            .join(&replay.created().attempt_id)
            .join("steps"),
    )
    .unwrap()
    .filter_map(Result::ok)
    .map(|entry| entry.path().join("attempt-0001"))
    .find(|path| path.join("invocation.json").exists())
    .unwrap();
    fs::remove_file(attempt.join("invocation.json")).unwrap();
    let log = fs::read(fixture.managed.join("bg1/WeiDU.log")).unwrap();
    let mut resume = executable_command(&fixture);
    resume.args(["--json", "resume", fixture.managed.to_str().unwrap()]);
    let output = resume.output().unwrap();
    assert_eq!(
        json_stdout(&output)["status"]["status"],
        "fresh_copy_required",
        "{}",
        stdout(&output)
    );
    assert_eq!(
        fs::read(fixture.managed.join("bg1/WeiDU.log")).unwrap(),
        log
    );
}

#[cfg(windows)]
#[test]
fn explicit_relative_tp2_is_rejected_before_staging_or_freezing() {
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
    assert_eq!(response["error"]["code"], "installer_launch_unsupported");
    assert!(stdout_from_value(&response).contains("unsupported explicit-tp2 invocation"));
    assert!(
        !fixture.managed.exists(),
        "must reject before creating the installation"
    );
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
