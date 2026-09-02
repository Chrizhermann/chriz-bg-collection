//! Create-once machine-readable receipts for managed installation attempts.

use std::collections::BTreeSet;
#[cfg(not(windows))]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::games::{GameRole, Storefront};
use crate::manifest::GameRoot;
use crate::orchestrator::{ReceiptDraft, ReceiptDraftOutcome, ReceiptWriter, StepFailure};
use crate::recipe_view::NormalizedSelection;
use crate::registry::{
    ManagedInstallRecord, ManagedInstallRegistry, RegistryError, REGISTRY_SCHEMA_VERSION,
};
use crate::resolve::InstallPlan;
use crate::session::FrozenIdentity;

/// Receipt schema emitted by this engine version.
pub const RECEIPT_SCHEMA_VERSION: u32 = 1;

const STATE_DIRECTORY: &str = ".chriz";
const ATTEMPTS_DIRECTORY: &str = "attempts";
const ATTEMPT_RECEIPT_FILE: &str = "receipt.json";
const INSTALL_RECEIPT_FILE: &str = "install-receipt.json";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Terminal outcome represented by one attempt receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ReceiptOutcome {
    /// Final verification passed and the managed installation is launchable.
    Succeeded,
    /// A retryable campaign step failed.
    Failed {
        /// Stable pipeline step that failed.
        step_id: String,
        /// Sanitizable human-readable failure evidence.
        detail: String,
    },
    /// Existing target evidence was unsafe or ambiguous.
    FreshCopyRequired {
        /// Stable pipeline step that could not be reconciled.
        step_id: String,
        /// Sanitizable human-readable evidence summary.
        detail: String,
    },
}

impl ReceiptOutcome {
    fn is_success(&self) -> bool {
        matches!(self, Self::Succeeded)
    }
}

/// Application, engine, schema, and immutable recipe versions used by an attempt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptVersions {
    /// Packaged desktop application version.
    pub application: String,
    /// Headless engine crate version.
    pub engine: String,
    /// Parsed collection-manifest schema version.
    pub manifest_schema: u32,
    /// Signed recipe release version.
    pub recipe: String,
}

/// One exact source-game identity accepted before staging.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceGameReceipt {
    /// Source role in the two-root EET build.
    pub role: GameRole,
    /// Storefront whose clean profile matched.
    pub storefront: Storefront,
    /// Exact executable/product version.
    pub version: String,
    /// Deterministic clean-source fingerprint.
    pub fingerprint: String,
}

/// How an immutable artifact became available to this attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCacheOutcome {
    /// A complete cache object was rehashed and reused.
    Hit,
    /// A transfer was verified and published to the cache.
    Downloaded,
    /// A user-supplied archive was verified and imported.
    Manual,
}

/// Exact acquisition identity and provenance for one payload archive.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReceipt {
    /// Stable recipe artifact id.
    pub id: String,
    /// Exact upstream version or revision.
    pub version: String,
    /// Immutable URL authored by the recipe or manual handoff URL.
    pub original_url: String,
    /// Final HTTPS response URL, or the same handoff URL for manual input.
    pub final_url: String,
    /// Verified archive byte length.
    pub length: u64,
    /// Verified archive SHA-256.
    pub sha256: String,
    /// Whether acquisition downloaded, imported, or reused the archive.
    pub cache_outcome: ArtifactCacheOutcome,
}

/// Exact executable identity for a WeiDU tool used by one or more runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeiDuToolReceipt {
    /// Stable tool artifact id.
    pub id: String,
    /// WeiDU version reported by the verified executable.
    pub version: String,
    /// Verified executable byte length.
    pub length: u64,
    /// Verified executable SHA-256.
    pub sha256: String,
}

/// One output-gated prompt observed during a WeiDU run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptReceipt {
    /// Expected output marker represented without process-local byte offsets.
    pub expected_output: String,
    /// Exact authored answer, with private paths replaced by semantic placeholders.
    pub answer: String,
    /// Whether the supervisor observed and answered the prompt.
    pub matched: bool,
}

/// Start and completion timestamps for one serialized WeiDU child.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunTiming {
    /// Unix epoch milliseconds immediately before spawn.
    pub started_at_millis: u64,
    /// Unix epoch milliseconds after output and log evidence were synchronized.
    pub completed_at_millis: u64,
}

