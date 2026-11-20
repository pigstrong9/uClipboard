use std::path::Path;

use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use tauri::{
    Emitter, Manager, PhysicalPosition, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::clipboard::hash::{hash_bytes, hash_files};
use crate::clipboard::source::{ClipboardSource, SystemClipboard};
use crate::media;
use crate::models::{ClipItem, ClipKind};
use crate::state::AppState;

fn with_thumb(mut item: ClipItem, state: &AppState) -> ClipItem {
    if item.kind == ClipKind::Image {
        item.thumb_data_url = media::thumb_data_url(&state.media_dir, &item.content);
    }
    item
}

fn item_hash(item: &ClipItem, media_dir: &Path) -> Result<String, String> {
    match item.kind {
        ClipKind::Text => Ok(hash_bytes(item.content.as_bytes())),
        ClipKind::Image => {
            let bytes = std::fs::read(media_dir.join(&item.content)).map_err(|e| e.to_string())?;
            Ok(hash_bytes(&bytes))
        }
        ClipKind::Files => {
            let files: Vec<String> =
                serde_json::from_str(&item.content).map_err(|e| e.to_string())?;
            Ok(hash_files(&files))
        }
    }
}

fn write_item_to_clipboard(app_state: &AppState, id: &str) -> Result<ClipItem, String> {
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .get(id)
        .map_err(|e| e.to_string())?;

    let source = SystemClipboard;
    if !source.write(&item, &app_state.media_dir) {
        return Err("写入剪贴板失败".to_string());
    }

    let hash = item_hash(&item, &app_state.media_dir)?;
    {
        let mut guard = app_state.suppress.lock().map_err(|e| e.to_string())?;
        *guard = Some(hash);
    }

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
        enigo.key(Key::Unicode('v'), Click).map_err(|e| e.to_string())?;
        enigo.key(Key::Meta, Release).map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Press).map_err(|e| e.to_string())?;
        enigo.key(Key::Unicode('v'), Click).map_err(|e| e.to_string())?;
        enigo.key(Key::Control, Release).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn transfer_items_inner(
    state: &AppState,
    ids: &[String],
) -> Result<Vec<ClipItem>, String> {
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

    let window = WebviewWindowBuilder::new(
        app,
        "transfer",
        WebviewUrl::App("transfer.html".into()),
    )
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
pub fn toggle_favorite(state: State<'_, AppState>, id: String) -> Result<ClipItem, String> {
    let app_state = state.inner();
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .toggle_favorite(&id)
        .map_err(|e| e.to_string())?;
    Ok(with_thumb(item, app_state))
}

#[tauri::command]
pub fn toggle_pin(state: State<'_, AppState>, id: String) -> Result<ClipItem, String> {
    let app_state = state.inner();
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .toggle_pin(&id)
        .map_err(|e| e.to_string())?;
    Ok(with_thumb(item, app_state))
}

#[tauri::command]
pub fn delete_item(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .inner()
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .delete(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_items(state: State<'_, AppState>) -> Result<(), String> {
    state
        .inner()
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .clear()
        .map_err(|e| e.to_string())
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
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("初始化粘贴失败: {e}"))?;

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
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("初始化粘贴失败: {e}"))?;

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
pub fn copy_transfer(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
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
    let suppress_hash;
    if items.len() == 1 {
        let item = &items[0];
        if !source.write(item, &app_state.media_dir) {
            return Err("写入剪贴板失败".to_string());
        }
        suppress_hash = item_hash(item, &app_state.media_dir)?;
    } else {
        if items.iter().any(|item| item.kind == ClipKind::Image) {
            return Err("多条合并复制暂不支持图片".to_string());
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
        suppress_hash = if files.is_empty() {
            hash_bytes(text.as_bytes())
        } else {
            hash_files(&files)
        };
    }

    {
        let mut guard = app_state.suppress.lock().map_err(|e| e.to_string())?;
        *guard = Some(suppress_hash);
    }
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
pub fn close_transfer(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
    let bytes = std::fs::read(app_state.media_dir.join(&item.content)).map_err(|e| e.to_string())?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}
