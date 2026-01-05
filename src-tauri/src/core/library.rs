use crate::core::cache::LibraryCache;
use crate::core::linker::Linker;
use crate::core::mod_fs::ModFS;
use crate::models::divider::MOD_ID_DIVIDER;
use crate::models::error::SError;
use crate::models::library_dto::LibraryDTO;
use crate::models::mod_dto::{Mod, ModManifest};
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
            std::fs::create_dir_all(dir)?;
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

    pub fn add_mod(&mut self, mod_root: &Utf8Path) -> Result<(), SError> {
        if self.is_running() {
            return Err(SError::GameOrServerRunning);
        }

        let fs = ModFS::new(mod_root, &SPTPaths::default())?;
        let mod_original = self.mods.get(&fs.id);

        self.cache
            .detect_collisions(&fs.files, mod_original.map(|_| fs.id.as_str()).or(None))?;

        if let Some(content) = mod_original {
            // backup
        }

        let dst = &self.lib_paths.mods.join(&fs.id);
        ModFS::copy_recursive(mod_root, dst)?;

        self.cache.add(dst, fs);



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

        errors.is_empty().then_some(()).ok_or_else(|| SError::Link)
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
