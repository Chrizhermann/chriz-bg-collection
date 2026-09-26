//! Seed only known, installed-component defaults in the final isolated profile.
//!
//! EET changes engine_name while installing. CDTweaks can consequently write its
//! enable flags into EET's shared profile, which CEBG intentionally does not use.
//! Never copy that profile: it can contain another playthrough's preferences.

use super::*;
use crate::weidu::log::parse_active_entries;

const DEFAULTS: [(&str, &str); 2] = [("IntervalSaveEnabled", "1"), ("IntervalSaveCombat", "0")];

fn settings_error(path: &Path, reason: impl Into<String>) -> EngineError {
    EngineError::GameIdentity {
        path: path.to_owned(),
        reason: format!("interval-save settings: {}", reason.into()),
    }
}

fn interval_saves_installed(layout: &ManagedLayout) -> Result<bool> {
    let path = layout.game_root().join("WeiDU.log");
    let Some(bytes) = read_optional_regular(&path)? else {
        return Ok(false);
    };
    // Annotations can use the mod's legacy encoding; active component identities
    // are ASCII and must parse even when the description is not UTF-8.
    let entries = parse_active_entries(&String::from_utf8_lossy(&bytes))
        .map_err(|error| settings_error(&path, error.to_string()))?;
    Ok(entries.iter().any(|entry| {
        entry.tp2_key == "cdtweaks/setup-cdtweaks.tp2" && (3350..=3357).contains(&entry.component)
    }))
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
        return Err(settings_error(path, "file has another hard link"));
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

fn missing_defaults(path: &Path, bytes: &[u8]) -> Result<Vec<(&'static str, &'static str)>> {
    let content = std::str::from_utf8(bytes)
        .map_err(|_| settings_error(path, "Baldur.lua is not UTF-8; left unchanged"))?;
    let mut present = [false; 2];
    let mut block_comment = false;
    for line in content.trim_start_matches('\u{feff}').lines() {
        let line = line.trim();
        // Engine-written Baldur.lua has no block comments. Don't infer defaults
        // from a manually commented-out example or an arbitrary Lua program.
        if line.starts_with("--[") {
            block_comment = true;
        }
        for (index, (key, _)) in DEFAULTS.iter().enumerate() {
            if !line.contains(key) {
                continue;
            }
            if block_comment {
                return Err(settings_error(
                    path,
                    format!("cannot interpret {key} in a block comment"),
                ));
            }
            if line.starts_with("--") {
                continue;
            }
            match assignment(line) {
                Some(("Script", actual, "0" | "1")) if actual == *key => present[index] = true,
                Some((section, actual, _)) if section != "Script" || actual != *key => {}
                _ => {
                    return Err(settings_error(
                        path,
                        format!("cannot interpret {key}; left unchanged"),
                    ))
                }
            }
        }
        if block_comment && line.contains("]]") {
            block_comment = false;
        }
    }
    Ok(DEFAULTS
        .into_iter()
        .enumerate()
        .filter_map(|(index, pair)| (!present[index]).then_some(pair))
        .collect())
}

pub(super) fn ensure_profile_settings(
    layout: &ManagedLayout,
    identity: &ReservedSaveIdentity,
) -> Result<()> {
    verify_reserved_save_identity(layout, identity)?;
    if !interval_saves_installed(layout)? {
        return Ok(());
    }
    let path = identity.save_root.join("Baldur.lua");
    let before = read_optional_regular(&path)?;
    let bytes = before.as_deref().unwrap_or_default();
    let missing = missing_defaults(&path, bytes)?;
    if missing.is_empty() {
        return Ok(());
    }
    let newline = if bytes.windows(2).any(|pair| pair == b"\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut after = bytes.to_vec();
    if !after.is_empty() && !after.ends_with(b"\n") {
        after.extend_from_slice(newline.as_bytes());
    }
    for (key, value) in missing {
        after.extend_from_slice(
            format!("SetPrivateProfileString('Script','{key}','{value}'){newline}").as_bytes(),
        );
    }
    let mut temporary =
        tempfile::NamedTempFile::new_in(&identity.save_root).map_err(|source| EngineError::Io {
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
    // Don't clobber preferences written after inspection (e.g. a concurrent editor).
    if read_optional_regular(&path)? != before {
        return Err(settings_error(
            &path,
            "file changed while preparing defaults; retry when the game is closed",
        ));
    }
    temporary.persist(&path).map_err(|error| EngineError::Io {
        path: path.clone(),
        source: error.error,
    })?;
    if read_optional_regular(&path)?.as_deref() != Some(after.as_slice()) {
        return Err(settings_error(
            &path,
            "written settings did not match read-back",
        ));
    }
    verify_profile_settings(layout, identity)
}

/// Check the final isolated profile before declaring an installation successful.
/// An explicit user value of 0 is valid; only missing required keys are a defect.
pub fn verify_profile_settings(
    layout: &ManagedLayout,
    identity: &ReservedSaveIdentity,
) -> Result<()> {
    verify_reserved_save_identity(layout, identity)?;
    if !interval_saves_installed(layout)? {
        return Ok(());
    }
    let path = identity.save_root.join("Baldur.lua");
    let bytes = read_optional_regular(&path)?.unwrap_or_default();
    let missing = missing_defaults(&path, &bytes)?;
    if !missing.is_empty() {
        return Err(settings_error(
            &path,
            "required settings are missing from the isolated game profile",
        ));
    }
    Ok(())
}
