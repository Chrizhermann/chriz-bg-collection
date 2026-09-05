pub mod bridge;
mod commands;
pub mod consistency;
pub mod error;
pub mod shortcut;
pub mod updates;

use bridge::NativeBridge;
use commands::BridgeState;
use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

/// Runs the desktop shell with only the reviewed native command surface enabled.
pub fn run() -> tauri::Result<()> {
    let startup_install_id = bridge::parse_startup_install_id(std::env::args_os());
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            let resource_dir = app.path().resource_dir()?;
            let cache_dir = app.path().app_cache_dir()?;
            app.manage(BridgeState::new(
                NativeBridge::from_resource_dir_with_cache(&resource_dir, &cache_dir)
                    .with_startup_install_id(startup_install_id.clone()),
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::select_profile,
            commands::installation_defaults,
            commands::discover_games,
            commands::choose_game_folder,
            commands::inspect_game_path,
            commands::evaluate_build,
            commands::inspect_manual_downloads,
            commands::inspect_destination,
            commands::choose_destination_folder,
            commands::freeze_review,
            commands::start_build,
            commands::resume_build,
            commands::supply_manual_archive,
            commands::open_manual_source,
            commands::get_run_snapshot,
            commands::continue_waiting,
            commands::pause_run,
            commands::cancel_run,
            commands::list_managed_installations,
            commands::export_diagnostics,
            commands::launch_install,
            commands::install_radar,
            commands::open_install_folder,
            commands::create_desktop_shortcut,
            commands::check_updates,
            commands::install_app_update,
            commands::activate_recipe_update,
        ])
        .build(tauri::generate_context!())?;
    app.run(|app, event| {
        let RunEvent::WindowEvent {
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } = event
        else {
            return;
        };
        let state = app.state::<BridgeState>();
        let Some(run_id) = state.active_run_id() else {
            return;
        };
        api.prevent_close();
        let app = app.clone();
        app.dialog()
            .message("A build is active. Pause safely waits for the current mod and records a resumable checkpoint. Stop now force-terminates WeiDU and may require repair. CEBG will remain open while either action finishes.")
            .title("Installation still running")
            .buttons(MessageDialogButtons::YesNoCancelCustom(
                "Pause safely".to_owned(),
                "Stop now".to_owned(),
                "Keep running".to_owned(),
            ))
            .show_with_result(move |choice| {
                let state = app.state::<BridgeState>();
                match choice {
                    MessageDialogResult::Custom(label) if label == "Pause safely" => {
                        state.pause_run_now(&run_id);
                    }
                    MessageDialogResult::Custom(label) if label == "Stop now" => {
                        state.cancel_run_now(&run_id);
                    }
                    _ => {}
                }
            });
    });
    Ok(())
}
