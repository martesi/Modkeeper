use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug, Default)]
pub struct ModManifest {
    pub guid: String,
    pub name: String,
    pub version: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq)]
pub enum ModType {
    Client,
    Server,
    Both,
    Unknown,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Mod {
    pub id: String,
    pub is_active: bool,
    pub mod_type: ModType,
    pub name: String,
    pub manifest: Option<ModManifest>,
    // files removed: only needed in cache, not for frontend display
}

