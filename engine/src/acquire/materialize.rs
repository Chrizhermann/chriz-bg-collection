use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::archive::{casefold, join_relative, normalize_relative_path, verify_extracted_artifact};
use super::{AcquireError, ExtractedArtifact};

const BUFFER_SIZE: usize = 64 * 1024;
const STATE_DIRECTORY: &str = ".chriz-bg-collection";
const PUBLICATION_MANIFEST: &str = "publication-manifest.json";

/// An explicit, hash-bound permission to replace one previously owned path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedCollisionRule {
    /// Exact destination-relative path.
    pub relative_path: String,
    /// Owner recorded for the current bytes.
    pub existing_owner: String,
    /// Owner publishing the replacement.
    pub incoming_owner: String,
    /// Exact SHA-256 of the bytes being replaced.
    pub expected_input_sha256: String,
    /// Exact SHA-256 of the replacement bytes.
    pub expected_output_sha256: String,
}

/// One declared projection of an extracted artifact into a staged game root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializationRequest {
    /// Stable operation identifier used to recognize only this operation's temporaries.
    pub materialization_id: String,
    /// Recipe feature that owns newly published paths.
    pub owner: String,
    /// Approved extraction roots to project.
    pub roots: Vec<String>,
    /// Approved exact TP2 files to project.
    pub tp2_paths: Vec<String>,
    /// Explicit replacement permissions.
    pub collision_rules: Vec<SignedCollisionRule>,
}

/// Durable, deterministic record of every engine-published destination path.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct PublicationManifest {
    /// Manifest schema version.
    pub version: u32,
    /// All currently published paths, sorted case-insensitively.
    pub entries: Vec<PublishedPath>,
}

/// One file recorded in a publication manifest.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PublishedPath {
    /// Portable destination-relative path.
    pub relative_path: String,
    /// Recipe feature that owns the current bytes.
    pub owner: String,
    /// Exact byte length.
    pub length: u64,
    /// Exact lowercase SHA-256.
    pub sha256: String,
}

/// Result of a completed materialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializationResult {
    /// Durable manifest path under the staged game root.
    pub manifest_path: PathBuf,
    /// Complete post-operation manifest.
    pub manifest: PublicationManifest,
}

#[derive(Clone, Debug)]
struct PayloadFile {
    relative_path: String,
    source: PathBuf,
    length: u64,
    sha256: String,
}

#[derive(Clone, Debug)]
enum PublishAction {
    Identical { preserve_owner: Option<String> },
    Create,
    Replace { expected_input_sha256: String },
}

#[derive(Clone, Debug)]
struct PlannedPublish {
    payload: PayloadFile,
    action: PublishAction,
}

