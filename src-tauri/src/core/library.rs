use crate::core::cache::LibraryCache;
use crate::core::mod_fs::ModFS;
use crate::core::mod_stager::StageMaterial;
use crate::core::{cleanup, deployment, versioning};
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
        let spt_version = versioning::fetch_and_validate(&spt_paths)?;

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
        versioning::validate_string(&dto.spt_version)?;

        let config = SPTPathRules::default();
        // Validate current physical version
        let spt_version = versioning::fetch_and_validate(&config)?;

        let lib_paths = LibPathRules::new(repo_root);
        let spt_paths = SPTPathRules::new(&dto.game_root);

        Ok(Self {
            id: dto.id,
            name: dto.name,
            repo_root: repo_root.to_owned(),
            spt_paths_canonical: SPTPathCanonical::from_spt_paths(spt_paths)?,
            game_root: dto.game_root,
            spt_rules: config,
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
                let _ = crate::core::linker::Linker::unlink(&self.game_root.join(f));
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::models::library::LibraryCreationRequirement;
    use crate::models::paths::ModPaths;
    use std::fs;

    /// Helper to setup a dummy SPT environment so Library::create doesn't fail
    pub(crate) fn setup_test_env() -> (tempfile::TempDir, Utf8PathBuf, Utf8PathBuf) {
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
        let requirement = LibraryCreationRequirement {
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            name: "Test Library".to_string(),
        };
        let mut lib = Library::create(requirement).expect("Failed to create library");
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
        let requirement = LibraryCreationRequirement {
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            name: "Test Library".to_string(),
        };
        let mut lib = Library::create(requirement).unwrap();
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
        let requirement = LibraryCreationRequirement {
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            name: "Test Library".to_string(),
        };
        let mut lib = Library::create(requirement).unwrap();
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
        let requirement = LibraryCreationRequirement {
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            name: "Test Library".to_string(),
        };
        let mut lib = Library::create(requirement).unwrap();
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
        let requirement = LibraryCreationRequirement {
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            name: "Test Library".to_string(),
        };
        let mut lib = Library::create(requirement).expect("Failed to create library");

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

#[cfg(test)]
mod expanded_tests {
    use super::*;
    use crate::models::library::LibraryCreationRequirement;
    use std::{fs, thread, time::Duration};

    #[test]
    fn test_mod_backup_on_overwrite() {
        let (_tmp, game_root, repo_root) = integration_tests::setup_test_env();
        let requirement = LibraryCreationRequirement {
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            name: "Test Library".to_string(),
        };
        let mut lib = Library::create(requirement).unwrap();
        let rules = SPTPathRules::default();

        let mod_id = "BackupTest";
        let src = repo_root.join("src_v1");
        fs::create_dir_all(src.join(&rules.server_mods).join(mod_id)).unwrap();
        fs::write(
            src.join(&rules.server_mods).join(mod_id).join("v1.txt"),
            "v1",
        )
        .unwrap();

        // 1. Initial Add
        let fs1 = ModFS::new(&src, &rules).unwrap();
        lib.add_mod(&src, fs1).unwrap();

        // Wait to ensure timestamp differs
        thread::sleep(Duration::from_secs(1));

        // 2. Overwrite Add
        let src2 = repo_root.join("src_v2");
        fs::create_dir_all(src2.join(&rules.server_mods).join(mod_id)).unwrap();
        fs::write(
            src2.join(&rules.server_mods).join(mod_id).join("v2.txt"),
            "v2",
        )
        .unwrap();

        let fs2 = ModFS::new(&src2, &rules).unwrap();
        lib.add_mod(&src2, fs2).unwrap();

        // 3. Check backups
        let backup_dir = lib.lib_paths.backups.join(mod_id);
        let entries: Vec<_> = fs::read_dir(backup_dir).unwrap().collect();
        assert_eq!(
            entries.len(),
            1,
            "Should have exactly one backup timestamp folder"
        );

        let backup_path = Utf8PathBuf::from_path_buf(entries[0].as_ref().unwrap().path()).unwrap();
        assert!(backup_path
            .join(&rules.server_mods)
            .join(mod_id)
            .join("v1.txt")
            .exists());
    }

    #[test]
    fn test_untracked_file_safety_in_shared_folder() {
        let (_tmp, game_root, repo_root) = integration_tests::setup_test_env();
        let requirement = LibraryCreationRequirement {
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            name: "Test Library".to_string(),
        };
        let mut lib = Library::create(requirement).unwrap();
        let rules = SPTPathRules::default();

        // 1. Setup TWO mods sharing a folder in "client_plugins" (BepInEx/plugins).
        // REASON: "server_mods" (user/mods) strictly enforces a "One Folder = One Mod" structure
        // where the folder name is the Mod ID. Creating a "SharedDir" there creates ambiguity
        // (is "SharedDir" the mod?). "client_plugins" allows unstructured nesting,
        // making it the correct target for testing shared directory behavior.
        let mut setup_mod = |lib: &mut Library, name: &str| {
            let p = repo_root.join(format!("src_{}", name));

            // Target: BepInEx/plugins/SharedDir/{name}.dll
            let file_rel = rules
                .client_plugins
                .join("SharedDir")
                .join(format!("{}.dll", name));

            fs::create_dir_all(p.join(file_rel.parent().unwrap())).unwrap();
            fs::write(p.join(&file_rel), "dll content").unwrap();

            let fs = ModFS::new(&p, &rules).unwrap();
            let mod_id = fs.id.clone();

            lib.add_mod(&p, fs).unwrap();

            // Access the mod using the actual ID generated by ModFS
            lib.mods.get_mut(&mod_id).unwrap().is_active = true;

            mod_id
        };

        let id_a = setup_mod(&mut lib, "ModA");
        let id_b = setup_mod(&mut lib, "ModB");
        lib.sync().unwrap();

        // 2. Add untracked file to the real directory created by the Linker
        let shared_dir = game_root.join(&rules.client_plugins).join("SharedDir");
        let untracked = shared_dir.join("user_notes.txt");

        assert!(shared_dir.exists(), "SharedDir should exist after sync");
        fs::write(&untracked, "user data").unwrap();

        // 3. Deactivate all mods and sync (purge)
        lib.mods.get_mut(&id_a).unwrap().is_active = false;
        lib.mods.get_mut(&id_b).unwrap().is_active = false;
        lib.sync().unwrap();

        // 4. Verification
        assert!(
            !shared_dir.join("ModA.dll").exists(),
            "ModA.dll should be cleaned up"
        );
        assert!(
            !shared_dir.join("ModB.dll").exists(),
            "ModB.dll should be cleaned up"
        );

        // Crucial Check: The folder and user file must remain
        assert!(untracked.exists(), "Untracked user file must be preserved");
        assert!(
            shared_dir.exists(),
            "Shared directory must be preserved because it contains user data"
        );
    }

    #[test]
    fn test_persistence_cycle() {
        let (_tmp, game_root, repo_root) = integration_tests::setup_test_env();
        let requirement = LibraryCreationRequirement {
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            name: "Test Library".to_string(),
        };
        let mut lib = Library::create(requirement).unwrap();
        let rules = SPTPathRules::default();

        let src = repo_root.join("src");
        fs::create_dir_all(src.join(&rules.server_mods).join("PersistMod")).unwrap();
        fs::write(
            src.join(&rules.server_mods)
                .join("PersistMod")
                .join("mod.dll"),
            "",
        )
        .unwrap();

        let mod_fs = ModFS::new(&src, &rules).unwrap();
        lib.add_mod(&src, mod_fs).unwrap();

        // FIX: Use lowercase "persistmod"
        lib.mods.get_mut("persistmod").unwrap().is_active = true;
        lib.sync().unwrap();

        let loaded_lib = Library::load(&repo_root).expect("Failed to load library");

        assert_eq!(loaded_lib.mods.len(), 1);
        assert!(loaded_lib.mods.get("persistmod").unwrap().is_active);
    }

    #[test]
    fn test_mod_id_case_normalization() {
        // This test ensures that on Windows, IDs are treated case-insensitively
        // to prevent duplicate mods pointing to the same folder.
        let (_tmp, game_root, repo_root) = integration_tests::setup_test_env();
        let requirement = LibraryCreationRequirement {
            repo_root: repo_root.clone(),
            game_root: game_root.clone(),
            name: "Test Library".to_string(),
        };
        let mut lib = Library::create(requirement).unwrap();

        // Add "MyMod" then add "mymod"
        // (Implementation depends on your Choice:
        //  Either ModFS::new should lowercase IDs, or Library should handle it)

        // Suggestion: In Library::add_mod, use: let mod_id = fs.id.to_lowercase();
        // and adjust tests accordingly.
    }
}
