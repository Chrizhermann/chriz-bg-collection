use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use std::process::Command;

use bg_engine::digest::sha256_bytes;
use bg_engine::recipe_envelope::{
    verify_recipe_package, KeyRotationStatement, RecipeEnvelope, RecipeTrustStore, TrustedPublicKey,
};
use minisign::{sign, KeyPair, SecretKey};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

struct TestKey {
    public: String,
    secret: SecretKey,
}

impl TestKey {
    fn generate() -> Self {
        let pair = KeyPair::generate_unencrypted_keypair().unwrap();
        Self {
            public: pair.pk.to_box().unwrap().to_string(),
            secret: pair.sk,
        }
    }

    fn sign(&self, bytes: &[u8]) -> String {
        sign(
            None,
            &self.secret,
            Cursor::new(bytes),
            Some("recipe envelope test"),
            None,
        )
        .unwrap()
        .into_string()
    }
}

fn ledger(version: &str, supersedes: Option<&str>, minimum_app: &str) -> String {
    let mut text = format!(
        "schema = 1\nrecipe_id = \"chriz-bg-collection\"\nversion = \"{version}\"\npublished_at = \"2026-09-04T00:00:00Z\"\nminimum_app_version = \"{minimum_app}\"\n"
    );
    if let Some(previous) = supersedes {
        text.push_str(&format!("supersedes = \"{previous}\"\n"));
    }
    text.push_str("changes = []\n");
    text
}

fn payload(version: &str, ledger: &str) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::default());
    writer.start_file("collection.toml", options).unwrap();
    writer.write_all(b"schema=2\ngame_build='2.7'\n").unwrap();
    writer
        .start_file(format!("releases/v{version}/ledger.toml"), options)
        .unwrap();
    writer.write_all(ledger.as_bytes()).unwrap();
    writer.finish().unwrap().into_inner()
}

fn signed_package(
    key: &TestKey,
    key_id: &str,
    version: &str,
    supersedes: Option<&str>,
    minimum_app: &str,
) -> (Vec<u8>, Vec<u8>, String) {
    let payload = payload(version, &ledger(version, supersedes, minimum_app));
    let envelope = RecipeEnvelope {
        recipe_id: "chriz-bg-collection".to_owned(),
        version: version.to_owned(),
        payload_sha256: sha256_bytes(&payload),
        key_id: key_id.to_owned(),
        minimum_app_version: minimum_app.to_owned(),
        published_at: "2026-09-04T00:00:00Z".to_owned(),
    };
    let envelope_bytes = serde_json::to_vec(&envelope).unwrap();
    let signature = key.sign(&envelope_bytes);
    (payload, envelope_bytes, signature)
}

fn trust(key_id: &str, key: &TestKey) -> RecipeTrustStore {
    RecipeTrustStore::new([TrustedPublicKey {
        key_id: key_id.to_owned(),
        minisign_public_key: key.public.clone(),
    }])
    .unwrap()
}

#[test]
fn verifies_signature_over_exact_envelope_and_digest_over_exact_payload_bytes() {
    let key = TestKey::generate();
    let (payload, envelope, signature) =
        signed_package(&key, "test-alpha", "0.1.0-alpha.1", None, "0.1.0");
    let verified = verify_recipe_package(
        &payload,
        &envelope,
        signature.as_bytes(),
        &trust("test-alpha", &key),
        "0.1.0",
        None,
    )
    .unwrap();
    assert_eq!(verified.payload_bytes, payload);
    assert_eq!(verified.envelope_bytes, envelope);
    assert_eq!(verified.envelope.version, "0.1.0-alpha.1");
}

#[test]
fn rejects_envelope_payload_and_ledger_tamper() {
    let key = TestKey::generate();
    let (payload_bytes, envelope, signature) =
        signed_package(&key, "test-alpha", "0.1.0-alpha.1", None, "0.1.0");
    let roots = trust("test-alpha", &key);

    let mut envelope_tamper = envelope.clone();
    envelope_tamper.push(b' ');
    assert!(verify_recipe_package(
        &payload_bytes,
        &envelope_tamper,
        signature.as_bytes(),
        &roots,
        "0.1.0",
        None,
    )
    .is_err());

    let mut payload_tamper = payload_bytes.clone();
    *payload_tamper.last_mut().unwrap() ^= 1;
    assert!(verify_recipe_package(
        &payload_tamper,
        &envelope,
        signature.as_bytes(),
        &roots,
        "0.1.0",
        None,
    )
    .is_err());

    let wrong_ledger_payload = payload(
        "0.1.0-alpha.1",
        &ledger("0.1.0-alpha.2", Some("0.1.0-alpha.1"), "0.1.0"),
    );
    let wrong_envelope = RecipeEnvelope {
        payload_sha256: sha256_bytes(&wrong_ledger_payload),
        ..serde_json::from_slice(&envelope).unwrap()
    };
    let wrong_envelope = serde_json::to_vec(&wrong_envelope).unwrap();
    let wrong_signature = key.sign(&wrong_envelope);
    assert!(verify_recipe_package(
        &wrong_ledger_payload,
        &wrong_envelope,
        wrong_signature.as_bytes(),
        &roots,
        "0.1.0",
        None,
    )
    .unwrap_err()
    .to_string()
    .contains("ledger"));
}

