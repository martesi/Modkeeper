use crate::models::error::SError;
use crate::models::paths::SPTPathRules;
use crate::models::test::{CreateSimulationGameRootOptions, TestGameRoot};
use camino::Utf8PathBuf;
use std::fs;
use uuid::Uuid;

/// Creates a simulation game root structure for testing purposes.
/// This command is only available in debug builds.
///
/// Creates all required SPT files and directory structure:
/// - SPT/SPT.Server.exe
/// - EscapeFromTarkov.exe
/// - SPT/user/sptRegistry/registry.json with the specified SPT version
/// - Directory structure for SPT/user/mods and BepInEx/plugins
#[tauri::command]
#[specta::specta]
pub async fn create_simulation_game_root(
    options: CreateSimulationGameRootOptions,
) -> Result<TestGameRoot, SError> {
    // Run in blocking thread since we're doing file I/O
    tauri::async_runtime::spawn_blocking(move || create_simulation_game_root_internal(options))
        .await
        .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

fn create_simulation_game_root_internal(
    options: CreateSimulationGameRootOptions,
) -> Result<TestGameRoot, SError> {
    // Determine base directory
    let (base_path, temp_dir_path) = if let Some(path) = options.base_path {
        (path, None)
    } else {
        // Create a persistent directory in the system temp location
        let temp_dir = std::env::temp_dir();
        let test_dir_name = format!("mod_keeper_test_{}", Uuid::new_v4());
        let test_path = temp_dir.join(test_dir_name);
        let path = Utf8PathBuf::from_path_buf(test_path)
            .map_err(|e| SError::IOError(format!("Failed to convert path: {}", e.display())))?;
        fs::create_dir_all(&path)?;
        (path.clone(), Some(path.to_string()))
    };

    let game_root = base_path.join("game");

    // Create directories
    fs::create_dir_all(&game_root)?;

    // Get the rules to find where SPT expects files
    let rules = SPTPathRules::new(&game_root);

    // Create essential SPT files so canonicalize() doesn't fail
    let essential_files = [&rules.server_exe, &rules.client_exe];

    for path in essential_files.iter() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, "dummy")?;
    }

    // Create registry.json file with SPT_Version
    if let Some(parent) = rules.server_registry.parent() {
        fs::create_dir_all(parent)?;
    }
    let registry_json = format!(r#"{{"SPT_Version": "{}"}}"#, options.spt_version);
    fs::write(&rules.server_registry, registry_json)?;

    // Create directory structure for mods
    fs::create_dir_all(&rules.server_mods)?;
    fs::create_dir_all(&rules.client_plugins)?;

    Ok(TestGameRoot {
        game_root,
        temp_dir_path,
    })
}
