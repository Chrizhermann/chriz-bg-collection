use std::io::Read;
use std::path::{Path, PathBuf};

use bg_engine::diagnostics::{export_diagnostics, DiagnosticsRequest};
use tempfile::TempDir;
use zip::ZipArchive;

fn write(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

fn zip_entries(path: &Path) -> Vec<(String, Vec<u8>)> {
    let file = std::fs::File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        entries.push((entry.name().to_owned(), bytes));
    }
    entries
}

struct Fixture {
    _temp: TempDir,
    managed: PathBuf,
    home: PathBuf,
    output: PathBuf,
}

fn fixture(include_success: bool) -> Fixture {
    let temp = TempDir::new().unwrap();
    let managed = temp.path().join("managed");
    let home = temp.path().join("Users/Christopher");
    let output = temp.path().join("exports/diagnostics.zip");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    let raw_home = home.to_string_lossy();
    write(
        &managed,
        ".chriz/attempts/attempt-001/receipt.json",
        &serde_json::to_vec(&serde_json::json!({
            "outcome": "failed",
            "path": raw_home.as_ref(),
            "token": "secret-token"
        }))
        .unwrap(),
    );
    write(
        &managed,
        ".chriz/ledger/0000000000.json",
        format!(r#"{{"managed_root":"{raw_home}\\Games\\RC"}}"#).as_bytes(),
    );
    write(&managed, ".chriz/recipe/payload.zip", b"PK\x03\x04recipe");
    write(
        &managed,
        ".chriz/recipe/envelope.json",
        br#"{"version":"alpha-1"}"#,
    );
    write(
        &managed,
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/process-output.log",
        format!("reading {raw_home}\\Games\\RC\nAuthorization: Bearer top-secret\nvisible line\n")
            .as_bytes(),
    );
    write(
        &managed,
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/weidu.debug.log",
        b"-----BEGIN PRIVATE KEY-----\nsecret material\n-----END PRIVATE KEY-----\nafter key\n",
    );
    write(
        &managed,
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/before.log",
        b"before snapshot\n",
    );
    write(
        &managed,
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/after.log",
        b"after snapshot\n",
    );
    write(
        &managed,
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/invocation.json",
        br#"{"identity_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
    );
    write(
        &managed,
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/prompt-results.jsonl",
        b"{\"index\":0,\"expected_output_sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"answer_sha256\":\"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"}\n",
    );
    if include_success {
        write(
            &managed,
            ".chriz/install-receipt.json",
            format!(r#"{{"outcome":"succeeded","save_root":"{raw_home}\\Saves"}}"#).as_bytes(),
        );
    }

    // These paths are deliberately tempting but outside the explicit bundle allowlist.
    write(&managed, "game/chitin.key", b"game content");
    write(
        &managed,
        ".chriz/attempts/attempt-001/archive.zip",
        b"private mod archive",
    );
    write(
        &managed,
        ".chriz/attempts/attempt-001/private-key.pem",
        b"private key material",
    );
    write(
        &managed,
        ".chriz/credentials.json",
        br#"{"password":"do-not-export"}"#,
    );
    write(&managed, ".chriz/unrelated.txt", b"outside allowlist");

    Fixture {
        _temp: temp,
        managed,
        home,
        output,
    }
}

#[test]
fn exports_eet_compatibility_hash_evidence_without_mod_payloads() {
    let fixture = fixture(false);
    let evidence = br#"{"fix_id":"eet-windows-documents-path-v1","relative_path":"EET/lib/macros.tph","observed_before_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","observed_after_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","changed":true}"#;
    write(
        &fixture.managed,
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/eet-compatibility.json",
        evidence,
    );
    write(
        &fixture.managed,
        "game/EET/lib/macros.tph",
        b"not for diagnostics",
    );
    let bundle = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: "attempt-001".to_owned(),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
    })
    .unwrap();
    let entries = zip_entries(&bundle.path);
    let exported = entries
        .iter()
        .find(|(name, _)| name.ends_with("/eet-compatibility.json"))
        .expect("the applied compatibility correction must be included in diagnostics");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&exported.1).unwrap(),
        serde_json::from_slice::<serde_json::Value>(evidence).unwrap()
    );
    assert!(!entries.iter().any(|(name, _)| name.ends_with("macros.tph")));
}

