use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use bg_engine::acquire::{
    extract_archive, AcquireError, ArchiveFormat, ArchiveLimits, ArchiveMode, ArchiveRequirements,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use zip::unstable::write::FileOptionsExt;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const TP2: &[u8] = include_bytes!("fixtures/archives/simple/mod/setup-mod.tp2");
const DATA: &[u8] = include_bytes!("fixtures/archives/simple/mod/data.txt");

#[derive(Clone)]
enum Entry<'a> {
    File(&'a str, &'a [u8], CompressionMethod),
    Directory(&'a str),
    Symlink(&'a str, &'a str),
    Encrypted(&'a str, &'a [u8]),
}

fn write_zip(path: &Path, entries: &[Entry<'_>]) {
    let file = File::create(path).unwrap();
    let mut writer = ZipWriter::new(file);
    for entry in entries {
        match entry {
            Entry::File(name, contents, compression) => {
                writer
                    .start_file(
                        *name,
                        SimpleFileOptions::default().compression_method(*compression),
                    )
                    .unwrap();
                writer.write_all(contents).unwrap();
            }
            Entry::Directory(name) => writer
                .add_directory(*name, SimpleFileOptions::default())
                .unwrap(),
            Entry::Symlink(name, target) => writer
                .add_symlink(*name, *target, SimpleFileOptions::default())
                .unwrap(),
            Entry::Encrypted(name, contents) => {
                let options = SimpleFileOptions::default()
                    .with_deprecated_encryption(b"test-password")
                    .unwrap();
                writer.start_file(*name, options).unwrap();
                writer.write_all(contents).unwrap();
            }
        }
    }
    writer.finish().unwrap();
}

fn sha256(path: &Path) -> String {
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

fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn edit_extraction_marker(root: &Path, edit: impl FnOnce(&mut serde_json::Value)) {
    let path = root.join(".chriz-bg-extraction.json");
    let mut marker: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    edit(&mut marker);
    let mut bytes = serde_json::to_vec_pretty(&marker).unwrap();
    bytes.push(b'\n');
    std::fs::write(path, bytes).unwrap();
}

fn requirements(path: &Path) -> ArchiveRequirements {
    ArchiveRequirements {
        artifact_sha256: sha256(path),
        format: ArchiveFormat::Zip,
        expected_roots: vec!["mod".to_owned()],
        expected_tp2_paths: vec!["mod/setup-mod.tp2".to_owned()],
        limits: ArchiveLimits::default(),
        mode: ArchiveMode::Public,
    }
}

fn published_path(cache: &Path, digest: &str) -> PathBuf {
    cache.join("sha256-v2").join(&digest[..2]).join(digest)
}

fn assert_rejected_before_publish(archive: &Path, cache: &Path, request: &ArchiveRequirements) {
    assert!(extract_archive(archive, cache, request).is_err());
    assert!(!published_path(cache, &request.artifact_sha256).exists());
}

#[test]
fn rejects_windows_and_cross_platform_path_attacks_before_writing() {
    let unsafe_paths = [
        "/absolute.txt",
        r"C:\drive.txt",
        r"\\server\share\file.txt",
        "../escape.txt",
        "mod/file.txt:stream",
        "mod/CON.txt",
        "mod/trailing.",
        "mod/trailing ",
    ];

    for (index, unsafe_path) in unsafe_paths.iter().enumerate() {
        let temp = TempDir::new().unwrap();
        let archive = temp.path().join(format!("unsafe-{index}.zip"));
        write_zip(
            &archive,
            &[
                Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
                Entry::File(unsafe_path, b"unsafe", CompressionMethod::Stored),
            ],
        );
        let cache = temp.path().join("extract");
        assert_rejected_before_publish(&archive, &cache, &requirements(&archive));
        assert!(!cache.join("temporary").exists());
    }
}

#[test]
fn rejects_links_duplicates_and_prefix_collisions_before_writing() {
    let cases = [
        vec![
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::Symlink("unpublished/link", "../outside"),
        ],
        vec![
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("unpublished/File.txt", b"one", CompressionMethod::Stored),
            Entry::File("UNPUBLISHED/file.TXT", b"two", CompressionMethod::Stored),
        ],
        vec![
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("unpublished/prefix", b"file", CompressionMethod::Stored),
            Entry::File(
                "unpublished/prefix/child",
                b"child",
                CompressionMethod::Stored,
            ),
        ],
    ];

    for (index, entries) in cases.iter().enumerate() {
        let temp = TempDir::new().unwrap();
        let archive = temp.path().join(format!("structure-{index}.zip"));
        write_zip(&archive, entries);
        let cache = temp.path().join("extract");
        assert_rejected_before_publish(&archive, &cache, &requirements(&archive));
        assert!(!cache.join("temporary").exists());
    }
}

#[test]
fn rejects_depth_count_size_and_compression_ratio_limits_before_writing() {
    let cases = [
        (
            vec![
                Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
                Entry::File(
                    "unpublished/deep/path/file",
                    b"x",
                    CompressionMethod::Stored,
                ),
            ],
            ArchiveLimits {
                max_depth: 3,
                ..ArchiveLimits::default()
            },
        ),
        (
            vec![
                Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
                Entry::File("unpublished/data", b"x", CompressionMethod::Stored),
            ],
            ArchiveLimits {
                max_entries: 1,
                ..ArchiveLimits::default()
            },
        ),
        (
            vec![
                Entry::File("mod/setup-mod.tp2", b"x", CompressionMethod::Stored),
                Entry::File("unpublished/large", TP2, CompressionMethod::Stored),
            ],
            ArchiveLimits {
                max_entry_uncompressed_bytes: (TP2.len() - 1) as u64,
                ..ArchiveLimits::default()
            },
        ),
        (
            vec![
                Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
                Entry::File("unpublished/data", DATA, CompressionMethod::Stored),
            ],
            ArchiveLimits {
                max_total_uncompressed_bytes: TP2.len() as u64,
                ..ArchiveLimits::default()
            },
        ),
        (
            vec![
                Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
                Entry::File(
                    "unpublished/bomb",
                    &[0_u8; 32_768],
                    CompressionMethod::Deflated,
                ),
            ],
            ArchiveLimits {
                max_compression_ratio: 2,
                ..ArchiveLimits::default()
            },
        ),
    ];

    for (index, (entries, limits)) in cases.iter().enumerate() {
        let temp = TempDir::new().unwrap();
        let archive = temp.path().join(format!("limit-{index}.zip"));
        write_zip(&archive, entries);
        let mut request = requirements(&archive);
        request.limits = limits.clone();
        let cache = temp.path().join("extract");
        assert_rejected_before_publish(&archive, &cache, &request);
        assert!(!cache.join("temporary").exists());
    }
}

#[test]
fn rejects_encrypted_and_unsupported_entries_before_writing() {
    let temp = TempDir::new().unwrap();
    let encrypted = temp.path().join("encrypted.zip");
    write_zip(
        &encrypted,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::Encrypted("unpublished/encrypted.bin", DATA),
        ],
    );
    let encrypted_cache = temp.path().join("encrypted-cache");
    assert_rejected_before_publish(&encrypted, &encrypted_cache, &requirements(&encrypted));

    let unsupported = temp.path().join("unsupported.zip");
    write_zip(
        &unsupported,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File(
                "unpublished/unsupported.bin",
                DATA,
                CompressionMethod::Stored,
            ),
        ],
    );
    let mut bytes = std::fs::read(&unsupported).unwrap();
    patch_last_compression_method(&mut bytes, 12);
    std::fs::write(&unsupported, bytes).unwrap();
    let unsupported_cache = temp.path().join("unsupported-cache");
    assert_rejected_before_publish(
        &unsupported,
        &unsupported_cache,
        &requirements(&unsupported),
    );
}

fn patch_last_compression_method(bytes: &mut [u8], method: u16) {
    let local = bytes
        .windows(4)
        .enumerate()
        .filter_map(|(index, value)| (value == b"PK\x03\x04").then_some(index))
        .next_back()
        .unwrap();
    bytes[local + 8..local + 10].copy_from_slice(&method.to_le_bytes());

    let central = bytes
        .windows(4)
        .enumerate()
        .filter_map(|(index, value)| (value == b"PK\x01\x02").then_some(index))
        .next_back()
        .unwrap();
    bytes[central + 10..central + 12].copy_from_slice(&method.to_le_bytes());
}

#[test]
fn rejects_missing_or_ambiguous_expected_tp2_roots() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("missing.zip");
    write_zip(
        &missing,
        &[Entry::File(
            "mod/readme.txt",
            b"no tp2",
            CompressionMethod::Stored,
        )],
    );
    assert_rejected_before_publish(
        &missing,
        &temp.path().join("missing-cache"),
        &requirements(&missing),
    );

    let ambiguous = temp.path().join("ambiguous.zip");
    write_zip(
        &ambiguous,
        &[
            Entry::File(
                "wrapper-a/mod/setup-mod.tp2",
                TP2,
                CompressionMethod::Stored,
            ),
            Entry::File(
                "wrapper-b/mod/setup-mod.tp2",
                TP2,
                CompressionMethod::Stored,
            ),
        ],
    );
    assert_rejected_before_publish(
        &ambiguous,
        &temp.path().join("ambiguous-cache"),
        &requirements(&ambiguous),
    );
}

