mod commands;
mod config;

use commands::{get_app_settings,get_repo_def};
use specta_typescript::Typescript;
use tauri::{DragDropEvent, Window, WindowEvent};
use tauri_specta::{collect_commands, Builder};

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
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![greet, get_app_settings,get_repo_def]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../.config/generated/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .on_window_event(handle_drop_event)
        // and finally tell Tauri how to invoke them
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            // This is also required if you want to use events
            builder.mount_events(app);
            Ok(())
        })
        // on an actual app, remove the string argument
        .run(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");
}
