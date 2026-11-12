use std::io::Read;
use std::path::{Path, PathBuf};

use bg_engine::diagnostics::{
    export_diagnostics, redact_for_sharing, redact_for_sharing_with_names, DiagnosticsError,
    DiagnosticsRequest,
};
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

#[test]
fn pre_spawn_guard_evidence_is_exported_and_personal_paths_are_redacted() {
    let fixture = fixture(false);
    write(
        &fixture.managed,
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/pre-spawn-failure.json",
        &serde_json::to_vec(&serde_json::json!({
            "run_id": "eet-core", "attempt": 1, "components": [0],
            "guard": "invocation-build", "failure": fixture.home.to_string_lossy(),
            "files_sha256": { "before.log": "a".repeat(64) }
        })).unwrap(),
    );
    export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed.clone(),
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output.clone(),
        redact_roots: vec![fixture.home.clone()],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    })
    .unwrap();
    let entries = zip_entries(&fixture.output);
    let (_, bytes) = entries
        .iter()
        .find(|(name, _)| name.ends_with("/pre-spawn-failure.json"))
        .expect("guard evidence must be in the diagnostic bundle");
    let evidence: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(evidence["guard"], "invocation-build");
    assert_eq!(evidence["files_sha256"]["before.log"], "a".repeat(64));
    assert!(!String::from_utf8_lossy(bytes).contains(fixture.home.to_string_lossy().as_ref()));
}

fn fixture(include_success: bool) -> Fixture {
    let temp = TempDir::new().unwrap();
    let managed = temp.path().join("managed");
    let home = temp.path().join("Users/Player");
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
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
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
fn exports_config_file_evidence_without_the_placed_file() {
    let fixture = fixture(false);
    let evidence = br#"[{"component":3347,"path":"cdtweaks/cdtweaks.txt","sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","created":true}]"#;
    write(
        &fixture.managed,
        ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/config-files.json",
        evidence,
    );
    write(
        &fixture.managed,
        "game/cdtweaks/cdtweaks.txt",
        b"not for diagnostics",
    );
    let bundle = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: Some("attempt-001".into()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    })
    .unwrap();
    let entries = zip_entries(&bundle.path);
    let exported = entries
        .iter()
        .find(|(name, _)| name.ends_with("/config-files.json"))
        .expect("placed configuration files must be listed in diagnostics");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&exported.1).unwrap(),
        serde_json::from_slice::<serde_json::Value>(evidence).unwrap()
    );
    assert!(!entries
        .iter()
        .any(|(name, _)| name.ends_with("cdtweaks.txt")));
}

#[test]
fn exports_ui_launch_correction_evidence_without_mod_payloads() {
    let fixture = fixture(false);
    let evidence = br#"{"fix_id":"infinity-ui-1.28-setup-name-v1","original_mode":"explicit-tp2","effective_mode":"setup-name"}"#;
    write(&fixture.managed, ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/ui-invocation-compatibility.json", evidence);
    write(
        &fixture.managed,
        "game/infinity_ui/infinity_ui.tp2",
        b"not for diagnostics",
    );
    let bundle = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: Some("attempt-001".into()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    })
    .unwrap();
    let entries = zip_entries(&bundle.path);
    let exported = entries
        .iter()
        .find(|(name, _)| name.ends_with("/ui-invocation-compatibility.json"))
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&exported.1).unwrap(),
        serde_json::from_slice::<serde_json::Value>(evidence).unwrap()
    );
    assert!(!entries
        .iter()
        .any(|(name, _)| name.ends_with("infinity_ui.tp2")));
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
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
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
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home.clone()],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
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
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output.clone(),
        redact_roots: vec![fixture.home.clone()],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
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
fn failure_bundle_does_not_require_a_success_receipt_and_replaces_existing_zip() {
    let fixture = fixture(false);
    let request = DiagnosticsRequest {
        managed_root: fixture.managed.clone(),
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output.clone(),
        redact_roots: vec![fixture.home.clone()],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    };

    export_diagnostics(&request).unwrap();
    export_diagnostics(&request).unwrap();
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
        attempt_id: Some(terminal_id.to_owned()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
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
        attempt_id: Some(terminal_id.to_owned()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    })
    .unwrap_err();

    assert!(error.to_string().contains("path-safe"), "{error}");
}

