//! Immutable update ledgers, player-facing classification, and recipe change coverage.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};

use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zip::ZipArchive;

const LEDGER_SCHEMA: u32 = 1;

/// Authored applicability of a recipe change to saved games.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SaveApplicability {
    CurrentSave,
    BeforeNpcJoin,
    BeforeAreaVisit,
    BeforeEvent,
    NextPlaythrough,
    NewGameOnly,
    Unknown,
}

impl SaveApplicability {
    fn is_conditional(self) -> bool {
        matches!(
            self,
            Self::BeforeNpcJoin | Self::BeforeAreaVisit | Self::BeforeEvent
        )
    }

    fn may_affect_current(self) -> bool {
        matches!(
            self,
            Self::CurrentSave | Self::BeforeNpcJoin | Self::BeforeAreaVisit | Self::BeforeEvent
        )
    }
}

/// Authored importance, deliberately independent of save applicability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Urgency {
    Critical,
    Recommended,
    Optional,
    Informational,
}

/// One human-readable update entry and the semantic recipe subjects it covers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeChange {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub save_applicability: SaveApplicability,
    pub urgency: Urgency,
    #[serde(default)]
    pub condition_note: Option<String>,
    #[serde(default)]
    pub covers: Vec<String>,
}

/// Immutable ledger shipped by one recipe release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeRelease {
    pub schema: u32,
    pub recipe_id: String,
    pub version: String,
    pub published_at: String,
    pub minimum_app_version: String,
    #[serde(default)]
    pub supersedes: Option<String>,
    #[serde(default)]
    pub changes: Vec<RecipeChange>,
}

/// The single presentation state for all releases after an installed snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateDisposition {
    UpToDate,
    DeferredForNextPlaythrough,
    MayAffectCurrentPlaythrough,
    UnknownApplicability,
    AppUpdateRequired,
}

/// Ordered accumulated changes and their presentation state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSummary {
    pub disposition: UpdateDisposition,
    pub latest_version: Option<String>,
    pub minimum_app_version: Option<String>,
    pub changes: Vec<RecipeChange>,
}

/// One changed semantic address from a pair of exact recipe payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeDifference {
    pub subject: String,
    pub cosmetic_only: bool,
}

/// Release-ledger and semantic-diff failure.
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("{path}: {message}")]
    Ledger { path: PathBuf, message: String },
    #[error("invalid update chain: {0}")]
    Chain(String),
    #[error("invalid recipe package: {0}")]
    Package(String),
    #[error("release ledger does not cover: {0}")]
    Coverage(String),
}

