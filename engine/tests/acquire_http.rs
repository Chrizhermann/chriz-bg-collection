use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use bg_engine::acquire::{
    validate_redirect_target, AcquireError, ArtifactCache, CacheDisposition, DownloadRequest,
};
use bg_engine::events::{ChannelSink, EngineEvent};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

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
                writeln!(
                    stream,
                    "Content-Length: {}\r",
                    reply.advertised_length.unwrap_or(reply.body.len())
                )
                .expect("write response length");
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
        max_attempts,
    }
}

fn cache(temp: &TempDir) -> ArtifactCache {
    ArtifactCache::open(temp.path().join("cache")).expect("open artifact cache")
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
