//! Independent regular-file staging for the two game roots in one managed install.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::ops::Range;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{EngineError, Result};
use crate::games::GameRole;

/// The fixed on-disk roots owned by one managed EET installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedLayout {
    root: PathBuf,
    bg1_source: PathBuf,
    bg2_source: PathBuf,
    bg1_root: PathBuf,
    game_root: PathBuf,
    state_root: PathBuf,
}

impl ManagedLayout {
    /// Validates two read-only sources and creates the fixed managed directory layout.
    ///
    /// Equal, nested, aliased, or reparse-backed roots are rejected before any managed
    /// directory is created. Existing managed roots may contain only the three paths owned
    /// by the installer: `bg1`, `game`, and `.chriz`.
    pub fn prepare(
        managed_root: impl AsRef<Path>,
        bg1_source: impl AsRef<Path>,
        bg2_source: impl AsRef<Path>,
    ) -> Result<Self> {
        let bg1_source = canonical_direct_directory(bg1_source.as_ref())?;
        let bg2_source = canonical_direct_directory(bg2_source.as_ref())?;
        reject_overlapping_roots(&bg1_source, &bg2_source)?;

        let requested_root = absolute_path(managed_root.as_ref())?;
        let prospective_root = canonicalize_for_creation(&requested_root)?;
        reject_overlapping_roots(&bg1_source, &prospective_root)?;
        reject_overlapping_roots(&bg2_source, &prospective_root)?;

        inspect_existing_managed_root(&requested_root)?;
        create_directories(&requested_root)?;
        let root = canonical_direct_directory(&requested_root)?;
        reject_overlapping_roots(&bg1_source, &root)?;
        reject_overlapping_roots(&bg2_source, &root)?;

        let state_root = root.join(".chriz");
        let bg1_root = root.join("bg1");
        let game_root = root.join("game");
        for path in [&state_root, &bg1_root, &game_root] {
            create_directories(path)?;
            ensure_direct_directory(path)?;
        }

        Ok(Self {
            root,
            bg1_source,
            bg2_source,
            bg1_root,
            game_root,
            state_root,
        })
    }

    /// Canonical managed root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Canonical, read-only BGEE+SoD source root.
    pub fn bg1_source(&self) -> &Path {
        &self.bg1_source
    }

    /// Canonical, read-only BG2EE source root.
    pub fn bg2_source(&self) -> &Path {
        &self.bg2_source
    }

    /// Managed BGEE+SoD working copy.
    pub fn bg1_root(&self) -> &Path {
        &self.bg1_root
    }

    /// Managed BG2EE/EET working copy.
    pub fn game_root(&self) -> &Path {
        &self.game_root
    }

    /// Installer-owned state root.
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    fn paths_for(&self, role: GameRole) -> (&Path, &Path, PathBuf) {
        match role {
            GameRole::BgeeSod => (
                &self.bg1_source,
                &self.bg1_root,
                self.state_root.join("staging/bg1"),
            ),
            GameRole::Bg2ee => (
                &self.bg2_source,
                &self.game_root,
                self.state_root.join("staging/game"),
            ),
        }
    }
}

const SAVE_OWNER_MARKER: &str = ".chriz-managed-install.json";
const SAVE_OWNER_SCHEMA: u32 = 1;

/// The isolated save identity assigned to one managed installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveIdentity {
    /// Value written to each game's `engine_name` assignment.
    pub engine_name: String,
    /// Exact Documents child the game derives from [`Self::engine_name`].
    pub save_root: PathBuf,
    marker: SaveOwner,
}

impl SaveIdentity {
    /// Installer ownership marker inside the isolated save root.
    pub fn owner_marker(&self) -> PathBuf {
        self.save_root.join(SAVE_OWNER_MARKER)
    }
}

/// A save identity whose on-disk ownership marker has been verified.
///
/// This type can only be constructed by [`reserve_save_identity`]. Staging APIs accept
/// it instead of a proposal so a colliding or unclaimed save root cannot be patched into
/// either game by mistake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservedSaveIdentity {
    /// Value written to each game's `engine_name` assignment.
    pub engine_name: String,
    /// Exact, installer-owned Documents child used for saves.
    pub save_root: PathBuf,
    marker: SaveOwner,
}

impl ReservedSaveIdentity {
    /// Installer ownership marker inside the isolated save root.
    pub fn owner_marker(&self) -> PathBuf {
        self.save_root.join(SAVE_OWNER_MARKER)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SaveOwner {
    schema: u32,
    install_id: String,
    managed_root: String,
    engine_name: String,
}

/// Resolves the current user's Documents directory without assuming its location.
pub trait DocumentsLocator {
    /// Returns the user-configured Documents known folder.
    fn documents_dir(&self) -> io::Result<PathBuf>;
}

/// Production Documents resolver.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemDocuments;

#[cfg(windows)]
impl DocumentsLocator for SystemDocuments {
    fn documents_dir(&self) -> io::Result<PathBuf> {
        use std::ffi::c_void;
        use std::os::windows::ffi::OsStringExt;
        use std::ptr;
        use windows_sys::Win32::Foundation::RPC_E_CHANGED_MODE;
        use windows_sys::Win32::System::Com::{
            CoInitializeEx, CoTaskMemFree, COINIT_APARTMENTTHREADED,
        };
        use windows_sys::Win32::UI::Shell::{FOLDERID_Documents, SHGetKnownFolderPath};

        let initialization =
            unsafe { CoInitializeEx(ptr::null(), COINIT_APARTMENTTHREADED as u32) };
        let _com_guard = if initialization >= 0 {
            ComInitializationGuard { balance: true }
        } else if initialization == RPC_E_CHANGED_MODE {
            ComInitializationGuard { balance: false }
        } else {
            return Err(io::Error::other(format!(
                "CoInitializeEx failed with HRESULT 0x{:08X}",
                initialization as u32
            )));
        };

        let mut raw = ptr::null_mut();
        let result =
            unsafe { SHGetKnownFolderPath(&FOLDERID_Documents, 0, ptr::null_mut(), &mut raw) };
        if result < 0 {
            if !raw.is_null() {
                unsafe { CoTaskMemFree(raw.cast::<c_void>()) };
            }
            return Err(io::Error::other(format!(
                "SHGetKnownFolderPath(FOLDERID_Documents) failed with HRESULT 0x{:08X}",
                result as u32
            )));
        }
        if raw.is_null() {
            return Err(io::Error::other(
                "SHGetKnownFolderPath(FOLDERID_Documents) returned a null path",
            ));
        }

        let length = unsafe {
            let mut length = 0_usize;
            while *raw.add(length) != 0 {
                length += 1;
            }
            length
        };
        let path = PathBuf::from(OsString::from_wide(unsafe {
            std::slice::from_raw_parts(raw, length)
        }));
        unsafe { CoTaskMemFree(raw.cast::<c_void>()) };
        Ok(path)
    }
}

#[cfg(windows)]
struct ComInitializationGuard {
    balance: bool,
}

#[cfg(windows)]
impl Drop for ComInitializationGuard {
    fn drop(&mut self) {
        if self.balance {
            unsafe { windows_sys::Win32::System::Com::CoUninitialize() };
        }
    }
}

#[cfg(not(windows))]
impl DocumentsLocator for SystemDocuments {
    fn documents_dir(&self) -> io::Result<PathBuf> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "the system Documents resolver is available only on Windows",
        ))
    }
}

