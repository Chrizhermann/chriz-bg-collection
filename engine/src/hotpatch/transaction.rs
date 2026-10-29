//! Hotpatch transactions, one folder each below [`super::STATE`]:
//!
//! - `prepared.json`: the durable barrier. Nothing in the game changes before it exists. It
//!   holds the game folder as it was, the package and copies of the root files that may change.
//! - `applied.json`: written only after WeiDU succeeded, the mod list grew by exactly the
//!   hotpatch's components, every change stayed inside the hotpatch's area, and an earlier copy
//!   of every changed file was kept.
//! - `restoring.json` / `restored.json`: Undo or recovery, which puts the folder back to
//!   `prepared.json`'s record byte for byte and then checks it.
//!
//! Files the game writes while running ([`super::catalog::RuntimeFiles`]) are never compared,
//! written or removed. Transactions chain by digest; only the latest applied one can be undone.
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::catalog::{Catalog, Hotpatch};
use super::{gate, Context, HotpatchPreview};
use crate::acquire::{
    extract_archive, extract_served_archive, AcceptedContent, ArchiveFormat, ArchiveLimits,
    ArchiveMode, ArchiveRequirements, ArtifactCache, DownloadRequest, DownloadRoute, StopSignal,
};
use crate::digest::sha256_bytes;
use crate::events::ConsoleSink;
use crate::game_files::{self, Difference, GameFiles};
use crate::held_folder::Existing;
use crate::manifest::{Artifact, ToolSpec};
use crate::patches::files;
use crate::preflight::{PreflightHost, SystemPreflight};
use crate::receipt::{InstallReceipt, LogComponentReceipt};
use crate::weidu::invocation::{Invocation, VerifiedWeidu};
use crate::weidu::log::{normalize_tp2_key, parse_active_entries};
use crate::weidu::runner::{self, RunOutcome, RunnerRequest};
use crate::weidu::verify::{reconcile, ExpectedRun, Reconciliation};

const SCHEMA: u32 = 1;
const RECORD_LIMIT: u64 = 512 * 1024 * 1024;
const PAYLOAD_LIMIT: u64 = 64 * 1024 * 1024;
const TOOL_LIMIT: u64 = 16 * 1024 * 1024;
const EVIDENCE_LIMIT: u64 = 64 * 1024 * 1024;

