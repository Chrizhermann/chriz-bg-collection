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
        format!(r#"{{"outcome":"failed","path":"{raw_home}","token":"secret-token"}}"#).as_bytes(),
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
