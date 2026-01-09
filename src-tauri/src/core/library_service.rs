use crate::config::global::GlobalConfig;
use crate::core::library::Library;
use crate::models::error::SError;
use crate::models::global::LibrarySwitch;
use crate::models::library::{LibraryCreationRequirement, LibraryDTO};
use crate::models::paths::LibPathRules;
use camino::{Utf8Path, Utf8PathBuf};
use tracing::error;

/// Service for managing library lifecycle operations.
/// Handles opening, creating, and querying libraries while updating global configuration.

/// Validates that a library directory has the required structure.
/// Checks for manifest.toml and required directories (mods/, backups/, staging/).
pub fn validate_library_structure(repo_root: &Utf8Path) -> Result<(), SError> {
    let lib_paths = LibPathRules::new(repo_root);

    // Check if manifest.toml exists and is readable
    if !lib_paths.manifest.exists() {
        return Err(SError::InvalidLibrary(
            repo_root.to_string(),
            "manifest.toml is missing".to_string(),
        ));
    }

    // Check if manifest.toml is readable (attempt to read it)
    if let Err(e) = Library::read_library_manifest(repo_root) {
        return Err(SError::InvalidLibrary(
            repo_root.to_string(),
            format!("manifest.toml is invalid or unreadable: {}", e),
        ));
    }

    // Check if required directories exist
    let required_dirs = [&lib_paths.mods, &lib_paths.backups, &lib_paths.staging];
    for dir in required_dirs.iter() {
        if !dir.exists() {
            return Err(SError::InvalidLibrary(
                repo_root.to_string(),
                format!("missing required directory: {}", dir.file_name().unwrap_or("unknown")),
            ));
        }
        if !dir.is_dir() {
            return Err(SError::InvalidLibrary(
                repo_root.to_string(),
                format!("expected directory but found file: {}", dir.file_name().unwrap_or("unknown")),
            ));
        }
    }

    Ok(())
}

/// Derives the library root path from the game root.
/// Returns game_root/.mod_keeper
pub fn derive_library_root(game_root: &Utf8Path) -> Utf8PathBuf {
    let spt_rules = crate::models::paths::SPTPathRules::default();
    game_root.join(&spt_rules.library_default)
}

/// Loads a library and updates the global configuration (MRU list and last_opened).
pub fn open_library(config: &mut GlobalConfig, path: &Utf8Path) -> Result<Library, SError> {
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
/// Derives repo_root from game_root as game_root/.mod_keeper.
/// If the library already exists and is valid, opens it instead of creating.
pub fn create_library(
    config: &mut GlobalConfig,
    requirement: LibraryCreationRequirement,
) -> Result<Library, SError> {
    // Derive the library root from game_root (always use game_root/.mod_keeper)
    let repo_root = derive_library_root(&requirement.game_root);

    // Check if the library directory already exists
    if repo_root.exists() {
        // Validate the existing library structure
        match validate_library_structure(&repo_root) {
            Ok(_) => {
                // Library exists and is valid, open it instead of creating
                return open_library(config, &repo_root);
            }
            Err(e) => {
                // Library exists but is invalid, return the error
                return Err(e);
            }
        }
    }

    // Library doesn't exist, create a new one
    // Update requirement with the derived repo_root
    let mut updated_requirement = requirement;
    updated_requirement.repo_root = repo_root.clone();

    let library = Library::create(updated_requirement.clone())?;

    // Update config only on success
    config.update_recent(&repo_root);
    config.save();

    Ok(library)
}

/// Returns a summary of all known libraries.
pub fn get_known_library_summary(config: &GlobalConfig) -> Vec<LibraryDTO> {
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
pub fn get_active_library_manifest(config: &GlobalConfig) -> Option<LibraryDTO> {
    config
        .last_opened
        .as_ref()
        .and_then(|path| Library::read_library_manifest(path).ok())
}

/// Converts the global configuration state into a LibrarySwitch DTO.
pub fn to_library_switch(config: &GlobalConfig) -> LibrarySwitch {
    LibrarySwitch {
        active: get_active_library_manifest(config),
        libraries: get_known_library_summary(config),
    }
}