/// The reports produced while staging both pristine game copies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialStageReports {
    /// BGEE+SoD copy, whose save identity is patched immediately.
    pub bg1: StageReport,
    /// BG2EE copy, left pristine until EET finalization completes.
    pub bg2: StageReport,
}

/// Derives, but does not reserve, a deterministic save identity for one install.
pub fn propose_save_identity(
    documents_root: impl AsRef<Path>,
    layout: &ManagedLayout,
    display_name: &str,
    install_id: &str,
) -> Result<SaveIdentity> {
    if install_id.trim().is_empty() {
        return Err(EngineError::GameIdentity {
            path: layout.root().to_path_buf(),
            reason: "install id must not be empty".to_owned(),
        });
    }
    let documents_root = documents_root.as_ref();
    if !documents_root.is_absolute() {
        return Err(EngineError::GameIdentity {
            path: documents_root.to_path_buf(),
            reason: "Documents path must be absolute".to_owned(),
        });
    }
    let metadata = fs::metadata(documents_root).map_err(|source| EngineError::Io {
        path: documents_root.to_path_buf(),
        source,
    })?;
    if !metadata.is_dir() {
        return Err(EngineError::GameIdentity {
            path: documents_root.to_path_buf(),
            reason: "Documents path is not a directory".to_owned(),
        });
    }

    let normalized_name = normalize_display_name(display_name);
    let managed_root = identity_path(layout.root())?;
    let mut digest = Sha256::new();
    digest.update(install_id.as_bytes());
    digest.update([0]);
    digest.update(managed_root.as_bytes());
    let suffix = &hex::encode(digest.finalize())[..12];
    let engine_name = format!("{normalized_name} - {suffix}");
    let save_root = documents_root.join(&engine_name);
    let marker = SaveOwner {
        schema: SAVE_OWNER_SCHEMA,
        install_id: install_id.to_owned(),
        managed_root,
        engine_name: engine_name.clone(),
    };
    Ok(SaveIdentity {
        engine_name,
        save_root,
        marker,
    })
}

/// Claims the unique save root, or proves it already belongs to the same install.
pub fn reserve_save_identity<L: DocumentsLocator>(
    locator: &L,
    layout: &ManagedLayout,
    display_name: &str,
    install_id: &str,
) -> Result<ReservedSaveIdentity> {
    let documents = locator.documents_dir().map_err(|source| EngineError::Io {
        path: PathBuf::from("<Documents known folder>"),
        source,
    })?;
    let identity = propose_save_identity(documents, layout, display_name, install_id)?;
    let newly_created = match fs::create_dir(&identity.save_root) {
        Ok(()) => true,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => false,
        Err(source) => {
            return Err(EngineError::Io {
                path: identity.save_root.clone(),
                source,
            })
        }
    };

    if newly_created {
        if let Err(error) = write_save_owner(&identity) {
            cleanup_failed_save_reservation(&identity)?;
            return Err(error);
        }
        return Ok(reserved_identity(identity));
    }

    let metadata = fs::symlink_metadata(&identity.save_root).map_err(|source| EngineError::Io {
        path: identity.save_root.clone(),
        source,
    })?;
    if metadata.file_type().is_symlink()
        || has_windows_reparse_attribute(&metadata)
        || !metadata.is_dir()
    {
        return Err(save_root_collision(
            &identity,
            "existing path is not a direct directory",
        ));
    }
    let existing = read_save_owner(&identity)?;
    if existing != identity.marker {
        return Err(save_root_collision(
            &identity,
            "ownership marker belongs to a different managed installation",
        ));
    }
    Ok(reserved_identity(identity))
}

/// Stages both pristine games and immediately isolates BGEE+SoD saves.
///
/// BG2EE deliberately retains its source identity here because EET may replace
/// `engine.lua`; call [`finalize_game_identity`] after all EET finalization/tail runs.
pub fn stage_initial_games(
    layout: &ManagedLayout,
    identity: &ReservedSaveIdentity,
) -> Result<InitialStageReports> {
    verify_reserved_save_identity(layout, identity)?;
    let bg1 = stage_game_copy(layout, GameRole::BgeeSod)?;
    let isolate_bg1 = verify_reserved_save_identity(layout, identity)
        .and_then(|()| patch_engine_name(layout.bg1_root(), identity));
    if let Err(error) = isolate_bg1 {
        let (_, target, scratch) = layout.paths_for(GameRole::BgeeSod);
        discard_role_stage(layout, target, &scratch)?;
        return Err(error);
    }
    let bg2 = stage_game_copy(layout, GameRole::Bg2ee)?;
    Ok(InitialStageReports { bg1, bg2 })
}

/// Reapplies and verifies the isolated identity after EET has finalized BG2EE.
pub fn finalize_game_identity(
    layout: &ManagedLayout,
    identity: &ReservedSaveIdentity,
) -> Result<()> {
    verify_reserved_save_identity(layout, identity)?;
    patch_engine_name(layout.game_root(), identity)
}

/// Reads the one effective `engine_name` assignment from a game root.
pub fn read_engine_name(game_root: impl AsRef<Path>) -> Result<String> {
    let path = game_root.as_ref().join("engine.lua");
    ensure_no_reparse_ancestors(&path)?;
    ensure_direct_file(&path)?;
    let content = fs::read_to_string(&path).map_err(|source| EngineError::Io {
        path: path.clone(),
        source,
    })?;
    let assignments = engine_name_assignments(&path, &content)?;
    if assignments.len() != 1 {
        return Err(EngineError::GameIdentity {
            path,
            reason: format!(
                "expected exactly one engine_name assignment, found {}",
                assignments.len()
            ),
        });
    }
    Ok(assignments[0].value.clone())
}

