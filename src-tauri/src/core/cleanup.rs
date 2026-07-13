use crate::core::deployment;
use crate::core::library::Library;
use crate::core::linker;
use crate::models::error::SError;
use crate::utils::file;
use camino::{Utf8Path, Utf8PathBuf};
use dunce::canonicalize as dunce_canon;
use std::collections::HashSet;
use walkdir::WalkDir;

/// Entry point for the cleanup logic.
/// Scans managed game directories and removes all symlinks pointing back into our repo.
pub fn purge(library: &Library) -> Result<(), SError> {
    let managed_scope = build_managed_scope(library);
    let roots = deployment::get_protected_paths_absolute(&library.game_root, &library.spt_rules);

    // Pre-compute canonical repo_root for robust symlink target comparison
    let canonical_repo_root = dunce_canon(&library.repo_root)
        .map(|p| Utf8PathBuf::from(p.to_string_lossy().into_owned()))
        .unwrap_or_else(|_| library.repo_root.clone());

    for root in roots.iter().filter(|r| r.exists()) {
        let mut it = WalkDir::new(root).contents_first(false).into_iter();

        while let Some(entry) = it.next() {
            let entry = entry.map_err(|e| SError::IOError(e.to_string()))?;
            let path = Utf8Path::from_path(entry.path()).ok_or(SError::Unexpected)?;

            if path == root {
                continue;
            }

            if process_entry(
                path,
                &library.game_root,
                &canonical_repo_root,
                &managed_scope,
                &entry,
            )? {
                it.skip_current_dir();
            }
        }
    }
    Ok(())
}

/// Processes a single filesystem entry.
/// Returns Ok(true) if the entry was removed (signaling to skip children).
fn process_entry(
    path: &Utf8Path,
    game_root: &Utf8Path,
    repo_root: &Utf8Path,
    managed_scope: &HashSet<Utf8PathBuf>,
    entry: &walkdir::DirEntry,
) -> Result<bool, SError> {
    let meta = entry.path().symlink_metadata()?;

    // Case A: Symlinks pointing back to our repo — remove them
    if meta.file_type().is_symlink() {
        let Ok(raw_target) = linker::read_link_target(path) else {
            return Ok(false);
        };

        // Canonicalize the target to handle \\?\ prefixes and case differences on Windows
        let target = dunce_canon(&raw_target)
            .map(|p| Utf8PathBuf::from(p.to_string_lossy().into_owned()))
            .unwrap_or(raw_target);

        if target.starts_with(repo_root) {
            // Check if target is a directory BEFORE unlinking (unlink removes the path)
            let target_is_dir = target.is_dir();
            linker::unlink(path)?;
            // Only skip children if this was a directory symlink.
            // For file symlinks, returning true would cause WalkDir to skip remaining siblings.
            return Ok(target_is_dir);
        }
        return Ok(false);
    }

    // Case B: Ancestor-only empty directory cleanup
    if meta.is_dir() {
        let rel_path = path.strip_prefix(game_root).unwrap_or(path);

        if file::is_dir_empty(path) && managed_scope.contains(rel_path) {
            let _ = file::remove_dir(path);
            return Ok(true);
        }
    }

    Ok(false)
}

/// Builds the set of relative paths that our managed mods have ever contributed to.
/// Used to identify empty directories that are safe to remove.
fn build_managed_scope(library: &Library) -> HashSet<Utf8PathBuf> {
    library
        .cache
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

/// Unlinks all symlinks and shared directories for a specific mod.
pub fn unlink_mod(
    library: &Library,
    mod_id: &str,
    unlink_paths: &HashSet<Utf8PathBuf>,
    shared_dirs: &HashSet<Utf8PathBuf>,
) -> Result<Vec<Utf8PathBuf>, SError> {
    let mut unlinked = Vec::new();
    let mod_source_dir = library.lib_paths.mods.join(mod_id);
    let protected_paths =
        deployment::get_protected_paths_absolute(&library.game_root, &library.spt_rules);

    for path in unlink_paths {
        if protected_paths.iter().any(|protected| path == protected) {
            continue;
        }
        if path.exists() || path.is_symlink() {
            // Only unlink if it's a symlink pointing to our mod's directory
            if let Ok(target) = linker::read_link_target(path) {
                if target.starts_with(&mod_source_dir) {
                    linker::unlink(path)?;
                    unlinked.push(path.clone());
                }
            }
        }
    }

    // Clean up empty shared directories (deepest first)
    let mut sorted_shared_dirs: Vec<_> = shared_dirs.iter().collect();
    sorted_shared_dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

    for shared_dir in sorted_shared_dirs {
        if protected_paths
            .iter()
            .any(|protected| shared_dir == protected)
        {
            continue;
        }
        if shared_dir.exists() && file::is_dir_empty(shared_dir) {
            let _ = file::remove_dir(shared_dir);
            unlinked.push(shared_dir.clone());
        }
    }

    Ok(unlinked)
}
