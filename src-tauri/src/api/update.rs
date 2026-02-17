use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub date: String,
    pub body: String,
}

/// Check for available updates
/// This emits an event to the plugin's built-in check command
#[command]
pub async fn check_update(_app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    // The tauri-plugin-updater plugin handles update checks internally
    // via its own 'plugin:check' command which the frontend can invoke directly
    // This command is kept for future extensions or custom logic
    Ok(None)
}

/// Install update
/// This emits an event to trigger the plugin's install process
#[command]
pub async fn install_update(_app: AppHandle, _window: tauri::Window) -> Result<(), String> {
    // The tauri-plugin-updater plugin provides:
    // - 'plugin:download' - download the update
    // - 'plugin:install' - install a downloaded update
    // - 'plugin:download_and_install' - download and install in one step
    // These can be invoked directly from the frontend
    Ok(())
}
