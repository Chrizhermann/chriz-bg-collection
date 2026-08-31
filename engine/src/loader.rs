//! Loading a complete manifest directory from disk.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::error::EngineError;
use crate::manifest::{Collection, ModFile};

/// Manifest schema version understood by this engine.
pub const SUPPORTED_SCHEMA: u32 = 1;

#[derive(Deserialize)]
struct SchemaProbe {
    schema: u32,
}

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
        let collection_path = dir.join("collection.toml");
        let collection_text = read_text(&collection_path)?;
        let schema: SchemaProbe = parse_toml(&collection_path, &collection_text)?;

        if schema.schema != SUPPORTED_SCHEMA {
            return Err(EngineError::UnsupportedSchema {
                path: collection_path,
                found: schema.schema,
                supported: SUPPORTED_SCHEMA,
            });
        }

        let collection: Collection = parse_toml(&collection_path, &collection_text)?;
        let root = std::fs::canonicalize(dir).map_err(|source| EngineError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
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
            let is_toml = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"));
            if !is_toml {
                continue;
            }

            let metadata = std::fs::metadata(&path).map_err(|source| EngineError::Io {
                path: path.clone(),
                source,
            })?;
            if !metadata.is_file() {
                continue;
            }

            let mod_file: ModFile = read_toml(&path)?;
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| EngineError::InvalidModFileStem { path: path.clone() })?
                .to_owned();
            let id = mod_file.id.clone();

            if id != stem {
                return Err(EngineError::ModIdMismatch { path, id, stem });
            }

            if let Some(first) = mod_paths.get(&id) {
                return Err(EngineError::DuplicateModId {
                    id,
                    first: first.clone(),
                    second: path,
                });
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

    /// Returns the conventional manifest path for a mod id.
    pub fn mod_path(&self, id: &str) -> PathBuf {
        self.root.join("mods").join(format!("{id}.toml"))
    }
}

fn read_toml<T>(path: &Path) -> crate::error::Result<T>
where
    T: DeserializeOwned,
{
    let text = read_text(path)?;
    parse_toml(path, &text)
}

fn read_text(path: &Path) -> crate::error::Result<String> {
    std::fs::read_to_string(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn parse_toml<T>(path: &Path, text: &str) -> crate::error::Result<T>
where
    T: DeserializeOwned,
{
    toml::from_str(text).map_err(|source| EngineError::ManifestParse {
        path: path.to_path_buf(),
        source,
    })
}
