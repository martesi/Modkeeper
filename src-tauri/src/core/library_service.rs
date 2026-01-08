use crate::config::global::GlobalConfig;
use crate::core::library::Library;
use crate::models::error::SError;
use crate::models::global::LibrarySwitch;
use crate::models::library::{LibraryCreationRequirement, LibraryDTO};
use camino::Utf8Path;
use tracing::error;

/// Service for managing library lifecycle operations.
/// Handles opening, creating, and querying libraries while updating global configuration.
pub struct LibraryService;

impl LibraryService {
    /// Creates a new LibraryService instance.
    pub fn new() -> Self {
        Self
    }

    /// Loads a library and updates the global configuration (MRU list and last_opened).
    pub fn open_library(&self, config: &mut GlobalConfig, path: &Utf8Path) -> Result<Library, SError> {
        // 1. Attempt to load the library first.
        // If this fails (e.g., path invalid, manifest missing), we propagate the error
        // and do NOT update the configuration.
        let library = Library::load(path)?;

        config.update_recent(path);

        // Persist the configuration changes
        config.save();

        Ok(library)
    }

    /// Creates a new library and updates the global configuration.
    pub fn create_library(
        &self,
        config: &mut GlobalConfig,
        requirement: LibraryCreationRequirement,
    ) -> Result<Library, SError> {
        // Create first (propagate error if invalid)
        let library = Library::create(requirement.clone())?;

        // Update config only on success
        config.update_recent(&requirement.repo_root);
        config.save();

        Ok(library)
    }

    /// Returns a summary of all known libraries.
    pub fn get_known_library_summary(&self, config: &GlobalConfig) -> Vec<LibraryDTO> {
        config
            .known_libraries
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

    /// Returns the manifest for the currently active library, if any.
    pub fn get_active_library_manifest(&self, config: &GlobalConfig) -> Option<LibraryDTO> {
        config
            .last_opened
            .as_ref()
            .and_then(|path| Library::read_library_manifest(path).ok())
    }

    /// Converts the global configuration state into a LibrarySwitch DTO.
    pub fn to_library_switch(&self, config: &GlobalConfig) -> LibrarySwitch {
        LibrarySwitch {
            active: self.get_active_library_manifest(config),
            libraries: self.get_known_library_summary(config),
        }
    }
}

impl Default for LibraryService {
    fn default() -> Self {
        Self::new()
    }
}