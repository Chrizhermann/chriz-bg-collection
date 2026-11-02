use std::collections::{BTreeMap, BTreeSet};
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
    ArtifactCacheOutcome, ArtifactReceipt, DownloadIdentity, InstallReceipt, ReceiptOutcome,
    ReviewedTool, RECEIPT_SCHEMA_VERSION,
};
use bg_engine::registry::{CampaignAvailability, ManagedInstallRegistry};
use bg_engine::session::SessionStore;
use bg_engine::stage::TEST_DOCUMENTS_VARIABLE;
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

/// What the test server answers for one path.
#[derive(Clone)]
struct Answer {
    status: &'static str,
    body: Vec<u8>,
    /// The announced length; `None` announces the body's.
    announced: Option<u64>,
    /// A path on this server under the host name `localhost`, which a recipe never reviews, to
    /// redirect to.
    elsewhere: Option<&'static str>,
}

impl Answer {
    fn file(body: Vec<u8>) -> Self {
        Self {
            status: "200 OK",
            body,
            announced: None,
            elsewhere: None,
        }
    }

    fn status(status: &'static str) -> Self {
        Self {
            status,
            body: Vec::new(),
            announced: None,
            elsewhere: None,
        }
    }

    /// A redirect to `path` on this server through another host name.
    fn elsewhere(path: &'static str) -> Self {
        Self {
            status: "302 Found",
            body: Vec::new(),
            announced: None,
            elsewhere: Some(path),
        }
    }
}

impl ArtifactServer {
    fn start(artifacts: BTreeMap<String, Vec<u8>>) -> Self {
        Self::start_with(artifacts, |_| {})
    }

    /// Like [`ArtifactServer::start`], calling `on_request` with each request path before
    /// answering it.
    fn start_with(
        artifacts: BTreeMap<String, Vec<u8>>,
        on_request: impl Fn(&str) + Send + 'static,
    ) -> Self {
        Self::start_answering(
            artifacts
                .into_iter()
                .map(|(path, body)| (path, vec![Answer::file(body)]))
                .collect(),
            on_request,
        )
    }

