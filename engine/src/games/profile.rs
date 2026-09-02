//! Storefront/build-specific clean-source profiles.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{GameError, GameRole, Result, Storefront};

const PROFILE_SCHEMA: u32 = 1;
const VERIFIED_ALPHA_LOCALE: &str = "en_US";

/// One independently authored source-game profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameProfile {
    /// Profile schema version.
    pub schema: u32,
    /// Stable profile identity.
    pub id: String,
    /// Logical EET source role.
    pub role: GameRole,
    /// Storefront whose layout and bytes were captured.
    pub storefront: Storefront,
    /// Verified installed locale.
    pub locale: String,
    /// Relative executable used for Windows ProductVersion inspection.
    pub executable: String,
    /// Exact supported four-part ProductVersion.
    pub product_version: String,
    /// Steam application id for Steam profiles.
    #[serde(default)]
    pub steam_app_id: Option<u32>,
    /// GOG installed-game ids accepted by this profile.
    #[serde(default)]
    pub gog_game_ids: Vec<String>,
    /// Paths that must exist as direct regular files.
    pub required_files: Vec<String>,
    /// Subset whose absence receives the dedicated Missing SoD finding.
    #[serde(default)]
    pub sod_files: Vec<String>,
    /// Complete path set used for clean-variant core matching.
    pub fingerprint_files: Vec<String>,
    /// Mod-sensitive files, trees, and matching root entries to inventory.
    pub inventory_surfaces: Vec<InventorySurface>,
    /// Known residue patterns that receive specific findings.
    #[serde(default)]
    pub forbidden_residue_patterns: Vec<ForbiddenResiduePattern>,
    /// Independently captured clean byte and inventory variants.
    pub allowed_clean_variants: Vec<AllowedCleanVariant>,
}

/// Kind of mod-sensitive inventory surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryKind {
    /// One exact regular file.
    File,
    /// Every descendant entry beneath one directory, without following links.
    Tree,
    /// Direct entries whose names match one of the authored patterns.
    Matching,
}

/// One profile-owned mod-sensitive inventory surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventorySurface {
    /// Relative file/directory path, or `.` for the game root.
    pub path: String,
    /// How the surface is enumerated.
    pub kind: InventoryKind,
    /// Case-insensitive wildcard patterns for a matching surface.
    #[serde(default)]
    pub patterns: Vec<String>,
}

/// One known dirty residue pattern with authored player-facing text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForbiddenResiduePattern {
    /// Relative directory whose entries are inspected.
    pub surface: String,
    /// Case-insensitive wildcard applied to slash-normalized relative entry names.
    pub pattern: String,
    /// Whether descendants are considered rather than direct children only.
    #[serde(default)]
    pub recursive: bool,
    /// Specific explanation rendered when the pattern matches.
    pub message: String,
}

/// One expected regular-file digest used by a clean variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileFingerprint {
    /// Slash-normalized path relative to the game root.
    pub path: String,
    /// Lowercase SHA-256 of the exact file bytes.
    pub sha256: String,
}

/// Expected kind and optional file digest for an inventoried entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedInventoryEntry {
    /// Slash-normalized path relative to the game root.
    pub path: String,
    /// Direct filesystem entry kind.
    pub kind: InventoryEntryKind,
    /// Required for files and absent for directories.
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Entry kind represented in an allowed inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryEntryKind {
    /// Direct regular file.
    File,
    /// Direct directory.
    Directory,
}

/// One complete allowed combination of core fingerprints and mod-sensitive inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllowedCleanVariant {
    /// Stable clean-variant identity.
    pub id: String,
    /// Exact core fingerprints for every `fingerprint_files` path.
    pub fingerprints: Vec<FileFingerprint>,
    /// Complete expected entry set across all inventory surfaces.
    pub inventory: Vec<ExpectedInventoryEntry>,
}

/// Validated deterministic collection of source profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfiles {
    profiles: Vec<GameProfile>,
}

