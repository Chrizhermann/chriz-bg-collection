//! Seed only known, installed-component settings in the final isolated profile.
//!
//! EET changes engine_name while installing. CDTweaks and EET itself consequently
//! write their settings into EET's shared profile, which CEBG intentionally does not
//! use. Never copy that profile: it can contain another playthrough's preferences.
//! Instead, the few settings an installed component relies on are reproduced here.

use super::*;
use crate::fs_retry::CheckedRenameError;
use crate::weidu::log::parse_active_entries;

/// How an existing value of a seeded key is treated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ownership {
    /// A player preference: any existing value is kept; only a missing key is seeded.
    Preference,
    /// Installation state that the component's own installer records. It is set to the
    /// installed state, as that installer would have done in its own profile.
    InstallState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Setting {
    section: &'static str,
    key: &'static str,
    value: &'static str,
    ownership: Ownership,
}

impl Setting {
    fn assignment(&self) -> String {
        format!(
            "SetPrivateProfileString('{}','{}','{}')",
            self.section, self.key, self.value
        )
    }
}

const PROFILE: &str = "profile settings";

fn settings_error(path: &Path, label: &str, reason: impl Into<String>) -> EngineError {
    EngineError::GameIdentity {
        path: path.to_owned(),
        reason: format!("{label}: {}", reason.into()),
    }
}

/// The settings that the installed components in `game_root` rely on.
fn required_settings(game_root: &Path) -> Result<Vec<Setting>> {
    let path = game_root.join("WeiDU.log");
    let Some(bytes) = read_optional_regular(&path)? else {
        return Ok(Vec::new());
    };
    // Annotations can use the mod's legacy encoding; active component identities
    // are ASCII and must parse even when the description is not UTF-8.
    let entries = parse_active_entries(&String::from_utf8_lossy(&bytes))
        .map_err(|error| settings_error(&path, PROFILE, error.to_string()))?;
    let installed = |tp2: &str, components: std::ops::RangeInclusive<u32>| {
        entries
            .iter()
            .any(|entry| entry.tp2_key == tp2 && components.contains(&entry.component))
    };
    let mut settings = Vec::new();
    if installed("cdtweaks/setup-cdtweaks.tp2", 3350..=3357) {
        for (key, value) in [("IntervalSaveEnabled", "1"), ("IntervalSaveCombat", "0")] {
            settings.push(Setting {
                section: "Script",
                key,
                value,
                ownership: Ownership::Preference,
            });
        }
    }
    // EET core records 1 and EET_end (any of its components) raises it to 2. Lefreut's
    // interface offers the trilogy campaigns only with this state or EET's own engine
    // name, and CEBG renames the engine for its isolated profile.
    let eet_state = if installed("eet_end/eet_end.tp2", 0..=u32::MAX) {
        Some("2")
    } else if installed("eet/eet.tp2", 0..=0) {
        Some("1")
    } else {
        None
    };
    if let Some(value) = eet_state {
        settings.push(Setting {
            section: "Program Options",
            key: "EET Installation State",
            value,
            ownership: Ownership::InstallState,
        });
    }
    Ok(settings)
}

fn read_optional_regular(path: &Path) -> Result<Option<Vec<u8>>> {
    ensure_no_reparse_ancestors(path)?;
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(EngineError::Io {
                path: path.into(),
                source,
            })
        }
    };
    ensure_direct_file(path)?;
    if hard_link_count(path, &metadata)? != 1 {
        return Err(settings_error(path, PROFILE, "file has another hard link"));
    }
    fs::read(path).map(Some).map_err(|source| EngineError::Io {
        path: path.into(),
        source,
    })
}

// Recognize the engine's data-file syntax, without ever executing Lua. Other
// settings are retained byte-for-byte. Only the first two arguments identify ownership;
// an old value is never an acceptance condition.
fn quoted<'a>(text: &mut &'a [u8]) -> Option<&'a [u8]> {
    *text = text.trim_ascii_start();
    let quote = *text.first()?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let rest = &text[1..];
    let end = rest.iter().position(|byte| *byte == quote)?;
    let value = &rest[..end];
    if value.contains(&b'\\') {
        return None;
    }
    *text = &rest[end + 1..];
    Some(value)
}

fn consume(text: &mut &[u8], token: u8) -> Option<()> {
    *text = text.trim_ascii_start().strip_prefix(&[token])?;
    Some(())
}

