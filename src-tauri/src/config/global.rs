// src/config/global.rs
use serde::{Deserialize, Serialize};
use camino::Utf8PathBuf;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GlobalConfig {
    pub last_opened_instance: Option<Utf8PathBuf>,
    pub known_instances: Vec<Utf8PathBuf>,
}

pub fn load_config() -> GlobalConfig {
    confy::load("mod_keeper", "config").unwrap_or_default()
}

pub fn save_config(config: GlobalConfig) {
    let _ = confy::store("mod_keeper", "config", config);
}