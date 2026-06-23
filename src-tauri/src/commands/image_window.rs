#[cfg(target_os = "windows")]
mod win {
    use std::collections::HashMap;
    use std::mem::size_of;
    use std::sync::{Arc, Condvar, Mutex, OnceLock};

    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::PCWSTR;

    const WM_IMG_REDRAW: u32 = WM_USER + 100;
    const WM_IMG_CLICK_THROUGH: u32 = WM_USER + 101;

    pub struct WinState {
        /// Premultiplied BGRA pixels at original resolution. Computed once at pin time.
        pub orig_bgra_premul: Vec<u8>,
        pub orig_w: u32,
        pub orig_h: u32,
        pub scale: f32,
        pub opacity: f32,
        pub click_through: bool,
        // True while a WM_IMG_REDRAW is already in the queue.
        // Prevents flooding the message queue with redundant renders on fast slider moves.
        pub pending_redraw: bool,
        // Last rendered scale result — reused when only opacity changes.
        pub cached_w: u32,
        pub cached_h: u32,
        pub cached_bgra: Vec<u8>,
        // Cursor offset from window top-left when a drag starts.
        pub drag_offset: Option<(i32, i32)>,
    }

    type Registry = Mutex<HashMap<String, (isize, Arc<Mutex<WinState>>)>>;
    static REGISTRY: OnceLock<Registry> = OnceLock::new();

