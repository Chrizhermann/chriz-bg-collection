//! Executable recipe schema v2.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Game installation root targeted by an installer run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameRoot {
    /// The staged Baldur's Gate: Enhanced Edition plus Siege of Dragonspear root.
    Bg1,
    /// The staged Baldur's Gate II: Enhanced Edition / EET root.
    Bg2,
}

/// Install phase of one explicit recipe run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    /// Work performed against BG1 before EET imports it.
    Bg1Preparation,
    /// Work performed against BG2 before EET initialization.
    Bg2Preparation,
    /// The EET initialization/import run.
    EetInitialization,
    /// Ordinary EET mod installation after initialization.
    Main,
    /// The EET finalization run.
    EetFinalization,
    /// An explicitly reviewed additive run allowed after EET finalization.
    PostEetEnd,
}

impl Phase {
    /// Returns the staged game root against which this phase executes.
    pub const fn game_root(self) -> GameRoot {
        match self {
            Self::Bg1Preparation => GameRoot::Bg1,
            Self::Bg2Preparation
            | Self::EetInitialization
            | Self::Main
            | Self::EetFinalization
            | Self::PostEetEnd => GameRoot::Bg2,
        }
    }
}

/// One typed extra argument supplied to a WeiDU installer run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum RunArg {
    /// A literal argument authored in the recipe.
    Literal(String),
    /// The path of a staged game root, resolved only at execution time.
    StagedRoot(GameRoot),
}

/// How the WeiDU executable locates an installer's TP2 file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvocationMode {
    /// Invoke through a conventional `setup-<name>` executable name.
    SetupName,
    /// Pass the declared TP2 path explicitly.
    ExplicitTp2,
}

/// How a source archive is located.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    /// An immutable asset attached to a GitHub release.
    GithubRelease,
    /// A GitHub-generated archive for an immutable tag.
    GithubTagArchive,
    /// A GitHub-generated archive for an immutable commit.
    GithubCommitZip,
    /// A page-gated or otherwise user-supplied archive.
    Manual,
}

/// Source identity and integrity pin for an artifact archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Kind of upstream source.
    pub kind: SourceKind,
    /// Immutable download URL, or the official handoff page for manual sources.
    pub url: String,
    /// Expected lowercase or uppercase hexadecimal SHA-256 digest.
    pub sha256: String,
}

/// Expected path inside an extracted artifact archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveSpec {
    /// Relative path that identifies the payload root or required file.
    pub path: String,
}

/// Public acquisition policy for an artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcquisitionPolicy {
    /// The installer may fetch the artifact but never redistribute it.
    FetchOnly,
    /// The user must supply the artifact through the manual acquisition flow.
    ManualUserSupplied,
    /// Bundling is permitted by reviewed provenance.
    BundlePermitted,
    /// The artifact is declared but unavailable for acquisition.
    Blocked,
}

/// Human-auditable origin and license information for an artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Canonical project or author homepage.
    pub homepage: String,
    /// License identifier or reviewed license description.
    pub license: String,
}

/// One independently acquired archive used by one or more installers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    /// Stable artifact id; must match its file stem.
    pub id: String,
    /// User-facing artifact name.
    pub name: String,
    /// Immutable upstream version or revision.
    pub version: String,
    /// Source identity and digest.
    pub source: Source,
    /// Expected archive layout.
    pub archive: ArchiveSpec,
    /// Policy controlling how the artifact may be obtained.
    pub acquisition: AcquisitionPolicy,
    /// Reviewed origin and license information.
    pub provenance: Provenance,
}

/// One installable WeiDU component (`DESIGNATED` number).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    /// WeiDU component number.
    pub id: u32,
    /// User-facing component name.
    pub name: String,
    /// Legacy scripted stdin, retained until typed prompt steps replace it.
    #[serde(default)]
    pub stdin: Option<String>,
    /// Ordered output-gated prompt answers for this component.
    #[serde(default)]
    pub prompts: Vec<PromptStep>,
}

/// A single installer definition from `mods/<id>.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModFile {
    /// Stable installer id; must match its file stem.
    pub id: String,
    /// Artifact whose extracted payload contains this installer.
    pub artifact_id: String,
    /// User-facing installer name.
    pub name: String,
    /// Relative path to the installer's TP2 file.
    pub tp2: String,
    /// WeiDU language number.
    pub language: u32,
    /// Artifact providing the WeiDU executable used for this installer.
    pub weidu_artifact_id: String,
    /// Method used to locate the TP2 at invocation time.
    pub invocation_mode: InvocationMode,
    /// Components declared by this installer.
    pub components: Vec<Component>,
}

/// One explicit invocation in the frozen collection order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    /// Globally unique logical run id.
    pub run_id: String,
    /// Installer id executed by this run.
    pub mod_id: String,
    /// EET install phase.
    pub phase: Phase,
    /// Exact ordered component numbers installed by this run.
    pub components: Vec<u32>,
    /// Typed extra invocation arguments.
    pub args: Vec<RunArg>,
}

