//! Create-once, append-only campaign identity and progress ledger.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};

use crate::digest::{selection_digest, sha256_bytes};
use crate::error::{EngineError, Result};
use crate::recipe_view::NormalizedSelection;

const STATE_DIRECTORY: &str = ".chriz";
const RECIPE_DIRECTORY: &str = "recipe";
const LEDGER_DIRECTORY: &str = "ledger";
const ATTEMPTS_DIRECTORY: &str = "attempts";
const PAYLOAD_FILE: &str = "payload.zip";
const ENVELOPE_FILE: &str = "envelope.json";
const LOCK_FILE: &str = "ledger.lock";
const FIRST_PREVIOUS_SHA256: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
const MAX_SEQUENCE: u64 = 9_999_999_999;

/// Core fingerprints for the exact source pair accepted when a campaign was created.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceGameFingerprints {
    /// BGEE plus SoD source identity.
    pub bg1: String,
    /// BG2EE source identity.
    pub bg2: String,
}

/// Immutable identity of one payload archive or executable tool.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenIdentity {
    /// Stable recipe artifact identifier.
    pub id: String,
    /// Exact upstream version or revision.
    pub version: String,
    /// Exact lowercase SHA-256.
    pub sha256: String,
    /// Exact byte length.
    pub length: u64,
}

/// Every consequential input frozen by the first ledger event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignCreated {
    /// Globally unique managed-install identifier.
    pub install_id: String,
    /// Identifier for the initial create-once attempt directory.
    pub attempt_id: String,
    /// Canonical managed campaign root.
    pub managed_root: PathBuf,
    /// Canonical content-addressed cache root used by every resume.
    pub cache_root: PathBuf,
    /// Exact signed recipe payload bytes copied to the managed root.
    pub recipe_payload: Vec<u8>,
    /// SHA-256 of the exact recipe payload bytes.
    pub recipe_payload_sha256: String,
    /// Exact signed recipe envelope bytes copied to the managed root.
    pub recipe_envelope: Vec<u8>,
    /// SHA-256 of the exact recipe envelope bytes.
    pub recipe_envelope_sha256: String,
    /// Stable digest of the normalized semantic selection.
    pub selection_sha256: String,
    /// Exact normalized semantic selection retained for replay.
    pub normalized_selection: NormalizedSelection,
    /// Stable digest of the exact resolved installation plan.
    pub plan_sha256: String,
    /// Exact accepted source-game fingerprints.
    pub source_games: SourceGameFingerprints,
    /// Payload artifact identities sorted by stable id.
    pub artifact_identities: Vec<FrozenIdentity>,
    /// Executable tool identities sorted by stable id.
    pub tool_identities: Vec<FrozenIdentity>,
    /// Frozen BG1 staged-copy path.
    pub staged_bg1: PathBuf,
    /// Frozen BG2/EET staged-copy path.
    pub staged_bg2: PathBuf,
}

/// One append-only campaign transition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SessionEvent {
    /// First and only campaign identity event.
    Created(Box<CampaignCreated>),
    /// Intent persisted before one consequential step begins.
    StepStarted {
        /// Stable pipeline step id.
        step_id: String,
        /// One-based attempt number for this step.
        attempt: u32,
    },
    /// Evidence reconciliation proved this step complete.
    StepCompleted {
        /// Stable pipeline step id.
        step_id: String,
        /// Attempt whose evidence was verified.
        attempt: u32,
    },
    /// A completed attempt failed without advancing the campaign.
    StepFailed {
        /// Stable pipeline step id.
        step_id: String,
        /// Failed attempt number.
        attempt: u32,
        /// Durable diagnostic summary.
        detail: String,
    },
    /// Evidence proves this managed copy must never be mutated again.
    FreshCopyRequired {
        /// Stable pipeline step id.
        step_id: String,
        /// Attempt whose installed state failed a postcondition.
        attempt: u32,
        /// Durable diagnostic summary.
        detail: String,
    },
}

