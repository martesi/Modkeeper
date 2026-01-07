use crate::core::cache::LibraryCache;
use crate::core::linker::Linker;
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::library_dto::LibraryDTO;
use crate::models::mod_dto::Mod;
use crate::models::paths::{LibPathRules, SPTPathCanonical, SPTPathRules};
use crate::utils::time::get_unix_timestamp;
use crate::utils::toml::Toml;
use crate::utils::version::read_pe_version;
use camino::{Utf8Path, Utf8PathBuf};
use semver::{Version, VersionReq};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
use walkdir::WalkDir;

type OwnershipMap = HashMap<Utf8PathBuf, Vec<String>>;

pub struct Library {
    id: String,
    repo_root: Utf8PathBuf,
    pub game_root: Utf8PathBuf,
    pub spt_rules: SPTPathRules,
    pub lib_paths: LibPathRules,
    pub spt_paths_canonical: SPTPathCanonical,
    pub cache: LibraryCache,
    spt_version: String,
    pub mods: BTreeMap<String, Mod>,
    is_dirty: bool,
}

impl Library {
    pub fn create(repo_root: &Utf8Path, game_root: &Utf8Path) -> Result<Self, SError> {
        let lib_paths = LibPathRules::new(game_root);
        for dir in [&lib_paths.mods, &lib_paths.backups, &lib_paths.staging] {
            std::fs::create_dir_all(dir)?;
        }

        let spt_paths = SPTPathRules::new(game_root);
        let inst = Self {
            id: uuid::Uuid::new_v4().to_string(),
            repo_root: repo_root.to_owned(),
            game_root: game_root.to_owned(),
            spt_version: Library::fetch_and_validate_spt_version(&spt_paths)?,
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
        // check the original spt_version when library is created
        // if not valid, return error directly
        Self::parse_spt_version(&dto.spt_version)
            .and_then(|spt_version| Self::validate_spt_version(&spt_version))?;

        let config = SPTPathRules::default();
        // When displaying, always use the current spt version
        let spt_version = Self::fetch_and_validate_spt_version(&config)?;
        let lib_paths = LibPathRules::new(repo_root);
        let spt_paths = SPTPathRules::new(&dto.game_root);
        let inst = Self {
            id: dto.id,
            repo_root: repo_root.to_owned(),
            spt_paths_canonical: SPTPathCanonical::from_spt_paths(spt_paths.clone())?,
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

    pub fn read_library_manifest(lib_root: &Utf8Path) -> Result<LibraryDTO, SError> {
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

    pub fn add_mod(&mut self, mod_root: &Utf8Path, fs:ModFS) -> Result<(), SError> {
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

        let updated_mod = self
            .mods
            .entry(mod_id.clone())
            .and_modify(|m| m.mod_type = fs.mod_type.clone())
            .or_insert(Mod {
                id: mod_id,
                is_active: false,
                mod_type: fs.mod_type.clone(),
                name: Default::default(),
                manifest: None,
            })
            .clone();
        self.cache.add(&dst, fs);

        self.is_dirty = true;
        self.persist()?;
        Ok(())
    }

    pub fn remove_mod(&mut self, id: &str) -> Result<(), SError> {
        if let Some(m) = self.cache.mods.remove(id) {
            m.files.iter().for_each(|f| {
                let _ = Linker::unlink(&self.game_root.join(f));
            });
            let _ = std::fs::remove_dir_all(self.repo_root.join("mods").join(id));
        }

        self.persist()?;

        Ok(())
    }

    /// Validates that no two active mods contain the same file.
    /// Note: Directories are allowed to overlap; only files cause collisions.
    fn check_file_collisions(&self) -> Result<(), SError> {
        let collisions = self
            .mods
            .iter()
            // 1. Filter for active mods only
            .filter(|(_, m)| m.is_active)
            // 2. Map to cache data (safely handling missing cache entries)
            .filter_map(|(id, _)| self.cache.mods.get(id).map(|fs| (id, fs)))
            // 3. Flatten into a stream of (FilePath, ModID)
            .flat_map(|(id, fs)| fs.files.iter().map(move |f| (f, id)))
            // 4. Fold: Accumulate (OwnershipMap, Errors)
            .fold(
                (HashMap::new(), BTreeSet::new()),
                |(mut owners, mut errors), (path, current_id)| {
                    if let Some(existing_owner) = owners.get(path) {
                        // Conflict detected: Add to errors
                        errors.insert(format!(
                            "File Conflict: '{}' is provided by both '{}' and '{}'.",
                            path, existing_owner, current_id
                        ));
                    } else {
                        // No conflict: Claim ownership
                        owners.insert(path, current_id);
                    }
                    (owners, errors)
                },
            )
            .1; // 5. Discard ownership map, keep errors

        // 6. Return Result based on collision set
        if collisions.is_empty() {
            Ok(())
        } else {
            Err(SError::FileCollision(collisions.into_iter().collect()))
        }
    }

    fn build_folder_ownership_map(&self) -> HashMap<Utf8PathBuf, Vec<String>> {
        self.mods
            .iter()
            // 1. Only process active mods
            .filter(|(_, m_dto)| m_dto.is_active)
            // 2. Pair the active mod ID with its cached file system data
            .filter_map(|(id, _)| self.cache.mods.get(id).map(|m_fs| (id, m_fs)))
            // 3. Flatten files into their individual ancestors (paths)
            .flat_map(|(id, m_fs)| {
                m_fs.files.iter().flat_map(move |file_path| {
                    file_path
                        .ancestors()
                        // Filter out empty or current-dir markers
                        .filter(|a| !a.as_str().is_empty() && *a != ".")
                        .map(move |ancestor| (ancestor.to_path_buf(), id.clone()))
                })
            })
            // 4. Fold the stream into the final HashMap
            .fold(HashMap::new(), |mut acc, (path, id)| {
                let entry = acc.entry(path).or_default();
                if !entry.contains(&id) {
                    entry.push(id);
                }
                acc
            })
    }

    fn execute_recursive_link(&self, ownership: &OwnershipMap) -> Result<(), SError> {
        self.cache
            .mods
            .iter()
            // 1. Filter for active mods only
            .filter(|(id, _)| self.mods.get(*id).map_or(false, |m| m.is_active))
            // 2. Flatten: Mod -> Files -> (ModID, FilePath)
            .flat_map(|(id, m_fs)| m_fs.files.iter().map(move |f| (id, f)))
            // 3. Process each file with early-exit logic
            .try_for_each(|(id, file_path)| {
                let mut current_path = Utf8PathBuf::new();

                // Walk the path components (Root -> File)
                for component in file_path.components() {
                    current_path.push(component);

                    // Retrieve ownership info (Safety: Map is built from the same cache data)
                    let owners = ownership.get(&current_path).ok_or_else(|| {
                        SError::ParseError(format!("Missing ownership for '{}'", current_path))
                    })?;

                    // Case A: Unique Ownership -> Link this path and STOP processing this file.
                    // We link the highest possible directory (or file) that is unique to this mod.
                    if owners.len() == 1 {
                        let src = self.lib_paths.mods.join(id).join(&current_path);
                        let dst = self.game_root.join(&current_path);

                        Linker::link(&src, &dst)?;
                        return Ok(());
                    }

                    // Case B: Shared Ownership -> This is a shared parent directory.
                    // Ensure it exists in the game folder, then continue drilling down.
                    let shared_dir = self.game_root.join(&current_path);
                    if !shared_dir.exists() {
                        std::fs::create_dir_all(&shared_dir)?;
                    }
                }
                Ok(())
            })
    }

    pub fn sync(&mut self) -> Result<(), SError> {
        // 1. First, ensure no two mods try to overwrite the same FILE
        self.check_file_collisions()?;

        // 2. Build the directory-aware ownership map for the recursive linker
        // (This map includes all parent directories of every file)
        let folder_ownership = self.build_folder_ownership_map();

        // 3. Perform the recursive deployment
        // - If folder is shared: create real directory
        // - If path is unique: link it (even if it's a folder)
        self.purge_managed_links()?;
        self.execute_recursive_link(&folder_ownership)?;

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

    pub fn to_frontend_dto(&self) -> LibraryDTO {
        let mut dto = self.to_dto();

        // Enrich the DTO mods with manifest data stored in the cache
        dto.mods.iter_mut().for_each(|(id, m)| {
            m.manifest = self.cache.manifests.get(id).cloned();
        });

        dto
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

    /// Scans the game directory and removes any files, links, or empty folders
    /// that belong to the managed library.
    ///
    /// This uses a "Whitelist" approach: matches Hard Link IDs against the repository
    /// to ensure 100% safety when deleting files.
    pub fn purge_managed_links(&self) -> Result<(), SError> {
        // 1. Build Whitelist of physical File IDs using functional chaining
        let managed_ids: HashSet<_> = self
            .cache
            .mods
            .iter()
            .flat_map(|(mod_id, mod_fs)| {
                mod_fs
                    .files
                    .iter()
                    .map(move |rel_path| self.lib_paths.mods.join(mod_id).join(rel_path))
            })
            .filter_map(|abs_path| Linker::get_id(&abs_path).ok())
            .collect();

        // 2. Define roots and create a flattened iterator of all entries
        let roots = [
            self.game_root.join(&self.spt_rules.server_mods),
            self.game_root.join(&self.spt_rules.client_plugins),
        ];

        roots
            .iter()
            .filter(|root| root.exists())
            .flat_map(|root| {
                WalkDir::new(root)
                    .contents_first(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .map(move |entry| (root, entry))
            })
            .try_for_each(|(root, entry)| -> Result<(), SError> {
                let path = Utf8Path::from_path(entry.path()).ok_or_else(|| {
                    SError::ParseError(format!("Invalid path: {:?}", entry.path()))
                })?;

                let meta = entry.path().symlink_metadata().ok();

                // Determine if this is a path we manage (Repo-linked or Hard-linked)
                let is_managed = meta
                    .as_ref()
                    .map(|m| {
                        let is_repo_link = (m.is_dir() || m.is_symlink())
                            && Linker::read_link_target(path)
                                .map(|t| t.starts_with(&self.repo_root))
                                .unwrap_or(false);

                        let is_known_hardlink = m.is_file()
                            && Linker::get_id(path)
                                .map(|id| managed_ids.contains(&id))
                                .unwrap_or(false);

                        is_repo_link || is_known_hardlink
                    })
                    .unwrap_or(false);

                // Determine if it's an empty "overlay" directory that needs cleanup
                let is_removable_folder = !is_managed
                    && meta.map(|m| m.is_dir()).unwrap_or(false)
                    && path != root
                    && self.is_dir_empty(path);

                // Execute actions based on derived state
                if is_managed {
                    Linker::unlink(path)?;
                } else if is_removable_folder {
                    let _ = std::fs::remove_dir(path);
                }

                Ok(())
            })
    }

    /// Helper to check if a directory is empty safely
    fn is_dir_empty(&self, path: &Utf8Path) -> bool {
        std::fs::read_dir(path)
            .map(|mut i| i.next().is_none())
            .unwrap_or(false)
    }
}
