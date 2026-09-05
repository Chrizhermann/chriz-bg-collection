//! Explicit, create-once provenance for a supervised recovery of a failed managed install.

use std::collections::BTreeSet;
#[cfg(not(windows))]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::digest::{plan_digest, sha256_bytes};
use crate::manifest::GameRoot;
use crate::postcondition::is_safe_normalized_relative_path;
use crate::receipt::{
    FinalReceiptState, InstallReceipt, LogComponentReceipt, ReceiptOutcome, RECEIPT_SCHEMA_VERSION,
};
use crate::session::FrozenIdentity;

/// Recovery receipt schema emitted by this engine version.
pub const RECOVERY_RECEIPT_SCHEMA_VERSION: u32 = 1;

const STATE_DIRECTORY: &str = ".chriz";
const ATTEMPTS_DIRECTORY: &str = "attempts";
const RECEIPT_FILE: &str = "install-receipt.json";
const ATTEMPT_RECEIPT_FILE: &str = "receipt.json";
const MAX_EVIDENCE_FILES: usize = 1_024;
const MAX_RELATIVE_PATH_BYTES: usize = 512;
const MAX_RELATIVE_PATH_COMPONENTS: usize = 64;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// One immutable file whose current bytes substantiate the recovery.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFile {
    /// Canonical forward-slash path relative to the managed root.
    pub path: PathBuf,
    /// SHA-256 of the exact file bytes.
    pub sha256: String,
}

/// One planned run whose payload artifact changed during supervised recovery.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReplacement {
    /// Stable run id from the frozen plan.
    pub run_id: String,
    /// Exact artifact identity frozen by the failed attempt.
    pub original: FrozenIdentity,
    /// Exact replacement artifact used by the supervised operation.
    pub replacement: FrozenIdentity,
}

/// Distinct provenance kind accepted at the stable completed-install receipt path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryKind {
    /// A bounded, human-supervised recovery with independently verified evidence.
    SupervisedRecovery,
}

/// Immutable record proving how a failed managed install reached a completed state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryReceipt {
    /// Recovery receipt schema version.
    pub schema_version: u32,
    /// Explicit receipt kind; this never masquerades as a successful ordinary attempt.
    pub kind: RecoveryKind,
    /// Stable path-safe recovery identifier.
    pub recovery_id: String,
    /// Managed-install id inherited from the failed receipt.
    pub install_id: String,
    /// Canonical managed installation root.
    pub managed_root: PathBuf,
    /// Attempt directory containing the immutable failed base receipt.
    pub base_attempt_id: String,
    /// SHA-256 of the exact failed base receipt bytes.
    pub base_receipt_sha256: String,
    /// Frozen recipe version used by the failed attempt.
    pub base_recipe_version: String,
    /// Frozen signed recipe-payload SHA-256 used by the failed attempt.
    pub base_recipe_payload_sha256: String,
    /// Frozen resolved-plan SHA-256 used by the failed attempt.
    pub base_plan_sha256: String,
    /// Changed payload identities, linked to exact frozen plan runs.
    pub replacements: Vec<ArtifactReplacement>,
    /// Nonempty manifest of immutable recovery evidence below the managed root.
    pub evidence: Vec<EvidenceFile>,
    /// Recovery completion time in Unix epoch milliseconds.
    pub completed_at_millis: u64,
    /// Final launch and log state verified after recovery.
    pub final_state: FinalReceiptState,
}

impl RecoveryReceipt {
    /// Returns a compact display version retaining the base recipe while marking the repair.
    pub fn effective_version(&self) -> String {
        format!("{} (repaired)", self.base_recipe_version)
    }