/// Points at one component in one explicit run.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentRef {
    /// Logical run containing the component.
    pub run_id: String,
    /// WeiDU component number.
    pub component: u32,
}

/// Curator decision controlling player visibility and default selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Decision {
    /// Omit the feature entirely from selection and execution.
    Excluded,
    /// Expose the feature unchecked by default.
    Optional,
    /// Expose the feature checked by default.
    Default,
    /// Include the feature whenever its parent is active, without an independent control.
    Mandatory,
}

/// Release readiness attached to one curated feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Readiness {
    /// The feature is ready for the selected recipe channel.
    Ready,
    /// The feature may resolve but must be labeled experimental.
    Experimental,
    /// The feature is documented but cannot resolve.
    Blocked,
}

/// A directional compatibility rule that makes its owning feature unavailable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conflict {
    /// Semantic id of the feature whose effective selection triggers the conflict.
    pub feature_id: String,
    /// Authored player-facing explanation.
    pub reason: String,
}

/// One selectable value for a choice input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputOption {
    /// Stable semantic option id stored in selections and receipts.
    pub id: String,
    /// Player-facing option title.
    pub title: String,
    /// Exact answer text emitted when a prompt references this option.
    pub answer: String,
}

/// Typed configuration accepted by a curated feature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum InputSpec {
    /// A true/false setting.
    Boolean {
        /// Stable input id within the feature.
        id: String,
        /// Initial value.
        default: bool,
    },
    /// A single choice from authored semantic options.
    Choice {
        /// Stable input id within the feature.
        id: String,
        /// Initial option id, including explicit `none` for optional-only groups.
        default: String,
        /// Available options in player-facing order.
        options: Vec<InputOption>,
    },
    /// A bounded whole-number setting.
    Integer {
        /// Stable input id within the feature.
        id: String,
        /// Initial value.
        default: i64,
        /// Inclusive minimum value.
        min: i64,
        /// Inclusive maximum value.
        max: i64,
    },
}

impl InputSpec {
    /// Returns the stable id of this input.
    pub fn id(&self) -> &str {
        match self {
            Self::Boolean { id, .. } | Self::Choice { id, .. } | Self::Integer { id, .. } => id,
        }
    }

    /// Returns the authored default as a typed value.
    pub fn default_value(&self) -> InputValue {
        match self {
            Self::Boolean { default, .. } => InputValue::Boolean(*default),
            Self::Choice { default, .. } => InputValue::Choice(default.clone()),
            Self::Integer { default, .. } => InputValue::Integer(*default),
        }
    }
}

/// A validated value for one feature input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum InputValue {
    /// A true/false value.
    Boolean(bool),
    /// A semantic option id.
    Choice(String),
    /// A whole-number value.
    Integer(i64),
}

/// Semantic address of one input owned by one feature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureInputRef {
    /// Stable owning feature id.
    pub feature_id: String,
    /// Stable input id within that feature.
    pub input_id: String,
}

/// Typed source of one prompt response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum PromptAnswer {
    /// A typed literal authored directly on the component.
    Literal(InputValue),
    /// A reference resolved from one validated feature input.
    Input(FeatureInputRef),
}

/// One output-gated prompt response, retained as an individual step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptStep {
    /// Output text that must be observed before sending the answer.
    pub expected_output: String,
    /// Typed literal or feature-input reference providing the answer.
    pub answer: PromptAnswer,
}

/// One player-facing semantic feature and its exact component expansion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    /// Stable selection identity.
    pub id: String,
    /// Player-facing title.
    pub title: String,
    /// Player-facing description.
    pub description: String,
    /// Player-facing category id; first occurrence defines category order.
    pub category: String,
    /// Curated visibility/default decision.
    pub decision: Decision,
    /// Release readiness.
    pub readiness: Readiness,
    /// Authored reason shown when readiness is blocked.
    #[serde(default)]
    pub unavailable_reason: Option<String>,
    /// Optional parent feature whose effective state gates this feature.
    #[serde(default)]
    pub parent: Option<String>,
    /// Exact component ownership in authored recipe runs.
    #[serde(default)]
    pub components: Vec<ComponentRef>,
    /// Other features that must be effective first.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Directional compatibility rules that disable this feature.
    #[serde(default)]
    pub conflicts: Vec<Conflict>,
    /// Typed configuration owned by this feature.
    #[serde(default)]
    pub inputs: Vec<InputSpec>,
}

/// Minimal preset metadata loaded from `presets/<id>.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresetFile {
    /// Stable preset id; must match its file stem.
    pub id: String,
    /// User-facing preset name.
    pub name: String,
    /// Semantic feature and input values, using the same keys as [`crate::resolve::Selection`].
    #[serde(default)]
    pub selections: BTreeMap<String, String>,
}

/// Collection-level executable recipe from `collection.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    /// Manifest schema version; this engine understands version 2.
    pub schema: u32,
    /// Required Enhanced Edition game build.
    pub game_build: String,
    /// Explicit ordered installer invocations.
    pub runs: Vec<Run>,
    /// Player-facing semantic features in authored display order.
    #[serde(default)]
    pub features: Vec<Feature>,
}
