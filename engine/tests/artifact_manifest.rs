use std::io::{Cursor, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use bg_engine::manifest::{Artifact, PeMachine, SourceKind};
use bg_engine::validate::{validate, Severity};
use bg_engine::Manifest;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;

const COMPLETE_WEIDU_ARTIFACT: &str = r#"
id = "weidu"
name = "WeiDU"
version = "249.00"
acquisition = "fetch-only"

[source]
kind = "github-release"
url = "https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-x64.zip"
reference = "v249.00"
expected_filename = "WeiDU-Windows-249-x64.zip"
expected_length = 1234
sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
redirect_hosts = ["release-assets.githubusercontent.com"]

[archive]
kind = "zip"
root_rule = "single-wrapper"
publish_roots = ["WeiDU.exe"]
tp2_paths = []

[archive.limits]
max_depth = 8
max_entries = 32
max_entry_uncompressed_bytes = 104857600
max_total_uncompressed_bytes = 209715200
max_compression_ratio = 100

[tool]
executable = "WeiDU.exe"
pe_machine = "x86-64"

[provenance]
homepage = "https://weidu.org"
license = "GPL-2.0-or-later"
url = "https://github.com/WeiDUorg/weidu/releases/tag/v249.00"
reviewed_on = "2026-09-03"
"#;

#[test]
fn parses_complete_immutable_tool_artifact_contract() {
    let artifact: Artifact = toml::from_str(COMPLETE_WEIDU_ARTIFACT).unwrap();

    assert_eq!(artifact.id, "weidu");
}

#[test]
fn one_artifact_can_publish_multiple_roots_and_tp2s() {
    let text = COMPLETE_WEIDU_ARTIFACT
        .replace(
            "publish_roots = [\"WeiDU.exe\"]",
            "publish_roots = [\"EET\", \"EET_END\"]",
        )
        .replace(
            "tp2_paths = []",
            "tp2_paths = [\"EET/EET.tp2\", \"EET_END/EET_END.tp2\"]",
        )
        .replace(
            "[tool]\nexecutable = \"WeiDU.exe\"\npe_machine = \"x86-64\"\n\n",
            "",
        );
    let artifact: Artifact = toml::from_str(&text).unwrap();

    assert_eq!(artifact.archive.publish_roots, ["EET", "EET_END"]);
    assert_eq!(
        artifact.archive.tp2_paths,
        ["EET/EET.tp2", "EET_END/EET_END.tp2"]
    );
}

fn recipe() -> Manifest {
    Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
        .unwrap()
}

#[track_caller]
fn assert_finding(manifest: &Manifest, rule: &str, severity: Severity) {
    let findings = validate(manifest);
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule == rule && finding.severity == severity),
        "expected {severity:?} finding {rule:?}, got {findings:#?}"
    );
}

#[test]
fn validates_payload_and_x64_tool_contracts() {
    let manifest = recipe();

    assert!(validate(&manifest).is_empty(), "{:#?}", validate(&manifest));
}

#[test]
fn public_contract_findings_cover_unfrozen_sources_and_unreviewed_redirects() {
    let mut manifest = recipe();
    let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
    artifact.source.sha256 = "0".repeat(64);
    artifact.source.url = "https://github.com/example/project/archive/refs/heads/main.zip".into();
    artifact.source.redirect_hosts.clear();

    assert_finding(&manifest, "unpinned-source", Severity::Warning);
    assert_finding(&manifest, "mutable-source", Severity::Warning);
    assert_finding(&manifest, "unreviewed-redirect", Severity::Warning);
}

#[test]
fn public_contract_rejects_a_moving_reference_even_when_the_url_is_pinned() {
    let mut manifest = recipe();
    manifest
        .artifacts
        .get_mut("eefixpack")
        .unwrap()
        .source
        .reference = "main".into();

    assert_finding(&manifest, "mutable-source", Severity::Warning);
}

#[test]
fn public_contract_requires_positive_immutable_references_by_source_kind() {
    let mut manifest = recipe();
    {
        let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
        artifact.source.kind = SourceKind::GithubTagArchive;
        artifact.source.url = "https://codeload.github.com/example/project/zip/develop".into();
        artifact.source.reference = "develop".into();
    }

    assert_finding(&manifest, "mutable-source", Severity::Warning);

    {
        let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
        artifact.source.url =
            "https://github.com/example/project/archive/refs/tags/v249.00.zip".into();
        artifact.source.reference = "v249.00".into();
    }
    assert!(
        validate(&manifest)
            .iter()
            .all(|finding| finding.rule != "mutable-source"),
        "version tag should be immutable: {:#?}",
        validate(&manifest)
    );

    let commit = "0123456789abcdef0123456789abcdef01234567";
    {
        let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
        artifact.source.kind = SourceKind::GithubCommitZip;
        artifact.source.url = format!("https://codeload.github.com/example/project/zip/{commit}");
        artifact.source.reference = commit.into();
    }
    assert!(
        validate(&manifest)
            .iter()
            .all(|finding| finding.rule != "mutable-source"),
        "full commit should be immutable: {:#?}",
        validate(&manifest)
    );
}