/// Safely project only declared payload roots and TP2s into an existing staged game.
pub fn materialize(
    artifact: &ExtractedArtifact,
    destination: &Path,
    request: &MaterializationRequest,
) -> Result<MaterializationResult, AcquireError> {
    validate_identifier(&request.materialization_id, "materialization_id")?;
    validate_owner(&request.owner)?;
    validate_directory(destination, "materialization destination")?;
    verify_extracted_artifact(artifact)?;

    let roots = normalize_requested_subset(
        &request.roots,
        &artifact.expected_roots,
        "materialization root",
    )?;
    let tp2_paths = normalize_requested_subset(
        &request.tp2_paths,
        &artifact.expected_tp2_paths,
        "materialization TP2 path",
    )?;
    if roots.is_empty() && tp2_paths.is_empty() {
        return Err(AcquireError::InvalidMaterialization(
            "at least one root or TP2 path must be declared".to_owned(),
        ));
    }
    let collision_rules = validate_collision_rules(request)?;
    let payload = collect_payload(artifact, &roots, &tp2_paths)?;
    let payload_by_key: BTreeMap<_, _> = payload
        .iter()
        .map(|file| (casefold(&file.relative_path), file))
        .collect();

    let state_directory = destination.join(STATE_DIRECTORY);
    validate_optional_state_directory(&state_directory)?;
    let manifest_path = state_directory.join(PUBLICATION_MANIFEST);
    let mut manifest = load_manifest(&manifest_path)?;
    validate_and_reconcile_manifest(
        destination,
        &mut manifest,
        &payload_by_key,
        &collision_rules,
        &request.owner,
    )?;

    let manifest_by_key: BTreeMap<_, _> = manifest
        .entries
        .iter()
        .map(|entry| (casefold(&entry.relative_path), entry.clone()))
        .collect();
    let mut planned = Vec::with_capacity(payload.len());
    for payload_file in payload {
        validate_destination_ancestors(destination, &payload_file.relative_path)?;
        let key = casefold(&payload_file.relative_path);
        let target = join_relative(destination, &payload_file.relative_path);
        let existing = inspect_optional_regular_file(&target)?;
        let recorded = manifest_by_key.get(&key);
        let action = match existing {
            None => PublishAction::Create,
            Some((length, hash))
                if length == payload_file.length && hash == payload_file.sha256 =>
            {
                PublishAction::Identical {
                    preserve_owner: recorded.map(|entry| entry.owner.clone()),
                }
            }
            Some((_length, hash)) => {
                let Some(recorded) = recorded else {
                    return Err(AcquireError::UndeclaredOverwrite {
                        path: payload_file.relative_path,
                    });
                };
                let Some(rule) = collision_rules.get(&key) else {
                    return Err(AcquireError::UndeclaredOverwrite {
                        path: payload_file.relative_path,
                    });
                };
                validate_collision_match(rule, recorded, &hash, &payload_file, &request.owner)?;
                PublishAction::Replace {
                    expected_input_sha256: hash,
                }
            }
        };
        planned.push(PlannedPublish {
            payload: payload_file,
            action,
        });
    }

    for plan in &planned {
        remove_owned_temporary(destination, plan, &request.materialization_id)?;
    }

    let mut entries: BTreeMap<String, PublishedPath> = manifest
        .entries
        .into_iter()
        .map(|entry| (casefold(&entry.relative_path), entry))
        .collect();
    for plan in planned {
        publish_one(destination, &plan, &request.materialization_id)?;
        let owner = match &plan.action {
            PublishAction::Identical {
                preserve_owner: Some(owner),
            } => owner.clone(),
            _ => request.owner.clone(),
        };
        entries.insert(
            casefold(&plan.payload.relative_path),
            PublishedPath {
                relative_path: plan.payload.relative_path,
                owner,
                length: plan.payload.length,
                sha256: plan.payload.sha256,
            },
        );
    }
    let mut manifest = PublicationManifest {
        version: 1,
        entries: entries.into_values().collect(),
    };
    manifest
        .entries
        .sort_by_key(|entry| casefold(&entry.relative_path));
    persist_manifest(
        &state_directory,
        &manifest_path,
        &manifest,
        &request.materialization_id,
    )?;
    Ok(MaterializationResult {
        manifest_path,
        manifest,
    })
}

fn validate_identifier(value: &str, label: &str) -> Result<(), AcquireError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(AcquireError::InvalidMaterialization(format!(
            "{label} must contain 1-128 ASCII letters, digits, '-' or '_'"
        )));
    }
    Ok(())
}

fn validate_owner(value: &str) -> Result<(), AcquireError> {
    if value.is_empty()
        || value.len() > 256
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(AcquireError::InvalidMaterialization(
            "owner must contain 1-256 ASCII letters, digits, '-', '_' or '.'".to_owned(),
        ));
    }
    Ok(())
}

fn validate_directory(path: &Path, label: &str) -> Result<(), AcquireError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| AcquireError::Io {
        action: "inspect directory",
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(AcquireError::InvalidMaterialization(format!(
            "{label} must be an existing non-symlink directory"
        )));
    }
    Ok(())
}

fn normalize_requested_subset(
    requested: &[String],
    approved: &[String],
    label: &str,
) -> Result<Vec<String>, AcquireError> {
    let approved: BTreeSet<_> = approved.iter().map(|path| casefold(path)).collect();
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for value in requested {
        let normalized = normalize_relative_path(value, false).map_err(|error| {
            AcquireError::InvalidMaterialization(format!("invalid {label} `{value}`: {error}"))
        })?;
        if is_engine_state_path(&normalized) {
            return Err(AcquireError::InvalidMaterialization(format!(
                "{label} `{value}` collides with the reserved engine state directory"
            )));
        }
        let key = casefold(&normalized);
        if !approved.contains(&key) {
            return Err(AcquireError::InvalidMaterialization(format!(
                "{label} `{value}` was not approved during extraction"
            )));
        }
        if !seen.insert(key) {
            return Err(AcquireError::InvalidMaterialization(format!(
                "duplicate case-insensitive {label} `{value}`"
            )));
        }
        result.push(normalized);
    }
    result.sort_by_key(|path| casefold(path));
    Ok(result)
}

