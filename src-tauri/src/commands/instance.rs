use tauri::State;
use crate::core::instance::Instance;
use crate::core::mod_manager::ModManagerInstance;
use crate::core::registry::AppRegistry;
use crate::models::instance_dto::ModManagerInstanceDTO;

#[tauri::command]
#[specta::specta]
pub async fn add_mod(state: State<'_, AppRegistry>, paths: Vec<String>) -> Result<String, String> {
    let mut active = state.active_instance.lock().await;
    match active.as_mut() {
        Some(inst) => inst.add_mod(paths.into_iter().map(|p| p.into()).collect()),
        None => Err("No active instance selected".into()),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn remove_mod(state: State<'_, AppRegistry>, id: String) -> Result<(), String> {
    let mut active = state.active_instance.lock().await;
    match active.as_mut() {
        Some(inst) => inst.remove_mod(&id),
        None => Err("No active instance selected".into()),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_current_instance(state: State<'_, AppRegistry>) -> Result<Option<ModManagerInstanceDTO>, String> {
    let active = state.active_instance.lock().await;
    // We can call .to_dto() because Instance implements ModManagerInstance
    Ok(active.as_ref().map(|inst| inst.to_dto()))
}

#[tauri::command]
#[specta::specta]
pub async fn switch_instance(state: State<'_, AppRegistry>, path: String) -> Result<ModManagerInstanceDTO, String> {
    let path_buf = camino::Utf8PathBuf::from(path);
    let new_inst = Instance::load(&path_buf)?;
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