/// One immutable JSON record in the campaign hash chain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerRecord {
    /// Zero-based sequence matching the record filename.
    pub sequence: u64,
    /// SHA-256 of the exact previous record bytes, or zeroes for sequence zero.
    pub previous_sha256: String,
    /// Record creation time in Unix milliseconds.
    pub recorded_at: u64,
    /// Persisted transition.
    pub event: SessionEvent,
}

/// Verified replay of every final ledger record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionReplay {
    /// Ordered records after filename, sequence, and hash-chain verification.
    pub records: Vec<LedgerRecord>,
    record_sha256: Vec<String>,
    created: CampaignCreated,
    unresolved: Option<(String, u32)>,
    fresh_copy_required: Option<FreshCopySeal>,
}

/// Durable terminal state preventing any further managed-copy mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreshCopySeal {
    /// Pipeline step whose installed state could not be accepted.
    pub step_id: String,
    /// Attempt that produced the unsafe installed state.
    pub attempt: u32,
    /// Stable reason presented on every later resume.
    pub detail: String,
}

impl SessionReplay {
    /// Returns the frozen campaign identity from record zero.
    pub fn created(&self) -> &CampaignCreated {
        &self.created
    }

    /// Returns a step whose persisted start has no terminal evidence record.
    pub fn unresolved_step(&self) -> Option<&str> {
        self.unresolved.as_ref().map(|(step, _)| step.as_str())
    }

    /// Returns the permanent fresh-copy seal, when target mutation is no longer allowed.
    pub fn fresh_copy_required(&self) -> Option<&FreshCopySeal> {
        self.fresh_copy_required.as_ref()
    }

    /// Verify that every consequential current input still equals the frozen identity.
    pub fn validate_resume(&self, expected: &CampaignCreated) -> Result<()> {
        let expected = normalize_created(expected.clone(), &self.created.managed_root)?;
        compare_identity("install id", &self.created.install_id, &expected.install_id)?;
        compare_identity("attempt id", &self.created.attempt_id, &expected.attempt_id)?;
        compare_identity(
            "managed root",
            &display_path(&self.created.managed_root),
            &display_path(&expected.managed_root),
        )?;
        compare_identity(
            "cache root",
            &display_path(&self.created.cache_root),
            &display_path(&expected.cache_root),
        )?;
        compare_identity(
            "recipe payload",
            &self.created.recipe_payload_sha256,
            &expected.recipe_payload_sha256,
        )?;
        compare_identity(
            "recipe envelope",
            &self.created.recipe_envelope_sha256,
            &expected.recipe_envelope_sha256,
        )?;
        compare_identity(
            "normalized selection",
            &self.created.selection_sha256,
            &expected.selection_sha256,
        )?;
        compare_identity(
            "resolved plan",
            &self.created.plan_sha256,
            &expected.plan_sha256,
        )?;
        compare_serialized(
            "source games",
            &self.created.source_games,
            &expected.source_games,
        )?;
        compare_serialized(
            "artifact identities",
            &self.created.artifact_identities,
            &expected.artifact_identities,
        )?;
        compare_serialized(
            "tool identities",
            &self.created.tool_identities,
            &expected.tool_identities,
        )?;
        compare_identity(
            "staged BG1 path",
            &display_path(&self.created.staged_bg1),
            &display_path(&expected.staged_bg1),
        )?;
        compare_identity(
            "staged BG2 path",
            &display_path(&self.created.staged_bg2),
            &display_path(&expected.staged_bg2),
        )?;
        Ok(())
    }
}

/// Filesystem-backed handle for one create-once managed campaign.
#[derive(Clone, Debug)]
pub struct SessionStore {
    managed_root: PathBuf,
    state_root: PathBuf,
    recipe_root: PathBuf,
    ledger_root: PathBuf,
    attempts_root: PathBuf,
    lock_path: PathBuf,
}