fn validate_collision_rules(
    request: &MaterializationRequest,
) -> Result<BTreeMap<String, SignedCollisionRule>, AcquireError> {
    let mut rules = BTreeMap::new();
    for authored in &request.collision_rules {
        let relative_path =
            normalize_relative_path(&authored.relative_path, false).map_err(|error| {
                AcquireError::InvalidMaterialization(format!(
                    "invalid collision path `{}`: {error}",
                    authored.relative_path
                ))
            })?;
        validate_owner(&authored.existing_owner)?;
        validate_owner(&authored.incoming_owner)?;
        let mut rule = authored.clone();
        rule.relative_path = relative_path.clone();
        rule.expected_input_sha256 = validate_hash(&rule.expected_input_sha256)?;
        rule.expected_output_sha256 = validate_hash(&rule.expected_output_sha256)?;
        if rules.insert(casefold(&relative_path), rule).is_some() {
            return Err(AcquireError::InvalidMaterialization(format!(
                "duplicate collision rule for `{relative_path}`"
            )));
        }
    }
    Ok(rules)
}

fn validate_hash(value: &str) -> Result<String, AcquireError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AcquireError::InvalidMaterialization(
            "collision hashes must contain exactly 64 hexadecimal characters".to_owned(),
        ));
    }
    Ok(value.to_ascii_lowercase())
}

fn collect_payload(
    artifact: &ExtractedArtifact,
    roots: &[String],
    tp2_paths: &[String],
) -> Result<Vec<PayloadFile>, AcquireError> {
    let mut selected = BTreeMap::<String, PathBuf>::new();
    for root in roots {
        let source_root = join_relative(&artifact.root, root);
        let metadata = fs::symlink_metadata(&source_root).map_err(|source| AcquireError::Io {
            action: "inspect declared payload root",
            path: source_root.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() || (!metadata.is_dir() && !metadata.is_file()) {
            return Err(AcquireError::InvalidMaterialization(format!(
                "declared payload root `{root}` must be a regular file or non-symlink directory"
            )));
        }
        if metadata.is_file() {
            if !is_archive_setup_executable(root)
                && selected.insert(casefold(root), source_root).is_some()
            {
                return Err(AcquireError::InvalidMaterialization(format!(
                    "duplicate payload path `{root}`"
                )));
            }
            continue;
        }
        for entry in WalkDir::new(&source_root).follow_links(false) {
            let entry =
                entry.map_err(|error| AcquireError::InvalidMaterialization(error.to_string()))?;
            if entry.file_type().is_symlink() {
                return Err(AcquireError::InvalidMaterialization(format!(
                    "payload symlink `{}` is forbidden",
                    entry.path().display()
                )));
            }
            if !entry.file_type().is_file() {
                continue;
            }
            let relative = entry.path().strip_prefix(&artifact.root).map_err(|_| {
                AcquireError::InvalidMaterialization(
                    "payload path escaped the extraction root".to_owned(),
                )
            })?;
            let relative = components_to_portable(relative)?;
            if is_archive_setup_executable(&relative) {
                continue;
            }
            if selected
                .insert(casefold(&relative), entry.path().to_path_buf())
                .is_some()
            {
                return Err(AcquireError::InvalidMaterialization(format!(
                    "duplicate payload path `{relative}`"
                )));
            }
        }
    }
    for tp2 in tp2_paths {
        let source = join_relative(&artifact.root, tp2);
        let metadata = fs::symlink_metadata(&source).map_err(|source_error| AcquireError::Io {
            action: "inspect declared TP2",
            path: source.clone(),
            source: source_error,
        })?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(AcquireError::InvalidMaterialization(format!(
                "declared TP2 `{tp2}` is not a regular file"
            )));
        }
        selected.entry(casefold(tp2)).or_insert(source);
    }
    let mut payload = Vec::with_capacity(selected.len());
    for (key, source) in selected {
        let relative_path =
            components_to_portable(source.strip_prefix(&artifact.root).map_err(|_| {
                AcquireError::InvalidMaterialization("payload escaped extraction root".to_owned())
            })?)?;
        if key != casefold(&relative_path) {
            return Err(AcquireError::InvalidMaterialization(
                "payload case-fold identity changed during enumeration".to_owned(),
            ));
        }
        let (length, sha256) = hash_regular_file(&source)?;
        payload.push(PayloadFile {
            relative_path,
            source,
            length,
            sha256,
        });
    }
    payload.sort_by_key(|file| casefold(&file.relative_path));
    Ok(payload)
}

fn is_archive_setup_executable(relative: &str) -> bool {
    let name = relative
        .rsplit('/')
        .next()
        .unwrap_or(relative)
        .to_ascii_lowercase();
    name.starts_with("setup-") && name.ends_with(".exe")
}

fn components_to_portable(path: &Path) -> Result<String, AcquireError> {
    let components: Result<Vec<_>, _> = path
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| {
                    AcquireError::InvalidMaterialization(
                        "payload path is not valid Unicode".to_owned(),
                    )
                })
        })
        .collect();
    let portable = components?.join("/");
    normalize_relative_path(&portable, false)
        .map_err(|error| AcquireError::InvalidMaterialization(error.to_string()))
}