#[test]
fn redacts_serde_escaped_windows_paths_in_json_evidence() {
    let fixture = fixture(false);
    let windows_home = PathBuf::from(r"C:\Users\Player");
    write(
        &fixture.managed,
        ".chriz/attempts/attempt-001/receipt.json",
        &serde_json::to_vec_pretty(&serde_json::json!({
            "managed_save_root": r"C:\Users\Player\Documents\Chriz BG Collection"
        }))
        .unwrap(),
    );

    let result = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output,
        redact_roots: vec![windows_home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    })
    .unwrap();

    let entries = zip_entries(&result.path);
    let receipt = entries
        .iter()
        .find(|(name, _)| name == "receipt/attempt-receipt.json")
        .map(|(_, bytes)| String::from_utf8(bytes.clone()).unwrap())
        .unwrap();
    assert!(!receipt.contains(r"C:\\Users\\Player"), "{receipt}");
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
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
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
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
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
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
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

#[test]
fn exports_only_named_patch_evidence_and_redacts_pending_json() {
    check_update_evidence("patches/cebg-v1");
}

#[test]
fn exports_only_named_mod_upgrade_evidence_without_backups_or_player_data() {
    check_update_evidence("mod-upgrades/v1");
}

fn check_update_evidence(namespace: &str) {
    let fixture = fixture(true);
    let transaction = format!(".chriz/{namespace}/1758326400000000000");
    let evidence = serde_json::to_vec(&serde_json::json!({
        "managed_root": fixture.home.join("Games/Test"),
        "nested": { "token": "invented-patch-secret" },
        "patch_id": "artisan-campaign-description-links-1.0"
    }))
    .unwrap();
    let mut expected_names = Vec::new();
    for stem in [
        "prepared",
        "applied",
        "restoring",
        "restored",
        "failure",
        "invocation",
    ] {
        for extension in ["json", "pending"] {
            let name = format!("{stem}.{extension}");
            write(
                &fixture.managed,
                &format!("{transaction}/{name}"),
                &evidence,
            );
            expected_names.push(format!("{namespace}/1758326400000000000/{name}"));
        }
    }
    for name in ["attempt.debug", "output.log"] {
        write(
            &fixture.managed,
            &format!("{transaction}/{name}"),
            format!(
                "reading {}\nAuthorization: Bearer invented-patch-token\nvisible patch log\n",
                fixture.home.display()
            )
            .as_bytes(),
        );
        expected_names.push(format!("{namespace}/1758326400000000000/{name}"));
    }
    for relative in [
        format!("{transaction}/affected/prepared.json"),
        format!("{transaction}/payload/invocation.json"),
        format!("{transaction}/weidu.exe"),
        format!("{transaction}/game/dialog.tlk"),
        format!("{transaction}/profile/save/BALDUR.GAM"),
        format!("{transaction}/notes.txt"),
        format!("{transaction}/unknown.json"),
        ".chriz/patches/cebg-v1/freeform/prepared.json".to_owned(),
        ".chriz/patches/cebg-v1/prepared.json".to_owned(),
        ".chriz/patches/other/123/prepared.json".to_owned(),
        format!(".chriz/patches/cebg-v1/{}/prepared.json", "1".repeat(40)),
    ] {
        write(
            &fixture.managed,
            &relative,
            b"private excluded patch material",
        );
    }
    let result = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed.clone(),
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output,
        redact_roots: vec![fixture.home.clone()],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    })
    .unwrap();
    let entries = zip_entries(&result.path);
    let patch_entries = entries
        .iter()
        .filter(|(name, _)| name.starts_with(namespace))
        .collect::<Vec<_>>();
    expected_names.sort();
    assert_eq!(
        patch_entries
            .iter()
            .map(|(name, _)| name.to_string())
            .collect::<Vec<_>>(),
        expected_names
    );
    for (name, bytes) in patch_entries {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains(fixture.home.to_string_lossy().as_ref()));
        assert!(!text.contains("invented-patch-secret"));
        assert!(!text.contains("invented-patch-token"));
        assert!(text.contains("<redacted-home>"));
        if name.ends_with(".json") || name.ends_with(".pending") {
            let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
            assert_eq!(value["nested"]["token"], "<redacted-sensitive-value>");
            assert_eq!(value["patch_id"], "artisan-campaign-description-links-1.0");
        } else {
            assert!(text.contains("visible patch log"));
        }
    }
    assert_eq!(
        std::fs::read(
            fixture
                .managed
                .join(format!("{transaction}/prepared.pending"))
        )
        .unwrap(),
        evidence,
    );
}

