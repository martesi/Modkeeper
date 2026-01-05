use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::mod_dto::{ModManifest, ModType};
use crate::models::paths::{ModPaths, SPTPathRules};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LibraryCache {
    pub mods: BTreeMap<String, ModFS>,
    pub manifests: BTreeMap<String, ModManifest>,
}

impl LibraryCache {
    pub fn build_from_mods(mods_base: &Utf8PathBuf, spt_paths: &SPTPathRules) -> Result<Self, SError> {
        let mut cache = Self::default();

        let entries = std::fs::read_dir(mods_base)?;

        for entry in entries.flatten() {
            let path = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|p| SError::ParseError(p.to_string_lossy().to_string()))?;
            if !path.is_dir() {
                return Err(SError::Unexpected(Some(format!(
                    "Expected mod folder, found file: {path}"
                ))));
            }

            cache.add(&path, ModFS::new(&path, spt_paths)?);
        }

        Ok(cache)
    }

    pub fn add(&mut self, root: &Utf8Path, fs: ModFS) {
        if let Ok(m) = ModFS::read_manifest(root) {
            self.manifests.insert(fs.id.clone(), m);
        }

        self.mods.insert(fs.id.clone(), fs);
    }

    pub fn detect_collisions(
        &self,
        new_files: &[Utf8PathBuf],
        exclude_id: Option<&str>,
    ) -> Result<(), SError> {
        // 1. Put new files into a HashSet for O(1) lookup performance.
        // This ensures that checking collisions is O(Total Library Files)
        // rather than O(Library Files * New Files).
        let new_files_set: std::collections::HashSet<&Utf8PathBuf> = new_files.iter().collect();

        // 2. Use a BTreeSet to collect colliding paths (automatic sorting and deduplication)
        let mut colliding_paths = std::collections::BTreeSet::new();

        for (id, mod_fs) in &self.mods {
            // Skip the mod we are currently updating/excluding
            if Some(id.as_str()) == exclude_id {
                continue;
            }

            for existing_file in &mod_fs.files {
                if new_files_set.contains(existing_file) {
                    colliding_paths.insert(existing_file.to_string());
                }
            }
        }

        if colliding_paths.is_empty() {
            Ok(())
        } else {
            // Return the list of paths that are already owned by other mods
            Err(SError::FileCollision(colliding_paths.into_iter().collect()))
        }
    }
}