fn validate_optional_state_directory(path: &Path) -> Result<(), AcquireError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(AcquireError::InvalidMaterialization(format!(
            "state path `{}` is not a regular directory",
            path.display()
        ))),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(AcquireError::Io {
            action: "inspect publication state directory",
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn load_manifest(path: &Path) -> Result<PublicationManifest, AcquireError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(PublicationManifest {
                version: 1,
                entries: Vec::new(),
            });
        }
        Err(source) => {
            return Err(AcquireError::Io {
                action: "open publication manifest",
                path: path.to_path_buf(),
                source,
            });
        }
    };
    let manifest: PublicationManifest =
        serde_json::from_reader(file).map_err(|source| AcquireError::Metadata {
            path: path.to_path_buf(),
            message: source.to_string(),
        })?;
    if manifest.version != 1 {
        return Err(AcquireError::PublicationMismatch {
            path: STATE_DIRECTORY.to_owned(),
            message: format!("unsupported manifest version {}", manifest.version),
        });
    }
    Ok(manifest)
}

fn validate_and_reconcile_manifest(
    destination: &Path,
    manifest: &mut PublicationManifest,
    payload: &BTreeMap<String, &PayloadFile>,
    rules: &BTreeMap<String, SignedCollisionRule>,
    incoming_owner: &str,
) -> Result<(), AcquireError> {
    let mut seen = BTreeSet::new();
    for entry in &mut manifest.entries {
        validate_owner(&entry.owner)?;
        entry.sha256 = validate_hash(&entry.sha256)?;
        let normalized = normalize_relative_path(&entry.relative_path, false).map_err(|error| {
            AcquireError::PublicationMismatch {
                path: entry.relative_path.clone(),
                message: error.to_string(),
            }
        })?;
        if is_engine_state_path(&normalized) {
            return Err(AcquireError::PublicationMismatch {
                path: normalized,
                message: "manifest path collides with the reserved engine state directory"
                    .to_owned(),
            });
        }
        let key = casefold(&normalized);
        if !seen.insert(key.clone()) {
            return Err(AcquireError::PublicationMismatch {
                path: normalized,
                message: "duplicate case-insensitive manifest path".to_owned(),
            });
        }
        entry.relative_path = normalized;
        validate_destination_ancestors(destination, &entry.relative_path)?;
        let target = join_relative(destination, &entry.relative_path);
        let Some((length, actual)) = inspect_optional_regular_file(&target)? else {
            return Err(AcquireError::PublicationMismatch {
                path: entry.relative_path.clone(),
                message: "recorded destination file is missing".to_owned(),
            });
        };
        if length == entry.length && actual == entry.sha256 {
            continue;
        }
        let recovered = payload.get(&key).is_some_and(|planned| {
            length == planned.length
                && actual == planned.sha256
                && rules.get(&key).is_some_and(|rule| {
                    rule.existing_owner == entry.owner
                        && rule.incoming_owner == incoming_owner
                        && rule.expected_input_sha256 == entry.sha256
                        && rule.expected_output_sha256 == actual
                })
        });
        if recovered {
            let planned = payload[&key];
            entry.owner = incoming_owner.to_owned();
            entry.length = planned.length;
            entry.sha256 = planned.sha256.clone();
            continue;
        }
        return Err(AcquireError::PublicationMismatch {
            path: entry.relative_path.clone(),
            message: format!(
                "manifest expected {} bytes {}, found {length} bytes {actual}",
                entry.length, entry.sha256
            ),
        });
    }
    manifest
        .entries
        .sort_by_key(|entry| casefold(&entry.relative_path));
    Ok(())
}

