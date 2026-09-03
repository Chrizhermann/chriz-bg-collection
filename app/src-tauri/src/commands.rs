//! Narrow Tauri command surface. Synchronous engine work always leaves the UI thread.

use std::path::PathBuf;

use bg_engine::games::GameRole;
use bg_engine::recipe_view::NormalizedSelection;
use tauri::ipc::Channel;
use tauri::State;

use crate::bridge::{
    BootstrapResponse, DestinationEvaluationResponse, EvaluateBuildResponse, FrozenReviewResponse,
    GameCandidateResponse, GameDiscoveryResponse, NativeBridge, RunEventEnvelope,
    RunSnapshotResponse, StartBuildResponse,
};
use crate::error::CommandError;

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

#[tauri::command]
pub async fn bootstrap(state: State<'_, BridgeState>) -> Result<BootstrapResponse, CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.bootstrap()).await
}

#[tauri::command]
pub async fn discover_games(
    state: State<'_, BridgeState>,
) -> Result<GameDiscoveryResponse, CommandError> {
    let bridge = state.bridge.clone();
    background(move || bridge.discover_games()).await
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
pub async fn freeze_review(
    state: State<'_, BridgeState>,
    selection: NormalizedSelection,
    destination: String,
    bg1_candidate_id: String,
    bg2_candidate_id: String,
) -> Result<FrozenReviewResponse, CommandError> {
    let bridge = state.bridge.clone();
    background(move || {
        bridge.freeze_review(
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