impl SessionStore {
    /// Create a new campaign and atomically publish its immutable record zero.
    pub fn create(managed_root: &Path, event: SessionEvent) -> Result<Self> {
        let canonical_root = canonical_directory(managed_root, "managed root")?;
        let SessionEvent::Created(created) = event else {
            return Err(ledger_error(
                managed_root,
                "the first campaign event must be Created",
            ));
        };
        let created = normalize_created(*created, &canonical_root)?;
        let store = Self::paths(canonical_root);
        match fs::create_dir(&store.state_root) {
            Ok(()) => {}
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(EngineError::SessionAlreadyExists {
                    path: store.state_root,
                });
            }
            Err(source) => {
                return Err(EngineError::Io {
                    path: store.state_root,
                    source,
                });
            }
        }
        for directory in [&store.recipe_root, &store.ledger_root, &store.attempts_root] {
            fs::create_dir(directory).map_err(|source| EngineError::Io {
                path: directory.to_path_buf(),
                source,
            })?;
        }
        let attempt_root = store.attempts_root.join(&created.attempt_id);
        fs::create_dir(&attempt_root).map_err(|source| EngineError::Io {
            path: attempt_root,
            source,
        })?;
        let lock = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&store.lock_path)
            .map_err(|source| EngineError::Io {
                path: store.lock_path.clone(),
                source,
            })?;
        lock.sync_all().map_err(|source| EngineError::Io {
            path: store.lock_path.clone(),
            source,
        })?;
        drop(lock);

        write_create_once(
            &store.recipe_root.join(PAYLOAD_FILE),
            &created.recipe_payload,
        )?;
        write_create_once(
            &store.recipe_root.join(ENVELOPE_FILE),
            &created.recipe_envelope,
        )?;
        let _guard = store.acquire_lock()?;
        store.publish_record(
            0,
            FIRST_PREVIOUS_SHA256.to_owned(),
            SessionEvent::Created(Box::new(created)),
        )?;
        Ok(store)
    }

    /// Open an existing managed campaign without creating fallback state.
    pub fn open(managed_root: &Path) -> Result<Self> {
        let canonical_root = canonical_directory(managed_root, "managed root")?;
        let store = Self::paths(canonical_root);
        for directory in [
            &store.state_root,
            &store.recipe_root,
            &store.ledger_root,
            &store.attempts_root,
        ] {
            validate_directory(directory)?;
        }
        validate_regular_file(&store.lock_path)?;
        Ok(store)
    }

    /// Append one transition after verifying the complete existing chain.
    pub fn append(&self, event: SessionEvent) -> Result<LedgerRecord> {
        if matches!(event, SessionEvent::Created(_)) {
            return Err(ledger_error(
                &self.ledger_root,
                "Created may only appear at sequence zero",
            ));
        }
        validate_progress_event(&event, &self.ledger_root)?;
        let _guard = self.acquire_lock()?;
        let replay = self.replay_unlocked()?;
        validate_transition(&replay, &event, &self.ledger_root)?;
        let sequence = u64::try_from(replay.records.len())
            .map_err(|_| ledger_error(&self.ledger_root, "record count does not fit in u64"))?;
        if sequence > MAX_SEQUENCE {
            return Err(ledger_error(
                &self.ledger_root,
                "ledger exhausted its ten-digit sequence space",
            ));
        }
        let previous_sha256 = replay
            .record_sha256
            .last()
            .cloned()
            .ok_or_else(|| ledger_error(&self.ledger_root, "ledger has no Created record"))?;
        self.publish_record(sequence, previous_sha256, event)
    }

    /// Verify and replay the complete immutable hash chain.
    pub fn replay(&self) -> Result<SessionReplay> {
        let _guard = self.acquire_lock()?;
        self.replay_unlocked()
    }

    fn paths(managed_root: PathBuf) -> Self {
        let state_root = managed_root.join(STATE_DIRECTORY);
        Self {
            managed_root,
            recipe_root: state_root.join(RECIPE_DIRECTORY),
            ledger_root: state_root.join(LEDGER_DIRECTORY),
            attempts_root: state_root.join(ATTEMPTS_DIRECTORY),
            lock_path: state_root.join(LOCK_FILE),
            state_root,
        }
    }

    fn acquire_lock(&self) -> Result<File> {
        validate_regular_file(&self.lock_path)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.lock_path)
            .map_err(|source| EngineError::Io {
                path: self.lock_path.clone(),
                source,
            })?;
        file.lock_exclusive().map_err(|source| EngineError::Io {
            path: self.lock_path.clone(),
            source,
        })?;
        Ok(file)
    }

    fn publish_record(
        &self,
        sequence: u64,
        previous_sha256: String,
        event: SessionEvent,
    ) -> Result<LedgerRecord> {
        let record = LedgerRecord {
            sequence,
            previous_sha256,
            recorded_at: current_unix_millis()?,
            event,
        };
        let final_path = self.ledger_root.join(record_name(sequence));
        let temporary_path = final_path.with_file_name(format!("{}.tmp", record_name(sequence)));
        if final_path.exists() {
            return Err(ledger_error(&final_path, "final record already exists"));
        }
        remove_owned_temporary(&temporary_path)?;
        let bytes = record_bytes(&record, &final_path)?;
        let mut temporary = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary_path)
            .map_err(|source| EngineError::Io {
                path: temporary_path.clone(),
                source,
            })?;
        temporary
            .write_all(&bytes)
            .map_err(|source| EngineError::Io {
                path: temporary_path.clone(),
                source,
            })?;
        temporary.sync_all().map_err(|source| EngineError::Io {
            path: temporary_path.clone(),
            source,
        })?;
        drop(temporary);
        if let Err(source) = fs::hard_link(&temporary_path, &final_path) {
            let _ = fs::remove_file(&temporary_path);
            if final_path.exists() {
                return Err(ledger_error(&final_path, "final record already exists"));
            }
            return Err(EngineError::Io {
                path: final_path,
                source,
            });
        }
        fs::remove_file(&temporary_path).map_err(|source| EngineError::Io {
            path: temporary_path,
            source,
        })?;
        sync_directory(&self.ledger_root)?;
        Ok(record)
    }

    fn replay_unlocked(&self) -> Result<SessionReplay> {
        validate_directory(&self.ledger_root)?;
        let mut finals = BTreeMap::<u64, PathBuf>::new();
        for entry in fs::read_dir(&self.ledger_root).map_err(|source| EngineError::Io {
            path: self.ledger_root.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| EngineError::Io {
                path: self.ledger_root.clone(),
                source,
            })?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|source| EngineError::Io {
                path: path.clone(),
                source,
            })?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(ledger_error(&path, "ledger entries must be regular files"));
            }
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                return Err(ledger_error(&path, "ledger filename is not valid Unicode"));
            };
            if name.ends_with(".tmp") {
                continue;
            }
            let sequence = parse_record_name(&name)
                .ok_or_else(|| ledger_error(&path, "unrecognized final record filename"))?;
            if finals.insert(sequence, path.clone()).is_some() {
                return Err(ledger_error(&path, "duplicate final sequence filename"));
            }
        }
        if finals.is_empty() {
            return Err(ledger_error(&self.ledger_root, "ledger is empty"));
        }

        let mut records = Vec::with_capacity(finals.len());
        let mut hashes = Vec::with_capacity(finals.len());
        let mut expected_previous = FIRST_PREVIOUS_SHA256.to_owned();
        for (expected_sequence, (filename_sequence, path)) in (0_u64..).zip(finals) {
            if filename_sequence != expected_sequence {
                return Err(ledger_error(
                    &path,
                    &format!(
                        "sequence gap: expected {expected_sequence:010}, found {filename_sequence:010}"
                    ),
                ));
            }
            let bytes = fs::read(&path).map_err(|source| EngineError::Io {
                path: path.clone(),
                source,
            })?;
            let record: LedgerRecord =
                serde_json::from_slice(&bytes).map_err(|source| EngineError::SessionJson {
                    path: path.clone(),
                    source,
                })?;
            if record.sequence != filename_sequence {
                return Err(ledger_error(
                    &path,
                    &format!(
                        "record sequence {} does not match filename {filename_sequence}",
                        record.sequence
                    ),
                ));
            }
            if record.previous_sha256 != expected_previous {
                return Err(ledger_error(
                    &path,
                    "previous_sha256 does not match the exact previous record bytes",
                ));
            }
            let hash = sha256_bytes(&bytes);
            expected_previous = hash.clone();
            hashes.push(hash);
            records.push(record);
        }

        let created = match &records[0].event {
            SessionEvent::Created(created) => {
                normalize_created(created.as_ref().clone(), &self.managed_root)?
            }
            _ => {
                return Err(ledger_error(
                    &self.ledger_root,
                    "record zero is not a Created event",
                ));
            }
        };
        if records
            .iter()
            .skip(1)
            .any(|record| matches!(record.event, SessionEvent::Created(_)))
        {
            return Err(ledger_error(
                &self.ledger_root,
                "Created appears after record zero",
            ));
        }
        verify_frozen_recipe(&self.recipe_root, &created)?;
        let attempt_root = self.attempts_root.join(&created.attempt_id);
        validate_directory(&attempt_root)?;

        let progress = replay_progress(&records, &self.ledger_root)?;
        Ok(SessionReplay {
            records,
            record_sha256: hashes,
            created,
            unresolved: progress.unresolved,
            fresh_copy_required: progress.fresh_copy_required,
        })
    }
}

