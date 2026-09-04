//! Stable failures crossing the native-command boundary.

use bg_engine::cli::CliError;
use serde::Serialize;

/// One recoverable command failure rendered by the desktop shell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandError {
    /// Stable machine-readable category.
    pub code: String,
    /// Short player-facing explanation.
    pub message: String,
    /// Concrete next action offered by the UI.
    pub recovery_action: String,
    /// Detailed local diagnostic text.
    pub technical_detail: String,
}

impl CommandError {
    /// Builds an explicit command-boundary failure.
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        recovery_action: impl Into<String>,
        technical_detail: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recovery_action: recovery_action.into(),
            technical_detail: technical_detail.into(),
        }
    }

    /// Converts the engine CLI's stable category without leaking an unstructured rejection.
    pub fn from_cli(error: CliError) -> Self {
        let code = error.code();
        let (message, recovery_action) = match code {
            "recipe_load_failed" | "validation_failed" => (
                "The installer recipe could not be verified.",
                "Repair or replace the installer recipe, then try again.",
            ),
            "game_profiles_failed" => (
                "Verified game detection data is unavailable.",
                "Install a release that includes verified game profiles.",
            ),
            "game_discovery_failed" => (
                "The installed games could not be detected safely.",
                "Retry detection or choose a game folder manually.",
            ),
            "game_inspection_failed" | "ambiguous_storefront" => (
                "That folder could not be accepted as a clean supported game.",
                "Choose a different clean game folder and inspect it again.",
            ),
            "invalid_selection" => (
                "The selected collection options are not compatible.",
                "Review the unavailable options and try again.",
            ),
            "campaign_error" | "recipe_freeze_failed" => (
                "The installation could not finish.",
                "See the technical log for the cause. Retry if available, or return to setup.",
            ),
            "unsafe_target" => (
                "The install folder could not be used.",
                "Return to setup and check the install location. See the technical log for details.",
            ),
            _ => (
                "The installer could not complete that check.",
                "Retry the check. If it still fails, keep the technical detail for diagnosis.",
            ),
        };
        let technical_detail = error.to_string();
        Self::new(code, message, recovery_action, technical_detail)
    }

    pub(crate) fn background_task(error: impl std::fmt::Display) -> Self {
        Self::new(
            "background_task_failed",
            "The installer could not complete that background check.",
            "Retry the check. If it still fails, keep the technical detail for diagnosis.",
            error.to_string(),
        )
    }
}
