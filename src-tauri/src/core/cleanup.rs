use crate::core::cache::LibraryCache;
use crate::core::linker::Linker;
use crate::models::error::SError;
use crate::models::paths::{LibPathRules, SPTPathRules};
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::HashSet;
use file_id::FileId;
use walkdir::WalkDir;

pub struct Cleaner<'a> {
    game_root: &'a Utf8Path,
    repo_root: &'a Utf8Path,
    spt_rules: &'a SPTPathRules,
    lib_paths: &'a LibPathRules,
}

impl<'a> Cleaner<'a> {
    pub fn new(
        game_root: &'a Utf8Path,
        repo_root: &'a Utf8Path,
        spt_rules: &'a SPTPathRules,
        lib_paths: &'a LibPathRules,
    ) -> Self {
        Self {
            game_root,
            repo_root,
            spt_rules,
            lib_paths,
        }
    }

    /// Scans the game directory and removes any files, links, or empty folders
    /// that belong to the managed library using a Whitelist approach.
    pub fn purge(&self, cache: &LibraryCache) -> Result<(), SError> {
        let managed_scope = self.build_managed_scope(cache);
        let managed_ids = self.build_managed_ids(cache);

        let roots = [
            self.game_root.join(&self.spt_rules.server_mods),
            self.game_root.join(&self.spt_rules.client_plugins),
        ];

        for root in roots.iter().filter(|r| r.exists()) {
            let mut it = WalkDir::new(root).contents_first(false).into_iter();

            while let Some(entry) = it.next() {
                let entry = entry.map_err(|e| SError::IOError(e.to_string()))?;
                let path = Utf8Path::from_path(entry.path()).ok_or(SError::Unexpected)?;

                if path == root {
                    continue;
                }

                // If processing returns true, we should skip children (e.g., removed a folder)
                if self.process_entry(path, &managed_scope, &managed_ids, &entry)? {
                    it.skip_current_dir();
                }
            }
        }
        Ok(())
    }

    fn process_entry(
        &self,
        path: &Utf8Path,
        managed_scope: &HashSet<Utf8PathBuf>,
        managed_ids: &HashSet<FileId>,
        entry: &walkdir::DirEntry,
    ) -> Result<bool, SError> {
        let meta = entry.path().symlink_metadata()?;

        // Case A: Managed Junctions/Symlinks
        if !meta.is_file() {
            if let Ok(target) = Linker::read_link_target(path) {
                if target.starts_with(self.repo_root) {
                    Linker::unlink(path)?;
                    return Ok(true); // Skip children
                }
            }
        }

        // Case B: Managed Hardlinks
        if meta.is_file() {
            if let Ok(id) = Linker::get_id(path) {
                if managed_ids.contains(&id) {
                    Linker::unlink(path)?;
                }
            }
        }

        // Case C: Ancestor-only Empty Directory Cleanup
        if meta.is_dir() && !meta.file_type().is_symlink() {
            let rel_path = path.strip_prefix(self.game_root).unwrap_or(path);
            if self.is_dir_empty(path) && managed_scope.contains(rel_path) {
                let _ = std::fs::remove_dir(path);
            }
        }

        Ok(false)
    }

    fn build_managed_scope(&self, cache: &LibraryCache) -> HashSet<Utf8PathBuf> {
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

    fn build_managed_ids(&self, cache: &LibraryCache) -> HashSet<FileId> {
        cache
            .mods
            .iter()
            .flat_map(|(id, fs)| {
                fs.files
                    .iter()
                    .map(move |f| self.lib_paths.mods.join(id).join(f))
            })
            .filter_map(|p| Linker::get_id(&p).ok())
            .collect()
    }

    fn is_dir_empty(&self, path: &Utf8Path) -> bool {
        std::fs::read_dir(path)
            .map(|mut i| i.next().is_none())
            .unwrap_or(false)
    }
}