    /// Validates recovery lineage and structure against one immutable failed receipt.
    pub fn validate(&self, base: &InstallReceipt) -> Result<(), String> {
        if self.schema_version != RECOVERY_RECEIPT_SCHEMA_VERSION {
            return Err(format!(
                "unsupported recovery schema {}; expected {RECOVERY_RECEIPT_SCHEMA_VERSION}",
                self.schema_version
            ));
        }
        validate_identifier(&self.recovery_id, "recovery id")?;
        validate_identifier(&self.install_id, "install id")?;
        validate_identifier(&self.base_attempt_id, "base attempt id")?;
        if base.schema_version != RECEIPT_SCHEMA_VERSION {
            return Err(format!(
                "unsupported base receipt schema {}; expected {RECEIPT_SCHEMA_VERSION}",
                base.schema_version
            ));
        }
        if !matches!(
            base.outcome,
            ReceiptOutcome::Failed { .. } | ReceiptOutcome::FreshCopyRequired { .. }
        ) || base.final_state.is_some()
        {
            return Err(
                "recovery base must be a failed or fresh-copy receipt without final state"
                    .to_owned(),
            );
        }
        if base.completed_at_millis < base.started_at_millis
            || self.completed_at_millis < base.completed_at_millis
        {
            return Err("recovery timestamps precede their immutable base state".to_owned());
        }
        if self.install_id != base.install_id {
            return Err("recovery install id differs from the base receipt".to_owned());
        }
        if self.managed_root != base.managed_root || !is_absolute_normalized(&self.managed_root) {
            return Err("recovery managed root differs from the normalized base root".to_owned());
        }
        if self.base_attempt_id != base.attempt_id {
            return Err("recovery base attempt id differs from the base receipt".to_owned());
        }
        validate_hash("base receipt", &self.base_receipt_sha256)?;
        let canonical_base = install_receipt_bytes(base)?;
        if !self
            .base_receipt_sha256
            .eq_ignore_ascii_case(&sha256_bytes(&canonical_base))
        {
            return Err("recovery base receipt digest does not match the base receipt".to_owned());
        }
        if self.base_recipe_version.trim().is_empty()
            || self.base_recipe_version != base.versions.recipe
        {
            return Err("recovery base recipe version differs from the base receipt".to_owned());
        }
        validate_hash("base recipe payload", &self.base_recipe_payload_sha256)?;
        if self.base_recipe_payload_sha256 != base.recipe_payload_sha256 {
            return Err("recovery recipe payload digest differs from the base receipt".to_owned());
        }
        validate_hash("base plan", &self.base_plan_sha256)?;
        if self.base_plan_sha256 != base.plan_sha256
            || plan_digest(&base.plan).map_err(|error| error.to_string())? != base.plan_sha256
        {
            return Err("recovery plan digest does not match the exact base plan".to_owned());
        }

        self.validate_replacements(base)?;
        validate_evidence_manifest(&self.evidence)?;
        validate_final_state(&self.managed_root, &self.final_state)
    }

    fn validate_replacements(&self, base: &InstallReceipt) -> Result<(), String> {
        let mut seen_runs = BTreeSet::new();
        for replacement in &self.replacements {
            if !seen_runs.insert(replacement.run_id.as_str()) {
                return Err(format!(
                    "recovery replacement run {:?} is duplicated",
                    replacement.run_id
                ));
            }
            let mut planned = base
                .plan
                .runs
                .iter()
                .filter(|run| run.run_id == replacement.run_id);
            let run = planned.next().ok_or_else(|| {
                format!(
                    "recovery replacement references unknown plan run {:?}",
                    replacement.run_id
                )
            })?;
            if planned.next().is_some() {
                return Err(format!(
                    "base plan run {:?} is ambiguous",
                    replacement.run_id
                ));
            }
            let matching_artifacts = base
                .artifacts
                .iter()
                .filter(|artifact| artifact.id == run.artifact_id)
                .collect::<Vec<_>>();
            if matching_artifacts.len() != 1 {
                return Err(format!(
                    "base receipt has no unique artifact for run {:?}",
                    replacement.run_id
                ));
            }
            let artifact = matching_artifacts[0];
            let expected_original = FrozenIdentity {
                id: artifact.id.clone(),
                version: artifact.version.clone(),
                sha256: artifact.sha256.clone(),
                length: artifact.length,
            };
            if replacement.original != expected_original {
                return Err(format!(
                    "recovery replacement for run {:?} does not name its exact frozen artifact",
                    replacement.run_id
                ));
            }
            validate_identity("original artifact", &replacement.original)?;
            validate_identity("replacement artifact", &replacement.replacement)?;
            if replacement.replacement.id != replacement.original.id {
                return Err(format!(
                    "recovery replacement for run {:?} changes artifact id",
                    replacement.run_id
                ));
            }
        }
        Ok(())
    }
}

