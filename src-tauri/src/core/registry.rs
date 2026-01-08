use crate::config::global::GlobalConfig;
use crate::core::library::Library;
use crate::core::mod_stager::StageMaterial;
use crate::models::error::SError;
use crate::utils::process::ProcessChecker;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use sysinfo::System;

pub struct AppRegistry {
    // Arc<Mutex<Option>> allows us to "swap" the entire instance safely
    pub active_instance: Arc<Mutex<Option<Library>>>,
    pub global_config: Arc<Mutex<GlobalConfig>>,
    pub sys: Mutex<System>,
}

impl AppRegistry {
    pub fn is_running<P: AsRef<Path>>(&self, canonical_paths: &[P]) -> bool {
        ProcessChecker::is_running(&mut self.sys.lock(), canonical_paths)
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

    pub fn get_stage_material(&self) -> Result<StageMaterial, SError> {
        self.active_instance
            .lock()
            .as_ref()
            .map(|v| v.stage_material())
            .ok_or(SError::NoActiveLibrary)
    }
}

impl Default for AppRegistry {
    fn default() -> Self {
        Self {
            active_instance: Arc::new(Mutex::new(None)),
            global_config: Arc::new(Mutex::new(GlobalConfig::load())),
            sys: Mutex::new(System::new()),
        }
    }
}
