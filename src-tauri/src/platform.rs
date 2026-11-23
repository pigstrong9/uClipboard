#[cfg(target_os = "windows")]
pub fn system_language() -> &'static str {
    use windows_sys::Win32::Globalization::GetSystemPreferredUILanguages;

    let mut count = 0u32;
    let mut chars = 0u32;
    // SAFETY: a null buffer with zero capacity is the documented way to ask for the sizes.
    let probed =
        unsafe { GetSystemPreferredUILanguages(0, &mut count, std::ptr::null_mut(), &mut chars) };
    if probed == 0 || count == 0 || chars == 0 {
        return "en-US";
    }
    let mut buffer = vec![0u16; chars as usize];
    // SAFETY: the buffer has at least the capacity the first call reported.
    let ok =
        unsafe { GetSystemPreferredUILanguages(0, &mut count, buffer.as_mut_ptr(), &mut chars) };
    if ok == 0 {
        return "en-US";
    }
    let written = (chars as usize).min(buffer.len());
    // The buffer is a run of NUL-terminated BCP-47 tags, so the first element wins.
    let preferred = String::from_utf16_lossy(&buffer[..written])
        .split('\0')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if preferred.starts_with("zh") {
        "zh-CN"
    } else {
        "en-US"
    }
}

#[cfg(not(target_os = "windows"))]
pub fn system_language() -> &'static str {
    let locale = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    if locale.starts_with("zh") {
        "zh-CN"
    } else {
        "en-US"
    }
}

#[cfg(target_os = "windows")]
pub fn foreground_target() -> Option<isize> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId, IsWindowVisible,
    };

    // SAFETY: every call here takes window handles obtained below and only reads state;
    // a null handle is rejected before use.
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() || IsWindowVisible(hwnd) == 0 {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == std::process::id() {
            return None;
        }
        Some(hwnd as isize)
    }
}

#[cfg(target_os = "windows")]
pub fn focus_window(hwnd: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        IsIconic, IsWindow, IsWindowVisible, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };

    // SAFETY: `hwnd` came from foreground_target and is revalidated by IsWindow before use,
    // which is what makes a handle for a window closed since then harmless.
    unsafe {
        let target = hwnd as windows_sys::Win32::Foundation::HWND;
        if IsWindow(target) == 0 || IsWindowVisible(target) == 0 {
            return false;
        }
        // SW_RESTORE also resets a maximized window to its restored size, so only call it
        // when the target is actually minimized; otherwise just raise it.
        if IsIconic(target) != 0 {
            ShowWindow(target, SW_RESTORE);
        }
        SetForegroundWindow(target) != 0
    }
}

#[cfg(target_os = "windows")]
fn process_name_for_window(hwnd: windows_sys::Win32::Foundation::HWND) -> Option<String> {
    use std::path::Path;

    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    // SAFETY: the buffer stays alive and is sized by `buffer.len()`, which is also the
    // capacity handed to the API; `size` comes back as the number of UTF-16 units written,
    // so slicing by it stays in bounds. The process handle is closed on every path.
    unsafe {
        if hwnd.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 || pid == std::process::id() {
            return None;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut buffer = vec![0u16; 2048];
        let mut size = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size);
        let _ = CloseHandle(process);
        if ok == 0 || size == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
    }
}

#[cfg(target_os = "windows")]
pub fn foreground_app_name() -> Option<String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    // SAFETY: takes no arguments and returns either null or a live window handle, which
    // `process_name_for_window` validates before use.
    let hwnd = unsafe { GetForegroundWindow() };
    process_name_for_window(hwnd)
}

/// Process that placed the current clipboard content, which is a better ignore-list
/// signal than whatever happens to be focused when the poll runs.
#[cfg(target_os = "windows")]
pub fn clipboard_owner_app_name() -> Option<String> {
    use windows_sys::Win32::System::DataExchange::GetClipboardOwner;
    // SAFETY: returns the clipboard owner window (possibly null); the handle is only read.
    let owner = unsafe { GetClipboardOwner() };
    process_name_for_window(owner).or_else(foreground_app_name)
}