#[test]
fn rejects_unknown_key_and_malformed_signature() {
    let key = TestKey::generate();
    let other = TestKey::generate();
    let (payload, envelope, signature) =
        signed_package(&key, "unknown", "0.1.0-alpha.1", None, "0.1.0");
    let roots = trust("other", &other);
    assert!(verify_recipe_package(
        &payload,
        &envelope,
        signature.as_bytes(),
        &roots,
        "0.1.0",
        None,
    )
    .unwrap_err()
    .to_string()
    .contains("unknown key"));

    let (payload, envelope, _) = signed_package(&other, "other", "0.1.0-alpha.1", None, "0.1.0");
    assert!(verify_recipe_package(
        &payload,
        &envelope,
        b"not a minisign signature",
        &roots,
        "0.1.0",
        None,
    )
    .is_err());
}

#[test]
fn rejects_replay_downgrade_and_minimum_app_violation() {
    let key = TestKey::generate();
    let roots = trust("test-alpha", &key);
    let (payload, envelope, signature) =
        signed_package(&key, "test-alpha", "0.1.0-alpha.1", None, "0.2.0");
    assert!(verify_recipe_package(
        &payload,
        &envelope,
        signature.as_bytes(),
        &roots,
        "0.1.0-alpha.1",
        None,
    )
    .unwrap_err()
    .to_string()
    .contains("minimum app"));

    for highest in ["0.1.0-alpha.1", "0.1.0-alpha.2"] {
        assert!(verify_recipe_package(
            &payload,
            &envelope,
            signature.as_bytes(),
            &roots,
            "0.2.0",
            Some(highest),
        )
        .unwrap_err()
        .to_string()
        .contains("newer"));
    }
}

#[test]
fn key_rotation_requires_statement_signed_by_current_trusted_key() {
    let old = TestKey::generate();
    let new = TestKey::generate();
    let mut roots = trust("old", &old);
    let (payload, envelope, signature) =
        signed_package(&new, "new", "0.1.0-alpha.2", Some("0.1.0-alpha.1"), "0.1.0");
    assert!(verify_recipe_package(
        &payload,
        &envelope,
        signature.as_bytes(),
        &roots,
        "0.1.0",
        Some("0.1.0-alpha.1"),
    )
    .is_err());

    let statement = serde_json::to_vec(&KeyRotationStatement {
        recipe_id: "chriz-bg-collection".to_owned(),
        from_key_id: "old".to_owned(),
        to_key_id: "new".to_owned(),
        to_minisign_public_key: new.public.clone(),
        activation_version: "0.1.0-alpha.2".to_owned(),
    })
    .unwrap();
    roots
        .authorize_rotation(&statement, old.sign(&statement).as_bytes())
        .unwrap();
    verify_recipe_package(
        &payload,
        &envelope,
        signature.as_bytes(),
        &roots,
        "0.1.0",
        Some("0.1.0-alpha.1"),
    )
    .unwrap();
}

fn write_recipe(root: &Path) {
    fs::create_dir_all(root.join("releases/v0.1.0-alpha.1")).unwrap();
    fs::write(root.join("collection.toml"), "schema=2\ngame_build='2.7'\n").unwrap();
    fs::write(
        root.join("releases/v0.1.0-alpha.1/ledger.toml"),
        ledger("0.1.0-alpha.1", None, "0.1.0-alpha.1"),
    )
    .unwrap();
}

fn author() -> Command {
    Command::new(env!("CARGO_BIN_EXE_chriz-bg-author"))
}

#[test]
fn author_emits_deterministic_payload_and_envelope_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let recipe = temp.path().join("recipe");
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    write_recipe(&recipe);

    for output in [&first, &second] {
        let result = author()
            .args([
                "package-recipe",
                "--recipe-root",
                recipe.to_str().unwrap(),
                "--version",
                "0.1.0-alpha.1",
                "--output-directory",
                output.to_str().unwrap(),
                "--key-id",
                "ephemeral-test",
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    assert_eq!(
        fs::read(first.join("payload.zip")).unwrap(),
        fs::read(second.join("payload.zip")).unwrap()
    );
    assert_eq!(
        fs::read(first.join("envelope.json")).unwrap(),
        fs::read(second.join("envelope.json")).unwrap()
    );
}

#[test]
fn author_verifier_accepts_only_the_injected_ephemeral_public_key() {
    let temp = tempfile::tempdir().unwrap();
    let recipe = temp.path().join("recipe");
    let output = temp.path().join("output");
    write_recipe(&recipe);
    let packaged = author()
        .args([
            "package-recipe",
            "--recipe-root",
            recipe.to_str().unwrap(),
            "--version",
            "0.1.0-alpha.1",
            "--output-directory",
            output.to_str().unwrap(),
            "--key-id",
            "ephemeral-test",
        ])
        .output()
        .unwrap();
    assert!(packaged.status.success());

    let key = TestKey::generate();
    fs::write(
        output.join("envelope.json.minisig"),
        key.sign(&fs::read(output.join("envelope.json")).unwrap()),
    )
    .unwrap();
    let public = temp.path().join("ephemeral.pub");
    fs::write(&public, &key.public).unwrap();
    let verified = author()
        .args([
            "verify-recipe",
            output.to_str().unwrap(),
            "--key-id",
            "ephemeral-test",
            "--trusted-public-key-file",
            public.to_str().unwrap(),
            "--running-app-version",
            "0.1.0-alpha.1",
        ])
        .output()
        .unwrap();
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
}
