//! Whole-mod updates: one installed mod, such as BuffBot, brought to a newer release in a
//! finished installation. WeiDU installs the mod's installed components again from the new
//! release, as the mod's own update path does. The update is a transaction of the hotpatch
//! history, so there is one chain: Undo always takes back the latest change, and the hotpatch
//! rule takes the game folder as the update left it.
//!
//! Left to itself, WeiDU would take every component after the updated ones off and put it back:
//! voice packs a player added, boss fights from a hotpatch, whatever stands there. So for the run
//! the mod list ends with the mod. WeiDU takes off only the old version and installs the new one,
//! and the rows after it come back as they were (see [`visible_log`]). Saves, the profile and the
//! original installation record are never written.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::catalog::{Catalog, ModUpdate, UpdateSource};
use super::transaction::{self, Applied, History, Package, Prepared};
use super::{gate, Context};
use crate::cli::{PlayerMods, VerifiedInstalledBaseline};
use crate::digest::sha256_bytes;
use crate::game_files::{self, GameFiles};
use crate::manifest::{ArchiveLimitsSpec, Artifact};
use crate::patches::files;
use crate::preflight::{PreflightHost, SystemPreflight};
use crate::receipt::LogComponentReceipt;
use crate::weidu::log::{
    normalize_tp2_key, parse_active_entries, parse_terminal_statuses, DebugStatus, LogEntry,
};

const LOG_LIMIT: u64 = 16 * 1024 * 1024;
const PAYLOAD_LIMIT: u64 = 64 * 1024 * 1024;
const TP2_LIMIT: u64 = 4 * 1024 * 1024;
const LIST_LIMIT: u64 = 4 * 1024 * 1024;
const COPY_LIMIT: u64 = 512 * 1024 * 1024;

/// The release an update installs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseRecord {
    pub tag: String,
    pub url: String,
    pub length: u64,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
}

/// What a whole-mod update transaction records next to its package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateRecord {
    pub mod_id: String,
    pub name: String,
    /// The installed version, as the mod list named it.
    pub from_version: String,
    pub to_version: String,
    pub release: ReleaseRecord,
    /// The mod's installer, as the mod list names it (normalized).
    pub tp2: String,
    /// The language number its rows have after the update.
    pub language: u32,
    /// Lowercase folder WeiDU keeps the mod's uninstall copies in.
    pub backup_folder: String,
    /// The whole mod list after the update, in order.
    pub rows: Vec<LogComponentReceipt>,
}

/// What Updates shows for one mod of one installation. Checking never writes the game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdatePreview {
    pub install_id: String,
    pub mod_id: String,
    pub title: String,
    /// `available`, `up-to-date`, `not-installed`, `refused`, `unavailable` or
    /// `needs-recovery`.
    pub state: String,
    pub detail: String,
    pub installed_version: Option<String>,
    pub target_version: Option<String>,
    pub release_title: Option<String>,
    pub release_url: Option<String>,
    /// Why the update is refused.
    pub refusals: Vec<String>,
    pub review_token: Option<String>,
    /// The latest change is an update of this mod and can be undone.
    pub can_undo: bool,
    pub undo_title: Option<String>,
    pub can_restore: bool,
    pub full_backup_bytes: String,
    pub save_backup_bytes: String,
    pub available_bytes: String,
    pub backup_path: String,
}

/// Whether `name` lies in what the update `p` may change: the mod's folder and WeiDU's records of
/// it, `override/`, the game language's dialog files, WeiDU's own files and logs, and the LuaJIT
/// files of BuffBot's loader helper.
pub(super) fn writable(name: &str, p: &Prepared, update: &UpdateRecord) -> bool {
    let below = |folder: &str| name == folder || name.starts_with(&format!("{folder}/"));
    let language = p.language.to_ascii_lowercase();
    matches!(
        name,
        "weidu.log"
            | "weidu.conf"
            | "weidu_external"
            | "weidu_external/backup"
            | "weidu_external/data"
            | "lua51.dll"
            | "luaprovider.dll"
    ) || below("override")
        || below(&p.package.mod_folder)
        || below(&update.backup_folder)
        || below(&format!("weidu_external/data/{}", p.package.mod_folder))
        || name == format!("lang/{language}/dialog.tlk")
        || name == format!("lang/{language}/dialogf.tlk")
        // A WeiDU log such as `setup-buffbot.debug` beside the game.
        || (!name.contains('/') && name.ends_with(".debug"))
}

/// The installed rows of one mod, from the current mod list.
#[derive(Debug, Clone)]
pub(super) struct Installed {
    /// Every active row, in order.
    pub all: Vec<LogEntry>,
    /// Indexes into `all` of the mod's rows, in order.
    pub own: Vec<usize>,
    pub language: u32,
    pub version: String,
}

impl Installed {
    fn components(&self) -> Vec<u32> {
        self.own
            .iter()
            .map(|index| self.all[*index].component)
            .collect()
    }
}

/// The installed version a mod-list row names, as in `// BuffBot: Main: v1.12.6`.
fn row_version(entry: &LogEntry) -> Option<String> {
    let annotation = entry.annotation.as_deref()?;
    let (_, version) = annotation.rsplit_once(": ")?;
    let version = version.trim();
    (!version.is_empty()).then(|| plain_version(version))
}

fn plain_version(version: &str) -> String {
    version.trim().trim_start_matches(['v', 'V']).to_owned()
}

fn installed(game: &Path, tp2_key: &str) -> Result<Option<Installed>, String> {
    let all = parse_active_entries(&String::from_utf8_lossy(&files::read(
        &game.join("WeiDU.log"),
        LOG_LIMIT,
    )?))
    .map_err(files::err)?;
    let own = all
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.tp2_key == tp2_key)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let Some(first) = own.first() else {
        return Ok(None);
    };
    let language = all[*first].language;
    let mut versions = Vec::new();
    for index in &own {
        let version = row_version(&all[*index]).unwrap_or_else(|| "unknown".into());
        if !versions.contains(&version) {
            versions.push(version);
        }
    }
    Ok(Some(Installed {
        all,
        own,
        language,
        version: versions.join(" and "),
    }))
}

/// The last package CEBG recorded for this mod. Availability compares our own
/// package history; a player's mod-list annotation never withholds a decided update.
pub(super) fn recorded_version<'a>(
    baseline: &'a VerifiedInstalledBaseline,
    mod_id: &str,
) -> Option<&'a str> {
    let plan = baseline
        .receipt
        .installed_plan
        .as_ref()
        .unwrap_or(&baseline.receipt.plan);
    let run = plan.runs.iter().rev().find(|run| run.mod_id == mod_id)?;
    if let Some(replacement) = baseline.replacements.get(&run.run_id) {
        return Some(&replacement.version);
    }
    baseline
        .manifest
        .artifacts
        .get(&run.artifact_id)
        .map(|artifact| artifact.version.as_str())
}

