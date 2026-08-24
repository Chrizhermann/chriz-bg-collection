//! Manifest schema v1: one TOML per mod plus the collection-level TOML.

use serde::{Deserialize, Serialize};

/// Install phase a mod belongs to, relative to the EET merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    Bg1PreMerge,
    Main,
    PostEetEnd,
}

/// How a mod's archive is obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    GithubRelease,
    GithubTagArchive,
    GithubCommitZip,
    Manual,
}

/// Where a mod's archive comes from and how it is verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub kind: SourceKind,
    /// Download URL; for kind = manual this is the page the user is sent to.
    pub url: String,
    pub sha256: String,
}

/// One installable WeiDU component (DESIGNATED number).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: u32,
    pub name: String,
    /// Scripted stdin for READLN prompts, verbatim (include trailing newline).
    #[serde(default)]
    pub stdin: Option<String>,
}

/// A single `manifest/mods/<id>.toml` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModFile {
    pub id: String,
    pub name: String,
    pub version: String,
    pub tp2: String,
    #[serde(default)]
    pub language: u32,
    pub weidu: String,
    pub platforms: Vec<String>,
    pub phase: Phase,
    pub source: Source,
    #[serde(default)]
    pub components: Vec<Component>,
}

/// One position in the flat install order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderEntry {
    /// mod id; granularity finer than a whole mod uses `components`
    pub id: String,
    /// None = all of the mod's components at this position
    #[serde(default)]
    pub components: Option<Vec<u32>>,
}

/// Points at one component of one mod.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentRef {
    pub mod_id: String,
    pub component: u32,
}

/// A user-facing on/off switch over mods and components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Toggle {
    pub id: String,
    pub name: String,
    pub default_on: bool,
    /// mod ids fully removed when off
    #[serde(default)]
    pub removes_mods: Vec<String>,
    /// components additionally removed when off — knock-on exclusions
    #[serde(default)]
    pub removes_components: Vec<ComponentRef>,
}

/// A set of mutually exclusive options the user picks one of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceGroup {
    pub id: String,
    pub name: String,
    pub default: String,
    pub options: Vec<ChoiceOption>,
}

/// One selectable option inside a [`ChoiceGroup`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceOption {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub adds_components: Vec<ComponentRef>,
    #[serde(default)]
    pub removes_components: Vec<ComponentRef>,
}

/// The collection-level `collection.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    /// Manifest schema version; this engine understands 1.
    pub schema: u32,
    pub game_build: String,
    #[serde(default)]
    pub order: Vec<OrderEntry>,
    #[serde(default)]
    pub toggles: Vec<Toggle>,
    #[serde(default)]
    pub choice_groups: Vec<ChoiceGroup>,
}
