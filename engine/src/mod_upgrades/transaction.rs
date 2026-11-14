use super::*;
use crate::{
    held_folder::{EntryError, EntryErrorKind},
    patches::files::Inventory,
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
    #[cfg(test)]
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
            #[cfg(test)]
            terminal: previous.clone().unwrap(),
        });
    }
    Ok(result)
}
#[cfg(test)]
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
    let log = crate::weidu::log::resolve_weidu_log(&root.join("game")).map_err(err)?;
    files::regular(&log)?;
    let current = files::hash(&log)?;
    if current != a.upgrade.log_sha256 {
        // Compatibility with a subsequent, independently verified legacy description fix.
        let transition = crate::patches::active_log_transition(root)?;
        if transition != Some((a.upgrade.log_sha256.clone(), current)) {
            return Err("The mod list changed outside the recorded update history.".into());
        }
    }
    Ok(Some(a.upgrade))
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
        let path = files::resolve_relative(game, name)?;
        files::regular(&path)?;
        if files::hash(&path)? != *hash {
            return Err(format!("File changed: {name}"));
        }
    }
    Ok(())
}
/// Write `bytes` to the game file `name` through the held game, creating it and any missing
/// folders. An existing file is written into, as a truncating write by path did.
pub(super) fn write_existing(
    held: &files::HeldTransaction,
    name: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let (folders, last) = files::split_relative(name)?;
    let folder = held
        .game
        .open_or_create_below(folders)
        .map_err(files::held)?;
    files::write_held(&folder, last, bytes)
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
    verify_guards(&ctx.game(), &guard)?;
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
/// Like [`verify_hashes`] for the files an update relies on but never writes, except that
/// `infinityloader.ini` may differ in its game-log setting alone: CEBG turns the game log for
/// crash reports on and off there, and that never touches what the update relies on.
fn verify_guards(game: &Path, hashes: &BTreeMap<String, String>) -> Result<(), String> {
    for (name, hash) in hashes {
        let path = files::resolve_relative(game, name)?;
        files::regular(&path)?;
        if files::hash(&path)? != *hash
            && !(name.eq_ignore_ascii_case("infinityloader.ini")
                && crate::game_log::unchanged_apart_from_the_setting(
                    &files::read(&path, crate::game_log::INI_LIMIT)?,
                    hash,
                ))
        {
            return Err(format!("File changed: {name}"));
        }
    }
    Ok(())
}

// Build historical records only for restoration fixtures. No new upgrades are offered.
#[cfg(test)]
pub(super) fn prepare(
    ctx: &Context<'_>,
    dir: &Path,
    plan: Plan,
    before: Inventory,
) -> Result<Prepared, String> {
    let mut saved = BTreeMap::new();
    for (name, stamp) in &before {
        if !stamp.directory && buffbot::writable(name) {
            let path = files::resolve_relative(&ctx.game(), name)?;
            let bytes = files::read(&path, 256 * 1024 * 1024)?;
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
#[cfg(test)]
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
            let path = files::resolve_relative(&ctx.game(), name)?;
            files::regular(&path)?;
            post.insert(name.clone(), files::hash(&path)?);
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
        log_sha256: files::hash(&crate::weidu::log::resolve_weidu_log(&ctx.game()).map_err(err)?)?,
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
    let _lock = ctx
        .locks
        .try_acquire(&ctx.record.managed_root)
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
    let mut existing = BTreeMap::new();
    for (n, h) in &p.plan.hashes {
        if files::resolve_relative(&ctx.game(), n)?.exists() {
            existing.insert(n.clone(), h.clone());
        }
    }
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
                        Some(files::hash(&files::resolve_relative(&ctx.game(), name)?)?)
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
    put_back(&files::hold(&ctx.record.managed_root, dir)?, &p, &r)?;
    files::json_new(
        &dir.join("restored.json"),
        &files::hash(&dir.join("prepared.json"))?,
    )
}
/// Restores the record `dir` through `held`, as production restoration does once its checks
/// have passed and its restoration record is written.
#[cfg(test)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(super) fn put_back_fixture(
    ctx: &Context<'_>,
    dir: &Path,
    held: &files::HeldTransaction,
) -> Result<(), String> {
    put_back(
        held,
        &load(&ctx.record.managed_root, dir)?,
        &read_json(&dir.join("restoring.json"))?,
    )
}
/// Put the affected files back from their backups and remove what the update created, all
/// through the held folders. Each restored file is written into the updated one, as before.
fn put_back(held: &files::HeldTransaction, p: &Prepared, r: &Restoring) -> Result<(), String> {
    const CHANGED: &str = "New update file changed during restoration.";
    // Validate the entire cleanup set before overwriting/removing anything.
    for (name, hash) in &r.removable {
        files::safe_relative(name)?;
        if p.before.contains_key(name) || !buffbot::writable(name) {
            return Err("Invalid update cleanup declaration.".into());
        }
        if let Some(hash) = hash {
            // Closed again at once, so that restoring can go on.
            files::created_file(&held.game, name, hash, CHANGED)?;
        } else {
            match held.game.open_below(name.split('/')) {
                Ok(_)
                | Err(EntryError {
                    kind: EntryErrorKind::Missing,
                    ..
                }) => {}
                Err(EntryError {
                    kind: EntryErrorKind::NotDirect,
                    ..
                }) => return Err("Update folder changed.".into()),
                Err(error) => return Err(files::held(error)),
            }
        }
    }
    for (name, stamp) in &p.before {
        if stamp.directory && buffbot::writable(name) {
            files::safe_relative(name)?;
            held.game
                .open_or_create_below(name.split('/'))
                .map_err(files::held)?;
        }
    }
    for (name, expected) in &p.backups {
        let bytes = files::read_held(&held.record, &format!("affected/{name}"), 256 * 1024 * 1024)?;
        if sha256_bytes(&bytes) != *expected {
            return Err(format!("Backup verification failed: {name}"));
        }
        write_existing(held, name, &bytes)?;
    }
    let mut remove = r.removable.iter().collect::<Vec<_>>();
    remove.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
    for (name, hash) in remove {
        files::remove_created(&held.game, name, hash.as_deref(), CHANGED)?;
    }
    files::verify_held_hashes(&held.game, &p.backups)
}
