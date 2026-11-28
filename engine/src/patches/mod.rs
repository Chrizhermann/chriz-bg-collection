//! A deliberately narrow existing-install patch path. Base receipts and saves are immutable.
pub mod catalog;
pub mod description;
pub(crate) mod files;
mod transaction;
#[cfg(all(test, windows))]
mod transaction_tests;

use crate::{
    digest::sha256_bytes,
    preflight::{PreflightHost, SystemPreflight},
    registry::ManagedInstallRecord,
};
use files::{err, Inventory};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
pub use transaction::{active_log_transition, apply, launch_guard, restore, verified_log_suffix};
pub use transaction::{apply_with_progress, restore_with_progress};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchPreview {
    pub install_id: String,
    pub patch_id: String,
    pub title: String,
    pub state: String,
    pub detail: String,
    pub base_recipe_version: String,
    pub applied_patch_ids: Vec<String>,
    pub review_token: Option<String>,
    pub can_undo: bool,
    pub can_restore: bool,
    pub full_backup_bytes: String,
    pub save_backup_bytes: String,
    pub available_bytes: String,
    pub backup_path: String,
}

pub struct Context<'a> {
    pub record: &'a ManagedInstallRecord,
    pub app_data: &'a Path,
    /// Where the installation is locked, independent of `app_data`.
    pub locks: &'a crate::lock::TargetLockNamespace,
    pub cache: &'a Path,
}
impl Context<'_> {
    pub fn game(&self) -> PathBuf {
        self.record.managed_root.join("game")
    }
    pub fn state(&self) -> PathBuf {
        self.record.managed_root.join(".chriz/patches/cebg-v1")
    }
    pub fn backups(&self) -> PathBuf {
        self.app_data
            .join("patch-backups")
            .join(&self.record.install_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Plan {
    pub description: description::DescriptionPlan,
    pub hashes: BTreeMap<String, String>,
    pub base_receipt_sha256: String,
}

fn validate_context(ctx: &Context<'_>) -> Result<(), String> {
    files::safe_relative(&ctx.record.install_id)?;
    if ctx.record.install_id.contains('/') {
        return Err("Invalid installation ID.".into());
    }
    let root = &ctx.record.managed_root;
    files::direct(root)?;
    files::direct(&ctx.game())?;
    // No folder-name heuristic (such as `steamapps` or Program Files): what keeps a fix inside a
    // CEBG copy is that the root is a registered installation whose receipt matches, below.
    if !root.is_absolute()
        || root.parent().is_none()
        || root.components().count() < 3
        || crate::preflight::is_creator_protected_destination(root)
    {
        return Err("Patching is restricted to a separate managed game installation.".into());
    }
    let completed = crate::recovery_receipt::read_completed_state(root)?;
    if completed.bg1_engine_name.is_none() {
        // The patch catalog targets the Enhanced Edition Trilogy only.
        return Err("Patches are available only for Baldur's Gate installations.".into());
    }
    if completed.bg2_engine_name != ctx.record.engine_name
        || completed.managed_save_root != ctx.record.managed_save_root
        || crate::stage::read_engine_name(ctx.game()).map_err(err)? != ctx.record.engine_name
    {
        return Err("Installation profile no longer matches its completed record.".into());
    }
    // Base receipt stays immutable, including on installations completed by supervised recovery.
    let receipt = root.join(".chriz/install-receipt.json");
    files::regular(&receipt)?;
    if files::hash(&receipt)? != ctx.record.receipt_sha256 {
        return Err("Installation receipt changed.".into());
    }
    Ok(())
}

fn tables(game: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    description::TABLES
        .iter()
        .map(|name| {
            let path = files::resolve_relative(game, &format!("override/{name}"))?;
            Ok(((*name).to_owned(), files::read(&path, 16 * 1024 * 1024)?))
        })
        .collect()
}

fn plan(ctx: &Context<'_>, catalog: &catalog::Catalog) -> Result<Plan, String> {
    crate::manual_checkpoint::game_update_guard(&ctx.record.managed_root)?;
    validate_context(ctx)?;
    let game = ctx.game();
    if !String::from_utf8(files::read(
        &files::resolve_relative(&game, "weidu.conf")?,
        65536,
    )?)
    .map_err(err)?
    .trim()
    .eq_ignore_ascii_case("lang_dir = en_us")
    {
        return Err("This first patch supports English installations only.".into());
    }
    let log_path = crate::weidu::log::resolve_weidu_log(&game).map_err(err)?;
    let log = files::read(&log_path, 16 * 1024 * 1024)?;
    let kitlist = files::read(
        &files::resolve_relative(&game, "override/KITLIST.2DA")?,
        16 * 1024 * 1024,
    )?;
    let tlk = files::read(
        &files::resolve_relative(&game, "lang/en_US/dialog.tlk")?,
        256 * 1024 * 1024,
    )?;
    let description = description::plan(
        &kitlist,
        &tables(&game)?,
        &log,
        &tlk,
        &catalog.supported_versions,
    )?;
    let mut paths = vec![
        "WeiDU.log".to_string(),
        "weidu.conf".into(),
        "engine.lua".into(),
        "chitin.key".into(),
        "override/KITLIST.2DA".into(),
    ];
    let mut game_executables = 0_usize;
    for candidate in crate::games::GameRole::Bg2ee.known_executables() {
        if files::resolve_relative(&game, candidate)?.exists() {
            paths.push((*candidate).to_string());
            game_executables += 1;
        }
    }
    if game_executables == 0 {
        paths.push("Baldur.exe".into());
    }
    paths.extend(description::TABLES.iter().map(|p| format!("override/{p}")));
    let lang_dir = files::resolve_relative(&game, "lang")?;
    for entry in walkdir::WalkDir::new(lang_dir).follow_links(false) {
        let entry = entry.map_err(err)?;
        if entry
            .metadata()
            .is_ok_and(|meta| crate::reparse::metadata_is_link(entry.path(), &meta))
        {
            return Err("Linked language resources are unsupported.".into());
        }
        if entry
            .path()
            .extension()
            .is_some_and(|x| x.eq_ignore_ascii_case("tlk"))
        {
            paths.push(
                entry
                    .path()
                    .strip_prefix(&game)
                    .map_err(err)?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    if files::resolve_relative(&game, "dialog.tlk")?.exists() {
        paths.push("dialog.tlk".into());
    }
    if files::resolve_relative(&game, "dialogf.tlk")?.exists() {
        paths.push("dialogf.tlk".into());
    }
    let mut hashes = BTreeMap::new();
    for name in paths {
        let p = files::resolve_relative(&game, &name)?;
        files::regular(&p)?;
        hashes.insert(name, files::hash(&p)?);
    }
    Ok(Plan {
        description,
        hashes,
        base_receipt_sha256: ctx.record.receipt_sha256.clone(),
    })
}

fn fingerprint(plan: &Plan, catalog: &catalog::Catalog, id: &str) -> Result<String, String> {
    Ok(sha256_bytes(
        &serde_json::to_vec(&(plan, catalog, id)).map_err(err)?,
    ))
}

pub fn inspect(ctx: &Context<'_>, catalog: &catalog::Catalog) -> Result<PatchPreview, String> {
    crate::mod_upgrades::launch_guard(&ctx.record.managed_root)?;
    validate_context(ctx)?;
    let mut preview = PatchPreview {
        install_id: ctx.record.install_id.clone(),
        patch_id: catalog.patch_id.clone(),
        title: catalog.title.clone(),
        state: "unsupported".into(),
        detail: String::new(),
        base_recipe_version: ctx.record.recipe_version.clone(),
        applied_patch_ids: Vec::new(),
        review_token: None,
        can_undo: false,
        can_restore: false,
        full_backup_bytes: "0".into(),
        save_backup_bytes: "0".into(),
        available_bytes: SystemPreflight
            .available_space(ctx.app_data)
            .map_err(err)?
            .to_string(),
        backup_path: ctx.backups().display().to_string(),
    };
    if let Some((dir, state)) = transaction::latest(ctx)? {
        match state.as_str() {
            "applied" => {
                preview.state = "applied".into();
                preview.detail="Description links repaired. Kit abilities and the base collection version are unchanged.".into();
                preview.applied_patch_ids.push(catalog.patch_id.clone());
                preview.can_undo = transaction::verify_post(ctx, &dir).is_ok();
                if !preview.can_undo {
                    preview.detail.push_str(
                        " Files have changed since patching; automatic undo is unavailable.",
                    );
                }
                return Ok(preview);
            }
            "prepared" | "restoring" => {
                preview.state = "needs-recovery".into();
                preview.detail="A patch did not finish. Restore its affected-file backup before playing. Your original installation receipt is preserved.".into();
                preview.can_restore = transaction::restorable(ctx, &dir).is_ok();
                return Ok(preview);
            }
            "restored" => {}
            _ => return Err("Unknown patch transaction state.".into()),
        }
    }
    let plan = match plan(ctx, catalog) {
        Ok(p) => p,
        Err(reason) => {
            if reason == "No installed supported Artisan components" {
                preview.state = "not-selected".into();
                preview.detail = "None of the Artisan components repaired by this fix are installed. Nothing will be added.".into();
                return Ok(preview);
            }
            preview.detail=format!("Not assessed for patching: {reason} Your game has not been changed. Use a new installation for other updates.");
            return Ok(preview);
        }
    };
    if plan.description.changes.is_empty() {
        preview.state = "already-fixed".into();
        preview.detail="These description links already match this game's installed kit text. No patch is needed.".into();
        return Ok(preview);
    }
    if files::resolve_relative(&ctx.game(), catalog::ADAPTER)?.exists() {
        preview.detail =
            "An existing copy of this adapter is present. Automatic replacement is not supported."
                .into();
        return Ok(preview);
    }
    let game_inventory = files::inventory(&ctx.game(), false)?;
    preview.full_backup_bytes = files::total(&game_inventory).to_string();
    preview.save_backup_bytes = files::inventory(&ctx.record.managed_save_root, true)
        .map(|i| files::total(&i))
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "unavailable".into());
    preview.state = "available".into();
    preview.detail = catalog.summary.clone();
    preview.review_token = Some(fingerprint(&plan, catalog, &ctx.record.install_id)?);
    Ok(preview)
}

fn unchanged_hashes(game: &Path, hashes: &BTreeMap<String, String>) -> Result<(), String> {
    for (name, expected) in hashes {
        let path = files::resolve_relative(game, name)?;
        files::regular(&path)?;
        if files::hash(&path)? != *expected {
            return Err(format!("File changed: {name}"));
        }
    }
    Ok(())
}
pub(crate) fn close_check(game: &Path, hashes: &BTreeMap<String, String>) -> Result<(), String> {
    close_check_in(game, "en_US", hashes, None)
}
/// Only after the caller has validated restoration ownership/backups. Existing
/// files remain exclusively probed; a missing TLK must not block its restoration.
pub(crate) fn close_restore_check(
    game: &Path,
    hashes: &BTreeMap<String, String>,
) -> Result<(), String> {
    close_check_in(game, "en_US", hashes, Some(&writable))
}
/// Nothing may run from `game` or hold the files named in `hashes` open; `language` is the
/// game's language folder. While restoring, the files `restorable` accepts may be missing.
pub(crate) fn close_check_in(
    game: &Path,
    language: &str,
    hashes: &BTreeMap<String, String>,
    restorable: Option<&dyn Fn(&str) -> bool>,
) -> Result<(), String> {
    if restorable.is_some() {
        crate::preflight::recheck_staging_target_before_mutation(game, language).map_err(err)?;
    } else {
        crate::preflight::recheck_target_before_mutation(game, language).map_err(err)?;
    }
    let mut paths = Vec::new();
    for name in hashes.keys() {
        let path = files::resolve_relative(game, name)?;
        if restorable.is_some_and(|restorable| restorable(&name.to_ascii_lowercase()))
            && !path.exists()
        {
            files::direct(path.parent().ok_or("Missing restore parent")?)?;
        } else {
            files::regular(&path)?;
            paths.push(path);
        }
    }
    SystemPreflight
        .probe_exclusive_writable_files(&paths)
        .map_err(|e| {
            format!(
                "Close the game or other writer: {} ({})",
                e.path.display(),
                e.source
            )
        })?;
    if has_running_installer_targeting(game)? {
        return Err("Close any installer targeting this game before patching.".into());
    }
    Ok(())
}

fn is_weidu_installer_exe_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("weidu.exe")
        || (name.len() > "setup-.exe".len()
            && name
                .get(..6)
                .is_some_and(|p| p.eq_ignore_ascii_case("setup-"))
            && name
                .get(name.len() - 4..)
                .is_some_and(|s| s.eq_ignore_ascii_case(".exe")))
}

#[cfg(any(target_os = "linux", test))]
fn is_weidu_installer_process_name(name: &str) -> bool {
    is_weidu_installer_exe_name(name)
        || name.eq_ignore_ascii_case("weidu")
        || (name.len() > "setup-".len()
            && name
                .get(..6)
                .is_some_and(|p| p.eq_ignore_ascii_case("setup-"))
            && !name.contains('.'))
}

#[cfg(windows)]
fn normalize_win32_match_text(text: &str) -> String {
    text.replace('/', "\\")
        .trim_start_matches("\\\\?\\")
        .to_ascii_lowercase()
}

#[cfg(windows)]
fn process_command_line(process: windows_sys::Win32::Foundation::HANDLE) -> Option<String> {
    #[repr(C)]
    struct UnicodeString {
        length: u16,
        maximum_length: u16,
        buffer: *const u16,
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtQueryInformationProcess(
            process_handle: windows_sys::Win32::Foundation::HANDLE,
            process_information_class: u32,
            process_information: *mut core::ffi::c_void,
            process_information_length: u32,
            return_length: *mut u32,
        ) -> i32;
    }
    const PROCESS_COMMAND_LINE_INFORMATION: u32 = 60;
    const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC000_0004_u32 as i32;
    const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023_u32 as i32;
    const STATUS_BUFFER_OVERFLOW: i32 = 0x8000_0005_u32 as i32;

    let mut words = vec![0_u64; 512]; // 4096 bytes, 8-byte aligned
    let mut needed = 0_u32;
    let mut status = unsafe {
        NtQueryInformationProcess(
            process,
            PROCESS_COMMAND_LINE_INFORMATION,
            words.as_mut_ptr().cast(),
            (words.len() * std::mem::size_of::<u64>()) as u32,
            &mut needed,
        )
    };
    if matches!(
        status,
        STATUS_INFO_LENGTH_MISMATCH | STATUS_BUFFER_TOO_SMALL | STATUS_BUFFER_OVERFLOW
    ) && needed > 0
        && needed <= 1024 * 1024
    {
        words.resize((needed as usize).div_ceil(std::mem::size_of::<u64>()), 0);
        status = unsafe {
            NtQueryInformationProcess(
                process,
                PROCESS_COMMAND_LINE_INFORMATION,
                words.as_mut_ptr().cast(),
                (words.len() * std::mem::size_of::<u64>()) as u32,
                &mut needed,
            )
        };
    }
    if status < 0 {
        return None;
    }
    let header = unsafe { &*(words.as_ptr() as *const UnicodeString) };
    if header.buffer.is_null() || header.length == 0 {
        return Some(String::new());
    }
    let byte_start = words.as_ptr() as usize;
    let byte_end = byte_start + words.len() * std::mem::size_of::<u64>();
    let ptr = header.buffer as usize;
    let byte_len = header.length as usize;
    if ptr < byte_start || ptr.saturating_add(byte_len) > byte_end {
        return None;
    }
    let slice = unsafe { std::slice::from_raw_parts(header.buffer, byte_len / 2) };
    Some(String::from_utf16_lossy(slice))
}

#[cfg(windows)]
fn has_running_installer_targeting(game: &Path) -> Result<bool, String> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER,
        ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    let target =
        normalize_win32_match_text(&crate::reparse::display_windows_path(game).to_string_lossy());
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(err(std::io::Error::last_os_error()));
    }
    let _snapshot = HandleGuard(snapshot);
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    if !has_entry {
        let error = unsafe { GetLastError() };
        if error != ERROR_NO_MORE_FILES {
            return Err(err(std::io::Error::from_raw_os_error(error as i32)));
        }
    }
    while has_entry {
        let name_len = entry
            .szExeFile
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(entry.szExeFile.len());
        let process_name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);
        if is_weidu_installer_exe_name(&process_name) {
            let handle =
                unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32ProcessID) };
            if handle.is_null() {
                let code = unsafe { GetLastError() };
                if !matches!(code, ERROR_ACCESS_DENIED | ERROR_INVALID_PARAMETER) {
                    return Err(err(std::io::Error::from_raw_os_error(code as i32)));
                }
            } else {
                let _guard = HandleGuard(handle);
                let mut image_buf = vec![0_u16; 4096];
                let mut image_len = image_buf.len() as u32;
                if unsafe {
                    QueryFullProcessImageNameW(
                        handle,
                        PROCESS_NAME_WIN32,
                        image_buf.as_mut_ptr(),
                        &mut image_len,
                    )
                } != 0
                {
                    let image_path = PathBuf::from(std::ffi::OsString::from_wide(
                        &image_buf[..image_len as usize],
                    ));
                    if crate::reparse::windows_path_starts_with(&image_path, game) {
                        return Ok(true);
                    }
                }
                if let Some(cmd) = process_command_line(handle) {
                    if !target.is_empty() && normalize_win32_match_text(&cmd).contains(&target) {
                        return Ok(true);
                    }
                }
            }
        }
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
        if !has_entry {
            let error = unsafe { GetLastError() };
            if error != ERROR_NO_MORE_FILES {
                return Err(err(std::io::Error::from_raw_os_error(error as i32)));
            }
        }
    }
    Ok(false)
}

