/// Runs the desktop shell with no privileged plugins or JavaScript commands enabled.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default().run(tauri::generate_context!())
}
