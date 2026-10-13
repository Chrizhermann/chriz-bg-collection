use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use bg_engine::acquire::{
    extract_archive, extract_archive_reporting_repair, AcquireError, ArchiveFormat, ArchiveLimits,
    ArchiveMode, ArchiveRequirements, ExtractedArtifact,
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

fn published_archive_parent(cache: &Path, digest: &str) -> PathBuf {
    cache.join("sha256-v3").join(&digest[..2]).join(digest)
}

fn assert_rejected_before_publish(archive: &Path, cache: &Path, request: &ArchiveRequirements) {
    assert!(extract_archive(archive, cache, request).is_err());
    assert!(!published_archive_parent(cache, &request.artifact_sha256).exists());
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

        // A successful extraction under permissive limits must not bypass any later
        // stricter limit, including limits violated only by unpublished archive entries.
        let warm_cache = temp.path().join("warm-extract");
        let permissive = requirements(&archive);
        let previous = extract_archive(&archive, &warm_cache, &permissive).unwrap();
        assert!(matches!(
            extract_archive(&archive, &warm_cache, &request),
            Err(AcquireError::ArchiveLimitExceeded(_))
        ));
        assert_eq!(
            previous,
            extract_archive(&archive, &warm_cache, &permissive).unwrap()
        );
        assert_eq!(
            std::fs::read_dir(previous.root.parent().unwrap())
                .unwrap()
                .count(),
            1
        );
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
    assert!(!published_archive_parent(&cache, &request.artifact_sha256).exists());
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
    assert!(!published_archive_parent(&inside_cache, &inside_request.artifact_sha256).exists());
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
    assert!(marker.contains("\"version\": 3"));
    assert!(!marker.contains("unpublished"));
}

#[test]
fn leaves_both_legacy_digest_only_extraction_namespaces_untouched() {
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
    let legacy_roots = ["sha256", "sha256-v2"].map(|namespace| {
        let legacy = temp
            .path()
            .join("extract")
            .join(namespace)
            .join(&digest[..2])
            .join(&digest);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("legacy-file"), b"leave me untouched").unwrap();
        legacy
    });

    let extracted = extract_archive(
        &archive,
        &temp.path().join("extract"),
        &requirements(&archive),
    )
    .unwrap();

    assert_eq!(
        extracted.root.parent().unwrap(),
        published_archive_parent(&temp.path().join("extract"), &digest)
    );
    assert_eq!(
        extracted.root.file_name().unwrap().to_str().unwrap(),
        extracted.extraction_contract_sha256
    );
    assert!(extracted.root.join("mod/setup-mod.tp2").is_file());
    for legacy in legacy_roots {
        assert_eq!(
            std::fs::read(legacy.join("legacy-file")).unwrap(),
            b"leave me untouched"
        );
        assert_eq!(std::fs::read_dir(legacy).unwrap().count(), 1);
    }
}

/// Extract again and require that the damaged published entry was replaced for `reason`.
/// The rebuilt entry must equal a fresh extraction and must be a plain cache hit afterwards.
fn assert_rebuilt(
    archive: &Path,
    cache: &Path,
    request: &ArchiveRequirements,
    original: &ExtractedArtifact,
    reason: &[&str],
) {
    let (rebuilt, repair) = extract_archive_reporting_repair(archive, cache, request).unwrap();
    let repair = repair.expect("damaged extraction must be reported as repaired");
    assert!(
        reason.iter().all(|part| repair.reason.contains(part)),
        "unexpected repair reason: {}",
        repair.reason
    );
    assert!(
        !repair.reason.contains(&*cache.to_string_lossy()),
        "repair reason must not contain the cache location: {}",
        repair.reason
    );
    assert_eq!(&rebuilt, original);
    assert_eq!(
        extract_archive_reporting_repair(archive, cache, request).unwrap(),
        (rebuilt, None)
    );
    let damaged = cache.join("damaged");
    assert!(!damaged.exists() || std::fs::read_dir(damaged).unwrap().next().is_none());
}

