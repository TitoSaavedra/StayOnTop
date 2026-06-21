#[cfg(target_os = "windows")]
mod win {
    use std::sync::{Arc, Condvar, Mutex, OnceLock};

    use windows::Win32::Foundation::{
        COLORREF, HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
    };
    use windows::Win32::Graphics::Gdi::{
        BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, CreateCompatibleDC, CreateDIBSection,
        DeleteDC, DeleteObject, DIB_RGB_COLORS, HDC, SelectObject,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowRect,
        IsIconic, IsWindowVisible, PostQuitMessage, RegisterClassExW, SetWindowPos,
        ShowWindow, TranslateMessage, UpdateLayeredWindow, CS_HREDRAW, CS_VREDRAW,
        HWND_TOPMOST, MSG, SWP_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE, ULW_ALPHA,
        WM_DESTROY, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOPMOST,
        WS_EX_TRANSPARENT, WS_POPUP,
    };
    use windows::core::PCWSTR;

    // Pixels of outer glow spilling outside the target window
    const GLOW_OUTER: i32 = 8;
    // Fully opaque pixels just inside the window edge
    const SOLID: i32 = 2;
    // Pixels that fade inward after the solid band
    const FADE_IN: i32 = 2;

    // Accent glow colour #ef6060 (matches $accent-glow in _variables.scss)
    const A_R: u32 = 0xEF;
    const A_G: u32 = 0x60;
    const A_B: u32 = 0x60;

