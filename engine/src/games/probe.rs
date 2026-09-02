//! Profile-driven, fail-closed inspection of one explicit source path.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::profile::{
    AllowedCleanVariant, GameProfile, GameProfiles, InventoryEntryKind, InventoryKind,
};
use super::{
    Eligibility, FindingKind, GameCandidate, GameError, GameFinding, GameRole, Result, Storefront,
};

/// Direct filesystem entry kind returned by the read-only provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// Direct regular file.
    File,
    /// Direct directory.
    Directory,
    /// Symbolic link or Windows reparse-backed symlink.
    Symlink,
    /// Any other filesystem object.
    Other,
}

/// One direct directory entry with already-inspected type information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    /// Provider-resolved full path.
    pub path: PathBuf,
    /// Direct entry name.
    pub name: OsString,
    /// Entry kind obtained without following links.
    pub kind: FileKind,
}

/// Strictly read-only filesystem operations needed by discovery and probing.
pub trait FileSystemProvider {
    /// Reads one file's bytes.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    /// Lists direct children without following symbolic links.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>>;
    /// Inspects one path without following symbolic links.
    fn file_kind(&self, path: &Path) -> io::Result<FileKind>;
    /// Canonicalizes an existing path without changing it.
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
    /// Reads the PE fixed-file `ProductVersion` as four numeric parts.
    fn product_version(&self, executable: &Path) -> io::Result<String>;
}

/// Host filesystem provider. It exposes no mutation operation.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemFileSystem;

impl FileSystemProvider for SystemFileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>> {
        fs::read_dir(path)?
            .map(|entry| {
                let entry = entry?;
                let entry_path = entry.path();
                let metadata = fs::symlink_metadata(&entry_path)?;
                Ok(DirectoryEntry {
                    path: entry_path,
                    name: entry.file_name(),
                    kind: metadata_kind(&metadata),
                })
            })
            .collect()
    }

    fn file_kind(&self, path: &Path) -> io::Result<FileKind> {
        fs::symlink_metadata(path).map(|metadata| metadata_kind(&metadata))
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        fs::canonicalize(path)
    }

    #[cfg(windows)]
    fn product_version(&self, executable: &Path) -> io::Result<String> {
        let bytes = fs::read(executable)?;
        let image = pelite::PeFile::from_bytes(&bytes).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("not a valid PE image: {error}"),
            )
        })?;
        let resources = image.resources().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("PE resources unavailable: {error}"),
            )
        })?;
        let version = resources.version_info().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("PE version information unavailable: {error}"),
            )
        })?;
        let fixed = version.fixed().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "PE version information has no VS_FIXEDFILEINFO",
            )
        })?;
        Ok(format!(
            "{}.{}.{}.{}",
            fixed.dwProductVersion.Major,
            fixed.dwProductVersion.Minor,
            fixed.dwProductVersion.Patch,
            fixed.dwProductVersion.Build
        ))
    }

    #[cfg(not(windows))]
    fn product_version(&self, _executable: &Path) -> io::Result<String> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Windows PE ProductVersion inspection is unavailable on this platform",
        ))
    }
}

/// Inspects one explicitly browsed role/storefront/path without consulting the registry.
///
/// The storefront is explicit because a raw directory path alone is not reliable store
/// provenance. The provider surface is read-only and no profile is derived from the
/// candidate being inspected.
pub fn inspect_game_path<F: FileSystemProvider>(
    profiles: &GameProfiles,
    fs_provider: &F,
    role: GameRole,
    storefront: Storefront,
    root: &Path,
) -> Result<GameCandidate> {
    let matching = profiles.matching(role, storefront).collect::<Vec<_>>();
    let first = matching
        .first()
        .copied()
        .ok_or(GameError::MissingProfile { role, storefront })?;
    if fs_provider
        .file_kind(root)
        .map_err(|source| GameError::Io {
            path: root.to_path_buf(),
            source,
        })?
        != FileKind::Directory
    {
        return Err(GameError::Io {
            path: root.to_path_buf(),
            source: io::Error::new(
                io::ErrorKind::InvalidInput,
                "source root is not a direct directory",
            ),
        });
    }
    let canonical_root = fs_provider
        .canonicalize(root)
        .map_err(|source| GameError::Io {
            path: root.to_path_buf(),
            source,
        })?;

    let version_path = canonical_root.join(normalized_path(&first.executable));
    let (build, version_error) = match direct_relative_kind(
        fs_provider,
        &canonical_root,
        &first.executable,
        FileKind::File,
    ) {
        Ok(()) => match fs_provider.product_version(&version_path) {
            Ok(version) => (Some(version), None),
            Err(error) => (None, Some(error.to_string())),
        },
        Err(()) => (
            None,
            Some("executable path contains a missing or indirect component".to_owned()),
        ),
    };
    let profile = build
        .as_deref()
        .and_then(|version| {
            matching
                .iter()
                .copied()
                .find(|profile| profile.product_version == version)
        })
        .unwrap_or(first);

    inspect_with_profile(fs_provider, profile, &canonical_root, build, version_error)
}

