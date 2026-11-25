use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use image::{ImageFormat, RgbaImage};
use tauri::{
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
#[cfg(not(target_os = "macos"))]
use tauri::{PhysicalPosition, PhysicalSize};

use crate::clipboard::source::{ClipboardSource, SystemClipboard};
use crate::state::AppState;

const WINDOW: &str = "screenshot";
/// Keep a single capture under ~480 MB of RGBA before the PNG is written.
const MAX_SCREEN_PIXELS: u64 = 120_000_000;
/// Whether the screen recording prompt has already been raised this run.
#[cfg(target_os = "macos")]
static SCREEN_CAPTURE_PROMPTED: AtomicBool = AtomicBool::new(false);

/// One overlay window per display, each carrying that display's own capture.
///
/// Every slice keeps its own image instead of being pasted into a shared canvas:
/// displays can have different pixel densities, and a single pixel canvas cannot hold
/// both a 2x and a 1x capture without either overlapping or leaving gaps.
#[derive(Clone)]
pub struct OverlaySlice {
    pub label: String,
    /// This display's capture, in its own native pixel resolution.
    pub image_path: PathBuf,
    pub width: u32,
    pub height: u32,
    /// Placement in the screen units the capture reports: points on macOS, physical
    /// pixels elsewhere.
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub struct ScreenshotSession {
    pub slices: Vec<OverlaySlice>,
    /// Set as soon as this session leaves the state slot, whether it was cancelled,
    /// cropped away or replaced by a later capture. Anything still holding a clone of
    /// the flag — the reveal fallback above all — must then stay out of the way.
    settled: Arc<AtomicBool>,
}

impl ScreenshotSession {
    /// A session that owns the screenshot flow before any pixels exist, so a second
    /// trigger cannot start alongside the first.
    fn placeholder() -> (Self, Arc<AtomicBool>) {
        let settled = Arc::new(AtomicBool::new(false));
        (
            Self {
                slices: Vec::new(),
                settled: Arc::clone(&settled),
            },
            settled,
        )
    }

    fn settle(&self) {
        self.settled.store(true, Ordering::Release);
    }

    fn labels(&self) -> Vec<String> {
        self.slices
            .iter()
            .map(|slice| slice.label.clone())
            .collect()
    }
}

/// Holds the screenshot slot from the moment a capture starts until its overlays exist.
///
/// Claiming the slot up front is what keeps a second trigger out — the session itself is
/// only installed once the pixels are on disk — and dropping the claim without committing
/// gives the slot back, so a failed round cannot leave screenshots busy forever.
struct Claim {
    app: AppHandle,
    settled: Arc<AtomicBool>,
    committed: bool,
}

impl Claim {
    fn acquire(app: &AppHandle) -> Result<Self, String> {
        let state = app.state::<AppState>();
        let mut guard = state
            .screenshot
            .lock()
            .map_err(|_| "screenshot.busy".to_string())?;
        if guard.is_some() {
            return Err("screenshot.busy".to_string());
        }
        let (session, settled) = ScreenshotSession::placeholder();
        *guard = Some(session);
        Ok(Self {
            app: app.clone(),
            settled,
            committed: false,
        })
    }

    fn settled(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.settled)
    }

    /// Installs the captured slices, unless the round was cancelled while they ran.
    /// Returns false when there is nothing to show and the caller must clean up.
    fn commit(&mut self, slices: &[OverlaySlice]) -> bool {
        let state = self.app.state::<AppState>();
        // A poisoned lock still holds the slot we claimed; recover it rather than strand
        // the screenshot in the busy state.
        let mut guard = state
            .screenshot
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if self.settled.load(Ordering::Acquire) {
            return false;
        }
        *guard = Some(ScreenshotSession {
            slices: slices.to_vec(),
            settled: Arc::clone(&self.settled),
        });
        self.committed = true;
        true
    }
}

impl Drop for Claim {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        let Some(state) = self.app.try_state::<AppState>() else {
            return;
        };
        let mut guard = state
            .screenshot
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if guard
            .as_ref()
            .is_some_and(|session| Arc::ptr_eq(&session.settled, &self.settled))
        {
            if let Some(session) = guard.take() {
                session.settle();
            }
        }
    }
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
    // Claim the slot before touching the display. Without this the second trigger that
    // arrives during the capture delay reaches the window build with duplicate labels,
    // and its error path closes the overlays the first round is still using.
    let mut claim = Claim::acquire(&app)?;

    // Without screen recording permission macOS hands back a wallpaper-only image
    // instead of failing, so the overlay would come up with nothing in it.
    #[cfg(target_os = "macos")]
    if crate::platform::screen_capture_trusted() == Some(false) {
        // macOS only surfaces the dialog the first time anyway, and every further press
        // would re-run a prompt that cannot succeed. Settings still asks on demand.
        if SCREEN_CAPTURE_PROMPTED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            crate::platform::request_screen_capture_access();
        }
        return Err("screenshot.permission".to_string());
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    if let Some(window) = app.get_webview_window("transfer") {
        let _ = window.hide();
    }

    let settled = claim.settled();
    let media_dir = app.state::<AppState>().media_dir.clone();
    let capture = tauri::async_runtime::spawn_blocking(move || {
        // The macOS capture path composites the pointer into the image. Hide it before
        // the settle delay so it is gone from every composited frame by capture time,
        // even if the pointer is still moving as the shortcut fires.
        #[cfg(target_os = "macos")]
        let _cursor = CursorGuard::hide();
        // Give the compositor a moment to actually remove our windows from the screen.
        std::thread::sleep(Duration::from_millis(200));
        capture_displays(&media_dir)
    })
    .await
    .map_err(|error| {
        log::error!("screenshot task failed: {error}");
        "screenshot.capture_failed".to_string()
    })??;

    // Capturing is slow enough for the user to cancel in between. If the claim no longer
    // owns the slot, those files belong to a session nobody will ever show.
    if !claim.commit(&capture) {
        log::debug!("screenshot capture discarded: session already settled");
        for slice in &capture {
            remove_capture(slice.image_path.clone());
        }
        return Ok(());
    }
    let slices = capture;

    let labels = slices
        .iter()
        .map(|slice| slice.label.clone())
        .collect::<Vec<_>>();

    for slice in &slices {
        let sized = WebviewWindowBuilder::new(
            &app,
            &slice.label,
            WebviewUrl::App("screenshot.html".into()),
        )
        .title("Screenshot")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .focused(false);
        // Start the webview at the display's own size: a default 800x600 page would map a
        // first selection through the wrong scale until the resize lands. macOS sizes the
        // window in points here and sets the real frame through AppKit when revealing it,
        // other platforms set the physical frame below.
        #[cfg(target_os = "macos")]
        let sized = sized.inner_size(slice.w, slice.h);

        let built = sized.build();

        let window = match built {
            Ok(window) => window,
            Err(error) => {
                log::error!("build screenshot window failed: {error}");
                close_all(&app);
                cleanup(&app);
                return Err("screenshot.window_failed".to_string());
            }
        };

        // macOS places each overlay through AppKit right before showing it, because
        // tao's physical/logical round-trip shifts the frame. Other platforms keep the
        // single canvas-sized window.
        #[cfg(not(target_os = "macos"))]
        {
            let _ = window.set_position(PhysicalPosition::new(slice.x as i32, slice.y as i32));
            let _ = window.set_size(PhysicalSize::new(slice.w as u32, slice.h as u32));
        }

        let handle = app.clone();
        window.on_window_event(move |event| {
            if matches!(event, WindowEvent::Destroyed) {
                cleanup(&handle);
            }
        });
    }

    // Keep the overlays hidden until the frontend confirms the capture image has
    // loaded. Showing it earlier paints a black fullscreen surface (body background
    // plus dim layer) ahead of the image, which reads as a screen flash to black.
    // A bounded fallback still reveals them so Esc stays usable if the image is slow
    // or a webview never reports back.
    let fallback = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1500));
        // A newer capture owns the overlays by now; revealing them from this stale timer
        // would show them before their images are ready.
        if settled.load(Ordering::Acquire) {
            return;
        }
        let handle = fallback.clone();
        let _ = fallback.run_on_main_thread(move || {
            for label in labels {
                let hidden = handle
                    .get_webview_window(&label)
                    .map(|window| !window.is_visible().unwrap_or(false))
                    .unwrap_or(false);
                if hidden {
                    reveal(&handle, &label);
                }
            }
        });
    });
    Ok(())
}

