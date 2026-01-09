use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GlobalConfig {
    pub known_libraries: Vec<Utf8PathBuf>,
}

impl GlobalConfig {
    pub fn load() -> GlobalConfig {
        confy::load("mod_keeper", "config").unwrap_or_default()
    }

    pub fn save(&self) {
        let _ = confy::store("mod_keeper", "config", self);
    }

    pub(crate) fn update_recent(&mut self, path: &Utf8Path) {
        // Remove existing entry to avoid duplicates
        self.known_libraries.retain(|p| p != path);

        // Insert at the front (Most Recently Used)
        self.known_libraries.insert(0, path.to_owned());

        self.save();
    }
}