pub(crate) fn inspect_profile_paths<F: FileSystemProvider>(
    fs_provider: &F,
    profiles: &[&GameProfile],
    root: &Path,
) -> Result<GameCandidate> {
    let profile = profiles.first().copied().ok_or_else(|| GameError::Io {
        path: root.to_path_buf(),
        source: io::Error::new(
            io::ErrorKind::InvalidInput,
            "no profiles supplied for discovered source",
        ),
    })?;
    if fs_provider
        .file_kind(root)
        .map_err(|source| GameError::Io {
            path: root.to_path_buf(),
            source,
        })?
        != FileKind::Directory
    {
        return Err(GameError::Io {
            path: root.to_path_buf(),
            source: io::Error::new(
                io::ErrorKind::InvalidInput,
                "source root is not a direct directory",
            ),
        });
    }
    let canonical_root = fs_provider
        .canonicalize(root)
        .map_err(|source| GameError::Io {
            path: root.to_path_buf(),
            source,
        })?;
    let version_path = canonical_root.join(normalized_path(&profile.executable));
    let (build, version_error) = match direct_relative_kind(
        fs_provider,
        &canonical_root,
        &profile.executable,
        FileKind::File,
    ) {
        Ok(()) => match fs_provider.product_version(&version_path) {
            Ok(version) => (Some(version), None),
            Err(error) => (None, Some(error.to_string())),
        },
        Err(()) => (
            None,
            Some("executable path contains a missing or indirect component".to_owned()),
        ),
    };
    let profile = build
        .as_deref()
        .and_then(|version| {
            profiles
                .iter()
                .copied()
                .find(|profile| profile.product_version == version)
        })
        .unwrap_or(profile);
    inspect_with_profile(fs_provider, profile, &canonical_root, build, version_error)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ObservedEntry {
    kind: InventoryEntryKind,
    sha256: Option<String>,
}

#[derive(Default)]
struct Observation {
    fingerprints: BTreeMap<String, String>,
    inventory: BTreeMap<String, ObservedEntry>,
    complete_fingerprints: bool,
    complete_inventory: bool,
}

fn inspect_with_profile<F: FileSystemProvider>(
    fs_provider: &F,
    profile: &GameProfile,
    root: &Path,
    build: Option<String>,
    version_error: Option<String>,
) -> Result<GameCandidate> {
    let mut findings = Vec::new();
    if build.as_deref() != Some(profile.product_version.as_str()) {
        let message = match (&build, version_error) {
            (Some(found), _) => format!(
                "ProductVersion {found} is unsupported; expected {} for profile {}.",
                profile.product_version, profile.id
            ),
            (None, Some(detail)) => format!(
                "Could not prove ProductVersion {} for profile {}: {detail}.",
                profile.product_version, profile.id
            ),
            (None, None) => format!(
                "Could not prove ProductVersion {} for profile {}.",
                profile.product_version, profile.id
            ),
        };
        findings.push(GameFinding {
            kind: FindingKind::UnsupportedVersion,
            message,
            paths: vec![root.join(normalized_path(&profile.executable))],
        });
    }
    if profile.locale != "en_US" {
        findings.push(GameFinding {
            kind: FindingKind::UnsupportedLocale,
            message: format!(
                "Profile locale {:?} is not the verified alpha locale en_US.",
                profile.locale
            ),
            paths: vec![root.join("lang")],
        });
    }

    let sod_paths = profile
        .sod_files
        .iter()
        .map(|path| path.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut missing_sod = Vec::new();
    let mut missing_required = Vec::new();
    for relative in &profile.required_files {
        let path = root.join(normalized_path(relative));
        match direct_relative_kind(fs_provider, root, relative, FileKind::File) {
            Ok(()) => {}
            Err(()) if sod_paths.contains(&relative.to_ascii_lowercase()) => missing_sod.push(path),
            Err(()) => missing_required.push(path),
        }
    }
    if !missing_sod.is_empty() {
        findings.push(GameFinding {
            kind: FindingKind::MissingSod,
            message: "Required Siege of Dragonspear payload is missing or unsafe.".to_owned(),
            paths: missing_sod,
        });
    }
    if !missing_required.is_empty() {
        findings.push(GameFinding {
            kind: FindingKind::Modified,
            message: "Required source files are missing or are not direct regular files."
                .to_owned(),
            paths: missing_required,
        });
    }

    let (observation, mut scan_paths) = observe_profile(fs_provider, profile, root);
    let residue_findings = scan_forbidden_residue(fs_provider, profile, root);
    for (message, paths) in residue_findings {
        findings.push(GameFinding {
            kind: FindingKind::Modified,
            message,
            paths,
        });
    }

    let fingerprint_variants = matching_fingerprint_variants(profile, &observation);
    if observation.complete_fingerprints && fingerprint_variants.is_empty() {
        findings.push(GameFinding {
            kind: FindingKind::UnknownFingerprint,
            message: format!(
                "Core files do not match any allowed clean variant for profile {}.",
                profile.id
            ),
            paths: profile
                .fingerprint_files
                .iter()
                .map(|path| root.join(normalized_path(path)))
                .collect(),
        });
    }

    let inventory_variants = matching_inventory_variants(profile, &observation);
    if !observation.complete_inventory || inventory_variants.is_empty() {
        if scan_paths.is_empty() {
            scan_paths.extend(
                profile
                    .inventory_surfaces
                    .iter()
                    .map(|surface| root.join(normalized_path(&surface.path))),
            );
        }
        findings.push(GameFinding {
            kind: FindingKind::Modified,
            message: "The mod-sensitive file inventory differs from every allowed clean variant."
                .to_owned(),
            paths: scan_paths,
        });
    }

    let same_variant = fingerprint_variants
        .intersection(&inventory_variants)
        .next()
        .is_some();
    if !same_variant
        && !findings
            .iter()
            .any(|finding| finding.kind == FindingKind::UnknownFingerprint)
        && observation.complete_fingerprints
        && observation.complete_inventory
    {
        findings.push(GameFinding {
            kind: FindingKind::UnknownFingerprint,
            message: "Core fingerprints and inventory match different clean variants.".to_owned(),
            paths: Vec::new(),
        });
    }

    let fingerprint = if observation.complete_fingerprints && observation.complete_inventory {
        Some(observation_digest(profile, build.as_deref(), &observation))
    } else {
        None
    };

    let has_blocker = findings.iter().any(|finding| {
        matches!(
            finding.kind,
            FindingKind::Modified
                | FindingKind::UnsupportedVersion
                | FindingKind::MissingSod
                | FindingKind::UnknownFingerprint
                | FindingKind::UnsupportedLocale
        )
    });
    let eligibility = if has_blocker || !same_variant {
        Eligibility::Ineligible
    } else if profile.storefront == Storefront::Gog {
        findings.push(GameFinding {
            kind: FindingKind::UnverifiedStorefront,
            message: "GOG 2.7.3 remains experimental until a complete clean GOG rehearsal passes."
                .to_owned(),
            paths: Vec::new(),
        });
        Eligibility::Experimental
    } else {
        findings.push(GameFinding {
            kind: FindingKind::Fresh,
            message: format!(
                "Matches verified clean profile {} and one complete allowed variant.",
                profile.id
            ),
            paths: Vec::new(),
        });
        Eligibility::Eligible
    };

    if profile.storefront == Storefront::Gog
        && !findings
            .iter()
            .any(|finding| finding.kind == FindingKind::UnverifiedStorefront)
    {
        findings.push(GameFinding {
            kind: FindingKind::UnverifiedStorefront,
            message: "GOG remains an unverified storefront for this alpha.".to_owned(),
            paths: Vec::new(),
        });
    }

    Ok(GameCandidate {
        role: profile.role,
        storefront: profile.storefront,
        root: root.to_path_buf(),
        build,
        eligibility,
        findings,
        fingerprint,
    })
}

fn observe_profile<F: FileSystemProvider>(
    fs_provider: &F,
    profile: &GameProfile,
    root: &Path,
) -> (Observation, Vec<PathBuf>) {
    let mut observation = Observation {
        complete_fingerprints: true,
        complete_inventory: true,
        ..Observation::default()
    };
    let mut problem_paths = Vec::new();

    for relative in &profile.fingerprint_files {
        let path = root.join(normalized_path(relative));
        match hash_profile_file(fs_provider, root, relative) {
            Ok(digest) => {
                observation
                    .fingerprints
                    .insert(folded_path(relative), digest);
            }
            Err(()) => {
                observation.complete_fingerprints = false;
                problem_paths.push(path);
            }
        }
    }

    for surface in &profile.inventory_surfaces {
        let surface_path = root.join(normalized_path(&surface.path));
        match surface.kind {
            InventoryKind::File => match hash_profile_file(fs_provider, root, &surface.path) {
                Ok(digest) => {
                    observation.inventory.insert(
                        folded_path(&surface.path),
                        ObservedEntry {
                            kind: InventoryEntryKind::File,
                            sha256: Some(digest),
                        },
                    );
                }
                Err(()) => {
                    observation.complete_inventory = false;
                    problem_paths.push(surface_path);
                }
            },
            InventoryKind::Tree => {
                let direct_directory =
                    direct_relative_kind(fs_provider, root, &surface.path, FileKind::Directory)
                        .is_ok();
                if !direct_directory
                    || collect_tree(
                        fs_provider,
                        root,
                        &surface_path,
                        &mut observation.inventory,
                        &mut problem_paths,
                    )
                    .is_err()
                {
                    observation.complete_inventory = false;
                    problem_paths.push(surface_path);
                }
            }
            InventoryKind::Matching => {
                match direct_relative_kind(fs_provider, root, &surface.path, FileKind::Directory)
                    .and_then(|()| sorted_entries(fs_provider, &surface_path))
                {
                    Ok(entries) => {
                        for entry in entries {
                            let name = entry.name.to_string_lossy();
                            let matches_surface = surface
                                .patterns
                                .iter()
                                .any(|pattern| wildcard_match(pattern, &name));
                            if !matches_surface {
                                continue;
                            }
                            if insert_entry(fs_provider, root, &entry, &mut observation.inventory)
                                .is_err()
                            {
                                observation.complete_inventory = false;
                                problem_paths.push(entry.path);
                            }
                        }
                    }
                    Err(()) => {
                        observation.complete_inventory = false;
                        problem_paths.push(surface_path);
                    }
                }
            }
        }
    }
    problem_paths.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
    problem_paths.dedup_by(|left, right| path_eq(left, right));
    (observation, problem_paths)
}

fn collect_tree<F: FileSystemProvider>(
    fs_provider: &F,
    root: &Path,
    directory: &Path,
    inventory: &mut BTreeMap<String, ObservedEntry>,
    problem_paths: &mut Vec<PathBuf>,
) -> std::result::Result<(), ()> {
    let entries = sorted_entries(fs_provider, directory)?;
    let mut complete = true;
    for entry in entries {
        let is_directory = entry.kind == FileKind::Directory;
        if insert_entry(fs_provider, root, &entry, inventory).is_err() {
            problem_paths.push(entry.path.clone());
            complete = false;
            continue;
        }
        if is_directory
            && collect_tree(fs_provider, root, &entry.path, inventory, problem_paths).is_err()
        {
            complete = false;
        }
    }
    if complete {
        Ok(())
    } else {
        Err(())
    }
}

fn insert_entry<F: FileSystemProvider>(
    fs_provider: &F,
    root: &Path,
    entry: &DirectoryEntry,
    inventory: &mut BTreeMap<String, ObservedEntry>,
) -> std::result::Result<(), ()> {
    entry.name.to_str().ok_or(())?;
    let relative = entry.path.strip_prefix(root).map_err(|_| ())?;
    let relative = relative.to_str().ok_or(())?.replace('\\', "/");
    let key = folded_path(&relative);
    if inventory.contains_key(&key) {
        return Err(());
    }
    let observed = match entry.kind {
        FileKind::File => ObservedEntry {
            kind: InventoryEntryKind::File,
            sha256: Some(hash_bytes(&fs_provider.read(&entry.path).map_err(|_| ())?)),
        },
        FileKind::Directory => ObservedEntry {
            kind: InventoryEntryKind::Directory,
            sha256: None,
        },
        FileKind::Symlink | FileKind::Other => return Err(()),
    };
    inventory.insert(key, observed);
    Ok(())
}

fn scan_forbidden_residue<F: FileSystemProvider>(
    fs_provider: &F,
    profile: &GameProfile,
    root: &Path,
) -> Vec<(String, Vec<PathBuf>)> {
    let mut findings = Vec::new();
    for residue in &profile.forbidden_residue_patterns {
        let directory = root.join(normalized_path(&residue.surface));
        let mut entries = Vec::new();
        if direct_relative_kind(fs_provider, root, &residue.surface, FileKind::Directory)
            .and_then(|()| {
                collect_pattern_entries(fs_provider, &directory, residue.recursive, &mut entries)
            })
            .is_err()
        {
            findings.push((
                format!(
                    "Could not completely inspect residue pattern {:?}; source is not fresh.",
                    residue.pattern
                ),
                vec![directory],
            ));
            continue;
        }
        let mut matched = entries
            .into_iter()
            .filter(|entry| {
                let relative = entry
                    .strip_prefix(&directory)
                    .unwrap_or(entry)
                    .to_string_lossy()
                    .replace('\\', "/");
                wildcard_match(&residue.pattern, &relative)
            })
            .collect::<Vec<_>>();
        if !matched.is_empty() {
            matched.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
            findings.push((residue.message.clone(), matched));
        }
    }
    findings
}

fn collect_pattern_entries<F: FileSystemProvider>(
    fs_provider: &F,
    directory: &Path,
    recursive: bool,
    output: &mut Vec<PathBuf>,
) -> std::result::Result<(), ()> {
    let entries = sorted_entries(fs_provider, directory)?;
    for entry in entries {
        output.push(entry.path.clone());
        if recursive && entry.kind == FileKind::Directory {
            collect_pattern_entries(fs_provider, &entry.path, true, output)?;
        }
    }
    Ok(())
}

fn matching_fingerprint_variants(
    profile: &GameProfile,
    observation: &Observation,
) -> BTreeSet<String> {
    if !observation.complete_fingerprints {
        return BTreeSet::new();
    }
    profile
        .allowed_clean_variants
        .iter()
        .filter(|variant| expected_fingerprints(variant) == observation.fingerprints)
        .map(|variant| variant.id.clone())
        .collect()
}

fn matching_inventory_variants(
    profile: &GameProfile,
    observation: &Observation,
) -> BTreeSet<String> {
    if !observation.complete_inventory {
        return BTreeSet::new();
    }
    profile
        .allowed_clean_variants
        .iter()
        .filter(|variant| expected_inventory(variant) == observation.inventory)
        .map(|variant| variant.id.clone())
        .collect()
}

fn expected_fingerprints(variant: &AllowedCleanVariant) -> BTreeMap<String, String> {
    variant
        .fingerprints
        .iter()
        .map(|entry| (folded_path(&entry.path), entry.sha256.clone()))
        .collect()
}

fn expected_inventory(variant: &AllowedCleanVariant) -> BTreeMap<String, ObservedEntry> {
    variant
        .inventory
        .iter()
        .map(|entry| {
            (
                folded_path(&entry.path),
                ObservedEntry {
                    kind: entry.kind,
                    sha256: entry.sha256.clone(),
                },
            )
        })
        .collect()
}

fn observation_digest(
    profile: &GameProfile,
    build: Option<&str>,
    observation: &Observation,
) -> String {
    let mut digest = Sha256::new();
    digest_field(&mut digest, b"profile", profile.id.as_bytes());
    digest_field(
        &mut digest,
        b"build",
        build.unwrap_or("<unknown>").as_bytes(),
    );
    for (path, sha256) in &observation.fingerprints {
        digest_field(&mut digest, b"fingerprint-path", path.as_bytes());
        digest_field(&mut digest, b"fingerprint-sha256", sha256.as_bytes());
    }
    for (path, entry) in &observation.inventory {
        digest_field(&mut digest, b"inventory-path", path.as_bytes());
        digest_field(
            &mut digest,
            b"inventory-kind",
            match entry.kind {
                InventoryEntryKind::File => b"file",
                InventoryEntryKind::Directory => b"directory",
            },
        );
        if let Some(sha256) = &entry.sha256 {
            digest_field(&mut digest, b"inventory-sha256", sha256.as_bytes());
        }
    }
    hex::encode(digest.finalize())
}

fn digest_field(digest: &mut Sha256, label: &[u8], value: &[u8]) {
    digest.update((label.len() as u64).to_le_bytes());
    digest.update(label);
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value);
}

fn hash_profile_file<F: FileSystemProvider>(
    fs_provider: &F,
    root: &Path,
    relative: &str,
) -> std::result::Result<String, ()> {
    direct_relative_kind(fs_provider, root, relative, FileKind::File)?;
    let path = root.join(normalized_path(relative));
    fs_provider
        .read(&path)
        .map(|bytes| hash_bytes(&bytes))
        .map_err(|_| ())
}

fn direct_relative_kind<F: FileSystemProvider>(
    fs_provider: &F,
    root: &Path,
    relative: &str,
    terminal_kind: FileKind,
) -> std::result::Result<(), ()> {
    if relative == "." {
        return if fs_provider.file_kind(root).map_err(|_| ())? == terminal_kind {
            Ok(())
        } else {
            Err(())
        };
    }

    let components = relative.split('/').collect::<Vec<_>>();
    let mut path = root.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        path.push(component);
        let expected = if index + 1 == components.len() {
            terminal_kind
        } else {
            FileKind::Directory
        };
        if fs_provider.file_kind(&path).map_err(|_| ())? != expected {
            return Err(());
        }
    }
    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn sorted_entries<F: FileSystemProvider>(
    fs_provider: &F,
    path: &Path,
) -> std::result::Result<Vec<DirectoryEntry>, ()> {
    let mut entries = fs_provider.read_dir(path).map_err(|_| ())?;
    entries.sort_by_key(|entry| entry.name.to_string_lossy().to_ascii_lowercase());
    Ok(entries)
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern = pattern.to_ascii_lowercase().into_bytes();
    let value = value.to_ascii_lowercase().into_bytes();
    let mut previous = vec![false; value.len() + 1];
    previous[0] = true;
    for token in pattern {
        let mut current = vec![false; value.len() + 1];
        if token == b'*' {
            current[0] = previous[0];
            for index in 1..=value.len() {
                current[index] = previous[index] || current[index - 1];
            }
        } else {
            for index in 1..=value.len() {
                current[index] =
                    previous[index - 1] && (token == b'?' || token == value[index - 1]);
            }
        }
        previous = current;
    }
    previous[value.len()]
}

fn normalized_path(value: &str) -> PathBuf {
    if value == "." {
        PathBuf::new()
    } else {
        value.split('/').collect()
    }
}

fn folded_path(value: &str) -> String {
    value.replace('\\', "/").to_ascii_lowercase()
}

fn path_eq(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

fn metadata_kind(metadata: &fs::Metadata) -> FileKind {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return FileKind::Symlink;
        }
    }
    if metadata.file_type().is_symlink() {
        FileKind::Symlink
    } else if metadata.is_file() {
        FileKind::File
    } else if metadata.is_dir() {
        FileKind::Directory
    } else {
        FileKind::Other
    }
}

#[cfg(test)]
mod tests {
    use super::wildcard_match;

    #[test]
    fn wildcard_matching_is_case_insensitive_and_complete() {
        assert!(wildcard_match("setup-*", "Setup-DlcMerger.exe"));
        assert!(wildcard_match("*.debug.log", "weidu.DEBUG.LOG"));
        assert!(!wildcard_match("weidu.log", "old-weidu.log"));
    }
}
