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

#[cfg(not(target_os = "windows"))]
pub fn foreground_target() -> Option<isize> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn focus_window(_hwnd: isize) -> bool {
    false
}
