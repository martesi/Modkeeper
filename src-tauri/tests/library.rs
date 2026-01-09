mod common;

use mod_keeper_lib::core::{cleanup, deployment, dto_builder, mod_manager, library_service};
use mod_keeper_lib::core::library::Library;
use mod_keeper_lib::core::mod_fs::ModFS;
use mod_keeper_lib::models::library::LibraryCreationRequirement;
use mod_keeper_lib::models::paths::SPTPathRules;
use mod_keeper_lib::models::error::SError;
use mod_keeper_lib::config::global::GlobalConfig;
use common::{create_test_mod, setup_test_env};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

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
    mod_manager::add_mod(&mut lib, mod_src_utf8, mod_fs)
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
            r#"{{"id": "{}", "name": "{}", "version": "1.0", "author": "test", "sptVersion": "3.9.0"}}"#,
            mod_id, mod_id
        );
        fs::write(manifest_dir.join("manifest.json"), manifest_json).unwrap();

        // 2. Create the colliding file
        let file_path = p.join(&colliding_file);
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(file_path, "some content").unwrap();

        // 3. Add to library and activate
        let fs = ModFS::new(&p, &rules).expect("Failed to parse mod");
        mod_manager::add_mod(&mut lib, &p, fs).expect("Failed to add mod");
        lib.mods.get_mut(mod_id).unwrap().is_active = true;
    };

    // These two mods have different IDs but both provide "BepInEx/plugins/conflict.dll"
    let conflict_path = "BepInEx/plugins/conflict.dll";
    add_named_mod("Mod_A", conflict_path);
    add_named_mod("Mod_B", conflict_path);

    // Act - sync using standalone functions
    let result = cleanup::purge(
        &lib.game_root,
        &lib.repo_root,
        &lib.spt_rules,
        &lib.lib_paths,
        &lib.cache,
    ).and_then(|_| deployment::deploy(
        &lib.game_root,
        &lib.lib_paths,
        &lib.spt_rules,
        &lib.mods,
        &lib.cache,
    ));

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

    let setup_mod = |lib: &mut Library, mod_id: &str, file_name: &str| {
        let p = repo_root.join(mod_id);

        // 1. Force the ID using a manifest
        let manifest_dir = p.join("manifest");
        fs::create_dir_all(&manifest_dir).unwrap();
        let manifest_json = format!(
            r#"{{"id": "{}", "name": "{}", "version": "1", "author": "t", "sptVersion": "3.9.0"}}"#,
            mod_id, mod_id
        );
        fs::write(manifest_dir.join("manifest.json"), manifest_json).unwrap();

        // 2. Create the overlapping directory structure
        let file_path = p.join(&rules.server_mods).join("CommonDir").join(file_name);
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(file_path, "data").unwrap();

        // 3. New ModFS will now resolve ID to mod_id ("ModA" or "ModB")
        let fs = ModFS::new(&p, &rules).unwrap();
        mod_manager::add_mod(lib, &p, fs).unwrap();

        // 4. This will no longer panic
        lib.mods.get_mut(mod_id).unwrap().is_active = true;
    };

    setup_mod(&mut lib, "ModA", "A.txt");
    setup_mod(&mut lib, "ModB", "B.txt");

    // Sync using standalone functions
    cleanup::purge(
        &lib.game_root,
        &lib.repo_root,
        &lib.spt_rules,
        &lib.lib_paths,
        &lib.cache,
    ).unwrap();
    deployment::deploy(
        &lib.game_root,
        &lib.lib_paths,
        &lib.spt_rules,
        &lib.mods,
        &lib.cache,
    ).expect("Sync failed");
    lib.mark_clean();
    lib.persist().unwrap();

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
    mod_manager::add_mod(&mut lib, &repo_root.join("src"), fs).unwrap();
    lib.mods.get_mut("DeleteMe").unwrap().is_active = true;

    // Sync
    cleanup::purge(&lib.game_root, &lib.repo_root, &lib.spt_rules, &lib.lib_paths, &lib.cache).unwrap();
    deployment::deploy(&lib.game_root, &lib.lib_paths, &lib.spt_rules, &lib.mods, &lib.cache).unwrap();
    lib.mark_clean();
    lib.persist().unwrap();

    let target_path = game_root.join(&rules.server_mods).join("DeleteMe");
    assert!(target_path.exists());

    // 2. Deactivate and sync
    lib.mods.get_mut("DeleteMe").unwrap().is_active = false;
    cleanup::purge(&lib.game_root, &lib.repo_root, &lib.spt_rules, &lib.lib_paths, &lib.cache).unwrap();
    deployment::deploy(&lib.game_root, &lib.lib_paths, &lib.spt_rules, &lib.mods, &lib.cache).unwrap();
    lib.mark_clean();
    lib.persist().unwrap();

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
        "id": "test-mod-id",
        "name": "Test Mod Name",
        "version": "1.0.0",
        "author": "someone",
        "sptVersion": "3.9.0"
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
    mod_manager::add_mod(&mut lib, mod_src_utf8, fs).expect("Add mod failed");

    // 4. Check Frontend DTO
    let dto = dto_builder::build_frontend_dto(&lib);
    let m = dto.mods.get("test-mod-id").expect("Mod not found in DTO");

    // Assert the manifest was successfully pulled from cache into the DTO
    assert!(m.manifest.is_some());
    assert_eq!(m.manifest.as_ref().unwrap().name, "Test Mod Name");
}

