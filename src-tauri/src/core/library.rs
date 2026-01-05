use crate::core::cache::LibraryCache;
use crate::core::linker::Linker;
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::library_dto::LibraryDTO;
use crate::models::mod_dto::Mod;
use crate::models::paths::{LibPathRules, SPTPathRules};
use crate::utils::time::get_unix_timestamp;
use crate::utils::toml::Toml;
use crate::utils::version::read_pe_version;
use camino::{Utf8Path, Utf8PathBuf};
use semver::{Version, VersionReq};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use sysinfo::System;

pub struct Library {
    pub id: String,
    pub repo_root: Utf8PathBuf,
    pub game_root: Utf8PathBuf,
    pub spt_rules: SPTPathRules,
    pub lib_paths: LibPathRules,
    pub cache: LibraryCache,
    pub spt_version: String,
    pub mods: BTreeMap<String, Mod>,
    is_dirty: bool,
}

impl Library {
    fn spt_paths(&self) -> SPTPathRules {
        SPTPathRules::new(&self.game_root)
    }

    pub fn create(repo_root: &Utf8PathBuf, game_root: &Utf8PathBuf) -> Result<Self, SError> {
        let lib_paths = LibPathRules::new(game_root);
        for dir in [&lib_paths.mods, &lib_paths.backups, &lib_paths.staging] {
            std::fs::create_dir_all(dir)?;
        }

        let config = SPTPathRules::default();

        let inst = Self {
            id: uuid::Uuid::new_v4().to_string(),
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            spt_version: Library::fetch_and_validate_spt_version(&SPTPathRules::new(game_root))?,
            cache: LibraryCache::default(),
            mods: Default::default(),
            lib_paths,
            spt_rules: config,
            is_dirty: false,
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

        let config = SPTPathRules::default();
        // When displaying, always use the current spt version
        let spt_version = Self::fetch_and_validate_spt_version(&config)?;
        let lib_paths = LibPathRules::new(repo_root);
        let inst = Self {
            id: dto.id,
            repo_root: repo_root.clone(),
            game_root: dto.game_root,
            spt_rules: config,
            cache: Toml::read(&lib_paths.cache)?,
            lib_paths,
            spt_version,
            mods: dto.mods,
            is_dirty: false,
        };

        Ok(inst)
    }

    pub fn read_library_manifest(lib_root: &Utf8PathBuf) -> Result<LibraryDTO, SError> {
        Toml::read::<LibraryDTO>(&LibPathRules::new(lib_root).manifest)
    }

    fn fetch_and_validate_spt_version(config: &SPTPathRules) -> Result<String, SError> {
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
        let paths = self.spt_paths();
        let server_name = paths.server_exe.file_name().unwrap_or_default();
        let client_name = paths.client_exe.file_name().unwrap_or_default();

        s.processes()
            .values()
            .any(|p| p.name() == server_name || p.name() == client_name)
    }

    pub fn add_mod(&mut self, mod_root: &Utf8Path) -> Result<(), SError> {
        if self.is_running() {
            return Err(SError::GameOrServerRunning);
        }

        let fs = ModFS::new(mod_root, &self.spt_rules)?;
        let mod_id = fs.id.clone();

        let dst = self.lib_paths.mods.join(&mod_id);
        if dst.exists() {
            // backups/{mod_id}/{unix_seconds}
            let backup_dir = self
                .lib_paths
                .backups
                .join(&mod_id)
                .join(get_unix_timestamp().to_string());

            std::fs::create_dir_all(&backup_dir)?;

            // Copy current state to backup before overwriting
            ModFS::copy_recursive(&dst, &backup_dir)?;
        }

        std::fs::create_dir_all(&dst)?;
        ModFS::copy_recursive(mod_root, &dst)?;

        self.mods.entry(mod_id.clone()).or_insert(Mod {
            id: mod_id,
            is_active: false,
            mod_type: fs.mod_type.clone(),
        });
        self.cache.add(&dst, fs);

        self.is_dirty = true;
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

    pub fn deploy_active_mods(&mut self) -> Result<(), SError> {
        if self.is_running() {
            return Err(SError::GameOrServerRunning);
        }

        // 1. Ownership Tracking: path -> mod_id
        let mut ownership_map: HashMap<&Utf8PathBuf, &String> = HashMap::new();
        let mut collisions = BTreeSet::new();

        // 2. Identify Collisions among ACTIVE mods
        for (id, m_dto) in &self.mods {
            if !m_dto.is_active {
                continue;
            }

            if let Some(m_fs) = self.cache.mods.get(id) {
                for file_path in &m_fs.files {
                    if let Some(owner_id) = ownership_map.get(file_path) {
                        collisions.insert(format!(
                            "Conflict: {} is claimed by both '{}' and '{}'",
                            file_path, owner_id, id
                        ));
                    } else {
                        ownership_map.insert(file_path, id);
                    }
                }
            }
        }

        if !collisions.is_empty() {
            return Err(SError::FileCollision(collisions.into_iter().collect()));
        }

        // 3. Deployment (Sync) logic
        // We walk through the cache. If mod is active, link. If not, unlink.
        for (id, m_fs) in &self.cache.mods {
            let is_active = self.mods.get(id).map_or(false, |m| m.is_active);

            for file_path in &m_fs.files {
                let src = self.lib_paths.mods.join(id).join(file_path);
                let dst = self.game_root.join(file_path);

                if is_active {
                    Linker::link(&src, &dst)?;
                } else {
                    let _ = Linker::unlink(&dst);
                }
            }
        }

        self.is_dirty = false;
        self.persist()?;
        Ok(())
    }

    pub fn to_dto(&self) -> LibraryDTO {
        LibraryDTO {
            id: self.id.to_owned(),
            game_root: self.game_root.to_owned(),
            repo_root: self.repo_root.to_owned(),
            spt_version: self.spt_version.to_owned(),
            mods: self.mods.to_owned(),
            is_dirty: self.is_dirty,
        }
    }

    fn persist(&self) -> Result<(), SError> {
        self.persist_manifest()?;
        self.persist_cache()?;
        Ok(())
    }

    fn persist_manifest(&self) -> Result<(), SError> {
        let dto = self.to_dto();
        Toml::write(&self.lib_paths.manifest, &dto)?;
        Ok(())
    }

    fn persist_cache(&self) -> Result<(), SError> {
        Toml::write(&self.lib_paths.cache, &self.cache)?;
        Ok(())
    }
}
