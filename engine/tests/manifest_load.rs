use std::ffi::OsString;
use std::io::ErrorKind;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use bg_engine::{error::EngineError, loader::SUPPORTED_SCHEMA, Manifest};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    root: PathBuf,
}

impl TestDir {
    fn new(test_name: &str) -> Self {
        loop {
            let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "chriz-bg-engine-manifest-load-{test_name}-{}-{sequence}",
                std::process::id()
            ));

            match std::fs::create_dir(&root) {
                Ok(()) => return Self { root },
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("failed to create {}: {error}", root.display()),
            }
        }
    }

    fn from_fixture(test_name: &str) -> Self {
        let temp = Self::new(test_name);
        let fixture = fixture_dir();
        std::fs::copy(
            fixture.join("collection.toml"),
            temp.path().join("collection.toml"),
        )
        .unwrap();
        for directory in ["artifacts", "mods", "presets"] {
            copy_dir(&fixture.join(directory), &temp.path().join(directory));
        }

        temp
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn canonical_path(&self) -> PathBuf {
        self.root.canonicalize().unwrap()
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest")
}

fn copy_dir(source: &Path, destination: &Path) {
    std::fs::create_dir(destination).unwrap();

    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if entry.file_type().unwrap().is_dir() {
            copy_dir(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path).unwrap();
        }
    }
}

#[cfg(unix)]
fn symlink_file(target: &Path, link: &Path) -> bool {
    std::os::unix::fs::symlink(target, link).unwrap();
    true
}

#[cfg(windows)]
fn symlink_file(target: &Path, link: &Path) -> bool {
    match std::os::windows::fs::symlink_file(target, link) {
        Ok(()) => true,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::PermissionDenied | ErrorKind::Unsupported
            ) =>
        {
            false
        }
        Err(error) => panic!(
            "failed to create test symlink {} -> {}: {error}",
            link.display(),
            target.display()
        ),
    }
}

fn create_case_distinct_copy(source: &Path, destination: &Path) -> bool {
    let contents = std::fs::read(source).unwrap();
    let mut destination_file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
    {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => return false,
        Err(error) => panic!(
            "failed to create case-distinct test file {}: {error}",
            destination.display()
        ),
    };
    destination_file.write_all(&contents).unwrap();
    true
}

#[cfg(unix)]
fn non_utf8_toml_name() -> OsString {
    use std::os::unix::ffi::OsStringExt;

    OsString::from_vec(vec![0xff, b'.', b't', b'o', b'm', b'l'])
}

#[cfg(windows)]
fn non_utf8_toml_name() -> OsString {
    use std::os::windows::ffi::OsStringExt;

    OsString::from_wide(&[
        0xd800,
        b'.' as u16,
        b't' as u16,
        b'o' as u16,
        b'm' as u16,
        b'l' as u16,
    ])
}

fn load_error(path: &Path) -> EngineError {
    match Manifest::load(path) {
        Ok(_) => panic!("expected loading {} to fail", path.display()),
        Err(error) => error,
    }
}

#[test]
fn loads_fixture_dir() {
    let root = fixture_dir();
    let noncanonical_root = root.join("mods").join("..");
    let manifest = Manifest::load(&noncanonical_root).unwrap();

    assert_eq!(manifest.root, root.canonicalize().unwrap());
    assert_eq!(
        manifest.conventional_mod_path("eet"),
        manifest.root.join("mods/eet.toml")
    );
    assert_eq!(manifest.collection.runs.len(), 2);
    assert_eq!(manifest.artifacts.len(), 2);
    assert_eq!(
        manifest.mods.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["eefixpack"]
    );
    assert_eq!(
        manifest
            .presets
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["recommended"]
    );
}

#[test]
fn missing_dir_is_io_error_with_path() {
    let temp = TestDir::new("missing-dir");
    let missing = temp.path().join("missing");
    let expected_path = missing.join("collection.toml");

    let error = load_error(&missing);

    match error {
        EngineError::Io { path, .. } => assert_eq!(path, expected_path),
        other => panic!("expected an I/O error, got {other:?}"),
    }
}

