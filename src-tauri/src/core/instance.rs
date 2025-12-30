use crate::core::cache::InstanceCache;
use crate::core::linker::Linker;
use crate::core::mod_manager::ModManagerInstance;
use crate::models::instance_dto::ModManagerInstanceDTO;
use crate::models::mod_dto::{Mod, ModCache, ModManifest};
use crate::models::paths::SptPathConfig;
use camino::Utf8PathBuf;
use std::collections::BTreeMap;
use sysinfo::System;

use crate::utils::version::read_pe_version;


pub struct Instance {
    pub id: String,
    pub repo_root: Utf8PathBuf,
    pub game_root: Utf8PathBuf,
    pub config: SptPathConfig,
    pub cache: InstanceCache,
    pub spt_version: String,
}

impl Instance {
    pub fn create(repo_root: &Utf8PathBuf, game_root: Option<&Utf8PathBuf>) -> Result<Self, String> {
        for dir in ["mods", "backups", "staging"] {
            std::fs::create_dir_all(repo_root.join(dir)).map_err(|e| e.to_string())?;
        }

        let config = SptPathConfig::default();
        let game_root = game_root.cloned().unwrap_or_else(|| repo_root.clone());
        let spt_version = Self::validate_spt_version(&game_root, &config)?;

        let inst = Self {
            id: repo_root.file_name().map(|s| s.to_string()).unwrap_or_else(|| "unknown".into()),
            repo_root: repo_root.clone(),
            game_root,
            config,
            cache: InstanceCache::default(),
            spt_version,
        };

        inst.persist_cache()?;
        Ok(inst)
    }

    pub fn read_instance_manifest(
        repo_root: &Utf8PathBuf,
    ) -> Result<ModManagerInstanceDTO, String> {
        let manifest_path = repo_root.join("manifest.toml");
        if !manifest_path.to_path_buf().exists() {
            return Err("manifest.toml not found".into());
        }
        let s = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        toml::from_str::<ModManagerInstanceDTO>(&s).map_err(|e| e.to_string())
    }

    pub fn load(repo_root: &Utf8PathBuf) -> Result<Self, String> {
        let dto = Self::read_instance_manifest(repo_root)?;
        let config = SptPathConfig::default();
        let game_root = Utf8PathBuf::from(dto.game_root);
        let spt_version = Self::validate_spt_version(&game_root, &config)?;

        let mut inst = Self {
            id: dto.id,
            repo_root: repo_root.clone(),
            game_root,
            config,
            cache: InstanceCache::new(),
            spt_version,
        };

        inst.load_cache_or_scan()?;
        Ok(inst)
    }

    pub fn scan_repo_internal(&mut self) -> Result<(), String> {
        let mods_base = self.repo_root.join("mods");

        let new_cache = if !mods_base.exists() {
            InstanceCache::default()
        } else {
            InstanceCache::build_from_mods(&mods_base, &self.config)?
        };

        self.cache = new_cache;
        Ok(())
    }

    fn resolve_id(&self, files: &[Utf8PathBuf], manifest: Option<&ModManifest>) -> String {
        manifest
            .and_then(|m| (!m.guid.is_empty()).then(|| m.guid.clone()))
            .or_else(|| {
                files.iter().find_map(|path| {
                    let parts: Vec<_> = path.components().map(|c| c.as_str()).collect();
                    let pos = parts.iter().position(|&p| p == "mods")?;
                    parts.get(pos + 1).map(|s| s.to_string())
                })
            })
            .or_else(|| {
                files.iter()
                    .find(|p| p.extension() == Some("dll"))
                    .and_then(|p| p.file_stem().map(|s| s.to_string()))
            })
            .unwrap_or_else(|| "unknown_mod".into())
    }