#[test]
fn archive_kind_must_agree_with_a_stable_expected_filename() {
    let mut manifest = recipe();
    manifest
        .artifacts
        .get_mut("eefixpack")
        .unwrap()
        .source
        .expected_filename = Some("ee-fixpack.iemod".into());

    assert_finding(&manifest, "artifact-contract", Severity::Warning);
}

#[test]
fn rejects_http_downgrade_and_ambiguous_tp2_roots() {
    let mut manifest = recipe();
    let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
    artifact.source.url = "http://example.invalid/ee-fixpack.zip".into();
    artifact.archive.publish_roots = vec!["EE_Fixpack".into(), "EE_Fixpack/lib".into()];
    artifact.archive.tp2_paths = vec!["EE_Fixpack/lib/setup.tp2".into()];

    assert_finding(&manifest, "sources", Severity::Error);
    assert_finding(&manifest, "archive-contract", Severity::Error);
}

#[test]
fn rejects_payload_without_tp2_and_shared_artifact_disagreement() {
    let mut manifest = recipe();
    manifest
        .artifacts
        .get_mut("eefixpack")
        .unwrap()
        .archive
        .tp2_paths
        .clear();

    assert_finding(&manifest, "archive-contract", Severity::Error);

    let mut manifest = recipe();
    manifest.mods.get_mut("eefixpack").unwrap().tp2 = "other/setup.tp2".into();

    assert_finding(&manifest, "shared-artifact", Severity::Error);
}

#[test]
fn duplicate_declarations_of_one_source_must_agree_on_immutable_bytes() {
    let mut manifest = recipe();
    let mut duplicate = manifest.artifacts["eefixpack"].clone();
    duplicate.id = "eefixpack-shadow".into();
    duplicate.source.sha256 = "a".repeat(64);
    manifest.artifacts.insert(duplicate.id.clone(), duplicate);

    assert_finding(&manifest, "shared-artifact", Severity::Error);
}

#[test]
fn same_source_url_cannot_evade_disagreement_with_changed_trust_metadata() {
    for mutation in [
        "reference",
        "version",
        "source-kind",
        "redirect-hosts",
        "provenance",
    ] {
        let mut manifest = recipe();
        let mut duplicate = manifest.artifacts["eefixpack"].clone();
        duplicate.id = format!("eefixpack-{mutation}");
        match mutation {
            "reference" => duplicate.source.reference = "v999.0".into(),
            "version" => duplicate.version = "999.0".into(),
            "source-kind" => duplicate.source.kind = SourceKind::GithubTagArchive,
            "redirect-hosts" => {
                duplicate.source.redirect_hosts = vec!["other.example.invalid".into()]
            }
            "provenance" => duplicate.provenance.reviewed_on = "2026-09-04".into(),
            _ => unreachable!(),
        }
        manifest.artifacts.insert(duplicate.id.clone(), duplicate);

        assert!(
            validate(&manifest)
                .iter()
                .any(|finding| finding.rule == "shared-artifact"),
            "mutation {mutation:?} evaded shared-source disagreement: {:#?}",
            validate(&manifest)
        );
    }
}

#[test]
fn shared_source_redirect_hosts_and_hash_use_normalized_semantics() {
    let mut manifest = recipe();
    let original = manifest.artifacts.get_mut("eefixpack").unwrap();
    original.source.redirect_hosts = vec![
        "downloads.example.invalid".into(),
        "objects.example.invalid".into(),
    ];
    let mut duplicate = original.clone();
    duplicate.id = "eefixpack-equivalent".into();
    duplicate.source.sha256.make_ascii_uppercase();
    duplicate.source.redirect_hosts = vec![
        "OBJECTS.EXAMPLE.INVALID".into(),
        "DOWNLOADS.EXAMPLE.INVALID".into(),
    ];
    manifest.artifacts.insert(duplicate.id.clone(), duplicate);

    assert!(
        validate(&manifest)
            .iter()
            .all(|finding| finding.rule != "shared-artifact"),
        "equivalent trust metadata disagreed: {:#?}",
        validate(&manifest)
    );
}

