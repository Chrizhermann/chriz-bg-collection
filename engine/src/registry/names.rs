//! Display names are editable AppData metadata, never installation identity.
//!
//! The original completion record, campaign ledger, game folder and save profile
//! are not rewritten. An OS file lock serializes new names and renames across app
//! processes. Removed installations leave harmless name metadata behind: only
//! names with a current registry identity reserve a name or appear in the UI.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};

use super::{
    create_directories, read_existing, unsafe_path, validate_identifier, CampaignCreated,
    ManagedInstallRegistry, RegistryError, SessionStore,
};

/// Suggested name for the first registered installation.
pub const DEFAULT_INSTALLATION_NAME: &str = "Chriz Easy BG";
/// Maximum accepted length in Unicode scalar values, excluding edge whitespace.
pub const MAX_INSTALLATION_NAME_CHARACTERS: usize = 80;

/// Folder under `LOCALAPPDATA` used by the Baldur's Gate app and every EET installation. Its
/// `locks` folder holds the target locks of every application, whatever folder an application
/// uses for its own registry.
pub const DEFAULT_APPLICATION_DATA_DIRECTORY: &str = "Chriz BG Collection";

/// Checks that `name` can be an application's own folder under `LOCALAPPDATA`: one plain
/// Windows folder name of 1-64 bytes that is not a reserved device name. Returns the reason
/// when it cannot.
pub fn check_application_data_directory(name: &str) -> Result<(), &'static str> {
    if name.trim().is_empty() || name.len() > 64 {
        return Err("must be 1-64 characters");
    }
    if matches!(name, "." | "..") || name.ends_with(['.', ' ']) || name.starts_with(' ') {
        return Err("is not a plain folder name");
    }
    if name.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
    }) {
        return Err("must be one folder name without path or reserved characters");
    }
    // Windows reserves a device name also with an extension or trailing spaces before it.
    let stem = name.split('.').next().unwrap_or(name).trim_end();
    if is_reserved_device_name(stem) {
        return Err("is a reserved Windows device name");
    }
    Ok(())
}

/// CON, PRN, AUX, NUL, and COM or LPT followed by one digit 0-9 or superscript 1-3, in any
/// ASCII case. Compared by character, so a multi-byte superscript is never misread.
fn is_reserved_device_name(stem: &str) -> bool {
    if ["CON", "PRN", "AUX", "NUL"]
        .iter()
        .any(|device| stem.eq_ignore_ascii_case(device))
    {
        return true;
    }
    let mut characters = stem.chars();
    let prefix = characters.by_ref().take(3).collect::<String>();
    let rest = characters.collect::<Vec<_>>();
    (prefix.eq_ignore_ascii_case("COM") || prefix.eq_ignore_ascii_case("LPT"))
        && matches!(
            rest.as_slice(),
            [digit] if digit.is_ascii_digit() || matches!(digit, '\u{b9}' | '\u{b2}' | '\u{b3}')
        )
}

/// Validate and trim a player-provided name. This does not check other installs.
pub fn normalize_display_name(name: &str) -> Result<String, RegistryError> {
    let name = name.trim();
    let detail = if name.is_empty() {
        Some("Enter a name for this installation.")
    } else if name.chars().count() > MAX_INSTALLATION_NAME_CHARACTERS {
        Some("Keep the installation name to 80 characters or fewer.")
    } else if name.chars().any(char::is_control) {
        Some("Installation names cannot contain control characters.")
    } else if matches!(name, "." | "..")
        || name.chars().any(|character| {
            matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
        })
    {
        Some("Installation names cannot contain < > : \" / \\ | ? * or consist only of dots.")
    } else {
        None
    };
    match detail {
        Some(detail) => Err(RegistryError::InvalidName(detail.to_owned())),
        None => Ok(name.to_owned()),
    }
}

fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NameRecord {
    schema_version: u32,
    install_id: String,
    managed_root: PathBuf,
    display_name: String,
}

struct NamedIdentity {
    root: PathBuf,
    name: String,
}

impl ManagedInstallRegistry {
    /// Suggest a currently unused numbered default; creation rechecks it under a lock.
    pub fn suggested_display_name(&self) -> Result<String, RegistryError> {
        let names = self.named_identities()?;
        for suffix in 1_u64.. {
            let name = if suffix == 1 {
                DEFAULT_INSTALLATION_NAME.to_owned()
            } else {
                format!("{DEFAULT_INSTALLATION_NAME} ({suffix})")
            };
            if !names
                .values()
                .any(|entry| name_key(&entry.name) == name_key(&name))
            {
                return Ok(name);
            }
        }
        Err(RegistryError::InvalidName(
            "No unused installation name is available.".to_owned(),
        ))
    }