fn normalize_display_name(display_name: &str) -> String {
    let mut normalized = String::new();
    let mut separator_pending = false;
    for character in display_name.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
            if separator_pending && !normalized.is_empty() {
                normalized.push(' ');
            }
            normalized.push(character);
            separator_pending = false;
        } else {
            separator_pending = true;
        }
        if normalized.len() >= 96 {
            break;
        }
    }
    let normalized = normalized.trim_matches([' ', '.', '-']).trim();
    if normalized.is_empty() {
        "Chriz EET".to_owned()
    } else {
        normalized.to_owned()
    }
}

fn identity_path(path: &Path) -> Result<String> {
    let value = path
        .to_str()
        .ok_or_else(|| EngineError::GameIdentity {
            path: path.to_path_buf(),
            reason: "managed root must be representable as Unicode".to_owned(),
        })?
        .replace('\\', "/");
    #[cfg(windows)]
    {
        Ok(value.to_ascii_lowercase())
    }
    #[cfg(not(windows))]
    {
        Ok(value)
    }
}

fn write_save_owner(identity: &SaveIdentity) -> Result<()> {
    let marker_path = identity.owner_marker();
    let payload = serde_json::to_vec_pretty(&identity.marker).map_err(|source| {
        EngineError::GameIdentity {
            path: marker_path.clone(),
            reason: format!("could not serialize ownership marker: {source}"),
        }
    })?;
    let mut marker = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&marker_path)
        .map_err(|source| EngineError::Io {
            path: marker_path.clone(),
            source,
        })?;
    marker
        .write_all(&payload)
        .and_then(|()| marker.sync_all())
        .map_err(|source| EngineError::Io {
            path: marker_path,
            source,
        })
}

fn read_save_owner(identity: &SaveIdentity) -> Result<SaveOwner> {
    let marker_path = identity.owner_marker();
    let metadata = match fs::symlink_metadata(&marker_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(save_root_collision(
                identity,
                "existing directory has no installer ownership marker",
            ))
        }
        Err(source) => {
            return Err(EngineError::Io {
                path: marker_path,
                source,
            })
        }
    };
    if metadata.file_type().is_symlink()
        || has_windows_reparse_attribute(&metadata)
        || !metadata.is_file()
    {
        return Err(save_root_collision(
            identity,
            "ownership marker is not a direct regular file",
        ));
    }
    let payload = fs::read(&marker_path).map_err(|source| EngineError::Io {
        path: marker_path,
        source,
    })?;
    serde_json::from_slice(&payload).map_err(|source| {
        save_root_collision(identity, format!("ownership marker is invalid: {source}"))
    })
}

fn save_root_collision(identity: &SaveIdentity, reason: impl Into<String>) -> EngineError {
    EngineError::SaveRootCollision {
        path: identity.save_root.clone(),
        reason: reason.into(),
    }
}

fn reserved_identity(identity: SaveIdentity) -> ReservedSaveIdentity {
    ReservedSaveIdentity {
        engine_name: identity.engine_name,
        save_root: identity.save_root,
        marker: identity.marker,
    }
}

fn verify_reserved_save_identity(
    layout: &ManagedLayout,
    identity: &ReservedSaveIdentity,
) -> Result<()> {
    if identity.engine_name != identity.marker.engine_name
        || identity
            .save_root
            .file_name()
            .and_then(|name| name.to_str())
            != Some(identity.engine_name.as_str())
    {
        return Err(EngineError::GameIdentity {
            path: identity.save_root.clone(),
            reason: "reserved identity fields no longer match its ownership record".to_owned(),
        });
    }
    if identity.marker.managed_root != identity_path(layout.root())? {
        return Err(EngineError::GameIdentity {
            path: identity.save_root.clone(),
            reason: "reserved identity belongs to a different managed layout".to_owned(),
        });
    }
    let root_metadata =
        fs::symlink_metadata(&identity.save_root).map_err(|source| EngineError::Io {
            path: identity.save_root.clone(),
            source,
        })?;
    reject_reparse(&identity.save_root, &root_metadata)?;
    if !root_metadata.is_dir() {
        return Err(EngineError::GameIdentity {
            path: identity.save_root.clone(),
            reason: "reserved save root is no longer a direct directory".to_owned(),
        });
    }
    let proposal = SaveIdentity {
        engine_name: identity.engine_name.clone(),
        save_root: identity.save_root.clone(),
        marker: identity.marker.clone(),
    };
    let existing = read_save_owner(&proposal)?;
    if existing != proposal.marker {
        return Err(save_root_collision(
            &proposal,
            "ownership marker changed after reservation",
        ));
    }
    Ok(())
}