#[test]
fn rejects_undeclared_tp2_inside_publish_root_before_writing() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("undeclared-inside.zip");
    write_zip(
        &archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/tools/live-patch.tp2", TP2, CompressionMethod::Stored),
        ],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);

    let error = extract_archive(&archive, &cache, &request).unwrap_err();

    assert!(matches!(
        error,
        AcquireError::ArchiveLayout(message)
            if message.contains("undeclared TP2 `mod/tools/live-patch.tp2`")
    ));
    assert!(!published_path(&cache, &request.artifact_sha256).exists());
    assert!(!cache.join("temporary").exists());
}

#[test]
fn tp2_publish_root_matching_is_case_insensitive_and_component_bounded() {
    let inside = TempDir::new().unwrap();
    let inside_archive = inside.path().join("inside-case.zip");
    write_zip(
        &inside_archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("MOD/Tools/Live-Patch.TP2", TP2, CompressionMethod::Stored),
        ],
    );
    let inside_cache = inside.path().join("extract");
    let inside_request = requirements(&inside_archive);
    let error = extract_archive(&inside_archive, &inside_cache, &inside_request).unwrap_err();
    assert!(matches!(
        error,
        AcquireError::ArchiveLayout(message)
            if message.contains("undeclared TP2 `MOD/Tools/Live-Patch.TP2`")
    ));
    assert!(!published_path(&inside_cache, &inside_request.artifact_sha256).exists());
    assert!(!inside_cache.join("temporary").exists());

    let sibling = TempDir::new().unwrap();
    let sibling_archive = sibling.path().join("sibling-prefix.zip");
    write_zip(
        &sibling_archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod-live/setup-live.tp2", TP2, CompressionMethod::Stored),
        ],
    );

    let extracted = extract_archive(
        &sibling_archive,
        &sibling.path().join("extract"),
        &requirements(&sibling_archive),
    )
    .unwrap();

    assert!(!extracted.root.join("mod-live/setup-live.tp2").exists());
}

