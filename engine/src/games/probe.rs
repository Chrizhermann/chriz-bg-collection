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
                    path: entry_path.clone(),
                    name: entry.file_name(),
                    kind: metadata_kind(&entry_path, &metadata),
                })
            })
            .collect()
    }

    fn file_kind(&self, path: &Path) -> io::Result<FileKind> {
        fs::symlink_metadata(path).map(|metadata| metadata_kind(path, &metadata))
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        crate::reparse::canonicalize_flexible(path)
    }

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
}

/// The file whose presence shows that Siege of Dragonspear is already merged into a BG:EE
/// folder, as in Beamdog client copies. EET tests the same file to find SoD.
const SOD_MERGED_MARKER: &str = "movies/sodcin05.wbm";

/// The store whose copies the collection is rehearsed with.
const TESTED_STOREFRONT: Storefront = Storefront::Steam;

/// Inspects one explicitly browsed role/storefront/path without consulting the registry.
///
/// The storefront is explicit because a raw directory path alone is not reliable store
/// provenance; [`detect_storefront`] gives the best guess for a browsed folder. When the
/// recipe has no profile for that store, the role's tested profiles are used as reference,
/// and the store difference is reported as a warning. The provider surface is read-only and
/// no profile is derived from the candidate being inspected.
pub fn inspect_game_path<F: FileSystemProvider>(
    profiles: &GameProfiles,
    fs_provider: &F,
    role: GameRole,
    storefront: Storefront,
    root: &Path,
) -> Result<GameCandidate> {
    inspect_game_path_with_language(profiles, fs_provider, role, storefront, root, None)
}

/// [`inspect_game_path`] with the game language a store's records name, such as Steam's
/// `german`; a language other than English is reported as a warning.
pub fn inspect_game_path_with_language<F: FileSystemProvider>(
    profiles: &GameProfiles,
    fs_provider: &F,
    role: GameRole,
    storefront: Storefront,
    root: &Path,
    language: Option<&str>,
) -> Result<GameCandidate> {
    let matching = role_profiles(profiles, role, storefront);
    if matching.is_empty() {
        return Err(GameError::MissingProfile { role, storefront });
    }
    inspect_profile_paths(fs_provider, &matching, root, storefront, language)
}

/// The profiles of `role` for `storefront`, or, when the recipe has none for that store, the
/// role's profiles of every store, the tested Steam ones first.
pub(crate) fn role_profiles(
    profiles: &GameProfiles,
    role: GameRole,
    storefront: Storefront,
) -> Vec<&GameProfile> {
    let mut matching = profiles.matching(role, storefront).collect::<Vec<_>>();
    if matching.is_empty() {
        matching = profiles
            .iter()
            .filter(|profile| profile.role == role)
            .collect();
    }
    matching.sort_by_key(|profile| {
        (
            profile.storefront != Storefront::Steam,
            profile.product_version.clone(),
            profile.id.clone(),
        )
    });
    matching
}

/// The store a browsed game folder most likely comes from, judged from the folder alone: a
/// Microsoft Store or Xbox app folder by its location, a GOG copy by its `goggame-*.info`
/// file, a Steam copy by a `steamapps` folder on its path and a Beamdog copy by a `Beamdog`
/// folder on its path. Anything else is [`Storefront::Other`].
pub fn detect_storefront<F: FileSystemProvider>(fs_provider: &F, root: &Path) -> Storefront {
    if is_protected_store_folder(root) || has_component(root, "xboxgames") {
        return Storefront::Microsoft;
    }
    if let Ok(entries) = fs_provider.read_dir(root) {
        if entries.iter().any(|entry| {
            entry.kind == FileKind::File
                && wildcard_match("goggame-*.info", &entry.name.to_string_lossy())
        }) {
            return Storefront::Gog;
        }
    }
    if has_component(root, "steamapps") {
        return Storefront::Steam;
    }
    if has_component(root, "beamdog") {
        return Storefront::Beamdog;
    }
    Storefront::Other
}

/// Whether `path` lies in the folder where Windows keeps Microsoft Store apps. Windows
/// can protect it. This selects the explanation after a real read failure, never eligibility.
pub(crate) fn is_protected_store_folder(path: &Path) -> bool {
    has_component(path, "windowsapps")
}

fn has_component(path: &Path, name: &str) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(name)
    })
}

/// A candidate that cannot be used because its folder is protected by Windows.
pub(crate) fn protected_candidate(role: GameRole, root: &Path) -> GameCandidate {
    GameCandidate {
        role,
        storefront: Storefront::Microsoft,
        root: root.to_path_buf(),
        build: None,
        eligibility: Eligibility::Ineligible,
        findings: vec![GameFinding {
            kind: FindingKind::ProtectedFolder,
            message: "This copy comes from the Microsoft Store or the Xbox app. Windows protects its folder, so CEBG cannot copy the game from it. Use a Steam or GOG copy of the game instead.".to_owned(),
            paths: vec![root.to_path_buf()],
        }],
        fingerprint: None,
    }
}

