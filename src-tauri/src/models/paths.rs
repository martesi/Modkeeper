use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct InstancePaths {
    #[specta(type = String)]
    pub client_plugins: Utf8PathBuf,
    #[specta(type = String)]
    pub server_mods: Utf8PathBuf,
    #[specta(type = String)]
    pub server_dll: Utf8PathBuf,
    #[specta(type = String)]
    pub server_exe: Utf8PathBuf,
    #[specta(type = String)]
    pub client_exe: Utf8PathBuf,
    #[specta(type = String)]
    pub manifest_folder: Utf8PathBuf,
    #[specta(type = String)]
    pub manifest_file: Utf8PathBuf,
}

impl Default for InstancePaths {
    fn default() -> Self {
        Self {
            client_plugins: "BepInEx/plugins".into(),
            server_mods: "SPT/user/mods".into(),
            server_dll: "SPT/SPT.Server.dll".into(),
            server_exe: "SPT/SPT.Server.exe".into(),
            client_exe: "EscapeFromTarkov.exe".into(),
            manifest_folder: "manifest".into(),
            manifest_file: "manifest/manifest.json".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct RepoConfig {
    #[specta(type = String)]
    pub backups: Utf8PathBuf,
    #[specta(type = String)]
    pub mods: Utf8PathBuf,
    #[specta(type = String)]
    pub staging: Utf8PathBuf,
    #[specta(type = String)]
    pub manifest: Utf8PathBuf,
    #[specta(type = String)]
    pub cache: Utf8PathBuf,
    #[specta(type = String)]
    pub id_divider: Utf8PathBuf,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            backups: "backups".into(),
            mods: "mods".into(),
            staging: "staging".into(),
            manifest: "manifest.toml".into(),
            cache: "cache.toml".into(),
            id_divider: "&&&".into(),
        }
    }
}
