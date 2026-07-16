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
    let mut config: AppConfig =
        toml::from_str(&content).map_err(|e| SError::ParseError(e.to_string()))?;
    // Value migration: configs written before the default was fixed carry bare "en", but the
    // frontend loads lingui catalogs by this value verbatim and only region-tagged ones exist.
    if config.settings.language == "en" {
        config.settings.language = "en-US".to_string();
    }
    Ok(config)
}

/// Atomically saves the App Config to the given path (C12/M7):
/// serialize to a temp file in the same directory, then rename over the target.
pub fn save_to(path: &Utf8PathBuf, config: &AppConfig) -> Result<(), SError> {
    let content =
        toml::to_string(config).map_err(|e| SError::ConfigSaveFailed(e.to_string()))?;
    file::atomic_write(path, content).map_err(|e| SError::ConfigSaveFailed(e.to_string()))
}

/// The App Config bound to the file it round-trips through. Keeping the path
/// explicit lets services and tests save without a hidden global location.
pub struct AppConfigStore {
    pub path: Utf8PathBuf,
    pub config: AppConfig,
}

impl AppConfigStore {
    pub fn save(&self) -> Result<(), SError> {
        save_to(&self.path, &self.config)
    }
}

/// Startup path: load the App Config, running the one-time confy migration when
/// no libraries are registered yet. Never blocks or fails startup - failures are
/// logged and returned as a warning for the frontend to surface later (C12).
/// A config that failed to parse is left untouched on disk (no silent reset).
pub fn load_or_migrate(
    old: &crate::config::global::GlobalConfig,
) -> (AppConfigStore, Option<String>) {
    let path = match config_path() {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("Failed to locate app config dir: {e}");
            let fallback = Utf8PathBuf::from_path_buf(std::env::temp_dir())
                .map(|dir| dir.join(CONFIG_FILE))
                .unwrap_or_else(|_| Utf8PathBuf::from(CONFIG_FILE));
            return (
                AppConfigStore {
                    path: fallback,
                    config: AppConfig::default(),
                },
                Some(e.to_string()),
            );
        }
    };

    let mut store = match load_from(&path) {
        Ok(config) => AppConfigStore { path, config },
        Err(e) => {
            tracing::error!("Failed to load app config: {e}");
            let warning = Some(e.to_string());
            return (
                AppConfigStore {
                    path,
                    config: AppConfig::default(),
                },
                warning,
            );
        }
    };

    if store.config.known_libraries.is_empty() {
        let migrated = app_config::migrate_from_confy(old);
        if !migrated.known_libraries.is_empty() {
            store.config.known_libraries = migrated.known_libraries;
            store.config.app_state.active_library_id = migrated.app_state.active_library_id;
            if let Err(e) = store.save() {
                tracing::error!("Failed to save migrated app config: {e}");
                return (store, Some(e.to_string()));
            }
        }
    }

    (store, None)
}
