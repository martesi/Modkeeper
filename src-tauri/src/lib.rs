mod commands;
mod config;
mod core;
mod models;
mod utils;

use crate::commands::library::sync_mods;
use crate::core::registry::AppRegistry;
use commands::library::{add_mods, remove_mods};
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};
// added import

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder =
        Builder::<tauri::Wry>::new().commands(collect_commands![add_mods, remove_mods, sync_mods]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../.config/generated/bindings.ts")
        .expect("Failed to export typescript bindings");

    // create the shared AppRegistry and manage it in the Tauri app state
    let app_registry = AppRegistry::default();

    tauri::Builder::default()
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
            Ok(())
        })
        // on an actual app, remove the string argument
        .run(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");
}
