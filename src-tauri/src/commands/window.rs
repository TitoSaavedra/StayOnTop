#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[cfg(target_os = "windows")]
mod win {
    use super::WindowRect;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, GetWindowRect, IsWindow, SetLayeredWindowAttributes, SetWindowLongW,
        SetWindowPos, GWL_EXSTYLE, HWND_NOTOPMOST, HWND_TOPMOST, LWA_ALPHA, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_EX_LAYERED, WS_EX_TRANSPARENT,
    };

    fn hwnd(raw: isize) -> HWND {
        HWND(raw as *mut _)
    }

    pub fn pin(hwnd_raw: isize, opacity: f32, click_through: bool) -> Result<(), String> {
        unsafe {
            let h = hwnd(hwnd_raw);

            SetWindowPos(h, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE)
                .map_err(|e| format!("SetWindowPos failed: {e}"))?;

            let mut ex_style = GetWindowLongW(h, GWL_EXSTYLE) as u32;
            ex_style |= WS_EX_LAYERED.0;
            if click_through {
                ex_style |= WS_EX_TRANSPARENT.0;
            } else {
                ex_style &= !WS_EX_TRANSPARENT.0;
            }
            SetWindowLongW(h, GWL_EXSTYLE, ex_style as i32);

            let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
            SetLayeredWindowAttributes(h, windows::Win32::Foundation::COLORREF(0), alpha, LWA_ALPHA)
                .map_err(|e| format!("SetLayeredWindowAttributes failed: {e}"))?;
        }
        Ok(())
    }

    pub fn unpin(hwnd_raw: isize) -> Result<(), String> {
        unsafe {
            let h = hwnd(hwnd_raw);

            SetWindowPos(h, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE)
                .map_err(|e| format!("SetWindowPos (unpin) failed: {e}"))?;

            let ex_style = GetWindowLongW(h, GWL_EXSTYLE) as u32;
            let cleaned = ex_style & !(WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0);
            SetWindowLongW(h, GWL_EXSTYLE, cleaned as i32);
        }
        Ok(())
    }

    pub fn set_opacity(hwnd_raw: isize, opacity: f32) -> Result<(), String> {
        unsafe {
            let h = hwnd(hwnd_raw);
            let ex_style = GetWindowLongW(h, GWL_EXSTYLE) as u32;
            if ex_style & WS_EX_LAYERED.0 == 0 {
                SetWindowLongW(h, GWL_EXSTYLE, (ex_style | WS_EX_LAYERED.0) as i32);
            }
            let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
            SetLayeredWindowAttributes(h, windows::Win32::Foundation::COLORREF(0), alpha, LWA_ALPHA)
                .map_err(|e| format!("SetLayeredWindowAttributes failed: {e}"))?;
        }
        Ok(())
    }

    pub fn set_click_through(hwnd_raw: isize, enabled: bool) -> Result<(), String> {
        unsafe {
            let h = hwnd(hwnd_raw);
            let mut ex_style = GetWindowLongW(h, GWL_EXSTYLE) as u32;
            if enabled {
                ex_style |= WS_EX_TRANSPARENT.0;
            } else {
                ex_style &= !WS_EX_TRANSPARENT.0;
            }
            SetWindowLongW(h, GWL_EXSTYLE, ex_style as i32);
        }
        Ok(())
    }

    pub fn is_valid(hwnd_raw: isize) -> bool {
        unsafe { IsWindow(hwnd(hwnd_raw)).as_bool() }
    }

    pub fn get_rect(hwnd_raw: isize) -> Option<WindowRect> {
        unsafe {
            let mut r = windows::Win32::Foundation::RECT::default();
            GetWindowRect(hwnd(hwnd_raw), &mut r).ok()?;
            Some(WindowRect {
                x: r.left,
                y: r.top,
                width: r.right - r.left,
                height: r.bottom - r.top,
            })
        }
    }

    pub fn set_pos_size(hwnd_raw: isize, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
        unsafe {
            SetWindowPos(
                hwnd(hwnd_raw),
                HWND::default(),
                x, y, width, height,
                SWP_NOACTIVATE | SWP_NOZORDER,
            )
            .map_err(|e| format!("SetWindowPos failed: {e}"))
        }
    }
}

pub fn do_unpin(hwnd_raw: isize) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return win::unpin(hwnd_raw);
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn pin_window(hwnd: isize, opacity: f32, click_through: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return win::pin(hwnd, opacity, click_through);
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn unpin_window(hwnd: isize) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return win::unpin(hwnd);
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn set_window_opacity(hwnd: isize, opacity: f32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return win::set_opacity(hwnd, opacity);
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn set_window_click_through(hwnd: isize, click_through: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return win::set_click_through(hwnd, click_through);
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

#[tauri::command]
pub fn is_window_valid(hwnd: isize) -> bool {
    #[cfg(target_os = "windows")]
    return win::is_valid(hwnd);
    #[cfg(not(target_os = "windows"))]
    false
}

#[tauri::command]
pub fn get_window_rect(hwnd: isize) -> Option<WindowRect> {
    #[cfg(target_os = "windows")]
    return win::get_rect(hwnd);
    #[cfg(not(target_os = "windows"))]
    None
}

#[tauri::command]
pub fn set_window_pos_size(hwnd: isize, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return win::set_pos_size(hwnd, x, y, width, height);
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

/// Returns the HWND of the current OS foreground window.
#[tauri::command]
pub fn get_foreground_window() -> Option<isize> {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        return Some(hwnd.0 as isize);
    }
    #[cfg(not(target_os = "windows"))]
    None
}