    /// Answers the requests of each path with its answers in turn, the last one from then on.
    fn start_answering(
        answers: BTreeMap<String, Vec<Answer>>,
        on_request: impl Fn(&str) + Send + 'static,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind artifact server");
        let address = listener.local_addr().expect("artifact server address");
        listener
            .set_nonblocking(true)
            .expect("make artifact server nonblocking");
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let signatures = AtomicU64::new(0);
        let thread = thread::spawn(move || {
            let mut asked = BTreeMap::new();
            while !thread_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((_stream, _)) if thread_stop.load(Ordering::Acquire) => break,
                    Ok((stream, _)) => {
                        serve_artifact(stream, &answers, &mut asked, &signatures, &on_request);
                    }
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
    answers: &BTreeMap<String, Vec<Answer>>,
    asked: &mut BTreeMap<String, usize>,
    signatures: &AtomicU64,
    on_request: &dyn Fn(&str),
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
    on_request(path);
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
    let answer = answers
        .get(path)
        .map(|answers| {
            let count = asked.entry(path.to_owned()).or_default();
            let answer = answers[(*count).min(answers.len() - 1)].clone();
            *count += 1;
            answer
        })
        .unwrap_or_else(|| Answer::status("404 Not Found"));
    let location = answer.elsewhere.map_or_else(String::new, |path| {
        let port = stream.local_addr().expect("artifact server address").port();
        format!("Location: http://localhost:{port}{path}\r\n")
    });
    write!(
        stream,
        "HTTP/1.1 {}\r\n{location}Content-Length: {}\r\nConnection: close\r\n\r\n",
        answer.status,
        answer.announced.unwrap_or(answer.body.len() as u64)
    )
    .expect("write artifact headers");
    stream.write_all(&answer.body).expect("write artifact body");
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
    let mut command = Command::new(env!("CARGO_BIN_EXE_chriz-bg-install"));
    // Process checks see no running program unless a test declares one, whatever else runs on
    // this PC.
    command.env("CHRIZ_BG_COLLECTION_TEST_RUNNING_EXECUTABLES", "");
    command
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

/// `args` with a preset `--answer` for each of `answers`, as tests and automation give them.
fn answering<'a>(mut args: Vec<&'a str>, answers: &[&'a str]) -> Vec<&'a str> {
    for answer in answers {
        args.extend(["--answer", answer]);
    }
    args
}

/// Installs a changed download, as the player's "Install it" does.
const INSTALL_CHANGED: &str = "changed-download=yes";
/// Continues without a mod whose download cannot be used, as "Continue without them" does.
const CONTINUE_WITHOUT: &str = "unusable-download=yes";

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

/// Makes the fixture's BG1 EE Fixpack part a fixed part of the collection, so that a run
/// failing there (`CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE`) stops the installation, as these resume
/// tests need. An optional part that fails is left out instead.
fn fail_once_at_a_fixed_part(fixture: &ExecutableFixture) {
    let collection = fixture.recipe.join("collection.toml");
    let text = fs::read_to_string(&collection)
        .unwrap()
        .replace("\r\n", "\n");
    let feature = "id = \"eefix-bg1\"\n";
    let start = text.find(feature).expect("fixture BG1 fixes feature");
    let decision = start
        + text[start..]
            .find("decision = \"default\"")
            .expect("fixture BG1 fixes decision");
    let mut changed = text.clone();
    changed.replace_range(
        decision..decision + "decision = \"default\"".len(),
        "decision = \"mandatory\"",
    );
    fs::write(collection, changed).unwrap();
}
fn failed_install_fixture() -> (TempDir, PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    let temp = fixture_tempdir();
    let recipe = recipe_with_profiles(&temp);
    use_production_invocation_mode(&recipe);
    let bg1 = copied_game(&temp, "steam-bgee-sod", "bg1");
    let bg2 = copied_game(&temp, "steam-bg2ee", "bg2");
    // A game that misses a file every installation needs stays a stop. (Leftovers of earlier
    // mod installs no longer are: the copy leaves them out.)
    fs::remove_file(bg1.join("chitin.key")).expect("remove a required game file");
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
        .env(TEST_DOCUMENTS_VARIABLE, &fixture.documents)
        .env("CHRIZ_TEST_MOCK_WEIDU_TP2", "EE_Fixpack/EE_Fixpack.tp2");
    command
}

/// A test build of the installer never falls back to the player's real Documents folder: an
/// install that reaches its save folder without an isolated one stops there.
#[cfg(windows)]
#[test]
fn install_without_an_isolated_documents_folder_stops_before_reserving_saves() {
    let fixture = executable_fixture();
    let mut install = executable_command(&fixture);
    install
        .env_remove(TEST_DOCUMENTS_VARIABLE)
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ));

    let output = install
        .output()
        .expect("run install without isolated Documents");

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("never use the real Documents folder"),
        "{}",
        stderr(&output)
    );
    assert!(!fixture.managed.join("bg1/engine.lua").exists());
}

/// A real program named like WeiDU, running outside the fixture's game folders until dropped.
#[cfg(windows)]
struct ForeignInstaller(std::process::Child);

#[cfg(windows)]
impl ForeignInstaller {
    fn start(executable: &Path) -> Self {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::copy(std::env::var_os("COMSPEC").unwrap(), executable).unwrap();
        let child = Command::new(executable)
            .args(["/D", "/C", "set /p wait_for_test="])
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        Self(child)
    }
}

#[cfg(windows)]
impl Drop for ForeignInstaller {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// In a test build the installer sees only the programs a test declares running, so another
/// program's WeiDU on this PC never decides a test (issue 9). A declared installer still blocks
/// the manual checkpoint, wherever it runs.
#[cfg(windows)]
#[test]
fn manual_checkpoint_checks_see_only_the_installers_a_test_declares() {
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
        .args(["--manual-pause-after", "eefixpack-bg1:0"]);
    let paused = install.output().unwrap();
    assert!(
        paused.status.success(),
        "{}
{}",
        stdout(&paused),
        stderr(&paused)
    );
    let paused = json_stdout(&paused);
    assert_eq!(paused["status"]["status"], "paused");
    let install_id = paused["install_id"].as_str().unwrap();
    let check = |declared: Option<&Path>| {
        let mut command = executable_command(&fixture);
        if let Some(declared) = declared {
            command.env("CHRIZ_BG_COLLECTION_TEST_RUNNING_EXECUTABLES", declared);
        }
        command
            .args([
                "--json",
                "check-manual-checkpoint",
                fixture.managed.to_str().unwrap(),
                "--install-id",
                install_id,
            ])
            .output()
            .unwrap()
    };
    let outside = fixture.managed.parent().unwrap().join("elsewhere");
    let _foreign = ForeignInstaller::start(&outside.join("weidu.exe"));

    // Tests declare no running program by default.
    let unaffected = check(None);
    assert!(
        unaffected.status.success(),
        "{}
{}",
        stdout(&unaffected),
        stderr(&unaffected)
    );

    let declared = outside.join("Downloads").join("Setup-SomeMod.exe");
    let blocked = check(Some(&declared));
    assert!(!blocked.status.success(), "{}", stdout(&blocked));
    let error = &json_stdout(&blocked)["error"];
    assert_eq!(
        error["code"], "manual_checkpoint_needs_attention",
        "{error}"
    );
    let message = error["message"].as_str().unwrap();
    assert!(
        message.contains("Close the manual installer")
            && message.contains(declared.to_str().unwrap()),
        "{message}"
    );
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
        "check-decision",
        "decide",
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

/// Fixture recipe whose `challenge-mode` group has the member `challenge-a` and a variant that
/// picks `pace-harsh` instead of the recommended `pace-calm`.
fn recipe_with_group_variant(temp: &TempDir) -> PathBuf {
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
members = ["challenge-a"]
variants = [{ choice_group = "pace", option = "pace-harsh", label = "Pace: harsh" }]

[[features]]
id = "challenge-a"
title = "Challenge A"
description = "Fixture challenge A"
category = "difficulty"
decision = "optional"
readiness = "ready"
components = [{ run_id = "eefixpack-bg1", component = 0 }]

[[features]]
id = "pace-calm"
title = "Calm pace"
description = "Fixture calm pace"
category = "difficulty"
group_label = "Pace"
choice_group = "pace"
decision = "default"
readiness = "ready"
conflicts = [{ feature_id = "pace-harsh", reason = "Choose one pace." }]
components = [{ run_id = "eefixpack-bg1", component = 2 }]

[[features]]
id = "pace-harsh"
title = "Harsh pace"
description = "Fixture harsh pace"
category = "difficulty"
group_label = "Pace"
choice_group = "pace"
decision = "optional"
readiness = "ready"
conflicts = [{ feature_id = "pace-calm", reason = "Choose one pace." }]
components = [{ run_id = "eefixpack-bg2", component = 0 }]
"#,
    );
    fs::write(collection, text).unwrap();
    recipe
}

#[test]
fn plan_group_switch_picks_its_variant_and_an_explicit_choice_wins() {
    let temp = fixture_tempdir();
    let recipe = recipe_with_group_variant(&temp);
    let recipe = recipe.to_str().unwrap();
    let plan = |extra: &[&str]| {
        let mut args = vec!["--json", "plan", recipe, "--preset", "recommended"];
        args.extend_from_slice(extra);
        let output = run(&args);
        assert!(output.status.success(), "{extra:?}: {}", stderr(&output));
        json_stdout(&output)
    };

    let off = plan(&[]);
    assert_eq!(
        planned_components(&off),
        vec![("eefixpack-bg1".to_owned(), vec![2])]
    );
    assert_eq!(off["groups"][0]["state"], "off");
    assert_eq!(off["groups"][0]["variants"][0]["choice_group"], "pace");
    assert_eq!(off["groups"][0]["variants"][0]["option"], "pace-harsh");
    assert_eq!(off["groups"][0]["variants"][0]["selected"], false);
    assert_eq!(off["groups"][0]["variants"][0]["available"], true);

    let on = plan(&["--group", "challenge-mode=on"]);
    assert_eq!(
        planned_components(&on),
        vec![
            ("eefixpack-bg1".to_owned(), vec![0]),
            ("eefixpack-bg2".to_owned(), vec![0]),
        ]
    );
    assert_eq!(on["groups"][0]["state"], "on");
    assert_eq!(on["groups"][0]["variants"][0]["selected"], true);
    assert_eq!(on["normalized_selection"]["features"]["pace-calm"], false);
    assert_eq!(on["normalized_selection"]["features"]["pace-harsh"], true);

    // An explicit option of the choice keeps the group from switching it, wherever it appears.
    for args in [
        ["--group", "challenge-mode=on", "--feature", "pace-calm=on"],
        ["--feature", "pace-calm=on", "--group", "challenge-mode=on"],
    ] {
        let kept = plan(&args);
        assert_eq!(
            planned_components(&kept),
            vec![("eefixpack-bg1".to_owned(), vec![0, 2])],
            "{args:?}"
        );
        assert_eq!(kept["groups"][0]["state"], "mixed", "{args:?}");
        assert_eq!(
            kept["normalized_selection"]["features"]["pace-harsh"], false,
            "{args:?}"
        );
    }

    // An explicit choice that agrees with the group still lets it settle the rest of the choice:
    // the variant option itself, or the recommended option switched off.
    for args in [
        ["--group", "challenge-mode=on", "--feature", "pace-harsh=on"],
        ["--feature", "pace-calm=off", "--group", "challenge-mode=on"],
    ] {
        let agreed = plan(&args);
        assert_eq!(
            planned_components(&agreed),
            planned_components(&on),
            "{args:?}"
        );
        assert_eq!(agreed["groups"][0]["state"], "on", "{args:?}");
        assert_eq!(
            agreed["normalized_selection"]["features"]["pace-calm"], false,
            "{args:?}"
        );
    }
    // Switching the variant option itself off keeps the recommended option.
    let declined = plan(&[
        "--group",
        "challenge-mode=on",
        "--feature",
        "pace-harsh=off",
    ]);
    assert_eq!(
        planned_components(&declined),
        vec![("eefixpack-bg1".to_owned(), vec![0, 2])]
    );
    assert_eq!(declined["groups"][0]["state"], "mixed");

    // Switching the group off again restores the recommended option.
    let toggled = plan(&[
        "--group",
        "challenge-mode=on",
        "--group",
        "challenge-mode=off",
    ]);
    assert_eq!(planned_components(&toggled), planned_components(&off));
    assert_eq!(toggled["normalized_selection"], off["normalized_selection"]);

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
        stdout(&human).contains(
            "Group challenge-mode (Challenge Mode): on, 1 of 1 available selected, 1 of 1 available variants picked"
        ),
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
    // the source-game guard, which the BG1 copy without its chitin.key then trips.
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
    fs::remove_file(bg1.join("chitin.key")).expect("remove a required game file");
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
fn path_errors_and_games_missing_required_files_fail_closed() {
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
    assert!(message.contains("chitin.key"), "{message}");
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
    let receipt_bytes = fs::read(&receipt_path).expect("durable success receipt");
    let receipt: InstallReceipt =
        serde_json::from_slice(&receipt_bytes).expect("parse durable success receipt");
    // Downloads that match their pins leave the records exactly as before changed downloads
    // were accepted.
    let evidence = fixture.managed.join(".chriz/evidence");
    for record in [
        receipt_bytes,
        fs::read(evidence.join("artifacts/eefixpack-2026.08.json")).unwrap(),
        fs::read(evidence.join("artifacts/weidu.json")).unwrap(),
        fs::read(evidence.join("tools/weidu.json")).unwrap(),
    ] {
        let text = String::from_utf8(record).unwrap();
        for field in [
            "served_difference",
            "left_out",
            "installed_plan",
            "\"reviewed\"",
            "decisions",
        ] {
            assert!(!text.contains(field), "{field} in {text}");
        }
    }
    // A receipt of an installation that asked nothing has exactly the fields it had before
    // questions existed, in their order, and its bytes are what the receipt type writes.
    let receipt_text = fs::read_to_string(&receipt_path).unwrap();
    let top_level = receipt_text
        .lines()
        .filter_map(|line| line.strip_prefix("  \""))
        .filter_map(|line| line.split_once('"').map(|(key, _)| key))
        .collect::<Vec<_>>();
    assert_eq!(
        top_level,
        [
            "schema_version",
            "install_id",
            "attempt_id",
            "evidence_attempt_id",
            "managed_root",
            "staged_bg1",
            "staged_bg2",
            "started_at_millis",
            "completed_at_millis",
            "outcome",
            "versions",
            "source_games",
            "recipe_payload_sha256",
            "recipe_envelope_sha256",
            "selection_sha256",
            "normalized_selection",
            "plan_sha256",
            "plan",
            "artifacts",
            "weidu_tools",
            "runs",
            "final_state",
        ]
    );
    let mut rewritten = serde_json::to_vec_pretty(&receipt).unwrap();
    rewritten.push(b'\n');
    assert_eq!(rewritten, receipt_text.as_bytes()); // Nothing was asked: no answer was stored, and the ledger holds no question.
    assert!(!evidence.join("decisions").exists());
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    assert!(!replay.records.iter().any(|record| matches!(
        record.event,
        bg_engine::session::SessionEvent::DecisionRequired { .. }
    )));
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

/// Adds one configuration file to the named component of the synthetic EE Fixpack installer.
fn author_config_file(fixture: &ExecutableFixture, component: &str, path: &str, content: &str) {
    let mod_file = fixture.recipe.join("mods/eefixpack.toml");
    let text = fs::read_to_string(&mod_file).expect("read synthetic installer");
    let anchor = format!("name = \"{component}\"");
    assert_eq!(text.matches(&anchor).count(), 1, "{anchor}");
    let entry = format!(
        "{anchor}\nconfig_files = [{{ path = {}, content = {} }}]",
        serde_json::to_string(path).unwrap(),
        serde_json::to_string(content).unwrap()
    );
    fs::write(&mod_file, text.replace(&anchor, &entry)).expect("author synthetic config file");
}

/// The evidence directory of one attempt of one pipeline step.
fn step_attempt(fixture: &ExecutableFixture, step_id: &str, attempt: u32) -> PathBuf {
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    let step_key = &sha256_bytes(step_id.as_bytes())[..16];
    fs::read_dir(
        fixture
            .managed
            .join(".chriz/attempts")
            .join(&replay.created().attempt_id)
            .join("steps"),
    )
    .unwrap()
    .filter_map(Result::ok)
    .find(|entry| entry.file_name().to_string_lossy().ends_with(step_key))
    .unwrap()
    .path()
    .join(format!("attempt-{attempt:04}"))
}

fn config_evidence(attempt: &Path) -> Value {
    serde_json::from_slice(&fs::read(attempt.join("config-files.json")).unwrap()).unwrap()
}

#[cfg(windows)]
/// Leftovers of earlier mod installs in a source game once stopped the installation ("the
/// game is modified"). Now the copy leaves them out, and the installation completes.
#[test]
fn leftovers_in_a_source_game_are_left_out_and_the_installation_completes() {
    let fixture = executable_fixture();
    let old_log =
        "// Log of Currently Installed WeiDU Mods\n~OLD/SETUP-OLD.TP2~ #0 #0 // Old mod\n";
    for game in [&fixture.bg1, &fixture.bg2] {
        fs::write(game.join("WeiDU.log"), old_log).unwrap();
        fs::write(game.join("setup-old.exe"), b"old installer").unwrap();
        fs::create_dir_all(game.join("stratagems")).unwrap();
        fs::write(game.join("stratagems/readme.txt"), b"old mod folder").unwrap();
    }
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
    assert_eq!(json_stdout(&output)["status"]["status"], "complete");
    for copy in ["bg1", "game"] {
        let copy = fixture.managed.join(copy);
        assert!(!copy.join("setup-old.exe").exists());
        assert!(!copy.join("stratagems").exists());
        if let Ok(log) = fs::read_to_string(copy.join("WeiDU.log")) {
            assert!(!log.contains("SETUP-OLD"), "{log}");
        }
    }
    // The source games are untouched.
    for game in [&fixture.bg1, &fixture.bg2] {
        assert_eq!(fs::read_to_string(game.join("WeiDU.log")).unwrap(), old_log);
        assert!(game.join("stratagems/readme.txt").is_file());
    }
}

/// A failed optional run once stopped the installation. Now it is left out, with everything
/// that needs it, and the installation completes; the receipt names the failed run.
#[test]
fn a_failed_optional_run_is_left_out_and_the_installation_completes() {
    let fixture = executable_fixture();
    let marker = fixture._temp.path().join("mock-weidu-failed-once");
    let mut command = executable_command(&fixture);
    command
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(install_args(
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
    assert_eq!(response["status"]["status"], "complete");
    assert!(marker.is_file(), "the first run failed once");
    assert!(response["events"].as_array().unwrap().iter().any(|event| {
        event["type"] == "step_finished"
            && event["id"] == "install:eefixpack-bg1"
            && event["outcome"] == "skipped"
    }));
    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.left_out.len(), 1, "{:?}", receipt.left_out);
    let left_out = &receipt.left_out[0];
    assert_eq!(left_out.artifact_id, "eefixpack");
    assert_eq!(left_out.failed_run.as_deref(), Some("eefixpack-bg1"));
    assert_eq!(left_out.runs, vec!["eefixpack-bg1".to_owned()]);
    let installed = receipt.installed_plan.as_ref().unwrap();
    assert!(installed.components_for("eefixpack-bg1").is_none());
    assert!(installed.components_for("eefixpack-bg2").is_some());
    // The receipt verifies against the ledger, as every completed installation's does.
    let report = cli()
        .env("LOCALAPPDATA", &fixture.app_data)
        .args(["--json", "report", fixture.managed.to_str().unwrap()])
        .output()
        .expect("run report");
    assert!(report.status.success(), "{}", stderr(&report));
}

#[test]
fn install_places_component_config_files_before_weidu_in_both_games() {
    const CONTENT: &str = "OUTER_SET fixed = 1\n";
    let fixture = executable_fixture();
    author_config_file(&fixture, "Core fixes", "EE_Fixpack/fixpack.txt", CONTENT);
    let mut command = executable_command(&fixture);
    // The synthetic WeiDU refuses to install unless the file is already in place.
    command.env(
        "CHRIZ_TEST_MOCK_WEIDU_REQUIRE_FILE",
        "EE_Fixpack/fixpack.txt",
    );
    command.args(install_args(
        &fixture.recipe,
        &fixture.bg1,
        &fixture.bg2,
        &fixture.managed,
        &fixture.cache,
    ));

    let output = command.output().expect("execute config-file install");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let expected = serde_json::json!([{
        "component": 0,
        "path": "EE_Fixpack/fixpack.txt",
        "sha256": sha256_bytes(CONTENT.as_bytes()),
        "created": true,
    }]);
    for (root, step) in [
        ("bg1", "install:eefixpack-bg1"),
        ("game", "install:eefixpack-bg2"),
    ] {
        assert_eq!(
            fs::read_to_string(fixture.managed.join(root).join("EE_Fixpack/fixpack.txt"))
                .expect("placed config file"),
            CONTENT
        );
        assert_eq!(config_evidence(&step_attempt(&fixture, step, 1)), expected);
    }
}

/// A failed placement leaves WeiDU unstarted, so resuming retries the run instead of demanding
/// a fresh copy of the games. A file placed before the failure is accepted again.
#[cfg(windows)]
#[test]
fn failed_config_file_placement_is_retried_on_resume() {
    const PLACED: &str = "OUTER_SET fixed = 1\n";
    const CONTENT: &str = "OUTER_SET fixed = 2\n";
    let fixture = executable_fixture();
    // The first file can be placed; the folder of the second one is missing.
    author_config_file(&fixture, "Core fixes", "EE_Fixpack/fixpack.txt", PLACED);
    author_config_file(
        &fixture,
        "BG1 preparation fixes",
        "EE_Fixpack/later/fixpack.txt",
        CONTENT,
    );
    let mut install = executable_command(&fixture);
    install.args(install_args(
        &fixture.recipe,
        &fixture.bg1,
        &fixture.bg2,
        &fixture.managed,
        &fixture.cache,
    ));

    let output = install
        .output()
        .expect("execute install without the folder");

    assert_eq!(output.status.code(), Some(1), "{}", stdout(&output));
    let response = json_stdout(&output);
    assert_eq!(
        response["status"]["step_id"], "install:eefixpack-bg1",
        "{response:#}"
    );
    assert!(
        stdout_from_value(&response).contains("is missing"),
        "{response:#}"
    );
    let failed = step_attempt(&fixture, "install:eefixpack-bg1", 1);
    assert!(failed.join("pre-spawn-failure.json").is_file());
    assert!(!failed.join("invocation.json").exists());
    assert!(!failed.join("config-files.json").exists());
    assert!(!fixture.managed.join("bg1/WeiDU.log").exists());
    assert_eq!(
        fs::read_to_string(fixture.managed.join("bg1/EE_Fixpack/fixpack.txt")).unwrap(),
        PLACED
    );

    // The folder appears, as once a passing problem is fixed.
    fs::create_dir(fixture.managed.join("bg1/EE_Fixpack/later")).unwrap();
    let mut resume = executable_command(&fixture);
    resume.args(["--json", "resume", fixture.managed.to_str().unwrap()]);
    let output = resume.output().expect("resume after the failed placement");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!(
        fs::read_to_string(fixture.managed.join("bg1/EE_Fixpack/later/fixpack.txt")).unwrap(),
        CONTENT
    );
    let receipt: InstallReceipt = serde_json::from_slice(
        &fs::read(fixture.managed.join(".chriz/install-receipt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt.runs[0].attempts.len(), 1);
    assert_eq!(receipt.runs[0].attempts[0].attempt, 2);
    assert_eq!(receipt.runs[0].attempts[0].components, vec![0, 2]);
    assert_eq!(
        config_evidence(&step_attempt(&fixture, "install:eefixpack-bg1", 2)),
        serde_json::json!([
            {
                "component": 0,
                "path": "EE_Fixpack/fixpack.txt",
                "sha256": sha256_bytes(PLACED.as_bytes()),
                "created": false,
            },
            {
                "component": 2,
                "path": "EE_Fixpack/later/fixpack.txt",
                "sha256": sha256_bytes(CONTENT.as_bytes()),
                "created": true,
            }
        ])
    );
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
    fail_once_at_a_fixed_part(&fixture);
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
    fail_once_at_a_fixed_part(&fixture);
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
    fail_once_at_a_fixed_part(&fixture);
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

/// Test builds download the `https://example.invalid/` links of test recipes from here.
#[cfg(windows)]
const DOWNLOAD_ORIGIN: &str = "CHRIZ_BG_COLLECTION_TEST_DOWNLOAD_ORIGIN";

/// Removes the verified download cache entry of `bytes`.
#[cfg(windows)]
fn remove_cached_download(fixture: &ExecutableFixture, bytes: &[u8]) {
    let digest = sha256_bytes(bytes);
    for extension in ["archive", "json"] {
        let file = fixture
            .cache
            .join("sha256")
            .join(&digest[..2])
            .join(format!("{digest}.{extension}"));
        if file.exists() {
            fs::remove_file(&file).expect("remove fixture cache entry");
        }
    }
}

#[cfg(windows)]
fn console_lines(response: &Value) -> Vec<String> {
    response["events"]
        .as_array()
        .expect("response events")
        .iter()
        .filter(|event| event["type"] == "console_line")
        .map(|event| event["line"].as_str().unwrap().to_owned())
        .collect()
}

#[cfg(windows)]
fn read_install_receipt(fixture: &ExecutableFixture) -> InstallReceipt {
    serde_json::from_slice(
        &fs::read(fixture.managed.join(".chriz/install-receipt.json"))
            .expect("read install receipt"),
    )
    .expect("parse install receipt")
}

#[cfg(windows)]
fn report_json(fixture: &ExecutableFixture) -> Value {
    let output = executable_command(fixture)
        .args(["--json", "report", fixture.managed.to_str().unwrap()])
        .output()
        .expect("report the installation");
    assert!(output.status.success(), "{}", stdout(&output));
    json_stdout(&output)
}

/// The official link of a mod serves a newer release than the reviewed one. The installer
/// installs it, says so, records both files, and a resumed installation keeps the accepted file
/// even though the link has changed once more.
#[cfg(windows)]
#[test]
fn a_changed_download_is_installed_recorded_and_kept_when_the_installation_resumes() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.payload);
    let newer = zip_bytes(&[
        (
            "EE_Fixpack/EE_Fixpack.tp2",
            b"BEGIN ~Synthetic EE Fixpack~\nDESIGNATED 0\n",
        ),
        (
            "EE_Fixpack/new-in-2026.09.txt",
            b"added by the newer release",
        ),
    ]);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/eefixpack.zip".to_owned(),
        newer.clone(),
    )]));

    let stopped = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .env(
            "CHRIZ_BG_COLLECTION_TEST_FAIL_RECHECK",
            "materialize:materialize:eefixpack:bg1",
        )
        .args(answering(
            install_args(
                &fixture.recipe,
                &fixture.bg1,
                &fixture.bg2,
                &fixture.managed,
                &fixture.cache,
            ),
            &[INSTALL_CHANGED],
        ))
        .output()
        .expect("start the installation");
    assert_eq!(stopped.status.code(), Some(1), "{}", stdout(&stopped));
    let stopped = json_stdout(&stopped);
    assert_eq!(stopped["status"]["step_id"], "materialize:eefixpack:bg1");
    let lines = console_lines(&stopped);
    assert!(
        lines.iter().any(|line| line.starts_with(
            "The download of EE Fixpack 2026.08 is newer or different than the reviewed version. Reviewed: ee-fixpack.zip, "
        ) && line.contains(&format!(
            "Installing what its official link serves: eefixpack.zip, {} bytes, SHA-256 {}, from 127.0.0.1.",
            newer.len(),
            sha256_bytes(&newer)
        ))),
        "{lines:#?}"
    );
    let evidence = read_artifact_receipt(
        &fixture
            .managed
            .join(".chriz/evidence/artifacts/eefixpack.json"),
    );
    let difference = evidence
        .served_difference
        .clone()
        .expect("the changed download is recorded");
    assert_eq!(
        difference.reviewed,
        DownloadIdentity {
            filename: Some("ee-fixpack.zip".to_owned()),
            length: fixture.payload.len() as u64,
            sha256: sha256_bytes(&fixture.payload),
        }
    );
    assert_eq!(
        difference.served,
        DownloadIdentity {
            filename: Some("eefixpack.zip".to_owned()),
            length: newer.len() as u64,
            sha256: sha256_bytes(&newer),
        }
    );
    assert!(difference.unreviewed_hosts.is_empty());
    assert_eq!(evidence.length, newer.len() as u64);
    assert_eq!(evidence.sha256, sha256_bytes(&newer));

    drop(link);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/eefixpack.zip".to_owned(),
        zip_bytes(&[(
            "EE_Fixpack/EE_Fixpack.tp2",
            b"BEGIN ~An even newer release~\nDESIGNATED 0\n",
        )]),
    )]));
    let resumed = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .args(["--json", "resume", fixture.managed.to_str().unwrap()])
        .output()
        .expect("resume the installation");
    assert!(
        resumed.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&resumed),
        stderr(&resumed)
    );

    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    assert!(receipt.left_out.is_empty());
    assert_eq!(receipt.installed_plan, None);
    assert_eq!(
        receipt
            .artifacts
            .iter()
            .find(|artifact| artifact.id == "eefixpack"),
        Some(&evidence)
    );
    for game in ["bg1", "game"] {
        assert!(fixture
            .managed
            .join(game)
            .join("EE_Fixpack/new-in-2026.09.txt")
            .is_file());
    }
    assert_eq!(
        report_json(&fixture)["notes"],
        serde_json::json!([
            "You chose to install a newer download of EE Fixpack (eefixpack.zip).",
            "If something does not work as expected, please report it so the collection can be fixed."
        ])
    );
    let decisions = &receipt.decisions;
    assert_eq!(decisions.len(), 1, "{decisions:#?}");
    assert_eq!(decisions[0].step_id, "acquire:eefixpack");
    assert_eq!(decisions[0].answer, bg_engine::decision::Answer::Yes);
    assert_eq!(
        decisions[0].source,
        bg_engine::decision::AnswerSource::Preset
    );
    assert_eq!(decisions[0].served, Some(difference.served.clone()));
}

/// Adds Snowytoes, a second mod from its own download, to the executable fixture's recipe.
#[cfg(windows)]
fn add_snowytoes(fixture: &ExecutableFixture, reviewed: &[u8]) {
    add_snowytoes_publishing(fixture, reviewed, &["Snowytoes"]);
}

