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
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use sysinfo::System;
use walkdir::WalkDir;

type OwnershipMap = HashMap<Utf8PathBuf, Vec<String>>;

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

    /// Validates that no two active mods contain the same file.
    /// Note: Directories are allowed to overlap; only files cause collisions.
    fn check_file_collisions(&self) -> Result<(), SError> {
        // Map of: Relative File Path -> Mod ID that owns it
        let mut file_ownership: HashMap<&Utf8PathBuf, &String> = HashMap::new();

        // Use BTreeSet to keep errors sorted and unique
        let mut collision_messages = BTreeSet::new();

        for (id, m_dto) in &self.mods {
            // Only check mods the user has marked as active
            if !m_dto.is_active {
                continue;
            }

            if let Some(m_fs) = self.cache.mods.get(id) {
                for file_path in &m_fs.files {
                    // Check if another active mod already claimed this specific file
                    if let Some(existing_owner_id) = file_ownership.get(file_path) {
                        collision_messages.insert(format!(
                            "File Conflict: '{}' is provided by both '{}' and '{}'.",
                            file_path, existing_owner_id, id
                        ));
                    } else {
                        file_ownership.insert(file_path, id);
                    }
                }
            }
        }

        if collision_messages.is_empty() {
            Ok(())
        } else {
            // Convert BTreeSet to Vec<String> for the SError variant
            Err(SError::FileCollision(
                collision_messages.into_iter().collect(),
            ))
        }
    }

    fn build_folder_ownership_map(&self) -> HashMap<Utf8PathBuf, Vec<String>> {
        let mut map: HashMap<Utf8PathBuf, Vec<String>> = HashMap::new();
        for (id, m_dto) in &self.mods {
            if !m_dto.is_active {
                continue;
            }
            if let Some(m_fs) = self.cache.mods.get(id) {
                for file_path in &m_fs.files {
                    // Add the file itself and every parent directory to the map
                    for ancestor in file_path.ancestors() {
                        if ancestor == "" || ancestor == "." {
                            continue;
                        }
                        let entry = map.entry(ancestor.to_path_buf()).or_default();
                        // Prevent duplicate IDs in the same path list
                        if !entry.contains(id) {
                            entry.push(id.clone());
                        }
                    }
                }
            }
        }
        map
    }

    fn execute_recursive_link(&self, ownership: &OwnershipMap) -> Result<(), SError> {
        for (id, m_fs) in &self.cache.mods {
            let is_active = self.mods.get(id).map_or(false, |m| m.is_active);
            if !is_active {
                continue;
            }

            for file_path in &m_fs.files {
                let mut current_link_path = Utf8PathBuf::new();
                let components: Vec<_> = file_path.components().collect();

                for (_, comp) in components.iter().enumerate() {
                    current_link_path.push(comp);

                    let owners = ownership.get(&current_link_path).unwrap();

                    // If this path is unique to THIS mod, we link it and stop drilling
                    if owners.len() == 1 {
                        let src = self.lib_paths.mods.join(id).join(&current_link_path);
                        let dst = self.game_root.join(&current_link_path);

                        Linker::link(&src, &dst)?;
                        break; // Move to next file
                    }

                    // If the path is shared (multiple mods), we must ensure a real directory exists
                    let shared_dir = self.game_root.join(&current_link_path);
                    if !shared_dir.exists() {
                        std::fs::create_dir_all(&shared_dir)?;
                    }

                    // If we reached the end of the components and it's still shared,
                    // it means two mods tried to provide the same FILE.
                    // (Handled by check_file_collisions, but good to keep in mind).
                }
            }
        }
        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), SError> {
        if self.is_running() {
            return Err(SError::GameOrServerRunning);
        }

        // 1. First, ensure no two mods try to overwrite the same FILE
        self.check_file_collisions()?;

        // 2. Build the directory-aware ownership map for the recursive linker
        // (This map includes all parent directories of every file)
        let folder_ownership = self.build_folder_ownership_map();

        // 3. Perform the recursive deployment
        // - If folder is shared: create real directory
        // - If path is unique: link it (even if it's a folder)
        self.purge_managed_links();
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
        // 1. Build a Whitelist of physical File IDs from the Repository.
        // This set represents every physical file we own.
        let mut managed_ids = HashSet::new();

        for (id, mod_fs) in &self.cache.mods {
            for rel_path in &mod_fs.files {
                let abs_path = self.lib_paths.mods.join(id).join(rel_path);

                // We attempt to get the ID. If a file in the repo is missing/locked,
                // we skip it (fail-safe: we just won't delete the corresponding link).
                if let Ok(file_id) = Linker::get_id(&abs_path) {
                    managed_ids.insert(file_id);
                }
            }
        }

        // 2. Define the roots to scan in the Game Directory
        let roots = [
            self.game_root.join(&self.spt_rules.server_mods),
            self.game_root.join(&self.spt_rules.client_plugins),
        ];

        for root in roots {
            if !root.exists() { continue; }

            // 3. Bottom-Up Walk
            // We use contents_first(true) so we process files inside a folder
            // before the folder itself. This allows us to delete empty folders
            // immediately after clearing their contents.
            let walker = WalkDir::new(&root)
                .contents_first(true)
                .into_iter()
                .filter_map(|e| e.ok());

            for entry in walker {
                let path = Utf8Path::from_path(entry.path())
                    .ok_or_else(|| SError::ParseError(format!("Invalid path: {:?}", entry.path())))?;

                // Use symlink_metadata to check the file type without following links
                let meta = match entry.path().symlink_metadata() {
                    Ok(m) => m,
                    Err(_) => continue, // Skip if we can't read metadata
                };

                let mut should_remove = false;

                // --- CASE A: Directory or Symbolic Link/Junction ---
                if meta.is_dir() || meta.is_symlink() {
                    // If it's a Junction or Symlink, check where it points.
                    // Linker::read_link_target handles both.
                    if let Ok(target) = Linker::read_link_target(path) {
                        // If it points into our repository, it's ours.
                        if target.starts_with(&self.repo_root) {
                            should_remove = true;
                        }
                    }
                }

                // --- CASE B: File (Potentially a Hard Link) ---
                if !should_remove && meta.is_file() {
                    // Check if the physical File ID matches one in our whitelist.
                    if let Ok(current_id) = Linker::get_id(path) {
                        if managed_ids.contains(&current_id) {
                            should_remove = true;
                        }
                    }
                }

                // --- EXECUTE REMOVAL ---
                if should_remove {
                    Linker::unlink(path)?;
                } else if meta.is_dir() && path != root {
                    // --- CASE C: Cleanup Empty Framework Folders ---
                    // If we didn't delete it explicitly (because it wasn't a link),
                    // check if it's now empty. This handles the "Virtual Overlay" folders.
                    if self.is_dir_empty(path) {
                        // We use standard remove_dir, as Linker::unlink is for targets we own.
                        // We strictly only remove empty directories here.
                        let _ = std::fs::remove_dir(path);
                    }
                }
            }
        }

        Ok(())
    }

    /// Helper to check if a directory is empty safely
    fn is_dir_empty(&self, path: &Utf8Path) -> bool {
        std::fs::read_dir(path)
            .map(|mut i| i.next().is_none())
            .unwrap_or(false)
    }
}