/// Active WeiDU.log identity for one component.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogComponentReceipt {
    /// Normalized TP2 path.
    pub tp2: String,
    /// WeiDU language number.
    pub language: u32,
    /// DESIGNATED component number.
    pub component: u32,
}

/// Exact before/after WeiDU.log evidence for one run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogDiffReceipt {
    /// SHA-256 of the synchronized pre-run log snapshot.
    pub before_sha256: String,
    /// SHA-256 of the synchronized post-run log snapshot.
    pub after_sha256: String,
    /// Strict ordered entries appended by the run.
    pub added: Vec<LogComponentReceipt>,
    /// Entries removed by the run; successful installs require this to be empty.
    pub removed: Vec<LogComponentReceipt>,
}

/// Complete durable evidence for one serialized WeiDU invocation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunReceipt {
    /// Stable recipe run id.
    pub run_id: String,
    /// Staged game root mutated by the run.
    pub target: GameRoot,
    /// Exact ordered selected component suffix.
    pub components: Vec<u32>,
    /// Output-gated prompt observations.
    pub prompts: Vec<PromptReceipt>,
    /// Spawn and evidence-synchronization timing.
    pub timing: RunTiming,
    /// Child process exit code.
    pub exit_code: i32,
    /// Warnings accepted or surfaced by verification.
    pub warnings: Vec<String>,
    /// Semantic digest of the constructed invocation.
    pub invocation_sha256: String,
    /// SHA-256 of captured stdout.
    pub stdout_sha256: String,
    /// SHA-256 of captured stderr.
    pub stderr_sha256: String,
    /// SHA-256 of the engine-owned WeiDU debug log.
    pub debug_sha256: String,
    /// Strict active-log change proved for this run.
    pub log_diff: LogDiffReceipt,
}

/// Exact final WeiDU.log identity for one staged root.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalLogReceipt {
    /// Staged root whose log was verified.
    pub target: GameRoot,
    /// SHA-256 of the final raw WeiDU.log bytes.
    pub sha256: String,
    /// Exact ordered active component identities.
    pub components: Vec<LogComponentReceipt>,
}

/// Launch and identity state present only after successful final verification.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalReceiptState {
    /// Final BG1 and BG2 WeiDU.log identities.
    pub logs: Vec<FinalLogReceipt>,
    /// Read-back BG1 staging identity.
    pub bg1_engine_name: String,
    /// Read-back final EET identity.
    pub bg2_engine_name: String,
    /// Reserved per-user directory for saves and configuration.
    pub managed_save_root: PathBuf,
    /// Verified executable used by Complete and Home.
    pub launch_path: PathBuf,
    /// Concise final-stack verification evidence.
    pub verification_summary: String,
}

/// Immutable machine-readable record of one installation attempt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallReceipt {
    /// Receipt schema version.
    pub schema_version: u32,
    /// Stable managed-install id.
    pub install_id: String,
    /// Stable attempt id naming the receipt directory.
    pub attempt_id: String,
    /// Campaign attempt directory containing the Task 13 step evidence for this outcome.
    pub evidence_attempt_id: String,
    /// Canonical root of the managed installation.
    pub managed_root: PathBuf,
    /// Frozen managed BG1 pre-merge staging root.
    pub staged_bg1: PathBuf,
    /// Frozen managed BG2/EET game root.
    pub staged_bg2: PathBuf,
    /// Attempt start in Unix epoch milliseconds.
    pub started_at_millis: u64,
    /// Terminal receipt time in Unix epoch milliseconds.
    pub completed_at_millis: u64,
    /// Successful, failed, or unsafe-to-resume result.
    pub outcome: ReceiptOutcome,
    /// Application and data format versions.
    pub versions: ReceiptVersions,
    /// Both clean source-game identities.
    pub source_games: Vec<SourceGameReceipt>,
    /// Frozen signed recipe-payload digest.
    pub recipe_payload_sha256: String,
    /// Frozen signed recipe-envelope digest.
    pub recipe_envelope_sha256: String,
    /// Canonical normalized-selection digest.
    pub selection_sha256: String,
    /// Exact normalized semantic selection used to resolve the plan.
    pub normalized_selection: NormalizedSelection,
    /// Canonical resolved-plan digest.
    pub plan_sha256: String,
    /// Exact resolved plan executed by the orchestrator.
    pub plan: InstallPlan,
    /// Every acquired payload archive.
    pub artifacts: Vec<ArtifactReceipt>,
    /// Every verified WeiDU executable.
    pub weidu_tools: Vec<WeiDuToolReceipt>,
    /// Per-run invocation and reconciliation evidence.
    pub runs: Vec<RunReceipt>,
    /// Final log, identity, save-root, and launch evidence on success.
    pub final_state: Option<FinalReceiptState>,
}

