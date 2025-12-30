pub mod global;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::BTreeMap;
use std::path::PathBuf;

const APP_NAME: &str = "shooter";

#[derive(Serialize, Deserialize, Type)]
pub struct AppSettings {
    pub version: u8,
    pub home: PathBuf,
}

impl Default for AppSettings {
    fn default() -> Self {
        let base_dir = ProjectDirs::from("com", "martes", "shooter")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
            })
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            version: 0,
            home: base_dir.join("mods"),
        }
    }
}

impl AppSettings {
    pub fn load() -> Result<AppSettings, confy::ConfyError> {
        confy::load(APP_NAME, None)
    }

    pub fn save(&self) -> Result<(), confy::ConfyError> {
        confy::store(APP_NAME, None, self)
    }
}

#[derive(Deserialize, Serialize, Type)]
pub enum ModType {
    Client = 0,
    Server = 1,
}

#[derive(Deserialize, Serialize, Type)]
pub struct ModFeat {
    r#type: ModType,
    path: PathBuf,
}

#[derive(Deserialize, Serialize, Type)]
pub struct ModDef {
    id: String,
    includes: Vec<ModFeat>,
    enabled: bool,
}

#[derive(Serialize, Deserialize, Default, Type)]
pub struct RepoDef {
    version: u8,
    records: BTreeMap<String, ModDef>,
}

impl RepoDef {
    pub fn load() -> Result<RepoDef, confy::ConfyError> {
        AppSettings::load().and_then(|settings| confy::load_path(settings.home))
    }

    pub fn save(&self) -> Result<(), confy::ConfyError> {
        AppSettings::load().and_then(|settings| confy::store_path(settings.home, self))
    }
}
