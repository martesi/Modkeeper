use camino::Utf8Path;
use mod_keeper_lib::models::paths::{ModPaths, SPTPathRules};
use std::fs;
use tempfile::TempDir;

/// Helper to setup a dummy SPT environment so Library::create doesn't fail
pub fn setup_test_env() -> (TempDir, camino::Utf8PathBuf, camino::Utf8PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();

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
pub fn create_test_mod(path: &Utf8Path, name: &str, is_server: bool) {
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
        r#"{{"id": "{}", "name": "{}", "version": "1.0.0", "author": "test", "sptVersion": "3.9.0"}}"#,
        name, name
    );
    fs::write(manifest_dir.join("manifest.json"), manifest_json).unwrap();
}
