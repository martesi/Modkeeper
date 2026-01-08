use crate::core::cache::LibraryCache;
use crate::core::mod_fs::ModFS;
use crate::core::mod_stager::StageMaterial;
use crate::core::{cleanup, deployment, version};
use crate::models::error::SError;
use crate::models::library::{LibraryCreationRequirement, LibraryDTO};
use crate::models::mod_dto::Mod;
use crate::models::paths::{LibPathRules, SPTPathCanonical, SPTPathRules};
use crate::utils::file::FileUtils;
use crate::utils::time::get_unix_timestamp;
use crate::utils::toml::Toml;
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::BTreeMap;
use std::default::Default;
use std::path::PathBuf;

pub struct Library {
    pub id: String,
    pub name: String,
    pub repo_root: Utf8PathBuf,
    pub game_root: Utf8PathBuf,
    pub spt_rules: SPTPathRules,
    pub lib_paths: LibPathRules,
    pub spt_paths_canonical: SPTPathCanonical,
    pub cache: LibraryCache,
    pub spt_version: String,
    pub mods: BTreeMap<String, Mod>,
    is_dirty: bool,
}

impl Library {
    pub fn create(requirement: LibraryCreationRequirement) -> Result<Self, SError> {
        let lib_paths = LibPathRules::new(&requirement.repo_root);
        for dir in [&lib_paths.mods, &lib_paths.backups, &lib_paths.staging] {
            std::fs::create_dir_all(dir)?;
        }

        let spt_paths = SPTPathRules::new(&requirement.game_root);
        let spt_version = version::fetch_and_validate(&spt_paths)?;

        let inst = Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: requirement.name,
            repo_root: requirement.repo_root,
            game_root: requirement.game_root,
            spt_version,
            cache: LibraryCache::default(),
            mods: Default::default(),
            spt_paths_canonical: SPTPathCanonical::from_spt_paths(spt_paths.clone())?,
            lib_paths,
            spt_rules: SPTPathRules::default(),
            is_dirty: false,
        };

