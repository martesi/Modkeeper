use crate::core::library::Library;
use crate::models::error::SError;
use crate::models::global::LibrarySwitch;
use crate::models::library_dto::LibraryDTO;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GlobalConfig {
    pub last_opened: Option<Utf8PathBuf>,
    pub known_libraries: Vec<Utf8PathBuf>,
}

impl GlobalConfig {
    pub fn load() -> GlobalConfig {
        confy::load("mod_keeper", "config").unwrap_or_default()
    }

    pub fn save(&self) {
        let _ = confy::store("mod_keeper", "config", self);
    }

    /// Loads a library and updates the global configuration (MRU list and last_opened).
    pub fn open_library(&mut self, path: &Utf8Path) -> Result<Library, SError> {
        let _path_buf = path.to_owned();

        // 1. Attempt to load the library first.
        // If this fails (e.g., path invalid, manifest missing), we propagate the error
        // and do NOT update the configuration.
        let library = Library::load(path)?;

        self.update_recent(path);

        // 4. Persist the configuration changes
        self.save();

        Ok(library)
    }

    pub fn get_known_library_summary(&self) -> Vec<LibraryDTO> {
        self.known_libraries
            .iter()
            .filter_map(|path| {
                // Attempt to read the manifest for each known path
                match Library::read_library_manifest(path) {
                    Ok(mut dto) => {
                        // Clear the mods field as requested (lightweight DTO)
                        dto.mods.clear();
                        Some(dto)
                    }
                    Err(e) => {
                        // Log the error but do not propagate it; skip this entry
                        error!("Failed to load library manifest: {e} at {path}");
                        None
                    }
                }
            })
            .collect()
    }

    pub fn create_library(
        &mut self,
        game_root: &Utf8Path,
        lib_root: &Utf8Path,
    ) -> Result<Library, SError> {
        let game_root_buf = game_root.to_path_buf();
        let lib_root_buf = lib_root.to_path_buf();

        // Create first (propagate error if invalid)
        let library = Library::create(&lib_root_buf, &game_root_buf)?;

        // Update config only on success
        self.update_recent(lib_root);
        self.save();

        Ok(library)
    }

    pub fn get_active_library_manifest(&self) -> Option<LibraryDTO> {
        self.last_opened
            .as_ref()
            .and_then(|path| Library::read_library_manifest(path).ok())
    }

    pub fn to_library_switch(&self) -> LibrarySwitch {
        LibrarySwitch {
            active: self.get_active_library_manifest(),
            libraries: self.get_known_library_summary(),
        }
    }

    fn update_recent(&mut self, path: &Utf8Path) {
        self.last_opened = Some(path.to_owned());

        // Remove existing entry to avoid duplicates
        self.known_libraries.retain(|p| p != path);

        // Insert at the front (Most Recently Used)
        self.known_libraries.insert(0, path.to_owned());

        self.save();
    }
}
