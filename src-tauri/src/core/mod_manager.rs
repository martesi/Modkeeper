use crate::models::instance_dto::ModManagerInstanceDTO;
use camino::Utf8PathBuf;

pub trait ModManagerInstance {
    fn is_running(&self) -> bool;
    fn add_mod(&mut self, files: Vec<Utf8PathBuf>) -> Result<String, String>;
    fn remove_mod(&mut self, id: &str) -> Result<(), String>;
    fn deploy_active_mods(&self) -> Result<(), String>;
    fn scan_repo(&mut self) -> Result<(), String>;
    fn to_dto(&self) -> ModManagerInstanceDTO;
}

