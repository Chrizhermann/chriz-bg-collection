use std::fs;
use std::path::Path;

use bg_engine::acquire::{
    extract_archive, AcquireError, ArchiveFormat, ArchiveLimits, ArchiveMode, ArchiveRequirements,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn rar4_header(kind: u8, flags: u16, body: &[u8]) -> Vec<u8> {
    let size = u16::try_from(7 + body.len()).unwrap();
    let mut protected = vec![kind];
    protected.extend_from_slice(&flags.to_le_bytes());
    protected.extend_from_slice(&size.to_le_bytes());
    protected.extend_from_slice(body);
    let mut header = (crc32(&protected) as u16).to_le_bytes().to_vec();
    header.extend(protected);
    header
}

fn rar4_stored_sfx(entries: &[(&str, &[u8], u16)]) -> Vec<u8> {
    let mut bytes = vec![b'M', b'Z'];
    bytes.resize(128, 0);
    bytes.extend_from_slice(b"Rar!\x1a\x07\x00");
    bytes.extend(rar4_header(0x73, 0, &[0; 6]));
    for (name, payload, extra_flags) in entries {
        let mut body = Vec::new();
        body.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        body.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        body.push(2); // Windows host OS.
        body.extend_from_slice(&crc32(payload).to_le_bytes());
        body.extend_from_slice(&0_u32.to_le_bytes());
        body.push(20);
        body.push(0x30); // Stored.
        body.extend_from_slice(&(name.len() as u16).to_le_bytes());
        body.extend_from_slice(&0x20_u32.to_le_bytes());
        body.extend_from_slice(name.as_bytes());
        bytes.extend(rar4_header(0x74, 0x8000 | extra_flags, &body));
        bytes.extend_from_slice(payload);
    }
    bytes.extend(rar4_header(0x7b, 0, &[]));
    bytes
}

fn requirements(path: &Path, roots: &[&str], tp2s: &[&str]) -> ArchiveRequirements {
    let bytes = fs::read(path).unwrap();
    ArchiveRequirements {
        artifact_sha256: hex::encode(Sha256::digest(bytes)),
        format: ArchiveFormat::SelfExtractingRar,
        expected_roots: roots.iter().map(|value| (*value).to_owned()).collect(),
        expected_tp2_paths: tp2s.iter().map(|value| (*value).to_owned()).collect(),
        limits: ArchiveLimits::default(),
        mode: ArchiveMode::Public,
    }
}

#[test]
fn extracts_a_validated_rar4_self_extracting_archive_without_running_it() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("evandra.exe");
    fs::write(
        &archive,
        rar4_stored_sfx(&[
            ("evandra/setup-evandra.tp2", b"BACKUP ~evandra/backup~", 0),
            ("evandra/data/payload.txt", b"payload", 0),
            ("setup-evandra.exe", b"must not be published", 0),
        ]),
    )
    .unwrap();

    let extracted = extract_archive(
        &archive,
        &temp.path().join("cache"),
        &requirements(&archive, &["evandra"], &["evandra/setup-evandra.tp2"]),
    )
    .unwrap();

    assert_eq!(
        fs::read(extracted.root.join("evandra/data/payload.txt")).unwrap(),
        b"payload"
    );
    assert!(!extracted.root.join("setup-evandra.exe").exists());
}

#[test]
fn rejects_unsafe_rar_paths_before_writing_any_entry() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("unsafe.exe");
    fs::write(
        &archive,
        rar4_stored_sfx(&[
            ("evandra/setup-evandra.tp2", b"tp2", 0),
            ("../escape.txt", b"escape", 0),
        ]),
    )
    .unwrap();
    let cache = temp.path().join("cache");

    assert!(matches!(
        extract_archive(
            &archive,
            &cache,
            &requirements(&archive, &["evandra"], &["evandra/setup-evandra.tp2"]),
        ),
        Err(AcquireError::UnsafeArchiveEntry { .. })
    ));
    assert!(!temp.path().join("escape.txt").exists());
    assert!(!cache.join("temporary").exists());
}

#[test]
fn rejects_a_rar_without_an_explicit_mz_self_extractor_header() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("not-an-sfx.exe");
    let mut bytes = rar4_stored_sfx(&[("evandra/setup-evandra.tp2", b"tp2", 0)]);
    bytes[..2].copy_from_slice(b"XX");
    fs::write(&archive, bytes).unwrap();

    assert!(matches!(
        extract_archive(
            &archive,
            &temp.path().join("cache"),
            &requirements(&archive, &["evandra"], &["evandra/setup-evandra.tp2"]),
        ),
        Err(AcquireError::PublicArchiveRejected(_))
    ));
}

#[test]
fn extracts_real_evandra_package_when_explicitly_supplied() {
    let Some(path) = std::env::var_os("CHRIZ_TEST_EVANDRA_SFX") else {
        return;
    };
    let archive = std::path::PathBuf::from(path);
    let temporary_cache;
    let cache = if let Some(path) = std::env::var_os("CHRIZ_TEST_EVANDRA_CACHE") {
        std::path::PathBuf::from(path)
    } else {
        temporary_cache = TempDir::new().unwrap();
        temporary_cache.path().join("cache")
    };
    let result = extract_archive(
        &archive,
        &cache,
        &requirements(&archive, &["Evandra"], &["Evandra/Setup-Evandra.tp2"]),
    )
    .unwrap();
    assert!(result.root.join("Evandra/Setup-Evandra.tp2").is_file());
    let mut file_count = 0_usize;
    let mut total_bytes = 0_u64;
    for entry in walkdir::WalkDir::new(&result.root) {
        let entry = entry.unwrap();
        if entry.file_type().is_file()
            && entry.file_name().to_string_lossy() != ".chriz-bg-extraction.json"
        {
            file_count += 1;
            total_bytes += entry.metadata().unwrap().len();
        }
    }
    let tp2 = fs::read(result.root.join("evandra/setup-evandra.tp2")).unwrap();
    println!(
        "evandra root={} extracted_files={file_count} extracted_bytes={total_bytes} tp2_sha256={}",
        result.root.display(),
        hex::encode(Sha256::digest(tp2))
    );
}
