//! Fail-closed installation preflight and immediately-before-mutation checks.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use thiserror::Error;

use crate::digest::{plan_digest, selection_digest, sha256_bytes};
use crate::games::{Eligibility, GameCandidate, GameRole};
use crate::resolve::InstallPlan;
use crate::session::{CampaignCreated, FrozenIdentity, SessionReplay};

const CREATOR_PROTECTED_ROOTS: [&str; 2] = [
    r"C:\Games\Baldur's Gate II Enhanced Edition modded",
    r"C:\Games\Baldurs Gate 1 and 2 mods",
];
const DEFAULT_LANGUAGE: &str = "en_US";
static WRITE_PROBE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// One payload or executable tool input required by the frozen plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequiredInput {
    /// Stable recipe id.
    pub id: String,
    /// Exact release version or immutable revision.
    pub version: String,
    /// Exact SHA-256 identity.
    pub sha256: String,
    /// Exact expected byte length.
    pub length: u64,
    /// Whether a verified cache entry or valid acquisition route is available.
    pub obtainable: bool,
}

/// Byte accounting whose checked sum must fit on the destination volume.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpaceRequirement {
    pub staged_copies: u64,
    pub downloads: u64,
    pub extraction: u64,
    pub safety_margin: u64,
}

impl SpaceRequirement {
    fn total(self) -> Result<u64, PreflightError> {
        self.staged_copies
            .checked_add(self.downloads)
            .and_then(|sum| sum.checked_add(self.extraction))
            .and_then(|sum| sum.checked_add(self.safety_margin))
            .ok_or(PreflightError::SpaceRequirementOverflow)
    }

    fn destination_total(self) -> Result<u64, PreflightError> {
        self.staged_copies
            .checked_add(self.safety_margin)
            .ok_or(PreflightError::SpaceRequirementOverflow)
    }

    fn cache_total(self) -> Result<u64, PreflightError> {
        self.downloads
            .checked_add(self.extraction)
            .ok_or(PreflightError::SpaceRequirementOverflow)
    }
}

/// Inputs checked before a build may move past its frozen Review screen.
pub struct InitialPreflight<'a> {
    /// Freshly inspected BGEE+SoD candidate.
    pub bg1_source: &'a GameCandidate,
    /// Freshly inspected BG2EE candidate.
    pub bg2_source: &'a GameCandidate,
    pub destination: &'a Path,
    pub space: SpaceRequirement,
    /// Verified append-only replay whose created event is the sole frozen identity.
    pub frozen_campaign: &'a SessionReplay,
    /// Exact plan recomputed from the trusted recipe and current server-side selection.
    pub current_plan: &'a InstallPlan,
    /// Opaque token stored beside the server-side review snapshot.
    pub expected_review_token: &'a str,
    /// Opaque token presented by the start-build command.
    pub presented_review_token: &'a str,
    pub required_artifacts: &'a [RequiredInput],
    pub required_tools: &'a [RequiredInput],
}

/// Successful disk evidence captured by initial preflight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreflightReport {
    pub required_space: u64,
    pub available_space: u64,
}

/// Platform operations injected for deterministic orchestration tests.
pub trait PreflightHost {
    fn available_space(&self, path: &Path) -> io::Result<u64>;
    fn volume_key(&self, path: &Path) -> io::Result<OsString>;
    fn probe_directory_writable(&self, path: &Path) -> io::Result<()>;
    fn running_executable_paths(&self) -> io::Result<Vec<PathBuf>>;
    fn probe_exclusive_writable_files(&self, paths: &[PathBuf]) -> Result<(), ExclusiveFileError>;
}

/// Exact file that prevented a simultaneous exclusive TLK probe.
#[derive(Debug)]
pub struct ExclusiveFileError {
    pub path: PathBuf,
    pub source: io::Error,
}

/// Production Windows-aware preflight host.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemPreflight;

impl PreflightHost for SystemPreflight {
    fn available_space(&self, path: &Path) -> io::Result<u64> {
        fs4::available_space(path)
    }