    fn validate_mod_add(&self, files: &[Utf8PathBuf]) -> Result<(), String> {
        let mod_folders = Self::detect_mod_folders(files)?;

        if mod_folders.len() > 1 {
            return Err(format!("Multi-mod detected ({} folders). Add individually.", mod_folders.len()));
        }

        let collisions = Self::check_mod_collisions(&self.cache.mods, files)?;
        if !collisions.is_empty() {
            return Err(format!("Collisions detected: {}", collisions.join(", ")));
        }

        Ok(())
    }

    fn stage_files_to_repo(&self, files: &[Utf8PathBuf], target_base: &Utf8PathBuf) -> Result<Vec<Utf8PathBuf>, String> {
        files.iter()
            .try_fold(Vec::new(), |mut acc, src| {
                if src.is_dir() {
                    self.stage_directory(src, target_base, &mut acc)?;
                } else {
                    self.stage_file(src, target_base, &mut acc)?;
                }
                Ok(acc)
            })
    }

    fn stage_directory(&self, src_dir: &Utf8PathBuf, target_base: &Utf8PathBuf, stored: &mut Vec<Utf8PathBuf>) -> Result<(), String> {
        walkdir::WalkDir::new(src_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file())
            .try_for_each(|entry| {
                let path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf()).map_err(|_| "Invalid path")?;
                let rel = path.strip_prefix(src_dir).map_err(|e| e.to_string())?;

                if rel.starts_with("manifest") { return Ok(()); }

                let dst = target_base.join(rel);
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }

                std::fs::copy(&path, &dst).map_err(|e| e.to_string())?;
                stored.push(rel.to_path_buf());
                Ok(())
            })
    }

    fn stage_file(&self, src: &Utf8PathBuf, target_base: &Utf8PathBuf, stored: &mut Vec<Utf8PathBuf>) -> Result<(), String> {
        let filename = src.file_name().ok_or("Invalid filename")?;
        let dst = target_base.join(filename);
        std::fs::copy(src, &dst).map_err(|e| e.to_string())?;
        stored.push(Utf8PathBuf::from(filename));
        Ok(())
    }

    fn detect_mod_folders(files: &[Utf8PathBuf]) -> Result<Vec<String>, String> {
        Self::normalize_mod_paths(files)
            .map(|normalized| {
                let roots: std::collections::HashSet<String> = normalized
                    .iter()
                    .filter_map(|f| {
                        f.components()
                            .next()
                            .and_then(|c| c.as_os_str().to_str())
                            .map(|s| s.to_string())
                    })
                    .collect();
                roots.into_iter().collect()
            })
    }

    fn normalize_mod_paths(files: &[Utf8PathBuf]) -> Result<Vec<Utf8PathBuf>, String> {
        let anchors = ["SPT", "BepInEx", "manifest"];

        Ok(files.iter().map(|f| {
            let parts: Vec<_> = f.components().map(|c| c.as_str()).collect();

            let start_idx = anchors.iter()
                .find_map(|&anchor| parts.iter().position(|&p| p == anchor))
                .map(|idx| if parts[idx] == "SPT" { idx + 1 } else { idx })
                .unwrap_or(0);

            parts[start_idx..].iter().collect::<Utf8PathBuf>()
        }).collect())
    }

    fn check_mod_collisions(existing: &BTreeMap<String, ModCache>, new_files: &[Utf8PathBuf]) -> Result<Vec<String>, String> {
        let new_folders = Self::detect_mod_folders(new_files)?;
        Ok(existing.keys()
            .filter(|id| new_folders.contains(id))
            .cloned()
            .collect())
    }

    fn validate_spt_version(game_root: &Utf8PathBuf, config: &SptPathConfig) -> Result<String, String> {
        let server_dll = game_root.join(&config.server_dll);
        let version = read_pe_version(&server_dll).map_err(|e| e.to_string())?;

        let major = version.split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        if major != 4 {
            return Err(format!("Unsupported SPT version: {} (4.x required)", version));
        }
        Ok(version)
    }
}