/// Paths durably published for an attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedReceipt {
    /// Attempt-local receipt, present for every terminal outcome.
    pub attempt_receipt: PathBuf,
    /// Stable successful-install receipt, present only for success.
    pub install_receipt: Option<PathBuf>,
}

/// Failure to validate or publish immutable receipt evidence.
#[derive(Debug, Error)]
pub enum ReceiptError {
    /// A root, state directory, or attempt directory is missing or unsafe.
    #[error("unsafe receipt path {path}: {message}")]
    UnsafePath {
        /// Rejected path.
        path: PathBuf,
        /// Validation detail.
        message: String,
    },
    /// The supplied receipt contradicts its store or schema contract.
    #[error("invalid receipt: {0}")]
    InvalidReceipt(String),
    /// A different immutable receipt already owns the destination.
    #[error("create-once receipt already exists with different bytes at {path}")]
    CreateOnceConflict {
        /// Existing final path.
        path: PathBuf,
    },
    /// JSON serialization failed.
    #[error("could not serialize receipt for {path}: {source}")]
    Json {
        /// Intended final path.
        path: PathBuf,
        /// Serialization detail.
        #[source]
        source: serde_json::Error,
    },
    /// Filesystem publication failed.
    #[error("{path}: {source}")]
    Io {
        /// Path being read, written, linked, or synchronized.
        path: PathBuf,
        /// Underlying error.
        #[source]
        source: std::io::Error,
    },
}

/// Filesystem-backed create-once publisher for one managed installation.
#[derive(Clone, Debug)]
pub struct ReceiptStore {
    managed_root: PathBuf,
    state_root: PathBuf,
    attempts_root: PathBuf,
    install_id: String,
}

/// Evidence accumulated by acquisition, execution, and final verification services.
///
/// Frozen digests and the exact plan come from [`ReceiptDraft`], so a caller cannot
/// accidentally substitute current recipe data when resuming an older campaign.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptEvidence {
    /// Application, engine, schema, and signed recipe versions.
    pub versions: ReceiptVersions,
    /// Exact BG1 and BG2 source metadata and fingerprints.
    pub source_games: Vec<SourceGameReceipt>,
    /// Acquisition provenance and cache outcomes for every payload.
    pub artifacts: Vec<ArtifactReceipt>,
    /// Verified WeiDU executable identities.
    pub weidu_tools: Vec<WeiDuToolReceipt>,
    /// Exact per-run process and log evidence.
    pub runs: Vec<RunReceipt>,
    /// Final log, identity, save-root, and launch evidence.
    pub final_state: Option<FinalReceiptState>,
}