/// Numeric parts of a version such as `1.12.6`, and whether it is a pre-release.
fn version_key(version: &str) -> Option<(Vec<u64>, bool)> {
    let (core, pre) = match version.split_once('-') {
        Some((core, _)) => (core, true),
        None => (version, false),
    };
    let parts = core
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    Some((parts, pre))
}

/// Whether `target` is a newer release than `installed`.
fn newer(target: &str, installed: &str) -> bool {
    match (version_key(target), version_key(installed)) {
        (Some((mut a, a_pre)), Some((mut b, b_pre))) => {
            let length = a.len().max(b.len());
            a.resize(length, 0);
            b.resize(length, 0);
            a > b || (a == b && b_pre && !a_pre)
        }
        // An unreadable version cannot be ordered; only the same text counts as current.
        _ => !target.eq_ignore_ascii_case(installed),
    }
}

/// The tokens of a TP2 file without comments: words, and strings in `~`, `"`, `%` or `~~~~~`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Text(String),
}

fn tp2_tokens(text: &str) -> Vec<Token> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let rest = &text[at..];
        if bytes[at].is_ascii_whitespace() {
            at += 1;
        } else if rest.starts_with("//") {
            at += rest.find('\n').unwrap_or(rest.len());
        } else if rest.starts_with("/*") {
            at += rest.find("*/").map_or(rest.len(), |end| end + 2);
        } else if let Some(inner) = rest.strip_prefix("~~~~~") {
            let end = inner.find("~~~~~").unwrap_or(inner.len());
            tokens.push(Token::Text(inner[..end].to_owned()));
            at += (5 + end + 5).min(rest.len());
        } else if let Some(quote) = ['~', '"', '%'].into_iter().find(|q| rest.starts_with(*q)) {
            let end = rest[1..].find(quote).map_or(rest.len() - 1, |end| end);
            tokens.push(Token::Text(rest[1..1 + end].to_owned()));
            at += (1 + end + 1).min(rest.len());
        } else {
            let end = rest
                .find(|c: char| c.is_ascii_whitespace() || matches!(c, '~' | '"'))
                .unwrap_or(rest.len())
                .max(1);
            tokens.push(Token::Word(rest[..end].to_owned()));
            at += end;
        }
    }
    tokens
}

/// What the update needs from a mod's TP2: its language folders in order, its version, where
/// WeiDU keeps its uninstall copies, and its designated component numbers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Tp2Facts {
    languages: Vec<String>,
    version: Option<String>,
    backup: Option<String>,
    designated: BTreeSet<u32>,
}

fn tp2_facts(text: &str) -> Tp2Facts {
    let tokens = tp2_tokens(text);
    let mut facts = Tp2Facts::default();
    for (index, token) in tokens.iter().enumerate() {
        let Token::Word(word) = token else {
            continue;
        };
        let next = tokens.get(index + 1);
        match (word.as_str(), next) {
            ("LANGUAGE", Some(Token::Text(_))) => {
                if let Some(Token::Text(folder)) = tokens.get(index + 2) {
                    facts.languages.push(folder.to_ascii_lowercase());
                }
            }
            ("VERSION", Some(Token::Text(version))) if facts.version.is_none() => {
                facts.version = Some(plain_version(version));
            }
            ("BACKUP", Some(Token::Text(folder))) if facts.backup.is_none() => {
                facts.backup = Some(
                    folder
                        .replace('\\', "/")
                        .trim_matches('/')
                        .to_ascii_lowercase(),
                );
            }
            ("DESIGNATED", Some(Token::Word(number))) => {
                if let Ok(number) = number.parse() {
                    facts.designated.insert(number);
                }
            }
            _ => {}
        }
    }
    facts
}

/// The installed mod's language in the new release's numbering, matched by its folder name.
fn new_language(
    old: &Tp2Facts,
    new: &Tp2Facts,
    installed: u32,
    title: &str,
) -> Result<u32, String> {
    let folder = old.languages.get(installed as usize).ok_or_else(|| {
        format!("The installed {title} installer's language list could not be read, so the update could not keep its language.")
    })?;
    new.languages
        .iter()
        .position(|candidate| candidate == folder)
        .map(|position| position as u32)
        .ok_or_else(|| format!("The new {title} release has no {folder} translation."))
}

/// The folder of a mod whose installer is `tp2`: its first folder, or `name` for a
/// `setup-name.tp2` at the top of the game folder.
fn tp2_folder(tp2: &str) -> Option<String> {
    let tp2 = normalize_tp2_key(tp2);
    if let Some((folder, _)) = tp2.split_once('/') {
        return Some(folder.to_owned());
    }
    tp2.strip_prefix("setup-")
        .and_then(|rest| rest.strip_suffix(".tp2"))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
}

/// Lowercase game-folder names an uninstall list `UNINSTALL.<n>` names.
fn uninstall_names(bytes: &[u8]) -> BTreeSet<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(|line| line.trim().replace('\\', "/").to_ascii_lowercase())
        .filter(|name| !name.is_empty() && files::safe_relative(name).is_ok())
        .collect()
}

fn uninstall_list(backup: &str, component: u32) -> String {
    format!("{backup}/{component}/uninstall.{component}")
}

/// Every file the components of one mod wrote, by WeiDU's uninstall lists in `folder`.
fn owned_files(
    game: &GameFiles,
    read: impl Fn(&str) -> Result<Vec<u8>, String>,
    backup: &str,
    components: &[u32],
) -> Result<BTreeSet<String>, String> {
    let mut names = BTreeSet::new();
    for component in components {
        let list = uninstall_list(backup, *component);
        if game.files.contains_key(&list) {
            names.extend(uninstall_names(&read(&list)?));
        }
    }
    Ok(names)
}

/// Everything the check found for one installation and the update of one mod.
pub(super) struct Assessment {
    pub refusals: Vec<String>,
    pub baseline: Option<VerifiedInstalledBaseline>,
    pub installed: Option<Installed>,
    pub current: Option<GameFiles>,
}

fn assess(ctx: &Context<'_>, entry: &ModUpdate) -> Assessment {
    // Other mods' rows do not matter here: the update leaves them alone, wherever they stand.
    let baseline = crate::cli::verify_installed_baseline(ctx.record, PlayerMods::Unchecked)
        .map_err(|error| error.to_string());
    assess_with(ctx, entry, baseline)
}

