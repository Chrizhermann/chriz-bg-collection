//! Loading a complete manifest directory from disk.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::error::EngineError;
use crate::manifest::{Collection, ModFile};

/// Manifest schema version understood by this engine.
pub const SUPPORTED_SCHEMA: u32 = 1;

/// A collection manifest and all of its per-mod manifest files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Directory containing `collection.toml` and the `mods` directory.
    pub root: PathBuf,
    /// Parsed collection-level manifest.
    pub collection: Collection,
    /// Parsed mod manifests keyed by mod id.
    pub mods: BTreeMap<String, ModFile>,
}

impl Manifest {
    /// Loads and parses a manifest directory.
    pub fn load(dir: &Path) -> crate::error::Result<Manifest> {
        let root = dir.to_path_buf();
        let collection_path = root.join("collection.toml");
        let collection: Collection = read_toml(&collection_path)?;

        if collection.schema != SUPPORTED_SCHEMA {
            return Err(EngineError::UnsupportedSchema {
                path: collection_path,
                found: collection.schema,
                supported: SUPPORTED_SCHEMA,
            });
        }

        let mods_path = root.join("mods");
        let entries = std::fs::read_dir(&mods_path).map_err(|source| EngineError::Io {
            path: mods_path.clone(),
            source,
        })?;
        let mut entries = entries
            .map(|entry| {
                entry.map_err(|source| EngineError::Io {
                    path: mods_path.clone(),
                    source,
                })
            })
            .collect::<crate::error::Result<Vec<_>>>()?;
        entries.sort_by_key(std::fs::DirEntry::file_name);

        let mut mods = BTreeMap::new();
        let mut mod_paths: BTreeMap<String, PathBuf> = BTreeMap::new();

        for entry in entries {
            let path = entry.path();
            if path.extension() != Some(OsStr::new("toml")) {
                continue;
            }

            let file_type = entry.file_type().map_err(|source| EngineError::Io {
                path: path.clone(),
                source,
            })?;
            if !file_type.is_file() {
                continue;
            }

            let mod_file: ModFile = read_toml(&path)?;
            let stem = path
                .file_stem()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_default();
            let id = mod_file.id.clone();

            if let Some(first) = mod_paths.get(&id) {
                return Err(EngineError::DuplicateModId {
                    id,
                    first: first.clone(),
                    second: path,
                });
            }

            if id != stem {
                return Err(EngineError::ModIdMismatch { path, id, stem });
            }

            mod_paths.insert(id.clone(), path);
            mods.insert(id, mod_file);
        }

        Ok(Manifest {
            root,
            collection,
            mods,
        })
    }
}

fn read_toml<T>(path: &Path) -> crate::error::Result<T>
where
    T: DeserializeOwned,
{
    let text = std::fs::read_to_string(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    toml::from_str(&text).map_err(|source| EngineError::ManifestParse {
        path: path.to_path_buf(),
        source,
    })
}
