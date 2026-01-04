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

    pub fn copy_recursive(src: &Utf8Path, dst: &Utf8Path) -> Result<(), SError> {
        // 1. Ensure the root destination directory exists
        std::fs::create_dir_all(dst)?;

        for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
            // 2. Convert standard Path to Camino Utf8Path
            let src_path = Utf8Path::from_path(entry.path())
                .ok_or_else(|| SError::ParseError(format!("Invalid UTF-8 path: {:?}", entry.path())))?;

            // 3. Calculate the relative path from the source root
            let rel_path = src_path
                .strip_prefix(src)?;

            // 4. Construct the final destination path
            let dst_path = dst.join(rel_path);

            if entry.file_type().is_dir() {
                // 5. If it's a directory, create it in the destination
                std::fs::create_dir_all(&dst_path)?;
            } else {
                // 6. If it's a file, ensure the parent directory exists (safety check)
                if let Some(parent) = dst_path.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                // 7. Copy the file (Note: This overwrites existing files at the destination)
                std::fs::copy(src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    pub fn new(root: &Utf8Path, spt_paths: &SPTPaths) -> Result<Self, SError> {
        let files = Self::collect_files(root); // Call once

        Ok(ModFS {
            id: Self::resolve_id(root, &spt_paths, &files)?,
            mod_type: Self::infer_mod_type(&files, &spt_paths),
            files, // Use the same vector
        })
    }
}
