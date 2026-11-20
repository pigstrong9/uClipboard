mod clipboard;
mod commands;
mod media;
mod models;
mod state;
mod storage;

use std::sync::{Arc, Mutex};

use tauri::Manager;

#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let store = storage::Store::open(&app_dir.join("uclipboard.db"))?;
            let store = Arc::new(Mutex::new(store));
            let suppress = Arc::new(Mutex::new(None));
            let media_dir = app_dir.clone();

            app.manage(state::AppState {
                store: store.clone(),
                suppress: suppress.clone(),
                media_dir: media_dir.clone(),
            });

            let handle = app.handle().clone();
            clipboard::monitor::spawn_monitor(handle, store, suppress, media_dir);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::list_items,
            commands::toggle_favorite,
            commands::toggle_pin,
            commands::delete_item,
            commands::clear_items,
            commands::copy_item,
            commands::stats,
            commands::favorite_count,
            commands::image_data_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running uClipboard");
}