/// Snapshot of the current session's slices.
fn session_slices(app: &AppHandle) -> Vec<OverlaySlice> {
    app.try_state::<AppState>()
        .and_then(|state| {
            state
                .screenshot
                .lock()
                .ok()
                .and_then(|guard| guard.as_ref().map(|session| session.slices.clone()))
        })
        .unwrap_or_default()
}

/// Places and shows one capture overlay. Must run on the main thread.
fn reveal(app: &AppHandle, label: &str) {
    let rect = app.try_state::<AppState>().and_then(|state| {
        state.screenshot.lock().ok().and_then(|guard| {
            guard.as_ref().and_then(|session| {
                session
                    .slices
                    .iter()
                    .find(|slice| slice.label == label)
                    .map(|slice| (slice.x, slice.y, slice.w, slice.h))
            })
        })
    });
    if let Some(window) = app.get_webview_window(label) {
        if let Some((x, y, w, h)) = rect {
            crate::platform::place_overlay(&window, x, y, w, h);
        }
        let _ = window.show();
        let _ = window.set_focus();
        crate::platform::focus_webview(&window);
    }
    crate::platform::activate_app();
}

#[tauri::command]
pub async fn start_region_capture(app: AppHandle) -> Result<(), String> {
    start(app).await
}

#[tauri::command]
pub fn screenshot_session(
    window: WebviewWindow,
    app: AppHandle,
) -> Result<ScreenshotSessionInfo, String> {
    let state = app.state::<AppState>();
    let guard = state
        .screenshot
        .lock()
        .map_err(|_| "screenshot.session_missing".to_string())?;
    let session = guard
        .as_ref()
        .ok_or_else(|| "screenshot.session_missing".to_string())?;
    let slice = session
        .slices
        .iter()
        .find(|slice| slice.label == window.label())
        .ok_or_else(|| "screenshot.session_missing".to_string())?;
    Ok(ScreenshotSessionInfo {
        image_path: slice.image_path.to_string_lossy().into_owned(),
        width: slice.width,
        height: slice.height,
    })
}