/// Load and fully validate one path-bearing release ledger.
pub fn load_release(path: &Path) -> Result<RecipeRelease, UpdateError> {
    let text = std::fs::read_to_string(path).map_err(|error| UpdateError::Ledger {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let release: RecipeRelease = toml::from_str(&text).map_err(|error| UpdateError::Ledger {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    validate_release(&release).map_err(|message| UpdateError::Ledger {
        path: path.to_path_buf(),
        message,
    })?;
    Ok(release)
}

pub(crate) fn validate_release(release: &RecipeRelease) -> Result<(), String> {
    if release.schema != LEDGER_SCHEMA {
        return Err(format!("unsupported ledger schema {}", release.schema));
    }
    if release.recipe_id.trim().is_empty() || release.published_at.trim().is_empty() {
        return Err("recipe id and publication date are required".to_owned());
    }
    let version = parse_version("release version", &release.version)?;
    parse_version("minimum app version", &release.minimum_app_version)?;
    if let Some(previous) = &release.supersedes {
        let previous = parse_version("supersedes version", previous)?;
        if version <= previous {
            return Err("release version must be newer than supersedes".to_owned());
        }
    }
    let mut ids = BTreeSet::new();
    for change in &release.changes {
        if change.id.trim().is_empty() || !ids.insert(change.id.as_str()) {
            return Err(format!("duplicate change id {:?}", change.id));
        }
        if change.title.trim().is_empty()
            || change.summary.trim().is_empty()
            || change.covers.is_empty()
            || change
                .covers
                .iter()
                .any(|subject| subject.trim().is_empty())
        {
            return Err(format!("change {:?} is incomplete", change.id));
        }
        if change.save_applicability.is_conditional()
            && change
                .condition_note
                .as_deref()
                .is_none_or(|note| note.trim().is_empty())
        {
            return Err(format!("conditional change {:?} needs guidance", change.id));
        }
    }
    Ok(())
}

fn parse_version(label: &str, value: &str) -> Result<Version, String> {
    Version::parse(value).map_err(|error| format!("invalid {label} {value:?}: {error}"))
}

/// Classify all immutable ledgers after an installed recipe without inspecting a save.
pub fn classify_updates(
    installed_version: &str,
    running_app_version: &str,
    releases: &[RecipeRelease],
) -> Result<UpdateSummary, UpdateError> {
    let installed =
        parse_version("installed version", installed_version).map_err(UpdateError::Chain)?;
    let running_app =
        parse_version("running app version", running_app_version).map_err(UpdateError::Chain)?;
    if releases.is_empty() {
        return Ok(UpdateSummary {
            disposition: UpdateDisposition::UpToDate,
            latest_version: None,
            minimum_app_version: None,
            changes: Vec::new(),
        });
    }

    let mut ordered = releases
        .iter()
        .map(|release| {
            validate_release(release).map_err(UpdateError::Chain)?;
            Ok((
                parse_version("release version", &release.version).map_err(UpdateError::Chain)?,
                release,
            ))
        })
        .collect::<Result<Vec<_>, UpdateError>>()?;
    ordered.sort_by(|left, right| left.0.cmp(&right.0));

    let recipe_id = ordered[0].1.recipe_id.as_str();
    let mut predecessor = installed;
    let mut seen_changes = BTreeSet::new();
    let mut changes = Vec::new();
    let mut required_app = running_app.clone();
    for (version, release) in &ordered {
        if release.recipe_id != recipe_id {
            return Err(UpdateError::Chain("recipe ids differ".to_owned()));
        }
        if version <= &predecessor {
            return Err(UpdateError::Chain(format!(
                "release {} is not newer than {}",
                release.version, predecessor
            )));
        }
        if release.supersedes.as_deref() != Some(predecessor.to_string().as_str()) {
            return Err(UpdateError::Chain(format!(
                "release {} supersedes {:?}, expected {}",
                release.version, release.supersedes, predecessor
            )));
        }
        predecessor = version.clone();
        let minimum = parse_version("minimum app version", &release.minimum_app_version)
            .map_err(UpdateError::Chain)?;
        required_app = required_app.max(minimum);
        for change in &release.changes {
            if !seen_changes.insert(change.id.as_str()) {
                return Err(UpdateError::Chain(format!(
                    "duplicate change id {:?} across releases",
                    change.id
                )));
            }
            changes.push(change.clone());
        }
    }

    let disposition = if required_app > running_app {
        UpdateDisposition::AppUpdateRequired
    } else if changes
        .iter()
        .any(|change| change.save_applicability == SaveApplicability::Unknown)
    {
        UpdateDisposition::UnknownApplicability
    } else if changes
        .iter()
        .any(|change| change.save_applicability.may_affect_current())
    {
        UpdateDisposition::MayAffectCurrentPlaythrough
    } else if changes.is_empty() {
        UpdateDisposition::UpToDate
    } else {
        UpdateDisposition::DeferredForNextPlaythrough
    };
    Ok(UpdateSummary {
        disposition,
        latest_version: ordered.last().map(|(_, release)| release.version.clone()),
        minimum_app_version: (required_app > running_app).then(|| required_app.to_string()),
        changes,
    })
}

/// Compare exact recipe ZIPs at the semantic addresses required by the update ledger.
pub fn diff_recipe_packages(
    previous: &[u8],
    current: &[u8],
) -> Result<Vec<RecipeDifference>, UpdateError> {
    let previous = read_zip(previous)?;
    let current = read_zip(current)?;
    let previous_collection = parse_collection(&previous)?;
    let current_collection = parse_collection(&current)?;
    let mut differences = Vec::new();

    diff_indexed(
        indexed_array(&previous_collection, "features", "id")?,
        indexed_array(&current_collection, "features", "id")?,
        "feature",
        true,
        &mut differences,
    );
    diff_inputs(&previous_collection, &current_collection, &mut differences)?;
    diff_indexed(
        indexed_array(&previous_collection, "runs", "run_id")?,
        indexed_array(&current_collection, "runs", "run_id")?,
        "run",
        false,
        &mut differences,
    );
    diff_toml_files(
        &previous,
        &current,
        "artifacts/",
        "artifact",
        &mut differences,
    )?;
    diff_raw_files(
        &previous,
        &current,
        "game-builds/",
        "game-profile",
        &mut differences,
    );
    differences.sort_by(|left, right| left.subject.cmp(&right.subject));
    Ok(differences)
}

/// Require every detected semantic address, including cosmetic-only ones, in the ledger.
pub fn ensure_change_coverage(
    differences: &[RecipeDifference],
    release: &RecipeRelease,
) -> Result<(), UpdateError> {
    validate_release(release).map_err(UpdateError::Coverage)?;
    let covered = release
        .changes
        .iter()
        .flat_map(|change| change.covers.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let missing = differences
        .iter()
        .filter(|difference| !covered.contains(difference.subject.as_str()))
        .map(|difference| difference.subject.as_str())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(UpdateError::Coverage(missing.join(", ")))
    }
}

fn read_zip(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, UpdateError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| UpdateError::Package(error.to_string()))?;
    let mut files = BTreeMap::new();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| UpdateError::Package(error.to_string()))?;
        if entry.is_dir() {
            continue;
        }
        let path = entry.name().replace('\\', "/");
        if !safe_relative(&path) || files.contains_key(&path) {
            return Err(UpdateError::Package(format!(
                "unsafe or duplicate path {path:?}"
            )));
        }
        let mut contents = Vec::new();
        entry
            .read_to_end(&mut contents)
            .map_err(|error| UpdateError::Package(error.to_string()))?;
        files.insert(path, contents);
    }
    Ok(files)
}

fn safe_relative(value: &str) -> bool {
    !value.contains(':')
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn parse_collection(files: &BTreeMap<String, Vec<u8>>) -> Result<toml::Value, UpdateError> {
    let bytes = files
        .get("collection.toml")
        .ok_or_else(|| UpdateError::Package("collection.toml is missing".to_owned()))?;
    let text = std::str::from_utf8(bytes)
        .map_err(|error| UpdateError::Package(format!("collection.toml: {error}")))?;
    toml::from_str(text).map_err(|error| UpdateError::Package(format!("collection.toml: {error}")))
}

fn indexed_array(
    collection: &toml::Value,
    field: &str,
    id_field: &str,
) -> Result<BTreeMap<String, toml::Value>, UpdateError> {
    collection
        .get(field)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .map(|value| {
            let id = value
                .get(id_field)
                .and_then(toml::Value::as_str)
                .ok_or_else(|| UpdateError::Package(format!("{field} entry lacks {id_field}")))?;
            Ok((id.to_owned(), value.clone()))
        })
        .collect()
}

fn diff_indexed(
    previous: BTreeMap<String, toml::Value>,
    current: BTreeMap<String, toml::Value>,
    prefix: &str,
    feature: bool,
    differences: &mut Vec<RecipeDifference>,
) {
    for id in previous
        .keys()
        .chain(current.keys())
        .collect::<BTreeSet<_>>()
    {
        let left = previous.get(id);
        let right = current.get(id);
        let (left_compare, right_compare) = if feature {
            (
                left.cloned().map(strip_feature_inputs),
                right.cloned().map(strip_feature_inputs),
            )
        } else {
            (left.cloned(), right.cloned())
        };
        if left_compare != right_compare {
            let cosmetic_only = feature
                && left_compare.is_some()
                && right_compare.is_some()
                && left_compare.map(strip_feature_text) == right_compare.map(strip_feature_text);
            differences.push(RecipeDifference {
                subject: format!("{prefix}:{id}"),
                cosmetic_only,
            });
        }
    }
}

fn strip_feature_inputs(mut value: toml::Value) -> toml::Value {
    if let Some(table) = value.as_table_mut() {
        table.remove("inputs");
    }
    value
}

fn strip_feature_text(mut value: toml::Value) -> toml::Value {
    if let Some(table) = value.as_table_mut() {
        for field in ["title", "description", "unavailable_reason"] {
            table.remove(field);
        }
    }
    value
}

fn diff_inputs(
    previous: &toml::Value,
    current: &toml::Value,
    differences: &mut Vec<RecipeDifference>,
) -> Result<(), UpdateError> {
    let previous = inputs(previous)?;
    let current = inputs(current)?;
    for id in previous
        .keys()
        .chain(current.keys())
        .collect::<BTreeSet<_>>()
    {
        if previous.get(id) != current.get(id) {
            differences.push(RecipeDifference {
                subject: format!("input:{id}"),
                cosmetic_only: false,
            });
        }
    }
    Ok(())
}

fn inputs(collection: &toml::Value) -> Result<BTreeMap<String, toml::Value>, UpdateError> {
    let features = indexed_array(collection, "features", "id")?;
    let mut inputs = BTreeMap::new();
    for (feature_id, feature) in features {
        for input in feature
            .get("inputs")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            let input_id = input
                .get("id")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| UpdateError::Package("input lacks id".to_owned()))?;
            inputs.insert(format!("{feature_id}/{input_id}"), input.clone());
        }
    }
    Ok(inputs)
}

