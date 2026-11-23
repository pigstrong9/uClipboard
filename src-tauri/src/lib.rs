mod clipboard;
mod commands;
mod hotkey_capture;
mod media;
mod models;
mod platform;
mod screenshot;
mod settings;
mod state;
mod storage;
mod tray;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{Emitter, Manager, PhysicalPosition, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

fn center_on_cursor(window: &tauri::WebviewWindow) {
    let Ok(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(monitors) = window.available_monitors() else {
        return;
    };
    let monitor = monitors.into_iter().find(|monitor| {
        let position = monitor.position();
        let size = monitor.size();
        cursor.x >= position.x as f64
            && cursor.y >= position.y as f64
            && cursor.x < (position.x + size.width as i32) as f64
            && cursor.y < (position.y + size.height as i32) as f64
    });
    let Some(monitor) = monitor else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };
    let work = monitor.work_area();
    let x = work.position.x + (work.size.width as i32 - size.width as i32) / 2;
    let y = work.position.y + (work.size.height as i32 - size.height as i32) / 2;
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

pub(crate) fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        center_on_cursor(&window);
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("main-shown", ());
    }
}

fn remember_target(app: &tauri::AppHandle) {
    if let Some(state) = app.try_state::<state::AppState>() {
        if let Some(hwnd) = platform::foreground_target() {
            if let Ok(mut guard) = state.target_hwnd.lock() {
                *guard = Some(hwnd);
            }
        }
    }
}

fn toggle_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        let minimized = window.is_minimized().unwrap_or(false);
        if visible && !minimized {
            let _ = window.hide();
        } else {
            remember_target(app);
            show_main(app);
        }
    }
}

fn on_shortcut(
    app: &tauri::AppHandle,
    shortcut: &Shortcut,
    event: tauri_plugin_global_shortcut::ShortcutEvent,
) {
    if event.state != tauri_plugin_global_shortcut::ShortcutState::Pressed {
        return;
    }
    let screenshot_hotkey = app
        .state::<state::AppState>()
        .settings
        .lock()
        .map(|settings| settings.screenshot_hotkey.clone())
        .unwrap_or_default();
    let is_screenshot = screenshot_hotkey
        .parse::<Shortcut>()
        .map(|configured| configured.id() == shortcut.id())
        .unwrap_or(false);
    if is_screenshot {
        screenshot::trigger(app.clone());
    } else {
        toggle_main(app);
    }
}

async fn check_for_update(app: tauri::AppHandle) {
    use tauri_plugin_updater::UpdaterExt;
    if let Ok(updater) = app.updater() {
        if let Ok(Some(update)) = updater.check().await {
            let _ = app.emit("update-available", update.version);
        }
    }
}

fn spawn_update_checker(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(24 * 60 * 60));
        let enabled = app
            .try_state::<state::AppState>()
            .and_then(|state| {
                state
                    .settings
                    .lock()
                    .ok()
                    .map(|settings| settings.auto_update)
            })
            .unwrap_or(false);
        if enabled {
            tauri::async_runtime::block_on(check_for_update(app.clone()));
        }
    });
}