/// A candidate whose folder could not be checked at all, for example because reading it was
/// denied. It is listed with the reason instead of hiding it or stopping the search.
pub(crate) fn unreadable_candidate(
    role: GameRole,
    storefront: Storefront,
    root: &Path,
    _error: &GameError,
) -> GameCandidate {
    if is_protected_store_folder(root) {
        return protected_candidate(role, root);
    }
    GameCandidate {
        role,
        storefront,
        root: root.to_path_buf(),
        build: None,
        eligibility: Eligibility::Ineligible,
        findings: vec![GameFinding {
            kind: FindingKind::Unreadable,
            message:
                "CEBG cannot read this game folder. Check that it can be read, then try again."
                    .to_owned(),
            paths: vec![root.to_path_buf()],
        }],
        fingerprint: None,
    }
}

/// Root entry patterns that mark leftovers of earlier mod installs, such as `WeiDU.log`,
/// `setup-*` programs and mod folders. A clean game has none of them; the copy of a source
/// game leaves them out.
pub fn left_out_root_patterns(profile: &GameProfile) -> Vec<String> {
    let mut patterns = Vec::<String>::new();
    let surfaces = profile
        .inventory_surfaces
        .iter()
        .filter(|surface| surface.kind == InventoryKind::Matching && surface.path == ".")
        .flat_map(|surface| surface.patterns.iter());
    let residue = profile
        .forbidden_residue_patterns
        .iter()
        .filter(|residue| residue.surface == "." && !residue.recursive)
        .map(|residue| &residue.pattern);
    for pattern in surfaces.chain(residue) {
        if !patterns
            .iter()
            .any(|known| known.eq_ignore_ascii_case(pattern))
        {
            patterns.push(pattern.clone());
        }
    }
    patterns
}

fn is_left_out(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|pattern| wildcard_match(pattern, name))
}

/// Whether `name` is a known mod, installer, uninstaller or overlay helper rather than the
/// game executable itself.
pub(crate) fn is_non_game_root_executable(name: &str) -> bool {
    const NON_GAME_PATTERNS: &[&str] = &[
        "setup-*",
        "weidu.exe",
        "infinityloader.exe",
        "oalinst.exe",
        "unins*",
        "*uninstall*",
        "dlcmerger*",
        "galaxy*",
        "crash*",
        "bg radar overlay.exe",
        "bgradaroverlay.exe",
        "vcredist*",
        "dxsetup.exe",
    ];
    NON_GAME_PATTERNS
        .iter()
        .any(|pattern| wildcard_match(pattern, name))
}

/// Resolves the game executable relative path in `root` for `role`.
///
/// Checks `preferred` first, then [`GameRole::known_executables`], and finally any unambiguous
/// direct regular `.exe` in `root` that is not a mod, installer or overlay helper. Returns
/// `None` when `preferred` or a matched known executable is a link/non-file, or when no game
/// executable exists.
pub fn resolve_game_executable<F: FileSystemProvider>(
    fs_provider: &F,
    root: &Path,
    role: GameRole,
    preferred: &str,
) -> Option<String> {
    match direct_relative_kind_or_missing(fs_provider, root, preferred, FileKind::File) {
        Ok(true) => return Some(preferred.to_owned()),
        Err(()) => return None,
        Ok(false) => {}
    }
    let entries = sorted_entries(fs_provider, root).ok()?;
    for known in role.known_executables() {
        if known.eq_ignore_ascii_case(preferred) {
            continue;
        }
        if let Some(entry) = entries.iter().find(|entry| {
            entry
                .name
                .to_str()
                .is_some_and(|name| name.eq_ignore_ascii_case(known))
        }) {
            return (entry.kind == FileKind::File)
                .then(|| entry.name.to_str().map(str::to_owned))
                .flatten();
        }
    }
    let candidates = entries
        .iter()
        .filter(|entry| entry.kind == FileKind::File)
        .filter_map(|entry| {
            let name = entry.name.to_str()?;
            let lower = name.to_ascii_lowercase();
            (lower.ends_with(".exe") && !is_non_game_root_executable(name)).then_some((entry, name))
        })
        .collect::<Vec<_>>();
    if let Some((_, name)) = candidates
        .iter()
        .find(|(entry, _)| fs_provider.product_version(&entry.path).is_ok())
    {
        return Some((*name).to_owned());
    }
    if candidates.len() == 1 {
        return Some(candidates[0].1.to_owned());
    }
    None
}

fn has_native_linux_game_markers<F: FileSystemProvider>(fs_provider: &F, root: &Path) -> bool {
    const NATIVE_LINUX_MARKERS: &[&str] = &[
        "BaldursGate",
        "BaldursGateII",
        "IcewindDale",
        "BaldursGate64",
        "BaldursGateII64",
        "IcewindDale64",
        "start.sh",
    ];
    NATIVE_LINUX_MARKERS.iter().any(|marker| {
        direct_relative_kind_or_missing(fs_provider, root, marker, FileKind::File) == Ok(true)
            || direct_relative_kind_or_missing(
                fs_provider,
                root,
                &format!("game/{marker}"),
                FileKind::File,
            ) == Ok(true)
    })
}

