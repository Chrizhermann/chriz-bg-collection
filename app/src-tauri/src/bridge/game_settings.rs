//! Installer-owned game settings that a completed installation may still lack, such
//! as EET's installation state that Lefreut's interfaces read. They are brought up to
//! date before each launch; a failure is reported but never prevents playing.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameSettingsState {
    Unchanged,
    Updated,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSettingsResponse {
    pub state: GameSettingsState,
    pub technical_detail: Option<String>,
}

impl GameSettingsResponse {
    pub(super) fn from_repair(result: Result<bool, String>) -> Self {
        match result {
            Ok(changed) => Self {
                state: if changed {
                    GameSettingsState::Updated
                } else {
                    GameSettingsState::Unchanged
                },
                technical_detail: None,
            },
            Err(error) => Self {
                state: GameSettingsState::Failed,
                technical_detail: Some(error),
            },
        }
    }
}
