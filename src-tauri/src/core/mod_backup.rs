use camino::{Utf8Path, Utf8PathBuf};

use crate::core::library::Library;
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::mod_backup::{BackupManifest, ModBackup};
use crate::models::paths::{BackupPathRules, LibPathRules};
use crate::utils::file::FileUtils;
use crate::utils::time::get_unix_timestamp;

/// Creates a backup of a mod at the current timestamp.
pub fn create_backup(library: &Library, mod_id: &str, name: &str) -> Result<(), SError> {
    let mod_dir = library.lib_paths.mods.join(mod_id);

    if !mod_dir.exists() {
        return Ok(()); // Nothing to backup
    }

    let timestamp = get_unix_timestamp().to_string();
    let backup_dir = library.lib_paths.backups.join(mod_id).join(&timestamp);
    let backup_rules = BackupPathRules::new(&backup_dir);

    std::fs::create_dir_all(&backup_rules.content)?;

    let manifest = BackupManifest {
        timestamp: timestamp.clone(),
        name: name.to_string(),
    };
    let manifest_content =
        toml::to_string_pretty(&manifest).map_err(|e| SError::ParseError(e.to_string()))?;
    std::fs::write(&backup_rules.manifest, manifest_content)?;

    FileUtils::copy_recursive(&mod_dir, &backup_rules.content)?;

    let game_config_path = library
        .game_root
        .join(&library.spt_rules.client_config)
        .join(format!("{}.cfg", mod_id));
    if game_config_path.exists() {
        std::fs::create_dir_all(&backup_rules.config)?;
        let backup_config_path = backup_rules.mod_config_file(mod_id);
        std::fs::copy(&game_config_path, &backup_config_path)?;
    }

    Ok(())
}

/// Lists all available backups for a given mod.
/// Returns backups in descending order (newest first).
pub fn list_backups(lib_paths: &LibPathRules, mod_id: &str) -> Result<Vec<ModBackup>, SError> {
    let backup_dir = lib_paths.backups.join(mod_id);

    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&backup_dir)?;
    let mut backups: Vec<ModBackup> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = Utf8PathBuf::from_path_buf(entry.path()).ok()?;

            // Read manifest.toml
            let backup_rules = BackupPathRules::new(&path);
            let manifest = read_manifest(&backup_rules.manifest).ok()?;

            // Check if config folder exists with files
            let has_config = backup_rules.config.exists()
                && std::fs::read_dir(&backup_rules.config)
                    .ok()?
                    .next()
                    .is_some();

            Some(ModBackup {
                timestamp: manifest.timestamp,
                name: manifest.name,
                has_config,
                path,
            })
        })
        .collect();

    // Sort descending (newest first)
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(backups)
}

/// Reads a backup manifest from a manifest.toml file
fn read_manifest(manifest_path: &Utf8Path) -> Result<BackupManifest, SError> {
    let content = std::fs::read_to_string(manifest_path)?;
    toml::from_str(&content).map_err(|e| SError::ParseError(e.to_string()))
}

/// Restores a mod from a backup.
/// If `restore_config` is true and the backup has a config file, it will be restored too.
pub fn restore_backup(
    library: &mut Library,
    mod_id: &str,
    timestamp: &str,
    restore_config: bool,
) -> Result<(), SError> {
    // Verify mod exists
    if !library.mods.contains_key(mod_id) {
        return Err(SError::ModNotFound(mod_id.to_string()));
    }

    let backup_dir = library.lib_paths.backups.join(mod_id).join(timestamp);

    if !backup_dir.exists() {
        return Err(SError::Unexpected);
    }

    let backup_rules = BackupPathRules::new(&backup_dir);
    let mod_dir = library.lib_paths.mods.join(mod_id);

    // Remove current mod directory
    if mod_dir.exists() {
        std::fs::remove_dir_all(&mod_dir)?;
    }

    // Restore mod content from backup
    std::fs::create_dir_all(&mod_dir)?;
    FileUtils::copy_recursive(&backup_rules.content, &mod_dir)?;

    // Restore config if requested and exists
    if restore_config {
        let backup_config_path = backup_rules.mod_config_file(mod_id);
        if backup_config_path.exists() {
            let game_config_dir = library.game_root.join(&library.spt_rules.client_config);
            let game_config_path = game_config_dir.join(format!("{}.cfg", mod_id));
            std::fs::create_dir_all(&game_config_dir)?;
            std::fs::copy(&backup_config_path, &game_config_path)?;
        }
    }

    // Rebuild the ModFS for the restored mod
    let restored_fs = ModFS::new(&mod_dir, &library.spt_rules)?;

    // Update cache with restored files
    library.cache.add(restored_fs.clone());

    // Update mod metadata if needed
    if let Some(mod_entry) = library.mods.get_mut(mod_id) {
        mod_entry.mod_type = restored_fs.mod_type.clone();
    }

    library.mark_dirty();
    library.persist()?;
    Ok(())
}

/// Removes a specific backup for a given mod.
pub fn remove_backup(
    lib_paths: &LibPathRules,
    mod_id: &str,
    timestamp: &str,
) -> Result<(), SError> {
    let backup_dir = lib_paths.backups.join(mod_id).join(timestamp);

    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir)?;
    }

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
