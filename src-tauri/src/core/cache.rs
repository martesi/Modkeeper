use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::mod_dto::{ModManifest, ModType};
use crate::models::paths::{ModPaths, SPTPaths};
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
    pub fn build_from_mods(mods_base: &Utf8PathBuf, spt_paths: &SPTPaths) -> Result<Self, SError> {
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

            cache.add(&path, &ModFS::new(&path, spt_paths)?);
        }

        Ok(cache)
    }

    pub fn add(&mut self, root: &Utf8Path, fs: &ModFS) {
        // 1. Always add the mod
        self.mods.insert(fs.id.clone(), fs.clone());

        // 2. Try to add the manifest, ignore failure
        if let Ok(m) = ModFS::read_manifest(root) {
            self.manifests.insert(fs.id.clone(), m);
        }
    }
}
