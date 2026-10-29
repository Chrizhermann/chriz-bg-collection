use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Cursor, Write};
use std::net::{Shutdown, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use bg_engine::acquire::{
    served_length_limit, validate_redirect_target, AcceptedContent, AcquireError, ArchiveFormat,
    ArtifactCache, CacheDisposition, DownloadRequest, DownloadRoute, ServedRedirects, ServedRules,
    StopSignal,
};
use bg_engine::events::{ChannelSink, EngineEvent};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Clone, Debug)]
struct SeenRequest {
    path: String,
    headers: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
struct Reply {
    status: u16,
    headers: Vec<(&'static str, String)>,
    body: Vec<u8>,
    /// The announced Content-Length. `None` announces none: the body then ends where the
    /// connection closes, so a body cut short looks complete to the client.
    advertised_length: Option<usize>,
}

impl Reply {
    fn complete(body: &[u8], etag: &str) -> Self {
        Self {
            status: 200,
            headers: vec![("ETag", etag.to_owned())],
            body: body.to_vec(),
            advertised_length: Some(body.len()),
        }
    }

    fn interrupted(prefix: &[u8], full_length: usize, etag: &str) -> Self {
        Self {
            status: 200,
            headers: vec![("ETag", etag.to_owned())],
            body: prefix.to_vec(),
            advertised_length: Some(full_length),
        }
    }
}

struct TestServer {
    base_url: String,
    requests: Arc<Mutex<Vec<SeenRequest>>>,
    handle: Option<JoinHandle<()>>,
}

impl TestServer {
    fn start<F>(request_count: usize, handler: F) -> Self
    where
        F: Fn(usize, &SeenRequest) -> Reply + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let address = listener.local_addr().expect("local test server address");
        listener
            .set_nonblocking(true)
            .expect("configure local test server");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let thread_requests = Arc::clone(&requests);
        let handler = Arc::new(handler);
        let handle = thread::spawn(move || {
            for index in 0..request_count {
                let deadline = Instant::now() + Duration::from_secs(5);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(
                                Instant::now() < deadline,
                                "local HTTP request arrived before timeout"
                            );
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("receive local HTTP request: {error}"),
                    }
                };
                stream
                    .set_nonblocking(false)
                    .expect("configure accepted test connection");
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("set request read timeout");
                let mut lines = BufReader::new(&mut stream).lines();
                let request_line = lines
                    .next()
                    .expect("request line exists")
                    .expect("read request line");
                let path = request_line
                    .split_ascii_whitespace()
                    .nth(1)
                    .expect("request target")
                    .to_owned();
                let mut request_headers = BTreeMap::new();
                for line in lines {
                    let line = line.expect("read request header");
                    if line.is_empty() {
                        break;
                    }
                    let (name, value) = line.split_once(':').expect("valid request header");
                    request_headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
                }
                let seen = SeenRequest {
                    path,
                    headers: request_headers,
                };
                thread_requests
                    .lock()
                    .expect("request log mutex")
                    .push(seen.clone());
                let reply = handler(index, &seen);
                let reason = match reply.status {
                    200 => "OK",
                    206 => "Partial Content",
                    302 => "Found",
                    503 => "Service Unavailable",
                    _ => "Test Response",
                };
                write!(stream, "HTTP/1.1 {} {}\r\n", reply.status, reason)
                    .expect("write response status");
                for (name, value) in reply.headers {
                    writeln!(stream, "{name}: {value}\r").expect("write response header");
                }
                if let Some(length) = reply.advertised_length {
                    writeln!(stream, "Content-Length: {length}\r").expect("write response length");
                }
                write!(stream, "Connection: close\r\n\r\n").expect("finish response headers");
                stream.write_all(&reply.body).expect("write response body");
                stream.flush().expect("flush response");
                stream
                    .shutdown(Shutdown::Both)
                    .expect("close response connection");
            }
        });
        Self {
            base_url: format!("http://{address}"),
            requests,
            handle: Some(handle),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn finish(mut self) -> Vec<SeenRequest> {
        self.handle
            .take()
            .expect("server join handle")
            .join()
            .expect("local server thread");
        Arc::try_unwrap(self.requests)
            .expect("no remaining request log owners")
            .into_inner()
            .expect("request log mutex")
    }
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn request(id: &str, url: String, bytes: &[u8], max_attempts: u32) -> DownloadRequest {
    DownloadRequest {
        request_id: id.to_owned(),
        url,
        expected_length: bytes.len() as u64,
        expected_sha256: sha256(bytes),
        redirect_hosts: Vec::new(),
        max_attempts,
        route: DownloadRoute::Generic,
    }
}

fn cache(temp: &TempDir) -> ArtifactCache {
    ArtifactCache::open(temp.path().join("cache")).expect("open artifact cache")
}

/// What a mod's changed ZIP download must still be.
const ZIP: ServedRules = ServedRules {
    format: ArchiveFormat::Zip,
    redirects: ServedRedirects::AnyHttpsHost,
};

/// A small ZIP archive whose only file holds `text`.
fn zip_archive(text: &[u8]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "Mod/readme.txt",
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )
        .expect("start ZIP entry");
    writer.write_all(text).expect("write ZIP entry");
    writer.finish().expect("finish ZIP").into_inner()
}

#[test]
fn streams_a_complete_download_and_progress() {
    let bytes = b"immutable archive contents";
    let server = TestServer::start(1, move |_, _| Reply::complete(bytes, "\"archive-v1\""));
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, events) = ChannelSink::unbounded();

