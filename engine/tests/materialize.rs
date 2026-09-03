use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use bg_engine::acquire::{
    extract_archive, materialize, AcquireError, ArchiveFormat, ArchiveLimits, ArchiveMode,
    ArchiveRequirements, ExtractedArtifact, MaterializationRequest, SignedCollisionRule,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const TP2: &[u8] = include_bytes!("fixtures/archives/simple/mod/setup-mod.tp2");

fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn sha256_file(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).unwrap();
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    hex::encode(hash.finalize())
}

fn extracted(
    temp: &TempDir,
    name: &str,
    files: &[(&str, &[u8])],
    roots: &[&str],
    tp2_paths: &[&str],
) -> ExtractedArtifact {
    let archive = temp.path().join(format!("{name}.zip"));
    let writer_file = File::create(&archive).unwrap();
    let mut writer = ZipWriter::new(writer_file);
    for (path, contents) in files {
        writer
            .start_file(
                *path,
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
            )
            .unwrap();
        writer.write_all(contents).unwrap();
    }
    writer.finish().unwrap();
    let requirements = ArchiveRequirements {
        artifact_sha256: sha256_file(&archive),
        format: ArchiveFormat::Zip,
        expected_roots: roots.iter().map(|value| (*value).to_owned()).collect(),
        expected_tp2_paths: tp2_paths.iter().map(|value| (*value).to_owned()).collect(),
        limits: ArchiveLimits::default(),
        mode: ArchiveMode::Public,
    };
    extract_archive(
        &archive,
        &temp.path().join(format!("extract-{name}")),
        &requirements,
    )
    .unwrap()
}

fn request(id: &str, owner: &str, roots: &[&str], tp2_paths: &[&str]) -> MaterializationRequest {
    MaterializationRequest {
        materialization_id: id.to_owned(),
        owner: owner.to_owned(),
        roots: roots.iter().map(|value| (*value).to_owned()).collect(),
        tp2_paths: tp2_paths.iter().map(|value| (*value).to_owned()).collect(),
        collision_rules: Vec::new(),
    }
}

fn temp_sibling(destination: &Path, materialization_id: &str) -> PathBuf {
    let name = destination.file_name().unwrap().to_string_lossy();
    destination.with_file_name(format!(".{name}.chriz-bg-{materialization_id}.tmp"))
}

#[test]
fn publishes_only_declared_payload_and_skips_archive_setup_executables() {
    let temp = TempDir::new().unwrap();
    let payload = extracted(
        &temp,
        "declared",
        &[
            ("mod/setup-mod.tp2", TP2),
            ("mod/data.txt", b"payload"),
            ("mod/setup-helper.exe", b"untrusted executable"),
            ("unrelated/readme.txt", b"not declared"),
        ],
        &["mod"],
        &["mod/setup-mod.tp2"],
    );
    let destination = temp.path().join("game");
    std::fs::create_dir(&destination).unwrap();

    let result = materialize(
        &payload,
        &destination,
        &request("declared-run", "mod", &["mod"], &["mod/setup-mod.tp2"]),
    )
    .unwrap();

    assert_eq!(
        std::fs::read(destination.join("mod/data.txt")).unwrap(),
        b"payload"
    );
    assert_eq!(
        std::fs::read(destination.join("mod/setup-mod.tp2")).unwrap(),
        TP2
    );
    assert!(!destination.join("mod/setup-helper.exe").exists());
    assert!(!destination.join("unrelated").exists());
    assert_eq!(result.manifest.entries.len(), 2);
    assert_eq!(
        result
            .manifest
            .entries
            .iter()
            .map(|entry| entry.relative_path.as_str())
            .collect::<Vec<_>>(),
        vec!["mod/data.txt", "mod/setup-mod.tp2"]
    );
    let first_manifest = std::fs::read(&result.manifest_path).unwrap();

    let repeated = materialize(
        &payload,
        &destination,
        &request("declared-run", "mod", &["mod"], &["mod/setup-mod.tp2"]),
    )
    .unwrap();
    assert_eq!(
        first_manifest,
        std::fs::read(repeated.manifest_path).unwrap()
    );
}

#[test]
fn accepts_but_does_not_materialize_auxiliary_tp2_outside_publish_roots() {
    let temp = TempDir::new().unwrap();
    let payload = extracted(
        &temp,
        "auxiliary-tp2",
        &[
            ("mod/setup-mod.tp2", TP2),
            ("mod/data.txt", b"payload"),
            ("live-patch/setup-live-patch.tp2", TP2),
        ],
        &["mod"],
        &["mod/setup-mod.tp2"],
    );
    assert!(payload
        .root
        .join("live-patch/setup-live-patch.tp2")
        .is_file());
    let destination = temp.path().join("game");
    std::fs::create_dir(&destination).unwrap();

    let result = materialize(
        &payload,
        &destination,
        &request("auxiliary-tp2-run", "mod", &["mod"], &["mod/setup-mod.tp2"]),
    )
    .unwrap();

    assert!(destination.join("mod/setup-mod.tp2").is_file());
    assert!(!destination.join("live-patch").exists());
    assert!(result
        .manifest
        .entries
        .iter()
        .all(|entry| !entry.relative_path.starts_with("live-patch/")));
}