    fn registry() -> &'static Registry {
        REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
    }

    /// Nearest-neighbor resize of premultiplied BGRA pixels.
    /// Fast enough for real-time slider updates (~0.5 ms for a 2000×2000 source).
    fn nn_scale_bgra(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
        let mut dst = vec![0u8; (dw * dh * 4) as usize];
        for dy in 0..dh {
            let sy = (dy * sh / dh) as usize;
            for dx in 0..dw {
                let sx = (dx * sw / dw) as usize;
                let si = (sy * sw as usize + sx) * 4;
                let di = (dy * dw + dx) as usize * 4;
                dst[di]     = src[si];
                dst[di + 1] = src[si + 1];
                dst[di + 2] = src[si + 2];
                dst[di + 3] = src[si + 3];
            }
        }
        dst
    }

    unsafe fn render(hwnd: HWND, state: &Mutex<WinState>) {
        let mut s = state.lock().unwrap();

        let new_w = ((s.orig_w as f32 * s.scale) as u32).max(1);
        let new_h = ((s.orig_h as f32 * s.scale) as u32).max(1);
        let opacity_alpha = (s.opacity.clamp(0.0, 1.0) * 255.0) as u8;

        // Resize only when dimensions changed — opacity-only updates skip this entirely.
        if new_w != s.cached_w || new_h != s.cached_h {
            s.cached_bgra = if new_w == s.orig_w && new_h == s.orig_h {
                s.orig_bgra_premul.clone()
            } else {
                nn_scale_bgra(&s.orig_bgra_premul, s.orig_w, s.orig_h, new_w, new_h)
            };
            s.cached_w = new_w;
            s.cached_h = new_h;
        }

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: new_w as i32,
                biHeight: -(new_h as i32), // top-down
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };

        let hdc_mem = CreateCompatibleDC(None);
        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let Ok(hbmp) =
            CreateDIBSection(hdc_mem, &bmi, DIB_RGB_COLORS, &mut bits, HANDLE::default(), 0)
        else {
            drop(s);
            let _ = DeleteDC(hdc_mem);
            return;
        };
        let old = SelectObject(hdc_mem, hbmp);

        let dst = std::slice::from_raw_parts_mut(bits as *mut u8, s.cached_bgra.len());
        dst.copy_from_slice(&s.cached_bgra);
        drop(s); // release lock before Win32 calls

        let size = SIZE { cx: new_w as i32, cy: new_h as i32 };
        let pt_src = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: 0,   // AC_SRC_OVER
            BlendFlags: 0,
            SourceConstantAlpha: opacity_alpha,
            AlphaFormat: 1, // AC_SRC_ALPHA (per-pixel alpha)
        };

        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            new_w as i32,
            new_h as i32,
            SWP_NOMOVE | SWP_NOACTIVATE,
        );
        let _ = UpdateLayeredWindow(
            hwnd,
            HDC::default(),
            None,
            Some(&size),
            hdc_mem,
            Some(&pt_src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );

        SelectObject(hdc_mem, old);
        let _ = DeleteObject(hbmp);
        let _ = DeleteDC(hdc_mem);
    }

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let cs = &*(lparam.0 as *const CREATESTRUCTW);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, cs.lpCreateParams as isize);
                LRESULT(0)
            }
            WM_DESTROY => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<WinState>;
                if !ptr.is_null() {
                    drop(Arc::from_raw(ptr));
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            // Treat the whole window as client area so WM_LBUTTONDOWN fires.
            // Alpha=0 pixels never receive mouse events on a layered window, so
            // transparent PNG regions automatically pass through to windows below.
            // (HTCAPTION would also drag but the system would constrain the window to screen.)
            WM_NCHITTEST => LRESULT(HTCLIENT as isize),
            WM_LBUTTONDOWN => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<WinState>;
                if !ptr.is_null() {
                    // Store click position within the window as drag origin.
                    let cx = (lparam.0 & 0xFFFF) as u16 as i16 as i32;
                    let cy = ((lparam.0 >> 16) & 0xFFFF) as u16 as i16 as i32;
                    (*ptr).lock().unwrap().drag_offset = Some((cx, cy));
                }
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<WinState>;
                if !ptr.is_null() {
                    // MK_LBUTTON (0x0001) tells us if the button is still held.
                    // If it was released outside our window, cancel drag here.
                    if (wparam.0 & 0x0001) == 0 {
                        (*ptr).lock().unwrap().drag_offset = None;
                    } else {
                        let drag = (*ptr).lock().unwrap().drag_offset;
                        if let Some((ox, oy)) = drag {
                            let mut cursor = POINT { x: 0, y: 0 };
                            let _ = GetCursorPos(&mut cursor);
                            let _ = SetWindowPos(
                                hwnd,
                                HWND(std::ptr::null_mut()),
                                cursor.x - ox,
                                cursor.y - oy,
                                0, 0,
                                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                            );
                        }
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<WinState>;
                if !ptr.is_null() {
                    (*ptr).lock().unwrap().drag_offset = None;
                }
                LRESULT(0)
            }
            WM_IMG_REDRAW => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<WinState>;
                if !ptr.is_null() {
                    // Clear flag first so new updates can queue another redraw while we render
                    { (*ptr).lock().unwrap().pending_redraw = false; }
                    render(hwnd, &*ptr);
                }
                LRESULT(0)
            }
            WM_IMG_CLICK_THROUGH => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<WinState>;
                if !ptr.is_null() {
                    let ct = (*ptr).lock().unwrap().click_through;
                    let mut ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                    if ct {
                        ex |= WS_EX_TRANSPARENT.0;
                    } else {
                        ex &= !WS_EX_TRANSPARENT.0;
                    }
                    SetWindowLongW(hwnd, GWL_EXSTYLE, ex as i32);
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    pub fn spawn_window(
        id: String,
        orig_bgra_premul: Vec<u8>,
        orig_w: u32,
        orig_h: u32,
        x: i32,
        y: i32,
        opacity: f32,
        scale: f32,
    ) -> Result<(), String> {
        let state = Arc::new(Mutex::new(WinState {
            orig_bgra_premul,
            orig_w,
            orig_h,
            scale,
            opacity,
            click_through: false,
            pending_redraw: false,
            cached_w: 0,
            cached_h: 0,
            cached_bgra: Vec::new(),
            drag_offset: None,
        }));

        let pair = Arc::new((Mutex::new(None::<isize>), Condvar::new()));
        let pair2 = pair.clone();
        let state2 = state.clone();

        std::thread::spawn(move || unsafe {
            let class_name: Vec<u16> = "StayOnTopImage\0".encode_utf16().collect();
            let hmod = GetModuleHandleW(None).unwrap();
            let hinst = HINSTANCE(hmod.0);

            let wc = WNDCLASSEXW {
                cbSize: size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                hInstance: hinst,
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            // May fail if class already registered by a prior window — that is fine
            let _ = RegisterClassExW(&wc);

            let state_ptr = Arc::into_raw(state2);
            let (sw, sh) = {
                let s = (*state_ptr).lock().unwrap();
                (
                    ((s.orig_w as f32 * s.scale) as i32).max(1),
                    ((s.orig_h as f32 * s.scale) as i32).max(1),
                )
            };

            let hwnd_result = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST,
                PCWSTR(class_name.as_ptr()),
                PCWSTR::null(),
                WS_POPUP,
                x,
                y,
                sw,
                sh,
                None,
                None,
                hinst,
                Some(state_ptr as *const _ as *const std::ffi::c_void),
            );

            match hwnd_result {
                Err(e) => {
                    let _ = Arc::from_raw(state_ptr);
                    let mut g = pair2.0.lock().unwrap();
                    *g = Some(0isize);
                    pair2.1.notify_one();
                    eprintln!("image_window: CreateWindowExW failed: {e}");
                }
                Ok(hwnd) => {
                    render(hwnd, &*state_ptr);
                    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

                    {
                        let mut g = pair2.0.lock().unwrap();
                        *g = Some(hwnd.0 as isize);
                        pair2.1.notify_one();
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
                }
            }
        });

        let (lock, cvar) = &*pair;
        let mut guard = lock.lock().unwrap();
        while guard.is_none() {
            guard = cvar.wait(guard).unwrap();
        }
        let hwnd_raw = guard.unwrap();

        if hwnd_raw == 0 {
            return Err("Failed to create image window".into());
        }

        registry().lock().unwrap().insert(id, (hwnd_raw, state));
        Ok(())
    }

    fn post(hwnd_raw: isize, msg: u32) {
        unsafe {
            let _ = PostMessageW(
                HWND(hwnd_raw as *mut _),
                msg,
                WPARAM(0),
                LPARAM(0),
            );
        }
    }

    pub fn close_window(id: &str) -> Result<(), String> {
        if let Some((hwnd_raw, _)) = registry().lock().unwrap().remove(id) {
            // Hide immediately so the user sees it gone right away;
            // WM_CLOSE then destroys the window and exits the thread cleanly.
            unsafe { let _ = ShowWindow(HWND(hwnd_raw as *mut _), SW_HIDE); }
            post(hwnd_raw, WM_CLOSE);
        }
        Ok(())
    }

    pub fn close_all() {
        let entries: Vec<_> = registry().lock().unwrap().drain().collect();
        for (_, (hwnd_raw, _)) in entries {
            unsafe { let _ = ShowWindow(HWND(hwnd_raw as *mut _), SW_HIDE); }
            post(hwnd_raw, WM_CLOSE);
        }
    }

    fn update_and_redraw(id: &str, f: impl FnOnce(&mut WinState)) -> Result<(), String> {
        let result = {
            let reg = registry().lock().unwrap();
            reg.get(id).map(|(h, s)| {
                let mut state = s.lock().unwrap();
                f(&mut state);
                let already = state.pending_redraw;
                state.pending_redraw = true;
                (*h, already)
            })
        };
        match result {
            Some((h, false)) => { post(h, WM_IMG_REDRAW); Ok(()) }
            Some((_, true))  => Ok(()), // render already queued, skip duplicate post
            None => Err(format!("image '{id}' not found")),
        }
    }

    pub fn set_opacity(id: &str, opacity: f32) -> Result<(), String> {
        update_and_redraw(id, |s| s.opacity = opacity)
    }

    pub fn set_scale(id: &str, scale: f32) -> Result<(), String> {
        update_and_redraw(id, |s| s.scale = scale)
    }

    pub fn set_click_through(id: &str, click_through: bool) -> Result<(), String> {
        let result = {
            let reg = registry().lock().unwrap();
            reg.get(id).map(|(h, s)| { s.lock().unwrap().click_through = click_through; *h })
        };
        match result {
            Some(h) => { post(h, WM_IMG_CLICK_THROUGH); Ok(()) }
            None => Err(format!("image '{id}' not found")),
        }
    }

    pub fn get_position(id: &str) -> Option<[i32; 2]> {
        let reg = registry().lock().unwrap();
        reg.get(id).map(|(hwnd_raw, _)| unsafe {
            let mut rect = RECT::default();
            let _ = GetWindowRect(HWND(*hwnd_raw as *mut _), &mut rect);
            [rect.left, rect.top]
        })
    }
}

// ── Persistence ──────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedImage {
    pub id: String,
    pub path: String,
    pub x: i32,
    pub y: i32,
    pub opacity: f32,
    pub scale: f32,
    pub click_through: bool,
}

fn images_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("StayOnTop").join("images.json")
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn pin_image(
    id: String,
    path: String,
    x: i32,
    y: i32,
    opacity: f32,
    scale: f32,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let img = image::open(&path).map_err(|e| format!("Failed to load image '{path}': {e}"))?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        // Convert RGBA → premultiplied BGRA once; render() never needs to do this again.
        let bgra_premul: Vec<u8> = rgba
            .pixels()
            .flat_map(|p| {
                let [r, g, b, a] = p.0;
                let a32 = a as u32;
                [
                    (b as u32 * a32 / 255) as u8,
                    (g as u32 * a32 / 255) as u8,
                    (r as u32 * a32 / 255) as u8,
                    a,
                ]
            })
            .collect();
        win::spawn_window(id, bgra_premul, w, h, x, y, opacity, scale)?;
    }
    Ok(())
}