#[test]
fn test_mod_backup_on_overwrite() {
    let (_tmp, game_root, repo_root) = setup_test_env();
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
    mod_manager::add_mod(&mut lib, &src, fs1).unwrap();

    // Wait to ensure timestamp differs
    std::thread::sleep(std::time::Duration::from_secs(1));

    // 2. Overwrite Add
    let src2 = repo_root.join("src_v2");
    fs::create_dir_all(src2.join(&rules.server_mods).join(mod_id)).unwrap();
    fs::write(
        src2.join(&rules.server_mods).join(mod_id).join("v2.txt"),
        "v2",
    )
    .unwrap();

    let fs2 = ModFS::new(&src2, &rules).unwrap();
    mod_manager::add_mod(&mut lib, &src2, fs2).unwrap();

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
    let (_tmp, game_root, repo_root) = setup_test_env();
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
    let setup_mod = |lib: &mut Library, name: &str| {
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

        mod_manager::add_mod(lib, &p, fs).unwrap();

        // Access the mod using the actual ID generated by ModFS
        lib.mods.get_mut(&mod_id).unwrap().is_active = true;

        mod_id
    };

    let id_a = setup_mod(&mut lib, "ModA");
    let id_b = setup_mod(&mut lib, "ModB");

    // Sync
    cleanup::purge(&lib.game_root, &lib.repo_root, &lib.spt_rules, &lib.lib_paths, &lib.cache).unwrap();
    deployment::deploy(&lib.game_root, &lib.lib_paths, &lib.spt_rules, &lib.mods, &lib.cache).unwrap();
    lib.mark_clean();
    lib.persist().unwrap();

    // 2. Add untracked file to the real directory created by the Linker
    let shared_dir = game_root.join(&rules.client_plugins).join("SharedDir");
    let untracked = shared_dir.join("user_notes.txt");

    assert!(shared_dir.exists(), "SharedDir should exist after sync");
    fs::write(&untracked, "user data").unwrap();

    // 3. Deactivate all mods and sync (purge)
    lib.mods.get_mut(&id_a).unwrap().is_active = false;
    lib.mods.get_mut(&id_b).unwrap().is_active = false;
    cleanup::purge(&lib.game_root, &lib.repo_root, &lib.spt_rules, &lib.lib_paths, &lib.cache).unwrap();
    deployment::deploy(&lib.game_root, &lib.lib_paths, &lib.spt_rules, &lib.mods, &lib.cache).unwrap();
    lib.mark_clean();
    lib.persist().unwrap();

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
    let (_tmp, game_root, repo_root) = setup_test_env();
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
    mod_manager::add_mod(&mut lib, &src, mod_fs).unwrap();

    // FIX: Use lowercase "persistmod"
    lib.mods.get_mut("persistmod").unwrap().is_active = true;
    cleanup::purge(&lib.game_root, &lib.repo_root, &lib.spt_rules, &lib.lib_paths, &lib.cache).unwrap();
    deployment::deploy(&lib.game_root, &lib.lib_paths, &lib.spt_rules, &lib.mods, &lib.cache).unwrap();
    lib.mark_clean();
    lib.persist().unwrap();

    let loaded_lib = Library::load(&repo_root).expect("Failed to load library");

    assert_eq!(loaded_lib.mods.len(), 1);
    assert!(loaded_lib.mods.get("persistmod").unwrap().is_active);
}

