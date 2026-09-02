//! Loading a complete executable recipe directory from disk.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::error::EngineError;
use crate::manifest::{Artifact, Collection, ModFile, PresetFile};

/// Manifest schema version understood by this engine.
pub const SUPPORTED_SCHEMA: u32 = 2;

#[derive(Deserialize)]
struct SchemaProbe {
    schema: u32,
}

#[derive(Clone, Copy)]
enum EntryKind {
    Artifact,
    Mod,
    Preset,
}

/// A collection recipe and all independently authored manifest files it references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Canonical directory containing the recipe.
    pub root: PathBuf,
    /// Parsed collection-level recipe.
    pub collection: Collection,
    /// Parsed artifact manifests keyed by artifact id.
    pub artifacts: BTreeMap<String, Artifact>,
    /// Parsed installer manifests keyed by installer id.
    pub mods: BTreeMap<String, ModFile>,
    /// Parsed preset manifests keyed by preset id.
    pub presets: BTreeMap<String, PresetFile>,
}

impl Manifest {
    /// Loads and parses an executable recipe directory.
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
        let artifacts = load_entries(
            &root.join("artifacts"),
            EntryKind::Artifact,
            |artifact: &Artifact| &artifact.id,
        )?;
        let mods = load_entries(&root.join("mods"), EntryKind::Mod, |mod_file: &ModFile| {
            &mod_file.id
        })?;
        let presets = load_entries(
            &root.join("presets"),
            EntryKind::Preset,
            |preset: &PresetFile| &preset.id,
        )?;

        Ok(Manifest {
            root,
            collection,
            artifacts,
            mods,
            presets,
        })
    }

    /// Constructs the conventional lowercase-extension path for an artifact id.
    ///
    /// This does not inspect the filesystem. A loaded recipe may have used a
    /// case-variant extension such as `.TOML`.
    pub fn conventional_artifact_path(&self, id: &str) -> PathBuf {
        self.root.join("artifacts").join(format!("{id}.toml"))
    }

    /// Constructs the conventional lowercase-extension path for an installer id.
    ///
    /// This does not inspect the filesystem. A loaded recipe may have used a
    /// case-variant extension such as `.TOML`.
    pub fn conventional_mod_path(&self, id: &str) -> PathBuf {
        self.root.join("mods").join(format!("{id}.toml"))
    }

    /// Constructs the conventional lowercase-extension path for a preset id.
    ///
    /// This does not inspect the filesystem. A loaded recipe may have used a
    /// case-variant extension such as `.TOML`.
    pub fn conventional_preset_path(&self, id: &str) -> PathBuf {
        self.root.join("presets").join(format!("{id}.toml"))
    }
}

fn load_entries<T, F>(
    directory: &Path,
    kind: EntryKind,
    id_of: F,
) -> crate::error::Result<BTreeMap<String, T>>
where
    T: DeserializeOwned,
    F: Fn(&T) -> &String,
{
    let entries = std::fs::read_dir(directory).map_err(|source| EngineError::Io {
        path: directory.to_path_buf(),
        source,
    })?;
    let mut entries = entries
        .map(|entry| {
            entry.map_err(|source| EngineError::Io {
                path: directory.to_path_buf(),
                source,
            })
        })
        .collect::<crate::error::Result<Vec<_>>>()?;
    entries.sort_by(|left, right| {
        let left_name = left.file_name();
        let right_name = right.file_name();
        left_name
            .to_string_lossy()
            .to_lowercase()
            .cmp(&right_name.to_string_lossy().to_lowercase())
            .then_with(|| left_name.cmp(&right_name))
    });

    let mut values = BTreeMap::new();
    let mut paths: BTreeMap<String, PathBuf> = BTreeMap::new();

    for entry in entries {
        let path = entry.path();
        if !has_toml_extension(&path) {
            continue;
        }

        let metadata = std::fs::metadata(&path).map_err(|source| EngineError::Io {
            path: path.clone(),
            source,
        })?;
        if !metadata.is_file() {
            continue;
        }

        let value: T = read_toml(&path)?;
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| invalid_stem(kind, path.clone()))?
            .to_owned();
        let id = id_of(&value).clone();

        if id != stem {
            return Err(id_mismatch(kind, path, id, stem));
        }

        if let Some(first) = paths.get(&id) {
            return Err(duplicate_id(kind, id, first.clone(), path));
        }

        paths.insert(id.clone(), path);
        values.insert(id, value);
    }

    Ok(values)
}

fn has_toml_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"))
}

fn invalid_stem(kind: EntryKind, path: PathBuf) -> EngineError {
    match kind {
        EntryKind::Artifact => EngineError::InvalidArtifactFileStem { path },
        EntryKind::Mod => EngineError::InvalidModFileStem { path },
        EntryKind::Preset => EngineError::InvalidPresetFileStem { path },
    }
}

fn id_mismatch(kind: EntryKind, path: PathBuf, id: String, stem: String) -> EngineError {
    match kind {
        EntryKind::Artifact => EngineError::ArtifactIdMismatch { path, id, stem },
        EntryKind::Mod => EngineError::ModIdMismatch { path, id, stem },
        EntryKind::Preset => EngineError::PresetIdMismatch { path, id, stem },
    }
}

fn duplicate_id(kind: EntryKind, id: String, first: PathBuf, second: PathBuf) -> EngineError {
    match kind {
        EntryKind::Artifact => EngineError::DuplicateArtifactId { id, first, second },
        EntryKind::Mod => EngineError::DuplicateModId { id, first, second },
        EntryKind::Preset => EngineError::DuplicatePresetId { id, first, second },
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
