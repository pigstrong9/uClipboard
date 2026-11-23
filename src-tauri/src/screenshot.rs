use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};
use std::time::Duration;

use image::{ImageFormat, Rgba, RgbaImage};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};

use crate::clipboard::source::{ClipboardSource, SystemClipboard};
use crate::state::AppState;

const WINDOW: &str = "screenshot";
/// Keep a single capture under ~480 MB of RGBA before the PNG is written.
const MAX_SCREEN_PIXELS: u64 = 120_000_000;

pub struct ScreenshotSession {
    path: PathBuf,
    width: u32,
    height: u32,
}

struct Capture {
    path: PathBuf,
    width: u32,
    height: u32,
    origin_x: i32,
    origin_y: i32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotSessionInfo {
    image_path: String,
    width: u32,
    height: u32,
}

pub fn trigger(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(code) = start(app.clone()).await {
            if code != "screenshot.busy" {
                fail(&app, &code);
            }
        }
    });
}

async fn start(app: AppHandle) -> Result<(), String> {
    if app.get_webview_window(WINDOW).is_some() {
        return Err("screenshot.busy".to_string());
    }
    {
        let state = app.state::<AppState>();
        let guard = state
            .screenshot
            .lock()
            .map_err(|_| "screenshot.busy".to_string())?;
        if guard.is_some() {
            return Err("screenshot.busy".to_string());
        }
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    if let Some(window) = app.get_webview_window("transfer") {
        let _ = window.hide();
    }

    let media_dir = app.state::<AppState>().media_dir.clone();
    let capture = tauri::async_runtime::spawn_blocking(move || {
        // Give the compositor a moment to actually remove our windows from the screen.
        std::thread::sleep(Duration::from_millis(200));
        capture_virtual_screen(&media_dir)
    })
    .await
    .map_err(|error| {
        log::error!("screenshot task failed: {error}");
        "screenshot.capture_failed".to_string()
    })??;

    {
        let state = app.state::<AppState>();
        let mut guard = state
            .screenshot
            .lock()
            .map_err(|_| "screenshot.busy".to_string())?;
        *guard = Some(ScreenshotSession {
            path: capture.path.clone(),
            width: capture.width,
            height: capture.height,
        });
    }

    let built = WebviewWindowBuilder::new(&app, WINDOW, WebviewUrl::App("screenshot.html".into()))
        .title("Screenshot")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .focused(false)
        .build();

    let window = match built {
        Ok(window) => window,
        Err(error) => {
            log::error!("build screenshot window failed: {error}");
            cleanup(&app);
            return Err("screenshot.window_failed".to_string());
        }
    };

    if let Err(error) =
        window.set_position(PhysicalPosition::new(capture.origin_x, capture.origin_y))
    {
        log::error!("position screenshot window failed: {error}");
        let _ = window.destroy();
        cleanup(&app);
        return Err("screenshot.window_failed".to_string());
    }
    if let Err(error) = window.set_size(PhysicalSize::new(capture.width, capture.height)) {
        log::error!("size screenshot window failed: {error}");
        let _ = window.destroy();
        cleanup(&app);
        return Err("screenshot.window_failed".to_string());
    }

    let handle = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            cleanup(&handle);
        }
    });

    // Keep the overlay hidden until the frontend confirms the capture image has
    // loaded. Showing it earlier paints a black fullscreen surface (body background
    // plus dim layer) ahead of the image, which reads as a screen flash to black.
    // A bounded fallback still reveals the window so Esc stays usable if the image
    // is slow or the webview never reports back.
    let fallback = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1500));
        if let Some(window) = fallback.get_webview_window(WINDOW) {
            if window.is_visible().unwrap_or(false) {
                return;
            }
            let _ = window.show();
            let _ = window.set_focus();
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn start_region_capture(app: AppHandle) -> Result<(), String> {
    start(app).await
}

#[tauri::command]
pub fn screenshot_session(app: AppHandle) -> Result<ScreenshotSessionInfo, String> {
    let state = app.state::<AppState>();
    let guard = state
        .screenshot
        .lock()
        .map_err(|_| "screenshot.session_missing".to_string())?;
    let session = guard
        .as_ref()
        .ok_or_else(|| "screenshot.session_missing".to_string())?;
    Ok(ScreenshotSessionInfo {
        image_path: session.path.to_string_lossy().into_owned(),
        width: session.width,
        height: session.height,
    })
}

#[tauri::command]
pub fn screenshot_ready(app: AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub fn cancel_region_capture(app: AppHandle) {
    cleanup(&app);
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.close();
    }
}

#[tauri::command]
pub async fn capture_region(
    app: AppHandle,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let session = {
        let state = app.state::<AppState>();
        let mut guard = state
            .screenshot
            .lock()
            .map_err(|_| "screenshot.session_missing".to_string())?;
        guard
            .take()
            .ok_or_else(|| "screenshot.session_missing".to_string())?
    };

    let path = session.path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&session.path).map_err(|error| {
            log::error!("read screenshot session failed: {error}");
            "screenshot.capture_failed".to_string()
        })?;
        let image = crate::media::decode_image_limited(&bytes).map_err(|error| {
            log::error!("decode screenshot failed: {error}");
            "screenshot.capture_failed".to_string()
        })?;
        let (x, y, width, height) = clamp_rect(image.width(), image.height(), x, y, width, height)
            .ok_or_else(|| "screenshot.session_missing".to_string())?;
        let cropped = image.crop_imm(x, y, width, height);
        let mut png = Cursor::new(Vec::new());
        cropped
            .write_to(&mut png, ImageFormat::Png)
            .map_err(|error| {
                log::error!("encode screenshot crop failed: {error}");
                "screenshot.capture_failed".to_string()
            })?;
        if !SystemClipboard.write_image_png(png.get_ref()) {
            return Err("screenshot.write_failed".to_string());
        }
        Ok(())
    })
    .await
    .map_err(|error| {
        log::error!("screenshot crop task failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;

    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.close();
    }
    remove_capture(path);

    match result {
        Ok(()) => Ok(()),
        Err(code) => {
            fail(&app, &code);
            Ok(())
        }
    }
}

fn fail(app: &AppHandle, code: &str) {
    log::error!("screenshot failed: {code}");
    crate::show_main(app);
    let _ = app.emit("screenshot-error", code.to_string());
}

fn cleanup(app: &AppHandle) {
    let state = app.state::<AppState>();
    let session = state
        .screenshot
        .lock()
        .ok()
        .and_then(|mut guard| guard.take());
    if let Some(session) = session {
        remove_capture(session.path);
    }
}

/// The overlay loads the capture through the asset protocol, so Windows can still hold
/// the file open for a moment after the window closes. Retry briefly instead of leaking
/// the temporary PNG.
fn remove_capture(path: PathBuf) {
    if std::fs::remove_file(&path).is_ok() || !path.exists() {
        return;
    }
    std::thread::spawn(move || {
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(100));
            if std::fs::remove_file(&path).is_ok() || !path.exists() {
                return;
            }
        }
        log::warn!("screenshot temp file cleanup failed: {}", path.display());
    });
}

