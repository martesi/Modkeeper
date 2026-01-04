// src/core/registry.rs
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::core::library::Library;
use crate::config::global::{GlobalConfig, load_config};

pub struct AppRegistry {
    // Arc<Mutex<Option>> allows us to "swap" the entire instance safely
    pub active_instance: Arc<Mutex<Option<Library>>>,
    pub global_config: Arc<Mutex<GlobalConfig>>,
}

impl AppRegistry {
    pub fn new() -> Self {
        Self {
            active_instance: Arc::new(Mutex::new(None)),
            global_config: Arc::new(Mutex::new(load_config())),
        }
    }
}