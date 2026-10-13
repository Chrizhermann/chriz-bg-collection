use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use bg_engine::acquire::{
    AcquireError, ArtifactCache, CacheDisposition, DownloadRequest, StopSignal,
};
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

/// Metadata that cannot be read at the moment, here because another program locks it, is not
/// proof of damage although it could be opened. Both files stay for the next request.
#[cfg(windows)]
#[test]
fn a_cache_entry_whose_metadata_cannot_be_read_is_kept() {
    use fs4::fs_std::FileExt;

    let bytes = b"artifact whose metadata is locked by another program";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let (url, server) = one_shot_server(bytes);
    let first = cache
        .acquire(&request("locked-metadata-first", url, bytes), &sink)
        .unwrap();
    server.join().unwrap();
    let locked = std::fs::File::open(&first.metadata_path).unwrap();
    locked.lock_exclusive().unwrap();

    let result = cache.acquire(
        &request(
            "locked-metadata-second",
            "https://network-must-not-be-used.invalid/archive".to_owned(),
            bytes,
        ),
        &sink,
    );

    assert!(
        matches!(
            result,
            Err(AcquireError::Io {
                action: "read cache metadata",
                ..
            })
        ),
        "{result:?}"
    );
    FileExt::unlock(&locked).unwrap();
    drop(locked);
    assert!(!temp.path().join("cache/damaged").exists());
    let hit = cache
        .acquire(
            &request(
                "locked-metadata-third",
                "https://network-must-not-be-used.invalid/archive".to_owned(),
                bytes,
            ),
            &sink,
        )
        .unwrap();
    assert_eq!(hit.disposition, CacheDisposition::Hit);
    assert_eq!(hit.repaired, None);
}

/// Archive and metadata agree, so a request pinning the same digest with another length is
/// wrong. It fails without evicting the entry that other requests share.
#[test]
fn a_request_with_a_wrong_length_keeps_a_valid_shared_entry() {
    let bytes = b"artifact pinned with two different lengths";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let (url, server) = one_shot_server(bytes);
    let first = cache
        .acquire(&request("length-first", url, bytes), &sink)
        .unwrap();
    server.join().unwrap();
    let mut wrong = request(
        "length-wrong",
        "https://network-must-not-be-used.invalid/archive".to_owned(),
        bytes,
    );
    wrong.expected_length += 1;

    let result = cache.acquire(&wrong, &sink);

    assert!(
        matches!(
            result,
            Err(AcquireError::LengthMismatch { expected, actual })
                if expected == bytes.len() as u64 + 1 && actual == bytes.len() as u64
        ),
        "{result:?}"
    );
    assert!(!temp.path().join("cache/damaged").exists());
    assert_eq!(std::fs::read(&first.archive_path).unwrap(), bytes);
    let hit = cache
        .acquire(
            &request(
                "length-correct",
                "https://network-must-not-be-used.invalid/archive".to_owned(),
                bytes,
            ),
            &sink,
        )
        .unwrap();
    assert_eq!(hit.disposition, CacheDisposition::Hit);
    assert_eq!(hit.repaired, None);
}