#[test]
fn rejects_unknown_overwrites_but_accepts_identical_existing_bytes() {
    let temp = TempDir::new().unwrap();
    let payload = extracted(
        &temp,
        "overwrite",
        &[("mod/setup-mod.tp2", TP2), ("mod/data.txt", b"expected")],
        &["mod"],
        &["mod/setup-mod.tp2"],
    );
    let destination = temp.path().join("game");
    std::fs::create_dir_all(destination.join("mod")).unwrap();
    std::fs::write(destination.join("mod/data.txt"), b"unknown").unwrap();
    let materialization = request("overwrite-run", "mod", &["mod"], &["mod/setup-mod.tp2"]);

    assert!(matches!(
        materialize(&payload, &destination, &materialization),
        Err(AcquireError::UndeclaredOverwrite { .. })
    ));
    assert_eq!(
        std::fs::read(destination.join("mod/data.txt")).unwrap(),
        b"unknown"
    );
    assert!(!destination.join("mod/setup-mod.tp2").exists());

    std::fs::write(destination.join("mod/data.txt"), b"expected").unwrap();
    let result = materialize(&payload, &destination, &materialization).unwrap();
    assert_eq!(result.manifest.entries.len(), 2);
}

#[test]
fn signed_collision_rule_requires_both_owners_and_exact_input_output_hashes() {
    let temp = TempDir::new().unwrap();
    let old = extracted(
        &temp,
        "old",
        &[("mod/setup-mod.tp2", TP2), ("mod/data.txt", b"old bytes")],
        &["mod"],
        &["mod/setup-mod.tp2"],
    );
    let new = extracted(
        &temp,
        "new",
        &[("mod/setup-mod.tp2", TP2), ("mod/data.txt", b"new bytes")],
        &["mod"],
        &["mod/setup-mod.tp2"],
    );
    let destination = temp.path().join("game");
    std::fs::create_dir(&destination).unwrap();
    materialize(
        &old,
        &destination,
        &request("old-run", "old-owner", &["mod"], &["mod/setup-mod.tp2"]),
    )
    .unwrap();

    let mut replacement = request("new-run", "new-owner", &["mod"], &["mod/setup-mod.tp2"]);
    replacement.collision_rules.push(SignedCollisionRule {
        relative_path: "mod/data.txt".to_owned(),
        existing_owner: "old-owner".to_owned(),
        incoming_owner: "new-owner".to_owned(),
        expected_input_sha256: sha256_bytes(b"old bytes"),
        expected_output_sha256: sha256_bytes(b"new bytes"),
    });

    let result = materialize(&new, &destination, &replacement).unwrap();
    assert_eq!(
        std::fs::read(destination.join("mod/data.txt")).unwrap(),
        b"new bytes"
    );
    let entry = result
        .manifest
        .entries
        .iter()
        .find(|entry| entry.relative_path == "mod/data.txt")
        .unwrap();
    assert_eq!(entry.owner, "new-owner");
    assert_eq!(entry.sha256, sha256_bytes(b"new bytes"));

    let other_destination = temp.path().join("other-game");
    std::fs::create_dir(&other_destination).unwrap();
    materialize(
        &old,
        &other_destination,
        &request("old-run-2", "old-owner", &["mod"], &["mod/setup-mod.tp2"]),
    )
    .unwrap();
    replacement.collision_rules[0].existing_owner = "wrong-owner".to_owned();
    assert!(matches!(
        materialize(&new, &other_destination, &replacement),
        Err(AcquireError::CollisionRuleRejected { .. })
    ));
    assert_eq!(
        std::fs::read(other_destination.join("mod/data.txt")).unwrap(),
        b"old bytes"
    );
}

