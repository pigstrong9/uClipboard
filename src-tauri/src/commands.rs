use std::path::PathBuf;
use std::time::Duration;

use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use tauri::{
    Emitter, Manager, PhysicalPosition, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::clipboard::source::{ClipboardSource, SystemClipboard};
use crate::media;
use crate::models::{ClipItem, ClipKind};
use crate::settings::{self, Settings as AppSettings};
use crate::state::AppState;

fn with_thumb(mut item: ClipItem, state: &AppState) -> ClipItem {
    if item.kind == ClipKind::Image {
        item.thumb_data_url = media::thumb_data_url(&state.media_dir, &item.content);
    }
    item
}

fn cleanup_media(app_state: &AppState) {
    let live = app_state
        .store
        .lock()
        .map(|store| store.image_paths().unwrap_or_default())
        .unwrap_or_default();
    media::cleanup_orphans(&app_state.media_dir, &live);
}

fn arm_suppression(app_state: &AppState) -> Result<(), String> {
    app_state
        .suppress
        .lock()
        .map_err(|e| e.to_string())?
        .arm(Duration::from_millis(1500));
    Ok(())
}

fn write_item_to_clipboard(app_state: &AppState, id: &str) -> Result<ClipItem, String> {
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .get(id)
        .map_err(|e| e.to_string())?;
    let plain_text = app_state
        .settings
        .lock()
        .map(|settings| settings.paste_plain_text)
        .unwrap_or(false);

    let source = SystemClipboard;
    if !source.write_with_options(&item, &app_state.media_dir, plain_text) {
        return Err("写入剪贴板失败".to_string());
    }
    arm_suppression(app_state)?;

    let updated = {
        let store = app_state.store.lock().map_err(|e| e.to_string())?;
        store.touch(id).map_err(|e| e.to_string())?;
        store.get(id).map_err(|e| e.to_string())?
    };
    Ok(with_thumb(updated, app_state))
}

fn send_paste_shortcut(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Press).map_err(|e| e.to_string())?;
        enigo
            .key(Key::Unicode('v'), Click)
            .map_err(|e| e.to_string())?;
        enigo.key(Key::Meta, Release).map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Press).map_err(|e| e.to_string())?;
        enigo
            .key(Key::Unicode('v'), Click)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Control, Release)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn transfer_items_inner(state: &AppState, ids: &[String]) -> Result<Vec<ClipItem>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for id in ids {
        if let Ok(item) = store.get(id) {
            items.push(with_thumb(item, state));
        }
    }
    Ok(items)
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
pub fn list_items(
    state: State<'_, AppState>,
    tab: String,
    query: String,
) -> Result<Vec<ClipItem>, String> {
    let app_state = state.inner();
    let items = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .list(&tab, &query)
        .map_err(|e| e.to_string())?;
    Ok(items
        .into_iter()
        .map(|item| with_thumb(item, app_state))
        .collect())
}

#[tauri::command]
pub fn toggle_favorite(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ClipItem, String> {
    let app_state = state.inner();
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .toggle_favorite(&id)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("transfer-changed", ());
    Ok(with_thumb(item, app_state))
}

#[tauri::command]
pub fn toggle_pin(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ClipItem, String> {
    let app_state = state.inner();
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .toggle_pin(&id)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("transfer-changed", ());
    Ok(with_thumb(item, app_state))
}

#[tauri::command]
pub fn delete_item(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let app_state = state.inner();
    app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .delete(&id)
        .map_err(|e| e.to_string())?;
    cleanup_media(app_state);
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
    cleanup_media(state.inner());
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

    let autostart_changed = previous.auto_start != normalized.auto_start;
    if autostart_changed {
        apply_autostart(&app, normalized.auto_start)?;
    }

    if previous.hotkey != normalized.hotkey {
        let manager = app.global_shortcut();
        let _ = manager.unregister(previous.hotkey.as_str());
        if let Err(error) = manager.register(normalized.hotkey.as_str()) {
            let _ = manager.register(previous.hotkey.as_str());
            if autostart_changed {
                let _ = apply_autostart(&app, previous.auto_start);
            }
            return Err(format!("快捷键注册失败: {error}"));
        }
    }

    if let Err(error) = settings::save(&app_state.config_path, &normalized) {
        if previous.hotkey != normalized.hotkey {
            let manager = app.global_shortcut();
            let _ = manager.unregister(normalized.hotkey.as_str());
            let _ = manager.register(previous.hotkey.as_str());
        }
        if previous.auto_start != normalized.auto_start {
            let _ = apply_autostart(&app, previous.auto_start);
        }
        return Err(error);
    }
    *app_state.settings.lock().map_err(|e| e.to_string())? = normalized.clone();
    if previous.language != normalized.language {
        let _ = crate::tray::refresh(&app);
    }
    let _ = app.emit("settings-changed", ());
    Ok(normalized)
}

#[tauri::command]
pub fn toggle_monitoring(app: tauri::AppHandle) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let mut updated = state.settings.lock().map_err(|e| e.to_string())?.clone();
    updated.monitor_paused = !updated.monitor_paused;
    settings::save(&state.config_path, &updated)?;
    *state.settings.lock().map_err(|e| e.to_string())? = updated.clone();
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub body: Option<String>,
    pub date: Option<String>,
}

#[tauri::command]
pub async fn check_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|error| error.to_string())?;
    let update = updater.check().await.map_err(|error| error.to_string())?;
    Ok(update.map(|update| UpdateInfo {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        body: update.body.clone(),
        date: update.date.map(|date| date.to_string()),
    }))
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|error| error.to_string())?;
    let update = updater
        .check()
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "当前已是最新版本".to_string())?;
    update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .map_err(|error| error.to_string())?;
    #[cfg(not(target_os = "windows"))]
    {
        app.restart()
    }
    #[cfg(target_os = "windows")]
    Ok(())
}