    /// Check a proposed name against completed, failed, incomplete and unavailable copies.
    /// This is advisory; creation and rename repeat the check under the registry name lock.
    pub fn validate_display_name(
        &self,
        name: &str,
        excluding_install_id: Option<&str>,
    ) -> Result<String, RegistryError> {
        let name = normalize_display_name(name)?;
        let identities = self.named_identities()?;
        if let Some(id) = excluding_install_id {
            if !identities.contains_key(id) {
                return Err(unknown_installation());
            }
        }
        check_unused(&identities, &name, excluding_install_id)?;
        Ok(name)
    }

    /// Resolve an editable display name without changing any immutable identity record.
    pub fn display_name(&self, install_id: &str) -> Result<String, RegistryError> {
        validate_identifier(install_id, "install id", &self.names_root)?;
        self.named_identities()?
            .remove(install_id)
            .map(|entry| entry.name)
            .ok_or_else(unknown_installation)
    }

    /// Rename one registered copy, including unavailable copies, without moving files.
    pub fn rename_display_name(
        &self,
        install_id: &str,
        name: &str,
    ) -> Result<String, RegistryError> {
        validate_identifier(install_id, "install id", &self.names_root)?;
        let name = normalize_display_name(name)?;
        let _guard = self.lock_names()?;
        let identities = self.named_identities()?;
        let identity = identities
            .get(install_id)
            .ok_or_else(unknown_installation)?;
        check_unused(&identities, &name, Some(install_id))?;
        self.write_name(install_id, &identity.root, &name)?;
        Ok(name)
    }

    /// Publish a named campaign atomically with respect to other new names and renames.
    ///
    /// The pointer is written first. If the process exits before the display sidecar
    /// is durable, its frozen recipe still supplies the same chosen name. A resumed
    /// campaign never overwrites a later display-only rename with the frozen name.
    pub fn publish_campaign_named(
        &self,
        created: &CampaignCreated,
        name: &str,
    ) -> Result<PathBuf, RegistryError> {
        let _guard = self.lock_names()?;
        let identities = self.named_identities()?;
        if let Some(existing) = identities.get(&created.install_id) {
            if existing.root != created.managed_root {
                return Err(RegistryError::DuplicateRoot {
                    install_id: created.install_id.clone(),
                    existing: existing.root.clone(),
                });
            }
            // Existing campaigns (including legacy duplicates) may resume. Their
            // existing chosen/renamed name wins over a replayed creation request.
            let path = self.publish_campaign(created)?;
            if self
                .read_name(&created.install_id, &created.managed_root)?
                .is_none()
            {
                if let Ok(existing_name) = normalize_display_name(&existing.name) {
                    self.write_name(&created.install_id, &created.managed_root, &existing_name)?;
                }
            }
            return Ok(path);
        }
        let name = normalize_display_name(name)?;
        check_unused(&identities, &name, None)?;
        let path = self.publish_campaign(created)?;
        self.write_name(&created.install_id, &created.managed_root, &name)?;
        Ok(path)
    }

    pub(super) fn lock_names(&self) -> Result<File, RegistryError> {
        let mut ancestor = Some(self.names_root.as_path());
        while let Some(path) = ancestor {
            if let Some(metadata) = metadata_if_present(path)? {
                reject_reparse(path, &metadata)?;
                if !metadata.is_dir() {
                    return Err(unsafe_path(
                        path,
                        "name metadata ancestor is not a directory",
                    ));
                }
            }
            ancestor = path.parent();
        }
        create_directories(&self.names_root)?;
        let lock_path = self.names_root.join(".names.lock");
        if let Some(metadata) = metadata_if_present(&lock_path)? {
            reject_reparse(&lock_path, &metadata)?;
            if !metadata.is_file() {
                return Err(unsafe_path(&lock_path, "name lock is not a regular file"));
            }
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|source| io_error(&lock_path, source))?;
        file.lock_exclusive()
            .map_err(|source| io_error(&lock_path, source))?;
        Ok(file)
    }

    fn named_identities(&self) -> Result<BTreeMap<String, NamedIdentity>, RegistryError> {
        let mut identities = BTreeMap::new();
        for card in self.list()? {
            let record = card.record;
            let name = self
                .read_name(&record.install_id, &record.managed_root)?
                .unwrap_or(record.display_name);
            identities.insert(
                record.install_id,
                NamedIdentity {
                    root: record.managed_root,
                    name,
                },
            );
        }
        for card in self.list_campaigns()? {
            let record = card.record;
            if let Some(existing) = identities.get(&record.install_id) {
                if existing.root != record.managed_root {
                    return Err(RegistryError::DuplicateRoot {
                        install_id: record.install_id,
                        existing: existing.root.clone(),
                    });
                }
                continue;
            }
            let name = match self.read_name(&record.install_id, &record.managed_root)? {
                Some(name) => name,
                None => legacy_campaign_name(&record.managed_root),
            };
            identities.insert(
                record.install_id,
                NamedIdentity {
                    root: record.managed_root,
                    name,
                },
            );
        }
        Ok(identities)
    }

