use crate::core::cache::LibraryCache;
use crate::core::linker;
use crate::models::error::SError;
use crate::models::paths::{LibPathRules, SPTPathRules};
use camino::{Utf8Path, Utf8PathBuf};
use file_id::FileId;
use std::collections::HashSet;
use walkdir::WalkDir;

/// Entry point for the cleanup logic.
/// Scans the game directory and removes managed files, links, or empty folders.
pub fn purge(
    game_root: &Utf8Path,
    repo_root: &Utf8Path,
    spt_rules: &SPTPathRules,
    lib_paths: &LibPathRules,
    cache: &LibraryCache,
) -> Result<(), SError> {
    let managed_scope = build_managed_scope(cache);
    let managed_ids = build_managed_ids(lib_paths, cache);

    let roots = [
        game_root.join(&spt_rules.server_mods),
        game_root.join(&spt_rules.client_plugins),
    ];

    for root in roots.iter().filter(|r| r.exists()) {
        let mut it = WalkDir::new(root).contents_first(false).into_iter();

        while let Some(entry) = it.next() {
            let entry = entry.map_err(|e| SError::IOError(e.to_string()))?;
            let path = Utf8Path::from_path(entry.path()).ok_or(SError::Unexpected)?;

            if path == root {
                continue;
            }

            // Process the entry. If it returns true, we skip children (e.g., directory was removed).
            if process_entry(
                path,
                game_root,
                repo_root,
                &managed_scope,
                &managed_ids,
                &entry,
            )? {
                it.skip_current_dir();
            }
        }
    }
    Ok(())
}

/// Processes a single filesystem entry to determine if it should be unlinked or removed.
/// Returns Ok(true) if the entry was a directory and was removed (signaling to skip children).
fn process_entry(
    path: &Utf8Path,
    game_root: &Utf8Path,
    repo_root: &Utf8Path,
    managed_scope: &HashSet<Utf8PathBuf>,
    managed_ids: &HashSet<FileId>,
    entry: &walkdir::DirEntry,
) -> Result<bool, SError> {
    let meta = entry.path().symlink_metadata()?;

    // Case A: Managed Junctions/Symlinks (pointing back to our repo)
    if !meta.is_file() {
        let Ok(target) = linker::read_link_target(path) else {
            return Ok(false);
        };

        if target.starts_with(repo_root) {
            linker::unlink(path)?;
            return Ok(true);
        }
    }

    // Case B: Managed Hardlinks (matched by physical file ID)
    if meta.is_file() {
        let Ok(id) = linker::get_id(path) else {
            return Ok(false);
        };

        if managed_ids.contains(&id) {
            linker::unlink(path)?;
        }
        return Ok(false);
    }

    // Case C: Ancestor-only Empty Directory Cleanup
    if meta.is_dir() && !meta.file_type().is_symlink() {
        let rel_path = path.strip_prefix(game_root).unwrap_or(path);

        // We only remove the directory if it's empty AND part of our known managed structure
        if is_dir_empty(path) && managed_scope.contains(rel_path) {
            let _ = std::fs::remove_dir(path);
            return Ok(true);
        }
    }

    Ok(false)
}

fn build_managed_scope(cache: &LibraryCache) -> HashSet<Utf8PathBuf> {
    cache
        .mods
        .values()
        .flat_map(|m_fs| {
            m_fs.files
                .iter()
                .flat_map(|f| f.ancestors().map(|a| a.to_path_buf()))
        })
        .filter(|a| !a.as_str().is_empty() && *a != ".")
        .collect()
}

fn build_managed_ids(lib_paths: &LibPathRules, cache: &LibraryCache) -> HashSet<FileId> {
    cache
        .mods
        .iter()
        .flat_map(|(id, fs)| {
            fs.files
                .iter()
                .map(move |f| lib_paths.mods.join(id).join(f))
        })
        .filter_map(|p| linker::get_id(&p).ok())
        .collect()
}

fn is_dir_empty(path: &Utf8Path) -> bool {
    std::fs::read_dir(path)
        .map(|mut i| i.next().is_none())
        .unwrap_or(false)
}

/// Unlinks all files, junctions, and shared directories for a specific mod.
/// Returns the list of paths that were unlinked/removed.
pub fn unlink_mod(
    _game_root: &Utf8Path,
    _repo_root: &Utf8Path,
    lib_paths: &LibPathRules,
    cache: &LibraryCache,
    mod_id: &str,
    unlink_paths: &HashSet<Utf8PathBuf>,
    shared_dirs: &HashSet<Utf8PathBuf>,
) -> Result<Vec<Utf8PathBuf>, SError> {
    let mut unlinked = Vec::new();
    let mod_source_dir = lib_paths.mods.join(mod_id);

    // Get the mod's file IDs for hard link matching
    let mod_file_ids: HashSet<FileId> = cache
        .mods
        .get(mod_id)
        .map(|m_fs| {
            m_fs.files
                .iter()
                .filter_map(|f| linker::get_id(&mod_source_dir.join(f)).ok())
                .collect()
        })
        .unwrap_or_default();

    // Unlink all paths that were uniquely owned by this mod
    for path in unlink_paths {
        if path.exists() || path.is_symlink() {
            // Check if it's a junction/symlink pointing to our mod
            if let Ok(target) = linker::read_link_target(path) {
                if target.starts_with(&mod_source_dir) {
                    linker::unlink(path)?;
                    unlinked.push(path.clone());
                    continue;
                }
            }

            // Check if it's a hard link matching our mod's file IDs
            if let Ok(id) = linker::get_id(path) {
                if mod_file_ids.contains(&id) {
                    linker::unlink(path)?;
                    unlinked.push(path.clone());
                }
            }
        }
    }

    // Clean up empty shared directories (walk from deepest to shallowest)
    let mut sorted_shared_dirs: Vec<_> = shared_dirs.iter().collect();
    sorted_shared_dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

    for shared_dir in sorted_shared_dirs {
        if shared_dir.exists() && is_dir_empty(shared_dir) {
            // Check if this directory is still needed by other mods
            // We can remove it if it's empty and was created for our mod
            let _ = std::fs::remove_dir(shared_dir);
            unlinked.push(shared_dir.clone());
        }
    }

    Ok(unlinked)
}
