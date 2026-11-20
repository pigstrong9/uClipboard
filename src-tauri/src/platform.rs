#[cfg(target_os = "windows")]
pub fn foreground_target() -> Option<isize> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId, IsWindowVisible,
    };

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
        IsWindow, IsWindowVisible, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };

    unsafe {
        let target = hwnd as windows_sys::Win32::Foundation::HWND;
        if IsWindow(target) == 0 || IsWindowVisible(target) == 0 {
            return false;
        }
        ShowWindow(target, SW_RESTORE);
        SetForegroundWindow(target) != 0
    }
}

#[cfg(target_os = "windows")]
pub fn foreground_app_name() -> Option<String> {
    use std::path::Path;

    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
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

#[cfg(not(target_os = "windows"))]
pub fn foreground_target() -> Option<isize> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn focus_window(_hwnd: isize) -> bool {
    false
}

#[cfg(target_os = "windows")]
pub fn clipboard_sequence() -> Option<u32> {
    use windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber;
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
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};

    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let format = unsafe { RegisterClipboardFormatW(wide.as_ptr()) };
    if format == 0 || unsafe { IsClipboardFormatAvailable(format) } == 0 {
        return None;
    }
    let mut opened = false;
    for _ in 0..10 {
        if unsafe { OpenClipboard(std::ptr::null_mut()) } != 0 {
            opened = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if !opened {
        return None;
    }
    let value = unsafe {
        let handle = GetClipboardData(format);
        if handle.is_null() {
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
    };
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
pub fn clipboard_sequence() -> Option<u32> {
    None
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

    let Ok(image) = image::load_from_memory(png) else {
        return false;
    };
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    if width == 0 || height == 0 {
        return false;
    }
    let pixel_bytes = width as usize * height as usize * 4;
    let total = 40 + pixel_bytes;
    let bitmap_mem = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total) };
    if bitmap_mem.is_null() {
        return false;
    }
    let bitmap_ptr = unsafe { GlobalLock(bitmap_mem) };
    if bitmap_ptr.is_null() {
        unsafe {
            let _ = GlobalFree(bitmap_mem);
        }
        return false;
    }
    let bitmap = unsafe { std::slice::from_raw_parts_mut(bitmap_ptr as *mut u8, total) };
    fn write_u32(buffer: &mut [u8], offset: usize, value: u32) {
        buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    write_u32(bitmap, 0, 40);
    write_u32(bitmap, 4, width);
    write_u32(bitmap, 8, height);
    bitmap[12..14].copy_from_slice(&1u16.to_le_bytes());
    bitmap[14..16].copy_from_slice(&32u16.to_le_bytes());
    write_u32(bitmap, 20, pixel_bytes as u32);
    write_u32(bitmap, 24, 2835);
    write_u32(bitmap, 28, 2835);

    let source = rgba.as_raw();
    for y in 0..height as usize {
        let source_row = &source[y * width as usize * 4..(y + 1) * width as usize * 4];
        let target_start = 40 + (height as usize - 1 - y) * width as usize * 4;
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
    unsafe {
        let _ = GlobalUnlock(bitmap_mem);
    }

    let mut opened = false;
    for _ in 0..10 {
        if unsafe { OpenClipboard(std::ptr::null_mut()) } != 0 {
            opened = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if !opened {
        unsafe {
            let _ = GlobalFree(bitmap_mem);
        }
        return false;
    }
    unsafe {
        let _ = EmptyClipboard();
    }
    let bitmap_ok = !unsafe { SetClipboardData(8, bitmap_mem) }.is_null();

    let png_format = {
        let name: Vec<u16> = "PNG".encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { RegisterClipboardFormatW(name.as_ptr()) }
    };
    if png_format != 0 && !png.is_empty() {
        let png_mem = unsafe { GlobalAlloc(GMEM_MOVEABLE, png.len()) };
        if !png_mem.is_null() {
            let png_ptr = unsafe { GlobalLock(png_mem) };
            if png_ptr.is_null() {
                unsafe {
                    let _ = GlobalFree(png_mem);
                }
            } else {
                unsafe {
                    std::ptr::copy_nonoverlapping(png.as_ptr(), png_ptr as *mut u8, png.len());
                    let _ = GlobalUnlock(png_mem);
                }
                if unsafe { SetClipboardData(png_format, png_mem) }.is_null() {
                    unsafe {
                        let _ = GlobalFree(png_mem);
                    }
                }
            }
        }
    }

    unsafe {
        let _ = CloseClipboard();
    }
    if !bitmap_ok {
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
