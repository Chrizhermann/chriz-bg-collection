use super::*;
use crate::{
    acquire::{
        extract_archive, ArchiveFormat, ArchiveLimits, ArchiveMode, ArchiveRequirements,
        ArtifactCache, DownloadRequest,
    },
    events::ConsoleSink,
    lock::TargetLock,
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

#[derive(Serialize, Deserialize)]
struct Prepared {
    schema: u32,
    patch_id: String,
    install_id: String,
    managed_root: PathBuf,
    plan: Plan,
    before: Inventory,
    backups: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
struct Applied {
    prepared_sha256: String,
    post_hashes: BTreeMap<String, String>,
    created: Vec<String>,
    after: Inventory,
}
#[derive(Serialize, Deserialize)]
struct Restoring {
    prepared_sha256: String,
    before: Inventory,
    removable: BTreeMap<String, Option<String>>,
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&files::read(path, 64 * 1024 * 1024)?).map_err(err)
}
fn has_later_state(dir: &Path) -> bool {
    [
        "applied.json",
        "applied.pending",
        "restoring.json",
        "restoring.pending",
        "restored.json",
        "restored.pending",
    ]
    .iter()
    .any(|n| dir.join(n).exists())
}
fn restored(dir: &Path) -> Result<bool, String> {
    if !dir.join("restored.json").exists() {
        return Ok(false);
    }
    let digest: String = read_json(&dir.join("restored.json"))?;
    if digest != files::hash(&dir.join("prepared.json"))? {
        return Err("Restoration evidence changed.".into());
    }
    Ok(true)
}

pub(super) fn latest(ctx: &Context<'_>) -> Result<Option<(PathBuf, String)>, String> {
    let state = ctx.state();
    if !state.exists() {
        return Ok(None);
    }
    files::direct(&state)?;
    let mut dirs = fs::read_dir(&state)
        .map_err(err)?
        .map(|e| e.map(|e| e.path()).map_err(err))
        .collect::<Result<Vec<_>, _>>()?;
    dirs.sort();
    let mut result = None;
    for dir in dirs {
        files::direct(&dir)?;
        if !dir.is_dir()
            || !dir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .bytes()
                .all(|b| b.is_ascii_digit())
        {
            return Err("Unrecognized patch transaction entry.".into());
        }
        if !dir.join("prepared.json").exists() {
            // By construction no game write happens before this durable barrier.
            if has_later_state(&dir) {
                return Err("Patch history is missing its preparation record.".into());
            }
            continue;
        }
        let _: Prepared = load_prepared(ctx, &dir)?;
        let state = if restored(&dir)? {
            "restored"
        } else if dir.join("restoring.json").exists() {
            "restoring"
        } else if dir.join("applied.json").exists() {
            "applied"
        } else {
            "prepared"
        };
        result = Some((dir, state.to_owned()));
    }
    Ok(result)
}
pub fn launch_guard(root: &Path) -> Result<(), String> {
    let state = root.join(".chriz/patches/cebg-v1");
    if !state.exists() {
        return Ok(());
    }
    files::direct(&state)?;
    for dir in fs::read_dir(state).map_err(err)? {
        let path = dir.map_err(err)?.path();
        files::direct(&path)?;
        if !path.join("prepared.json").exists() {
            if has_later_state(&path) {
                return Err("Patch history is missing its preparation record.".into());
            }
            continue; // An unpublished preparation cannot have touched the game.
        }
        let is_restored = restored(&path)?;
        if !is_restored
            && (!path.join("applied.json").exists() || path.join("restoring.json").exists())
        {
            return Err("This installation has an unfinished patch. Open Updates and restore its backup before playing.".into());
        }
        if path.join("applied.json").exists() && !is_restored {
            let applied: Applied = read_json(&path.join("applied.json"))?;
            if files::hash(&path.join("prepared.json"))? != applied.prepared_sha256 {
                return Err("Patch history changed; launch blocked pending diagnosis.".into());
            }
        }
    }
    Ok(())
}
/// Cheap receipt overlay for the launcher. Does not rewrite or broaden the base recipe.
pub fn verified_log_suffix(
    root: &Path,
) -> Result<Vec<crate::receipt::LogComponentReceipt>, String> {
    launch_guard(root)?;
    let state = root.join(".chriz/patches/cebg-v1");
    if !state.exists() {
        return Ok(vec![]);
    }
    let mut suffix = Vec::new();
    for entry in fs::read_dir(state).map_err(err)? {
        let dir = entry.map_err(err)?.path();
        if !dir.join("prepared.json").exists()
            || restored(&dir)?
            || !dir.join("applied.json").exists()
        {
            continue;
        }
        let p: Prepared = read_json(&dir.join("prepared.json"))?;
        let a: Applied = read_json(&dir.join("applied.json"))?;
        let log = root.join("game/WeiDU.log");
        files::regular(&log)?;
        if p.schema != 1
            || p.patch_id != catalog::PATCH_ID
            || p.managed_root != root
            || p.plan.base_receipt_sha256 != files::hash(&root.join(".chriz/install-receipt.json"))?
            || a.post_hashes.get("weidu.log") != Some(&files::hash(&log)?)
        {
            return Err("Patch history no longer matches this mod list.".into());
        }
        if !suffix.is_empty() {
            return Err("Multiple active patch receipts are unsupported.".into());
        }
        suffix.push(crate::receipt::LogComponentReceipt {
            tp2: "akcb_kit_descriptions/setup-akcb_kit_descriptions.tp2".into(),
            language: 0,
            component: 0,
        });
    }
    Ok(suffix)
}
fn load_prepared(ctx: &Context<'_>, dir: &Path) -> Result<Prepared, String> {
    files::direct(dir)?;
    if dir.parent() != Some(ctx.state().as_path()) {
        return Err("Patch history is outside this installation.".into());
    }
    let p: Prepared = read_json(&dir.join("prepared.json"))?;
    if p.schema != 1
        || p.patch_id != catalog::PATCH_ID
        || p.install_id != ctx.record.install_id
        || p.managed_root != ctx.record.managed_root
        || p.plan.base_receipt_sha256 != ctx.record.receipt_sha256
    {
        return Err("Patch baseline does not match this installation.".into());
    }
    for name in p
        .plan
        .hashes
        .keys()
        .chain(p.before.keys())
        .chain(p.backups.keys())
    {
        files::safe_relative(name)?;
    }
    if p.backups.len() != 5
        || p.backups.iter().any(|(n, h)| {
            !writable(n)
                || p.plan
                    .hashes
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(n))
                    .map(|(_, hash)| hash)
                    != Some(h)
        })
    {
        return Err("Incomplete or mismatched affected-file backup declaration.".into());
    }
    Ok(p)
}
fn check_undeclared(ctx: &Context<'_>, p: &Prepared) -> Result<Inventory, String> {
    let now = files::inventory(&ctx.game(), false)?;
    let unexpected = changed(&p.before, &now)
        .into_iter()
        .filter(|n| !allowed(n))
        .collect::<Vec<_>>();
    if !unexpected.is_empty() {
        return Err(format!(
            "Files outside the patch changed; automatic restoration is blocked: {:?}",
            &unexpected[..unexpected.len().min(5)]
        ));
    }
    Ok(now)
}
pub(super) fn verify_post(ctx: &Context<'_>, dir: &Path) -> Result<(), String> {
    let p = load_prepared(ctx, dir)?;
    verify_backups(dir, &p)?;
    let a: Applied = read_json(&dir.join("applied.json"))?;
    if files::hash(&dir.join("prepared.json"))? != a.prepared_sha256 {
        return Err("Patch evidence changed.".into());
    }
    for name in a.post_hashes.keys().chain(a.created.iter()) {
        files::safe_relative(name)?;
        if !allowed(&name.to_ascii_lowercase()) {
            return Err("Invalid patch output declaration.".into());
        }
    }
    unchanged_hashes(&ctx.game(), &a.post_hashes)?;
    let now = files::inventory(&ctx.game(), false)?;
    // Runtime writes outside the patch need not block undo. New writes inside its
    // adapter directory do: never delete a user-added file while undoing.
    if changed(&a.after, &now)
        .iter()
        .any(|n| writable(n) || adapter_scope(n))
    {
        return Err("Patched files changed after application.".into());
    }
    unchanged_hashes(
        &ctx.game(),
        &p.plan
            .hashes
            .iter()
            .filter(|(n, _)| !writable(&n.to_ascii_lowercase()))
            .map(|(n, h)| (n.clone(), h.clone()))
            .collect(),
    )
}
pub(super) fn restorable(ctx: &Context<'_>, dir: &Path) -> Result<(), String> {
    let p = load_prepared(ctx, dir)?;
    verify_backups(dir, &p)?;
    unchanged_hashes(
        &ctx.game(),
        &p.plan
            .hashes
            .iter()
            .filter(|(n, _)| !writable(&n.to_ascii_lowercase()))
            .map(|(n, h)| (n.clone(), h.clone()))
            .collect(),
    )?;
    restore_footprint(ctx, dir, &p)
}
fn verify_backups(dir: &Path, p: &Prepared) -> Result<(), String> {
    for (name, hash) in &p.backups {
        if !writable(name) {
            return Err("Unexpected backup declaration.".into());
        }
        let path = dir.join("affected").join(name);
        files::regular(&path)?;
        if files::hash(&path)? != *hash {
            return Err("Affected-file backup changed.".into());
        }
    }
    if p.backups.len() != 5 {
        return Err("Incomplete affected-file backup.".into());
    }
    Ok(())
}
fn restore_footprint(ctx: &Context<'_>, dir: &Path, p: &Prepared) -> Result<(), String> {
    if dir.join("restoring.json").exists() {
        let r: Restoring = read_json(&dir.join("restoring.json"))?;
        if r.prepared_sha256 != files::hash(&dir.join("prepared.json"))? {
            return Err("Restoration evidence changed.".into());
        }
        let now = files::inventory(&ctx.game(), false)?;
        if changed(&r.before, &now).iter().any(|n| !allowed(n)) {
            return Err("Files outside the patch changed during restoration.".into());
        }
        for (name, hash) in &r.removable {
            files::safe_relative(name)?;
            if p.before.contains_key(name) || !adapter_scope(name) || !allowed(name) {
                return Err("Invalid restoration cleanup declaration.".into());
            }
            let path = ctx.game().join(name);
            if path.exists() {
                if let Some(hash) = hash {
                    files::regular(&path)?;
                    if files::hash(&path)? != *hash {
                        return Err("Patch-owned file changed during restoration.".into());
                    }
                } else {
                    files::direct(&path)?;
                    if !path.is_dir() {
                        return Err("Patch-owned directory changed during restoration.".into());
                    }
                }
            }
        }
    } else {
        check_undeclared(ctx, p)?;
    }
    Ok(())
}

