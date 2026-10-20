mod commands;
#[cfg(target_os = "macos")]
mod mac;
mod prompter;
mod shortcuts;
mod state;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(shortcuts::plugin())
        .setup(|app| {
            let state = state::AppState::load(app.handle())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_scripts,
            commands::read_script,
            commands::write_script,
            commands::create_script,
            commands::delete_script,
            commands::parse_script,
            commands::get_settings,
            commands::save_settings,
            commands::open_prompter,
            commands::close_prompter,
            commands::get_active_script,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Eyeline");
}
