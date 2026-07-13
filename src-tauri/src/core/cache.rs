use crate::core::mod_fs::{self, ModFS};
use crate::models::error::SError;
use crate::models::paths::SPTPathRules;
use crate::utils::file;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LibraryCache {
    pub mods: BTreeMap<String, ModFS>,
}

/// Represents a mod folder that was renamed to match its resolved ID.
pub struct RenamedMod {
    pub old_name: String,
    pub new_name: String,
    pub was_active: bool,
}

/// Result of normalizing mod folder names.
pub struct NormalizationResult {
    pub renamed: Vec<RenamedMod>,
}

/// Detects folder name vs resolved ID mismatches and renames folders.
/// Returns list of renames performed for caller to clean up orphaned data.
pub fn normalize_mod_folders(
    mods_base: &Utf8PathBuf,
    spt_paths: &SPTPathRules,
    mods_state: &BTreeMap<String, crate::models::mod_dto::Mod>,
) -> Result<NormalizationResult, SError> {
    let mut renamed = Vec::new();

    let entries: Vec<_> = file::read_dir(mods_base)?
        .flatten()
        .filter_map(|e| {
            Utf8PathBuf::from_path_buf(e.path())
                .ok()
                .filter(|p| p.is_dir())
        })
        .collect();

    for path in entries {
        let folder_name = path.file_name().ok_or(SError::Unexpected)?.to_string();

        let mod_fs = mod_fs::scan(&path, spt_paths)?;
        let resolved_id = &mod_fs.id;

        if folder_name != *resolved_id {
            let new_path = mods_base.join(resolved_id);

            // Check for conflict
            if new_path.exists() {
                return Err(SError::ModIdConflict(
                    folder_name.clone(),
                    resolved_id.clone(),
                ));
            }

            // Get enabled state before rename
            let was_active = mods_state
                .get(&folder_name)
                .map(|m| m.is_active)
                .unwrap_or(false);

            // Rename folder
            file::rename(&path, &new_path)?;

            renamed.push(RenamedMod {
                old_name: folder_name,
                new_name: resolved_id.clone(),
                was_active,
            });
        }
    }

    Ok(NormalizationResult { renamed })
}

impl LibraryCache {
    pub fn build(mods_base: &Utf8PathBuf, spt_paths: &SPTPathRules) -> Result<Self, SError> {
        let mut cache = Self::default();

        let entries = file::read_dir(mods_base)?;

        for entry in entries.flatten() {
            let path = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|p| SError::ParseError(p.to_string_lossy().to_string()))?;
            if !path.is_dir() {
                return Err(SError::Unexpected);
            }

            cache.add(mod_fs::scan(&path, spt_paths)?);
        }

        Ok(cache)
    }

    pub fn add(&mut self, fs: ModFS) {
        self.mods.insert(fs.id.clone(), fs);
    }
}
