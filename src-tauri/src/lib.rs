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
use std::time::{Duration, Instant};

use tauri::{Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// Returns the cursor position in physical screen coordinates.
///
/// `WebviewWindow::cursor_position()` is window-relative. Feeding that value into a
/// monitor lookup makes the "display under the cursor" flip between displays on every
/// show when the pointer sits on a secondary monitor, so the window lands in a
/// different place each time and wheel scrolling only works on alternating summons.
#[cfg(target_os = "windows")]
fn cursor_screen_position() -> Option<PhysicalPosition<f64>> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut point) == 0 {
            return None;
        }
        Some(PhysicalPosition::new(point.x as f64, point.y as f64))
    }
}

#[cfg(not(target_os = "windows"))]
fn cursor_screen_position(window: &tauri::WebviewWindow) -> Option<PhysicalPosition<f64>> {
    let relative = window.cursor_position().ok()?;
    let outer = window.outer_position().ok()?;
    Some(PhysicalPosition::new(
        outer.x as f64 + relative.x,
        outer.y as f64 + relative.y,
    ))
}

fn monitor_under_cursor(window: &tauri::WebviewWindow) -> Option<tauri::Monitor> {
    #[cfg(target_os = "windows")]
    let cursor = cursor_screen_position()?;
    #[cfg(not(target_os = "windows"))]
    let cursor = cursor_screen_position(window)?;
    window
        .available_monitors()
        .ok()?
        .into_iter()
        .find(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            cursor.x >= position.x as f64
                && cursor.y >= position.y as f64
                && cursor.x < (position.x + size.width as i32) as f64
                && cursor.y < (position.y + size.height as i32) as f64
        })
}

/// The base window size stays the floor on small screens, while high-resolution
/// displays get a proportionally larger popup. Height follows the monitor work
/// area and width keeps the 4:3 ratio of the base window, so the list layout
/// never turns into a narrow strip on wide screens.
fn adaptive_window_size(work_size: PhysicalSize<u32>, scale_factor: f64) -> LogicalSize<f64> {
    const BASE_WIDTH: f64 = 880.0;
    const BASE_HEIGHT: f64 = 660.0;
    const MAX_WIDTH: f64 = 1200.0;
    const MAX_HEIGHT: f64 = 900.0;
    const MIN_WIDTH: f64 = 720.0;
    const MIN_HEIGHT: f64 = 520.0;

    let scale = if scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    let work_width = work_size.width as f64 / scale;
    let work_height = work_size.height as f64 / scale;
    if !work_width.is_finite()
        || !work_height.is_finite()
        || work_width <= 0.0
        || work_height <= 0.0
    {
        return LogicalSize::new(BASE_WIDTH, BASE_HEIGHT);
    }

    // Leave a small gap so the window never sits flush against the screen edges.
    let max_width = (work_width - 32.0).max(MIN_WIDTH);
    let max_height = (work_height - 32.0).max(MIN_HEIGHT);
    let height = (work_height * 0.62)
        .clamp(BASE_HEIGHT, MAX_HEIGHT)
        .min(max_height);
    let width = (height * BASE_WIDTH / BASE_HEIGHT)
        .clamp(BASE_WIDTH, MAX_WIDTH)
        .min(max_width);
    // Re-derive the height after width clamping so the ratio stays 4:3.
    let height = (width * BASE_HEIGHT / BASE_WIDTH)
        .clamp(MIN_HEIGHT, MAX_HEIGHT)
        .min(max_height);
    LogicalSize::new(width, height)
}

fn fit_and_center_on_cursor(window: &tauri::WebviewWindow) {
    let Some(monitor) = monitor_under_cursor(window) else {
        return;
    };
    let work = monitor.work_area();
    let scale = monitor.scale_factor();
    let size = adaptive_window_size(work.size, scale);
    let physical = size.to_physical::<u32>(scale);
    let x = work.position.x + (work.size.width as i32 - physical.width as i32) / 2;
    let y = work.position.y + (work.size.height as i32 - physical.height as i32) / 2;
    // Move first so the window picks up the target monitor's scale factor, then
    // apply the logical size (Tauri converts it with the now-current scale).
    let _ = window.set_position(PhysicalPosition::new(x, y));
    let _ = window.set_size(size);
}