#[test]
fn rejects_oversized_patch_evidence_without_publishing_a_bundle() {
    let fixture = fixture(false);
    let relative = ".chriz/patches/cebg-v1/123/output.log";
    write(&fixture.managed, relative, b"");
    std::fs::OpenOptions::new()
        .write(true)
        .open(fixture.managed.join(relative))
        .unwrap()
        .set_len(64 * 1024 * 1024 + 1)
        .unwrap();
    let error = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output.clone(),
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    })
    .unwrap_err();
    assert!(error.to_string().contains("size limit"), "{error}");
    assert!(!fixture.output.exists());
}

#[test]
fn rejects_a_directory_at_a_named_patch_evidence_file() {
    let fixture = fixture(false);
    std::fs::create_dir_all(
        fixture
            .managed
            .join(".chriz/patches/cebg-v1/123/prepared.json"),
    )
    .unwrap();
    let error = export_diagnostics(&DiagnosticsRequest {
        managed_root: fixture.managed,
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output.clone(),
        redact_roots: vec![fixture.home],
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    })
    .unwrap_err();
    assert!(
        error.to_string().contains("direct non-symlink file"),
        "{error}"
    );
    assert!(!fixture.output.exists());
}

const STEP_EVIDENCE: &str = ".chriz/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001";

fn named_request(fixture: &Fixture, users: &[&str], computers: &[&str]) -> DiagnosticsRequest {
    DiagnosticsRequest {
        managed_root: fixture.managed.clone(),
        attempt_id: Some("attempt-001".to_owned()),
        output_path: fixture.output.clone(),
        redact_roots: vec![fixture.home.clone()],
        redact_user_names: users.iter().map(|name| (*name).to_owned()).collect(),
        redact_computer_names: computers.iter().map(|name| (*name).to_owned()).collect(),
        missing_receipt_reason: None,
    }
}

fn recorded_request(fixture: &Fixture, reason: Option<String>) -> DiagnosticsRequest {
    DiagnosticsRequest {
        attempt_id: None,
        missing_receipt_reason: reason,
        ..named_request(fixture, &["Max"], &[])
    }
}

fn entry_text(entries: &[(String, Vec<u8>)], name: &str) -> String {
    let (_, bytes) = entries
        .iter()
        .find(|(entry, _)| entry == name)
        .unwrap_or_else(|| panic!("missing {name}"));
    String::from_utf8(bytes.clone()).unwrap()
}

fn entry_json(entries: &[(String, Vec<u8>)], name: &str) -> serde_json::Value {
    serde_json::from_str(&entry_text(entries, name))
        .unwrap_or_else(|error| panic!("{name} is not valid JSON: {error}"))
}

