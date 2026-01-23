use camino::Utf8Path;

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
