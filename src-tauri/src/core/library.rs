use crate::core::cache::LibraryCache;
use crate::core::mod_stager::StageMaterial;
use crate::core::version;
use crate::models::error::SError;
use crate::models::library::{LibraryCreationRequirement, LibraryDTO};
use crate::models::mod_dto::Mod;
use crate::models::paths::{LibPathRules, SPTPathCanonical, SPTPathRules};
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
    pub(crate) is_dirty: bool,
}

impl Library {
    pub fn create(requirement: LibraryCreationRequirement) -> Result<Self, SError> {
        // Ensure the repo_root directory exists
        std::fs::create_dir_all(&requirement.repo_root)?;

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


    /// Marks the library as dirty (modified).
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Clears the dirty flag.
    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    /// Persists the library manifest and cache to disk.
    pub fn persist(&self) -> Result<(), SError> {
        Toml::write(&self.lib_paths.manifest, &self.to_dto())?;
        Toml::write(&self.lib_paths.cache, &self.cache)?;
        Ok(())
    }
}

