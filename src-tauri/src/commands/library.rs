use crate::core::global_service;
use crate::core::library::Library;
use crate::core::registry::AppRegistry;
use crate::core::{library_service, mod_backup, mod_manager};
use crate::models::error::SError;
use crate::models::global::LibrarySwitch;
use crate::models::library::LibraryDTO;
use crate::models::mod_backup::ModBackup;
use crate::utils::thread::{with_lib_arc, with_lib_arc_mut};
use camino::Utf8PathBuf;
use tauri::{AppHandle, State};
use tracing::debug;

#[tauri::command]
#[specta::specta]
pub async fn add_mods(
    state: State<'_, AppRegistry>,
    paths: Vec<String>,
    unknown_mod_name: String,
    backup_name: String,
) -> Result<LibraryDTO, SError> {
    let inputs = paths
        .into_iter()
        .map(Utf8PathBuf::from)
        .collect::<Vec<Utf8PathBuf>>();

    let material = state.get_stage_material(unknown_mod_name.clone())?;
    debug!("staging_material: {:?}", material);

    // Clone the Arc handle so we can move it into the 'static blocking thread.
    // 'state' cannot be moved, but the Arc inside it can be cloned.
    let instance_handle = state.active_instance.clone();

    tauri::async_runtime::spawn_blocking(move || {
        library_service::install_mods(instance_handle, &inputs, &material, &backup_name)
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub async fn remove_mods(
    state: State<'_, AppRegistry>,
    ids: Vec<String>,
) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    // Offload synchronous file IO and locking to a blocking thread
    tauri::async_runtime::spawn_blocking(move || {
        library_service::remove_mods(instance_handle, &ids)
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub async fn sync_mods(state: State<'_, AppRegistry>) -> Result<LibraryDTO, SError> {
    if state.is_game_or_server_running() {
        return Err(SError::GameOrServerRunning.into());
    }

    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc_mut(instance_handle, |inst| {
            inst.sync().map(|_| inst.to_dto())
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}

#[tauri::command]
#[specta::specta]
pub async fn get_library(state: State<'_, AppRegistry>) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc(instance_handle, |inst| {
            inst.to_dto()
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_mod(
    state: State<'_, AppRegistry>,
    id: String,
    is_active: bool,
) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc_mut(instance_handle, |inst| {
            mod_manager::toggle_mod(inst, &id, is_active)
                .map(|_| inst.to_dto())
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}

#[tauri::command]
#[specta::specta]
pub async fn get_backups(
    state: State<'_, AppRegistry>,
    mod_id: String,
) -> Result<Vec<ModBackup>, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let lib_paths = instance_handle
            .lock()
            .as_ref()
            .ok_or(SError::NoActiveLibrary)?
            .lib_paths
            .clone();
        mod_backup::list_backups(&lib_paths, &mod_id)
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub async fn restore_backup(
    state: State<'_, AppRegistry>,
    mod_id: String,
    timestamp: String,
    restore_config: bool,
) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc_mut(instance_handle, |inst| {
            mod_backup::restore_backup(inst, &mod_id, &timestamp, restore_config)
                .map(|_| inst.to_dto())
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}

#[tauri::command]
#[specta::specta]
pub async fn create_backup(
    state: State<'_, AppRegistry>,
    mod_id: String,
    backup_name: String,
) -> Result<(), SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let inst_guard = instance_handle.lock();
        let inst = inst_guard.as_ref().ok_or(SError::NoActiveLibrary)?;
        mod_backup::create_backup(inst, &mod_id, &backup_name)
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub async fn remove_backup(
    state: State<'_, AppRegistry>,
    mod_id: String,
    timestamp: String,
) -> Result<(), SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), SError> {
        let lib_paths = {
            let lock = instance_handle.lock();
            let inst = lock.as_ref().ok_or(SError::NoActiveLibrary)?;
            inst.lib_paths.clone()
        };
        mod_backup::remove_backup(&lib_paths, &mod_id, &timestamp)
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub async fn rename_library(
    state: State<'_, AppRegistry>,
    name: String,
    library_id: Option<String>,
) -> Result<LibrarySwitch, SError> {
    let config_handle = state.global_config.clone();
    let instance_handle = state.active_instance.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let target_path = global_service::resolve_target_library_path(
            library_id,
            &config_handle,
            &instance_handle,
        )?;

        match target_path {
            Some(path) => {
                let mut library = Library::load(&path)?;
                library_service::rename_library(&mut library, name)?;
                library.persist()?;
            }
            _ => {
                with_lib_arc_mut(instance_handle.clone(), |inst| {
                    library_service::rename_library(inst, name)
                })??;
            }
        }

        Ok(library_service::to_library_switch(
            &config_handle.lock(),
            instance_handle.lock().as_ref(),
        ))
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub async fn rebuild_library_cache(
    state: State<'_, AppRegistry>,
    library_id: Option<String>,
) -> Result<LibrarySwitch, SError> {
    let config_handle = state.global_config.clone();
    let instance_handle = state.active_instance.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let target_path = global_service::resolve_target_library_path(
            library_id,
            &config_handle,
            &instance_handle,
        )?;

        match target_path {
            Some(path) => {
                let mut library = Library::load(&path)?;
                library_service::rebuild_library_cache(&mut library)?;
                library.persist()?;
            }
            _ => {
                with_lib_arc_mut(instance_handle.clone(), |inst| {
                    library_service::rebuild_library_cache(inst)
                })??;
            }
        }

        Ok(library_service::to_library_switch(
            &config_handle.lock(),
            instance_handle.lock().as_ref(),
        ))
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub async fn reveal_mod(
    app: AppHandle,
    state: State<'_, AppRegistry>,
    mod_id: String,
) -> Result<(), SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc(instance_handle, |inst| {
            library_service::reveal_mod(&app, inst, &mod_id)
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}