/// [`assess`] with the installation's verified recipe and receipt already read.
pub(super) fn assess_with(
    ctx: &Context<'_>,
    entry: &ModUpdate,
    baseline: Result<VerifiedInstalledBaseline, String>,
) -> Assessment {
    let mut assessment = Assessment {
        refusals: Vec::new(),
        baseline: None,
        installed: None,
        current: None,
    };
    let baseline = match baseline {
        Ok(baseline) => baseline,
        Err(error) => {
            assessment.refusals.push(error);
            return assessment;
        }
    };
    let Some(installer) = baseline.manifest.mods.get(&entry.mod_id) else {
        // The collection never had the mod, so it is not installed.
        assessment.baseline = Some(baseline);
        return assessment;
    };
    let tp2_key = normalize_tp2_key(&installer.tp2);
    match installed(&ctx.game(), &tp2_key) {
        Ok(Some(found)) => assessment.installed = Some(found),
        Ok(None) => {
            assessment.baseline = Some(baseline);
            return assessment;
        }
        Err(error) => {
            assessment.refusals.push(error);
            return assessment;
        }
    }
    // Both older records compare the mod list byte for byte, so the game's checks would stop
    // after an update. Each can be undone under Updates.
    if baseline.description_fix {
        assessment.refusals.push(format!(
            "An earlier game fix is recorded for this mod list. Undo it under Updates before updating {}.",
            entry.title
        ));
    }
    if baseline.upgraded {
        assessment.refusals.push(format!(
            "An earlier {} update is recorded for this mod list. Undo it under Updates first.",
            entry.title
        ));
    }
    let found = assessment.installed.as_ref().expect("set above");
    for component in found.components() {
        let asks = installer
            .components
            .iter()
            .find(|declared| declared.id == component)
            .is_some_and(|declared| declared.stdin.is_some() || !declared.prompts.is_empty());
        let scripted = baseline.receipt.plan.runs.iter().any(|run| {
            run.mod_id == entry.mod_id
                && run.components.contains(&component)
                && (!run.args.is_empty() || !run.prompt_scripts.is_empty())
        });
        if asks || scripted {
            assessment.refusals.push(format!(
                "{} component {component} asks questions while installing, so it cannot be updated unattended.",
                entry.title
            ));
        }
    }
    match game_files::capture(&ctx.game()) {
        Ok(now) => assessment.current = Some(now),
        Err(error) => assessment
            .refusals
            .push(format!("The game folder could not be read: {error}")),
    }
    assessment.baseline = Some(baseline);
    assessment
}

/// The release an update installs, with its package as a recipe artifact.
pub(super) struct Release {
    pub record: ReleaseRecord,
    pub version: String,
    pub artifact: Artifact,
}

/// Bigger than any release so far, so that a growing mod still fits; the recipe's own limits
/// are kept where they are larger.
fn update_limits(installed: &ArchiveLimitsSpec) -> ArchiveLimitsSpec {
    ArchiveLimitsSpec {
        max_depth: installed.max_depth.max(8),
        max_entries: installed.max_entries.max(2048),
        max_entry_uncompressed_bytes: installed.max_entry_uncompressed_bytes.max(16 << 20),
        max_total_uncompressed_bytes: installed.max_total_uncompressed_bytes.max(64 << 20),
        max_compression_ratio: installed.max_compression_ratio.max(100),
    }
}

/// The release `entry` offers now, for an installation whose recipe pins `installed`.
pub(super) fn release(entry: &ModUpdate, installed: &Artifact) -> Result<Release, String> {
    match &entry.source {
        UpdateSource::Pinned(artifact) => Ok(Release {
            record: ReleaseRecord {
                tag: artifact.source.reference.clone(),
                url: artifact.source.url.clone(),
                length: artifact
                    .source
                    .expected_length
                    .ok_or("The release has no pinned length.")?,
                sha256: artifact.source.sha256.to_ascii_lowercase(),
                title: None,
                page: None,
            },
            version: plain_version(&artifact.version),
            artifact: artifact.as_ref().clone(),
        }),
        UpdateSource::LatestRelease { repository, asset } => {
            let latest = super::github::latest_release(repository, asset)?;
            let mut artifact = installed.clone();
            let version = plain_version(&latest.tag);
            artifact.id = format!("{}-{version}", entry.mod_id);
            artifact.version = version.clone();
            artifact.source.url = latest.asset_url.clone();
            artifact.source.reference = latest.tag.clone();
            artifact.source.expected_filename = Some(latest.asset_name.clone());
            artifact.source.expected_length = Some(latest.asset_length);
            artifact.source.sha256 = latest.sha256.clone();
            for host in super::github::DOWNLOAD_HOSTS {
                if !artifact
                    .source
                    .redirect_hosts
                    .iter()
                    .any(|known| known == host)
                {
                    artifact.source.redirect_hosts.push((*host).to_owned());
                }
            }
            artifact.archive.limits = update_limits(&installed.archive.limits);
            Ok(Release {
                record: ReleaseRecord {
                    tag: latest.tag,
                    url: latest.asset_url,
                    length: latest.asset_length,
                    sha256: latest.sha256,
                    title: latest.title,
                    page: latest.page,
                },
                version,
                artifact,
            })
        }
    }
}

/// The token a player confirms: it changes with the catalog, the release, the installation's
/// records and every byte of its game folder.
fn review_token(
    ctx: &Context<'_>,
    catalog: &Catalog,
    entry: &ModUpdate,
    release: &ReleaseRecord,
    current: &GameFiles,
    head: Option<&str>,
) -> Result<String, String> {
    Ok(sha256_bytes(
        &serde_json::to_vec(&(
            &catalog.digest,
            "update",
            &entry.mod_id,
            release,
            &ctx.record.install_id,
            &ctx.record.receipt_sha256,
            sha256_bytes(&serde_json::to_vec(current).map_err(files::err)?),
            head,
        ))
        .map_err(files::err)?,
    ))
}

