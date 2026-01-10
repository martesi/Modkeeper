use camino::Utf8PathBuf;
use mod_keeper_lib::core::mod_fs::ModFS;
use mod_keeper_lib::models::error::SError;
use mod_keeper_lib::models::mod_dto::ModType;
use mod_keeper_lib::models::paths::{ModPaths, SPTPathRules};
use mod_keeper_lib::utils::file::FileUtils;
use mod_keeper_lib::utils::id::hash_id;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_resolve_id_server_only() {
    let temp = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let rules = SPTPathRules::default();

    // Server structure: SPT/user/mods/WeatherMod/mod.dll
    let server_path = root.join(&rules.server_mods).join("WeatherMod/mod.dll");
    fs::create_dir_all(server_path.parent().unwrap()).unwrap();
    fs::write(server_path, "").unwrap();

    let mod_fs = ModFS::new(&root, &rules).unwrap();

    // Server ID should be hashed version of "weathermod"
    let expected_id = hash_id("weathermod");
    assert_eq!(mod_fs.id, expected_id);
    assert_eq!(mod_fs.mod_type, ModType::Server);
}

#[test]
fn test_resolve_id_client_only() {
    let temp = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let rules = SPTPathRules::default();

    // Client structure: BepInEx/plugins/AuthorName/Logic.dll
    let client_path = root
        .join(&rules.client_plugins)
        .join("AuthorName/Logic.dll");
    fs::create_dir_all(client_path.parent().unwrap()).unwrap();
    fs::write(client_path, "").unwrap();

    let mod_fs = ModFS::new(&root, &rules).unwrap();

    // Client ID should be hashed version of "authorname/logic.dll" (lowercase)
    let expected_id = hash_id("authorname/logic.dll");
    assert_eq!(mod_fs.id, expected_id);
    assert_eq!(mod_fs.mod_type, ModType::Client);
}

#[test]
fn test_resolve_id_combined() {
    let temp = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let rules = SPTPathRules::default();

    // Create both
    let s_path = root.join(&rules.server_mods).join("CoreMod/mod.dll");
    let c_path = root.join(&rules.client_plugins).join("Fixes.dll");

    fs::create_dir_all(s_path.parent().unwrap()).unwrap();
    fs::create_dir_all(c_path.parent().unwrap()).unwrap();
    fs::write(s_path, "").unwrap();
    fs::write(c_path, "").unwrap();

    let mod_fs = ModFS::new(&root, &rules).unwrap();

    // BTreeSet sorts alphabetically:
    // "CoreMod" vs "Fixes.dll"
    // Result: "coremodfixes.dll" (lowercase, no divider) -> hashed
    let expected_id = hash_id("coremodfixes.dll");
    assert_eq!(mod_fs.id, expected_id);
    assert_eq!(mod_fs.mod_type, ModType::Both);
}

#[test]
fn test_resolve_id_ignores_non_dll_in_client_plugins() {
    let temp = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let rules = SPTPathRules::default();

    // .txt files in plugins should be ignored for ID generation
    let txt_path = root.join(&rules.client_plugins).join("readme.txt");
    fs::create_dir_all(txt_path.parent().unwrap()).unwrap();
    fs::write(txt_path, "").unwrap();

    // Valid server mod to ensure the test doesn't fail on "empty"
    let s_path = root.join(&rules.server_mods).join("ValidMod/mod.dll");
    fs::create_dir_all(s_path.parent().unwrap()).unwrap();
    fs::write(s_path, "").unwrap();

    let mod_fs = ModFS::new(&root, &rules).unwrap();

    // ID should be hashed version of "validmod" (lowercase), the readme.txt is ignored
    let expected_id = hash_id("validmod");
    assert_eq!(mod_fs.id, expected_id);
}

#[test]
fn test_collect_files_excludes_manifest() {
    let temp = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let rules = SPTPathRules::default();

    // Create manifest folder
    let manifest_dir = root.join(ModPaths::default().folder);
    fs::create_dir_all(&manifest_dir).unwrap();
    fs::write(manifest_dir.join("info.json"), "").unwrap();

    // Create actual mod file
    let mod_file = root.join(&rules.server_mods).join("Mod/mod.dll");
    fs::create_dir_all(mod_file.parent().unwrap()).unwrap();
    fs::write(&mod_file, "").unwrap();

    let mod_fs = ModFS::new(&root, &rules).unwrap();

    // Only the dll should be in the files list
    assert_eq!(mod_fs.files.len(), 1);
    assert!(mod_fs.files[0].ends_with("mod.dll"));
}

#[test]
fn test_executables_detection() {
    let temp = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let rules = SPTPathRules::default();

    // Create a server dll (to pass ID resolution)
    let mod_dir = root.join(&rules.server_mods).join("ModWithTools");
    fs::create_dir_all(&mod_dir).unwrap();
    fs::write(mod_dir.join("mod.dll"), "").unwrap();

    // Create exes in different locations
    fs::write(root.join("root_tool.exe"), "").unwrap();
    let deep_dir = mod_dir.join("tools/sub");
    fs::create_dir_all(&deep_dir).unwrap();
    fs::write(deep_dir.join("nested_tool.exe"), "").unwrap();

    let mod_fs = ModFS::new(&root, &rules).unwrap();

    assert_eq!(mod_fs.executables.len(), 2);
    assert!(mod_fs
        .executables
        .iter()
        .any(|e| e.ends_with("root_tool.exe")));
    assert!(mod_fs
        .executables
        .iter()
        .any(|e| e.ends_with("nested_tool.exe")));
}

#[test]
fn test_copy_recursive_overwrite_and_deep_nesting() {
    let temp = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let src = root.join("src");
    let dst = root.join("dst");

    // 1. Setup source with deep nesting
    let deep_file = src.join("a/b/c/d.txt");
    fs::create_dir_all(deep_file.parent().unwrap()).unwrap();
    fs::write(&deep_file, "new content").unwrap();

    // 2. Setup destination with an old version of the file
    let old_file = dst.join("a/b/c/d.txt");
    fs::create_dir_all(old_file.parent().unwrap()).unwrap();
    fs::write(&old_file, "old content").unwrap();

    // 3. Act
    FileUtils::copy_recursive(&src, &dst).unwrap();

    // 4. Assert
    let content = fs::read_to_string(dst.join("a/b/c/d.txt")).unwrap();
    assert_eq!(content, "new content"); // Verified overwrite
}

#[test]
fn test_new_fails_for_invalid_structure() {
    let temp = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let rules = SPTPathRules::default();

    // Create files that don't match rules (not in SPT/user/mods or BepInEx/plugins)
    fs::create_dir_all(root.join("RandomFolder")).unwrap();
    fs::write(root.join("RandomFolder/something.dll"), "").unwrap();

    let result = ModFS::new(&root, &rules);

    assert!(result.is_err());
    // Match against your specific error variant
    match result.unwrap_err() {
        SError::UnableToDetermineModId => {}
        e => panic!("Expected UnableToDetermineModId, got {:?}", e),
    }
}

#[test]
fn test_deterministic_id_sorting() {
    let temp = tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
    let rules = SPTPathRules::default();

    // Create mods out of alphabetical order
    let mods = ["Z_mod", "A_mod", "M_mod"];
    for m in mods {
        let p = root.join(&rules.server_mods).join(m).join("mod.dll");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, "").unwrap();
    }

    let mod_fs = ModFS::new(&root, &rules).unwrap();

    // BTreeSet should have forced: a_modm_modz_mod (lowercase, no divider) -> hashed
    let expected_id = hash_id("a_modm_modz_mod");
    assert_eq!(mod_fs.id, expected_id);
}
