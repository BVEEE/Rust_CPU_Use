use crate::state::WindowInfo;
use anyhow::Result;

/// Enumerate all visible top-level windows
pub fn enumerate_windows() -> Result<Vec<WindowInfo>> {
    // Implementation will use Win32 API - only works on Windows
    #[cfg(target_os = "windows")]
    {
        enumerate_windows_impl()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(vec![])
    }
}

#[cfg(target_os = "windows")]
fn enumerate_windows_impl() -> Result<Vec<WindowInfo>> {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, IsWindowVisible};

    let mut windows = Vec::new();

    unsafe extern "system" fn enum_proc(
        hwnd: HWND,
        lparam: LPARAM,
    ) -> windows::Win32::Foundation::BOOL {
        let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

        unsafe {
            if IsWindowVisible(hwnd).as_bool() {
                let mut text: [u16; 512] = [0; 512];
                let len = GetWindowTextW(hwnd, &mut text);

                if len > 0 {
                    let title = String::from_utf16_lossy(&text[..len as usize]);
                    windows.push(WindowInfo {
                        handle: hwnd.0 as usize as u64,
                        title,
                        is_visible: true,
                        is_focused: false, // Will be updated by get_focused_window
                    });
                }
            }
        }

        true.into()
    }

    unsafe {
        EnumWindows(Some(enum_proc), LPARAM(&mut windows as *mut _ as isize))?;
    }

    Ok(windows)
}

/// Get the currently focused window handle
pub fn get_focused_window() -> Result<Option<u64>> {
    #[cfg(target_os = "windows")]
    {
        get_focused_window_impl()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn get_focused_window_impl() -> Result<Option<u64>> {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            Ok(None)
        } else {
            Ok(Some(hwnd.0 as usize as u64))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_windows_returns_result() {
        let result = enumerate_windows();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_focused_window_returns_result() {
        let result = get_focused_window();
        assert!(result.is_ok());
    }
}
