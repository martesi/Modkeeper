use crate::core::library::Library;
use crate::core::registry::AppRegistry;
use crate::models::error::SError;
use crate::models::global::LibrarySwitch;
use crate::utils::context::TaskContext;
use camino::Utf8PathBuf;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn open_library(
    state: State<'_, AppRegistry>,
    path: String,
) -> Result<LibrarySwitch, SError> {
    let path_buf = Utf8PathBuf::from(path);

    // Clone BOTH handles to move them into the blocking thread
    let config_handle = state.global_config.clone();
    let instance_handle = state.active_instance.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // 1. Lock Config, Load Library (IO), Update Config
        // We scope this block so we can release the config lock before acquiring the instance lock
        // (though keeping it held is also safe here since the order is fixed).
        let (lib, switch_dto) = {
            let mut config = config_handle.lock();
            let lib = config.open_library(&path_buf)?; // Uses the GlobalConfig::open we implemented
            (lib, config.to_library_switch())
        };

        // 2. Lock Instance and Swap
        // IMPORTANT: This drops the *old* Library instance.
        // Doing this here ensures any heavy resource cleanup (closing files, freeing RAM)
        // happens on this blocking thread, not the async runtime.
        *instance_handle.lock() = Some(lib);

        Ok(switch_dto)
    })
        .await
        .map_err(|e| SError::AsyncRuntimeError(e.to_string()))? // Unwraps JoinError
}