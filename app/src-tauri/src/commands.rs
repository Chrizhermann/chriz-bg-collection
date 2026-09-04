//! Narrow Tauri command surface. Synchronous engine work always leaves the UI thread.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bg_engine::games::GameRole;
use bg_engine::recipe_view::NormalizedSelection;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::bridge::{
    BootstrapResponse, DesktopShortcutResponse, DestinationEvaluationResponse,
    DiagnosticsExportResponse, EvaluateBuildResponse, FrozenReviewResponse, GameCandidateResponse,
    GameDiscoveryResponse, InstallationDefaultsResponse, ManagedInstallationResponse,
    ManualArchiveResponse, NativeBridge, RunEventEnvelope, RunSnapshotResponse, StartBuildResponse,
};
use crate::error::CommandError;
use crate::updates::UpdateCenterResponse;

/// Managed bridge state shared by native commands.
#[derive(Clone)]
pub struct BridgeState {
    bridge: Arc<Mutex<NativeBridge>>,
    application_update: Arc<Mutex<Option<Update>>>,
}

impl BridgeState {
    /// Creates state for one immutable bridge configuration.
    pub fn new(bridge: NativeBridge) -> Self {
        Self {
            bridge: Arc::new(Mutex::new(bridge)),
            application_update: Arc::new(Mutex::new(None)),
        }
    }
}

async fn background<T, F>(operation: F) -> Result<T, CommandError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, CommandError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(CommandError::background_task)?
}

fn local_path(selected: Option<FilePath>) -> Result<Option<PathBuf>, CommandError> {
    selected
        .map(|path| {
            path.simplified().into_path().map_err(|error| {
                CommandError::new(
                    "local_choice_invalid",
                    "The selected item could not be read as a local Windows path.",
                    "Choose a local file or folder and try again.",
                    error.to_string(),
                )
            })
        })
        .transpose()
}

#[tauri::command]
pub async fn bootstrap(state: State<'_, BridgeState>) -> Result<BootstrapResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.bootstrap()).await
}

#[tauri::command]
pub fn installation_defaults(app: AppHandle) -> Result<InstallationDefaultsResponse, CommandError> {
    let home = app.path().home_dir().map_err(|error| {
        CommandError::new(
            "home_path_unavailable",
            "The default installation location could not be found.",
            "Choose an installation location manually.",
            error.to_string(),
        )
    })?;
    crate::bridge::installation_defaults(&home)
}

#[tauri::command]
pub async fn select_profile(
    state: State<'_, BridgeState>,
    profile_id: String,
) -> Result<BootstrapResponse, CommandError> {
    let shared = Arc::clone(&state.bridge);
    background(move || {
        let mut active = shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let selected = active.select_profile(&profile_id)?;
        let summary = selected.bootstrap()?;
        *active = selected;
        Ok(summary)
    })
    .await
}

#[tauri::command]
pub async fn discover_games(
    state: State<'_, BridgeState>,
) -> Result<GameDiscoveryResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.discover_games()).await
}

#[tauri::command]
pub async fn choose_game_folder(
    app: AppHandle,
    state: State<'_, BridgeState>,
    role: GameRole,
) -> Result<Option<GameCandidateResponse>, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || {
        let title = match role {
            GameRole::BgeeSod => "Choose Baldur's Gate: Enhanced Edition with SoD",
            GameRole::Bg2ee => "Choose Baldur's Gate II: Enhanced Edition",
        };
        let selected = local_path(app.dialog().file().set_title(title).blocking_pick_folder())?;
        bridge.choose_game_folder(role, selected)
    })
    .await
}

#[tauri::command]
pub async fn inspect_game_path(
    state: State<'_, BridgeState>,
    role: GameRole,
    path: String,
) -> Result<GameCandidateResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.inspect_game_path(role, &PathBuf::from(path))).await
}

#[tauri::command]
pub async fn evaluate_build(
    state: State<'_, BridgeState>,
    selection: NormalizedSelection,
) -> Result<EvaluateBuildResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.evaluate_build(&selection)).await
}

#[tauri::command]
pub async fn inspect_destination(
    state: State<'_, BridgeState>,
    path: String,
    bg1_candidate_id: String,
    bg2_candidate_id: String,
) -> Result<DestinationEvaluationResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || {
        bridge.inspect_destination(&PathBuf::from(path), &bg1_candidate_id, &bg2_candidate_id)
    })
    .await
}

