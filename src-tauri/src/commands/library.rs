use crate::core::mod_stager;
use crate::core::registry::AppRegistry;
use crate::models::error::SError;
use crate::models::library::LibraryDTO;
use crate::models::task_status::TaskStatus;
use crate::utils::context::TaskContext;
use crate::utils::thread::{with_lib_arc, with_lib_arc_mut};
use camino::Utf8PathBuf;
use tauri::ipc::Channel;
use tauri::State;
use tracing::{debug, info, instrument};

#[tauri::command]
#[specta::specta]
#[instrument(skip(channel, state))]
pub async fn add_mods(
    state: State<'_, AppRegistry>,
    paths: Vec<String>,
    channel: Channel<TaskStatus>,
) -> Result<LibraryDTO, SError> {
    let inputs = paths
        .into_iter()
        .map(Utf8PathBuf::from)
        .collect::<Vec<Utf8PathBuf>>();

    let material = state.get_stage_material()?;
    debug!("staging_material: {:?}", material);

    // Clone the Arc handle so we can move it into the 'static blocking thread.
    // 'state' cannot be moved, but the Arc inside it can be cloned.
    let instance_handle = state.active_instance.clone();

    TaskContext::provide(channel, move || {
        info!("Staging mod files");
        // 1. Resolve (Heavy Compute/IO)
        // We do this here to avoid blocking the async runtime
        let staged_mods = mod_stager::resolve(&inputs, &material)?;
        debug!("staged_mods: {:?}", staged_mods);

        with_lib_arc_mut(instance_handle, |inst| {
            info!("Adding mods to library");
            // 3. Install & Cleanup
            // Using try_for_each for early exit on error
            staged_mods
                .into_iter()
                .try_for_each(|staged| {
                    debug!("current: {:?}", staged);
                    inst.add_mod(&staged.source_path, staged.fs.clone())
                        .and_then(|_| mod_stager::clean_up(&staged))
                })
                .map(|_| inst.to_frontend_dto())
        })
    })
    .await??
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(channel, state))]
pub async fn remove_mods(
    state: State<'_, AppRegistry>,
    ids: Vec<String>,
    channel: Channel<TaskStatus>,
) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    // Offload synchronous file IO and locking to a blocking thread
    TaskContext::provide(channel, move || {
        with_lib_arc_mut(instance_handle, |inst| -> Result<LibraryDTO, SError> {
            ids.iter()
                .try_for_each(|mod_id| {
                    debug!("Removing mod {}", mod_id);
                    inst.remove_mod(mod_id)
                })
                .map(|_| inst.to_frontend_dto())
        })
        // Iterate and remove each mod, exiting immediately on the first error
    })
    .await??
}

#[tauri::command]
#[specta::specta]
#[instrument(skip_all)]
pub async fn sync_mods(
    state: State<'_, AppRegistry>,
    channel: Channel<TaskStatus>,
) -> Result<LibraryDTO, SError> {
    if state.is_game_or_server_running() {
        return Err(SError::GameOrServerRunning.into());
    }

    let instance_handle = state.active_instance.clone();
    TaskContext::provide(channel, move || {
        with_lib_arc_mut(instance_handle, |inst| {
            inst.sync().map(|_| inst.to_frontend_dto())
        })
    })
    .await??
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn get_library(state: State<'_, AppRegistry>) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc(instance_handle, |inst| inst.to_frontend_dto())
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn toggle_mod(
    state: State<'_, AppRegistry>,
    id: String,
    is_active: bool,
) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc_mut(instance_handle, |inst| {
            inst.toggle_mod(&id, is_active)
                .map(|_| inst.to_frontend_dto())
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn get_backups(
    state: State<'_, AppRegistry>,
    mod_id: String,
) -> Result<Vec<String>, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc(instance_handle, |inst| inst.get_backups(&mod_id))
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn restore_backup(
    state: State<'_, AppRegistry>,
    mod_id: String,
    timestamp: String,
) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc_mut(instance_handle, |inst| {
            inst.restore_backup(&mod_id, &timestamp)
                .map(|_| inst.to_frontend_dto())
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn get_mod_documentation(
    state: State<'_, AppRegistry>,
    mod_id: String,
) -> Result<String, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc(instance_handle, |inst| inst.get_mod_documentation(&mod_id))
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}
