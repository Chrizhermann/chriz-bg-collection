use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use bg_engine::acquire::{AcquireError, ArtifactCache, CacheDisposition, DownloadRequest};
use bg_engine::events::ChannelSink;
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use tiny_http::{Header, Response, Server, StatusCode};

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn request(id: &str, url: String, bytes: &[u8]) -> DownloadRequest {
    DownloadRequest {
        request_id: id.to_owned(),
        url,
        expected_length: bytes.len() as u64,
        expected_sha256: sha256(bytes),
        redirect_hosts: Vec::new(),
        max_attempts: 1,
    }
}

fn one_shot_server(bytes: &'static [u8]) -> (String, thread::JoinHandle<()>) {
    let server = Server::http("127.0.0.1:0").unwrap();
    let address = server.server_addr().to_ip().unwrap();
    let handle = thread::spawn(move || {
        let request = server
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .expect("download request");
        let response = Response::new(
            StatusCode(200),
            vec![Header::from_bytes("ETag", "\"cache-v1\"").unwrap()],
            Cursor::new(bytes),
            Some(bytes.len()),
            None,
        );
        request.respond(response).unwrap();
    });
    (format!("http://{address}/archive"), handle)
}

#[test]
fn a_cache_hit_is_rehashed_and_does_not_touch_the_network() {
    let bytes = b"cacheable artifact";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let (url, server) = one_shot_server(bytes);
    let download = request("first", url, bytes);
    let (sink, _) = ChannelSink::unbounded();

    let first = cache.acquire(&download, &sink).unwrap();
    server.join().unwrap();
    let second = cache
        .acquire(
            &DownloadRequest {
                request_id: "second".to_owned(),
                url: "https://network-must-not-be-used.invalid/archive".to_owned(),
                ..download
            },
            &sink,
        )
        .unwrap();

    assert_eq!(first.disposition, CacheDisposition::Downloaded);
    assert_eq!(second.disposition, CacheDisposition::Hit);
    assert_eq!(first.archive_path, second.archive_path);
    assert_eq!(std::fs::read(second.archive_path).unwrap(), bytes);
}

/// Download once, damage the published entry with `damage`, then acquire again from a fresh
/// server. The damaged entry must be set aside, downloaded anew and reported for `reason`.
fn assert_damaged_entry_is_downloaded_again(
    name: &str,
    damage: impl FnOnce(&std::path::Path, &std::path::Path),
    reason: &[&str],
) {
    static BYTES: &[u8] = b"artifact that is damaged after publication";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let (url, server) = one_shot_server(BYTES);
    let first = cache
        .acquire(&request(&format!("{name}-first"), url, BYTES), &sink)
        .unwrap();
    server.join().unwrap();
    damage(&first.archive_path, &first.metadata_path);

    let (url, server) = one_shot_server(BYTES);
    let second = cache
        .acquire(&request(&format!("{name}-second"), url, BYTES), &sink)
        .unwrap();
    server.join().unwrap();

    let repair = second.repaired.expect("damaged entry must be reported");
    assert!(
        reason.iter().all(|part| repair.reason.contains(part)),
        "unexpected repair reason: {}",
        repair.reason
    );
    assert_eq!(second.disposition, CacheDisposition::Downloaded);
    assert_eq!(second.archive_path, first.archive_path);
    assert_eq!(std::fs::read(&second.archive_path).unwrap(), BYTES);
    let damaged = temp.path().join("cache/damaged");
    assert!(!damaged.exists() || std::fs::read_dir(damaged).unwrap().next().is_none());

    let third = cache
        .acquire(
            &request(
                &format!("{name}-third"),
                "https://network-must-not-be-used.invalid/archive".to_owned(),
                BYTES,
            ),
            &sink,
        )
        .unwrap();
    assert_eq!(third.disposition, CacheDisposition::Hit);
    assert_eq!(third.repaired, None);
}

#[test]
fn a_cache_entry_missing_its_archive_is_downloaded_again() {
    assert_damaged_entry_is_downloaded_again(
        "missing-archive",
        |archive, _metadata| std::fs::remove_file(archive).unwrap(),
        &["archive and metadata must both exist"],
    );
}

#[test]
fn a_cache_entry_missing_its_metadata_is_downloaded_again() {
    assert_damaged_entry_is_downloaded_again(
        "missing-metadata",
        |_archive, metadata| std::fs::remove_file(metadata).unwrap(),
        &["archive and metadata must both exist"],
    );
}

