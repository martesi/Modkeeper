use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConfig {
    /// The currently active library path. If set, this library will be opened on startup.
    /// Cleared when user explicitly closes a library.
    pub library_last: Option<Utf8PathBuf>,

    /// List of closed/recent library paths (not including the active one).
    pub library_recent: Vec<Utf8PathBuf>,
}

#[cfg(debug_assertions)]
const CONFIG_NAME: &str = "config_debug";

#[cfg(not(debug_assertions))]
const CONFIG_NAME: &str = "config";

impl GlobalConfig {
    pub fn load() -> GlobalConfig {
        confy::load("Modkeeper", CONFIG_NAME).unwrap_or_default()
    }

    pub fn save(&self) {
        let _ = confy::store("Modkeeper", CONFIG_NAME, self);
    }

    /// Opens a library: closes the current library_last (if any), sets new path as library_last.
    pub(crate) fn open_library(&mut self, path: &Utf8Path) {
        // Close the currently active library (if any) before opening the new one
        if let Some(ref prev_path) = self.library_last.take() {
            if prev_path != path {
                // Add previous library to front of recent list
                self.library_recent.retain(|p| p != prev_path);
                self.library_recent.insert(0, prev_path.clone());
            }
        }

        // Remove new path from recent list (it's now active, not "recent")
        self.library_recent.retain(|p| p != path);

        // Set as active library
        self.library_last = Some(path.to_owned());
    }

    /// Closes a library: clears library_last, adds to front of library_recent.
    pub(crate) fn close_library(&mut self, path: &Utf8Path) {
        // Clear active library
        self.library_last = None;

        // Remove if already in list (to avoid duplicates)
        self.library_recent.retain(|p| p != path);

        // Add to front of recent list
        self.library_recent.insert(0, path.to_owned());
    }

    /// Removes a library completely from both library_last and library_recent.
    pub(crate) fn remove_library(&mut self, path: &Utf8Path) {
        self.library_recent.retain(|p| p != path);
        if self.library_last.as_ref().map(|p| p.as_path()) == Some(path) {
            self.library_last = None;
        }
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