#[test]
fn test_mod_id_case_normalization() {
    // This test ensures that on Windows, IDs are treated case-insensitively
    // to prevent duplicate mods pointing to the same folder.
    let (_tmp, _game_root, _repo_root) = setup_test_env();
    let requirement = LibraryCreationRequirement {
        repo_root: _repo_root.clone(),
        game_root: _game_root.clone(),
        name: "Test Library".to_string(),
    };
    let _lib = Library::create(requirement).unwrap();

    // Add "MyMod" then add "mymod"
    // (Implementation depends on your Choice:
    //  Either ModFS::new should lowercase IDs, or Library should handle it)

    // Suggestion: In Library::add_mod, use: let mod_id = fs.id.to_lowercase();
    // and adjust tests accordingly.
}

#[test]
fn test_derive_library_root() {
    let (_tmp, game_root, _repo_root) = setup_test_env();

    let derived = library_service::derive_library_root(&game_root);
    let expected = game_root.join(".mod_keeper");

    assert_eq!(derived, expected);
}

#[test]
fn test_validate_library_structure_valid() {
    let (_tmp, game_root, repo_root) = setup_test_env();

    // Create a valid library
    let requirement = LibraryCreationRequirement {
        repo_root: repo_root.clone(),
        game_root: game_root.clone(),
        name: "Test Library".to_string(),
    };
    Library::create(requirement).expect("Failed to create library");

    // Validate should succeed
    let result = library_service::validate_library_structure(&repo_root);
    assert!(result.is_ok(), "Valid library should pass validation");
}

#[test]
fn test_validate_library_structure_missing_manifest() {
    let (_tmp, _game_root, repo_root) = setup_test_env();

    // Create directory structure but no manifest
    std::fs::create_dir_all(repo_root.join("mods")).unwrap();
    std::fs::create_dir_all(repo_root.join("backups")).unwrap();
    std::fs::create_dir_all(repo_root.join("staging")).unwrap();

    // Validate should fail with InvalidLibrary error
    let result = library_service::validate_library_structure(&repo_root);
    assert!(result.is_err(), "Library without manifest should fail validation");

    match result {
        Err(SError::InvalidLibrary(path, reason)) => {
            assert_eq!(path, repo_root.to_string());
            assert!(reason.contains("manifest.toml"));
        }
        Err(e) => panic!("Expected InvalidLibrary error, got: {}", e),
        Ok(_) => panic!("Expected error but got Ok"),
    }
}

#[test]
fn test_validate_library_structure_missing_directory() {
    let (_tmp, game_root, repo_root) = setup_test_env();

    // Create a library
    let requirement = LibraryCreationRequirement {
        repo_root: repo_root.clone(),
        game_root: game_root.clone(),
        name: "Test Library".to_string(),
    };
    Library::create(requirement).expect("Failed to create library");

    // Remove one of the required directories
    std::fs::remove_dir_all(repo_root.join("backups")).unwrap();

    // Validate should fail
    let result = library_service::validate_library_structure(&repo_root);
    assert!(result.is_err(), "Library with missing directory should fail validation");

    match result {
        Err(SError::InvalidLibrary(path, reason)) => {
            assert_eq!(path, repo_root.to_string());
            assert!(reason.contains("backups") || reason.contains("missing required directory"));
        }
        Err(e) => panic!("Expected InvalidLibrary error, got: {}", e),
        Ok(_) => panic!("Expected error but got Ok"),
    }
}

