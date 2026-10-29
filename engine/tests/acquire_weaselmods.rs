//! Acquisition of Weasel Mods download-manager packages, against a local fixture server that
//! answers like `downloads.weaselmods.net`. Only the ignored live test contacts the real host.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use bg_engine::acquire::{
    AcquireError, ArchiveFormat, ArtifactCache, CacheDisposition, DownloadRequest, DownloadRoute,
    ServedArtifact, ServedRedirects, ServedRules, StopSignal, DEFAULT_DOWNLOAD_ATTEMPTS,
};
use bg_engine::events::ChannelSink;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const PACKAGE_PATH: &str = "/download/bethany/?wpdmdl=751";

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
    /// `None` omits `Content-Length` and ends the body by closing the connection.
    advertised_length: Option<usize>,
}

impl Reply {
    /// What the download manager answers for a package: an anonymous ZIP attachment.
    fn package(body: &[u8], disposition_filename: &str) -> Self {
        Self {
            status: 200,
            headers: vec![
                (
                    "Set-Cookie",
                    "__wpdm_client=d87a5592142396cb9c8e88148a902526; path=/; secure; HttpOnly"
                        .to_owned(),
                ),
                ("Content-Description", "File Transfer".to_owned()),
                ("Content-Type", "application/zip".to_owned()),
                (
                    "Content-disposition",
                    format!("attachment;filename=\"{disposition_filename}\""),
                ),
                ("Content-Transfer-Encoding", "binary".to_owned()),
            ],
            body: body.to_vec(),
            advertised_length: Some(body.len()),
        }
    }

    fn redirect(location: String) -> Self {
        Self {
            status: 302,
            headers: vec![("Location", location)],
            body: Vec::new(),
            advertised_length: Some(0),
        }
    }

    fn with_header(mut self, name: &'static str, value: &str) -> Self {
        self.headers.push((name, value.to_owned()));
        self
    }

    fn without_header(mut self, name: &str) -> Self {
        self.headers
            .retain(|(header, _)| !header.eq_ignore_ascii_case(name));
        self
    }
}

struct TestServer {
    base_url: String,
    requests: Arc<Mutex<Vec<SeenRequest>>>,
    handle: Option<JoinHandle<()>>,
}

