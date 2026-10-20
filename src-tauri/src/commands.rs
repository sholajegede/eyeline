use crate::{prompter, state::AppState};
use eyeline_core::{parse_script as parse, Script, ScriptMeta, Settings};
use tauri::{AppHandle, Emitter, Manager, State};

fn lock<'a, T>(m: &'a std::sync::Mutex<T>) -> Result<std::sync::MutexGuard<'a, T>, String> {
    m.lock().map_err(|_| "internal state is unavailable".to_string())
}

#[tauri::command]
pub fn list_scripts(state: State<'_, AppState>) -> Result<Vec<ScriptMeta>, String> {
    lock(&state.store)?.list().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_script(state: State<'_, AppState>, id: String) -> Result<String, String> {
    lock(&state.store)?.read(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_script(state: State<'_, AppState>, id: String, content: String) -> Result<(), String> {
    lock(&state.store)?.write(&id, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_script(state: State<'_, AppState>, title: String) -> Result<String, String> {
    lock(&state.store)?.create(&title).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_script(state: State<'_, AppState>, id: String) -> Result<(), String> {
    lock(&state.store)?.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn parse_script(text: String) -> Script {
    parse(&text)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(lock(&state.settings)?.clone())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let clean = settings.sanitized();
    clean.save(&state.settings_path).map_err(|e| e.to_string())?;
    *lock(&state.settings)? = clean.clone();
    if let Some(win) = app.get_webview_window("prompter") {
        win.set_content_protected(clean.hide_from_capture)
            .map_err(|e| e.to_string())?;
    }
    app.emit("settings-changed", &clean).map_err(|e| e.to_string())?;
    Ok(clean)
}

#[tauri::command]
pub async fn open_prompter(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    lock(&state.store)?.read(&id).map_err(|e| e.to_string())?;
    *lock(&state.active_script)? = Some(id.clone());
    let settings = lock(&state.settings)?.clone();

    if app.get_webview_window("prompter").is_some() {
        prompter::show(&app).map_err(|e| e.to_string())?;
        app.emit("prompter-load", &id).map_err(|e| e.to_string())
    } else {
        prompter::create(&app, &settings).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn close_prompter(app: AppHandle) -> Result<(), String> {
    match app.get_webview_window("prompter") {
        Some(win) => win.close().map_err(|e| e.to_string()),
        None => Ok(()),
    }
}

#[tauri::command]
pub fn get_active_script(state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(lock(&state.active_script)?.clone())
}
