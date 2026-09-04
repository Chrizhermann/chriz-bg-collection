use std::io::Cursor;
use std::thread;
use std::time::Duration;

use bg_engine::events::ChannelSink;
use bg_engine::radar::{
    check_latest_from, install, status, RadarError, RadarRelease, RadarState, RADAR_EXECUTABLE,
    RADAR_INSTALL_DIRECTORY,
};
use sevenz_rust2::compress_to_path;
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use tiny_http::{Header, Response, Server, StatusCode};

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn one_shot_server(bytes: Vec<u8>, content_type: &str) -> (String, thread::JoinHandle<()>) {
    let server = Server::http("127.0.0.1:0").unwrap();
    let address = server.server_addr().to_ip().unwrap();
    let content_type = content_type.to_owned();
    let handle = thread::spawn(move || {
        let request = server
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .expect("request");
        let response = Response::new(
            StatusCode(200),
            vec![Header::from_bytes("Content-Type", content_type).unwrap()],
            Cursor::new(bytes.clone()),
            Some(bytes.len()),
            None,
        );
        request.respond(response).unwrap();
    });
    (format!("http://{address}/payload"), handle)
}

fn release(tag: &str, asset_url: String, archive: &[u8]) -> RadarRelease {
    RadarRelease {
        release_id: if tag == "2.1.0.0" { 381_826_828 } else { 1 },
        tag: tag.to_owned(),
        published_at: "2026-09-03T07:41:37Z".to_owned(),
        asset_id: 543_014_181,
        asset_name: "BG.Radar.Overlay.7z.7z".to_owned(),
        asset_url,
        asset_length: archive.len() as u64,
        sha256: sha256(archive),
    }
}

fn radar_archive(temp: &TempDir, name: &str, exe: &[u8], include_locale: bool) -> Vec<u8> {
    let source = temp.path().join(format!("{name}-source"));
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join(RADAR_EXECUTABLE), exe).unwrap();
    if include_locale {
        std::fs::create_dir_all(source.join("Locales")).unwrap();
        std::fs::write(source.join("Locales/en_us.txt"), b"English").unwrap();
    }
    let archive = temp.path().join(format!("{name}.7z"));
    compress_to_path(&source, &archive).unwrap();
    std::fs::read(archive).unwrap()
}

fn managed_layout(temp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let managed = temp.path().join("managed");
    let game = managed.join("game");
    std::fs::create_dir_all(&game).unwrap();
    (managed, game)
}

#[test]
fn latest_release_uses_the_real_repository_shape_and_advertised_digest() {
    let body = br#"{
      "id": 381826828,
      "tag_name": "2.1.0.0",
      "published_at": "2026-09-03T07:41:37Z",
      "draft": false,
      "prerelease": false,
      "assets": [{
        "id": 543014181,
        "name": "BG.Radar.Overlay.7z.7z",
        "size": 68133036,
        "browser_download_url": "https://github.com/tapahob/BG2RadarOverlay/releases/download/2.1.0.0/BG.Radar.Overlay.7z.7z",
        "digest": "sha256:8cd0348011638398d14d005fdda620032ca5ffa8833c86586d209229e3e567ce"
      }]
    }"#;
    let (url, server) = one_shot_server(body.to_vec(), "application/json");

    let latest = check_latest_from(&url).unwrap();
    server.join().unwrap();

    assert_eq!(latest.release_id, 381_826_828);
    assert_eq!(latest.tag, "2.1.0.0");
    assert_eq!(latest.asset_id, 543_014_181);
    assert_eq!(latest.asset_length, 68_133_036);
    assert_eq!(
        latest.sha256,
        "8cd0348011638398d14d005fdda620032ca5ffa8833c86586d209229e3e567ce"
    );
}

#[test]
fn first_install_extracts_the_verified_7z_and_reports_up_to_date() {
    let temp = TempDir::new().unwrap();
    let archive = radar_archive(&temp, "v1", b"radar-v1", true);
    let (url, server) = one_shot_server(archive.clone(), "application/octet-stream");
    let latest = release("2.1.0.0", url, &archive);
    let (managed, game) = managed_layout(&temp);
    let (sink, _) = ChannelSink::unbounded();

    let installed = install(temp.path().join("cache"), &managed, &game, &latest, &sink).unwrap();
    server.join().unwrap();

    let executable = game.join(RADAR_INSTALL_DIRECTORY).join(RADAR_EXECUTABLE);
    assert_eq!(installed.version, "2.1.0.0");
    assert_eq!(installed.executable, executable);
    assert!(installed.downloaded);
    assert_eq!(std::fs::read(&installed.executable).unwrap(), b"radar-v1");
    assert!(game
        .join(RADAR_INSTALL_DIRECTORY)
        .join("Locales/en_us.txt")
        .is_file());
    assert!(managed
        .join(".chriz/addons/bg-radar-overlay.json")
        .is_file());

    let current = status(&managed, &game, Some(&latest)).unwrap();
    assert_eq!(current.state, RadarState::UpToDate);
    assert_eq!(current.installed_version.as_deref(), Some("2.1.0.0"));
    assert_eq!(current.latest_version.as_deref(), Some("2.1.0.0"));
}