fn download(
    cache: &ArtifactCache,
    id: &str,
    url: &str,
    length: u64,
    hash: &str,
    redirect_hosts: Vec<String>,
) -> Result<PathBuf, String> {
    Ok(cache
        .acquire(
            &DownloadRequest {
                request_id: id.into(),
                url: url.into(),
                expected_length: length,
                expected_sha256: hash.into(),
                redirect_hosts,
                max_attempts: crate::acquire::DEFAULT_DOWNLOAD_ATTEMPTS,
            },
            &ConsoleSink,
        )
        .map_err(err)?
        .archive_path)
}
fn prepare_payload(ctx: &Context<'_>, dir: &Path) -> Result<PathBuf, String> {
    let cache = ArtifactCache::open(ctx.cache).map_err(err)?;
    for (name, length, hash) in catalog::FILES {
        let url=format!("https://raw.githubusercontent.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/{}/live-patch/{}/{name}",catalog::OWNER_COMMIT,catalog::ADAPTER);
        let source = download(
            &cache,
            &format!("description-patch-{}", name.replace('/', "-")),
            &url,
            length,
            hash,
            vec![],
        )?;
        let bytes = files::read(&source, 65536)?;
        if sha256_bytes(&bytes) != hash {
            return Err("Adapter bytes changed.".into());
        }
        files::create(&dir.join("payload").join(name), &bytes)?;
    }
    let source = download(
        &cache,
        "patch-weidu-249",
        "https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip",
        2154755,
        "b156910cbec69359fc2e42f6739aa959d49047d6fd3dc172f6bed88ffad8f927",
        vec!["release-assets.githubusercontent.com".into()],
    )?;
    let extraction = extract_archive(
        &source,
        &ctx.cache.join("patch-extracted"),
        &ArchiveRequirements {
            artifact_sha256: "b156910cbec69359fc2e42f6739aa959d49047d6fd3dc172f6bed88ffad8f927"
                .into(),
            format: ArchiveFormat::Zip,
            expected_roots: vec!["weidu.exe".into()],
            expected_tp2_paths: vec![],
            limits: ArchiveLimits {
                max_entries: 64,
                max_total_uncompressed_bytes: 16777216,
                ..Default::default()
            },
            mode: ArchiveMode::Public,
        },
    )
    .map_err(err)?;
    let source = extraction.root.join("weidu.exe");
    VerifiedWeidu::verify(&source, catalog::TOOL_HASH).map_err(err)?;
    let tool = dir.join("weidu.exe");
    files::create(&tool, &files::read(&source, 8 * 1024 * 1024)?)?;
    Ok(tool)
}

