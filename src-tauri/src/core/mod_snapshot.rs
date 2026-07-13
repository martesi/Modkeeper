//! Internal upgrade-safety only (§7f) - unreachable from commands.
//! Before an existing mod's folder is overwritten, one transient snapshot is
//! taken; a successful overwrite discards it, a failed one restores from it.
//! At most one snapshot exists per mod.

use crate::core::library::Library;
use crate::models::error::SError;
use crate::models::paths::LibPathRules;
use crate::utils::file;
use camino::Utf8PathBuf;

fn snapshot_dir(lib_paths: &LibPathRules, mod_id: &str) -> Utf8PathBuf {
    lib_paths.backups.join(mod_id).join("snapshot")
}

fn config_file(library: &Library, mod_id: &str) -> Utf8PathBuf {
    library
        .game_root
        .join(&library.spt_rules.client_config)
        .join(format!("{}.cfg", mod_id))
}

/// Snapshots the mod's current folder (and its game config file, if any).
pub fn take(library: &Library, mod_id: &str) -> Result<Utf8PathBuf, SError> {
    let snap_dir = snapshot_dir(&library.lib_paths, mod_id);
    if snap_dir.exists() {
        file::remove_dir_all(&snap_dir)?;
    }

    let content = snap_dir.join("content");
    file::create_dir_all(&content)?;
    file::copy_recursive(&library.lib_paths.mods.join(mod_id), &content)?;

    let game_config = config_file(library, mod_id);
    if game_config.exists() {
        let config_dir = snap_dir.join("config");
        file::create_dir_all(&config_dir)?;
        file::copy(&game_config, &config_dir.join(format!("{}.cfg", mod_id)))?;
    }

    Ok(snap_dir)
}

/// Puts the mod folder (and config file) back exactly as it was, then
/// discards the snapshot.
pub fn restore(library: &Library, mod_id: &str) -> Result<(), SError> {
    let snap_dir = snapshot_dir(&library.lib_paths, mod_id);
    let mod_dir = library.lib_paths.mods.join(mod_id);

    if mod_dir.exists() {
        file::remove_dir_all(&mod_dir)?;
    }
    file::create_dir_all(&mod_dir)?;
    file::copy_recursive(&snap_dir.join("content"), &mod_dir)?;

    let saved_config = snap_dir.join("config").join(format!("{}.cfg", mod_id));
    if saved_config.exists() {
        let game_config = config_file(library, mod_id);
        if let Some(parent) = game_config.parent() {
            file::create_dir_all(parent)?;
        }
        file::copy(&saved_config, &game_config)?;
    }

    discard(library, mod_id)
}

/// Drops the snapshot after a successful overwrite.
pub fn discard(library: &Library, mod_id: &str) -> Result<(), SError> {
    cleanup(&library.lib_paths, mod_id)
}

/// Defensive cleanup of any leftover snapshot dir (crash backstop) - also
/// called by remove_mod.
pub fn cleanup(lib_paths: &LibPathRules, mod_id: &str) -> Result<(), SError> {
    let mod_backup_dir = lib_paths.backups.join(mod_id);
    if mod_backup_dir.exists() {
        file::remove_dir_all(&mod_backup_dir)?;
    }
    Ok(())
}
