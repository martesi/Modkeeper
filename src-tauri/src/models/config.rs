// src/models/config.rs
use serde::{Deserialize, Serialize};
use specta::Type;
use camino::Utf8PathBuf;

#[derive(Serialize, Deserialize, Type, Default, Clone)]
pub struct GlobalConfig {
    #[specta(type = Option<String>)]
    pub last_opened_instance: Option<Utf8PathBuf>,
    #[specta(type = Vec<String>)]
    pub known_instance_paths: Vec<Utf8PathBuf>,
}