/// Failure to assemble, publish, or register a real orchestration receipt.
#[derive(Debug, Error)]
pub enum ManagedReceiptError {
    /// Accumulated evidence contradicts the frozen campaign.
    #[error("receipt evidence does not match the frozen campaign: {0}")]
    Evidence(String),
    /// Create-once receipt publication failed.
    #[error(transparent)]
    Receipt(#[from] ReceiptError),
    /// Create-once managed-install registration failed.
    #[error(transparent)]
    Registry(#[from] RegistryError),
    /// Reading the just-published receipt for its registry digest failed.
    #[error("{path}: {source}")]
    Io {
        /// Published successful receipt path.
        path: PathBuf,
        /// Underlying read failure.
        #[source]
        source: std::io::Error,
    },
}

/// Real Task-13 receipt sink: publish full evidence, then register the successful install.
#[derive(Debug)]
pub struct ManagedReceiptWriter {
    store: ReceiptStore,
    registry: ManagedInstallRegistry,
    display_name: String,
    evidence: ReceiptEvidence,
}

impl ManagedReceiptWriter {
    /// Construct a writer from already synchronized evidence services.
    pub fn new(
        store: ReceiptStore,
        registry: ManagedInstallRegistry,
        display_name: String,
        evidence: ReceiptEvidence,
    ) -> Self {
        Self {
            store,
            registry,
            display_name,
            evidence,
        }
    }

    /// Assemble, publish, and register the successful campaign hand-off.
    pub fn publish_success(
        &mut self,
        draft: &ReceiptDraft,
    ) -> Result<PublishedReceipt, ManagedReceiptError> {
        if !matches!(draft.outcome, ReceiptDraftOutcome::Succeeded) {
            return Err(ManagedReceiptError::Evidence(
                "publish_success received a non-success draft".to_owned(),
            ));
        }
        self.publish_terminal(draft)
    }

    fn publish_terminal(
        &mut self,
        draft: &ReceiptDraft,
    ) -> Result<PublishedReceipt, ManagedReceiptError> {
        let succeeded = matches!(draft.outcome, ReceiptDraftOutcome::Succeeded);
        self.validate_frozen_evidence(draft, succeeded)?;
        let final_state = if succeeded {
            Some(self.evidence.final_state.clone().ok_or_else(|| {
                ManagedReceiptError::Evidence("successful campaign has no final state".to_owned())
            })?)
        } else {
            None
        };
        let outcome = match &draft.outcome {
            ReceiptDraftOutcome::Succeeded => ReceiptOutcome::Succeeded,
            ReceiptDraftOutcome::Failed { step_id, detail } => ReceiptOutcome::Failed {
                step_id: step_id.clone(),
                detail: detail.clone(),
            },
            ReceiptDraftOutcome::FreshCopyRequired { step_id, detail } => {
                ReceiptOutcome::FreshCopyRequired {
                    step_id: step_id.clone(),
                    detail: detail.clone(),
                }
            }
        };
        let receipt = InstallReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            install_id: draft.install_id.clone(),
            attempt_id: draft.attempt_id.clone(),
            evidence_attempt_id: draft.created.attempt_id.clone(),
            managed_root: draft.created.managed_root.clone(),
            staged_bg1: draft.created.staged_bg1.clone(),
            staged_bg2: draft.created.staged_bg2.clone(),
            started_at_millis: draft.started_at_millis,
            completed_at_millis: draft.completed_at_millis,
            outcome,
            versions: self.evidence.versions.clone(),
            source_games: self.evidence.source_games.clone(),
            recipe_payload_sha256: draft.created.recipe_payload_sha256.clone(),
            recipe_envelope_sha256: draft.created.recipe_envelope_sha256.clone(),
            selection_sha256: draft.created.selection_sha256.clone(),
            normalized_selection: draft.created.normalized_selection.clone(),
            plan_sha256: draft.created.plan_sha256.clone(),
            plan: draft.plan.clone(),
            artifacts: self.evidence.artifacts.clone(),
            weidu_tools: self.evidence.weidu_tools.clone(),
            runs: self.evidence.runs.clone(),
            final_state: final_state.clone(),
        };
        let published = self.store.publish(&receipt)?;
        let Some(final_state) = final_state else {
            return Ok(published);
        };
        let install_path = published.install_receipt.as_ref().ok_or_else(|| {
            ManagedReceiptError::Evidence(
                "successful receipt publisher omitted install-receipt.json".to_owned(),
            )
        })?;
        let receipt_bytes = fs::read(install_path).map_err(|source| ManagedReceiptError::Io {
            path: install_path.clone(),
            source,
        })?;
        let record = ManagedInstallRecord {
            schema_version: REGISTRY_SCHEMA_VERSION,
            install_id: draft.install_id.clone(),
            display_name: self.display_name.clone(),
            managed_root: draft.created.managed_root.clone(),
            recipe_version: self.evidence.versions.recipe.clone(),
            recipe_sha256: draft.created.recipe_payload_sha256.clone(),
            engine_name: final_state.bg2_engine_name,
            managed_save_root: final_state.managed_save_root,
            launch_path: final_state.launch_path,
            receipt_sha256: crate::digest::sha256_bytes(&receipt_bytes),
            completed_at_millis: draft.completed_at_millis,
        };
        self.registry.publish(&record)?;
        Ok(published)
    }

    fn validate_frozen_evidence(
        &self,
        draft: &ReceiptDraft,
        require_complete: bool,
    ) -> Result<(), ManagedReceiptError> {
        if self.display_name.trim().is_empty() {
            return Err(ManagedReceiptError::Evidence(
                "managed-install display name is empty".to_owned(),
            ));
        }
        if draft.install_id != draft.created.install_id {
            return Err(ManagedReceiptError::Evidence(
                "draft install id differs from CampaignCreated".to_owned(),
            ));
        }
        if require_complete {
            if draft.attempt_id != draft.created.attempt_id {
                return Err(ManagedReceiptError::Evidence(
                    "successful draft attempt id differs from CampaignCreated".to_owned(),
                ));
            }
        } else if !is_terminal_attempt_id(&draft.attempt_id) {
            return Err(ManagedReceiptError::Evidence(
                "failed draft does not use a stable terminal attempt id".to_owned(),
            ));
        }
        if crate::digest::selection_digest(&draft.created.normalized_selection)
            .map_err(|error| ManagedReceiptError::Evidence(error.to_string()))?
            != draft.created.selection_sha256
        {
            return Err(ManagedReceiptError::Evidence(
                "normalized selection does not match its frozen digest".to_owned(),
            ));
        }
        if crate::digest::plan_digest(&draft.plan)
            .map_err(|error| ManagedReceiptError::Evidence(error.to_string()))?
            != draft.created.plan_sha256
        {
            return Err(ManagedReceiptError::Evidence(
                "exact plan does not match its frozen digest".to_owned(),
            ));
        }
        validate_source_evidence(&self.evidence.source_games, draft, require_complete)?;
        validate_identity_evidence(
            "artifact",
            &self.evidence.artifacts,
            &draft.created.artifact_identities,
            |artifact| {
                (
                    &artifact.id,
                    &artifact.version,
                    artifact.length,
                    &artifact.sha256,
                )
            },
            require_complete,
        )?;
        validate_identity_evidence(
            "WeiDU tool",
            &self.evidence.weidu_tools,
            &draft.created.tool_identities,
            |tool| (&tool.id, &tool.version, tool.length, &tool.sha256),
            require_complete,
        )?;
        if require_complete && self.evidence.runs.len() != draft.plan.runs.len() {
            return Err(ManagedReceiptError::Evidence(format!(
                "run evidence count {} differs from plan count {}",
                self.evidence.runs.len(),
                draft.plan.runs.len()
            )));
        }
        let mut seen_runs = BTreeSet::new();
        for evidence in &self.evidence.runs {
            let planned = draft
                .plan
                .runs
                .iter()
                .find(|planned| planned.run_id == evidence.run_id);
            if !seen_runs.insert(&evidence.run_id)
                || !planned.is_some_and(|planned| {
                    evidence.target == planned.target && evidence.components == planned.components
                })
            {
                return Err(ManagedReceiptError::Evidence(format!(
                    "run evidence for {:?} differs from the exact plan or is duplicated",
                    evidence.run_id
                )));
            }
        }
        Ok(())
    }
}

impl ReceiptWriter for ManagedReceiptWriter {
    fn write(&mut self, draft: &ReceiptDraft) -> Result<(), StepFailure> {
        self.publish_terminal(draft)
            .map(|_| ())
            .map_err(|error| StepFailure::new(error.to_string()))
    }
}

fn validate_source_evidence(
    sources: &[SourceGameReceipt],
    draft: &ReceiptDraft,
    require_complete: bool,
) -> Result<(), ManagedReceiptError> {
    let mut bg1 = 0_usize;
    let mut bg2 = 0_usize;
    for source in sources {
        let expected = match source.role {
            GameRole::BgeeSod => {
                bg1 += 1;
                &draft.created.source_games.bg1
            }
            GameRole::Bg2ee => {
                bg2 += 1;
                &draft.created.source_games.bg2
            }
        };
        if !source.fingerprint.eq_ignore_ascii_case(expected) {
            return Err(ManagedReceiptError::Evidence(
                "source fingerprints differ from CampaignCreated".to_owned(),
            ));
        }
    }
    if bg1 > 1 || bg2 > 1 || (require_complete && (bg1 != 1 || bg2 != 1)) {
        return Err(ManagedReceiptError::Evidence(
            "source evidence is duplicated or incomplete".to_owned(),
        ));
    }
    Ok(())
}

fn validate_identity_evidence<'a, T, F>(
    label: &str,
    evidence: &'a [T],
    frozen: &[FrozenIdentity],
    fields: F,
    require_complete: bool,
) -> Result<(), ManagedReceiptError>
where
    F: Fn(&'a T) -> (&'a String, &'a String, u64, &'a String),
{
    if require_complete && evidence.len() != frozen.len() {
        return Err(ManagedReceiptError::Evidence(format!(
            "{label} evidence count {} differs from frozen count {}",
            evidence.len(),
            frozen.len()
        )));
    }
    let mut seen = BTreeSet::new();
    for item in evidence {
        let (id, version, length, sha256) = fields(item);
        let matches = frozen.iter().filter(|identity| {
            id == &identity.id
                && version == &identity.version
                && length == identity.length
                && sha256.eq_ignore_ascii_case(&identity.sha256)
        });
        if !seen.insert(id) || matches.count() != 1 {
            return Err(ManagedReceiptError::Evidence(format!(
                "{label} {:?} does not exactly match its frozen identity",
                id
            )));
        }
    }
    Ok(())
}

