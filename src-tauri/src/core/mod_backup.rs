use camino::Utf8PathBuf;

use crate::core::library::Library;
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::mod_backup::ModBackup;
use crate::models::paths::LibPathRules;
use crate::utils::file::FileUtils;
use crate::utils::time::get_unix_timestamp;

/// Creates a backup of a mod at the current timestamp.
/// Backup is stored at: `backups/{mod_id}/{timestamp}/`
pub fn create_backup(lib_paths: &LibPathRules, mod_id: &str) -> Result<(), SError> {
    let mod_dir = lib_paths.mods.join(mod_id);

    if !mod_dir.exists() {
        return Ok(()); // Nothing to backup
    }

    let timestamp = get_unix_timestamp().to_string();
    let backup_dir = lib_paths.backups.join(mod_id).join(&timestamp);

    std::fs::create_dir_all(&backup_dir)?;
    FileUtils::copy_recursive(&mod_dir, &backup_dir)?;
    Ok(())
}

/// Lists all available backups for a given mod.
/// Returns timestamps in descending order (newest first).
pub fn list_backups(lib_paths: &LibPathRules, mod_id: &str) -> Result<Vec<ModBackup>, SError> {
    let backup_dir = lib_paths.backups.join(mod_id);

    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&backup_dir)?;
    let mut backups: Vec<ModBackup> = entries
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                Some(ModBackup {
                    timestamp: e.file_name().into_string().ok()?,
                    path: Utf8PathBuf::from_path_buf(e.path()).ok()?,
                })
            })
        })
        .collect();

    // Sort descending (newest first)
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(backups)
}

/// Restores a mod from a backup.
/// Creates a backup of the current state before restoring.
pub fn restore_backup(library: &mut Library, mod_id: &str, timestamp: &str) -> Result<(), SError> {
    // Verify mod exists
    if !library.mods.contains_key(mod_id) {
        return Err(SError::ModNotFound(mod_id.to_string()));
    }

    let backup_dir = library.lib_paths.backups.join(mod_id).join(timestamp);

    if !backup_dir.exists() {
        return Err(SError::Unexpected);
    }

    let mod_dir = library.lib_paths.mods.join(mod_id);

    // Remove current mod directory
    if mod_dir.exists() {
        std::fs::remove_dir_all(&mod_dir)?;
    }

    // Restore from backup
    std::fs::create_dir_all(&mod_dir)?;
    FileUtils::copy_recursive(&backup_dir, &mod_dir)?;

    // Rebuild the ModFS for the restored mod
    let restored_fs = ModFS::new(&mod_dir, &library.spt_rules)?;

    // Update cache with restored files
    library.cache.add(&mod_dir, restored_fs.clone());

    // Update mod metadata if needed
    if let Some(mod_entry) = library.mods.get_mut(mod_id) {
        mod_entry.mod_type = restored_fs.mod_type.clone();
    }

    library.mark_dirty();
    library.persist()?;
    Ok(())
}

/// Removes all backups for a given mod.
pub fn remove_all_backups(lib_paths: &LibPathRules, mod_id: &str) -> Result<(), SError> {
    let backup_dir = lib_paths.backups.join(mod_id);

    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir)?;
    }

    Ok(())
}
