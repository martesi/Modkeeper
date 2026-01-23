use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use parking_lot::{RawMutex, lock_api::Mutex};

use crate::{config::global::GlobalConfig, core::library::Library, models::error::SError};

/// Removes a library from known_libraries without deleting files.
/// Returns true if the library was in the list.
pub fn close_library(config: &mut GlobalConfig, repo_root: &Utf8Path) -> Result<bool, SError> {
    let was_in_list = config.known_libraries.iter().any(|p| p == repo_root);
    config.known_libraries.retain(|p| p != repo_root);
    if was_in_list {
        config.save();
    }
    Ok(was_in_list)
}

/// Removes a library: unlinks all mods, removes from known_libraries, and deletes the directory.
/// Returns true if the library was in the list.
pub fn remove_library(config: &mut GlobalConfig, repo_root: &Utf8Path) -> Result<bool, SError> {
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

    // Remove from known_libraries
    let was_in_list = config.known_libraries.iter().any(|p| p == repo_root);
    config.known_libraries.retain(|p| p != repo_root);
    if was_in_list {
        config.save();
    }

    // Remove library_root directory
    if repo_root.exists() {
        std::fs::remove_dir_all(repo_root)?;
    }

    Ok(was_in_list)
}

/// Finds a library by its ID from known_libraries.
/// Returns the repo_root path of the matching library.
pub fn find_library_by_id(config: &GlobalConfig, library_id: &str) -> Result<Utf8PathBuf, SError> {
    for path in &config.known_libraries {
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