pub mod bridge;
mod commands;
pub mod error;

use bridge::NativeBridge;
use commands::BridgeState;
use tauri::Manager;

/// Runs the desktop shell with only the reviewed native command surface enabled.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let resource_dir = app.path().resource_dir()?;
            app.manage(BridgeState::new(NativeBridge::from_resource_dir(
                &resource_dir,
            )));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::discover_games,
            commands::inspect_game_path,
            commands::evaluate_build,
        ])
        .run(tauri::generate_context!())
}