#[test]
fn extracts_and_records_only_files_within_declared_publish_roots() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("publish-roots-only.zip");
    write_zip(
        &archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/data.txt", DATA, CompressionMethod::Stored),
            Entry::File(
                "unpublished/setup-helper.exe",
                b"unused executable",
                CompressionMethod::Stored,
            ),
            Entry::File("unpublished/setup-live.tp2", TP2, CompressionMethod::Stored),
            Entry::File(
                "unpublished/readme.txt",
                b"outside the payload",
                CompressionMethod::Stored,
            ),
        ],
    );

    let extracted = extract_archive(
        &archive,
        &temp.path().join("extract"),
        &requirements(&archive),
    )
    .unwrap();

    assert!(extracted.root.join("mod/setup-mod.tp2").is_file());
    assert!(extracted.root.join("mod/data.txt").is_file());
    assert!(!extracted.root.join("unpublished").exists());
    let marker = std::fs::read_to_string(extracted.root.join(".chriz-bg-extraction.json")).unwrap();
    assert!(marker.contains("\"version\": 2"));
    assert!(!marker.contains("unpublished"));
}

#[test]
fn ignores_the_unreleased_v1_extraction_namespace() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("valid.zip");
    write_zip(
        &archive,
        &[Entry::File(
            "mod/setup-mod.tp2",
            TP2,
            CompressionMethod::Stored,
        )],
    );
    let digest = sha256(&archive);
    let legacy = temp
        .path()
        .join("extract/sha256")
        .join(&digest[..2])
        .join(&digest);
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("untrusted-v1-file"), b"ignore me").unwrap();

    let extracted = extract_archive(
        &archive,
        &temp.path().join("extract"),
        &requirements(&archive),
    )
    .unwrap();

    assert_eq!(
        extracted.root,
        published_path(&temp.path().join("extract"), &digest)
    );
    assert!(legacy.join("untrusted-v1-file").is_file());
}