impl Instance {
    fn load_cache_or_scan(&mut self) -> Result<(), String> {
        let cache_path = self.repo_root.join("cache.toml");

        let cache = std::fs::read_to_string(&cache_path)
            .ok()
            .and_then(|s| toml::from_str::<InstanceCache>(&s).ok());

        match cache {
            Some(c) => {
                self.cache = c;
                Ok(())
            }
            None => {
                self.scan_repo_internal()
            }
        }
    }

    fn persist_cache(&self) -> Result<(), String> {
        let dto = self.to_dto();
        let manifest_path = self.repo_root.join("manifest.toml");
        if let Ok(t) = toml::to_string(&dto) {
            let _ = std::fs::write(&manifest_path, t);
        }

        let cache_path = self.repo_root.join("cache.toml");
        if let Ok(ct) = toml::to_string(&self.cache) {
            let _ = std::fs::write(&cache_path, ct);
        }

        Ok(())
    }
}

impl ModManagerInstance for Instance {
    fn is_running(&self) -> bool {
        let s = System::new_all();
        let server_name = self.config.server_exe.file_name().unwrap_or_default();
        let client_name = self.config.client_exe.file_name().unwrap_or_default();

        s.processes()
            .values()
            .any(|p| p.name() == server_name || p.name() == client_name)
    }

    fn add_mod(&mut self, files: Vec<Utf8PathBuf>) -> Result<String, String> {
        if self.is_running() {
            return Err("Game is running".into());
        }

        let id = self.resolve_id(&files, None);

        self.validate_mod_add(&files)?;

        let target_base = self.repo_root.join("mods").join(&id);
        std::fs::create_dir_all(&target_base).map_err(|e| e.to_string())?;

        let stored_paths = self.stage_files_to_repo(&files, &target_base)?;
        let mod_type = InstanceCache::infer_mod_type(&stored_paths, &self.config);

        self.cache.mods.insert(
            id.clone(),
            ModCache {
                id: id.clone(),
                is_active: false,
                mod_type,
                files: stored_paths,
            },
        );

        self.persist_cache()?;

        Ok(id)
    }

    fn remove_mod(&mut self, id: &str) -> Result<(), String> {
        if self.is_running() {
            return Err("Game is running".into());
        }

        if let Some(m) = self.cache.mods.remove(id) {
            for f in &m.files {
                let _ = Linker::unlink(&self.game_root.join(f));
            }
            let _ = std::fs::remove_dir_all(self.repo_root.join("mods").join(id));
        }

        self.persist_cache()?;

        Ok(())
    }

    fn deploy_active_mods(&self) -> Result<(), String> {
        if self.is_running() { return Err("Game is running".into()); }

        let errors: Vec<_> = self.cache.mods.values()
            .flat_map(|m| m.files.iter().map(move |f| (m, f)))
            .filter_map(|(m, file_path)| {
                let src = self.repo_root.join("mods").join(&m.id).join(file_path);
                let dst = self.game_root.join(file_path);

                let res = if m.is_active { Linker::link(&src, &dst) } else { Linker::unlink(&dst) };
                res.err().map(|e| format!("{}: {}", file_path, e))
            })
            .collect();

        if errors.is_empty() { Ok(()) } else { Err(errors.join("\n")) }
    }

    fn scan_repo(&mut self) -> Result<(), String> {
        self.scan_repo_internal()
    }

    fn to_dto(&self) -> ModManagerInstanceDTO {
        ModManagerInstanceDTO {
            id: self.id.clone(),
            game_root: self.game_root.to_string(),
            repo_root: self.repo_root.to_string(),
            spt_version: self.spt_version.clone(),
            mods: self
                .cache
                .mods
                .values()
                .map(|mc| Mod {
                    id: mc.id.clone(),
                    is_active: mc.is_active,
                    mod_type: mc.mod_type.clone(),
                })
                .collect(),
        }
    }
}