fn validate_destination_ancestors(
    destination: &Path,
    relative_path: &str,
) -> Result<(), AcquireError> {
    let mut current = destination.to_path_buf();
    let components: Vec<_> = relative_path.split('/').collect();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(AcquireError::InvalidMaterialization(format!(
                    "destination ancestor `{}` is not a regular directory",
                    current.display()
                )));
            }
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => break,
            Err(source) => {
                return Err(AcquireError::Io {
                    action: "inspect destination ancestor",
                    path: current,
                    source,
                });
            }
        }
    }
    Ok(())
}

fn inspect_optional_regular_file(path: &Path) -> Result<Option<(u64, String)>, AcquireError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            hash_regular_file(path).map(Some)
        }
        Ok(_) => Err(AcquireError::InvalidMaterialization(format!(
            "destination path `{}` is not a regular file",
            path.display()
        ))),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(AcquireError::Io {
            action: "inspect destination file",
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn validate_collision_match(
    rule: &SignedCollisionRule,
    recorded: &PublishedPath,
    actual_input: &str,
    payload: &PayloadFile,
    incoming_owner: &str,
) -> Result<(), AcquireError> {
    if rule.existing_owner != recorded.owner
        || rule.incoming_owner != incoming_owner
        || rule.expected_input_sha256 != actual_input
        || rule.expected_output_sha256 != payload.sha256
    {
        return Err(AcquireError::CollisionRuleRejected {
            path: payload.relative_path.clone(),
            message: "owners or exact input/output hashes do not match".to_owned(),
        });
    }
    Ok(())
}

fn remove_owned_temporary(
    destination: &Path,
    plan: &PlannedPublish,
    materialization_id: &str,
) -> Result<(), AcquireError> {
    let target = join_relative(destination, &plan.payload.relative_path);
    let temporary = temporary_sibling(&target, materialization_id)?;
    match fs::symlink_metadata(&temporary) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            fs::remove_file(&temporary).map_err(|source| AcquireError::Io {
                action: "remove owned materialization temporary",
                path: temporary,
                source,
            })
        }
        Ok(_) => Err(AcquireError::InvalidMaterialization(format!(
            "owned temporary `{}` is not a regular file",
            temporary.display()
        ))),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(AcquireError::Io {
            action: "inspect owned materialization temporary",
            path: temporary,
            source,
        }),
    }
}

fn publish_one(
    destination: &Path,
    plan: &PlannedPublish,
    materialization_id: &str,
) -> Result<(), AcquireError> {
    if matches!(plan.action, PublishAction::Identical { .. }) {
        return Ok(());
    }
    let target = join_relative(destination, &plan.payload.relative_path);
    let parent = target.parent().ok_or_else(|| {
        AcquireError::InvalidMaterialization("destination file has no parent".to_owned())
    })?;
    fs::create_dir_all(parent).map_err(|source| AcquireError::Io {
        action: "create materialization directory",
        path: parent.to_path_buf(),
        source,
    })?;
    validate_destination_ancestors(destination, &plan.payload.relative_path)?;
    let temporary = temporary_sibling(&target, materialization_id)?;
    write_synced_temporary(&plan.payload, &temporary)?;

    let publication = match &plan.action {
        PublishAction::Create => {
            if target.exists() {
                Err(AcquireError::UndeclaredOverwrite {
                    path: plan.payload.relative_path.clone(),
                })
            } else {
                fs::hard_link(&temporary, &target).map_err(|source| AcquireError::Io {
                    action: "publish materialized file without replacement",
                    path: target.clone(),
                    source,
                })?;
                fs::remove_file(&temporary).map_err(|source| AcquireError::Io {
                    action: "remove published materialization temporary",
                    path: temporary.clone(),
                    source,
                })?;
                Ok(())
            }
        }
        PublishAction::Replace {
            expected_input_sha256,
        } => {
            let Some((_length, actual)) = inspect_optional_regular_file(&target)? else {
                return Err(AcquireError::CollisionRuleRejected {
                    path: plan.payload.relative_path.clone(),
                    message: "replacement input disappeared before publication".to_owned(),
                });
            };
            if &actual != expected_input_sha256 {
                Err(AcquireError::CollisionRuleRejected {
                    path: plan.payload.relative_path.clone(),
                    message: "replacement input changed before publication".to_owned(),
                })
            } else {
                fs::rename(&temporary, &target).map_err(|source| AcquireError::Io {
                    action: "publish signed collision replacement",
                    path: target,
                    source,
                })?;
                Ok(())
            }
        }
        PublishAction::Identical { .. } => unreachable!(),
    };
    if publication.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    publication
}