/// Check one installation for an update of `mod_id`. Reads the game folder and asks where the
/// newest release is; never writes the game.
pub fn inspect(
    ctx: &Context<'_>,
    catalog: &Catalog,
    mod_id: &str,
) -> Result<ModUpdatePreview, String> {
    crate::patches::launch_guard(&ctx.record.managed_root)?;
    crate::mod_upgrades::launch_guard(&ctx.record.managed_root)?;
    super::validate_context(ctx)?;
    let entry = catalog
        .update(mod_id)
        .ok_or("This app offers no update for that mod.")?;
    let history = History::load(&ctx.record.managed_root, &ctx.record.install_id)?;
    let mut preview = ModUpdatePreview {
        install_id: ctx.record.install_id.clone(),
        mod_id: entry.mod_id.clone(),
        title: entry.title.clone(),
        state: "refused".into(),
        detail: String::new(),
        installed_version: None,
        target_version: None,
        release_title: None,
        release_url: None,
        refusals: Vec::new(),
        review_token: None,
        can_undo: false,
        undo_title: None,
        can_restore: false,
        full_backup_bytes: "0".into(),
        save_backup_bytes: "0".into(),
        available_bytes: SystemPreflight
            .available_space(ctx.app_data)
            .map_err(files::err)?
            .to_string(),
        backup_path: ctx.backups().display().to_string(),
    };
    if let Some(unfinished) = history.unfinished() {
        preview.state = "needs-recovery".into();
        preview.detail = "A game update did not finish. Restore the game files before playing. Your saves and the original installation record are untouched.".into();
        preview.can_restore = transaction::restorable(ctx, catalog, unfinished).is_ok();
        return Ok(preview);
    }
    let assessment = assess(ctx, entry);
    if let (Some(target), Some(current)) = (history.undo_target(), &assessment.current) {
        if target
            .update
            .as_ref()
            .is_some_and(|update| update.mod_id == entry.mod_id)
        {
            preview.undo_title = Some(target.title(catalog));
            preview.can_undo = transaction::undoable(ctx, catalog, target, current).is_ok();
        }
    }
    let Some(baseline) = &assessment.baseline else {
        preview.refusals = assessment.refusals;
        preview.detail = format!(
            "{} cannot be updated in this game. Your game has not been changed.",
            entry.title
        );
        return Ok(preview);
    };
    let Some(found) = &assessment.installed else {
        preview.state = "not-installed".into();
        preview.detail = format!(
            "{} is not installed in this game. This update does not add it.",
            entry.title
        );
        return Ok(preview);
    };
    preview.installed_version = Some(found.version.clone());
    let installer = &baseline.manifest.mods[&entry.mod_id];
    let pinned = baseline
        .manifest
        .artifacts
        .get(&installer.artifact_id)
        .ok_or("The installation's recipe has no package for the mod.")?;
    let release = match release(entry, pinned) {
        Ok(release) => release,
        Err(error) => {
            preview.state = "unavailable".into();
            preview.detail = format!(
                "The newest {} release could not be checked: {error}",
                entry.title
            );
            return Ok(preview);
        }
    };
    preview.target_version = Some(release.version.clone());
    preview.release_title = release.record.title.clone();
    preview.release_url = release.record.page.clone();
    if recorded_version(baseline, &entry.mod_id)
        .is_some_and(|version| !newer(&release.version, version))
    {
        preview.state = "up-to-date".into();
        preview.detail = format!(
            "{} {} was installed through this app. No newer release is available.",
            entry.title,
            recorded_version(baseline, &entry.mod_id).unwrap_or("unknown")
        );
        return Ok(preview);
    }
    if !assessment.refusals.is_empty() {
        preview.refusals = assessment.refusals;
        preview.detail = format!(
            "The {} update is not offered for this game. Your game has not been changed.",
            entry.title
        );
        return Ok(preview);
    }
    let current = assessment
        .current
        .ok_or("The game folder could not be read.")?;
    preview.full_backup_bytes = current
        .files
        .values()
        .map(|identity| identity.length)
        .sum::<u64>()
        .to_string();
    preview.save_backup_bytes = files::inventory(&ctx.record.managed_save_root, true)
        .map(|inventory| files::total(&inventory).to_string())
        .unwrap_or_else(|_| "unavailable".into());
    preview.review_token = Some(review_token(
        ctx,
        catalog,
        entry,
        &release.record,
        &current,
        history.head().as_deref(),
    )?);
    preview.state = "available".into();
    preview.detail = entry.summary.clone();
    Ok(preview)
}

pub fn apply(
    ctx: &Context<'_>,
    catalog: &Catalog,
    mod_id: &str,
    token: &str,
    full_backup: bool,
    save_backup: bool,
) -> Result<ModUpdatePreview, String> {
    apply_with_progress(
        ctx,
        catalog,
        mod_id,
        token,
        full_backup,
        save_backup,
        &|_| {},
    )
}

/// The release's files below the folder it publishes, by their name in the game folder.
fn release_payload(
    root: &Path,
    mod_folder: &str,
    tp2: &str,
) -> Result<BTreeMap<String, PathBuf>, String> {
    let mut payload = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root.join(mod_folder)).follow_links(false) {
        let entry = entry.map_err(files::err)?;
        if entry.file_type().is_dir() {
            continue;
        }
        if !entry.file_type().is_file() {
            return Err("The release contains a link.".into());
        }
        let name = entry
            .path()
            .strip_prefix(root)
            .map_err(files::err)?
            .to_string_lossy()
            .replace('\\', "/");
        files::safe_relative(&name)?;
        if name
            .to_ascii_lowercase()
            .starts_with(&format!("{}/backup/", mod_folder.to_ascii_lowercase()))
        {
            return Err("The release contains a backup folder.".into());
        }
        payload.insert(name, entry.path().to_path_buf());
    }
    if !payload.keys().any(|name| name.eq_ignore_ascii_case(tp2)) {
        return Err("The release is missing its installer.".into());
    }
    Ok(payload)
}

#[allow(clippy::too_many_arguments)]
pub fn apply_with_progress(
    ctx: &Context<'_>,
    catalog: &Catalog,
    mod_id: &str,
    token: &str,
    full_backup: bool,
    save_backup: bool,
    progress: crate::operation_progress::ProgressSink<'_>,
) -> Result<ModUpdatePreview, String> {
    crate::operation_progress::stage(progress, "checking");
    let root = &ctx.record.managed_root;
    let _lock = ctx.locks.try_acquire(root).map_err(files::err)?;
    crate::patches::launch_guard(root)?;
    crate::mod_upgrades::launch_guard(root)?;
    super::validate_context(ctx)?;
    let entry = catalog
        .update(mod_id)
        .ok_or("This app offers no update for that mod.")?;
    let history = History::load(root, &ctx.record.install_id)?;
    if history.unfinished().is_some() {
        return Err("A game update did not finish. Restore it before applying another.".into());
    }
    let assessment = assess(ctx, entry);
    if !assessment.refusals.is_empty() {
        return Err(assessment.refusals.join(" "));
    }
    let baseline = assessment
        .baseline
        .ok_or("The installation could not be verified.")?;
    let found = assessment
        .installed
        .ok_or_else(|| format!("{} is not installed in this game.", entry.title))?;
    let current = assessment
        .current
        .ok_or("The game folder could not be read.")?;
    let installer = baseline.manifest.mods[&entry.mod_id].clone();
    let pinned = baseline
        .manifest
        .artifacts
        .get(&installer.artifact_id)
        .ok_or("The installation's recipe has no package for the mod.")?;
    let release = release(entry, pinned)?;
    if recorded_version(&baseline, &entry.mod_id)
        .is_some_and(|version| !newer(&release.version, version))
    {
        return Err(format!("{} is already up to date.", entry.title));
    }
    if review_token(
        ctx,
        catalog,
        entry,
        &release.record,
        &current,
        history.head().as_deref(),
    )? != token
    {
        return Err(
            "The game or the newest release changed since you checked. Check again before updating."
                .into(),
        );
    }
    let language = super::language_folder(&ctx.game())?;
    crate::patches::close_check_in(
        &ctx.game(),
        &language,
        &transaction::guarded(&current, &language),
        None,
    )?;
    let id = format!(
        "{:020}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(files::err)?
            .as_nanos()
    );
    let dir = ctx.state().join(&id);
    let game_bytes = current
        .files
        .values()
        .map(|identity| identity.length)
        .sum::<u64>();
    let profile_bytes = if save_backup {
        files::total(&files::inventory(&ctx.record.managed_save_root, true)?)
    } else {
        0
    };
    let required = (if full_backup { game_bytes } else { 0 })
        .saturating_add(profile_bytes)
        .saturating_add(512 * 1024 * 1024);
    if SystemPreflight
        .available_space(ctx.app_data)
        .map_err(files::err)?
        < required
        || SystemPreflight.available_space(root).map_err(files::err)? < 512 * 1024 * 1024
    {
        return Err("Not enough free space for the selected backups and the update.".into());
    }
    files::create(&dir.join("identity"), ctx.record.install_id.as_bytes())?;
    crate::operation_progress::stage(progress, "downloading");
    // No game writes happen here, even if a download fails.
    let extracted = transaction::unpack(ctx, &release.artifact)?;
    let plan = plan(
        ctx, entry, &baseline, &found, &release, &extracted, language,
    )?;
    let (tool, tool_sha256) = transaction::weidu_tool(
        ctx,
        &baseline.manifest,
        &baseline.manifest.mods[&entry.mod_id].weidu_artifact_id,
    )?;
    let backup = ctx.backups().join(&id);
    files::create(&backup.join("identity"), ctx.record.install_id.as_bytes())?;
    if full_backup {
        files::copy_tree_with_progress(
            &ctx.game(),
            &backup.join("game"),
            false,
            "backup-game",
            progress,
        )?;
    }
    if save_backup {
        files::copy_tree_with_progress(
            &ctx.record.managed_save_root,
            &backup.join("profile"),
            true,
            "backup-saves",
            progress,
        )?;
    }
    crate::operation_progress::stage(progress, "backup-files");
    let prepared = prepare(
        ctx,
        catalog,
        &dir,
        &plan,
        &current,
        &tool,
        tool_sha256,
        history.head(),
    )?;
    crate::operation_progress::stage(progress, "applying");
    if let Err(error) = execute(ctx, catalog, &dir, &prepared, plan.new_version.as_deref()) {
        crate::operation_progress::stage(progress, "restoring");
        return Err(transaction::fail_and_restore(
            ctx,
            catalog,
            &dir,
            error,
            &format!("The {} update", entry.title),
        ));
    }
    crate::operation_progress::stage(progress, "verifying");
    inspect(ctx, catalog, mod_id)
}

