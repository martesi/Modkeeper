use crate::core::cache::LibraryCache;
use crate::core::linker;
use crate::models::error::SError;
use crate::models::mod_dto::Mod;
use crate::models::paths::{LibPathRules, SPTPathRules};
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

type OwnershipMap = HashMap<Utf8PathBuf, Vec<String>>;

/// Entry point for deployment logic.
/// Performs conflict detection and recursive linking of active mods.
pub fn deploy(
    game_root: &Utf8Path,
    lib_paths: &LibPathRules,
    spt_rules: &SPTPathRules,
    mods: &BTreeMap<String, Mod>,
    cache: &LibraryCache,
) -> Result<(), SError> {
    check_file_collisions(mods, cache)?;

    let folder_ownership = build_folder_ownership_map(spt_rules, mods, cache);

    execute_recursive_link(game_root, lib_paths, mods, cache, &folder_ownership)
}

/// Validates that no two active mods provide the same file.
fn check_file_collisions(mods: &BTreeMap<String, Mod>, cache: &LibraryCache) -> Result<(), SError> {
    let mut owners: HashMap<Utf8PathBuf, String> = HashMap::new();
    let mut collisions = BTreeSet::new();

    for (path, current_id) in iter_active_files(mods, cache) {
        let Some(existing_owner) = owners.insert(path.to_owned(), current_id.to_string()) else {
            continue;
        };

        if existing_owner != current_id {
            collisions.insert(format!(
                "File Conflict: '{}' is provided by both '{}' and '{}'.",
                path, existing_owner, current_id
            ));
        }
    }

    if collisions.is_empty() {
        return Ok(());
    }

    Err(SError::FileCollision(collisions.into_iter().collect()))
}

fn build_folder_ownership_map(
    spt_rules: &SPTPathRules,
    mods: &BTreeMap<String, Mod>,
    cache: &LibraryCache,
) -> OwnershipMap {
    // 1. Initialize with System roots (e.g., user/mods, BepInEx/plugins)
    let mut acc: OwnershipMap = [&spt_rules.server_mods, &spt_rules.client_plugins]
        .iter()
        .flat_map(|path| path.ancestors())
        .filter(|a| !a.as_str().is_empty() && *a != ".")
        .map(|a| (a.to_path_buf(), vec!["__SYSTEM__".to_string()]))
        .collect();

    // 2. Populate with Mod folder structures
    iter_active_files_and_ancestors(mods, cache).for_each(|(path, id)| {
        let entry = acc.entry(path.to_path_buf()).or_default();
        if !entry.contains(&id.to_string()) {
            entry.push(id.to_string());
        }
    });

    acc
}

fn execute_recursive_link(
    game_root: &Utf8Path,
    lib_paths: &LibPathRules,
    mods: &BTreeMap<String, Mod>,
    cache: &LibraryCache,
    ownership: &OwnershipMap,
) -> Result<(), SError> {
    cache
        .mods
        .iter()
        .filter(|(id, _)| mods.get(*id).map_or(false, |m| m.is_active))
        .flat_map(|(id, m_fs)| m_fs.files.iter().map(move |f| (id, f)))
        .try_for_each(|(id, file_path)| {
            let mut current_path = Utf8PathBuf::new();

            for component in file_path.components() {
                current_path.push(component);

                let owners = ownership.get(&current_path).ok_or_else(|| {
                    SError::ParseError(format!("Missing ownership for '{}'", current_path))
                })?;

                // Case A: Unique Ownership -> Link high level directory/file and exit file loop
                if owners.len() == 1 {
                    let src = lib_paths.mods.join(id).join(&current_path);
                    let dst = game_root.join(&current_path);
                    linker::link(&src, &dst)?;
                    return Ok(());
                }

                // Case B: Shared -> This is a parent directory. Ensure physical dir exists.
                let shared_dir = game_root.join(&current_path);
                if !shared_dir.exists() {
                    std::fs::create_dir_all(&shared_dir)?;
                }
            }
            Ok(())
        })
}

// --- Iteration Helpers ---

fn iter_active_files<'a>(
    mods: &'a BTreeMap<String, Mod>,
    cache: &'a LibraryCache,
) -> impl Iterator<Item = (&'a Utf8Path, &'a str)> {
    cache
        .mods
        .iter()
        .filter(move |(id, _)| mods.get(*id).map_or(false, |m| m.is_active))
        .flat_map(|(id, fs)| fs.files.iter().map(move |f| (f.as_path(), id.as_str())))
}

fn iter_active_files_and_ancestors<'a>(
    mods: &'a BTreeMap<String, Mod>,
    cache: &'a LibraryCache,
) -> impl Iterator<Item = (&'a Utf8Path, &'a str)> {
    cache
        .mods
        .iter()
        .filter(move |(id, _)| mods.get(*id).map_or(false, |m| m.is_active))
        .flat_map(|(id, fs)| {
            fs.files.iter().flat_map(move |f| {
                f.ancestors()
                    .filter(|a| !a.as_str().is_empty() && *a != ".")
                    .map(move |a| (a, id.as_str()))
            })
        })
}

/// Finds all paths that would be linked for a specific mod.
/// Returns the paths that are uniquely owned by this mod (that need unlinking)
/// and the shared directories that were created for this mod's files.
/// Always includes the specified mod in ownership calculation regardless of its active status.
pub fn find_mod_links(
    game_root: &Utf8Path,
    _lib_paths: &LibPathRules,
    spt_rules: &SPTPathRules,
    mods: &BTreeMap<String, Mod>,
    cache: &LibraryCache,
    mod_id: &str,
) -> Result<(HashSet<Utf8PathBuf>, HashSet<Utf8PathBuf>), SError> {
    // Get the mod's files from cache
    let mod_fs = cache
        .mods
        .get(mod_id)
        .ok_or_else(|| SError::ModNotFound(mod_id.to_string()))?;

    // Build ownership map including this mod (treat it as active for ownership calculation)
    let mut folder_ownership = build_folder_ownership_map(spt_rules, mods, cache);

    // Ensure the mod being removed is included in ownership map (regardless of active status)
    // This allows us to find links even if the mod wasn't marked as active
    for file_path in &mod_fs.files {
        for ancestor in file_path.ancestors() {
            if ancestor.as_str().is_empty() || ancestor == "." {
                continue;
            }
            let entry = folder_ownership.entry(ancestor.to_path_buf()).or_default();
            if !entry.contains(&mod_id.to_string()) {
                entry.push(mod_id.to_string());
            }
        }
    }

    let mut unlink_paths = HashSet::new();
    let mut shared_dirs = HashSet::new();

    // Iterate through each file in the mod
    for file_path in &mod_fs.files {
        let mut current_path = Utf8PathBuf::new();

        for component in file_path.components() {
            current_path.push(component);

            let owners = folder_ownership.get(&current_path).ok_or_else(|| {
                SError::ParseError(format!("Missing ownership for '{}'", current_path))
            })?;

            // Case A: Unique Ownership -> This path would be linked
            if owners.len() == 1 && owners.contains(&mod_id.to_string()) {
                let dst = game_root.join(&current_path);
                unlink_paths.insert(dst);
                // We found the link point, exit file loop
                break;
            }

            // Case B: Shared -> This is a parent directory that might need cleanup
            if owners.len() > 1 {
                let shared_dir = game_root.join(&current_path);
                shared_dirs.insert(shared_dir);
            }
        }
    }

    Ok((unlink_paths, shared_dirs))
}