#[test]
fn public_contract_requires_complete_identity_and_x64_weidu() {
    let mut manifest = recipe();
    let artifact = manifest.artifacts.get_mut("weidu").unwrap();
    artifact.source.reference.clear();
    artifact.source.expected_filename = None;
    artifact.source.expected_length = None;
    artifact.provenance.reviewed_on.clear();
    artifact.tool.as_mut().unwrap().pe_machine = PeMachine::X86;

    assert_finding(&manifest, "artifact-contract", Severity::Warning);
    assert_finding(&manifest, "tool-architecture", Severity::Warning);
}

#[test]
fn public_contract_rejects_malformed_evidence_urls_and_impossible_review_dates() {
    let mut manifest = recipe();
    let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
    artifact.source.url = "https://".into();
    artifact.provenance.url = "https://".into();
    artifact.provenance.reviewed_on = "2026-13-40".into();

    assert_finding(&manifest, "sources", Severity::Error);
    assert_finding(&manifest, "artifact-contract", Severity::Warning);
}

#[test]
fn acquisition_policy_is_exactly_one_toml_value() {
    let missing = COMPLETE_WEIDU_ARTIFACT.replace("acquisition = \"fetch-only\"\n", "");
    assert!(toml::from_str::<Artifact>(&missing).is_err());

    let duplicate = COMPLETE_WEIDU_ARTIFACT.replacen(
        "acquisition = \"fetch-only\"",
        "acquisition = \"fetch-only\"\nacquisition = \"blocked\"",
        1,
    );
    assert!(toml::from_str::<Artifact>(&duplicate).is_err());
}

fn sample_archive(machine: Option<u16>) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    writer
        .start_file("sample-v1/sample/setup-sample.tp2", options)
        .unwrap();
    writer
        .write_all(
            b"VERSIONED ~not-version-evidence~\nBEGINNING ~not-a-component~\nVERSION ~v1.2.3~\nBEGIN ~Main component~\nDESIGNATED 0\n",
        )
        .unwrap();
    writer
        .start_file("sample-v1/sample/readme.txt", options)
        .unwrap();
    writer.write_all(b"sample payload").unwrap();
    if let Some(machine) = machine {
        writer.start_file("sample-v1/WeiDU.exe", options).unwrap();
        writer.write_all(&minimal_pe(machine)).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn minimal_pe(machine: u16) -> Vec<u8> {
    let mut bytes = vec![0_u8; 0x88];
    bytes[0..2].copy_from_slice(b"MZ");
    bytes[0x3c..0x40].copy_from_slice(&0x80_u32.to_le_bytes());
    bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
    bytes[0x84..0x86].copy_from_slice(&machine.to_le_bytes());
    bytes
}

fn serve_once(body: Vec<u8>) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept inspection request: {error}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        let mut request = [0_u8; 4096];
        let _ = stream.read(&mut request).unwrap();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(&body).unwrap();
    });
    (format!("http://{address}/sample-v1.zip"), handle)
}

fn serve_cross_host_redirect(body: Vec<u8>) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut first, _) = listener.accept().unwrap();
        let mut request = [0_u8; 4096];
        let _ = first.read(&mut request).unwrap();
        write!(
            first,
            "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:{}/sample-v1.zip\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            address.port()
        )
        .unwrap();
        drop(first);

        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match listener.accept() {
                Ok((mut second, _)) => {
                    second.set_nonblocking(false).unwrap();
                    let _ = second.read(&mut request).unwrap();
                    write!(
                        second,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .unwrap();
                    second.write_all(&body).unwrap();
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept redirected request: {error}"),
            }
        }
    });
    (
        format!("http://localhost:{}/sample-v1.zip", address.port()),
        handle,
    )
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[test]
fn inspect_downloads_only_to_quarantine_and_reports_bounded_static_evidence() {
    let archive = sample_archive(None);
    let expected_hash = sha256(&archive);
    let expected_length = archive.len() as u64;
    let (url, server) = serve_once(archive);
    let temp = tempfile::tempdir().unwrap();
    let recipe = temp.path().join("artifact.toml");
    std::fs::write(&recipe, COMPLETE_WEIDU_ARTIFACT).unwrap();
    let recipe_before = std::fs::read(&recipe).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
        .args(["artifact", "inspect", &url, "--cache-root"])
        .arg(temp.path().join("cache"))
        .args(["--expected-tp2", "sample/setup-sample.tp2"])
        .output()
        .unwrap();
    server.join().unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["final_url"], url);
    assert_eq!(report["length"], expected_length);
    assert_eq!(report["sha256"], expected_hash);
    assert_eq!(report["archive"]["kind"], "zip");
    assert_eq!(
        report["archive"]["expected_tp2_matches"][0]["matches"][0],
        "sample-v1/sample/setup-sample.tp2"
    );
    assert_eq!(report["component_menu_evidence"][0]["component"], 0);
    assert_eq!(
        report["component_menu_evidence"][0]["title"],
        "Main component"
    );
    assert_eq!(report["version_evidence"][0]["value"], "v1.2.3");
    assert_eq!(report["version_evidence"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["component_menu_evidence"].as_array().unwrap().len(),
        1
    );
    let quarantine = Path::new(report["quarantined_path"].as_str().unwrap());
    assert!(quarantine.is_file());
    assert!(quarantine
        .components()
        .any(|component| component.as_os_str() == "authoring-quarantine"));
    assert!(!temp.path().join("cache/production").exists());
    assert_eq!(std::fs::read(recipe).unwrap(), recipe_before);
}