#[test]
fn parse_error_carries_path() {
    let temp = TestDir::from_fixture("parse-error");
    let mod_path = temp.path().join("mods/eefixpack.toml");
    std::fs::write(&mod_path, "id = [").unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::ManifestParse { path, .. } => {
            assert_eq!(path, temp.canonical_path().join("mods/eefixpack.toml"));
        }
        other => panic!("expected a manifest parse error, got {other:?}"),
    }
}

#[test]
fn rejects_unsupported_schema() {
    let temp = TestDir::from_fixture("unsupported-schema");
    let collection_path = temp.path().join("collection.toml");
    let collection = std::fs::read_to_string(&collection_path).unwrap();
    let schema_three = collection.replacen("schema = 2", "schema = 3", 1);
    assert_ne!(schema_three, collection, "fixture schema marker changed");
    std::fs::write(&collection_path, schema_three).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::UnsupportedSchema {
            path,
            found,
            supported,
        } => {
            assert_eq!(path, collection_path);
            assert_eq!(found, 3);
            assert_eq!(supported, SUPPORTED_SCHEMA);
            assert_eq!(SUPPORTED_SCHEMA, 2);
        }
        other => panic!("expected an unsupported schema error, got {other:?}"),
    }
}

#[test]
fn rejects_unsupported_schema_before_strict_future_field_validation() {
    let temp = TestDir::from_fixture("unsupported-schema-future-field");
    let collection_path = temp.path().join("collection.toml");
    let collection = std::fs::read_to_string(&collection_path).unwrap();
    let schema_three = collection.replacen("schema = 2", "schema = 3", 1);
    assert_ne!(schema_three, collection, "fixture schema marker changed");
    let future_collection = schema_three.replacen(
        "game_build = \"2.7.3.0\"",
        "game_build = \"2.7.3.0\"\nfuture_collection_setting = true",
        1,
    );
    assert_ne!(
        future_collection, schema_three,
        "fixture game-build marker changed"
    );
    std::fs::write(&collection_path, future_collection).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::UnsupportedSchema {
            path,
            found,
            supported,
        } => {
            assert_eq!(path, collection_path);
            assert_eq!(found, 3);
            assert_eq!(supported, SUPPORTED_SCHEMA);
        }
        other => panic!("expected an unsupported schema error, got {other:?}"),
    }
}

#[test]
fn rejects_id_file_stem_mismatch() {
    let temp = TestDir::from_fixture("id-stem-mismatch");
    let source = temp.path().join("mods/eefixpack.toml");
    let other = temp.path().join("mods/other.toml");
    std::fs::copy(source, &other).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::ModIdMismatch { path, id, stem } => {
            assert_eq!(path, temp.canonical_path().join("mods/other.toml"));
            assert_eq!(id, "eefixpack");
            assert_eq!(stem, "other");
        }
        other => panic!("expected a mod id mismatch error, got {other:?}"),
    }
}

#[test]
fn reports_stem_mismatch_before_duplicate_id() {
    let temp = TestDir::from_fixture("mismatch-before-duplicate");
    let first = temp.path().join("mods/eefixpack.toml");
    let second = temp.path().join("mods/zduplicate.toml");
    std::fs::copy(&first, &second).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::ModIdMismatch { path, id, stem } => {
            assert_eq!(id, "eefixpack");
            assert_eq!(path, temp.canonical_path().join("mods/zduplicate.toml"));
            assert_eq!(stem, "zduplicate");
        }
        other => panic!("expected a mod id mismatch error, got {other:?}"),
    }
}

#[test]
fn rejects_duplicate_id_from_case_distinct_toml_extensions() {
    let temp = TestDir::from_fixture("duplicate-id-case-distinct-extension");
    let lowercase = temp.path().join("mods/eefixpack.toml");
    let uppercase = temp.path().join("mods/eefixpack.TOML");
    if !create_case_distinct_copy(&lowercase, &uppercase) {
        // Case-insensitive filesystems cannot represent both directory entries.
        return;
    }

    let error = load_error(temp.path());

    match error {
        EngineError::DuplicateModId { id, first, second } => {
            assert_eq!(id, "eefixpack");
            let mut actual = vec![first, second];
            actual.sort();
            let root = temp.canonical_path().join("mods");
            let mut expected = vec![root.join("eefixpack.toml"), root.join("eefixpack.TOML")];
            expected.sort();
            assert_eq!(actual, expected);
        }
        other => panic!("expected a duplicate mod id error, got {other:?}"),
    }
}