fn write_synced_temporary(payload: &PayloadFile, temporary: &Path) -> Result<(), AcquireError> {
    let mut input = File::open(&payload.source).map_err(|source| AcquireError::Io {
        action: "open extracted payload",
        path: payload.source.clone(),
        source,
    })?;
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)
        .map_err(|source| AcquireError::Io {
            action: "create materialization temporary",
            path: temporary.to_path_buf(),
            source,
        })?;
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = input.read(&mut buffer).map_err(|source| AcquireError::Io {
            action: "read extracted payload",
            path: payload.source.clone(),
            source,
        })?;
        if read == 0 {
            break;
        }
        output
            .write_all(&buffer[..read])
            .map_err(|source| AcquireError::Io {
                action: "write materialization temporary",
                path: temporary.to_path_buf(),
                source,
            })?;
        length = length.checked_add(read as u64).ok_or_else(|| {
            AcquireError::InvalidMaterialization("payload length overflowed u64".to_owned())
        })?;
        hash.update(&buffer[..read]);
    }
    let actual = hex::encode(hash.finalize());
    if length != payload.length || actual != payload.sha256 {
        return Err(AcquireError::PublicationMismatch {
            path: payload.relative_path.clone(),
            message: "extracted source changed during materialization".to_owned(),
        });
    }
    output.sync_all().map_err(|source| AcquireError::Io {
        action: "sync materialization temporary",
        path: temporary.to_path_buf(),
        source,
    })
}

fn temporary_sibling(target: &Path, materialization_id: &str) -> Result<PathBuf, AcquireError> {
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            AcquireError::InvalidMaterialization("destination filename is not Unicode".to_owned())
        })?;
    Ok(target.with_file_name(format!(".{name}.chriz-bg-{materialization_id}.tmp")))
}

fn persist_manifest(
    state_directory: &Path,
    manifest_path: &Path,
    manifest: &PublicationManifest,
    materialization_id: &str,
) -> Result<(), AcquireError> {
    fs::create_dir_all(state_directory).map_err(|source| AcquireError::Io {
        action: "create publication state directory",
        path: state_directory.to_path_buf(),
        source,
    })?;
    validate_directory(state_directory, "publication state directory")?;
    let temporary = temporary_sibling(manifest_path, materialization_id)?;
    match fs::symlink_metadata(&temporary) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            fs::remove_file(&temporary).map_err(|source| AcquireError::Io {
                action: "remove owned manifest temporary",
                path: temporary.clone(),
                source,
            })?;
        }
        Ok(_) => {
            return Err(AcquireError::InvalidMaterialization(format!(
                "manifest temporary `{}` is not a regular file",
                temporary.display()
            )));
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(AcquireError::Io {
                action: "inspect manifest temporary",
                path: temporary,
                source,
            });
        }
    }
    let mut bytes =
        serde_json::to_vec_pretty(manifest).map_err(|source| AcquireError::Metadata {
            path: manifest_path.to_path_buf(),
            message: source.to_string(),
        })?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|source| AcquireError::Io {
            action: "create publication manifest temporary",
            path: temporary.clone(),
            source,
        })?;
    file.write_all(&bytes).map_err(|source| AcquireError::Io {
        action: "write publication manifest temporary",
        path: temporary.clone(),
        source,
    })?;
    file.sync_all().map_err(|source| AcquireError::Io {
        action: "sync publication manifest temporary",
        path: temporary.clone(),
        source,
    })?;
    drop(file);
    fs::rename(&temporary, manifest_path).map_err(|source| AcquireError::Io {
        action: "publish publication manifest",
        path: manifest_path.to_path_buf(),
        source,
    })
}

fn hash_regular_file(path: &Path) -> Result<(u64, String), AcquireError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| AcquireError::Io {
        action: "inspect regular file",
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(AcquireError::InvalidMaterialization(format!(
            "`{}` is not a regular file",
            path.display()
        )));
    }
    let mut file = File::open(path).map_err(|source| AcquireError::Io {
        action: "open file for hashing",
        path: path.to_path_buf(),
        source,
    })?;
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).map_err(|source| AcquireError::Io {
            action: "read file for hashing",
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        length = length.checked_add(read as u64).ok_or_else(|| {
            AcquireError::InvalidMaterialization("file length overflowed u64".to_owned())
        })?;
        hash.update(&buffer[..read]);
    }
    Ok((length, hex::encode(hash.finalize())))
}

fn is_engine_state_path(relative_path: &str) -> bool {
    relative_path
        .split('/')
        .next()
        .is_some_and(|component| component.eq_ignore_ascii_case(STATE_DIRECTORY))
}