fn normalize_created(
    mut created: CampaignCreated,
    canonical_managed_root: &Path,
) -> Result<CampaignCreated> {
    validate_identifier(&created.install_id, "install id", canonical_managed_root)?;
    validate_identifier(&created.attempt_id, "attempt id", canonical_managed_root)?;
    let event_managed_root = canonical_directory(&created.managed_root, "managed root")?;
    if event_managed_root != canonical_managed_root {
        return Err(identity_mismatch(
            "managed root",
            &display_path(canonical_managed_root),
            &display_path(&event_managed_root),
        ));
    }
    created.managed_root = event_managed_root;
    created.cache_root = canonical_directory(&created.cache_root, "cache root")?;
    if created.staged_bg1 != canonical_managed_root.join("bg1") {
        return Err(identity_mismatch(
            "staged BG1 path",
            &display_path(&canonical_managed_root.join("bg1")),
            &display_path(&created.staged_bg1),
        ));
    }
    if created.staged_bg2 != canonical_managed_root.join("game") {
        return Err(identity_mismatch(
            "staged BG2 path",
            &display_path(&canonical_managed_root.join("game")),
            &display_path(&created.staged_bg2),
        ));
    }
    validate_exact_hash(
        "recipe payload",
        &created.recipe_payload_sha256,
        &sha256_bytes(&created.recipe_payload),
    )?;
    validate_exact_hash(
        "recipe envelope",
        &created.recipe_envelope_sha256,
        &sha256_bytes(&created.recipe_envelope),
    )?;
    let selection_sha256 = selection_digest(&created.normalized_selection)?;
    validate_exact_hash(
        "normalized selection",
        &created.selection_sha256,
        &selection_sha256,
    )?;
    created.recipe_payload_sha256 = created.recipe_payload_sha256.to_ascii_lowercase();
    created.recipe_envelope_sha256 = created.recipe_envelope_sha256.to_ascii_lowercase();
    created.selection_sha256 = created.selection_sha256.to_ascii_lowercase();
    created.plan_sha256 = normalize_hash("resolved plan", &created.plan_sha256)?;
    created.source_games.bg1 = normalize_hash("BG1 source fingerprint", &created.source_games.bg1)?;
    created.source_games.bg2 = normalize_hash("BG2 source fingerprint", &created.source_games.bg2)?;
    normalize_identities(
        "artifact identities",
        &mut created.artifact_identities,
        canonical_managed_root,
    )?;
    normalize_identities(
        "tool identities",
        &mut created.tool_identities,
        canonical_managed_root,
    )?;
    Ok(created)
}