/// Verifies every recorded evidence file below a direct, non-reparse managed root.
pub fn verify_evidence(root: &Path, evidence: &[EvidenceFile]) -> Result<(), String> {
    validate_evidence_manifest(evidence)?;
    let root = canonical_direct_directory(root)?;
    for item in evidence {
        let relative = evidence_path(&item.path)?;
        let path = direct_file_below(&root, &relative)?;
        let actual = sha256_file(&path)?;
        if !actual.eq_ignore_ascii_case(&item.sha256) {
            return Err(format!(
                "recovery evidence hash mismatch for {}",
                item.path.display()
            ));
        }
    }
    Ok(())
}

/// Reads either an ordinary successful receipt or a verified explicit recovery state.
pub fn read_completed_state(root: &Path) -> Result<FinalReceiptState, String> {
    let root = canonical_direct_directory(root)?;
    let receipt_path = direct_file_below(&root, Path::new(STATE_DIRECTORY).join(RECEIPT_FILE))?;
    let bytes = read_file(&receipt_path)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{}: invalid receipt JSON: {error}", receipt_path.display()))?;
    if value.get("kind").is_some() {
        let recovery: RecoveryReceipt = serde_json::from_value(value).map_err(|error| {
            format!(
                "{}: invalid recovery receipt: {error}",
                receipt_path.display()
            )
        })?;
        let (base, base_bytes) = read_base_receipt(&root, &recovery.base_attempt_id)?;
        if !recovery
            .base_receipt_sha256
            .eq_ignore_ascii_case(&sha256_bytes(&base_bytes))
        {
            return Err("recovery base receipt file hash does not match its lineage".to_owned());
        }
        recovery.validate(&base)?;
        verify_evidence(&root, &recovery.evidence)?;
        return Ok(recovery.final_state);
    }

    let ordinary: InstallReceipt = serde_json::from_value(value).map_err(|error| {
        format!(
            "{}: invalid install receipt: {error}",
            receipt_path.display()
        )
    })?;
    if !matches!(ordinary.outcome, ReceiptOutcome::Succeeded) {
        return Err("ordinary install receipt is not succeeded".to_owned());
    }
    ordinary
        .final_state
        .ok_or_else(|| "successful install receipt has no final state".to_owned())
}

/// Verifies and create-once publishes a recovery receipt at the stable install path.
/// Byte-identical calls are idempotent; a different existing document is never replaced.
pub fn publish(root: &Path, receipt: &RecoveryReceipt) -> Result<PathBuf, String> {
    let root = canonical_direct_directory(root)?;
    if receipt.managed_root != root {
        return Err("recovery receipt managed root differs from the publication root".to_owned());
    }
    let (base, base_bytes) = read_base_receipt(&root, &receipt.base_attempt_id)?;
    if !receipt
        .base_receipt_sha256
        .eq_ignore_ascii_case(&sha256_bytes(&base_bytes))
    {
        return Err("recovery base receipt file hash does not match its lineage".to_owned());
    }
    receipt.validate(&base)?;
    verify_evidence(&root, &receipt.evidence)?;

    let mut bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("could not serialize recovery receipt: {error}"))?;
    bytes.push(b'\n');
    let state_root = root.join(STATE_DIRECTORY);
    validate_direct_directory(&state_root)?;
    let path = state_root.join(RECEIPT_FILE);
    reconcile_existing(&path, &bytes)?;
    publish_if_missing(&path, &bytes)?;
    Ok(path)
}