#[test]
fn uppercase_toml_extension_is_loaded_and_checked() {
    let temp = TestDir::from_fixture("uppercase-toml-extension");
    let source = temp.path().join("mods/eefixpack.toml");
    let uppercase = temp.path().join("mods/EEFIXPACK.TOML");
    std::fs::rename(source, &uppercase).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::ModIdMismatch { path, id, stem } => {
            assert_eq!(path, temp.canonical_path().join("mods/EEFIXPACK.TOML"));
            assert_eq!(id, "eefixpack");
            assert_eq!(stem, "EEFIXPACK");
        }
        other => panic!("expected a mod id mismatch error, got {other:?}"),
    }
}

#[test]
fn case_variant_toml_extension_loads_but_conventional_path_stays_lowercase() {
    let temp = TestDir::from_fixture("case-variant-toml-extension");
    let source = temp.path().join("mods/eefixpack.toml");
    let case_variant = temp.path().join("mods/eefixpack.TOML");
    std::fs::rename(source, case_variant).unwrap();

    let manifest = Manifest::load(temp.path()).unwrap();

    assert!(manifest.mods.contains_key("eefixpack"));
    assert_eq!(
        manifest.conventional_mod_path("eefixpack"),
        manifest.root.join("mods/eefixpack.toml")
    );
}

#[test]
fn follows_toml_file_symlinks() {
    let temp = TestDir::from_fixture("file-symlink");
    let target = temp.path().join("mods/eefixpack.toml");
    let link = temp.path().join("mods/linked.toml");
    if !symlink_file(&target, &link) {
        return;
    }

    let error = load_error(temp.path());

    match error {
        EngineError::ModIdMismatch { path, id, stem } => {
            assert_eq!(path, temp.canonical_path().join("mods/linked.toml"));
            assert_eq!(id, "eefixpack");
            assert_eq!(stem, "linked");
        }
        other => panic!("expected a mod id mismatch error, got {other:?}"),
    }
}

#[test]
fn broken_toml_symlink_is_io_error_with_path() {
    let temp = TestDir::from_fixture("broken-file-symlink");
    let missing_target = temp.path().join("missing.toml");
    let link = temp.path().join("mods/broken.toml");
    if !symlink_file(&missing_target, &link) {
        return;
    }

    let error = load_error(temp.path());

    match error {
        EngineError::Io { path, .. } => {
            assert_eq!(path, temp.canonical_path().join("mods/broken.toml"));
        }
        other => panic!("expected an I/O error, got {other:?}"),
    }
}

#[test]
fn rejects_non_utf8_mod_file_stem_explicitly() {
    let temp = TestDir::from_fixture("non-utf8-stem");
    let source = temp.path().join("mods/eefixpack.toml");
    let invalid_path = temp.path().join("mods").join(non_utf8_toml_name());
    std::fs::copy(source, invalid_path).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::InvalidModFileStem { path } => assert_eq!(
            path,
            temp.canonical_path()
                .join("mods")
                .join(non_utf8_toml_name())
        ),
        other => panic!("expected an invalid mod file stem error, got {other:?}"),
    }
}

#[test]
fn missing_mods_dir_is_io_error_with_path() {
    let temp = TestDir::from_fixture("missing-mods-dir");
    let mods_path = temp.path().join("mods");
    std::fs::remove_dir_all(&mods_path).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::Io { path, .. } => {
            assert_eq!(path, temp.canonical_path().join("mods"));
        }
        other => panic!("expected an I/O error, got {other:?}"),
    }
}

#[test]
fn ignores_non_toml_files() {
    let temp = TestDir::from_fixture("ignore-non-toml");
    std::fs::write(temp.path().join("mods/README.md"), "fixture notes").unwrap();

    let manifest = Manifest::load(temp.path()).unwrap();

    assert_eq!(manifest.mods.len(), 1);
    assert!(manifest.mods.contains_key("eefixpack"));
}

#[test]
fn ignores_toml_subdirectories() {
    let temp = TestDir::from_fixture("ignore-toml-subdirectory");
    std::fs::create_dir(temp.path().join("mods/ignored.toml")).unwrap();

    let manifest = Manifest::load(temp.path()).unwrap();

    assert_eq!(manifest.mods.len(), 1);
    assert!(manifest.mods.contains_key("eefixpack"));
}