#[test]
fn cache_hit_rebuilds_a_tree_and_marker_record_outside_publish_roots() {
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

    assert_rebuilt(
        &archive,
        &cache,
        &request,
        &extracted,
        &["outside declared publish roots"],
    );
    assert!(!extracted.root.join("MOD-LIVE").exists());
    let marker = std::fs::read_to_string(extracted.root.join(".chriz-bg-extraction.json")).unwrap();
    assert!(!marker.contains("MOD-LIVE"));
}

#[test]
fn cache_hit_rebuilds_a_missing_declared_tp2() {
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

    assert_rebuilt(
        &archive,
        &cache,
        &request,
        &extracted,
        &["expected TP2", "setup-mod.tp2"],
    );
    assert_eq!(
        std::fs::read(extracted.root.join("mod/setup-mod.tp2")).unwrap(),
        TP2
    );
}

#[test]
fn cache_hit_rebuilds_a_publish_root_without_a_recorded_regular_file() {
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

    assert_rebuilt(
        &archive,
        &cache,
        &request,
        &extracted,
        &["publish root", "extra"],
    );
    assert_eq!(
        std::fs::read(extracted.root.join("extra/readme.txt")).unwrap(),
        DATA
    );
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

fn write_multi_root_archive(path: &Path) -> (ArchiveRequirements, ArchiveRequirements) {
    write_zip(
        path,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("extra/setup-extra.tp2", TP2, CompressionMethod::Stored),
            Entry::File("gui/setup-gui.tp2", TP2, CompressionMethod::Stored),
        ],
    );
    let mut two_roots = requirements(path);
    two_roots.expected_roots.push("extra".to_owned());
    two_roots
        .expected_tp2_paths
        .push("extra/setup-extra.tp2".to_owned());
    let mut three_roots = two_roots.clone();
    three_roots.expected_roots.push("gui".to_owned());
    three_roots
        .expected_tp2_paths
        .push("gui/setup-gui.tp2".to_owned());
    (two_roots, three_roots)
}

#[test]
fn same_archive_layouts_have_distinct_stable_cache_entries() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("multiple-layouts.zip");
    let cache = temp.path().join("extract");
    let (two_roots, three_roots) = write_multi_root_archive(&archive);

    let first = extract_archive(&archive, &cache, &two_roots).unwrap();
    let expanded = extract_archive(&archive, &cache, &three_roots).unwrap();
    let reused = extract_archive(&archive, &cache, &two_roots).unwrap();

    assert_ne!(first.root, expanded.root);
    assert_eq!(first, reused);
    assert!(!first.root.join("gui").exists());
    assert!(expanded.root.join("gui/setup-gui.tp2").is_file());
    assert_eq!(first.root.parent(), expanded.root.parent());

    let mut equivalent = two_roots.clone();
    equivalent.expected_roots.reverse();
    equivalent.expected_tp2_paths.reverse();
    for path in equivalent
        .expected_roots
        .iter_mut()
        .chain(&mut equivalent.expected_tp2_paths)
    {
        *path = path.replace('/', "\\").to_ascii_uppercase();
    }
    assert_eq!(
        first,
        extract_archive(&archive, &cache, &equivalent).unwrap()
    );
}

