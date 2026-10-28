//! The hotpatches one app offers, read from its bundled catalog folder: `catalog.toml`, plus the
//! package of each hotpatch in the recipe's own artifact and mod formats below `artifacts/` and
//! `mods/`. The catalog ships inside the app like its recipe; every package is pinned by exact
//! length and SHA-256, as recipe artifacts are.
use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::digest::sha256_bytes;
use crate::manifest::{
    AcquisitionPolicy, ArchiveKind, Artifact, CollectionGame, ModFile, SourceKind,
};
use crate::patches::files;
use crate::updates::SaveApplicability;

/// Format of `catalog.toml`.
pub const CATALOG_SCHEMA: u32 = 1;
const FILE_LIMIT: u64 = 1024 * 1024;

/// Christopher's rule for how far an installation may differ from the recommended setup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateRules {
    /// Preset of the installation's own recipe that counts as the recommended setup.
    pub recommended_preset: String,
    /// Most mods that may differ from the recommended setup, counting neither free mods nor
    /// required ones that were merely kept.
    pub max_changed_mods: usize,
    /// Mods whose choices never count, such as Tweaks Anthology or the interface.
    #[serde(default)]
    pub free_mods: BTreeSet<String>,
    /// Mods that must stay installed, such as Sword Coast Stratagems.
    #[serde(default)]
    pub required_mods: BTreeSet<String>,
}

/// One collection release that hotpatches are made for, identified by its recipe content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Base {
    pub id: String,
    pub label: String,
    pub recipe_version: String,
    /// [`super::recipe_content_digest`] of the released recipe.
    pub content_sha256: String,
}

/// A component one installed mod must have before a hotpatch fits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentRequirement {
    #[serde(rename = "mod")]
    pub mod_id: String,
    pub any_of: Vec<u32>,
}

/// One changelog line and what it means for running saves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub title: String,
    pub summary: String,
    pub save_applicability: SaveApplicability,
    #[serde(default)]
    pub condition_note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogFile {
    schema: u32,
    game: CollectionGame,
    #[serde(default)]
    runtime_files: Vec<String>,
    gate: GateRules,
    #[serde(default)]
    bases: Vec<Base>,
    #[serde(default)]
    hotpatches: Vec<HotpatchEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct HotpatchEntry {
    id: String,
    title: String,
    summary: String,
    bases: Vec<String>,
    #[serde(default)]
    requires: Vec<String>,
    artifact: String,
    #[serde(rename = "mod")]
    mod_id: String,
    components: Vec<u32>,
    #[serde(default)]
    requires_components: Vec<ComponentRequirement>,
    changes: Vec<Change>,
}

/// One hotpatch: new components of one pinned mod release, installed after everything else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hotpatch {
    pub id: String,
    pub title: String,
    pub summary: String,
    /// Ids of the bases it fits.
    pub bases: Vec<String>,
    /// Hotpatches that must already be applied, in catalog order.
    pub requires: Vec<String>,
    pub artifact: Artifact,
    pub installer: ModFile,
    /// Components installed, in this order.
    pub components: Vec<u32>,
    pub requires_components: Vec<ComponentRequirement>,
    pub changes: Vec<Change>,
}

impl Hotpatch {
    /// The single folder the package publishes into the game folder, lowercase.
    pub fn mod_folder(&self) -> String {
        self.artifact.archive.publish_roots[0].to_ascii_lowercase()
    }
}

/// Paths the game, its loader or installed mods write while the game runs. They neither block a
/// hotpatch nor may a hotpatch write them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFiles(Vec<String>);

impl RuntimeFiles {
    /// Whether the lowercase relative `name` is written at runtime.
    pub fn matches(&self, name: &str) -> bool {
        self.0.iter().any(|pattern| pattern_matches(pattern, name))
    }
}

/// The whole validated catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Catalog {
    pub game: CollectionGame,
    pub runtime_files: RuntimeFiles,
    pub gate: GateRules,
    pub bases: Vec<Base>,
    /// In the order they apply.
    pub hotpatches: Vec<Hotpatch>,
    /// Digest of every catalog file read, so a review token never outlives a catalog change.
    pub digest: String,
}

impl Catalog {
    pub fn base(&self, id: &str) -> Option<&Base> {
        self.bases.iter().find(|base| base.id == id)
    }
    pub fn hotpatch(&self, id: &str) -> Option<&Hotpatch> {
        self.hotpatches.iter().find(|hotpatch| hotpatch.id == id)
    }
}

