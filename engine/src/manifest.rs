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
    /// A package of the Weasel Mods download manager, fetched unattended from its anonymous
    /// direct link `https://downloads.weaselmods.net/download/<slug>/?wpdmdl=<package id>`.
    ///
    /// The link names a package, not a revision: the author may replace the file behind it,
    /// so `reference` holds the release version and the pinned `expected_length` and `sha256`
    /// decide whether the served archive is still the reviewed one. Fetching fails closed
    /// unless the URL has exactly that canonical shape on that host, the response is a ZIP
    /// (`Content-Type: application/zip`) with an exact `Content-Length`, and its
    /// `Content-Disposition` attachment filename, percent-decoded the way a browser saves it,
    /// equals `expected_filename`. Redirects are followed only to hosts listed in
    /// `redirect_hosts`; with none listed, any redirect fails. No cookies, login or
    /// per-visit page token are used. The archive kind must be `zip`.
    WeaselmodsDownload,
}

/// Source identity and integrity pin for an artifact archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Kind of upstream source.
    pub kind: SourceKind,
    /// Immutable download URL, or the official handoff page for manual sources.
    pub url: String,
    /// Exact upstream tag, commit, release, or otherwise immutable revision label.
    pub reference: String,
    /// Stable upstream filename, when the publication route defines one.
    #[serde(default)]
    pub expected_filename: Option<String>,
    /// Exact archive byte length, when the publication route exposes stable bytes.
    #[serde(default)]
    pub expected_length: Option<u64>,
    /// Expected lowercase or uppercase hexadecimal SHA-256 digest.
    pub sha256: String,
    /// Redirect destination hosts reviewed for this exact source route.
    #[serde(default)]
    pub redirect_hosts: Vec<String>,
}

/// Supported immutable archive framing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveKind {
    /// A standard ZIP archive.
    Zip,
    /// An Infinity Engine mod archive using ZIP framing.
    Iemod,
    /// A Windows executable containing an embedded RAR archive; the executable is never run.
    SelfExtractingRar,
}

/// How the payload is rooted inside the downloaded archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveRootRule {
    /// Declared publication paths are present at archive root.
    Direct,
    /// Every payload path is beneath one wrapper directory that is stripped.
    SingleWrapper,
    /// Either direct layout or one wrapper is accepted when exactly one layout matches.
    DirectOrSingleWrapper,
}

/// Recipe-authored resource limits applied before archive extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveLimitsSpec {
    /// Maximum number of path components in one archive entry.
    pub max_depth: usize,
    /// Maximum central-directory entry count.
    pub max_entries: usize,
    /// Maximum uncompressed size of one regular file.
    pub max_entry_uncompressed_bytes: u64,
    /// Maximum aggregate uncompressed size of all regular files.
    pub max_total_uncompressed_bytes: u64,
    /// Maximum integer ratio of uncompressed to compressed bytes.
    pub max_compression_ratio: u64,
}

/// Exact expected layout inside an artifact archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveSpec {
    /// Archive framing used by the immutable source.
    pub kind: ArchiveKind,
    /// Whether the declared payload is direct or beneath one wrapper.
    pub root_rule: ArchiveRootRule,
    /// Exact files or directory roots permitted to be published.
    pub publish_roots: Vec<String>,
    /// Exact WeiDU TP2 paths expected after wrapper removal.
    pub tp2_paths: Vec<String>,
    /// Bounds applied before extraction writes any entry.
    pub limits: ArchiveLimitsSpec,
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
    /// URL supporting the reviewed source and distribution decision.
    pub url: String,
    /// Calendar date on which the provenance evidence was reviewed.
    pub reviewed_on: String,
}

/// Expected Windows PE machine architecture for an executable tool artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PeMachine {
    /// 32-bit Intel x86 (`IMAGE_FILE_MACHINE_I386`).
    X86,
    /// 64-bit AMD/Intel (`IMAGE_FILE_MACHINE_AMD64`).
    X86_64,
    /// 64-bit ARM (`IMAGE_FILE_MACHINE_ARM64`).
    Arm64,
}

/// Executable-tool evidence attached only to tool artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolSpec {
    /// Exact executable path after archive wrapper removal.
    pub executable: String,
    /// Exact extracted executable byte length.
    pub expected_length: u64,
    /// Exact lowercase or uppercase SHA-256 of the extracted executable bytes.
    pub sha256: String,
    /// Numeric WeiDU version parsed from the executable's path-prefixed `--version` output.
    pub weidu_version: String,
    /// Expected PE COFF machine field.
    pub pe_machine: PeMachine,
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
    /// Executable identity when this artifact supplies a tool rather than a mod payload.
    #[serde(default)]
    pub tool: Option<ToolSpec>,
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
    /// Configuration files the installer reads while installing this component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config_files: Vec<ConfigFile>,
}

/// Largest configuration file a component may place.
pub const MAX_CONFIG_FILE_BYTES: usize = 64 * 1024;