/// The pinned package and how WeiDU installs it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Package {
    pub artifact_id: String,
    pub version: String,
    pub sha256: String,
    pub tp2: String,
    pub language: u32,
    pub components: Vec<u32>,
    /// Lowercase folder the package publishes into the game folder.
    pub mod_folder: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Prepared {
    pub schema: u32,
    pub hotpatch_id: String,
    pub install_id: String,
    pub managed_root: PathBuf,
    pub base_receipt_sha256: String,
    /// Terminal digest of the transaction before this one.
    pub previous: Option<String>,
    pub catalog_digest: String,
    pub package: Package,
    /// `weidu.conf`'s language folder, as WeiDU is told to use it.
    pub language: String,
    pub before: GameFiles,
    /// Copies in `affected/` of files that may change, by lowercase name, with their SHA-256.
    pub saved: BTreeMap<String, String>,
    /// Package files written into the game, by their name there, with their SHA-256; copies in
    /// `payload/`.
    pub payload: BTreeMap<String, String>,
    pub tool_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Applied {
    pub prepared_sha256: String,
    pub after: GameFiles,
    pub suffix: Vec<LogComponentReceipt>,
    pub log_sha256: String,
    /// SHA-256 of earlier file versions kept as `preimages/<sha256>`.
    pub preimages: BTreeSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Restoring {
    prepared_sha256: String,
    undo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum State {
    Prepared,
    Applied,
    Restoring,
    Restored,
}

pub(super) struct Entry {
    pub dir: PathBuf,
    pub state: State,
    pub hotpatch_id: String,
    /// Digest chaining the next transaction to this one's latest record.
    pub terminal: String,
}

/// Every published transaction of one installation, oldest first.
pub(super) struct History {
    pub entries: Vec<Entry>,
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&files::read(path, RECORD_LIMIT)?).map_err(files::err)
}

fn later_records(dir: &Path) -> bool {
    [
        "applied.json",
        "applied.pending",
        "restoring.json",
        "restoring.pending",
        "restored.json",
        "restored.pending",
    ]
    .iter()
    .any(|name| dir.join(name).exists())
}

fn load_prepared(root: &Path, install_id: &str, dir: &Path) -> Result<Prepared, String> {
    files::direct(dir)?;
    if dir.parent() != Some(root.join(super::STATE).as_path()) {
        return Err("Hotpatch record is outside this installation.".into());
    }
    let p: Prepared = read_json(&dir.join("prepared.json"))?;
    let receipt = root.join(".chriz/install-receipt.json");
    if p.schema != SCHEMA
        || p.managed_root != root
        || p.install_id != install_id
        || p.base_receipt_sha256 != files::hash(&receipt)?
    {
        return Err("Hotpatch history does not match this installation.".into());
    }
    p.before.validate()?;
    for name in p.saved.keys().chain(p.payload.keys()) {
        files::safe_relative(name)?;
    }
    Ok(p)
}

impl History {
    pub fn load(root: &Path, install_id: &str) -> Result<History, String> {
        let parent = root.join(".chriz/hotpatches");
        if !parent.exists() {
            return Ok(History { entries: vec![] });
        }
        files::direct(&parent)?;
        for entry in fs::read_dir(&parent).map_err(files::err)? {
            let entry = entry.map_err(files::err)?;
            files::direct(&entry.path())?;
            if entry.file_name() != "v1" || !entry.path().is_dir() {
                return Err(
                    "This installation has a hotpatch history this app cannot read.".into(),
                );
            }
        }
        let state = root.join(super::STATE);
        if !state.exists() {
            return Ok(History { entries: vec![] });
        }
        let mut dirs = fs::read_dir(&state)
            .map_err(files::err)?
            .map(|entry| entry.map(|entry| entry.path()).map_err(files::err))
            .collect::<Result<Vec<_>, _>>()?;
        dirs.sort();
        let mut entries: Vec<Entry> = Vec::new();
        for dir in dirs {
            files::direct(&dir)?;
            let id = dir
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            if !dir.is_dir() || id.len() != 20 || !id.bytes().all(|b| b.is_ascii_digit()) {
                return Err("Unrecognized hotpatch record.".into());
            }
            if !dir.join("prepared.json").exists() {
                // No game write happens before the preparation record.
                if later_records(&dir) {
                    return Err("A hotpatch record is missing its preparation.".into());
                }
                continue;
            }
            if entries
                .last()
                .is_some_and(|last| matches!(last.state, State::Prepared | State::Restoring))
            {
                return Err("A hotpatch was recorded after an unfinished one.".into());
            }
            let p = load_prepared(root, install_id, &dir)?;
            if p.previous.as_deref() != entries.last().map(|last| last.terminal.as_str()) {
                return Err("The hotpatch history chain changed.".into());
            }
            let prepared_sha256 = files::hash(&dir.join("prepared.json"))?;
            let (state, record) = if dir.join("restored.json").exists() {
                let digest: String = read_json(&dir.join("restored.json"))?;
                if digest != prepared_sha256 {
                    return Err("A hotpatch restoration record changed.".into());
                }
                (State::Restored, "restored.json")
            } else if dir.join("restoring.json").exists() {
                let r: Restoring = read_json(&dir.join("restoring.json"))?;
                if r.prepared_sha256 != prepared_sha256 {
                    return Err("A hotpatch restoration record changed.".into());
                }
                (State::Restoring, "restoring.json")
            } else if dir.join("applied.json").exists() {
                let a: Applied = read_json(&dir.join("applied.json"))?;
                if a.prepared_sha256 != prepared_sha256 {
                    return Err("A hotpatch record changed.".into());
                }
                a.after.validate()?;
                (State::Applied, "applied.json")
            } else {
                (State::Prepared, "prepared.json")
            };
            let terminal = sha256_bytes(
                &serde_json::to_vec(&(&id, files::hash(&dir.join(record))?)).map_err(files::err)?,
            );
            entries.push(Entry {
                dir,
                state,
                hotpatch_id: p.hotpatch_id,
                terminal,
            });
        }
        Ok(History { entries })
    }

    pub fn head(&self) -> Option<String> {
        self.entries.last().map(|entry| entry.terminal.clone())
    }

    pub fn unfinished(&self) -> Option<&Entry> {
        self.entries
            .last()
            .filter(|entry| matches!(entry.state, State::Prepared | State::Restoring))
    }

    /// Hotpatches applied and not undone, oldest first.
    pub fn applied_ids(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|entry| entry.state == State::Applied)
            .map(|entry| entry.hotpatch_id.clone())
            .collect()
    }

    /// The latest applied hotpatch, when everything after it was undone.
    pub fn undo_target(&self) -> Option<&Entry> {
        let position = self
            .entries
            .iter()
            .rposition(|entry| entry.state == State::Applied)?;
        self.entries[position + 1..]
            .iter()
            .all(|entry| entry.state == State::Restored)
            .then(|| &self.entries[position])
    }

    /// The game folder as the installer and the hotpatches since left it.
    pub fn expected_state(&self, root: &Path) -> Result<GameFiles, String> {
        match self.entries.last() {
            None => game_files::read_completed(root),
            Some(entry) => match entry.state {
                State::Applied => Ok(read_json::<Applied>(&entry.dir.join("applied.json"))?.after),
                State::Restored => {
                    Ok(read_json::<Prepared>(&entry.dir.join("prepared.json"))?.before)
                }
                State::Prepared | State::Restoring => {
                    Err("A hotpatch did not finish. Restore it first.".into())
                }
            },
        }
    }
}

/// Launching waits for an unfinished hotpatch to be restored.
pub fn launch_guard(root: &Path) -> Result<(), String> {
    let receipt = root.join(".chriz/install-receipt.json");
    if !root.join(".chriz/hotpatches").exists() {
        return Ok(());
    }
    let install_id = install_id_of(&receipt)?;
    if History::load(root, &install_id)?.unfinished().is_some() {
        return Err(
            "A hotpatch did not finish. Open Updates and restore the game files before playing."
                .into(),
        );
    }
    Ok(())
}

fn install_id_of(receipt: &Path) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_slice(&files::read(receipt, 64 * 1024 * 1024)?).map_err(files::err)?;
    value
        .get("install_id")
        .and_then(|id| id.as_str())
        .map(str::to_owned)
        .ok_or_else(|| "The installation receipt has no installation ID.".into())
}

/// The mod-list rows applied hotpatches added after the completed installation's, in order.
/// The current `WeiDU.log` must be exactly the one the latest hotpatch record left.
pub fn verified_log_suffix(root: &Path) -> Result<Vec<LogComponentReceipt>, String> {
    if !root.join(".chriz/hotpatches").exists() {
        return Ok(vec![]);
    }
    let install_id = install_id_of(&root.join(".chriz/install-receipt.json"))?;
    let history = History::load(root, &install_id)?;
    if history.unfinished().is_some() {
        return Err("A hotpatch did not finish. Restore it first.".into());
    }
    let mut suffix = Vec::new();
    for entry in &history.entries {
        if entry.state == State::Applied {
            suffix.extend(read_json::<Applied>(&entry.dir.join("applied.json"))?.suffix);
        }
    }
    if !history.entries.is_empty() {
        let expected = history.expected_state(root)?;
        let log = root.join("game/WeiDU.log");
        files::regular(&log)?;
        let actual = files::hash(&log)?;
        if expected
            .files
            .get("weidu.log")
            .map(|identity| &identity.sha256)
            != Some(&actual)
        {
            // Also after an Undo, so not "after the last hotpatch".
            return Err(
                "The mod list changed after the installer and its hotpatches left it.".into(),
            );
        }
    }
    Ok(suffix)
}

