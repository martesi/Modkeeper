use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModBackup {
    pub timestamp: String,
    pub name: String,
    pub has_config: bool,
    #[specta(type = String)]
    pub path: Utf8PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub timestamp: String,
    pub name: String,
}