#[test]
fn summary_explains_empty_attempts_without_claiming_the_process_never_started() {
    let fixture = fixture(false);
    write(&fixture.managed, ".chriz/attempts/attempt-001/receipt.json", &serde_json::to_vec(&serde_json::json!({
        "install_id": "my-install", "attempt_id": "attempt-001", "evidence_attempt_id": "attempt-001",
        "versions": {"application": "alpha.10", "engine": "0.1.0", "recipe": "alpha.11"},
        "outcome": {"status": "failed", "step_id": "install:earlier-mod", "detail": "An earlier run failed"},
        "runs": [{"run_id": "buffbot-bg2", "target": "bg2", "components": [1, 0], "attempts": []}]
    })).unwrap());
    let bundle = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: "attempt-001".to_owned(),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
    })
    .unwrap();
    let entries = zip_entries(&bundle.path);
    assert!(bundle.entries.contains(&"START-HERE.txt".to_owned()));
    let summary = String::from_utf8(
        entries
            .iter()
            .find(|(name, _)| name == "START-HERE.txt")
            .unwrap()
            .1
            .clone(),
    )
    .unwrap();
    assert!(summary.contains("Outcome: failed"));
    assert!(summary.contains("Stopped at: install:earlier-mod"));
    assert!(summary.contains("Application: alpha.10"));
    assert!(
        summary.contains("buffbot-bg2 [bg2]: 2 planned components; no finalized attempt recorded")
    );
    assert!(summary.contains("does not prove that the process never started"));
    assert!(summary.contains("Later resumes"));
}

#[test]
fn summary_keeps_partial_log_additions_and_redacts_private_failure_details() {
    let fixture = fixture(false);
    write(&fixture.managed, ".chriz/attempts/attempt-001/receipt.json", &serde_json::to_vec(&serde_json::json!({
        "outcome": {"status": "failed", "step_id": "install:buffbot", "detail": format!("Failed under {}", fixture.home.display())},
        "runs": [{"run_id": "buffbot-bg2", "target": "bg2", "components": [1, 0], "attempts": [{
            "exit_code": 0, "log_diff": {"added": [{"component": 1}], "removed": []}
        }]}]
    })).unwrap());
    let bundle = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: "attempt-001".to_owned(),
        output_path: fixture.output,
        redact_roots: vec![fixture.home.clone()],
    })
    .unwrap();
    let entries = zip_entries(&bundle.path);
    let summary = String::from_utf8(
        entries
            .iter()
            .find(|(name, _)| name == "START-HERE.txt")
            .unwrap()
            .1
            .clone(),
    )
    .unwrap();
    assert!(
        summary.contains("1 finalized attempt(s); 1/2 log additions recorded; last exit code 0")
    );
    assert!(!summary.contains(&*fixture.home.to_string_lossy()));
    assert!(summary.contains("<redacted-home>"));
    assert!(!summary.contains("BuffBot installed successfully"));
}

#[test]
fn success_bundle_uses_an_explicit_allowlist_and_redacts_personal_or_secret_text() {
    let fixture = fixture(true);
    let result = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed.clone(),
        attempt_id: "attempt-001".to_owned(),
        output_path: fixture.output.clone(),
        redact_roots: vec![fixture.home.clone()],
    })
    .unwrap();

    let entries = zip_entries(&result.path);
    let names = entries
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"receipt/install-receipt.json"));
    assert!(names.contains(&"receipt/attempt-receipt.json"));
    assert!(names.contains(&"ledger/0000000000.json"));
    assert!(names.contains(&"recipe/payload.zip"));
    assert!(names.contains(&"recipe/envelope.json"));
    let evidence_root = "logs/steps/0001-0123456789abcdef/attempt-0001";
    assert!(names.contains(&format!("{evidence_root}/process-output.log").as_str()));
    assert!(names.contains(&format!("{evidence_root}/weidu.debug.log").as_str()));
    assert!(names.contains(&format!("{evidence_root}/before.log").as_str()));
    assert!(names.contains(&format!("{evidence_root}/after.log").as_str()));
    assert!(names.contains(&format!("{evidence_root}/invocation.json").as_str()));
    assert!(names.contains(&format!("{evidence_root}/prompt-results.jsonl").as_str()));
    assert!(!names.iter().any(|name| name.contains("archive")));
    assert!(!names.iter().any(|name| name.contains("private-key")));
    assert!(!names.iter().any(|name| name.contains("credentials")));
    assert!(!names.iter().any(|name| name.contains("game/")));
    assert!(!names.iter().any(|name| name.contains("unrelated")));

    let searchable = entries
        .iter()
        .filter(|(name, _)| !name.ends_with("payload.zip"))
        .flat_map(|(_, bytes)| bytes.iter().copied())
        .collect::<Vec<_>>();
    let searchable = String::from_utf8(searchable).unwrap();
    assert!(!searchable.contains(&*fixture.home.to_string_lossy()));
    assert!(!searchable.contains("secret-token"));
    assert!(!searchable.contains("top-secret"));
    assert!(!searchable.contains("secret material"));
    assert!(searchable.contains("<redacted-home>"));
    assert!(searchable.contains("visible line"));
    assert!(searchable.contains("after key"));
}