#[test]
fn concurrent_layouts_and_identical_requests_publish_independent_verified_entries() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("concurrent-layouts.zip");
    let cache = temp.path().join("extract");
    let (two_roots, three_roots) = write_multi_root_archive(&archive);
    let requests = [&two_roots, &three_roots, &two_roots, &three_roots];
    let barrier = std::sync::Barrier::new(requests.len());
    let results = std::thread::scope(|scope| {
        let workers = requests
            .into_iter()
            .map(|request| {
                let (archive, cache, barrier) = (&archive, &cache, &barrier);
                scope.spawn(move || {
                    barrier.wait();
                    extract_archive(archive, cache, request)
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap().unwrap())
            .collect::<Vec<_>>()
    });

    assert_eq!(results[0], results[2]);
    assert_eq!(results[1], results[3]);
    assert_ne!(results[0].root, results[1].root);
    for (result, request) in results.iter().zip(requests) {
        assert_eq!(*result, extract_archive(&archive, &cache, request).unwrap());
    }
    assert_eq!(
        std::fs::read_dir(cache.join("temporary")).unwrap().count(),
        0
    );
}

#[test]
fn cache_contract_separates_format_mode_and_limits() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("contract.zip");
    let cache = temp.path().join("extract");
    let (original, _) = write_multi_root_archive(&archive);
    let first = extract_archive(&archive, &cache, &original).unwrap();
    let mut changed_format = original.clone();
    changed_format.format = ArchiveFormat::Iemod;
    let mut changed_mode = original.clone();
    changed_mode.mode = ArchiveMode::Authoring;
    changed_mode.artifact_sha256 = "0".repeat(64);
    let mut changed_limits = original.clone();
    changed_limits.limits.max_entries += 1;
    let mut roots = std::collections::BTreeSet::from([first.root.clone()]);
    for request in [changed_format, changed_mode, changed_limits] {
        let extracted = extract_archive(&archive, &cache, &request).unwrap();
        assert!(roots.insert(extracted.root.clone()));
        assert_eq!(
            extracted,
            extract_archive(&archive, &cache, &request).unwrap()
        );
    }

    let mut wrong_format = original.clone();
    wrong_format.format = ArchiveFormat::SelfExtractingRar;
    assert!(matches!(
        extract_archive(&archive, &cache, &wrong_format),
        Err(AcquireError::PublicArchiveRejected(_))
    ));
    let mut unpinned_public = original.clone();
    unpinned_public.artifact_sha256 = "0".repeat(64);
    assert!(matches!(
        extract_archive(&archive, &cache, &unpinned_public),
        Err(AcquireError::PublicArchiveRejected(_))
    ));
}

#[test]
fn changed_tp2_contract_is_checked_against_the_archive() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("tp2-contract.zip");
    let cache = temp.path().join("extract");
    let (original, _) = write_multi_root_archive(&archive);
    let first = extract_archive(&archive, &cache, &original).unwrap();
    let mut changed = original.clone();
    changed
        .expected_tp2_paths
        .push("mod/missing.tp2".to_owned());

    assert!(matches!(
        extract_archive(&archive, &cache, &changed),
        Err(AcquireError::ArchiveLayout(_))
    ));
    assert_eq!(first, extract_archive(&archive, &cache, &original).unwrap());
}

#[test]
fn cache_hit_never_uses_a_modified_payload_or_contract_marker() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("tampered.zip");
    let (request, _) = write_multi_root_archive(&archive);
    let payload_cache = temp.path().join("payload-cache");
    let extracted = extract_archive(&archive, &payload_cache, &request).unwrap();
    let payload = extracted.root.join("mod/setup-mod.tp2");
    std::fs::write(&payload, b"modified payload").unwrap();
    assert_rebuilt(
        &archive,
        &payload_cache,
        &request,
        &extracted,
        &["setup-mod.tp2", "no longer matches"],
    );
    assert_eq!(std::fs::read(payload).unwrap(), TP2);

    let marker_cache = temp.path().join("marker-cache");
    let extracted = extract_archive(&archive, &marker_cache, &request).unwrap();
    edit_extraction_marker(&extracted.root, |marker| {
        marker["contract_sha256"] = serde_json::json!("0".repeat(64));
    });
    assert_rebuilt(
        &archive,
        &marker_cache,
        &request,
        &extracted,
        &["does not match the requested archive layout"],
    );
}

