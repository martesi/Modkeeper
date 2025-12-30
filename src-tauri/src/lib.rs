mod commands;
mod config;
mod core;
mod models;
mod utils;

use crate::commands::instance::{get_current_instance, switch_instance};
use commands::instance::{add_mod, remove_mod};
use specta_typescript::Typescript;
use tauri::{DragDropEvent, Window, WindowEvent};
use tauri_specta::{collect_commands, Builder};
use crate::core::registry::AppRegistry; // added import

#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn handle_drop_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event {
        println!("File dropped on window: {}", window.label());
        for path in paths {
            println!(
                "Path: {:?}, name: {:?}, extension: {:?}",
                path,
                path.file_name(),
                path.extension(),
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        greet,
        add_mod,
        remove_mod,
        get_current_instance,
        switch_instance,
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../.config/generated/bindings.ts")
        .expect("Failed to export typescript bindings");

    // create the shared AppRegistry and manage it in the Tauri app state
    let app_registry = AppRegistry::new();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .on_window_event(handle_drop_event)
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