// Read literal keys across line breaks without decoding unrelated profile bytes.
fn assignment_key(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let mut text = line
        .trim_ascii_start()
        .strip_prefix(b"SetPrivateProfileString")?;
    consume(&mut text, b'(')?;
    let section = quoted(&mut text)?;
    consume(&mut text, b',')?;
    let key = quoted(&mut text)?;
    Some((section, key))
}

/// Locate the end of a complete call without interpreting its old value or executing Lua.
fn is_single_assignment(line: &str) -> bool {
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0usize;
    let bytes = line.as_bytes();
    for (index, byte) in bytes.iter().copied().enumerate() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            continue;
        }
        if byte == b'-' && bytes.get(index + 1) == Some(&b'-') {
            return false;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => depth += 1,
            b')' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    let tail = line[index + 1..].trim().trim_start_matches(';').trim();
                    return tail.is_empty() || tail.starts_with("--");
                }
            }
            _ => {}
        }
    }
    false
}

/// What must change so the profile holds every required setting.
#[derive(Debug, Default)]
struct Changes {
    /// Whole assignment lines (byte ranges without their line ending) to rewrite.
    replace: Vec<(Range<usize>, Setting)>,
    append: Vec<Setting>,
    incomplete_comment: bool,
}

impl Changes {
    fn first(&self) -> Option<&Setting> {
        self.replace
            .first()
            .map(|(_, setting)| setting)
            .or(self.append.first())
    }
}

fn changes(_path: &Path, bytes: &[u8], settings: &[Setting]) -> Result<Changes> {
    let mut changes = Changes::default();
    let mut found = vec![false; settings.len()];
    let mut offset = if bytes.starts_with(b"\xef\xbb\xbf") {
        3
    } else {
        0
    };
    let mut block_end: Option<String> = None;
    for full_line in bytes[offset..].split_inclusive(|byte| *byte == b'\n') {
        let text = full_line.strip_suffix(b"\n").unwrap_or(full_line);
        let text = text.strip_suffix(b"\r").unwrap_or(text);
        let range = offset..offset + text.len();
        offset += full_line.len();
        let line = String::from_utf8_lossy(text);
        let line = line.trim();
        if let Some(tail) = line.strip_prefix("--[") {
            let equals = tail.bytes().take_while(|byte| *byte == b'=').count();
            if tail.as_bytes().get(equals) == Some(&b'[') {
                block_end = Some(format!("]{}]", "=".repeat(equals)));
            }
        }
        if let Some(end) = &block_end {
            if line.contains(end) {
                block_end = None;
            }
            continue;
        }
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        let assignment = assignment_key(&bytes[range.start..]);
        // Unknown executable text or a multiline call may set the owned state later.
        // Complete assignments to other literal keys preserve it and keep normal bytes.
        if assignment.is_none() || !is_single_assignment(line) {
            for (index, setting) in settings.iter().enumerate() {
                if setting.ownership == Ownership::InstallState {
                    found[index] = false;
                }
            }
        }
        if let Some((section, key)) = assignment {
            for (index, setting) in settings.iter().enumerate() {
                if section == setting.section.as_bytes() && key == setting.key.as_bytes() {
                    if setting.ownership == Ownership::Preference {
                        found[index] = true;
                    } else {
                        // A complete assignment occupies this line. For a multiline call or
                        // extra Lua statements, leave its bytes and append our final assignment.
                        if is_single_assignment(line) {
                            found[index] = true;
                            changes.replace.push((range.clone(), *setting));
                        }
                    }
                }
            }
        }
    }
    changes.incomplete_comment = block_end.is_some();
    for (setting, found) in settings.iter().zip(found) {
        if !found {
            changes.append.push(*setting);
        }
    }
    Ok(changes)
}

fn apply(content: &[u8], changes: &Changes) -> Vec<u8> {
    let newline: &[u8] = if content.windows(2).any(|bytes| bytes == b"\r\n") {
        b"\r\n"
    } else {
        b"\n"
    };
    let mut replace = changes.replace.clone();
    replace.sort_by_key(|(range, _)| range.start);
    let mut after = Vec::with_capacity(content.len() + 128);
    let mut cursor = 0;
    for (range, setting) in replace {
        after.extend_from_slice(&content[cursor..range.start]);
        after.extend_from_slice(setting.assignment().as_bytes());
        cursor = range.end;
    }
    after.extend_from_slice(&content[cursor..]);
    if !changes.append.is_empty() {
        if !after.is_empty() && !after.ends_with(b"\n") {
            after.extend_from_slice(newline);
        }
        for setting in &changes.append {
            after.extend_from_slice(setting.assignment().as_bytes());
            after.extend_from_slice(newline);
        }
    }
    after
}