/// Whether `name` lies in what a hotpatch for `package` may change.
fn writable(name: &str, package: &Package, language: &str) -> bool {
    let below = |folder: &str| name == folder || name.starts_with(&format!("{folder}/"));
    let language = language.to_ascii_lowercase();
    matches!(
        name,
        "weidu.log"
            | "weidu.conf"
            | "weidu_external"
            | "weidu_external/backup"
            | "weidu_external/data"
    ) || below("override")
        || below(&package.mod_folder)
        || below(&format!("weidu_external/backup/{}", package.mod_folder))
        || below(&format!("weidu_external/data/{}", package.mod_folder))
        || name == format!("lang/{language}/dialog.tlk")
        || name == format!("lang/{language}/dialogf.tlk")
}

/// Root files copied before WeiDU runs: those it writes, and the key a misbehaving package
/// could change, so even that can be put back.
fn root_files(language: &str) -> Vec<String> {
    let language = language.to_ascii_lowercase();
    vec![
        "weidu.log".into(),
        "weidu.conf".into(),
        "chitin.key".into(),
        format!("lang/{language}/dialog.tlk"),
        format!("lang/{language}/dialogf.tlk"),
    ]
}

pub(super) struct Sources {
    /// Package files by their name in the game folder, original case.
    pub payload: BTreeMap<String, PathBuf>,
    pub tool: PathBuf,
    pub tool_sha256: String,
}

fn requirements(artifact: &Artifact) -> ArchiveRequirements {
    ArchiveRequirements {
        artifact_sha256: artifact.source.sha256.clone(),
        format: ArchiveFormat::Zip,
        expected_roots: artifact.archive.publish_roots.clone(),
        expected_tp2_paths: artifact.archive.tp2_paths.clone(),
        limits: ArchiveLimits {
            max_depth: artifact.archive.limits.max_depth,
            max_entries: artifact.archive.limits.max_entries,
            max_entry_uncompressed_bytes: artifact.archive.limits.max_entry_uncompressed_bytes,
            max_total_uncompressed_bytes: artifact.archive.limits.max_total_uncompressed_bytes,
            max_compression_ratio: artifact.archive.limits.max_compression_ratio,
        },
        mode: ArchiveMode::Public,
    }
}

fn unpack(ctx: &Context<'_>, artifact: &Artifact) -> Result<PathBuf, String> {
    let cache = ArtifactCache::open(ctx.cache).map_err(files::err)?;
    let acquired = cache
        .acquire(
            &DownloadRequest {
                request_id: format!("hotpatch-{}", artifact.id),
                url: artifact.source.url.clone(),
                expected_length: artifact
                    .source
                    .expected_length
                    .ok_or("The package has no pinned length.")?,
                expected_sha256: artifact.source.sha256.clone(),
                redirect_hosts: artifact.source.redirect_hosts.clone(),
                max_attempts: crate::acquire::DEFAULT_DOWNLOAD_ATTEMPTS,
                route: crate::acquire::DownloadRoute::Generic,
            },
            &ConsoleSink,
        )
        .map_err(files::err)?;
    Ok(extract_archive(
        &acquired.archive_path,
        &ctx.cache.join("hotpatch-extracted"),
        &requirements(artifact),
    )
    .map_err(files::err)?
    .root)
}

/// The WeiDU an installation was made with.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct InstalledTool {
    /// The archive the installer accepted instead of the pinned one, when the tool's official
    /// link served a different file; `None` for the reviewed archive.
    pub served: Option<AcceptedContent>,
    /// SHA-256 of the WeiDU program the installation ran.
    pub executable_sha256: String,
}

/// Which WeiDU `receipt`'s installation used for the tool `artifact`: the reviewed one, or the
/// changed download its installer accepted and the program it verified and recorded from it.
pub(super) fn installed_tool(
    receipt: &InstallReceipt,
    artifact: &Artifact,
    spec: &ToolSpec,
) -> Result<InstalledTool, String> {
    let Some(acquired) = receipt
        .artifacts
        .iter()
        .find(|acquired| acquired.id == artifact.id && acquired.served_difference.is_some())
    else {
        return Ok(InstalledTool {
            served: None,
            executable_sha256: spec.sha256.to_ascii_lowercase(),
        });
    };
    let program = receipt
        .weidu_tools
        .iter()
        .find(|tool| tool.id == artifact.id)
        .ok_or("The installation record names no WeiDU program for its changed WeiDU download.")?;
    Ok(InstalledTool {
        served: Some(AcceptedContent {
            length: acquired.length,
            sha256: acquired.sha256.to_ascii_lowercase(),
        }),
        executable_sha256: program.sha256.to_ascii_lowercase(),
    })
}

/// [`unpack`] for a changed download an installation was made with: exactly that file, from the
/// cache or from the official link while it still serves it, unpacked by the rules for served
/// archives as the installer did.
pub(super) fn unpack_served(
    ctx: &Context<'_>,
    artifact: &Artifact,
    accepted: &AcceptedContent,
) -> Result<PathBuf, String> {
    let cache = ArtifactCache::open(ctx.cache).map_err(files::err)?;
    let served = cache
        .acquire_as_served(
            &DownloadRequest {
                request_id: format!("hotpatch-{}", artifact.id),
                url: artifact.source.url.clone(),
                expected_length: artifact
                    .source
                    .expected_length
                    .ok_or("The package has no pinned length.")?,
                expected_sha256: artifact.source.sha256.clone(),
                redirect_hosts: artifact.source.redirect_hosts.clone(),
                max_attempts: crate::acquire::DEFAULT_DOWNLOAD_ATTEMPTS,
                route: DownloadRoute::for_source(&artifact.source).map_err(files::err)?,
            },
            Some(accepted),
            &ConsoleSink,
            &StopSignal::never(),
        )
        .map_err(files::err)?;
    let digest = &served.artifact.metadata.sha256;
    if !digest.eq_ignore_ascii_case(&accepted.sha256) {
        return Err(format!(
            "This game was installed with a newer or different download of {} (SHA-256 {}). That file is no longer in the download folder, and its official link now serves another one (SHA-256 {digest}). A hotpatch runs only with the WeiDU the game was installed with.",
            artifact.name, accepted.sha256
        ));
    }
    let mut requirements = requirements(artifact);
    requirements.artifact_sha256 = accepted.sha256.clone();
    requirements.limits = requirements.limits.for_served_archive();
    Ok(extract_served_archive(
        &served.artifact.archive_path,
        &ctx.cache.join("hotpatch-extracted"),
        &requirements,
        &StopSignal::never(),
    )
    .map_err(files::err)?
    .0
    .root)
}