fn normalize_identities(
    label: &'static str,
    identities: &mut [FrozenIdentity],
    error_path: &Path,
) -> Result<()> {
    let mut ids = BTreeSet::new();
    for identity in identities.iter_mut() {
        validate_artifact_identifier(&identity.id, label, error_path)?;
        if identity.version.trim().is_empty() {
            return Err(ledger_error(
                error_path,
                &format!("{label} contains an empty version"),
            ));
        }
        identity.sha256 = normalize_hash(label, &identity.sha256)?;
        if !ids.insert(identity.id.clone()) {
            return Err(ledger_error(
                error_path,
                &format!("{label} contains duplicate id {:?}", identity.id),
            ));
        }
    }
    identities.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(())
}

fn validate_exact_hash(field: &'static str, authored: &str, actual: &str) -> Result<()> {
    let authored = normalize_hash(field, authored)?;
    if authored != actual {
        return Err(identity_mismatch(field, actual, &authored));
    }
    Ok(())
}

fn normalize_hash(field: &'static str, value: &str) -> Result<String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(identity_mismatch(field, "64 hexadecimal characters", value));
    }
    Ok(value.to_ascii_lowercase())
}

fn validate_identifier(value: &str, label: &str, path: &Path) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(ledger_error(
            path,
            &format!("{label} must contain 1-128 ASCII letters, digits, '-' or '_'"),
        ));
    }
    Ok(())
}

