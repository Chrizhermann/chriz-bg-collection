//! Append-only app-data registry for completed managed installations.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::digest::sha256_bytes;

/// Managed-install registry schema emitted by this engine version.
pub const REGISTRY_SCHEMA_VERSION: u32 = 1;

const RECORDS_DIRECTORY: &str = "managed-installs";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Immutable discovery record for one successfully receipted installation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedInstallRecord {
    /// Registry record schema version.
    pub schema_version: u32,
    /// Globally unique managed-install id and record filename stem.
    pub install_id: String,
    /// Player-facing collection name captured at completion.
    pub display_name: String,
    /// Original absolute target path; retained even if the directory later moves.
    pub managed_root: PathBuf,
    /// Immutable recipe release used by this install.
    pub recipe_version: String,
    /// Exact signed recipe payload digest.
    pub recipe_sha256: String,
    /// Read-back final EET `engine_name`.
    pub engine_name: String,
    /// Reserved per-user save and configuration directory.
    pub managed_save_root: PathBuf,
    /// Verified launch executable, normally `InfinityLoader.exe` when EEex is selected.
    pub launch_path: PathBuf,
    /// SHA-256 of `.chriz/install-receipt.json`.
    pub receipt_sha256: String,
    /// Successful receipt completion time in Unix epoch milliseconds.
    pub completed_at_millis: u64,
}

/// Whether a registry record still resolves to its exact receipted target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallAvailability {
    /// Managed root and successful receipt remain present and byte-exact.
    Available,
    /// Target is missing, moved, replaced, or no longer carries the exact receipt.
    Stale,
}

/// One immutable record projected as a Home-screen card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedInstallCard {
    /// Stored record, never discarded merely because its target is unavailable.
    pub record: ManagedInstallRecord,
    /// Current read-only target availability.
    pub availability: InstallAvailability,
}

/// Failure to publish or read the immutable managed-install registry.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// App-data, record, target, or receipt path is unsafe.
    #[error("unsafe registry path {path}: {message}")]
    UnsafePath {
        /// Rejected path.
        path: PathBuf,
        /// Validation detail.
        message: String,
    },
    /// A record contradicts its schema or target receipt.
    #[error("invalid managed-install record: {0}")]
    InvalidRecord(String),
    /// The stable id already belongs to another managed root.
    #[error(
        "managed install id {install_id:?} already belongs to a different managed root: {existing}"
    )]
    DuplicateRoot {
        /// Colliding stable id.
        install_id: String,
        /// Root retained by the create-once record.
        existing: PathBuf,
    },
    /// The existing immutable record differs in another field.
    #[error("create-once managed-install record differs at {path}")]
    CreateOnceConflict {
        /// Existing record path.
        path: PathBuf,
    },
    /// Registry JSON could not be serialized or parsed.
    #[error("registry JSON error at {path}: {message}")]
    Json {
        /// Record path.
        path: PathBuf,
        /// Serde detail.
        message: String,
    },
    /// Filesystem access failed.
    #[error("{path}: {source}")]
    Io {
        /// Path being inspected or written.
        path: PathBuf,
        /// Underlying error.
        #[source]
        source: std::io::Error,
    },
}

/// Filesystem-backed immutable record directory under application data.
#[derive(Clone, Debug)]
pub struct ManagedInstallRegistry {
    records_root: PathBuf,
}

impl ManagedInstallRegistry {
    /// Open or create the dedicated `managed-installs` record directory.
    pub fn open_or_create(app_data_root: &Path) -> Result<Self, RegistryError> {
        if !app_data_root.is_absolute() {
            return Err(unsafe_path(
                app_data_root,
                "application-data root must be absolute",
            ));
        }
        ensure_existing_ancestors_are_direct(app_data_root)?;
        create_directories(app_data_root)?;
        validate_direct_directory(app_data_root)?;
        let records_root = app_data_root.join(RECORDS_DIRECTORY);
        create_directories(&records_root)?;
        validate_direct_directory(&records_root)?;
        Ok(Self { records_root })
    }

    /// Publish one record without replacing any existing record for its install id.
    ///
    /// A byte-identical replay is accepted so callers can reconcile a crash after the
    /// create-once link became durable but before the UI observed completion.
    pub fn publish(&self, record: &ManagedInstallRecord) -> Result<PathBuf, RegistryError> {
        self.validate_record(record)?;
        let path = self
            .records_root
            .join(format!("{}.json", record.install_id));
        let bytes = record_bytes(record, &path)?;
        if let Some(existing) = read_existing(&path)? {
            let parsed: ManagedInstallRecord =
                serde_json::from_slice(&existing).map_err(|source| RegistryError::Json {
                    path: path.clone(),
                    message: source.to_string(),
                })?;
            if parsed.managed_root != record.managed_root {
                return Err(RegistryError::DuplicateRoot {
                    install_id: record.install_id.clone(),
                    existing: parsed.managed_root,
                });
            }
            if existing == bytes {
                return Ok(path);
            }
            return Err(RegistryError::CreateOnceConflict { path });
        }

        publish_create_once(&path, &bytes)?;
        Ok(path)
    }

