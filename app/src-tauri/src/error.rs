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
        if code.starts_with("manual_checkpoint") || code == "invalid_manual_breakpoint" {
            return Self::new(code, error.to_string(), "Close the game and manual installers. Keep all WeiDU.log rows; correct the issue or restore your checkpoint backup, then check again.", error.to_string());
        }
        let app = crate::flavor::CURRENT.short_name;
        let (message, recovery_action): (String, String) = match code {
            "folder_access_denied" => (
                "Windows blocked access to an installation or save folder.".to_owned(),
                format!("Check the folder in Error details and its Windows permissions. If Windows Security blocked {app}, allow this app rather than turning protection off."),
            ),
            "folder_unavailable" => (
                "A folder needed for installation or saves is unavailable.".to_owned(),
                "Check the exact path in Error details. Reconnect its drive or make a redirected folder available, then retry if offered.".to_owned(),
            ),
            "folder_in_use" => (
                format!("Another program is using a folder or file {app} needs."),
                "Close the game and any other installer using this copy, then retry if offered.".to_owned(),
            ),
            "folder_conflict" => (
                "The installation or save folder conflicts with existing files.".to_owned(),
                "Keep existing files. Check Error details and choose a new empty installation folder if needed.".to_owned(),
            ),
            "folder_failed" => (
                format!("{app} could not prepare an installation or save folder."),
                "Check the operation and path in Error details. Export diagnostics if it keeps failing.".to_owned(),
            ),
            "recipe_load_failed" | "validation_failed" => (
                "The installer recipe could not be verified.".to_owned(),
                "Repair or replace the installer recipe, then try again.".to_owned(),
            ),
            "game_profiles_failed" => (
                "Verified game detection data is unavailable.".to_owned(),
                "Install a release that includes verified game profiles.".to_owned(),
            ),
            "game_discovery_failed" => (
                "The installed games could not be detected safely.".to_owned(),
                "Retry detection or choose a game folder manually.".to_owned(),
            ),
            "game_inspection_failed" | "ambiguous_storefront" => (
                "That folder could not be accepted as a clean supported game.".to_owned(),
                "Choose a different clean game folder and inspect it again.".to_owned(),
            ),
            "invalid_selection" => (
                "The selected collection options are not compatible.".to_owned(),
                "Review the unavailable options and try again.".to_owned(),
            ),
            "installer_launch_unsupported" => (
                format!("This {app} release cannot start one of the selected mod installers."),
                format!("Install a corrected {app} release, then retry this installation in the same folder. Keep the existing installation files."),
            ),
            "campaign_error" | "campaign_failed" | "recipe_freeze_failed" => (
                "The installation could not finish.".to_owned(),
                "See the technical log for the cause. Retry if available, or return to setup.".to_owned(),
            ),
            "unsafe_target" => (
                "The install folder could not be used.".to_owned(),
                "Return to setup and check the install location. See the technical log for details.".to_owned(),
            ),
            _ => (
                "The installer could not complete that check.".to_owned(),
                "Retry the check. If it still fails, keep the technical detail for diagnosis.".to_owned(),
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

#[cfg(test)]
mod folder_error_tests {
    use super::*;
    use bg_engine::cli::{CampaignReport, CampaignStatus};

    #[test]
    fn unsupported_installer_requires_a_corrected_app_and_preserves_the_current_copy() {
        let technical = "explicit-relative-TP2 invocation is unavailable until the signed recipe schema records separately reviewed compatibility evidence; use setup-name mode";
        let error = CommandError::from_cli(CliError::from_campaign(CampaignReport {
            install_id: "fixture".into(),
            managed_root: "C:/Fixture/Managed".into(),
            plan_sha256: "11".repeat(32),
            status: CampaignStatus::Failed {
                step_id: "install:game-interface-infinity-bg2".into(),
                reason: technical.into(),
            },
        }));
        // Errors name the compiled app: CEBG, or Chriz Easy IWD in the Icewind Dale build.
        let app = crate::flavor::CURRENT.short_name;
        assert_eq!(error.code, "installer_launch_unsupported");
        assert_eq!(
            error.message,
            format!("This {app} release cannot start one of the selected mod installers.")
        );
        assert!(error
            .recovery_action
            .contains(&format!("corrected {app} release")));
        assert!(error.recovery_action.contains("same folder"));
        assert!(!error.recovery_action.contains("Retry the check"));
        assert_eq!(error.technical_detail, technical);
    }

    #[test]
    fn contextual_folder_codes_keep_diagnostics_and_give_specific_recovery() {
        for (code, expected) in [
            ("folder_access_denied", "Windows blocked"),
            ("folder_unavailable", "unavailable"),
            ("folder_in_use", "Another program"),
            ("folder_conflict", "conflicts"),
            ("folder_failed", "could not prepare"),
        ] {
            let reason =
                format!("[{code}] create save folder C:\\Fixture\\Documents: OS error evidence");
            let error = CommandError::from_cli(CliError::from_campaign(CampaignReport {
                install_id: "fixture".into(),
                managed_root: "C:/Fixture/Managed".into(),
                plan_sha256: "11".repeat(32),
                status: CampaignStatus::Failed {
                    step_id: "stage:bg1".into(),
                    reason: reason.clone(),
                },
            }));
            assert_eq!(error.code, code);
            assert!(error.message.contains(expected));
            assert!(error.technical_detail.contains(&reason));
            assert!(!error.recovery_action.contains("run as administrator"));
            assert!(!error.message.contains("could not complete that check"));
        }
    }

    #[test]
    fn unclassified_message_cannot_accidentally_acquire_a_folder_code() {
        let error = CommandError::from_cli(CliError::from_campaign(CampaignReport {
            install_id: "fixture".into(),
            managed_root: "C:/Fixture".into(),
            plan_sha256: "11".repeat(32),
            status: CampaignStatus::Failed {
                step_id: "install:fixture".into(),
                reason: "a mod mentions folder_access_denied in its own output".into(),
            },
        }));
        assert_eq!(error.code, "campaign_failed");
        assert_eq!(error.message, "The installation could not finish.");
    }
}
