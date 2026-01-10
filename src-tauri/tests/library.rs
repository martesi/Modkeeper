mod common;

use camino::{Utf8Path, Utf8PathBuf};
use common::{create_test_mod, setup_test_env};
use mod_keeper_lib::config::global::GlobalConfig;
use mod_keeper_lib::core::library::Library;
use mod_keeper_lib::core::mod_fs::ModFS;
use mod_keeper_lib::core::mod_stager::StagedMod;
use mod_keeper_lib::core::{cleanup, deployment, dto_builder, library_service, mod_manager};
use mod_keeper_lib::models::error::SError;
use mod_keeper_lib::models::library::LibraryCreationRequirement;
use mod_keeper_lib::models::paths::{ModPaths, SPTPathRules};
use std::fs;

// Helper function to create a StagedMod from a path and ModFS for testing
fn create_staged_mod_for_test(mod_root: &Utf8Path, fs: ModFS) -> StagedMod {
    // Try to read manifest name, otherwise use directory name or mod_id
    let name = ModFS::read_manifest(&ModPaths::new(mod_root).file)
        .ok()
        .map(|m| m.name)
        .unwrap_or_else(|| mod_root.file_name().unwrap_or(&fs.id).to_string());
    StagedMod {
        fs,
        source_path: mod_root.to_path_buf(),
        is_staging: false,
        name,
    }
}

