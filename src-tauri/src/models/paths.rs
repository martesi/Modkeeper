use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct SptPathConfig {
    #[specta(type = String)] pub client_plugins: Utf8PathBuf,
    #[specta(type = String)] pub server_mods: Utf8PathBuf,
    #[specta(type = String)] pub server_dll: Utf8PathBuf,
    #[specta(type = String)] pub server_exe: Utf8PathBuf,
    #[specta(type = String)] pub client_exe: Utf8PathBuf,
}

impl Default for SptPathConfig {
    fn default() -> Self {
        Self {
            client_plugins: "BepInEx/plugins".into(),
            server_mods: "SPT/user/mods".into(),
            server_dll: "SPT/SPT.Server.dll".into(),
            server_exe: "SPT/SPT.Server.exe".into(),
            client_exe: "EscapeFromTarkov.exe".into(),
        }
    }
}