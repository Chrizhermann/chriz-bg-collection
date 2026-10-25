//! The game's own log for crash reports. A player turns it on or off per installation, outside
//! the normal flow, and sees where the log, the log of the session before and the crash dumps
//! are. Before each launch the log of the last session is kept; a failure is reported but never
//! prevents playing.

use bg_engine::game_log::{GameLogFile, GameLogFileState, GameLogSetting, GameLogStatus, KeptLog};
use serde::Serialize;

use super::{display_windows_path, CommandError, NativeBridge};

fn game_log_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "game_log_failed",
        "The game log setting wasn't changed.",
        "Close the game and wait for any update of this installation to finish, then try again.",
        error.to_string(),
    )
}

/// Paths in engine messages without the `\\?\` prefix of canonical Windows paths.
fn display_detail(detail: Option<String>) -> Option<String> {
    detail.map(|detail| detail.replace(r"\\?\UNC\", r"\\").replace(r"\\?\", ""))
}

/// Whether the game writes its log for crash reports, and where that evidence is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLogStatusResponse {
    pub install_id: String,
    pub setting: GameLogSetting,
    /// The log file set outside CEBG, or why the setting cannot be changed.
    pub detail: Option<String>,
    pub log: GameLogFileResponse,
    pub previous_log: GameLogFileResponse,
    pub crash_folder: String,
    /// Crashes with a report in the crash folder; `None` when it cannot be looked into.
    pub crash_reports: Option<usize>,
}

impl GameLogStatusResponse {
    fn new(install_id: &str, status: GameLogStatus) -> Result<Self, CommandError> {
        Ok(Self {
            install_id: install_id.to_owned(),
            setting: status.setting,
            detail: display_detail(status.detail),
            log: GameLogFileResponse::new(status.log)?,
            previous_log: GameLogFileResponse::new(status.previous_log)?,
            crash_folder: display_windows_path(&status.crash_folder)?,
            crash_reports: status.crash_reports,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLogFileResponse {
    pub path: String,
    pub state: GameLogFileState,
    /// Decimal byte counts remain strings across the native boundary.
    pub size_bytes: Option<String>,
    pub modified_at_millis: Option<u64>,
    pub detail: Option<String>,
}

impl GameLogFileResponse {
    fn new(file: GameLogFile) -> Result<Self, CommandError> {
        Ok(Self {
            path: display_windows_path(&file.path)?,
            state: file.state,
            size_bytes: file.size_bytes.map(|bytes| bytes.to_string()),
            modified_at_millis: file.modified_at_millis,
            detail: display_detail(file.detail),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameLogLaunchState {
    /// The game log is off or set outside CEBG.
    NotEnabled,
    NothingToKeep,
    Kept,
    Failed,
}

/// What became of the last session's game log before the launch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLogLaunchResponse {
    pub state: GameLogLaunchState,
    /// Why the log could not be kept.
    pub technical_detail: Option<String>,
}

impl GameLogLaunchResponse {
    pub(super) fn from_kept(kept: KeptLog) -> Self {
        let (state, technical_detail) = match kept {
            KeptLog::NotEnabled => (GameLogLaunchState::NotEnabled, None),
            KeptLog::NothingToKeep => (GameLogLaunchState::NothingToKeep, None),
            KeptLog::Kept { .. } => (GameLogLaunchState::Kept, None),
            KeptLog::Failed { reason } => (GameLogLaunchState::Failed, Some(reason)),
        };
        Self {
            state,
            technical_detail: display_detail(technical_detail),
        }
    }
}

impl NativeBridge {
    /// The game-log setting of one available installation and where its evidence is. Reads only.
    pub fn game_log_status(&self, install_id: &str) -> Result<GameLogStatusResponse, CommandError> {
        let record = self.available_managed_install(install_id)?.record;
        GameLogStatusResponse::new(
            install_id,
            bg_engine::game_log::status(&record.managed_root),
        )
    }

    /// Turns the game log of one available installation on or off for its next start. Refused
    /// while the game runs, and while any build, update or removal runs.
    pub fn set_game_log(
        &self,
        install_id: &str,
        enabled: bool,
    ) -> Result<GameLogStatusResponse, CommandError> {
        let _guard = self.begin_app_update()?;
        let record = self.available_managed_install(install_id)?.record;
        let locks = self
            .target_locks
            .as_ref()
            .ok_or_else(|| game_log_error("Application data is unavailable."))?;
        let status = bg_engine::game_log::set(locks, &record.managed_root, enabled)
            .map_err(game_log_error)?;
        GameLogStatusResponse::new(install_id, status)
    }
}
