use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

/// The legacy confy-persisted config. Kept read-only as the source for the
/// one-time App Config migration (store::load_or_migrate); nothing writes it.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConfig {
    /// The previously active library path.
    pub library_last: Option<Utf8PathBuf>,

    /// List of closed/recent library paths (not including the active one).
    pub library_recent: Vec<Utf8PathBuf>,
}

#[cfg(debug_assertions)]
const CONFIG_NAME: &str = "config_debug";

#[cfg(not(debug_assertions))]
const CONFIG_NAME: &str = "config";

impl GlobalConfig {
    /// Reads the file exactly where confy ("Modkeeper", CONFIG_NAME) wrote it:
    /// ProjectDirs("rs", "", "Modkeeper").config_dir()/<CONFIG_NAME>.toml.
    /// Mirrors confy's default-on-any-failure behavior; fine for a legacy
    /// read-only migration source.
    pub fn load() -> GlobalConfig {
        directories::ProjectDirs::from("rs", "", "Modkeeper")
            .map(|dirs| dirs.config_dir().join(format!("{CONFIG_NAME}.toml")))
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Returns all known library paths (active + recent).
    pub fn all_libraries(&self) -> Vec<&Utf8PathBuf> {
        let mut result: Vec<&Utf8PathBuf> = Vec::new();
        if let Some(ref last) = self.library_last {
            result.push(last);
        }
        result.extend(self.library_recent.iter());
        result
    }
}
