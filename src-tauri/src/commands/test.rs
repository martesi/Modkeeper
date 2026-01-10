use crate::models::error::SError;
use crate::models::paths::SPTPathRules;
use camino::Utf8PathBuf;
use std::fs;
use uuid::Uuid;

const DEFAULT_SPT_VERSION: &str = "SPT 4.0.11 - 278e72";

/// Creates a simulation game root structure for testing purposes.
/// This command is only available in debug builds.
///
/// Creates all required SPT files and directory structure:
/// - SPT/SPT.Server.exe
/// - EscapeFromTarkov.exe
/// - SPT/user/sptRegistry/registry.json with the default SPT version
/// - Directory structure for SPT/user/mods and BepInEx/plugins
///
/// Returns the path of the generated game root directory.
#[tauri::command]
#[specta::specta]
pub async fn create_simulation_game_root(base_path: Option<String>) -> Result<String, SError> {
    // Run in blocking thread since we're doing file I/O
    let base_path_utf8 = base_path.map(Utf8PathBuf::from);

    tauri::async_runtime::spawn_blocking(move || {
        create_simulation_game_root_internal(base_path_utf8)
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

fn create_simulation_game_root_internal(base_path: Option<Utf8PathBuf>) -> Result<String, SError> {
    // Determine game root path
    let game_root = if let Some(path) = base_path {
        // Check if path is empty (empty string)
        if path.as_str().trim().is_empty() {
            // If empty, use temp directory directly as game root (no subdirectory)
            let temp_dir = std::env::temp_dir();
            let test_dir_name = format!("mod_keeper_test_{}", Uuid::new_v4());
            let test_path = temp_dir.join(test_dir_name);
            let game_root = Utf8PathBuf::from_path_buf(test_path)
                .map_err(|e| SError::IOError(format!("Failed to convert path: {}", e.display())))?;
            fs::create_dir_all(&game_root)?;
            game_root
        } else {
            // Non-empty path provided - create a test directory under it, then "game" subdirectory
            let test_dir_name = format!("mod_keeper_test_{}", Uuid::new_v4());
            let test_dir = path.join(test_dir_name);
            fs::create_dir_all(&test_dir)?;
            test_dir.join("game")
        }
    } else {
        // No path provided - use temp directory directly as game root (no subdirectory)
        let temp_dir = std::env::temp_dir();
        let test_dir_name = format!("mod_keeper_test_{}", Uuid::new_v4());
        let test_path = temp_dir.join(test_dir_name);
        let game_root = Utf8PathBuf::from_path_buf(test_path)
            .map_err(|e| SError::IOError(format!("Failed to convert path: {}", e.display())))?;
        fs::create_dir_all(&game_root)?;
        game_root
    };

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
    let registry_json = format!(r#"{{"SPT_Version": "{}"}}"#, DEFAULT_SPT_VERSION);
    fs::write(&rules.server_registry, registry_json)?;

    // Create directory structure for mods
    fs::create_dir_all(&rules.server_mods)?;
    fs::create_dir_all(&rules.client_plugins)?;

    Ok(game_root.to_string())
}