#[test]
fn failure_bundle_does_not_require_a_success_receipt_and_never_overwrites_output() {
    let fixture = fixture(false);
    let request = DiagnosticsRequest {
        managed_root: fixture.managed.clone(),
        attempt_id: "attempt-001".to_owned(),
        output_path: fixture.output.clone(),
        redact_roots: vec![fixture.home.clone()],
    };

    export_diagnostics(&request).unwrap();
    let before = std::fs::read(&fixture.output).unwrap();
    let error = export_diagnostics(&request).unwrap_err();

    assert!(error.to_string().contains("already exists"), "{error}");
    assert_eq!(std::fs::read(&fixture.output).unwrap(), before);
    let names = zip_entries(&fixture.output)
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    assert!(!names.contains(&"receipt/install-receipt.json".to_owned()));
    assert!(names.contains(&"receipt/attempt-receipt.json".to_owned()));
}

#[test]
fn terminal_failure_bundle_follows_its_receipted_step_evidence_attempt() {
    let fixture = fixture(false);
    let terminal_id = "terminal-0000000009-aaaaaaaaaaaaaaaa";
    write(
        &fixture.managed,
        &format!(".chriz/attempts/{terminal_id}/receipt.json"),
        br#"{"attempt_id":"terminal-0000000009-aaaaaaaaaaaaaaaa","evidence_attempt_id":"attempt-001","outcome":{"status":"failed"}}"#,
    );

    let result = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: terminal_id.to_owned(),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
    })
    .unwrap();

    let names = zip_entries(&result.path)
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    assert!(names
        .contains(&"logs/steps/0001-0123456789abcdef/attempt-0001/process-output.log".to_owned()));
}

#[test]
fn receipt_evidence_attempt_cannot_escape_the_attempts_directory() {
    let fixture = fixture(false);
    let terminal_id = "terminal-0000000009-bbbbbbbbbbbbbbbb";
    write(
        &fixture.managed,
        &format!(".chriz/attempts/{terminal_id}/receipt.json"),
        br#"{"evidence_attempt_id":".."}"#,
    );

    let error = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: terminal_id.to_owned(),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
    })
    .unwrap_err();

    assert!(error.to_string().contains("path-safe"), "{error}");
}

#[test]
fn redacts_serde_escaped_windows_paths_in_json_evidence() {
    let fixture = fixture(false);
    let windows_home = PathBuf::from(r"C:\Users\Christopher");
    write(
        &fixture.managed,
        ".chriz/attempts/attempt-001/receipt.json",
        &serde_json::to_vec_pretty(&serde_json::json!({
            "managed_save_root": r"C:\Users\Christopher\Documents\Chriz BG Collection"
        }))
        .unwrap(),
    );

    let result = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: "attempt-001".to_owned(),
        output_path: fixture.output,
        redact_roots: vec![windows_home],
    })
    .unwrap();

    let entries = zip_entries(&result.path);
    let receipt = entries
        .iter()
        .find(|(name, _)| name == "receipt/attempt-receipt.json")
        .map(|(_, bytes)| String::from_utf8(bytes.clone()).unwrap())
        .unwrap();
    assert!(!receipt.contains(r"C:\\Users\\Christopher"), "{receipt}");
    assert!(receipt.contains("<redacted-home>"), "{receipt}");
}

