use std::path::{Path, PathBuf};
// src/core/registry.rs
use crate::config::global::{load_config, GlobalConfig};
use crate::core::library::Library;
use crate::models::paths::SPTPathCanonical;
use crate::utils::process::ProcessChecker;
use parking_lot::Mutex;
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

    pub fn get_canonical_spt_paths_from_active_instance(&self) -> Option<Vec<PathBuf>> {
        self.active_instance.lock().as_ref().map(|v| {
            vec![
                v.spt_paths_canonical.client_exe.clone(),
                v.spt_paths_canonical.server_exe.clone(),
            ]
        })
    }
    pub fn is_game_or_server_running(&self) -> bool {
        self.get_canonical_spt_paths_from_active_instance()
            .map(|v| self.is_running(&v))
            .unwrap_or(false)
    }
}

impl Default for AppRegistry {
    fn default() -> Self {
        Self {
            active_instance: Arc::new(Mutex::new(None)),
            global_config: Arc::new(Mutex::new(load_config())),
            sys: Mutex::new(System::new()),
        }
    }
}
