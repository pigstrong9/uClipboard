use std::path::Path;

use tauri::State;

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
    let app_state = state.inner();
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .get(&id)
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
        store.touch(&id).map_err(|e| e.to_string())?;
        store.get(&id).map_err(|e| e.to_string())?
    };
    Ok(with_thumb(updated, app_state))
}

#[tauri::command]
pub fn paste_item(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<ClipItem, String> {
    use enigo::{
        Direction::{Click, Press, Release},
        Enigo, Key, Keyboard, Settings,
    };
    use tauri::Manager;

    let app_state = state.inner();
    let item = app_state
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .get(&id)
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
        store.touch(&id).map_err(|e| e.to_string())?;
        store.get(&id).map_err(|e| e.to_string())?
    };

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("初始化粘贴失败: {e}"))?;

    window.hide().map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    let paste_result = (|| -> Result<(), String> {
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
    })();

    if let Err(error) = paste_result {
        let _ = window.show();
        let _ = window.set_focus();
        return Err(format!("自动粘贴失败: {error}"));
    }

    Ok(with_thumb(updated, app_state))
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
