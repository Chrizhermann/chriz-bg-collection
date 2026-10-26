pub mod application_data;
pub mod bridge;
mod commands;
pub mod consistency;
pub mod error;
pub mod flavor;
mod installation_lookup_trace;
mod removal_size;
pub mod shortcut;
pub mod updates;

use bridge::NativeBridge;
use commands::BridgeState;
use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

/// Runs the desktop shell with only the reviewed native command surface enabled.
pub fn run() -> tauri::Result<()> {
    let startup_install_id = bridge::parse_startup_install_id(std::env::args_os());
    let product = flavor::CURRENT;
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());
    // An app without its own signed update feed never loads the updater.
    if product.app_updates {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }
    let app = builder
        .setup(move |app| {
            // Folders, registry and texts follow the flavor compiled from this configuration.
            let identifier = &app.config().identifier;
            if identifier != product.identifier {
                return Err(format!(
                    "this {} build was configured as {identifier:?}, not {:?}",
                    product.product_name, product.identifier
                )
                .into());
            }
            if product.standalone.is_some() {
                // Every engine operation of this process then fails closed for a recipe whose
                // records belong in another application data folder.
                bg_engine::cli::set_application_data_directory(product.application_data_directory)
                    .map_err(|error| error.to_string())?;
            }
            let resource_dir = app.path().resource_dir()?;
            let cache_dir = app.path().app_cache_dir()?;
            if cfg!(feature = "installation-lookup-diagnostics") {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_title(&format!("{} — Diagnostics", product.product_name))?;
                }
            }
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
            commands::rename_installation,
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
            commands::get_manual_checkpoint,
            commands::check_manual_checkpoint,
            commands::accept_manual_checkpoint,
            commands::open_manual_checkpoint_folder,
            commands::supply_manual_archive,
            commands::open_manual_source,
            commands::get_run_snapshot,
            commands::continue_waiting,
            commands::pause_run,
            commands::cancel_run,
            commands::list_managed_installations,
            commands::preview_installation_removal,
            commands::remove_installation,
            commands::export_diagnostics,
            commands::launch_install,
            commands::game_log_status,
            commands::set_game_log,
            commands::install_radar,
            commands::open_install_folder,
            commands::create_desktop_shortcut,
            commands::check_updates,
            commands::record_install_lookup_client,
            commands::inspect_installed_update,
            commands::inspect_mod_upgrade,
            commands::apply_mod_upgrade,
            commands::restore_mod_upgrade,
            commands::inspect_install_patches,
            commands::apply_install_patch,
            commands::undo_install_patch,
            commands::restore_install_patch,
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
        if state.file_update_active() {
            api.prevent_close();
            app.dialog().message(format!("A file update or backup is still running. Keep {} open until it finishes.", flavor::CURRENT.short_name)).title("Update in progress").show(|_| {});
            return;
        }
        let Some(run_id) = state.active_run_id() else {
            return;
        };
        api.prevent_close();
        let app = app.clone();
        app.dialog()
            .message(format!("A build is active. Pause safely waits for the current mod and records a resumable checkpoint. Stop now force-terminates WeiDU and may require repair. {} will remain open while either action finishes.", flavor::CURRENT.short_name))
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