#[test]
fn a_user_name_as_a_path_segment_on_another_drive_is_redacted_in_logs_and_json() {
    let fixture = fixture(false);
    write(
        &fixture.managed,
        &format!("{STEP_EVIDENCE}/process-output.log"),
        b"reading D:\\Spiele\\Max\\BG2\\weidu.log\n\
kept D:\\Maxwell\\x and D:/Max-Saves/y and D:\\Max_old\\z\n\
max played well\n\
end at D:/Games/MAX",
    );
    write(
        &fixture.managed,
        &format!("{STEP_EVIDENCE}/invocation.json"),
        &serde_json::to_vec(&serde_json::json!({
            "game_root": "D:\\Spiele\\Max\\BG2",
            "kept": "D:\\Maxwell\\x",
            "note": "max played well",
            "by_path": { "D:\\Spiele\\Max": "keyed by a private path" }
        }))
        .unwrap(),
    );

    let bundle = export_diagnostics(&named_request(&fixture, &["Max"], &[])).unwrap();

    let entries = zip_entries(&bundle.path);
    let evidence = "logs/steps/0001-0123456789abcdef/attempt-0001";
    assert_eq!(
        entry_text(&entries, &format!("{evidence}/process-output.log")),
        "reading D:\\Spiele\\<redacted-user>\\BG2\\weidu.log\n\
kept D:\\Maxwell\\x and D:/Max-Saves/y and D:\\Max_old\\z\n\
max played well\n\
end at D:/Games/<redacted-user>"
    );
    let invocation = entry_json(&entries, &format!("{evidence}/invocation.json"));
    assert_eq!(invocation["game_root"], "D:\\Spiele\\<redacted-user>\\BG2");
    assert_eq!(invocation["kept"], "D:\\Maxwell\\x");
    assert_eq!(invocation["note"], "max played well");
    assert_eq!(invocation["by_path"], "<redacted-sensitive-object>");
}

#[test]
fn the_profile_folder_of_any_user_is_redacted_but_shared_profiles_stay() {
    let fixture = fixture(false);
    write(
        &fixture.managed,
        &format!("{STEP_EVIDENCE}/process-output.log"),
        b"opened D:\\Users\\Max Mustermann\\Documents\\x.log\n\
shared C:\\Users\\Public\\Documents\\a.txt and C:\\users\\DEFAULT USER\\NTUSER.DAT\n\
also C:\\Users\\All Users\\x and C:\\Users\\Default\\y\n\
unix /Users/eva/Library/x\n\
quoted \"C:\\Users\\Ingrid\" and <C:\\Users\\Ole>\n\
not a segment C:\\MyUsers\\Kai\\x and C:\\Users.old\\Kai\n",
    );
    write(
        &fixture.managed,
        &format!("{STEP_EVIDENCE}/invocation.json"),
        &serde_json::to_vec(&serde_json::json!({
            "documents": "E:\\Users\\Someone Else\\Documents",
            "shared": "C:\\Users\\Public\\Documents"
        }))
        .unwrap(),
    );

    let bundle = export_diagnostics(&named_request(&fixture, &[], &[])).unwrap();

    let entries = zip_entries(&bundle.path);
    let evidence = "logs/steps/0001-0123456789abcdef/attempt-0001";
    assert_eq!(
        entry_text(&entries, &format!("{evidence}/process-output.log")),
        "opened D:\\Users\\<redacted-user>\\Documents\\x.log\n\
shared C:\\Users\\Public\\Documents\\a.txt and C:\\users\\DEFAULT USER\\NTUSER.DAT\n\
also C:\\Users\\All Users\\x and C:\\Users\\Default\\y\n\
unix /Users/<redacted-user>/Library/x\n\
quoted \"C:\\Users\\<redacted-user>\" and <C:\\Users\\<redacted-user>>\n\
not a segment C:\\MyUsers\\Kai\\x and C:\\Users.old\\Kai\n"
    );
    let invocation = entry_json(&entries, &format!("{evidence}/invocation.json"));
    assert_eq!(
        invocation["documents"],
        "E:\\Users\\<redacted-user>\\Documents"
    );
    assert_eq!(invocation["shared"], "C:\\Users\\Public\\Documents");
}

#[test]
fn json_escaped_double_backslashes_in_a_text_log_are_redacted() {
    let fixture = fixture(false);
    write(
        &fixture.managed,
        &format!("{STEP_EVIDENCE}/process-output.log"),
        br#"{"path":"D:\\Users\\Eva\\x","game":"E:\\Spiele\\Max\\BG2","kept":"D:\\Users\\Public\\y"}
"#,
    );

    let bundle = export_diagnostics(&named_request(&fixture, &["Max"], &[])).unwrap();

    let entries = zip_entries(&bundle.path);
    assert_eq!(
        entry_text(
            &entries,
            "logs/steps/0001-0123456789abcdef/attempt-0001/process-output.log"
        ),
        r#"{"path":"D:\\Users\\<redacted-user>\\x","game":"E:\\Spiele\\<redacted-user>\\BG2","kept":"D:\\Users\\Public\\y"}
"#
    );
}

