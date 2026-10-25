//! The game's own log for crash reports: InfinityLoader's `LogFile` setting in the game folder's
//! `InfinityLoader.ini`, and the copy of the previous session's log that CEBG keeps.
//!
//! InfinityLoader reads `LogFile` from the `[General]` section of `InfinityLoader.ini` in its
//! working folder, the game folder, and mirrors all console output into that file. A missing or
//! blank value writes no log. The loader empties the log at every start and keeps it open until
//! the game ends; others can read it meanwhile but not change, rename or remove it. When the game
//! crashes, the loader's crash handler copies the log into `InfinityLoader_Crash`, next to the
//! crash dumps. The loader can neither append to a log nor name one per session, so CEBG keeps
//! the log of the previous session itself whenever it starts the game.
//!
//! The setting is changed, and the log is copied, through the game folder held from the managed
//! root down, so a link swapped in meanwhile can never redirect them. What CEBG only shows the
//! player is read by path inside that held folder, so looking never keeps a game that starts
//! meanwhile from its files: the setting shared with writers, since the loader may save its
//! settings as it starts and fails if it cannot, and only the size and time of the log files,
//! without access to their content, since a starting game that finds its log opened for reading
//! writes no log.

use std::collections::BTreeSet;
use std::fs;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::Serialize;

use crate::digest::sha256_bytes;
use crate::held_folder::{EntryError, EntryErrorKind, HeldFolder};
use crate::lock::TargetLock;
use crate::preflight::{PreflightError, PreflightHost, SystemPreflight};

/// The log file CEBG sets, in the game folder.
pub const LOG_FILE: &str = "InfinityLoader.log";
/// Where CEBG keeps the log of the session before, each time it starts the game.
pub const PREVIOUS_LOG_FILE: &str = "InfinityLoader-previous.log";
/// Where InfinityLoader saves its crash dumps and a copy of the log, in the game folder.
pub const CRASH_FOLDER: &str = "InfinityLoader_Crash";
const INI: &str = "InfinityLoader.ini";
const GAME: &str = "game";
/// A loader configuration takes a few KiB.
pub(crate) const INI_LIMIT: u64 = 1024 * 1024;
/// The most of a log that is kept: its end, where a crash or freeze shows.
const KEEP_LIMIT: u64 = 64 * 1024 * 1024;

/// Whether the game writes its log.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameLogSetting {
    /// `LogFile` is missing or blank: the game writes no log.
    Off,
    /// `LogFile` names [`LOG_FILE`].
    On,
    /// `LogFile` names another file, set outside CEBG.
    Other,
    /// There is no usable `InfinityLoader.ini` to change.
    Unavailable,
}

/// What is known about one file of the game log.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameLogFileState {
    Missing,
    Present,
    /// A folder, link or other entry where the file belongs.
    NotAFile,
    Unreadable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GameLogFile {
    pub path: PathBuf,
    pub state: GameLogFileState,
    pub size_bytes: Option<u64>,
    pub modified_at_millis: Option<u64>,
    /// Why it is [`GameLogFileState::Unreadable`].
    pub detail: Option<String>,
}

/// The game-log setting of one installation and where its evidence is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GameLogStatus {
    pub setting: GameLogSetting,
    /// The configured log file for [`GameLogSetting::Other`], or why the setting is
    /// [`GameLogSetting::Unavailable`].
    pub detail: Option<String>,
    pub log: GameLogFile,
    pub previous_log: GameLogFile,
    pub crash_folder: PathBuf,
    /// How many crashes left a report in the crash folder; `None` when it cannot be looked into.
    pub crash_reports: Option<usize>,
}

/// What became of the previous session's log when the game was started.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum KeptLog {
    /// The game log is off or set outside CEBG, so nothing was copied.
    NotEnabled,
    /// There was no log of a previous session, or it was empty.
    NothingToKeep,
    /// The log is now [`PREVIOUS_LOG_FILE`]; only its end when it was over the size limit.
    Kept { bytes: u64, shortened: bool },
    /// The previous log stays as it was.
    Failed { reason: String },
}

/// The game-log setting of the installation in `managed_root` and the files of its evidence.
/// Reads only, and never fails: what cannot be read is reported in the status.
pub fn status(managed_root: &Path) -> GameLogStatus {
    let game_path = managed_root.join(GAME);
    let crash_folder = game_path.join(CRASH_FOLDER);
    let game = match HeldFolder::open(managed_root, [GAME]) {
        Ok(game) => game,
        Err(error) => {
            let message = entry_message(error);
            let unknown = |name: &str| GameLogFile {
                path: game_path.join(name),
                state: GameLogFileState::Unreadable,
                size_bytes: None,
                modified_at_millis: None,
                detail: Some(message.clone()),
            };
            return GameLogStatus {
                setting: GameLogSetting::Unavailable,
                detail: Some(message.clone()),
                log: unknown(LOG_FILE),
                previous_log: unknown(PREVIOUS_LOG_FILE),
                crash_folder,
                crash_reports: None,
            };
        }
    };
    let (setting, detail) = match read_ini_shared(&game) {
        Ok(Some(bytes)) => LoaderIni::parse(&bytes).setting(),
        Ok(None) => (
            GameLogSetting::Unavailable,
            Some(format!("This installation has no {INI}.")),
        ),
        Err(error) => (GameLogSetting::Unavailable, Some(error)),
    };
    GameLogStatus {
        setting,
        detail,
        log: file_facts(&game, LOG_FILE),
        previous_log: file_facts(&game, PREVIOUS_LOG_FILE),
        crash_folder,
        crash_reports: crash_reports(game),
    }
}

/// Turn the game log of the installation in `managed_root` on or off for the next game start.
/// Only the `LogFile` value in the `[General]` section of `InfinityLoader.ini` changes, and only
/// when it is not already as requested. A log set outside CEBG counts as on.
pub fn set(app_data: &Path, managed_root: &Path, enabled: bool) -> Result<GameLogStatus, String> {
    set_with(app_data, managed_root, enabled, &SystemPreflight)
}

/// [`set`] with the programs that run supplied by `host`.
pub fn set_with(
    app_data: &Path,
    managed_root: &Path,
    enabled: bool,
    host: &dyn PreflightHost,
) -> Result<GameLogStatus, String> {
    let _lock =
        TargetLock::try_acquire(crate::lock::shared_target_lock_root(app_data), managed_root)
            .map_err(|error| format!("This installation is in use: {error}"))?;
    crate::patches::launch_guard(managed_root)?;
    crate::mod_upgrades::launch_guard(managed_root)?;
    // The loader's crash handler reads the setting when the game crashes: changing it while the
    // game runs would leave that crash without its log, or copy an old log as if it were its own.
    // CEBG's own Play waits for the lock held here; a game started another way in the moment
    // until the file is replaced is not seen.
    crate::preflight::check_no_program_running_under(&managed_root.join(GAME), host).map_err(
        |error| match error {
            PreflightError::TargetProcessRunning { executable } => format!(
                "Close the game and other programs from this installation first; still running: {}",
                executable.display()
            ),
            error => error.to_string(),
        },
    )?;
    set_in(
        &HeldFolder::open(managed_root, [GAME]).map_err(entry_message)?,
        enabled,
    )?;
    Ok(status(managed_root))
}

/// Change the setting in the held game folder, replacing `InfinityLoader.ini` in one step.
fn set_in(game: &HeldFolder, enabled: bool) -> Result<(), String> {
    let before = read_ini(game)?.ok_or_else(|| format!("This installation has no {INI}."))?;
    let ini = LoaderIni::parse(&before);
    let after = match (ini.setting(), enabled) {
        ((GameLogSetting::Unavailable, detail), _) => return Err(detail.unwrap_or_default()),
        ((GameLogSetting::On | GameLogSetting::Other, _), true)
        | ((GameLogSetting::Off, _), false) => return Ok(()),
        ((GameLogSetting::Off, _), true) => ini.with_value(LOG_FILE.as_bytes()),
        ((GameLogSetting::On | GameLogSetting::Other, _), false) => ini.with_value(b""),
    };
    let after = after.ok_or_else(|| format!("{INI} has no [General] section."))?;
    game.replace_file(INI, &after)
        .map_err(|error| format!("{}: {error}", game.path().join(INI).display()))
}

