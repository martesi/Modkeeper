use crate::core::cleanup;
use crate::core::deployment;
use crate::core::library::Library;
use crate::core::mod_backup;
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::mod_dto::Mod;
use crate::utils::file::FileUtils;
use camino::Utf8Path;

/// Adds or updates a mod in the library.
/// Creates a backup if the mod already exists.
pub fn add_mod(library: &mut Library, mod_root: &Utf8Path, fs: ModFS) -> Result<(), SError> {
    let mod_id = fs.id.clone();
    let dst = library.lib_paths.mods.join(&mod_id);

    // Create backup if mod already exists
    if dst.exists() {
        mod_backup::create_backup(&library.lib_paths, &mod_id)?;
    }

    std::fs::create_dir_all(&dst)?;
    FileUtils::copy_recursive(mod_root, &dst)?;

    library
        .mods
        .entry(mod_id.clone())
        .and_modify(|m| {
            m.mod_type = fs.mod_type.clone();
            m.icon_data = None; // Reset icon_data when updating
        })
        .or_insert_with(|| Mod {
            id: mod_id.clone(),
            is_active: false,
            mod_type: fs.mod_type.clone(),
            name: Default::default(),
            manifest: None,
            icon_data: None,
        });

    library.cache.add(&dst, fs);
    library.mark_dirty();
    library.persist()?;
    Ok(())
}

/// Removes a mod from the library.
/// Unlinks files, junctions, and shared directories, then removes from filesystem.
/// Always attempts to unlink regardless of active status, as library state may not be synced.
/// Does not mark library dirty as sync status already reflects unlinked state.
pub fn remove_mod(library: &mut Library, id: &str) -> Result<(), SError> {
    // Get mod's ModFS from cache before removing
    let mod_fs_exists = library.cache.mods.contains_key(id);

    // Always attempt to unlink - active status may not match filesystem state
    if mod_fs_exists {
        // Find what paths need to be unlinked (treats mod as active for ownership calculation)
        let (unlink_paths, shared_dirs) = deployment::find_mod_links(
            &library.game_root,
            &library.lib_paths,
            &library.spt_rules,
            &library.mods,
            &library.cache,
            id,
        )?;

        // Unlink all paths and shared directories
        cleanup::unlink_mod(
            &library.game_root,
            &library.repo_root,
            &library.lib_paths,
            &library.cache,
            id,
            &unlink_paths,
            &shared_dirs,
            &library.spt_rules,
        )?;
    }

    // Remove all backups for this mod
    mod_backup::remove_all_backups(&library.lib_paths, id)?;

    // Remove mod directory from filesystem
    let mod_dir = library.lib_paths.mods.join(id);
    if mod_dir.exists() {
        std::fs::remove_dir_all(&mod_dir)?;
    }

    // Remove from cache and mods map
    library.cache.mods.remove(id);
    library.mods.remove(id);

    // Do NOT mark dirty - sync status already reflects the unlinked state
    library.persist()?;
    Ok(())
}

/// Toggles the active state of a mod.
pub fn toggle_mod(library: &mut Library, id: &str, is_active: bool) -> Result<(), SError> {
    let mod_entry = library
        .mods
        .get_mut(id)
        .ok_or_else(|| SError::ModNotFound(id.to_string()))?;
    mod_entry.is_active = is_active;
    library.mark_dirty();
    library.persist()?;
    Ok(())
}