/// Download (or reuse) and check the package and the WeiDU the installation was made with.
fn sources(
    ctx: &Context<'_>,
    hotpatch: &Hotpatch,
    base: &crate::Manifest,
    receipt: &InstallReceipt,
) -> Result<Sources, String> {
    let root = unpack(ctx, &hotpatch.artifact)?;
    let published = &hotpatch.artifact.archive.publish_roots[0];
    let mut payload = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root.join(published)).follow_links(false) {
        let entry = entry.map_err(files::err)?;
        if entry.file_type().is_dir() {
            continue;
        }
        if !entry.file_type().is_file() {
            return Err("The package contains a link.".into());
        }
        let name = entry
            .path()
            .strip_prefix(&root)
            .map_err(files::err)?
            .to_string_lossy()
            .replace('\\', "/");
        files::safe_relative(&name)?;
        // WeiDU keeps uninstall copies in a mod's backup folder; a package must not bring one.
        if name
            .to_ascii_lowercase()
            .starts_with(&format!("{}/backup/", hotpatch.mod_folder()))
        {
            return Err("The package contains a backup folder.".into());
        }
        payload.insert(name, entry.path().to_path_buf());
    }
    if !payload
        .keys()
        .any(|name| name.eq_ignore_ascii_case(&hotpatch.installer.tp2))
    {
        return Err("The package is missing its installer.".into());
    }
    let tool_artifact = base
        .artifacts
        .get(&hotpatch.installer.weidu_artifact_id)
        .ok_or("The installation's recipe has no WeiDU for this hotpatch.")?;
    let tool_spec = tool_artifact
        .tool
        .as_ref()
        .ok_or("The installation's WeiDU artifact is not a tool.")?;
    let installed = installed_tool(receipt, tool_artifact, tool_spec)?;
    let root = match &installed.served {
        None => unpack(ctx, tool_artifact)?,
        Some(accepted) => unpack_served(ctx, tool_artifact, accepted)?,
    };
    let tool = root.join(&tool_spec.executable);
    VerifiedWeidu::verify(&tool, &installed.executable_sha256).map_err(files::err)?;
    Ok(Sources {
        payload,
        tool,
        tool_sha256: installed.executable_sha256,
    })
}

pub fn apply(
    ctx: &Context<'_>,
    catalog: &Catalog,
    token: &str,
    full_backup: bool,
    save_backup: bool,
) -> Result<HotpatchPreview, String> {
    apply_with_progress(ctx, catalog, token, full_backup, save_backup, &|_| {})
}

pub fn apply_with_progress(
    ctx: &Context<'_>,
    catalog: &Catalog,
    token: &str,
    full_backup: bool,
    save_backup: bool,
    progress: crate::operation_progress::ProgressSink<'_>,
) -> Result<HotpatchPreview, String> {
    crate::operation_progress::stage(progress, "checking");
    let root = &ctx.record.managed_root;
    let _lock = ctx.locks.try_acquire(root).map_err(files::err)?;
    crate::patches::launch_guard(root)?;
    crate::mod_upgrades::launch_guard(root)?;
    super::validate_context(ctx)?;
    let history = History::load(root, &ctx.record.install_id)?;
    if history.unfinished().is_some() {
        return Err("A hotpatch did not finish. Restore it before applying another.".into());
    }
    let assessment = gate::assess(ctx, catalog, &history);
    if !assessment.refusals.is_empty() {
        return Err(assessment.refusals.join(" "));
    }
    let hotpatch = assessment
        .next
        .ok_or("No hotpatch is waiting for this installation.")?;
    let current = assessment
        .current
        .ok_or("The game folder could not be read.")?;
    let baseline = assessment
        .baseline
        .ok_or("The installation could not be verified.")?;
    if super::review_token(ctx, catalog, &hotpatch, &current, history.head().as_deref())? != token {
        return Err(
            "The installation changed since you checked it. Check again before applying.".into(),
        );
    }
    let language = super::language_folder(&ctx.game())?;
    let installed = parse_active_entries(&String::from_utf8_lossy(&files::read(
        &ctx.game().join("WeiDU.log"),
        16 * 1024 * 1024,
    )?))
    .map_err(files::err)?;
    let tp2_key = normalize_tp2_key(&hotpatch.installer.tp2);
    let present = installed
        .iter()
        .filter(|entry| entry.tp2_key == tp2_key)
        .map(|entry| entry.component)
        .collect::<BTreeSet<_>>();
    if hotpatch
        .components
        .iter()
        .any(|component| present.contains(component))
    {
        return Err("A component of this hotpatch is already installed.".into());
    }
    let mod_folder = hotpatch.mod_folder();
    if present.is_empty()
        && (current.folders.contains(&mod_folder) || current.files.contains_key(&mod_folder))
    {
        return Err("The game folder already has a copy of this hotpatch's mod folder.".into());
    }
    crate::patches::close_check_in(&ctx.game(), &language, &guarded(&current, &language), None)?;

    let id = format!(
        "{:020}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(files::err)?
            .as_nanos()
    );
    let dir = ctx.state().join(&id);
    let game_bytes = current
        .files
        .values()
        .map(|identity| identity.length)
        .sum::<u64>();
    let profile_bytes = if save_backup {
        files::total(&files::inventory(&ctx.record.managed_save_root, true)?)
    } else {
        0
    };
    let required = (if full_backup { game_bytes } else { 0 })
        .saturating_add(profile_bytes)
        .saturating_add(512 * 1024 * 1024);
    if SystemPreflight
        .available_space(ctx.app_data)
        .map_err(files::err)?
        < required
        || SystemPreflight.available_space(root).map_err(files::err)? < 512 * 1024 * 1024
    {
        return Err("Not enough free space for the selected backups and the hotpatch.".into());
    }
    files::create(&dir.join("identity"), ctx.record.install_id.as_bytes())?;
    crate::operation_progress::stage(progress, "downloading");
    // No game writes happen here, even if a download fails.
    let sources = sources(ctx, &hotpatch, &baseline.manifest, &baseline.receipt)?;
    let backup = ctx.backups().join(&id);
    files::create(&backup.join("identity"), ctx.record.install_id.as_bytes())?;
    if full_backup {
        files::copy_tree_with_progress(
            &ctx.game(),
            &backup.join("game"),
            false,
            "backup-game",
            progress,
        )?;
    }
    if save_backup {
        files::copy_tree_with_progress(
            &ctx.record.managed_save_root,
            &backup.join("profile"),
            true,
            "backup-saves",
            progress,
        )?;
    }
    crate::operation_progress::stage(progress, "backup-files");
    let prepared = prepare_record(
        ctx,
        catalog,
        &dir,
        &hotpatch,
        language,
        &current,
        &sources,
        history.head(),
    )?;
    crate::operation_progress::stage(progress, "applying");
    if let Err(error) = execute(ctx, catalog, &dir, &prepared) {
        let _ = files::json_new(&dir.join("failure.json"), &error);
        crate::operation_progress::stage(progress, "restoring");
        return match restore_inner(ctx, catalog, &dir, false) {
            Ok(()) => Err(format!(
                "The hotpatch could not be applied; your game files were put back. {error}"
            )),
            Err(recovery) => Err(format!(
                "The hotpatch could not be applied and needs restoring before you play. {error} {recovery}"
            )),
        };
    }
    crate::operation_progress::stage(progress, "verifying");
    super::inspect(ctx, catalog)
}