#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--autostart")
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let store = storage::Store::open(&app_dir.join("uclipboard.db"))?;
            let settings_path = app_dir.join("config.json");
            let loaded = settings::load(&settings_path);
            let _ = settings::save(&settings_path, &loaded);
            let settings = Arc::new(Mutex::new(loaded.clone()));
            let store = Arc::new(Mutex::new(store));
            let suppress = Arc::new(Mutex::new(state::Suppression::default()));
            let media_dir = app_dir.clone();

            let live = store
                .lock()
                .map_err(|error| error.to_string())
                .and_then(|store| {
                    let _ = store.prune(loaded.history_limit, loaded.retention_days);
                    store.image_paths().map_err(|error| error.to_string())
                });
            match live {
                Ok(live) => media::cleanup_orphans(&media_dir, &live),
                Err(error) => log::warn!("startup media cleanup skipped: {error}"),
            }

            app.manage(state::AppState {
                store: store.clone(),
                settings: settings.clone(),
                config_path: settings_path,
                suppress: suppress.clone(),
                media_dir: media_dir.clone(),
                transfer_ids: Mutex::new(Vec::new()),
                target_hwnd: Mutex::new(None),
                screenshot: Mutex::new(None),
            });

            #[cfg(not(debug_assertions))]
            {
                use tauri_plugin_autostart::ManagerExt;
                let manager = app.autolaunch();
                if loaded.auto_start {
                    let _ = manager.enable();
                } else {
                    let _ = manager.disable();
                }
            }

            let handle = app.handle().clone();
            clipboard::monitor::spawn_monitor(handle, store, settings, suppress, media_dir);

            {
                let plugin = tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(on_shortcut)
                    .build();
                app.handle().plugin(plugin)?;
            }

            let mut shortcut_notice: Option<&'static str> = None;
            {
                let manager = app.global_shortcut();
                if let Err(error) = manager.register(loaded.hotkey.as_str()) {
                    eprintln!("register global shortcut {} failed: {error}", loaded.hotkey);
                    if loaded.hotkey != "Alt+V" {
                        if let Err(fallback) = manager.register("Alt+V") {
                            eprintln!("fallback global shortcut failed: {fallback}");
                        }
                    }
                }
                if let Err(error) = manager.register(loaded.screenshot_hotkey.as_str()) {
                    eprintln!(
                        "register screenshot shortcut {} failed: {error}",
                        loaded.screenshot_hotkey
                    );
                    let fallback = "Ctrl+Shift+A";
                    if fallback != loaded.screenshot_hotkey && manager.register(fallback).is_ok() {
                        let mut updated = loaded.clone();
                        updated.screenshot_hotkey = fallback.to_string();
                        let state = app.state::<state::AppState>();
                        if let Ok(mut guard) = state.settings.lock() {
                            *guard = updated.clone();
                        }
                        if let Err(error) = settings::save(&state.config_path, &updated) {
                            log::error!("save screenshot shortcut fallback failed: {error}");
                        }
                        shortcut_notice = Some("screenshot.hotkey_fallback");
                    } else {
                        shortcut_notice = Some("screenshot.hotkey_failed");
                    }
                }
            }

            if let Some(code) = shortcut_notice {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    // The main window is shown shortly after setup; wait for the
                    // frontend listeners before surfacing the notice.
                    std::thread::sleep(Duration::from_millis(1500));
                    let _ = handle.emit("screenshot-error", code.to_string());
                });
            }

            tray::setup(app.handle())?;

            if let Some(main) = app.get_webview_window("main") {
                let handle = app.handle().clone();
                main.on_window_event(move |event| match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    WindowEvent::Focused(false) => {
                        let hide_on_blur = handle
                            .try_state::<state::AppState>()
                            .and_then(|state| {
                                state
                                    .settings
                                    .lock()
                                    .ok()
                                    .map(|settings| settings.hide_on_blur)
                            })
                            .unwrap_or(false);
                        if !hide_on_blur {
                            return;
                        }
                        let transfer_visible = handle
                            .get_webview_window("transfer")
                            .and_then(|window| window.is_visible().ok())
                            .unwrap_or(false);
                        if transfer_visible {
                            return;
                        }
                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    _ => {}
                });
            }

            if loaded.auto_update {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(check_for_update(handle));
            }
            spawn_update_checker(app.handle().clone());

            let autostart = std::env::args().any(|arg| arg == "--autostart");
            if !autostart {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    // The configured hidden window becomes visible only after the
                    // initial window state has been applied by the runtime.
                    std::thread::sleep(Duration::from_millis(250));
                    show_main(&handle);
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::list_items,
            commands::get_item,
            commands::toggle_favorite,
            commands::toggle_pin,
            commands::delete_item,
            commands::clear_items,
            commands::copy_item,
            commands::paste_item,
            commands::paste_transfer_item,
            commands::open_transfer,
            commands::transfer_items,
            commands::copy_transfer,
            commands::favorite_transfer,
            commands::close_transfer,
            commands::stats,
            commands::favorite_count,
            commands::image_file_path,
            commands::get_settings,
            commands::update_settings,
            commands::toggle_monitoring,
            commands::hide_main,
            commands::accessibility_status,
            commands::open_accessibility_settings,
            commands::open_logs_dir,
            commands::check_update,
            commands::install_update,
            screenshot::start_region_capture,
            screenshot::screenshot_session,
            screenshot::screenshot_ready,
            screenshot::capture_region,
            screenshot::cancel_region_capture,
            hotkey_capture::begin_hotkey_capture,
            hotkey_capture::end_hotkey_capture
        ])
        .run(tauri::generate_context!())
        .expect("error while running uClipboard");
}
