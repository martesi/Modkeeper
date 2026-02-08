use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use parking_lot::{RawMutex, lock_api::Mutex};

use crate::{config::global::GlobalConfig, core::library::Library, models::error::SError};

/// Closes a library: moves from library_last to front of library_recent.
pub fn close_library(config: &mut GlobalConfig, repo_root: &Utf8Path) -> Result<(), SError> {
    config.close_library(repo_root);
    config.save();
    Ok(())
}

/// Removes a library: unlinks all mods, removes completely, and deletes the directory.
pub fn remove_library(config: &mut GlobalConfig, repo_root: &Utf8Path) -> Result<(), SError> {
    use crate::core::cleanup;

    // Load library to get cache for unlinking mods
    let library = Library::load(repo_root).ok();

    // If library exists and is loaded, unlink all mods
    if let Some(lib) = library.as_ref() {
        cleanup::purge(
            &lib.game_root,
            &lib.repo_root,
            &lib.spt_rules,
            &lib.lib_paths,
            &lib.cache,
        )?;
    }

    // Remove from both library_last and library_recent
    config.remove_library(repo_root);
    config.save();

    // Remove library_root directory
    if repo_root.exists() {
        std::fs::remove_dir_all(repo_root)?;
    }

    Ok(())
}

/// Finds a library by its ID from all known libraries.
/// Returns the repo_root path of the matching library.
pub fn find_library_by_id(config: &GlobalConfig, library_id: &str) -> Result<Utf8PathBuf, SError> {
    for path in config.all_libraries() {
        match Library::read_library_manifest(path) {
            Ok(dto) => {
                if dto.id == library_id {
                    return Ok(path.clone());
                }
            }
            Err(_) => {
                // Skip libraries that fail to load manifest
                continue;
            }
        }
    }
    Err(SError::InvalidLibrary(
        library_id.to_string(),
        "Library not found in known libraries".to_string(),
    ))
}

pub fn resolve_target_library_path(
    library_id: Option<String>,
    config_handle: &Arc<Mutex<RawMutex, GlobalConfig>>,
    instance_handle: &Arc<Mutex<RawMutex, Option<Library>>>,
) -> Result<Option<Utf8PathBuf>, SError> {
    let Some(lib_id) = library_id else {
        return Ok(None);
    };

    let path = {
        let config = config_handle.lock();
        find_library_by_id(&config, &lib_id)?
    };

    let is_active = instance_handle
        .lock()
        .as_ref()
        .map(|lib| lib.repo_root == path)
        .unwrap_or(false);

    Ok((!is_active).then_some(path))
}
