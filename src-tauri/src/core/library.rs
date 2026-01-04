use crate::core::cache::LibraryCache;
use crate::core::linker::Linker;
use crate::models::divider::MOD_ID_DIVIDER;
use crate::models::error::SError;
use crate::models::instance_dto::ModManagerInstanceDTO;
use crate::models::mod_dto::{Mod, ModCache, ModManifest};
use crate::models::paths::{LibPaths, SPTPaths};
use crate::utils::version::read_pe_version;
use crate::utils::toml::Toml;
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::{BTreeMap, HashSet};
use sysinfo::System;

pub struct Library {
    pub id: String,
    pub repo_root: Utf8PathBuf,
    pub game_root: Utf8PathBuf,
    pub spt_paths: SPTPaths,
    pub lib_paths: LibPaths,
    pub cache: LibraryCache,
    pub spt_version: String,
}

impl Library {
    pub fn create(repo_root: &Utf8PathBuf, game_root: &Utf8PathBuf) -> Result<Self, SError> {
        for dir in ["mods", "backups", "staging"] {
            std::fs::create_dir_all(repo_root.join(dir))
                .map_err(|e| SError::IOError(e.to_string()))?;
        }

        let config = SPTPaths::new(game_root);
        let spt_version = Library::validate_spt_version(game_root, &config)?;

        let inst = Self {
            id: uuid::Uuid::new_v4().to_string(),
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            spt_paths: config,
            cache: LibraryCache::default(),
            lib_paths: LibPaths::default(),
            spt_version,
        };

        inst.persist_cache()?;
        Ok(inst)
    }

    pub fn read_instance_manifest(
        repo_root: &Utf8PathBuf,
    ) -> Result<ModManagerInstanceDTO, SError> {
        let manifest_path = repo_root.join(LibPaths::default().manifest);
        if !manifest_path.to_path_buf().exists() {
            return Err(SError::FileOrDirectoryNotFound(repo_root.to_string()));
        }
        Toml::read::<ModManagerInstanceDTO>(&manifest_path)
    }

    pub fn load(repo_root: &Utf8PathBuf) -> Result<Self, SError> {
        let dto = Self::read_instance_manifest(repo_root)?;
        let config = SPTPaths::default();
        let game_root = Utf8PathBuf::from(dto.game_root);
        let spt_version = Self::validate_spt_version(&game_root, &config)?;

        let mut inst = Self {
            id: dto.id,
            repo_root: repo_root.clone(),
            game_root,
            spt_paths: config,
            lib_paths: LibPaths::default(),
            cache: LibraryCache::new(),
            spt_version,
        };

        inst.cache = inst.scan_repo_internal()?;

        Ok(inst)
    }

    pub fn scan_repo_internal(&mut self) -> Result<LibraryCache, SError> {
        let mods_base = self.repo_root.join(self.lib_paths.mods.as_str());

        let new_cache = if !mods_base.exists() {
            LibraryCache::default()
        } else {
            LibraryCache::build_from_mods(&mods_base, &self.spt_paths)?
        };

        Ok(new_cache)
    }

    /**
    @hint
    instead of requesting manifest from outside, manifest should be fetched from a fixed position
    at {config.manifest_file}; If file can be parsed, guid is used as id;
    if this file is not present, fallback to get all folders under {server_mods}, join them with {config.id_divider}
    and used as id;
    if no folder found, fallback to get all folder/file names under {client_plugins} joined with {config.id_divider} as id;
    Err returned if unsolved.
    */
    fn resolve_id(&self, files: &[Utf8PathBuf]) -> Result<String, SError> {
        self.read_manifest_guid()
            .or_else(|_| self.collect_ids_from_path(files, &self.spt_paths.server_mods))
            .or_else(|_| self.collect_ids_from_path(files, &self.spt_paths.client_plugins))
            .map_err(|_| SError::UnableToDetermineModId)
    }