/// Damage that appears while a download runs is found by the check under the digest lock
/// before publication, set aside and reported.
#[test]
fn damage_that_appears_during_a_download_is_repaired_before_publication() {
    static BYTES: &[u8] = b"artifact damaged by another program during its download";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let digest = sha256(BYTES);
    let metadata = temp
        .path()
        .join("cache/sha256")
        .join(&digest[..2])
        .join(format!("{digest}.json"));
    let server = Server::http("127.0.0.1:0").unwrap();
    let url = format!("http://{}/archive", server.server_addr().to_ip().unwrap());
    let (requested, request_seen) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel::<()>();
    let server = thread::spawn(move || {
        let incoming = server
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .expect("download request");
        requested.send(()).unwrap();
        released.recv().unwrap();
        incoming
            .respond(Response::new(
                StatusCode(200),
                vec![Header::from_bytes("ETag", "\"cache-v1\"").unwrap()],
                Cursor::new(BYTES),
                Some(BYTES.len()),
                None,
            ))
            .unwrap();
    });

    let acquired = thread::scope(|scope| {
        let download = scope.spawn(|| {
            let (sink, _) = ChannelSink::unbounded();
            cache.acquire(&request("during-download", url, BYTES), &sink)
        });
        request_seen
            .recv_timeout(Duration::from_secs(5))
            .expect("the download started");
        // Half of an entry, as an interrupted publication by another program would leave it.
        std::fs::create_dir_all(metadata.parent().unwrap()).unwrap();
        std::fs::write(&metadata, b"{}").unwrap();
        release.send(()).unwrap();
        download.join().unwrap().unwrap()
    });
    server.join().unwrap();

    assert_eq!(acquired.disposition, CacheDisposition::Downloaded);
    let repair = acquired.repaired.expect("damage must be reported");
    assert!(
        repair
            .reason
            .contains("archive and metadata must both exist"),
        "{}",
        repair.reason
    );
    assert_eq!(std::fs::read(&acquired.archive_path).unwrap(), BYTES);
    let (sink, _) = ChannelSink::unbounded();
    let hit = cache
        .acquire(
            &request(
                "after-download",
                "https://network-must-not-be-used.invalid/archive".to_owned(),
                BYTES,
            ),
            &sink,
        )
        .unwrap();
    assert_eq!(hit.disposition, CacheDisposition::Hit);
}