#[test]
fn cache_hit_rejects_a_matching_tree_and_marker_record_outside_publish_roots() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("valid.zip");
    write_zip(
        &archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/data.txt", DATA, CompressionMethod::Stored),
        ],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    let injected = b"matching outside bytes";
    std::fs::create_dir(extracted.root.join("MOD-LIVE")).unwrap();
    std::fs::write(extracted.root.join("MOD-LIVE/injected.txt"), injected).unwrap();
    edit_extraction_marker(&extracted.root, |marker| {
        marker["files"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "relative_path": "MOD-LIVE/injected.txt",
                "length": injected.len(),
                "sha256": sha256_bytes(injected),
            }));
    });

    let error = extract_archive(&archive, &cache, &request).unwrap_err();

    assert!(matches!(
        error,
        AcquireError::CorruptCache { message, .. }
            if message.contains("outside declared publish roots")
    ));
}

#[test]
fn cache_hit_requires_every_declared_tp2_to_be_a_recorded_regular_file() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("valid.zip");
    write_zip(
        &archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/data.txt", DATA, CompressionMethod::Stored),
        ],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    std::fs::remove_file(extracted.root.join("mod/setup-mod.tp2")).unwrap();
    edit_extraction_marker(&extracted.root, |marker| {
        marker["files"].as_array_mut().unwrap().retain(|record| {
            !record["relative_path"]
                .as_str()
                .unwrap()
                .eq_ignore_ascii_case("mod/setup-mod.tp2")
        });
    });

    let error = extract_archive(&archive, &cache, &request).unwrap_err();

    assert!(matches!(
        error,
        AcquireError::CorruptCache { message, .. }
            if message.contains("expected TP2") && message.contains("setup-mod.tp2")
    ));
}

#[test]
fn cache_hit_requires_every_publish_root_to_own_a_recorded_regular_file() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("multiple-roots.zip");
    write_zip(
        &archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("extra/readme.txt", DATA, CompressionMethod::Stored),
        ],
    );
    let cache = temp.path().join("extract");
    let mut request = requirements(&archive);
    request.expected_roots.push("extra".to_owned());
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    std::fs::remove_file(extracted.root.join("extra/readme.txt")).unwrap();
    std::fs::remove_dir(extracted.root.join("extra")).unwrap();
    edit_extraction_marker(&extracted.root, |marker| {
        marker["files"].as_array_mut().unwrap().retain(|record| {
            !record["relative_path"]
                .as_str()
                .unwrap()
                .eq_ignore_ascii_case("extra/readme.txt")
        });
    });

    let error = extract_archive(&archive, &cache, &request).unwrap_err();

    assert!(matches!(
        error,
        AcquireError::CorruptCache { message, .. }
            if message.contains("publish root") && message.contains("extra")
    ));
}

#[test]
fn cache_hit_root_and_tp2_ownership_is_case_insensitive_but_component_bounded() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("case.zip");
    write_zip(
        &archive,
        &[
            Entry::File("MOD/SETUP-MOD.TP2", TP2, CompressionMethod::Stored),
            Entry::File("MOD/data.txt", DATA, CompressionMethod::Stored),
            Entry::File("mod-live/setup-live.tp2", TP2, CompressionMethod::Stored),
        ],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);

    let first = extract_archive(&archive, &cache, &request).unwrap();
    let second = extract_archive(&archive, &cache, &request).unwrap();

    assert_eq!(first, second);
    assert!(second.root.join("MOD/SETUP-MOD.TP2").is_file());
    assert!(!second.root.join("mod-live").exists());
}