#[tauri::command]
pub fn screenshot_ready(window: WebviewWindow, app: AppHandle) {
    let label = window.label().to_string();
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || reveal(&handle, &label));
}

#[tauri::command]
pub fn cancel_region_capture(app: AppHandle) {
    close_all(&app);
    cleanup(&app);
}

#[tauri::command]
pub async fn capture_region(
    window: WebviewWindow,
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
        let session = guard
            .take()
            .ok_or_else(|| "screenshot.session_missing".to_string())?;
        session.settle();
        session
    };
    // The selection can only ever be made inside one overlay window, so it maps into that
    // display's own capture rather than a shared canvas.
    let path = session
        .slices
        .iter()
        .find(|slice| slice.label == window.label())
        .map(|slice| slice.image_path.clone())
        .ok_or_else(|| "screenshot.session_missing".to_string())?;
    let labels = session.labels();
    let paths = session
        .slices
        .iter()
        .map(|slice| slice.image_path.clone())
        .collect::<Vec<_>>();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|error| {
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

    for label in labels {
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.close();
        }
    }
    for path in paths {
        remove_capture(path);
    }

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

fn close_all(app: &AppHandle) {
    for slice in session_slices(app) {
        if let Some(window) = app.get_webview_window(&slice.label) {
            let _ = window.close();
        }
    }
}

