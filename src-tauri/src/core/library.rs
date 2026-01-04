use crate::core::cache::LibraryCache;
use crate::core::linker::Linker;
use crate::models::divider::MOD_ID_DIVIDER;
use crate::models::error::SError;
use crate::models::library_dto::LibraryDTO;
use crate::models::mod_dto::{Mod, ModCache, ModManifest};
use crate::models::paths::{LibPaths, SPTPaths};
use crate::utils::toml::Toml;
use crate::utils::version::read_pe_version;
use camino::{Utf8Path, Utf8PathBuf};
use semver::{Version, VersionReq};
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
    pub mods: BTreeMap<String, Mod>,
}

impl Library {
    pub fn create(repo_root: &Utf8PathBuf, game_root: &Utf8PathBuf) -> Result<Self, SError> {
        let lib_paths = LibPaths::new(game_root);
        for dir in [&lib_paths.mods, &lib_paths.backups, &lib_paths.staging] {
            std::fs::create_dir_all(dir).map_err(|e| SError::IOError(e.to_string()))?;
        }

        let config = SPTPaths::new(game_root);

        let inst = Self {
            id: uuid::Uuid::new_v4().to_string(),
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            spt_version: Library::fetch_and_validate_spt_version(&config)?,
            cache: LibraryCache::default(),
            mods: Default::default(),
            lib_paths,
            spt_paths: config,
        };

        inst.persist()?;
        Ok(inst)
    }

    pub fn load(repo_root: &Utf8PathBuf) -> Result<Self, SError> {
        let dto = Self::read_library_manifest(repo_root)?;
        // check the original spt_version when library is created
        // if not valid, return error directly
        Self::parse_spt_version(&dto.spt_version)
            .and_then(|spt_version| Self::validate_spt_version(&spt_version))?;

        let config = SPTPaths::new(repo_root);
        // When displaying, always use the current spt version
        let spt_version = Self::fetch_and_validate_spt_version(&config)?;
        let lib_paths = LibPaths::new(repo_root);
        let inst = Self {
            id: dto.id,
            repo_root: repo_root.clone(),
            game_root: dto.game_root,
            spt_paths: config,
            cache: Toml::read(&lib_paths.cache)?,
            lib_paths,
            spt_version,
            mods: dto.mods,
        };

        Ok(inst)
    }

    pub fn read_library_manifest(lib_root: &Utf8PathBuf) -> Result<LibraryDTO, SError> {
        Toml::read::<LibraryDTO>(&LibPaths::new(lib_root).manifest)
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

    fn fetch_and_validate_spt_version(config: &SPTPaths) -> Result<String, SError> {
        read_pe_version(&config.server_dll)
            .map_err(|e| SError::ParseError(e))
            .and_then(|version| Self::parse_spt_version(&version))
            .and_then(|v| {
                Self::validate_spt_version(&v)
                    .map(|result| result)
                    .and_then(|_| Ok(v.to_string()))
                    .or_else(|_| Err(SError::UnsupportedSPTVersion(v.to_string())))
            })
    }

    fn parse_spt_version(version_str: &str) -> Result<Version, SError> {
        Version::parse(version_str).map_err(|e| SError::ParseError(e.to_string()))
    }

    fn validate_spt_version(version: &Version) -> Result<bool, SError> {
        VersionReq::parse(">=4, <5")
            .map(|req| req.matches(&version))
            .map_err(|e| SError::ParseError(e.to_string()))
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
                mod_type,
                files: stored_paths,
            },
        );

        self.persist()?;

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

        self.persist()?;

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
                let is_active = self.mods.get(&m.id)?.is_active;
                let res = if is_active {
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
            .ok_or_else(|| SError::Link)
    }

    pub fn to_dto(&self) -> LibraryDTO {
        LibraryDTO {
            id: self.id.to_owned(),
            game_root: self.game_root.to_owned(),
            repo_root: self.repo_root.to_owned(),
            spt_version: self.spt_version.to_owned(),
            mods: self.mods.to_owned(),
        }
    }

    fn persist(&self) -> Result<(), SError> {
        let dto = self.to_dto();
        Toml::write(&self.lib_paths.manifest, &dto)?;
        Toml::write(&self.lib_paths.cache, &self.cache)?;
        Ok(())
    }
}