#[test]
fn update_replaces_owned_files_removes_retired_ones_and_preserves_config() {
    let temp = TempDir::new().unwrap();
    let first_archive = radar_archive(&temp, "v1", b"radar-v1", true);
    let (first_url, first_server) =
        one_shot_server(first_archive.clone(), "application/octet-stream");
    let first = release("2.0.8.0", first_url, &first_archive);
    let (managed, game) = managed_layout(&temp);
    let (sink, _) = ChannelSink::unbounded();
    install(temp.path().join("cache"), &managed, &game, &first, &sink).unwrap();
    first_server.join().unwrap();

    let install_root = game.join(RADAR_INSTALL_DIRECTORY);
    std::fs::write(install_root.join("config.cfg"), b"user settings").unwrap();

    let second_archive = radar_archive(&temp, "v2", b"radar-v2", false);
    let (second_url, second_server) =
        one_shot_server(second_archive.clone(), "application/octet-stream");
    let second = release("2.1.0.0", second_url, &second_archive);
    let updated = install(temp.path().join("cache"), &managed, &game, &second, &sink).unwrap();
    second_server.join().unwrap();

    assert_eq!(std::fs::read(updated.executable).unwrap(), b"radar-v2");
    assert_eq!(
        std::fs::read(install_root.join("config.cfg")).unwrap(),
        b"user settings"
    );
    assert!(!install_root.join("Locales/en_us.txt").exists());
}

#[test]
fn modified_owned_file_blocks_update_before_download() {
    let temp = TempDir::new().unwrap();
    let archive = radar_archive(&temp, "v1", b"radar-v1", false);
    let (url, server) = one_shot_server(archive.clone(), "application/octet-stream");
    let first = release("2.0.8.0", url, &archive);
    let (managed, game) = managed_layout(&temp);
    let (sink, _) = ChannelSink::unbounded();
    install(temp.path().join("cache"), &managed, &game, &first, &sink).unwrap();
    server.join().unwrap();

    let executable = game.join(RADAR_INSTALL_DIRECTORY).join(RADAR_EXECUTABLE);
    std::fs::write(&executable, b"locally modified").unwrap();
    let next = RadarRelease {
        tag: "2.1.0.0".to_owned(),
        asset_url: "http://127.0.0.1:9/radar.7z".to_owned(),
        ..first
    };

    let error = install(temp.path().join("cache"), &managed, &game, &next, &sink).unwrap_err();
    assert!(matches!(error, RadarError::ModifiedFiles { .. }));
    assert_eq!(std::fs::read(executable).unwrap(), b"locally modified");
    assert_eq!(
        status(&managed, &game, Some(&next)).unwrap().state,
        RadarState::Modified
    );
}

#[test]
fn install_refuses_an_unmanaged_overlay_collision() {
    let temp = TempDir::new().unwrap();
    let archive = radar_archive(&temp, "v1", b"new", false);
    let latest = release(
        "2.1.0.0",
        "http://127.0.0.1:9/radar.7z".to_owned(),
        &archive,
    );
    let (managed, game) = managed_layout(&temp);
    let install_root = game.join(RADAR_INSTALL_DIRECTORY);
    std::fs::create_dir_all(&install_root).unwrap();
    std::fs::write(install_root.join(RADAR_EXECUTABLE), b"unmanaged").unwrap();
    let (sink, _) = ChannelSink::unbounded();

    let error = install(temp.path().join("cache"), &managed, &game, &latest, &sink).unwrap_err();
    assert!(matches!(error, RadarError::UnmanagedInstall { .. }));
    assert_eq!(
        std::fs::read(install_root.join(RADAR_EXECUTABLE)).unwrap(),
        b"unmanaged"
    );
}

#[test]
fn install_refuses_an_empty_symlinked_overlay_directory_before_download() {
    let temp = TempDir::new().unwrap();
    let archive = radar_archive(&temp, "v1", b"new", false);
    let latest = release(
        "2.1.0.0",
        "http://127.0.0.1:9/radar.7z".to_owned(),
        &archive,
    );
    let (managed, game) = managed_layout(&temp);
    let outside = temp.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    let install_root = game.join(RADAR_INSTALL_DIRECTORY);
    #[cfg(windows)]
    if let Err(error) = std::os::windows::fs::symlink_dir(&outside, &install_root) {
        if matches!(
            error.kind(),
            std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::Unsupported
        ) {
            return;
        }
        panic!("create test directory symlink: {error}");
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, &install_root).unwrap();
    let (sink, _) = ChannelSink::unbounded();

    let error = install(temp.path().join("cache"), &managed, &game, &latest, &sink).unwrap_err();
    assert!(matches!(error, RadarError::InvalidRoots(_)));
}

#[test]
fn same_tag_with_replaced_release_asset_is_an_update() {
    let temp = TempDir::new().unwrap();
    let archive = radar_archive(&temp, "v1", b"radar-v1", false);
    let (url, server) = one_shot_server(archive.clone(), "application/octet-stream");
    let first = release("2.1.0.0", url, &archive);
    let (managed, game) = managed_layout(&temp);
    let (sink, _) = ChannelSink::unbounded();
    install(temp.path().join("cache"), &managed, &game, &first, &sink).unwrap();
    server.join().unwrap();

    let latest = RadarRelease {
        asset_id: first.asset_id + 1,
        sha256: "ab".repeat(32),
        ..first
    };
    assert_eq!(
        status(&managed, &game, Some(&latest)).unwrap().state,
        RadarState::UpdateAvailable
    );
}
