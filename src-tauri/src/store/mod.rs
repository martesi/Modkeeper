pub mod app_config;

use crate::models::error::SError;
use crate::utils::file;
use app_config::AppConfig;
use camino::Utf8PathBuf;
use directories::ProjectDirs;

#[cfg(debug_assertions)]
const CONFIG_FILE: &str = "config_debug.toml";

#[cfg(not(debug_assertions))]
const CONFIG_FILE: &str = "config.toml";

pub fn config_path() -> Result<Utf8PathBuf, SError> {
    let dirs = ProjectDirs::from("", "", "Modkeeper")
        .ok_or_else(|| SError::IOError("Unable to determine config directory".to_string()))?;
    let dir = Utf8PathBuf::from_path_buf(dirs.config_dir().to_path_buf())
        .map_err(|p| SError::ParseError(format!("Non-UTF-8 config path: {}", p.display())))?;
    Ok(dir.join(CONFIG_FILE))
}

/// Loads the App Config from the given path.
/// A missing file is a normal first run and yields the default config;
/// read or parse failures are surfaced, never silently reset.
pub fn load_from(path: &Utf8PathBuf) -> Result<AppConfig, SError> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = file::read_to_string(path)?;
    toml::from_str(&content).map_err(|e| SError::ParseError(e.to_string()))
}

/// Atomically saves the App Config to the given path (C12/M7):
/// serialize to a temp file in the same directory, then rename over the target.
pub fn save_to(path: &Utf8PathBuf, config: &AppConfig) -> Result<(), SError> {
    let content =
        toml::to_string(config).map_err(|e| SError::ConfigSaveFailed(e.to_string()))?;
    file::atomic_write(path, content).map_err(|e| SError::ConfigSaveFailed(e.to_string()))
}

pub fn load() -> Result<AppConfig, SError> {
    load_from(&config_path()?)
}

pub fn save(config: &AppConfig) -> Result<(), SError> {
    save_to(&config_path()?, config)
}

/// Startup path: load the App Config, running the one-time confy migration when
/// no libraries are registered yet. Never blocks or fails startup - failures are
/// logged and returned as a warning for the frontend to surface later (C12).
/// A config that failed to parse is left untouched on disk (no silent reset).
pub fn load_or_migrate(old: &crate::config::global::GlobalConfig) -> (AppConfig, Option<String>) {
    let mut config = match load() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load app config: {e}");
            return (AppConfig::default(), Some(e.to_string()));
        }
    };

    if config.known_libraries.is_empty() {
        let migrated = app_config::migrate_from_confy(old);
        if !migrated.known_libraries.is_empty() {
            config.known_libraries = migrated.known_libraries;
            config.app_state.active_library_id = migrated.app_state.active_library_id;
            if let Err(e) = save(&config) {
                tracing::error!("Failed to save migrated app config: {e}");
                return (config, Some(e.to_string()));
            }
        }
    }

    (config, None)
}
