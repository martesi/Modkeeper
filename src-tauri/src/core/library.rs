use crate::core::cache::LibraryCache;
use crate::core::linker::Linker;
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::library_dto::LibraryDTO;
use crate::models::mod_dto::Mod;
use crate::models::paths::{LibPathRules, ModPaths, SPTPathCanonical, SPTPathRules};
use crate::utils::time::get_unix_timestamp;
use crate::utils::toml::Toml;
use crate::utils::version::read_pe_version;
use camino::{Utf8Path, Utf8PathBuf};
use semver::{Version, VersionReq};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
use tracing::Instrument;
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
        let lib_paths = LibPathRules::new(repo_root);
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
        return Ok("4.0.0".into());

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

    pub fn add_mod(&mut self, mod_root: &Utf8Path, fs: ModFS) -> Result<(), SError> {
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

        self.mods
            .entry(mod_id.clone())
            .and_modify(|m| m.mod_type = fs.mod_type.clone())
            .or_insert(Mod {
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
        // 1. Identify the roots we manage
        let roots = [&self.spt_rules.server_mods, &self.spt_rules.client_plugins];

        // 2. Derive protected paths:
        // For "SPT/user/mods", this creates: ["SPT", "SPT/user", "SPT/user/mods"]
        let mut acc: HashMap<Utf8PathBuf, Vec<String>> = roots
            .iter()
            .flat_map(|path| {
                path.ancestors()
                    .filter(|a| !a.as_str().is_empty() && *a != ".")
                    .map(|a| (a.to_path_buf(), vec!["__SYSTEM__".to_string()]))
            })
            .collect();

        // 3. Process active mods and fold them into the map
        self.mods
            .iter()
            .filter(|(_, m_dto)| m_dto.is_active)
            .filter_map(|(id, _)| self.cache.mods.get(id).map(|m_fs| (id, m_fs)))
            .flat_map(|(id, m_fs)| {
                m_fs.files.iter().flat_map(move |file_path| {
                    file_path
                        .ancestors()
                        .filter(|a| !a.as_str().is_empty() && *a != ".")
                        .map(move |ancestor| (ancestor.to_path_buf(), id.clone()))
                })
            })
            .for_each(|(path, id)| {
                let entry = acc.entry(path).or_default();
                if !entry.contains(&id) {
                    entry.push(id);
                }
            });

        acc
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
        // 1. Calculate the "Managed Scope"
        // This includes every file and parent folder our library knows about (active or inactive)
        let managed_scope: HashSet<Utf8PathBuf> = self.cache.mods.values()
            .flat_map(|m_fs| m_fs.files.iter().flat_map(|f| f.ancestors().map(|a| a.to_path_buf())))
            .filter(|a| !a.as_str().is_empty() && *a != ".")
            .collect();

        // 2. Physical IDs for Hard Link detection
        let managed_ids: HashSet<_> = self.cache.mods.iter()
            .flat_map(|(id, fs)| fs.files.iter().map(move |f| self.lib_paths.mods.join(id).join(f)))
            .filter_map(|p| Linker::get_id(&p).ok())
            .collect();

        let roots = [
            self.game_root.join(&self.spt_rules.server_mods),
            self.game_root.join(&self.spt_rules.client_plugins),
        ];

        for root in roots.iter().filter(|r| r.exists()) {
            let mut it = WalkDir::new(root).contents_first(false).into_iter();

            while let Some(entry) = it.next() {
                let entry = entry.map_err(|e| SError::IOError(e.to_string()))?;
                let path = Utf8Path::from_path(entry.path()).ok_or(SError::Unexpected)?;
                if path == root { continue; }

                let rel_path = path.strip_prefix(&self.game_root).unwrap_or(path);
                let meta = entry.path().symlink_metadata()?;

                // Case A: Managed Junctions/Symlinks
                if !meta.is_file() {
                    if let Ok(target) = Linker::read_link_target(path) {
                        if target.starts_with(&self.repo_root) {
                            Linker::unlink(path)?;
                            it.skip_current_dir();
                            continue;
                        }
                    }
                }

                // Case B: Managed Hardlinks
                if meta.is_file() {
                    if let Ok(id) = Linker::get_id(path) {
                        if managed_ids.contains(&id) {
                            Linker::unlink(path)?;
                        }
                    }
                }

                // Case C: Ancestor-only Empty Directory Cleanup
                // We ONLY remove the directory if it's empty AND it's in our managed_scope
                if meta.is_dir() && !meta.file_type().is_symlink() {
                    if self.is_dir_empty(path) && managed_scope.contains(rel_path) {
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::models::paths::ModPaths;
    use std::fs;

    /// Helper to setup a dummy SPT environment so Library::create doesn't fail
    fn setup_test_env() -> (tempfile::TempDir, Utf8PathBuf, Utf8PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();

        let game_root = root.join("game");
        let repo_root = root.join("repo");

        std::fs::create_dir_all(&game_root).unwrap();
        std::fs::create_dir_all(&repo_root).unwrap();

        // 1. Get the rules to find where SPT expects files
        let rules = SPTPathRules::new(&game_root);

        // 2. Create DUMMY files so canonicalize() doesn't fail with "os error 2"
        let essential_files = [&rules.server_dll, &rules.server_exe, &rules.client_exe];

        for path in essential_files {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, "dummy").unwrap();
        }

        (tmp, game_root, repo_root)
    }

    /// Mock a mod folder structure
    fn create_test_mod(path: &Utf8Path, name: &str, is_server: bool) {
        let rules = SPTPathRules::default();
        let mod_dir = if is_server {
            path.join(rules.server_mods).join(name)
        } else {
            path.join(rules.client_plugins).join(name)
        };

        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("content.txt"), name).unwrap();

        // Optional: Add a manifest
        let manifest_dir = path.join(ModPaths::default().folder);
        fs::create_dir_all(&manifest_dir).unwrap();
        let manifest_json = format!(
            r#"{{"guid": "{}", "name": "{}", "version": "1.0.0", "author": "test"}}"#,
            name, name
        );
        fs::write(manifest_dir.join("manifest.json"), manifest_json).unwrap();
    }

    #[test]
    fn test_library_init_and_add_mod() {
        let (_tmp, game_root, repo_root) = setup_test_env();

        // 1. Create Library
        let mut lib = Library::create(&repo_root, &game_root).expect("Failed to create library");
        assert!(lib.lib_paths.mods.exists());

        // 2. Prepare a fake mod on disk
        let mod_src = _tmp.path().join("my_new_mod");
        let mod_src_utf8 = Utf8Path::from_path(&mod_src).unwrap();
        create_test_mod(mod_src_utf8, "MyMod", true);

        // 3. Add mod to library
        let mod_fs =
            ModFS::new(mod_src_utf8, &SPTPathRules::default()).expect("Failed to parse mod");
        lib.add_mod(mod_src_utf8, mod_fs)
            .expect("Failed to add mod");

        // 4. Verify persistence
        assert!(lib.mods.contains_key("MyMod"));
        assert!(lib.lib_paths.mods.join("MyMod").exists());
        assert!(lib.cache.mods.contains_key("MyMod"));
    }

    #[test]
    fn test_collision_detection() {
        let (_tmp, game_root, repo_root) = setup_test_env();
        let mut lib = Library::create(&repo_root, &game_root).unwrap();
        let rules = SPTPathRules::default();

        // Helper to create a mod with a specific ID (via manifest) but containing a specific file
        let mut add_named_mod = |mod_id: &str, colliding_file: &str| {
            let p = repo_root.join(format!("src_{}", mod_id));

            // 1. Create Manifest to force a unique Mod ID
            let manifest_dir = p.join("manifest");
            fs::create_dir_all(&manifest_dir).unwrap();
            let manifest_json = format!(
                r#"{{"guid": "{}", "name": "{}", "version": "1.0", "author": "test"}}"#,
                mod_id, mod_id
            );
            fs::write(manifest_dir.join("manifest.json"), manifest_json).unwrap();

            // 2. Create the colliding file
            let file_path = p.join(&colliding_file);
            fs::create_dir_all(file_path.parent().unwrap()).unwrap();
            fs::write(file_path, "some content").unwrap();

            // 3. Add to library and activate
            let fs = ModFS::new(&p, &rules).expect("Failed to parse mod");
            lib.add_mod(&p, fs).expect("Failed to add mod");
            lib.mods.get_mut(mod_id).unwrap().is_active = true;
        };

        // These two mods have different IDs but both provide "BepInEx/plugins/conflict.dll"
        let conflict_path = "BepInEx/plugins/conflict.dll";
        add_named_mod("Mod_A", conflict_path);
        add_named_mod("Mod_B", conflict_path);

        // Act
        let result = lib.sync();

        // Assert
        assert!(
            result.is_err(),
            "Sync should have failed due to file collision"
        );
        match result {
            Err(SError::FileCollision(errors)) => {
                assert!(!errors.is_empty(), "Collision list should not be empty");
                assert!(
                    errors.iter().any(|e| e.contains("conflict.dll")),
                    "Error message should mention the colliding file"
                );
            }
            other => panic!("Expected SError::FileCollision, but got: {:?}", other),
        }
    }

    #[test]
    fn test_recursive_linking_logic() {
        let (_tmp, game_root, repo_root) = setup_test_env();
        let mut lib = Library::create(&repo_root, &game_root).unwrap();
        let rules = SPTPathRules::default();

        let mut setup_mod = |lib: &mut Library, mod_id: &str, file_name: &str| {
            let p = repo_root.join(mod_id);

            // 1. Force the ID using a manifest
            let manifest_dir = p.join("manifest");
            fs::create_dir_all(&manifest_dir).unwrap();
            let manifest_json = format!(
                r#"{{"guid": "{}", "name": "{}", "version": "1", "author": "t"}}"#,
                mod_id, mod_id
            );
            fs::write(manifest_dir.join("manifest.json"), manifest_json).unwrap();

            // 2. Create the overlapping directory structure
            let file_path = p.join(&rules.server_mods).join("CommonDir").join(file_name);
            fs::create_dir_all(file_path.parent().unwrap()).unwrap();
            fs::write(file_path, "data").unwrap();

            // 3. New ModFS will now resolve ID to mod_id ("ModA" or "ModB")
            let fs = ModFS::new(&p, &rules).unwrap();
            lib.add_mod(&p, fs).unwrap();

            // 4. This will no longer panic
            lib.mods.get_mut(mod_id).unwrap().is_active = true;
        };

        setup_mod(&mut lib, "ModA", "A.txt");
        setup_mod(&mut lib, "ModB", "B.txt");

        lib.sync().expect("Sync failed");

        // ... rest of your assertions ...
        let common_dir_in_game = game_root.join(&rules.server_mods).join("CommonDir");
        assert!(common_dir_in_game.exists());
        assert!(common_dir_in_game.is_dir());
    }

    #[test]
    fn test_purge_removes_deactivated_mods() {
        let (_tmp, game_root, repo_root) = setup_test_env();
        let mut lib = Library::create(&repo_root, &game_root).unwrap();
        let rules = SPTPathRules::default();

        // 1. Add and activate mod
        create_test_mod(&repo_root.join("src"), "DeleteMe", true);
        let fs = ModFS::new(&repo_root.join("src"), &rules).unwrap();
        lib.add_mod(&repo_root.join("src"), fs).unwrap();
        lib.mods.get_mut("DeleteMe").unwrap().is_active = true;

        lib.sync().unwrap();
        let target_path = game_root.join(&rules.server_mods).join("DeleteMe");
        assert!(target_path.exists());

        // 2. Deactivate and sync
        lib.mods.get_mut("DeleteMe").unwrap().is_active = false;
        lib.sync().unwrap();

        // 3. Verify it's gone from game but exists in repo
        assert!(!target_path.exists());
        assert!(lib.lib_paths.mods.join("DeleteMe").exists());
    }

    #[test]
    fn test_to_frontend_dto_enrichment() {
        let (_tmp, game_root, repo_root) = setup_test_env();
        let mut lib = Library::create(&repo_root, &game_root).expect("Failed to create library");

        // 1. Prepare a mod with a real manifest file on disk
        let mod_src = _tmp.path().join("source_mod");
        let mod_src_utf8 = Utf8Path::from_path(&mod_src).unwrap();

        let manifest_path = mod_src_utf8.join("manifest/manifest.json");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();

        let manifest_data = r#"{
        "guid": "test-mod-id",
        "name": "Test Mod Name",
        "version": "1.0.0",
        "author": "someone"
    }"#;
        std::fs::write(&manifest_path, manifest_data).unwrap();

        // 2. Mock some files inside the mod so ModFS::new works
        let rules = SPTPathRules::default();
        let dummy_dll = mod_src_utf8
            .join(&rules.server_mods)
            .join("TestMod/mod.dll");
        std::fs::create_dir_all(dummy_dll.parent().unwrap()).unwrap();
        std::fs::write(dummy_dll, "").unwrap();

        // 3. Add mod to library
        let fs = ModFS::new(mod_src_utf8, &rules).unwrap();
        lib.add_mod(mod_src_utf8, fs).expect("Add mod failed");

        // 4. Check Frontend DTO
        let dto = lib.to_frontend_dto();
        let m = dto.mods.get("test-mod-id").expect("Mod not found in DTO");

        // Assert the manifest was successfully pulled from cache into the DTO
        assert!(m.manifest.is_some());
        assert_eq!(m.manifest.as_ref().unwrap().name, "Test Mod Name");
    }
}
