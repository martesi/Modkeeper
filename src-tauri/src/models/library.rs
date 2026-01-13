use crate::models::mod_dto::Mod;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct LibraryDTO {
    pub id: String,
    pub name: String,
    #[specta(type=String)]
    pub game_root: Utf8PathBuf,
    #[specta(type=String)]
    pub repo_root: Utf8PathBuf,
    pub spt_version: String,
    pub mods: BTreeMap<String, Mod>,
    pub is_dirty: bool,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct LibraryCreationRequirement {
    #[specta(type=String)]
    pub game_root: Utf8PathBuf,
    #[specta(type=Option<String>)]
    pub repo_root: Option<Utf8PathBuf>,
    pub name: String,
}
