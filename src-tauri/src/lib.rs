pub mod commands;
pub mod config;
pub mod core;
pub mod models;
pub mod utils;

use crate::commands::global::{close_library, create_library, init, open_library, remove_library};
use crate::commands::library::{
    add_mods, get_backups, get_library, get_mod_documentation, remove_mods, rename_library,
    restore_backup, sync_mods, toggle_mod,
};
use crate::core::registry::AppRegistry;
use parking_lot::Mutex;
use specta_typescript::Typescript;
use std::sync::Arc;
use tauri_specta::{collect_commands, Builder};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

/// Stage 1: Setup command handler with all registered commands
fn setup_command_handler() -> Builder<tauri::Wry> {
    #[cfg(debug_assertions)]
    {
        use crate::commands::test::create_simulation_game_root;
        Builder::<tauri::Wry>::new().commands(collect_commands![
            // library
            add_mods,
            remove_mods,
            sync_mods,
            get_library,
            toggle_mod,
            get_backups,
            restore_backup,
            get_mod_documentation,
            rename_library,
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
    #[cfg(not(debug_assertions))]
    {
        Builder::<tauri::Wry>::new().commands(collect_commands![
            // library
            add_mods,
            remove_mods,
            sync_mods,
            get_library,
            toggle_mod,
            get_backups,
            restore_backup,
            get_mod_documentation,
            rename_library,
            // global
            open_library,
            create_library,
            close_library,
            remove_library,
            init
        ])
    }
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
    tauri::Builder::default().plugin(tauri_plugin_dialog::init())
}

/// Helper: Load the initial library from known libraries in a background thread
fn load_initial_library(
    config_handle: Arc<Mutex<crate::config::global::GlobalConfig>>,
    instance_handle: Arc<Mutex<Option<crate::core::library::Library>>>,
) {
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
        // Mount events for the command handler
        builder.mount_events(app);

        // Load the initial library in the background
        load_initial_library(config_handle, instance_handle);

        // Start timer to check if init was called within 10 seconds
        start_init_timeout_checker(init_called);

        Ok(())
    }
}

/// Set up logging to both console and file
/// Uses RUST_LOG environment variable to control log level (e.g., RUST_LOG=debug,info,warn,error)
/// Returns a guard that must be kept alive for file logging to work
fn setup_logging() -> tracing_appender::non_blocking::WorkerGuard {
    // Set up a file-based logger
    let file_appender = tracing_appender::rolling::never("logs", "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Create env filter with default level if RUST_LOG is not set
    // Include the crate name to ensure instrument spans are logged
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("mod_keeper=debug,debug"));

    // Create layers for both file and console output
    // Enable span recording to capture instrument spans (entry and exit events)
    let file_layer = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(non_blocking)
        .with_target(false)
        .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::EXIT);

    let console_layer = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::io::stdout)
        .with_target(false)
        .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::EXIT);

    // Build and initialize the subscriber with both layers
    Registry::default()
        .with(env_filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    guard
}

/// Stage 6-7: Main entry point - orchestrates all initialization stages
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set up logging to both console and file
    // Keep guard alive for the application lifetime
    let _log_guard = setup_logging();

    // Log that logging is initialized
    tracing::info!("Logging initialized - output to console and logs/app.log");

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