/// What an update installs and how, decided before anything is written.
pub(super) struct Plan {
    pub package: Package,
    pub update: UpdateRecord,
    /// `weidu.conf`'s language folder.
    pub language: String,
    /// The release's files, by their name in the game folder.
    pub payload: BTreeMap<String, PathBuf>,
    /// The version declared by the release, retained as package metadata.
    pub new_version: Option<String>,
}

/// Decide the update of `found` to `release`, unpacked at `extracted`: the mod's folder, its
/// language in the new installer, the components and the mod list the run must leave.
pub(super) fn plan(
    ctx: &Context<'_>,
    entry: &ModUpdate,
    baseline: &VerifiedInstalledBaseline,
    found: &Installed,
    release: &Release,
    extracted: &Path,
    language: String,
) -> Result<Plan, String> {
    let installer = baseline
        .manifest
        .mods
        .get(&entry.mod_id)
        .ok_or("The installation's recipe has no installer for the mod.")?;
    let tp2 = installer.tp2.replace('\\', "/");
    let mod_folder = tp2_folder(&tp2).ok_or("The mod's installer is not in a mod folder.")?;
    let [root] = release.artifact.archive.publish_roots.as_slice() else {
        return Err("The release does not publish exactly the mod's folder.".into());
    };
    if !root.eq_ignore_ascii_case(&mod_folder) {
        return Err("The release does not publish exactly the mod's folder.".into());
    }
    let old_facts = tp2_facts(&String::from_utf8_lossy(&files::read(
        &ctx.game().join(&tp2),
        TP2_LIMIT,
    )?));
    let backup_folder = old_facts
        .backup
        .clone()
        .filter(|folder| files::safe_relative(folder).is_ok())
        .ok_or_else(|| {
            format!(
                "The installed {} installer does not say where WeiDU keeps its uninstall copies.",
                entry.title
            )
        })?;
    let payload = release_payload(extracted, root, &tp2)?;
    let new_tp2 = payload
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(&tp2))
        .map(|(_, path)| path.clone())
        .ok_or("The release is missing its installer.")?;
    let new_facts = tp2_facts(&String::from_utf8_lossy(&files::read(&new_tp2, TP2_LIMIT)?));
    let components = found.components();
    if !new_facts.designated.is_empty() {
        if let Some(missing) = components
            .iter()
            .find(|component| !new_facts.designated.contains(component))
        {
            return Err(format!(
                "The new {} release no longer has component {missing}.",
                entry.title
            ));
        }
    }
    if new_facts
        .backup
        .as_deref()
        .is_some_and(|folder| folder != backup_folder)
    {
        return Err(format!(
            "The new {} release keeps its uninstall copies elsewhere, so WeiDU could not take the old version off.",
            entry.title
        ));
    }
    let language_number = new_language(&old_facts, &new_facts, found.language, &entry.title)?;
    let rows = found
        .all
        .iter()
        .enumerate()
        .map(|(index, row)| LogComponentReceipt {
            tp2: row.tp2_key.clone(),
            language: if found.own.contains(&index) {
                language_number
            } else {
                row.language
            },
            component: row.component,
        })
        .collect();
    Ok(Plan {
        package: Package {
            artifact_id: release.artifact.id.clone(),
            version: release.version.clone(),
            sha256: release.artifact.source.sha256.to_ascii_lowercase(),
            tp2: tp2.clone(),
            language: language_number,
            components,
            mod_folder: mod_folder.to_ascii_lowercase(),
        },
        update: UpdateRecord {
            mod_id: entry.mod_id.clone(),
            name: entry.title.clone(),
            from_version: found.version.clone(),
            to_version: release.version.clone(),
            release: release.record.clone(),
            tp2: normalize_tp2_key(&tp2),
            language: language_number,
            backup_folder,
            rows,
        },
        language,
        payload,
        new_version: new_facts.version,
    })
}

/// Every existing file the run may replace or remove: WeiDU's own files, the mod's folder,
/// WeiDU's records of it, and every file its uninstall lists name.
fn saved_names(game: &Path, current: &GameFiles, p: &Plan) -> Result<BTreeSet<String>, String> {
    let read = |name: &str| files::read(&game.join(name), LIST_LIMIT);
    let mut names = transaction::guarded(current, &p.language)
        .into_keys()
        .collect::<BTreeSet<_>>();
    for name in ["lua51.dll", "luaprovider.dll"] {
        if current.files.contains_key(name) {
            names.insert(name.to_owned());
        }
    }
    names.extend(owned_files(
        current,
        read,
        &p.update.backup_folder,
        &p.package.components,
    )?);
    let folders = [
        p.package.mod_folder.clone(),
        p.update.backup_folder.clone(),
        format!("weidu_external/data/{}", p.package.mod_folder),
    ];
    for name in current.files.keys() {
        if folders
            .iter()
            .any(|folder| name.starts_with(&format!("{folder}/")))
        {
            names.insert(name.clone());
        }
    }
    names.retain(|name| current.files.contains_key(name));
    Ok(names)
}