#[test]
fn structured_evidence_stays_parseable_while_nested_secrets_are_redacted() {
    let fixture = fixture(false);
    let receipt_path = fixture
        .managed
        .join(".chriz/attempts/attempt-001/receipt.json");
    let original = serde_json::to_vec_pretty(&serde_json::json!({
        "evidence_attempt_id": "attempt-001",
        "download": {
            "resolved_url": "https://example.invalid/archive.zip?token=invented-query-secret",
            "clientSecret": "invented-client-secret",
            "nested": [
                fixture.home.join("Games/CEBG"),
                {"Authorization": "Bearer invented-bearer-secret"}
            ]
        },
        "path_map": { fixture.home.to_string_lossy().to_string(): "private keyed entry" },
        "safe": "visible diagnostic value"
    }))
    .unwrap();
    std::fs::write(&receipt_path, &original).unwrap();
    let jsonl_path = fixture.managed.join(
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/prompt-results.jsonl",
    );
    let original_jsonl = b"{\"url\":\"https://example.invalid/prompt?token=invented-jsonl-secret\",\"safe\":\"first\"}\n{\"nested\":[{\"password\":\"invented-jsonl-password\"}]}\n";
    std::fs::write(&jsonl_path, original_jsonl).unwrap();

    let result = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: "attempt-001".to_owned(),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
    })
    .unwrap();

    assert_eq!(std::fs::read(receipt_path).unwrap(), original);
    assert_eq!(std::fs::read(jsonl_path).unwrap(), original_jsonl);
    let entries = zip_entries(&result.path);
    for (name, bytes) in &entries {
        if name.ends_with(".json") {
            serde_json::from_slice::<serde_json::Value>(bytes)
                .unwrap_or_else(|error| panic!("{name} is not valid JSON: {error}"));
        } else if name.ends_with(".jsonl") {
            for (index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
                if !line.is_empty() {
                    serde_json::from_slice::<serde_json::Value>(line).unwrap_or_else(|error| {
                        panic!("{name} line {} is not valid JSON: {error}", index + 1)
                    });
                }
            }
        }
    }
    let receipt = entries
        .iter()
        .find(|(name, _)| name == "receipt/attempt-receipt.json")
        .unwrap();
    let sanitized: serde_json::Value = serde_json::from_slice(&receipt.1).unwrap();
    let serialized = serde_json::to_string(&sanitized).unwrap();
    let searchable = entries
        .iter()
        .filter(|(name, _)| !name.ends_with("payload.zip"))
        .flat_map(|(_, bytes)| bytes.iter().copied())
        .collect::<Vec<_>>();
    let searchable = String::from_utf8(searchable).unwrap();
    for secret in [
        "invented-query-secret",
        "invented-client-secret",
        "invented-bearer-secret",
        "invented-jsonl-secret",
        "invented-jsonl-password",
    ] {
        assert!(!searchable.contains(secret), "leaked {secret}");
    }
    assert!(
        !serialized.contains("Users"),
        "leaked private path: {serialized}"
    );
    assert_eq!(sanitized["safe"], "visible diagnostic value");
    assert_eq!(sanitized["path_map"], "<redacted-sensitive-object>");
    assert_eq!(
        sanitized["download"]["clientSecret"],
        "<redacted-sensitive-value>"
    );
    assert_eq!(
        sanitized["download"]["nested"][1]["Authorization"],
        "<redacted-sensitive-value>"
    );
}

#[test]
fn exports_only_fixed_evidence_names_at_the_task13_attempt_depth() {
    let fixture = fixture(false);
    let valid_root = ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001";
    write(
        &fixture.managed,
        &format!("{valid_root}/notes.txt"),
        b"not engine evidence",
    );
    write(
        &fixture.managed,
        ".chriz/attempts/attempt-001/steps/freeform/attempt-0001/process-output.log",
        b"invalid step directory",
    );
    write(
        &fixture.managed,
        ".chriz/attempts/attempt-001/steps/0002-fedcba9876543210/attempt-current/process-output.log",
        b"invalid attempt directory",
    );
    write(
        &fixture.managed,
        &format!("{valid_root}/nested/process-output.log"),
        b"invalid extra depth",
    );

    let result = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: "attempt-001".to_owned(),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
    })
    .unwrap();
    let names = zip_entries(&result.path)
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();

    assert!(!names.iter().any(|name| name.ends_with("notes.txt")));
    assert!(!names.iter().any(|name| name.contains("/freeform/")));
    assert!(!names.iter().any(|name| name.contains("/attempt-current/")));
    assert!(!names.iter().any(|name| name.contains("/nested/")));
}

#[test]
fn redacts_oauth_aws_camelcase_cookie_and_query_credentials() {
    let fixture = fixture(false);
    let evidence = fixture.managed.join(
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/process-output.log",
    );
    std::fs::write(
        evidence,
        b"github oauth gho_abcdefghijklmnopqrstuvwxyz123456\n\
aws access key AKIAIOSFODNN7EXAMPLE\n\
{\"clientSecret\":\"camel-case-value\"}\n\
Cookie: session=browser-cookie-value\n\
GET https://example.invalid/archive?download=1&token=query-value\n\
safe diagnostic line\n",
    )
    .unwrap();

    let result = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: "attempt-001".to_owned(),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
    })
    .unwrap();
    let searchable = zip_entries(&result.path)
        .into_iter()
        .filter(|(name, _)| !name.ends_with("payload.zip"))
        .flat_map(|(_, bytes)| bytes)
        .collect::<Vec<_>>();
    let searchable = String::from_utf8(searchable).unwrap();

    for secret in [
        "gho_abcdefghijklmnopqrstuvwxyz123456",
        "AKIAIOSFODNN7EXAMPLE",
        "camel-case-value",
        "browser-cookie-value",
        "query-value",
    ] {
        assert!(
            !searchable.contains(secret),
            "leaked {secret}: {searchable}"
        );
    }
    assert!(searchable.contains("safe diagnostic line"));
}
