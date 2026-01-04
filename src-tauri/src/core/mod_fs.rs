use crate::models::divider::MOD_ID_DIVIDER;
use crate::models::error::SError;
use crate::models::mod_dto::{ModManifest, ModType};
use crate::models::paths::{LibPaths, ModPaths, SPTPaths};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

// Internal cache representation: includes files but NOT sent to frontend
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModFS {
    pub id: String,
    pub mod_type: ModType,
    pub files: Vec<Utf8PathBuf>,
}

impl ModFS {
    fn read_manifest_guid(manifest_path: &Utf8Path) -> Result<String, SError> {
        Ok(Self::read_manifest(manifest_path)?.guid)
    }

    pub fn read_manifest(path: &Utf8Path) -> Result<ModManifest, SError> {
        Ok(serde_json::from_reader(std::fs::File::open(path)?)?)
    }

    pub fn resolve_id(
        mod_root: &Utf8Path,
        spt_paths: &SPTPaths,
        files: &[Utf8PathBuf],
    ) -> Result<String, SError> {
        // 1. Priority 1: Manifest check
        let mod_paths = ModPaths::new(mod_root);
        if let Ok(guid) = Self::read_manifest_guid(&mod_paths.file) {
            return Ok(guid);
        }

        // 2. Single-pass collection using BTreeSet for automatic sorting
        let ids: std::collections::BTreeSet<String> = files
            .iter()
            .filter_map(|path| {
                // Server check
                if let Ok(rel) = path.strip_prefix(&spt_paths.server_mods) {
                    return rel.components().next().map(|c| c.as_str().to_string());
                }

                // Client check (DLLs only)
                if path.extension() == Some("dll") {
                    if let Ok(rel) = path.strip_prefix(&spt_paths.client_plugins) {
                        return Some(rel.as_str().to_string());
                    }
                }

                None // Ignore file if it matches neither
            })
            .collect();

        if ids.is_empty() {
            return Err(SError::UnableToDetermineModId);
        }

        // 3. Final join (ids is already sorted because it's a BTreeSet)
        Ok(ids.into_iter().collect::<Vec<_>>().join(MOD_ID_DIVIDER))
    }

    pub fn infer_mod_type(files: &[Utf8PathBuf], config: &SPTPaths) -> ModType {
        let has_client = files.iter().any(|p| p.starts_with(&config.client_plugins));
        let has_server = files.iter().any(|p| p.starts_with(&config.server_mods));

        match (has_client, has_server) {
            (true, true) => ModType::Both,
            (true, false) => ModType::Client,
            (false, true) => ModType::Server,
            _ => ModType::Unknown,
        }
    }

    fn collect_files(base: &Utf8Path) -> Vec<Utf8PathBuf> {
        let manifest_folder_name = ModPaths::default().folder;
        WalkDir::new(base)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| {
                let path = Utf8PathBuf::from_path_buf(e.path().to_path_buf()).ok()?;
                let rel = path.strip_prefix(base).ok()?;
                // Skip files inside the 'manifest' directory
                if rel.starts_with(&manifest_folder_name) {
                    return None;
                }
                Some(rel.to_path_buf())
            })
            .collect()
    }

    pub fn new(root: &Utf8Path, spt_paths: &SPTPaths) -> Result<Self, SError> {
        Ok(ModFS {
            id: Self::resolve_id(root, &spt_paths, &Self::collect_files(root))?,
            mod_type: Self::infer_mod_type(&Self::collect_files(root), &spt_paths),
            files: Self::collect_files(root),
        })
    }
}