    static OVERLAY: OnceLock<isize> = OnceLock::new();

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_DESTROY {
            PostQuitMessage(0);
            return LRESULT(0);
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    fn spawn_overlay() -> isize {
        let pair = Arc::new((Mutex::new(None::<Option<isize>>), Condvar::new()));
        let pair2 = pair.clone();

        std::thread::spawn(move || unsafe {
            let class_name: Vec<u16> = "StayOnTopOverlay\0".encode_utf16().collect();
            let hmod = GetModuleHandleW(None).unwrap();
            let hinst = HINSTANCE(hmod.0);

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                hInstance: hinst,
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            RegisterClassExW(&wc);

            let result = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
                PCWSTR(class_name.as_ptr()),
                PCWSTR::null(),
                WS_POPUP,
                0, 0, 100, 100,
                None, None, hinst, None,
            );

            let raw = result.map(|h| h.0 as isize).ok();
            {
                let mut lock = pair2.0.lock().unwrap();
                *lock = Some(raw);
                pair2.1.notify_one();
            }

            if raw.unwrap_or(0) == 0 {
                return;
            }

            let mut msg = MSG::default();
            loop {
                match GetMessageW(&mut msg, None, 0, 0) {
                    r if r.0 <= 0 => break,
                    _ => {}
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        });

        let (lock, cvar) = &*pair;
        let mut guard = lock.lock().unwrap();
        while guard.is_none() {
            guard = cvar.wait(guard).unwrap();
        }
        guard.unwrap().unwrap_or(0)
    }

    fn overlay_hwnd() -> Option<HWND> {
        let raw = *OVERLAY.get_or_init(spawn_overlay);
        if raw == 0 { None } else { Some(HWND(raw as *mut _)) }
    }

    // Signed distance from window interior edge.
    // Positive  → inside the window (0 = on edge)
    // Negative  → outside the window
    fn signed_dist(x_rel: i32, y_rel: i32, w: i32, h: i32) -> i32 {
        if x_rel >= 0 && x_rel < w && y_rel >= 0 && y_rel < h {
            // inside: min distance to any of the four edges
            [x_rel, w - 1 - x_rel, y_rel, h - 1 - y_rel]
                .iter()
                .copied()
                .min()
                .unwrap()
        } else {
            // outside: Chebyshev distance (gives square corners, fits window shape)
            let ox = if x_rel < 0 { -x_rel } else if x_rel >= w { x_rel - w + 1 } else { 0 };
            let oy = if y_rel < 0 { -y_rel } else if y_rel >= h { y_rel - h + 1 } else { 0 };
            -(ox.max(oy))
        }
    }

    fn alpha_for_dist(d: i32) -> u8 {
        if d < -GLOW_OUTER { return 0; }
        if d < 0 {
            // Outer glow: quadratic ramp → makes it look soft
            let t = (d + GLOW_OUTER) as f32 / GLOW_OUTER as f32;
            return (t * t * 210.0) as u8;
        }
        if d < SOLID { return 255; }
        if d < SOLID + FADE_IN {
            let t = 1.0 - (d - SOLID + 1) as f32 / (FADE_IN + 1) as f32;
            return (t * 160.0) as u8;
        }
        0
    }

    // Premultiplied BGRA u32 for the accent colour at given alpha.
    // DIB memory layout on little-endian: bytes [B, G, R, A] at consecutive addresses.
    fn premul(alpha: u8) -> u32 {
        let a = alpha as u32;
        let b = (A_B * a) / 255;
        let g = (A_G * a) / 255;
        let r = (A_R * a) / 255;
        b | (g << 8) | (r << 16) | (a << 24)
    }

    pub fn show(target_raw: isize) {
        unsafe {
            let Some(ov) = overlay_hwnd() else { return };
            let target = HWND(target_raw as *mut _);

            // Hide overlay for windows that aren't on-screen
            if !IsWindowVisible(target).as_bool() || IsIconic(target).as_bool() {
                let _ = ShowWindow(ov, SW_HIDE);
                return;
            }

            let mut rect = RECT::default();
            if GetWindowRect(target, &mut rect).is_err() { return; }

            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w <= 0 || h <= 0 { return; }

            // Overlay is larger than the target window to hold the outer glow
            let ow = w + 2 * GLOW_OUTER;
            let oh = h + 2 * GLOW_OUTER;

            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: ow,
                    biHeight: -oh, // top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    ..Default::default()
                },
                ..Default::default()
            };

            let hdc_mem = CreateCompatibleDC(None);
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let hbmp = match CreateDIBSection(
                hdc_mem, &bmi, DIB_RGB_COLORS, &mut bits, HANDLE::default(), 0,
            ) {
                Ok(h) => h,
                Err(_) => { let _ = DeleteDC(hdc_mem); return; }
            };
            let old = SelectObject(hdc_mem, hbmp);

            // Fill glow pixels directly in DIB memory (no GDI drawing calls needed)
            let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, (ow * oh) as usize);
            for py in 0..oh {
                for px in 0..ow {
                    let x_rel = px - GLOW_OUTER;
                    let y_rel = py - GLOW_OUTER;
                    let d = signed_dist(x_rel, y_rel, w, h);
                    let a = alpha_for_dist(d);
                    pixels[(py * ow + px) as usize] = if a == 0 { 0 } else { premul(a) };
                }
            }

            let pt_zero = POINT { x: 0, y: 0 };
            let size = SIZE { cx: ow, cy: oh };
            let blend = BLENDFUNCTION {
                BlendOp: 0,               // AC_SRC_OVER
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: 1,           // AC_SRC_ALPHA (per-pixel alpha)
            };

            let _ = SetWindowPos(
                ov, HWND_TOPMOST,
                rect.left - GLOW_OUTER, rect.top - GLOW_OUTER, ow, oh,
                SWP_NOACTIVATE,
            );
            let _ = UpdateLayeredWindow(
                ov,
                HDC::default(), // null → use screen DC
                None,           // position already set via SetWindowPos
                Some(&size),
                hdc_mem,        // source DC — must be HDC, not Option<HDC>
                Some(&pt_zero),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
            let _ = ShowWindow(ov, SW_SHOWNOACTIVATE);

            SelectObject(hdc_mem, old);
            let _ = DeleteObject(hbmp);
            let _ = DeleteDC(hdc_mem);
        }
    }

    pub fn hide() {
        if let Some(&raw) = OVERLAY.get() {
            if raw != 0 {
                unsafe { let _ = ShowWindow(HWND(raw as *mut _), SW_HIDE); }
            }
        }
    }
}

#[tauri::command]
pub fn show_highlight(hwnd: isize) {
    #[cfg(target_os = "windows")]
    win::show(hwnd);
}

#[tauri::command]
pub fn clear_highlight() {
    #[cfg(target_os = "windows")]
    win::hide();
}