/// Root files that WeiDU writes, with their digests now.
fn guarded(current: &GameFiles, language: &str) -> BTreeMap<String, String> {
    root_files(language)
        .into_iter()
        .filter_map(|name| {
            let identity = current.files.get(&name)?;
            Some((name, identity.sha256.clone()))
        })
        .collect()
}

/// Write the preparation into `dir`: the package, WeiDU, a copy of every existing file the
/// package or WeiDU may replace and, last, `prepared.json`. Nothing in the game changes here.
#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_record(
    ctx: &Context<'_>,
    catalog: &Catalog,
    dir: &Path,
    hotpatch: &Hotpatch,
    language: String,
    current: &GameFiles,
    sources: &Sources,
    previous: Option<String>,
) -> Result<Prepared, String> {
    let guarded = guarded(current, &language);
    let mut saved_names = guarded.keys().cloned().collect::<BTreeSet<_>>();
    let mut payload = BTreeMap::new();
    for (name, path) in &sources.payload {
        let bytes = files::read(path, PAYLOAD_LIMIT)?;
        files::create(&dir.join("payload").join(name), &bytes)?;
        let lower = name.to_ascii_lowercase();
        if current.files.contains_key(&lower) {
            saved_names.insert(lower);
        }
        payload.insert(name.clone(), sha256_bytes(&bytes));
    }
    let tool = dir.join("weidu.exe");
    files::create(&tool, &files::read(&sources.tool, TOOL_LIMIT)?)?;
    if files::hash(&tool)? != sources.tool_sha256 {
        return Err("The WeiDU copy changed.".into());
    }
    let mut saved = BTreeMap::new();
    for name in &saved_names {
        let bytes = files::read(&ctx.game().join(name), 512 * 1024 * 1024)?;
        let hash = sha256_bytes(&bytes);
        if current.files.get(name).map(|identity| &identity.sha256) != Some(&hash) {
            return Err(format!(
                "A file changed while it was being backed up: {name}"
            ));
        }
        files::create(&dir.join("affected").join(name), &bytes)?;
        saved.insert(name.clone(), hash);
    }
    crate::patches::close_check_in(&ctx.game(), &language, &guarded, None)?;
    let before = game_files::capture(&ctx.game())?;
    if before != *current {
        return Err("The game changed while the hotpatch was being prepared.".into());
    }
    let prepared = Prepared {
        schema: SCHEMA,
        hotpatch_id: hotpatch.id.clone(),
        install_id: ctx.record.install_id.clone(),
        managed_root: ctx.record.managed_root.clone(),
        base_receipt_sha256: ctx.record.receipt_sha256.clone(),
        previous,
        catalog_digest: catalog.digest.clone(),
        package: Package {
            artifact_id: hotpatch.artifact.id.clone(),
            version: hotpatch.artifact.version.clone(),
            sha256: hotpatch.artifact.source.sha256.to_ascii_lowercase(),
            tp2: hotpatch.installer.tp2.replace('\\', "/"),
            language: hotpatch.installer.language,
            components: hotpatch.components.clone(),
            mod_folder: hotpatch.mod_folder(),
        },
        language,
        before,
        saved,
        payload,
        tool_sha256: sources.tool_sha256.clone(),
    };
    files::json_new(&dir.join("prepared.json"), &prepared)?;
    Ok(prepared)
}

fn execute(ctx: &Context<'_>, catalog: &Catalog, dir: &Path, p: &Prepared) -> Result<(), String> {
    place_payload(ctx, dir, p)?;
    run_weidu(ctx, dir, p)?;
    record_applied(ctx, catalog, dir, p)
}

