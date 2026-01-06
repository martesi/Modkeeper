use crate::core::library::Library;
use crate::core::mod_stager::ModStager;
use crate::core::registry::AppRegistry;
use crate::models::error::SError;
use crate::models::library_dto::LibraryDTO;
use crate::models::task_status::TaskStatus;
use crate::utils::context::TaskContext;
use camino::Utf8PathBuf;
use log::{debug, info};
use tauri::ipc::Channel;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn add_mod(
    state: State<'_, AppRegistry>,
    paths: Vec<String>,
    channel: Channel<TaskStatus>,
) -> Result<(), SError> {
    info!("Starting task add");
    debug!("paths: {:?}", paths);

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
        let staged_mods = ModStager::resolve(&inputs, &material)?;
        debug!("staged_mods: {:?}", staged_mods);

        // 2. Lock (Synchronous)
        // Since we are now in a blocking thread, we can safely park the thread
        let mut active = instance_handle.lock();
        let inst = active.as_mut().ok_or(SError::Unexpected)?;

        info!("Adding mods to library");
        // 3. Install & Cleanup
        // Using try_for_each for early exit on error
       let r=  staged_mods.into_iter().try_for_each(|staged| {
            debug!("current: {:?}", staged);
            inst.add_mod(&staged.source_path, staged.fs.clone())
                .and_then(|_| ModStager::clean_up(&staged))
        });

        info!("Mods added");

        r
    })
        .await?
}

#[tauri::command]
#[specta::specta]
pub async fn remove_mod(state: State<'_, AppRegistry>, id: String) -> Result<(), SError> {
    let mut active = state.active_instance.lock().await;
    match active.as_mut() {
        Some(inst) => inst.remove_mod(&id),
        None => Err(SError::Unexpected),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_current_instance(
    state: State<'_, AppRegistry>,
) -> Result<Option<LibraryDTO>, String> {
    let active = state.active_instance.lock().await;
    // We can call .to_dto() because Instance implements ModManagerInstance
    Ok(active.as_ref().map(|inst| inst.to_dto()))
}

#[tauri::command]
#[specta::specta]
pub async fn switch_instance(
    state: State<'_, AppRegistry>,
    path: String,
) -> Result<LibraryDTO, SError> {
    let path_buf = camino::Utf8PathBuf::from(path);
    let new_inst = Library::load(&path_buf)?;
    let dto = new_inst.to_dto();

    // Swap the instance in the Mutex
    let mut active = state.active_instance.lock().await;
    *active = Some(new_inst);

    // Update Global Config
    let mut conf = state.global_config.lock().await;
    conf.last_opened_instance = Some(path_buf.clone());

    /*
     @hint otherwise we move its path to the top.
     this instances list is also used for frontend displaying
       so we want to avoid duplicates,
       and read their manifest for basic info.
       of course, we don't need to load their cache and other info
       since it's activate them.
    */
    if !conf.known_instances.contains(&path_buf) {
        conf.known_instances.push(path_buf.clone());
    }
    crate::config::global::save_config(conf.clone());

    Ok(dto)
}
