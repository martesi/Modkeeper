use crate::models::error::SError;
use crate::models::mod_dto::{ModManifest, ModType};
use crate::models::paths::{ModPaths, SPTPathRules};
use crate::utils::id::hash_id;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

// Internal cache representation: includes files but NOT sent to frontend
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModFS {
    pub id: String,
    pub mod_type: ModType,
    pub files: Vec<Utf8PathBuf>,
    pub executables: Vec<Utf8PathBuf>,
}

impl ModFS {
    fn read_manifest_guid(manifest_path: &Utf8Path) -> Result<String, SError> {
        Ok(Self::read_manifest(manifest_path)?.id)
    }

    pub fn read_manifest(path: &Utf8Path) -> Result<ModManifest, SError> {
        Ok(serde_json::from_reader(std::fs::File::open(path)?)?)
    }

    pub fn resolve_id(
        mod_root: &Utf8Path,
        spt_paths: &SPTPathRules,
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
                        // Normalize path separators to forward slashes for consistent hashing
                        return Some(rel.as_str().replace('\\', "/").to_string());
                    }
                }

                None // Ignore file if it matches neither
            })
            .collect();

        if ids.is_empty() {
            return Err(SError::UnableToDetermineModId);
        }

        // 3. Concatenate sorted IDs and hash the result
        let concatenated = ids.into_iter().collect::<Vec<_>>().join("").to_lowercase();
        Ok(hash_id(&concatenated))
    }

    pub fn infer_mod_type(files: &[Utf8PathBuf], config: &SPTPathRules) -> ModType {
        let has_client = files.iter().any(|p| p.starts_with(&config.client_plugins));
        let has_server = files.iter().any(|p| p.starts_with(&config.server_mods));

        match (has_client, has_server) {
            (true, true) => ModType::Both,
            (true, false) => ModType::Client,
            (false, true) => ModType::Server,
            _ => ModType::Unknown,
        }
    }

    fn collect_files(base: &Utf8Path) -> (Vec<Utf8PathBuf>, Vec<Utf8PathBuf>) {
        let manifest_folder = ModPaths::default().folder;

        WalkDir::new(base)
            .into_iter()
            // 1. Convert Result<DirEntry> to Option<DirEntry>
            .filter_map(Result::ok)
            // 2. Filter for files only
            .filter(|e| e.path().is_file())
            // 3. Transform to Utf8PathBuf and strip prefix using Result/Option combinators
            .filter_map(|entry| {
                Utf8PathBuf::from_path_buf(entry.path().to_path_buf())
                    .ok()
                    .and_then(|path| path.strip_prefix(base).ok().map(|p| p.to_path_buf()))
            })
            // 4. Remove manifest files
            .filter(|rel| !rel.starts_with(&manifest_folder))
            // 5. Fold into a tuple of (AllFiles, Executables)
            .fold((Vec::new(), Vec::new()), |(mut all, mut exes), path| {
                // Use Option::filter to handle the conditional push without an "if"
                path.extension()
                    .filter(|&ext| ext == "exe")
                    .inspect(|_| exes.push(path.clone()));

                all.push(path);
                (all, exes)
            })
    }

    pub fn new(root: &Utf8Path, spt_paths: &SPTPathRules) -> Result<Self, SError> {
        let (files, executables) = Self::collect_files(root); // Call once

        Ok(ModFS {
            id: Self::resolve_id(root, &spt_paths, &files)?,
            mod_type: Self::infer_mod_type(&files, &spt_paths),
            files, // Use the same vector
            executables,
        })
    }
}