/// [`add_snowytoes`], with the folders its archive places in the game.
#[cfg(windows)]
fn add_snowytoes_publishing(fixture: &ExecutableFixture, reviewed: &[u8], roots: &[&str]) {
    fs::write(
        fixture.recipe.join("artifacts/snowytoes.toml"),
        artifact_toml(
            "snowytoes",
            "Snowytoes",
            "1.6",
            "https://example.invalid/snowytoes.zip",
            "Snowytoes-v1.6.zip",
            reviewed,
            roots,
            &["Snowytoes/setup-snowytoes.tp2"],
            None,
        ),
    )
    .expect("write the Snowytoes download");
    fs::write(
        fixture.recipe.join("mods/snowytoes.toml"),
        r#"id = "snowytoes"
artifact_id = "snowytoes"
name = "Snowytoes"
tp2 = "Snowytoes/setup-snowytoes.tp2"
language = 0
weidu_artifact_id = "weidu"
invocation_mode = "setup-name"

[[components]]
id = 0
name = "Snowy areas"
"#,
    )
    .expect("write the Snowytoes installer");
    let collection = fixture.recipe.join("collection.toml");
    let mut text = fs::read_to_string(&collection).expect("read fixture collection");
    text.push_str(
        r#"
[[runs]]
run_id = "snowytoes-bg2"
mod_id = "snowytoes"
phase = "bg2-preparation"
components = [0]
args = []

[[features]]
id = "snowytoes"
title = "Snowytoes"
description = "Fixture Snowytoes"
category = "foundation"
decision = "default"
readiness = "ready"
components = [{ run_id = "snowytoes-bg2", component = 0 }]
"#,
    );
    fs::write(collection, text).expect("add Snowytoes to the fixture collection");
}

/// The official link of a mod now serves an archive in which its installer cannot be found.
/// The installation leaves that mod out, continues, keeps that decision when it resumes, and
/// names the mod in its end summary.
#[cfg(windows)]
#[test]
fn a_mod_whose_changed_download_cannot_be_used_is_left_out_and_named_in_the_end_summary() {
    let fixture = executable_fixture();
    let reviewed = zip_bytes(&[(
        "Snowytoes/setup-snowytoes.tp2",
        b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
    )]);
    add_snowytoes(&fixture, &reviewed);
    let unusable = zip_bytes(&[(
        "Snowytoes-v1.7/readme.txt",
        b"the installer is now a separate download",
    )]);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/snowytoes.zip".to_owned(),
        unusable.clone(),
    )]));
    let marker = fixture._temp.path().join("mock-weidu-failed-once");
    fail_once_at_a_fixed_part(&fixture);

    let failed = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(answering(
            install_args(
                &fixture.recipe,
                &fixture.bg1,
                &fixture.bg2,
                &fixture.managed,
                &fixture.cache,
            ),
            &[CONTINUE_WITHOUT],
        ))
        .output()
        .expect("start the installation");
    assert_eq!(failed.status.code(), Some(1), "{}", stdout(&failed));
    let failed = json_stdout(&failed);
    assert_eq!(failed["status"]["step_id"], "install:eefixpack-bg1");
    let left_out = failed["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["type"] == "artifact_left_out")
        .collect::<Vec<_>>();
    assert_eq!(left_out.len(), 1, "{:#?}", failed["events"]);
    assert_eq!(left_out[0]["artifact_id"], "snowytoes");
    assert_eq!(left_out[0]["runs"], serde_json::json!(["snowytoes-bg2"]));
    assert_eq!(left_out[0]["changed_runs"], serde_json::json!([]));

    // Resuming rebuilds the plan from the ledger without asking the link again.
    drop(link);
    let resumed = executable_command(&fixture)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(["resume", fixture.managed.to_str().unwrap()])
        .output()
        .expect("resume the installation");
    let printed = stdout(&resumed);
    assert!(
        resumed.status.success(),
        "stdout:\n{printed}\nstderr:\n{}",
        stderr(&resumed)
    );
    let summary = printed
        .lines()
        .skip_while(|line| !line.starts_with("Managed installation ready at "))
        .skip(1)
        .collect::<Vec<_>>();
    assert_eq!(
        summary,
        [
            "You chose to continue without Snowytoes (the newer download is packed in a way the installer can't read).",
            "If something does not work as expected, please report it so the collection can be fixed."
        ],
        "{printed}"
    );

    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    assert_eq!(receipt.left_out.len(), 1);
    let snowytoes = &receipt.left_out[0];
    assert_eq!(snowytoes.artifact_id, "snowytoes");
    assert_eq!(snowytoes.runs, ["snowytoes-bg2"]);
    assert!(snowytoes.changed_runs.is_empty());
    assert!(
        snowytoes.reason.starts_with(&format!(
            "its official link now serves a different archive (snowytoes.zip, {} bytes, SHA-256 {}), which cannot be installed: ",
            unusable.len(),
            sha256_bytes(&unusable)
        )),
        "{}",
        snowytoes.reason
    );
    let frozen_runs = receipt
        .plan
        .runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        frozen_runs,
        ["eefixpack-bg1", "eefixpack-bg2", "snowytoes-bg2"]
    );
    let installed_runs = receipt
        .installed_plan
        .as_ref()
        .expect("the installed plan is recorded")
        .runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(installed_runs, ["eefixpack-bg1", "eefixpack-bg2"]);
    assert_eq!(receipt.runs.len(), 2);
    assert!(!fixture.managed.join("game/Snowytoes").exists());
    let report = report_json(&fixture);
    assert_eq!(
        report["notes"][0],
        "You chose to continue without Snowytoes (the newer download is packed in a way the installer can't read)."
    );
    let decision = &receipt.decisions[..];
    assert_eq!(decision.len(), 1, "{decision:#?}");
    assert_eq!(
        decision[0].kind,
        bg_engine::decision::QuestionKind::UnusableDownload
    );
    assert_eq!(decision[0].removed_runs, ["snowytoes-bg2"]);
    assert_eq!(
        decision[0].reason.as_ref().map(|reason| &reason.detail),
        Some(&snowytoes.reason)
    );
}

/// Test builds of the installer skip the waits between download attempts when this is set.
#[cfg(windows)]
const NO_RETRY_WAIT: &str = "CHRIZ_BG_COLLECTION_TEST_NO_RETRY_WAIT";

/// The official link of a mod serves an error page instead of its archive, or no file at all,
/// on every attempt. The installer asks again with the usual waits, then leaves the mod out and
/// installs the rest.
#[cfg(windows)]
#[test]
fn a_mod_whose_link_serves_no_archive_is_left_out_after_every_attempt() {
    let page =
        b"<!DOCTYPE html>\n<html><body>This download is not available right now.</body></html>\n";
    for (served, reason, short) in [
        (
            Some(page.to_vec()),
            "its official link does not serve the archive (it does not begin like a ZIP archive but with the text \"<!DOCTYPE html> <html><body>This\"), and still did not on the last of 5 attempts; the last answer came from 127.0.0.1",
            "its link no longer serves the archive",
        ),
        (
            None,
            "its official link answers HTTP 404 (no such file) from 127.0.0.1, and still did on the last of 5 attempts",
            "its link no longer has the file",
        ),
    ] {
        let fixture = executable_fixture();
        let reviewed = zip_bytes(&[(
            "Snowytoes/setup-snowytoes.tp2",
            b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
        )]);
        add_snowytoes(&fixture, &reviewed);
        let link = ArtifactServer::start(
            served
                .map(|body| ("/snowytoes.zip".to_owned(), body))
                .into_iter()
                .collect(),
        );

        let output = executable_command(&fixture)
            .env(DOWNLOAD_ORIGIN, &link.base_url)
            .env(NO_RETRY_WAIT, "1")
            .args(answering(
                install_args(
                    &fixture.recipe,
                    &fixture.bg1,
                    &fixture.bg2,
                    &fixture.managed,
                    &fixture.cache,
                ),
                &[CONTINUE_WITHOUT],
            ))
            .output()
            .expect("run the installation");
        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            stdout(&output),
            stderr(&output)
        );
        let response = json_stdout(&output);
        let lines = console_lines(&response);
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.starts_with("The download of snowytoes gave no usable file: "))
                .count(),
            4,
            "{lines:#?}"
        );

        let receipt = read_install_receipt(&fixture);
        assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
        assert_eq!(receipt.left_out.len(), 1);
        assert_eq!(receipt.left_out[0].artifact_id, "snowytoes");
        assert_eq!(receipt.left_out[0].reason, reason);
        assert_eq!(receipt.left_out[0].runs, ["snowytoes-bg2"]);
        // Nothing of the answer was accepted, cached or placed.
        assert!(!receipt
            .artifacts
            .iter()
            .any(|artifact| artifact.id == "snowytoes"));
        assert!(!fixture.cache.join("partial/snowytoes.part").exists());
        assert!(!fixture.managed.join("game/Snowytoes").exists());
        assert_eq!(
            report_json(&fixture)["notes"],
            serde_json::json!([
                format!("You chose to continue without Snowytoes ({short})."),
                "If something does not work as expected, please report it so the collection can be fixed."
            ])
        );
    }
}

/// The changed archive of a mod would overwrite a file that an earlier mod placed. Nothing of
/// it is placed yet, so the installer leaves the mod out instead of requiring a fresh copy, and
/// a resume keeps that decision.
#[cfg(windows)]
#[test]
fn a_changed_archive_that_would_overwrite_another_mods_file_is_left_out() {
    let fixture = executable_fixture();
    let fixpack_tp2: &[u8] = b"BEGIN ~Synthetic EE Fixpack~\nDESIGNATED 0\n";
    // The reviewed release carries an identical copy of an EE Fixpack file.
    let reviewed = zip_bytes(&[
        (
            "Snowytoes/setup-snowytoes.tp2",
            b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
        ),
        ("EE_Fixpack/EE_Fixpack.tp2", fixpack_tp2),
    ]);
    add_snowytoes_publishing(&fixture, &reviewed, &["Snowytoes", "EE_Fixpack"]);
    let changed = zip_bytes(&[
        (
            "Snowytoes/setup-snowytoes.tp2",
            b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
        ),
        ("EE_Fixpack/EE_Fixpack.tp2", b"BEGIN ~Its own edit~\n"),
    ]);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/snowytoes.zip".to_owned(),
        changed.clone(),
    )]));
    let marker = fixture._temp.path().join("mock-weidu-failed-once");
    fail_once_at_a_fixed_part(&fixture);

    let failed = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(answering(
            install_args(
                &fixture.recipe,
                &fixture.bg1,
                &fixture.bg2,
                &fixture.managed,
                &fixture.cache,
            ),
            &[INSTALL_CHANGED, CONTINUE_WITHOUT],
        ))
        .output()
        .expect("start the installation");
    assert_eq!(failed.status.code(), Some(1), "{}", stdout(&failed));
    let failed = json_stdout(&failed);
    assert_eq!(failed["status"]["step_id"], "install:eefixpack-bg1");
    let left_out = failed["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["type"] == "artifact_left_out")
        .collect::<Vec<_>>();
    assert_eq!(left_out.len(), 1, "{:#?}", failed["events"]);
    assert_eq!(left_out[0]["runs"], serde_json::json!(["snowytoes-bg2"]));
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    assert!(replay.records.iter().any(|record| matches!(
        &record.event,
        bg_engine::session::SessionEvent::ArtifactLeftOut { step_id, artifact_id, .. }
            if step_id == "materialize:snowytoes:bg2" && artifact_id == "snowytoes"
    )));

    drop(link);
    let resumed = executable_command(&fixture)
        .env("CHRIZ_TEST_MOCK_WEIDU_FAIL_ONCE", &marker)
        .args(["resume", fixture.managed.to_str().unwrap()])
        .output()
        .expect("resume the installation");
    assert!(
        resumed.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&resumed),
        stderr(&resumed)
    );

    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    assert_eq!(receipt.left_out.len(), 1);
    assert_eq!(
        receipt.left_out[0].reason,
        format!(
            "its official link now serves a different archive (snowytoes.zip, {} bytes, SHA-256 {}), which cannot be installed: its file `EE_Fixpack/EE_Fixpack.tp2` would overwrite different content that the game or an earlier mod already placed",
            changed.len(),
            sha256_bytes(&changed)
        )
    );
    assert_eq!(
        fs::read(fixture.managed.join("game/EE_Fixpack/EE_Fixpack.tp2")).unwrap(),
        fixpack_tp2
    );
    assert!(!fixture.managed.join("game/Snowytoes").exists());
    let installed_runs = receipt
        .installed_plan
        .as_ref()
        .expect("the installed plan is recorded")
        .runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(installed_runs, ["eefixpack-bg1", "eefixpack-bg2"]);
}

/// The changed archive of a mod has a file where an earlier mod placed a folder, or needs a
/// folder where an earlier mod placed a file. Nothing of it is placed yet, so the installer
/// leaves the mod out, as for an overwrite, instead of stopping at that step on every resume.
#[cfg(windows)]
#[test]
fn a_changed_archive_that_meets_a_file_or_folder_of_another_mod_is_left_out() {
    let fixpack_tp2: &[u8] = b"BEGIN ~Synthetic EE Fixpack~\nDESIGNATED 0\n";
    let snowytoes_tp2: &[u8] = b"BEGIN ~Snowytoes~\nDESIGNATED 0\n";
    for (in_the_way, reason) in [
        (
            "EE_Fixpack/EE_Fixpack.tp2/readme.txt",
            "its file `EE_Fixpack/EE_Fixpack.tp2/readme.txt` cannot be placed: the game or an earlier mod already placed a file `EE_Fixpack/EE_Fixpack.tp2` where it needs a folder",
        ),
        (
            "EE_Fixpack",
            "its file `EE_Fixpack` cannot be placed: the game or an earlier mod already placed a folder of that name",
        ),
    ] {
        let fixture = executable_fixture();
        // The reviewed release carries an identical copy of an EE Fixpack file.
        let reviewed = zip_bytes(&[
            ("Snowytoes/setup-snowytoes.tp2", snowytoes_tp2),
            ("EE_Fixpack/EE_Fixpack.tp2", fixpack_tp2),
        ]);
        add_snowytoes_publishing(&fixture, &reviewed, &["Snowytoes", "EE_Fixpack"]);
        let changed = zip_bytes(&[
            ("Snowytoes/setup-snowytoes.tp2", snowytoes_tp2),
            (in_the_way, b"in the way"),
        ]);
        let link = ArtifactServer::start(BTreeMap::from([(
            "/snowytoes.zip".to_owned(),
            changed.clone(),
        )]));

        let output = executable_command(&fixture)
            .env(DOWNLOAD_ORIGIN, &link.base_url)
            .args(answering(
                install_args(
                    &fixture.recipe,
                    &fixture.bg1,
                    &fixture.bg2,
                    &fixture.managed,
                    &fixture.cache,
                ),
                &[INSTALL_CHANGED, CONTINUE_WITHOUT],
            ))
            .output()
            .expect("run the installation");

        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            stdout(&output),
            stderr(&output)
        );
        let receipt = read_install_receipt(&fixture);
        assert_eq!(receipt.left_out.len(), 1);
        assert_eq!(
            receipt.left_out[0].reason,
            format!(
                "its official link now serves a different archive (snowytoes.zip, {} bytes, SHA-256 {}), which cannot be installed: {reason}",
                changed.len(),
                sha256_bytes(&changed)
            )
        );
        assert_eq!(receipt.left_out[0].runs, ["snowytoes-bg2"]);
        assert_eq!(
            fs::read(fixture.managed.join("game/EE_Fixpack/EE_Fixpack.tp2")).unwrap(),
            fixpack_tp2
        );
        assert!(!fixture.managed.join("game/Snowytoes").exists());
        // Nothing began to place its files, so nothing records that it did.
        assert_eq!(
            fs::read_dir(fixture.managed.join(".chriz/evidence/publications"))
                .map_or(0, Iterator::count),
            0
        );
    }
}