/// Writes the settings that `game_root`'s installed components need into the profile
/// in `save_root`. Returns whether the profile changed.
fn ensure_settings(game_root: &Path, save_root: &Path) -> Result<bool> {
    let settings = required_settings(game_root)?;
    if settings.is_empty() {
        return Ok(false);
    }
    let path = save_root.join("Baldur.lua");
    let before = read_optional_regular(&path)?;
    let bytes = before.as_deref().unwrap_or_default();
    let changes = changes(&path, bytes, &settings)?;
    if changes.incomplete_comment {
        eprintln!(
            "Profile settings: kept {} because its Lua block comment is unfinished.",
            path.display()
        );
        return Ok(false);
    }
    if changes.first().is_none() {
        return Ok(false);
    }
    let after = apply(bytes, &changes);
    if after == bytes {
        return Ok(false);
    }
    let mut temporary =
        tempfile::NamedTempFile::new_in(save_root).map_err(|source| EngineError::Io {
            path: path.clone(),
            source,
        })?;
    temporary
        .write_all(&after)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|source| EngineError::Io {
            path: path.clone(),
            source,
        })?;
    // Don't clobber preferences written after inspection (e.g. a concurrent editor). The check
    // runs again before every attempt while a scanner briefly holds the profile.
    let unchanged = || {
        if read_optional_regular(&path)? != before {
            return Err(settings_error(
                &path,
                PROFILE,
                "file changed while preparing settings; retry when the game is closed",
            ));
        }
        Ok(())
    };
    match fs_retry::persist_checked(temporary, &path, unchanged) {
        Ok(()) => {}
        Err(CheckedRenameError::Check(error)) => return Err(error),
        Err(CheckedRenameError::Rename(source)) => {
            return Err(EngineError::Io {
                path: path.clone(),
                source,
            })
        }
    }
    if read_optional_regular(&path)?.as_deref() != Some(after.as_slice()) {
        return Err(settings_error(
            &path,
            PROFILE,
            "written settings did not match read-back",
        ));
    }
    Ok(true)
}

fn verify_settings(game_root: &Path, save_root: &Path) -> Result<()> {
    ensure_settings(game_root, save_root).map(|_| ())
}

pub(super) fn ensure_profile_settings(
    layout: &ManagedLayout,
    identity: &ReservedSaveIdentity,
) -> Result<()> {
    verify_reserved_save_identity(layout, identity)?;
    if layout.is_standalone() {
        // The seeded settings are Baldur's Gate profile values written to `Baldur.lua`;
        // a standalone game keeps its own profile file and needs none of them.
        return Ok(());
    }
    ensure_settings(layout.game_root(), &identity.save_root).map(|_| ())
}

/// Ensure the final isolated profile has the owned install state and missing defaults.
/// Existing player preferences remain theirs, regardless of their values.
pub fn verify_profile_settings(
    layout: &ManagedLayout,
    identity: &ReservedSaveIdentity,
) -> Result<()> {
    verify_reserved_save_identity(layout, identity)?;
    if layout.is_standalone() {
        return Ok(());
    }
    verify_settings(layout.game_root(), &identity.save_root)
}

/// Brings a completed installation's isolated profile up to date with the settings
/// its installed components need, for installations made before a setting was
/// seeded. Only a save folder that still carries this installation's ownership
/// marker is changed. Returns whether the profile changed.
///
/// This applies only to EET installations: the settings are Baldur's Gate `Baldur.lua`
/// values, so callers must not pass a standalone (for example Icewind Dale) installation.
pub fn repair_installed_profile_settings(
    managed_root: &Path,
    install_id: &str,
    engine_name: &str,
    save_root: &Path,
) -> Result<bool> {
    let game_root = verify_installed_save_root(managed_root, install_id, engine_name, save_root)?;
    ensure_settings(&game_root, save_root)
}