/// Copy the log of the last session to [`PREVIOUS_LOG_FILE`] before the game starts and empties
/// it, when the game log is on. Never fails: a log that cannot be kept is reported instead, but
/// only when the game is set to write it, so a game folder CEBG cannot hold, such as a link,
/// never warns about a log that is off.
pub fn keep_previous(managed_root: &Path) -> KeptLog {
    HeldFolder::open(managed_root, [GAME])
        .map_err(entry_message)
        .and_then(|game| keep_in(&game, KEEP_LIMIT))
        .unwrap_or_else(|reason| {
            if log_on_for_the_loader(managed_root) {
                KeptLog::Failed { reason }
            } else {
                KeptLog::NotEnabled
            }
        })
}

/// Whether the loader will write [`LOG_FILE`], read by path as the loader reads its settings.
/// Settings it cannot read leave its log off.
fn log_on_for_the_loader(managed_root: &Path) -> bool {
    let path = managed_root.join(GAME).join(INI);
    fs::File::open(&path)
        .map_err(|error| error.to_string())
        .and_then(|file| read_limited(&file, &path))
        .is_ok_and(|bytes| LoaderIni::parse(&bytes).setting().0 == GameLogSetting::On)
}

/// Keep the log in the held game folder, or only its last `limit` bytes.
fn keep_in(game: &HeldFolder, limit: u64) -> Result<KeptLog, String> {
    let Some(ini) = read_ini(game)? else {
        return Ok(KeptLog::NotEnabled);
    };
    if LoaderIni::parse(&ini).setting().0 != GameLogSetting::On {
        return Ok(KeptLog::NotEnabled);
    }
    let Some((content, shortened)) = read_log_end(game, limit)? else {
        return Ok(KeptLog::NothingToKeep);
    };
    game.replace_file(PREVIOUS_LOG_FILE, &content)
        .map_err(|error| format!("{}: {error}", game.path().join(PREVIOUS_LOG_FILE).display()))?;
    Ok(KeptLog::Kept {
        bytes: content.len() as u64,
        shortened,
    })
}

/// The log of the last session, or only its last `limit` bytes after a note saying so, and
/// whether it was shortened. `None` when there is none or it is empty. Nothing can change the log
/// while it is read, so the copy is never torn; it is closed again when this returns, before
/// CEBG starts the game. A game started another way in that moment writes no log.
fn read_log_end(game: &HeldFolder, limit: u64) -> Result<Option<(Vec<u8>, bool)>, String> {
    let path = game.path().join(LOG_FILE);
    let (file, metadata) = match game.open_file(LOG_FILE) {
        Ok(found) => found,
        Err(error) if in_use(&error) => {
            return Err("The game log is in use, probably by a game that is still running.".into())
        }
        Err(EntryError {
            kind: EntryErrorKind::Missing,
            ..
        }) => return Ok(None),
        Err(error) => return Err(entry_message(error)),
    };
    let size = metadata.len();
    if size == 0 {
        return Ok(None);
    }
    let read_error = |error: std::io::Error| format!("{}: {error}", path.display());
    let shortened = size > limit;
    let mut content = Vec::new();
    if shortened {
        content.extend_from_slice(
            format!("[CEBG] This log had {size} bytes; only its last {limit} bytes were kept.\r\n")
                .as_bytes(),
        );
        (&file)
            .seek(SeekFrom::Start(size - limit))
            .map_err(read_error)?;
    }
    (&file)
        .take(limit)
        .read_to_end(&mut content)
        .map_err(read_error)?;
    Ok(Some((content, shortened)))
}

/// Whether the `InfinityLoader.ini` `bytes` had `expected_sha256` before the game log was turned
/// on or off: their game-log setting is one that CEBG writes, and setting it back, to blank, to
/// [`LOG_FILE`] or to no `LogFile` line at all, gives bytes with that hash.
pub(crate) fn unchanged_apart_from_the_setting(bytes: &[u8], expected_sha256: &str) -> bool {
    let ini = LoaderIni::parse(bytes);
    matches!(ini.setting().0, GameLogSetting::Off | GameLogSetting::On)
        && [
            ini.with_value(b""),
            ini.with_value(LOG_FILE.as_bytes()),
            ini.without_line(),
        ]
        .into_iter()
        .flatten()
        .any(|variant| sha256_bytes(&variant) == expected_sha256)
}

/// The bytes of `InfinityLoader.ini` in the held game folder, `None` when there is none. Nothing
/// can change the file while it is read, so it is the base of a change.
fn read_ini(game: &HeldFolder) -> Result<Option<Vec<u8>>, String> {
    let path = game.path().join(INI);
    match game.open_file(INI) {
        Ok((file, _)) => read_limited(&file, &path).map(Some),
        Err(EntryError {
            kind: EntryErrorKind::Missing,
            ..
        }) => Ok(None),
        Err(error) => Err(entry_message(error)),
    }
}

/// The bytes of `InfinityLoader.ini` in the held game folder, `None` when there is none, read by
/// path and shared with writers, so a game starting meanwhile can still save its settings, which
/// fails its start otherwise. Only for what CEBG shows, never for a change or what it copies.
fn read_ini_shared(game: &HeldFolder) -> Result<Option<Vec<u8>>, String> {
    let path = game.path().join(INI);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() && !is_link_or_reparse(&metadata) => {}
        Ok(_) => {
            return Err(entry_message(EntryError {
                path,
                kind: EntryErrorKind::NotDirect,
            }))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    }
    let file = fs::File::open(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    read_limited(&file, &path).map(Some)
}

/// At most [`INI_LIMIT`] bytes of a loader configuration.
fn read_limited(file: &fs::File, path: &Path) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    file.take(INI_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() as u64 > INI_LIMIT {
        return Err(format!(
            "{} is larger than {INI_LIMIT} bytes.",
            path.display()
        ));
    }
    Ok(bytes)
}

/// What the player is shown about the file `name` in the game folder. Read by path with no access
/// to the content, so a game starting meanwhile still opens its log.
fn file_facts(game: &HeldFolder, name: &str) -> GameLogFile {
    let mut facts = GameLogFile {
        path: game.path().join(name),
        state: GameLogFileState::Present,
        size_bytes: None,
        modified_at_millis: None,
        detail: None,
    };
    match fs::symlink_metadata(&facts.path) {
        Ok(metadata) if metadata.is_file() && !is_link_or_reparse(&metadata) => {
            facts.size_bytes = Some(metadata.len());
            facts.modified_at_millis = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .and_then(|elapsed| u64::try_from(elapsed.as_millis()).ok());
        }
        Ok(_) => facts.state = GameLogFileState::NotAFile,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            facts.state = GameLogFileState::Missing;
        }
        Err(error) => {
            facts.state = GameLogFileState::Unreadable;
            facts.detail = Some(error.to_string());
        }
    }
    facts
}

#[cfg(windows)]
fn is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

/// The number of crashes with a report in the crash folder of the held `game` folder: each leaves
/// `InfinityLoader_Crash_<number>` with `.log`, `_small.dmp` or `_big.dmp`.
fn crash_reports(game: HeldFolder) -> Option<usize> {
    let folder = match game.descend(CRASH_FOLDER) {
        Ok(folder) => folder,
        Err(EntryError {
            kind: EntryErrorKind::Missing,
            ..
        }) => return Some(0),
        Err(_) => return None,
    };
    let names = folder.names().ok()?;
    Some(
        names
            .iter()
            .filter_map(|name| crash_number(name.to_str()?))
            .collect::<BTreeSet<_>>()
            .len(),
    )
}

