use crate::commands::log_err;
use crate::core::global_service;
use crate::core::registry::AppRegistry;
use crate::models::error::SError;
use crate::models::workspace::{ActivateLibraryInput, CreateLibraryInput, LibraryWorkspace};
use crate::store::app_config::AppSettings;
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
        use window_vibrancy::{NSVisualEffectMaterial, apply_vibrancy};

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

/// The startup command: returns the full workspace and, as the first frontend
/// call, carries the watchdog handoff (C11) and shows the window (prevents an
/// unstyled flash). Also delivers any pending startup config warning once (C12).
#[tauri::command]
#[specta::specta]
pub async fn get_library_workspace(
    app_handle: AppHandle,
    state: State<'_, AppRegistry>,
) -> Result<LibraryWorkspace, SError> {
    // Watchdog handoff: the startup timeout checker watches this flag (C11)
    state
        .init_called
        .store(true, std::sync::atomic::Ordering::Relaxed);

    let config_handle = state.app_config.clone();
    let instance_handle = state.active_instance.clone();
    let warning_handle = state.config_warning.clone();

    let result = log_err(
        tauri::async_runtime::spawn_blocking(move || {
            let store = config_handle.lock();
            let instance = instance_handle.lock();
            let warning = warning_handle.lock().take();
            Ok(global_service::assemble_workspace(
                &store.config,
                instance.as_ref(),
                warning,
            ))
        })
        .await
        .map_err(|e| SError::AsyncRuntimeError(e.to_string()))
        .and_then(|r| r),
    );

    app_handle.get_webview_window("main").inspect(|window| {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
    });
    result
}

#[tauri::command]
#[specta::specta]
pub async fn activate_library(
    state: State<'_, AppRegistry>,
    input: ActivateLibraryInput,
) -> Result<LibraryWorkspace, SError> {
    let config_handle = state.app_config.clone();
    let instance_handle = state.active_instance.clone();

    log_err(
        tauri::async_runtime::spawn_blocking(move || {
            global_service::activate_library(&config_handle, &instance_handle, &input.library_id)?;

            let store = config_handle.lock();
            let instance = instance_handle.lock();
            Ok(global_service::assemble_workspace(
                &store.config,
                instance.as_ref(),
                None,
            ))
        })
        .await
        .map_err(|e| SError::AsyncRuntimeError(e.to_string()))
        .and_then(|r| r),
    )
}

#[tauri::command]
#[specta::specta]
pub async fn create_library(
    state: State<'_, AppRegistry>,
    input: CreateLibraryInput,
) -> Result<LibraryWorkspace, SError> {
    let config_handle = state.app_config.clone();
    let instance_handle = state.active_instance.clone();

    log_err(
        tauri::async_runtime::spawn_blocking(move || {
            global_service::create_library(&config_handle, &instance_handle, input)?;

            let store = config_handle.lock();
            let instance = instance_handle.lock();
            Ok(global_service::assemble_workspace(
                &store.config,
                instance.as_ref(),
                None,
            ))
        })
        .await
        .map_err(|e| SError::AsyncRuntimeError(e.to_string()))
        .and_then(|r| r),
    )
}

#[tauri::command]
#[specta::specta]
pub async fn get_settings(state: State<'_, AppRegistry>) -> Result<AppSettings, SError> {
    Ok(global_service::get_settings(&state.app_config.lock()))
}

#[tauri::command]
#[specta::specta]
pub async fn save_settings(
    state: State<'_, AppRegistry>,
    settings: AppSettings,
) -> Result<AppSettings, SError> {
    let config_handle = state.app_config.clone();
    log_err(
        tauri::async_runtime::spawn_blocking(move || {
            global_service::save_settings(&mut config_handle.lock(), settings)
        })
        .await
        .map_err(|e| SError::AsyncRuntimeError(e.to_string()))
        .and_then(|r| r),
    )
}
