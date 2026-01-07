use crate::core::cache::LibraryCache;
use crate::core::linker::Linker;
use crate::models::error::SError;
use crate::models::mod_dto::Mod;
use crate::models::paths::{LibPathRules, SPTPathRules};
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::{BTreeMap, BTreeSet, HashMap};

type OwnershipMap = HashMap<Utf8PathBuf, Vec<String>>;

pub struct Deployer<'a> {
    game_root: &'a Utf8Path,
    lib_paths: &'a LibPathRules,
    spt_rules: &'a SPTPathRules,
}

impl<'a> Deployer<'a> {
    pub fn new(
        game_root: &'a Utf8Path,
        lib_paths: &'a LibPathRules,
        spt_rules: &'a SPTPathRules,
    ) -> Self {
        Self {
            game_root,
            lib_paths,
            spt_rules,
        }
    }

    pub fn deploy(
        &self,
        mods: &BTreeMap<String, Mod>,
        cache: &LibraryCache,
    ) -> Result<(), SError> {
        self.check_file_collisions(mods, cache)?;

        let folder_ownership = self.build_folder_ownership_map(mods, cache);
        self.execute_recursive_link(mods, cache, &folder_ownership)?;

        Ok(())
    }

    /// Validates that no two active mods contain the same file.
    fn check_file_collisions(
        &self,
        mods: &BTreeMap<String, Mod>,
        cache: &LibraryCache,
    ) -> Result<(), SError> {
        let mut owners: HashMap<Utf8PathBuf, String> = HashMap::new();
        let mut collisions = BTreeSet::new();

        let active_files = self.iter_active_files(mods, cache);

        for (path, current_id) in active_files {
            if let Some(existing_owner) = owners.insert(path.to_owned(), current_id.to_string()) {
                if existing_owner != current_id {
                    collisions.insert(format!(
                        "File Conflict: '{}' is provided by both '{}' and '{}'.",
                        path, existing_owner, current_id
                    ));
                }
            }
        }

        if collisions.is_empty() {
            Ok(())
        } else {
            Err(SError::FileCollision(collisions.into_iter().collect()))
        }
    }

    fn build_folder_ownership_map(
        &self,
        mods: &BTreeMap<String, Mod>,
        cache: &LibraryCache,
    ) -> OwnershipMap {
        let system_roots = [&self.spt_rules.server_mods, &self.spt_rules.client_plugins];

        // Initialize with System folders
        let mut acc: OwnershipMap = system_roots
            .iter()
            .flat_map(|path| path.ancestors())
            .filter(|a| !a.as_str().is_empty() && *a != ".")
            .map(|a| (a.to_path_buf(), vec!["__SYSTEM__".to_string()]))
            .collect();

        // Populate with Mod folders
        self.iter_active_files_and_ancestors(mods, cache)
            .for_each(|(path, id)| {
                let entry = acc.entry(path.to_path_buf()).or_default();
                let id_str = id.to_string();
                if !entry.contains(&id_str) {
                    entry.push(id_str);
                }
            });

        acc
    }

    fn execute_recursive_link(
        &self,
        mods: &BTreeMap<String, Mod>,
        cache: &LibraryCache,
        ownership: &OwnershipMap,
    ) -> Result<(), SError> {
        cache
            .mods
            .iter()
            // Filter active
            .filter(|(id, _)| mods.get(*id).map_or(false, |m| m.is_active))
            // Flatten to (ModID, FilePath)
            .flat_map(|(id, m_fs)| m_fs.files.iter().map(move |f| (id, f)))
            .try_for_each(|(id, file_path)| {
                let mut current_path = Utf8PathBuf::new();

                for component in file_path.components() {
                    current_path.push(component);

                    let owners = ownership.get(&current_path).ok_or_else(|| {
                        SError::ParseError(format!("Missing ownership for '{}'", current_path))
                    })?;

                    // Case A: Unique Ownership -> Link high level, stop recursion
                    if owners.len() == 1 {
                        let src = self.lib_paths.mods.join(id).join(&current_path);
                        let dst = self.game_root.join(&current_path);
                        Linker::link(&src, &dst)?;
                        return Ok(());
                    }

                    // Case B: Shared -> Create folder, continue recursion
                    let shared_dir = self.game_root.join(&current_path);
                    if !shared_dir.exists() {
                        std::fs::create_dir_all(&shared_dir)?;
                    }
                }
                Ok(())
            })
    }

    // --- Iteration Helpers ---

    fn iter_active_files<'b>(
        &'b self,
        mods: &'b BTreeMap<String, Mod>,
        cache: &'b LibraryCache,
    ) -> impl Iterator<Item = (&'b Utf8Path, &'b str)> {
        cache
            .mods
            .iter()
            .filter(move |(id, _)| mods.get(*id).map_or(false, |m| m.is_active))
            .flat_map(|(id, fs)| fs.files.iter().map(move |f| (f.as_path(), id.as_str())))
    }

    fn iter_active_files_and_ancestors<'b>(
        &'b self,
        mods: &'b BTreeMap<String, Mod>,
        cache: &'b LibraryCache,
    ) -> impl Iterator<Item = (&'b Utf8Path, &'b str)> {
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
}