fn crash_number(name: &str) -> Option<u64> {
    let name = name.to_ascii_lowercase();
    let rest = name.strip_prefix("infinityloader_crash_")?;
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let (number, kind) = rest.split_at(digits);
    if number.is_empty() || !matches!(kind, ".log" | "_small.dmp" | "_big.dmp") {
        return None;
    }
    number.parse().ok()
}

/// Open by a program that writes it, as the running game does with its log.
fn in_use(error: &EntryError) -> bool {
    // ERROR_SHARING_VIOLATION
    matches!(&error.kind, EntryErrorKind::Io(error) if error.raw_os_error() == Some(32))
}

fn entry_message(error: EntryError) -> String {
    let path = error.path.display();
    match error.kind {
        EntryErrorKind::Missing => format!("Missing file or folder: {path}"),
        EntryErrorKind::NotDirect => {
            format!("A link or another unexpected entry is in the way: {path}")
        }
        EntryErrorKind::Io(error) => format!("{path}: {error}"),
    }
}

/// `InfinityLoader.ini` as Windows reads it for the loader: the first `[General]` section, and in
/// it the first `LogFile` key. Names match ignoring case, and a value is read without the spaces
/// around it and without one pair of quotes around it. A line ends at a line feed, a carriage
/// return or both, and each line keeps its own line ending, so a change to the value leaves every
/// other byte as it was. A byte order mark is part of the first line, as Windows reads it, and a
/// file saved as UTF-16 is not changed.
struct LoaderIni<'a> {
    bytes: &'a [u8],
    /// The first `[General]` header line, its line ending included.
    general: Option<Range<usize>>,
    /// The first `LogFile` line in that section.
    log_file: Option<LogFileLine>,
}

struct LogFileLine {
    /// The whole line, its line ending included.
    line: Range<usize>,
    /// The value: after `=` and the spaces after it, up to the line ending.
    value: Range<usize>,
}

impl<'a> LoaderIni<'a> {
    fn parse(bytes: &'a [u8]) -> Self {
        let mut general = None;
        let mut log_file = None;
        let mut in_general = false;
        let mut start = 0;
        while start < bytes.len() {
            let (content_end, end) = line_end(bytes, start);
            let content = &bytes[start..content_end];
            if let Some(name) = section_name(trim(content)) {
                in_general = general.is_none() && name.eq_ignore_ascii_case(b"General");
                if in_general {
                    general = Some(start..end);
                }
            } else if in_general && log_file.is_none() {
                if let Some(equals) = content.iter().position(|byte| *byte == b'=') {
                    if trim(&content[..equals]).eq_ignore_ascii_case(b"LogFile") {
                        let mut value = start + equals + 1;
                        while value < content_end && is_blank(bytes[value]) {
                            value += 1;
                        }
                        log_file = Some(LogFileLine {
                            line: start..end,
                            value: value..content_end,
                        });
                    }
                }
            }
            start = end;
        }
        Self {
            bytes,
            general,
            log_file,
        }
    }

    /// The setting, with the configured file for [`GameLogSetting::Other`] or why it is
    /// [`GameLogSetting::Unavailable`].
    fn setting(&self) -> (GameLogSetting, Option<String>) {
        if self.bytes.starts_with(b"\xFF\xFE") || self.bytes.starts_with(b"\xFE\xFF") {
            return (
                GameLogSetting::Unavailable,
                Some(format!(
                    "{INI} is saved as UTF-16, which CEBG does not change."
                )),
            );
        }
        if self.general.is_none() {
            return (
                GameLogSetting::Unavailable,
                Some(format!("{INI} has no [General] section.")),
            );
        }
        match self.configured() {
            None => (GameLogSetting::Off, None),
            Some(name) if name.eq_ignore_ascii_case(LOG_FILE.as_bytes()) => {
                (GameLogSetting::On, None)
            }
            Some(name) => (
                GameLogSetting::Other,
                Some(String::from_utf8_lossy(name).into_owned()),
            ),
        }
    }

    /// The log file the loader writes, `None` when it writes none.
    fn configured(&self) -> Option<&'a [u8]> {
        let line = self.log_file.as_ref()?;
        let value = trim(&self.bytes[line.value.clone()]);
        let value = match value {
            [quote @ (b'"' | b'\''), inner @ .., last] if last == quote => inner,
            _ => value,
        };
        (!value.is_empty()).then_some(value)
    }

    /// These bytes with `value` as the log file: in place of the value of the first `LogFile`
    /// line, or in a new line right after the `[General]` header. `None` without that section.
    fn with_value(&self, value: &[u8]) -> Option<Vec<u8>> {
        let bytes = self.bytes;
        if let Some(line) = &self.log_file {
            return Some([&bytes[..line.value.start], value, &bytes[line.value.end..]].concat());
        }
        let header = self.general.clone()?;
        let line = [b"LogFile=".as_slice(), value].concat();
        Some(match ending_of(&bytes[header.clone()]) {
            // The header is the last line.
            [] => [bytes, line_ending(bytes), &line].concat(),
            ending => [&bytes[..header.end], &line, ending, &bytes[header.end..]].concat(),
        })
    }

    /// These bytes without the first `LogFile` line, as before such a line was added.
    fn without_line(&self) -> Option<Vec<u8>> {
        let line = self.log_file.as_ref()?;
        let bytes = self.bytes;
        let mut start = line.line.start;
        if ending_of(&bytes[line.line.clone()]).is_empty() {
            // The last line: a line ending was added before it, too.
            start -= ending_of(&bytes[..start]).len();
        }
        Some([&bytes[..start], &bytes[line.line.end..]].concat())
    }
}

/// Where the line starting at `start` ends, before and after its line ending. As for Windows, a
/// line ends at a line feed, a carriage return, or a carriage return and a line feed together.
fn line_end(bytes: &[u8], start: usize) -> (usize, usize) {
    let Some(offset) = bytes[start..]
        .iter()
        .position(|byte| matches!(byte, b'\n' | b'\r'))
    else {
        return (bytes.len(), bytes.len());
    };
    let content_end = start + offset;
    let crlf = bytes[content_end] == b'\r' && bytes.get(content_end + 1) == Some(&b'\n');
    (content_end, content_end + if crlf { 2 } else { 1 })
}

/// The line ending at the end of `text`, empty when there is none.
fn ending_of(text: &[u8]) -> &[u8] {
    let content = text.strip_suffix(b"\n").unwrap_or(text);
    let content = content.strip_suffix(b"\r").unwrap_or(content);
    &text[content.len()..]
}

/// The name of a `[section]` header line as Windows reads it: up to the first `]`, or to the end
/// of the line without one, and without the spaces around it.
fn section_name(line: &[u8]) -> Option<&[u8]> {
    let inner = line.strip_prefix(b"[")?;
    let close = inner
        .iter()
        .position(|byte| *byte == b']')
        .unwrap_or(inner.len());
    Some(trim(&inner[..close]))
}

/// The line ending this file uses first, or Windows' own.
fn line_ending(bytes: &[u8]) -> &'static [u8] {
    match bytes.iter().position(|byte| matches!(byte, b'\n' | b'\r')) {
        Some(at) if bytes[at] == b'\n' => b"\n",
        Some(at) if bytes.get(at + 1) != Some(&b'\n') => b"\r",
        _ => b"\r\n",
    }
}

fn is_blank(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | 0x0b | 0x0c | 0x1a)
}