#[test]
fn the_computer_name_is_redacted_as_a_whole_word_only() {
    let fixture = fixture(false);
    write(
        &fixture.managed,
        &format!("{STEP_EVIDENCE}/process-output.log"),
        b"host SPIELRECHNER started\n\
share \\\\spielrechner\\saves\n\
kept SPIELRECHNER2 and MY-SPIELRECHNER and spielrechner_old\n\
(SpielRechner)",
    );
    write(
        &fixture.managed,
        &format!("{STEP_EVIDENCE}/invocation.json"),
        br#"{"host":"SPIELRECHNER","kept":"SPIELRECHNERS"}"#,
    );

    let bundle = export_diagnostics(&named_request(&fixture, &[], &["SPIELRECHNER"])).unwrap();

    let entries = zip_entries(&bundle.path);
    let evidence = "logs/steps/0001-0123456789abcdef/attempt-0001";
    assert_eq!(
        entry_text(&entries, &format!("{evidence}/process-output.log")),
        "host <redacted-computer> started\n\
share \\\\<redacted-computer>\\saves\n\
kept SPIELRECHNER2 and MY-SPIELRECHNER and spielrechner_old\n\
(<redacted-computer>)"
    );
    let invocation = entry_json(&entries, &format!("{evidence}/invocation.json"));
    assert_eq!(invocation["host"], "<redacted-computer>");
    assert_eq!(invocation["kept"], "SPIELRECHNERS");
}

#[test]
fn shared_text_is_redacted_like_diagnostics_text() {
    let text = "Install failed: D:\\Spiele\\Max\\BG2\\weidu.log on SPIELRECHNER\n\
Authorization: Bearer invented-share-secret\n\
saves in E:\\Private Root\\Saves\n\
profile C:\\Users\\Eva\\AppData\\x";
    let shared = redact_for_sharing_with_names(
        text,
        &[PathBuf::from("E:\\Private Root")],
        &["Max".to_owned()],
        &["SPIELRECHNER".to_owned()],
    );
    assert_eq!(
        shared,
        "Install failed: D:\\Spiele\\<redacted-user>\\BG2\\weidu.log on <redacted-computer>\n\
<redacted-sensitive-line>\n\
saves in <redacted-home>\\Saves\n\
profile C:\\Users\\<redacted-user>\\AppData\\x"
    );

    // Too short names are never redacted.
    let short = "D:\\Spiele\\M\\x on PC";
    assert_eq!(
        redact_for_sharing_with_names(short, &[], &["M".to_owned()], &["PC".to_owned()]),
        short
    );

    // Names and text outside ASCII are matched without splitting a character.
    assert_eq!(
        redact_for_sharing_with_names(
            "C:\\Users\\Jürgen\\ä Max\\x and D:\\Jürgen\\y, Größe. Größe-Rechner äGröße",
            &[],
            &["jürgen".to_owned(), "Max".to_owned()],
            &["Größe".to_owned()],
        ),
        "C:\\Users\\<redacted-user>\\ä Max\\x and D:\\<redacted-user>\\y, <redacted-computer>. Größe-Rechner äGröße"
    );

    // The environment-based variant is public and redacts the authored roots as well.
    assert_eq!(
        redact_for_sharing(
            "at E:\\Private Root\\x",
            &[PathBuf::from("E:\\Private Root")]
        ),
        "at <redacted-home>\\x"
    );
}

