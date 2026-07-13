use crate::config::global::GlobalConfig;
use crate::core::library::Library;
use crate::core::mod_stager::StageMaterial;
use crate::models::error::SError;
use crate::store::{self, app_config::AppConfig};
use crate::utils::process;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use sysinfo::System;

pub struct AppRegistry {
    // Arc<Mutex<Option>> allows us to "swap" the entire instance safely
    pub active_instance: Arc<Mutex<Option<Library>>>,
    pub global_config: Arc<Mutex<GlobalConfig>>,
    pub app_config: Arc<Mutex<AppConfig>>,
    /// Startup config load/save problem to surface once via the frontend (C12).
    pub config_warning: Mutex<Option<String>>,
    pub sys: Mutex<System>,
    /// Tracks whether the init command has been called
    pub init_called: Arc<AtomicBool>,
}

impl AppRegistry {
    pub fn is_running<P: AsRef<Path>>(&self, canonical_paths: &[P]) -> bool {
        process::is_running(&mut self.sys.lock(), canonical_paths)
    }

    pub fn get_canonical_spt_paths(&self) -> Option<Vec<PathBuf>> {
        self.active_instance
            .lock()
            .as_ref()
            .map(|v| v.spt_canonical_paths())
    }
    pub fn is_game_or_server_running(&self) -> bool {
        self.get_canonical_spt_paths()
            .map(|v| self.is_running(&v))
            .unwrap_or(false)
    }

    pub fn get_stage_material(&self, unknown_mod_name: String) -> Result<StageMaterial, SError> {
        self.active_instance
            .lock()
            .as_ref()
            .map(|v| v.stage_material(unknown_mod_name))
            .ok_or(SError::NoActiveLibrary)
    }
}

impl Default for AppRegistry {
    fn default() -> Self {
        let global_config = GlobalConfig::load();
        let (app_config, config_warning) = store::load_or_migrate(&global_config);

        Self {
            active_instance: Arc::new(Mutex::new(None)),
            global_config: Arc::new(Mutex::new(global_config)),
            app_config: Arc::new(Mutex::new(app_config)),
            config_warning: Mutex::new(config_warning),
            sys: Mutex::new(System::new()),
            init_called: Arc::new(AtomicBool::new(false)),
        }
    }
}