fn cleanup_failed_save_reservation(identity: &SaveIdentity) -> Result<()> {
    let root_metadata =
        fs::symlink_metadata(&identity.save_root).map_err(|source| EngineError::Io {
            path: identity.save_root.clone(),
            source,
        })?;
    reject_reparse(&identity.save_root, &root_metadata)?;
    if !root_metadata.is_dir() {
        return Err(save_root_collision(
            identity,
            "failed reservation path is no longer a directory",
        ));
    }
    let marker_path = identity.owner_marker();
    match fs::symlink_metadata(&marker_path) {
        Ok(metadata) => {
            reject_reparse(&marker_path, &metadata)?;
            if !metadata.is_file() {
                return Err(save_root_collision(
                    identity,
                    "failed reservation left a non-file ownership marker",
                ));
            }
            fs::remove_file(&marker_path).map_err(|source| EngineError::Io {
                path: marker_path,
                source,
            })?;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(EngineError::Io {
                path: marker_path,
                source,
            })
        }
    }
    let mut entries = fs::read_dir(&identity.save_root).map_err(|source| EngineError::Io {
        path: identity.save_root.clone(),
        source,
    })?;
    if entries
        .next()
        .transpose()
        .map_err(|source| EngineError::Io {
            path: identity.save_root.clone(),
            source,
        })?
        .is_some()
    {
        return Err(save_root_collision(
            identity,
            "failed reservation directory is no longer empty",
        ));
    }
    fs::remove_dir(&identity.save_root).map_err(|source| EngineError::Io {
        path: identity.save_root.clone(),
        source,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EngineNameAssignment {
    value: String,
    value_range: Range<usize>,
}

fn engine_name_assignments(path: &Path, content: &str) -> Result<Vec<EngineNameAssignment>> {
    let mut assignments = Vec::new();
    let mut line_start = 0_usize;
    for full_line in content.split_inclusive('\n') {
        let line = full_line
            .strip_suffix('\n')
            .unwrap_or(full_line)
            .strip_suffix('\r')
            .unwrap_or_else(|| full_line.strip_suffix('\n').unwrap_or(full_line));
        if let Some(assignment) = parse_engine_name_assignment(path, line, line_start)? {
            assignments.push(assignment);
        }
        line_start += full_line.len();
    }
    Ok(assignments)
}

fn parse_engine_name_assignment(
    path: &Path,
    line: &str,
    line_start: usize,
) -> Result<Option<EngineNameAssignment>> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("--") || !trimmed.starts_with("engine_name") {
        return Ok(None);
    }
    let key_offset = line.len() - trimmed.len();
    let after_key = &trimmed["engine_name".len()..];
    if after_key
        .chars()
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric())
    {
        return Ok(None);
    }
    let after_space = after_key.trim_start();
    if !after_space.starts_with('=') {
        return Ok(None);
    }
    let after_equals = after_space[1..].trim_start();
    let Some(quote) = after_equals
        .chars()
        .next()
        .filter(|quote| matches!(quote, '\'' | '"'))
    else {
        return Err(malformed_engine_name(path, "value must be a quoted string"));
    };
    let opening_offset = line.len() - after_equals.len();
    let value_text = &after_equals[quote.len_utf8()..];
    let mut escaped = false;
    let mut closing_relative = None;
    for (offset, character) in value_text.char_indices() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == quote {
            closing_relative = Some(offset);
            break;
        }
    }
    let Some(closing_relative) = closing_relative else {
        return Err(malformed_engine_name(
            path,
            "quoted value is not terminated",
        ));
    };
    let remainder = &value_text[closing_relative + quote.len_utf8()..];
    let remainder = remainder.trim();
    if !remainder.is_empty() && remainder != ";" && !remainder.starts_with("--") {
        return Err(malformed_engine_name(
            path,
            "unexpected content follows the quoted value",
        ));
    }
    let raw_value = &value_text[..closing_relative];
    let value = decode_lua_string(path, raw_value, quote)?;
    let value_start = line_start + opening_offset + quote.len_utf8();
    let value_end = value_start + raw_value.len();
    debug_assert_eq!(
        &line[key_offset..key_offset + "engine_name".len()],
        "engine_name"
    );
    Ok(Some(EngineNameAssignment {
        value,
        value_range: value_start..value_end,
    }))
}

fn decode_lua_string(path: &Path, raw: &str, quote: char) -> Result<String> {
    let mut decoded = String::new();
    let mut characters = raw.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            decoded.push(character);
            continue;
        }
        let Some(escaped) = characters.next() else {
            return Err(malformed_engine_name(path, "value ends with an escape"));
        };
        match escaped {
            '\\' => decoded.push('\\'),
            '"' if quote == '"' => decoded.push('"'),
            '\'' if quote == '\'' => decoded.push('\''),
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            other => {
                decoded.push('\\');
                decoded.push(other);
            }
        }
    }
    Ok(decoded)
}

fn malformed_engine_name(path: &Path, reason: impl Into<String>) -> EngineError {
    EngineError::GameIdentity {
        path: path.to_path_buf(),
        reason: format!(
            "expected exactly one valid engine_name assignment: {}",
            reason.into()
        ),
    }
}

fn patch_engine_name(game_root: &Path, identity: &ReservedSaveIdentity) -> Result<()> {
    let path = game_root.join("engine.lua");
    ensure_no_reparse_ancestors(&path)?;
    ensure_direct_file(&path)?;
    let mut content = fs::read_to_string(&path).map_err(|source| EngineError::Io {
        path: path.clone(),
        source,
    })?;
    let assignments = engine_name_assignments(&path, &content)?;
    if assignments.len() != 1 {
        return Err(EngineError::GameIdentity {
            path,
            reason: format!(
                "expected exactly one engine_name assignment, found {}",
                assignments.len()
            ),
        });
    }
    let escaped_name = identity
        .engine_name
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    content.replace_range(assignments[0].value_range.clone(), &escaped_name);

    let temporary = game_root.join(".engine.lua.chriz-part");
    let mut output = create_fresh_temporary_file(&temporary)?;
    output
        .write_all(content.as_bytes())
        .and_then(|()| output.sync_all())
        .map_err(|source| EngineError::Io {
            path: temporary.clone(),
            source,
        })?;
    drop(output);
    publish_replace(&temporary, &path)?;

    let found = read_engine_name(game_root)?;
    if found != identity.engine_name {
        return Err(EngineError::GameIdentity {
            path,
            reason: format!(
                "engine_name read-back mismatch: expected {:?}, found {:?}",
                identity.engine_name, found
            ),
        });
    }
    Ok(())
}

/// Summary of one fully verified regular-file copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageReport {
    /// Deterministic content fingerprint of the staged source tree.
    pub source_fingerprint: String,
    /// Files copied or replaced during this invocation.
    pub copied_files: u64,
    /// Existing independent files retained after size and SHA-256 verification.
    pub reused_files: u64,
    /// Total bytes represented by the complete staged tree.
    pub total_bytes: u64,
}

/// Observer seam used for progress reporting and deterministic interruption tests.
pub trait StageObserver {
    /// Called after one destination file has been durably published.
    fn file_published(&mut self, relative: &Path) -> io::Result<()>;
}

#[derive(Debug, Default)]
struct NoopObserver;

impl StageObserver for NoopObserver {
    fn file_published(&mut self, _relative: &Path) -> io::Result<()> {
        Ok(())
    }
}

/// Stages one source role using regular copies and verifies the complete result.
pub fn stage_game_copy(layout: &ManagedLayout, role: GameRole) -> Result<StageReport> {
    stage_game_copy_with_observer(layout, role, &mut NoopObserver)
}