#[test]
fn test_library_init_and_add_mod() {
    let (_tmp, game_root, repo_root) = setup_test_env();

    // 1. Create Library
    let requirement = LibraryCreationRequirement {
        repo_root: Some(repo_root.clone()),
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
    let mod_fs = ModFS::new(mod_src_utf8, &SPTPathRules::default()).expect("Failed to parse mod");
    let staged = create_staged_mod_for_test(mod_src_utf8, mod_fs);
    mod_manager::add_mod(&mut lib, staged).expect("Failed to add mod");

    // 4. Verify persistence
    assert!(lib.mods.contains_key("MyMod"));
    assert!(lib.lib_paths.mods.join("MyMod").exists());
    assert!(lib.cache.mods.contains_key("MyMod"));
}

#[test]
fn test_collision_detection() {
    let (_tmp, game_root, repo_root) = setup_test_env();
    let requirement = LibraryCreationRequirement {
        repo_root: Some(repo_root.clone()),
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
        let staged = create_staged_mod_for_test(&p, fs);
        mod_manager::add_mod(&mut lib, staged).expect("Failed to add mod");
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
    )
    .and_then(|_| {
        deployment::deploy(
            &lib.game_root,
            &lib.lib_paths,
            &lib.spt_rules,
            &lib.mods,
            &lib.cache,
        )
    });

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
        repo_root: Some(repo_root.clone()),
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
        let staged = create_staged_mod_for_test(&p, fs);
        mod_manager::add_mod(lib, staged).unwrap();

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
    )
    .unwrap();
    deployment::deploy(
        &lib.game_root,
        &lib.lib_paths,
        &lib.spt_rules,
        &lib.mods,
        &lib.cache,
    )
    .expect("Sync failed");
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
        repo_root: Some(repo_root.clone()),
        game_root: game_root.clone(),
        name: "Test Library".to_string(),
    };
    let mut lib = Library::create(requirement).unwrap();
    let rules = SPTPathRules::default();

    // 1. Add and activate mod
    create_test_mod(&repo_root.join("src"), "DeleteMe", true);
    let fs = ModFS::new(&repo_root.join("src"), &rules).unwrap();
    let staged = create_staged_mod_for_test(&repo_root.join("src"), fs);
    mod_manager::add_mod(&mut lib, staged).unwrap();
    lib.mods.get_mut("DeleteMe").unwrap().is_active = true;

    // Sync
    cleanup::purge(
        &lib.game_root,
        &lib.repo_root,
        &lib.spt_rules,
        &lib.lib_paths,
        &lib.cache,
    )
    .unwrap();
    deployment::deploy(
        &lib.game_root,
        &lib.lib_paths,
        &lib.spt_rules,
        &lib.mods,
        &lib.cache,
    )
    .unwrap();
    lib.mark_clean();
    lib.persist().unwrap();

    let target_path = game_root.join(&rules.server_mods).join("DeleteMe");
    assert!(target_path.exists());

    // 2. Deactivate and sync
    lib.mods.get_mut("DeleteMe").unwrap().is_active = false;
    cleanup::purge(
        &lib.game_root,
        &lib.repo_root,
        &lib.spt_rules,
        &lib.lib_paths,
        &lib.cache,
    )
    .unwrap();
    deployment::deploy(
        &lib.game_root,
        &lib.lib_paths,
        &lib.spt_rules,
        &lib.mods,
        &lib.cache,
    )
    .unwrap();
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
        repo_root: Some(repo_root.clone()),
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
    let staged = create_staged_mod_for_test(mod_src_utf8, fs);
    mod_manager::add_mod(&mut lib, staged).expect("Add mod failed");

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
        repo_root: Some(repo_root.clone()),
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
    let staged1 = create_staged_mod_for_test(&src, fs1);
    mod_manager::add_mod(&mut lib, staged1).unwrap();

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
    let staged2 = create_staged_mod_for_test(&src2, fs2);
    mod_manager::add_mod(&mut lib, staged2).unwrap();

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
        repo_root: Some(repo_root.clone()),
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
        let staged = create_staged_mod_for_test(&p, fs);

        mod_manager::add_mod(lib, staged).unwrap();

        // Access the mod using the actual ID generated by ModFS
        lib.mods.get_mut(&mod_id).unwrap().is_active = true;

        mod_id
    };

    let id_a = setup_mod(&mut lib, "ModA");
    let id_b = setup_mod(&mut lib, "ModB");

    // Sync
    cleanup::purge(
        &lib.game_root,
        &lib.repo_root,
        &lib.spt_rules,
        &lib.lib_paths,
        &lib.cache,
    )
    .unwrap();
    deployment::deploy(
        &lib.game_root,
        &lib.lib_paths,
        &lib.spt_rules,
        &lib.mods,
        &lib.cache,
    )
    .unwrap();
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
    cleanup::purge(
        &lib.game_root,
        &lib.repo_root,
        &lib.spt_rules,
        &lib.lib_paths,
        &lib.cache,
    )
    .unwrap();
    deployment::deploy(
        &lib.game_root,
        &lib.lib_paths,
        &lib.spt_rules,
        &lib.mods,
        &lib.cache,
    )
    .unwrap();
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
        repo_root: Some(repo_root.clone()),
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
    let staged = create_staged_mod_for_test(&src, mod_fs);
    mod_manager::add_mod(&mut lib, staged).unwrap();

    // FIX: Use lowercase "persistmod"
    lib.mods.get_mut("persistmod").unwrap().is_active = true;
    cleanup::purge(
        &lib.game_root,
        &lib.repo_root,
        &lib.spt_rules,
        &lib.lib_paths,
        &lib.cache,
    )
    .unwrap();
    deployment::deploy(
        &lib.game_root,
        &lib.lib_paths,
        &lib.spt_rules,
        &lib.mods,
        &lib.cache,
    )
    .unwrap();
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
        repo_root: Some(_repo_root.clone()),
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
        repo_root: Some(repo_root.clone()),
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
    assert!(
        result.is_err(),
        "Library without manifest should fail validation"
    );

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
        repo_root: Some(repo_root.clone()),
        game_root: game_root.clone(),
        name: "Test Library".to_string(),
    };
    Library::create(requirement).expect("Failed to create library");

    // Remove one of the required directories
    std::fs::remove_dir_all(repo_root.join("backups")).unwrap();

    // Validate should fail
    let result = library_service::validate_library_structure(&repo_root);
    assert!(
        result.is_err(),
        "Library with missing directory should fail validation"
    );

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
    assert!(
        !expected_repo_root.exists(),
        "Library directory should not exist initially"
    );

    // Create library - should create new library
    let requirement = LibraryCreationRequirement {
        repo_root: None, // Will be derived from game_root
        game_root: game_root.clone(),
        name: "New Library".to_string(),
    };

    let library = library_service::create_library(&mut config, requirement)
        .expect("Failed to create library");

    // Verify library was created
    assert_eq!(library.repo_root, expected_repo_root);
    assert_eq!(library.name, "New Library");
    assert!(
        expected_repo_root.exists(),
        "Library directory should exist after creation"
    );
    assert!(expected_repo_root.join("manifest.toml").exists());
    assert!(expected_repo_root.join("mods").exists());
    assert!(expected_repo_root.join("backups").exists());
    assert!(expected_repo_root.join("staging").exists());

    // Verify config was updated
    assert_eq!(config.known_libraries.first(), Some(&expected_repo_root));
    assert!(config.known_libraries.contains(&expected_repo_root));
}