impl TestServer {
    /// Answers exactly `request_count` requests, then stops accepting.
    fn start<F>(request_count: usize, handler: F) -> Self
    where
        F: Fn(usize, &SeenRequest, &str) -> Reply + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let address = listener.local_addr().expect("local test server address");
        let base_url = format!("http://{address}");
        listener
            .set_nonblocking(true)
            .expect("configure local test server");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let thread_requests = Arc::clone(&requests);
        let thread_base = base_url.clone();
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
                let mut headers = BTreeMap::new();
                for line in lines {
                    let line = line.expect("read request header");
                    if line.is_empty() {
                        break;
                    }
                    let (name, value) = line.split_once(':').expect("valid request header");
                    headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
                }
                let seen = SeenRequest { path, headers };
                thread_requests
                    .lock()
                    .expect("request log mutex")
                    .push(seen.clone());
                let reply = handler(index, &seen, &thread_base);
                let reason = match reply.status {
                    200 => "OK",
                    302 => "Found",
                    _ => "Test Response",
                };
                write!(stream, "HTTP/1.1 {} {}\r\n", reply.status, reason)
                    .expect("write response status");
                for (name, value) in &reply.headers {
                    write!(stream, "{name}: {value}\r\n").expect("write response header");
                }
                if let Some(length) = reply.advertised_length {
                    write!(stream, "Content-Length: {length}\r\n").expect("write response length");
                }
                write!(stream, "Connection: close\r\n\r\n").expect("finish response headers");
                // The client may already have refused the response and closed its side.
                let _ = stream.write_all(&reply.body);
                let _ = stream.flush();
                let _ = stream.shutdown(Shutdown::Both);
            }
        });
        Self {
            base_url,
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

fn package_request(url: String, bytes: &[u8], expected_filename: &str) -> DownloadRequest {
    DownloadRequest {
        request_id: "bethany-1.03".to_owned(),
        url,
        expected_length: bytes.len() as u64,
        expected_sha256: sha256(bytes),
        redirect_hosts: Vec::new(),
        max_attempts: DEFAULT_DOWNLOAD_ATTEMPTS,
        route: DownloadRoute::WeaselmodsDownload {
            expected_filename: expected_filename.to_owned(),
        },
    }
}

fn cache(temp: &TempDir) -> ArtifactCache {
    ArtifactCache::open(temp.path().join("cache")).expect("open artifact cache")
}

/// Neither a published archive nor a partial download is left behind.
#[track_caller]
fn assert_nothing_kept(temp: &TempDir, request: &DownloadRequest) {
    let cache = temp.path().join("cache");
    let digest = &request.expected_sha256;
    assert!(!cache
        .join(format!("sha256/{}/{digest}.archive", &digest[..2]))
        .exists());
    assert!(!cache
        .join(format!("partial/{}.part", request.request_id))
        .exists());
}

fn archive_bytes() -> Vec<u8> {
    // The route checks headers only; archive layout is checked later by extraction.
    b"PK\x03\x04 synthetic package archive".to_vec()
}

/// A complete ZIP archive, as a changed download must be to be accepted.
fn zip_archive(text: &[u8]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    writer
        .start_file(
            "bethany/readme.txt",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    writer.write_all(text).unwrap();
    writer.finish().unwrap().into_inner()
}

#[test]
fn downloads_a_package_anonymously_and_checks_the_served_name() {
    let bytes = archive_bytes();
    let served = bytes.clone();
    let server = TestServer::start(1, move |_, _, _| {
        Reply::package(&served, "Bethany-v1.03.zip")
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let request = package_request(server.url(PACKAGE_PATH), &bytes, "Bethany-v1.03.zip");

    let acquired = cache(&temp).acquire(&request, &sink).unwrap();

    assert_eq!(acquired.disposition, CacheDisposition::Downloaded);
    assert_eq!(std::fs::read(&acquired.archive_path).unwrap(), bytes);
    assert_eq!(acquired.metadata.final_url, server.url(PACKAGE_PATH));
    let seen = server.finish();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].path, PACKAGE_PATH);
    for header in ["cookie", "authorization", "range", "referer"] {
        assert!(
            !seen[0].headers.contains_key(header),
            "request sent {header}: {:?}",
            seen[0].headers
        );
    }
}

#[test]
fn cookies_from_an_earlier_response_are_never_sent_back() {
    let bytes = archive_bytes();
    let served = bytes.clone();
    let server = TestServer::start(2, move |index, _, _| match index {
        0 => Reply {
            status: 503,
            headers: vec![(
                "Set-Cookie",
                "__wpdm_client=d87a5592142396cb9c8e88148a902526; path=/".to_owned(),
            )],
            body: Vec::new(),
            advertised_length: Some(0),
        },
        _ => Reply::package(&served, "Bethany-v1.03.zip"),
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();

    cache(&temp)
        .acquire(
            &package_request(server.url(PACKAGE_PATH), &bytes, "Bethany-v1.03.zip"),
            &sink,
        )
        .unwrap();

    let seen = server.finish();
    assert_eq!(seen.len(), 2);
    assert!(seen
        .iter()
        .all(|request| !request.headers.contains_key("cookie")));
}

#[test]
fn a_percent_encoded_served_name_is_decoded_like_a_browser_saves_it() {
    let bytes = archive_bytes();
    let served = bytes.clone();
    let server = TestServer::start(1, move |_, _, _| {
        Reply::package(&served, "Lanna%20v1.8.zip")
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let request = package_request(
        server.url("/download/lanna/?wpdmdl=437"),
        &bytes,
        "Lanna v1.8.zip",
    );

    let acquired = cache(&temp).acquire(&request, &sink).unwrap();

    assert_eq!(std::fs::read(&acquired.archive_path).unwrap(), bytes);
    assert_eq!(server.finish()[0].path, "/download/lanna/?wpdmdl=437");
}

#[test]
fn a_different_or_doubly_encoded_served_name_is_refused() {
    for served_name in ["Lanna%2520v1.8.zip", "Lanna v1.9.zip", "Lanna%v1.8.zip"] {
        let bytes = archive_bytes();
        let served = bytes.clone();
        let name = served_name.to_owned();
        let server = TestServer::start(1, move |_, _, _| Reply::package(&served, &name));
        let temp = TempDir::new().unwrap();
        let (sink, _) = ChannelSink::unbounded();
        let request = package_request(
            server.url("/download/lanna/?wpdmdl=437"),
            &bytes,
            "Lanna v1.8.zip",
        );

        let result = cache(&temp).acquire(&request, &sink);

        assert!(
            matches!(&result, Err(AcquireError::UnexpectedResponse { message, .. }) if message.contains("filename")),
            "{served_name}: {result:?}"
        );
        assert_nothing_kept(&temp, &request);
        assert_eq!(server.finish().len(), 1, "{served_name} was retried");
    }
}

#[test]
fn a_wrong_hash_is_refused_and_nothing_is_cached() {
    let pinned = archive_bytes();
    let mut served = pinned.clone();
    *served.last_mut().unwrap() ^= 0xff;
    let server = TestServer::start(1, move |_, _, _| {
        Reply::package(&served, "Bethany-v1.03.zip")
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let request = package_request(server.url(PACKAGE_PATH), &pinned, "Bethany-v1.03.zip");

    let result = cache(&temp).acquire(&request, &sink);

    assert!(
        matches!(result, Err(AcquireError::HashMismatch { ref expected, .. }) if *expected == request.expected_sha256),
        "{result:?}"
    );
    assert_nothing_kept(&temp, &request);
    server.finish();
}

#[test]
fn a_wrong_length_is_refused_before_the_body_is_kept() {
    let pinned = archive_bytes();
    let mut served = pinned.clone();
    served.extend_from_slice(b" and more");
    let server = TestServer::start(1, move |_, _, _| {
        Reply::package(&served, "Bethany-v1.03.zip")
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let request = package_request(server.url(PACKAGE_PATH), &pinned, "Bethany-v1.03.zip");

    let result = cache(&temp).acquire(&request, &sink);

    assert!(
        matches!(result, Err(AcquireError::LengthMismatch { expected, actual })
            if expected == pinned.len() as u64 && actual == pinned.len() as u64 + 9),
        "{result:?}"
    );
    assert_nothing_kept(&temp, &request);
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn an_html_page_instead_of_the_zip_fails_at_once_without_keeping_it() {
    let pinned = archive_bytes();
    let page = b"<!DOCTYPE html><html><body>Download link expired</body></html>".to_vec();
    let server = TestServer::start(1, move |_, _, _| Reply {
        status: 200,
        headers: vec![("Content-Type", "text/html; charset=UTF-8".to_owned())],
        advertised_length: Some(page.len()),
        body: page.clone(),
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let request = package_request(server.url(PACKAGE_PATH), &pinned, "Bethany-v1.03.zip");

    let result = cache(&temp).acquire(&request, &sink);

    assert!(
        matches!(&result, Err(AcquireError::UnexpectedResponse { message, .. }) if message.contains("text/html")),
        "{result:?}"
    );
    assert_nothing_kept(&temp, &request);
    // Fatal, so the five-attempt budget is not spent on it.
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn a_zip_response_without_its_declared_name_or_length_is_refused() {
    let bytes = archive_bytes();
    for (missing, reply) in [
        (
            "Content-disposition",
            Reply::package(&bytes, "Bethany-v1.03.zip").without_header("Content-disposition"),
        ),
        (
            "Content-Length",
            Reply {
                advertised_length: None,
                ..Reply::package(&bytes, "Bethany-v1.03.zip")
            },
        ),
        (
            "Content-Type",
            Reply::package(&bytes, "Bethany-v1.03.zip").without_header("Content-Type"),
        ),
        (
            "attachment",
            Reply::package(&bytes, "Bethany-v1.03.zip")
                .without_header("Content-disposition")
                .with_header(
                    "Content-Disposition",
                    "inline; filename=\"Bethany-v1.03.zip\"",
                ),
        ),
    ] {
        let server = TestServer::start(1, move |_, _, _| reply.clone());
        let temp = TempDir::new().unwrap();
        let (sink, _) = ChannelSink::unbounded();
        let request = package_request(server.url(PACKAGE_PATH), &bytes, "Bethany-v1.03.zip");

        let result = cache(&temp).acquire(&request, &sink);

        assert!(
            matches!(&result, Err(AcquireError::UnexpectedResponse { .. })),
            "missing {missing}: {result:?}"
        );
        assert_nothing_kept(&temp, &request);
        assert_eq!(server.finish().len(), 1);
    }
}

#[test]
fn a_redirect_fails_unless_the_artifact_lists_its_host() {
    let bytes = archive_bytes();

    // No reviewed host: even a redirect on the same host fails before it is followed.
    let server = TestServer::start(1, |_, _, base| {
        Reply::redirect(format!("{base}/wp-content/uploads/Bethany-v1.03.zip"))
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let request = package_request(server.url(PACKAGE_PATH), &bytes, "Bethany-v1.03.zip");
    let result = cache(&temp).acquire(&request, &sink);
    assert!(
        matches!(result, Err(AcquireError::UnreviewedRedirect { .. })),
        "{result:?}"
    );
    assert_nothing_kept(&temp, &request);
    assert_eq!(server.finish().len(), 1);

    // A reviewed host is followed, and the served archive is still checked.
    let served = bytes.clone();
    let server = TestServer::start(2, move |index, _, base| match index {
        0 => Reply::redirect(format!("{base}/wp-content/uploads/Bethany-v1.03.zip")),
        _ => Reply::package(&served, "Bethany-v1.03.zip"),
    });
    let temp = TempDir::new().unwrap();
    let request = DownloadRequest {
        redirect_hosts: vec!["127.0.0.1".to_owned()],
        ..package_request(server.url(PACKAGE_PATH), &bytes, "Bethany-v1.03.zip")
    };
    let acquired = cache(&temp).acquire(&request, &sink).unwrap();
    assert_eq!(
        acquired.metadata.final_url,
        server.url("/wp-content/uploads/Bethany-v1.03.zip")
    );
    let seen = server.finish();
    assert_eq!(seen[1].path, "/wp-content/uploads/Bethany-v1.03.zip");
}

#[test]
fn a_redirect_to_an_unlisted_host_fails_before_it_is_contacted() {
    let bytes = archive_bytes();
    let server = TestServer::start(1, |_, _, base| {
        // The same server under another host name, which the artifact does not list.
        let other_host = base.replace("127.0.0.1", "localhost");
        Reply::redirect(format!("{other_host}/download/bethany/?wpdmdl=751"))
    });
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let request = DownloadRequest {
        redirect_hosts: vec!["127.0.0.1".to_owned()],
        ..package_request(server.url(PACKAGE_PATH), &bytes, "Bethany-v1.03.zip")
    };

    let result = cache(&temp).acquire(&request, &sink);

    assert!(
        matches!(&result, Err(AcquireError::UnreviewedRedirect { to, .. }) if to.contains("localhost")),
        "{result:?}"
    );
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn a_link_on_another_host_or_of_another_shape_is_refused_before_any_connection() {
    let bytes = archive_bytes();
    // Started but expecting no request: any connection would show up in its log.
    let server = TestServer::start(0, |_, _, _| unreachable!());
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    for url in [
        "https://example.invalid/download/bethany/?wpdmdl=751".to_owned(),
        "https://downloads.weaselmods.net.example.invalid/download/bethany/?wpdmdl=751".to_owned(),
        "https://weaselmods.net/download/bethany/?wpdmdl=751".to_owned(),
        "http://downloads.weaselmods.net/download/bethany/?wpdmdl=751".to_owned(),
        "https://downloads.weaselmods.net/download/bethany/?wpdmdl=751&refresh=6adad8d9a8013"
            .to_owned(),
        server.url("/wp-content/uploads/Bethany-v1.03.zip"),
        server.url("/download/bethany/"),
    ] {
        let request = package_request(url.clone(), &bytes, "Bethany-v1.03.zip");
        let result = cache(&temp).acquire(&request, &sink);
        assert!(
            matches!(
                result,
                Err(AcquireError::InvalidUrl { .. } | AcquireError::InsecureUrl { .. })
            ),
            "{url}: {result:?}"
        );
    }
    for unusable in ["", "../Bethany-v1.03.zip", "Bethany-v1.03.zip."] {
        let request = package_request(server.url(PACKAGE_PATH), &bytes, unusable);
        let result = cache(&temp).acquire(&request, &sink);
        assert!(
            matches!(result, Err(AcquireError::InvalidRequest(_))),
            "{unusable:?}: {result:?}"
        );
    }
    assert!(server.finish().is_empty());
}

/// What an installation takes from a package link: whatever it serves.
fn acquire_as_served(
    temp: &TempDir,
    request: &DownloadRequest,
) -> Result<ServedArtifact, AcquireError> {
    let (sink, _) = ChannelSink::unbounded();
    cache(temp).acquire_as_served(
        request,
        None,
        ServedRules {
            format: ArchiveFormat::Zip,
            redirects: ServedRedirects::AnyHttpsHost,
        },
        &sink,
        &StopSignal::never(),
    )
}

#[test]
fn an_installation_takes_the_newer_package_a_link_now_serves() {
    // As for Snowytoes Hamlet on 2026-10-30: the link of the reviewed 1.6 serves 1.7.
    let pinned = archive_bytes();
    let newer = zip_archive(b"the next release");
    let body = newer.clone();
    let server = TestServer::start(1, move |_, _, _| Reply::package(&body, "Bethany-v1.04.zip"));
    let temp = TempDir::new().unwrap();
    let request = package_request(server.url(PACKAGE_PATH), &pinned, "Bethany-v1.03.zip");

    let served = acquire_as_served(&temp, &request).unwrap();

    assert!(!served.matches_pin(&request));
    assert_eq!(served.served_filename.as_deref(), Some("Bethany-v1.04.zip"));
    assert_eq!(served.artifact.metadata.length, newer.len() as u64);
    assert_eq!(served.artifact.metadata.sha256, sha256(&newer));
    assert_eq!(std::fs::read(&served.artifact.archive_path).unwrap(), newer);
    assert!(served.unreviewed_hosts.is_empty());
    // The normal acquisition of the pin still refuses that response.
    assert_eq!(server.finish().len(), 1);
    let server = TestServer::start(1, move |_, _, _| {
        Reply::package(&newer, "Bethany-v1.04.zip")
    });
    let (sink, _) = ChannelSink::unbounded();
    let strict = cache(&TempDir::new().unwrap()).acquire(
        &DownloadRequest {
            url: server.url(PACKAGE_PATH),
            ..request
        },
        &sink,
    );
    assert!(
        matches!(&strict, Err(AcquireError::UnexpectedResponse { message, .. }) if message.contains("filename")),
        "{strict:?}"
    );
    server.finish();
}

#[test]
fn an_installation_follows_a_package_link_to_another_host_and_reports_it() {
    let bytes = archive_bytes();
    let body = bytes.clone();
    let server = TestServer::start(2, move |index, _, base| match index {
        0 => Reply::redirect(format!(
            "{}/wp-content/uploads/Bethany-v1.03.zip",
            base.replace("127.0.0.1", "localhost")
        )),
        _ => Reply::package(&body, "Bethany-v1.03.zip"),
    });
    let temp = TempDir::new().unwrap();
    let request = package_request(server.url(PACKAGE_PATH), &bytes, "Bethany-v1.03.zip");

    let served = acquire_as_served(&temp, &request).unwrap();

    assert!(served.matches_pin(&request), "the file is the reviewed one");
    assert_eq!(served.unreviewed_hosts, ["localhost"]);
    assert!(served
        .artifact
        .metadata
        .final_url
        .starts_with("http://localhost:"));
    assert_eq!(
        server.finish()[1].path,
        "/wp-content/uploads/Bethany-v1.03.zip"
    );
}

#[test]
fn an_installation_asks_again_for_a_page_served_instead_of_the_package_and_keeps_none() {
    // A changed package has other headers than the reviewed one, so the page is not refused by
    // its headers here. It is no ZIP archive, though: it is asked for again, refused on the
    // last attempt, and nothing of it is kept. The installer then leaves the mod out.
    let pinned = archive_bytes();
    let page = b"<!DOCTYPE html><html><body>Download link expired</body></html>".to_vec();
    let body = page.clone();
    let server = TestServer::start(2, move |_, _, _| Reply {
        status: 200,
        headers: vec![("Content-Type", "text/html; charset=UTF-8".to_owned())],
        advertised_length: Some(body.len()),
        body: body.clone(),
    });
    let temp = TempDir::new().unwrap();
    let request = DownloadRequest {
        // The first repeat follows at once; later ones would wait.
        max_attempts: 2,
        ..package_request(server.url(PACKAGE_PATH), &pinned, "Bethany-v1.03.zip")
    };

    let result = acquire_as_served(&temp, &request);

    assert!(
        matches!(&result, Err(AcquireError::ServedNotArchive { problem, .. })
            if problem.starts_with("it does not begin like a ZIP archive but with the text \"<!DOCTYPE html>")),
        "{result:?}"
    );
    assert_nothing_kept(&temp, &request);
    assert_eq!(server.finish().len(), 2);
}

#[test]
fn an_installation_still_refuses_a_package_link_of_another_shape() {
    let bytes = archive_bytes();
    let temp = TempDir::new().unwrap();
    let request = package_request(
        "http://127.0.0.1:9/download/bethany/?wpdmdl=751&other=1".to_owned(),
        &bytes,
        "Bethany-v1.03.zip",
    );

    let result = acquire_as_served(&temp, &request);

    assert!(
        matches!(&result, Err(AcquireError::InvalidUrl { .. })),
        "{result:?}"
    );
}

/// Downloads one small package from the real host and checks it against the pin the IWD:EE
/// recipe draft recorded for it (Bethany 1.03, reviewed 2026-10-25).
///
/// Run with `cargo test -p chriz-bg-engine --test acquire_weaselmods -- --ignored`.
#[test]
#[ignore = "contacts downloads.weaselmods.net"]
fn live_package_link_serves_its_pinned_archive() {
    let temp = TempDir::new().unwrap();
    let (sink, _) = ChannelSink::unbounded();
    let request = DownloadRequest {
        request_id: "bethany-1.03".to_owned(),
        url: "https://downloads.weaselmods.net/download/bethany/?wpdmdl=751".to_owned(),
        expected_length: 3_759_411,
        expected_sha256: "b18ca9501ac37c5d5db6c4e25850a3c877d17cd232545268d0bf9c49684d5f32"
            .to_owned(),
        redirect_hosts: Vec::new(),
        max_attempts: DEFAULT_DOWNLOAD_ATTEMPTS,
        route: DownloadRoute::WeaselmodsDownload {
            expected_filename: "Bethany-v1.03.zip".to_owned(),
        },
    };

    let acquired = cache(&temp).acquire(&request, &sink).unwrap();

    assert_eq!(acquired.disposition, CacheDisposition::Downloaded);
    assert_eq!(acquired.metadata.length, 3_759_411);
    assert_eq!(acquired.metadata.sha256, request.expected_sha256);
    assert_eq!(acquired.metadata.final_url, request.url);
    assert!(Path::new(&acquired.archive_path).is_file());
}