/// A placing step of a changed archive writes its first file, and the installation stops
/// before the step records that file, as after a crash. Someone then edits the file. The
/// collision this causes on resume is no proof that nothing was written: the step had begun to
/// place files, so the copy stays sealed instead of leaving the mod out with its file behind.
#[cfg(windows)]
#[test]
fn an_interrupted_placing_of_a_changed_archive_stays_sealed_when_its_file_changed() {
    let fixture = executable_fixture();
    let reviewed = zip_bytes(&[(
        "Snowytoes/setup-snowytoes.tp2",
        b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
    )]);
    add_snowytoes(&fixture, &reviewed);
    let changed = zip_bytes(&[
        ("Snowytoes/readme.txt", b"new in 1.7"),
        (
            "Snowytoes/setup-snowytoes.tp2",
            b"BEGIN ~Snowytoes 1.7~\nDESIGNATED 0\n",
        ),
    ]);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/snowytoes.zip".to_owned(),
        changed.clone(),
    )]));

    let interrupted = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .env(
            "CHRIZ_BG_COLLECTION_TEST_INTERRUPT_PUBLICATION",
            "snowytoes",
        )
        .args(answering(
            install_args(
                &fixture.recipe,
                &fixture.bg1,
                &fixture.bg2,
                &fixture.managed,
                &fixture.cache,
            ),
            &[INSTALL_CHANGED],
        ))
        .output()
        .expect("start the installation");
    assert_eq!(
        interrupted.status.code(),
        Some(1),
        "{}",
        stdout(&interrupted)
    );
    assert_eq!(
        json_stdout(&interrupted)["status"]["step_id"],
        "materialize:snowytoes:bg2"
    );
    // The first file is placed, the second is not, and the publication manifest records
    // neither.
    let placed = fixture.managed.join("game/Snowytoes/readme.txt");
    assert_eq!(fs::read(&placed).unwrap(), b"new in 1.7");
    assert!(!fixture
        .managed
        .join("game/Snowytoes/setup-snowytoes.tp2")
        .exists());
    let manifest = fs::read_to_string(
        fixture
            .managed
            .join("game/.chriz-bg-collection/publication-manifest.json"),
    )
    .unwrap();
    assert!(!manifest.contains("Snowytoes"), "{manifest}");
    fs::write(&placed, b"edited by hand").unwrap();

    drop(link);
    let resumed = executable_command(&fixture)
        .args(["--json", "resume", fixture.managed.to_str().unwrap()])
        .output()
        .expect("resume the installation");

    assert_eq!(resumed.status.code(), Some(1), "{}", stdout(&resumed));
    let resumed = json_stdout(&resumed);
    assert_eq!(
        resumed["status"]["status"], "fresh_copy_required",
        "{resumed:#}"
    );
    assert_eq!(resumed["status"]["step_id"], "materialize:snowytoes:bg2");
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    assert!(!replay.records.iter().any(|record| matches!(
        &record.event,
        bg_engine::session::SessionEvent::ArtifactLeftOut { .. }
    )));
    assert!(replay.records.iter().any(|record| matches!(
        &record.event,
        bg_engine::session::SessionEvent::FreshCopyRequired { step_id, .. }
            if step_id == "materialize:snowytoes:bg2"
    )));
    assert_eq!(fs::read(&placed).unwrap(), b"edited by hand");
}

/// The record that a placing step of a changed archive begins to write is itself interrupted,
/// as by a crash or a full disk: half of it is written and its temporary file stays behind,
/// before any file of the mod is placed. The step has not begun, so a resume places the mod;
/// and when a file of another mod now stands in its way, the resume leaves it out, as before
/// any write. A half-written record must never stop every later resume.
#[cfg(windows)]
#[test]
fn an_interrupted_record_that_a_placing_step_begins_leaves_the_step_to_begin_again() {
    let interrupt = |fixture: &ExecutableFixture, link: &ArtifactServer, mod_id: &str| {
        let interrupted = executable_command(fixture)
            .env(DOWNLOAD_ORIGIN, &link.base_url)
            .env("CHRIZ_BG_COLLECTION_TEST_INTERRUPT_RECORD", "publications")
            .args(answering(
                install_args(
                    &fixture.recipe,
                    &fixture.bg1,
                    &fixture.bg2,
                    &fixture.managed,
                    &fixture.cache,
                ),
                &[INSTALL_CHANGED],
            ))
            .output()
            .expect("start the installation");
        assert_eq!(
            interrupted.status.code(),
            Some(1),
            "{}",
            stdout(&interrupted)
        );
        let step_id = json_stdout(&interrupted)["status"]["step_id"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(
            step_id.starts_with(&format!("materialize:{mod_id}:")),
            "{step_id}"
        );
        // Only the temporary file of the unfinished record is there, no record.
        let left_behind = files(&fixture.managed.join(".chriz/evidence/publications"));
        assert_eq!(left_behind.len(), 1, "{left_behind:?}");
        assert_ne!(
            left_behind[0]
                .extension()
                .and_then(|extension| extension.to_str()),
            Some("json"),
            "{left_behind:?}"
        );
    };
    let resume = |fixture: &ExecutableFixture| {
        let resumed = executable_command(fixture)
            .args([
                "--json",
                "resume",
                fixture.managed.to_str().unwrap(),
                "--answer",
                CONTINUE_WITHOUT,
            ])
            .output()
            .expect("resume the installation");
        assert!(
            resumed.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            stdout(&resumed),
            stderr(&resumed)
        );
        let records = files(&fixture.managed.join(".chriz/evidence/publications"))
            .into_iter()
            .filter(|path| {
                path.extension().and_then(|extension| extension.to_str()) == Some("json")
            })
            .map(|path| {
                let record: serde_json::Value =
                    serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
                record["step_id"].as_str().unwrap().to_owned()
            })
            .collect::<BTreeSet<_>>();
        (read_install_receipt(fixture), records)
    };

    // A changed EE Fixpack: the resume places it in both games.
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.payload);
    let changed = zip_bytes(&[
        (
            "EE_Fixpack/EE_Fixpack.tp2",
            b"BEGIN ~A newer release~\nDESIGNATED 0\n",
        ),
        ("EE_Fixpack/newer.txt", b"new in this release"),
    ]);
    let link = ArtifactServer::start(BTreeMap::from([("/eefixpack.zip".to_owned(), changed)]));
    interrupt(&fixture, &link, "eefixpack");
    for game in ["bg1", "game"] {
        assert!(!fixture.managed.join(game).join("EE_Fixpack").exists());
    }
    drop(link);
    let (receipt, records) = resume(&fixture);
    assert!(receipt.left_out.is_empty());
    for game in ["bg1", "game"] {
        assert_eq!(
            fs::read(fixture.managed.join(game).join("EE_Fixpack/newer.txt")).unwrap(),
            b"new in this release"
        );
    }
    assert_eq!(
        records,
        BTreeSet::from([
            "materialize:eefixpack:bg1".to_owned(),
            "materialize:eefixpack:bg2".to_owned(),
        ])
    );

    // A changed Snowytoes that a file now stands in the way of: the resume leaves it out.
    let fixture = executable_fixture();
    add_snowytoes(
        &fixture,
        &zip_bytes(&[(
            "Snowytoes/setup-snowytoes.tp2",
            b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
        )]),
    );
    let link = ArtifactServer::start(BTreeMap::from([(
        "/snowytoes.zip".to_owned(),
        zip_bytes(&[(
            "Snowytoes/setup-snowytoes.tp2",
            b"BEGIN ~Snowytoes 1.7~\nDESIGNATED 0\n",
        )]),
    )]));
    interrupt(&fixture, &link, "snowytoes");
    assert!(!fixture.managed.join("game/Snowytoes").exists());
    fs::write(fixture.managed.join("game/Snowytoes"), b"in the way").unwrap();
    drop(link);
    let (receipt, records) = resume(&fixture);
    assert_eq!(receipt.left_out.len(), 1);
    assert_eq!(receipt.left_out[0].runs, ["snowytoes-bg2"]);
    assert_eq!(
        fs::read(fixture.managed.join("game/Snowytoes")).unwrap(),
        b"in the way"
    );
    assert!(records.is_empty(), "{records:?}");
}

fn files(folder: &Path) -> Vec<PathBuf> {
    fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect()
}

/// A changed archive whose stored data is damaged, as when its server keeps a broken copy: its
/// headers and directory are intact, so the download looks complete, and only unpacking finds
/// the bad checksum. The installer removes the damaged file from the download folder, asks the
/// link again with the usual waits, and leaves the mod out once every attempt was damaged. A
/// link that serves an intact copy on a later attempt installs that copy.
#[cfg(windows)]
#[test]
fn a_damaged_changed_archive_is_asked_for_again_and_left_out_only_when_it_stays_damaged() {
    let data: Vec<u8> = (0..4096_u32)
        .map(|index| (index.wrapping_mul(2_654_435_761) >> 11) as u8)
        .collect();
    // An intact changed archive of `folder`, and the same with one stored data byte changed.
    let archives = |tp2: &str, data_path: &str| {
        let intact = zip_bytes(&[
            (tp2, b"BEGIN ~A newer release~\nDESIGNATED 0\n"),
            (data_path, &data),
        ]);
        let mut damaged = intact.clone();
        let data_start = intact
            .windows(data.len())
            .position(|window| window == data.as_slice())
            .expect("the stored data is in the archive");
        damaged[data_start + 100] ^= 0x55;
        (intact, damaged)
    };
    let request_counter = |path: &'static str| {
        let asked = Arc::new(AtomicU64::new(0));
        let counted = Arc::clone(&asked);
        (asked, move |requested: &str| {
            if requested == path {
                counted.fetch_add(1, Ordering::SeqCst);
            }
        })
    };
    let install = |fixture: &ExecutableFixture, link: &ArtifactServer| {
        let output = executable_command(fixture)
            .env(DOWNLOAD_ORIGIN, &link.base_url)
            .env(NO_RETRY_WAIT, "1")
            .args(answering(
                install_args(
                    &fixture.recipe,
                    &fixture.bg1,
                    &fixture.bg2,
                    &fixture.managed,
                    &fixture.cache,
                ),
                &[INSTALL_CHANGED, CONTINUE_WITHOUT],
            ))
            .output()
            .expect("run the installation");
        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            stdout(&output),
            stderr(&output)
        );
        console_lines(&json_stdout(&output))
    };
    let not_cached = |fixture: &ExecutableFixture, bytes: &[u8]| {
        let digest = sha256_bytes(bytes);
        !fixture
            .cache
            .join("sha256")
            .join(&digest[..2])
            .join(format!("{digest}.archive"))
            .exists()
    };

    // Every answer is damaged: Snowytoes is left out after the last attempt.
    let fixture = executable_fixture();
    add_snowytoes(
        &fixture,
        &zip_bytes(&[(
            "Snowytoes/setup-snowytoes.tp2",
            b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
        )]),
    );
    let (_, damaged) = archives("Snowytoes/setup-snowytoes.tp2", "Snowytoes/data.bin");
    let (asked, on_request) = request_counter("/snowytoes.zip");
    let link = ArtifactServer::start_answering(
        BTreeMap::from([(
            "/snowytoes.zip".to_owned(),
            vec![Answer::file(damaged.clone())],
        )]),
        on_request,
    );
    let lines = install(&fixture, &link);
    assert_eq!(asked.load(Ordering::SeqCst), 5);
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with(
                "The download of snowytoes gave no usable file: its archive is damaged (archive entry `Snowytoes/data.bin` is damaged: "
            ))
            .count(),
        4,
        "{lines:#?}"
    );
    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    assert_eq!(receipt.left_out.len(), 1);
    let reason = &receipt.left_out[0].reason;
    assert!(
        reason.starts_with(&format!(
            "its official link now serves a different archive (snowytoes.zip, {} bytes, SHA-256 {}), which cannot be installed: archive entry `Snowytoes/data.bin` is damaged: ",
            damaged.len(),
            sha256_bytes(&damaged)
        )) && reason.ends_with("; it was still damaged on the last of 5 attempts"),
        "{reason}"
    );
    assert_eq!(receipt.left_out[0].runs, ["snowytoes-bg2"]);
    // Nothing of it was accepted, kept or placed.
    assert!(!receipt
        .artifacts
        .iter()
        .any(|artifact| artifact.id == "snowytoes"));
    assert!(not_cached(&fixture, &damaged));
    assert!(!fixture.managed.join("game/Snowytoes").exists());

    // The second answer is intact: the changed EE Fixpack is installed from it.
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.payload);
    let (intact, damaged) = archives("EE_Fixpack/EE_Fixpack.tp2", "EE_Fixpack/data.bin");
    let (asked, on_request) = request_counter("/eefixpack.zip");
    let link = ArtifactServer::start_answering(
        BTreeMap::from([(
            "/eefixpack.zip".to_owned(),
            vec![Answer::file(damaged.clone()), Answer::file(intact.clone())],
        )]),
        on_request,
    );
    let lines = install(&fixture, &link);
    assert_eq!(asked.load(Ordering::SeqCst), 2);
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with(
                "The download of eefixpack gave no usable file: its archive is damaged (archive entry `EE_Fixpack/data.bin` is damaged: "
            ))
            .count(),
        1,
        "{lines:#?}"
    );
    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    assert!(receipt.left_out.is_empty());
    let recorded = receipt
        .artifacts
        .iter()
        .find(|artifact| artifact.id == "eefixpack")
        .expect("the intact copy is recorded");
    assert_eq!(recorded.sha256, sha256_bytes(&intact));
    assert!(not_cached(&fixture, &damaged));
    for game in ["bg1", "game"] {
        assert_eq!(
            fs::read(fixture.managed.join(game).join("EE_Fixpack/data.bin")).unwrap(),
            data
        );
    }
}

/// The review's coverage gap: a link that answers 410 and one that announces a changed
/// download larger than the installer accepts leave the mod out and install the rest, as a
/// link that answers 404 does. A size the installer refuses is not asked for again.
#[cfg(windows)]
#[test]
fn a_mod_whose_link_is_gone_or_serves_too_much_is_left_out() {
    let reviewed = zip_bytes(&[(
        "Snowytoes/setup-snowytoes.tp2",
        b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
    )]);
    let limit = 1024 * 1024 * 1024_u64;
    for (answer, reason, requests) in [
        (
            Answer::status("410 Gone"),
            "its official link answers HTTP 410 (no such file) from 127.0.0.1, and still did on the last of 5 attempts".to_owned(),
            5,
        ),
        (
            Answer {
                status: "200 OK",
                body: Vec::new(),
                announced: Some(limit + 1),
                elsewhere: None,
            },
            format!(
                "its official link now serves a download of at least {} bytes, more than the {limit} bytes the installer accepts for a changed download",
                limit + 1
            ),
            1,
        ),
    ] {
        let fixture = executable_fixture();
        add_snowytoes(&fixture, &reviewed);
        let asked = Arc::new(AtomicU64::new(0));
        let counted = Arc::clone(&asked);
        let link = ArtifactServer::start_answering(
            BTreeMap::from([("/snowytoes.zip".to_owned(), vec![answer])]),
            move |path| {
                if path == "/snowytoes.zip" {
                    counted.fetch_add(1, Ordering::SeqCst);
                }
            },
        );

        let output = executable_command(&fixture)
            .env(DOWNLOAD_ORIGIN, &link.base_url)
            .env(NO_RETRY_WAIT, "1")
            .args(answering(
                install_args(
                    &fixture.recipe,
                    &fixture.bg1,
                    &fixture.bg2,
                    &fixture.managed,
                    &fixture.cache,
                ),
                &[CONTINUE_WITHOUT],
            ))
            .output()
            .expect("run the installation");

        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            stdout(&output),
            stderr(&output)
        );
        assert_eq!(asked.load(Ordering::SeqCst), requests);
        let receipt = read_install_receipt(&fixture);
        assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
        assert_eq!(receipt.left_out.len(), 1);
        assert_eq!(receipt.left_out[0].artifact_id, "snowytoes");
        assert_eq!(receipt.left_out[0].reason, reason);
        assert_eq!(receipt.left_out[0].runs, ["snowytoes-bg2"]);
        assert!(!fixture.cache.join("partial/snowytoes.part").exists());
        assert!(!fixture.managed.join("game/Snowytoes").exists());
        assert_eq!(receipt.runs.len(), 2);
    }
}