#[tauri::command]
pub fn unpin_image(id: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    win::close_window(&id)?;
    Ok(())
}

#[tauri::command]
pub fn set_image_opacity(id: String, opacity: f32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    win::set_opacity(&id, opacity)?;
    Ok(())
}

#[tauri::command]
pub fn set_image_scale(id: String, scale: f32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    win::set_scale(&id, scale)?;
    Ok(())
}

#[tauri::command]
pub fn set_image_click_through(id: String, click_through: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    win::set_click_through(&id, click_through)?;
    Ok(())
}

#[tauri::command]
pub fn get_image_position(id: String) -> Option<[i32; 2]> {
    #[cfg(target_os = "windows")]
    return win::get_position(&id);
    #[cfg(not(target_os = "windows"))]
    None
}

#[tauri::command]
pub fn get_pinned_images() -> Vec<PersistedImage> {
    let images: Vec<PersistedImage> = std::fs::read_to_string(images_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    // Silently drop entries whose files no longer exist on disk
    images
        .into_iter()
        .filter(|img| std::path::Path::new(&img.path).exists())
        .collect()
}

#[tauri::command]
pub fn save_pinned_images(images: Vec<PersistedImage>) -> Result<(), String> {
    let path = images_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    serde_json::to_string_pretty(&images)
        .map_err(|e| e.to_string())
        .and_then(|json| std::fs::write(&path, json).map_err(|e| e.to_string()))
}

pub fn close_all_image_windows() {
    #[cfg(target_os = "windows")]
    win::close_all();
}
