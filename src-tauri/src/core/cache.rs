use crate::models::mod_dto::{ModCache, ModManifest, ModType};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct InstanceCache {
    pub mods: BTreeMap<String, ModCache>,
    pub manifests: BTreeMap<String, ModManifest>,
}

impl InstanceCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.mods.is_empty() && self.manifests.is_empty()
    }

    pub fn build_from_mods(
        mods_base: &Utf8PathBuf,
        config: &crate::models::paths::SptPathConfig,
    ) -> Result<Self, String> {
        let mut cache = Self::default();

        let entries = std::fs::read_dir(mods_base).map_err(|e| e.to_string())?;

        for entry in entries.flatten() {
            let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|p| p.to_string_lossy().to_string())?;
            if !path.is_dir() { continue; }

            let id = path.file_name().unwrap_or_default().to_string();

            if let Ok(mod_cache) = Self::scan_mod_folder(&path, &id, config) {
                cache.mods.insert(id.clone(), mod_cache);
                if let Ok(manifest) = Self::load_mod_manifest(mods_base, &id) {
                    cache.manifests.insert(id, manifest);
                }
            }
        }

        Ok(cache)
    }

    fn scan_mod_folder(
        mod_path: &Utf8PathBuf,
        id: &str,
        config: &crate::models::paths::SptPathConfig,
    ) -> Result<ModCache, String> {
        let files = Self::collect_mod_files(mod_path)?;
        let mod_type = Self::infer_mod_type(&files, config);

        Ok(ModCache {
            id: id.to_string(),
            is_active: false,
            mod_type,
            files,
        })
    }

    fn collect_mod_files(base: &Utf8PathBuf) -> Result<Vec<Utf8PathBuf>, String> {
        Ok(WalkDir::new(base)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| {
                let path = Utf8PathBuf::from_path_buf(e.path().to_path_buf()).ok()?;
                let rel = path.strip_prefix(base).ok()?;
                // Skip files inside the 'manifest' directory
                if rel.starts_with("manifest") { return None; }
                Some(rel.to_path_buf())
            })
            .collect())
    }

    pub fn infer_mod_type(
        files: &[Utf8PathBuf],
        config: &crate::models::paths::SptPathConfig,
    ) -> ModType {
        let has_client = files.iter().any(|p| p.starts_with(&config.client_plugins));
        let has_server = files.iter().any(|p| p.starts_with(&config.server_mods));

        match (has_client, has_server) {
            (true, true) => ModType::Both,
            (true, false) => ModType::Client,
            (false, true) => ModType::Server,
            _ => ModType::Unknown,
        }
    }

    fn load_mod_manifest(
        mods_base: &Utf8PathBuf,
        id: &str,
    ) -> Result<ModManifest, String> {
        let manifest_path = mods_base.join(id).join("manifest").join("manifest.json");
        let s = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<ModManifest>(&s).map_err(|e| e.to_string())
    }
}

