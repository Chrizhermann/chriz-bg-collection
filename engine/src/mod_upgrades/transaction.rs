use super::*;
use crate::{
    acquire::{
        extract_archive, ArchiveFormat, ArchiveLimits, ArchiveMode, ArchiveRequirements,
        ArtifactCache, DownloadRequest,
    },
    events::ConsoleSink,
    lock::TargetLock,
    patches::files::Inventory,
    weidu::{
        invocation::{Invocation, VerifiedWeidu},
        runner::{self, RunOutcome, RunnerRequest},
    },
};
use std::{
    fs::OpenOptions,
    io::Write,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedUpgrade {
    pub run_id: String,
    pub version: String,
    pub artifact_sha256: String,
    pub before_log_sha256: String,
    pub log_sha256: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Prepared {
    pub schema: u32,
    pub upgrade_id: String,
    pub plan: Plan,
    pub before: Inventory,
    pub backups: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Applied {
    pub prepared_sha256: String,
    pub upgrade: AppliedUpgrade,
    pub after: Inventory,
    pub post_hashes: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Restoring {
    prepared_sha256: String,
    before: Inventory,
    removable: BTreeMap<String, Option<String>>,
}
pub(super) struct HistoryEntry {
    pub id: String,
    pub dir: PathBuf,
    pub state: String,
    pub terminal: String,
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&files::read(path, 64 * 1024 * 1024)?).map_err(err)
}
fn changed(a: &Inventory, b: &Inventory) -> Vec<String> {
    a.keys()
        .chain(b.keys())
        .filter(|n| a.get(*n) != b.get(*n))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
pub(super) fn load(root: &Path, dir: &Path) -> Result<Prepared, String> {
    files::direct(dir)?;
    if dir.parent() != Some(root.join(".chriz/mod-upgrades/v1").as_path()) {
        return Err("Update record is outside this installation.".into());
    }
    let p: Prepared = read_json(&dir.join("prepared.json"))?;
    let receipt = files::read(&root.join(".chriz/install-receipt.json"), 64 * 1024 * 1024)?;
    let r: serde_json::Value = serde_json::from_slice(&receipt).map_err(err)?;
    if p.schema != 1
        || p.upgrade_id != buffbot::UPGRADE_ID
        || p.plan.managed_root != root
        || p.plan.base_receipt_sha256 != sha256_bytes(&receipt)
        || r.get("install_id").and_then(|v| v.as_str()) != Some(p.plan.install_id.as_str())
    {
        return Err("Update history does not match this installation.".into());
    }
    for name in p
        .before
        .keys()
        .chain(p.plan.hashes.keys())
        .chain(p.backups.keys())
    {
        files::safe_relative(name)?;
    }
    let expected = p
        .before
        .iter()
        .filter(|(n, s)| !s.directory && buffbot::writable(n))
        .map(|(n, _)| n.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if expected != p.backups.keys().cloned().collect()
        || !p.backups.contains_key("weidu.log")
        || !p.backups.contains_key("lang/en_us/dialog.tlk")
        || p.backups
            .iter()
            .any(|(n, h)| p.plan.hashes.get(n) != Some(h))
    {
        return Err("Incomplete update backup declaration.".into());
    }
    Ok(p)
}
pub(super) fn history(root: &Path) -> Result<Vec<HistoryEntry>, String> {
    let parent = root.join(".chriz/mod-upgrades");
    if !parent.exists() {
        return Ok(vec![]);
    }
    files::direct(&parent)?;
    for entry in fs::read_dir(&parent).map_err(err)? {
        let e = entry.map_err(err)?;
        files::direct(&e.path())?;
        if e.file_name() != "v1" || !e.path().is_dir() {
            return Err("Unsupported mod-update history.".into());
        }
    }
    let state = parent.join("v1");
    if !state.exists() {
        return Ok(vec![]);
    }
    let mut dirs = fs::read_dir(&state)
        .map_err(err)?
        .map(|e| e.map(|e| e.path()).map_err(err))
        .collect::<Result<Vec<_>, _>>()?;
    dirs.sort();
    let mut result: Vec<HistoryEntry> = vec![];
    let mut previous = None;
    let mut active = false;
    for dir in dirs {
        files::direct(&dir)?;
        let id = dir
            .file_name()
            .ok_or("Missing update ID")?
            .to_string_lossy()
            .into_owned();
        if !dir.is_dir() || id.len() != 20 || !id.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Unrecognized mod-update transaction.".into());
        }
        if !dir.join("prepared.json").exists() {
            if [
                "applied.json",
                "applied.pending",
                "restoring.json",
                "restoring.pending",
                "restored.json",
                "restored.pending",
            ]
            .iter()
            .any(|n| dir.join(n).exists())
            {
                return Err("Update preparation evidence is missing.".into());
            }
            continue; // No game writes precede prepared.json.
        }
        if active {
            return Err("Unexpected later update after an active or interrupted upgrade.".into());
        }
        let p = load(root, &dir)?;
        if p.plan.previous != previous {
            return Err("Update history chain changed.".into());
        }
        let prepared_hash = files::hash(&dir.join("prepared.json"))?;
        let (status, terminal) = if dir.join("restored.json").exists() {
            let hash: String = read_json(&dir.join("restored.json"))?;
            if hash != prepared_hash {
                return Err("Restoration record changed.".into());
            }
            ("restored", files::hash(&dir.join("restored.json"))?)
        } else if dir.join("restoring.json").exists() {
            let r: Restoring = read_json(&dir.join("restoring.json"))?;
            if r.prepared_sha256 != prepared_hash {
                return Err("Restoration identity changed.".into());
            }
            ("restoring", files::hash(&dir.join("restoring.json"))?)
        } else if dir.join("applied.json").exists() {
            let a: Applied = read_json(&dir.join("applied.json"))?;
            for name in a.after.keys().chain(a.post_hashes.keys()) {
                files::safe_relative(name)?;
            }
            let expected = a
                .after
                .iter()
                .filter(|(n, s)| !s.directory && buffbot::writable(n))
                .map(|(n, _)| n.clone())
                .collect::<std::collections::BTreeSet<_>>();
            if a.prepared_sha256 != prepared_hash
                || a.upgrade.run_id != buffbot::RUN_ID
                || a.upgrade.version != buffbot::NEW_VERSION
                || a.upgrade.artifact_sha256 != buffbot::NEW_HASH
                || p.plan.hashes.get("weidu.log") != Some(&a.upgrade.before_log_sha256)
                || a.post_hashes.get("weidu.log") != Some(&a.upgrade.log_sha256)
                || a.post_hashes
                    .keys()
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>()
                    != expected
                || !a.post_hashes.contains_key(buffbot::TLK_PATH)
            {
                return Err("Completed update identity changed.".into());
            }
            ("applied", files::hash(&dir.join("applied.json"))?)
        } else {
            ("prepared", prepared_hash)
        };
        active = status != "restored";
        // Include directory identity so a restored/reapplied identical mod remains ordered.
        previous = Some(sha256_bytes(
            &serde_json::to_vec(&(&id, &terminal)).map_err(err)?,
        ));
        result.push(HistoryEntry {
            id,
            dir,
            state: status.into(),
            terminal: previous.clone().unwrap(),
        });
    }
    Ok(result)
}
pub(super) fn head(root: &Path) -> Result<Option<String>, String> {
    Ok(history(root)?.last().map(|h| h.terminal.clone()))
}
pub fn launch_guard(root: &Path) -> Result<(), String> {
    if history(root)?
        .last()
        .is_some_and(|h| matches!(h.state.as_str(), "prepared" | "restoring"))
    {
        return Err(
            "A mod update did not finish. Open Updates and restore its backup before playing."
                .into(),
        );
    }
    Ok(())
}
pub fn active_upgrade(root: &Path) -> Result<Option<AppliedUpgrade>, String> {
    launch_guard(root)?;
    let h = history(root)?;
    let Some(last) = h.last().filter(|h| h.state == "applied") else {
        return Ok(None);
    };
    let a: Applied = read_json(&last.dir.join("applied.json"))?;
    files::regular(&root.join("game/WeiDU.log"))?;
    let current = files::hash(&root.join("game/WeiDU.log"))?;
    if current != a.upgrade.log_sha256 {
        // Compatibility with a subsequent, independently verified legacy description fix.
        let transition = crate::patches::active_log_transition(root)?;
        if transition != Some((a.upgrade.log_sha256.clone(), current)) {
            return Err("The mod list changed outside the recorded update history.".into());
        }
    }
    Ok(Some(a.upgrade))
}

pub(super) struct Payloads {
    pub old: PathBuf,
    pub new: PathBuf,
    pub tool: PathBuf,
}
pub(super) fn payloads(ctx: &Context<'_>) -> Result<Payloads, String> {
    let cache = ArtifactCache::open(ctx.cache).map_err(err)?;
    let unpack = |id: &str,
                  url: String,
                  length: u64,
                  hash: &str,
                  root: &str,
                  tp2: Vec<String>|
     -> Result<PathBuf, String> {
        let acquired = cache
            .acquire(
                &DownloadRequest {
                    request_id: id.into(),
                    url,
                    expected_length: length,
                    expected_sha256: hash.into(),
                    redirect_hosts: vec!["release-assets.githubusercontent.com".into()],
                    max_attempts: crate::acquire::DEFAULT_DOWNLOAD_ATTEMPTS,
                },
                &ConsoleSink,
            )
            .map_err(err)?;
        let extraction = extract_archive(
            &acquired.archive_path,
            &ctx.cache.join("upgrade-extracted"),
            &ArchiveRequirements {
                artifact_sha256: hash.into(),
                format: ArchiveFormat::Zip,
                expected_roots: vec![root.into()],
                expected_tp2_paths: tp2,
                limits: ArchiveLimits {
                    max_entries: 256,
                    max_depth: 8,
                    max_entry_uncompressed_bytes: 8 * 1024 * 1024,
                    max_total_uncompressed_bytes: 32 * 1024 * 1024,
                    ..Default::default()
                },
                mode: ArchiveMode::Public,
            },
        )
        .map_err(err)?;
        Ok(extraction.root)
    };
    let old=unpack("upgrade-buffbot-old",format!("https://github.com/Chrizhermann/bg-eeex-buffbot/releases/download/v{0}/buffbot-v{0}.zip",buffbot::OLD_VERSION),4_240_743,buffbot::OLD_HASH,"buffbot",vec!["buffbot/setup-buffbot.tp2".into()])?;
    let new=unpack("upgrade-buffbot-new",format!("https://github.com/Chrizhermann/bg-eeex-buffbot/releases/download/v{0}/buffbot-v{0}.zip",buffbot::NEW_VERSION),4_241_150,buffbot::NEW_HASH,"buffbot",vec!["buffbot/setup-buffbot.tp2".into()])?;
    buffbot::validate_sources(&old, &new)?;
    let tool = unpack(
        "upgrade-weidu-249",
        "https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip"
            .into(),
        2_154_755,
        "b156910cbec69359fc2e42f6739aa959d49047d6fd3dc172f6bed88ffad8f927",
        "weidu.exe",
        vec![],
    )?
    .join("weidu.exe");
    VerifiedWeidu::verify(&tool, crate::patches::catalog::TOOL_HASH).map_err(err)?;
    Ok(Payloads { old, new, tool })
}

fn verify_hashes(
    game: &Path,
    hashes: &BTreeMap<String, String>,
    skip_runtime: bool,
) -> Result<(), String> {
    for (name, hash) in hashes {
        if skip_runtime && buffbot::runtime_mutable(name) {
            continue;
        }
        files::regular(&game.join(name))?;
        if files::hash(&game.join(name))? != *hash {
            return Err(format!("File changed: {name}"));
        }
    }
    Ok(())
}
fn write_existing(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        files::regular(path)?;
    } else {
        let parent = path.parent().ok_or("Missing file parent")?;
        let ancestor = parent
            .ancestors()
            .find(|p| p.exists())
            .ok_or("Missing folder")?;
        files::direct(ancestor)?;
        fs::create_dir_all(parent).map_err(err)?;
        files::direct(parent)?;
    }
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(err)?;
    f.write_all(bytes).and_then(|_| f.sync_all()).map_err(err)
}
fn backups(dir: &Path, p: &Prepared) -> Result<(), String> {
    for (name, hash) in &p.backups {
        files::regular(&dir.join("affected").join(name))?;
        if files::hash(&dir.join("affected").join(name))? != *hash {
            return Err(format!("Backup verification failed: {name}"));
        }
    }
    Ok(())
}
pub(super) fn check_restore(ctx: &Context<'_>, dir: &Path, undo: bool) -> Result<(), String> {
    let p = load(&ctx.record.managed_root, dir)?;
    backups(dir, &p)?;
    if history(&ctx.record.managed_root)?
        .last()
        .is_none_or(|h| h.dir != dir)
    {
        return Err("Only the latest update can be restored.".into());
    }
    if undo && dir.join("restoring.json").exists() {
        return Err("Restoration has already started; choose Restore.".into());
    }
    let guard = p
        .plan
        .hashes
        .iter()
        .filter(|(n, _)| !buffbot::writable(n) && !buffbot::runtime_mutable(n))
        .map(|(n, h)| (n.clone(), h.clone()))
        .collect();
    verify_hashes(&ctx.game(), &guard, false)?;
    let now = files::inventory(&ctx.game(), false)?;
    if undo {
        let a: Applied = read_json(&dir.join("applied.json"))?;
        verify_hashes(&ctx.game(), &a.post_hashes, true)?;
        // Hashes above establish file contents. A later fix's successful Undo may
        // restore identical bytes with newer timestamps; that is not drift.
        if a.after.keys().chain(now.keys()).any(|n| {
            buffbot::writable(n)
                && !buffbot::runtime_mutable(n)
                && a.after.get(n).map(|s| s.directory) != now.get(n).map(|s| s.directory)
        }) {
            return Err("Relevant files or a later game fix changed. Undo later fixes first; keep this backup.".into());
        }
    } else {
        let comparison = if dir.join("restoring.json").exists() {
            read_json::<Restoring>(&dir.join("restoring.json"))?.before
        } else {
            p.before.clone()
        };
        if changed(&comparison, &now)
            .iter()
            .any(|n| !buffbot::writable(n))
        {
            return Err(
                "Files outside the update changed; automatic restoration needs investigation."
                    .into(),
            );
        }
    }
    Ok(())
}

pub fn apply(
    ctx: &Context<'_>,
    review_token: &str,
    full: bool,
    saves: bool,
) -> Result<ModUpgradePreview, String> {
    apply_with_progress(ctx, review_token, full, saves, &|_| {})
}

pub fn apply_with_progress(
    ctx: &Context<'_>,
    review_token: &str,
    full: bool,
    saves: bool,
    progress: crate::operation_progress::ProgressSink<'_>,
) -> Result<ModUpgradePreview, String> {
    crate::operation_progress::stage(progress, "checking");
    let _lock = TargetLock::try_acquire(ctx.app_data.join("locks"), &ctx.record.managed_root)
        .map_err(err)?;
    validate_context(ctx)?;
    launch_guard(&ctx.record.managed_root)?;
    if active_upgrade(&ctx.record.managed_root)?.is_some() {
        return inspect(ctx);
    }
    crate::operation_progress::stage(progress, "downloading");
    let source = payloads(ctx)?;
    crate::operation_progress::stage(progress, "checking");
    let p = plan(ctx, &source.old, &source.new)?;
    if token(&p)? != review_token {
        return Err("Installation changed since review. Check installed mods again.".into());
    }
    crate::patches::close_check(&ctx.game(), &p.hashes)?;
    let before = files::inventory(&ctx.game(), false)?;
    let affected_bytes = before
        .iter()
        .filter(|(n, s)| !s.directory && buffbot::writable(n))
        .map(|(_, s)| s.length)
        .sum::<u64>();
    let profile = if saves {
        Some(files::inventory(&ctx.record.managed_save_root, true)?)
    } else {
        None
    };
    let external = (if full { files::total(&before) } else { 0 })
        .checked_add(profile.as_ref().map(files::total).unwrap_or(0))
        .ok_or("Backup size overflow")?;
    // Conservative on separate volumes; also accounts for both backup sets when
    // app-data and the installation share a volume.
    let required = external
        .saturating_add(affected_bytes)
        .saturating_add(768 * 1024 * 1024);
    if SystemPreflight.available_space(ctx.app_data).map_err(err)? < required
        || SystemPreflight
            .available_space(&ctx.record.managed_root)
            .map_err(err)?
            < required
    {
        return Err("Not enough free space for the selected backups and update.".into());
    }
    let id = format!(
        "{:020}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(err)?
            .as_nanos()
    );
    let dir = ctx.state().join(&id);
    files::create(&dir.join("identity"), ctx.record.install_id.as_bytes())?;
    let backup = ctx.backups().join(&id);
    files::create(&backup.join("identity"), ctx.record.install_id.as_bytes())?;
    if full {
        files::copy_tree_with_progress(
            &ctx.game(),
            &backup.join("game"),
            false,
            "backup-game",
            progress,
        )?;
    }
    if saves {
        files::copy_tree_with_progress(
            &ctx.record.managed_save_root,
            &backup.join("profile"),
            true,
            "backup-saves",
            progress,
        )?;
    }
    crate::operation_progress::stage(progress, "backup-files");
    let prepared = prepare(ctx, &dir, p, before)?;
    crate::patches::close_check(&ctx.game(), &prepared.plan.hashes)?;
    verify_hashes(&ctx.game(), &prepared.plan.hashes, false)?;
    if files::inventory(&ctx.game(), false)? != prepared.before {
        return Err("Game changed while its backup was being prepared.".into());
    }
    files::json_new(&dir.join("prepared.json"), &prepared)?;
    crate::operation_progress::stage(progress, "applying");
    let result = execute(ctx, &dir, &prepared, &source, progress);
    if let Err(e) = result {
        let _ = files::json_new(&dir.join("failure.json"), &e);
        // Keep the failure visible and offer Restore; never blindly rerun a partial reinstall.
        let mut preview = inspect(ctx)?;
        preview.detail =
            format!("BuffBot did not finish updating. Restore the backup before playing. {e}");
        return Ok(preview);
    }
    crate::operation_progress::stage(progress, "verifying");
    inspect(ctx)
}
pub(super) fn prepare(
    ctx: &Context<'_>,
    dir: &Path,
    plan: Plan,
    before: Inventory,
) -> Result<Prepared, String> {
    let mut saved = BTreeMap::new();
    for (name, stamp) in &before {
        if !stamp.directory && buffbot::writable(name) {
            let bytes = files::read(&ctx.game().join(name), 256 * 1024 * 1024)?;
            let hash = sha256_bytes(&bytes);
            if plan.hashes.get(name) != Some(&hash) {
                return Err("A file changed before backup.".into());
            }
            files::create(&dir.join("affected").join(name), &bytes)?;
            saved.insert(name.clone(), hash);
        }
    }
    let p = Prepared {
        schema: 1,
        upgrade_id: buffbot::UPGRADE_ID.into(),
        plan,
        before,
        backups: saved,
    };
    backups(dir, &p)?;
    Ok(p)
}
fn execute(
    ctx: &Context<'_>,
    dir: &Path,
    p: &Prepared,
    sources: &Payloads,
    progress: crate::operation_progress::ProgressSink<'_>,
) -> Result<(), String> {
    // Source-tree identity is checked again immediately before publication.
    buffbot::validate_sources(&sources.old, &sources.new)?;
    for entry in walkdir::WalkDir::new(sources.new.join("buffbot")).follow_links(false) {
        let entry = entry.map_err(err)?;
        if entry.file_type().is_dir() {
            continue;
        }
        let relative = entry.path().strip_prefix(&sources.new).map_err(err)?;
        let name = relative.to_string_lossy().replace('\\', "/");
        if !buffbot::writable(&name.to_ascii_lowercase()) {
            return Err("Unexpected upgrade source output.".into());
        }
        let bytes = files::read(entry.path(), 16 * 1024 * 1024)?;
        write_existing(&ctx.game().join(relative), &bytes)?;
    }
    VerifiedWeidu::verify(&sources.tool, crate::patches::catalog::TOOL_HASH).map_err(err)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1);
    }
    let _tool = options.open(&sources.tool).map_err(err)?;
    if files::hash(&sources.tool)? != crate::patches::catalog::TOOL_HASH {
        return Err("Installer tool changed.".into());
    }
    let debug = dir.join("attempt.debug");
    let args = [
        "buffbot/setup-buffbot.tp2",
        "--language",
        "0",
        "--use-lang",
        "en_US",
        "--force-install-list",
        "1",
        "0",
        "--no-exit-pause",
        "--skip-at-view",
        "--noautoupdate",
        "--log",
    ];
    let mut args = args
        .into_iter()
        .map(std::ffi::OsString::from)
        .collect::<Vec<_>>();
    args.push(debug.clone().into());
    files::json_new(&dir.join("invocation.json"), &(&sources.tool, &args))?;
    let invocation = Invocation {
        program: sources.tool.clone(),
        cwd: ctx.game(),
        identity_digest: sha256_bytes(&serde_json::to_vec(&args).map_err(err)?),
        args,
        prompts: vec![],
        debug_path: debug,
    };
    let (_sender, receiver) = crossbeam_channel::unbounded();
    let outcome = runner::run(
        RunnerRequest {
            invocation,
            step_id: buffbot::UPGRADE_ID.into(),
            output_log: dir.join("output.log"),
            silence_threshold: Duration::from_secs(300),
        },
        receiver,
        ConsoleSink,
    );
    if outcome != (RunOutcome::Exited { code: 0 }) {
        return Err(format!("WeiDU did not finish successfully: {outcome:?}"));
    }
    crate::operation_progress::stage(progress, "verifying");
    buffbot::verify_after(
        &ctx.game(),
        &sources.new,
        &files::read(
            &dir.join("affected/lang/en_us/dialog.tlk"),
            256 * 1024 * 1024,
        )?,
        &files::read(&dir.join("affected/override/bfbot_strrefs.txt"), 65536)?,
    )?;
    let old = files::read(&dir.join("affected/weidu.log"), 16 * 1024 * 1024)?;
    let new = files::read(&ctx.game().join("WeiDU.log"), 16 * 1024 * 1024)?;
    verify_tail(&new, buffbot::NEW_VERSION)?;
    let rows = |b: &[u8]| -> Result<Vec<(String, u32, u32)>, String> {
        Ok(
            crate::weidu::log::parse_active_entries(std::str::from_utf8(b).map_err(err)?)
                .map_err(err)?
                .into_iter()
                .map(|r| (r.tp2_key, r.language, r.component))
                .collect(),
        )
    };
    if rows(&old)? != rows(&new)? {
        return Err("WeiDU changed another mod or the component order.".into());
    }
    finish(ctx, dir, p)
}
pub(super) fn finish(ctx: &Context<'_>, dir: &Path, p: &Prepared) -> Result<(), String> {
    let guards = p
        .plan
        .hashes
        .iter()
        .filter(|(n, _)| !buffbot::writable(n))
        .map(|(n, h)| (n.clone(), h.clone()))
        .collect();
    verify_hashes(&ctx.game(), &guards, false)?;
    let after = files::inventory(&ctx.game(), false)?;
    if changed(&p.before, &after)
        .iter()
        .any(|n| !buffbot::writable(n))
    {
        return Err("Unexpected files changed outside the update.".into());
    }
    let mut post = BTreeMap::new();
    for (name, stamp) in &after {
        if !stamp.directory && buffbot::writable(name) {
            files::regular(&ctx.game().join(name))?;
            post.insert(name.clone(), files::hash(&ctx.game().join(name))?);
        }
    }
    let upgrade = AppliedUpgrade {
        run_id: buffbot::RUN_ID.into(),
        version: buffbot::NEW_VERSION.into(),
        artifact_sha256: buffbot::NEW_HASH.into(),
        before_log_sha256: p
            .plan
            .hashes
            .get("weidu.log")
            .ok_or("Original mod list missing")?
            .clone(),
        log_sha256: files::hash(&ctx.game().join("WeiDU.log"))?,
    };
    files::json_new(
        &dir.join("applied.json"),
        &Applied {
            prepared_sha256: files::hash(&dir.join("prepared.json"))?,
            upgrade,
            after,
            post_hashes: post,
        },
    )
}

pub fn restore(ctx: &Context<'_>, undo: bool) -> Result<ModUpgradePreview, String> {
    restore_with_progress(ctx, undo, &|_| {})
}

pub fn restore_with_progress(
    ctx: &Context<'_>,
    undo: bool,
    progress: crate::operation_progress::ProgressSink<'_>,
) -> Result<ModUpgradePreview, String> {
    crate::operation_progress::stage(progress, "checking");
    let _lock = TargetLock::try_acquire(ctx.app_data.join("locks"), &ctx.record.managed_root)
        .map_err(err)?;
    validate_context(ctx)?;
    let h = history(&ctx.record.managed_root)?;
    let last = h.last().ok_or("No mod update to restore")?;
    if (undo && last.state != "applied")
        || (!undo && !matches!(last.state.as_str(), "prepared" | "restoring"))
    {
        return Err("That update action is no longer available. Check again.".into());
    }
    let p = load(&ctx.record.managed_root, &last.dir)?;
    check_restore(ctx, &last.dir, undo)?;
    let existing = p
        .plan
        .hashes
        .iter()
        .filter(|(n, _)| ctx.game().join(n).exists())
        .map(|(n, h)| (n.clone(), h.clone()))
        .collect();
    crate::patches::close_restore_check(&ctx.game(), &existing)?;
    crate::operation_progress::stage(progress, "restoring");
    restore_files(ctx, &last.dir, undo)?;
    crate::operation_progress::stage(progress, "verifying");
    inspect(ctx)
}
pub(super) fn restore_files(ctx: &Context<'_>, dir: &Path, undo: bool) -> Result<(), String> {
    check_restore(ctx, dir, undo)?;
    let p = load(&ctx.record.managed_root, dir)?;
    if !dir.join("restoring.json").exists() {
        let before = files::inventory(&ctx.game(), false)?;
        let mut removable = BTreeMap::new();
        for (name, stamp) in &before {
            if !p.before.contains_key(name) && buffbot::writable(name) {
                removable.insert(
                    name.clone(),
                    if stamp.directory {
                        None
                    } else {
                        Some(files::hash(&ctx.game().join(name))?)
                    },
                );
            }
        }
        files::json_new(
            &dir.join("restoring.json"),
            &Restoring {
                prepared_sha256: files::hash(&dir.join("prepared.json"))?,
                before,
                removable,
            },
        )?;
    }
    check_restore(ctx, dir, false)?;
    let r: Restoring = read_json(&dir.join("restoring.json"))?;
    // Validate the entire cleanup set before overwriting/removing anything.
    for (name, hash) in &r.removable {
        files::safe_relative(name)?;
        if p.before.contains_key(name) || !buffbot::writable(name) {
            return Err("Invalid update cleanup declaration.".into());
        }
        let path = ctx.game().join(name);
        if path.exists() {
            files::direct(&path)?;
            if let Some(hash) = hash {
                files::regular(&path)?;
                if files::hash(&path)? != *hash {
                    return Err("New update file changed during restoration.".into());
                }
            } else if !path.is_dir() {
                return Err("Update folder changed.".into());
            }
        }
    }
    for (name, stamp) in &p.before {
        if stamp.directory && buffbot::writable(name) {
            let path = ctx.game().join(name);
            let ancestor = path
                .ancestors()
                .find(|p| p.exists())
                .ok_or("Missing restore parent")?;
            files::direct(ancestor)?;
            fs::create_dir_all(&path).map_err(err)?;
            files::direct(&path)?;
        }
    }
    for name in p.backups.keys() {
        let bytes = files::read(&dir.join("affected").join(name), 256 * 1024 * 1024)?;
        write_existing(&ctx.game().join(name), &bytes)?;
    }
    let mut remove = r.removable.keys().collect::<Vec<_>>();
    remove.sort_by_key(|n| std::cmp::Reverse(n.len()));
    for name in remove {
        let path = ctx.game().join(name);
        if !path.exists() {
            continue;
        }
        files::direct(&path)?;
        if r.removable[name].is_none() {
            fs::remove_dir(path).map_err(err)?;
        } else {
            files::regular(&path)?;
            fs::remove_file(path).map_err(err)?;
        }
    }
    verify_hashes(&ctx.game(), &p.backups, false)?;
    files::json_new(
        &dir.join("restored.json"),
        &files::hash(&dir.join("prepared.json"))?,
    )
}