fn trim(mut bytes: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = bytes {
        if !is_blank(*first) {
            break;
        }
        bytes = rest;
    }
    while let [rest @ .., last] = bytes {
        if !is_blank(*last) {
            break;
        }
        bytes = rest;
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::held_folder::tests::entries;
    use std::fs;

    /// The loader settings EEex installs, shortened: `LogFile` is blank among the other
    /// `[General]` keys, and the loader caches its patterns in later sections.
    const EEEX_INI: &str = "\n[General]\n\n; 0 -> Normal behavior (default).\n\nDebug=0\n\n\
        ExeNames=Baldur.exe,BaldurII.exe,icewind.exe,SiegeOfDragonspear.exe\n\n\
        ; The name of the log file where all console output will be mirrored. If this field is \
        blank no log is automatically written.\n\nLogFile=\n\nLuaLibrary=lua51.dll\n\n\
        LuaPatchMode=REPLACE_INTERNAL_WITH_EXTERNAL\n\n[Auto-Generated]\nCachedExeTime=13434029074137\n\n\
        [Hardcoded_free]\nPattern=743753\nOperations=ADD -3\n";

    fn on(ini: &str) -> String {
        ini.replacen("LogFile=\n", "LogFile=InfinityLoader.log\n", 1)
    }

    fn setting(bytes: &[u8]) -> (GameLogSetting, Option<String>) {
        LoaderIni::parse(bytes).setting()
    }

    fn with_value(bytes: &[u8], value: &str) -> Vec<u8> {
        LoaderIni::parse(bytes)
            .with_value(value.as_bytes())
            .unwrap()
    }

    /// An installation below `parent` whose game folder holds `ini`.
    fn installation(parent: &Path, ini: &[u8]) -> PathBuf {
        let root = parent.join("managed");
        fs::create_dir_all(root.join(GAME)).unwrap();
        fs::write(root.join(GAME).join(INI), ini).unwrap();
        root
    }

    fn ini_of(root: &Path) -> Vec<u8> {
        fs::read(root.join(GAME).join(INI)).unwrap()
    }

    /// The programs that run.
    struct Programs(Vec<PathBuf>);

    impl PreflightHost for Programs {
        fn available_space(&self, _: &Path) -> std::io::Result<u64> {
            unreachable!()
        }
        fn volume_key(&self, _: &Path) -> std::io::Result<std::ffi::OsString> {
            unreachable!()
        }
        fn probe_directory_writable(&self, _: &Path) -> std::io::Result<()> {
            unreachable!()
        }
        fn running_executable_paths(&self) -> std::io::Result<Vec<PathBuf>> {
            Ok(self.0.clone())
        }
        fn probe_exclusive_writable_files(
            &self,
            _: &[PathBuf],
        ) -> Result<(), crate::preflight::ExclusiveFileError> {
            unreachable!()
        }
    }

    #[test]
    fn turning_the_log_on_and_off_changes_only_its_value() {
        for ini in [EEEX_INI.to_owned(), EEEX_INI.replace('\n', "\r\n")] {
            let bytes = ini.as_bytes();
            assert_eq!(setting(bytes), (GameLogSetting::Off, None));

            let enabled = with_value(bytes, LOG_FILE);
            let expected = ini.replacen("LogFile=", "LogFile=InfinityLoader.log", 1);
            assert_eq!(String::from_utf8(enabled.clone()).unwrap(), expected);
            assert_eq!(setting(&enabled), (GameLogSetting::On, None));

            assert_eq!(with_value(&enabled, ""), bytes);
        }
    }

    /// Lines keep their own line endings, and bytes that are not UTF-8 stay as they are.
    #[test]
    fn mixed_line_endings_a_byte_order_mark_and_other_bytes_are_kept() {
        let ini = b"\xEF\xBB\xBF; settings\r\n[General]\r\n; caf\xE9 \xFF\n\
            ExeNames=Baldur.exe\rLogFile=\nLuaLibrary=lua51.dll\r\n";
        assert_eq!(setting(ini), (GameLogSetting::Off, None));

        let enabled = with_value(ini, LOG_FILE);
        assert_eq!(
            enabled,
            b"\xEF\xBB\xBF; settings\r\n[General]\r\n; caf\xE9 \xFF\n\
            ExeNames=Baldur.exe\rLogFile=InfinityLoader.log\nLuaLibrary=lua51.dll\r\n"
        );
        assert_eq!(with_value(&enabled, ""), ini);
    }

    /// Windows ends a line at a carriage return alone, too: the value ends there, and the key
    /// after it is never part of it.
    #[test]
    fn a_carriage_return_alone_ends_a_line() {
        let ini = b"[General]\nLogFile=InfinityLoader.log\rLuaLibrary=lua51.dll\n";
        assert_eq!(setting(ini), (GameLogSetting::On, None));
        let disabled = with_value(ini, "");
        assert_eq!(disabled, b"[General]\nLogFile=\rLuaLibrary=lua51.dll\n");
        assert_eq!(with_value(&disabled, LOG_FILE), ini);

        // A file with carriage returns only, with and without the line.
        let ini = b"[General]\rDebug=0\rLuaLibrary=lua51.dll\r";
        let enabled = with_value(ini, LOG_FILE);
        assert_eq!(
            enabled,
            b"[General]\rLogFile=InfinityLoader.log\rDebug=0\rLuaLibrary=lua51.dll\r"
        );
        assert_eq!(setting(&enabled), (GameLogSetting::On, None));
        assert_eq!(LoaderIni::parse(&enabled).without_line().unwrap(), ini);
        let ini = b"Debug=0\r[General]";
        let enabled = with_value(ini, LOG_FILE);
        assert_eq!(enabled, b"Debug=0\r[General]\rLogFile=InfinityLoader.log");
        assert_eq!(LoaderIni::parse(&enabled).without_line().unwrap(), ini);
    }

    /// Section names as Windows reads them: up to the first `]` or the end of the line, without
    /// the spaces around them. A byte order mark belongs to the first line, so a header there is
    /// none, and a file saved as UTF-16 is never changed.
    #[test]
    fn section_headers_are_read_as_windows_reads_them() {
        for (ini, expected) in [
            (
                &b"[ General ]\nLogFile=InfinityLoader.log\n[General]\nLogFile=\n"[..],
                GameLogSetting::On,
            ),
            (
                &b"[\tgeneral ; note\nLogFile=x.log\n"[..],
                GameLogSetting::Unavailable,
            ),
            (
                &b"[ General\nLogFile=InfinityLoader.log\n"[..],
                GameLogSetting::On,
            ),
            (
                &b"[General] ; note\nLogFile=InfinityLoader.log\n"[..],
                GameLogSetting::On,
            ),
            (
                &b"[Gen]eral]\nLogFile=InfinityLoader.log\n"[..],
                GameLogSetting::Unavailable,
            ),
            (
                &b"\xEF\xBB\xBF[General]\nLogFile=InfinityLoader.log\n"[..],
                GameLogSetting::Unavailable,
            ),
            (
                &b"\xEF\xBB\xBF\n[General]\nLogFile=\n"[..],
                GameLogSetting::Off,
            ),
        ] {
            assert_eq!(setting(ini).0, expected, "{}", String::from_utf8_lossy(ini));
        }
        assert_eq!(
            with_value(b"[ General ]\nLogFile=\n[General]\nLogFile=\n", LOG_FILE),
            b"[ General ]\nLogFile=InfinityLoader.log\n[General]\nLogFile=\n"
        );

        let utf16: Vec<u8> = [0xFEFF_u16]
            .into_iter()
            .chain("[General]\r\nLogFile=\r\n".encode_utf16())
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(
            setting(&utf16),
            (
                GameLogSetting::Unavailable,
                Some("InfinityLoader.ini is saved as UTF-16, which CEBG does not change.".into())
            )
        );
    }

    /// The log file as the loader gets it: each case where reading by hand could go wrong,
    /// compared with what `GetPrivateProfileStringW` itself reads from the same bytes.
    #[cfg(windows)]
    #[test]
    fn the_log_file_is_the_one_windows_reads_for_the_loader() {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::System::WindowsProgramming::GetPrivateProfileStringW;

        fn wide(text: &OsStr) -> Vec<u16> {
            text.encode_wide().chain([0]).collect()
        }
        let crlf = EEEX_INI.replace('\n', "\r\n");
        let enabled = on(EEEX_INI);
        let cases: Vec<&[u8]> = vec![
            EEEX_INI.as_bytes(),
            crlf.as_bytes(),
            enabled.as_bytes(),
            b"",
            b"[General]",
            b"[General]\nLogFile=InfinityLoader.log\rLuaLibrary=lua51.dll\n",
            b"[General]\rDebug=0\rLogFile=x.log\r",
            b"[General]\r\rLogFile=x.log\n",
            b"[General]\r\nLogFile=x.log\r\r\n",
            b"[General]\nLogFile=x.log\r; note\n",
            b"[ General ]\nLogFile=a.log\n[General]\nLogFile=b.log\n",
            b"[\tGeneral\x1a]\nLogFile=a.log\n",
            b"[ General\nLogFile=a.log\n",
            b"[General ; note\nLogFile=a.log\n",
            b"[General] ; note\nLogFile=a.log\n",
            b"[General]x\nLogFile=a.log\n",
            b"[Gen]eral]\nLogFile=a.log\n",
            b"  [general]  \nlogfile=a.log\n",
            b"\xEF\xBB\xBF[General]\nLogFile=a.log\n",
            b"\xEF\xBB\xBF\n[General]\nLogFile=a.log\n",
            b"[General]\nLogFile=a.log\n[General]\nLogFile=b.log\n",
            b"[General]\nDebug=0\n[General]\nLogFile=b.log\n",
            b"[General]\nLogFile=a.log\nLogFile=b.log\n",
            b"[Other]\nLogFile=a.log\n[General]\nDebug=0\n",
            b"LogFile=a.log\n[General]\n",
            b"[General]\nLogFile=\"a.log\"\n",
            b"[General]\nLogFile='a.log'\n",
            b"[General]\nLogFile=\"a.log\n",
            b"[General]\nLogFile=\"\"\n",
            b"[General]\nLogFile=   \n",
            b"[General]\n  LogFile  =  a b.log  \n",
            b"[General]\n\tLogFile\t=\x0ba.log\x0c\x1a\n",
            b"[General]\n\x1aLogFile=a.log\n",
            b"[General]\nLogFile=a=b.log\n",
            b"[General]\n;LogFile=a.log\n",
            b"[General]\nLogFile\n",
        ];
        let temp = tempfile::tempdir().unwrap();
        let (section, key, unset) = (
            wide(OsStr::new("General")),
            wide(OsStr::new("LogFile")),
            wide(OsStr::new("\u{1}")),
        );
        for (index, ini) in cases.into_iter().enumerate() {
            let path = temp.path().join(format!("{index}.ini"));
            fs::write(&path, ini).unwrap();
            let file = wide(path.as_os_str());
            let mut buffer = [0_u16; 512];
            // SAFETY: every string is NUL-terminated and outlives the call, and the buffer length
            // is given in UTF-16 units.
            let read = unsafe {
                GetPrivateProfileStringW(
                    section.as_ptr(),
                    key.as_ptr(),
                    unset.as_ptr(),
                    buffer.as_mut_ptr(),
                    buffer.len() as u32,
                    file.as_ptr(),
                )
            };
            let windows = String::from_utf16(&buffer[..read as usize]).unwrap();
            let windows = (!windows.is_empty() && windows != "\u{1}").then_some(windows);
            let ours = LoaderIni::parse(ini)
                .configured()
                .map(|value| String::from_utf8(value.to_vec()).unwrap());
            assert_eq!(
                ours,
                windows,
                "{}",
                String::from_utf8_lossy(ini).escape_debug()
            );
        }
    }

    /// Without `LogFile` in `[General]`, a line is added right after its header, with that
    /// header's line ending. A key of that name in another section does not count.
    #[test]
    fn a_missing_log_file_line_is_added_after_the_general_header() {
        for ending in ["\n", "\r\n"] {
            let ini =
                format!("[General]{ending}Debug=0{ending}[Other]{ending}LogFile=other.log{ending}");
            assert_eq!(setting(ini.as_bytes()), (GameLogSetting::Off, None));

            let enabled = with_value(ini.as_bytes(), LOG_FILE);
            assert_eq!(
                String::from_utf8(enabled.clone()).unwrap(),
                format!(
                    "[General]{ending}LogFile=InfinityLoader.log{ending}Debug=0{ending}[Other]{ending}LogFile=other.log{ending}"
                )
            );
            assert_eq!(
                LoaderIni::parse(&enabled).without_line().unwrap(),
                ini.as_bytes()
            );
        }

        // The header as the last line, without a line ending.
        let ini = b"ExeNames=Baldur.exe\r\n[General]";
        let enabled = with_value(ini, LOG_FILE);
        assert_eq!(
            enabled,
            b"ExeNames=Baldur.exe\r\n[General]\r\nLogFile=InfinityLoader.log"
        );
        assert_eq!(LoaderIni::parse(&enabled).without_line().unwrap(), ini);
    }

    /// As Windows reads the file: the first `[General]` section and the first `LogFile` in it.
    #[test]
    fn only_the_first_general_section_and_its_first_log_file_count() {
        let ini = b"[General]\nLogFile=\nLogFile=second.log\n[General]\nLogFile=third.log\n";
        assert_eq!(setting(ini), (GameLogSetting::Off, None));
        assert_eq!(
            with_value(ini, LOG_FILE),
            b"[General]\nLogFile=InfinityLoader.log\nLogFile=second.log\n[General]\nLogFile=third.log\n"
        );

        let ini = b"[General]\nDebug=0\n[General]\nLogFile=InfinityLoader.log\n";
        assert_eq!(setting(ini), (GameLogSetting::Off, None));
        assert_eq!(
            with_value(ini, LOG_FILE),
            b"[General]\nLogFile=InfinityLoader.log\nDebug=0\n[General]\nLogFile=InfinityLoader.log\n"
        );
    }

    #[test]
    fn names_ignore_case_and_values_their_spaces_and_quotes() {
        for (ini, expected) in [
            (
                &b"[general]\n  logfile =  \"InfinityLoader.LOG\"  \n"[..],
                (GameLogSetting::On, None),
            ),
            (
                &b"[General]\nLogFile='InfinityLoader.log'\n"[..],
                (GameLogSetting::On, None),
            ),
            (
                &b"[General]\nLogFile=  \t \n"[..],
                (GameLogSetting::Off, None),
            ),
            (
                &b"[General]\nLogFile=\"\"\n"[..],
                (GameLogSetting::Off, None),
            ),
            (
                &b"[General]\n;LogFile=InfinityLoader.log\n"[..],
                (GameLogSetting::Off, None),
            ),
            (
                &b"[General]\nLogFile = logs\\game.log\n"[..],
                (GameLogSetting::Other, Some("logs\\game.log".to_owned())),
            ),
            (
                &b"[General]\nLogFile=\"InfinityLoader.log\n"[..],
                (
                    GameLogSetting::Other,
                    Some("\"InfinityLoader.log".to_owned()),
                ),
            ),
        ] {
            assert_eq!(setting(ini), expected, "{}", String::from_utf8_lossy(ini));
        }

        // The spaces around `=` stay; the value alone changes.
        let ini = b"[General]\r\nLogFile = \r\n";
        let enabled = with_value(ini, LOG_FILE);
        assert_eq!(enabled, b"[General]\r\nLogFile = InfinityLoader.log\r\n");
        assert_eq!(with_value(&enabled, ""), ini);
    }

    #[test]
    fn an_ini_without_a_general_section_is_unavailable() {
        for ini in [
            &b""[..],
            b"[Other]\nLogFile=InfinityLoader.log\n",
            b"\xFF\xFE[\0G\0",
        ] {
            assert_eq!(setting(ini).0, GameLogSetting::Unavailable);
            assert!(LoaderIni::parse(ini)
                .with_value(LOG_FILE.as_bytes())
                .is_none());
        }
    }

    /// Only a change of the game-log setting, as CEBG makes it, keeps the file unchanged.
    #[test]
    fn only_the_log_setting_may_differ_from_a_recorded_hash() {
        let off = EEEX_INI.as_bytes();
        let enabled = on(EEEX_INI);
        let recorded_off = sha256_bytes(off);
        let recorded_on = sha256_bytes(enabled.as_bytes());

        assert!(unchanged_apart_from_the_setting(
            enabled.as_bytes(),
            &recorded_off
        ));
        assert!(unchanged_apart_from_the_setting(off, &recorded_on));
        assert!(unchanged_apart_from_the_setting(off, &recorded_off));
        // Any other change counts, with or without the log.
        let other = enabled.replace("Debug=0", "Debug=1");
        assert!(!unchanged_apart_from_the_setting(
            other.as_bytes(),
            &recorded_off
        ));
        let other = EEEX_INI.replace("LuaLibrary=lua51.dll", "LuaLibrary=");
        assert!(!unchanged_apart_from_the_setting(
            other.as_bytes(),
            &recorded_off
        ));
        // A log set outside CEBG is not CEBG's change.
        let custom = EEEX_INI.replacen("LogFile=", "LogFile=mine.log", 1);
        assert!(!unchanged_apart_from_the_setting(
            custom.as_bytes(),
            &recorded_off
        ));
        // A line CEBG added, on or off again.
        let without = b"[General]\nDebug=0\n";
        let added = with_value(without, LOG_FILE);
        assert!(unchanged_apart_from_the_setting(
            &added,
            &sha256_bytes(without)
        ));
        let off_again = with_value(&added, "");
        assert!(unchanged_apart_from_the_setting(
            &off_again,
            &sha256_bytes(without)
        ));
    }

    #[test]
    fn crash_reports_are_counted_once_per_crash() {
        assert_eq!(crash_number("InfinityLoader_Crash_0.log"), Some(0));
        assert_eq!(crash_number("InfinityLoader_Crash_12_small.dmp"), Some(12));
        assert_eq!(crash_number("infinityloader_crash_3_BIG.DMP"), Some(3));
        for name in [
            "InfinityLoader_Crash_.log",
            "InfinityLoader_Crash_1.txt",
            "InfinityLoader_Crash_1_small.dmp.tmp",
            "Other_Crash_1.log",
        ] {
            assert_eq!(crash_number(name), None, "{name}");
        }

        let temp = tempfile::tempdir().unwrap();
        let root = installation(temp.path(), EEEX_INI.as_bytes());
        let reports = || crash_reports(HeldFolder::open(&root, [GAME]).unwrap());
        assert_eq!(reports(), Some(0));
        let crashes = root.join(GAME).join(CRASH_FOLDER);
        fs::create_dir(&crashes).unwrap();
        for name in [
            "InfinityLoader_Crash_0_big.dmp",
            "InfinityLoader_Crash_0_small.dmp",
            "InfinityLoader_Crash_0.log",
            "InfinityLoader_Crash_1_big.dmp",
            "InfinityLoader_Crash_1_small.dmp",
            "notes.txt",
        ] {
            fs::write(crashes.join(name), b"crash").unwrap();
        }
        assert_eq!(reports(), Some(2));
    }

    #[test]
    fn status_reports_the_setting_and_where_the_evidence_is() {
        let temp = tempfile::tempdir().unwrap();
        let root = installation(temp.path(), EEEX_INI.as_bytes());
        fs::write(root.join(GAME).join(LOG_FILE), b"last session").unwrap();
        fs::create_dir(root.join(GAME).join(PREVIOUS_LOG_FILE)).unwrap();

        let status = status(&root);
        assert_eq!(status.setting, GameLogSetting::Off);
        assert_eq!(status.detail, None);
        assert_eq!(
            status.log.path,
            root.join("game").join("InfinityLoader.log")
        );
        assert_eq!(status.log.state, GameLogFileState::Present);
        assert_eq!(status.log.size_bytes, Some(12));
        assert!(status.log.modified_at_millis.is_some());
        assert_eq!(
            status.previous_log.path,
            root.join("game").join("InfinityLoader-previous.log")
        );
        assert_eq!(status.previous_log.state, GameLogFileState::NotAFile);
        assert_eq!(
            status.crash_folder,
            root.join("game").join("InfinityLoader_Crash")
        );
        assert_eq!(status.crash_reports, Some(0));

        fs::remove_dir(root.join(GAME).join(PREVIOUS_LOG_FILE)).unwrap();
        fs::write(root.join(GAME).join(INI), on(EEEX_INI)).unwrap();
        let status = super::status(&root);
        assert_eq!(status.setting, GameLogSetting::On);
        assert_eq!(status.previous_log.state, GameLogFileState::Missing);

        fs::remove_file(root.join(GAME).join(INI)).unwrap();
        let status = super::status(&root);
        assert_eq!(status.setting, GameLogSetting::Unavailable);
        assert_eq!(
            status.detail.as_deref(),
            Some("This installation has no InfinityLoader.ini.")
        );

        let status = super::status(&temp.path().join("absent"));
        assert_eq!(status.setting, GameLogSetting::Unavailable);
        assert_eq!(status.log.state, GameLogFileState::Unreadable);
        assert_eq!(status.crash_reports, None);
    }

    #[test]
    fn the_switch_changes_only_the_log_setting_and_only_when_needed() {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app");
        let root = installation(temp.path(), EEEX_INI.as_bytes());
        let none = Programs(vec![]);

        let status = set_with(&app, &root, true, &none).unwrap();
        assert_eq!(status.setting, GameLogSetting::On);
        assert_eq!(ini_of(&root), on(EEEX_INI).as_bytes());
        // Nothing is written when the setting is already as requested.
        let identity = same_file::Handle::from_path(root.join(GAME).join(INI)).unwrap();
        assert_eq!(
            set_with(&app, &root, true, &none).unwrap().setting,
            GameLogSetting::On
        );
        assert_eq!(
            same_file::Handle::from_path(root.join(GAME).join(INI)).unwrap(),
            identity
        );
        // A file that is open cannot be replaced.
        drop(identity);

        let status = set_with(&app, &root, false, &none).unwrap();
        assert_eq!(status.setting, GameLogSetting::Off);
        assert_eq!(ini_of(&root), EEEX_INI.as_bytes());
        assert_eq!(entries(&root.join(GAME)), [INI]);

        // A log set outside CEBG counts as on, and turning the log off blanks it.
        let custom = EEEX_INI.replacen("LogFile=", "LogFile=mine.log", 1);
        fs::write(root.join(GAME).join(INI), &custom).unwrap();
        let status = set_with(&app, &root, true, &none).unwrap();
        assert_eq!(status.setting, GameLogSetting::Other);
        assert_eq!(status.detail.as_deref(), Some("mine.log"));
        assert_eq!(ini_of(&root), custom.as_bytes());
        set_with(&app, &root, false, &none).unwrap();
        assert_eq!(ini_of(&root), EEEX_INI.as_bytes());
    }

    #[test]
    fn the_switch_refuses_while_the_game_runs_or_the_installation_is_busy() {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app");
        let root = installation(temp.path(), EEEX_INI.as_bytes());
        let game = root.join(GAME).join("Baldur.exe");
        fs::write(&game, b"game").unwrap();
        let elsewhere = temp.path().join("other.exe");
        fs::write(&elsewhere, b"other").unwrap();

        let error =
            set_with(&app, &root, true, &Programs(vec![elsewhere.clone(), game])).unwrap_err();
        assert!(
            error.starts_with("Close the game and other programs from this installation first"),
            "{error}"
        );
        assert!(error.ends_with("Baldur.exe"), "{error}");
        set_with(&app, &root, true, &Programs(vec![elsewhere])).unwrap();
        assert_eq!(ini_of(&root), on(EEEX_INI).as_bytes());

        let lock =
            TargetLock::try_acquire(crate::lock::shared_target_lock_root(&app), &root).unwrap();
        let error = set_with(&app, &root, false, &Programs(vec![])).unwrap_err();
        assert!(error.starts_with("This installation is in use"), "{error}");
        drop(lock);

        // A whole-mod update that did not finish must be restored first.
        let record = root.join(".chriz/mod-upgrades/v1/00000000000000000001");
        fs::create_dir_all(&record).unwrap();
        fs::write(record.join("applied.pending"), b"partial completion").unwrap();
        assert!(set_with(&app, &root, false, &Programs(vec![])).is_err());
        assert_eq!(ini_of(&root), on(EEEX_INI).as_bytes());
    }

    #[test]
    fn the_switch_refuses_an_ini_it_cannot_change() {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app");
        let root = installation(temp.path(), b"[Other]\nLogFile=\n");
        let none = Programs(vec![]);

        let error = set_with(&app, &root, true, &none).unwrap_err();
        assert_eq!(error, "InfinityLoader.ini has no [General] section.");
        assert_eq!(ini_of(&root), b"[Other]\nLogFile=\n");

        fs::remove_file(root.join(GAME).join(INI)).unwrap();
        fs::create_dir(root.join(GAME).join(INI)).unwrap();
        let error = set_with(&app, &root, true, &none).unwrap_err();
        assert!(error.contains("unexpected entry"), "{error}");
        assert_eq!(entries(&root.join(GAME).join(INI)), Vec::<String>::new());
    }

    /// The lines that differ between `a` and `b`: those of each between the lines both start and
    /// end with.
    fn differing_lines<'a>(a: &'a [u8], b: &'a [u8]) -> (Vec<&'a [u8]>, Vec<&'a [u8]>) {
        let a: Vec<_> = a.split_inclusive(|byte| *byte == b'\n').collect();
        let b: Vec<_> = b.split_inclusive(|byte| *byte == b'\n').collect();
        let start = a.iter().zip(&b).take_while(|(a, b)| a == b).count();
        let end = a[start..]
            .iter()
            .rev()
            .zip(b[start..].iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        (
            a[start..a.len() - end].to_vec(),
            b[start..b.len() - end].to_vec(),
        )
    }

    /// Real loader settings, such as those EEex installs or those of a played game: switching
    /// the log and back changes their `LogFile` line alone, and gives the same bytes again when
    /// that line holds a value CEBG writes. Reads the files named in `CEBG_LOADER_INI` and
    /// switches copies of them only.
    #[test]
    #[ignore = "requires CEBG_LOADER_INI naming real InfinityLoader.ini files; reads them only"]
    fn real_loader_settings_change_only_in_their_log_file_line() {
        let paths = std::env::var_os("CEBG_LOADER_INI").expect("set CEBG_LOADER_INI");
        let none = Programs(vec![]);
        for path in std::env::split_paths(&paths) {
            let name = path.display();
            let original = fs::read(&path).unwrap();
            let parsed = LoaderIni::parse(&original);
            let (setting, detail) = parsed.setting();
            assert!(
                matches!(setting, GameLogSetting::Off | GameLogSetting::On),
                "{name}: {setting:?} {detail:?}"
            );
            let cebg_value = parsed.log_file.as_ref().is_some_and(|line| {
                let value = &original[line.value.clone()];
                value.is_empty() || value == LOG_FILE.as_bytes()
            });
            let recorded = sha256_bytes(&original);

            let temp = tempfile::tempdir().unwrap();
            let app = temp.path().join("app");
            let root = installation(temp.path(), &original);
            // Switched to the other setting, then back.
            for enabled in [
                setting == GameLogSetting::Off,
                setting == GameLogSetting::On,
            ] {
                let status = set_with(&app, &root, enabled, &none).unwrap();
                let expected = if enabled {
                    GameLogSetting::On
                } else {
                    GameLogSetting::Off
                };
                assert_eq!((status.setting, status.detail), (expected, None), "{name}");
                let changed = ini_of(&root);
                assert!(
                    unchanged_apart_from_the_setting(&changed, &recorded),
                    "{name}"
                );
                let (before, after) = differing_lines(&original, &changed);
                assert!(before.len() <= 1 && after.len() <= 1, "{name}");
                for line in before.into_iter().chain(after) {
                    let key = line.split(|byte| *byte == b'=').next().map(trim);
                    assert!(
                        key.is_some_and(|key| key.eq_ignore_ascii_case(b"LogFile")),
                        "{name}: {}",
                        String::from_utf8_lossy(line)
                    );
                }
            }
            if cebg_value {
                assert_eq!(ini_of(&root), original, "{name}");
            }
            assert_eq!(entries(&root.join(GAME)), [INI], "{name}");
        }
    }

    #[test]
    fn the_last_log_is_kept_before_the_game_starts() {
        let temp = tempfile::tempdir().unwrap();
        let root = installation(temp.path(), on(EEEX_INI).as_bytes());
        let game = root.join(GAME);
        fs::write(game.join(LOG_FILE), b"session two").unwrap();
        fs::write(game.join(PREVIOUS_LOG_FILE), b"session one").unwrap();

        assert_eq!(
            keep_previous(&root),
            KeptLog::Kept {
                bytes: 11,
                shortened: false
            }
        );
        assert_eq!(
            fs::read(game.join(PREVIOUS_LOG_FILE)).unwrap(),
            b"session two"
        );
        assert_eq!(fs::read(game.join(LOG_FILE)).unwrap(), b"session two");
        assert_eq!(entries(&game), [PREVIOUS_LOG_FILE, INI, LOG_FILE]);
    }

    #[test]
    fn nothing_is_kept_without_the_log_or_a_last_session() {
        let temp = tempfile::tempdir().unwrap();
        let root = installation(temp.path(), EEEX_INI.as_bytes());
        let game = root.join(GAME);
        fs::write(game.join(LOG_FILE), b"from an earlier test").unwrap();
        fs::write(game.join(PREVIOUS_LOG_FILE), b"session one").unwrap();

        assert_eq!(keep_previous(&root), KeptLog::NotEnabled);
        let custom = EEEX_INI.replacen("LogFile=", "LogFile=InfinityLoader.txt", 1);
        fs::write(game.join(INI), custom).unwrap();
        assert_eq!(keep_previous(&root), KeptLog::NotEnabled);
        fs::remove_file(game.join(INI)).unwrap();
        assert_eq!(keep_previous(&root), KeptLog::NotEnabled);

        fs::write(game.join(INI), on(EEEX_INI)).unwrap();
        fs::write(game.join(LOG_FILE), b"").unwrap();
        assert_eq!(keep_previous(&root), KeptLog::NothingToKeep);
        fs::remove_file(game.join(LOG_FILE)).unwrap();
        assert_eq!(keep_previous(&root), KeptLog::NothingToKeep);
        assert_eq!(
            fs::read(game.join(PREVIOUS_LOG_FILE)).unwrap(),
            b"session one"
        );

        // Settings the loader cannot read leave its log off, as does a missing game folder.
        fs::remove_file(game.join(INI)).unwrap();
        fs::create_dir(game.join(INI)).unwrap();
        assert_eq!(keep_previous(&root), KeptLog::NotEnabled);
        assert_eq!(
            keep_previous(&temp.path().join("absent")),
            KeptLog::NotEnabled
        );
    }

    #[test]
    fn only_the_end_of_a_very_long_log_is_kept() {
        let temp = tempfile::tempdir().unwrap();
        let root = installation(temp.path(), on(EEEX_INI).as_bytes());
        let game = root.join(GAME);
        fs::write(game.join(LOG_FILE), b"0123456789abcdefghij").unwrap();

        let held = HeldFolder::open(&root, [GAME]).unwrap();
        let expected = "[CEBG] This log had 20 bytes; only its last 8 bytes were kept.\r\ncdefghij";
        assert_eq!(
            keep_in(&held, 8).unwrap(),
            KeptLog::Kept {
                bytes: expected.len() as u64,
                shortened: true
            }
        );
        assert_eq!(
            fs::read_to_string(game.join(PREVIOUS_LOG_FILE)).unwrap(),
            expected
        );
        // At the limit, the whole log is kept.
        assert_eq!(
            keep_in(&held, 20).unwrap(),
            KeptLog::Kept {
                bytes: 20,
                shortened: false
            }
        );
    }

    /// The running game holds its log open for writing and lets others only read it.
    #[cfg(windows)]
    #[test]
    fn a_log_in_use_is_neither_kept_nor_changed() {
        use std::io::Write;
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

        let temp = tempfile::tempdir().unwrap();
        let root = installation(temp.path(), on(EEEX_INI).as_bytes());
        let game = root.join(GAME);
        fs::write(game.join(PREVIOUS_LOG_FILE), b"session one").unwrap();
        let mut running = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .share_mode(FILE_SHARE_READ)
            .open(game.join(LOG_FILE))
            .unwrap();
        running.write_all(b"session two").unwrap();

        let KeptLog::Failed { reason } = keep_previous(&root) else {
            panic!("a log in use was kept");
        };
        assert!(reason.contains("in use"), "{reason}");
        drop(running);

        assert_eq!(
            fs::read(game.join(PREVIOUS_LOG_FILE)).unwrap(),
            b"session one"
        );
        assert_eq!(entries(&game), [PREVIOUS_LOG_FILE, INI, LOG_FILE]);
    }

    /// The facts about the log need no access to its content: they are read even while it is
    /// open without any sharing, so reading them never keeps a starting game from opening it.
    #[cfg(windows)]
    #[test]
    fn the_status_never_opens_the_log_for_reading() {
        use std::io::Write;
        use std::os::windows::fs::OpenOptionsExt;

        let temp = tempfile::tempdir().unwrap();
        let root = installation(temp.path(), on(EEEX_INI).as_bytes());
        let mut exclusive = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .share_mode(0)
            .open(root.join(GAME).join(LOG_FILE))
            .unwrap();
        exclusive.write_all(b"session two").unwrap();
        exclusive.flush().unwrap();

        let status = status(&root);
        assert_eq!(status.setting, GameLogSetting::On);
        assert_eq!(status.log.state, GameLogFileState::Present);
        assert_eq!(status.log.size_bytes, Some(11));
    }

    /// A starting loader may save its settings, and fails to start if it cannot: Windows refuses
    /// that while another program reads the file without sharing it for writing. The status reads
    /// the setting the way such a writer allows, here while one has the file open.
    #[cfg(windows)]
    #[test]
    fn the_status_reads_the_setting_while_the_loader_writes_it() {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};

        let temp = tempfile::tempdir().unwrap();
        let root = installation(temp.path(), on(EEEX_INI).as_bytes());
        let writing = fs::OpenOptions::new()
            .write(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .open(root.join(GAME).join(INI))
            .unwrap();
        let status = status(&root);
        assert_eq!((status.setting, status.detail), (GameLogSetting::On, None));
        drop(writing);
    }

    /// The game folder and the ini as junctions: neither is followed, and what they point to is
    /// never touched.
    #[cfg(windows)]
    #[test]
    fn the_switch_never_writes_through_a_junction() {
        use crate::held_folder::tests::junction;

        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app");
        let outside = installation(&temp.path().join("outside"), EEEX_INI.as_bytes());
        let none = Programs(vec![]);

        let linked_game = temp.path().join("linked-game");
        fs::create_dir(&linked_game).unwrap();
        junction(&linked_game.join(GAME), &outside.join(GAME));
        let error = set_with(&app, &linked_game, true, &none).unwrap_err();
        assert!(error.contains("links and reparse points"), "{error}");

        let linked_ini = temp.path().join("linked-ini");
        fs::create_dir_all(linked_ini.join(GAME)).unwrap();
        junction(&linked_ini.join(GAME).join(INI), &outside.join(GAME));
        let error = set_with(&app, &linked_ini, true, &none).unwrap_err();
        assert!(error.contains("unexpected entry"), "{error}");
        let status = status(&linked_ini);
        assert_eq!(status.setting, GameLogSetting::Unavailable);
        assert!(
            status
                .detail
                .as_deref()
                .unwrap()
                .contains("unexpected entry"),
            "{status:?}"
        );

        assert_eq!(ini_of(&outside), EEEX_INI.as_bytes());
        assert_eq!(entries(&outside.join(GAME)), [INI]);
    }

    /// No log is kept from a game folder CEBG cannot hold, such as a junction. The loader follows
    /// it, so that is reported only when the settings there turn the log on.
    #[cfg(windows)]
    #[test]
    fn a_log_behind_a_link_is_reported_only_when_the_game_writes_it() {
        use crate::held_folder::tests::junction;

        let temp = tempfile::tempdir().unwrap();
        let outside = installation(&temp.path().join("outside"), EEEX_INI.as_bytes());
        fs::write(outside.join(GAME).join(LOG_FILE), b"last session").unwrap();
        let linked = temp.path().join("linked");
        fs::create_dir(&linked).unwrap();
        junction(&linked.join(GAME), &outside.join(GAME));

        assert_eq!(keep_previous(&linked), KeptLog::NotEnabled);
        fs::write(outside.join(GAME).join(INI), on(EEEX_INI)).unwrap();
        assert_eq!(
            keep_previous(&linked),
            KeptLog::Failed {
                reason: format!(
                    "A link or another unexpected entry is in the way: {}",
                    linked.join(GAME).display()
                )
            }
        );
        assert_eq!(entries(&outside.join(GAME)), [INI, LOG_FILE]);
    }

    /// Re-pointing a link above the managed root after the game folder was opened redirects every
    /// path through it. The setting and the kept log must still change in the folder that was
    /// opened.
    #[cfg(windows)]
    #[test]
    fn the_held_game_folder_is_changed_after_a_link_above_is_repointed() {
        use crate::held_folder::tests::junction;

        let temp = tempfile::tempdir().unwrap();
        let checked = installation(&temp.path().join("checked"), EEEX_INI.as_bytes());
        fs::write(checked.join(GAME).join(LOG_FILE), b"checked session").unwrap();
        let other = installation(&temp.path().join("other"), EEEX_INI.as_bytes());
        let link = temp.path().join("link");
        junction(&link, &temp.path().join("checked"));

        let held = HeldFolder::open(&link.join("managed"), [GAME]).unwrap();
        fs::remove_dir(&link).unwrap();
        junction(&link, &temp.path().join("other"));
        set_in(&held, true).unwrap();
        assert_eq!(
            keep_in(&held, KEEP_LIMIT).unwrap(),
            KeptLog::Kept {
                bytes: 15,
                shortened: false
            }
        );
        drop(held);

        assert_eq!(ini_of(&checked), on(EEEX_INI).as_bytes());
        assert_eq!(
            fs::read(checked.join(GAME).join(PREVIOUS_LOG_FILE)).unwrap(),
            b"checked session"
        );
        assert_eq!(ini_of(&other), EEEX_INI.as_bytes());
        assert_eq!(entries(&other.join(GAME)), [INI]);
    }
}