    let acquired = cache
        .acquire(
            &request("complete", server.url("/archive"), bytes, 2),
            &sink,
        )
        .unwrap();

    assert_eq!(acquired.disposition, CacheDisposition::Downloaded);
    assert_eq!(std::fs::read(&acquired.archive_path).unwrap(), bytes);
    assert_eq!(acquired.metadata.original_url, server.url("/archive"));
    assert_eq!(acquired.metadata.final_url, server.url("/archive"));
    assert_eq!(acquired.metadata.etag.as_deref(), Some("\"archive-v1\""));
    assert_eq!(acquired.metadata.length, bytes.len() as u64);
    assert_eq!(acquired.metadata.sha256, sha256(bytes));
    assert!(events.try_iter().any(|event| {
        event
            == EngineEvent::StepProgress {
                id: "complete".to_owned(),
                done: bytes.len() as u64,
                total: bytes.len() as u64,
            }
    }));
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn interrupted_response_leaves_resumable_partial_but_no_cache_object() {
    let bytes = b"abcdefghij";
    let prefix = &bytes[..4];
    let server = TestServer::start(1, move |_, _| {
        Reply::interrupted(prefix, bytes.len(), "\"archive-v1\"")
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("interrupted", server.url("/archive"), bytes, 1);

    assert!(matches!(
        cache.acquire(&download, &sink),
        Err(AcquireError::RetryExhausted { attempts: 1, .. })
    ));
    assert_eq!(
        std::fs::read(temp.path().join("cache/partial/interrupted.part")).unwrap(),
        prefix
    );
    assert!(!temp
        .path()
        .join(format!(
            "cache/sha256/{}/{}.archive",
            &download.expected_sha256[..2],
            download.expected_sha256
        ))
        .exists());
    server.finish();
}

#[test]
fn resumes_only_an_exact_206_with_a_strong_matching_etag() {
    let bytes = b"abcdefghij";
    let prefix_len = 4;
    let server = TestServer::start(2, move |index, _| match index {
        0 => Reply::interrupted(&bytes[..prefix_len], bytes.len(), "\"archive-v1\""),
        1 => Reply {
            status: 206,
            headers: vec![
                ("ETag", "\"archive-v1\"".to_owned()),
                (
                    "Content-Range",
                    format!(
                        "bytes {prefix_len}-{}/{total}",
                        bytes.len() - 1,
                        total = bytes.len()
                    ),
                ),
            ],
            body: bytes[prefix_len..].to_vec(),
            advertised_length: Some(bytes.len() - prefix_len),
        },
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("resume", server.url("/archive"), bytes, 1);

    assert!(cache.acquire(&download, &sink).is_err());
    let acquired = cache.acquire(&download, &sink).unwrap();

    assert_eq!(std::fs::read(acquired.archive_path).unwrap(), bytes);
    let seen = server.finish();
    assert!(!seen[0].headers.contains_key("range"));
    assert_eq!(seen[1].headers.get("range").unwrap(), "bytes=4-");
    assert_eq!(seen[1].headers.get("if-range").unwrap(), "\"archive-v1\"");
}

#[test]
fn changed_etag_discards_the_range_response_and_restarts_from_zero() {
    let bytes = b"abcdefghij";
    let prefix_len = 4;
    let server = TestServer::start(3, move |index, _| match index {
        0 => Reply::interrupted(&bytes[..prefix_len], bytes.len(), "\"archive-v1\""),
        1 => Reply {
            status: 206,
            headers: vec![
                ("ETag", "\"archive-v2\"".to_owned()),
                (
                    "Content-Range",
                    format!(
                        "bytes {prefix_len}-{}/{total}",
                        bytes.len() - 1,
                        total = bytes.len()
                    ),
                ),
            ],
            body: bytes[prefix_len..].to_vec(),
            advertised_length: Some(bytes.len() - prefix_len),
        },
        2 => Reply::complete(bytes, "\"archive-v2\""),
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("changed-etag", server.url("/archive"), bytes, 1);

    assert!(cache.acquire(&download, &sink).is_err());
    let acquired = cache.acquire(&download, &sink).unwrap();

    assert_eq!(acquired.metadata.etag.as_deref(), Some("\"archive-v2\""));
    let seen = server.finish();
    assert!(seen[1].headers.contains_key("range"));
    assert!(!seen[2].headers.contains_key("range"));
}

#[test]
fn a_200_response_to_range_is_used_as_a_clean_byte_zero_restart() {
    let bytes = b"abcdefghij";
    let prefix_len = 4;
    let server = TestServer::start(2, move |index, _| match index {
        0 => Reply::interrupted(&bytes[..prefix_len], bytes.len(), "\"archive-v1\""),
        1 => Reply::complete(bytes, "\"archive-v2\""),
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("range-ignored", server.url("/archive"), bytes, 1);

    assert!(cache.acquire(&download, &sink).is_err());
    let acquired = cache.acquire(&download, &sink).unwrap();

    assert_eq!(std::fs::read(acquired.archive_path).unwrap(), bytes);
    let seen = server.finish();
    assert_eq!(seen[1].headers.get("range").unwrap(), "bytes=4-");
}

#[test]
fn an_invalid_content_range_is_discarded_before_a_fresh_request() {
    let bytes = b"abcdefghij";
    let prefix_len = 4;
    let server = TestServer::start(3, move |index, _| match index {
        0 => Reply::interrupted(&bytes[..prefix_len], bytes.len(), "\"archive-v1\""),
        1 => Reply {
            status: 206,
            headers: vec![
                ("ETag", "\"archive-v1\"".to_owned()),
                ("Content-Range", "bytes 3-9/10".to_owned()),
            ],
            body: bytes[prefix_len..].to_vec(),
            advertised_length: Some(bytes.len() - prefix_len),
        },
        2 => Reply::complete(bytes, "\"archive-v1\""),
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("invalid-range", server.url("/archive"), bytes, 1);

    assert!(cache.acquire(&download, &sink).is_err());
    let acquired = cache.acquire(&download, &sink).unwrap();

    assert_eq!(std::fs::read(acquired.archive_path).unwrap(), bytes);
    let seen = server.finish();
    assert!(seen[1].headers.contains_key("range"));
    assert!(!seen[2].headers.contains_key("range"));
}

#[test]
fn follows_a_local_redirect_but_rejects_https_to_http_before_requesting_it() {
    let bytes = b"redirected archive";
    let server = TestServer::start(2, move |index, _| match index {
        0 => Reply {
            status: 302,
            headers: vec![("Location", "/final".to_owned())],
            body: Vec::new(),
            advertised_length: Some(0),
        },
        1 => Reply::complete(bytes, "\"redirect-v1\""),
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();

    let acquired = cache
        .acquire(&request("redirect", server.url("/start"), bytes, 1), &sink)
        .unwrap();

    assert_eq!(acquired.metadata.final_url, server.url("/final"));
    assert!(matches!(
        validate_redirect_target(
            "https://downloads.example.invalid/archive",
            "http://127.0.0.1:1234/archive"
        ),
        Err(AcquireError::RedirectDowngrade { .. })
    ));
    let seen = server.finish();
    assert_eq!(seen[0].path, "/start");
    assert_eq!(seen[1].path, "/final");
}

#[test]
fn retries_transient_statuses_only_up_to_the_configured_limit() {
    let bytes = b"never returned";
    let server = TestServer::start(3, |_, _| Reply {
        status: 503,
        headers: Vec::new(),
        body: Vec::new(),
        advertised_length: Some(0),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();

    assert!(matches!(
        cache.acquire(&request("retry", server.url("/archive"), bytes, 3), &sink),
        Err(AcquireError::RetryExhausted { attempts: 3, .. })
    ));
    assert_eq!(server.finish().len(), 3);
}

#[test]
fn transient_failure_recovers_and_later_acquisition_uses_verified_cache() {
    let bytes = b"archive recovered after temporary HTTP 500";
    let server = TestServer::start(2, move |index, _| match index {
        0 => Reply {
            status: 500,
            headers: Vec::new(),
            body: Vec::new(),
            advertised_length: Some(0),
        },
        1 => Reply::complete(bytes, "\"recovered-v1\""),
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, events) = ChannelSink::unbounded();
    let download = request("recovery", server.url("/archive"), bytes, 5);

    let acquired = cache.acquire(&download, &sink).unwrap();
    assert_eq!(acquired.disposition, CacheDisposition::Downloaded);
    assert_eq!(std::fs::read(&acquired.archive_path).unwrap(), bytes);
    assert!(events.try_iter().any(|event| matches!(event,
        EngineEvent::ConsoleLine { line, .. } if line.contains("immediately") && line.contains("2/5")
    )));
    assert_eq!(server.finish().len(), 2);

    // The server is gone. A subsequent resume must reuse the verified archive.
    let cached = cache.acquire(&download, &sink).unwrap();
    assert_eq!(cached.disposition, CacheDisposition::Hit);
    assert_eq!(cached.archive_path, acquired.archive_path);
    assert!(events
        .try_iter()
        .all(|event| matches!(event, EngineEvent::StepProgress { .. })));
}

#[test]
fn permanent_status_is_not_retried_even_with_five_attempt_budget() {
    let bytes = b"missing archive";
    let server = TestServer::start(1, |_, _| Reply {
        status: 404,
        headers: Vec::new(),
        body: Vec::new(),
        advertised_length: Some(0),
    });
    let temp = TempDir::new().unwrap();
    let (sink, events) = ChannelSink::unbounded();
    let result = cache(&temp).acquire(&request("missing", server.url("/archive"), bytes, 5), &sink);
    assert!(matches!(
        result,
        Err(AcquireError::HttpStatus { status: 404, .. })
    ));
    assert_eq!(server.finish().len(), 1);
    assert!(events.try_iter().next().is_none());
}

fn cache_object(temp: &TempDir, digest: &str) -> std::path::PathBuf {
    temp.path()
        .join("cache/sha256")
        .join(&digest[..2])
        .join(format!("{digest}.archive"))
}

#[test]
fn a_changed_download_is_accepted_cached_by_its_own_digest_and_reused_on_resume() {
    let reviewed = b"Snowytoes v1.6 as reviewed";
    let served = zip_archive(b"Snowytoes v1.7, a newer release of the same mod");
    let body = served.clone();
    let server = TestServer::start(1, move |_, _| {
        let mut reply = Reply::complete(&body, "\"v1.7\"");
        reply.headers.push((
            "Content-Disposition",
            "attachment; filename=\"Snowytoes-v1.7.zip\"".to_owned(),
        ));
        reply
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("snowytoes", server.url("/download?wpdmdl=557"), reviewed, 1);

    let first = cache
        .acquire_as_served(&download, None, ZIP, &sink, &StopSignal::never())
        .unwrap();

    assert!(!first.matches_pin(&download));
    assert_eq!(first.artifact.disposition, CacheDisposition::Downloaded);
    assert_eq!(first.artifact.metadata.length, served.len() as u64);
    assert_eq!(first.artifact.metadata.sha256, sha256(&served));
    assert_eq!(first.served_filename.as_deref(), Some("Snowytoes-v1.7.zip"));
    assert!(first.unreviewed_hosts.is_empty());
    assert_eq!(
        first.artifact.archive_path,
        cache_object(&temp, &sha256(&served))
    );
    assert_eq!(std::fs::read(&first.artifact.archive_path).unwrap(), served);
    assert!(!cache_object(&temp, &download.expected_sha256).exists());
    assert_eq!(server.finish().len(), 1);

    // The server is gone: a resumed campaign reuses exactly the archive it accepted.
    let accepted = AcceptedContent {
        length: first.artifact.metadata.length,
        sha256: first.artifact.metadata.sha256.clone(),
    };
    let resumed = cache
        .acquire_as_served(&download, Some(&accepted), ZIP, &sink, &StopSignal::never())
        .unwrap();
    assert_eq!(resumed.artifact.disposition, CacheDisposition::Hit);
    assert_eq!(resumed.artifact.archive_path, first.artifact.archive_path);
    assert_eq!(resumed.served_filename, None);

    // The strict acquisition of the pin still refuses the changed bytes.
    let strict_server = TestServer::start(1, move |_, _| Reply::complete(&served, "\"v1.7\""));
    let strict = request("snowytoes", strict_server.url("/download"), reviewed, 1);
    assert!(matches!(
        cache.acquire(&strict, &sink),
        Err(AcquireError::LengthMismatch { .. })
    ));
    strict_server.finish();
}

#[test]
fn a_campaign_that_accepted_a_changed_file_keeps_it_even_when_the_reviewed_one_is_cached() {
    let reviewed = b"the reviewed release";
    let served = zip_archive(b"the release the link serves now");
    let body = served.clone();
    let server = TestServer::start(3, move |index, _| match index {
        0 | 2 => Reply::complete(&body, "\"now\""),
        1 => Reply::complete(reviewed, "\"reviewed\""),
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("kept", server.url("/archive"), reviewed, 1);
    let first = cache
        .acquire_as_served(&download, None, ZIP, &sink, &StopSignal::never())
        .unwrap();
    let accepted = AcceptedContent {
        length: first.artifact.metadata.length,
        sha256: first.artifact.metadata.sha256.clone(),
    };
    // Another installation puts the reviewed release into the shared cache.
    cache
        .acquire(
            &request("other", server.url("/archive"), reviewed, 1),
            &sink,
        )
        .unwrap();

    let kept = cache
        .acquire_as_served(&download, Some(&accepted), ZIP, &sink, &StopSignal::never())
        .unwrap();
    assert_eq!(kept.artifact.disposition, CacheDisposition::Hit);
    assert_eq!(kept.artifact.metadata.sha256, sha256(&served));

    // Lost from the cache, the accepted file is fetched again instead of the cached reviewed one.
    std::fs::remove_file(&first.artifact.archive_path).unwrap();
    std::fs::remove_file(&first.artifact.metadata_path).unwrap();
    let fetched = cache
        .acquire_as_served(&download, Some(&accepted), ZIP, &sink, &StopSignal::never())
        .unwrap();
    assert_eq!(fetched.artifact.disposition, CacheDisposition::Downloaded);
    assert_eq!(
        std::fs::read(&fetched.artifact.archive_path).unwrap(),
        served
    );

    // Without an accepted file the reviewed one is used.
    let fresh = cache
        .acquire_as_served(&download, None, ZIP, &sink, &StopSignal::never())
        .unwrap();
    assert!(fresh.matches_pin(&download));
    assert_eq!(fresh.artifact.disposition, CacheDisposition::Hit);
    assert_eq!(server.finish().len(), 3);
}

#[test]
fn a_download_that_still_matches_its_pin_is_the_reviewed_cache_entry() {
    let bytes = b"reviewed archive, still served unchanged";
    let server = TestServer::start(1, move |_, _| Reply::complete(bytes, "\"same\""));
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("unchanged", server.url("/archive"), bytes, 1);

    // The reviewed bytes need not look like an archive: they were reviewed.
    let served = cache
        .acquire_as_served(&download, None, ZIP, &sink, &StopSignal::never())
        .unwrap();
    server.finish();

    assert!(served.matches_pin(&download));
    assert_eq!(
        served.artifact.archive_path,
        cache_object(&temp, &download.expected_sha256)
    );
    // The normal acquisition finds the very same entry without asking the server again.
    let strict = cache.acquire(&download, &sink).unwrap();
    assert_eq!(strict.disposition, CacheDisposition::Hit);
    assert_eq!(strict.archive_path, served.artifact.archive_path);
    assert_eq!(strict.metadata, served.artifact.metadata);
}

#[test]
fn an_interrupted_changed_download_resumes_only_against_the_same_entity() {
    let reviewed = b"old";
    let served = zip_archive(b"a changed archive that arrives in two parts");
    let body = served.clone();
    let prefix_len = 10;
    let server = TestServer::start(2, move |index, _| match index {
        0 => Reply::interrupted(&body[..prefix_len], body.len(), "\"changed-v2\""),
        1 => Reply {
            status: 206,
            headers: vec![
                ("ETag", "\"changed-v2\"".to_owned()),
                (
                    "Content-Range",
                    format!(
                        "bytes {prefix_len}-{}/{total}",
                        body.len() - 1,
                        total = body.len()
                    ),
                ),
            ],
            body: body[prefix_len..].to_vec(),
            advertised_length: Some(body.len() - prefix_len),
        },
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("changed-resume", server.url("/archive"), reviewed, 1);

    assert!(matches!(
        cache.acquire_as_served(&download, None, ZIP, &sink, &StopSignal::never()),
        Err(AcquireError::RetryExhausted { attempts: 1, .. })
    ));
    let acquired = cache
        .acquire_as_served(&download, None, ZIP, &sink, &StopSignal::never())
        .unwrap();

    assert_eq!(
        std::fs::read(&acquired.artifact.archive_path).unwrap(),
        served
    );
    assert_eq!(acquired.artifact.metadata.sha256, sha256(&served));
    let seen = server.finish();
    assert!(!seen[0].headers.contains_key("range"));
    assert_eq!(
        seen[1].headers.get("range").unwrap(),
        &format!("bytes={prefix_len}-")
    );
    assert_eq!(seen[1].headers.get("if-range").unwrap(), "\"changed-v2\"");
}

#[test]
fn a_changed_download_follows_a_redirect_to_another_host_and_reports_it() {
    let reviewed = b"reviewed";
    let served = zip_archive(b"served from a mirror the recipe never reviewed");
    let body = served.clone();
    let server = TestServer::start(3, move |index, seen| match index {
        0 | 2 => {
            let port = seen.headers["host"].rsplit(':').next().unwrap().to_owned();
            Reply {
                status: 302,
                headers: vec![("Location", format!("http://localhost:{port}/mirror.zip"))],
                body: Vec::new(),
                advertised_length: Some(0),
            }
        }
        1 => Reply::complete(&body, "\"mirror\""),
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();

    let changed = cache
        .acquire_as_served(
            &request("mirrored", server.url("/start"), reviewed, 1),
            None,
            ZIP,
            &sink,
            &StopSignal::never(),
        )
        .unwrap();
    // The normal acquisition of a pin refuses the same redirect before following it.
    let strict = cache.acquire(&request("strict", server.url("/start"), reviewed, 1), &sink);

    assert_eq!(changed.unreviewed_hosts, ["localhost"]);
    assert!(changed
        .artifact
        .metadata
        .final_url
        .starts_with("http://localhost:"));
    assert_eq!(changed.served_filename.as_deref(), Some("mirror.zip"));
    assert_eq!(
        std::fs::read(&changed.artifact.archive_path).unwrap(),
        served
    );
    assert!(
        matches!(strict, Err(AcquireError::UnreviewedRedirect { .. })),
        "{strict:?}"
    );
    let seen = server.finish();
    assert_eq!(seen[1].path, "/mirror.zip");
    assert_eq!(seen.len(), 3);
    // An HTTPS link that redirects to plain HTTP is refused under every redirect policy,
    // before the HTTP target is contacted.
    assert!(matches!(
        validate_redirect_target(
            "https://weasel.example.invalid/?wpdmdl=557",
            "http://mirror.example.invalid/a.zip"
        ),
        Err(AcquireError::RedirectDowngrade { .. })
    ));
}

/// The program that runs every mod comes only from where it was reviewed: a changed tool from a
/// host its recipe did not review is refused and not kept. The reviewed tool is accepted from
/// such a host, since its digest proves it, so a moved download host stops nothing.
#[test]
fn a_changed_tool_comes_only_from_its_reviewed_hosts() {
    let reviewed = zip_archive(b"the reviewed WeiDU");
    let changed = zip_archive(b"a WeiDU from somewhere else");
    let (reviewed_body, changed_body) = (reviewed.clone(), changed.clone());
    let server = TestServer::start(4, move |index, seen| match index {
        0 | 2 => {
            let port = seen.headers["host"].rsplit(':').next().unwrap().to_owned();
            Reply {
                status: 302,
                headers: vec![("Location", format!("http://localhost:{port}/weidu.zip"))],
                body: Vec::new(),
                advertised_length: Some(0),
            }
        }
        1 => Reply::complete(&changed_body, "\"elsewhere\""),
        3 => Reply::complete(&reviewed_body, "\"reviewed\""),
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();
    let download = request("weidu", server.url("/weidu"), &reviewed, 3);
    let tool = ServedRules {
        format: ArchiveFormat::Zip,
        redirects: ServedRedirects::ReviewedHostsForChanges,
    };

    let refused = cache.acquire_as_served(&download, None, tool, &sink, &StopSignal::never());
    assert!(
        matches!(&refused, Err(AcquireError::UnreviewedRedirect { from, to })
            if *from == download.url && to.starts_with("http://localhost:")),
        "{refused:?}"
    );
    // A refusal is final: no further attempt, and nothing was kept.
    for file in partial_files(&temp, "weidu") {
        assert!(!file.exists(), "{}", file.display());
    }
    assert_eq!(cached_objects(&temp), 0);

    let accepted = cache
        .acquire_as_served(&download, None, tool, &sink, &StopSignal::never())
        .unwrap();
    assert!(accepted.matches_pin(&download));
    assert_eq!(accepted.unreviewed_hosts, ["localhost"]);
    assert_eq!(server.finish().len(), 4);
}

#[test]
fn a_changed_download_beyond_the_served_limit_is_refused_before_it_is_stored() {
    let reviewed = b"a small reviewed archive";
    let limit = served_length_limit(reviewed.len() as u64);
    assert_eq!(limit, 1024 * 1024 * 1024);
    assert_eq!(served_length_limit(100 * 1024 * 1024), 1600 * 1024 * 1024);
    let server = TestServer::start(1, move |_, _| Reply {
        status: 200,
        headers: vec![("ETag", "\"huge\"".to_owned())],
        body: Vec::new(),
        advertised_length: Some(limit as usize + 1),
    });
    let temp = TempDir::new().unwrap();
    let cache = cache(&temp);
    let (sink, _) = ChannelSink::unbounded();

    let result = cache.acquire_as_served(
        &request("huge", server.url("/archive"), reviewed, 3),
        None,
        ZIP,
        &sink,
        &StopSignal::never(),
    );

    assert!(
        matches!(
            result,
            Err(AcquireError::ServedDownloadTooLarge { length, limit: refused })
                if length == limit + 1 && refused == limit
        ),
        "{result:?}"
    );
    // A refusal is final: no retry, and nothing was written.
    assert_eq!(server.finish().len(), 1);
    assert!(!temp.path().join("cache/partial/huge.part").exists());
}

/// The download of `id` in the cache below `temp`, while it is incomplete.
fn partial_files(temp: &TempDir, id: &str) -> [std::path::PathBuf; 2] {
    let partial = temp.path().join("cache/partial");
    [
        partial.join(format!("{id}.part")),
        partial.join(format!("{id}.json")),
    ]
}

fn cached_objects(temp: &TempDir) -> usize {
    let objects = temp.path().join("cache/sha256");
    if !objects.exists() {
        return 0;
    }
    walkdir::WalkDir::new(objects)
        .into_iter()
        .filter(|entry| entry.as_ref().unwrap().file_type().is_file())
        .count()
}

/// A server that ends a changed download early without announcing its length makes the
/// transfer look complete. The file is no complete archive, so it is dropped and asked for
/// again; the next answer is the whole archive.
#[test]
fn a_changed_download_cut_short_without_a_length_is_asked_for_again() {
    let reviewed = b"the reviewed release";
    let served = zip_archive(b"a changed release that the first answer cuts short");
    let body = served.clone();
    let server = TestServer::start(2, move |index, _| match index {
        0 => Reply {
            status: 200,
            headers: vec![("ETag", "\"changed\"".to_owned())],
            body: body[..body.len() - 20].to_vec(),
            advertised_length: None,
        },
        1 => Reply {
            status: 200,
            headers: vec![("ETag", "\"changed\"".to_owned())],
            body: body.clone(),
            advertised_length: None,
        },
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let (sink, events) = ChannelSink::unbounded();

    let acquired = cache(&temp)
        .acquire_as_served(
            &request("cut-short", server.url("/archive"), reviewed, 2),
            None,
            ZIP,
            &sink,
            &StopSignal::never(),
        )
        .unwrap();

    assert_eq!(
        std::fs::read(&acquired.artifact.archive_path).unwrap(),
        served
    );
    let seen = server.finish();
    assert_eq!(seen.len(), 2);
    // The second request starts over instead of resuming the dropped file.
    assert!(!seen[1].headers.contains_key("range"));
    let lines = events
        .try_iter()
        .filter_map(|event| match event {
            EngineEvent::ConsoleLine { line, .. } => Some(line),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "{lines:#?}");
    assert!(
        lines[0].starts_with("The download of cut-short gave no usable file: ")
            && lines[0].contains("it is not a complete ZIP archive")
            && lines[0].ends_with("Asking again immediately (attempt 2/2)."),
        "{lines:#?}"
    );
    for file in partial_files(&temp, "cut-short") {
        assert!(!file.exists(), "{}", file.display());
    }
}

/// An error page instead of the archive is asked for again with the usual waits, and when every
/// attempt gets it, it is the answer: nothing is kept.
#[test]
fn a_page_served_instead_of_a_changed_archive_is_refused_after_every_attempt() {
    let reviewed = b"the reviewed release";
    let page = b"<!DOCTYPE html>\n<html><body>Download limit reached</body></html>\n";
    let server = TestServer::start(3, move |_, _| {
        let mut reply = Reply::complete(page, "\"page\"");
        reply.headers.push(("Content-Type", "text/html".to_owned()));
        reply
    });
    let temp = TempDir::new().unwrap();
    let (sink, events) = ChannelSink::unbounded();

    let result = cache(&temp).acquire_as_served(
        &request("page", server.url("/archive"), reviewed, 3),
        None,
        ZIP,
        &sink,
        &StopSignal::never(),
    );

    let Err(AcquireError::ServedNotArchive { url, problem }) = result else {
        panic!("expected a refused page, got {result:?}");
    };
    assert_eq!(url, server.url("/archive"));
    assert_eq!(
        problem,
        "it does not begin like a ZIP archive but with the text \"<!DOCTYPE html> <html><body>Down\""
    );
    assert_eq!(server.finish().len(), 3);
    assert_eq!(
        events
            .try_iter()
            .filter(
                |event| matches!(event, EngineEvent::ConsoleLine { line, .. }
                if line.starts_with("The download of page gave no usable file: "))
            )
            .count(),
        2
    );
    for file in partial_files(&temp, "page") {
        assert!(!file.exists(), "{}", file.display());
    }
    assert_eq!(cached_objects(&temp), 0);
}

/// A real archive is accepted even when its server calls it a page: the bytes decide.
#[test]
fn a_changed_archive_with_a_wrong_content_type_is_still_the_archive() {
    let reviewed = b"the reviewed release";
    let served = zip_archive(b"a release its server labels as a page");
    let body = served.clone();
    let server = TestServer::start(1, move |_, _| {
        let mut reply = Reply::complete(&body, "\"mislabelled\"");
        reply.headers.push(("Content-Type", "text/html".to_owned()));
        reply
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();

    let acquired = cache(&temp)
        .acquire_as_served(
            &request("mislabelled", server.url("/archive"), reviewed, 1),
            None,
            ZIP,
            &sink,
            &StopSignal::never(),
        )
        .unwrap();

    assert_eq!(
        std::fs::read(&acquired.artifact.archive_path).unwrap(),
        served
    );
    assert_eq!(server.finish().len(), 1);
}

/// An official link may lack its file only for a moment, as while a release is replaced, so a
/// changed download asks a missing file again; the pinned download keeps stopping at once.
#[test]
fn a_missing_changed_download_is_asked_for_again_before_it_counts() {
    let reviewed = b"the reviewed release";
    let served = zip_archive(b"the release that is back after a moment");
    let body = served.clone();
    let server = TestServer::start(2, move |index, _| match index {
        0 => Reply {
            status: 404,
            headers: Vec::new(),
            body: Vec::new(),
            advertised_length: Some(0),
        },
        1 => Reply::complete(&body, "\"back\""),
        _ => unreachable!(),
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let acquired = cache(&temp)
        .acquire_as_served(
            &request("back", server.url("/archive"), reviewed, 2),
            None,
            ZIP,
            &sink,
            &StopSignal::never(),
        )
        .unwrap();
    assert_eq!(
        std::fs::read(&acquired.artifact.archive_path).unwrap(),
        served
    );
    assert_eq!(server.finish().len(), 2);

    for status in [404, 410] {
        let server = TestServer::start(2, move |_, _| Reply {
            status,
            headers: Vec::new(),
            body: Vec::new(),
            advertised_length: Some(0),
        });
        let temp = TempDir::new().unwrap();
        let result = cache(&temp).acquire_as_served(
            &request("gone", server.url("/archive"), reviewed, 2),
            None,
            ZIP,
            &sink,
            &StopSignal::never(),
        );
        assert!(
            matches!(result, Err(AcquireError::HttpStatus { status: refused, .. }) if refused == status),
            "{result:?}"
        );
        assert_eq!(server.finish().len(), 2);
    }
}
