use eyeline_core::{Settings, Store};
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub store: Mutex<Store>,
    pub settings: Mutex<Settings>,
    pub settings_path: PathBuf,
    pub active_script: Mutex<Option<String>>,
}

impl AppState {
    pub fn load(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let settings_path = app.path().app_config_dir()?.join("settings.json");
        let settings = Settings::load(&settings_path);
        let dir = match &settings.scripts_dir {
            Some(custom) => PathBuf::from(custom),
            None => app.path().document_dir()?.join("Eyeline"),
        };
        let store = Store::open(dir)?;
        Ok(Self {
            store: Mutex::new(store),
            settings: Mutex::new(settings),
            settings_path,
            active_script: Mutex::new(None),
        })
    }
}