/// An installation interrupted before its first receipt: ledger, frozen recipe and step
/// evidence of two attempts, no attempt receipt.
fn recorded_fixture() -> Fixture {
    let temp = TempDir::new().unwrap();
    let managed = temp.path().join("managed");
    let home = temp.path().join("Users/Player");
    let output = temp.path().join("exports/diagnostics.zip");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    let raw_home = home.to_string_lossy();
    write(
        &managed,
        ".chriz/ledger/0000000000.json",
        &serde_json::to_vec(&serde_json::json!({
            "event": "created", "managed_root": home.join("Games")
        }))
        .unwrap(),
    );
    write(
        &managed,
        ".chriz/ledger/0000000001.json",
        br#"{"event":"step_started"}"#,
    );
    write(&managed, ".chriz/recipe/payload.zip", b"PK\x03\x04recipe");
    write(
        &managed,
        ".chriz/recipe/envelope.json",
        br#"{"version":"alpha-1"}"#,
    );
    write(
        &managed,
        &format!("{STEP_EVIDENCE}/process-output.log"),
        format!(
            "reading D:\\Spiele\\Max\\BG2 and {raw_home}\\Games\n\
Authorization: Bearer invented-recorded-secret\n\
first attempt line\n"
        )
        .as_bytes(),
    );
    // The same step in a later attempt; the attempt id keeps both apart.
    write(
        &managed,
        ".chriz/attempts/attempt-002/steps/0001-0123456789abcdef/attempt-0001/process-output.log",
        b"second attempt line\n",
    );
    write(
        &managed,
        ".chriz/attempts/attempt-002/steps/0002-fedcba9876543210/attempt-0001/weidu.debug.log",
        b"weidu still running\n",
    );
    // Neither a folder whose name is no attempt id nor a file is an attempt.
    write(
        &managed,
        ".chriz/attempts/not an id/steps/0001-0123456789abcdef/attempt-0001/process-output.log",
        b"not an attempt\n",
    );
    write(&managed, ".chriz/attempts/stray.json", b"not an attempt\n");
    write(&managed, ".chriz/unrelated.txt", b"outside allowlist");
    write(&managed, "game/chitin.key", b"game content");
    Fixture {
        _temp: temp,
        managed,
        home,
        output,
    }
}

#[test]
fn an_installation_without_a_terminal_receipt_exports_what_it_recorded() {
    let fixture = recorded_fixture();
    let reason = format!(
        "no immutable attempt receipt exists under {}",
        fixture.home.join("Games").display()
    );
    let request = recorded_request(&fixture, Some(reason));

    let bundle = export_diagnostics(&request).unwrap();

    let entries = zip_entries(&bundle.path);
    let names = entries
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    assert_eq!(bundle.entries, names);
    assert_eq!(
        names,
        [
            "START-HERE.txt",
            "ledger/0000000000.json",
            "ledger/0000000001.json",
            "logs/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/process-output.log",
            "logs/attempts/attempt-002/steps/0001-0123456789abcdef/attempt-0001/process-output.log",
            "logs/attempts/attempt-002/steps/0002-fedcba9876543210/attempt-0001/weidu.debug.log",
            "recipe/envelope.json",
            "recipe/payload.zip",
        ]
    );
    let start_here = entry_text(&entries, "START-HERE.txt");
    assert!(
        start_here.contains("No finished attempt is recorded for this installation"),
        "{start_here}"
    );
    assert!(
        start_here.contains("interrupted or is still running"),
        "{start_here}"
    );
    assert!(
        start_here.contains(
            "Why no receipt summary: no immutable attempt receipt exists under <redacted-home>"
        ),
        "{start_here}"
    );
    assert!(start_here.contains("logs/attempts/"), "{start_here}");
    assert!(
        start_here.ends_with("Personal paths, your Windows user name, the computer name and common secrets are redacted, but review the ZIP before sharing.\n"),
        "{start_here}"
    );
    let first = entry_text(
        &entries,
        "logs/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/process-output.log",
    );
    assert_eq!(
        first,
        "reading D:\\Spiele\\<redacted-user>\\BG2 and <redacted-home>\\Games\n\
<redacted-sensitive-line>\n\
first attempt line\n"
    );
    let ledger = entry_json(&entries, "ledger/0000000000.json");
    assert!(
        ledger["managed_root"]
            .as_str()
            .unwrap()
            .starts_with("<redacted-home>"),
        "{ledger}"
    );
    assert_eq!(
        std::fs::read(&bundle.path).unwrap(),
        std::fs::read(&fixture.output).unwrap()
    );

    // Re-exporting to the same regular .zip path atomically overwrites it.
    let overwritten = export_diagnostics(&request).unwrap();
    assert_eq!(overwritten.path, bundle.path);
    assert!(bg_engine::reparse::same_normalized_windows_path(
        &overwritten.path,
        &bg_engine::reparse::canonicalize_flexible(&fixture.output).unwrap()
    ));
    assert!(overwritten.path.is_file());

    // An existing directory at the output path is still refused.
    std::fs::remove_file(&fixture.output).unwrap();
    std::fs::create_dir(&fixture.output).unwrap();
    let error = export_diagnostics(&request).unwrap_err();
    assert!(
        matches!(error, DiagnosticsError::UnsafePath { .. }),
        "{error}"
    );
    assert!(fixture.output.is_dir());
}

