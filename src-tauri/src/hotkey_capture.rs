use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::state::AppState;

/// While the settings recorder is active we unregister uClipboard's own shortcuts so
/// pressing them cannot toggle the window or start a screenshot. Other applications'
/// system-wide hotkeys cannot be suppressed portably; those still win.
static SUSPENDED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn begin_hotkey_capture(app: AppHandle) -> Result<(), String> {
    let (hotkey, screenshot_hotkey) = {
        let state = app.state::<AppState>();
        let settings = state
            .settings
            .lock()
            .map_err(|_| "hotkey.capture_failed".to_string())?;
        (settings.hotkey.clone(), settings.screenshot_hotkey.clone())
    };
    let manager = app.global_shortcut();
    let _ = manager.unregister(hotkey.as_str());
    let _ = manager.unregister(screenshot_hotkey.as_str());
    SUSPENDED.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn end_hotkey_capture(app: AppHandle) {
    if !SUSPENDED.swap(false, Ordering::SeqCst) {
        return;
    }
    let (hotkey, screenshot_hotkey) = {
        let state = app.state::<AppState>();
        let settings = match state.settings.lock() {
            Ok(settings) => settings.clone(),
            Err(_) => return,
        };
        (settings.hotkey.clone(), settings.screenshot_hotkey.clone())
    };
    let manager = app.global_shortcut();
    if let Err(error) = manager.register(hotkey.as_str()) {
        log::error!("restore global shortcut failed: {error}");
    }
    if let Err(error) = manager.register(screenshot_hotkey.as_str()) {
        log::error!("restore screenshot shortcut failed: {error}");
    }
}