/// Write the preparation into `dir`: the release, WeiDU, a copy of every existing file the run
/// may replace or remove and, last, `prepared.json`. Nothing in the game changes here.
#[allow(clippy::too_many_arguments)]
pub(super) fn prepare(
    ctx: &Context<'_>,
    catalog: &Catalog,
    dir: &Path,
    p: &Plan,
    current: &GameFiles,
    tool: &Path,
    tool_sha256: String,
    previous: Option<String>,
) -> Result<Prepared, String> {
    let saved_names = saved_names(&ctx.game(), current, p)?;
    let mut payload = BTreeMap::new();
    for (name, path) in &p.payload {
        let bytes = files::read(path, PAYLOAD_LIMIT)?;
        files::create(&dir.join("payload").join(name), &bytes)?;
        payload.insert(name.clone(), sha256_bytes(&bytes));
    }
    let copy = dir.join("weidu.exe");
    files::create(&copy, &files::read(tool, 16 * 1024 * 1024)?)?;
    if files::hash(&copy)? != tool_sha256 {
        return Err("The WeiDU copy changed.".into());
    }
    let mut saved = BTreeMap::new();
    for name in &saved_names {
        let bytes = files::read(&ctx.game().join(name), COPY_LIMIT)?;
        let hash = sha256_bytes(&bytes);
        if current.files.get(name).map(|identity| &identity.sha256) != Some(&hash) {
            return Err(format!(
                "A file changed while it was being backed up: {name}"
            ));
        }
        files::create(&dir.join("affected").join(name), &bytes)?;
        saved.insert(name.clone(), hash);
    }
    let guarded = transaction::guarded(current, &p.language);
    crate::patches::close_check_in(&ctx.game(), &p.language, &guarded, None)?;
    let before = game_files::capture(&ctx.game())?;
    if before != *current {
        return Err("The game changed while the update was being prepared.".into());
    }
    let prepared = Prepared {
        schema: transaction::SCHEMA,
        hotpatch_id: format!("update:{}", p.update.mod_id),
        install_id: ctx.record.install_id.clone(),
        managed_root: ctx.record.managed_root.clone(),
        base_receipt_sha256: ctx.record.receipt_sha256.clone(),
        previous,
        catalog_digest: catalog.digest.clone(),
        package: p.package.clone(),
        language: p.language.clone(),
        before,
        saved,
        payload,
        tool_sha256,
        update: Some(p.update.clone()),
    };
    files::json_new(&dir.join("prepared.json"), &prepared)?;
    Ok(prepared)
}

fn execute(
    ctx: &Context<'_>,
    catalog: &Catalog,
    dir: &Path,
    p: &Prepared,
    new_version: Option<&str>,
) -> Result<(), String> {
    place_release(ctx, dir, p)?;
    hide_later_rows(ctx, dir, p)?;
    let evidence = transaction::invoke_weidu(ctx, dir, p, false)?;
    reinstalled_as_planned(
        &files::read(&dir.join("affected/weidu.log"), LOG_LIMIT)?,
        &files::read(&ctx.game().join("WeiDU.log"), LOG_LIMIT)?,
        &evidence,
        p,
        new_version,
    )?;
    put_later_rows_back(ctx, dir, p)?;
    record_applied(ctx, catalog, dir, p)
}

/// The one active row on `line`, if it holds one. Rows can hold bytes in a mod's own encoding, so
/// the mod list stays bytes and only this reading is lossy.
fn log_row(line: &[u8]) -> Result<Option<LogEntry>, String> {
    Ok(parse_active_entries(&String::from_utf8_lossy(line))
        .map_err(files::err)?
        .into_iter()
        .next())
}

fn log_lines(log: &[u8]) -> impl Iterator<Item = &[u8]> {
    log.split_inclusive(|byte| *byte == b'\n')
}

/// `log` as WeiDU sees it during the run: without the rows after the mod's first one that are not
/// its own. With them in place, WeiDU would take off every one of them and install it again,
/// which would run the installers of mods this app knows nothing about. Without them it takes off
/// only the old version: the mod's files, by its own uninstall lists.
fn visible_log(log: &[u8], tp2_key: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(log.len());
    let mut own_seen = false;
    for line in log_lines(log) {
        if let Some(row) = log_row(line)? {
            if row.tp2_key == tp2_key {
                own_seen = true;
            } else if own_seen {
                continue;
            }
        }
        out.extend_from_slice(line);
    }
    Ok(out)
}

/// The whole mod list after the run: `before`, with each of the mod's rows replaced by the row
/// WeiDU wrote for that component in `after`. Every other line stays byte for byte.
fn merged_log(before: &[u8], after: &[u8], tp2_key: &str) -> Result<Vec<u8>, String> {
    let without_end = |line: &'_ [u8]| -> usize {
        line.iter()
            .rposition(|byte| !matches!(byte, b'\r' | b'\n'))
            .map_or(0, |last| last + 1)
    };
    let mut written = BTreeMap::new();
    for line in log_lines(after) {
        if let Some(row) = log_row(line)? {
            if row.tp2_key == tp2_key {
                written.insert(row.component, line[..without_end(line)].to_vec());
            }
        }
    }
    let mut out = Vec::with_capacity(before.len());
    for line in log_lines(before) {
        match log_row(line)? {
            Some(row) if row.tp2_key == tp2_key => {
                let new = written
                    .get(&row.component)
                    .ok_or("WeiDU left a component uninstalled.")?;
                out.extend_from_slice(new);
                out.extend_from_slice(&line[without_end(line)..]);
            }
            _ => out.extend_from_slice(line),
        }
    }
    Ok(out)
}

/// Before WeiDU runs: the mod list ends with the mod ([`visible_log`]). The record holds the whole
/// list, so an interrupted run is put back with it.
pub(super) fn hide_later_rows(ctx: &Context<'_>, dir: &Path, p: &Prepared) -> Result<(), String> {
    let update = p.update.as_ref().ok_or("Not an update record.")?;
    let log = files::read(&dir.join("affected/weidu.log"), LOG_LIMIT)?;
    let held = files::hold(&ctx.record.managed_root, dir)?;
    files::write_held(&held.game, "WeiDU.log", &visible_log(&log, &update.tp2)?)
}

/// After WeiDU ran: the rows that were hidden come back, and the mod list must be the planned one.
pub(super) fn put_later_rows_back(
    ctx: &Context<'_>,
    dir: &Path,
    p: &Prepared,
) -> Result<(), String> {
    let update = p.update.as_ref().ok_or("Not an update record.")?;
    let merged = merged_log(
        &files::read(&dir.join("affected/weidu.log"), LOG_LIMIT)?,
        &files::read(&ctx.game().join("WeiDU.log"), LOG_LIMIT)?,
        &update.tp2,
    )?;
    let rows = parse_active_entries(&String::from_utf8_lossy(&merged)).map_err(files::err)?;
    if rows
        .iter()
        .map(row_identity)
        .ne(update.rows.iter().map(receipt_identity))
    {
        return Err("The mod list could not be put back as planned.".into());
    }
    let held = files::hold(&ctx.record.managed_root, dir)?;
    files::write_held(&held.game, "WeiDU.log", &merged)
}