fn has_direct_bif_archive<F: FileSystemProvider>(fs_provider: &F, root: &Path) -> bool {
    if direct_relative_kind(fs_provider, root, "data", FileKind::Directory).is_err() {
        return false;
    }
    sorted_entries(fs_provider, &root.join("data")).is_ok_and(|entries| {
        entries.iter().any(|entry| {
            entry.kind == FileKind::File
                && entry
                    .name
                    .to_str()
                    .is_some_and(|name| name.to_ascii_lowercase().ends_with(".bif"))
        })
    })
}

fn is_data_bif_path(relative: &str) -> bool {
    let folded = folded_path(relative);
    folded.starts_with("data/")
        && !folded["data/".len()..].contains('/')
        && folded.ends_with(".bif")
}

pub(crate) fn inspect_profile_paths<F: FileSystemProvider>(
    fs_provider: &F,
    profiles: &[&GameProfile],
    root: &Path,
    storefront: Storefront,
    language: Option<&str>,
) -> Result<GameCandidate> {
    let profile = profiles.first().copied().ok_or_else(|| GameError::Io {
        path: root.to_path_buf(),
        source: io::Error::new(
            io::ErrorKind::InvalidInput,
            "no profiles supplied for discovered source",
        ),
    })?;
    // Try the source itself. A folder name or storefront says nothing about read access.
    if let Err(source) = fs_provider.read_dir(root) {
        if matches!(
            source.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
        ) {
            return Err(GameError::Io {
                path: root.to_path_buf(),
                source,
            });
        }
        return Ok(unreadable_candidate(
            profile.role,
            storefront,
            root,
            &GameError::Io {
                path: root.to_path_buf(),
                source,
            },
        ));
    }
    let root_kind = fs_provider
        .file_kind(root)
        .map_err(|source| GameError::Io {
            path: root.to_path_buf(),
            source,
        })?;
    if !matches!(root_kind, FileKind::Directory | FileKind::Symlink) {
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
    if root_kind == FileKind::Symlink
        && fs_provider
            .file_kind(&canonical_root)
            .map_err(|source| GameError::Io {
                path: canonical_root.clone(),
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
    let resolved_executable = resolve_game_executable(
        fs_provider,
        &canonical_root,
        profile.role,
        &profile.executable,
    );
    let (build, version_error) = match &resolved_executable {
        Some(executable) => {
            let version_path = canonical_root.join(normalized_path(executable));
            match fs_provider.product_version(&version_path) {
                Ok(version) => (Some(version), None),
                Err(error) => (None, Some(error.to_string())),
            }
        }
        None => (
            None,
            Some("the game program is missing or is a link".to_owned()),
        ),
    };
    let same_version = |candidate: &&GameProfile| {
        build
            .as_deref()
            .is_some_and(|version| candidate.product_version == version)
    };
    let profile = profiles
        .iter()
        .copied()
        .find(|candidate| same_version(candidate) && candidate.storefront == storefront)
        .or_else(|| profiles.iter().copied().find(same_version))
        .unwrap_or(profile);
    let resolved_executable = if resolved_executable.is_some() {
        resolved_executable
    } else {
        resolve_game_executable(
            fs_provider,
            &canonical_root,
            profile.role,
            &profile.executable,
        )
    };
    inspect_with_profile(
        fs_provider,
        profile,
        &canonical_root,
        Source {
            build,
            version_error,
            resolved_executable,
            storefront,
            language,
        },
    )
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
    /// Profiled paths that are absent although the profile expects them, slash-folded.
    missing: BTreeSet<String>,
    /// Profiled paths that could not be read, or that are links or of the wrong kind.
    unreadable: Vec<PathBuf>,
}

/// What is known about a source besides its files.
struct Source<'a> {
    build: Option<String>,
    version_error: Option<String>,
    resolved_executable: Option<String>,
    storefront: Storefront,
    language: Option<&'a str>,
}

fn inspect_with_profile<F: FileSystemProvider>(
    fs_provider: &F,
    profile: &GameProfile,
    root: &Path,
    source: Source<'_>,
) -> Result<GameCandidate> {
    let Source {
        build,
        version_error,
        resolved_executable,
        storefront,
        language,
    } = source;
    let effective_executable = resolved_executable
        .as_deref()
        .unwrap_or(profile.executable.as_str());
    let mut findings = Vec::new();
    let tested_build = build.as_deref() == Some(profile.product_version.as_str());
    if !tested_build {
        let tested = &profile.product_version;
        let message = match (&build, version_error) {
            (Some(found), _) => format!(
                "Game version {found}. The collection is tested with version {tested}; other versions install too, but some mods may not work as expected."
            ),
            (None, Some(detail)) => format!(
                "Could not read the game version ({detail}). The collection is tested with version {tested}."
            ),
            (None, None) => format!(
                "Could not read the game version. The collection is tested with version {tested}."
            ),
        };
        findings.push(GameFinding {
            kind: FindingKind::UnsupportedVersion,
            message,
            paths: vec![root.join(normalized_path(effective_executable))],
        });
    }
    let other_language = language.filter(|language| !is_english(language));
    if let Some(language) = other_language {
        findings.push(GameFinding {
            kind: FindingKind::UnsupportedLocale,
            message: format!(
                "This game is set to {}. The collection installs English texts, so set the game's language to English before you play; otherwise texts that mods add may be missing.",
                language_name(language)
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
        let check_relative = if relative.eq_ignore_ascii_case(&profile.executable) {
            effective_executable
        } else {
            relative.as_str()
        };
        let path = root.join(normalized_path(relative));
        match direct_relative_kind_or_missing(fs_provider, root, check_relative, FileKind::File) {
            Ok(true) => {}
            Ok(false)
                if !sod_paths.contains(&relative.to_ascii_lowercase())
                    && is_data_bif_path(relative)
                    && has_direct_bif_archive(fs_provider, root) => {}
            Ok(false) | Err(()) if sod_paths.contains(&relative.to_ascii_lowercase()) => {
                missing_sod.push(path);
            }
            Ok(false) | Err(()) => missing_required.push(relative.clone()),
        }
    }
    let sod_merged = !missing_sod.is_empty()
        && profile.role == GameRole::BgeeSod
        && direct_relative_kind(fs_provider, root, SOD_MERGED_MARKER, FileKind::File).is_ok();
    if sod_merged {
        findings.push(GameFinding {
            kind: FindingKind::SodMerged,
            message: "Siege of Dragonspear is already merged into this game folder, as in copies from the Beamdog client. The collection skips its DLC Merger step for this copy.".to_owned(),
            paths: vec![root.join(normalized_path(SOD_MERGED_MARKER))],
        });
    } else if !missing_sod.is_empty() {
        findings.push(GameFinding {
            kind: FindingKind::MissingSod,
            message: "Siege of Dragonspear is missing. The collection needs Baldur's Gate: Enhanced Edition with Siege of Dragonspear installed.".to_owned(),
            paths: missing_sod,
        });
    }
    if !missing_required.is_empty() {
        let missing_executable = missing_required
            .iter()
            .any(|path| path.eq_ignore_ascii_case(&profile.executable));
        let english = missing_required
            .iter()
            .any(|path| path.eq_ignore_ascii_case("lang/en_US/dialog.tlk"));
        let message = if missing_executable && has_native_linux_game_markers(fs_provider, root) {
            format!(
                "This folder holds the native Linux build instead of the Windows build that EEex and InfinityLoader need ({}). Switch this game to the Windows version run via Proton or Wine (Steam: Properties -> Compatibility -> Force the use of a specific Steam Play compatibility tool (Proton); Heroic/GOG: install the Windows build), then check again.",
                missing_required.join(", ")
            )
        } else if english {
            format!(
                "The English game texts are missing ({}). The collection installs English texts and needs them. Repair or reinstall the game with English included, then check again.",
                missing_required.join(", ")
            )
        } else {
            format!(
                "Game files that every installation needs are missing or are links: {}. Repair or reinstall the game, for example with your store's verify or repair option, then check again.",
                missing_required.join(", ")
            )
        };
        findings.push(GameFinding {
            kind: FindingKind::MissingFiles,
            message,
            paths: missing_required
                .iter()
                .map(|relative| root.join(normalized_path(relative)))
                .collect(),
        });
    }

    let left_out_patterns = left_out_root_patterns(profile);
    let observation = observe_profile(
        fs_provider,
        profile,
        root,
        &left_out_patterns,
        effective_executable,
    );
    findings.extend(left_out_findings(
        fs_provider,
        profile,
        root,
        &left_out_patterns,
    ));
    for (message, paths) in scan_forbidden_residue(fs_provider, profile, root) {
        findings.push(GameFinding {
            kind: FindingKind::Modified,
            message,
            paths,
        });
    }
    if !observation.unreadable.is_empty() {
        if is_protected_store_folder(root) {
            return Ok(protected_candidate(profile.role, root));
        }
        let names = observation
            .unreadable
            .iter()
            .map(|path| {
                path.strip_prefix(root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect::<Vec<_>>();
        findings.push(GameFinding {
            kind: FindingKind::Unreadable,
            message: format!(
                "Some game files could not be read or are links: {}. Close programs that use the game, check that the folder can be read, then check again.",
                names.join(", ")
            ),
            paths: observation.unreadable.clone(),
        });
    }

    // What the copy installs from: every observed entry except the leftovers it leaves out.
    let copied_inventory = observation
        .inventory
        .iter()
        .filter(|(path, _)| path.contains('/') || !is_left_out(&left_out_patterns, path))
        .map(|(path, entry)| (path.clone(), entry.clone()))
        .collect::<BTreeMap<_, _>>();
    let fingerprint_variants = matching_fingerprint_variants(profile, &observation);
    let inventory_variants = matching_inventory_variants(profile, &observation, &copied_inventory);
    let same_variant = fingerprint_variants
        .intersection(&inventory_variants)
        .next()
        .is_some();
    // The profile knows this store's clean files: its own store, or a folder whose store
    // cannot be told. Only Steam copies are the tested setup; every other store installs
    // with a warning, even when its files match its own profile.
    let reference_store = storefront == profile.storefront || storefront == Storefront::Other;
    let blocked = findings
        .iter()
        .any(|finding| finding.kind.blocks_installation());
    let tested = reference_store
        && profile.storefront == TESTED_STOREFRONT
        && tested_build
        && same_variant
        && other_language.is_none()
        && !sod_merged
        && !blocked;
    let storefront = if tested && storefront == Storefront::Other {
        profile.storefront
    } else {
        storefront
    };

    if !same_variant && !blocked && reference_store && tested_build {
        if !observation.complete_fingerprints || fingerprint_variants.is_empty() {
            findings.push(GameFinding {
                kind: FindingKind::UnknownFingerprint,
                message: format!(
                    "The game's program or data files differ from a clean version {} game. The installation continues, but it may fail or behave differently.",
                    profile.product_version
                ),
                paths: profile
                    .fingerprint_files
                    .iter()
                    .map(|path| {
                        let actual = if path.eq_ignore_ascii_case(&profile.executable) {
                            effective_executable
                        } else {
                            path.as_str()
                        };
                        root.join(normalized_path(actual))
                    })
                    .collect(),
            });
        }
        if observation.complete_inventory && inventory_variants.is_empty() {
            findings.push(GameFinding {
                kind: FindingKind::Modified,
                message: "Files that mods change (chitin.key, dialog.tlk, engine.lua or the override folder) differ from a clean game, so mods were probably installed into this folder. The installation continues, but it may fail or behave differently. For the best result, restore a clean game first, for example with your store's verify or repair option.".to_owned(),
                paths: profile
                    .inventory_surfaces
                    .iter()
                    .filter(|surface| surface.kind != InventoryKind::Matching)
                    .map(|surface| root.join(normalized_path(&surface.path)))
                    .collect(),
            });
        }
        if !fingerprint_variants.is_empty() && !inventory_variants.is_empty() {
            findings.push(GameFinding {
                kind: FindingKind::UnknownFingerprint,
                message: "The game's files mix two different clean game versions. The installation continues, but it may fail or behave differently.".to_owned(),
                paths: Vec::new(),
            });
        }
    } else if !same_variant && !blocked {
        // Another store or version: its clean files are not known, so only what a clean game
        // never has is worth a word.
        let override_files = copied_inventory
            .iter()
            .filter(|(path, entry)| {
                path.starts_with("override/") && entry.kind == InventoryEntryKind::File
            })
            .count();
        if override_files > 0 {
            findings.push(GameFinding {
                kind: FindingKind::Modified,
                message: format!(
                    "The override folder holds {override_files} {}. A clean game has none there, so they may be left over from mods. They are copied as they are; if the installation fails, try a clean game.",
                    if override_files == 1 { "file" } else { "files" }
                ),
                paths: vec![root.join("override")],
            });
        }
    }

    let store_note = match storefront {
        Storefront::Steam => None,
        Storefront::Gog => Some("GOG copies are less tested than Steam copies. CEBG runs the same checks; if something does not work, please report it."),
        Storefront::Beamdog => Some("Copies from the Beamdog client are less tested than Steam copies. CEBG runs the same checks; if something does not work, please report it."),
        Storefront::Microsoft => Some("Microsoft Store and Xbox app copies were never tested with the collection. CEBG runs the same checks; if something does not work, please report it."),
        Storefront::Other => Some("CEBG could not tell which store this game comes from, and it differs from the tested Steam game. CEBG runs the same checks; if something does not work, please report it."),
    };
    if let Some(message) = store_note.filter(|_| !blocked) {
        findings.push(GameFinding {
            kind: FindingKind::UnverifiedStorefront,
            message: message.to_owned(),
            paths: Vec::new(),
        });
    }

    let fingerprint = observation
        .unreadable
        .is_empty()
        .then(|| observation_digest(profile, build.as_deref(), &observation));
    let eligibility = if blocked || fingerprint.is_none() {
        Eligibility::Ineligible
    } else if tested {
        findings.push(GameFinding {
            kind: FindingKind::Fresh,
            message: format!(
                "Clean {} game, version {}: the tested setup.",
                storefront.display_name(),
                profile.product_version
            ),
            paths: Vec::new(),
        });
        Eligibility::Eligible
    } else {
        Eligibility::Experimental
    };

    Ok(GameCandidate {
        role: profile.role,
        storefront,
        root: root.to_path_buf(),
        build,
        eligibility,
        findings,
        fingerprint,
    })
}

/// Whether a store's language name or code means English.
fn is_english(language: &str) -> bool {
    let language = language.trim().to_ascii_lowercase();
    language.is_empty()
        || language == "english"
        || language == "en"
        || language.starts_with("en_")
        || language.starts_with("en-")
}

/// A store's language name as a player reads it: `german` becomes `German`.
fn language_name(language: &str) -> String {
    let language = language.trim();
    let mut characters = language.chars();
    match characters.next() {
        Some(first)
            if language
                .chars()
                .all(|character| character.is_ascii_lowercase()) =>
        {
            first.to_ascii_uppercase().to_string() + characters.as_str()
        }
        _ => language.to_owned(),
    }
}

/// The warnings for leftovers of earlier mod installs that the copy leaves out: the profile's
/// own words for the leftovers it names, and one finding for the rest.
fn left_out_findings<F: FileSystemProvider>(
    fs_provider: &F,
    profile: &GameProfile,
    root: &Path,
    patterns: &[String],
) -> Vec<GameFinding> {
    let Ok(entries) = sorted_entries(fs_provider, root) else {
        // The root itself could not be listed; the observation reports that.
        return Vec::new();
    };
    let mut findings = Vec::new();
    let mut named = BTreeSet::new();
    for residue in profile
        .forbidden_residue_patterns
        .iter()
        .filter(|residue| residue.surface == "." && !residue.recursive)
    {
        let matched = entries
            .iter()
            .filter(|entry| wildcard_match(&residue.pattern, &entry.name.to_string_lossy()))
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();
        if !matched.is_empty() {
            named.extend(matched.iter().cloned());
            findings.push(GameFinding {
                kind: FindingKind::LeftOut,
                message: format!("{} The copy leaves them out.", residue.message.trim_end()),
                paths: matched,
            });
        }
    }
    let rest = entries
        .iter()
        .filter(|entry| {
            is_left_out(patterns, &entry.name.to_string_lossy()) && !named.contains(&entry.path)
        })
        .collect::<Vec<_>>();
    if !rest.is_empty() {
        findings.push(GameFinding {
            kind: FindingKind::LeftOut,
            message: format!(
                "Leftovers of earlier mod installs are in this folder ({}). The copy leaves them out, so they do not affect the new installation.",
                rest.iter()
                    .map(|entry| entry.name.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            paths: rest.iter().map(|entry| entry.path.clone()).collect(),
        });
    }
    findings
}

fn observe_profile<F: FileSystemProvider>(
    fs_provider: &F,
    profile: &GameProfile,
    root: &Path,
    left_out_patterns: &[String],
    effective_executable: &str,
) -> Observation {
    let mut observation = Observation {
        complete_fingerprints: true,
        complete_inventory: true,
        ..Observation::default()
    };

    for relative in &profile.fingerprint_files {
        let actual_relative = if relative.eq_ignore_ascii_case(&profile.executable) {
            effective_executable
        } else {
            relative.as_str()
        };
        let path = root.join(normalized_path(actual_relative));
        match direct_relative_kind_or_missing(fs_provider, root, actual_relative, FileKind::File) {
            Ok(true) => match fs_provider.read(&path) {
                Ok(bytes) => {
                    observation
                        .fingerprints
                        .insert(folded_path(actual_relative), hash_bytes(&bytes));
                }
                Err(_) => {
                    observation.complete_fingerprints = false;
                    observation.unreadable.push(path);
                }
            },
            Ok(false) => {
                observation.complete_fingerprints = false;
                observation.missing.insert(folded_path(actual_relative));
            }
            Err(()) => {
                observation.complete_fingerprints = false;
                observation.unreadable.push(path);
            }
        }
    }

    for surface in &profile.inventory_surfaces {
        let surface_path = root.join(normalized_path(&surface.path));
        match surface.kind {
            InventoryKind::File => match direct_relative_kind_or_missing(
                fs_provider,
                root,
                &surface.path,
                FileKind::File,
            ) {
                Ok(false) if surface.allow_missing => {}
                Ok(true) => match fs_provider.read(&surface_path) {
                    Ok(bytes) => {
                        observation.inventory.insert(
                            folded_path(&surface.path),
                            ObservedEntry {
                                kind: InventoryEntryKind::File,
                                sha256: Some(hash_bytes(&bytes)),
                            },
                        );
                    }
                    Err(_) => {
                        observation.complete_inventory = false;
                        observation.unreadable.push(surface_path);
                    }
                },
                Ok(false) => {
                    observation.complete_inventory = false;
                    observation.missing.insert(folded_path(&surface.path));
                }
                Err(()) => {
                    observation.complete_inventory = false;
                    observation.unreadable.push(surface_path);
                }
            },
            InventoryKind::Tree => match direct_relative_kind_or_missing(
                fs_provider,
                root,
                &surface.path,
                FileKind::Directory,
            ) {
                Ok(false) if surface.allow_missing => {}
                Ok(true) => {
                    let mut problems = Vec::new();
                    let listed = collect_tree(
                        fs_provider,
                        root,
                        &surface_path,
                        &mut observation.inventory,
                        &mut problems,
                    );
                    if listed.is_err() {
                        observation.complete_inventory = false;
                        if problems.is_empty() {
                            problems.push(surface_path);
                        }
                        observation.unreadable.extend(problems);
                    }
                }
                Ok(false) => {
                    observation.complete_inventory = false;
                    observation.missing.insert(folded_path(&surface.path));
                }
                Err(()) => {
                    observation.complete_inventory = false;
                    observation.unreadable.push(surface_path);
                }
            },
            InventoryKind::Matching => {
                match direct_relative_kind(fs_provider, root, &surface.path, FileKind::Directory)
                    .and_then(|()| sorted_entries(fs_provider, &surface_path))
                {
                    Ok(entries) => {
                        for entry in entries {
                            if crate::reparse::is_benign_os_metadata_filename(&entry.name)
                                && entry.kind == FileKind::File
                            {
                                continue;
                            }
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
                                // A leftover the copy leaves out never needs to be read.
                                if surface.path == "." && is_left_out(left_out_patterns, &name) {
                                    continue;
                                }
                                observation.complete_inventory = false;
                                observation.unreadable.push(entry.path);
                            }
                        }
                    }
                    Err(()) => {
                        observation.complete_inventory = false;
                        observation.unreadable.push(surface_path);
                    }
                }
            }
        }
    }
    observation
        .unreadable
        .sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
    observation
        .unreadable
        .dedup_by(|left, right| path_eq(left, right));
    observation
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
        if crate::reparse::is_benign_os_metadata_filename(&entry.name)
            && entry.kind == FileKind::File
        {
            continue;
        }
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
    // Root leftovers are left out of the copy and reported by `left_out_findings`; only
    // residue elsewhere, which the copy keeps, is reported here.
    for residue in profile
        .forbidden_residue_patterns
        .iter()
        .filter(|residue| residue.surface != "." || residue.recursive)
    {
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
                    "Could not completely check {} for leftovers of earlier mod installs ({:?}).",
                    residue.surface, residue.pattern
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
            findings.push((
                format!(
                    "{} The installation continues, but they may affect it.",
                    residue.message.trim_end()
                ),
                matched,
            ));
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
    copied_inventory: &BTreeMap<String, ObservedEntry>,
) -> BTreeSet<String> {
    if !observation.complete_inventory {
        return BTreeSet::new();
    }
    profile
        .allowed_clean_variants
        .iter()
        .filter(|variant| expected_inventory(variant) == *copied_inventory)
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
    // Absent profiled files are part of the identity, so that one appearing later is a
    // change. A source without any keeps the digest it always had.
    for path in &observation.missing {
        digest_field(&mut digest, b"missing-path", path.as_bytes());
    }
    hex::encode(digest.finalize())
}

fn digest_field(digest: &mut Sha256, label: &[u8], value: &[u8]) {
    digest.update((label.len() as u64).to_le_bytes());
    digest.update(label);
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value);
}

fn direct_relative_kind<F: FileSystemProvider>(
    fs_provider: &F,
    root: &Path,
    relative: &str,
    terminal_kind: FileKind,
) -> std::result::Result<(), ()> {
    if direct_relative_kind_or_missing(fs_provider, root, relative, terminal_kind)? {
        Ok(())
    } else {
        Err(())
    }
}

fn direct_relative_kind_or_missing<F: FileSystemProvider>(
    fs_provider: &F,
    root: &Path,
    relative: &str,
    terminal_kind: FileKind,
) -> std::result::Result<bool, ()> {
    if relative == "." {
        return fs_provider
            .file_kind(root)
            .map(|kind| kind == terminal_kind)
            .map_err(|_| ());
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
        match fs_provider.file_kind(&path) {
            Ok(kind) if kind == expected => {}
            Ok(_) => return Err(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(_) => return Err(()),
        }
    }
    Ok(true)
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

pub(crate) fn wildcard_match(pattern: &str, value: &str) -> bool {
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

fn metadata_kind(path: &Path, metadata: &fs::Metadata) -> FileKind {
    if crate::reparse::metadata_is_link(path, metadata) {
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
    use super::*;
    use crate::games::{Eligibility, GameProfiles, GameRole, Storefront};

    fn write_u16_le(buf: &mut [u8], offset: usize, value: u16) {
        buf[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32_le(buf: &mut [u8], offset: usize, value: u32) {
        buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn minimal_pe_with_product_version(major: u16, minor: u16, patch: u16, build: u16) -> Vec<u8> {
        let mut pe = vec![0_u8; 0x400];
        pe[0..2].copy_from_slice(b"MZ");
        write_u32_le(&mut pe, 0x3C, 0x40);
        pe[0x40..0x44].copy_from_slice(b"PE\0\0");

        // COFF header at 0x44
        write_u16_le(&mut pe, 0x44, 0x8664); // AMD64
        write_u16_le(&mut pe, 0x46, 1); // NumberOfSections
        write_u16_le(&mut pe, 0x54, 0xF0); // SizeOfOptionalHeader
        write_u16_le(&mut pe, 0x56, 0x0022); // Characteristics

        // Optional header PE32+ at 0x58
        write_u16_le(&mut pe, 0x58, 0x020B);
        write_u32_le(&mut pe, 0x78, 0x1000); // SectionAlignment
        write_u32_le(&mut pe, 0x7C, 0x200); // FileAlignment
        write_u32_le(&mut pe, 0x90, 0x2000); // SizeOfImage
        write_u32_le(&mut pe, 0x94, 0x200); // SizeOfHeaders
        write_u32_le(&mut pe, 0xC4, 16); // NumberOfRvaAndSizes
                                         // DataDirectory[2] = IMAGE_DIRECTORY_ENTRY_RESOURCE at 0x58 + 112 + 16 = 0xD8
        write_u32_le(&mut pe, 0xD8, 0x1000);
        write_u32_le(&mut pe, 0xDC, 0x200);

        // .rsrc section header at 0x148
        pe[0x148..0x150].copy_from_slice(b".rsrc\0\0\0");
        write_u32_le(&mut pe, 0x150, 0x200); // VirtualSize
        write_u32_le(&mut pe, 0x154, 0x1000); // VirtualAddress
        write_u32_le(&mut pe, 0x158, 0x200); // SizeOfRawData
        write_u32_le(&mut pe, 0x15C, 0x200); // PointerToRawData
        write_u32_le(&mut pe, 0x16C, 0x4000_0040);

        // .rsrc payload at file offset 0x200 (RVA 0x1000)
        let rsrc = 0x200;
        write_u16_le(&mut pe, rsrc + 0x0E, 1); // root: 1 id entry
        write_u32_le(&mut pe, rsrc + 0x10, 16); // RT_VERSION
        write_u32_le(&mut pe, rsrc + 0x14, 0x8000_0018);

        write_u16_le(&mut pe, rsrc + 0x18 + 0x0E, 1); // id dir
        write_u32_le(&mut pe, rsrc + 0x28, 1); // id = 1
        write_u32_le(&mut pe, rsrc + 0x2C, 0x8000_0030);

        write_u16_le(&mut pe, rsrc + 0x30 + 0x0E, 1); // lang dir
        write_u32_le(&mut pe, rsrc + 0x40, 0x0409);
        write_u32_le(&mut pe, rsrc + 0x44, 0x48);

        // IMAGE_RESOURCE_DATA_ENTRY at rsrc + 0x48
        write_u32_le(&mut pe, rsrc + 0x48, 0x1058); // RVA of VS_VERSION_INFO
        write_u32_le(&mut pe, rsrc + 0x4C, 0x5C); // Size = 92 bytes

        // VS_VERSION_INFO at rsrc + 0x58
        let ver = rsrc + 0x58;
        write_u16_le(&mut pe, ver, 0x5C); // wLength
        write_u16_le(&mut pe, ver + 2, 52); // wValueLength
        write_u16_le(&mut pe, ver + 4, 0); // wType (binary)
        for (i, unit) in "VS_VERSION_INFO\0".encode_utf16().enumerate() {
            write_u16_le(&mut pe, ver + 6 + i * 2, unit);
        }
        let fixed = rsrc + 0x80;
        write_u32_le(&mut pe, fixed, 0xFEEF_04BD); // dwSignature
        write_u32_le(&mut pe, fixed + 4, 0x0001_0000); // dwStrucVersion
        write_u32_le(
            &mut pe,
            fixed + 16,
            (u32::from(major) << 16) | u32::from(minor),
        );
        write_u32_le(
            &mut pe,
            fixed + 20,
            (u32::from(patch) << 16) | u32::from(build),
        );
        pe
    }

    #[test]
    fn wildcard_matching_is_case_insensitive_and_complete() {
        assert!(wildcard_match("setup-*", "Setup-DlcMerger.exe"));
        assert!(wildcard_match("*.debug.log", "weidu.DEBUG.LOG"));
        assert!(!wildcard_match("weidu.log", "old-weidu.log"));
    }

    #[test]
    fn system_product_version_reads_pe_fixed_file_info_cross_platform() {
        let temp = tempfile::tempdir().unwrap();
        let exe = temp.path().join("Baldur.exe");
        fs::write(&exe, minimal_pe_with_product_version(2, 7, 3, 0)).unwrap();
        assert_eq!(SystemFileSystem.product_version(&exe).unwrap(), "2.7.3.0");

        let non_pe = temp.path().join("start.sh");
        fs::write(&non_pe, b"#!/bin/sh\n").unwrap();
        assert!(SystemFileSystem.product_version(&non_pe).is_err());
    }

    #[test]
    fn native_linux_game_build_reports_proton_and_windows_build_guidance() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Baldurs Gate II Enhanced Edition");
        fs::create_dir_all(root.join("lang/en_US")).unwrap();
        fs::create_dir_all(root.join("data")).unwrap();
        fs::write(root.join("BaldursGateII64"), b"\x7fELF").unwrap();
        fs::write(root.join("chitin.key"), b"key").unwrap();
        fs::write(root.join("engine.lua"), b"engine_name = \"BG2\"").unwrap();
        fs::write(root.join("lang/en_US/dialog.tlk"), b"tlk").unwrap();
        fs::write(root.join("data/DEFAULT.BIF"), b"bif").unwrap();

        let profiles = GameProfiles::load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../manifest/game-builds"),
        )
        .unwrap();
        let candidate = inspect_game_path(
            &profiles,
            &SystemFileSystem,
            GameRole::Bg2ee,
            Storefront::Steam,
            &root,
        )
        .unwrap();
        assert_eq!(candidate.eligibility, Eligibility::Ineligible);
        let finding = candidate
            .findings
            .iter()
            .find(|f| f.kind == FindingKind::MissingFiles)
            .expect("expected MissingFiles finding");
        assert!(finding.message.contains("native Linux build"));
        assert!(finding.message.contains("Proton"));
        assert!(finding.message.contains("Windows build"));
    }
}
