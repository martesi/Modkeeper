use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct ModBackup {
    pub timestamp: String,
    #[specta(type = String)]
    pub path: Utf8PathBuf,
}