/// Stages one source role while reporting each published file.
///
/// This is equivalent to [`stage_game_copy`]; the observer cannot alter path selection or
/// verification. If the source changes at any point, the installer-owned role directory is
/// discarded and the operation fails.
pub fn stage_game_copy_with_observer<O: StageObserver>(
    layout: &ManagedLayout,
    role: GameRole,
    observer: &mut O,
) -> Result<StageReport> {
    let (source, target, scratch) = layout.paths_for(role);
    let before = snapshot_tree(source)?;
    create_directories(target)?;
    ensure_expected_destination_entries(target, &before)?;
    create_directories(&scratch)?;
    ensure_direct_directory(&scratch)?;

    let mut copied_files = 0_u64;
    let mut reused_files = 0_u64;
    let copy_result = (|| {
        for entry in &before.entries {
            let destination = target.join(&entry.relative);
            match &entry.kind {
                SnapshotKind::Directory => {
                    create_directories(&destination)?;
                    ensure_direct_directory(&destination)?;
                }
                SnapshotKind::File { length, sha256 } => {
                    let source_file = source.join(&entry.relative);
                    ensure_direct_file(&source_file)?;
                    if hash_and_length(&source_file)? != (*length, sha256.clone()) {
                        return Err(source_changed_error(
                            source,
                            &before.identity_fingerprint,
                            None,
                        ));
                    }

                    if reusable_destination(&source_file, &destination, *length, sha256)? {
                        reused_files += 1;
                    } else {
                        copy_regular_file(
                            &source_file,
                            &destination,
                            &scratch,
                            &entry.relative,
                            *length,
                            sha256,
                        )?;
                        copied_files += 1;
                        observer.file_published(&entry.relative).map_err(|source| {
                            EngineError::Io {
                                path: destination.clone(),
                                source,
                            }
                        })?;
                    }
                }
            }
        }

        let after = snapshot_tree(source)?;
        if after != before {
            return Err(source_changed_error(
                source,
                &before.identity_fingerprint,
                Some(&after.identity_fingerprint),
            ));
        }
        verify_staged_tree(source, target, &before)?;
        Ok(())
    })();

    if let Err(error) = copy_result {
        let already_source_changed = matches!(&error, EngineError::StagingSourceChanged { .. });
        match snapshot_tree(source) {
            Ok(after) if !already_source_changed && after == before => return Err(error),
            Ok(after) => {
                discard_role_stage(layout, target, &scratch)?;
                if already_source_changed {
                    return Err(error);
                }
                return Err(source_changed_error(
                    source,
                    &before.identity_fingerprint,
                    Some(&after.identity_fingerprint),
                ));
            }
            Err(recheck_error) => {
                discard_role_stage(layout, target, &scratch)?;
                if already_source_changed {
                    return Err(error);
                }
                return Err(EngineError::StagingSourceChanged {
                    source_root: source.to_path_buf(),
                    expected: before.identity_fingerprint.clone(),
                    found: format!("source could not be re-verified: {recheck_error}"),
                });
            }
        }
    }
    remove_owned_scratch(layout, &scratch)?;

    Ok(StageReport {
        source_fingerprint: before.fingerprint,
        copied_files,
        reused_files,
        total_bytes: before.total_bytes,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TreeSnapshot {
    entries: Vec<SnapshotEntry>,
    fingerprint: String,
    identity_fingerprint: String,
    total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnapshotEntry {
    relative: PathBuf,
    kind: SnapshotKind,
    metadata_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SnapshotKind {
    Directory,
    File { length: u64, sha256: String },
}

fn snapshot_tree(root: &Path) -> Result<TreeSnapshot> {
    ensure_direct_directory(root)?;
    let mut entries = Vec::new();
    collect_snapshot_entries(root, root, &mut entries)?;
    entries.sort_by_key(|entry| path_sort_key(&entry.relative));

    let mut digest = Sha256::new();
    let mut identity_digest = Sha256::new();
    let mut total_bytes = 0_u64;
    for entry in &entries {
        let relative = portable_relative_path(&entry.relative)?;
        digest.update((relative.len() as u64).to_le_bytes());
        digest.update(relative.as_bytes());
        identity_digest.update((relative.len() as u64).to_le_bytes());
        identity_digest.update(relative.as_bytes());
        identity_digest.update((entry.metadata_identity.len() as u64).to_le_bytes());
        identity_digest.update(entry.metadata_identity.as_bytes());
        match &entry.kind {
            SnapshotKind::Directory => {
                digest.update(*b"d");
                identity_digest.update(*b"d");
            }
            SnapshotKind::File { length, sha256 } => {
                digest.update(*b"f");
                digest.update(length.to_le_bytes());
                digest.update(sha256.as_bytes());
                identity_digest.update(*b"f");
                identity_digest.update(length.to_le_bytes());
                identity_digest.update(sha256.as_bytes());
                total_bytes = total_bytes.checked_add(*length).ok_or_else(|| {
                    EngineError::StagingVerification {
                        path: root.to_path_buf(),
                        reason: "source byte count overflowed u64".to_owned(),
                    }
                })?;
            }
        }
    }

    Ok(TreeSnapshot {
        entries,
        fingerprint: hex::encode(digest.finalize()),
        identity_fingerprint: hex::encode(identity_digest.finalize()),
        total_bytes,
    })
}

fn collect_snapshot_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<SnapshotEntry>,
) -> Result<()> {
    let mut children = fs::read_dir(directory)
        .map_err(|source| EngineError::Io {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|source| EngineError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
    children.sort_by_key(|entry| path_sort_key(Path::new(&entry.file_name())));

    for child in children {
        let path = child.path();
        let metadata = direct_metadata(&path)?;
        let relative = path
            .strip_prefix(root)
            .map_err(|_| EngineError::StagingVerification {
                path: path.clone(),
                reason: "source entry escaped its canonical root".to_owned(),
            })?
            .to_path_buf();
        if metadata.is_dir() {
            entries.push(SnapshotEntry {
                relative: relative.clone(),
                kind: SnapshotKind::Directory,
                metadata_identity: source_metadata_identity(&path, &metadata)?,
            });
            collect_snapshot_entries(root, &path, entries)?;
        } else if metadata.is_file() {
            let (length, sha256) = hash_and_length(&path)?;
            entries.push(SnapshotEntry {
                relative,
                kind: SnapshotKind::File { length, sha256 },
                metadata_identity: source_metadata_identity(&path, &metadata)?,
            });
        } else {
            return Err(EngineError::UnsafeStagingEntry {
                path,
                reason: "only direct regular files and directories may be staged".to_owned(),
            });
        }
    }
    Ok(())
}

fn ensure_expected_destination_entries(target: &Path, source: &TreeSnapshot) -> Result<()> {
    ensure_direct_directory(target)?;
    let expected = source
        .entries
        .iter()
        .map(|entry| (path_sort_key(&entry.relative), &entry.kind))
        .collect::<BTreeMap<_, _>>();
    let mut actual = Vec::new();
    collect_direct_paths(target, target, &mut actual)?;
    for (relative, metadata) in actual {
        let key = path_sort_key(&relative);
        let Some(expected_kind) = expected.get(&key) else {
            return Err(EngineError::UnsafeStagingEntry {
                path: target.join(relative),
                reason: "entry is not present in the read-only source tree".to_owned(),
            });
        };
        let kind_matches = matches!(
            (expected_kind, metadata.is_file(), metadata.is_dir()),
            (SnapshotKind::File { .. }, true, false) | (SnapshotKind::Directory, false, true)
        );
        if !kind_matches {
            return Err(EngineError::UnsafeStagingEntry {
                path: target.join(relative),
                reason: "entry kind differs from the read-only source tree".to_owned(),
            });
        }
    }
    Ok(())
}

fn collect_direct_paths(
    root: &Path,
    directory: &Path,
    paths: &mut Vec<(PathBuf, fs::Metadata)>,
) -> Result<()> {
    let children = fs::read_dir(directory).map_err(|source| EngineError::Io {
        path: directory.to_path_buf(),
        source,
    })?;
    for child in children {
        let child = child.map_err(|source| EngineError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = child.path();
        let metadata = direct_metadata(&path)?;
        let relative = path
            .strip_prefix(root)
            .map_err(|_| EngineError::StagingVerification {
                path: path.clone(),
                reason: "destination entry escaped its managed root".to_owned(),
            })?
            .to_path_buf();
        paths.push((relative, metadata.clone()));
        if metadata.is_dir() {
            collect_direct_paths(root, &path, paths)?;
        }
    }
    Ok(())
}

fn reusable_destination(
    source: &Path,
    destination: &Path,
    expected_length: u64,
    expected_sha256: &str,
) -> Result<bool> {
    let metadata = match fs::symlink_metadata(destination) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(EngineError::Io {
                path: destination.to_path_buf(),
                source,
            })
        }
    };
    reject_reparse(destination, &metadata)?;
    if !metadata.is_file() {
        return Err(EngineError::UnsafeStagingEntry {
            path: destination.to_path_buf(),
            reason: "expected a direct regular destination file".to_owned(),
        });
    }
    if same_file::is_same_file(source, destination).map_err(|source| EngineError::Io {
        path: destination.to_path_buf(),
        source,
    })? {
        return Ok(false);
    }
    if hard_link_count(destination, &metadata)? != 1 {
        return Ok(false);
    }
    Ok(hash_and_length(destination)? == (expected_length, expected_sha256.to_owned()))
}

fn copy_regular_file(
    source: &Path,
    destination: &Path,
    scratch_root: &Path,
    relative: &Path,
    expected_length: u64,
    expected_sha256: &str,
) -> Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| EngineError::StagingVerification {
            path: destination.to_path_buf(),
            reason: "destination file has no parent".to_owned(),
        })?;
    create_directories(parent)?;
    ensure_direct_directory(parent)?;

    let scratch_file = scratch_file_path(scratch_root, relative)?;
    let scratch_parent = scratch_file
        .parent()
        .ok_or_else(|| EngineError::StagingVerification {
            path: scratch_file.clone(),
            reason: "scratch file has no parent".to_owned(),
        })?;
    create_directories(scratch_parent)?;
    ensure_direct_directory(scratch_parent)?;
    let mut input = File::open(source).map_err(|source_error| EngineError::Io {
        path: source.to_path_buf(),
        source: source_error,
    })?;
    let mut output = create_fresh_temporary_file(&scratch_file)?;
    io::copy(&mut input, &mut output).map_err(|source| EngineError::Io {
        path: scratch_file.clone(),
        source,
    })?;
    output.sync_all().map_err(|source| EngineError::Io {
        path: scratch_file.clone(),
        source,
    })?;
    drop(output);
    let permissions = fs::metadata(source)
        .map_err(|source_error| EngineError::Io {
            path: source.to_path_buf(),
            source: source_error,
        })?
        .permissions();
    fs::set_permissions(&scratch_file, permissions).map_err(|source| EngineError::Io {
        path: scratch_file.clone(),
        source,
    })?;

    if hash_and_length(&scratch_file)? != (expected_length, expected_sha256.to_owned()) {
        return Err(EngineError::StagingSourceChanged {
            source_root: source.to_path_buf(),
            expected: expected_sha256.to_owned(),
            found: hash_and_length(&scratch_file)?.1,
        });
    }
    publish_replace(&scratch_file, destination)?;
    Ok(())
}

fn verify_staged_tree(source: &Path, target: &Path, expected: &TreeSnapshot) -> Result<()> {
    let actual = snapshot_tree(target)?;
    if actual.fingerprint != expected.fingerprint || actual.total_bytes != expected.total_bytes {
        return Err(EngineError::StagingVerification {
            path: target.to_path_buf(),
            reason: format!(
                "tree fingerprint {} does not match source {}",
                actual.fingerprint, expected.fingerprint
            ),
        });
    }
    for entry in &expected.entries {
        if matches!(entry.kind, SnapshotKind::File { .. }) {
            let source_file = source.join(&entry.relative);
            let destination_file = target.join(&entry.relative);
            let destination_metadata = direct_metadata(&destination_file)?;
            if hard_link_count(&destination_file, &destination_metadata)? != 1 {
                return Err(EngineError::StagingVerification {
                    path: destination_file,
                    reason: "destination regular file has another hard link".to_owned(),
                });
            }
            if same_file::is_same_file(&source_file, &destination_file).map_err(|source| {
                EngineError::Io {
                    path: destination_file.clone(),
                    source,
                }
            })? {
                return Err(EngineError::StagingVerification {
                    path: destination_file,
                    reason: "destination still shares source file identity".to_owned(),
                });
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn source_metadata_identity(path: &Path, metadata: &fs::Metadata) -> Result<String> {
    use std::os::windows::fs::MetadataExt;

    Ok(format!(
        "{}:{}:{}:{}:{}",
        metadata.file_attributes(),
        metadata.creation_time(),
        metadata.last_write_time(),
        metadata.file_size(),
        file_identity_hash(path)?,
    ))
}

#[cfg(unix)]
fn source_metadata_identity(_path: &Path, metadata: &fs::Metadata) -> Result<String> {
    use std::os::unix::fs::MetadataExt;

    Ok(format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        metadata.dev(),
        metadata.ino(),
        metadata.mode(),
        metadata.size(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    ))
}

#[cfg(not(any(unix, windows)))]
fn source_metadata_identity(path: &Path, metadata: &fs::Metadata) -> Result<String> {
    Ok(format!(
        "{}:{:?}:{:?}",
        metadata.len(),
        metadata.permissions().readonly(),
        (metadata.modified().ok(), file_identity_hash(path)?),
    ))
}

fn file_identity_hash(path: &Path) -> Result<u64> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let handle = same_file::Handle::from_path(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut hasher = DefaultHasher::new();
    handle.hash(&mut hasher);
    Ok(hasher.finish())
}

#[cfg(windows)]
fn hard_link_count(path: &Path, _metadata: &fs::Metadata) -> Result<u64> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };

    let file = File::open(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    let result =
        unsafe { GetFileInformationByHandle(file.as_raw_handle().cast(), &mut information) };
    if result == 0 {
        return Err(EngineError::Io {
            path: path.to_path_buf(),
            source: io::Error::last_os_error(),
        });
    }
    Ok(u64::from(information.nNumberOfLinks))
}

#[cfg(unix)]
fn hard_link_count(_path: &Path, metadata: &fs::Metadata) -> Result<u64> {
    use std::os::unix::fs::MetadataExt;

    Ok(metadata.nlink())
}

#[cfg(not(any(unix, windows)))]
fn hard_link_count(_path: &Path, _metadata: &fs::Metadata) -> Result<u64> {
    Ok(1)
}

fn hash_and_length(path: &Path) -> Result<(u64, String)> {
    let mut file = File::open(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut length = 0_u64;
    loop {
        let read = file.read(&mut buffer).map_err(|source| EngineError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        length =
            length
                .checked_add(read as u64)
                .ok_or_else(|| EngineError::StagingVerification {
                    path: path.to_path_buf(),
                    reason: "file length overflowed u64".to_owned(),
                })?;
    }
    Ok((length, hex::encode(digest.finalize())))
}

fn inspect_existing_managed_root(root: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(EngineError::Io {
                path: root.to_path_buf(),
                source,
            })
        }
    };
    reject_reparse(root, &metadata)?;
    if !metadata.is_dir() {
        return Err(EngineError::UnsafeStagingEntry {
            path: root.to_path_buf(),
            reason: "managed root exists but is not a directory".to_owned(),
        });
    }
    for child in fs::read_dir(root).map_err(|source| EngineError::Io {
        path: root.to_path_buf(),
        source,
    })? {
        let child = child.map_err(|source| EngineError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let name = child.file_name();
        let allowed = [".chriz", "bg1", "game"]
            .iter()
            .any(|allowed| os_name_eq(&name, allowed));
        if !allowed {
            return Err(EngineError::UnsafeStagingEntry {
                path: child.path(),
                reason: "existing managed root contains an unowned top-level entry".to_owned(),
            });
        }
        let metadata = direct_metadata(&child.path())?;
        if !metadata.is_dir() {
            return Err(EngineError::UnsafeStagingEntry {
                path: child.path(),
                reason: "managed top-level entries must be direct directories".to_owned(),
            });
        }
    }
    Ok(())
}

fn canonical_direct_directory(path: &Path) -> Result<PathBuf> {
    let absolute = absolute_path(path)?;
    ensure_no_reparse_ancestors(&absolute)?;
    ensure_direct_directory(&absolute)?;
    fs::canonicalize(&absolute).map_err(|source| EngineError::Io {
        path: absolute,
        source,
    })
}

fn canonicalize_for_creation(path: &Path) -> Result<PathBuf> {
    let absolute = absolute_path(path)?;
    let mut ancestor = absolute.as_path();
    let mut suffix = Vec::<OsString>::new();
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                reject_reparse(ancestor, &metadata)?;
                if !metadata.is_dir() {
                    return Err(EngineError::UnsafeStagingEntry {
                        path: ancestor.to_path_buf(),
                        reason: "existing ancestor is not a directory".to_owned(),
                    });
                }
                ensure_no_reparse_ancestors(ancestor)?;
                let mut canonical =
                    fs::canonicalize(ancestor).map_err(|source| EngineError::Io {
                        path: ancestor.to_path_buf(),
                        source,
                    })?;
                for component in suffix.iter().rev() {
                    canonical.push(component);
                }
                return Ok(canonical);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let name = ancestor
                    .file_name()
                    .ok_or_else(|| EngineError::UnsafeStagingEntry {
                        path: absolute.clone(),
                        reason: "could not find an existing directory ancestor".to_owned(),
                    })?;
                suffix.push(name.to_os_string());
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| EngineError::UnsafeStagingEntry {
                        path: absolute.clone(),
                        reason: "could not find an existing directory ancestor".to_owned(),
                    })?;
            }
            Err(source) => {
                return Err(EngineError::Io {
                    path: ancestor.to_path_buf(),
                    source,
                })
            }
        }
    }
}

fn reject_overlapping_roots(left: &Path, right: &Path) -> Result<()> {
    if path_starts_with(left, right) || path_starts_with(right, left) {
        return Err(EngineError::UnsafeStagingRelationship {
            source_root: left.to_path_buf(),
            target: right.to_path_buf(),
            reason: "roots are equal or one contains the other".to_owned(),
        });
    }
    Ok(())
}

fn path_starts_with(path: &Path, base: &Path) -> bool {
    let path_components = normalized_components(path);
    let base_components = normalized_components(base);
    path_components.len() >= base_components.len()
        && path_components
            .iter()
            .zip(base_components.iter())
            .all(|(left, right)| component_eq(left, right))
}

fn normalized_components(path: &Path) -> Vec<OsString> {
    path.components()
        .filter_map(|component| match component {
            Component::Prefix(prefix) => Some(prefix.as_os_str().to_os_string()),
            Component::RootDir => Some(OsString::from(std::path::MAIN_SEPARATOR.to_string())),
            Component::Normal(value) => Some(value.to_os_string()),
            Component::CurDir => None,
            Component::ParentDir => Some(OsString::from("..")),
        })
        .collect()
}

fn component_eq(left: &OsString, right: &OsString) -> bool {
    #[cfg(windows)]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn os_name_eq(value: &OsString, expected: &str) -> bool {
    #[cfg(windows)]
    {
        value.to_string_lossy().eq_ignore_ascii_case(expected)
    }
    #[cfg(not(windows))]
    {
        value == expected
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Err(EngineError::UnsafeStagingEntry {
        path: path.to_path_buf(),
        reason: "staging paths must be absolute".to_owned(),
    })
}

fn create_directories(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            reject_reparse(path, &metadata)?;
            if metadata.is_dir() {
                return Ok(());
            }
            return Err(EngineError::UnsafeStagingEntry {
                path: path.to_path_buf(),
                reason: "expected a direct directory".to_owned(),
            });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(EngineError::Io {
                path: path.to_path_buf(),
                source,
            })
        }
    }
    let parent = path
        .parent()
        .ok_or_else(|| EngineError::UnsafeStagingEntry {
            path: path.to_path_buf(),
            reason: "directory has no parent".to_owned(),
        })?;
    if parent != path {
        create_directories(parent)?;
    }
    ensure_direct_directory(parent)?;
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => ensure_direct_directory(path),
        Err(source) => Err(EngineError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn ensure_direct_directory(path: &Path) -> Result<()> {
    let metadata = direct_metadata(path)?;
    if !metadata.is_dir() {
        return Err(EngineError::UnsafeStagingEntry {
            path: path.to_path_buf(),
            reason: "expected a direct directory".to_owned(),
        });
    }
    Ok(())
}

fn ensure_direct_file(path: &Path) -> Result<()> {
    let metadata = direct_metadata(path)?;
    if !metadata.is_file() {
        return Err(EngineError::UnsafeStagingEntry {
            path: path.to_path_buf(),
            reason: "expected a direct regular file".to_owned(),
        });
    }
    Ok(())
}

fn create_fresh_temporary_file(path: &Path) -> Result<File> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            reject_reparse(path, &metadata)?;
            if !metadata.is_file() {
                return Err(EngineError::UnsafeStagingEntry {
                    path: path.to_path_buf(),
                    reason: "temporary entry is not a direct regular file".to_owned(),
                });
            }
            fs::remove_file(path).map_err(|source| EngineError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(EngineError::Io {
                path: path.to_path_buf(),
                source,
            })
        }
    }
    let parent = path
        .parent()
        .ok_or_else(|| EngineError::UnsafeStagingEntry {
            path: path.to_path_buf(),
            reason: "temporary file has no parent".to_owned(),
        })?;
    ensure_no_reparse_ancestors(parent)?;
    ensure_direct_directory(parent)?;
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|source| EngineError::Io {
            path: path.to_path_buf(),
            source,
        })
}

fn direct_metadata(path: &Path) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    reject_reparse(path, &metadata)?;
    Ok(metadata)
}

