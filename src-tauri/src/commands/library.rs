use crate::core::registry::AppRegistry;
use crate::core::{
    cleanup, deployment, dto_builder, library_service, mod_backup, mod_documentation, mod_manager,
    mod_stager,
};
use crate::models::error::SError;
use crate::models::global::LibrarySwitch;
use crate::models::library::LibraryDTO;
use crate::models::task_status::TaskStatus;
use crate::utils::context::TaskContext;
use crate::utils::thread::{with_lib_arc, with_lib_arc_mut};
use camino::Utf8PathBuf;
use tauri::ipc::Channel;
use tauri::State;
use tracing::{debug, info};

#[tauri::command]
#[specta::specta]
pub async fn add_mods(
    state: State<'_, AppRegistry>,
    paths: Vec<String>,
    unknown_mod_name: String,
    channel: Channel<TaskStatus>,
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
                    // Extract cleanup data before moving staged into add_mod
                    let is_staging = staged.is_staging;
                    let source_path = staged.source_path.clone();
                    mod_manager::add_mod(inst, staged)
                        .and_then(|_| mod_stager::clean_up(is_staging, &source_path))
                })
                .map(|_| dto_builder::build_frontend_dto(inst))
        })
    })
    .await??
}

#[tauri::command]
#[specta::specta]
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
                    mod_manager::remove_mod(inst, mod_id)
                })
                .map(|_| dto_builder::build_frontend_dto(inst))
        })
        // Iterate and remove each mod, exiting immediately on the first error
    })
    .await??
}

#[tauri::command]
#[specta::specta]
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
            // 1. Purge existing managed links
            cleanup::purge(
                &inst.game_root,
                &inst.repo_root,
                &inst.spt_rules,
                &inst.lib_paths,
                &inst.cache,
            )?;

            // 2. Deploy active mods
            deployment::deploy(
                &inst.game_root,
                &inst.lib_paths,
                &inst.spt_rules,
                &inst.mods,
                &inst.cache,
            )?;

            inst.mark_clean();
            inst.persist()?;
            Ok(dto_builder::build_frontend_dto(inst))
        })
    })
    .await??
}

#[tauri::command]
#[specta::specta]
pub async fn get_library(state: State<'_, AppRegistry>) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc(instance_handle, |inst| {
            dto_builder::build_frontend_dto(inst)
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
                .map(|_| dto_builder::build_frontend_dto(inst))
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
) -> Result<Vec<String>, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc(instance_handle, |inst| {
            mod_backup::list_backups(&inst.lib_paths, &mod_id)
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}

#[tauri::command]
#[specta::specta]
pub async fn restore_backup(
    state: State<'_, AppRegistry>,
    mod_id: String,
    timestamp: String,
) -> Result<LibraryDTO, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc_mut(instance_handle, |inst| {
            mod_backup::restore_backup(inst, &mod_id, &timestamp)
                .map(|_| dto_builder::build_frontend_dto(inst))
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}

#[tauri::command]
#[specta::specta]
pub async fn get_mod_documentation(
    state: State<'_, AppRegistry>,
    mod_id: String,
) -> Result<String, SError> {
    let instance_handle = state.active_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_lib_arc(instance_handle, |inst| {
            mod_documentation::read_documentation(inst, &mod_id)
        })
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))??
}

#[tauri::command]
#[specta::specta]
pub async fn rename_library(
    state: State<'_, AppRegistry>,
    name: String,
) -> Result<LibrarySwitch, SError> {
    let config_handle = state.global_config.clone();
    let instance_handle = state.active_instance.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // Update library name via service
        with_lib_arc_mut(instance_handle.clone(), |inst| {
            library_service::rename_library(inst, name)
        })??;

        // Return updated switch
        let config = config_handle.lock();
        let instance_guard = instance_handle.lock();
        let active_lib = instance_guard.as_ref();
        Ok(library_service::to_library_switch(&config, active_lib))
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}
