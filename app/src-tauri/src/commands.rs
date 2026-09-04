//! Narrow Tauri command surface. Synchronous engine work always leaves the UI thread.

use std::path::PathBuf;

use bg_engine::games::GameRole;
use bg_engine::recipe_view::NormalizedSelection;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

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
    bridge: NativeBridge,
}

impl BridgeState {
    /// Creates state for one immutable bridge configuration.
    pub fn new(bridge: NativeBridge) -> Self {
        Self { bridge }
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
    let bridge = state.bridge.clone();
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
pub async fn discover_games(
    state: State<'_, BridgeState>,
) -> Result<GameDiscoveryResponse, CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.discover_games()).await
}

#[tauri::command]
pub async fn choose_game_folder(
    app: AppHandle,
    state: State<'_, BridgeState>,
    role: GameRole,
) -> Result<Option<GameCandidateResponse>, CommandError> {
    let bridge = state.bridge.clone();
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
    let bridge = state.bridge.clone();
    background(move || bridge.inspect_game_path(role, &PathBuf::from(path))).await
}

#[tauri::command]
pub async fn evaluate_build(
    state: State<'_, BridgeState>,
    selection: NormalizedSelection,
) -> Result<EvaluateBuildResponse, CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.evaluate_build(&selection)).await
}

#[tauri::command]
pub async fn inspect_destination(
    state: State<'_, BridgeState>,
    path: String,
    bg1_candidate_id: String,
    bg2_candidate_id: String,
) -> Result<DestinationEvaluationResponse, CommandError> {
    let bridge = state.bridge.clone();
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
    let bridge = state.bridge.clone();
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
    let bridge = state.bridge.clone();
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
    let bridge = state.bridge.clone();
    background(move || bridge.open_manual_source(&artifact_id)).await
}

#[tauri::command]
pub async fn list_managed_installations(
    state: State<'_, BridgeState>,
) -> Result<Vec<ManagedInstallationResponse>, CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.list_managed_installations()).await
}

#[tauri::command]
pub async fn export_diagnostics(
    app: AppHandle,
    state: State<'_, BridgeState>,
    install_id: String,
) -> Result<Option<DiagnosticsExportResponse>, CommandError> {
    let bridge = state.bridge.clone();
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
    let bridge = state.bridge.clone();
    background(move || bridge.launch_install(&install_id)).await
}

#[tauri::command]
pub async fn open_install_folder(
    state: State<'_, BridgeState>,
    install_id: String,
) -> Result<(), CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.open_install_folder(&install_id)).await
}

#[tauri::command]
pub async fn create_desktop_shortcut(
    state: State<'_, BridgeState>,
    install_id: String,
) -> Result<DesktopShortcutResponse, CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.create_desktop_shortcut(&install_id)).await
}

#[tauri::command]
pub async fn check_updates(
    state: State<'_, BridgeState>,
) -> Result<UpdateCenterResponse, CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.unconfigured_update_center(env!("CARGO_PKG_VERSION"))).await
}

#[tauri::command]
pub async fn install_app_update(
    state: State<'_, BridgeState>,
    version: String,
) -> Result<(), CommandError> {
    let bridge = state.bridge.clone();
    background(move || {
        bridge.ensure_update_idle()?;
        Err(CommandError::new(
            "app_update_unconfigured",
            "Application updates are not configured in this build.",
            "Keep using the current application until a signed release channel is published.",
            format!("requested application version {version}; release endpoint and public key are absent"),
        ))
    })
    .await
}

#[tauri::command]
pub async fn activate_recipe_update(
    state: State<'_, BridgeState>,
    version: String,
) -> Result<(), CommandError> {
    let bridge = state.bridge.clone();
    background(move || {
        bridge.ensure_update_idle()?;
        Err(CommandError::new(
            "recipe_update_unconfigured",
            "Recipe updates are not configured in this build.",
            "Keep using the bundled recipe until a signed release channel is published.",
            format!("requested recipe version {version}; no verified candidate is staged"),
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
    let bridge = state.bridge.clone();
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
    let bridge = state.bridge.clone();
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
    let bridge = state.bridge.clone();
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
    let bridge = state.bridge.clone();
    background(move || bridge.get_run_snapshot(&run_id)).await
}

#[tauri::command]
pub async fn continue_waiting(
    state: State<'_, BridgeState>,
    run_id: String,
) -> Result<(), CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.continue_waiting(&run_id)).await
}

#[tauri::command]
pub async fn cancel_run(state: State<'_, BridgeState>, run_id: String) -> Result<(), CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.cancel_run(&run_id)).await
}