#[test]
fn a_recorded_export_names_no_reason_when_none_is_given_and_needs_no_ledger_or_attempts() {
    let temp = TempDir::new().unwrap();
    let managed = temp.path().join("managed");
    write(
        &managed,
        ".chriz/recipe/envelope.json",
        br#"{"version":"1"}"#,
    );
    let output = temp.path().join("diagnostics.zip");

    let bundle = export_diagnostics(&DiagnosticsRequest {
        managed_root: managed,
        attempt_id: None,
        output_path: output,
        redact_roots: Vec::new(),
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: None,
    })
    .unwrap();

    assert_eq!(bundle.entries, ["START-HERE.txt", "recipe/envelope.json"]);
    let start_here = entry_text(&zip_entries(&bundle.path), "START-HERE.txt");
    assert!(start_here.contains("No finished attempt is recorded"));
    assert!(
        !start_here.contains("Why no receipt summary"),
        "{start_here}"
    );
}

#[test]
fn a_recorded_export_includes_attempt_receipts_that_exist() {
    let fixture = recorded_fixture();
    write(
        &fixture.managed,
        ".chriz/attempts/attempt-002/receipt.json",
        &serde_json::to_vec(&serde_json::json!({
            "outcome": "failed", "path": fixture.home.join("Games"), "token": "invented-receipt-token"
        }))
        .unwrap(),
    );

    let bundle = export_diagnostics(&recorded_request(&fixture, None)).unwrap();

    let entries = zip_entries(&bundle.path);
    let receipt = entry_json(
        &entries,
        "receipt/attempts/attempt-002/attempt-receipt.json",
    );
    assert_eq!(receipt["outcome"], "failed");
    assert_eq!(receipt["token"], "<redacted-sensitive-value>");
    assert!(receipt["path"]
        .as_str()
        .unwrap()
        .starts_with("<redacted-home>"));
    assert!(!entries
        .iter()
        .any(|(name, _)| name == "receipt/attempt-receipt.json"));
}

#[test]
fn a_recorded_export_with_nothing_selectable_fails_and_publishes_nothing() {
    let temp = TempDir::new().unwrap();
    let managed = temp.path().join("managed");
    write(&managed, ".chriz/unrelated.txt", b"outside allowlist");
    std::fs::create_dir_all(managed.join(".chriz/ledger")).unwrap();
    std::fs::create_dir_all(managed.join(".chriz/attempts/attempt-001/steps")).unwrap();
    let exports = temp.path().join("exports");
    std::fs::create_dir_all(&exports).unwrap();
    let output = exports.join("diagnostics.zip");

    let error = export_diagnostics(&DiagnosticsRequest {
        managed_root: managed.clone(),
        attempt_id: None,
        output_path: output.clone(),
        redact_roots: Vec::new(),
        redact_user_names: Vec::new(),
        redact_computer_names: Vec::new(),
        missing_receipt_reason: Some("interrupted".to_owned()),
    })
    .unwrap_err();

    match error {
        DiagnosticsError::MissingEvidence { path } => {
            assert!(path.ends_with(".chriz"), "{}", path.display());
        }
        other => panic!("unexpected error: {other}"),
    }
    assert!(!output.exists());
    assert_eq!(std::fs::read_dir(&exports).unwrap().count(), 0);
}