pub fn apply(
    ctx: &Context<'_>,
    catalog: &catalog::Catalog,
    token: &str,
    full_backup: bool,
    save_backup: bool,
) -> Result<PatchPreview, String> {
    let _lock = TargetLock::try_acquire(ctx.app_data.join("locks"), &ctx.record.managed_root)
        .map_err(err)?;
    launch_guard(&ctx.record.managed_root)?;
    let plan = plan(ctx, catalog)?;
    if plan.description.changes.is_empty() {
        return inspect(ctx, catalog);
    }
    if fingerprint(&plan, catalog, &ctx.record.install_id)? != token {
        return Err(
            "Installation changed since you checked it. Check again before applying.".into(),
        );
    }
    if ctx.game().join(catalog::ADAPTER).exists() {
        return Err(
            "The patch adapter already exists; automatic replacement is not allowed.".into(),
        );
    }
    close_check(&ctx.game(), &plan.hashes)?;
    let before = files::inventory(&ctx.game(), false)?;
    let profile = if save_backup {
        Some(files::inventory(&ctx.record.managed_save_root, true)?)
    } else {
        None
    };
    let full_bytes = if full_backup {
        files::total(&before)
    } else {
        0
    };
    let save_bytes = profile.as_ref().map(files::total).unwrap_or(0);
    let required = full_bytes
        .checked_add(save_bytes)
        .and_then(|n| n.checked_add(512 * 1024 * 1024))
        .ok_or("Backup size overflow")?;
    if SystemPreflight.available_space(ctx.app_data).map_err(err)? < required
        || SystemPreflight
            .available_space(&ctx.record.managed_root)
            .map_err(err)?
            < 128 * 1024 * 1024
    {
        return Err("Not enough free space for the selected backups and patch.".into());
    }
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(err)?
        .as_nanos()
        .to_string();
    let dir = ctx.state().join(&id);
    files::create(&dir.join("identity"), ctx.record.install_id.as_bytes())?;
    let tool = prepare_payload(ctx, &dir)?; // No game writes, even if a download fails.
    let backup_root = ctx.backups().join(&id);
    files::create(
        &backup_root.join("identity"),
        ctx.record.install_id.as_bytes(),
    )?;
    if full_backup {
        files::copy_tree(&ctx.game(), &backup_root.join("game"), false)?;
    }
    if save_backup {
        files::copy_tree(
            &ctx.record.managed_save_root,
            &backup_root.join("profile"),
            true,
        )?;
    }
    let mut backups = BTreeMap::new();
    for name in [
        "override/bgclatxt.2da",
        "override/clastext.2da",
        "override/sodcltxt.2da",
        "weidu.log",
        "weidu.conf",
    ] {
        let bytes = files::read(&ctx.game().join(name), 16 * 1024 * 1024)?;
        let hash = sha256_bytes(&bytes);
        files::create(&dir.join("affected").join(name), &bytes)?;
        backups.insert(name.into(), hash);
    }
    close_check(&ctx.game(), &plan.hashes)?;
    unchanged_hashes(&ctx.game(), &plan.hashes)?;
    if files::inventory(&ctx.game(), false)? != before {
        return Err("Game changed while preparing the backup.".into());
    }
    let prepared = Prepared {
        schema: 1,
        patch_id: catalog.patch_id.clone(),
        install_id: ctx.record.install_id.clone(),
        managed_root: ctx.record.managed_root.clone(),
        plan,
        before,
        backups,
    };
    files::json_new(&dir.join("prepared.json"), &prepared)?;
    let result = apply_inner(ctx, &dir, &tool, &prepared);
    if let Err(error) = result {
        let _ = files::json_new(&dir.join("failure.json"), &error);
        match restore_inner(ctx, &dir, false) {
            Ok(()) => {
                return Err(format!(
                    "Patch failed; original affected files were restored. {error}"
                ))
            }
            Err(recovery) => {
                return Err(format!(
                    "Patch needs recovery; do not launch this copy. {error}; {recovery}"
                ))
            }
        }
    }
    inspect(ctx, catalog)
}

