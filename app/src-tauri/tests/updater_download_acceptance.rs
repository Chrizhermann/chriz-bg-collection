use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::json;
use tauri_plugin_updater::{Error as UpdaterError, UpdaterExt};

struct LocalFeed {
    endpoint: String,
    server: JoinHandle<Vec<String>>,
}

fn spawn_feed(version: &str, signature: &str, artifact: Vec<u8>) -> LocalFeed {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback update server");
    let address = listener.local_addr().expect("read loopback server address");
    let endpoint = format!("http://{address}/latest.json");
    let artifact_url = format!("http://{address}/update.exe");
    let feed = serde_json::to_vec(&json!({
        "version": version,
        "notes": "Local signed-updater acceptance fixture.",
        "pub_date": "2026-09-05T00:00:00Z",
        "platforms": {
            "windows-x86_64": {
                "url": artifact_url,
                "signature": signature,
            }
        }
    }))
    .expect("serialize local update feed");

    let server = thread::spawn(move || {
        let mut paths = Vec::new();
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().expect("accept updater request");
            let path = {
                let mut reader = BufReader::new(&mut stream);
                let mut first_line = String::new();
                reader
                    .read_line(&mut first_line)
                    .expect("read request line");
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).expect("read request header");
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                first_line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("/")
                    .to_owned()
            };
            paths.push(path.clone());
            let (status, content_type, body) = match path.as_str() {
                "/latest.json" => ("200 OK", "application/json", feed.as_slice()),
                "/update.exe" => ("200 OK", "application/octet-stream", artifact.as_slice()),
                _ => ("404 Not Found", "text/plain", b"not found".as_slice()),
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response headers");
            stream.write_all(body).expect("write response body");
            stream.flush().expect("flush response");
        }
        paths
    });

    LocalFeed { endpoint, server }
}

fn acceptance_input(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name} for the ignored acceptance test"))
}

#[test]
#[ignore = "requires an explicitly selected, locally built signed NSIS artifact"]
fn real_tauri_updater_downloads_and_verifies_the_signed_nsis() {
    let setup_path = PathBuf::from(acceptance_input("CEBG_UPDATER_SETUP"));
    let version = acceptance_input("CEBG_UPDATER_VERSION");
    let current_version = std::env::var("CEBG_UPDATER_CURRENT_VERSION")
        .unwrap_or_else(|_| "0.1.0-alpha.2".to_owned());
    let setup = fs::read(&setup_path).expect("read signed NSIS setup");
    assert!(!setup.is_empty(), "signed NSIS setup must not be empty");
    let signature = fs::read_to_string(format!("{}.sig", setup_path.display()))
        .expect("read setup's adjacent Tauri signature");
    assert!(!signature.trim().is_empty(), "signature must not be empty");

    let mut context = tauri::generate_context!();
    context.package_info_mut().version = current_version.parse().expect("parse current version");
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context)
        .expect("build updater acceptance app without a native window");

    let valid_feed = spawn_feed(&version, signature.trim(), setup.clone());
    let valid_endpoint = valid_feed
        .endpoint
        .parse()
        .expect("parse loopback feed URL");
    let downloaded = tauri::async_runtime::block_on(async {
        let updater = app
            .updater_builder()
            .endpoints(vec![valid_endpoint])
            .expect("use loopback endpoint in this debug-only test")
            .no_proxy()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("build updater");
        let update = updater
            .check()
            .await
            .expect("check local signed feed")
            .expect("newer update should be offered");
        assert_eq!(update.current_version, current_version);
        assert_eq!(update.version, version);
        update
            .download(|_, _| {}, || {})
            .await
            .expect("download and verify actual signed NSIS")
    });
    assert_eq!(
        downloaded, setup,
        "verified bytes must equal the selected setup"
    );
    assert_eq!(
        valid_feed.server.join().expect("join valid feed server"),
        ["/latest.json", "/update.exe"]
    );

    let mut tampered = setup;
    let last = tampered.last_mut().expect("nonempty setup already checked");
    *last ^= 1;
    let tampered_feed = spawn_feed(&version, signature.trim(), tampered);
    let tampered_endpoint = tampered_feed
        .endpoint
        .parse()
        .expect("parse tampered loopback feed URL");
    let error = tauri::async_runtime::block_on(async {
        let updater = app
            .updater_builder()
            .endpoints(vec![tampered_endpoint])
            .expect("use loopback endpoint in this debug-only test")
            .no_proxy()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("build updater");
        let update = updater
            .check()
            .await
            .expect("check local tampered feed")
            .expect("feed still announces a newer update");
        update
            .download(|_, _| {}, || {})
            .await
            .expect_err("the real updater must reject changed bytes")
    });
    assert!(
        matches!(error, UpdaterError::Minisign(_)),
        "tampered artifact must fail in the Minisign verifier, got: {error}"
    );
    assert_eq!(
        tampered_feed
            .server
            .join()
            .expect("join tampered feed server"),
        ["/latest.json", "/update.exe"]
    );
}