    /// Read every immutable record and project missing or changed targets as stale.
    pub fn list(&self) -> Result<Vec<ManagedInstallCard>, RegistryError> {
        validate_direct_directory(&self.records_root)?;
        let mut paths = Vec::new();
        for entry in fs::read_dir(&self.records_root).map_err(|source| RegistryError::Io {
            path: self.records_root.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| RegistryError::Io {
                path: self.records_root.clone(),
                source,
            })?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|source| RegistryError::Io {
                path: path.clone(),
                source,
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(unsafe_path(
                    &path,
                    "registry entries must be direct regular files",
                ));
            }
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                return Err(unsafe_path(&path, "registry filename is not valid Unicode"));
            };
            if name.contains(".tmp.") {
                continue;
            }
            let Some(id) = name.strip_suffix(".json") else {
                return Err(unsafe_path(&path, "unrecognized registry filename"));
            };
            validate_identifier(id, "install id", &path)?;
            paths.push((id.to_owned(), path));
        }
        paths.sort_by(|left, right| left.0.cmp(&right.0));

        let mut cards = Vec::with_capacity(paths.len());
        for (filename_id, path) in paths {
            let bytes = fs::read(&path).map_err(|source| RegistryError::Io {
                path: path.clone(),
                source,
            })?;
            let record: ManagedInstallRecord =
                serde_json::from_slice(&bytes).map_err(|source| RegistryError::Json {
                    path: path.clone(),
                    message: source.to_string(),
                })?;
            if record.install_id != filename_id {
                return Err(RegistryError::InvalidRecord(format!(
                    "install id {:?} does not match filename {filename_id:?}",
                    record.install_id
                )));
            }
            validate_stored_record(&record)?;
            let availability = availability(&record);
            cards.push(ManagedInstallCard {
                record,
                availability,
            });
        }
        Ok(cards)
    }

    fn validate_record(&self, record: &ManagedInstallRecord) -> Result<(), RegistryError> {
        validate_stored_record(record)?;
        validate_direct_directory(&record.managed_root)?;
        validate_direct_file(&record.launch_path)?;
        let canonical_root = canonicalize(&record.managed_root)?;
        let canonical_launch = canonicalize(&record.launch_path)?;
        if !canonical_launch.starts_with(&canonical_root) {
            return Err(RegistryError::InvalidRecord(
                "launch path must be inside the managed root".to_owned(),
            ));
        }
        let receipt = record.managed_root.join(".chriz/install-receipt.json");
        validate_direct_file(&receipt)?;
        let actual = hash_file(&receipt)?;
        if actual != record.receipt_sha256.to_ascii_lowercase() {
            return Err(RegistryError::InvalidRecord(format!(
                "receipt SHA-256 mismatch: expected {}, found {actual}",
                record.receipt_sha256
            )));
        }
        Ok(())
    }
}