/// Creates a directory link at `link`; `false` when this platform cannot create one.
fn link_directory(link: &Path, target: &Path) -> bool {
    #[cfg(windows)]
    {
        // A junction needs no privilege, unlike a symbolic link.
        std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .is_ok_and(|output| output.status.success())
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = (link, target);
        false
    }
}

#[test]
fn a_link_in_place_of_an_attempt_folder_stops_the_recorded_export() {
    let fixture = recorded_fixture();
    let outside = fixture._temp.path().join("outside-attempt");
    write(
        &outside,
        "steps/0001-0123456789abcdef/attempt-0001/process-output.log",
        b"outside the installation\n",
    );
    let link = fixture.managed.join(".chriz/attempts/attempt-009");
    if !link_directory(&link, &outside) {
        eprintln!("this platform cannot create a directory link; skipped");
        return;
    }

    let error = export_diagnostics(&recorded_request(&fixture, None)).unwrap_err();

    assert!(
        matches!(error, DiagnosticsError::UnsafePath { .. }),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains("links and reparse points are forbidden in diagnostics evidence"),
        "{error}"
    );
    assert!(!fixture.output.exists());
}

/// A log the running installation holds open without sharing is named in the bundle instead
/// of stopping an export without a terminal receipt; an export of a terminal receipt still
/// stops.
#[cfg(windows)]
#[test]
fn a_log_locked_by_a_running_installation_is_named_instead_of_exported() {
    use std::os::windows::fs::OpenOptionsExt;

    let recorded = recorded_fixture();
    let locked = recorded
        .managed
        .join(format!("{STEP_EVIDENCE}/process-output.log"));
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&locked)
        .unwrap();

    let bundle = export_diagnostics(&recorded_request(&recorded, None)).unwrap();

    let entries = zip_entries(&bundle.path);
    let text = entry_text(
        &entries,
        "logs/attempts/attempt-001/steps/0001-0123456789abcdef/attempt-0001/process-output.log",
    );
    assert!(
        text.starts_with("This file could not be read while diagnostics were exported: "),
        "{text}"
    );
    assert_eq!(text.lines().count(), 1, "{text}");
    assert_eq!(
        entry_text(
            &entries,
            "logs/attempts/attempt-002/steps/0001-0123456789abcdef/attempt-0001/process-output.log"
        ),
        "second attempt line\n"
    );

    // With a terminal receipt the same lock still stops the export.
    let receipted = fixture(false);
    let receipted_lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(
            receipted
                .managed
                .join(format!("{STEP_EVIDENCE}/process-output.log")),
        )
        .unwrap();
    let error = export_diagnostics(&named_request(&receipted, &[], &[])).unwrap_err();
    assert!(matches!(error, DiagnosticsError::Io { .. }), "{error}");
    assert!(!receipted.output.exists());
    drop(receipted_lock);
    drop(lock);
}

#[test]
fn the_cli_export_falls_back_to_recorded_evidence_without_a_receipt() {
    let fixture = recorded_fixture();

    let bundle =
        bg_engine::cli::diagnostics_for_managed_install(&fixture.managed, &fixture.output).unwrap();

    let entries = zip_entries(&bundle.path);
    let start_here = entry_text(&entries, "START-HERE.txt");
    assert!(
        start_here.contains("No finished attempt is recorded"),
        "{start_here}"
    );
    assert!(
        start_here.contains("Why no receipt summary: could not open frozen campaign"),
        "{start_here}"
    );
    assert!(bundle.entries.contains(
        &"logs/attempts/attempt-002/steps/0002-fedcba9876543210/attempt-0001/weidu.debug.log"
            .to_owned()
    ));
    assert!(!entries
        .iter()
        .flat_map(|(_, bytes)| bytes.iter().copied())
        .collect::<Vec<_>>()
        .windows(b"invented-recorded-secret".len())
        .any(|window| window == b"invented-recorded-secret"));
}
