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

#[cfg(not(target_os = "windows"))]
pub fn foreground_app_name() -> Option<String> {
    None
}