/// The reported case: the published directory survived a partial manual deletion, but its
/// marker and some payload files did not.
#[test]
fn cache_hit_rebuilds_a_partially_deleted_extraction_without_its_marker() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("partially-deleted.zip");
    write_zip(
        &archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/lib/deep/data.txt", DATA, CompressionMethod::Stored),
        ],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    std::fs::remove_file(extracted.root.join(".chriz-bg-extraction.json")).unwrap();
    std::fs::remove_file(extracted.root.join("mod/setup-mod.tp2")).unwrap();

    assert_rebuilt(
        &archive,
        &cache,
        &request,
        &extracted,
        &["inspect extraction marker", ".chriz-bg-extraction.json"],
    );
    assert_eq!(
        std::fs::read(extracted.root.join("mod/setup-mod.tp2")).unwrap(),
        TP2
    );
    assert_eq!(
        std::fs::read(extracted.root.join("mod/lib/deep/data.txt")).unwrap(),
        DATA
    );
}

#[test]
fn cache_hit_rebuilds_an_unreadable_marker_and_a_root_that_is_not_a_directory() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("unreadable-marker.zip");
    write_zip(
        &archive,
        &[Entry::File(
            "mod/setup-mod.tp2",
            TP2,
            CompressionMethod::Stored,
        )],
    );
    let request = requirements(&archive);

    let marker_cache = temp.path().join("marker-cache");
    let extracted = extract_archive(&archive, &marker_cache, &request).unwrap();
    std::fs::write(
        extracted.root.join(".chriz-bg-extraction.json"),
        b"{ truncated",
    )
    .unwrap();
    assert_rebuilt(
        &archive,
        &marker_cache,
        &request,
        &extracted,
        &["unreadable cache record", ".chriz-bg-extraction.json"],
    );

    let file_cache = temp.path().join("file-cache");
    let extracted = extract_archive(&archive, &file_cache, &request).unwrap();
    std::fs::remove_dir_all(&extracted.root).unwrap();
    std::fs::write(&extracted.root, b"not a directory").unwrap();
    assert_rebuilt(
        &archive,
        &file_cache,
        &request,
        &extracted,
        &["not a regular directory"],
    );
    assert!(extracted.root.is_dir());
}

/// A file another program holds open cannot be proven damaged, so the entry stays in place
/// and the step fails instead of discarding something that may still be in use.
#[cfg(windows)]
#[test]
fn cache_hit_keeps_an_entry_whose_files_cannot_be_read() {
    use std::os::windows::fs::OpenOptionsExt;

    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("locked.zip");
    write_zip(
        &archive,
        &[Entry::File(
            "mod/setup-mod.tp2",
            TP2,
            CompressionMethod::Stored,
        )],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    let payload = extracted.root.join("mod/setup-mod.tp2");
    let exclusive = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&payload)
        .unwrap();

    let error = extract_archive_reporting_repair(&archive, &cache, &request).unwrap_err();

    assert!(matches!(error, AcquireError::Io { .. }), "{error:?}");
    drop(exclusive);
    assert!(!cache.join("damaged").exists());
    assert_eq!(std::fs::read(&payload).unwrap(), TP2);
    assert_eq!(
        extract_archive_reporting_repair(&archive, &cache, &request).unwrap(),
        (extracted, None)
    );
}

/// A marker that cannot be read at the moment, here because another program locks it, is not
/// proof of damage either, even though it could be opened.
#[cfg(windows)]
#[test]
fn cache_hit_keeps_an_entry_whose_marker_cannot_be_read() {
    use fs4::fs_std::FileExt;

    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("locked-marker.zip");
    write_zip(
        &archive,
        &[Entry::File(
            "mod/setup-mod.tp2",
            TP2,
            CompressionMethod::Stored,
        )],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    let marker = File::open(extracted.root.join(".chriz-bg-extraction.json")).unwrap();
    marker.lock_exclusive().unwrap();

    let error = extract_archive_reporting_repair(&archive, &cache, &request).unwrap_err();

    assert!(
        matches!(
            error,
            AcquireError::Io {
                action: "read extraction marker",
                ..
            }
        ),
        "{error:?}"
    );
    FileExt::unlock(&marker).unwrap();
    drop(marker);
    assert!(!cache.join("damaged").exists());
    assert_eq!(
        extract_archive_reporting_repair(&archive, &cache, &request).unwrap(),
        (extracted, None)
    );
}

/// Denies everyone the right to list a directory until dropped, as an access-control change
/// or a security product might. The owner can always restore it.
#[cfg(windows)]
struct DenyListing<'a>(&'a Path);