/// Copy the package from the record into the game folder.
pub(super) fn place_payload(ctx: &Context<'_>, dir: &Path, p: &Prepared) -> Result<(), String> {
    // The folders are no longer held once WeiDU runs.
    let held = files::hold(&ctx.record.managed_root, dir)?;
    for (name, hash) in &p.payload {
        let bytes = files::read_held(&held.record, &format!("payload/{name}"), PAYLOAD_LIMIT)?;
        if sha256_bytes(&bytes) != *hash {
            return Err("The hotpatch package changed.".into());
        }
        let (folders, last) = files::split_relative(name)?;
        let folder = held
            .game
            .open_or_create_below(folders)
            .map_err(files::held)?;
        if p.saved.contains_key(&name.to_ascii_lowercase()) {
            files::write_held(&folder, last, &bytes)?;
        } else {
            folder
                .write_file(last, &bytes, Existing::Refuse)
                .map_err(files::held)?;
        }
    }
    Ok(())
}

fn run_weidu(ctx: &Context<'_>, dir: &Path, p: &Prepared) -> Result<(), String> {
    let tool = dir.join("weidu.exe");
    VerifiedWeidu::verify(&tool, &p.tool_sha256).map_err(files::err)?;
    // Nobody may replace the executable while it runs.
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1);
    }
    let _tool_guard = options.open(&tool).map_err(files::err)?;
    if files::hash(&tool)? != p.tool_sha256 {
        return Err("The WeiDU copy changed.".into());
    }
    let debug = dir.join("attempt.debug");
    let mut args = vec![
        std::ffi::OsString::from(&p.package.tp2),
        "--language".into(),
        p.package.language.to_string().into(),
        "--use-lang".into(),
        p.language.clone().into(),
        "--force-install-list".into(),
    ];
    args.extend(
        p.package
            .components
            .iter()
            .map(|component| component.to_string().into()),
    );
    args.extend(
        [
            "--no-exit-pause",
            "--skip-at-view",
            "--safe-exit",
            "--noautoupdate",
            "--log",
        ]
        .map(std::ffi::OsString::from),
    );
    args.push(debug.clone().into());
    files::json_new(&dir.join("invocation.json"), &(&tool, &args))?;
    let invocation = Invocation {
        identity_digest: sha256_bytes(
            &serde_json::to_vec(&(&tool, ctx.game(), &args)).map_err(files::err)?,
        ),
        program: tool.clone(),
        cwd: ctx.game(),
        args,
        prompts: vec![],
        debug_path: debug.clone(),
    };
    let (_sender, receiver) = crossbeam_channel::unbounded();
    let outcome = runner::run(
        RunnerRequest {
            invocation,
            step_id: format!("hotpatch:{}", p.hotpatch_id),
            output_log: dir.join("output.log"),
            silence_threshold: Duration::from_secs(300),
        },
        receiver,
        ConsoleSink,
    );
    let RunOutcome::Exited { code } = outcome else {
        return Err(format!("WeiDU did not finish: {outcome:?}"));
    };
    let evidence = |path: &Path| {
        if path.exists() {
            files::read(path, EVIDENCE_LIMIT)
        } else {
            Ok(Vec::new())
        }
    };
    finished_as_planned(
        &files::read(&dir.join("affected/weidu.log"), 16 * 1024 * 1024)?,
        &files::read(&ctx.game().join("WeiDU.log"), 16 * 1024 * 1024)?,
        &evidence(&debug)?,
        &evidence(&dir.join("output.log"))?,
        &p.package,
        code,
    )
}

/// Judges WeiDU's run by its evidence, as a fresh installation does: exactly the hotpatch's
/// components after the others, a result line for each, and an exit code that agrees with them
/// (3 when a component installed with warnings).
pub(super) fn finished_as_planned(
    before: &[u8],
    after: &[u8],
    debug: &[u8],
    stdout: &[u8],
    package: &Package,
    exit_code: i32,
) -> Result<(), String> {
    let terminal = crate::cli::select_terminal_evidence(debug, stdout);
    match reconcile(
        &String::from_utf8_lossy(before),
        &String::from_utf8_lossy(after),
        &String::from_utf8_lossy(terminal),
        &ExpectedRun {
            tp2: package.tp2.clone(),
            language: package.language,
            components: package.components.clone(),
            exit_code,
        },
    ) {
        Reconciliation::ProvenDone => Ok(()),
        other => Err(format!(
            "WeiDU did not install exactly the hotpatch's components ({other:?}, exit code {exit_code})."
        )),
    }
}

/// Check what WeiDU did and publish the applied record: exactly the hotpatch's rows after the
/// earlier ones, nothing outside the hotpatch's area, an earlier copy of every changed file.
pub(super) fn record_applied(
    ctx: &Context<'_>,
    catalog: &Catalog,
    dir: &Path,
    p: &Prepared,
) -> Result<(), String> {
    let old = parse_active_entries(&String::from_utf8_lossy(&files::read(
        &dir.join("affected/weidu.log"),
        16 * 1024 * 1024,
    )?))
    .map_err(files::err)?;
    let new = parse_active_entries(&String::from_utf8_lossy(&files::read(
        &ctx.game().join("WeiDU.log"),
        16 * 1024 * 1024,
    )?))
    .map_err(files::err)?;
    let key = normalize_tp2_key(&p.package.tp2);
    let suffix = p
        .package
        .components
        .iter()
        .map(|component| LogComponentReceipt {
            tp2: key.clone(),
            language: p.package.language,
            component: *component,
        })
        .collect::<Vec<_>>();
    let row = |entry: &crate::weidu::log::LogEntry| {
        (entry.tp2_key.clone(), entry.language, entry.component)
    };
    let expected = old
        .iter()
        .map(row)
        .chain(
            suffix
                .iter()
                .map(|row| (row.tp2.clone(), row.language, row.component)),
        )
        .collect::<Vec<_>>();
    if new.iter().map(row).collect::<Vec<_>>() != expected {
        return Err(
            "WeiDU did not install exactly the hotpatch's components after the others.".into(),
        );
    }
    let after = game_files::capture(&ctx.game())?;
    let changed = p
        .before
        .differences(&after, |name| catalog.runtime_files.matches(name));
    let outside = changed
        .iter()
        .filter(|(name, _)| !writable(name, &p.package, &p.language))
        .map(|(name, difference)| (name.clone(), *difference))
        .collect::<BTreeMap<_, _>>();
    if !outside.is_empty() {
        return Err(format!(
            "The hotpatch changed files outside its area: {}.",
            gate::summarize(&outside)
        ));
    }
    let preimages = keep_preimages(ctx, dir, p, &after, &changed)?;
    let log_sha256 = after
        .files
        .get("weidu.log")
        .ok_or("The mod list is missing after the hotpatch.")?
        .sha256
        .clone();
    files::json_new(
        &dir.join("applied.json"),
        &Applied {
            prepared_sha256: files::hash(&dir.join("prepared.json"))?,
            after,
            suffix,
            log_sha256,
            preimages,
        },
    )
}