/// Prepares a completed installation's isolated profile for the player to open.
/// The game reads custom portraits from the profile's `portraits` folder but never
/// creates it, so players cannot find where to put them; it is created here. An
/// existing entry of that name is the player's and is left as it is. Only a save
/// folder that still carries this installation's ownership marker is changed.
/// Returns the profile folder.
pub fn prepare_installed_profile_folder(
    managed_root: &Path,
    install_id: &str,
    engine_name: &str,
    save_root: &Path,
) -> Result<PathBuf> {
    verify_installed_save_root(managed_root, install_id, engine_name, save_root)?;
    let portraits = save_root.join("portraits");
    match fs::create_dir(&portraits) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(source) => return Err(io_failure("create portraits folder", &portraits, source)),
    }
    Ok(save_root.to_path_buf())
}

/// Proves that `save_root` is still the profile the completed installation in
/// `managed_root` reserved, and that its game still uses it. Returns the game folder.
fn verify_installed_save_root(
    managed_root: &Path,
    install_id: &str,
    engine_name: &str,
    save_root: &Path,
) -> Result<PathBuf> {
    let root = canonical_direct_directory(managed_root)
        .map_err(|error| folder_context("check installation folder", managed_root, error))?;
    let game_root = root.join("game");
    if read_engine_name(&game_root)? != engine_name {
        return Err(EngineError::GameIdentity {
            path: game_root.join("engine.lua"),
            reason: "the game's engine_name differs from its installation record".to_owned(),
        });
    }
    let identity = SaveIdentity {
        engine_name: engine_name.to_owned(),
        save_root: save_root.to_path_buf(),
        marker: SaveOwner {
            schema: SAVE_OWNER_SCHEMA,
            install_id: install_id.to_owned(),
            managed_root: identity_path(&root)?,
            engine_name: engine_name.to_owned(),
        },
    };
    if save_root.file_name().and_then(|name| name.to_str()) != Some(engine_name) {
        return Err(save_root_collision(
            &identity,
            "save folder name differs from the game's engine_name",
        ));
    }
    let metadata = fs::symlink_metadata(save_root)
        .map_err(|source| io_failure("check save folder", save_root, source))?;
    reject_reparse(save_root, &metadata)?;
    if !metadata.is_dir() {
        return Err(save_root_collision(
            &identity,
            "save folder is not a directory",
        ));
    }
    if read_save_owner(&identity)? != identity.marker {
        return Err(save_root_collision(
            &identity,
            "ownership marker belongs to a different managed installation",
        ));
    }
    Ok(game_root)
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use crate::fs_retry::test_hook::{
        hold_until_first_wait, hold_without_delete_sharing, with_wait_hook,
    };

    const SEEDED: &str = "SetPrivateProfileString('Script','IntervalSaveEnabled','1')\n\
                          SetPrivateProfileString('Script','IntervalSaveCombat','0')\n";

    /// A game with CDTweaks interval saves installed and a save folder whose profile has `profile`.
    fn interval_fixture(profile: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let game = temp.path().join("game");
        let save = temp.path().join("save");
        fs::create_dir(&game).unwrap();
        fs::create_dir(&save).unwrap();
        fs::write(
            game.join("WeiDU.log"),
            "~CDTWEAKS/SETUP-CDTWEAKS.TP2~ #0 #3354 // Interval saves\n",
        )
        .unwrap();
        fs::write(save.join("Baldur.lua"), profile).unwrap();
        (temp, game, save)
    }

    fn names(folder: &Path) -> Vec<String> {
        let mut names = fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[test]
    fn seeding_outlasts_a_brief_hold_on_the_profile() {
        let (_temp, game, save) = interval_fixture("");
        let profile = save.join("Baldur.lua");

        let (result, waits) = hold_until_first_wait(&profile, || ensure_settings(&game, &save));

        assert!(result.unwrap());
        assert!(waits >= 1, "the held profile must fail the first attempt");
        assert_eq!(fs::read_to_string(&profile).unwrap(), SEEDED);
        assert_eq!(names(&save), ["Baldur.lua"]);
    }

    #[test]
    fn a_profile_changed_during_the_retries_is_kept_and_the_temporary_removed() {
        use std::cell::Cell;
        use std::rc::Rc;

        let (_temp, game, save) = interval_fixture("");
        let profile = save.join("Baldur.lua");
        let edited = "SetPrivateProfileString('Script','IntervalSaveEnabled','0')\n";
        // The game writes its own settings while the first attempt is blocked, then lets go.
        let mut holder = Some(hold_without_delete_sharing(&profile));
        let waits = Rc::new(Cell::new(0));
        let counted = Rc::clone(&waits);
        let written = profile.clone();

        let result = with_wait_hook(
            move |_| {
                if counted.replace(counted.get() + 1) == 0 {
                    fs::write(&written, edited).unwrap();
                    drop(holder.take());
                }
            },
            || ensure_settings(&game, &save),
        );

        let error = result.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("file changed while preparing settings"),
            "{error}"
        );
        // The change is seen right before the second attempt, which then never runs.
        assert_eq!(waits.get(), 1);
        assert_eq!(fs::read_to_string(&profile).unwrap(), edited);
        assert_eq!(names(&save), ["Baldur.lua"]);
    }
}