#[tauri::command]
pub fn copy_item(state: State<'_, AppState>, id: String) -> Result<ClipItem, String> {
    write_item_to_clipboard(state.inner(), &id)
}

#[tauri::command]
pub fn paste_item(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ClipItem, String> {
    let app_state = state.inner();
    let updated = write_item_to_clipboard(app_state, &id)?;

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("初始化粘贴失败: {e}"))?;

    window.hide().map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    if let Err(error) = send_paste_shortcut(&mut enigo) {
        let _ = window.show();
        let _ = window.set_focus();
        return Err(format!("自动粘贴失败: {error}"));
    }

    Ok(updated)
}

#[tauri::command]
pub fn paste_transfer_item(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ClipItem, String> {
    let app_state = state.inner();
    let updated = write_item_to_clipboard(app_state, &id)?;

    let transfer = app
        .get_webview_window("transfer")
        .ok_or_else(|| "中转窗口不存在".to_string())?;
    let main = app.get_webview_window("main");
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("初始化粘贴失败: {e}"))?;

    #[cfg(target_os = "windows")]
    {
        let target = app_state.target_hwnd.lock().ok().and_then(|guard| *guard);
        if let Some(hwnd) = target {
            if crate::platform::focus_window(hwnd) {
                std::thread::sleep(std::time::Duration::from_millis(160));
                if send_paste_shortcut(&mut enigo).is_ok() {
                    let _ = app.emit("clipboard-changed", ());
                    return Ok(updated);
                }
            }
        }
    }

    transfer.hide().map_err(|e| e.to_string())?;
    if let Some(main) = &main {
        if main.is_visible().unwrap_or(false) {
            let _ = main.hide();
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(200));

    if let Err(error) = send_paste_shortcut(&mut enigo) {
        let _ = transfer.show();
        let _ = transfer.set_focus();
        return Err(format!("自动粘贴失败: {error}"));
    }

    std::thread::sleep(std::time::Duration::from_millis(120));
    transfer.show().map_err(|e| e.to_string())?;
    let _ = app.emit("clipboard-changed", ());
    Ok(updated)
}

#[tauri::command]
pub async fn open_transfer(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<Vec<ClipItem>, String> {
    let app_state = state.inner();
    let mut unique: Vec<String> = Vec::new();
    {
        let store = app_state.store.lock().map_err(|e| e.to_string())?;
        for id in ids {
            if unique.iter().any(|existing| existing == &id) {
                continue;
            }
            if store.get(&id).is_ok() {
                unique.push(id);
            }
        }
    }

    if unique.is_empty() {
        return Ok(Vec::new());
    }

    *app_state.transfer_ids.lock().map_err(|e| e.to_string())? = unique.clone();
    let window = ensure_transfer_window(&app).map_err(|e| {
        eprintln!("open_transfer ensure failed: {e}");
        e
    })?;
    window.show().map_err(|e| {
        eprintln!("open_transfer show failed: {e}");
        e.to_string()
    })?;
    app.emit("transfer-changed", ())
        .map_err(|e| e.to_string())?;
    transfer_items_inner(app_state, &unique)
}

#[tauri::command]
pub fn transfer_items(state: State<'_, AppState>) -> Result<Vec<ClipItem>, String> {
    let app_state = state.inner();
    let ids = app_state
        .transfer_ids
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    transfer_items_inner(app_state, &ids)
}

#[tauri::command]
pub fn copy_transfer(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<usize, String> {
    let app_state = state.inner();
    let ids = app_state
        .transfer_ids
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let items = transfer_items_inner(app_state, &ids)?;
    if items.is_empty() {
        return Err("没有可复制的内容".to_string());
    }

    let source = SystemClipboard;
    let plain_text = app_state
        .settings
        .lock()
        .map(|settings| settings.paste_plain_text)
        .unwrap_or(false);
    if items.len() == 1 {
        let item = &items[0];
        if !source.write_with_options(item, &app_state.media_dir, plain_text) {
            return Err("写入剪贴板失败".to_string());
        }
    } else if items.iter().all(|item| item.kind == ClipKind::Image) {
        let paths: Vec<PathBuf> = items
            .iter()
            .map(|item| app_state.media_dir.join(&item.content))
            .collect();
        let png = media::merge_images_vertical(&paths)?;
        if !source.write_image_png(&png) {
            return Err("写入合并图片失败".to_string());
        }
    } else {
        if items.iter().any(|item| item.kind == ClipKind::Image) {
            return Err("图片暂不能与文本或文件合并复制".to_string());
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
        let text_arg = if text.is_empty() {
            None
        } else {
            Some(text.as_str())
        };
        if !source.write_merged(text_arg, &files) {
            return Err("写入剪贴板失败".to_string());
        }
    }
    arm_suppression(app_state)?;
    {
        let store = app_state.store.lock().map_err(|e| e.to_string())?;
        for item in &items {
            store.touch(&item.id).map_err(|e| e.to_string())?;
        }
    }
    let _ = app.emit("clipboard-changed", ());
    Ok(items.len())
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
        return Err("没有可收藏的内容".to_string());
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
pub fn image_data_url(state: State<'_, AppState>, id: String) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let app_state = state.inner();
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .get(&id)
        .map_err(|e| e.to_string())?;
    if item.kind != ClipKind::Image {
        return Err("不是图片条目".to_string());
    }
    let bytes =
        std::fs::read(app_state.media_dir.join(&item.content)).map_err(|e| e.to_string())?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}