#[cfg(windows)]
impl<'a> DenyListing<'a> {
    fn new(directory: &'a Path) -> Self {
        let output = std::process::Command::new("icacls")
            .arg(directory)
            .args(["/deny", "*S-1-1-0:(RD)"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        Self(directory)
    }
}

#[cfg(windows)]
impl Drop for DenyListing<'_> {
    fn drop(&mut self) {
        let _ = std::process::Command::new("icacls")
            .arg(self.0)
            .args(["/remove:d", "*S-1-1-0"])
            .output();
    }
}

/// A directory that cannot be listed at the moment is not proof of damage: the entry is kept
/// and the step fails until the directory can be read again.
#[cfg(windows)]
#[test]
fn cache_hit_keeps_an_entry_whose_directories_cannot_be_listed() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("unlisted.zip");
    write_zip(
        &archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/lib/deep/data.txt", DATA, CompressionMethod::Stored),
        ],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    let library = extracted.root.join("mod/lib");

    let denied = DenyListing::new(&library);
    let error = extract_archive_reporting_repair(&archive, &cache, &request).unwrap_err();
    drop(denied);

    assert!(
        matches!(
            &error,
            AcquireError::Io {
                action: "list published extraction",
                source,
                ..
            } if source.kind() == std::io::ErrorKind::PermissionDenied
        ),
        "{error:?}"
    );
    assert!(!cache.join("damaged").exists());
    assert_eq!(std::fs::read(library.join("deep/data.txt")).unwrap(), DATA);
    assert_eq!(
        extract_archive_reporting_repair(&archive, &cache, &request).unwrap(),
        (extracted, None)
    );
}

/// A link inside a published extraction makes it damaged. Setting it aside removes the link
/// itself and never what it points to, and the reason names it relative to the extraction.
#[cfg(windows)]
#[test]
fn cache_hit_rebuilds_an_entry_with_a_directory_link_without_following_it() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("linked.zip");
    write_zip(
        &archive,
        &[Entry::File(
            "mod/setup-mod.tp2",
            TP2,
            CompressionMethod::Stored,
        )],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    let outside = temp.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("keep.txt"), b"outside the cache").unwrap();
    // A junction needs no privilege, unlike a symbolic link.
    let output = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(extracted.root.join("mod").join("linked"))
        .arg(&outside)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_rebuilt(
        &archive,
        &cache,
        &request,
        &extracted,
        &["non-regular extracted path `mod/linked`"],
    );
    assert!(!extracted.root.join("mod/linked").exists());
    assert_eq!(
        std::fs::read(outside.join("keep.txt")).unwrap(),
        b"outside the cache"
    );
}

/// The lock that serializes validation, repair and publication of one extraction identity.
fn extraction_lock(cache: &Path, extracted: &ExtractedArtifact) -> File {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(cache.join("locks").join(format!(
            "{}.{}.lock",
            extracted.artifact_sha256, extracted.extraction_contract_sha256
        )))
        .unwrap()
}