fn diff_toml_files(
    previous: &BTreeMap<String, Vec<u8>>,
    current: &BTreeMap<String, Vec<u8>>,
    directory: &str,
    prefix: &str,
    differences: &mut Vec<RecipeDifference>,
) -> Result<(), UpdateError> {
    let parse = |bytes: &[u8]| -> Result<toml::Value, UpdateError> {
        let text =
            std::str::from_utf8(bytes).map_err(|error| UpdateError::Package(error.to_string()))?;
        toml::from_str(text).map_err(|error| UpdateError::Package(error.to_string()))
    };
    for path in matching_paths(previous, current, directory) {
        let left = previous.get(path).map(|bytes| parse(bytes)).transpose()?;
        let right = current.get(path).map(|bytes| parse(bytes)).transpose()?;
        if left != right {
            let id = right
                .as_ref()
                .or(left.as_ref())
                .and_then(|value| value.get("id"))
                .and_then(toml::Value::as_str)
                .unwrap_or_else(|| path.trim_start_matches(directory).trim_end_matches(".toml"));
            differences.push(RecipeDifference {
                subject: format!("{prefix}:{id}"),
                cosmetic_only: false,
            });
        }
    }
    Ok(())
}

fn diff_raw_files(
    previous: &BTreeMap<String, Vec<u8>>,
    current: &BTreeMap<String, Vec<u8>>,
    directory: &str,
    prefix: &str,
    differences: &mut Vec<RecipeDifference>,
) {
    for path in matching_paths(previous, current, directory) {
        if previous.get(path) != current.get(path) {
            differences.push(RecipeDifference {
                subject: format!("{prefix}:{}", path.trim_start_matches(directory)),
                cosmetic_only: false,
            });
        }
    }
}

fn matching_paths<'a>(
    previous: &'a BTreeMap<String, Vec<u8>>,
    current: &'a BTreeMap<String, Vec<u8>>,
    directory: &str,
) -> BTreeSet<&'a str> {
    previous
        .keys()
        .chain(current.keys())
        .filter(|path| path.starts_with(directory) && path.ends_with(".toml"))
        .map(String::as_str)
        .collect()
}