/// Read and check the catalog folder `dir`.
pub fn load(dir: &Path) -> Result<Catalog, String> {
    let mut digest_input = Vec::new();
    let mut read = |relative: &str| -> Result<String, String> {
        let bytes = files::read(&dir.join(relative), FILE_LIMIT)?;
        digest_input.extend_from_slice(relative.as_bytes());
        digest_input.push(0);
        digest_input.extend_from_slice(sha256_bytes(&bytes).as_bytes());
        String::from_utf8(bytes).map_err(|_| format!("{relative} is not UTF-8."))
    };
    let text = read("catalog.toml")?;
    let file: CatalogFile =
        toml::from_str(&text).map_err(|error| format!("catalog.toml: {error}"))?;
    if file.schema != CATALOG_SCHEMA {
        return Err("This hotpatch catalog has an unknown format.".into());
    }
    validate_rules(&file.gate)?;
    let mut runtime = Vec::new();
    for pattern in &file.runtime_files {
        validate_pattern(pattern)?;
        runtime.push(pattern.clone());
    }
    let mut base_ids = BTreeSet::new();
    for base in &file.bases {
        identifier(&base.id)?;
        if !base_ids.insert(base.id.clone())
            || base.label.trim().is_empty()
            || base.recipe_version.trim().is_empty()
            || !hex64(&base.content_sha256)
        {
            return Err(format!("Invalid hotpatch base {:?}.", base.id));
        }
    }
    let mut hotpatches: Vec<Hotpatch> = Vec::new();
    for entry in file.hotpatches {
        identifier(&entry.id)?;
        if hotpatches.iter().any(|earlier| earlier.id == entry.id) {
            return Err(format!("Hotpatch {:?} is listed twice.", entry.id));
        }
        if entry.title.trim().is_empty() || entry.summary.trim().is_empty() {
            return Err(format!(
                "Hotpatch {:?} needs a title and a summary.",
                entry.id
            ));
        }
        if entry.bases.is_empty() || entry.bases.iter().any(|id| !base_ids.contains(id)) {
            return Err(format!("Hotpatch {:?} names an unknown base.", entry.id));
        }
        if entry
            .requires
            .iter()
            .any(|id| !hotpatches.iter().any(|earlier| earlier.id == *id))
        {
            return Err(format!(
                "Hotpatch {:?} requires a hotpatch that is not listed before it.",
                entry.id
            ));
        }
        if entry.changes.is_empty()
            || entry
                .changes
                .iter()
                .any(|change| change.title.trim().is_empty() || change.summary.trim().is_empty())
        {
            return Err(format!(
                "Hotpatch {:?} needs a complete changelog.",
                entry.id
            ));
        }
        identifier(&entry.artifact)?;
        identifier(&entry.mod_id)?;
        let artifact: Artifact =
            toml::from_str(&read(&format!("artifacts/{}.toml", entry.artifact))?)
                .map_err(|error| format!("artifacts/{}.toml: {error}", entry.artifact))?;
        let installer: ModFile = toml::from_str(&read(&format!("mods/{}.toml", entry.mod_id))?)
            .map_err(|error| format!("mods/{}.toml: {error}", entry.mod_id))?;
        let hotpatch = Hotpatch {
            id: entry.id,
            title: entry.title,
            summary: entry.summary,
            bases: entry.bases,
            requires: entry.requires,
            artifact,
            installer,
            components: entry.components,
            requires_components: entry.requires_components,
            changes: entry.changes,
        };
        validate_package(&hotpatch, &entry.artifact, &entry.mod_id)?;
        hotpatches.push(hotpatch);
    }
    Ok(Catalog {
        game: file.game,
        runtime_files: RuntimeFiles(runtime),
        gate: file.gate,
        bases: file.bases,
        hotpatches,
        digest: sha256_bytes(&digest_input),
    })
}

fn validate_rules(rules: &GateRules) -> Result<(), String> {
    identifier(&rules.recommended_preset)?;
    for id in rules.free_mods.iter().chain(&rules.required_mods) {
        identifier(id)?;
    }
    Ok(())
}