fn payload_artifact_toml(url: &str, bytes: &[u8], sha256: &str) -> String {
    format!(
        r#"id = "sample"
name = "Sample"
version = "1.2.3"
acquisition = "fetch-only"

[source]
kind = "github-release"
url = "{url}"
reference = "v1.2.3"
expected_filename = "sample-v1.zip"
expected_length = {}
sha256 = "{sha256}"
redirect_hosts = []

[archive]
kind = "zip"
root_rule = "single-wrapper"
publish_roots = ["sample"]
tp2_paths = ["sample/setup-sample.tp2"]

[archive.limits]
max_depth = 8
max_entries = 32
max_entry_uncompressed_bytes = 1048576
max_total_uncompressed_bytes = 2097152
max_compression_ratio = 100

[provenance]
homepage = "https://example.invalid/sample"
license = "MIT"
url = "https://example.invalid/sample/v1.2.3"
reviewed_on = "2026-09-03"
"#,
        bytes.len()
    )
}

fn assert_verify_rejects_contract_before_acquisition(artifact_text: &str, expected_rule: &str) {
    let temp = tempfile::tempdir().unwrap();
    let artifact_path = temp.path().join("sample.toml");
    std::fs::write(&artifact_path, artifact_text).unwrap();
    let cache_root = temp.path().join("cache");

    let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
        .args(["artifact", "verify"])
        .arg(&artifact_path)
        .arg("--cache-root")
        .arg(&cache_root)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected_rule),
        "expected {expected_rule:?}, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !cache_root.exists(),
        "invalid contract reached acquisition: {}",
        cache_root.display()
    );
}

#[test]
fn verify_rejects_moving_url_and_reference_before_acquisition() {
    let archive = sample_archive(None);
    let artifact = payload_artifact_toml(
        "http://127.0.0.1:1/zip/develop",
        &archive,
        &sha256(&archive),
    )
    .replace("reference = \"v1.2.3\"", "reference = \"develop\"");

    assert_verify_rejects_contract_before_acquisition(&artifact, "mutable-source");
}

#[test]
fn verify_rejects_malformed_provenance_and_review_date_before_acquisition() {
    let archive = sample_archive(None);
    let artifact = payload_artifact_toml(
        "http://127.0.0.1:1/sample-v1.zip",
        &archive,
        &sha256(&archive),
    )
    .replace(
        "url = \"https://example.invalid/sample/v1.2.3\"",
        "url = \"https://\"",
    )
    .replace(
        "reviewed_on = \"2026-09-03\"",
        "reviewed_on = \"2026-13-40\"",
    );

    assert_verify_rejects_contract_before_acquisition(&artifact, "artifact-contract");
}

#[test]
fn verify_rejects_overlapping_publish_roots_before_acquisition() {
    let archive = sample_archive(None);
    let artifact = payload_artifact_toml(
        "http://127.0.0.1:1/sample-v1.zip",
        &archive,
        &sha256(&archive),
    )
    .replace(
        "publish_roots = [\"sample\"]",
        "publish_roots = [\"sample\", \"sample/lib\"]",
    );

    assert_verify_rejects_contract_before_acquisition(&artifact, "archive-contract");
}