#[cfg(test)]
pub(super) fn run_prepared_fixture(
    ctx: &Context<'_>,
    dir: &Path,
    tool: &Path,
) -> Result<(), String> {
    apply_inner(ctx, dir, tool, &load_prepared(ctx, dir)?)
}
fn apply_inner(ctx: &Context<'_>, dir: &Path, tool: &Path, p: &Prepared) -> Result<(), String> {
    for (name, _, hash) in catalog::FILES {
        let bytes = files::read(&dir.join("payload").join(name), 65536)?;
        if sha256_bytes(&bytes) != hash {
            return Err("Adapter identity changed.".into());
        }
        files::create(&ctx.game().join(catalog::ADAPTER).join(name), &bytes)?;
    }
    VerifiedWeidu::verify(tool, catalog::TOOL_HASH).map_err(err)?;
    // Deny write/delete replacement of the executable while the runner owns it.
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1);
    }
    let _tool_guard = options.open(tool).map_err(err)?;
    if files::hash(tool)? != catalog::TOOL_HASH {
        return Err("WeiDU identity changed.".into());
    }
    let debug = dir.join("attempt.debug");
    let args = [
        format!("{}/setup-{}.tp2", catalog::ADAPTER, catalog::ADAPTER),
        "--language".into(),
        "0".into(),
        "--use-lang".into(),
        "en_US".into(),
        "--force-install-list".into(),
        "0".into(),
        "--no-exit-pause".into(),
        "--skip-at-view".into(),
        "--safe-exit".into(),
        "--noautoupdate".into(),
        "--log".into(),
        debug.to_string_lossy().into_owned(),
    ]
    .into_iter()
    .map(Into::into)
    .collect::<Vec<_>>();
    files::json_new(&dir.join("invocation.json"), &(tool, &args))?;
    let identity_digest =
        sha256_bytes(&serde_json::to_vec(&(tool, ctx.game(), &args)).map_err(err)?);
    let invocation = Invocation {
        program: tool.to_path_buf(),
        cwd: ctx.game(),
        args,
        prompts: vec![],
        debug_path: debug,
        identity_digest,
    };
    let (_sender, receiver) = crossbeam_channel::unbounded();
    let outcome = runner::run(
        RunnerRequest {
            invocation,
            step_id: catalog::PATCH_ID.into(),
            output_log: dir.join("output.log"),
            silence_threshold: Duration::from_secs(300),
        },
        receiver,
        ConsoleSink,
    );
    if outcome != (RunOutcome::Exited { code: 0 }) {
        return Err(format!("WeiDU did not complete successfully: {outcome:?}"));
    }
    description::verify(&p.plan.description, &tables(&ctx.game())?)?;
    let unchanged = p
        .plan
        .hashes
        .iter()
        .filter(|(n, _)| !writable(&n.to_ascii_lowercase()))
        .map(|(n, h)| (n.clone(), h.clone()))
        .collect();
    unchanged_hashes(&ctx.game(), &unchanged)?;
    let old = fs::read_to_string(dir.join("affected/weidu.log")).map_err(err)?;
    let new = String::from_utf8(files::read(
        &ctx.game().join("WeiDU.log"),
        16 * 1024 * 1024,
    )?)
    .map_err(err)?;
    let old = crate::weidu::log::parse_active_entries(&old).map_err(err)?;
    let new = crate::weidu::log::parse_active_entries(&new).map_err(err)?;
    // WeiDU may regenerate comments from TP2 translations; component identities/order
    // must remain exact. Original version evidence stays in the immutable preimage.
    let tuple = |r: &crate::weidu::log::LogEntry| (r.tp2_key.clone(), r.language, r.component);
    if new.len() != old.len() + 1
        || !new.iter().zip(&old).all(|(a, b)| tuple(a) == tuple(b))
        || new.last().is_none_or(|r| {
            r.tp2_key != "akcb_kit_descriptions/setup-akcb_kit_descriptions.tp2"
                || r.language != 0
                || r.component != 0
        })
    {
        return Err("WeiDU changed outside the expected single patch suffix.".into());
    }
    let after = check_undeclared(ctx, p)?;
    let mut hashes = BTreeMap::new();
    let mut created = vec![];
    for name in changed(&p.before, &after) {
        if !p.before.contains_key(&name) {
            created.push(name.clone());
        }
        if after.get(&name).is_some_and(|s| !s.directory) {
            files::regular(&ctx.game().join(&name))?;
            hashes.insert(name.clone(), files::hash(&ctx.game().join(&name))?);
        }
    }
    files::json_new(
        &dir.join("applied.json"),
        &Applied {
            prepared_sha256: files::hash(&dir.join("prepared.json"))?,
            post_hashes: hashes,
            created,
            after,
        },
    )
}