#[tauri::command]
pub async fn choose_destination_folder(
    app: AppHandle,
    state: State<'_, BridgeState>,
    bg1_candidate_id: String,
    bg2_candidate_id: String,
) -> Result<Option<DestinationEvaluationResponse>, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || {
        let selected = local_path(
            app.dialog()
                .file()
                .set_title("Choose an install location")
                .blocking_pick_folder(),
        )?;
        bridge.choose_destination_folder(selected, &bg1_candidate_id, &bg2_candidate_id)
    })
    .await
}

#[tauri::command]
pub async fn supply_manual_archive(
    app: AppHandle,
    state: State<'_, BridgeState>,
    artifact_id: String,
) -> Result<Option<ManualArchiveResponse>, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || {
        let selected = local_path(
            app.dialog()
                .file()
                .set_title(format!("Choose downloaded archive for {artifact_id}"))
                .blocking_pick_file(),
        )?;
        bridge.supply_manual_archive(&artifact_id, selected)
    })
    .await
}

#[tauri::command]
pub async fn open_manual_source(
    state: State<'_, BridgeState>,
    artifact_id: String,
) -> Result<(), CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.open_manual_source(&artifact_id)).await
}

#[tauri::command]
pub async fn list_managed_installations(
    state: State<'_, BridgeState>,
) -> Result<Vec<ManagedInstallationResponse>, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.list_managed_installations()).await
}

#[tauri::command]
pub async fn export_diagnostics(
    app: AppHandle,
    state: State<'_, BridgeState>,
    install_id: String,
) -> Result<Option<DiagnosticsExportResponse>, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || {
        let selected = local_path(
            app.dialog()
                .file()
                .set_title("Save installer diagnostics")
                .blocking_save_file(),
        )?;
        bridge.export_diagnostics(&install_id, selected)
    })
    .await
}

#[tauri::command]
pub async fn launch_install(
    state: State<'_, BridgeState>,
    install_id: String,
) -> Result<(), CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.launch_install(&install_id)).await
}

#[tauri::command]
pub async fn install_radar(
    state: State<'_, BridgeState>,
    install_id: String,
) -> Result<bg_engine::radar::RadarInstallResult, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.install_radar(&install_id)).await
}

#[tauri::command]
pub async fn open_install_folder(
    state: State<'_, BridgeState>,
    install_id: String,
) -> Result<(), CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.open_install_folder(&install_id)).await
}

#[tauri::command]
pub async fn create_desktop_shortcut(
    state: State<'_, BridgeState>,
    install_id: String,
) -> Result<DesktopShortcutResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.create_desktop_shortcut(&install_id)).await
}

#[tauri::command]
pub async fn check_updates(
    app: AppHandle,
    state: State<'_, BridgeState>,
) -> Result<UpdateCenterResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let mut summary =
        background(move || bridge.unconfigured_update_center(env!("CARGO_PKG_VERSION"))).await?;
    let radar_bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let radar_check = tauri::async_runtime::spawn_blocking(move || radar_bridge.radar_update());
    let updater = app
        .updater_builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(update_error)?;
    let candidate = updater.check().await;
    summary.checked_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .ok();
    summary.recipe.current_version = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .recipe_version()?
        .unwrap_or_else(|| "bundled".to_owned());
    summary.recipe.state = crate::updates::RecipeUpdateState::UpToDate;
    summary.recipe.detail = "The collection is included with your CEBG version.".to_owned();
    summary.recipe.disposition = "up-to-date".to_owned();
    match candidate {
        Ok(candidate) => {
            summary.network_state = "online".to_owned();
            summary.application.state = if candidate.is_some() {
                "available"
            } else {
                "up-to-date"
            }
            .to_owned();
            summary.application.detail = if candidate.is_some() {
                "A newer CEBG version is available."
            } else {
                "You're using the latest CEBG version."
            }
            .to_owned();
            if let Some(update) = &candidate {
                summary.application.available_version = Some(update.version.clone());
                summary.application.release_notes = update.body.clone();
                if let Some(recipe_version) = update
                    .raw_json
                    .get("recipe_version")
                    .and_then(|value| value.as_str())
                {
                    if recipe_version != summary.recipe.current_version {
                        summary.recipe.state = crate::updates::RecipeUpdateState::RequiresApp;
                        summary.recipe.available_version = Some(recipe_version.to_owned());
                        summary.recipe.disposition = "app-update-required".to_owned();
                        summary.recipe.detail = "Install the CEBG update to get this collection. Existing games remain on their recorded version.".to_owned();
                    }
                }
            }
            *state
                .application_update
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = candidate;
        }
        Err(error) => {
            summary.network_state = "offline".to_owned();
            summary.application.state = "offline".to_owned();
            summary.application.detail = format!("Could not check for CEBG updates: {error}");
            summary.recipe.state = crate::updates::RecipeUpdateState::Offline;
            summary.recipe.detail = "Collection updates could not be checked.".to_owned();
            *state
                .application_update
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
        }
    }
    summary.radar = radar_check.await.ok();
    for copy in &mut summary.managed_copies {
        if copy.state != "stale" {
            copy.state = if copy.installed_recipe_version.as_deref()
                == Some(summary.recipe.current_version.as_str())
            {
                "up-to-date"
            } else {
                "update-available"
            }
            .to_owned();
            copy.detail = if copy.state == "up-to-date" { "Built with this collection version." } else { "A newer setup can be installed separately. Your current game and saves stay unchanged." }.to_owned();
        }
    }
    Ok(summary)
}