/// When only part of a damaged entry could be set aside, the next request finds the rest
/// damaged too and completes the repair.
#[cfg(windows)]
#[test]
fn an_entry_left_partly_set_aside_is_repaired_by_the_next_request() {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_SHARE_READ: u32 = 0x1;
    static BYTES: &[u8] = b"artifact whose metadata cannot be moved at first";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let (url, server) = one_shot_server(BYTES);
    let first = cache
        .acquire(&request("partly-first", url, BYTES), &sink)
        .unwrap();
    server.join().unwrap();
    std::fs::write(&first.archive_path, b"changed").unwrap();
    // Another program reads the metadata and does not allow it to be moved meanwhile.
    let reader = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(&first.metadata_path)
        .unwrap();

    let result = cache.acquire(
        &request(
            "partly-second",
            "https://network-must-not-be-used.invalid/archive".to_owned(),
            BYTES,
        ),
        &sink,
    );

    assert!(
        matches!(
            result,
            Err(AcquireError::Io {
                action: "set aside damaged cache entry",
                ..
            })
        ),
        "{result:?}"
    );
    drop(reader);
    assert!(!first.archive_path.exists());
    assert!(first.metadata_path.exists());

    let (url, server) = one_shot_server(BYTES);
    let repaired = cache
        .acquire(&request("partly-third", url, BYTES), &sink)
        .unwrap();
    server.join().unwrap();
    let repair = repaired
        .repaired
        .expect("the remaining half must be repaired");
    assert!(
        repair
            .reason
            .contains("archive and metadata must both exist"),
        "{}",
        repair.reason
    );
    assert_eq!(repaired.disposition, CacheDisposition::Downloaded);
    assert_eq!(std::fs::read(&repaired.archive_path).unwrap(), BYTES);
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

fn stoppable() -> (Arc<AtomicBool>, StopSignal) {
    let stopped = Arc::new(AtomicBool::new(false));
    let signal = StopSignal::new({
        let stopped = Arc::clone(&stopped);
        move || stopped.load(Ordering::SeqCst)
    });
    (stopped, signal)
}

fn open_entry_lock(path: &std::path::Path) -> std::fs::File {
    std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .unwrap()
}

/// Runs `acquire` on another thread, lets `hold` make another program hold `lock`, and stops
/// once the acquisition has waited. The wait must end within five seconds; the lock is released
/// afterwards either way, so that a wait that ignored Stop still ends the test.
fn stop_while_waiting(
    lock: &std::fs::File,
    stopped: &AtomicBool,
    hold: impl FnOnce(),
    acquire: impl FnOnce() -> Result<bg_engine::acquire::AcquiredArtifact, AcquireError> + Send,
) -> Result<bg_engine::acquire::AcquiredArtifact, AcquireError> {
    use fs4::fs_std::FileExt;

    thread::scope(|scope| {
        let waiting = scope.spawn(acquire);
        hold();
        thread::sleep(Duration::from_millis(400));
        let waited = !waiting.is_finished();
        stopped.store(true, Ordering::SeqCst);
        let deadline = Instant::now() + Duration::from_secs(5);
        while !waiting.is_finished() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        let ended = waiting.is_finished();
        FileExt::unlock(lock).unwrap();
        assert!(waited, "the request must wait for the lock");
        assert!(ended, "Stop did not end the wait");
        waiting.join().unwrap()
    })
}

/// Stop ends a wait for a download whose cache entry another process locks and never
/// releases, for example because it hangs or is suspended. The network is never contacted.
#[test]
fn a_stop_ends_the_wait_for_a_download_lock_that_is_never_released() {
    use fs4::fs_std::FileExt;

    let bytes = b"artifact whose download another program never finishes";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let download = request(
        "stopped-wait",
        "https://network-must-not-be-used.invalid/archive".to_owned(),
        bytes,
    );
    let lock_path = temp
        .path()
        .join("cache/locks")
        .join(format!("{}.lock", download.expected_sha256));
    let lock = open_entry_lock(&lock_path);
    lock.lock_exclusive().unwrap();
    let (stopped, stop) = stoppable();

    let result = stop_while_waiting(
        &lock,
        &stopped,
        || {},
        || cache.acquire_with_stop(&download, &sink, &stop),
    );

    match result {
        Err(AcquireError::Stopped { lock }) => assert_eq!(lock, lock_path),
        other => panic!("expected a stopped wait, got {other:?}"),
    }
    let digest = &download.expected_sha256;
    assert!(!temp
        .path()
        .join(format!("cache/sha256/{}/{digest}.archive", &digest[..2]))
        .exists());
}

/// Stop also ends the wait to publish a finished download while another process that started
/// downloading the same artifact holds the entry and never releases it.
#[test]
fn a_stop_ends_the_wait_to_publish_a_finished_download() {
    use fs4::fs_std::FileExt;

    let bytes: &'static [u8] = b"artifact another program locks while this one downloads it";
    let temp = TempDir::new().unwrap();
    let cache = ArtifactCache::open(temp.path().join("cache")).unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let server = Server::http("127.0.0.1:0").unwrap();
    let address = server.server_addr().to_ip().unwrap();
    let download = request(
        "stopped-publish",
        format!("http://{address}/archive"),
        bytes,
    );
    let lock_path = temp
        .path()
        .join("cache/locks")
        .join(format!("{}.lock", download.expected_sha256));
    let lock = open_entry_lock(&lock_path);
    let (stopped, stop) = stoppable();

    let result = thread::scope(|scope| {
        let (arrived, request_arrived) = std::sync::mpsc::channel();
        let (respond, may_respond) = std::sync::mpsc::channel::<()>();
        scope.spawn(move || {
            let request = server
                .recv_timeout(Duration::from_secs(5))
                .unwrap()
                .expect("download request");
            arrived.send(()).unwrap();
            may_respond.recv().unwrap();
            let response = Response::new(
                StatusCode(200),
                Vec::new(),
                Cursor::new(bytes),
                Some(bytes.len()),
                None,
            );
            request.respond(response).unwrap();
        });
        stop_while_waiting(
            &lock,
            &stopped,
            || {
                request_arrived
                    .recv_timeout(Duration::from_secs(5))
                    .expect("the download must start");
                // The first check found no entry and released the lock before downloading.
                assert!(lock.try_lock_exclusive().unwrap());
                respond.send(()).unwrap();
            },
            || cache.acquire_with_stop(&download, &sink, &stop),
        )
    });

    match result {
        Err(AcquireError::Stopped { lock }) => assert_eq!(lock, lock_path),
        other => panic!("expected a stopped wait, got {other:?}"),
    }
    let digest = &download.expected_sha256;
    assert!(!temp
        .path()
        .join(format!("cache/sha256/{}/{digest}.archive", &digest[..2]))
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