pub fn restore(
    ctx: &Context<'_>,
    catalog: &catalog::Catalog,
    undo: bool,
) -> Result<PatchPreview, String> {
    let _lock = TargetLock::try_acquire(ctx.app_data.join("locks"), &ctx.record.managed_root)
        .map_err(err)?;
    validate_context(ctx)?;
    let (dir, state) = latest(ctx)?.ok_or("There is no patch to restore.")?;
    if undo && state != "applied" {
        return Err("Only the latest completed patch can be undone.".into());
    }
    if !undo && state != "prepared" && state != "restoring" {
        return Err("No interrupted patch needs restoration.".into());
    }
    restore_inner(ctx, &dir, undo)?;
    inspect(ctx, catalog)
}
fn restore_inner(ctx: &Context<'_>, dir: &Path, undo: bool) -> Result<(), String> {
    let p = load_prepared(ctx, dir)?;
    close_check_inner(&ctx.game(), &p.plan.hashes, true)?;
    if undo {
        verify_post(ctx, dir)?;
    } else {
        restorable(ctx, dir)?;
    }
    for (name, expected) in &p.backups {
        files::regular(&dir.join("affected").join(name))?;
        if files::hash(&dir.join("affected").join(name))? != *expected {
            return Err("Backup hash mismatch.".into());
        }
    }
    if !dir.join("restoring.json").exists() {
        let before = files::inventory(&ctx.game(), false)?;
        let mut removable = BTreeMap::new();
        for (name, stamp) in &before {
            if !p.before.contains_key(name) && adapter_scope(name) && allowed(name) {
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
    restorable(ctx, dir)?;
    for name in p.backups.keys() {
        let target = ctx.game().join(name);
        if target.exists() {
            files::regular(&target)?;
        } else {
            files::direct(target.parent().unwrap())?;
        }
        let bytes = files::read(&dir.join("affected").join(name), 16 * 1024 * 1024)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(target)
            .map_err(err)?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(err)?;
    }
    let r: Restoring = read_json(&dir.join("restoring.json"))?;
    let mut created = r.removable.keys().cloned().collect::<Vec<_>>();
    created.sort_by_key(|n| std::cmp::Reverse(n.len()));
    for name in created {
        let target = ctx.game().join(&name);
        if !target.exists() {
            continue;
        }
        files::direct(&target)?;
        if r.removable[&name].is_none() {
            fs::remove_dir(target).map_err(err)?;
        } else {
            files::regular(&target)?;
            fs::remove_file(target).map_err(err)?;
        }
    }
    unchanged_hashes(&ctx.game(), &p.plan.hashes)?;
    files::json_new(
        &dir.join("restored.json"),
        &files::hash(&dir.join("prepared.json"))?,
    )
}