fn validate_stored_record(record: &ManagedInstallRecord) -> Result<(), RegistryError> {
    if record.schema_version != REGISTRY_SCHEMA_VERSION {
        return Err(RegistryError::InvalidRecord(format!(
            "unsupported schema {}; expected {REGISTRY_SCHEMA_VERSION}",
            record.schema_version
        )));
    }
    validate_identifier(&record.install_id, "install id", &record.managed_root)?;
    for (label, value) in [
        ("display name", record.display_name.as_str()),
        ("recipe version", record.recipe_version.as_str()),
        ("engine name", record.engine_name.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(RegistryError::InvalidRecord(format!(
                "{label} must not be empty"
            )));
        }
    }
    validate_hash("recipe", &record.recipe_sha256)?;
    validate_hash("receipt", &record.receipt_sha256)?;
    for (label, path) in [
        ("managed root", &record.managed_root),
        ("managed save root", &record.managed_save_root),
        ("launch path", &record.launch_path),
    ] {
        if !path.is_absolute() {
            return Err(RegistryError::InvalidRecord(format!(
                "{label} must be absolute: {}",
                path.display()
            )));
        }
        if path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
        {
            return Err(RegistryError::InvalidRecord(format!(
                "{label} must not contain parent or current-directory segments: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn availability(record: &ManagedInstallRecord) -> InstallAvailability {
    let receipt = record.managed_root.join(".chriz/install-receipt.json");
    let available = direct_directory_exists(&record.managed_root)
        && direct_file_exists(&receipt)
        && direct_file_exists(&record.launch_path)
        && canonical_launch_is_contained(record)
        && hash_file(&receipt)
            .map(|hash| hash == record.receipt_sha256.to_ascii_lowercase())
            .unwrap_or(false);
    if available {
        InstallAvailability::Available
    } else {
        InstallAvailability::Stale
    }
}

fn canonical_launch_is_contained(record: &ManagedInstallRecord) -> bool {
    canonicalize(&record.managed_root)
        .and_then(|root| canonicalize(&record.launch_path).map(|launch| (root, launch)))
        .map(|(root, launch)| launch.starts_with(root))
        .unwrap_or(false)
}

fn canonicalize(path: &Path) -> Result<PathBuf, RegistryError> {
    fs::canonicalize(path).map_err(|source| RegistryError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn hash_file(path: &Path) -> Result<String, RegistryError> {
    let bytes = fs::read(path).map_err(|source| RegistryError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(sha256_bytes(&bytes))
}

fn record_bytes(record: &ManagedInstallRecord, path: &Path) -> Result<Vec<u8>, RegistryError> {
    let mut bytes = serde_json::to_vec_pretty(record).map_err(|source| RegistryError::Json {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_existing(path: &Path) -> Result<Option<Vec<u8>>, RegistryError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(unsafe_path(path, "record destination is not a direct file"));
            }
            fs::read(path)
                .map(Some)
                .map_err(|source| RegistryError::Io {
                    path: path.to_path_buf(),
                    source,
                })
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(RegistryError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn publish_create_once(path: &Path, bytes: &[u8]) -> Result<(), RegistryError> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| unsafe_path(path, "record filename is not valid Unicode"))?;
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = path.with_file_name(format!(".{name}.tmp.{}.{}", std::process::id(), sequence));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|source| RegistryError::Io {
                path: temporary.clone(),
                source,
            })?;
        file.write_all(bytes).map_err(|source| RegistryError::Io {
            path: temporary.clone(),
            source,
        })?;
        file.sync_all().map_err(|source| RegistryError::Io {
            path: temporary.clone(),
            source,
        })?;
        drop(file);
        fs::hard_link(&temporary, path).map_err(|source| {
            if path.exists() {
                RegistryError::CreateOnceConflict {
                    path: path.to_path_buf(),
                }
            } else {
                RegistryError::Io {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })
    })();
    let cleanup = fs::remove_file(&temporary);
    if let Err(error) = result {
        let _ = cleanup;
        return Err(error);
    }
    cleanup.map_err(|source| RegistryError::Io {
        path: temporary,
        source,
    })
}

fn validate_hash(label: &str, value: &str) -> Result<(), RegistryError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(RegistryError::InvalidRecord(format!(
            "{label} SHA-256 is not 64 hexadecimal characters"
        )));
    }
    Ok(())
}

fn validate_identifier(value: &str, label: &str, path: &Path) -> Result<(), RegistryError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(unsafe_path(path, &format!("{label} is not path-safe")));
    }
    Ok(())
}

fn create_directories(path: &Path) -> Result<(), RegistryError> {
    match fs::create_dir_all(path) {
        Ok(()) => Ok(()),
        Err(source) => Err(RegistryError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn ensure_existing_ancestors_are_direct(path: &Path) -> Result<(), RegistryError> {
    let mut current = Some(path);
    while let Some(candidate) = current {
        match fs::symlink_metadata(candidate) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(unsafe_path(
                        candidate,
                        "existing ancestor must be a direct directory",
                    ));
                }
            }
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(RegistryError::Io {
                    path: candidate.to_path_buf(),
                    source,
                });
            }
        }
        current = candidate.parent();
    }
    Ok(())
}

fn validate_direct_directory(path: &Path) -> Result<(), RegistryError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| RegistryError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(unsafe_path(path, "expected a direct non-symlink directory"));
    }
    Ok(())
}

fn validate_direct_file(path: &Path) -> Result<(), RegistryError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| RegistryError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(unsafe_path(path, "expected a direct non-symlink file"));
    }
    Ok(())
}

fn direct_directory_exists(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn direct_file_exists(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn unsafe_path(path: &Path, message: &str) -> RegistryError {
    RegistryError::UnsafePath {
        path: path.to_path_buf(),
        message: message.to_owned(),
    }
}