#[cfg(target_os = "linux")]
fn has_running_installer_targeting(game: &Path) -> Result<bool, String> {
    linux_installer_guard::has_running_installer_targeting(
        Path::new("/proc"),
        game,
        Some(std::process::id()),
    )
}

#[cfg(not(any(windows, target_os = "linux")))]
fn has_running_installer_targeting(_game: &Path) -> Result<bool, String> {
    Ok(false)
}

#[cfg(any(target_os = "linux", test))]
mod linux_installer_guard {
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    use super::is_weidu_installer_process_name;

    #[derive(Debug, Default)]
    pub(super) struct InstallerProcSnapshot {
        pub(super) comm: Option<String>,
        pub(super) exe: Option<PathBuf>,
        pub(super) cwd: Option<PathBuf>,
        pub(super) cmdline: Vec<String>,
    }

    pub(super) fn has_running_installer_targeting(
        proc_root: &Path,
        game: &Path,
        self_pid: Option<u32>,
    ) -> Result<bool, String> {
        let entries = fs::read_dir(proc_root).map_err(super::err)?;
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) if is_ignored_proc_error(&error) => continue,
                Err(error) => return Err(super::err(error)),
            };
            let name = entry.file_name();
            let Some(pid_text) = name
                .to_str()
                .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
            else {
                continue;
            };
            let Ok(pid) = pid_text.parse::<u32>() else {
                continue;
            };
            if self_pid.is_some_and(|me| me == pid) {
                continue;
            }
            let pid_dir = entry.path();
            let comm = match fs::read_to_string(pid_dir.join("comm")) {
                Ok(text) => Some(text.trim_end_matches(['\r', '\n']).to_owned()),
                Err(error) if is_ignored_proc_error(&error) => None,
                Err(error) => return Err(super::err(error)),
            };
            let cmdline = match fs::read(pid_dir.join("cmdline")) {
                Ok(bytes) => parse_nul_args(&bytes),
                Err(error) if is_ignored_proc_error(&error) => Vec::new(),
                Err(error) => return Err(super::err(error)),
            };
            let exe = read_proc_link(&pid_dir.join("exe")).map_err(super::err)?;
            let preliminary = InstallerProcSnapshot {
                comm,
                exe,
                cwd: None,
                cmdline,
            };
            if !snapshot_is_weidu_installer(&preliminary) {
                continue;
            }
            let cwd = read_proc_link(&pid_dir.join("cwd")).map_err(super::err)?;
            let snapshot = InstallerProcSnapshot { cwd, ..preliminary };
            if snapshot_targets_game(&snapshot, game) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn is_ignored_proc_error(error: &io::Error) -> bool {
        matches!(
            error.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied | io::ErrorKind::InvalidInput
        ) || error.raw_os_error() == Some(3)
    }

    fn read_proc_link(path: &Path) -> io::Result<Option<PathBuf>> {
        match fs::read_link(path) {
            Ok(target) => {
                let cleaned = match target.to_str().and_then(|s| s.strip_suffix(" (deleted)")) {
                    Some(trimmed) if !trimmed.is_empty() => PathBuf::from(trimmed),
                    _ => target,
                };
                Ok(Some(cleaned))
            }
            Err(error) if is_ignored_proc_error(&error) => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn parse_nul_args(bytes: &[u8]) -> Vec<String> {
        bytes
            .split(|byte| *byte == 0)
            .filter(|part| !part.is_empty())
            .map(|part| String::from_utf8_lossy(part).into_owned())
            .collect()
    }

    fn extract_file_name(raw: &str) -> &str {
        raw.rsplit(['/', '\\'])
            .find(|segment| !segment.is_empty())
            .unwrap_or(raw)
    }

    fn normalize_posix_match_path(path: &Path) -> PathBuf {
        let text = path.to_string_lossy();
        normalize_raw_match_path(&text)
    }

    fn normalize_raw_match_path(raw: &str) -> PathBuf {
        let trimmed = raw.trim();
        let without_wine_z = if trimmed.len() >= 3
            && (trimmed.as_bytes()[0] == b'z' || trimmed.as_bytes()[0] == b'Z')
            && trimmed.as_bytes()[1] == b':'
            && (trimmed.as_bytes()[2] == b'\\' || trimmed.as_bytes()[2] == b'/')
        {
            &trimmed[2..]
        } else {
            trimmed
        };
        PathBuf::from(without_wine_z.replace('\\', "/"))
    }

    fn snapshot_is_weidu_installer(snapshot: &InstallerProcSnapshot) -> bool {
        if snapshot
            .comm
            .as_deref()
            .is_some_and(is_weidu_installer_process_name)
        {
            return true;
        }
        if snapshot
            .exe
            .as_ref()
            .and_then(|exe| exe.file_name())
            .and_then(|name| name.to_str())
            .is_some_and(is_weidu_installer_process_name)
        {
            return true;
        }
        snapshot
            .cmdline
            .iter()
            .any(|arg| is_weidu_installer_process_name(extract_file_name(arg)))
    }

    pub(super) fn snapshot_targets_game(snapshot: &InstallerProcSnapshot, game: &Path) -> bool {
        if !snapshot_is_weidu_installer(snapshot) {
            return false;
        }
        let target = normalize_posix_match_path(game);
        if target.as_os_str().is_empty() {
            return false;
        }
        if snapshot
            .exe
            .as_ref()
            .is_some_and(|exe| normalize_posix_match_path(exe).starts_with(&target))
        {
            return true;
        }
        if snapshot
            .cwd
            .as_ref()
            .is_some_and(|cwd| normalize_posix_match_path(cwd).starts_with(&target))
        {
            return true;
        }
        snapshot.cmdline.iter().any(|arg| {
            let candidate = arg.strip_prefix("--game=").unwrap_or(arg.as_str());
            !candidate.is_empty() && normalize_raw_match_path(candidate).starts_with(&target)
        })
    }
}

fn changed(before: &Inventory, after: &Inventory) -> Vec<String> {
    before
        .keys()
        .chain(after.keys())
        .filter(|k| before.get(*k) != after.get(*k))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn writable(name: &str) -> bool {
    [
        "override/bgclatxt.2da",
        "override/clastext.2da",
        "override/sodcltxt.2da",
        "weidu.log",
        "weidu.conf",
    ]
    .contains(&name)
}
fn adapter_scope(name: &str) -> bool {
    name == "akcb_kit_descriptions" || name.starts_with("akcb_kit_descriptions/")
}
fn allowed(name: &str) -> bool {
    // Exact footprint observed with pinned WeiDU 249, not an entire directory wildcard.
    writable(name)
        || [
            "akcb_kit_descriptions",
            "akcb_kit_descriptions/lib",
            "akcb_kit_descriptions/backup",
            "akcb_kit_descriptions/backup/0",
            "akcb_kit_descriptions/setup-akcb_kit_descriptions.tp2",
            "akcb_kit_descriptions/lib/kit_strref.tpa",
            "akcb_kit_descriptions/backup/0/args.0",
            "akcb_kit_descriptions/backup/0/args.0.text",
            "akcb_kit_descriptions/backup/0/bgclatxt.2da",
            "akcb_kit_descriptions/backup/0/clastext.2da",
            "akcb_kit_descriptions/backup/0/sodcltxt.2da",
            "akcb_kit_descriptions/backup/0/mappings.0",
            "akcb_kit_descriptions/backup/0/move.0",
            "akcb_kit_descriptions/backup/0/other.0",
            "akcb_kit_descriptions/backup/0/readln.0",
            "akcb_kit_descriptions/backup/0/readln.0.text",
            "akcb_kit_descriptions/backup/0/tlkpath.0",
            "akcb_kit_descriptions/backup/0/uninstall.0",
        ]
        .contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn installer_process_name_filter_matches_weidu_and_setup_exes() {
        assert!(is_weidu_installer_exe_name("WeiDU.exe"));
        assert!(is_weidu_installer_exe_name("setup-eet.exe"));
        assert!(is_weidu_installer_exe_name("Setup-Stratagems.EXE"));
        assert!(!is_weidu_installer_exe_name("setup-.exe"));
        assert!(!is_weidu_installer_exe_name("notepad.exe"));
        assert!(!is_weidu_installer_exe_name("setup-eet.txt"));
        assert!(is_weidu_installer_process_name("weidu"));
        assert!(is_weidu_installer_process_name("setup-stratagems"));
        assert!(!is_weidu_installer_process_name("setup-eet.txt"));
        assert!(!is_weidu_installer_process_name("setup-"));
    }

    #[test]
    fn linux_installer_guard_detects_targeted_game_via_cwd_cmdline_or_wine_z() {
        let game = Path::new("/home/deck/Games/CEBG/game");
        let other = Path::new("/home/deck/Games/CEBG/game2");

        // Native weidu with cwd = game
        assert!(linux_installer_guard::snapshot_targets_game(
            &linux_installer_guard::InstallerProcSnapshot {
                comm: Some("weidu".into()),
                exe: Some(PathBuf::from("/home/deck/.local/share/chriz/weidu")),
                cwd: Some(game.to_path_buf()),
                cmdline: vec!["weidu".into(), "foo.tp2".into()],
            },
            game
        ));

        // Prefix-boundary safety: /home/.../game2 must NOT match /home/.../game
        assert!(!linux_installer_guard::snapshot_targets_game(
            &linux_installer_guard::InstallerProcSnapshot {
                comm: Some("weidu".into()),
                exe: Some(other.join("setup-foo")),
                cwd: Some(other.to_path_buf()),
                cmdline: vec![other.join("setup-foo").to_string_lossy().into_owned()],
            },
            game
        ));

        // Wine Z:\... targeting via /proc scan
        let temp = TempDir::new().unwrap();
        let pid_dir = temp.path().join("404");
        std::fs::create_dir(&pid_dir).unwrap();
        std::fs::write(pid_dir.join("comm"), "setup-eet.exe\n").unwrap();
        std::fs::write(
            pid_dir.join("cmdline"),
            b"Z:\\home\\deck\\Games\\CEBG\\game\\setup-eet.exe\0--skip-at-view\0",
        )
        .unwrap();
        assert!(linux_installer_guard::has_running_installer_targeting(
            temp.path(),
            game,
            Some(999)
        )
        .unwrap());
        assert!(!linux_installer_guard::has_running_installer_targeting(
            temp.path(),
            other,
            Some(999)
        )
        .unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn nt_query_process_command_line_reads_current_process() {
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        let cmd = process_command_line(unsafe { GetCurrentProcess() })
            .expect("NtQueryInformationProcess(60) should read current process command line");
        assert!(!cmd.is_empty());
    }
}
