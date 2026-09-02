//! Persisted install-session state and resume semantics.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{EngineError, Result};
use crate::manifest::{Artifact, Collection, ModFile, PresetFile};
use crate::resolve::Selection;
use crate::Manifest;

const SESSION_FILE_NAME: &str = "session.json";
const SESSION_TEMP_FILE_NAME: &str = "session.json.tmp";

#[derive(Serialize)]
struct CanonicalManifest<'a> {
    collection: &'a Collection,
    artifacts: &'a BTreeMap<String, Artifact>,
    mods: &'a BTreeMap<String, ModFile>,
    presets: &'a BTreeMap<String, PresetFile>,
}

/// Persisted state for one install attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    /// SHA-256 over canonicalized manifest content.
    pub manifest_fingerprint: String,
    /// User selection resolved by this install attempt.
    pub selection: Selection,
    /// Pipeline steps in execution order.
    pub steps: Vec<StepRecord>,
}

/// Persisted state for one pipeline step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepRecord {
    /// Stable identifier for the pipeline step.
    pub id: String,
    /// Current execution state.
    pub status: StepStatus,
    /// Optional diagnostic or outcome detail.
    pub detail: Option<String>,
}

/// Execution state of a persisted pipeline step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// The step has not started.
    Pending,
    /// The step was in progress when the session was last written.
    Running,
    /// The step completed successfully.
    Done,
    /// The step failed or was interrupted and must be retried.
    Failed,
}

impl Session {
    /// Atomically writes this session to `session.json` inside `target_dir`.
    ///
    /// The target directory must already exist. The complete JSON document is
    /// written and flushed to a sibling temporary file before an atomic rename
    /// publishes it as `session.json`.
    pub fn save(&self, target_dir: &Path) -> Result<()> {
        let session_path = target_dir.join(SESSION_FILE_NAME);
        let temp_path = target_dir.join(SESSION_TEMP_FILE_NAME);
        let mut json =
            serde_json::to_vec_pretty(self).map_err(|source| EngineError::SessionJson {
                path: session_path.clone(),
                source,
            })?;
        json.push(b'\n');

        let result = write_and_publish(&temp_path, &session_path, &json);
        if result.is_err() {
            let _ = std::fs::remove_file(&temp_path);
        }
        result
    }

    /// Loads `session.json` and prepares interrupted work for resume.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::ManifestFingerprintMismatch`] when the persisted
    /// session belongs to different manifest content. Resume is forbidden in
    /// that case; the target must be rebuilt instead.
    pub fn load(target_dir: &Path, expected_manifest_fingerprint: &str) -> Result<Session> {
        let path = target_dir.join(SESSION_FILE_NAME);
        let text = std::fs::read_to_string(&path).map_err(|source| EngineError::Io {
            path: path.clone(),
            source,
        })?;
        let mut session: Session =
            serde_json::from_str(&text).map_err(|source| EngineError::SessionJson {
                path: path.clone(),
                source,
            })?;

        if session.manifest_fingerprint != expected_manifest_fingerprint {
            return Err(EngineError::ManifestFingerprintMismatch {
                path,
                expected: expected_manifest_fingerprint.to_owned(),
                found: session.manifest_fingerprint,
            });
        }

        for step in &mut session.steps {
            if step.status == StepStatus::Running {
                step.status = StepStatus::Failed;
            }
        }

        Ok(session)
    }

    /// Returns the index of the first incomplete or retryable step.
    ///
    /// Despite the legacy method name, this includes `Failed` and `Running`
    /// steps as well as `Pending`; only `Done` steps are skipped.
    pub fn next_pending(&self) -> Option<usize> {
        self.steps
            .iter()
            .position(|step| step.status != StepStatus::Done)
    }
}

/// Computes a deterministic SHA-256 over parsed manifest content.
///
/// The manifest root is excluded, so relocating identical manifest content
/// does not invalidate a session. The collection and all sorted artifact,
/// installer, and preset maps are serialized together, making TOML whitespace
/// and comments irrelevant.
pub fn manifest_fingerprint(manifest: &Manifest) -> Result<String> {
    let canonical = CanonicalManifest {
        collection: &manifest.collection,
        artifacts: &manifest.artifacts,
        mods: &manifest.mods,
        presets: &manifest.presets,
    };
    let json =
        serde_json::to_vec(&canonical).map_err(|source| EngineError::ManifestFingerprint {
            path: manifest.root.clone(),
            source,
        })?;

    Ok(hex::encode(Sha256::digest(json)))
}

fn write_and_publish(temp_path: &Path, session_path: &Path, contents: &[u8]) -> Result<()> {
    match std::fs::remove_file(temp_path) {
        Ok(()) => {}
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(EngineError::Io {
                path: temp_path.to_path_buf(),
                source,
            });
        }
    }

    let mut temp_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp_path)
        .map_err(|source| EngineError::Io {
            path: temp_path.to_path_buf(),
            source,
        })?;
    temp_file
        .write_all(contents)
        .map_err(|source| EngineError::Io {
            path: temp_path.to_path_buf(),
            source,
        })?;
    temp_file.sync_all().map_err(|source| EngineError::Io {
        path: temp_path.to_path_buf(),
        source,
    })?;
    drop(temp_file);

    std::fs::rename(temp_path, session_path).map_err(|source| EngineError::Io {
        path: session_path.to_path_buf(),
        source,
    })
}