#[test]
fn a_cache_entry_with_changed_archive_bytes_is_downloaded_again() {
    assert_damaged_entry_is_downloaded_again(
        "changed-archive",
        |archive, _metadata| std::fs::write(archive, b"changed").unwrap(),
        &["metadata/file mismatch"],
    );
}

#[test]
fn a_cache_entry_with_unreadable_metadata_is_downloaded_again() {
    assert_damaged_entry_is_downloaded_again(
        "unreadable-metadata",
        |_archive, metadata| std::fs::write(metadata, b"{ truncated").unwrap(),
        &["unreadable cache record", ".json"],
    );
}

/// An archive another program holds open cannot be proven damaged; it stays in place.
#[cfg(windows)]
#[test]
fn a_cache_entry_that_cannot_be_read_is_kept() {
    use std::os::windows::fs::OpenOptionsExt;

    let bytes = b"artifact held open by another program";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let (url, server) = one_shot_server(bytes);
    let first = cache
        .acquire(&request("locked-first", url, bytes), &sink)
        .unwrap();
    server.join().unwrap();
    let exclusive = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&first.archive_path)
        .unwrap();

    let result = cache.acquire(
        &request(
            "locked-second",
            "https://network-must-not-be-used.invalid/archive".to_owned(),
            bytes,
        ),
        &sink,
    );

    assert!(matches!(result, Err(AcquireError::Io { .. })), "{result:?}");
    drop(exclusive);
    assert!(!temp.path().join("cache/damaged").exists());
    assert_eq!(std::fs::read(&first.archive_path).unwrap(), bytes);
}

#[test]
fn hash_mismatch_never_publishes_a_cache_object() {
    let bytes = b"wrong artifact";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let (url, server) = one_shot_server(bytes);
    let (sink, _) = ChannelSink::unbounded();
    let download = DownloadRequest {
        request_id: "hash-mismatch".to_owned(),
        url,
        expected_length: bytes.len() as u64,
        expected_sha256: "00".repeat(32),
        redirect_hosts: Vec::new(),
        max_attempts: 1,
    };

    assert!(matches!(
        cache.acquire(&download, &sink),
        Err(AcquireError::HashMismatch { .. })
    ));
    server.join().unwrap();
    assert!(!temp
        .path()
        .join(format!(
            "cache/sha256/00/{}.archive",
            download.expected_sha256
        ))
        .exists());
}

#[test]
fn concurrent_requests_for_one_digest_publish_one_verified_object() {
    let bytes = b"one digest from two requests";
    let server = Server::http("127.0.0.1:0").unwrap();
    let address = server.server_addr().to_ip().unwrap();
    let url = format!("http://{address}/archive");
    let request_count = Arc::new(AtomicUsize::new(0));
    let server_count = Arc::clone(&request_count);
    let server_thread = thread::spawn(move || {
        while server_count.load(Ordering::SeqCst) < 2 {
            let Some(request) = server.recv_timeout(Duration::from_millis(750)).unwrap() else {
                break;
            };
            server_count.fetch_add(1, Ordering::SeqCst);
            let response = Response::new(
                StatusCode(200),
                vec![Header::from_bytes("ETag", "\"concurrent-v1\"").unwrap()],
                Cursor::new(bytes),
                Some(bytes.len()),
                None,
            );
            request.respond(response).unwrap();
        }
    });
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let left_cache = cache.clone();
    let right_cache = cache.clone();
    let left_url = url.clone();
    let right_url = url;

    let left = thread::spawn(move || {
        let (sink, _) = ChannelSink::unbounded();
        left_cache.acquire(&request("left", left_url, bytes), &sink)
    });
    let right = thread::spawn(move || {
        let (sink, _) = ChannelSink::unbounded();
        right_cache.acquire(&request("right", right_url, bytes), &sink)
    });

    let left = left.join().unwrap().unwrap();
    let right = right.join().unwrap().unwrap();
    server_thread.join().unwrap();
    assert_eq!(left.archive_path, right.archive_path);
    assert_eq!(std::fs::read(&left.archive_path).unwrap(), bytes);
    let metadata_count = std::fs::read_dir(left.archive_path.parent().unwrap())
        .unwrap()
        .filter(|entry| entry.as_ref().unwrap().path().extension().unwrap() == "json")
        .count();
    assert_eq!(metadata_count, 1);
    assert!((1..=2).contains(&request_count.load(Ordering::SeqCst)));
}