    fn read_name(&self, install_id: &str, root: &Path) -> Result<Option<String>, RegistryError> {
        let Some(directory) = metadata_if_present(&self.names_root)? else {
            return Ok(None);
        };
        reject_reparse(&self.names_root, &directory)?;
        if !directory.is_dir() {
            return Err(unsafe_path(
                &self.names_root,
                "name metadata is not a directory",
            ));
        }
        let path = self.names_root.join(format!("{install_id}.json"));
        if let Some(metadata) = metadata_if_present(&path)? {
            reject_reparse(&path, &metadata)?;
        }
        let Some(bytes) = read_existing(&path)? else {
            return Ok(None);
        };
        let record: NameRecord =
            serde_json::from_slice(&bytes).map_err(|source| RegistryError::Json {
                path: path.clone(),
                message: source.to_string(),
            })?;
        if record.schema_version != 1
            || record.install_id != install_id
            || record.managed_root != root
        {
            return Err(RegistryError::InvalidRecord(format!(
                "installation name metadata does not match its registered identity at {}",
                path.display()
            )));
        }
        let name = normalize_display_name(&record.display_name)?;
        if name != record.display_name {
            return Err(RegistryError::InvalidRecord(format!(
                "installation name metadata is not normalized at {}",
                path.display()
            )));
        }
        Ok(Some(name))
    }

    fn write_name(&self, install_id: &str, root: &Path, name: &str) -> Result<(), RegistryError> {
        validate_identifier(install_id, "install id", &self.names_root)?;
        let path = self.names_root.join(format!("{install_id}.json"));
        // Reject an existing foreign identity or indirect destination before replacement.
        self.read_name(install_id, root)?;
        let record = NameRecord {
            schema_version: 1,
            install_id: install_id.to_owned(),
            managed_root: root.to_path_buf(),
            display_name: name.to_owned(),
        };
        let mut bytes =
            serde_json::to_vec_pretty(&record).map_err(|source| RegistryError::Json {
                path: path.clone(),
                message: source.to_string(),
            })?;
        bytes.push(b'\n');
        let mut temporary = tempfile::NamedTempFile::new_in(&self.names_root)
            .map_err(|source| io_error(&self.names_root, source))?;
        temporary
            .write_all(&bytes)
            .map_err(|source| io_error(&path, source))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|source| io_error(&path, source))?;
        temporary
            .persist(&path)
            .map_err(|error| io_error(&path, error.error))?;
        Ok(())
    }
}

fn check_unused(
    identities: &BTreeMap<String, NamedIdentity>,
    name: &str,
    excluding_id: Option<&str>,
) -> Result<(), RegistryError> {
    if let Some((install_id, _)) = identities.iter().find(|(id, entry)| {
        Some(id.as_str()) != excluding_id && name_key(&entry.name) == name_key(name)
    }) {
        return Err(RegistryError::DuplicateName {
            name: name.to_owned(),
            install_id: install_id.clone(),
        });
    }
    Ok(())
}

fn legacy_campaign_name(root: &Path) -> String {
    // The frozen display name is read only. No ledger/receipt migration is needed.
    let frozen_name = SessionStore::open(root)
        .and_then(|store| store.replay_for_inspection())
        .ok()
        .and_then(|replay| {
            serde_json::from_slice::<serde_json::Value>(&replay.created().recipe_payload).ok()
        })
        .and_then(|value| {
            value
                .get("display_name")
                .and_then(|name| name.as_str())
                .map(str::to_owned)
        });
    frozen_name
        .filter(|name| !name.trim().is_empty())
        .or_else(|| {
            root.file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| DEFAULT_INSTALLATION_NAME.to_owned())
}

fn metadata_if_present(path: &Path) -> Result<Option<fs::Metadata>, RegistryError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(io_error(path, source)),
    }
}

fn reject_reparse(path: &Path, metadata: &fs::Metadata) -> Result<(), RegistryError> {
    #[cfg(windows)]
    let reparse = {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    };
    #[cfg(not(windows))]
    let reparse = false;
    if metadata.file_type().is_symlink() || reparse {
        return Err(unsafe_path(
            path,
            "name metadata must not use symbolic links or reparse points",
        ));
    }
    Ok(())
}

fn unknown_installation() -> RegistryError {
    RegistryError::InvalidRecord(
        "This installation is no longer registered. Refresh My installs.".to_owned(),
    )
}

fn io_error(path: &Path, source: std::io::Error) -> RegistryError {
    RegistryError::Io {
        path: path.to_path_buf(),
        source,
    }
}
