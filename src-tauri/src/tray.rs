use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::commands;
use crate::state::AppState;

fn labels(app: &AppHandle) -> (String, String, String, String, String) {
    let language = app
        .try_state::<AppState>()
        .and_then(|state| {
            state
                .settings
                .lock()
                .ok()
                .map(|settings| settings.language.clone())
        })
        .unwrap_or_else(|| "system".to_string());
    if language == "en-US" {
        (
            "Show uClipboard".to_string(),
            "Pause / Resume".to_string(),
            "Clear History".to_string(),
            "Settings".to_string(),
            "Quit".to_string(),
        )
    } else {
        (
            "显示主窗口".to_string(),
            "暂停 / 恢复监控".to_string(),
            "清空历史".to_string(),
            "设置".to_string(),
            "退出".to_string(),
        )
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let (show_label, pause_label, clear_label, settings_label, quit_label) = labels(app);
    let show = MenuItem::with_id(app, "show", show_label, true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", pause_label, true, None::<&str>)?;
    let clear = MenuItem::with_id(app, "clear", clear_label, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", settings_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    Menu::with_items(app, &[&show, &pause, &clear, &separator, &settings, &quit])
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let Some(icon) = app.default_window_icon().cloned() else {
        return Ok(());
    };
    let menu = build_menu(app)?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("uClipboard")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => crate::show_main(app),
            "pause" => match commands::toggle_monitoring(app.clone()) {
                Ok(paused) => {
                    let key = if paused {
                        "monitorPausedToast"
                    } else {
                        "monitorResumedToast"
                    };
                    let _ = app.emit("toast-key", key);
                }
                Err(error) => {
                    let _ = app.emit("toast-message", error);
                }
            },
            "clear" => {
                crate::show_main(app);
                let _ = app.emit("request-clear", ());
            }
            "settings" => {
                crate::show_main(app);
                let _ = app.emit("open-settings", ());
            }
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
                crate::show_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn refresh(app: &AppHandle) -> tauri::Result<()> {
    let Some(tray) = app.tray_by_id("main") else {
        return Ok(());
    };
    tray.set_menu(Some(build_menu(app)?))
}
