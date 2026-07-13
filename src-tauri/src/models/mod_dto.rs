use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq)]
pub enum ModType {
    Client,
    Server,
    Both,
    Unknown,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub id: String,
    pub is_active: bool,
    #[serde(rename = "type")]
    pub mod_type: ModType,
    pub name: String,
}
