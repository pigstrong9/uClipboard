use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

#[cfg(not(target_os = "windows"))]
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use tauri::{
    Emitter, Manager, PhysicalPosition, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::clipboard::hash::{hash_bytes, hash_files};
use crate::clipboard::source::{ClipboardSource, SystemClipboard};
use crate::media;
use crate::models::{ClipItem, ClipKind, ClipSummary};
use crate::settings::{self, Settings as AppSettings};
use crate::state::{AppState, SharedState};

fn with_thumb_path(mut item: ClipItem, shared: &SharedState) -> ClipItem {
    if item.kind == ClipKind::Image {
        item.thumb_path = media::thumb_path(&shared.media_dir, &item.content);
    }
    item
}

/// Hash of the bytes that will end up on the clipboard, used to distinguish our own
/// write from a real copy made right afterwards.
fn clipboard_hash(item: &ClipItem, media_dir: &std::path::Path) -> Option<String> {
    match item.kind {
        ClipKind::Text => Some(hash_bytes(item.content.as_bytes())),
        ClipKind::Files => serde_json::from_str::<Vec<String>>(&item.content)
            .ok()
            .map(|files| {
                files
                    .into_iter()
                    .filter(|path| !path.is_empty() && std::path::Path::new(path).exists())
                    .collect::<Vec<_>>()
            })
            .filter(|files| !files.is_empty())
            .map(|files| hash_files(&files)),
        ClipKind::Image => std::fs::read(media_dir.join(&item.content))
            .ok()
            .map(|bytes| hash_bytes(&bytes)),
    }
}

fn cleanup_media(shared: &SharedState) -> Result<(), String> {
    let live = {
        let store = shared.store.lock().map_err(|e| e.to_string())?;
        store.image_paths().map_err(|e| e.to_string())?
    };
    media::cleanup_orphans(&shared.media_dir, &live);
    Ok(())
}

/// Applies new history/retention limits immediately instead of waiting for the next
/// capture or the six-hour maintenance tick.
fn prune_history(shared: &SharedState, settings: &AppSettings) {
    let removed = {
        let Ok(store) = shared.store.lock() else {
            return;
        };
        match store.prune(settings.history_limit, settings.retention_days) {
            Ok(images) => images,
            Err(error) => {
                log::error!("settings prune failed: {error}");
                return;
            }
        }
    };
    media::remove_media_files(&shared.media_dir, &removed);
    if let Err(error) = cleanup_media(shared) {
        log::warn!("media cleanup skipped after settings change: {error}");
    }
}

fn arm_suppression(
    shared: &SharedState,
    hash: Option<String>,
    allow_mismatch: bool,
) -> Result<(), String> {
    shared.suppress.lock().map_err(|e| e.to_string())?.arm(
        Duration::from_millis(1500),
        hash,
        allow_mismatch,
    );
    Ok(())
}

fn write_item_to_clipboard(shared: &SharedState, id: &str) -> Result<ClipItem, String> {
    let item = shared
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .get(id)
        .map_err(|e| e.to_string())?;
    let plain_text = shared
        .settings
        .lock()
        .map(|settings| settings.paste_plain_text)
        .unwrap_or(false);

    let source = SystemClipboard;
    let hash = clipboard_hash(&item, &shared.media_dir);
    let allow_mismatch = item.kind == ClipKind::Image;
    if !source.write_with_options(&item, &shared.media_dir, plain_text) {
        return Err("clipboard.write_failed".to_string());
    }
    arm_suppression(shared, hash, allow_mismatch)?;

    let updated = {
        let store = shared.store.lock().map_err(|e| e.to_string())?;
        store.touch(id).map_err(|e| e.to_string())?;
        store.get(id).map_err(|e| e.to_string())?
    };
    Ok(with_thumb_path(updated, shared))
}

#[cfg(target_os = "windows")]
fn send_paste_shortcut(_enigo: &mut ()) -> Result<(), String> {
    if crate::platform::send_paste_keys() {
        Ok(())
    } else {
        Err("paste.send_failed".to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn send_paste_shortcut(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // kVK_ANSI_V. `Key::Unicode('v')` resolves through the Text Input Source layout
        // tables, which can block off the main thread and yields keycode 0 with some
        // input sources, so Cmd+V never actually leaves as V.
        const KEYCODE_V: u32 = 9;
        enigo.key(Key::Meta, Press).map_err(|e| e.to_string())?;
        std::thread::sleep(Duration::from_millis(15));
        let clicked = enigo
            .key(Key::Other(KEYCODE_V), Click)
            .map_err(|e| e.to_string());
        std::thread::sleep(Duration::from_millis(15));
        // Release Command even if the V press failed, otherwise the modifier sticks.
        enigo.key(Key::Meta, Release).map_err(|e| e.to_string())?;
        clicked?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Press).map_err(|e| e.to_string())?;
        std::thread::sleep(Duration::from_millis(15));
        enigo
            .key(Key::Unicode('v'), Click)
            .map_err(|e| e.to_string())?;
        std::thread::sleep(Duration::from_millis(15));
        enigo
            .key(Key::Control, Release)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// macOS needs the whole application deactivated, not just its window hidden, before a
/// synthetic Cmd+V will reach the previous foreground app.
#[cfg(target_os = "macos")]
fn hide_before_paste(app: &tauri::AppHandle, window: &WebviewWindow) {
    let _ = window.hide();
    // AppHandle has no app-level hide in Tauri 2; dispatch NSApplication.hide to the
    // main thread instead.
    let handle = app.clone();
    let _ = handle.run_on_main_thread(crate::platform::hide_app);
}

#[cfg(not(target_os = "macos"))]
fn hide_before_paste(_app: &tauri::AppHandle, window: &WebviewWindow) {
    let _ = window.hide();
}

fn transfer_items_inner(shared: &SharedState, ids: &[String]) -> Result<Vec<ClipItem>, String> {
    let items = {
        let store = shared.store.lock().map_err(|e| e.to_string())?;
        ids.iter()
            .filter_map(|id| store.get(id).ok())
            .collect::<Vec<_>>()
    };
    Ok(items
        .into_iter()
        .map(|item| with_thumb_path(item, shared))
        .collect())
}

fn position_transfer_window(
    app: &tauri::AppHandle,
    transfer: &WebviewWindow,
) -> Result<(), String> {
    let Some(main) = app.get_webview_window("main") else {
        return Ok(());
    };
    let main_pos = main.outer_position().map_err(|e| e.to_string())?;
    let main_size = main.outer_size().map_err(|e| e.to_string())?;
    let transfer_size = transfer.outer_size().map_err(|e| e.to_string())?;

    let mut x = main_pos.x + main_size.width as i32 + 8;
    let mut y = main_pos.y + main_size.height as i32 - transfer_size.height as i32;

    if let Ok(Some(monitor)) = main.current_monitor() {
        let monitor_pos = monitor.position();
        let monitor_size = monitor.size();
        if x + transfer_size.width as i32 > monitor_pos.x + monitor_size.width as i32 {
            x = main_pos.x - transfer_size.width as i32 - 8;
        }
        if x < monitor_pos.x {
            x = monitor_pos.x;
        }
        if y + transfer_size.height as i32 > monitor_pos.y + monitor_size.height as i32 {
            y = monitor_pos.y + monitor_size.height as i32 - transfer_size.height as i32;
        }
        if y < monitor_pos.y {
            y = monitor_pos.y;
        }
    }

    transfer
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())
}

fn ensure_transfer_window(app: &tauri::AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("transfer") {
        position_transfer_window(app, &window)?;
        return Ok(window);
    }

    let window =
        WebviewWindowBuilder::new(app, "transfer", WebviewUrl::App("transfer.html".into()))
            .title("临时中转")
            .inner_size(300.0, 380.0)
            .min_inner_size(260.0, 260.0)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .transparent(true)
            .visible(false)
            .focused(false)
            .build()
            .map_err(|e| {
                eprintln!("build transfer failed: {e}");
                e.to_string()
            })?;

    position_transfer_window(app, &window)?;
    Ok(window)
}

#[tauri::command]
pub async fn list_items(
    state: State<'_, AppState>,
    tab: String,
    query: String,
) -> Result<Vec<ClipSummary>, String> {
    let shared = state.shared();
    // Favorites and pinned entries are exempt from pruning, so they must not be hidden
    // by the history limit either.
    let limit = if tab == "favorite" {
        -1
    } else {
        shared
            .settings
            .lock()
            .map(|settings| {
                if settings.history_limit == 0 {
                    -1
                } else {
                    settings.history_limit as i64
                }
            })
            .unwrap_or(500)
    };
    // The query hits SQLite and the thumbnail lookup touches the filesystem; keep both
    // off the webview's command thread so a large history cannot stall the UI.
    tauri::async_runtime::spawn_blocking(move || {
        shared
            .store
            .lock()
            .map_err(|e| e.to_string())?
            .list_summaries(&tab, &query, limit, &shared.media_dir)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_item(state: State<'_, AppState>, id: String) -> Result<ClipItem, String> {
    let shared = state.shared();
    tauri::async_runtime::spawn_blocking(move || {
        let item = shared
            .store
            .lock()
            .map_err(|e| e.to_string())?
            .get(&id)
            .map_err(|e| e.to_string())?;
        Ok(with_thumb_path(item, &shared))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn toggle_favorite(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ClipItem, String> {
    let shared = state.shared();
    let item = shared
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .toggle_favorite(&id)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("transfer-changed", ());
    Ok(with_thumb_path(item, &shared))
}

#[tauri::command]
pub fn toggle_pin(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ClipItem, String> {
    let shared = state.shared();
    let item = shared
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .toggle_pin(&id)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("transfer-changed", ());
    Ok(with_thumb_path(item, &shared))
}

#[tauri::command]
pub fn delete_item(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let shared = state.shared();
    shared
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .delete(&id)
        .map_err(|e| e.to_string())?;
    if let Err(error) = cleanup_media(&shared) {
        eprintln!("media cleanup skipped after delete: {error}");
    }
    let _ = app.emit("transfer-changed", ());
    Ok(())
}

pub fn clear_history(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(window) = app.get_webview_window("transfer") {
        let _ = window.hide();
    }
    state
        .transfer_ids
        .lock()
        .map_err(|e| e.to_string())?
        .clear();
    state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .clear()
        .map_err(|e| e.to_string())?;
    if let Err(error) = cleanup_media(&state.shared()) {
        eprintln!("media cleanup skipped after clear: {error}");
    }
    let _ = app.emit("clipboard-changed", ());
    let _ = app.emit("transfer-changed", ());
    Ok(())
}

#[tauri::command]
pub fn clear_items(app: tauri::AppHandle) -> Result<(), String> {
    clear_history(&app)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    state
        .inner()
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let app_state = state.inner();
    let normalized = settings.normalized();
    let previous = app_state
        .settings
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    if normalized.hotkeys_conflict() {
        return Err("settings.hotkey_conflict".to_string());
    }

    let autostart_changed = previous.auto_start != normalized.auto_start;
    if autostart_changed {
        apply_autostart(&app, normalized.auto_start).map_err(|error| {
            log::error!("apply autostart failed: {error}");
            "settings.autostart_failed".to_string()
        })?;
    }

    let hotkey_changed = previous.hotkey != normalized.hotkey;
    let screenshot_hotkey_changed = previous.screenshot_hotkey != normalized.screenshot_hotkey;
    if hotkey_changed || screenshot_hotkey_changed {
        let manager = app.global_shortcut();
        if hotkey_changed {
            let _ = manager.unregister(previous.hotkey.as_str());
        }
        if screenshot_hotkey_changed {
            let _ = manager.unregister(previous.screenshot_hotkey.as_str());
        }

        let mut failure = None;
        let mut screenshot_failed = false;
        if hotkey_changed {
            if let Err(error) = manager.register(normalized.hotkey.as_str()) {
                failure = Some(error.to_string());
            }
        }
        if failure.is_none() && screenshot_hotkey_changed {
            if let Err(error) = manager.register(normalized.screenshot_hotkey.as_str()) {
                failure = Some(error.to_string());
                screenshot_failed = true;
            }
        }

        if let Some(error) = failure {
            if hotkey_changed {
                let _ = manager.unregister(normalized.hotkey.as_str());
            }
            if screenshot_hotkey_changed {
                let _ = manager.unregister(normalized.screenshot_hotkey.as_str());
            }
            let _ = manager.register(previous.hotkey.as_str());
            let _ = manager.register(previous.screenshot_hotkey.as_str());
            if autostart_changed {
                let _ = apply_autostart(&app, previous.auto_start);
            }
            log::error!("register hotkey failed: {error}");
            return Err(if screenshot_failed {
                "screenshot.hotkey_failed"
            } else {
                "settings.hotkey_failed"
            }
            .to_string());
        }
    }

    if let Err(error) = settings::save(&app_state.config_path, &normalized) {
        if hotkey_changed || screenshot_hotkey_changed {
            let manager = app.global_shortcut();
            if hotkey_changed {
                let _ = manager.unregister(normalized.hotkey.as_str());
            }
            if screenshot_hotkey_changed {
                let _ = manager.unregister(normalized.screenshot_hotkey.as_str());
            }
            let _ = manager.register(previous.hotkey.as_str());
            let _ = manager.register(previous.screenshot_hotkey.as_str());
        }
        if previous.auto_start != normalized.auto_start {
            let _ = apply_autostart(&app, previous.auto_start);
        }
        log::error!("save settings failed: {error}");
        return Err("settings.save_failed".to_string());
    }
    *app_state.settings.lock().map_err(|e| e.to_string())? = normalized.clone();

    if previous.history_limit != normalized.history_limit
        || previous.retention_days != normalized.retention_days
    {
        let shared = app_state.shared();
        let snapshot = normalized.clone();
        if let Err(error) =
            tauri::async_runtime::spawn_blocking(move || prune_history(&shared, &snapshot)).await
        {
            log::error!("settings prune task failed: {error}");
        }
        let _ = app.emit("clipboard-changed", ());
    }

    if previous.language != normalized.language {
        let _ = crate::tray::refresh(&app);
    }
    let _ = app.emit("settings-changed", ());
    Ok(normalized)
}

#[tauri::command]
pub fn toggle_monitoring(app: tauri::AppHandle) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let (updated, previous_paused) = {
        let mut guard = state.settings.lock().map_err(|e| e.to_string())?;
        let previous_paused = guard.monitor_paused;
        guard.monitor_paused = !guard.monitor_paused;
        (guard.clone(), previous_paused)
    };
    if settings::save(&state.config_path, &updated).is_err() {
        if let Ok(mut guard) = state.settings.lock() {
            guard.monitor_paused = previous_paused;
        }
        return Err("settings.save_failed".to_string());
    }
    let _ = app.emit("settings-changed", ());
    Ok(updated.monitor_paused)
}

fn apply_autostart(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Ok(());
    }
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|error| error.to_string())
    } else {
        manager.disable().map_err(|error| error.to_string())
    }
}

#[tauri::command]
pub fn hide_main(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn accessibility_status() -> Option<bool> {
    crate::platform::accessibility_trusted()
}

#[tauri::command]
pub fn open_accessibility_settings() -> bool {
    crate::platform::open_accessibility_settings()
}

#[tauri::command]
pub fn request_accessibility_permission() -> bool {
    let before = crate::platform::accessibility_trusted();
    let trusted = crate::platform::request_accessibility_access();
    log::info!("accessibility permission requested: before={before:?} trusted={trusted}");
    trusted
}

#[tauri::command]
pub fn screenshot_permission_status() -> Option<bool> {
    crate::platform::screen_capture_trusted()
}

#[tauri::command]
pub fn request_screenshot_permission() -> bool {
    let before = crate::platform::screen_capture_trusted();
    let granted = crate::platform::request_screen_capture_access();
    log::info!("screen capture permission requested: before={before:?} granted={granted}");
    granted
}

#[tauri::command]
pub fn open_screen_recording_settings() -> bool {
    crate::platform::open_screen_recording_settings()
}

#[tauri::command]
pub fn open_logs_dir(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_log_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    if crate::platform::open_path(&dir) {
        Ok(())
    } else {
        Err("logs.open_failed".to_string())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub body: Option<String>,
    pub date: Option<String>,
}

/// Compares two dotted numeric versions (with optional leading `v`), e.g. `v0.1.2` > `0.1.1`.
/// Returns true only when `latest` is strictly newer than `current`.
fn is_newer_version(latest: &str, current: &str) -> bool {
    fn parse(version: &str) -> Option<Vec<u64>> {
        let trimmed = version
            .trim()
            .trim_start_matches('v')
            .trim_start_matches('V');
        let core = trimmed.split(['-', '+']).next().unwrap_or(trimmed);
        let parts: Option<Vec<u64>> = core
            .split('.')
            .map(|part| part.parse::<u64>().ok())
            .collect();
        parts.filter(|parts| !parts.is_empty())
    }

    let (Some(latest), Some(current)) = (parse(latest), parse(current)) else {
        return false;
    };
    for index in 0..latest.len().max(current.len()) {
        let left = latest.get(index).copied().unwrap_or(0);
        let right = current.get(index).copied().unwrap_or(0);
        if left != right {
            return left > right;
        }
    }
    false
}

#[tauri::command]
pub async fn check_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    #[derive(serde::Deserialize)]
    struct GithubRelease {
        tag_name: String,
        body: Option<String>,
        published_at: Option<String>,
    }

    let client = reqwest::Client::builder()
        .user_agent("uClipboard")
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .get("https://api.github.com/repos/pigstrong9/uClipboard/releases/latest")
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let release: GithubRelease = response.json().await.map_err(|error| error.to_string())?;

    let current_version = app.package_info().version.to_string();
    if !is_newer_version(&release.tag_name, &current_version) {
        return Ok(None);
    }
    Ok(Some(UpdateInfo {
        version: release.tag_name,
        current_version,
        body: release.body,
        date: release.published_at,
    }))
}

/// Opens the latest release page in the default browser so the user can pick
/// the right package (dmg on macOS, exe on Windows) and install it themselves.
#[tauri::command]
pub fn open_release_page() -> Result<(), String> {
    const URL: &str = "https://github.com/pigstrong9/uClipboard/releases/latest";
    if crate::platform::open_url(URL) {
        Ok(())
    } else {
        Err("update.open_failed".to_string())
    }
}

#[tauri::command]
pub async fn copy_item(state: State<'_, AppState>, id: String) -> Result<ClipItem, String> {
    let shared = state.shared();
    tauri::async_runtime::spawn_blocking(move || write_item_to_clipboard(&shared, &id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn paste_item(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ClipItem, String> {
    let shared = state.shared();
    #[cfg(target_os = "macos")]
    {
        let trusted = crate::platform::accessibility_trusted();
        log::debug!("paste_item start: accessibility={trusted:?}");
        if trusted == Some(false) {
            return Err("paste.accessibility_required".to_string());
        }
    }
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "window.missing".to_string())?;
    let task_window = window.clone();
    let task_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<ClipItem, String> {
        let updated = write_item_to_clipboard(&shared, &id)?;
        #[cfg(target_os = "windows")]
        let mut enigo = ();
        #[cfg(not(target_os = "windows"))]
        let mut enigo = Enigo::new(&Settings::default()).map_err(|error| {
            log::error!("init paste failed: {error}");
            "paste.init_failed".to_string()
        })?;

        hide_before_paste(&task_app, &task_window);
        std::thread::sleep(Duration::from_millis(200));
        send_paste_shortcut(&mut enigo).map_err(|error| {
            log::error!("send paste shortcut failed: {error}");
            "paste.send_failed".to_string()
        })?;
        Ok(updated)
    })
    .await
    .map_err(|e| e.to_string())?;

    match result {
        Ok(updated) => Ok(updated),
        Err(error) => {
            let _ = window.show();
            let _ = window.set_focus();
            let _ = app.run_on_main_thread(crate::platform::activate_app);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn paste_transfer_item(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ClipItem, String> {
    let shared = state.shared();
    #[cfg(target_os = "macos")]
    if crate::platform::accessibility_trusted() == Some(false) {
        return Err("paste.accessibility_required".to_string());
    }
    let transfer = app
        .get_webview_window("transfer")
        .ok_or_else(|| "window.missing".to_string())?;

    let task_app = app.clone();
    let task_transfer = transfer.clone();
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<ClipItem, String> {
        let updated = write_item_to_clipboard(&shared, &id)?;
        #[cfg(target_os = "windows")]
        let mut enigo = ();
        #[cfg(not(target_os = "windows"))]
        let mut enigo = Enigo::new(&Settings::default()).map_err(|error| {
            log::error!("init paste failed: {error}");
            "paste.init_failed".to_string()
        })?;

        #[cfg(target_os = "windows")]
        {
            let target = task_app
                .state::<AppState>()
                .target_hwnd
                .lock()
                .ok()
                .and_then(|guard| *guard);
            if let Some(hwnd) = target {
                if crate::platform::focus_window(hwnd) {
                    std::thread::sleep(Duration::from_millis(160));
                    if send_paste_shortcut(&mut enigo).is_ok() {
                        return Ok(updated);
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Hiding just the window leaves uClipboard active, so Cmd+V would be sent
            // back to us instead of the app the user was working in.
            let handle = task_app.clone();
            let _ = handle.run_on_main_thread(crate::platform::hide_app);
        }
        #[cfg(not(target_os = "macos"))]
        {
            task_transfer.hide().map_err(|e| e.to_string())?;
            if let Some(main) = task_app.get_webview_window("main") {
                if main.is_visible().unwrap_or(false) {
                    let _ = main.hide();
                }
            }
        }
        std::thread::sleep(Duration::from_millis(200));

        send_paste_shortcut(&mut enigo).map_err(|error| {
            log::error!("send paste shortcut failed: {error}");
            "paste.send_failed".to_string()
        })?;

        std::thread::sleep(Duration::from_millis(120));
        #[cfg(target_os = "macos")]
        {
            let handle = task_app.clone();
            let _ = handle.run_on_main_thread(crate::platform::show_app);
        }
        task_transfer.show().map_err(|e| e.to_string())?;
        Ok(updated)
    })
    .await
    .map_err(|e| e.to_string())?;

    match result {
        Ok(updated) => {
            let _ = app.emit("clipboard-changed", ());
            Ok(updated)
        }
        Err(error) => {
            let _ = transfer.show();
            let _ = transfer.set_focus();
            let _ = app.run_on_main_thread(crate::platform::activate_app);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn open_transfer(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<Vec<ClipItem>, String> {
    let shared = state.shared();
    let unique = {
        let store = shared.store.lock().map_err(|e| e.to_string())?;
        let mut seen = HashSet::new();
        let mut unique = Vec::new();
        for id in ids {
            if !seen.insert(id.clone()) {
                continue;
            }
            if store.get(&id).is_ok() {
                unique.push(id);
            }
        }
        unique
    };

    if unique.is_empty() {
        return Ok(Vec::new());
    }

    *state.transfer_ids.lock().map_err(|e| e.to_string())? = unique.clone();
    let window = ensure_transfer_window(&app).map_err(|e| {
        eprintln!("open_transfer ensure failed: {e}");
        e
    })?;
    window.show().map_err(|e| {
        eprintln!("open_transfer show failed: {e}");
        e.to_string()
    })?;
    // Same macOS quirk as the main panel: a window created hidden never becomes key, so
    // the webview has to be put back in the responder chain before it can take keys.
    #[cfg(target_os = "macos")]
    {
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            if let Some(window) = handle.get_webview_window("transfer") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            crate::platform::activate_app();
            if let Some(window) = handle.get_webview_window("transfer") {
                let _ = window.set_focus();
                crate::platform::focus_webview(&window);
            }
        });
    }
    // The window is only hidden when closed, so its React state survives; tell it to
    // drop the previous search/expand state before new items arrive.
    app.emit("transfer-opened", ()).map_err(|e| e.to_string())?;
    app.emit("transfer-changed", ())
        .map_err(|e| e.to_string())?;
    transfer_items_inner(&shared, &unique)
}

#[tauri::command]
pub fn transfer_items(state: State<'_, AppState>) -> Result<Vec<ClipItem>, String> {
    let shared = state.shared();
    let ids = state
        .transfer_ids
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    transfer_items_inner(&shared, &ids)
}

#[tauri::command]
pub async fn copy_transfer(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let shared = state.shared();
    let ids = state
        .transfer_ids
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let count = tauri::async_runtime::spawn_blocking(move || -> Result<usize, String> {
        let items = transfer_items_inner(&shared, &ids)?;
        if items.is_empty() {
            return Err("clipboard.copy_empty".to_string());
        }

        let source = SystemClipboard;
        let plain_text = shared
            .settings
            .lock()
            .map(|settings| settings.paste_plain_text)
            .unwrap_or(false);
        let mut suppress_hash = None;
        let mut allow_mismatch = false;
        if items.len() == 1 {
            let item = &items[0];
            suppress_hash = clipboard_hash(item, &shared.media_dir);
            allow_mismatch = item.kind == ClipKind::Image;
            if !source.write_with_options(item, &shared.media_dir, plain_text) {
                return Err("clipboard.write_failed".to_string());
            }
        } else if items.iter().all(|item| item.kind == ClipKind::Image) {
            let paths: Vec<PathBuf> = items
                .iter()
                .map(|item| shared.media_dir.join(&item.content))
                .collect();
            let png = media::merge_images_vertical(&paths)?;
            // The merged PNG is re-encoded by the OS, so only the time window can match it.
            allow_mismatch = true;
            if !source.write_image_png(&png) {
                return Err("clipboard.write_failed".to_string());
            }
        } else {
            if items.iter().any(|item| item.kind == ClipKind::Image) {
                return Err("clipboard.merge_mixed".to_string());
            }
            let text = items
                .iter()
                .filter(|item| item.kind == ClipKind::Text)
                .map(|item| item.content.clone())
                .collect::<Vec<_>>()
                .join("\n");
            let mut files = Vec::new();
            for item in items.iter().filter(|item| item.kind == ClipKind::Files) {
                if let Ok(paths) = serde_json::from_str::<Vec<String>>(&item.content) {
                    files.extend(paths);
                }
            }
            // `write_merged` drops missing paths before writing, so hash the same list.
            files.retain(|path| std::path::Path::new(path).exists());
            suppress_hash = if files.is_empty() {
                (!text.is_empty()).then(|| hash_bytes(text.as_bytes()))
            } else {
                Some(hash_files(&files))
            };
            let text_arg = if text.is_empty() {
                None
            } else {
                Some(text.as_str())
            };
            if !source.write_merged(text_arg, &files) {
                return Err("clipboard.write_failed".to_string());
            }
        }
        arm_suppression(&shared, suppress_hash, allow_mismatch)?;
        {
            let store = shared.store.lock().map_err(|e| e.to_string())?;
            for item in &items {
                store.touch(&item.id).map_err(|e| e.to_string())?;
            }
        }
        Ok(items.len())
    })
    .await
    .map_err(|e| e.to_string())??;
    let _ = app.emit("clipboard-changed", ());
    Ok(count)
}

#[tauri::command]
pub fn favorite_transfer(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let app_state = state.inner();
    let ids = app_state
        .transfer_ids
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let store = app_state.store.lock().map_err(|e| e.to_string())?;
    let items: Vec<ClipItem> = ids.iter().filter_map(|id| store.get(id).ok()).collect();
    if items.is_empty() {
        return Err("clipboard.favorite_empty".to_string());
    }
    let all_favorite = items.iter().all(|item| item.favorite);
    for item in &items {
        if item.favorite != !all_favorite {
            store.toggle_favorite(&item.id).map_err(|e| e.to_string())?;
        }
    }
    drop(store);
    let _ = app.emit("clipboard-changed", ());
    let _ = app.emit("transfer-changed", ());
    Ok(!all_favorite)
}

#[tauri::command]
pub fn close_transfer(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state
        .inner()
        .transfer_ids
        .lock()
        .map_err(|e| e.to_string())?
        .clear();
    if let Some(window) = app.get_webview_window("transfer") {
        let _ = window.hide();
    }
    let _ = app.emit("transfer-changed", ());
    Ok(())
}

#[tauri::command]
pub fn stats(state: State<'_, AppState>) -> Result<i64, String> {
    state
        .inner()
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .count()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn favorite_count(state: State<'_, AppState>) -> Result<i64, String> {
    state
        .inner()
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .favorite_count()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn image_file_path(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let app_state = state.inner();
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .get(&id)
        .map_err(|e| e.to_string())?;
    if item.kind != ClipKind::Image {
        return Err("clipboard.not_image".to_string());
    }
    Ok(app_state
        .media_dir
        .join(&item.content)
        .to_string_lossy()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state;
    use crate::storage::Store;
    use std::sync::{Arc, Mutex};

    #[test]
    fn cleanup_media_keeps_images_when_the_store_is_unavailable() {
        let dir = std::env::temp_dir().join(format!("uclipboard-cleanup-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(dir.join("images")).unwrap();
        let image = dir.join("images").join("deadbeef.png");
        std::fs::write(&image, b"png").unwrap();

        let store = Arc::new(Mutex::new(Store::in_memory().unwrap()));
        {
            let poisoned = Arc::clone(&store);
            let handle = std::thread::spawn(move || {
                let _guard = poisoned.lock().unwrap_or_else(|error| error.into_inner());
                panic!("poison the store mutex");
            });
            let _ = handle.join();
        }

        let app_state = AppState {
            store,
            settings: Arc::new(Mutex::new(AppSettings::default())),
            config_path: dir.join("config.json"),
            suppress: Arc::new(Mutex::new(state::Suppression::default())),
            media_dir: dir.clone(),
            transfer_ids: Mutex::new(Vec::new()),
            target_hwnd: Mutex::new(None),
            screenshot: Mutex::new(None),
            main_shown_at: Mutex::new(None),
        };

        assert!(cleanup_media(&app_state.shared()).is_err());
        assert!(image.exists(), "an unreadable store must not delete images");

        // An empty live list wipes everything, which is what the old fallback produced.
        media::cleanup_orphans(&dir, &[]);
        assert!(!image.exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