#[test]
fn declared_tp2_requires_exactly_one_publish_root() {
    for (index, roots, tp2_path, entries) in [
        (
            0,
            vec!["mod".to_owned()],
            "live-patch/setup-live.tp2".to_owned(),
            vec![
                Entry::File("mod/data.txt", DATA, CompressionMethod::Stored),
                Entry::File("live-patch/setup-live.tp2", TP2, CompressionMethod::Stored),
            ],
        ),
        (
            1,
            vec!["mod".to_owned(), "mod/lib".to_owned()],
            "mod/lib/setup-mod.tp2".to_owned(),
            vec![Entry::File(
                "mod/lib/setup-mod.tp2",
                TP2,
                CompressionMethod::Stored,
            )],
        ),
    ] {
        let temp = TempDir::new().unwrap();
        let archive = temp.path().join(format!("tp2-owner-{index}.zip"));
        write_zip(&archive, &entries);
        let cache = temp.path().join("extract");
        let request = ArchiveRequirements {
            artifact_sha256: sha256(&archive),
            format: ArchiveFormat::Zip,
            expected_roots: roots,
            expected_tp2_paths: vec![tp2_path],
            limits: ArchiveLimits::default(),
            mode: ArchiveMode::Public,
        };

        let error = extract_archive(&archive, &cache, &request).unwrap_err();

        assert!(
            matches!(
                &error,
                AcquireError::InvalidRequest(message)
                    if message.contains("expected exactly one publish root owner")
            ),
            "unexpected error: {error:?}"
        );
        assert!(!published_path(&cache, &request.artifact_sha256).exists());
        assert!(!cache.join("temporary").exists());
    }
}

#[test]
fn extracts_zip_iemod_and_one_github_wrapper_to_the_exact_layout() {
    for (index, extension) in ["zip", "iemod"].iter().enumerate() {
        let temp = TempDir::new().unwrap();
        let archive = temp.path().join(format!("valid-{index}.{extension}"));
        write_zip(
            &archive,
            &[
                Entry::Directory("mod/"),
                Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Deflated),
                Entry::File("mod/data.txt", DATA, CompressionMethod::Deflated),
            ],
        );
        let extracted = extract_archive(
            &archive,
            &temp.path().join("extract"),
            &requirements(&archive),
        )
        .unwrap();
        assert_eq!(extracted.wrapper_directory, None);
        assert_eq!(
            std::fs::read(extracted.root.join("mod/data.txt")).unwrap(),
            DATA
        );
        assert_eq!(
            std::fs::read(extracted.root.join("mod/setup-mod.tp2")).unwrap(),
            TP2
        );
    }

    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("wrapped.zip");
    write_zip(
        &archive,
        &[
            Entry::Directory("repository-v1/"),
            Entry::File(
                "repository-v1/mod/setup-mod.tp2",
                TP2,
                CompressionMethod::Stored,
            ),
            Entry::File(
                "repository-v1/mod/data.txt",
                DATA,
                CompressionMethod::Stored,
            ),
        ],
    );
    let extracted = extract_archive(
        &archive,
        &temp.path().join("extract"),
        &requirements(&archive),
    )
    .unwrap();
    assert_eq!(
        extracted.wrapper_directory.as_deref(),
        Some("repository-v1")
    );
    assert_eq!(
        std::fs::read(extracted.root.join("mod/data.txt")).unwrap(),
        DATA
    );
}

#[test]
fn public_mode_rejects_zero_hashes_and_self_extracting_executables() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("valid.zip");
    write_zip(
        &archive,
        &[Entry::File(
            "mod/setup-mod.tp2",
            TP2,
            CompressionMethod::Stored,
        )],
    );
    let mut zero = requirements(&archive);
    zero.artifact_sha256 = "0".repeat(64);
    assert!(extract_archive(&archive, &temp.path().join("zero"), &zero).is_err());

    let sfx = temp.path().join("self-extracting.zip");
    let mut prefixed = b"MZ self extracting executable stub".to_vec();
    prefixed.extend(std::fs::read(&archive).unwrap());
    std::fs::write(&sfx, prefixed).unwrap();
    assert_rejected_before_publish(&sfx, &temp.path().join("sfx"), &requirements(&sfx));
}

#[test]
fn rejects_a_content_addressed_extraction_that_gained_unrecorded_files() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("valid.zip");
    write_zip(
        &archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/data.txt", DATA, CompressionMethod::Stored),
        ],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    std::fs::write(extracted.root.join("mod/injected.txt"), b"not verified").unwrap();

    assert!(extract_archive(&archive, &cache, &request).is_err());
}
