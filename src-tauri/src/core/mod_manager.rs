use crate::core::library::Library;
use crate::core::linker;
use crate::core::mod_backup;
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::mod_dto::Mod;
use crate::utils::file::FileUtils;
use camino::Utf8Path;

/// Adds or updates a mod in the library.
/// Creates a backup if the mod already exists.
pub fn add_mod(
    library: &mut Library,
    mod_root: &Utf8Path,
    fs: ModFS,
) -> Result<(), SError> {
    let mod_id = fs.id.clone();
    let dst = library.lib_paths.mods.join(&mod_id);

    // Create backup if mod already exists
    if dst.exists() {
        mod_backup::create_backup(&library.lib_paths, &mod_id)?;
    }

    std::fs::create_dir_all(&dst)?;
    FileUtils::copy_recursive(mod_root, &dst)?;

    library.mods
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
/// Unlinks files and removes from filesystem.
pub fn remove_mod(
    library: &mut Library,
    id: &str,
) -> Result<(), SError> {
    // Remove from Cache and Filesystem
    if let Some(m) = library.cache.mods.remove(id) {
        // Note: We deliberately do not unlink here individually.
        // A full sync() is required to properly clean up state,
        // otherwise we risk leaving broken links if the user doesn't sync immediately.
        // However, to strictly follow previous logic, we unlink specific files:
        for f in &m.files {
            let _ = linker::unlink(&library.game_root.join(f));
        }
        let _ = std::fs::remove_dir_all(library.lib_paths.mods.join(id));
    }

    library.mods.remove(id);
    library.mark_dirty();
    library.persist()?;
    Ok(())
}

/// Toggles the active state of a mod.
pub fn toggle_mod(
    library: &mut Library,
    id: &str,
    is_active: bool,
) -> Result<(), SError> {
    let mod_entry = library.mods
        .get_mut(id)
        .ok_or_else(|| SError::ModNotFound(id.to_string()))?;
    mod_entry.is_active = is_active;
    library.mark_dirty();
    library.persist()?;
    Ok(())
}