// Artifact IDs include dotted upstream versions. Install/attempt IDs above remain stricter.
fn validate_artifact_identifier(value: &str, label: &str, path: &Path) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || matches!(value, "." | "..")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(ledger_error(
            path,
            &format!("{label} contains invalid artifact id {value:?}"),
        ));
    }
    Ok(())
}

fn validate_progress_event(event: &SessionEvent, path: &Path) -> Result<()> {
    match event {
        SessionEvent::Created(_) => Err(ledger_error(path, "Created cannot be appended")),
        SessionEvent::StepStarted { step_id, attempt }
        | SessionEvent::StepCompleted { step_id, attempt } => {
            validate_step(step_id, *attempt, path)
        }
        SessionEvent::StepFailed {
            step_id,
            attempt,
            detail,
        }
        | SessionEvent::FreshCopyRequired {
            step_id,
            attempt,
            detail,
        } => {
            validate_step(step_id, *attempt, path)?;
            if detail.trim().is_empty() {
                return Err(ledger_error(path, "terminal step detail may not be empty"));
            }
            Ok(())
        }
    }
}

fn validate_step(step_id: &str, attempt: u32, path: &Path) -> Result<()> {
    if step_id.is_empty()
        || step_id.len() > 256
        || !step_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'))
    {
        return Err(ledger_error(path, "invalid pipeline step id"));
    }
    if attempt == 0 {
        return Err(ledger_error(path, "step attempts are one-based"));
    }
    Ok(())
}

fn validate_transition(replay: &SessionReplay, next: &SessionEvent, path: &Path) -> Result<()> {
    if replay.fresh_copy_required.is_some() {
        return Err(ledger_error(
            path,
            "campaign is sealed as fresh-copy-required; no later progress is legal",
        ));
    }
    if let (
        None,
        SessionEvent::FreshCopyRequired {
            step_id, attempt, ..
        },
    ) = (&replay.unresolved, next)
    {
        let upgrades_last_failure = replay.records.last().is_some_and(|record| {
            matches!(
                &record.event,
                SessionEvent::StepFailed {
                    step_id: failed_step,
                    attempt: failed_attempt,
                    ..
                } if failed_step == step_id && failed_attempt == attempt
            )
        });
        if upgrades_last_failure {
            return Ok(());
        }
    }
    match (&replay.unresolved, next) {
        (None, SessionEvent::StepStarted { .. }) => Ok(()),
        (
            Some((current_step, current_attempt)),
            SessionEvent::StepCompleted { step_id, attempt }
            | SessionEvent::StepFailed {
                step_id, attempt, ..
            }
            | SessionEvent::FreshCopyRequired {
                step_id, attempt, ..
            },
        ) if current_step == step_id && current_attempt == attempt => Ok(()),
        (Some(_), SessionEvent::StepStarted { .. }) => Err(ledger_error(
            path,
            "cannot start another step before reconciling the unresolved step",
        )),
        (
            None,
            SessionEvent::StepCompleted { .. }
            | SessionEvent::StepFailed { .. }
            | SessionEvent::FreshCopyRequired { .. },
        ) => Err(ledger_error(
            path,
            "terminal step event has no matching StepStarted",
        )),
        (
            Some(_),
            SessionEvent::StepCompleted { .. }
            | SessionEvent::StepFailed { .. }
            | SessionEvent::FreshCopyRequired { .. },
        ) => Err(ledger_error(
            path,
            "terminal step event does not match the unresolved step",
        )),
        (_, SessionEvent::Created(_)) => Err(ledger_error(path, "Created cannot be appended")),
    }
}

