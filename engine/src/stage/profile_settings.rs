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
    /// A player preference: any recognized value is kept; only a missing key is seeded.
    Preference,
    /// Installation state that the component's own installer records. It is set to the
    /// installed state, as that installer would have done in its own profile.
    InstallState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Setting {
    label: &'static str,
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

const INTERVAL_SAVES: &str = "interval-save settings";
const EET_STATE: &str = "EET installation state";
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
                label: INTERVAL_SAVES,
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
            label: EET_STATE,
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
// settings are retained byte-for-byte. Unsupported syntax for an owned key is
// reported instead of guessing or overriding a user's preference.
fn quoted<'a>(text: &mut &'a str) -> Option<&'a str> {
    *text = text.trim_start();
    let quote = text.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }
    let rest = &text[1..];
    let end = rest.find(quote)?;
    let value = &rest[..end];
    if value.contains('\\') {
        return None;
    }
    *text = &rest[end + 1..];
    Some(value)
}

fn consume(text: &mut &str, token: char) -> Option<()> {
    *text = text.trim_start().strip_prefix(token)?;
    Some(())
}

fn assignment(line: &str) -> Option<(&str, &str, &str)> {
    let mut text = line.trim_start().strip_prefix("SetPrivateProfileString")?;
    consume(&mut text, '(')?;
    let section = quoted(&mut text)?;
    consume(&mut text, ',')?;
    let key = quoted(&mut text)?;
    consume(&mut text, ',')?;
    let value = quoted(&mut text)?;
    consume(&mut text, ')')?;
    let tail = text.trim().strip_prefix(';').unwrap_or(text.trim()).trim();
    (tail.is_empty() || tail.starts_with("--")).then_some((section, key, value))
}

/// What must change so the profile holds every required setting.
#[derive(Debug, Default)]
struct Changes {
    /// Whole assignment lines (byte ranges without their line ending) to rewrite.
    replace: Vec<(Range<usize>, Setting)>,
    append: Vec<Setting>,
}

impl Changes {
    fn first(&self) -> Option<&Setting> {
        self.replace
            .first()
            .map(|(_, setting)| setting)
            .or(self.append.first())
    }
}

fn changes(path: &Path, bytes: &[u8], settings: &[Setting]) -> Result<Changes> {
    let label = settings.first().map_or(PROFILE, |setting| setting.label);
    let content = std::str::from_utf8(bytes)
        .map_err(|_| settings_error(path, label, "Baldur.lua is not UTF-8; left unchanged"))?;
    let mut assignments: Vec<Vec<(Range<usize>, &str)>> = vec![Vec::new(); settings.len()];
    let mut offset = if content.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let mut block_comment = false;
    for full_line in content[offset..].split_inclusive('\n') {
        let text = full_line.strip_suffix('\n').unwrap_or(full_line);
        let text = text.strip_suffix('\r').unwrap_or(text);
        let range = offset..offset + text.len();
        offset += full_line.len();
        let line = text.trim();
        // Engine-written Baldur.lua has no block comments. Don't infer settings
        // from a manually commented-out example or an arbitrary Lua program.
        if line.starts_with("--[") {
            block_comment = true;
        }
        for (index, setting) in settings.iter().enumerate() {
            if !line.contains(setting.key) {
                continue;
            }
            if block_comment {
                return Err(settings_error(
                    path,
                    setting.label,
                    format!("cannot interpret {} in a block comment", setting.key),
                ));
            }
            if line.starts_with("--") {
                continue;
            }
            match assignment(line) {
                Some((section, key, value)) if section == setting.section && key == setting.key => {
                    if setting.ownership == Ownership::Preference && !matches!(value, "0" | "1") {
                        return Err(settings_error(
                            path,
                            setting.label,
                            format!("cannot interpret {}; left unchanged", setting.key),
                        ));
                    }
                    assignments[index].push((range.clone(), value));
                }
                Some(_) => {}
                None => {
                    return Err(settings_error(
                        path,
                        setting.label,
                        format!("cannot interpret {}; left unchanged", setting.key),
                    ))
                }
            }
        }
        if block_comment && line.contains("]]") {
            block_comment = false;
        }
    }

    let mut changes = Changes::default();
    for (setting, found) in settings.iter().zip(assignments) {
        match (setting.ownership, found.as_slice()) {
            (_, []) => changes.append.push(*setting),
            (Ownership::Preference, _) => {}
            (Ownership::InstallState, found)
                if found.iter().all(|(_, value)| *value == setting.value) => {}
            (Ownership::InstallState, [(range, _)]) => {
                changes.replace.push((range.clone(), *setting));
            }
            (Ownership::InstallState, _) => {
                return Err(settings_error(
                    path,
                    setting.label,
                    format!(
                        "{} is assigned more than once with different values; left unchanged",
                        setting.key
                    ),
                ))
            }
        }
    }
    Ok(changes)
}

fn apply(content: &str, changes: &Changes) -> Vec<u8> {
    let newline = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut replace = changes.replace.clone();
    replace.sort_by_key(|(range, _)| range.start);
    let mut after = String::with_capacity(content.len() + 128);
    let mut cursor = 0;
    for (range, setting) in replace {
        after.push_str(&content[cursor..range.start]);
        after.push_str(&setting.assignment());
        cursor = range.end;
    }
    after.push_str(&content[cursor..]);
    if !changes.append.is_empty() {
        if !after.is_empty() && !after.ends_with('\n') {
            after.push_str(newline);
        }
        for setting in &changes.append {
            after.push_str(&setting.assignment());
            after.push_str(newline);
        }
    }
    after.into_bytes()
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
    if changes.first().is_none() {
        return Ok(false);
    }
    // `changes` succeeded, so the content is UTF-8.
    let after = apply(std::str::from_utf8(bytes).unwrap_or_default(), &changes);
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
    let settings = required_settings(game_root)?;
    if settings.is_empty() {
        return Ok(());
    }
    let path = save_root.join("Baldur.lua");
    let bytes = read_optional_regular(&path)?.unwrap_or_default();
    if let Some(setting) = changes(&path, &bytes, &settings)?.first() {
        return Err(settings_error(
            &path,
            setting.label,
            "required settings are missing or outdated in the isolated game profile",
        ));
    }
    Ok(())
}

pub(super) fn ensure_profile_settings(
    layout: &ManagedLayout,
    identity: &ReservedSaveIdentity,
) -> Result<()> {
    verify_reserved_save_identity(layout, identity)?;
    ensure_settings(layout.game_root(), &identity.save_root)?;
    verify_profile_settings(layout, identity)
}

/// Check the final isolated profile before declaring an installation successful.
/// An explicit user preference such as an interval-save value of 0 is valid; only
/// missing settings and outdated installation state are defects.
pub fn verify_profile_settings(
    layout: &ManagedLayout,
    identity: &ReservedSaveIdentity,
) -> Result<()> {
    verify_reserved_save_identity(layout, identity)?;
    verify_settings(layout.game_root(), &identity.save_root)
}

/// Brings a completed installation's isolated profile up to date with the settings
/// its installed components need, for installations made before a setting was
/// seeded. Only a save folder that still carries this installation's ownership
/// marker is changed. Returns whether the profile changed.
pub fn repair_installed_profile_settings(
    managed_root: &Path,
    install_id: &str,
    engine_name: &str,
    save_root: &Path,
) -> Result<bool> {
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
    let changed = ensure_settings(&game_root, save_root)?;
    verify_settings(&game_root, save_root)?;
    Ok(changed)
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
