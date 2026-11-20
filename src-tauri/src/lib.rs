mod clipboard;
mod commands;
mod media;
mod models;
mod state;
mod storage;

use std::sync::{Arc, Mutex};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

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

            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(|app, _shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                show_main(app);
                            }
                        })
                        .build(),
                )?;

                if let Err(error) = app.handle().global_shortcut().register("alt+v") {
                    eprintln!("register Alt+V failed: {error}");
                }
            }

            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or("missing application icon")?;
            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("uClipboard")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;

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
            commands::paste_item,
            commands::stats,
            commands::favorite_count,
            commands::image_data_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running uClipboard");
}
