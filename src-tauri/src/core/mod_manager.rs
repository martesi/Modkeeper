use crate::models::instance_dto::ModManagerInstanceDTO;
use camino::Utf8PathBuf;
use crate::models::error::SError;

pub trait ModManagerInstance {
    fn is_running(&self) -> bool;
    fn add_mod(&mut self, files: Vec<Utf8PathBuf>) -> Result<(), SError>;
    fn remove_mod(&mut self, id: &str) -> Result<(), SError>;
    fn deploy_active_mods(&self) -> Result<(), SError>;
    fn scan_repo(&mut self) -> Result<(), SError>;
    fn to_dto(&self) -> ModManagerInstanceDTO;
}