/// A recipe-authored text file that a mod reads at install time, such as the Tweaks
/// Anthology's `cdtweaks.txt`. It is placed before the run that installs its component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    /// Normalized path relative to the staged target root, below a folder that the installer's
    /// artifact publishes.
    pub path: String,
    /// Exact file content.
    pub content: String,
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

/// A typed assertion that must hold after WeiDU proves a run installed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Postcondition {
    /// Requires and/or forbids exact byte markers in one bounded text file.
    TextFileMarkers {
        /// Normalized path relative to the staged target root.
        path: String,
        /// Exact case-sensitive byte strings that must be present.
        required: Vec<String>,
        /// Exact case-sensitive byte strings that must be absent.
        forbidden: Vec<String>,
        /// Maximum accepted file size and read length.
        max_bytes: u64,
    },
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
    /// Assertions checked after install-log reconciliation succeeds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub postconditions: Vec<Postcondition>,
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
    /// Semantic features of which at least one must be effective before this prompt is authored.
    /// An empty list keeps the prompt unconditional.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub when_any_features: Vec<String>,
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
    /// Optional source name retained from the curation catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_label: Option<String>,
    /// Context shared by related options, such as potion stacking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_label: Option<String>,
    /// Authored same-purpose choice identity; not a general conflict group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice_group: Option<String>,
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
    /// At least one of these features must be effective; empty means no OR gate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires_any: Vec<String>,
    /// Directional compatibility rules that disable this feature.
    #[serde(default)]
    pub conflicts: Vec<Conflict>,
    /// Typed configuration owned by this feature.
    #[serde(default)]
    pub inputs: Vec<InputSpec>,
}

/// A player-facing group of optional features that can be switched together.
///
/// Group state is always derived from the member features' selections and is never stored in
/// selections, presets, or receipts; members remain individually selectable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureGroup {
    /// Stable kebab-case group id.
    pub id: String,
    /// Player-facing title, also used to mark each member.
    pub title: String,
    /// Player-facing explanation shown with the group control.
    pub description: String,
    /// Member feature ids in player-facing order.
    pub members: Vec<String>,
    /// Choice-group alternatives the group picks while it is on, in player-facing order.
    /// Omitted when empty so groups without variants serialize, and therefore freeze and
    /// digest, exactly as before variants existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<FeatureGroupVariant>,
}

/// One alternative of an authored choice group that a feature group picks while it is on.
///
/// Switching the group on picks `option` and deselects the choice group's other options;
/// switching it off restores the choice group's recipe default. Like member state, whether the
/// variant is picked is derived from the selection and never stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureGroupVariant {
    /// Authored `choice_group` id shared by the alternatives.
    pub choice_group: String,
    /// Feature id of the alternative picked while the group is on; never the recipe default.
    pub option: String,
    /// Short player-facing name for the picked alternative; the option title when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
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

/// Game a collection installs.
///
/// Omitted from serialized recipes when it is the default [`CollectionGame::Eet`], so every
/// existing Baldur's Gate recipe freezes and digests exactly as before the field existed.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum CollectionGame {
    /// Enhanced Edition Trilogy built from a BG:EE+SoD source and a BG2:EE source.
    #[default]
    Eet,
    /// A standalone Icewind Dale: Enhanced Edition collection built from one source game.
    Iwdee,
}

impl CollectionGame {
    /// Returns `true` for the default EET collection; used to omit the field when serializing.
    pub fn is_eet(&self) -> bool {
        *self == Self::Eet
    }

    /// Returns `true` when the collection installs one standalone source game.
    pub const fn is_standalone(self) -> bool {
        !matches!(self, Self::Eet)
    }

    /// Returns the player-facing name of the installed game.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Eet => "Enhanced Edition Trilogy",
            Self::Iwdee => "Icewind Dale: Enhanced Edition",
        }
    }

    /// Returns the single source-game role of a standalone collection, or `None` for EET.
    pub const fn standalone_role(self) -> Option<crate::games::GameRole> {
        match self {
            Self::Eet => None,
            Self::Iwdee => Some(crate::games::GameRole::Iwdee),
        }
    }
}

/// Collection-level executable recipe from `collection.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    /// Manifest schema version; this engine understands version 2.
    pub schema: u32,
    /// Game this collection installs. Defaults to EET and is omitted when it is EET.
    #[serde(default, skip_serializing_if = "CollectionGame::is_eet")]
    pub game: CollectionGame,
    /// Folder under `LOCALAPPDATA` that keeps this collection's installation registry and
    /// evidence. A standalone collection must name its own folder; an EET collection never
    /// names one and always uses `Chriz BG Collection`. Omitted when absent, so EET recipes
    /// serialize and digest exactly as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_data_directory: Option<String>,
    /// Required Enhanced Edition game build.
    pub game_build: String,
    /// Explicit ordered installer invocations.
    pub runs: Vec<Run>,
    /// Player-facing semantic features in authored display order.
    #[serde(default)]
    pub features: Vec<Feature>,
    /// Optional player-facing feature groups. Omitted when empty so recipes without groups
    /// serialize, and therefore freeze and digest, exactly as before groups existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub feature_groups: Vec<FeatureGroup>,
}