struct ReplayProgress {
    unresolved: Option<(String, u32)>,
    fresh_copy_required: Option<FreshCopySeal>,
}

fn replay_progress(records: &[LedgerRecord], path: &Path) -> Result<ReplayProgress> {
    let mut unresolved = None;
    let mut fresh_copy_required = None;
    let mut last_failed = None;
    for record in records.iter().skip(1) {
        if fresh_copy_required.is_some() {
            return Err(ledger_error(
                path,
                "progress appears after a fresh-copy-required seal",
            ));
        }
        validate_progress_event(&record.event, path)?;
        match &record.event {
            SessionEvent::StepStarted { step_id, attempt } => {
                if unresolved.is_some() {
                    return Err(ledger_error(
                        path,
                        "a second StepStarted appears before terminal evidence",
                    ));
                }
                unresolved = Some((step_id.clone(), *attempt));
                last_failed = None;
            }
            SessionEvent::StepCompleted { step_id, attempt } => match unresolved.as_ref() {
                Some((current_step, current_attempt))
                    if current_step == step_id && current_attempt == attempt =>
                {
                    unresolved = None;
                    last_failed = None;
                }
                _ => {
                    return Err(ledger_error(
                        path,
                        "terminal event does not match its StepStarted",
                    ));
                }
            },
            SessionEvent::StepFailed {
                step_id, attempt, ..
            } => match unresolved.as_ref() {
                Some((current_step, current_attempt))
                    if current_step == step_id && current_attempt == attempt =>
                {
                    unresolved = None;
                    last_failed = Some((step_id.clone(), *attempt));
                }
                _ => {
                    return Err(ledger_error(
                        path,
                        "terminal event does not match its StepStarted",
                    ));
                }
            },
            SessionEvent::FreshCopyRequired {
                step_id,
                attempt,
                detail,
            } => {
                let matches_active = unresolved.as_ref() == Some(&(step_id.clone(), *attempt));
                let upgrades_failure = last_failed.as_ref() == Some(&(step_id.clone(), *attempt));
                if matches_active || upgrades_failure {
                    unresolved = None;
                    last_failed = None;
                    fresh_copy_required = Some(FreshCopySeal {
                        step_id: step_id.clone(),
                        attempt: *attempt,
                        detail: detail.clone(),
                    });
                } else {
                    return Err(ledger_error(
                        path,
                        "fresh-copy-required seal does not match its StepStarted",
                    ));
                }
            }
            SessionEvent::Created(_) => {
                return Err(ledger_error(path, "Created appears after record zero"));
            }
        }
    }
    Ok(ReplayProgress {
        unresolved,
        fresh_copy_required,
    })
}

fn verify_frozen_recipe(recipe_root: &Path, created: &CampaignCreated) -> Result<()> {
    let payload_path = recipe_root.join(PAYLOAD_FILE);
    let envelope_path = recipe_root.join(ENVELOPE_FILE);
    let payload = read_regular_file(&payload_path)?;
    let envelope = read_regular_file(&envelope_path)?;
    if payload != created.recipe_payload {
        return Err(identity_mismatch(
            "frozen recipe payload",
            &created.recipe_payload_sha256,
            &sha256_bytes(&payload),
        ));
    }
    if envelope != created.recipe_envelope {
        return Err(identity_mismatch(
            "frozen recipe envelope",
            &created.recipe_envelope_sha256,
            &sha256_bytes(&envelope),
        ));
    }
    Ok(())
}

