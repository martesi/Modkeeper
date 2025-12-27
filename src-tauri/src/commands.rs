use crate::config::{AppSettings, RepoDef};
use tauri_plugin_log::log::error;

#[tauri::command]
#[specta::specta]
pub fn get_app_settings() -> Option<AppSettings> {
    match AppSettings::load() {
        Ok(settings) => Some(settings),
        Err(err) => {
            error!("Error while sending app settings to frontend: {:?}", err);
            None
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_repo_def() -> Option<RepoDef> {
    match RepoDef::load() {
        Ok(def) => Some(def),
        Err(e) => {
            error!("Error while sending repo def to frontend: {:?}", e);
            None
        }
    }
}