pub(crate) fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Some(state) = app.try_state::<state::AppState>() {
            if let Ok(mut guard) = state.main_shown_at.lock() {
                *guard = Some(Instant::now());
            }
        }
        let _ = window.unminimize();
        fit_and_center_on_cursor(&window);
        let _ = window.show();

        // A plain set_focus() is frequently rejected by Windows' foreground lock when
        // the window was just hidden: it stays visible but never receives focus, so
        // WM_MOUSEWHEEL goes to the previous app until the user clicks us.
        #[cfg(target_os = "windows")]
        {
            if let Ok(hwnd) = window.hwnd() {
                if !crate::platform::activate_window(hwnd.0 as isize) {
                    log::warn!("main window stayed inactive after show");
                }
                if !crate::platform::focus_webview_child(hwnd.0 as isize) {
                    log::warn!("webview focus was not applied after show");
                }
                crate::platform::cancel_alt_menu_mode(hwnd.0 as isize);
            }
        }
        // Activation owns the top-level window; sync the webview too, otherwise the
        // WebView2 child can keep a stale visibility/focus state after hide/show and
        // drop the wheel even though the window is foreground.
        let _ = window.set_focus();
        let webview: &tauri::Webview = window.as_ref();
        let _ = webview.show();
        let _ = webview.set_focus();

        // `show` from a worker thread is queued on the main thread, so activation has to
        // be queued behind it or the app is asked to activate while still windowless.
        #[cfg(target_os = "macos")]
        {
            let handle = app.clone();
            let _ = app.run_on_main_thread(move || {
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                crate::platform::activate_app();
                // Window focus alone leaves the WKWebView out of the responder chain on
                // macOS, so keystrokes never reach the page until the user clicks.
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window.set_focus();
                    crate::platform::focus_webview(&window);
                }
            });
        }
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
    if let Ok(Some(info)) = crate::commands::check_update(app.clone()).await {
        let _ = app.emit("update-available", info.version);
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
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            // Menu bar app: no Dock icon, mirroring skipTaskbar on Windows.
            #[cfg(target_os = "macos")]
            let _ = app
                .handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory);

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
                main_shown_at: Mutex::new(None),
            });

            #[cfg(not(debug_assertions))]
            {
                use tauri_plugin_autostart::ManagerExt;
                let manager = app.autolaunch();
                if loaded.auto_start {
                    if let Err(error) = manager.enable() {
                        log::error!("enable autostart failed: {error}");
                    }
                } else if let Err(error) = manager.disable() {
                    log::error!("disable autostart failed: {error}");
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
                    WindowEvent::Focused(focused) => {
                        if *focused {
                            return;
                        }
                        let state = handle.try_state::<state::AppState>();
                        let hide_on_blur = state
                            .as_ref()
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
                        // Showing the window can briefly emit Focused(false) while
                        // Windows settles the foreground window; ignoring that grace
                        // period prevents the "flash then disappear" behaviour.
                        let just_shown = state
                            .as_ref()
                            .and_then(|state| {
                                state.main_shown_at.lock().ok().and_then(|guard| *guard)
                            })
                            .map(|at| at.elapsed() < Duration::from_millis(500))
                            .unwrap_or(false);
                        if just_shown {
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
            commands::request_accessibility_permission,
            commands::screenshot_permission_status,
            commands::request_screenshot_permission,
            commands::open_screen_recording_settings,
            commands::open_logs_dir,
            commands::check_update,
            commands::open_release_page,
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

#[cfg(test)]
mod tests {
    use super::adaptive_window_size;
    use tauri::PhysicalSize;

    fn assert_size(work: (u32, u32), scale: f64, expected: (f64, f64)) {
        let size = adaptive_window_size(PhysicalSize::new(work.0, work.1), scale);
        assert!(
            (size.width - expected.0).abs() < 0.5 && (size.height - expected.1).abs() < 0.5,
            "work {:?} @ {scale} gave {}x{}, expected {}x{}",
            work,
            size.width,
            size.height,
            expected.0,
            expected.1
        );
    }

    #[test]
    fn keeps_the_base_size_on_1080p() {
        assert_size((1920, 1040), 1.0, (880.0, 660.0));
    }

    #[test]
    fn grows_on_4k_at_100_percent() {
        assert_size((3840, 2120), 1.0, (1200.0, 900.0));
    }

    #[test]
    fn grows_on_4k_at_150_percent() {
        // 2560x1413 logical work area: 62% -> 876 high, 4:3 -> 1168 wide.
        assert_size((3840, 2120), 1.5, (1168.36, 876.27));
    }

    #[test]
    fn stays_inside_small_work_areas() {
        assert_size((1366, 728), 1.0, (880.0, 660.0));
        assert_size((800, 480), 1.0, (768.0, 520.0));
    }

    #[test]
    fn falls_back_when_the_work_area_is_missing() {
        assert_size((0, 0), 1.0, (880.0, 660.0));
    }
}