fn cleanup(app: &AppHandle) {
    let state = app.state::<AppState>();
    let session = state
        .screenshot
        .lock()
        .ok()
        .and_then(|mut guard| guard.take());
    if let Some(session) = session {
        session.settle();
        for slice in &session.slices {
            remove_capture(slice.image_path.clone());
        }
        for label in session.labels() {
            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.close();
            }
        }
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

/// A display's capture plus its frame in the units the platform reports: points on
/// macOS, physical pixels elsewhere.
struct MonitorShot {
    frame: (i32, i32, u32, u32),
    image: RgbaImage,
}

fn enumerate_monitors() -> Result<Vec<xcap::Monitor>, String> {
    xcap::Monitor::all().map_err(|error| {
        log::error!("enumerate monitors failed: {error}");
        "screenshot.capture_failed".to_string()
    })
}

fn monitor_frame(monitor: &xcap::Monitor) -> Result<(i32, i32, u32, u32), String> {
    let x = monitor.x().map_err(|error| {
        log::error!("monitor x failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;
    let y = monitor.y().map_err(|error| {
        log::error!("monitor y failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;
    let width = monitor.width().map_err(|error| {
        log::error!("monitor width failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;
    let height = monitor.height().map_err(|error| {
        log::error!("monitor height failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;
    Ok((x, y, width, height))
}

fn capture_shots(monitors: &[xcap::Monitor]) -> Result<Vec<MonitorShot>, String> {
    if monitors.is_empty() {
        return Err("screenshot.capture_failed".to_string());
    }
    let mut shots: Vec<MonitorShot> = Vec::new();
    let mut total_pixels = 0u64;
    for monitor in monitors {
        let frame = monitor_frame(monitor)?;
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
            shots.push(MonitorShot { frame, image });
        }
    }
    if shots.is_empty() {
        return Err("screenshot.capture_failed".to_string());
    }
    Ok(shots)
}

/// Overlay labels include the capture round so a new screenshot can never
/// collide with a previous window still being torn down. Window `close()` is
/// asynchronous: the `Destroyed` event that clears the session can arrive after
/// the user has already triggered another capture, and reusing a fixed label
/// then makes `WebviewWindowBuilder::build()` fail with a duplicate-label
/// error ("无法创建截图窗口" in the UI). Unique labels remove that race.
fn slice_label(round: &str, index: usize) -> String {
    if index == 0 {
        format!("{WINDOW}-{round}")
    } else {
        format!("{WINDOW}-{round}-{index}")
    }
}

/// Writes one capture into the media directory. All captures of a round share a uuid so
/// a listing still groups them together. The image is moved in because a Retina capture
/// is far too large to copy on the way to the encoder.
fn write_capture(
    media_dir: &Path,
    round: &str,
    index: usize,
    image: RgbaImage,
) -> Result<PathBuf, String> {
    let images = media_dir.join("images");
    std::fs::create_dir_all(&images).map_err(|error| {
        log::error!("create screenshot dir failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;
    let name = if index == 0 {
        format!("shot_{round}.png")
    } else {
        format!("shot_{round}-{index}.png")
    };
    let path = images.join(name);
    let file = std::fs::File::create(&path).map_err(|error| {
        log::error!("create screenshot file failed: {error}");
        "screenshot.capture_failed".to_string()
    })?;
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut BufWriter::new(file), ImageFormat::Png)
        .map_err(|error| {
            let _ = std::fs::remove_file(&path);
            log::error!("write screenshot png failed: {error}");
            "screenshot.capture_failed".to_string()
        })?;
    Ok(path)
}

/// macOS gives every display its own image and its own overlay window, so nothing is
/// ever resampled and mixed pixel densities cannot overlap into one canvas.
#[cfg(target_os = "macos")]
fn capture_displays(media_dir: &Path) -> Result<Vec<OverlaySlice>, String> {
    let shots = capture_shots(&enumerate_monitors()?)?;
    let round = uuid::Uuid::now_v7().to_string();
    let mut slices: Vec<OverlaySlice> = Vec::with_capacity(shots.len());
    for (index, shot) in shots.into_iter().enumerate() {
        let (x, y, w, h) = shot.frame;
        let (width, height) = (shot.image.width(), shot.image.height());
        let image_path = match write_capture(media_dir, &round, index, shot.image) {
            Ok(path) => path,
            Err(code) => {
                for slice in &slices {
                    let _ = std::fs::remove_file(&slice.image_path);
                }
                return Err(code);
            }
        };
        slices.push(OverlaySlice {
            label: slice_label(&round, index),
            image_path,
            width,
            height,
            x: f64::from(x),
            y: f64::from(y),
            w: f64::from(w),
            h: f64::from(h),
        });
    }
    Ok(slices)
}

/// Elsewhere one window covers the whole virtual screen, so the displays are pasted into
/// one canvas of physical pixels.
#[cfg(not(target_os = "macos"))]
fn capture_displays(media_dir: &Path) -> Result<Vec<OverlaySlice>, String> {
    let shots = capture_shots(&enumerate_monitors()?)?;
    let rects: Vec<(i32, i32, u32, u32)> = shots
        .iter()
        .map(|shot| {
            (
                shot.frame.0,
                shot.frame.1,
                shot.image.width(),
                shot.image.height(),
            )
        })
        .collect();
    let (origin_x, origin_y, width, height) =
        canvas_bounds(&rects).ok_or_else(|| "screenshot.too_large".to_string())?;

    let mut canvas = RgbaImage::from_pixel(width, height, image::Rgba([0, 0, 0, 255]));
    for shot in &shots {
        image::imageops::overlay(
            &mut canvas,
            &shot.image,
            i64::from(shot.frame.0 - origin_x),
            i64::from(shot.frame.1 - origin_y),
        );
    }

    let round = uuid::Uuid::now_v7().to_string();
    let path = write_capture(media_dir, &round, 0, canvas)?;
    Ok(vec![OverlaySlice {
        label: slice_label(&round, 0),
        image_path: path,
        width,
        height,
        x: f64::from(origin_x),
        y: f64::from(origin_y),
        w: f64::from(width),
        h: f64::from(height),
    }])
}

/// Hides the system pointer on every display until dropped. The hide/show calls are
/// reference counted per display, so the guard must pair each one exactly once.
#[cfg(target_os = "macos")]
struct CursorGuard {
    displays: Vec<u32>,
}

#[cfg(target_os = "macos")]
impl CursorGuard {
    fn hide() -> Self {
        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGGetActiveDisplayList(
                max_displays: u32,
                displays: *mut u32,
                count: *mut u32,
            ) -> i32;
            fn CGDisplayHideCursor(display: u32) -> i32;
        }
        let mut displays: Vec<u32> = Vec::new();
        // SAFETY: the list is filled by CoreGraphics into a buffer sized from its own
        // reported count, and hide/show are paired in Drop.
        unsafe {
            let mut count = 0u32;
            if CGGetActiveDisplayList(0, std::ptr::null_mut(), &mut count) != 0 {
                return Self { displays };
            }
            let mut ids = vec![0u32; count as usize];
            let mut actual = count;
            if CGGetActiveDisplayList(count, ids.as_mut_ptr(), &mut actual) != 0 {
                return Self { displays };
            }
            ids.truncate(actual as usize);
            for id in &ids {
                CGDisplayHideCursor(*id);
            }
            displays = ids;
        }
        Self { displays }
    }
}

#[cfg(target_os = "macos")]
impl Drop for CursorGuard {
    fn drop(&mut self) {
        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGDisplayShowCursor(display: u32) -> i32;
        }
        // SAFETY: only decrements the hide counter incremented in `hide`.
        unsafe {
            for id in &self.displays {
                CGDisplayShowCursor(*id);
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
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

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn bounds_include_negative_monitors() {
        let bounds = canvas_bounds(&[(0, 0, 100, 80), (-50, 20, 80, 60)]).unwrap();
        assert_eq!(bounds, (-50, 0, 150, 80));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn bounds_handle_empty_and_overflow() {
        assert_eq!(canvas_bounds(&[]), None);
        assert_eq!(canvas_bounds(&[(i32::MAX, 0, 10, 10)]), None);
    }

    #[test]
    fn every_display_gets_its_own_overlay_label() {
        assert_eq!(slice_label("round-a", 0), "screenshot-round-a");
        assert_eq!(slice_label("round-a", 1), "screenshot-round-a-1");
        assert_eq!(slice_label("round-a", 2), "screenshot-round-a-2");
    }

    #[test]
    fn overlay_labels_are_unique_across_rounds() {
        assert_ne!(slice_label("round-a", 0), slice_label("round-b", 0));
    }

    #[test]
    fn region_is_clamped_to_the_capture() {
        assert_eq!(clamp_rect(100, 100, -10, -5, 30, 20), Some((0, 0, 20, 15)));
        assert_eq!(clamp_rect(100, 100, 90, 90, 30, 30), Some((90, 90, 10, 10)));
        assert_eq!(clamp_rect(100, 100, 100, 0, 10, 10), None);
        assert_eq!(clamp_rect(100, 100, 0, 0, 0, 10), None);
    }
}