#[test]
fn test_create_library_when_mod_keeper_not_exists() {
    let (_tmp, game_root, _repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // .mod_keeper should not exist yet
    let expected_repo_root = game_root.join(".mod_keeper");
    assert!(!expected_repo_root.exists(), "Library directory should not exist initially");

    // Create library - should create new library
    let requirement = LibraryCreationRequirement {
        repo_root: expected_repo_root.clone(), // This will be overridden by create_library
        game_root: game_root.clone(),
        name: "New Library".to_string(),
    };

    let library = library_service::create_library(&mut config, requirement)
        .expect("Failed to create library");

    // Verify library was created
    assert_eq!(library.repo_root, expected_repo_root);
    assert_eq!(library.name, "New Library");
    assert!(expected_repo_root.exists(), "Library directory should exist after creation");
    assert!(expected_repo_root.join("manifest.toml").exists());
    assert!(expected_repo_root.join("mods").exists());
    assert!(expected_repo_root.join("backups").exists());
    assert!(expected_repo_root.join("staging").exists());

    // Verify config was updated
    assert_eq!(config.last_opened, Some(expected_repo_root.clone()));
    assert!(config.known_libraries.contains(&expected_repo_root));
}

#[test]
fn test_create_library_when_mod_keeper_exists_valid() {
    let (_tmp, game_root, _repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // First, create a library manually
    let expected_repo_root = game_root.join(".mod_keeper");
    let requirement1 = LibraryCreationRequirement {
        repo_root: expected_repo_root.clone(),
        game_root: game_root.clone(),
        name: "Original Library".to_string(),
    };
    let original_lib = Library::create(requirement1).expect("Failed to create original library");
    original_lib.persist().expect("Failed to persist library");

    // Now try to create library again - should open existing instead
    let requirement2 = LibraryCreationRequirement {
        repo_root: expected_repo_root.clone(),
        game_root: game_root.clone(),
        name: "New Library Name".to_string(), // This name should be ignored
    };

    let opened_lib = library_service::create_library(&mut config, requirement2)
        .expect("Failed to open existing library");

    // Verify we got the original library, not a new one
    assert_eq!(opened_lib.id, original_lib.id);
    assert_eq!(opened_lib.name, original_lib.name); // Should keep original name
    assert_eq!(opened_lib.repo_root, expected_repo_root);

    // Verify config was updated
    assert_eq!(config.last_opened, Some(expected_repo_root.clone()));
}

#[test]
fn test_create_library_when_mod_keeper_exists_invalid() {
    let (_tmp, game_root, _repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // Create an invalid library directory (missing manifest)
    let expected_repo_root = game_root.join(".mod_keeper");
    std::fs::create_dir_all(expected_repo_root.join("mods")).unwrap();
    std::fs::create_dir_all(expected_repo_root.join("backups")).unwrap();
    std::fs::create_dir_all(expected_repo_root.join("staging")).unwrap();
    // Intentionally don't create manifest.toml

    // Try to create library - should return InvalidLibrary error
    let requirement = LibraryCreationRequirement {
        repo_root: expected_repo_root.clone(),
        game_root: game_root.clone(),
        name: "Invalid Library".to_string(),
    };

    let result = library_service::create_library(&mut config, requirement);

    assert!(result.is_err(), "Should return error for invalid library");

    match result {
        Err(SError::InvalidLibrary(path, reason)) => {
            assert_eq!(path, expected_repo_root.to_string());
            assert!(reason.contains("manifest.toml"));
        }
        Err(e) => panic!("Expected InvalidLibrary error, got: {}", e),
        Ok(_) => panic!("Expected error but got Ok"),
    }

    // Config should not be updated on error
    assert_eq!(config.last_opened, None);
}