fn is_terminal_attempt_id(value: &str) -> bool {
    value
        .strip_prefix("terminal-")
        .and_then(|suffix| suffix.split_once('-'))
        .is_some_and(|(sequence, digest)| {
            sequence.len() == 10
                && sequence.bytes().all(|byte| byte.is_ascii_digit())
                && digest.len() == 16
                && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
}

impl ReceiptStore {
    /// Open existing campaign state without creating a fallback managed root.
    pub fn open(managed_root: &Path, install_id: &str) -> Result<Self, ReceiptError> {
        validate_identifier(install_id, "install id", managed_root)?;
        let managed_root = canonical_direct_directory(managed_root)?;
        let state_root = managed_root.join(STATE_DIRECTORY);
        let attempts_root = state_root.join(ATTEMPTS_DIRECTORY);
        validate_direct_directory(&state_root)?;
        validate_direct_directory(&attempts_root)?;
        Ok(Self {
            managed_root,
            state_root,
            attempts_root,
            install_id: install_id.to_owned(),
        })
    }

    /// Publish an attempt receipt and, on success, the stable installation receipt.
    ///
    /// Existing byte-identical files are accepted to reconcile a crash between the two
    /// create-once publications. Different bytes are never replaced.
    pub fn publish(&self, receipt: &InstallReceipt) -> Result<PublishedReceipt, ReceiptError> {
        self.validate(receipt)?;
        let attempt_root = self.attempts_root.join(&receipt.attempt_id);
        ensure_attempt_directory(&self.attempts_root, &attempt_root)?;
        let attempt_receipt = attempt_root.join(ATTEMPT_RECEIPT_FILE);
        let install_receipt = receipt
            .outcome
            .is_success()
            .then(|| self.state_root.join(INSTALL_RECEIPT_FILE));
        let bytes = receipt_bytes(receipt, &attempt_receipt)?;

        reconcile_existing(&attempt_receipt, &bytes)?;
        if let Some(path) = &install_receipt {
            reconcile_existing(path, &bytes)?;
        }
        publish_if_missing(&attempt_receipt, &bytes)?;
        if let Some(path) = &install_receipt {
            publish_if_missing(path, &bytes)?;
        }

        Ok(PublishedReceipt {
            attempt_receipt,
            install_receipt,
        })
    }

    fn validate(&self, receipt: &InstallReceipt) -> Result<(), ReceiptError> {
        if receipt.schema_version != RECEIPT_SCHEMA_VERSION {
            return Err(ReceiptError::InvalidReceipt(format!(
                "unsupported schema {}; expected {RECEIPT_SCHEMA_VERSION}",
                receipt.schema_version
            )));
        }
        if receipt.install_id != self.install_id {
            return Err(ReceiptError::InvalidReceipt(format!(
                "install id {:?} does not match store {:?}",
                receipt.install_id, self.install_id
            )));
        }
        validate_identifier(&receipt.attempt_id, "attempt id", &self.managed_root)?;
        validate_identifier(
            &receipt.evidence_attempt_id,
            "evidence attempt id",
            &self.managed_root,
        )?;
        if receipt.completed_at_millis < receipt.started_at_millis {
            return Err(ReceiptError::InvalidReceipt(
                "completion timestamp precedes start timestamp".to_owned(),
            ));
        }
        if receipt.outcome.is_success() != receipt.final_state.is_some() {
            return Err(ReceiptError::InvalidReceipt(
                "successful receipts require final state and failed receipts must omit it"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

fn receipt_bytes(receipt: &InstallReceipt, path: &Path) -> Result<Vec<u8>, ReceiptError> {
    let mut bytes = serde_json::to_vec_pretty(receipt).map_err(|source| ReceiptError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn reconcile_existing(path: &Path, expected: &[u8]) -> Result<(), ReceiptError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(unsafe_path(
                    path,
                    "receipt destination is not a direct file",
                ));
            }
            let actual = fs::read(path).map_err(|source| ReceiptError::Io {
                path: path.to_path_buf(),
                source,
            })?;
            if actual != expected {
                return Err(ReceiptError::CreateOnceConflict {
                    path: path.to_path_buf(),
                });
            }
            Ok(())
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(ReceiptError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn publish_if_missing(path: &Path, bytes: &[u8]) -> Result<(), ReceiptError> {
    match fs::symlink_metadata(path) {
        Ok(_) => return reconcile_existing(path, bytes),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(ReceiptError::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    }
    let parent = path
        .parent()
        .ok_or_else(|| unsafe_path(path, "receipt destination has no parent"))?;
    validate_direct_directory(parent)?;
    let temporary = temporary_sibling(path)?;
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|source| ReceiptError::Io {
                path: temporary.clone(),
                source,
            })?;
        file.write_all(bytes).map_err(|source| ReceiptError::Io {
            path: temporary.clone(),
            source,
        })?;
        file.sync_all().map_err(|source| ReceiptError::Io {
            path: temporary.clone(),
            source,
        })?;
        drop(file);
        match fs::hard_link(&temporary, path) {
            Ok(()) => Ok(()),
            Err(link_source) => match fs::symlink_metadata(path) {
                Ok(_) => reconcile_existing(path, bytes),
                Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                    Err(ReceiptError::Io {
                        path: path.to_path_buf(),
                        source: link_source,
                    })
                }
                Err(source) => Err(ReceiptError::Io {
                    path: path.to_path_buf(),
                    source,
                }),
            },
        }
    })();
    let cleanup = fs::remove_file(&temporary);
    if let Err(error) = result {
        let _ = cleanup;
        return Err(error);
    }
    cleanup.map_err(|source| ReceiptError::Io {
        path: temporary,
        source,
    })?;
    sync_directory(parent)
}

fn ensure_attempt_directory(attempts_root: &Path, attempt_root: &Path) -> Result<(), ReceiptError> {
    validate_direct_directory(attempts_root)?;
    match fs::symlink_metadata(attempt_root) {
        Ok(_) => validate_direct_directory(attempt_root),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            match fs::create_dir(attempt_root) {
                Ok(()) => sync_directory(attempts_root),
                Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                    validate_direct_directory(attempt_root)
                }
                Err(source) => Err(ReceiptError::Io {
                    path: attempt_root.to_path_buf(),
                    source,
                }),
            }
        }
        Err(source) => Err(ReceiptError::Io {
            path: attempt_root.to_path_buf(),
            source,
        }),
    }
}

fn temporary_sibling(path: &Path) -> Result<PathBuf, ReceiptError> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| unsafe_path(path, "receipt filename is not valid Unicode"))?;
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(path.with_file_name(format!(".{name}.{}.{}.tmp", std::process::id(), sequence)))
}

fn validate_identifier(value: &str, label: &str, path: &Path) -> Result<(), ReceiptError> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(unsafe_path(path, &format!("{label} is not path-safe")));
    }
    Ok(())
}

fn canonical_direct_directory(path: &Path) -> Result<PathBuf, ReceiptError> {
    validate_direct_directory(path)?;
    fs::canonicalize(path).map_err(|source| ReceiptError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_direct_directory(path: &Path) -> Result<(), ReceiptError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| ReceiptError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(unsafe_path(path, "expected a direct non-symlink directory"));
    }
    Ok(())
}

fn unsafe_path(path: &Path, message: &str) -> ReceiptError {
    ReceiptError::UnsafePath {
        path: path.to_path_buf(),
        message: message.to_owned(),
    }
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<(), ReceiptError> {
    Ok(())
}

#[cfg(not(windows))]
fn sync_directory(path: &Path) -> Result<(), ReceiptError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| ReceiptError::Io {
            path: path.to_path_buf(),
            source,
        })
}