/// The official link of the installer tool serves a newer WeiDU build than the reviewed one.
/// The installer checks it as a changed tool, installs every mod with it, says so, and records
/// both builds; the report reads and checks that record.
#[cfg(windows)]
#[test]
fn a_changed_weidu_download_is_checked_used_and_recorded() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.tool_archive);
    let reviewed_tool = fs::read(MOCK_CHILD).expect("read the synthetic WeiDU");
    // Bytes after a Windows program's image do not change what it does.
    let mut newer = reviewed_tool.clone();
    newer.extend_from_slice(b"\0a newer build of the same program");
    let newer_archive = zip_bytes(&[("weidu.exe", &newer)]);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/weidu.zip".to_owned(),
        newer_archive.clone(),
    )]));

    let output = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .args(answering(
            install_args(
                &fixture.recipe,
                &fixture.bg1,
                &fixture.bg2,
                &fixture.managed,
                &fixture.cache,
            ),
            &[INSTALL_CHANGED],
        ))
        .output()
        .expect("run the installation");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let lines = console_lines(&json_stdout(&output));
    assert!(
        lines.iter().any(|line| line.starts_with(
            "The download of WeiDU 249.00 is newer or different than the reviewed version."
        )),
        "{lines:#?}"
    );

    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    let archive = receipt
        .artifacts
        .iter()
        .find(|artifact| artifact.id == "weidu")
        .expect("the tool archive is recorded");
    assert_eq!(archive.sha256, sha256_bytes(&newer_archive));
    assert!(archive.served_difference.is_some());
    let tool = receipt
        .weidu_tools
        .iter()
        .find(|tool| tool.id == "weidu")
        .expect("the tool is recorded");
    assert_eq!(tool.version, "24900");
    assert_eq!(tool.length, newer.len() as u64);
    assert_eq!(tool.sha256, sha256_bytes(&newer));
    assert_eq!(
        tool.reviewed,
        Some(ReviewedTool {
            version: "24900".to_owned(),
            length: reviewed_tool.len() as u64,
            sha256: sha256_bytes(&reviewed_tool),
        })
    );
    assert_eq!(receipt.runs.len(), 2);
    assert_eq!(
        report_json(&fixture)["notes"],
        serde_json::json!([
            "You chose to install a newer download of WeiDU (weidu.zip).",
            "If something does not work as expected, please report it so the collection can be fixed."
        ])
    );
}

/// The installer tool's link serves a changed build that cannot be used, while the reviewed
/// one reaches the download folder, as when another installation finishes downloading it at
/// the same time. The installer uses the reviewed tool and says so; nothing records the
/// changed one.
#[cfg(windows)]
#[test]
fn an_unusable_changed_weidu_gives_way_to_the_reviewed_one_in_the_download_folder() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.tool_archive);
    let unusable = zip_bytes(&[("weidu.exe", b"MZ, but no program at all")]);
    let cache = fixture.cache.clone();
    let reviewed_url = format!("{}/weidu.zip", fixture._server.base_url);
    let reviewed = fixture.tool_archive.clone();
    let link = ArtifactServer::start_with(
        BTreeMap::from([("/weidu.zip".to_owned(), unusable.clone())]),
        move |path| {
            if path == "/weidu.zip" {
                let (sink, _events) = ChannelSink::unbounded();
                ArtifactCache::open(&cache)
                    .expect("open the download folder")
                    .acquire(
                        &DownloadRequest {
                            request_id: "another-installation".to_owned(),
                            url: reviewed_url.clone(),
                            expected_length: reviewed.len() as u64,
                            expected_sha256: sha256_bytes(&reviewed),
                            redirect_hosts: Vec::new(),
                            max_attempts: 1,
                        },
                        &sink,
                    )
                    .expect("another installation downloads the reviewed tool");
            }
        },
    );

    let output = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .output()
        .expect("run the installation");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let lines = console_lines(&json_stdout(&output));
    assert!(
        lines.iter().any(|line| line.starts_with(
            "The download of WeiDU 249.00 cannot be used: it is newer or different than the reviewed version, and its WeiDU program does not work: "
        ) && line.ends_with("Using the reviewed WeiDU from the download folder instead.")),
        "{lines:#?}"
    );

    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    let archive = receipt
        .artifacts
        .iter()
        .find(|artifact| artifact.id == "weidu")
        .expect("the tool archive is recorded");
    assert_eq!(archive.sha256, sha256_bytes(&fixture.tool_archive));
    assert_eq!(archive.served_difference, None);
    assert_eq!(archive.cache_outcome, ArtifactCacheOutcome::Hit);
    assert!(receipt
        .weidu_tools
        .iter()
        .all(|tool| tool.reviewed.is_none()));
    // The installation used only reviewed files, so its end summary has nothing to add.
    assert_eq!(report_json(&fixture)["notes"], serde_json::json!([]));
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
    fail_once_at_a_fixed_part(&fixture);
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

/// Stops the install of eefixpack-bg2 before WeiDU starts. `kind` names the guard that fails, or
/// is "weidu-executable" for a failure to get the run's WeiDU executable.
#[cfg(windows)]
fn blocked_before_spawn_fixture(kind: &str) -> (ExecutableFixture, PathBuf) {
    let fixture = executable_fixture();
    let mut install = executable_command(&fixture);
    if kind == "weidu-executable" {
        install.env("CHRIZ_BG_COLLECTION_TEST_FAIL_TOOL", "eefixpack-bg2");
    } else {
        install.env(
            "CHRIZ_BG_COLLECTION_TEST_FAIL_RECHECK",
            format!("{kind}:install:eefixpack-bg2"),
        );
    }
    install.args(install_args(
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
    if kind == "weidu-executable" {
        assert!(
            stdout(&output).contains("simulated failure to get the WeiDU executable"),
            "{}",
            stdout(&output)
        );
    }
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
    assert!(attempt.join("pre-spawn-failure.json").is_file());
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
        ("weidu-executable", false),
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

// ----- Questions about downloads, end to end -----

/// Adds Snowfall, a mod from its own download whose feature needs Snowytoes.
#[cfg(windows)]
fn add_snowfall(fixture: &ExecutableFixture, reviewed: &[u8]) {
    fs::write(
        fixture.recipe.join("artifacts/snowfall.toml"),
        artifact_toml(
            "snowfall",
            "Snowfall",
            "2.0",
            "https://example.invalid/snowfall.zip",
            "Snowfall-v2.0.zip",
            reviewed,
            &["Snowfall"],
            &["Snowfall/setup-snowfall.tp2"],
            None,
        ),
    )
    .expect("write the Snowfall download");
    fs::write(
        fixture.recipe.join("mods/snowfall.toml"),
        r#"id = "snowfall"
artifact_id = "snowfall"
name = "Snowfall"
tp2 = "Snowfall/setup-snowfall.tp2"
language = 0
weidu_artifact_id = "weidu"
invocation_mode = "setup-name"

[[components]]
id = 0
name = "Falling snow"
"#,
    )
    .expect("write the Snowfall installer");
    let collection = fixture.recipe.join("collection.toml");
    let mut text = fs::read_to_string(&collection).expect("read fixture collection");
    text.push_str(
        r#"
[[runs]]
run_id = "snowfall-bg2"
mod_id = "snowfall"
phase = "bg2-preparation"
components = [0]
args = []

[[features]]
id = "snowfall"
title = "Snowfall"
description = "Fixture Snowfall"
category = "foundation"
decision = "default"
readiness = "ready"
requires = ["snowytoes"]
components = [{ run_id = "snowfall-bg2", component = 0 }]
"#,
    );
    fs::write(collection, text).expect("add Snowfall to the fixture collection");
}

/// Makes Snowytoes a part the collection always installs.
#[cfg(windows)]
fn make_snowytoes_fixed(fixture: &ExecutableFixture) {
    let collection = fixture.recipe.join("collection.toml");
    let text = fs::read_to_string(&collection).expect("read fixture collection");
    let optional =
        "description = \"Fixture Snowytoes\"\ncategory = \"foundation\"\ndecision = \"default\"";
    assert_eq!(text.matches(optional).count(), 1);
    fs::write(
        collection,
        text.replace(
            optional,
            "description = \"Fixture Snowytoes\"\ncategory = \"foundation\"\ndecision = \"mandatory\"",
        ),
    )
    .expect("make Snowytoes fixed");
}

#[cfg(windows)]
fn snowytoes_release(version: &str) -> Vec<u8> {
    zip_bytes(&[(
        "Snowytoes/setup-snowytoes.tp2",
        format!("BEGIN ~Snowytoes {version}~\nDESIGNATED 0\n").as_bytes(),
    )])
}

#[cfg(windows)]
fn snowfall_release() -> Vec<u8> {
    zip_bytes(&[(
        "Snowfall/setup-snowfall.tp2",
        b"BEGIN ~Snowfall~\nDESIGNATED 0\n",
    )])
}

/// Exit code and JSON output of one installer command of `fixture`, with the link `origin`.
#[cfg(windows)]
fn run_installer(
    fixture: &ExecutableFixture,
    origin: Option<&ArtifactServer>,
    args: &[&str],
) -> (Option<i32>, Value) {
    run_installer_with(fixture, origin, &[], args)
}

/// [`run_installer`] with further environment variables.
#[cfg(windows)]
fn run_installer_with(
    fixture: &ExecutableFixture,
    origin: Option<&ArtifactServer>,
    envs: &[(&str, &std::ffi::OsStr)],
    args: &[&str],
) -> (Option<i32>, Value) {
    let mut command = executable_command(fixture);
    command.envs(envs.iter().copied());
    command.env(NO_RETRY_WAIT, "1");
    if let Some(origin) = origin {
        command.env(DOWNLOAD_ORIGIN, &origin.base_url);
    }
    let output = command.args(args).output().expect("run the installer");
    (output.status.code(), json_stdout(&output))
}

#[cfg(windows)]
fn install_answering(
    fixture: &ExecutableFixture,
    origin: &ArtifactServer,
    answers: &[&str],
) -> (Option<i32>, Value) {
    let args = answering(
        install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ),
        answers,
    );
    run_installer(fixture, Some(origin), &args)
}

#[cfg(windows)]
fn resume_answering(
    fixture: &ExecutableFixture,
    origin: Option<&ArtifactServer>,
    answers: &[&str],
) -> (Option<i32>, Value) {
    let args = answering(
        vec!["--json", "resume", fixture.managed.to_str().unwrap()],
        answers,
    );
    run_installer(fixture, origin, &args)
}

/// The question the installation waits for, as `check-decision` shows it, or `null`.
#[cfg(windows)]
fn check_decision(fixture: &ExecutableFixture, install_id: &str) -> Value {
    let (code, response) = run_installer(
        fixture,
        None,
        &[
            "--json",
            "check-decision",
            fixture.managed.to_str().unwrap(),
            "--install-id",
            install_id,
        ],
    );
    assert_eq!(code, Some(0), "{response:#}");
    response["decision"].clone()
}

/// Answers `decision` with `answer` and `token`; exit code and JSON output.
#[cfg(windows)]
fn decide_with(
    fixture: &ExecutableFixture,
    decision: &Value,
    token: &str,
    answer: &str,
) -> (Option<i32>, Value) {
    run_installer(
        fixture,
        None,
        &[
            "--json",
            "decide",
            fixture.managed.to_str().unwrap(),
            "--install-id",
            decision["install_id"].as_str().unwrap(),
            "--decision",
            decision["decision_id"].as_str().unwrap(),
            "--token",
            token,
            "--answer",
            answer,
        ],
    )
}

#[cfg(windows)]
fn decide(fixture: &ExecutableFixture, decision: &Value, answer: &str) {
    let (code, response) = decide_with(
        fixture,
        decision,
        decision["token"].as_str().unwrap(),
        answer,
    );
    assert_eq!(code, Some(0), "{response:#}");
}

/// The id of the question a response says the installation waits for.
#[cfg(windows)]
fn waiting_for(code: Option<i32>, response: &Value, step_id: &str) -> String {
    assert_eq!(code, Some(0), "{response:#}");
    assert_eq!(response["ok"], true, "{response:#}");
    assert_eq!(
        response["status"]["status"], "decision_required",
        "{response:#}"
    );
    assert_eq!(response["status"]["step_id"], step_id, "{response:#}");
    let decision_id = response["status"]["decision_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        response["events"].as_array().unwrap().iter().any(|event| {
            event["type"] == "decision_required" && event["decision_id"] == decision_id.as_str()
        }),
        "{response:#}"
    );
    decision_id
}