#[tauri::command]
pub async fn install_app_update(
    state: State<'_, BridgeState>,
    version: String,
) -> Result<(), CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let update = state
        .application_update
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
        .filter(|update| update.version == version)
        .cloned()
        .ok_or_else(|| {
            CommandError::new(
                "update_not_checked",
                "Check for updates first.",
                "Check for updates, then select the available version.",
                "No checked application update matches the requested version.",
            )
        })?;
    let _guard = bridge.begin_app_update()?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(update_error)
}

fn update_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "application_update_failed",
        "The CEBG update could not be installed.",
        "Check your connection and try again.",
        error.to_string(),
    )
}

#[tauri::command]
pub async fn activate_recipe_update(
    state: State<'_, BridgeState>,
    version: String,
) -> Result<(), CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || {
        bridge.ensure_update_idle()?;
        Err(CommandError::new(
            "recipe_requires_app_update",
            "This collection is delivered with a CEBG update.",
            "Install the application update, then create a new installation.",
            format!("requested recipe version {version}; collection replacement requires the signed app package"),
        ))
    })
    .await
}

#[tauri::command]
pub async fn freeze_review(
    state: State<'_, BridgeState>,
    display_name: String,
    selection: NormalizedSelection,
    destination: String,
    bg1_candidate_id: String,
    bg2_candidate_id: String,
) -> Result<FrozenReviewResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || {
        bridge.freeze_review(
            &display_name,
            &selection,
            &PathBuf::from(destination),
            &bg1_candidate_id,
            &bg2_candidate_id,
        )
    })
    .await
}

#[tauri::command]
pub async fn start_build(
    state: State<'_, BridgeState>,
    review_token: String,
    on_event: Channel<RunEventEnvelope>,
) -> Result<StartBuildResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || {
        bridge.start_build(&review_token, move |event| {
            let _ = on_event.send(event);
        })
    })
    .await
}

#[tauri::command]
pub async fn resume_build(
    state: State<'_, BridgeState>,
    install_id: String,
    on_event: Channel<RunEventEnvelope>,
) -> Result<StartBuildResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || {
        bridge.resume_build(&install_id, move |event| {
            let _ = on_event.send(event);
        })
    })
    .await
}

#[tauri::command]
pub async fn get_run_snapshot(
    state: State<'_, BridgeState>,
    run_id: String,
) -> Result<RunSnapshotResponse, CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.get_run_snapshot(&run_id)).await
}

#[tauri::command]
pub async fn continue_waiting(
    state: State<'_, BridgeState>,
    run_id: String,
) -> Result<(), CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.continue_waiting(&run_id)).await
}

#[tauri::command]
pub async fn cancel_run(state: State<'_, BridgeState>, run_id: String) -> Result<(), CommandError> {
    let bridge = state
        .bridge
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    background(move || bridge.cancel_run(&run_id)).await
}
