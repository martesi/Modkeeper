// src/models/instance_dto.rs
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::models::mod_dto::Mod;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct ModManagerInstanceDTO {
    pub id: String,
    pub game_root: String,
    pub repo_root: String,
    pub spt_version: String,
    pub mods: Vec<Mod>,
}