#[test]
fn resume_removes_only_its_temp_and_reconciles_an_identical_published_prefix() {
    let temp = TempDir::new().unwrap();
    let payload = extracted(
        &temp,
        "resume",
        &[
            ("mod/setup-mod.tp2", TP2),
            ("mod/a.txt", b"a"),
            ("mod/b.txt", b"b"),
        ],
        &["mod"],
        &["mod/setup-mod.tp2"],
    );
    let destination = temp.path().join("game");
    std::fs::create_dir_all(destination.join("mod")).unwrap();
    std::fs::write(destination.join("mod/a.txt"), b"a").unwrap();
    let owned_temp = temp_sibling(&destination.join("mod/b.txt"), "resume-run");
    std::fs::write(&owned_temp, b"truncated").unwrap();
    let unrelated_temp = destination.join("mod/.b.txt.someone-else.tmp");
    std::fs::write(&unrelated_temp, b"keep").unwrap();

    let result = materialize(
        &payload,
        &destination,
        &request("resume-run", "mod", &["mod"], &["mod/setup-mod.tp2"]),
    )
    .unwrap();

    assert!(!owned_temp.exists());
    assert_eq!(std::fs::read(unrelated_temp).unwrap(), b"keep");
    assert_eq!(std::fs::read(destination.join("mod/b.txt")).unwrap(), b"b");
    assert_eq!(result.manifest.entries.len(), 3);
}

#[test]
fn a_truncated_previously_recorded_destination_fails_closed() {
    let temp = TempDir::new().unwrap();
    let payload = extracted(
        &temp,
        "truncated",
        &[("mod/setup-mod.tp2", TP2), ("mod/data.txt", b"complete")],
        &["mod"],
        &["mod/setup-mod.tp2"],
    );
    let destination = temp.path().join("game");
    std::fs::create_dir(&destination).unwrap();
    let materialization = request("truncate-run", "mod", &["mod"], &["mod/setup-mod.tp2"]);
    materialize(&payload, &destination, &materialization).unwrap();
    std::fs::write(destination.join("mod/data.txt"), b"trunc").unwrap();

    assert!(matches!(
        materialize(&payload, &destination, &materialization),
        Err(AcquireError::PublicationMismatch { .. })
    ));
}

#[test]
fn one_extraction_supports_split_payloads_and_independent_game_roots() {
    let temp = TempDir::new().unwrap();
    let payload = extracted(
        &temp,
        "split",
        &[
            ("one/setup-one.tp2", TP2),
            ("one/data.txt", b"one"),
            ("two/setup-two.tp2", TP2),
            ("two/data.txt", b"two"),
        ],
        &["one", "two"],
        &["one/setup-one.tp2", "two/setup-two.tp2"],
    );
    let bg1 = temp.path().join("bg1");
    let bg2 = temp.path().join("bg2");
    std::fs::create_dir(&bg1).unwrap();
    std::fs::create_dir(&bg2).unwrap();

    materialize(
        &payload,
        &bg1,
        &request("one-bg1", "one", &["one"], &["one/setup-one.tp2"]),
    )
    .unwrap();
    materialize(
        &payload,
        &bg1,
        &request("two-bg1", "two", &["two"], &["two/setup-two.tp2"]),
    )
    .unwrap();
    materialize(
        &payload,
        &bg2,
        &request("one-bg2", "one", &["one"], &["one/setup-one.tp2"]),
    )
    .unwrap();

    assert_eq!(std::fs::read(bg1.join("one/data.txt")).unwrap(), b"one");
    assert_eq!(std::fs::read(bg1.join("two/data.txt")).unwrap(), b"two");
    assert_eq!(std::fs::read(bg2.join("one/data.txt")).unwrap(), b"one");
    assert!(!bg2.join("two").exists());
}

#[test]
fn refuses_to_materialize_files_added_after_verified_extraction() {
    let temp = TempDir::new().unwrap();
    let payload = extracted(
        &temp,
        "tampered",
        &[("mod/setup-mod.tp2", TP2), ("mod/data.txt", b"verified")],
        &["mod"],
        &["mod/setup-mod.tp2"],
    );
    std::fs::write(payload.root.join("mod/injected.txt"), b"not verified").unwrap();
    let destination = temp.path().join("game");
    std::fs::create_dir(&destination).unwrap();

    assert!(materialize(
        &payload,
        &destination,
        &request("tampered-run", "mod", &["mod"], &["mod/setup-mod.tp2"]),
    )
    .is_err());
    assert!(!destination.join("mod").exists());
}

#[test]
fn reserves_the_engine_state_directory_from_archive_payloads() {
    let temp = TempDir::new().unwrap();
    let payload = extracted(
        &temp,
        "state-collision",
        &[(".chriz-bg-collection/setup-state.tp2", TP2)],
        &[".chriz-bg-collection"],
        &[".chriz-bg-collection/setup-state.tp2"],
    );
    let destination = temp.path().join("game");
    std::fs::create_dir(&destination).unwrap();

    assert!(materialize(
        &payload,
        &destination,
        &request(
            "state-collision",
            "state",
            &[".chriz-bg-collection"],
            &[".chriz-bg-collection/setup-state.tp2"],
        ),
    )
    .is_err());
    assert!(!destination.join(".chriz-bg-collection").exists());
}