fn ensure_no_reparse_ancestors(path: &Path) -> Result<()> {
    let mut current = Some(path);
    while let Some(candidate) = current {
        match fs::symlink_metadata(candidate) {
            Ok(metadata) => reject_reparse(candidate, &metadata)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(EngineError::Io {
                    path: candidate.to_path_buf(),
                    source,
                })
            }
        }
        current = candidate.parent();
    }
    Ok(())
}

fn reject_reparse(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    if metadata.file_type().is_symlink() || has_windows_reparse_attribute(metadata) {
        return Err(EngineError::UnsafeStagingEntry {
            path: path.to_path_buf(),
            reason: "symbolic links, junctions, and other reparse points are forbidden".to_owned(),
        });
    }
    Ok(())
}

#[cfg(windows)]
fn has_windows_reparse_attribute(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn has_windows_reparse_attribute(_metadata: &fs::Metadata) -> bool {
    false
}

fn portable_relative_path(path: &Path) -> Result<String> {
    let text = path
        .to_str()
        .ok_or_else(|| EngineError::UnsafeStagingEntry {
            path: path.to_path_buf(),
            reason: "staging paths must be representable as Unicode".to_owned(),
        })?;
    Ok(text.replace('\\', "/"))
}

fn path_sort_key(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    {
        value.to_ascii_lowercase()
    }
    #[cfg(not(windows))]
    {
        value
    }
}

fn scratch_file_path(scratch_root: &Path, relative: &Path) -> Result<PathBuf> {
    let file_name = relative
        .file_name()
        .ok_or_else(|| EngineError::StagingVerification {
            path: relative.to_path_buf(),
            reason: "source file has no file name".to_owned(),
        })?;
    let mut scratch_name = file_name.to_os_string();
    scratch_name.push(".chriz-part");
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    Ok(scratch_root.join(parent).join(scratch_name))
}

#[cfg(windows)]
fn publish_replace(source: &Path, destination: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source_wide = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination_wide = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let result = unsafe {
        MoveFileExW(
            source_wide.as_ptr(),
            destination_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        return Err(EngineError::Io {
            path: destination.to_path_buf(),
            source: io::Error::last_os_error(),
        });
    }
    Ok(())
}

#[cfg(not(windows))]
fn publish_replace(source: &Path, destination: &Path) -> Result<()> {
    fs::rename(source, destination).map_err(|source| EngineError::Io {
        path: destination.to_path_buf(),
        source,
    })
}

fn source_changed_error(source: &Path, expected: &str, found: Option<&str>) -> EngineError {
    EngineError::StagingSourceChanged {
        source_root: source.to_path_buf(),
        expected: expected.to_owned(),
        found: found
            .unwrap_or("source file no longer matches its initial hash")
            .to_owned(),
    }
}

fn discard_role_stage(layout: &ManagedLayout, target: &Path, scratch: &Path) -> Result<()> {
    let expected_target = target == layout.bg1_root || target == layout.game_root;
    if !expected_target || target.parent() != Some(layout.root.as_path()) {
        return Err(EngineError::StagingVerification {
            path: target.to_path_buf(),
            reason: "refused to discard a path outside the fixed managed layout".to_owned(),
        });
    }
    remove_direct_directory_if_present(target)?;
    remove_owned_scratch(layout, scratch)
}

fn remove_owned_scratch(layout: &ManagedLayout, scratch: &Path) -> Result<()> {
    let expected_bg1 = layout.state_root.join("staging/bg1");
    let expected_game = layout.state_root.join("staging/game");
    if scratch != expected_bg1 && scratch != expected_game {
        return Err(EngineError::StagingVerification {
            path: scratch.to_path_buf(),
            reason: "refused to remove scratch outside the fixed managed layout".to_owned(),
        });
    }
    remove_direct_directory_if_present(scratch)
}

fn remove_direct_directory_if_present(path: &Path) -> Result<()> {
    ensure_no_reparse_ancestors(path)?;
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(EngineError::Io {
                path: path.to_path_buf(),
                source,
            })
        }
    };
    reject_reparse(path, &metadata)?;
    if !metadata.is_dir() {
        return Err(EngineError::UnsafeStagingEntry {
            path: path.to_path_buf(),
            reason: "installer-owned cleanup target is not a direct directory".to_owned(),
        });
    }
    let mut entries = Vec::new();
    collect_direct_paths(path, path, &mut entries)?;
    ensure_no_reparse_ancestors(path)?;
    fs::remove_dir_all(path).map_err(|source| EngineError::Io {
        path: path.to_path_buf(),
        source,
    })
}