    fn volume_key(&self, path: &Path) -> io::Result<OsString> {
        system_volume_key(path)
    }

    fn probe_directory_writable(&self, path: &Path) -> io::Result<()> {
        probe_directory_writable(path)
    }

    fn running_executable_paths(&self) -> io::Result<Vec<PathBuf>> {
        system_running_executable_paths()
    }

    fn probe_exclusive_writable_files(&self, paths: &[PathBuf]) -> Result<(), ExclusiveFileError> {
        probe_exclusive_writable_files(paths)
    }
}

/// A preflight failure that must be corrected rather than overridden.
#[derive(Debug, Error)]
pub enum PreflightError {
    #[error("protected creator reference path cannot be an install destination: {path}")]
    ProtectedDestination { path: PathBuf },
    #[error("source game {label} is unavailable at {path}: {reason}")]
    SourceUnavailable {
        label: &'static str,
        path: PathBuf,
        reason: String,
    },
    #[error("source game paths resolve to the same directory: {path}")]
    DuplicateSource { path: PathBuf },
    #[error("destination is not writable at {path}: {source}")]
    DestinationNotWritable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cache root is not writable at {path}: {source}")]
    CacheNotWritable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("disk-space requirement overflowed u64")]
    SpaceRequirementOverflow,
    #[error("insufficient disk space: need {required} bytes, have {available} bytes")]
    InsufficientSpace { required: u64, available: u64 },
    #[error("insufficient cache disk space: need {required} bytes, have {available} bytes")]
    InsufficientCacheSpace { required: u64, available: u64 },
    #[error("frozen Review token or bound digest changed")]
    ReviewMismatch,
    #[error("frozen campaign mismatch for {field}: {reason}")]
    FrozenCampaignMismatch { field: &'static str, reason: String },
    #[error("source game {label} is not a fresh eligible candidate: {reason}")]
    SourceNotFresh { label: &'static str, reason: String },
    #[error("required {kind} {id:?} is unavailable: {reason}")]
    RequiredInputUnavailable {
        kind: &'static str,
        id: String,
        reason: String,
    },
    #[error("could not enumerate running process executable paths: {source}")]
    ProcessEnumeration {
        #[source]
        source: io::Error,
    },
    #[error("a process executable under the managed target is active: {executable}")]
    TargetProcessRunning { executable: PathBuf },
    #[error("target is unavailable at {path}: {reason}")]
    TargetUnavailable { path: PathBuf, reason: String },
    #[error("dialog TLK is not exclusively writable at {path}: {source}")]
    TlkUnavailable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// Runs the production initial preflight.
pub fn initial_preflight(
    request: &InitialPreflight<'_>,
) -> Result<PreflightReport, PreflightError> {
    initial_preflight_with(request, &SystemPreflight)
}

/// Runs initial preflight using an injected platform host.
pub fn initial_preflight_with(
    request: &InitialPreflight<'_>,
    host: &dyn PreflightHost,
) -> Result<PreflightReport, PreflightError> {
    if is_creator_protected_destination(request.destination) {
        return Err(PreflightError::ProtectedDestination {
            path: request.destination.to_path_buf(),
        });
    }

    let frozen = request.frozen_campaign.created();
    verify_frozen_campaign(frozen, request.current_plan)?;
    if request.expected_review_token.trim().is_empty()
        || request.expected_review_token != request.presented_review_token
    {
        return Err(PreflightError::ReviewMismatch);
    }

    let bg1 = validate_source_candidate(
        request.bg1_source,
        GameRole::BgeeSod,
        "BGEE+SoD",
        &frozen.source_games.bg1,
    )?;
    let bg2 = validate_source_candidate(
        request.bg2_source,
        GameRole::Bg2ee,
        "BG2EE",
        &frozen.source_games.bg2,
    )?;
    if paths_equal(&bg1, &bg2) {
        return Err(PreflightError::DuplicateSource { path: bg1 });
    }
    if request.bg1_source.build != request.bg2_source.build {
        return Err(PreflightError::SourceNotFresh {
            label: "source pair",
            reason: "supported build identities differ".to_owned(),
        });
    }
    validate_required_inputs(
        "artifact",
        request.required_artifacts,
        &frozen.artifact_identities,
    )?;
    validate_required_inputs("tool", request.required_tools, &frozen.tool_identities)?;

    let (target, writable_root) = destination_and_writable_root(request.destination)?;
    if is_creator_protected_destination(&target) {
        return Err(PreflightError::ProtectedDestination { path: target });
    }
    if !paths_equal(&target, &frozen.managed_root) {
        return Err(PreflightError::FrozenCampaignMismatch {
            field: "managed root",
            reason: format!(
                "expected {}, found {}",
                frozen.managed_root.display(),
                target.display()
            ),
        });
    }
    if path_is_under(&target, &bg1)
        || path_is_under(&bg1, &target)
        || path_is_under(&target, &bg2)
        || path_is_under(&bg2, &target)
    {
        return Err(PreflightError::DestinationNotWritable {
            path: target,
            source: io::Error::other("destination overlaps a read-only source game"),
        });
    }

    let cache = validate_cache_root(&frozen.cache_root)?;
    if is_creator_protected_destination(&cache) {
        return Err(PreflightError::CacheNotWritable {
            path: cache,
            source: io::Error::new(
                io::ErrorKind::PermissionDenied,
                "cache root is inside a protected creator reference path",
            ),
        });
    }
    if path_is_under(&cache, &bg1)
        || path_is_under(&bg1, &cache)
        || path_is_under(&cache, &bg2)
        || path_is_under(&bg2, &cache)
        || path_is_under(&cache, &target)
        || path_is_under(&target, &cache)
    {
        return Err(PreflightError::CacheNotWritable {
            path: cache,
            source: io::Error::other("cache root overlaps a source game or managed target"),
        });
    }
    host.probe_directory_writable(&writable_root)
        .map_err(|source| PreflightError::DestinationNotWritable {
            path: writable_root.clone(),
            source,
        })?;
    host.probe_directory_writable(&cache)
        .map_err(|source| PreflightError::CacheNotWritable {
            path: cache.clone(),
            source,
        })?;
    check_processes_under(&target, host)?;
    probe_existing_target_tlks(request.destination, DEFAULT_LANGUAGE, host)?;

    let required_space = request.space.total()?;
    let destination_volume = host.volume_key(&writable_root).map_err(|source| {
        PreflightError::DestinationNotWritable {
            path: writable_root.clone(),
            source,
        }
    })?;
    let cache_volume =
        host.volume_key(&cache)
            .map_err(|source| PreflightError::CacheNotWritable {
                path: cache.clone(),
                source,
            })?;
    let destination_available = host.available_space(&writable_root).map_err(|source| {
        PreflightError::DestinationNotWritable {
            path: writable_root,
            source,
        }
    })?;
    let available_space = if destination_volume == cache_volume {
        if destination_available < required_space {
            return Err(PreflightError::InsufficientSpace {
                required: required_space,
                available: destination_available,
            });
        }
        destination_available
    } else {
        let destination_required = request.space.destination_total()?;
        if destination_available < destination_required {
            return Err(PreflightError::InsufficientSpace {
                required: destination_required,
                available: destination_available,
            });
        }
        let cache_required = request.space.cache_total()?;
        let cache_available =
            host.available_space(&cache)
                .map_err(|source| PreflightError::CacheNotWritable {
                    path: cache,
                    source,
                })?;
        if cache_available < cache_required {
            return Err(PreflightError::InsufficientCacheSpace {
                required: cache_required,
                available: cache_available,
            });
        }
        destination_available.saturating_add(cache_available)
    };
    Ok(PreflightReport {
        required_space,
        available_space,
    })
}

/// Repeats the target-process and TLK checks immediately before one mutating WeiDU run.
pub fn recheck_target_before_mutation(
    target_root: &Path,
    language: &str,
) -> Result<(), PreflightError> {
    recheck_target_before_mutation_with(target_root, language, &SystemPreflight)
}

/// Injected form of [`recheck_target_before_mutation`].
pub fn recheck_target_before_mutation_with(
    target_root: &Path,
    language: &str,
    host: &dyn PreflightHost,
) -> Result<(), PreflightError> {
    validate_language_component(language)?;
    let target = validate_target_directory(target_root)?;
    check_processes_under(&target, host)?;
    let requested_tlks = target_tlk_paths(&target, language);
    let mut tlks = Vec::with_capacity(requested_tlks.len());
    for tlk in &requested_tlks {
        tlks.push(validate_tlk(tlk, &target)?);
    }
    host.probe_exclusive_writable_files(&tlks)
        .map_err(|error| PreflightError::TlkUnavailable {
            path: error.path,
            source: error.source,
        })?;
    Ok(())
}

/// True only for either protected creator reference root or one of its descendants.
///
/// This is a lexical, case-insensitive Windows comparison and performs no filesystem IO,
/// so a protected path is refused before preflight can touch it.
pub fn is_creator_protected_destination(path: &Path) -> bool {
    #[cfg(windows)]
    {
        let candidate = windows_path_key(path);
        CREATOR_PROTECTED_ROOTS.iter().any(|protected| {
            let protected = windows_path_key(Path::new(protected));
            candidate == protected || candidate.starts_with(&(protected + "\\"))
        })
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

fn validate_source_candidate(
    candidate: &GameCandidate,
    expected_role: GameRole,
    label: &'static str,
    expected_fingerprint: &str,
) -> Result<PathBuf, PreflightError> {
    if candidate.role != expected_role {
        return Err(PreflightError::SourceNotFresh {
            label,
            reason: format!(
                "expected role {expected_role:?}, found {:?}",
                candidate.role
            ),
        });
    }
    if candidate.eligibility != Eligibility::Eligible {
        return Err(PreflightError::SourceNotFresh {
            label,
            reason: format!("eligibility is {:?}", candidate.eligibility),
        });
    }
    if candidate.fingerprint.as_deref() != Some(expected_fingerprint) {
        return Err(PreflightError::SourceNotFresh {
            label,
            reason: "current fingerprint differs from the frozen campaign".to_owned(),
        });
    }
    let path = &candidate.root;
    validate_direct_directory(path).map_err(|reason| PreflightError::SourceUnavailable {
        label,
        path: path.to_path_buf(),
        reason,
    })?;
    fs::read_dir(path)
        .map_err(|error| PreflightError::SourceUnavailable {
            label,
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?
        .collect::<io::Result<Vec<_>>>()
        .map_err(|error| PreflightError::SourceUnavailable {
            label,
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    let canonical = fs::canonicalize(path).map_err(|error| PreflightError::SourceUnavailable {
        label,
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    if !paths_equal(&canonical, &candidate.root) {
        return Err(PreflightError::SourceNotFresh {
            label,
            reason: "candidate root is not the current canonical path".to_owned(),
        });
    }
    Ok(canonical)
}

fn verify_frozen_campaign(
    frozen: &CampaignCreated,
    current_plan: &InstallPlan,
) -> Result<(), PreflightError> {
    let checks = [
        (
            "recipe payload",
            sha256_bytes(&frozen.recipe_payload),
            frozen.recipe_payload_sha256.as_str(),
        ),
        (
            "normalized selection",
            selection_digest(&frozen.normalized_selection).map_err(|error| {
                PreflightError::FrozenCampaignMismatch {
                    field: "normalized selection",
                    reason: error.to_string(),
                }
            })?,
            frozen.selection_sha256.as_str(),
        ),
        (
            "resolved plan",
            plan_digest(current_plan).map_err(|error| PreflightError::FrozenCampaignMismatch {
                field: "resolved plan",
                reason: error.to_string(),
            })?,
            frozen.plan_sha256.as_str(),
        ),
    ];
    for (field, current, expected) in checks {
        if current != expected {
            return Err(PreflightError::FrozenCampaignMismatch {
                field,
                reason: format!("expected {expected}, found {current}"),
            });
        }
    }
    Ok(())
}

fn validate_required_inputs(
    kind: &'static str,
    inputs: &[RequiredInput],
    expected: &[FrozenIdentity],
) -> Result<(), PreflightError> {
    let mut ids = BTreeSet::new();
    let mut identities = Vec::with_capacity(inputs.len());
    for input in inputs {
        let reason = if input.id.trim().is_empty() {
            Some("id must not be empty".to_owned())
        } else if !ids.insert(input.id.clone()) {
            Some("id is duplicated".to_owned())
        } else if input.version.trim().is_empty() {
            Some("version must not be empty".to_owned())
        } else if let Err(reason) = validate_sha256(&input.sha256) {
            Some(reason)
        } else if input.length == 0 {
            Some("expected length must be greater than zero".to_owned())
        } else if !input.obtainable {
            Some("no verified cache entry or valid acquisition route exists".to_owned())
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(PreflightError::RequiredInputUnavailable {
                kind,
                id: input.id.clone(),
                reason,
            });
        }
        identities.push(FrozenIdentity {
            id: input.id.clone(),
            version: input.version.clone(),
            sha256: input.sha256.clone(),
            length: input.length,
        });
    }
    identities.sort_by(|left, right| left.id.cmp(&right.id));
    let mut expected = expected.to_vec();
    expected.sort_by(|left, right| left.id.cmp(&right.id));
    if identities != expected {
        return Err(PreflightError::FrozenCampaignMismatch {
            field: if kind == "artifact" {
                "artifact identities"
            } else {
                "tool identities"
            },
            reason: "current required input set or pin differs from the frozen campaign".to_owned(),
        });
    }
    Ok(())
}

fn validate_sha256(digest: &str) -> Result<(), String> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || digest.bytes().all(|byte| byte == b'0')
    {
        return Err("SHA-256 must be exactly 64 lowercase hexadecimal characters".to_owned());
    }
    Ok(())
}

fn destination_and_writable_root(path: &Path) -> Result<(PathBuf, PathBuf), PreflightError> {
    let target = absolute_path(path).map_err(|source| PreflightError::DestinationNotWritable {
        path: path.to_path_buf(),
        source,
    })?;
    match fs::symlink_metadata(&target) {
        Ok(metadata) => {
            validate_directory_metadata(&target, &metadata).map_err(|reason| {
                PreflightError::DestinationNotWritable {
                    path: target.clone(),
                    source: io::Error::other(reason),
                }
            })?;
            let canonical = fs::canonicalize(&target).map_err(|source| {
                PreflightError::DestinationNotWritable {
                    path: target.clone(),
                    source,
                }
            })?;
            Ok((canonical.clone(), canonical))
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            let writable_root = nearest_existing_directory(&target)?;
            Ok((target, writable_root))
        }
        Err(source) => Err(PreflightError::DestinationNotWritable {
            path: target,
            source,
        }),
    }
}

fn nearest_existing_directory(path: &Path) -> Result<PathBuf, PreflightError> {
    let mut candidate = path.parent();
    while let Some(parent) = candidate {
        match fs::symlink_metadata(parent) {
            Ok(metadata) => {
                validate_directory_metadata(parent, &metadata).map_err(|reason| {
                    PreflightError::DestinationNotWritable {
                        path: parent.to_path_buf(),
                        source: io::Error::other(reason),
                    }
                })?;
                return fs::canonicalize(parent).map_err(|source| {
                    PreflightError::DestinationNotWritable {
                        path: parent.to_path_buf(),
                        source,
                    }
                });
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                candidate = parent.parent();
            }
            Err(source) => {
                return Err(PreflightError::DestinationNotWritable {
                    path: parent.to_path_buf(),
                    source,
                });
            }
        }
    }
    Err(PreflightError::DestinationNotWritable {
        path: path.to_path_buf(),
        source: io::Error::new(io::ErrorKind::NotFound, "no existing destination ancestor"),
    })
}

fn probe_existing_target_tlks(
    destination: &Path,
    language: &str,
    host: &dyn PreflightHost,
) -> Result<(), PreflightError> {
    for target in [destination.join("bg1"), destination.join("game")] {
        let canonical_target = match fs::symlink_metadata(&target) {
            Ok(_) => Some(validate_target_directory(&target)?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(PreflightError::TargetUnavailable {
                    path: target,
                    reason: error.to_string(),
                });
            }
        };
        let Some(canonical_target) = canonical_target else {
            continue;
        };
        let tlks = target_tlk_paths(&canonical_target, language);
        let mut existing = Vec::new();
        for tlk in tlks {
            match fs::symlink_metadata(&tlk) {
                Ok(_) => {
                    existing.push(validate_tlk(&tlk, &canonical_target)?);
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(source) => {
                    return Err(PreflightError::TlkUnavailable { path: tlk, source });
                }
            }
        }
        host.probe_exclusive_writable_files(&existing)
            .map_err(|error| PreflightError::TlkUnavailable {
                path: error.path,
                source: error.source,
            })?;
    }
    Ok(())
}

fn check_processes_under(target: &Path, host: &dyn PreflightHost) -> Result<(), PreflightError> {
    let running = host
        .running_executable_paths()
        .map_err(|source| PreflightError::ProcessEnumeration { source })?;
    for executable in running {
        let canonical = match fs::canonicalize(&executable) {
            Ok(canonical) => canonical,
            Err(source) => {
                let raw = absolute_path(&executable).unwrap_or_else(|_| executable.to_path_buf());
                if path_is_under(&raw, target) {
                    return Err(PreflightError::ProcessEnumeration {
                        source: io::Error::new(
                            source.kind(),
                            format!(
                                "could not canonicalize target-local process {}: {source}",
                                executable.display()
                            ),
                        ),
                    });
                }
                continue;
            }
        };
        if path_is_under(&canonical, target) {
            return Err(PreflightError::TargetProcessRunning {
                executable: canonical,
            });
        }
    }
    Ok(())
}

fn target_tlk_paths(target_root: &Path, language: &str) -> [PathBuf; 2] {
    [
        target_root.join("dialog.tlk"),
        target_root.join("lang").join(language).join("dialog.tlk"),
    ]
}

fn validate_target_directory(path: &Path) -> Result<PathBuf, PreflightError> {
    validate_direct_directory(path).map_err(|reason| PreflightError::TargetUnavailable {
        path: path.to_path_buf(),
        reason,
    })?;
    fs::canonicalize(path).map_err(|error| PreflightError::TargetUnavailable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

fn validate_cache_root(path: &Path) -> Result<PathBuf, PreflightError> {
    validate_direct_directory(path).map_err(|reason| PreflightError::CacheNotWritable {
        path: path.to_path_buf(),
        source: io::Error::other(reason),
    })?;
    let canonical = fs::canonicalize(path).map_err(|source| PreflightError::CacheNotWritable {
        path: path.to_path_buf(),
        source,
    })?;
    if !paths_equal(&canonical, path) {
        return Err(PreflightError::CacheNotWritable {
            path: path.to_path_buf(),
            source: io::Error::other("cache root is not the current canonical path"),
        });
    }
    Ok(canonical)
}

fn validate_tlk(path: &Path, canonical_target: &Path) -> Result<PathBuf, PreflightError> {
    let mut parent = path
        .parent()
        .ok_or_else(|| PreflightError::TlkUnavailable {
            path: path.to_path_buf(),
            source: io::Error::other("TLK has no parent directory"),
        })?;
    loop {
        validate_direct_directory(parent).map_err(|reason| PreflightError::TlkUnavailable {
            path: parent.to_path_buf(),
            source: io::Error::other(reason),
        })?;
        if paths_equal(parent, canonical_target) {
            break;
        }
        if !path_is_under(parent, canonical_target) {
            return Err(PreflightError::TlkUnavailable {
                path: parent.to_path_buf(),
                source: io::Error::other("TLK parent escapes the canonical target"),
            });
        }
        parent = parent
            .parent()
            .ok_or_else(|| PreflightError::TlkUnavailable {
                path: path.to_path_buf(),
                source: io::Error::other("TLK ancestry does not reach the canonical target"),
            })?;
    }
    let metadata = fs::symlink_metadata(path).map_err(|source| PreflightError::TlkUnavailable {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || is_windows_reparse_point(&metadata) {
        return Err(PreflightError::TlkUnavailable {
            path: path.to_path_buf(),
            source: io::Error::other("links and reparse points are not allowed"),
        });
    }
    if !metadata.is_file() {
        return Err(PreflightError::TlkUnavailable {
            path: path.to_path_buf(),
            source: io::Error::other("expected a regular file"),
        });
    }
    let canonical = fs::canonicalize(path).map_err(|source| PreflightError::TlkUnavailable {
        path: path.to_path_buf(),
        source,
    })?;
    if !path_is_under(&canonical, canonical_target) {
        return Err(PreflightError::TlkUnavailable {
            path: canonical,
            source: io::Error::other("TLK resolves outside the canonical target"),
        });
    }
    Ok(canonical)
}

fn validate_language_component(language: &str) -> Result<(), PreflightError> {
    if language.is_empty()
        || !language
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(PreflightError::TargetUnavailable {
            path: PathBuf::from(language),
            reason: "language must be one safe path component".to_owned(),
        });
    }
    Ok(())
}

fn validate_direct_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    validate_directory_metadata(path, &metadata)
}

fn validate_directory_metadata(path: &Path, metadata: &fs::Metadata) -> Result<(), String> {
    if metadata.file_type().is_symlink() || is_windows_reparse_point(metadata) {
        return Err(format!(
            "links and reparse points are not allowed: {}",
            path.display()
        ));
    }
    if !metadata.is_dir() {
        return Err("expected a directory".to_owned());
    }
    Ok(())
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        windows_path_key(left) == windows_path_key(right)
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn path_is_under(candidate: &Path, root: &Path) -> bool {
    #[cfg(windows)]
    {
        let candidate = windows_path_key(candidate);
        let root = windows_path_key(root);
        candidate == root || candidate.starts_with(&(root + "\\"))
    }
    #[cfg(not(windows))]
    {
        candidate.starts_with(root)
    }
}

#[cfg(windows)]
fn windows_path_key(path: &Path) -> String {
    use std::path::Component;

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
        }
    }
    let mut key = normalized.to_string_lossy().replace('/', "\\");
    if let Some(stripped) = key.strip_prefix(r"\\?\") {
        key = stripped.to_owned();
    }
    while key.len() > 3 && key.ends_with('\\') {
        key.pop();
    }
    key.to_lowercase()
}

fn probe_directory_writable(path: &Path) -> io::Result<()> {
    for _ in 0..32 {
        let sequence = WRITE_PROBE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let probe = path.join(format!(
            ".chriz-preflight-write-{}-{sequence}",
            std::process::id()
        ));
        match OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&probe)
        {
            Ok(file) => {
                drop(file);
                return fs::remove_file(probe);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not reserve a unique destination write probe",
    ))
}

#[cfg(windows)]
fn system_volume_key(path: &Path) -> io::Result<OsString> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    use windows_sys::Win32::Storage::FileSystem::GetVolumePathNameW;

    let mut input: Vec<u16> = path.as_os_str().encode_wide().collect();
    input.push(0);
    let mut output = vec![0_u16; 32_768];
    if unsafe { GetVolumePathNameW(input.as_ptr(), output.as_mut_ptr(), output.len() as u32) } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let length = output
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(output.len());
    Ok(OsString::from_wide(&output[..length]))
}

#[cfg(not(windows))]
fn system_volume_key(path: &Path) -> io::Result<OsString> {
    use std::os::unix::fs::MetadataExt;

    Ok(fs::metadata(path)?.dev().to_string().into())
}

#[cfg(windows)]
fn probe_exclusive_writable_files(paths: &[PathBuf]) -> Result<(), ExclusiveFileError> {
    use std::os::windows::fs::OpenOptionsExt;

    let mut handles = Vec::with_capacity(paths.len());
    for path in paths {
        let handle = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(path)
            .map_err(|source| ExclusiveFileError {
                path: path.clone(),
                source,
            })?;
        handles.push(handle);
    }
    drop(handles);
    Ok(())
}

#[cfg(not(windows))]
fn probe_exclusive_writable_files(paths: &[PathBuf]) -> Result<(), ExclusiveFileError> {
    use fs4::fs_std::FileExt;

    let mut handles = Vec::with_capacity(paths.len());
    for path in paths {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|source| ExclusiveFileError {
                path: path.clone(),
                source,
            })?;
        match file
            .try_lock_exclusive()
            .map_err(|source| ExclusiveFileError {
                path: path.clone(),
                source,
            })? {
            true => handles.push(file),
            false => {
                return Err(ExclusiveFileError {
                    path: path.clone(),
                    source: io::Error::new(io::ErrorKind::WouldBlock, "file lock is held"),
                });
            }
        }
    }
    for file in &handles {
        let _ = FileExt::unlock(file);
    }
    Ok(())
}

#[cfg(windows)]
fn system_running_executable_paths() -> io::Result<Vec<PathBuf>> {
    use std::mem::size_of;
    use std::os::windows::ffi::OsStringExt;

    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let _snapshot = HandleGuard(snapshot);
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut paths = Vec::new();
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    if !has_entry {
        let error = unsafe { GetLastError() };
        if error != ERROR_NO_MORE_FILES {
            return Err(io::Error::from_raw_os_error(error as i32));
        }
    }
    while has_entry {
        let name_length = entry
            .szExeFile
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(entry.szExeFile.len());
        let process_name = String::from_utf16_lossy(&entry.szExeFile[..name_length]);
        let relevant = is_relevant_process_name(&process_name);
        let process =
            unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32ProcessID) };
        if process.is_null() {
            if relevant {
                let source = io::Error::last_os_error();
                return Err(io::Error::new(
                    source.kind(),
                    format!(
                        "could not inspect relevant process {process_name:?} (pid {}): {source}",
                        entry.th32ProcessID
                    ),
                ));
            }
        } else {
            let process = HandleGuard(process);
            let mut buffer = vec![0_u16; 32_768];
            let mut length = buffer.len() as u32;
            if unsafe {
                QueryFullProcessImageNameW(
                    process.0,
                    PROCESS_NAME_WIN32,
                    buffer.as_mut_ptr(),
                    &mut length,
                )
            } != 0
            {
                paths.push(PathBuf::from(OsString::from_wide(
                    &buffer[..length as usize],
                )));
            } else if relevant {
                let source = io::Error::last_os_error();
                return Err(io::Error::new(
                    source.kind(),
                    format!(
                        "could not resolve relevant process {process_name:?} (pid {}): {source}",
                        entry.th32ProcessID
                    ),
                ));
            }
        }
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
        if !has_entry {
            let error = unsafe { GetLastError() };
            if error != ERROR_NO_MORE_FILES {
                return Err(io::Error::from_raw_os_error(error as i32));
            }
        }
    }
    Ok(paths)
}

#[cfg(windows)]
fn is_relevant_process_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    matches!(
        name.as_str(),
        "baldur.exe" | "infinityloader.exe" | "weidu.exe" | "eeex.exe"
    ) || (name.starts_with("setup-") && name.ends_with(".exe"))
}

#[cfg(not(windows))]
fn system_running_executable_paths() -> io::Result<Vec<PathBuf>> {
    Ok(Vec::new())
}

#[cfg(windows)]
fn is_windows_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_windows_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}