#[cfg(windows)]
fn labels(decision: &Value) -> Vec<String> {
    decision["texts"]["answers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|answer| answer["label"].as_str().unwrap().to_owned())
        .collect()
}

#[cfg(windows)]
fn is_cached(fixture: &ExecutableFixture, bytes: &[u8]) -> bool {
    let digest = sha256_bytes(bytes);
    fixture
        .cache
        .join("sha256")
        .join(&digest[..2])
        .join(format!("{digest}.archive"))
        .exists()
}

/// A link that counts the requests for `path`.
#[cfg(windows)]
fn counting_link(
    answers: BTreeMap<String, Vec<Answer>>,
    path: &'static str,
) -> (ArtifactServer, Arc<AtomicU64>) {
    let asked = Arc::new(AtomicU64::new(0));
    let counted = Arc::clone(&asked);
    let link = ArtifactServer::start_answering(answers, move |requested| {
        if requested == path {
            counted.fetch_add(1, Ordering::SeqCst);
        }
    });
    (link, asked)
}

/// A changed download stops the installation with a question and records nothing; check-decision
/// shows it with its texts and token, and a resume without an answer asks the link nothing.
/// "Stop" ends the attempt; the next resume asks the link again and the question comes back
/// with a new id, which an answer to the old one cannot answer. "Install it" then completes
/// with an end-summary line, and the receipt keeps both answers.
#[cfg(windows)]
#[test]
fn a_changed_download_waits_for_the_players_answer() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.payload);
    let newer = zip_bytes(&[
        (
            "EE_Fixpack/EE_Fixpack.tp2",
            b"BEGIN ~Synthetic EE Fixpack~\nDESIGNATED 0\n",
        ),
        (
            "EE_Fixpack/new-in-2026.09.txt",
            b"added by the newer release",
        ),
    ]);
    let (link, asked) = counting_link(
        BTreeMap::from([(
            "/eefixpack.zip".to_owned(),
            vec![Answer::file(newer.clone())],
        )]),
        "/eefixpack.zip",
    );

    let (code, response) = install_answering(&fixture, &link, &[]);
    let first_id = waiting_for(code, &response, "acquire:eefixpack");
    assert_eq!(response["status"]["question"]["kind"], "changed_download");
    assert_eq!(asked.load(Ordering::SeqCst), 1);
    assert!(!fixture
        .managed
        .join(".chriz/evidence/artifacts/eefixpack.json")
        .exists());
    let install_id = response["install_id"].as_str().unwrap().to_owned();

    let decision = check_decision(&fixture, &install_id);
    assert_eq!(decision["decision_id"], first_id.as_str());
    assert_eq!(
        decision["texts"]["message"],
        "A newer version of EE Fixpack was found. The reviewed version is no longer available at this link. Install it anyway?"
    );
    assert_eq!(
        labels(&decision),
        [
            "Install it",
            "Leave it out and continue",
            "Stop the installation"
        ]
    );
    assert_eq!(decision["texts"]["focus"], "yes");
    assert_eq!(decision["texts"]["details"][0], "File: eefixpack.zip");
    assert!(decision.get("answer").is_none(), "{decision:#}");
    // The human form prints the exact command lines that answer it.
    let human = executable_command(&fixture)
        .args([
            "check-decision",
            fixture.managed.to_str().unwrap(),
            "--install-id",
            &install_id,
        ])
        .output()
        .unwrap();
    let printed = stdout(&human);
    assert!(
        printed.contains(&format!(
            "Install it: chriz-bg-install decide \"{}\" --install-id {install_id} --decision {first_id} --token {} --answer yes",
            fixture.managed.display(),
            decision["token"].as_str().unwrap()
        )),
        "{printed}"
    );

    // Without an answer, a resume stops again with the same question and no download.
    let (code, response) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(waiting_for(code, &response, "acquire:eefixpack"), first_id);
    assert_eq!(asked.load(Ordering::SeqCst), 1);

    // A stale token is refused; the same answer twice is accepted, another one refused.
    let (code, refused) = decide_with(&fixture, &decision, &"0".repeat(64), "no");
    assert_eq!(code, Some(1));
    assert_eq!(refused["error"]["code"], "decision_stale", "{refused:#}");
    decide(&fixture, &decision, "no");
    decide(&fixture, &decision, "no");
    let (code, refused) = decide_with(
        &fixture,
        &decision,
        decision["token"].as_str().unwrap(),
        "yes",
    );
    assert_eq!(code, Some(1));
    assert_eq!(
        refused["error"]["code"], "decision_already_answered",
        "{refused:#}"
    );
    assert_eq!(check_decision(&fixture, &install_id)["answer"], "no");

    let (code, stopped) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(1), "{stopped:#}");
    assert_eq!(stopped["status"]["status"], "failed");
    assert_eq!(stopped["status"]["step_id"], "acquire:eefixpack");
    assert_eq!(
        stopped["status"]["reason"],
        "You chose to stop: a newer version of EE Fixpack was found at its link."
    );
    assert_eq!(check_decision(&fixture, &install_id), Value::Null);

    // The next attempt asks the link again; the question comes back with a new id.
    let (code, response) = resume_answering(&fixture, Some(&link), &[]);
    let second_id = waiting_for(code, &response, "acquire:eefixpack");
    assert_ne!(first_id, second_id);
    assert_eq!(asked.load(Ordering::SeqCst), 2);
    let (code, refused) = decide_with(
        &fixture,
        &decision,
        decision["token"].as_str().unwrap(),
        "yes",
    );
    assert_eq!(code, Some(1));
    assert_eq!(refused["error"]["code"], "decision_stale", "{refused:#}");

    decide(&fixture, &check_decision(&fixture, &install_id), "yes");
    let (code, finished) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(0), "{finished:#}");
    assert_eq!(finished["status"]["status"], "complete");
    let receipt = read_install_receipt(&fixture);
    let answers = receipt
        .decisions
        .iter()
        .map(|decision| {
            (
                decision.decision_id.as_str(),
                decision.answer,
                decision.source,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        answers,
        [
            (
                first_id.as_str(),
                bg_engine::decision::Answer::No,
                bg_engine::decision::AnswerSource::Player
            ),
            (
                second_id.as_str(),
                bg_engine::decision::Answer::Yes,
                bg_engine::decision::AnswerSource::Player
            ),
        ]
    );
    for game in ["bg1", "game"] {
        assert!(fixture
            .managed
            .join(game)
            .join("EE_Fixpack/new-in-2026.09.txt")
            .is_file());
    }
    assert_eq!(
        report_json(&fixture)["notes"],
        serde_json::json!([
            "You chose to install a newer download of EE Fixpack (eefixpack.zip).",
            "If something does not work as expected, please report it so the collection can be fixed."
        ])
    );
    // The diagnostics export carries both answers beside the ledger that holds the questions.
    let bundle = fixture._temp.path().join("diagnostics.zip");
    let (code, exported) = run_installer(
        &fixture,
        None,
        &[
            "--json",
            "diagnostics",
            fixture.managed.to_str().unwrap(),
            "--output",
            bundle.to_str().unwrap(),
        ],
    );
    assert_eq!(code, Some(0), "{exported:#}");
    let entries = exported["entries"].as_array().unwrap();
    for id in [&first_id, &second_id] {
        assert!(
            entries
                .iter()
                .any(|entry| entry == &format!("evidence/decisions/{id}.json")),
            "{entries:#?}"
        );
    }

    // Each receipt names exactly the answers the ledger recorded up to it: the report refuses
    // a receipt whose answers are missing, cut short or reordered, the failed receipt of the
    // "No" included.
    let attempts = fixture.managed.join(".chriz/attempts");
    let receipts = fs::read_dir(&attempts)
        .unwrap()
        .map(|entry| entry.unwrap().path().join("receipt.json"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    let failed = receipts
        .iter()
        .find(|path| {
            path.parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("terminal-")
        })
        .expect("the failed receipt of the No");
    let succeeded = receipts.iter().find(|path| *path != failed).unwrap();
    type Edit = fn(&mut Value);
    let edits: [(&PathBuf, Edit); 4] = [
        (succeeded, |receipt| {
            receipt.as_object_mut().unwrap().remove("decisions");
        }),
        (succeeded, |receipt| {
            receipt["decisions"].as_array_mut().unwrap().truncate(1);
        }),
        (succeeded, |receipt| {
            receipt["decisions"].as_array_mut().unwrap().reverse();
        }),
        (failed, |receipt| {
            receipt.as_object_mut().unwrap().remove("decisions");
        }),
    ];
    for (path, edit) in edits {
        let original = fs::read(path).unwrap();
        let mut receipt: Value = serde_json::from_slice(&original).unwrap();
        assert_eq!(
            receipt["decisions"].as_array().map(Vec::len),
            Some(if path == failed { 1 } else { 2 })
        );
        edit(&mut receipt);
        fs::write(path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        let output = executable_command(&fixture)
            .args(["--json", "report", fixture.managed.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{}", stdout(&output));
        assert!(
            stdout(&output).contains("answers to questions differ from the campaign ledger"),
            "{}",
            stdout(&output)
        );
        fs::write(path, original).unwrap();
    }
    report_json(&fixture);
}

/// A mod whose link has no file any more (410), whose newer download stays damaged, or whose
/// newer download would replace another mod's file is asked about with the mods that depend on
/// it. "Continue without them" leaves them out; "Stop" stops.
#[cfg(windows)]
#[test]
fn an_unusable_download_asks_with_the_mods_that_depend_on_it() {
    let fixpack_tp2: &[u8] = b"BEGIN ~Synthetic EE Fixpack~\nDESIGNATED 0\n";
    let data: Vec<u8> = (0..4096_u32)
        .map(|index| (index.wrapping_mul(2_654_435_761) >> 11) as u8)
        .collect();
    let mut damaged = zip_bytes(&[
        (
            "Snowytoes/setup-snowytoes.tp2",
            b"BEGIN ~Snowytoes 1.7~\nDESIGNATED 0\n",
        ),
        ("Snowytoes/data.bin", &data),
    ]);
    let data_start = damaged
        .windows(data.len())
        .position(|window| window == data.as_slice())
        .unwrap();
    damaged[data_start + 100] ^= 0x55;
    let cases = [
        (
            "gone",
            Answer::status("410 Gone"),
            "its link no longer has the file",
            "acquire:snowytoes",
        ),
        (
            "damaged",
            Answer::file(damaged.clone()),
            "the newer download is damaged",
            "acquire:snowytoes",
        ),
        (
            "collision",
            Answer::file(zip_bytes(&[
                (
                    "Snowytoes/setup-snowytoes.tp2",
                    b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
                ),
                ("EE_Fixpack/EE_Fixpack.tp2", b"BEGIN ~Its own edit~\n"),
            ])),
            "the newer download would replace files of another mod",
            "materialize:snowytoes:bg2",
        ),
    ];
    for (case, served, short, step) in cases {
        for answer in ["yes", "no"] {
            let fixture = executable_fixture();
            if case == "collision" {
                let reviewed = zip_bytes(&[
                    (
                        "Snowytoes/setup-snowytoes.tp2",
                        b"BEGIN ~Snowytoes~\nDESIGNATED 0\n",
                    ),
                    ("EE_Fixpack/EE_Fixpack.tp2", fixpack_tp2),
                ]);
                add_snowytoes_publishing(&fixture, &reviewed, &["Snowytoes", "EE_Fixpack"]);
            } else {
                add_snowytoes(&fixture, &snowytoes_release("1.6"));
            }
            add_snowfall(&fixture, &snowfall_release());
            let link = ArtifactServer::start_answering(
                BTreeMap::from([
                    ("/snowytoes.zip".to_owned(), vec![served.clone()]),
                    (
                        "/snowfall.zip".to_owned(),
                        vec![Answer::file(snowfall_release())],
                    ),
                ]),
                |_| {},
            );
            // The changed download in the collision case passes its checks; it is installed,
            // and only placing it finds the other mod's file.
            let presets: &[&str] = if case == "collision" {
                &[INSTALL_CHANGED]
            } else {
                &[]
            };

            let (code, response) = install_answering(&fixture, &link, presets);

            waiting_for(code, &response, step);
            let install_id = response["install_id"].as_str().unwrap().to_owned();
            let decision = check_decision(&fixture, &install_id);
            assert_eq!(decision["question"]["kind"], "unusable_download", "{case}");
            assert_eq!(
                decision["texts"]["message"],
                format!(
                    "Snowytoes can't be installed ({short}). It and the mods that depend on it (Snowfall) will be left out. Continue the installation?"
                ),
                "{case}"
            );
            assert_eq!(
                labels(&decision),
                ["Continue without them", "Stop the installation"],
                "{case}"
            );
            assert_eq!(
                decision["question"]["leave_out"]["removed_runs"],
                serde_json::json!(["snowytoes-bg2", "snowfall-bg2"]),
                "{case}"
            );
            decide(&fixture, &decision, answer);
            let (code, response) = resume_answering(&fixture, Some(&link), &[]);
            if answer == "no" {
                assert_eq!(code, Some(1), "{case}: {response:#}");
                assert_eq!(response["status"]["step_id"], step, "{case}");
                assert_eq!(
                    response["status"]["reason"],
                    format!("You chose to stop: Snowytoes can't be installed ({short})."),
                    "{case}"
                );
                assert!(!fixture.managed.join(".chriz/install-receipt.json").exists());
                let report = report_json(&fixture);
                // The collision case also holds the preset answer that installed the newer
                // download before placing it found the other mod's file.
                let decisions = report["receipt"]["decisions"].as_array().unwrap();
                assert_eq!(
                    decisions.len(),
                    if case == "collision" { 2 } else { 1 },
                    "{case}"
                );
                let last = decisions.last().unwrap();
                assert_eq!(last["answer"], "no", "{case}");
                assert_eq!(last["outcome"], "applied", "{case}");
                // Only the newer download that the collision case installed took effect.
                let expected = if case == "collision" {
                    serde_json::json!([
                        "You chose to install a newer download of Snowytoes (snowytoes.zip).",
                        "If something does not work as expected, please report it so the collection can be fixed."
                    ])
                } else {
                    serde_json::json!([])
                };
                assert_eq!(report["notes"], expected, "{case}");
                continue;
            }
            assert_eq!(code, Some(0), "{case}: {response:#}");
            let receipt = read_install_receipt(&fixture);
            assert_eq!(receipt.left_out.len(), 1, "{case}");
            assert_eq!(
                receipt.left_out[0].runs,
                ["snowytoes-bg2", "snowfall-bg2"],
                "{case}"
            );
            assert_eq!(
                receipt.decisions.len(),
                if case == "collision" { 2 } else { 1 },
                "{case}"
            );
            assert_eq!(
                receipt.decisions.last().unwrap().removed_runs,
                ["snowytoes-bg2", "snowfall-bg2"],
                "{case}"
            );
            assert!(!fixture.managed.join("game/Snowfall").exists(), "{case}");
            assert_eq!(
                report_json(&fixture)["notes"][0],
                format!(
                    "You chose to continue without Snowytoes ({short}). Also left out: Snowfall."
                ),
                "{case}"
            );
        }
    }
}

/// A newer installer tool came from a host the recipe did not review, and the download folder
/// holds no reviewed one: its file is not kept and the player is asked, with the focus on "No".
/// "Install it" downloads it again and accepts only the same file, also after a crash and a
/// lost download folder; a link that serves a third file ends that attempt, the next one asks
/// anew, and "Install it" then installs the third file. No refused file ever runs.
#[cfg(windows)]
#[test]
fn a_tool_from_an_unreviewed_host_is_asked_about_and_downloaded_again() {
    let reviewed_tool = fs::read(MOCK_CHILD).expect("read the synthetic WeiDU");
    let program = |suffix: &[u8]| {
        let mut bytes = reviewed_tool.clone();
        bytes.extend_from_slice(suffix);
        bytes
    };
    let newer_program = program(b"\0a newer build from elsewhere");
    let third_program = program(b"\0yet another build");
    let newer = zip_bytes(&[("weidu.exe", &newer_program)]);
    let third = zip_bytes(&[("weidu.exe", &third_program)]);
    let ran = |log: &Path, program: &[u8]| {
        fs::read_to_string(log)
            .unwrap_or_default()
            .lines()
            .any(|line| line == sha256_bytes(program))
    };

    // The same file again: accepted, also after a crash once it was recorded and a lost
    // download folder.
    let fixture = executable_fixture();
    let log = fixture._temp.path().join("runs.log");
    remove_cached_download(&fixture, &fixture.tool_archive);
    let link = ArtifactServer::start_answering(
        BTreeMap::from([
            (
                "/weidu.zip".to_owned(),
                vec![Answer::elsewhere("/moved/weidu.zip")],
            ),
            (
                "/moved/weidu.zip".to_owned(),
                vec![Answer::file(newer.clone())],
            ),
        ]),
        |_| {},
    );
    let logged = [("CHRIZ_TEST_MOCK_RUN_LOG", log.as_os_str())];
    let install = answering(
        install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ),
        &[],
    );
    let (code, response) = run_installer_with(&fixture, Some(&link), &logged, &install);
    waiting_for(code, &response, "acquire:weidu");
    // Refused before it was unpacked: it never ran.
    assert!(!ran(&log, &newer_program));
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    let decision = check_decision(&fixture, &install_id);
    assert_eq!(decision["question"]["kind"], "unreviewed_tool");
    assert_eq!(
        decision["question"]["served"]["unreviewed_hosts"],
        serde_json::json!(["localhost"])
    );
    let final_url = decision["question"]["served"]["final_url"]
        .as_str()
        .unwrap();
    assert!(
        final_url.starts_with("http://localhost:") && final_url.ends_with("/moved/weidu.zip"),
        "{final_url}"
    );
    assert_ne!(decision["question"]["served"]["link"], final_url);
    assert_eq!(decision["question"]["served"]["filename"], "weidu.zip");
    assert_eq!(
        decision["texts"]["message"],
        "A newer version of WeiDU was found. The reviewed version is no longer available at this link. Install it anyway? It came from localhost, which the collection did not review. If you're not sure, stop the installation."
    );
    assert_eq!(labels(&decision), ["Install it", "Stop the installation"]);
    assert_eq!(decision["texts"]["focus"], "no");
    assert!(!is_cached(&fixture, &newer));
    decide(&fixture, &decision, "yes");
    let resume = vec!["--json", "resume", fixture.managed.to_str().unwrap()];
    let crashed = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .env("CHRIZ_BG_COLLECTION_TEST_EXIT_AFTER_ACCEPTED", "weidu")
        .env("CHRIZ_TEST_MOCK_RUN_LOG", &log)
        .args(&resume)
        .output()
        .unwrap();
    assert_eq!(crashed.status.code(), Some(86), "{}", stdout(&crashed));
    assert!(fixture
        .managed
        .join(".chriz/evidence/artifacts/weidu.json")
        .is_file());
    remove_cached_download(&fixture, &newer);
    let (code, finished) = run_installer_with(&fixture, Some(&link), &logged, &resume);
    assert_eq!(code, Some(0), "{finished:#}");
    let receipt = read_install_receipt(&fixture);
    let archive = receipt
        .artifacts
        .iter()
        .find(|artifact| artifact.id == "weidu")
        .unwrap();
    assert_eq!(archive.sha256, sha256_bytes(&newer));
    assert_eq!(archive.final_url, final_url);
    assert_eq!(
        archive.served_difference.as_ref().unwrap().unreviewed_hosts,
        ["localhost"]
    );
    assert!(ran(&log, &newer_program));
    assert_eq!(
        report_json(&fixture)["notes"],
        serde_json::json!([
            "You chose to use a newer installer tool WeiDU (WeiDU 24900), which came from localhost.",
            "If something does not work as expected, please report it so the collection can be fixed."
        ])
    );

    // A third file when downloading again: the attempt ends, the next one asks anew, and
    // "Install it" installs the third file.
    let fixture = executable_fixture();
    let log = fixture._temp.path().join("runs.log");
    let logged = [("CHRIZ_TEST_MOCK_RUN_LOG", log.as_os_str())];
    remove_cached_download(&fixture, &fixture.tool_archive);
    let link = ArtifactServer::start_answering(
        BTreeMap::from([
            (
                "/weidu.zip".to_owned(),
                vec![Answer::elsewhere("/moved/weidu.zip")],
            ),
            (
                "/moved/weidu.zip".to_owned(),
                vec![Answer::file(newer.clone()), Answer::file(third.clone())],
            ),
        ]),
        |_| {},
    );
    let install = answering(
        install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ),
        &[],
    );
    let resume = vec!["--json", "resume", fixture.managed.to_str().unwrap()];
    let (code, response) = run_installer_with(&fixture, Some(&link), &logged, &install);
    let first_id = waiting_for(code, &response, "acquire:weidu");
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    decide(&fixture, &check_decision(&fixture, &install_id), "yes");
    let (code, stopped) = run_installer_with(&fixture, Some(&link), &logged, &resume);
    assert_eq!(code, Some(1), "{stopped:#}");
    assert_eq!(stopped["status"]["step_id"], "acquire:weidu");
    assert!(
        stopped["status"]["reason"]
            .as_str()
            .unwrap()
            .contains("its link changed again"),
        "{stopped:#}"
    );
    assert!(!is_cached(&fixture, &third));
    let (code, response) = run_installer_with(&fixture, Some(&link), &logged, &resume);
    let second_id = waiting_for(code, &response, "acquire:weidu");
    assert_ne!(first_id, second_id);
    assert_eq!(
        response["status"]["question"]["served"]["sha256"],
        sha256_bytes(&third)
    );
    // Neither refused file has run.
    assert!(!ran(&log, &newer_program));
    assert!(!ran(&log, &third_program));
    decide(&fixture, &check_decision(&fixture, &install_id), "yes");
    let (code, finished) = run_installer_with(&fixture, Some(&link), &logged, &resume);
    assert_eq!(code, Some(0), "{finished:#}");
    assert!(ran(&log, &third_program));
    assert!(!ran(&log, &newer_program));
    let receipt = read_install_receipt(&fixture);
    let outcomes = receipt
        .decisions
        .iter()
        .map(|decision| (decision.decision_id.as_str(), decision.outcome))
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes,
        [
            (
                first_id.as_str(),
                bg_engine::decision::DecisionOutcome::Failed
            ),
            (
                second_id.as_str(),
                bg_engine::decision::DecisionOutcome::Applied
            ),
        ]
    );
    assert_eq!(
        report_json(&fixture)["notes"],
        serde_json::json!([
            "You chose to use a newer installer tool WeiDU (WeiDU 24900), which came from localhost.",
            "If something does not work as expected, please report it so the collection can be fixed."
        ])
    );
}

/// An answer file that someone wrote with an answer its question does not offer is not
/// applied: the question is withdrawn, the attempt ends, and the next one asks anew.
#[cfg(windows)]
#[test]
fn a_stored_answer_the_question_does_not_offer_is_refused() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.tool_archive);
    let reviewed_tool = fs::read(MOCK_CHILD).expect("read the synthetic WeiDU");
    let mut program = reviewed_tool.clone();
    program.extend_from_slice(b"\0a newer build from elsewhere");
    let link = ArtifactServer::start_answering(
        BTreeMap::from([
            (
                "/weidu.zip".to_owned(),
                vec![Answer::elsewhere("/moved/weidu.zip")],
            ),
            (
                "/moved/weidu.zip".to_owned(),
                vec![Answer::file(zip_bytes(&[("weidu.exe", &program)]))],
            ),
        ]),
        |_| {},
    );
    let (code, response) = install_answering(&fixture, &link, &[]);
    let first_id = waiting_for(code, &response, "acquire:weidu");
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    let decision = check_decision(&fixture, &install_id);
    let decisions = fixture.managed.join(".chriz/evidence/decisions");
    fs::create_dir_all(&decisions).unwrap();
    fs::write(
        decisions.join(format!("{first_id}.json")),
        serde_json::to_vec_pretty(&serde_json::json!({
            "decision_id": first_id,
            "token": decision["token"],
            "answer": "leave_out",
            "answered_at": 1,
        }))
        .unwrap(),
    )
    .unwrap();

    let (code, refused) = run_installer(
        &fixture,
        None,
        &[
            "--json",
            "check-decision",
            fixture.managed.to_str().unwrap(),
            "--install-id",
            &install_id,
        ],
    );
    assert_eq!(code, Some(1));
    assert_eq!(
        refused["error"]["code"], "decision_needs_attention",
        "{refused:#}"
    );
    let (code, stopped) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(1), "{stopped:#}");
    assert!(
        stopped["status"]["reason"]
            .as_str()
            .unwrap()
            .contains("which the question does not offer"),
        "{stopped:#}"
    );
    let (code, response) = resume_answering(&fixture, Some(&link), &[]);
    let second_id = waiting_for(code, &response, "acquire:weidu");
    assert_ne!(first_id, second_id);
    decide(&fixture, &check_decision(&fixture, &install_id), "yes");
    let (code, finished) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(0), "{finished:#}");
}