#[cfg(not(target_os = "windows"))]
pub fn foreground_target() -> Option<isize> {
    None
}

#[cfg(target_os = "windows")]
pub fn clipboard_sequence() -> Option<u32> {
    use windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber;
    // SAFETY: takes no arguments and reads a process-wide counter.
    let sequence = unsafe { GetClipboardSequenceNumber() };
    if sequence == 0 {
        None
    } else {
        Some(sequence)
    }
}

#[cfg(target_os = "windows")]
pub fn clipboard_format_dword(name: &str) -> Option<u32> {
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
        RegisterClipboardFormatW,
    };
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `wide` is a NUL-terminated UTF-16 buffer that outlives the call, and
    // RegisterClipboardFormatW only reads it.
    let format = unsafe { RegisterClipboardFormatW(wide.as_ptr()) };
    // SAFETY: querying a registered format id needs no clipboard access or shared state.
    if format == 0 || unsafe { IsClipboardFormatAvailable(format) } == 0 {
        return None;
    }
    let mut opened = false;
    for _ in 0..10 {
        // SAFETY: a null owner window is valid; OpenClipboard only reads the argument.
        if unsafe { OpenClipboard(std::ptr::null_mut()) } != 0 {
            opened = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if !opened {
        return None;
    }
    // SAFETY: reads clipboard data; the invariants are documented at the point of use below.
    let value = unsafe {
        let handle = GetClipboardData(format);
        if handle.is_null() {
            None
        } else {
            // SAFETY: the payload is a DWORD only by convention — any local process can
            // register data under this name — so GlobalSize is checked before the read,
            // and GlobalLock/GlobalUnlock are paired on the one path that uses the pointer.
            if GlobalSize(handle) < std::mem::size_of::<u32>() {
                None
            } else {
                let pointer = GlobalLock(handle);
                if pointer.is_null() {
                    None
                } else {
                    let value = std::ptr::read_unaligned(pointer as *const u32);
                    let _ = GlobalUnlock(handle);
                    Some(value)
                }
            }
        }
    };
    // SAFETY: the clipboard was opened above and is not held across any await or thread.
    unsafe {
        let _ = CloseClipboard();
    }
    value
}

#[cfg(not(target_os = "windows"))]
pub fn foreground_app_name() -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn clipboard_owner_app_name() -> Option<String> {
    foreground_app_name()
}

/// A revision that changes whenever the system clipboard changes.
///
/// Windows has `GetClipboardSequenceNumber`; elsewhere clipboard-rs already owns the
/// platform watcher (NSPasteboard `changeCount` on macOS, XFixes events on X11), so the
/// watcher only increments a counter instead of decoding content on every poll.
#[cfg(not(target_os = "windows"))]
pub fn clipboard_sequence() -> Option<u32> {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::{Arc, OnceLock};
    use std::time::Duration;

    use clipboard_rs::{ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext};

    struct ChangeCounter(Arc<AtomicU32>);

    impl ClipboardHandler for ChangeCounter {
        fn on_clipboard_change(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    // `None` means the watcher could not be created; the monitor then falls back to
    // reading every poll instead of stalling with a counter that never advances.
    static WATCHER: OnceLock<Option<Arc<AtomicU32>>> = OnceLock::new();
    static WATCHER_FAILED: AtomicBool = AtomicBool::new(false);
    let counter = WATCHER.get_or_init(|| {
        let counter = Arc::new(AtomicU32::new(0));
        let mut watcher =
            ClipboardWatcherContext::new_with_interval(Duration::from_millis(150)).ok()?;
        watcher.add_handler(ChangeCounter(Arc::clone(&counter)));
        std::thread::spawn(move || {
            // `start_watch` blocks for the life of the process. Returning (or panicking,
            // e.g. when XFixes is missing) means notifications are gone, so report the
            // watcher as failed and let the monitor go back to polling.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                watcher.start_watch();
            }));
            WATCHER_FAILED.store(true, Ordering::Relaxed);
        });
        Some(counter)
    });
    if WATCHER_FAILED.load(Ordering::Relaxed) {
        return None;
    }
    counter
        .as_ref()
        .map(|counter| counter.load(Ordering::Relaxed))
}

