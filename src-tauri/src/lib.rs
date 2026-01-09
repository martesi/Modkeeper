pub mod commands;
pub mod config;
pub mod core;
pub mod models;
pub mod utils;

use crate::commands::global::{create_library, init, open_library};
use crate::commands::library::{
    add_mods, get_backups, get_library, get_mod_documentation, remove_mods, restore_backup,
    sync_mods, toggle_mod,
};
use crate::core::registry::AppRegistry;
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};
// added import

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        // library
        add_mods,
        remove_mods,
        sync_mods,
        get_library,
        toggle_mod,
        get_backups,
        restore_backup,
        get_mod_documentation,
        // global
        open_library,
        create_library,
        init
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/gen/bindings.ts")
        .expect("Failed to export typescript bindings");

    // create the shared AppRegistry and manage it in the Tauri app state
    let app_registry = AppRegistry::default();

    // Clone handles for background thread before moving into setup
    let config_handle = app_registry.global_config.clone();
    let instance_handle = app_registry.active_instance.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        // and finally tell Tauri how to invoke them
        .invoke_handler(builder.invoke_handler())
        .manage(app_registry) // register the shared AppRegistry state
        .setup(move |app| {
            // This is also required if you want to use events
            builder.mount_events(app);

            // Spawn background thread to load the first library from known_libraries
            let config_handle = config_handle.clone();
            let instance_handle = instance_handle.clone();

            tauri::async_runtime::spawn_blocking(move || {
                let first_library_path = config_handle.lock().known_libraries.first().cloned();

                if let Some(path) = first_library_path {
                    match crate::core::library::Library::load(&path) {
                        Ok(library) => {
                            *instance_handle.lock() = Some(library);
                        }
                        Err(e) => {
                            tracing::error!("Failed to load library from {}: {}", path, e);
                            // Leave active_instance as None on failure
                        }
                    }
                }
            });

            Ok(())
        })
        // on an actual app, remove the string argument
        .run(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");
}