/// Answers are read and written only inside the installation: when `.chriz/evidence` is a
/// junction to a folder elsewhere, an answer there is never used, `decide` writes nothing, and
/// the diagnostics export refuses to follow it.
#[cfg(windows)]
#[test]
fn answers_are_never_read_or_written_through_a_redirected_evidence_folder() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.payload);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/eefixpack.zip".to_owned(),
        zip_bytes(&[(
            "EE_Fixpack/EE_Fixpack.tp2",
            b"BEGIN ~A newer release~\nDESIGNATED 0\n",
        )]),
    )]));
    let (code, response) = install_answering(&fixture, &link, &[]);
    let decision_id = waiting_for(code, &response, "acquire:eefixpack");
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    let decision = check_decision(&fixture, &install_id);

    // The evidence moves elsewhere, and a junction takes its place, with an answer there.
    let evidence = fixture.managed.join(".chriz/evidence");
    let elsewhere = fixture._temp.path().join("elsewhere-evidence");
    fs::rename(&evidence, &elsewhere).unwrap();
    let linked = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&evidence)
        .arg(&elsewhere)
        .output()
        .unwrap();
    assert!(linked.status.success(), "{}", stderr(&linked));
    fs::create_dir_all(elsewhere.join("decisions")).unwrap();
    fs::write(
        elsewhere.join(format!("decisions/{decision_id}.json")),
        serde_json::to_vec_pretty(&serde_json::json!({
            "decision_id": decision_id,
            "token": decision["token"],
            "answer": "yes",
            "answered_at": 1,
        }))
        .unwrap(),
    )
    .unwrap();

    let (code, refused) = run_installer(
        &fixture,
        None,
        &[
            "--json",
            "check-decision",
            fixture.managed.to_str().unwrap(),
            "--install-id",
            &install_id,
        ],
    );
    assert_eq!(code, Some(1), "{refused:#}");
    let (code, refused) = decide_with(
        &fixture,
        &decision,
        decision["token"].as_str().unwrap(),
        "no",
    );
    assert_eq!(code, Some(1), "{refused:#}");
    assert_eq!(
        refused["error"]["code"], "decision_needs_attention",
        "{refused:#}"
    );
    assert_eq!(
        fs::read_dir(elsewhere.join("decisions")).unwrap().count(),
        1,
        "decide wrote through the junction"
    );
    let (code, stopped) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(1), "{stopped:#}");
    // The answer elsewhere is never used: the question is withdrawn and the installation stops.
    assert!(
        stopped
            .to_string()
            .contains("is not a direct folder of this installation"),
        "{stopped:#}"
    );
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    assert!(!replay.records.iter().any(|record| matches!(
        record.event,
        bg_engine::session::SessionEvent::DecisionAnswered { .. }
    )));
    assert!(!replay.records.iter().any(|record| matches!(
        &record.event,
        bg_engine::session::SessionEvent::StepCompleted { step_id, .. }
            if step_id == "acquire:eefixpack"
    )));
}

/// The diagnostics export never follows a junction in place of the evidence folder or of its
/// answers folder.
#[cfg(windows)]
fn assert_diagnostics_refuse_a_junction_at(fixture: &ExecutableFixture, folder: &Path) {
    let elsewhere = fixture._temp.path().join("moved-away");
    fs::rename(folder, &elsewhere).unwrap();
    let linked = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(folder)
        .arg(&elsewhere)
        .output()
        .unwrap();
    assert!(linked.status.success(), "{}", stderr(&linked));
    let bundle = fixture._temp.path().join("diagnostics.zip");
    let (code, exported) = run_installer(
        fixture,
        None,
        &[
            "--json",
            "diagnostics",
            fixture.managed.to_str().unwrap(),
            "--output",
            bundle.to_str().unwrap(),
        ],
    );
    assert_eq!(code, Some(1), "{exported:#}");
    assert!(
        exported
            .to_string()
            .contains("forbidden in diagnostics evidence"),
        "{exported:#}"
    );
    assert!(!bundle.exists());
    fs::remove_dir(folder).unwrap();
    fs::rename(&elsewhere, folder).unwrap();
}

/// check-decision, then "Leave it out", then resume: the reviewed archive came back to the
/// download folder meanwhile, so the question no longer applies and the mod is installed.
#[cfg(windows)]
#[test]
fn leave_it_out_installs_the_mod_when_its_reviewed_file_came_back() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.payload);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/eefixpack.zip".to_owned(),
        zip_bytes(&[
            (
                "EE_Fixpack/EE_Fixpack.tp2",
                b"BEGIN ~A newer release~\nDESIGNATED 0\n",
            ),
            ("EE_Fixpack/newer.txt", b"new in this release"),
        ]),
    )]));
    let (code, response) = install_answering(&fixture, &link, &[]);
    waiting_for(code, &response, "acquire:eefixpack");
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    let decision = check_decision(&fixture, &install_id);
    assert!(labels(&decision).contains(&"Leave it out and continue".to_owned()));

    // Another installation downloads the reviewed archive meanwhile.
    let (sink, _events) = ChannelSink::unbounded();
    ArtifactCache::open(&fixture.cache)
        .unwrap()
        .acquire(
            &DownloadRequest {
                request_id: "another-installation".to_owned(),
                url: format!("{}/eefixpack.zip", fixture._server.base_url),
                expected_length: fixture.payload.len() as u64,
                expected_sha256: sha256_bytes(&fixture.payload),
                redirect_hosts: Vec::new(),
                max_attempts: 1,
            },
            &sink,
        )
        .unwrap();
    decide(&fixture, &decision, "leave-out");
    let (code, finished) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(0), "{finished:#}");
    let receipt = read_install_receipt(&fixture);
    assert!(receipt.left_out.is_empty());
    let recorded = receipt
        .artifacts
        .iter()
        .find(|artifact| artifact.id == "eefixpack")
        .unwrap();
    assert_eq!(recorded.sha256, sha256_bytes(&fixture.payload));
    assert_eq!(recorded.served_difference, None);
    assert_eq!(
        receipt.decisions[0].outcome,
        bg_engine::decision::DecisionOutcome::Withdrawn
    );
    assert!(!fixture.managed.join("game/EE_Fixpack/newer.txt").exists());
    assert!(fixture
        .managed
        .join("game/EE_Fixpack/EE_Fixpack.tp2")
        .is_file());
    assert_eq!(report_json(&fixture)["notes"], serde_json::json!([]));

    // The diagnostics export refuses a junction at either folder on the way to the answers.
    let evidence = fixture.managed.join(".chriz/evidence");
    assert!(evidence.join("decisions").is_dir());
    assert_diagnostics_refuse_a_junction_at(&fixture, &evidence.join("decisions"));
    assert_diagnostics_refuse_a_junction_at(&fixture, &evidence);
    let bundle = fixture._temp.path().join("diagnostics.zip");
    let (code, exported) = run_installer(
        &fixture,
        None,
        &[
            "--json",
            "diagnostics",
            fixture.managed.to_str().unwrap(),
            "--output",
            bundle.to_str().unwrap(),
        ],
    );
    assert_eq!(code, Some(0), "{exported:#}");
}

/// A part the collection always installs never becomes a question, even with a preset answer:
/// the installation stops and says why.
#[cfg(windows)]
#[test]
fn an_unusable_fixed_part_stops_without_a_question() {
    let fixture = executable_fixture();
    add_snowytoes(&fixture, &snowytoes_release("1.6"));
    make_snowytoes_fixed(&fixture);
    let link = ArtifactServer::start_answering(
        BTreeMap::from([(
            "/snowytoes.zip".to_owned(),
            vec![Answer::status("410 Gone")],
        )]),
        |_| {},
    );

    let (code, response) = install_answering(&fixture, &link, &[CONTINUE_WITHOUT]);

    assert_eq!(code, Some(1), "{response:#}");
    assert_eq!(response["status"]["status"], "failed");
    assert_eq!(response["status"]["step_id"], "acquire:snowytoes");
    let reason = response["status"]["reason"].as_str().unwrap();
    assert!(
        reason.starts_with(
            "Snowytoes can't be installed (its link no longer has the file), and the collection can't be installed without it. Try again later; if it keeps happening, please report it.\n"
        ),
        "{reason}"
    );
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    assert!(!replay.records.iter().any(|record| matches!(
        record.event,
        bg_engine::session::SessionEvent::DecisionRequired { .. }
    )));
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    assert_eq!(check_decision(&fixture, &install_id), Value::Null);
}

/// "Leave it out and continue" is offered for a changed download of a mod the recipe can do
/// without, and leaves it out with the mods that need it. A mod the collection always installs
/// is only offered "Install it" and "Stop": a preset "leave it out" leaves the question waiting,
/// and the answer is refused.
#[cfg(windows)]
#[test]
fn leaving_out_is_offered_only_for_a_changed_mod_the_collection_can_do_without() {
    let changed = snowytoes_release("1.7");

    let fixture = executable_fixture();
    add_snowytoes(&fixture, &snowytoes_release("1.6"));
    add_snowfall(&fixture, &snowfall_release());
    let link = ArtifactServer::start(BTreeMap::from([
        ("/snowytoes.zip".to_owned(), changed.clone()),
        ("/snowfall.zip".to_owned(), snowfall_release()),
    ]));
    let (code, response) = install_answering(&fixture, &link, &["changed-download=leave-out"]);
    assert_eq!(code, Some(0), "{response:#}");
    assert_eq!(response["status"]["status"], "complete");
    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.left_out.len(), 1);
    assert_eq!(receipt.left_out[0].artifact_id, "snowytoes");
    assert_eq!(receipt.left_out[0].runs, ["snowytoes-bg2", "snowfall-bg2"]);
    assert_eq!(
        receipt.decisions[0].answer,
        bg_engine::decision::Answer::LeaveOut
    );
    assert_eq!(
        report_json(&fixture)["notes"][0],
        "You chose to continue without Snowytoes (a newer download than the reviewed one was found: snowytoes.zip). Also left out: Snowfall."
    );
    assert!(!fixture.managed.join("game/Snowytoes").exists());

    let fixture = executable_fixture();
    add_snowytoes(&fixture, &snowytoes_release("1.6"));
    make_snowytoes_fixed(&fixture);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/snowytoes.zip".to_owned(),
        changed.clone(),
    )]));
    let (code, response) = install_answering(&fixture, &link, &["changed-download=leave-out"]);
    waiting_for(code, &response, "acquire:snowytoes");
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    let decision = check_decision(&fixture, &install_id);
    assert_eq!(labels(&decision), ["Install it", "Stop the installation"]);
    assert!(decision["question"].get("leave_out").is_none());
    let (code, refused) = decide_with(
        &fixture,
        &decision,
        decision["token"].as_str().unwrap(),
        "leave-out",
    );
    assert_eq!(code, Some(1));
    assert_eq!(
        refused["error"]["code"], "decision_answer_invalid",
        "{refused:#}"
    );
}

/// An answer is written complete or not at all: an interrupted `decide` leaves only a temporary
/// file, which counts as no answer, and the question still waits until it is answered again.
#[cfg(windows)]
#[test]
fn an_interrupted_answer_leaves_the_question_waiting() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.payload);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/eefixpack.zip".to_owned(),
        zip_bytes(&[(
            "EE_Fixpack/EE_Fixpack.tp2",
            b"BEGIN ~A newer release~\nDESIGNATED 0\n",
        )]),
    )]));
    let (code, response) = install_answering(&fixture, &link, &[]);
    let decision_id = waiting_for(code, &response, "acquire:eefixpack");
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    let decision = check_decision(&fixture, &install_id);

    let interrupted = executable_command(&fixture)
        .env("CHRIZ_BG_COLLECTION_TEST_INTERRUPT_RECORD", "decisions")
        .args([
            "--json",
            "decide",
            fixture.managed.to_str().unwrap(),
            "--install-id",
            &install_id,
            "--decision",
            &decision_id,
            "--token",
            decision["token"].as_str().unwrap(),
            "--answer",
            "yes",
        ])
        .output()
        .unwrap();
    assert_eq!(
        interrupted.status.code(),
        Some(1),
        "{}",
        stdout(&interrupted)
    );
    let left_behind = files(&fixture.managed.join(".chriz/evidence/decisions"));
    assert_eq!(left_behind.len(), 1, "{left_behind:?}");
    assert_ne!(
        left_behind[0]
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("json")
    );
    assert!(check_decision(&fixture, &install_id)
        .get("answer")
        .is_none());
    let (code, response) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(
        waiting_for(code, &response, "acquire:eefixpack"),
        decision_id
    );

    decide(&fixture, &decision, "yes");
    let (code, finished) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(0), "{finished:#}");
    assert_eq!(finished["status"]["status"], "complete");
}