#[cfg(not(target_os = "windows"))]
pub fn clipboard_format_dword(_name: &str) -> Option<u32> {
    None
}

#[cfg(target_os = "windows")]
pub fn set_clipboard_image_png(png: &[u8]) -> bool {
    use windows_sys::Win32::Foundation::GlobalFree;
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
    };
    use windows_sys::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT,
    };

    const HEADER_SIZE: usize = 40;
    const CF_BITMAP: u32 = 8;
    const PNG_FORMAT_NAME: &str = "PNG";
    // 2835 px/metre is the value Windows itself writes for 72 dpi bitmaps.
    const PIXELS_PER_METRE: u32 = 2835;

    let Ok(image) = crate::media::decode_image_limited(png) else {
        return false;
    };
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    if width == 0 || height == 0 {
        return false;
    }
    let pixel_bytes = width as usize * height as usize * 4;
    // The header stores the pixel length as u32; reject sizes that would truncate it.
    if pixel_bytes > u32::MAX as usize - HEADER_SIZE {
        return false;
    }
    let total = HEADER_SIZE + pixel_bytes;
    // SAFETY: a plain allocation request; the null check below covers failure.
    let bitmap_mem = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total) };
    if bitmap_mem.is_null() {
        return false;
    }
    // SAFETY: locking a block we just allocated; the null check covers a failed lock.
    let bitmap_ptr = unsafe { GlobalLock(bitmap_mem) };
    if bitmap_ptr.is_null() {
        // SAFETY: bitmap_mem came from GlobalAlloc and has not been handed to the clipboard.
        unsafe {
            let _ = GlobalFree(bitmap_mem);
        }
        return false;
    }
    // SAFETY: the block was allocated with `total` bytes and is locked for our exclusive use.
    let bitmap = unsafe { std::slice::from_raw_parts_mut(bitmap_ptr as *mut u8, total) };
    fn write_u32(buffer: &mut [u8], offset: usize, value: u32) {
        buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    write_u32(bitmap, 0, HEADER_SIZE as u32);
    write_u32(bitmap, 4, width);
    write_u32(bitmap, 8, height);
    bitmap[12..14].copy_from_slice(&1u16.to_le_bytes());
    bitmap[14..16].copy_from_slice(&32u16.to_le_bytes());
    write_u32(bitmap, 20, pixel_bytes as u32);
    write_u32(bitmap, 24, PIXELS_PER_METRE);
    write_u32(bitmap, 28, PIXELS_PER_METRE);

    let source = rgba.as_raw();
    for y in 0..height as usize {
        let source_row = &source[y * width as usize * 4..(y + 1) * width as usize * 4];
        let target_start = HEADER_SIZE + (height as usize - 1 - y) * width as usize * 4;
        let target_row = &mut bitmap[target_start..target_start + width as usize * 4];
        for x in 0..width as usize {
            let pixel = &source_row[x * 4..x * 4 + 4];
            let alpha = pixel[3] as u32;
            let target = &mut target_row[x * 4..x * 4 + 4];
            target[0] = ((pixel[2] as u32 * alpha + 255 * (255 - alpha)) / 255) as u8;
            target[1] = ((pixel[1] as u32 * alpha + 255 * (255 - alpha)) / 255) as u8;
            target[2] = ((pixel[0] as u32 * alpha + 255 * (255 - alpha)) / 255) as u8;
            target[3] = 255;
        }
    }
    // SAFETY: matches the GlobalLock taken above, before any ownership transfer.
    unsafe {
        let _ = GlobalUnlock(bitmap_mem);
    }

    let mut opened = false;
    for _ in 0..10 {
        // SAFETY: a null owner window is valid.
        if unsafe { OpenClipboard(std::ptr::null_mut()) } != 0 {
            opened = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if !opened {
        // SAFETY: the block was never handed to the clipboard, so we still own it.
        unsafe {
            let _ = GlobalFree(bitmap_mem);
        }
        return false;
    }
    // SAFETY: the clipboard is open on this thread and closed again below.
    unsafe {
        let _ = EmptyClipboard();
    }
    // SAFETY: the clipboard is open and bitmap_mem is an unlocked global block. On success
    // SetClipboardData consumes the block, so only the failure path may still free it.
    let bitmap_ok = !unsafe { SetClipboardData(CF_BITMAP, bitmap_mem) }.is_null();

    let png_format = {
        let name: Vec<u16> = PNG_FORMAT_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: `name` is NUL-terminated and lives until RegisterClipboardFormatW returns.
        unsafe { RegisterClipboardFormatW(name.as_ptr()) }
    };
    if png_format != 0 && !png.is_empty() {
        // SAFETY: png is non-empty here, so the requested size is never zero.
        let png_mem = unsafe { GlobalAlloc(GMEM_MOVEABLE, png.len()) };
        if !png_mem.is_null() {
            // SAFETY: locking a block we just allocated.
            let png_ptr = unsafe { GlobalLock(png_mem) };
            if png_ptr.is_null() {
                // SAFETY: still owned, never passed to the clipboard.
                unsafe {
                    let _ = GlobalFree(png_mem);
                }
            } else {
                // SAFETY: both buffers are `png.len()` bytes and do not overlap.
                unsafe {
                    std::ptr::copy_nonoverlapping(png.as_ptr(), png_ptr as *mut u8, png.len());
                    let _ = GlobalUnlock(png_mem);
                }
                // SAFETY: clipboard still open; a null return means the block was not consumed.
                if unsafe { SetClipboardData(png_format, png_mem) }.is_null() {
                    // SAFETY: the transfer failed, so this block is still ours to free.
                    unsafe {
                        let _ = GlobalFree(png_mem);
                    }
                }
            }
        }
    }

    // SAFETY: paired with the OpenClipboard above.
    unsafe {
        let _ = CloseClipboard();
    }
    if !bitmap_ok {
        // SAFETY: SetClipboardData failed, so bitmap_mem was not consumed by the system.
        unsafe {
            let _ = GlobalFree(bitmap_mem);
        }
    }
    bitmap_ok
}

#[cfg(target_os = "windows")]
pub fn send_paste_keys() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
    };

    fn key(vk: u16, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    let size = std::mem::size_of::<INPUT>() as i32;
    // SAFETY: each array outlives its SendInput call, the count matches the array length,
    // and `size` is the real size_of::<INPUT>() the API requires.
    unsafe {
        let control_down = [key(VK_CONTROL, false)];
        if SendInput(1, control_down.as_ptr(), size) != 1 {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        let v_press = [key(VK_V, false), key(VK_V, true)];
        let v_ok = SendInput(2, v_press.as_ptr(), size) == 2;
        std::thread::sleep(std::time::Duration::from_millis(20));
        // Always release Ctrl, otherwise the target app keeps the modifier stuck.
        let control_up = [key(VK_CONTROL, true)];
        let released = SendInput(1, control_up.as_ptr(), size) == 1;
        v_ok && released
    }
}

/// macOS synthesizes `Cmd+V` through the accessibility API, which the user must allow
/// explicitly. `None` means the platform has no such permission model.
#[cfg(target_os = "macos")]
pub fn accessibility_trusted() -> Option<bool> {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> u8;
    }
    // SAFETY: the function takes no arguments and only reads the process trust state.
    Some(unsafe { AXIsProcessTrusted() != 0 })
}

#[cfg(not(target_os = "macos"))]
pub fn accessibility_trusted() -> Option<bool> {
    None
}

#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> bool {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .is_ok()
}

#[cfg(not(target_os = "macos"))]
pub fn open_accessibility_settings() -> bool {
    false
}

/// Opens a directory in the platform file manager.
pub fn open_path(path: &std::path::Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .is_ok()
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn().is_ok()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .is_ok()
    }
}

/// Deactivates the whole application (not just a window) before synthesizing Cmd+V, so
/// the keystroke goes to the app the user was working in. Must run on the main thread.
#[cfg(target_os = "macos")]
pub fn hide_app() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;
    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        app.hide(None);
    }
}

/// Brings the application back after a transfer-window paste.
#[cfg(target_os = "macos")]
pub fn show_app() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;
    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        app.unhide(None);
        app.activate();
    }
}