fn row_identity(row: &LogEntry) -> (String, u32, u32) {
    (row.tp2_key.clone(), row.language, row.component)
}

fn receipt_identity(row: &LogComponentReceipt) -> (String, u32, u32) {
    (normalize_tp2_key(&row.tp2), row.language, row.component)
}

/// Replace the mod's folder with the release: files the release no longer has go, its files are
/// written. Every replaced or removed file has a copy in the record.
pub(super) fn place_release(ctx: &Context<'_>, dir: &Path, p: &Prepared) -> Result<(), String> {
    let held = files::hold(&ctx.record.managed_root, dir)?;
    let shipped = p
        .payload
        .keys()
        .map(|name| name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let prefix = format!("{}/", p.package.mod_folder);
    for (name, identity) in &p.before.files {
        if name.starts_with(&prefix) && !shipped.contains(name) {
            if p.saved.get(name) != Some(&identity.sha256) {
                return Err(format!("{name} has no copy in the record."));
            }
            files::remove_created(
                &held.game,
                name,
                Some(identity.sha256.as_str()),
                "A file of the old release changed during the update.",
            )?;
        }
    }
    for (name, hash) in &p.payload {
        let bytes = files::read_held(&held.record, &format!("payload/{name}"), PAYLOAD_LIMIT)?;
        if sha256_bytes(&bytes) != *hash {
            return Err("The update package changed.".into());
        }
        let lower = name.to_ascii_lowercase();
        if p.before.files.contains_key(&lower) && !p.saved.contains_key(&lower) {
            return Err(format!("{lower} has no copy in the record."));
        }
        let (folders, last) = files::split_relative(name)?;
        let folder = held
            .game
            .open_or_create_below(folders)
            .map_err(files::held)?;
        files::write_held(&folder, last, &bytes)?;
    }
    Ok(())
}

/// Judges WeiDU's run on the list it saw: an exit code that agrees with its result lines, no
/// component that failed, a result line for every component, the same rows in the same order
/// with the updated mod's in its new language. Version annotations are descriptive only.
pub(super) fn reinstalled_as_planned(
    before: &[u8],
    after: &[u8],
    evidence: &transaction::WeiduEvidence,
    p: &Prepared,
    _new_version: Option<&str>,
) -> Result<(), String> {
    let update = p.update.as_ref().ok_or("Not an update record.")?;
    let terminal = crate::cli::select_terminal_evidence(&evidence.debug, &evidence.stdout);
    let statuses = parse_terminal_statuses(&String::from_utf8_lossy(terminal));
    let failed = statuses.contains(&DebugStatus::NotInstalledDueToErrors);
    let warned = statuses.contains(&DebugStatus::InstalledWithWarnings);
    let installed = statuses
        .iter()
        .filter(|status| {
            matches!(
                status,
                DebugStatus::SuccessfullyInstalled | DebugStatus::InstalledWithWarnings
            )
        })
        .count();
    let expected_code = if warned { 3 } else { 0 };
    let rerun = p.package.components.len();
    if failed || evidence.exit_code != expected_code || installed < rerun {
        return Err(format!(
            "WeiDU did not install every component again (exit code {}, {installed} of at least {rerun} results).",
            evidence.exit_code
        ));
    }
    let before_text = String::from_utf8_lossy(before);
    let after_text = String::from_utf8_lossy(after);
    let rows = parse_active_entries(&after_text).map_err(files::err)?;
    // WeiDU notes the old installation of every component it installs again as "Recently
    // Uninstalled" (seen in the 2026-11-04 lab); such a line for a component that is not active
    // again means it stayed uninstalled.
    for line in new_uninstall_lines(&before_text, &after_text) {
        let active = line.as_ref().is_some_and(|(tp2, component)| {
            rows.iter()
                .any(|row| row.tp2_key == *tp2 && row.component == *component)
        });
        if !active {
            return Err("WeiDU left a component uninstalled.".into());
        }
    }
    // The list WeiDU saw: everything before the mod's first row, and the mod's own rows.
    let first = update
        .rows
        .iter()
        .position(|row| normalize_tp2_key(&row.tp2) == update.tp2)
        .unwrap_or(update.rows.len());
    let expected = update
        .rows
        .iter()
        .enumerate()
        .filter(|(index, row)| *index < first || normalize_tp2_key(&row.tp2) == update.tp2)
        .map(|(_, row)| receipt_identity(row));
    if rows.iter().map(row_identity).ne(expected) {
        return Err("WeiDU changed the mod list's order or another mod's rows.".into());
    }
    Ok(())
}

/// The components named by the "Recently Uninstalled" lines `after` has beyond those `before`
/// has, as (installer, component); `None` for a line that cannot be read.
fn new_uninstall_lines(before: &str, after: &str) -> Vec<Option<(String, u32)>> {
    const PREFIX: &str = "recently uninstalled:";
    let lines = |text: &str| {
        text.lines()
            .filter_map(|line| {
                let comment = line.trim().strip_prefix("//")?.trim_start();
                let head = comment.get(..PREFIX.len())?;
                head.eq_ignore_ascii_case(PREFIX)
                    .then(|| comment[PREFIX.len()..].trim().to_owned())
            })
            .collect::<Vec<_>>()
    };
    let mut existing: BTreeMap<String, usize> = BTreeMap::new();
    for line in lines(before) {
        *existing.entry(line.to_ascii_lowercase()).or_default() += 1;
    }
    let mut out = Vec::new();
    for line in lines(after) {
        let count = existing.entry(line.to_ascii_lowercase()).or_default();
        if *count > 0 {
            *count -= 1;
            continue;
        }
        // `~TP2~ #language #component`, then the old annotation.
        let named = line.strip_prefix('~').and_then(|rest| {
            let (tp2, tail) = rest.split_once('~')?;
            let mut fields = tail.split_whitespace();
            fields.next()?.strip_prefix('#')?.parse::<u32>().ok()?;
            let component = fields.next()?.strip_prefix('#')?.parse::<u32>().ok()?;
            Some((normalize_tp2_key(tp2), component))
        });
        out.push(named);
    }
    out
}

/// Check what the run did and publish the applied record: nothing outside the update's area, in
/// `override/` nothing but the mod's own files, the old release's plain copies replaced by the
/// new release's, and an earlier copy of every changed file.
pub(super) fn record_applied(
    ctx: &Context<'_>,
    catalog: &Catalog,
    dir: &Path,
    p: &Prepared,
) -> Result<(), String> {
    let update = p.update.as_ref().ok_or("Not an update record.")?;
    let after = game_files::capture(&ctx.game())?;
    let changed = p
        .before
        .differences(&after, |name| catalog.runtime_files.matches(name));
    let outside = changed
        .iter()
        .filter(|(name, _)| !writable(name, p, update))
        .map(|(name, difference)| (name.clone(), *difference))
        .collect::<BTreeMap<_, _>>();
    if !outside.is_empty() {
        return Err(format!(
            "The update changed files outside its area: {}.",
            gate::summarize(&outside)
        ));
    }
    // Files the updated mod wrote, before and after: they may change.
    let mut own = owned_files(
        &p.before,
        |name| files::read(&dir.join("affected").join(name), LIST_LIMIT),
        &update.backup_folder,
        &p.package.components,
    )?;
    own.extend(owned_files(
        &after,
        |name| files::read(&ctx.game().join(name), LIST_LIMIT),
        &update.backup_folder,
        &p.package.components,
    )?);
    // Outside `override/` the area holds the mod's folder and WeiDU's records. In `override/` only
    // the files the mod wrote may change: every other mod stays as it is.
    let different = changed
        .iter()
        .filter(|(name, _)| name.starts_with("override/") && !own.contains(*name))
        .map(|(name, difference)| (name.clone(), *difference))
        .collect::<BTreeMap<_, _>>();
    if !different.is_empty() {
        return Err(format!(
            "The update changed files of other mods: {}.",
            gate::summarize(&different)
        ));
    }
    let preimages = transaction::keep_preimages(ctx, dir, p, &after, &changed)?;
    let log_sha256 = after
        .files
        .get("weidu.log")
        .ok_or("The mod list is missing after the update.")?
        .sha256
        .clone();
    files::json_new(
        &dir.join("applied.json"),
        &Applied {
            prepared_sha256: files::hash(&dir.join("prepared.json"))?,
            after,
            suffix: Vec::new(),
            log_sha256,
            preimages,
        },
    )
}

/// Whether Undo would take back an update of `mod_id`: the latest change of the installation's
/// history is that update and nothing is unfinished. Reads the history only, so it needs neither
/// the network nor the game folder; Undo itself checks the folder.
pub fn undo_offered(ctx: &Context<'_>, mod_id: &str) -> Result<bool, String> {
    let history = History::load(&ctx.record.managed_root, &ctx.record.install_id)?;
    Ok(history.unfinished().is_none()
        && history.undo_target().is_some_and(|target| {
            target
                .update
                .as_ref()
                .is_some_and(|update| update.mod_id == mod_id)
        }))
}

/// The mods updated in an installation and not undone, with their releases, oldest first.
pub fn applied(root: &Path) -> Result<Vec<UpdateRecord>, String> {
    if !root.join(".chriz/hotpatches").exists() {
        return Ok(Vec::new());
    }
    let install_id = transaction::install_id_of(&root.join(".chriz/install-receipt.json"))?;
    let history = History::load(root, &install_id)?;
    Ok(history.applied_updates().into_iter().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUFFBOT_TP2: &str = r#"BACKUP ~weidu_external/backup/buffbot~
AUTHOR ~Chrizhermann~
VERSION ~v1.16.0~
// LANGUAGE ~Commented~ ~commented~ ~x~
/* LANGUAGE ~Block~ ~block~ ~x~ */
LANGUAGE
  ~English~
  ~english~
  ~buffbot/lang/english/setup.tra~
LANGUAGE ~Simplified Chinese~ ~schinese~ ~buffbot/lang/schinese/setup.tra~
LANGUAGE "German" "german" "buffbot/lang/german/setup.tra"
BEGIN @100
DESIGNATED 1
COPY ~buffbot/lang/%bfbot_selected_language%/setup.tra~ ~override/bfbot_l10n.tra~
BEGIN @111
DESIGNATED 0
"#;

    #[test]
    fn tp2_facts_read_languages_version_backup_and_components() {
        let facts = tp2_facts(BUFFBOT_TP2);
        assert_eq!(facts.languages, ["english", "schinese", "german"]);
        assert_eq!(facts.version.as_deref(), Some("1.16.0"));
        assert_eq!(
            facts.backup.as_deref(),
            Some("weidu_external/backup/buffbot")
        );
        assert_eq!(facts.designated, BTreeSet::from([0, 1]));
    }

    #[test]
    fn the_language_follows_its_folder_not_its_number() {
        let old = tp2_facts(BUFFBOT_TP2);
        let mut reordered = old.clone();
        reordered.languages = vec!["german".into(), "english".into()];
        assert_eq!(new_language(&old, &old, 2, "BuffBot").unwrap(), 2);
        assert_eq!(new_language(&old, &reordered, 2, "BuffBot").unwrap(), 0);
        assert_eq!(new_language(&old, &reordered, 0, "BuffBot").unwrap(), 1);
        let error = new_language(&old, &reordered, 1, "BuffBot").unwrap_err();
        assert!(error.contains("schinese"), "{error}");
        assert!(new_language(&old, &old, 9, "BuffBot").is_err());
    }

    #[test]
    fn versions_order_numerically_and_releases_after_pre_releases() {
        assert!(newer("1.16.0", "1.12.6"));
        assert!(newer("1.12.10", "1.12.9"));
        assert!(newer("1.8.4", "1.8.4-alpha"));
        assert!(newer("1.9.0", "1.8.4-alpha"));
        assert!(newer("2", "1.99.99"));
        assert!(!newer("1.12.6", "1.12.6"));
        assert!(!newer("1.12.6", "1.16.0"));
        assert!(!newer("1.8.4-alpha", "1.8.4"));
        assert!(newer("1.16.0", "unknown"));
        assert!(!newer("1.16.0", "1.16.0.0"));
    }

    #[test]
    fn row_versions_come_from_the_annotation() {
        let rows = parse_active_entries(
            "~BUFFBOT/SETUP-BUFFBOT.TP2~ #0 #1 // BuffBot: EEex LuaJIT Support (auto-detected): v1.12.6\r\n~BUFFBOT/SETUP-BUFFBOT.TP2~ #0 #0 // BuffBot: In-Game Buff Automation\r\n",
        )
        .unwrap();
        assert_eq!(row_version(&rows[0]).as_deref(), Some("1.12.6"));
        assert_eq!(
            row_version(&rows[1]).as_deref(),
            Some("In-Game Buff Automation")
        );
    }

    #[test]
    fn mod_folders_come_from_the_installer_path() {
        assert_eq!(
            tp2_folder("buffbot/setup-buffbot.tp2").as_deref(),
            Some("buffbot")
        );
        assert_eq!(
            tp2_folder("BuffBot\\Setup-BuffBot.TP2").as_deref(),
            Some("buffbot")
        );
        assert_eq!(
            tp2_folder("setup-eeexremote.tp2").as_deref(),
            Some("eeexremote")
        );
        assert_eq!(tp2_folder("setup-.tp2"), None);
        assert_eq!(tp2_folder("mod.tp2"), None);
    }

    #[test]
    fn uninstall_lists_name_lowercase_game_files_only() {
        let names = uninstall_names(
            b"override\\M_BfBot.lua\r\noverride/BfBotCor.lua\r\n\r\n..\\outside\r\nC:/x\r\n",
        );
        assert_eq!(
            names,
            BTreeSet::from([
                "override/m_bfbot.lua".to_owned(),
                "override/bfbotcor.lua".to_owned()
            ])
        );
    }
}