#[cfg(test)]
mod overwrite_tests {
    use super::*;

    #[test]
    fn installed_state_is_written_for_changed_values_duplicates_and_other_text() {
        let state = Setting {
            section: "Program Options",
            key: "EET Installation State",
            value: "2",
            ownership: Ownership::InstallState,
        };
        let decided = format!("{}\n", state.assignment());
        for input in [
            "SetPrivateProfileString('Program Options','EET Installation State','0')\n",
            "SetPrivateProfileString('Program Options','EET Installation State',42)\n",
            "SetPrivateProfileString('Program Options','EET Installation State','anderer Text')\n",
        ] {
            let changed = apply(
                input.as_bytes(),
                &changes(Path::new("Baldur.lua"), input.as_bytes(), &[state]).unwrap(),
            );
            assert_eq!(changed, decided.as_bytes());
            assert_eq!(
                apply(
                    &changed,
                    &changes(Path::new("Baldur.lua"), &changed, &[state]).unwrap()
                ),
                changed
            );
        }
        let normal = format!("{decided}SetPrivateProfileString('Graphics','Width','1920')\n");
        assert_eq!(
            apply(
                normal.as_bytes(),
                &changes(Path::new("Baldur.lua"), normal.as_bytes(), &[state]).unwrap()
            ),
            normal.as_bytes()
        );
        let duplicated = format!(
            "{}{}",
            decided, "SetPrivateProfileString('Program Options','EET Installation State',-1)\n"
        );
        assert_eq!(
            apply(
                duplicated.as_bytes(),
                &changes(Path::new("Baldur.lua"), duplicated.as_bytes(), &[state]).unwrap()
            ),
            decided.repeat(2).as_bytes()
        );
        let other_bytes = b"-- commentaire: \xe9\r\n";
        assert_eq!(
            apply(
                other_bytes,
                &changes(Path::new("Baldur.lua"), other_bytes, &[state]).unwrap()
            ),
            [
                other_bytes.as_slice(),
                state.assignment().as_bytes(),
                b"\r\n"
            ]
            .concat()
        );
    }

    #[test]
    fn multiline_preference_keys_keep_the_players_value_and_unrelated_bytes() {
        let preference = Setting {
            section: "Script",
            key: "IntervalSaveEnabled",
            value: "1",
            ownership: Ownership::Preference,
        };
        let input = b"SetPrivateProfileString(\n  'Script',\n  'IntervalSaveEnabled',\n  'my choice')\n-- commentaire: \xe9\n";
        let output = apply(
            input,
            &changes(Path::new("Baldur.lua"), input, &[preference]).unwrap(),
        );
        assert_eq!(output, input);
    }

    #[test]
    fn multiline_assignments_get_a_final_owned_assignment_and_preferences_stay() {
        let state = Setting {
            section: "Program Options",
            key: "EET Installation State",
            value: "2",
            ownership: Ownership::InstallState,
        };
        let preference = Setting {
            section: "Script",
            key: "IntervalSaveEnabled",
            value: "1",
            ownership: Ownership::Preference,
        };
        let input = b"SetPrivateProfileString('Program Options','EET Installation State','2')\nSetPrivateProfileString('Program Options','EET Installation State',\n 42)\nSetPrivateProfileString('Script','IntervalSaveEnabled','my choice')\n";
        let output = apply(
            input,
            &changes(Path::new("Baldur.lua"), input, &[state, preference]).unwrap(),
        );
        assert_eq!(
            output,
            [input.as_slice(), state.assignment().as_bytes(), b"\n"].concat()
        );
        assert_eq!(
            apply(
                &output,
                &changes(Path::new("Baldur.lua"), &output, &[state, preference]).unwrap()
            ),
            output
        );
    }
}
