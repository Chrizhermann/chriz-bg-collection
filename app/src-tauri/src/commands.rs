//! Narrow Tauri command surface. Synchronous engine work always leaves the UI thread.

use std::path::PathBuf;

use bg_engine::games::GameRole;
use bg_engine::recipe_view::NormalizedSelection;
use tauri::State;

use crate::bridge::{
    BootstrapResponse, EvaluateBuildResponse, GameCandidateResponse, GameDiscoveryResponse,
    NativeBridge,
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