/// Keep an earlier copy of every file that changed or went away and is not among the saved
/// root files: WeiDU's own uninstall copies of overwritten files hold those bytes, found by
/// their digest among the files the hotpatch created.
fn keep_preimages(
    ctx: &Context<'_>,
    dir: &Path,
    p: &Prepared,
    now: &GameFiles,
    changed: &BTreeMap<String, Difference>,
) -> Result<BTreeSet<String>, String> {
    let mut kept = existing_preimages(dir)?;
    let mut sources: BTreeMap<&str, &str> = BTreeMap::new();
    for (name, difference) in changed {
        if *difference == Difference::Added {
            if let Some(identity) = now.files.get(name) {
                sources.entry(identity.sha256.as_str()).or_insert(name);
            }
        }
    }
    for (name, difference) in changed {
        let Some(earlier) = p.before.files.get(name) else {
            continue;
        };
        if *difference == Difference::Added
            || p.saved.get(name) == Some(&earlier.sha256)
            || kept.contains(&earlier.sha256)
        {
            continue;
        }
        let source = sources.get(earlier.sha256.as_str()).ok_or_else(|| {
            format!("No earlier copy of {name} was kept, so the hotpatch could not be undone.")
        })?;
        let bytes = files::read(&ctx.game().join(source), 512 * 1024 * 1024)?;
        if sha256_bytes(&bytes) != earlier.sha256 {
            return Err(format!("The earlier copy of {name} changed."));
        }
        files::create(&dir.join("preimages").join(&earlier.sha256), &bytes)?;
        kept.insert(earlier.sha256.clone());
    }
    Ok(kept)
}

fn existing_preimages(dir: &Path) -> Result<BTreeSet<String>, String> {
    let folder = dir.join("preimages");
    if !folder.exists() {
        return Ok(BTreeSet::new());
    }
    files::direct(&folder)?;
    let mut kept = BTreeSet::new();
    for entry in fs::read_dir(&folder).map_err(files::err)? {
        let path = entry.map_err(files::err)?.path();
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        files::regular(&path)?;
        if files::hash(&path)? != name {
            return Err("A kept earlier file copy changed.".into());
        }
        kept.insert(name);
    }
    Ok(kept)
}

/// Whether an unfinished transaction can be restored automatically.
pub(super) fn restorable(
    ctx: &Context<'_>,
    catalog: &Catalog,
    entry: &Entry,
) -> Result<(), String> {
    let p = load_prepared(&ctx.record.managed_root, &ctx.record.install_id, &entry.dir)?;
    let now = game_files::capture(&ctx.game())?;
    plan_restore(ctx, catalog, &entry.dir, &p, &now).map(|_| ())
}

/// Whether the latest applied hotpatch can be undone: nothing but runtime files changed since.
pub(super) fn undoable(
    ctx: &Context<'_>,
    catalog: &Catalog,
    entry: &Entry,
    now: &GameFiles,
) -> Result<(), String> {
    let p = load_prepared(&ctx.record.managed_root, &ctx.record.install_id, &entry.dir)?;
    let a: Applied = read_json(&entry.dir.join("applied.json"))?;
    let drift = a
        .after
        .differences(now, |name| catalog.runtime_files.matches(name));
    if !drift.is_empty() {
        return Err(format!(
            "Files changed after the hotpatch, so it cannot be undone automatically: {}.",
            gate::summarize(&drift)
        ));
    }
    plan_restore(ctx, catalog, &entry.dir, &p, now).map(|_| ())
}

/// What restoring writes and removes, with the bytes of every file it writes back.
struct RestorePlan {
    write: BTreeMap<String, Vec<u8>>,
    remove_files: BTreeMap<String, String>,
    remove_folders: Vec<String>,
    create_folders: Vec<String>,
}

