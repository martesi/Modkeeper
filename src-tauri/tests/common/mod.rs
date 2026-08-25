use camino::{Utf8Path, Utf8PathBuf};
use mod_keeper_lib::models::paths::SPTPathRules;
use std::fs;
use tempfile::TempDir;

pub fn setup_test_env() -> (TempDir, Utf8PathBuf, Utf8PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();

    let game_root = root.join("game");
    let repo_root = root.join("repo");
    fs::create_dir_all(&game_root).unwrap();
    fs::create_dir_all(&repo_root).unwrap();

    let game_root = canonicalize(&game_root);
    let repo_root = canonicalize(&repo_root);
    let rules = SPTPathRules::new(&game_root);

    for path in [&rules.server_exe, &rules.client_exe] {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "dummy").unwrap();
    }

    if let Some(parent) = rules.server_registry.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let registry_json = r#"{"SPT_Version": "SPT 4.0.11 - 278e72"}"#;
    fs::write(&rules.server_registry, registry_json).unwrap();

    (tmp, game_root, repo_root)
}

#[allow(dead_code)]
pub fn create_test_mod(path: &Utf8Path, name: &str, is_server: bool) {
    let rules = SPTPathRules::default();
    let mod_dir = if is_server {
        path.join(rules.server_mods).join(name)
    } else {
        path.join(rules.client_plugins).join(name)
    };

    fs::create_dir_all(&mod_dir).unwrap();
    fs::write(mod_dir.join("content.txt"), name).unwrap();
}

fn canonicalize(path: &Utf8Path) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(dunce::canonicalize(path).unwrap()).unwrap()
}
