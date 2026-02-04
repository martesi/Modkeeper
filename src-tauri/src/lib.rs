pub mod commands;
pub mod config;
pub mod core;
pub mod models;
pub mod utils;

use crate::commands::global::{close_library, create_library, init, open_library, remove_library};
use crate::commands::library::{
    add_mods, create_backup, get_backups, get_library, get_mod_documentation,
    rebuild_library_cache, remove_backup, remove_mods, rename_library, restore_backup, reveal_mod,
    sync_mods, toggle_mod,
};
use crate::core::registry::AppRegistry;
use parking_lot::Mutex;
use specta_typescript::Typescript;
use std::sync::Arc;
use tauri_specta::{collect_commands, Builder};

/// Stage 1: Setup command handler with all registered commands
fn setup_command_handler() -> Builder<tauri::Wry> {
    use crate::commands::test::create_simulation_game_root;
    Builder::<tauri::Wry>::new().commands(collect_commands![
        // library
        add_mods,
        remove_mods,
        sync_mods,
        get_library,
        toggle_mod,
        get_backups,
        create_backup,
        restore_backup,
        remove_backup,
        get_mod_documentation,
        rename_library,
        rebuild_library_cache,
        reveal_mod,
        // global
        open_library,
        create_library,
        close_library,
        remove_library,
        init,
        // test (debug only)
        create_simulation_game_root,
    ])
}

/// Stage 2: Export TypeScript bindings (debug builds only)
fn export_typescript_bindings(builder: &Builder<tauri::Wry>) {
    #[cfg(debug_assertions)]
    {
        builder
            .export(Typescript::default(), "../src/gen/bindings.ts")
            .expect("Failed to export typescript bindings");
    }
}

/// Stage 3: Initialize application state (AppRegistry and handles)
fn initialize_app_state() -> (
    AppRegistry,
    Arc<Mutex<crate::config::global::GlobalConfig>>,
    Arc<Mutex<Option<crate::core::library::Library>>>,
) {
    let app_registry = AppRegistry::default();
    let config_handle = app_registry.global_config.clone();
    let instance_handle = app_registry.active_instance.clone();

    (app_registry, config_handle, instance_handle)
}

/// Stage 4: Register Tauri plugins
fn register_plugins() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
}

/// Helper: Load the initial library from library_last in a background thread
fn load_initial_library(
    config_handle: Arc<Mutex<crate::config::global::GlobalConfig>>,
    instance_handle: Arc<Mutex<Option<crate::core::library::Library>>>,
) {
    tauri::async_runtime::spawn_blocking(move || {
        let last_library_path = config_handle.lock().library_last.clone();

        if let Some(path) = last_library_path {
            match crate::core::library::Library::load(&path) {
                Ok(library) => {
                    *instance_handle.lock() = Some(library);
                }
                Err(e) => {
                    tracing::error!("Failed to load library from {}: {}", path, e);
                    // Clear library_last on failure to prevent repeated failures
                    let mut config = config_handle.lock();
                    config.library_last = None;
                    config.save();
                }
            }
        }
    });
}

/// Helper: Start a timer that checks if init command was called within 10 seconds
/// If init is not called, the application will exit with an error
fn start_init_timeout_checker(init_called: Arc<std::sync::atomic::AtomicBool>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        if !init_called.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::error!(
                "init command was not called within 10 seconds of setup. Application will exit."
            );
            std::process::exit(1);
        }
    });
}

/// Stage 5: Setup application (mount events and load initial library)
fn setup_application(
    builder: Builder<tauri::Wry>,
    config_handle: Arc<Mutex<crate::config::global::GlobalConfig>>,
    instance_handle: Arc<Mutex<Option<crate::core::library::Library>>>,
    init_called: Arc<std::sync::atomic::AtomicBool>,
) -> impl FnOnce(&mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    move |app| {
        // Apply window effects
        #[cfg(windows)]
        {
            use tauri::Manager;
            use window_vibrancy::apply_mica;
            let window = app.get_webview_window("main").unwrap();
            apply_mica(&window, None).unwrap();
        }

        // Mount events for the command handler
        builder.mount_events(app);

        // Load the initial library in the background
        load_initial_library(config_handle, instance_handle);

        // Start timer to check if init was called within 10 seconds
        start_init_timeout_checker(init_called);

        Ok(())
    }
}

/// Stage 6-7: Main entry point - orchestrates all initialization stages
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Stage 1: Setup command handler
    let builder = setup_command_handler();

    // Stage 2: Export TypeScript bindings (debug only)
    export_typescript_bindings(&builder);

    // Stage 3: Initialize application state
    let (app_registry, config_handle, instance_handle) = initialize_app_state();
    let init_called = app_registry.init_called.clone();

    // Stage 4: Register plugins
    let tauri_builder = register_plugins();

    // Stage 5: Get invoke handler before moving builder into setup
    let invoke_handler = builder.invoke_handler();

    // Stage 6: Configure application setup
    let setup_fn = setup_application(builder, config_handle, instance_handle, init_called);

    // Stage 7: Build and run the application
    tauri_builder
        .invoke_handler(invoke_handler)
        .manage(app_registry)
        .setup(setup_fn)
        .run(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");
}