impl GameProfiles {
    /// Loads every direct `.toml` profile in deterministic case-insensitive order.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let root = path.as_ref();
        let entries = fs::read_dir(root).map_err(|source| GameError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| GameError::Io {
                path: root.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("toml"))
            {
                paths.push(path);
            }
        }
        paths.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());

        let mut profiles = Vec::with_capacity(paths.len());
        let mut ids = BTreeSet::new();
        for path in paths {
            let text = fs::read_to_string(&path).map_err(|source| GameError::Io {
                path: path.clone(),
                source,
            })?;
            let mut profile: GameProfile =
                toml::from_str(&text).map_err(|source| GameError::ProfileParse {
                    path: path.clone(),
                    source,
                })?;
            validate_profile(&mut profile, &path)?;
            if !ids.insert(profile.id.to_ascii_lowercase()) {
                return Err(GameError::InvalidProfile {
                    path,
                    message: format!("duplicate profile id {:?}", profile.id),
                });
            }
            profiles.push(profile);
        }
        if profiles.is_empty() {
            return Err(GameError::InvalidProfile {
                path: root.to_path_buf(),
                message: "profile directory contains no .toml files".to_owned(),
            });
        }
        validate_discovery_identities(&profiles, root)?;
        Ok(Self { profiles })
    }

    /// Number of loaded profiles.
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// Whether no profiles are loaded.
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    /// All profiles in deterministic file order.
    pub fn iter(&self) -> impl Iterator<Item = &GameProfile> {
        self.profiles.iter()
    }

    /// Profiles for one explicit role and storefront.
    pub fn matching(
        &self,
        role: GameRole,
        storefront: Storefront,
    ) -> impl Iterator<Item = &GameProfile> {
        self.profiles
            .iter()
            .filter(move |profile| profile.role == role && profile.storefront == storefront)
    }
}

fn validate_discovery_identities(profiles: &[GameProfile], root: &Path) -> Result<()> {
    let mut executables = BTreeMap::<String, String>::new();
    let mut builds = BTreeSet::new();
    for profile in profiles {
        let identifiers = match profile.storefront {
            Storefront::Steam => profile
                .steam_app_id
                .map(|app_id| vec![app_id.to_string()])
                .unwrap_or_default(),
            Storefront::Gog => profile.gog_game_ids.clone(),
        };
        for identifier in identifiers {
            let identity = format!(
                "{:?}:{:?}:{}",
                profile.storefront,
                profile.role,
                identifier.to_ascii_lowercase()
            );
            let executable = profile.executable.to_ascii_lowercase();
            if let Some(existing) = executables.insert(identity.clone(), executable.clone()) {
                if existing != executable {
                    return invalid(
                        root,
                        format!(
                            "discovery identifier {identifier:?} uses inconsistent executables for {:?} on {:?}",
                            profile.role, profile.storefront
                        ),
                    );
                }
            }
            if !builds.insert(format!("{identity}:{}", profile.product_version)) {
                return invalid(
                    root,
                    format!(
                        "discovery identifier {identifier:?} repeats ProductVersion {:?} for {:?} on {:?}; use clean variants inside one profile",
                        profile.product_version, profile.role, profile.storefront
                    ),
                );
            }
        }
    }
    Ok(())
}