/// A request that finds an entry damaged validates it again under the identity lock. While
/// another request holds that lock to republish the entry, this one waits and then uses the
/// replacement; it never sets aside an entry it has not validated.
#[test]
fn a_damaged_entry_is_validated_again_under_its_lock_before_it_is_set_aside() {
    use fs4::fs_std::FileExt;

    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("serialized-repair.zip");
    write_zip(
        &archive,
        &[Entry::File(
            "mod/setup-mod.tp2",
            TP2,
            CompressionMethod::Stored,
        )],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    let marker = extracted.root.join(".chriz-bg-extraction.json");
    let held_marker = temp.path().join("held-marker.json");
    std::fs::rename(&marker, &held_marker).unwrap();

    let lock = extraction_lock(&cache, &extracted);
    lock.lock_exclusive().unwrap();
    std::thread::scope(|scope| {
        let waiting = scope.spawn(|| extract_archive_reporting_repair(&archive, &cache, &request));
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(!waiting.is_finished(), "the request must wait for the lock");
        assert!(extracted.root.join("mod/setup-mod.tp2").is_file());
        assert!(!cache.join("damaged").exists());

        // The lock holder republishes a valid entry before releasing the lock.
        std::fs::rename(&held_marker, &marker).unwrap();
        FileExt::unlock(&lock).unwrap();
        assert_eq!(waiting.join().unwrap().unwrap(), (extracted.clone(), None));
    });
    assert!(!cache.join("damaged").exists());
}

#[test]
fn concurrent_requests_repair_a_damaged_entry_exactly_once() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("concurrent-repair.zip");
    write_zip(
        &archive,
        &[
            Entry::File("mod/setup-mod.tp2", TP2, CompressionMethod::Stored),
            Entry::File("mod/lib/deep/data.txt", DATA, CompressionMethod::Stored),
        ],
    );
    let cache = temp.path().join("extract");
    let request = requirements(&archive);
    let extracted = extract_archive(&archive, &cache, &request).unwrap();
    std::fs::remove_file(extracted.root.join(".chriz-bg-extraction.json")).unwrap();
    std::fs::remove_file(extracted.root.join("mod/setup-mod.tp2")).unwrap();

    let barrier = std::sync::Barrier::new(4);
    let results = std::thread::scope(|scope| {
        let workers = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    extract_archive_reporting_repair(&archive, &cache, &request)
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(results.iter().all(|(artifact, _)| *artifact == extracted));
    let repairs = results
        .iter()
        .filter_map(|(_, repair)| repair.as_ref())
        .collect::<Vec<_>>();
    assert_eq!(repairs.len(), 1, "{repairs:?}");
    assert!(repairs[0].reason.contains("inspect extraction marker"));
    assert_eq!(
        extract_archive_reporting_repair(&archive, &cache, &request).unwrap(),
        (extracted, None)
    );
    let damaged = cache.join("damaged");
    assert!(!damaged.exists() || std::fs::read_dir(damaged).unwrap().next().is_none());
}

#[test]
fn populated_cache_still_rejects_noncanonical_duplicate_and_ambiguous_requests() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("invalid-contracts.zip");
    let cache = temp.path().join("extract");
    let (original, _) = write_multi_root_archive(&archive);
    let first = extract_archive(&archive, &cache, &original).unwrap();
    let mut duplicate_root = original.clone();
    duplicate_root.expected_roots.push("MOD".to_owned());
    let mut duplicate_tp2 = original.clone();
    duplicate_tp2
        .expected_tp2_paths
        .push("MOD\\SETUP-MOD.TP2".to_owned());
    let mut ambiguous_owner = original.clone();
    ambiguous_owner
        .expected_roots
        .push("mod/setup-mod.tp2".to_owned());

    for invalid in [duplicate_root, duplicate_tp2, ambiguous_owner] {
        assert!(matches!(
            extract_archive(&archive, &cache, &invalid),
            Err(AcquireError::InvalidRequest(_))
        ));
    }
    assert_eq!(first, extract_archive(&archive, &cache, &original).unwrap());
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
        assert!(!published_archive_parent(&cache, &request.artifact_sha256).exists());
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
fn rebuilds_a_content_addressed_extraction_that_gained_unrecorded_files() {
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

    assert_rebuilt(
        &archive,
        &cache,
        &request,
        &extracted,
        &["unrecorded extracted file", "mod/injected.txt"],
    );
    assert!(!extracted.root.join("mod/injected.txt").exists());
}