fn read_regular_file(path: &Path) -> Result<Vec<u8>> {
    validate_regular_file(path)?;
    fs::read(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_create_once(path: &Path, bytes: &[u8]) -> Result<()> {
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ledger_error(path, "frozen recipe filename is not valid Unicode"))?;
    let temporary = path.with_file_name(format!(".{filename}.tmp"));
    remove_owned_temporary(&temporary)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|source| EngineError::Io {
            path: temporary.clone(),
            source,
        })?;
    file.write_all(bytes).map_err(|source| EngineError::Io {
        path: temporary.clone(),
        source,
    })?;
    file.sync_all().map_err(|source| EngineError::Io {
        path: temporary.clone(),
        source,
    })?;
    drop(file);
    if let Err(source) = fs::hard_link(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        if path.exists() {
            return Err(ledger_error(path, "create-once file already exists"));
        }
        return Err(EngineError::Io {
            path: path.to_path_buf(),
            source,
        });
    }
    fs::remove_file(&temporary).map_err(|source| EngineError::Io {
        path: temporary,
        source,
    })?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

fn remove_owned_temporary(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            fs::remove_file(path).map_err(|source| EngineError::Io {
                path: path.to_path_buf(),
                source,
            })
        }
        Ok(_) => Err(ledger_error(path, "temporary path is not a regular file")),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(EngineError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn record_bytes(record: &LedgerRecord, path: &Path) -> Result<Vec<u8>> {
    let mut bytes =
        serde_json::to_vec_pretty(record).map_err(|source| EngineError::SessionJson {
            path: path.to_path_buf(),
            source,
        })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn current_unix_millis() -> Result<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| EngineError::SessionClock)?;
    u64::try_from(duration.as_millis()).map_err(|_| EngineError::SessionClock)
}

fn record_name(sequence: u64) -> String {
    format!("{sequence:010}.json")
}

fn parse_record_name(name: &str) -> Option<u64> {
    let digits = name.strip_suffix(".json")?;
    if digits.len() != 10 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(ledger_error(
            path,
            &format!("{label} must be an existing non-symlink directory"),
        ));
    }
    fs::canonicalize(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(ledger_error(path, "expected a non-symlink directory"));
    }
    Ok(())
}

fn validate_regular_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(ledger_error(path, "expected a regular non-symlink file"));
    }
    Ok(())
}

fn compare_identity(field: &'static str, frozen: &str, current: &str) -> Result<()> {
    if frozen == current {
        Ok(())
    } else {
        Err(identity_mismatch(field, frozen, current))
    }
}

fn compare_serialized<T: Serialize + PartialEq>(
    field: &'static str,
    frozen: &T,
    current: &T,
) -> Result<()> {
    if frozen == current {
        return Ok(());
    }
    let frozen = serde_json::to_vec(frozen).map_err(|source| EngineError::CanonicalDigest {
        context: field,
        source,
    })?;
    let current = serde_json::to_vec(current).map_err(|source| EngineError::CanonicalDigest {
        context: field,
        source,
    })?;
    Err(identity_mismatch(
        field,
        &sha256_bytes(&frozen),
        &sha256_bytes(&current),
    ))
}

fn identity_mismatch(field: &'static str, expected: &str, found: &str) -> EngineError {
    EngineError::SessionIdentityMismatch {
        field,
        expected: expected.to_owned(),
        found: found.to_owned(),
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn ledger_error(path: &Path, message: &str) -> EngineError {
    EngineError::SessionLedger {
        path: path.to_path_buf(),
        message: message.to_owned(),
    }
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<()> {
    // Windows does not provide a portable std API for opening directories for fsync.
    Ok(())
}

#[cfg(not(windows))]
fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| EngineError::Io {
            path: path.to_path_buf(),
            source,
        })
}