fn validate_profile(profile: &mut GameProfile, path: &Path) -> Result<()> {
    if profile.schema != PROFILE_SCHEMA {
        return invalid(
            path,
            format!("schema {} is not {PROFILE_SCHEMA}", profile.schema),
        );
    }
    if profile.id.trim().is_empty() {
        return invalid(path, "profile id is empty");
    }
    if profile.locale != VERIFIED_ALPHA_LOCALE {
        return invalid(
            path,
            format!(
                "locale {:?} is not the verified alpha locale {VERIFIED_ALPHA_LOCALE:?}",
                profile.locale
            ),
        );
    }
    if !is_four_part_version(&profile.product_version) {
        return invalid(
            path,
            format!(
                "ProductVersion {:?} is not four numeric parts",
                profile.product_version
            ),
        );
    }
    match profile.storefront {
        Storefront::Steam if profile.steam_app_id.is_none() => {
            return invalid(path, "Steam profile has no steam_app_id")
        }
        Storefront::Gog if profile.gog_game_ids.is_empty() => {
            return invalid(path, "GOG profile has no gog_game_ids")
        }
        _ => {}
    }

    validate_paths(path, "required_files", &profile.required_files, false)?;
    validate_paths(path, "sod_files", &profile.sod_files, false)?;
    validate_paths(path, "fingerprint_files", &profile.fingerprint_files, false)?;
    validate_relative(path, "executable", &profile.executable, false)?;

    let required = folded_set(&profile.required_files);
    let mandatory_required = [
        profile.executable.as_str(),
        "chitin.key",
        "dialog.tlk",
        "lang/en_US/dialog.tlk",
        "engine.lua",
    ];
    if mandatory_required
        .iter()
        .any(|required_path| !required.contains(&required_path.to_ascii_lowercase()))
    {
        return invalid(
            path,
            "required_files do not satisfy the minimum safety contract (executable, chitin.key, root and en_US TLKs, engine.lua)",
        );
    }
    if profile.fingerprint_files.is_empty() {
        return invalid(
            path,
            "fingerprint_files do not satisfy the minimum safety contract",
        );
    }
    let fingerprint_paths = folded_set(&profile.fingerprint_files);
    if !fingerprint_paths.contains(&profile.executable.to_ascii_lowercase()) {
        return invalid(
            path,
            "the ProductVersion executable must also be present in fingerprint_files",
        );
    }
    for sod in &profile.sod_files {
        if !required.contains(&sod.to_ascii_lowercase()) {
            return invalid(
                path,
                format!("SoD path {sod:?} is not included in required_files"),
            );
        }
    }
    for fingerprint in &profile.fingerprint_files {
        if !required.contains(&fingerprint.to_ascii_lowercase()) {
            return invalid(
                path,
                format!("fingerprint path {fingerprint:?} is not included in required_files"),
            );
        }
    }

    for surface in &profile.inventory_surfaces {
        validate_relative(path, "inventory surface", &surface.path, true)?;
        match surface.kind {
            InventoryKind::Matching if surface.patterns.is_empty() => {
                return invalid(path, "matching inventory surface has no patterns")
            }
            InventoryKind::File | InventoryKind::Tree if !surface.patterns.is_empty() => {
                return invalid(path, "non-matching inventory surface declares patterns")
            }
            _ => {}
        }
        for pattern in &surface.patterns {
            validate_pattern(path, pattern)?;
        }
    }
    let required_file_surfaces = [
        "chitin.key",
        "dialog.tlk",
        "lang/en_US/dialog.tlk",
        "engine.lua",
    ];
    if required_file_surfaces.iter().any(|required_path| {
        !profile.inventory_surfaces.iter().any(|surface| {
            surface.kind == InventoryKind::File && surface.path.eq_ignore_ascii_case(required_path)
        })
    }) {
        return invalid(
            path,
            "inventory_surfaces do not satisfy the minimum safety contract for core files",
        );
    }
    if !profile.inventory_surfaces.iter().any(|surface| {
        surface.kind == InventoryKind::Tree && surface.path.eq_ignore_ascii_case("override")
    }) {
        return invalid(
            path,
            "inventory_surfaces do not include the required override tree safety surface",
        );
    }
    let root_patterns = profile
        .inventory_surfaces
        .iter()
        .filter(|surface| surface.kind == InventoryKind::Matching && surface.path == ".")
        .flat_map(|surface| surface.patterns.iter())
        .collect::<Vec<_>>();
    let root_sentinels = ["setup-probe.exe", "weidu.log", "probe.debug"];
    if root_sentinels.iter().any(|sentinel| {
        !root_patterns
            .iter()
            .any(|pattern| wildcard_match(pattern, sentinel))
    }) {
        return invalid(
            path,
            "inventory_surfaces root matching patterns do not cover setup, WeiDU, and debug residue",
        );
    }
    for required_path in &profile.required_files {
        let covered = fingerprint_paths.contains(&required_path.to_ascii_lowercase())
            || profile
                .inventory_surfaces
                .iter()
                .any(|surface| inventory_surface_covers_required(surface, required_path));
        if !covered {
            return invalid(
                path,
                format!(
                    "required file {required_path:?} is not covered by fingerprint_files or a deterministic inventory surface"
                ),
            );
        }
    }
    if profile.role == GameRole::BgeeSod {
        if profile.sod_files.is_empty() {
            return invalid(path, "BGEE+SoD profile has no authored SoD safety evidence");
        }
        let fingerprints = folded_set(&profile.fingerprint_files);
        if profile
            .sod_files
            .iter()
            .any(|sod| !fingerprints.contains(&sod.to_ascii_lowercase()))
        {
            return invalid(
                path,
                "BGEE+SoD profile does not fingerprint every authored SoD file",
            );
        }
    }
    for residue in &profile.forbidden_residue_patterns {
        validate_relative(path, "residue surface", &residue.surface, true)?;
        validate_pattern(path, &residue.pattern)?;
        if residue.message.trim().is_empty() {
            return invalid(path, "forbidden residue pattern has an empty message");
        }
    }
    if profile.allowed_clean_variants.is_empty() {
        return invalid(path, "profile has no allowed_clean_variants");
    }

    let expected_fingerprint_paths = folded_set(&profile.fingerprint_files);
    let mut variant_ids = BTreeSet::new();
    for variant in &mut profile.allowed_clean_variants {
        if variant.id.trim().is_empty() || !variant_ids.insert(variant.id.to_ascii_lowercase()) {
            return invalid(
                path,
                format!("invalid or duplicate variant id {:?}", variant.id),
            );
        }
        let variant_paths = variant
            .fingerprints
            .iter()
            .map(|entry| entry.path.to_ascii_lowercase())
            .collect::<BTreeSet<_>>();
        if variant_paths.len() != variant.fingerprints.len()
            || variant_paths != expected_fingerprint_paths
        {
            return invalid(
                path,
                format!(
                    "variant {:?} fingerprints do not exactly cover fingerprint_files",
                    variant.id
                ),
            );
        }
        for fingerprint in &mut variant.fingerprints {
            validate_relative(path, "variant fingerprint", &fingerprint.path, false)?;
            normalize_digest(path, &mut fingerprint.sha256)?;
        }
        let mut inventory_paths = BTreeSet::new();
        for entry in &mut variant.inventory {
            validate_relative(path, "variant inventory", &entry.path, false)?;
            if !inventory_paths.insert(entry.path.to_ascii_lowercase()) {
                return invalid(
                    path,
                    format!(
                        "variant {:?} repeats inventory path {:?}",
                        variant.id, entry.path
                    ),
                );
            }
            match (&entry.kind, &mut entry.sha256) {
                (InventoryEntryKind::File, Some(digest)) => normalize_digest(path, digest)?,
                (InventoryEntryKind::File, None) => {
                    return invalid(path, format!("file {:?} has no SHA-256", entry.path))
                }
                (InventoryEntryKind::Directory, Some(_)) => {
                    return invalid(
                        path,
                        format!("directory {:?} unexpectedly has a SHA-256", entry.path),
                    )
                }
                (InventoryEntryKind::Directory, None) => {}
            }
        }
    }
    Ok(())
}