        inst.persist()?;
        Ok(inst)
    }

    pub fn load(repo_root: &Utf8Path) -> Result<Self, SError> {
        let dto = Self::read_library_manifest(repo_root)?;

        // Validate historical version
        version::validate_string(&dto.spt_version)?;

        let lib_paths = LibPathRules::new(repo_root);
        let spt_paths = SPTPathRules::new(&dto.game_root);
        // Validate current physical version using the game_root from the loaded library
        let spt_version = version::fetch_and_validate(&spt_paths)?;

        Ok(Self {
            id: dto.id,
            name: dto.name,
            repo_root: repo_root.to_owned(),
            spt_paths_canonical: SPTPathCanonical::from_spt_paths(spt_paths.clone())?,
            game_root: dto.game_root,
            spt_rules: SPTPathRules::default(),
            cache: Toml::read(&lib_paths.cache)?,
            lib_paths,
            spt_version,
            mods: dto.mods,
            is_dirty: false,
        })
    }

    pub fn read_library_manifest(lib_root: &Utf8Path) -> Result<LibraryDTO, SError> {
        Toml::read::<LibraryDTO>(&LibPathRules::new(lib_root).manifest)
    }

    pub fn add_mod(&mut self, mod_root: &Utf8Path, fs: ModFS) -> Result<(), SError> {
        let mod_id = fs.id.clone();
        let dst = self.lib_paths.mods.join(&mod_id);

        // Create backup if mod already exists
        if dst.exists() {
            self.create_backup_for_mod(&mod_id)?;
        }

        std::fs::create_dir_all(&dst)?;
        FileUtils::copy_recursive(mod_root, &dst)?;

        self.mods
            .entry(mod_id.clone())
            .and_modify(|m| m.mod_type = fs.mod_type.clone())
            .or_insert_with(|| Mod {
                id: mod_id.clone(),
                is_active: false,
                mod_type: fs.mod_type.clone(),
                name: Default::default(),
                manifest: None,
            });

        self.cache.add(&dst, fs);
        self.is_dirty = true;
        self.persist()?;
        Ok(())
    }

    pub fn remove_mod(&mut self, id: &str) -> Result<(), SError> {
        // Remove from Cache and Filesystem
        if let Some(m) = self.cache.mods.remove(id) {
            // Note: We deliberately do not unlink here individually.
            // A full sync() is required to properly clean up state,
            // otherwise we risk leaving broken links if the user doesn't sync immediately.
            // However, to strictly follow previous logic, we unlink specific files:
            for f in &m.files {
                let _ = crate::core::linker::unlink(&self.game_root.join(f));
            }
            let _ = std::fs::remove_dir_all(self.repo_root.join("mods").join(id));
        }

        self.mods.remove(id);
        self.is_dirty = true;
        self.persist()?;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), SError> {
        // 1. Purge existing managed links
        cleanup::purge(
            &self.game_root,
            &self.repo_root,
            &self.spt_rules,
            &self.lib_paths,
            &self.cache,
        )?;

        // 2. Deploy active mods
        deployment::deploy(
            &self.game_root,
            &self.lib_paths,
            &self.spt_rules,
            &self.mods,
            &self.cache,
        )?;

        self.is_dirty = false;
        self.persist()?;
        Ok(())
    }

    pub fn to_dto(&self) -> LibraryDTO {
        LibraryDTO {
            id: self.id.to_owned(),
            name: self.name.to_owned(),
            game_root: self.game_root.to_owned(),
            repo_root: self.repo_root.to_owned(),
            spt_version: self.spt_version.to_owned(),
            mods: self.mods.to_owned(),
            is_dirty: self.is_dirty,
        }
    }

    pub fn to_frontend_dto(&self) -> LibraryDTO {
        let mut dto = self.to_dto();
        for (id, m) in &mut dto.mods {
            m.manifest = self.cache.manifests.get(id).cloned();
        }
        dto
    }

    pub fn stage_material(&self) -> StageMaterial {
        StageMaterial {
            rules: self.spt_rules.clone(),
            root: self.lib_paths.staging.clone(),
        }
    }

    pub fn spt_canonical_paths(&self) -> Vec<PathBuf> {
        vec![
            self.spt_paths_canonical.client_exe.clone(),
            self.spt_paths_canonical.server_exe.clone(),
        ]
    }

    pub fn toggle_mod(&mut self, id: &str, is_active: bool) -> Result<(), SError> {
        let mod_entry = self
            .mods
            .get_mut(id)
            .ok_or_else(|| SError::ModNotFound(id.to_string()))?;
        mod_entry.is_active = is_active;
        self.is_dirty = true;
        self.persist()?;
        Ok(())
    }

    pub fn get_backups(&self, mod_id: &str) -> Result<Vec<String>, SError> {
        let backup_dir = self.lib_paths.backups.join(mod_id);

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&backup_dir)?;
        let mut timestamps: Vec<String> = entries
            .filter_map(|entry| entry.ok().and_then(|e| e.file_name().into_string().ok()))
            .collect();

        // Sort descending (newest first)
        timestamps.sort_by(|a, b| b.cmp(a));

        Ok(timestamps)
    }

    pub fn restore_backup(&mut self, mod_id: &str, timestamp: &str) -> Result<(), SError> {
        // Verify mod exists
        if !self.mods.contains_key(mod_id) {
            return Err(SError::ModNotFound(mod_id.to_string()));
        }

        let backup_dir = self.lib_paths.backups.join(mod_id).join(timestamp);

        if !backup_dir.exists() {
            return Err(SError::Unexpected);
        }

        let mod_dir = self.lib_paths.mods.join(mod_id);

        // Create a new backup of current state before restoring
        if mod_dir.exists() {
            self.create_backup_for_mod(mod_id)?;
        }

        // Remove current mod directory
        if mod_dir.exists() {
            std::fs::remove_dir_all(&mod_dir)?;
        }

        // Restore from backup
        std::fs::create_dir_all(&mod_dir)?;
        FileUtils::copy_recursive(&backup_dir, &mod_dir)?;

        // Rebuild the ModFS for the restored mod
        let restored_fs = ModFS::new(&mod_dir, &self.spt_rules)?;

        // Update cache with restored files
        self.cache.add(&mod_dir, restored_fs.clone());

        // Update mod metadata if needed
        if let Some(mod_entry) = self.mods.get_mut(mod_id) {
            mod_entry.mod_type = restored_fs.mod_type.clone();
        }

        self.is_dirty = true;
        self.persist()?;
        Ok(())
    }

    /// Creates a backup of the current mod state.
    /// Backup is stored at: `backups/{mod_id}/{timestamp}/`
    fn create_backup_for_mod(&self, mod_id: &str) -> Result<(), SError> {
        let mod_dir = self.lib_paths.mods.join(mod_id);

        if !mod_dir.exists() {
            return Ok(()); // Nothing to backup
        }

        let timestamp = get_unix_timestamp().to_string();
        let backup_dir = self.lib_paths.backups.join(mod_id).join(&timestamp);

        std::fs::create_dir_all(&backup_dir)?;
        FileUtils::copy_recursive(&mod_dir, &backup_dir)?;
        Ok(())
    }

    fn persist(&self) -> Result<(), SError> {
        Toml::write(&self.lib_paths.manifest, &self.to_dto())?;
        Toml::write(&self.lib_paths.cache, &self.cache)?;
        Ok(())
    }
}

