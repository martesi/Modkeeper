use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use specta::Type;

/// Return type for game root creation command
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct TestGameRoot {
    #[specta(type=String)]
    pub game_root: Utf8PathBuf,
    #[specta(type=Option<String>)]
    pub temp_dir_path: Option<String>,
}

/// Options for creating a simulation game root
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct CreateSimulationGameRootOptions {
    /// SPT version string (default: "SPT 4.0.11 - 278e72")
    #[serde(default = "default_spt_version")]
    pub spt_version: String,
    /// Optional base path. If not provided, uses a temporary directory
    #[specta(type=Option<String>)]
    pub base_path: Option<Utf8PathBuf>,
}

fn default_spt_version() -> String {
    "SPT 4.0.11 - 278e72".to_string()
}

/// Options for creating test mods (for future use)
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct TestModOptions {
    pub mod_id: String,
    pub mod_name: String,
    pub mod_type: String, // "Client", "Server", or "Both"
    pub version: String,
}

/// Options for creating test libraries with mods (for future use)
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct TestLibraryOptions {
    #[specta(type=String)]
    pub game_root: Utf8PathBuf,
    pub name: String,
    pub mods: Option<Vec<TestModOptions>>,
}
