use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use bg_engine::acquire::{
    extract_archive, ArchiveFormat, ArchiveLimits, ArchiveMode, ArchiveRequirements,
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
    cache.join("sha256").join(&digest[..2]).join(digest)
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
            Entry::Symlink("mod/link", "../outside"),
        ],
        vec![
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/File.txt", b"one", CompressionMethod::Stored),
            Entry::File("MOD/file.TXT", b"two", CompressionMethod::Stored),
        ],
        vec![
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/prefix", b"file", CompressionMethod::Stored),
            Entry::File("mod/prefix/child", b"child", CompressionMethod::Stored),
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
                Entry::File("mod/deep/path/file", b"x", CompressionMethod::Stored),
            ],
            ArchiveLimits {
                max_depth: 3,
                ..ArchiveLimits::default()
            },
        ),
        (
            vec![
                Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
                Entry::File("mod/data", b"x", CompressionMethod::Stored),
            ],
            ArchiveLimits {
                max_entries: 1,
                ..ArchiveLimits::default()
            },
        ),
        (
            vec![Entry::File(
                "mod/setup-mod.tp2",
                TP2,
                CompressionMethod::Stored,
            )],
            ArchiveLimits {
                max_entry_uncompressed_bytes: (TP2.len() - 1) as u64,
                ..ArchiveLimits::default()
            },
        ),
        (
            vec![
                Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
                Entry::File("mod/data", DATA, CompressionMethod::Stored),
            ],
            ArchiveLimits {
                max_total_uncompressed_bytes: TP2.len() as u64,
                ..ArchiveLimits::default()
            },
        ),
        (
            vec![
                Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
                Entry::File("mod/bomb", &[0_u8; 32_768], CompressionMethod::Deflated),
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
    write_zip(&encrypted, &[Entry::Encrypted("mod/setup-mod.tp2", TP2)]);
    let encrypted_cache = temp.path().join("encrypted-cache");
    assert_rejected_before_publish(&encrypted, &encrypted_cache, &requirements(&encrypted));

    let unsupported = temp.path().join("unsupported.zip");
    write_zip(
        &unsupported,
        &[Entry::File(
            "mod/setup-mod.tp2",
            TP2,
            CompressionMethod::Stored,
        )],
    );
    let mut bytes = std::fs::read(&unsupported).unwrap();
    patch_compression_method(&mut bytes, 12);
    std::fs::write(&unsupported, bytes).unwrap();
    let unsupported_cache = temp.path().join("unsupported-cache");
    assert_rejected_before_publish(
        &unsupported,
        &unsupported_cache,
        &requirements(&unsupported),
    );
}

fn patch_compression_method(bytes: &mut [u8], method: u16) {
    for index in 0..bytes.len().saturating_sub(12) {
        if bytes[index..].starts_with(b"PK\x03\x04") {
            bytes[index + 8..index + 10].copy_from_slice(&method.to_le_bytes());
        } else if bytes[index..].starts_with(b"PK\x01\x02") {
            bytes[index + 10..index + 12].copy_from_slice(&method.to_le_bytes());
        }
    }
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