fn validate_evidence_manifest(evidence: &[EvidenceFile]) -> Result<(), String> {
    if evidence.is_empty() || evidence.len() > MAX_EVIDENCE_FILES {
        return Err(format!(
            "recovery evidence manifest must contain 1..={MAX_EVIDENCE_FILES} files"
        ));
    }
    let mut seen = BTreeSet::new();
    for item in evidence {
        let path = evidence_path(&item.path)?;
        validate_hash("recovery evidence", &item.sha256)?;
        let identity = path.to_string_lossy().to_ascii_lowercase();
        if !seen.insert(identity) {
            return Err(format!(
                "recovery evidence path {} is duplicated",
                item.path.display()
            ));
        }
    }
    Ok(())
}

fn validate_final_state(root: &Path, state: &FinalReceiptState) -> Result<(), String> {
    if state.bg1_engine_name.trim().is_empty()
        || state.bg2_engine_name.trim().is_empty()
        || state.verification_summary.trim().is_empty()
    {
        return Err(
            "recovery final state has empty engine identity or verification summary".to_owned(),
        );
    }
    if !is_absolute_normalized(&state.managed_save_root) {
        return Err("recovery managed save root must be absolute and normalized".to_owned());
    }
    if !is_absolute_normalized(&state.launch_path)
        || !path_is_strictly_below(root, &state.launch_path)
    {
        return Err("recovery launch path must be normalized below the managed root".to_owned());
    }
    if state.logs.len() != 2 {
        return Err("recovery final state requires exactly one BG1 and one BG2 log".to_owned());
    }
    let mut targets = BTreeSet::new();
    let mut components = BTreeSet::new();
    for log in &state.logs {
        validate_hash("final WeiDU.log", &log.sha256)?;
        if !targets.insert(log.target) {
            return Err("recovery final state duplicates a staged-root log".to_owned());
        }
        if log.components.is_empty() {
            return Err("recovery final WeiDU.log has no active component identities".to_owned());
        }
        for component in &log.components {
            validate_log_component(component)?;
            if !components.insert((
                log.target,
                component.tp2.to_ascii_lowercase(),
                component.language,
                component.component,
            )) {
                return Err(
                    "recovery final state duplicates an active component identity".to_owned(),
                );
            }
        }
    }
    if !targets.contains(&GameRoot::Bg1) || !targets.contains(&GameRoot::Bg2) {
        return Err("recovery final state requires both BG1 and BG2 logs".to_owned());
    }
    Ok(())
}

fn validate_log_component(component: &LogComponentReceipt) -> Result<(), String> {
    if component.tp2.len() > MAX_RELATIVE_PATH_BYTES
        || !is_safe_normalized_relative_path(&component.tp2)
    {
        return Err(format!(
            "unsafe recovery log component path {:?}",
            component.tp2
        ));
    }
    Ok(())
}

fn validate_identity(label: &str, identity: &FrozenIdentity) -> Result<(), String> {
    validate_identifier(&identity.id, &format!("{label} id"))?;
    if identity.version.trim().is_empty() || identity.version.len() > 256 {
        return Err(format!("{label} version is empty or too long"));
    }
    validate_hash(label, &identity.sha256)?;
    if identity.length == 0 {
        return Err(format!("{label} length must be positive"));
    }
    Ok(())
}

fn validate_identifier(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(format!("{label} is not a bounded path-safe identifier"));
    }
    Ok(())
}

fn validate_hash(label: &str, value: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "{label} SHA-256 is not exactly 64 hexadecimal characters"
        ));
    }
    Ok(())
}

fn evidence_path(path: &Path) -> Result<PathBuf, String> {
    let text = path
        .to_str()
        .ok_or_else(|| format!("recovery evidence path {} is not Unicode", path.display()))?;
    let component_count = text.split('/').count();
    if text.len() > MAX_RELATIVE_PATH_BYTES
        || component_count > MAX_RELATIVE_PATH_COMPONENTS
        || !is_safe_normalized_relative_path(text)
    {
        return Err(format!(
            "recovery evidence path {} is not bounded and target-relative",
            path.display()
        ));
    }
    Ok(PathBuf::from(text))
}

