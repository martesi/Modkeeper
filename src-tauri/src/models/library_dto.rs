use std::collections::BTreeMap;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::models::mod_dto::Mod;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct LibraryDTO {
    pub id: String,
    #[specta(type=String)]
    pub game_root: Utf8PathBuf,
    #[specta(type=String)]
    pub repo_root: Utf8PathBuf,
    pub spt_version: String,
    pub mods: BTreeMap<String, Mod>,
}