/// Writes a one-shot script that the engine runs right before it opens an evidence file, in
/// test builds: it moves `.chriz/evidence` elsewhere and puts a junction in its place, after
/// every path check passed, as another program could.
#[cfg(windows)]
fn swap_evidence_before_use(fixture: &ExecutableFixture) -> (PathBuf, PathBuf) {
    // cmd neither runs scripts at nor moves folders by `\\?\` paths.
    let plain = |path: &Path| {
        PathBuf::from(
            path.to_string_lossy()
                .trim_start_matches(r"\\?\")
                .to_owned(),
        )
    };
    let evidence = plain(&fixture.managed.join(".chriz").join("evidence"));
    let elsewhere = plain(&fixture._temp.path().join("swapped-evidence"));
    let script = plain(&fixture._temp.path().join("swap-evidence.cmd"));
    fs::write(
        &script,
        format!(
            "@move \"{}\" \"{}\" >nul\r\n@mklink /J \"{}\" \"{}\" >nul\r\n",
            evidence.display(),
            elsewhere.display(),
            evidence.display(),
            elsewhere.display()
        ),
    )
    .unwrap();
    (script, elsewhere)
}

/// Puts the evidence folder back after [`swap_evidence_before_use`] ran.
#[cfg(windows)]
fn undo_evidence_swap(fixture: &ExecutableFixture, elsewhere: &Path) {
    let evidence = fixture.managed.join(".chriz").join("evidence");
    fs::remove_dir(&evidence).unwrap();
    fs::rename(elsewhere, &evidence).unwrap();
}

const BEFORE_EVIDENCE_USE: &str = "CHRIZ_BG_COLLECTION_TEST_BEFORE_EVIDENCE_USE";

/// The evidence folder is replaced by a junction after every path check, right before an
/// evidence file is opened. Answers are then neither written nor read through it, and the
/// diagnostics export does not follow it: each checks the handle it actually uses.
#[cfg(windows)]
#[test]
fn a_folder_replaced_after_the_checks_never_leads_answers_or_diagnostics_elsewhere() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.payload);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/eefixpack.zip".to_owned(),
        zip_bytes(&[(
            "EE_Fixpack/EE_Fixpack.tp2",
            b"BEGIN ~A newer release~\nDESIGNATED 0\n",
        )]),
    )]));
    let (code, response) = install_answering(&fixture, &link, &[]);
    let decision_id = waiting_for(code, &response, "acquire:eefixpack");
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    let decision = check_decision(&fixture, &install_id);
    let decide_args = |answer: &'static str| {
        vec![
            "--json".to_owned(),
            "decide".to_owned(),
            fixture.managed.to_str().unwrap().to_owned(),
            "--install-id".to_owned(),
            install_id.clone(),
            "--decision".to_owned(),
            decision_id.clone(),
            "--token".to_owned(),
            decision["token"].as_str().unwrap().to_owned(),
            "--answer".to_owned(),
            answer.to_owned(),
        ]
    };

    // Publishing: nothing is written through the junction.
    let (script, elsewhere) = swap_evidence_before_use(&fixture);
    let decided = executable_command(&fixture)
        .env(BEFORE_EVIDENCE_USE, &script)
        .args(decide_args("yes"))
        .output()
        .unwrap();
    assert!(
        elsewhere.is_dir(),
        "the swap did not run: {}",
        script.display()
    );
    assert_eq!(
        decided.status.code(),
        Some(1),
        "{} {:?}",
        stdout(&decided),
        files(&elsewhere.join("decisions"))
    );
    assert!(
        !elsewhere
            .join(format!("decisions/{decision_id}.json"))
            .exists(),
        "the answer was written through the junction"
    );
    undo_evidence_swap(&fixture, &elsewhere);

    // Reading: an answer reached only through the junction is never applied.
    decide(&fixture, &decision, "yes");
    let (script, elsewhere) = swap_evidence_before_use(&fixture);
    let resumed = executable_command(&fixture)
        .env(BEFORE_EVIDENCE_USE, &script)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .args(["--json", "resume", fixture.managed.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(resumed.status.code(), Some(1), "{}", stdout(&resumed));
    assert!(elsewhere.is_dir(), "the swap did not run");
    undo_evidence_swap(&fixture, &elsewhere);
    let replay = SessionStore::open(&fixture.managed)
        .unwrap()
        .replay()
        .unwrap();
    assert!(!replay.records.iter().any(|record| matches!(
        record.event,
        bg_engine::session::SessionEvent::DecisionAnswered { .. }
    )));

    // Diagnostics: a terminal receipt with an answer, then the swap while exporting.
    let (code, response) = resume_answering(&fixture, Some(&link), &[]);
    waiting_for(code, &response, "acquire:eefixpack");
    decide(&fixture, &check_decision(&fixture, &install_id), "no");
    let (code, stopped) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(1), "{stopped:#}");
    let (script, elsewhere) = swap_evidence_before_use(&fixture);
    let bundle = fixture._temp.path().join("diagnostics.zip");
    let exported = executable_command(&fixture)
        .env(BEFORE_EVIDENCE_USE, &script)
        .args([
            "--json",
            "diagnostics",
            fixture.managed.to_str().unwrap(),
            "--output",
            bundle.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(exported.status.code(), Some(1), "{}", stdout(&exported));
    assert!(
        stdout(&exported).contains("is not inside this installation's evidence"),
        "{}",
        stdout(&exported)
    );
    assert!(!bundle.exists());
    undo_evidence_swap(&fixture, &elsewhere);
    let (code, exported) = run_installer(
        &fixture,
        None,
        &[
            "--json",
            "diagnostics",
            fixture.managed.to_str().unwrap(),
            "--output",
            bundle.to_str().unwrap(),
        ],
    );
    assert_eq!(code, Some(0), "{exported:#}");
}

/// Artifact and tool evidence is read and written only inside the installation as well: an
/// evidence folder that is a junction to a copy elsewhere stops the resume instead of being
/// followed.
#[cfg(windows)]
#[test]
fn artifact_evidence_is_never_followed_through_a_junction() {
    let fixture = executable_fixture();
    let stopped = executable_command(&fixture)
        .env(
            "CHRIZ_BG_COLLECTION_TEST_FAIL_RECHECK",
            "materialize:materialize:eefixpack:bg1",
        )
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .output()
        .unwrap();
    assert_eq!(stopped.status.code(), Some(1), "{}", stdout(&stopped));
    let evidence = fixture.managed.join(".chriz/evidence");
    assert!(evidence.join("artifacts/eefixpack.json").is_file());
    let elsewhere = fixture._temp.path().join("elsewhere-evidence");
    fs::rename(&evidence, &elsewhere).unwrap();
    let linked = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&evidence)
        .arg(&elsewhere)
        .output()
        .unwrap();
    assert!(linked.status.success(), "{}", stderr(&linked));
    let before = files(&elsewhere.join("artifacts")).len();

    let (code, resumed) = resume_answering(&fixture, None, &[]);

    assert_eq!(code, Some(1), "{resumed:#}");
    assert!(
        resumed
            .to_string()
            .contains("is not a direct folder of this installation"),
        "{resumed:#}"
    );
    assert_eq!(files(&elsewhere.join("artifacts")).len(), before);
    assert!(!fixture.managed.join("game/EE_Fixpack").exists());
}

/// Artifact evidence is written complete or not at all: an installation interrupted while it
/// writes the record of an accepted download leaves only a temporary file, and the resume
/// records it and completes instead of stopping on a half-written record forever.
#[cfg(windows)]
#[test]
fn an_interrupted_artifact_record_leaves_the_installation_resumable() {
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.payload);
    let link = ArtifactServer::start(BTreeMap::from([(
        "/eefixpack.zip".to_owned(),
        zip_bytes(&[(
            "EE_Fixpack/EE_Fixpack.tp2",
            b"BEGIN ~A newer release~\nDESIGNATED 0\n",
        )]),
    )]));
    let interrupted = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .env("CHRIZ_BG_COLLECTION_TEST_INTERRUPT_RECORD", "artifacts")
        .args(answering(
            install_args(
                &fixture.recipe,
                &fixture.bg1,
                &fixture.bg2,
                &fixture.managed,
                &fixture.cache,
            ),
            &[INSTALL_CHANGED],
        ))
        .output()
        .unwrap();
    assert_eq!(
        interrupted.status.code(),
        Some(1),
        "{}",
        stdout(&interrupted)
    );
    let artifacts = fixture.managed.join(".chriz/evidence/artifacts");
    let left_behind = files(&artifacts);
    assert_eq!(left_behind.len(), 1, "{left_behind:?}");
    assert_ne!(
        left_behind[0]
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("json"),
        "{left_behind:?}"
    );

    let (code, finished) = resume_answering(&fixture, Some(&link), &[INSTALL_CHANGED]);

    assert_eq!(code, Some(0), "{finished:#}");
    assert!(artifacts.join("eefixpack.json").is_file());
    assert_eq!(
        read_install_receipt(&fixture).outcome,
        ReceiptOutcome::Succeeded
    );
}

/// An accepted tool's record survived a crash, the download folder was lost, the next resume
/// could not reach its link, and a later one fetched it again without asking. The answer that
/// supplied the installed tool took effect, and the end summary says so.
#[cfg(windows)]
#[test]
fn an_answer_whose_recorded_file_is_installed_after_a_failed_recovery_took_effect() {
    let reviewed_tool = fs::read(MOCK_CHILD).expect("read the synthetic WeiDU");
    let mut program = reviewed_tool.clone();
    program.extend_from_slice(b"\0a newer build from elsewhere");
    let newer = zip_bytes(&[("weidu.exe", &program)]);
    let fixture = executable_fixture();
    remove_cached_download(&fixture, &fixture.tool_archive);
    let unavailable = Answer::status("503 Service Unavailable");
    let link = ArtifactServer::start_answering(
        BTreeMap::from([
            (
                "/weidu.zip".to_owned(),
                vec![Answer::elsewhere("/moved/weidu.zip")],
            ),
            (
                "/moved/weidu.zip".to_owned(),
                vec![
                    Answer::file(newer.clone()),
                    Answer::file(newer.clone()),
                    unavailable.clone(),
                    unavailable.clone(),
                    unavailable.clone(),
                    unavailable.clone(),
                    unavailable,
                    Answer::file(newer.clone()),
                ],
            ),
        ]),
        |_| {},
    );
    let (code, response) = install_answering(&fixture, &link, &[]);
    waiting_for(code, &response, "acquire:weidu");
    let install_id = response["install_id"].as_str().unwrap().to_owned();
    decide(&fixture, &check_decision(&fixture, &install_id), "yes");
    let crashed = executable_command(&fixture)
        .env(DOWNLOAD_ORIGIN, &link.base_url)
        .env("CHRIZ_BG_COLLECTION_TEST_EXIT_AFTER_ACCEPTED", "weidu")
        .args(["--json", "resume", fixture.managed.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(crashed.status.code(), Some(86), "{}", stdout(&crashed));
    remove_cached_download(&fixture, &newer);

    let (code, failed) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(1), "{failed:#}");
    assert_eq!(failed["status"]["step_id"], "acquire:weidu");
    let (code, finished) = resume_answering(&fixture, Some(&link), &[]);
    assert_eq!(code, Some(0), "{finished:#}");

    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.decisions.len(), 1);
    assert_eq!(
        receipt.decisions[0].outcome,
        bg_engine::decision::DecisionOutcome::Applied
    );
    assert_eq!(
        report_json(&fixture)["notes"],
        serde_json::json!([
            "You chose to use a newer installer tool WeiDU (WeiDU 24900), which came from localhost.",
            "If something does not work as expected, please report it so the collection can be fixed."
        ])
    );
}

/// The records of WeiDU attempts are read and written only inside the installation: an
/// `evidence/runs` folder that is a junction to a copy elsewhere stops the resume instead of
/// being followed.
#[cfg(windows)]
#[test]
fn run_attempt_records_are_never_followed_through_a_junction() {
    let fixture = executable_fixture();
    let stopped = executable_command(&fixture)
        .env(
            "CHRIZ_BG_COLLECTION_TEST_FAIL_RECHECK",
            "invocation-build:install:eefixpack-bg2",
        )
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .output()
        .unwrap();
    assert_eq!(stopped.status.code(), Some(1), "{}", stdout(&stopped));
    let runs = fixture.managed.join(".chriz/evidence/runs");
    assert!(runs.join("eefixpack-bg1/attempt-0001.json").is_file());
    let elsewhere = fixture._temp.path().join("elsewhere-runs");
    fs::rename(&runs, &elsewhere).unwrap();
    let linked = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&runs)
        .arg(&elsewhere)
        .output()
        .unwrap();
    assert!(linked.status.success(), "{}", stderr(&linked));

    let (code, resumed) = resume_answering(&fixture, None, &[]);

    assert_eq!(code, Some(1), "{resumed:#}");
    assert!(
        resumed
            .to_string()
            .contains("is not a direct folder of this installation"),
        "{resumed:#}"
    );
    assert!(!elsewhere.join("eefixpack-bg2").exists());
    assert!(!fixture.managed.join(".chriz/install-receipt.json").exists());
}

/// A WeiDU attempt record is written complete or not at all: an installation interrupted while
/// it writes one leaves only a temporary file, and the resume completes.
#[cfg(windows)]
#[test]
fn an_interrupted_run_attempt_record_leaves_the_installation_resumable() {
    let fixture = executable_fixture();
    let interrupted = executable_command(&fixture)
        .env("CHRIZ_BG_COLLECTION_TEST_INTERRUPT_RECORD", "eefixpack-bg1")
        .args(install_args(
            &fixture.recipe,
            &fixture.bg1,
            &fixture.bg2,
            &fixture.managed,
            &fixture.cache,
        ))
        .output()
        .unwrap();
    assert_eq!(
        interrupted.status.code(),
        Some(1),
        "{}",
        stdout(&interrupted)
    );
    let records = fixture.managed.join(".chriz/evidence/runs/eefixpack-bg1");
    let left_behind = files(&records);
    assert!(
        !left_behind.is_empty()
            && left_behind.iter().all(|path| path
                .extension()
                .and_then(|extension| extension.to_str())
                != Some("json")),
        "{left_behind:?}"
    );
    // The leftover has the publisher's reserved name; only exactly that is skipped.
    let leftover = left_behind[0].file_name().unwrap().to_str().unwrap();
    assert!(
        leftover.starts_with(".cebg-record-")
            && leftover.ends_with(".partial")
            && leftover.len() == ".cebg-record-".len() + 12 + ".partial".len(),
        "{leftover}"
    );

    // Look-alikes and a folder in the reserved format are refused, not skipped.
    for (name, folder) in [
        (".tmpA1b2C3", false),
        (".cebg-record-short.partial", false),
        (".cebg-record-abcdefghijkl.partial.json", false),
        (".cebg-record-abcdefghijkl.partial", true),
    ] {
        let impostor = records.join(name);
        if folder {
            fs::create_dir(&impostor).unwrap();
        } else {
            fs::write(&impostor, b"{}").unwrap();
        }
        let (code, refused) = resume_answering(&fixture, None, &[]);
        assert_eq!(code, Some(1), "{name}: {refused:#}");
        assert!(
            refused.to_string().contains("unexpected run evidence file"),
            "{name}: {refused:#}"
        );
        if folder {
            fs::remove_dir(&impostor).unwrap();
        } else {
            fs::remove_file(&impostor).unwrap();
        }
    }

    let (code, finished) = resume_answering(&fixture, None, &[]);

    assert_eq!(code, Some(0), "{finished:#}");
    let receipt = read_install_receipt(&fixture);
    assert_eq!(receipt.outcome, ReceiptOutcome::Succeeded);
    assert!(receipt
        .runs
        .iter()
        .find(|run| run.run_id == "eefixpack-bg1")
        .is_some_and(|run| !run.attempts.is_empty()));
}