#[test]
fn verify_rejects_non_x64_tool_contract_before_acquisition() {
    let archive = sample_archive(None);
    let artifact = payload_artifact_toml(
        "http://127.0.0.1:1/sample-v1.zip",
        &archive,
        &sha256(&archive),
    )
    .replace(
        "publish_roots = [\"sample\"]",
        "publish_roots = [\"sample\", \"WeiDU.exe\"]",
    )
    .replace(
        "[provenance]",
        "[tool]\nexecutable = \"WeiDU.exe\"\npe_machine = \"x86\"\n\n[provenance]",
    );

    assert_verify_rejects_contract_before_acquisition(&artifact, "tool-architecture");
}

#[test]
fn verify_uses_production_cache_and_proves_pinned_archive_layout() {
    let archive = sample_archive(None);
    let digest = sha256(&archive);
    let (url, server) = serve_once(archive.clone());
    let temp = tempfile::tempdir().unwrap();
    let artifact_path = temp.path().join("sample.toml");
    std::fs::write(
        &artifact_path,
        payload_artifact_toml(&url, &archive, &digest),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
        .args(["artifact", "verify"])
        .arg(&artifact_path)
        .arg("--cache-root")
        .arg(temp.path().join("cache"))
        .output()
        .unwrap();
    server.join().unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["verified"], true);
    assert_eq!(report["artifact_id"], "sample");
    assert_eq!(report["sha256"], digest);
    assert_eq!(report["archive"]["wrapper_directory"], "sample-v1");
    assert!(temp.path().join("cache/production/sha256").is_dir());
    assert!(!temp.path().join("cache/authoring-quarantine").exists());
}

#[test]
fn verify_accepts_a_stable_filename_when_the_source_route_has_no_filename() {
    let archive = sample_archive(None);
    let digest = sha256(&archive);
    let (url, server) = serve_once(archive.clone());
    let url = url.replace("/sample-v1.zip", "/refs/tags/v1.2.3");
    let temp = tempfile::tempdir().unwrap();
    let artifact_path = temp.path().join("sample.toml");
    std::fs::write(
        &artifact_path,
        payload_artifact_toml(&url, &archive, &digest),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
        .args(["artifact", "verify"])
        .arg(&artifact_path)
        .arg("--cache-root")
        .arg(temp.path().join("cache"))
        .output()
        .unwrap();
    server.join().unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn verify_fails_when_downloaded_bytes_drift_from_the_contract() {
    let archive = sample_archive(None);
    let (url, server) = serve_once(archive.clone());
    let temp = tempfile::tempdir().unwrap();
    let artifact_path = temp.path().join("sample.toml");
    std::fs::write(
        &artifact_path,
        payload_artifact_toml(&url, &archive, &"a".repeat(64)),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
        .args(["artifact", "verify"])
        .arg(&artifact_path)
        .arg("--cache-root")
        .arg(temp.path().join("cache"))
        .output()
        .unwrap();
    server.join().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SHA-256 mismatch"));
}

#[test]
fn verify_rejects_pe_machine_drift_for_tool_artifacts() {
    let archive = sample_archive(Some(0x014c));
    let digest = sha256(&archive);
    let (url, server) = serve_once(archive.clone());
    let temp = tempfile::tempdir().unwrap();
    let artifact_path = temp.path().join("weidu.toml");
    let artifact = payload_artifact_toml(&url, &archive, &digest)
        .replacen("id = \"sample\"", "id = \"weidu\"", 1)
        .replace(
            "[provenance]",
            "[tool]\nexecutable = \"WeiDU.exe\"\npe_machine = \"x86-64\"\n\n[provenance]",
        )
        .replace(
            "publish_roots = [\"sample\"]",
            "publish_roots = [\"sample\", \"WeiDU.exe\"]",
        );
    std::fs::write(&artifact_path, artifact).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
        .args(["artifact", "verify"])
        .arg(&artifact_path)
        .arg("--cache-root")
        .arg(temp.path().join("cache"))
        .output()
        .unwrap();
    server.join().unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("PE machine"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn verify_rejects_a_redirect_to_an_unreviewed_host_before_downloading_it() {
    let archive = sample_archive(None);
    let digest = sha256(&archive);
    let (url, server) = serve_cross_host_redirect(archive.clone());
    let temp = tempfile::tempdir().unwrap();
    let artifact_path = temp.path().join("sample.toml");
    std::fs::write(
        &artifact_path,
        payload_artifact_toml(&url, &archive, &digest),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
        .args(["artifact", "verify"])
        .arg(&artifact_path)
        .arg("--cache-root")
        .arg(temp.path().join("cache"))
        .output()
        .unwrap();
    server.join().unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unreviewed redirect"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