fn install_receipt_bytes(receipt: &InstallReceipt) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("could not serialize base receipt: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_base_receipt(root: &Path, attempt_id: &str) -> Result<(InstallReceipt, Vec<u8>), String> {
    validate_identifier(attempt_id, "base attempt id")?;
    let relative = Path::new(STATE_DIRECTORY)
        .join(ATTEMPTS_DIRECTORY)
        .join(attempt_id)
        .join(ATTEMPT_RECEIPT_FILE);
    let path = direct_file_below(root, relative)?;
    let bytes = read_file(&path)?;
    let receipt = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{}: invalid base receipt: {error}", path.display()))?;
    Ok((receipt, bytes))
}

fn canonical_direct_directory(path: &Path) -> Result<PathBuf, String> {
    validate_direct_directory(path)?;
    fs::canonicalize(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn validate_direct_directory(path: &Path) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_dir() || is_link_or_reparse(&metadata) {
        return Err(format!(
            "{} is not a direct non-reparse directory",
            path.display()
        ));
    }
    Ok(())
}

fn direct_file_below(root: &Path, relative: impl AsRef<Path>) -> Result<PathBuf, String> {
    let relative = relative.as_ref();
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "path {} is not a direct relative path",
            relative.display()
        ));
    }
    validate_direct_directory(root)?;
    let components = relative.components().collect::<Vec<_>>();
    if components.is_empty() {
        return Err("direct relative path is empty".to_owned());
    }
    let mut current = root.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err("direct relative path contains a non-normal component".to_owned());
        };
        current.push(name);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| format!("{}: {error}", current.display()))?;
        if is_link_or_reparse(&metadata) {
            return Err(format!(
                "{} is a symbolic link or reparse point",
                current.display()
            ));
        }
        let is_last = index + 1 == components.len();
        if (is_last && !metadata.is_file()) || (!is_last && !metadata.is_dir()) {
            return Err(format!(
                "{} has the wrong direct file type",
                current.display()
            ));
        }
    }
    Ok(current)
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn reconcile_existing(path: &Path, expected: &[u8]) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file() || is_link_or_reparse(&metadata) {
                return Err(format!(
                    "{} is not a direct recovery receipt file",
                    path.display()
                ));
            }
            let actual = read_file(path)?;
            if actual != expected {
                return Err(format!(
                    "create-once recovery receipt already exists with different bytes at {}",
                    path.display()
                ));
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

fn publish_if_missing(path: &Path, bytes: &[u8]) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(_) => return reconcile_existing(path, bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("{}: {error}", path.display())),
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent", path.display()))?;
    validate_direct_directory(parent)?;
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(
        ".{RECEIPT_FILE}.{}.{}.tmp",
        std::process::id(),
        sequence
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        file.write_all(bytes)
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        drop(file);
        match fs::hard_link(&temporary, path) {
            Ok(()) => Ok(()),
            Err(link_error) => match fs::symlink_metadata(path) {
                Ok(_) => reconcile_existing(path, bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    Err(format!("{}: {link_error}", path.display()))
                }
                Err(error) => Err(format!("{}: {error}", path.display())),
            },
        }
    })();
    let cleanup = fs::remove_file(&temporary);
    if let Err(error) = result {
        let _ = cleanup;
        return Err(error);
    }
    cleanup.map_err(|error| format!("{}: {error}", temporary.display()))?;
    sync_directory(parent)
}

fn is_absolute_normalized(path: &Path) -> bool {
    path.is_absolute()
        && path
            .components()
            .all(|component| !matches!(component, Component::CurDir | Component::ParentDir))
}

fn path_is_strictly_below(root: &Path, path: &Path) -> bool {
    let root_components = root.components().collect::<Vec<_>>();
    let path_components = path.components().collect::<Vec<_>>();
    path_components.len() > root_components.len()
        && root_components
            .iter()
            .zip(path_components.iter())
            .all(|(left, right)| component_eq(left, right))
}

fn component_eq(left: &Component<'_>, right: &Component<'_>) -> bool {
    #[cfg(windows)]
    {
        left.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink() || has_windows_reparse_attribute(metadata)
}

#[cfg(windows)]
fn has_windows_reparse_attribute(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn has_windows_reparse_attribute(_metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("{}: {error}", path.display()))
}