fn capture_virtual_screen(media_dir: &Path) -> Result<Capture, String> {
    let monitors = xcap::Monitor::all().map_err(|error| {
        log::error!("enumerate monitors failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;
    if monitors.is_empty() {
        return Err("screenshot.capture_failed".to_string());
    }

    let mut shots: Vec<(i32, i32, RgbaImage)> = Vec::new();
    let mut total_pixels = 0u64;
    for monitor in monitors {
        let scale = monitor.scale_factor().unwrap_or(1.0);
        let x = physical_origin(
            monitor.x().map_err(|error| {
                log::error!("monitor x failed: {error}");
                "screenshot.capture_failed".to_string()
            })?,
            scale,
        );
        let y = physical_origin(
            monitor.y().map_err(|error| {
                log::error!("monitor y failed: {error}");
                "screenshot.capture_failed".to_string()
            })?,
            scale,
        );
        let image = monitor.capture_image().map_err(|error| {
            log::error!("capture monitor failed: {error}");
            "screenshot.capture_failed".to_string()
        })?;
        total_pixels =
            total_pixels.saturating_add(u64::from(image.width()) * u64::from(image.height()));
        if total_pixels > MAX_SCREEN_PIXELS {
            return Err("screenshot.too_large".to_string());
        }
        if image.width() > 0 && image.height() > 0 {
            shots.push((x, y, image));
        }
    }
    if shots.is_empty() {
        return Err("screenshot.capture_failed".to_string());
    }

    let rects: Vec<(i32, i32, u32, u32)> = shots
        .iter()
        .map(|(x, y, image)| (*x, *y, image.width(), image.height()))
        .collect();
    let (origin_x, origin_y, width, height) =
        canvas_bounds(&rects).ok_or_else(|| "screenshot.too_large".to_string())?;

    let mut canvas = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 255]));
    for (x, y, image) in shots {
        image::imageops::overlay(
            &mut canvas,
            &image,
            i64::from(x - origin_x),
            i64::from(y - origin_y),
        );
    }

    let images = media_dir.join("images");
    std::fs::create_dir_all(&images).map_err(|error| {
        log::error!("create screenshot dir failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;
    let path = images.join(format!("shot_{}.png", uuid::Uuid::now_v7()));
    let file = std::fs::File::create(&path).map_err(|error| {
        log::error!("create screenshot file failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;
    image::DynamicImage::ImageRgba8(canvas)
        .write_to(&mut BufWriter::new(file), ImageFormat::Png)
        .map_err(|error| {
            let _ = std::fs::remove_file(&path);
            log::error!("write screenshot png failed: {error}");
            "screenshot.capture_failed".to_string()
        })?;

    Ok(Capture {
        path,
        width,
        height,
        origin_x,
        origin_y,
    })
}

#[cfg(target_os = "macos")]
fn physical_origin(value: i32, scale: f32) -> i32 {
    (value as f32 * scale).round() as i32
}

#[cfg(not(target_os = "macos"))]
fn physical_origin(value: i32, _scale: f32) -> i32 {
    value
}

pub(crate) fn canvas_bounds(rects: &[(i32, i32, u32, u32)]) -> Option<(i32, i32, u32, u32)> {
    let mut iter = rects.iter();
    let (first_x, first_y, first_w, first_h) = *iter.next()?;
    let mut min_x = first_x;
    let mut min_y = first_y;
    let mut max_x = first_x.checked_add(first_w as i32)?;
    let mut max_y = first_y.checked_add(first_h as i32)?;
    for (x, y, width, height) in iter {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(x.checked_add(*width as i32)?);
        max_y = max_y.max(y.checked_add(*height as i32)?);
    }
    Some((min_x, min_y, (max_x - min_x) as u32, (max_y - min_y) as u32))
}

pub(crate) fn clamp_rect(
    image_width: u32,
    image_height: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Option<(u32, u32, u32, u32)> {
    if width == 0 || height == 0 || image_width == 0 || image_height == 0 {
        return None;
    }
    let x0 = i64::from(x).max(0);
    let y0 = i64::from(y).max(0);
    let x1 = (i64::from(x) + i64::from(width)).min(i64::from(image_width));
    let y1 = (i64::from(y) + i64::from(height)).min(i64::from(image_height));
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some((x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_include_negative_monitors() {
        let bounds = canvas_bounds(&[(0, 0, 100, 80), (-50, 20, 80, 60)]).unwrap();
        assert_eq!(bounds, (-50, 0, 150, 80));
    }

    #[test]
    fn bounds_handle_empty_and_overflow() {
        assert_eq!(canvas_bounds(&[]), None);
        assert_eq!(canvas_bounds(&[(i32::MAX, 0, 10, 10)]), None);
    }

    #[test]
    fn region_is_clamped_to_the_capture() {
        assert_eq!(clamp_rect(100, 100, -10, -5, 30, 20), Some((0, 0, 20, 15)));
        assert_eq!(clamp_rect(100, 100, 90, 90, 30, 30), Some((90, 90, 10, 10)));
        assert_eq!(clamp_rect(100, 100, 100, 0, 10, 10), None);
        assert_eq!(clamp_rect(100, 100, 0, 0, 0, 10), None);
    }
}