fn validate_package(hotpatch: &Hotpatch, artifact_id: &str, mod_id: &str) -> Result<(), String> {
    let refuse = |what: &str| Err(format!("Hotpatch {:?}: {what}", hotpatch.id));
    let artifact = &hotpatch.artifact;
    let installer = &hotpatch.installer;
    if artifact.id != artifact_id || installer.id != mod_id || installer.artifact_id != artifact.id
    {
        return refuse("its artifact and mod files do not name each other.");
    }
    if artifact.acquisition != AcquisitionPolicy::FetchOnly
        || artifact.source.kind != SourceKind::GithubRelease
        || !artifact.source.url.starts_with("https://")
        || artifact.source.expected_length.is_none()
        || !hex64(&artifact.source.sha256)
        || artifact.archive.kind != ArchiveKind::Zip
        || artifact.tool.is_some()
    {
        return refuse("its package must be a pinned GitHub release ZIP of a mod.");
    }
    let [root] = artifact.archive.publish_roots.as_slice() else {
        return refuse("its package must publish exactly one mod folder.");
    };
    identifier(&root.to_ascii_lowercase())?;
    let tp2 = installer.tp2.replace('\\', "/");
    files::safe_relative(&tp2)?;
    if artifact.archive.tp2_paths != [installer.tp2.clone()]
        || !tp2
            .to_ascii_lowercase()
            .starts_with(&format!("{}/", root.to_ascii_lowercase()))
    {
        return refuse("its installer must lie in the mod folder it publishes.");
    }
    if hotpatch.components.is_empty()
        || hotpatch.components.iter().collect::<BTreeSet<_>>().len() != hotpatch.components.len()
    {
        return refuse("it must install distinct components.");
    }
    for id in &hotpatch.components {
        let Some(component) = installer.components.iter().find(|c| c.id == *id) else {
            return refuse("it installs a component its mod file does not declare.");
        };
        // A hotpatch runs unattended; a component that asks questions does not fit.
        if component.stdin.is_some()
            || !component.prompts.is_empty()
            || !component.config_files.is_empty()
        {
            return refuse("its components must install without questions or settings.");
        }
    }
    for requirement in &hotpatch.requires_components {
        identifier(&requirement.mod_id)?;
        if requirement.any_of.is_empty() {
            return refuse("a component requirement names no component.");
        }
    }
    Ok(())
}

fn identifier(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_.".contains(&b))
        || value.starts_with('.')
    {
        return Err(format!("Invalid hotpatch catalog identifier {value:?}."));
    }
    Ok(())
}

