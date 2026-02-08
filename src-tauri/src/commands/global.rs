use crate::core::registry::AppRegistry;
use crate::core::{global_service, library_service};
use crate::models::error::SError;
use crate::models::global::LibrarySwitch;
use crate::models::library::LibraryCreationRequirement;
use camino::Utf8PathBuf;
use tauri::{AppHandle, Manager, State};

/// Detect if running on Windows 11 (build >= 22000)
#[cfg(target_os = "windows")]
fn is_windows_11() -> bool {
    use std::process::Command;
    Command::new("cmd")
        .args(["/C", "ver"])
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            // Windows 11 has build number >= 22000
            // Format is like "Microsoft Windows [Version 10.0.22631.4602]"
            output
                .split('.')
                .nth(2)
                .and_then(|s| {
                    s.chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                        .ok()
                })
                .map(|build| build >= 22000)
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// Apply window vibrancy effect based on OS and theme
/// - Windows 11: Mica effect with dark mode support
/// - Windows 10: Acrylic effect with tint color
/// - macOS: Vibrancy effect (follows system appearance)
#[tauri::command]
#[specta::specta]
pub fn apply_window_effect(app_handle: AppHandle, is_dark: Option<bool>) {
    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::{apply_acrylic, apply_mica};

        let Some(window) = app_handle.get_webview_window("main") else {
            tracing::warn!("Failed to get main window for window effect");
            return;
        };

        if is_windows_11() {
            if let Err(e) = apply_mica(&window, is_dark) {
                tracing::warn!("Failed to apply Mica effect: {}", e);
            }
        } else {
            // Windows 10: use acrylic with tint color based on theme
            let color = match is_dark {
                Some(true) | None => Some((18, 18, 18, 125)),
                Some(false) => Some((255, 255, 255, 125)),
            };
            if let Err(e) = apply_acrylic(&window, color) {
                tracing::warn!("Failed to apply Acrylic effect: {}", e);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

        let Some(window) = app_handle.get_webview_window("main") else {
            tracing::warn!("Failed to get main window for window effect");
            return;
        };

        // AppearanceBased follows system dark/light mode automatically
        if let Err(e) = apply_vibrancy(&window, NSVisualEffectMaterial::AppearanceBased, None, None)
        {
            tracing::warn!("Failed to apply vibrancy effect: {}", e);
        }
    }

    // Suppress unused variable warning on platforms where we don't use these
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = app_handle;
        let _ = is_dark;
        tracing::debug!("Window effects are not supported on this platform");
    }
}

#[tauri::command]
#[specta::specta]
pub async fn open_library(
    state: State<'_, AppRegistry>,
    path: String,
) -> Result<LibrarySwitch, SError> {
    let path_buf = Utf8PathBuf::from(path);

    // Clone BOTH handles to move them into the blocking thread
    let config_handle = state.global_config.clone();
    let instance_handle = state.active_instance.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // 1. Lock Config, Load Library (IO), Update Config
        // We scope this block so we can release the config lock before acquiring the instance lock
        // (though keeping it held is also safe here since the order is fixed).
        let (lib, switch_dto) = {
            let mut config = config_handle.lock();
            let lib = library_service::open_library(&mut config, &path_buf)?;
            let switch = library_service::to_library_switch(&config, Some(&lib));
            (lib, switch)
        };

        // 2. Lock Instance and Swap
        // IMPORTANT: This drops the *old* Library instance.
        // Doing this here ensures any heavy resource cleanup (closing files, freeing RAM)
        // happens on this blocking thread, not the async runtime.
        *instance_handle.lock() = Some(lib);

        Ok(switch_dto)
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))? // Unwraps JoinError
}

#[tauri::command]
#[specta::specta]
pub async fn create_library(
    state: State<'_, AppRegistry>,
    requirement: LibraryCreationRequirement,
) -> Result<LibrarySwitch, SError> {
    // Clone handles to move into the blocking thread
    let config_handle = state.global_config.clone();
    let instance_handle = state.active_instance.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // 1. Lock Config, Create Library on disk, Update MRU
        let (lib, switch) = {
            let mut config = config_handle.lock();
            let lib = library_service::create_library(&mut config, requirement)?;
            let switch = library_service::to_library_switch(&config, Some(&lib));
            (lib, switch)
        };

        // 2. Lock Instance and Swap
        // This overwrites the old instance, triggering its Drop (cleanup) on this worker thread.
        *instance_handle.lock() = Some(lib);

        Ok(switch)
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))? // Unwrap the JoinHandle error
}

#[tauri::command]
#[specta::specta]
pub async fn init(
    app_handle: AppHandle,
    state: State<'_, AppRegistry>,
) -> Result<LibrarySwitch, SError> {
    // Mark that init has been called
    state
        .init_called
        .store(true, std::sync::atomic::Ordering::Relaxed);

    // Get current state (library already loaded in background thread)
    let config_handle = state.global_config.clone();
    let instance_handle = state.active_instance.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let config = config_handle.lock();
        let instance_guard = instance_handle.lock();
        let active_library = instance_guard.as_ref();
        Ok(library_service::to_library_switch(&config, active_library))
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?;

    app_handle.get_webview_window("main").inspect(|window| {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
    });
    result
}

#[tauri::command]
#[specta::specta]
pub async fn close_library(
    state: State<'_, AppRegistry>,
    repo_root: String,
) -> Result<LibrarySwitch, SError> {
    let path_buf = Utf8PathBuf::from(repo_root);
    let config_handle = state.global_config.clone();
    let instance_handle = state.active_instance.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // Check if this is the active library
        let is_active = {
            let instance_guard = instance_handle.lock();
            instance_guard
                .as_ref()
                .map(|lib| lib.repo_root == path_buf)
                .unwrap_or(false)
        };

        // Close library via service
        let mut config = config_handle.lock();
        global_service::close_library(&mut config, &path_buf)?;
        drop(config);

        // If closing active library, clear the instance
        if is_active {
            *instance_handle.lock() = None;
        }

        // Return updated switch
        let config = config_handle.lock();
        let instance_guard = instance_handle.lock();
        let active_lib = instance_guard.as_ref();
        Ok(library_service::to_library_switch(&config, active_lib))
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}

#[tauri::command]
#[specta::specta]
pub async fn remove_library(
    state: State<'_, AppRegistry>,
    repo_root: String,
) -> Result<LibrarySwitch, SError> {
    // Check if game/server is running before proceeding
    if state.is_game_or_server_running() {
        return Err(SError::GameOrServerRunning);
    }

    let path_buf = Utf8PathBuf::from(repo_root);
    let config_handle = state.global_config.clone();
    let instance_handle = state.active_instance.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // Check if this is the active library
        let is_active = {
            let instance_guard = instance_handle.lock();
            instance_guard
                .as_ref()
                .map(|lib| lib.repo_root == path_buf)
                .unwrap_or(false)
        };

        // Remove library via service (unlinks mods, removes from config, deletes directory)
        let mut config = config_handle.lock();
        global_service::remove_library(&mut config, &path_buf)?;
        drop(config);

        // If removing active library, clear the instance
        if is_active {
            *instance_handle.lock() = None;
        }

        // Return updated switch
        let config = config_handle.lock();
        let instance_guard = instance_handle.lock();
        let active_lib = instance_guard.as_ref();
        Ok(library_service::to_library_switch(&config, active_lib))
    })
    .await
    .map_err(|e| SError::AsyncRuntimeError(e.to_string()))?
}