fn validate_paths(path: &Path, field: &str, values: &[String], allow_dot: bool) -> Result<()> {
    let mut folded = BTreeSet::new();
    for value in values {
        validate_relative(path, field, value, allow_dot)?;
        if !folded.insert(value.to_ascii_lowercase()) {
            return invalid(path, format!("{field} repeats path {value:?}"));
        }
    }
    Ok(())
}

fn validate_relative(path: &Path, field: &str, value: &str, allow_dot: bool) -> Result<()> {
    if value.is_empty() || value.contains('\\') || value.contains(':') {
        return invalid(
            path,
            format!("{field} path {value:?} is not safe and normalized"),
        );
    }
    if allow_dot && value == "." {
        return Ok(());
    }
    let candidate = Path::new(value);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return invalid(
            path,
            format!("{field} path {value:?} is not relative and confined"),
        );
    }
    Ok(())
}

fn validate_pattern(path: &Path, value: &str) -> Result<()> {
    if value.is_empty() || value.contains('\\') || value.contains(':') || value.contains("..") {
        return invalid(path, format!("unsafe inventory pattern {value:?}"));
    }
    Ok(())
}

fn normalize_digest(path: &Path, digest: &mut String) -> Result<()> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return invalid(path, format!("invalid SHA-256 {digest:?}"));
    }
    digest.make_ascii_lowercase();
    Ok(())
}

fn folded_set(values: &[String]) -> BTreeSet<String> {
    values
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect()
}

fn is_four_part_version(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 4
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn inventory_surface_covers_required(surface: &InventorySurface, required_path: &str) -> bool {
    match surface.kind {
        InventoryKind::File => surface.path.eq_ignore_ascii_case(required_path),
        InventoryKind::Tree => {
            let tree = surface.path.trim_end_matches('/');
            required_path.eq_ignore_ascii_case(tree)
                || required_path
                    .get(..tree.len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(tree))
                    && required_path.as_bytes().get(tree.len()) == Some(&b'/')
        }
        InventoryKind::Matching => false,
    }
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

fn invalid<T>(path: &Path, message: impl Into<String>) -> Result<T> {
    Err(GameError::InvalidProfile {
        path: PathBuf::from(path),
        message: message.into(),
    })
}