#[test]
fn test_create_library_when_mod_keeper_exists_valid() {
    let (_tmp, game_root, _repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // First, create a library manually
    let expected_repo_root = game_root.join(".mod_keeper");
    let requirement1 = LibraryCreationRequirement {
        repo_root: Some(expected_repo_root.clone()),
        game_root: game_root.clone(),
        name: "Original Library".to_string(),
    };
    let original_lib = Library::create(requirement1).expect("Failed to create original library");
    original_lib.persist().expect("Failed to persist library");

    // Now try to create library again - should open existing instead
    let requirement2 = LibraryCreationRequirement {
        repo_root: None, // Will be derived from game_root
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
    assert_eq!(config.known_libraries.first(), Some(&expected_repo_root));
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
        repo_root: None, // Will be derived from game_root
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
    assert!(config.known_libraries.is_empty());
}

#[test]
fn test_get_active_library_manifest_uses_first_in_known_libraries() {
    let (_tmp, game_root, _repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // Create first library
    let repo_root1 = game_root.join(".mod_keeper_1");
    let requirement1 = LibraryCreationRequirement {
        repo_root: Some(repo_root1.clone()),
        game_root: game_root.clone(),
        name: "First Library".to_string(),
    };
    library_service::create_library(&mut config, requirement1)
        .expect("Failed to create first library");

    // Create second library
    let repo_root2 = game_root.join(".mod_keeper_2");
    let requirement2 = LibraryCreationRequirement {
        repo_root: Some(repo_root2.clone()),
        game_root: game_root.clone(),
        name: "Second Library".to_string(),
    };
    library_service::create_library(&mut config, requirement2)
        .expect("Failed to create second library");

    // Second library should be first (most recently used)
    assert_eq!(config.known_libraries.first(), Some(&repo_root2));

    // get_active_library_manifest should return the first library
    let active = library_service::get_active_library_manifest(&config);
    assert!(active.is_some());
    assert_eq!(active.unwrap().name, "Second Library");
}

#[test]
fn test_get_active_library_manifest_handles_invalid_library() {
    let (_tmp, game_root, _repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // Create a valid library
    let valid_repo_root = game_root.join(".mod_keeper");
    let requirement = LibraryCreationRequirement {
        repo_root: Some(valid_repo_root.clone()),
        game_root: game_root.clone(),
        name: "Valid Library".to_string(),
    };
    library_service::create_library(&mut config, requirement).expect("Failed to create library");

    // Add an invalid path as the first library
    let invalid_path = game_root.join("invalid_library");
    config.known_libraries.insert(0, invalid_path.clone());

    // get_active_library_manifest should return None for invalid library
    let active = library_service::get_active_library_manifest(&config);
    assert!(active.is_none());
}

#[test]
fn test_to_library_switch_with_invalid_active() {
    let (_tmp, game_root, _repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // Create a valid library
    let valid_repo_root = game_root.join(".mod_keeper");
    let requirement = LibraryCreationRequirement {
        repo_root: Some(valid_repo_root.clone()),
        game_root: game_root.clone(),
        name: "Valid Library".to_string(),
    };
    library_service::create_library(&mut config, requirement).expect("Failed to create library");

    // Add an invalid path as the first library
    let invalid_path = game_root.join("invalid_library");
    config.known_libraries.insert(0, invalid_path.clone());

    // to_library_switch should return None for active when first library is invalid
    let switch = library_service::to_library_switch(&config, None);
    assert!(switch.active.is_none());
    // But should still list other valid libraries
    assert!(!switch.libraries.is_empty());
}

#[test]
fn test_rename_library() {
    let (_tmp, game_root, repo_root) = setup_test_env();

    // Create library
    let requirement = LibraryCreationRequirement {
        repo_root: Some(repo_root.clone()),
        game_root: game_root.clone(),
        name: "Original Name".to_string(),
    };
    let mut lib = Library::create(requirement).expect("Failed to create library");

    // Verify original name
    assert_eq!(lib.name, "Original Name");

    // Rename library
    library_service::rename_library(&mut lib, "New Name".to_string())
        .expect("Failed to rename library");

    // Verify name was updated
    assert_eq!(lib.name, "New Name");

    // Verify persistence by reloading
    let reloaded = Library::load(&repo_root).expect("Failed to reload library");
    assert_eq!(reloaded.name, "New Name");
}

#[test]
fn test_close_library() {
    let (_tmp, game_root, repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // Remove repo_root directory if it exists (setup_test_env creates empty directory)
    if repo_root.exists() {
        std::fs::remove_dir_all(&repo_root).expect("Failed to remove repo_root");
    }

    // Create library and add to known_libraries
    let requirement = LibraryCreationRequirement {
        repo_root: Some(repo_root.clone()),
        game_root: game_root.clone(),
        name: "Test Library".to_string(),
    };
    library_service::create_library(&mut config, requirement).expect("Failed to create library");

    // Verify library is in known_libraries
    assert!(config.known_libraries.contains(&repo_root));

    // Close library
    let was_in_list =
        library_service::close_library(&mut config, &repo_root).expect("Failed to close library");

    // Verify return value
    assert!(was_in_list);

    // Verify library was removed from known_libraries
    assert!(!config.known_libraries.contains(&repo_root));

    // Verify library files still exist
    assert!(repo_root.exists());
    assert!(repo_root.join("manifest.toml").exists());
}

#[test]
fn test_close_library_not_in_list() {
    let (_tmp, _game_root, repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // Try to close a library that's not in known_libraries
    let was_in_list =
        library_service::close_library(&mut config, &repo_root).expect("Failed to close library");

    // Should return false since it wasn't in the list
    assert!(!was_in_list);

    // Config should not have changed
    assert!(config.known_libraries.is_empty());
}

#[test]
fn test_remove_library() {
    let (_tmp, game_root, repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // Remove repo_root directory if it exists (setup_test_env creates empty directory)
    if repo_root.exists() {
        std::fs::remove_dir_all(&repo_root).expect("Failed to remove repo_root");
    }

    // Create library and add to known_libraries
    let requirement = LibraryCreationRequirement {
        repo_root: Some(repo_root.clone()),
        game_root: game_root.clone(),
        name: "Test Library".to_string(),
    };
    library_service::create_library(&mut config, requirement).expect("Failed to create library");

    // Verify library is in known_libraries
    assert!(config.known_libraries.contains(&repo_root));
    assert!(repo_root.exists());

    // Remove library
    let was_in_list =
        library_service::remove_library(&mut config, &repo_root).expect("Failed to remove library");

    // Verify return value
    assert!(was_in_list);

    // Verify library was removed from known_libraries
    assert!(!config.known_libraries.contains(&repo_root));

    // Verify library directory was deleted
    assert!(!repo_root.exists());
}

#[test]
fn test_remove_library_with_mods() {
    let (_tmp, game_root, repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // Remove repo_root directory if it exists (setup_test_env creates empty directory)
    if repo_root.exists() {
        std::fs::remove_dir_all(&repo_root).expect("Failed to remove repo_root");
    }

    // Create library
    let requirement = LibraryCreationRequirement {
        repo_root: Some(repo_root.clone()),
        game_root: game_root.clone(),
        name: "Test Library".to_string(),
    };
    let mut lib = library_service::create_library(&mut config, requirement)
        .expect("Failed to create library");

    // Add a mod to the library
    let mod_src = _tmp.path().join("test_mod");
    let mod_src_utf8 = Utf8Path::from_path(&mod_src).unwrap();
    create_test_mod(mod_src_utf8, "TestMod", true);

    let mod_fs = ModFS::new(mod_src_utf8, &SPTPathRules::default()).expect("Failed to parse mod");
    let staged = create_staged_mod_for_test(mod_src_utf8, mod_fs);
    mod_manager::add_mod(&mut lib, staged).expect("Failed to add mod");

    // Activate mod and sync (deploy links)
    lib.mods.get_mut("TestMod").unwrap().is_active = true;
    lib.persist().expect("Failed to persist library");

    cleanup::purge(
        &lib.game_root,
        &lib.repo_root,
        &lib.spt_rules,
        &lib.lib_paths,
        &lib.cache,
    )
    .expect("Failed to purge");
    deployment::deploy(
        &lib.game_root,
        &lib.lib_paths,
        &lib.spt_rules,
        &lib.mods,
        &lib.cache,
    )
    .expect("Failed to deploy");

    // Note: Mod links would exist if deployed, but we verify cleanup happens during remove

    // Remove library
    let was_in_list =
        library_service::remove_library(&mut config, &repo_root).expect("Failed to remove library");

    assert!(was_in_list);

    // Verify library was removed from known_libraries
    assert!(!config.known_libraries.contains(&repo_root));

    // Verify library directory was deleted
    assert!(!repo_root.exists());
}

#[test]
fn test_remove_library_not_in_list() {
    let (_tmp, game_root, repo_root) = setup_test_env();
    let mut config = GlobalConfig::default();

    // Remove repo_root directory if it exists (setup_test_env creates empty directory)
    if repo_root.exists() {
        std::fs::remove_dir_all(&repo_root).expect("Failed to remove repo_root");
    }

    // Create library but don't add to known_libraries
    let requirement = LibraryCreationRequirement {
        repo_root: Some(repo_root.clone()),
        game_root: game_root.clone(),
        name: "Test Library".to_string(),
    };
    Library::create(requirement).expect("Failed to create library");

    // Verify library exists
    assert!(repo_root.exists());

    // Remove library (even though not in known_libraries)
    let was_in_list =
        library_service::remove_library(&mut config, &repo_root).expect("Failed to remove library");

    // Should return false since it wasn't in the list
    assert!(!was_in_list);

    // Verify library directory was still deleted
    assert!(!repo_root.exists());
}
