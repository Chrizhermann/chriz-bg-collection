use std::io::ErrorKind;
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
        let mods = temp.path().join("mods");
        std::fs::create_dir(&mods).unwrap();

        let fixture = fixture_dir();
        std::fs::copy(
            fixture.join("collection.toml"),
            temp.path().join("collection.toml"),
        )
        .unwrap();
        for name in ["eet.toml", "testmod.toml"] {
            std::fs::copy(fixture.join("mods").join(name), mods.join(name)).unwrap();
        }

        temp
    }

    fn path(&self) -> &Path {
        &self.root
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

fn load_error(path: &Path) -> EngineError {
    match Manifest::load(path) {
        Ok(_) => panic!("expected loading {} to fail", path.display()),
        Err(error) => error,
    }
}

#[test]
fn loads_fixture_dir() {
    let root = fixture_dir();
    let manifest = Manifest::load(&root).unwrap();

    assert_eq!(manifest.root, root);
    assert_eq!(manifest.collection.order.len(), 3);
    assert_eq!(
        manifest.mods.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["eet", "testmod"]
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
    let mod_path = temp.path().join("mods/testmod.toml");
    std::fs::write(&mod_path, "id = [").unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::ManifestParse { path, .. } => assert_eq!(path, mod_path),
        other => panic!("expected a manifest parse error, got {other:?}"),
    }
}

#[test]
fn rejects_unsupported_schema() {
    let temp = TestDir::from_fixture("unsupported-schema");
    let collection_path = temp.path().join("collection.toml");
    let collection = std::fs::read_to_string(&collection_path).unwrap();
    std::fs::write(
        &collection_path,
        collection.replacen("schema = 1", "schema = 2", 1),
    )
    .unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::UnsupportedSchema {
            path,
            found,
            supported,
        } => {
            assert_eq!(path, collection_path);
            assert_eq!(found, 2);
            assert_eq!(supported, SUPPORTED_SCHEMA);
            assert_eq!(SUPPORTED_SCHEMA, 1);
        }
        other => panic!("expected an unsupported schema error, got {other:?}"),
    }
}

#[test]
fn rejects_id_file_stem_mismatch() {
    let temp = TestDir::from_fixture("id-stem-mismatch");
    let source = temp.path().join("mods/testmod.toml");
    let other = temp.path().join("mods/other.toml");
    std::fs::copy(source, &other).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::ModIdMismatch { path, id, stem } => {
            assert_eq!(path, other);
            assert_eq!(id, "testmod");
            assert_eq!(stem, "other");
        }
        other => panic!("expected a mod id mismatch error, got {other:?}"),
    }
}

#[test]
fn rejects_duplicate_mod_id() {
    let temp = TestDir::from_fixture("duplicate-id");
    let first = temp.path().join("mods/testmod.toml");
    let second = temp.path().join("mods/zduplicate.toml");
    std::fs::copy(&first, &second).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::DuplicateModId {
            id,
            first: actual_first,
            second: actual_second,
        } => {
            assert_eq!(id, "testmod");
            assert_eq!(actual_first, first);
            assert_eq!(actual_second, second);
        }
        other => panic!("expected a duplicate mod id error, got {other:?}"),
    }
}

#[test]
fn missing_mods_dir_is_io_error_with_path() {
    let temp = TestDir::from_fixture("missing-mods-dir");
    let mods_path = temp.path().join("mods");
    std::fs::remove_dir_all(&mods_path).unwrap();

    let error = load_error(temp.path());

    match error {
        EngineError::Io { path, .. } => assert_eq!(path, mods_path),
        other => panic!("expected an I/O error, got {other:?}"),
    }
}

#[test]
fn ignores_non_toml_files() {
    let temp = TestDir::from_fixture("ignore-non-toml");
    std::fs::write(temp.path().join("mods/README.md"), "fixture notes").unwrap();

    let manifest = Manifest::load(temp.path()).unwrap();

    assert_eq!(manifest.mods.len(), 2);
    assert!(manifest.mods.contains_key("eet"));
    assert!(manifest.mods.contains_key("testmod"));
}

#[test]
fn ignores_toml_subdirectories() {
    let temp = TestDir::from_fixture("ignore-toml-subdirectory");
    std::fs::create_dir(temp.path().join("mods/ignored.toml")).unwrap();

    let manifest = Manifest::load(temp.path()).unwrap();

    assert_eq!(manifest.mods.len(), 2);
    assert!(manifest.mods.contains_key("eet"));
    assert!(manifest.mods.contains_key("testmod"));
}