    fn read_manifest_guid(&self) -> Result<String, String> {
        let manifest_path = self.repo_root.join(&self.lib_paths.manifest);
        std::fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|json| serde_json::from_str::<ModManifest>(&json).ok())
            .filter(|m| !m.guid.is_empty())
            .map(|m| m.guid)
            .ok_or_else(|| "manifest guid not found".into())
    }

    fn collect_ids_from_path(
        &self,
        files: &[Utf8PathBuf],
        search_path: &Utf8PathBuf,
    ) -> Result<String, String> {
        let search_str = search_path.to_string();
        let search_parts: Vec<&str> = search_str.split('/').collect();
        let ids: Vec<String> = files
            .iter()
            .filter_map(|path| self.extract_id_after_path(path, &search_parts))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        (!ids.is_empty())
            .then(|| ids.join(MOD_ID_DIVIDER))
            .ok_or_else(|| "No ids found in path".into())
    }

    fn extract_id_after_path(&self, path: &Utf8PathBuf, search_parts: &[&str]) -> Option<String> {
        let parts: Vec<&str> = path.components().map(|c| c.as_str()).collect();
        (0..=parts.len().saturating_sub(search_parts.len()))
            .find(|&i| &parts[i..i + search_parts.len()] == search_parts)
            .and_then(|idx| parts.get(idx + search_parts.len()))
            .map(|s| s.to_string())
    }

    fn validate_mod_add(&self, files: &[Utf8PathBuf], mod_id: &str) -> Result<(), SError> {
        let mods_to_check: BTreeMap<String, ModCache> = self
            .cache
            .mods
            .iter()
            .filter(|(id, _)| *id != mod_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Self::check_mod_collisions(&mods_to_check, files)?
            .into_iter()
            .collect::<Vec<_>>()
            .is_empty()
            .then_some(())
            .ok_or_else(|| {
                SError::FileCollision(
                    Self::check_mod_collisions(&mods_to_check, files).unwrap_or_default(),
                )
            })
    }

    fn stage_files_to_repo(
        &self,
        files: &[Utf8PathBuf],
        target_base: &Utf8PathBuf,
    ) -> Result<Vec<Utf8PathBuf>, SError> {
        files.iter().try_fold(Vec::new(), |mut acc, src| {
            if src.is_dir() {
                self.stage_directory(src, target_base, &mut acc)?;
            } else {
                self.stage_file(src, target_base, &mut acc)?;
            }
            Ok(acc)
        })
    }

    fn stage_directory(
        &self,
        src_dir: &Utf8PathBuf,
        target_base: &Utf8PathBuf,
        stored: &mut Vec<Utf8PathBuf>,
    ) -> Result<(), SError> {
        walkdir::WalkDir::new(src_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file())
            .try_for_each(|entry| {
                let path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf())
                    .map_err(|_| SError::ParseError(entry.path().to_string_lossy().to_string()))?;
                let rel = path
                    .strip_prefix(src_dir)
                    .map_err(|e| SError::ParseError(e.to_string()))?;

                rel.starts_with("manifest")
                    .then_some(())
                    .map(Some)
                    .unwrap_or_else(|| {
                        self.copy_and_store_file(&path, target_base, rel, stored)
                            .ok()
                    });
                Ok(())
            })
    }

    fn copy_and_store_file(
        &self,
        src: &Utf8PathBuf,
        target_base: &Utf8PathBuf,
        rel: &Utf8Path,
        stored: &mut Vec<Utf8PathBuf>,
    ) -> Result<(), String> {
        let dst = target_base.join(rel);
        dst.parent()
            .map(|parent| std::fs::create_dir_all(parent).map_err(|e| e.to_string()))
            .transpose()?;

        std::fs::copy(src, &dst).map_err(|e| e.to_string())?;
        stored.push(rel.to_path_buf());
        Ok(())
    }

    fn stage_file(
        &self,
        src: &Utf8PathBuf,
        target_base: &Utf8PathBuf,
        stored: &mut Vec<Utf8PathBuf>,
    ) -> Result<(), SError> {
        let filename = src.file_name().ok_or(SError::ParseError(src.to_string()))?;
        let dst = target_base.join(filename);
        std::fs::copy(src, &dst).map_err(|e| SError::IOError(e.to_string()))?;
        stored.push(Utf8PathBuf::from(filename));
        Ok(())
    }

    fn detect_mod_folders(files: &[Utf8PathBuf]) -> Result<Vec<String>, SError> {
        let folders: HashSet<String> = Self::normalize_mod_paths(files)
            .into_iter()
            .filter_map(|f| {
                f.components()
                    .next() // Get the first part of the path (the top folder)
                    .map(|c| c.as_str().to_string())
            })
            .collect();

        // Wrap the resulting Vec in Ok() to match the return type
        Ok(folders.into_iter().collect())
    }

    /**
    @hint
    A server mod starts as a folder within {config.server_mods}, a client mod starts as a folder/file within {config.client_plugins}.
    We normalize all paths by removing leading parts.
    manifest is a mod self-description folder, as in {config.manifest_folder}, should be ignored.
    Remove anchors here as it's not the correct standing for paths to strip.
    */
    fn normalize_mod_paths(files: &[Utf8PathBuf]) -> Vec<Utf8PathBuf> {
        files
            .iter()
            .filter(|f| f.components().find(|c| c.as_str() == "manifest").is_none())
            .cloned()
            .collect()
    }

    fn check_mod_collisions(
        existing: &BTreeMap<String, ModCache>,
        new_files: &[Utf8PathBuf],
    ) -> Result<Vec<String>, SError> {
        let new_folders = Self::detect_mod_folders(new_files)?;
        Ok(existing
            .keys()
            .filter(|id| new_folders.contains(id))
            .cloned()
            .collect())
    }

    fn validate_spt_version(game_root: &Utf8PathBuf, config: &SPTPaths) -> Result<String, SError> {
        read_pe_version(&game_root.join(&config.server_dll))
            .map_err(SError::ParseError)
            .and_then(|v| {
                let major = v.split('.').next().and_then(|s| s.parse::<u32>().ok());
                if major == Some(4) {
                    Ok(v)
                } else {
                    Err(SError::UnsupportedSPTVersion(v))
                }
            })
    }

    fn is_running(&self) -> bool {
        let s = System::new_all();
        let server_name = self.spt_paths.server_exe.file_name().unwrap_or_default();
        let client_name = self.spt_paths.client_exe.file_name().unwrap_or_default();

        s.processes()
            .values()
            .any(|p| p.name() == server_name || p.name() == client_name)
    }

    pub fn add_mod(&mut self, files: Vec<Utf8PathBuf>) -> Result<(), SError> {
        if self.is_running() {
            return Err(SError::GameOrServerRunning);
        }

        let id = self.resolve_id(&files)?;

        /*
          @hint
          remove this method, use check_mod_collisions directly;
          give a cloned mods cache without the current mod id to check collisions;
        */
        self.validate_mod_add(&files, &id)?;

        let target_base = self.repo_root.join("mods").join(&id);
        std::fs::create_dir_all(&target_base).map_err(|e| SError::IOError(e.to_string()))?;

        let stored_paths = self.stage_files_to_repo(&files, &target_base)?;
        let mod_type = LibraryCache::infer_mod_type(&stored_paths, &self.spt_paths);

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

        // @TODO return Mod instead
        Ok(())
    }

    pub fn remove_mod(&mut self, id: &str) -> Result<(), SError> {
        if self.is_running() {
            return Err(SError::GameOrServerRunning);
        }

        if let Some(m) = self.cache.mods.remove(id) {
            m.files.iter().for_each(|f| {
                let _ = Linker::unlink(&self.game_root.join(f));
            });
            let _ = std::fs::remove_dir_all(self.repo_root.join("mods").join(id));
        }

        self.persist_cache()?;

        Ok(())
    }

    pub fn deploy_active_mods(&self) -> Result<(), SError> {
        if self.is_running() {
            return Err(SError::GameOrServerRunning);
        }

        let errors: Vec<_> = self
            .cache
            .mods
            .values()
            .flat_map(|m| m.files.iter().map(move |f| (m, f)))
            .filter_map(|(m, file_path)| {
                let src = self.repo_root.join("mods").join(&m.id).join(file_path);
                let dst = self.game_root.join(file_path);
                let res = if m.is_active {
                    Linker::link(&src, &dst)
                } else {
                    Linker::unlink(&dst)
                };
                res.err().map(|e| e.to_string())
            })
            .collect();

        errors
            .is_empty()
            .then_some(())
            .ok_or_else(|| SError::Link())
    }

    pub fn scan_repo(&mut self) -> Result<(), SError> {
        self.scan_repo_internal()
            .inspect(|v| self.cache = v.clone())
            .and_then(|_| Ok(()))
    }

    pub fn to_dto(&self) -> ModManagerInstanceDTO {
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

    fn persist_cache(&self) -> Result<(), SError> {
        let dto = self.to_dto();
        Toml::write(&self.repo_root.join(&self.lib_paths.manifest), &dto)?;
        Toml::write(&self.repo_root.join(&self.lib_paths.cache), &self.cache)?;
        Ok(())
    }
}
