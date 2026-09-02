#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = chriz_bg_app_lib::run() {
        eprintln!("failed to run installer shell: {error}");
        std::process::exit(1);
    }
}
