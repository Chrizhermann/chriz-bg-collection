use std::path::Path;

use bg_engine::acquire::{provide_manual_archive, AcquireError};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[test]
fn explicit_manual_selection_accepts_only_the_matching_regular_archive() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("selected.iemod");
    let bytes = b"user-selected archive bytes";
    std::fs::write(&path, bytes).unwrap();

    let verified = provide_manual_archive(&path, &sha256(bytes)).unwrap();

    assert_eq!(verified.path, Path::new(&path));
    assert_eq!(verified.sha256, sha256(bytes));
    assert_eq!(verified.length, bytes.len() as u64);
}

#[test]
fn explicit_manual_selection_rejects_a_hash_mismatch() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("selected.zip");
    std::fs::write(&path, b"unexpected archive").unwrap();

    assert!(matches!(
        provide_manual_archive(&path, &"11".repeat(32)),
        Err(AcquireError::HashMismatch { .. })
    ));
}
