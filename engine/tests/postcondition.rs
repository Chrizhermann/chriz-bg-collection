use std::path::Path;

#[cfg(windows)]
use std::io;

use bg_engine::manifest::Postcondition;
use bg_engine::postcondition::verify;

fn markers(path: &str, required: &[&str], forbidden: &[&str], max_bytes: u64) -> Postcondition {
    Postcondition::TextFileMarkers {
        path: path.to_owned(),
        required: required.iter().map(|marker| (*marker).to_owned()).collect(),
        forbidden: forbidden
            .iter()
            .map(|marker| (*marker).to_owned())
            .collect(),
        max_bytes,
    }
}

#[track_caller]
fn assert_error_contains(root: &Path, postcondition: Postcondition, expected: &str) {
    let error = verify(root, &[postcondition]).unwrap_err();
    assert!(error.to_string().contains(expected), "{error}");
}

#[test]
fn exact_case_sensitive_byte_markers_pass_without_utf8_decoding() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("game");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("weidu.conf"), b"\xfflang_dir = en_US\n").unwrap();

    verify(
        &root,
        &[markers(
            "weidu.conf",
            &["lang_dir = en_US"],
            &["lang_dir = ko_KR"],
            4096,
        )],
    )
    .unwrap();

    assert_error_contains(
        &root,
        markers("weidu.conf", &["lang_dir = en_us"], &[], 4096),
        "required marker",
    );
}

#[test]
fn missing_required_or_present_forbidden_marker_fails() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("game");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("weidu.conf"), b"lang_dir = en_US\n").unwrap();

    assert_error_contains(
        &root,
        markers("weidu.conf", &["missing"], &[], 4096),
        "required marker",
    );
    assert_error_contains(
        &root,
        markers("weidu.conf", &[], &["lang_dir = en_US"], 4096),
        "forbidden marker",
    );
}

#[test]
fn unsafe_path_directory_and_oversized_file_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("game");
    std::fs::create_dir_all(root.join("directory")).unwrap();
    std::fs::write(root.join("large.txt"), b"12345").unwrap();
    std::fs::write(temp.path().join("outside.txt"), b"marker").unwrap();

    assert_error_contains(
        &root,
        markers("../outside.txt", &["marker"], &[], 4096),
        "unsafe",
    );
    assert_error_contains(
        &root,
        markers("override/NUL.txt", &["marker"], &[], 4096),
        "unsafe",
    );
    assert_error_contains(
        &root,
        markers("override/*.txt", &["marker"], &[], 4096),
        "unsafe",
    );
    assert_error_contains(
        &root,
        markers("large.txt", &["marker"], &["marker"], 4096),
        "unsafe",
    );
    assert_error_contains(&root, markers("large.txt", &["12345"], &[], 4), "unsafe");
    assert_error_contains(
        &root,
        markers("directory", &["marker"], &[], 4096),
        "regular file",
    );
    assert_error_contains(
        &root,
        markers("large.txt", &["1"], &[], 4),
        "exceeds max_bytes",
    );
}

#[test]
fn final_file_symlink_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("game");
    std::fs::create_dir(&root).unwrap();
    let outside = temp.path().join("outside.txt");
    std::fs::write(&outside, b"marker").unwrap();
    let link = root.join("linked.txt");
    if !create_file_symlink(&outside, &link) {
        return;
    }

    assert_error_contains(
        &root,
        markers("linked.txt", &["marker"], &[], 4096),
        "regular file",
    );
}

#[test]
fn intermediate_directory_symlink_cannot_escape_the_staged_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("game");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("marker.txt"), b"marker").unwrap();
    let link = root.join("linked");
    if !create_directory_symlink(&outside, &link) {
        return;
    }

    assert_error_contains(
        &root,
        markers("linked/marker.txt", &["marker"], &[], 4096),
        "escapes staged target",
    );
}

#[test]
fn staged_root_symlink_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let actual = temp.path().join("actual-game");
    std::fs::create_dir(&actual).unwrap();
    std::fs::write(actual.join("marker.txt"), b"marker").unwrap();
    let linked_root = temp.path().join("linked-game");
    if !create_directory_symlink(&actual, &linked_root) {
        return;
    }

    assert_error_contains(
        &linked_root,
        markers("marker.txt", &["marker"], &[], 4096),
        "non-symlink directory",
    );
}

#[cfg(unix)]
fn create_file_symlink(source: &Path, destination: &Path) -> bool {
    std::os::unix::fs::symlink(source, destination).unwrap();
    true
}

#[cfg(unix)]
fn create_directory_symlink(source: &Path, destination: &Path) -> bool {
    std::os::unix::fs::symlink(source, destination).unwrap();
    true
}

#[cfg(windows)]
fn create_file_symlink(source: &Path, destination: &Path) -> bool {
    create_windows_symlink(|| std::os::windows::fs::symlink_file(source, destination))
}

#[cfg(windows)]
fn create_directory_symlink(source: &Path, destination: &Path) -> bool {
    create_windows_symlink(|| std::os::windows::fs::symlink_dir(source, destination))
}

#[cfg(windows)]
fn create_windows_symlink(create: impl FnOnce() -> io::Result<()>) -> bool {
    match create() {
        Ok(()) => true,
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
            ) =>
        {
            false
        }
        Err(error) => panic!("failed to create test symlink: {error}"),
    }
}