fn hex64(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Patterns are lowercase relative names: an exact name, `folder/**` for a folder and everything
/// below it, or a last segment with `*` wildcards such as `override/bfbotbg*.mos`.
fn validate_pattern(pattern: &str) -> Result<(), String> {
    let invalid = || Err(format!("Invalid runtime file pattern {pattern:?}."));
    if pattern.to_ascii_lowercase() != pattern {
        return invalid();
    }
    let (folders, last) = match pattern.rsplit_once('/') {
        Some((folders, last)) => (Some(folders), last),
        None => (None, pattern),
    };
    if let Some(folders) = folders {
        if folders.contains('*') || files::safe_relative(folders).is_err() {
            return invalid();
        }
    }
    if last == "**" {
        return if folders.is_some() { Ok(()) } else { invalid() };
    }
    if last.is_empty()
        || last.contains("**")
        || files::safe_relative(&last.replace('*', "x")).is_err()
    {
        return invalid();
    }
    Ok(())
}

fn pattern_matches(pattern: &str, name: &str) -> bool {
    if let Some(folder) = pattern.strip_suffix("/**") {
        return name == folder || name.starts_with(&format!("{folder}/"));
    }
    let (pattern_folder, pattern_last) = pattern.rsplit_once('/').unwrap_or(("", pattern));
    let (name_folder, name_last) = name.rsplit_once('/').unwrap_or(("", name));
    pattern_folder == name_folder && wildcard(pattern_last.as_bytes(), name_last.as_bytes())
}

fn wildcard(pattern: &[u8], text: &[u8]) -> bool {
    match pattern.split_first() {
        None => text.is_empty(),
        Some((b'*', rest)) => (0..=text.len()).any(|skip| wildcard(rest, &text[skip..])),
        Some((first, rest)) => text
            .split_first()
            .is_some_and(|(head, tail)| head == first && wildcard(rest, tail)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    /// The Icewind Dale app's catalog fits the recipe it ships with: the same game and mods, and
    /// a base for the current release whose digest is that recipe's. A recipe change before the
    /// release therefore needs the digest updated here as well.
    #[test]
    fn the_bundled_iwd_catalog_fits_the_current_release() {
        let catalog = load(&repository().join("hotpatches/iwd")).unwrap();
        let recipe = crate::Manifest::load(&repository().join("recipes/iwd-current")).unwrap();
        assert_eq!(catalog.game, recipe.collection.game);
        assert!(recipe
            .presets
            .contains_key(&catalog.gate.recommended_preset));
        for id in catalog
            .gate
            .free_mods
            .iter()
            .chain(&catalog.gate.required_mods)
        {
            assert!(recipe.mods.contains_key(id), "unknown mod {id:?}");
        }
        let release: serde_json::Value = serde_json::from_slice(
            &std::fs::read(repository().join("recipes/iwd-current/release.json")).unwrap(),
        )
        .unwrap();
        let version = release["version"].as_str().unwrap();
        // A carriage return in a recipe value would make the digest depend on the checkout's
        // line endings.
        fn carriage_return(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::String(text) => text.contains('\r'),
                serde_json::Value::Array(items) => items.iter().any(carriage_return),
                serde_json::Value::Object(fields) => fields
                    .iter()
                    .any(|(key, value)| key.contains('\r') || carriage_return(value)),
                _ => false,
            }
        }
        let content = serde_json::to_value((
            &recipe.collection,
            &recipe.artifacts,
            &recipe.mods,
            &recipe.presets,
        ))
        .unwrap();
        assert!(
            !carriage_return(&content),
            "a recipe value holds a carriage return"
        );
        let digest = super::super::recipe_content_digest(&recipe).unwrap();
        let base = catalog
            .bases
            .iter()
            .find(|base| base.recipe_version == version)
            .unwrap_or_else(|| {
                panic!("hotpatches/iwd/catalog.toml needs a base for {version} with content_sha256 = {digest:?}")
            });
        assert_eq!(
            base.content_sha256, digest,
            "recipes/iwd-current changed: set content_sha256 of base {:?} to {digest:?}",
            base.id
        );
    }

    /// What played IWD installations wrote into the game folder (Christopher's own one, and the
    /// 0.1.1 release lab, 2026-10-29) must not make them look edited by hand; the files mods and
    /// hotpatches install must stay checked.
    #[test]
    fn the_bundled_iwd_catalog_ignores_what_a_played_game_writes() {
        let catalog = load(&repository().join("hotpatches/iwd")).unwrap();
        for name in [
            "infinityloader.ini",
            "infinityloader.db",
            "infinityloader.log",
            "eeex_scripts/extra_pattern_dbs/v2.7.3.0.db",
            "buffbot_exec.log",
            "override/bfbtch.spl",
            "override/bfbotbg2.mos",
            "override/eeex_remote_ready.json",
            "override/eeex_remote_result.json.tmp",
            "override/eeex_remote_cmd.lua.run",
        ] {
            assert!(catalog.runtime_files.matches(name), "{name}");
        }
        for name in [
            "weidu.log",
            "chitin.key",
            "lang/en_us/dialog.tlk",
            "override/ui.menu",
            "override/m___eeex.lua",
            "override/m_eeexrc.lua",
            "eeex_scripts/extra_pattern_dbs",
            "eeex_scripts/extra_pattern_dbs/readme.txt",
            "eeex/loader/v2.7.3.0/pattern_dbs/v2.7.3.0.db",
            "chriz-iwd-encounters/setup-chriz-iwd-encounters.tp2",
        ] {
            assert!(!catalog.runtime_files.matches(name), "{name}");
        }
    }

    /// An installation keeps its recipe as JSON; read back, it has the digest it was made with.
    #[test]
    fn a_frozen_recipe_keeps_its_content_digest() {
        let recipe = crate::Manifest::load(&repository().join("recipes/iwd-current")).unwrap();
        let frozen = serde_json::to_vec(&(
            &recipe.collection,
            &recipe.artifacts,
            &recipe.mods,
            &recipe.presets,
        ))
        .unwrap();
        let (collection, artifacts, mods, presets) = serde_json::from_slice(&frozen).unwrap();
        let thawed = crate::Manifest {
            root: std::path::PathBuf::from("elsewhere"),
            collection,
            artifacts,
            mods,
            presets,
        };
        assert_eq!(
            super::super::recipe_content_digest(&thawed).unwrap(),
            super::super::recipe_content_digest(&recipe).unwrap()
        );
    }

    #[test]
    fn runtime_patterns_match_exact_names_folders_and_wildcards_only() {
        let runtime = RuntimeFiles(vec![
            "infinityloader.ini".into(),
            "infinityloader_crash/**".into(),
            "buffbot_*.log".into(),
            "override/bfbotbg*.mos".into(),
        ]);
        for name in [
            "infinityloader.ini",
            "infinityloader_crash",
            "infinityloader_crash/dump/1.dmp",
            "buffbot_exec.log",
            "override/bfbotbg.mos",
            "override/bfbotbg3.mos",
        ] {
            assert!(runtime.matches(name), "{name}");
        }
        for name in [
            "infinityloader.ini.bak",
            "infinityloader_crashes/x",
            "override/buffbot_exec.log",
            "override/bfbotbg.mos/x",
            "bfbotbg.mos",
            "weidu.log",
        ] {
            assert!(!runtime.matches(name), "{name}");
        }
        for bad in ["**", "Override/x", "a/*/b", "../x", "a/**/b", "a/***"] {
            assert!(validate_pattern(bad).is_err(), "{bad}");
        }
        for good in ["a", "a/b/**", "a/*.log", "*.log"] {
            validate_pattern(good).unwrap();
        }
    }
}