fn plan_restore(
    ctx: &Context<'_>,
    catalog: &Catalog,
    dir: &Path,
    p: &Prepared,
    now: &GameFiles,
) -> Result<RestorePlan, String> {
    let changed = p
        .before
        .differences(now, |name| catalog.runtime_files.matches(name));
    let outside = changed
        .iter()
        .filter(|(name, _)| {
            !writable(name, &p.package, &p.language) && !p.saved.contains_key(*name)
        })
        .map(|(name, difference)| (name.clone(), *difference))
        .collect::<BTreeMap<_, _>>();
    if !outside.is_empty() {
        return Err(format!(
            "Files outside the hotpatch changed, so automatic restoring needs a closer look: {}.",
            gate::summarize(&outside)
        ));
    }
    let kept = existing_preimages(dir)?;
    let mut plan = RestorePlan {
        write: BTreeMap::new(),
        remove_files: BTreeMap::new(),
        remove_folders: Vec::new(),
        create_folders: Vec::new(),
    };
    for (name, difference) in &changed {
        let was_file = p.before.files.get(name);
        let was_folder = p.before.folders.contains(name);
        let is_file = now.files.get(name);
        let is_folder = now.folders.contains(name);
        if (was_file.is_some() && is_folder) || (was_folder && is_file.is_some()) {
            return Err(format!(
                "{name} changed between file and folder; automatic restoring is unavailable."
            ));
        }
        match (difference, was_file, is_file) {
            (Difference::Added, None, Some(identity)) => {
                plan.remove_files
                    .insert(name.clone(), identity.sha256.clone());
            }
            (Difference::Added, None, None) => plan.remove_folders.push(name.clone()),
            (Difference::Removed, None, None) => plan.create_folders.push(name.clone()),
            (_, Some(earlier), _) => {
                let bytes = if p.saved.get(name) == Some(&earlier.sha256) {
                    files::read(&dir.join("affected").join(name), 512 * 1024 * 1024)?
                } else if kept.contains(&earlier.sha256) {
                    files::read(
                        &dir.join("preimages").join(&earlier.sha256),
                        512 * 1024 * 1024,
                    )?
                } else {
                    // An interrupted run may not have kept it yet: WeiDU's copy is still there.
                    let source = now
                        .files
                        .iter()
                        .find(|(other, identity)| {
                            identity.sha256 == earlier.sha256
                                && !p.before.files.contains_key(*other)
                        })
                        .map(|(other, _)| other.clone())
                        .ok_or_else(|| format!("No earlier copy of {name} is available."))?;
                    files::read(&ctx.game().join(source), 512 * 1024 * 1024)?
                };
                if sha256_bytes(&bytes) != earlier.sha256 {
                    return Err(format!("The earlier copy of {name} changed."));
                }
                plan.write.insert(name.clone(), bytes);
            }
            _ => return Err(format!("{name} cannot be restored automatically.")),
        }
    }
    // Deepest first, so each folder is empty when its turn comes.
    plan.remove_folders
        .sort_by_key(|name| std::cmp::Reverse(name.matches('/').count()));
    plan.create_folders
        .sort_by_key(|name| name.matches('/').count());
    Ok(plan)
}

pub fn restore(
    ctx: &Context<'_>,
    catalog: &Catalog,
    undo: bool,
) -> Result<HotpatchPreview, String> {
    restore_with_progress(ctx, catalog, undo, &|_| {})
}

/// Undo the latest applied hotpatch (`undo`), or finish restoring an interrupted one.
pub fn restore_with_progress(
    ctx: &Context<'_>,
    catalog: &Catalog,
    undo: bool,
    progress: crate::operation_progress::ProgressSink<'_>,
) -> Result<HotpatchPreview, String> {
    crate::operation_progress::stage(progress, "checking");
    let root = &ctx.record.managed_root;
    let _lock = ctx.locks.try_acquire(root).map_err(files::err)?;
    crate::patches::launch_guard(root)?;
    crate::mod_upgrades::launch_guard(root)?;
    super::validate_context(ctx)?;
    let history = History::load(root, &ctx.record.install_id)?;
    let entry = if undo {
        if history.unfinished().is_some() {
            return Err("A hotpatch did not finish. Restore it first.".into());
        }
        history
            .undo_target()
            .ok_or("There is no hotpatch to undo.")?
    } else {
        history
            .unfinished()
            .ok_or("No interrupted hotpatch needs restoring.")?
    };
    if undo {
        undoable(ctx, catalog, entry, &game_files::capture(&ctx.game())?)?;
    }
    crate::operation_progress::stage(progress, "restoring");
    restore_inner(ctx, catalog, &entry.dir, undo)?;
    crate::operation_progress::stage(progress, "verifying");
    super::inspect(ctx, catalog)
}

fn restore_inner(
    ctx: &Context<'_>,
    catalog: &Catalog,
    dir: &Path,
    undo: bool,
) -> Result<(), String> {
    let p = load_prepared(&ctx.record.managed_root, &ctx.record.install_id, dir)?;
    // Restoring writes the saved files back, so they may be missing.
    crate::patches::close_check_in(&ctx.game(), &p.language, &p.saved, Some(&|_| true))?;
    let now = game_files::capture(&ctx.game())?;
    let plan = plan_restore(ctx, catalog, dir, &p, &now)?;
    if !dir.join("restoring.json").exists() {
        files::json_new(
            &dir.join("restoring.json"),
            &Restoring {
                prepared_sha256: files::hash(&dir.join("prepared.json"))?,
                undo,
            },
        )?;
    }
    // Earlier copies found only among WeiDU's files are kept before those files go.
    for (name, bytes) in &plan.write {
        let hash = sha256_bytes(bytes);
        let kept = dir.join("preimages").join(&hash);
        if p.saved.get(name) != Some(&hash) && !kept.exists() {
            files::create(&kept, bytes)?;
        }
    }
    let held = files::hold(&ctx.record.managed_root, dir)?;
    for (name, hash) in &plan.remove_files {
        files::remove_created(
            &held.game,
            name,
            Some(hash.as_str()),
            "A file the hotpatch created changed during restoring.",
        )?;
    }
    for name in &plan.remove_folders {
        files::remove_created(&held.game, name, None, "")?;
    }
    for name in &plan.create_folders {
        held.game
            .open_or_create_below(name.split('/'))
            .map_err(files::held)?;
    }
    for (name, bytes) in &plan.write {
        let (folders, last) = files::split_relative(name)?;
        let folder = held
            .game
            .open_or_create_below(folders)
            .map_err(files::held)?;
        files::write_held(&folder, last, bytes)?;
    }
    drop(held);
    let restored = game_files::capture(&ctx.game())?;
    let left = p
        .before
        .differences(&restored, |name| catalog.runtime_files.matches(name));
    if !left.is_empty() {
        return Err(format!(
            "Restoring did not reach the earlier state: {}.",
            gate::summarize(&left)
        ));
    }
    files::json_new(
        &dir.join("restored.json"),
        &files::hash(&dir.join("prepared.json"))?,
    )
}
