use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub window_title: String,
    pub hwnd: isize,
    pub icon: Option<String>,
}

#[cfg(target_os = "windows")]
mod win {
    use super::ProcessInfo;
    use std::collections::HashMap;
    use windows::Win32::Foundation::{BOOL, CloseHandle, HANDLE, HWND, LPARAM};
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, HBRUSH,
    };
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON};
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyIcon, DrawIconEx, EnumWindows, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible, DI_NORMAL,
    };
    use windows::core::{PCWSTR, PWSTR};

    struct EnumState {
        map: HashMap<u32, Vec<(isize, String)>>,
    }

    unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = &mut *(lparam.0 as *mut EnumState);
        if IsWindowVisible(hwnd).as_bool() {
            let len = GetWindowTextLengthW(hwnd);
            if len > 0 {
                let mut buf = vec![0u16; (len + 1) as usize];
                GetWindowTextW(hwnd, &mut buf);
                let title = String::from_utf16_lossy(&buf[..len as usize]);
                let mut pid = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if pid != 0 {
                    state.map.entry(pid).or_default().push((hwnd.0 as isize, title));
                }
            }
        }
        BOOL(1)
    }

    fn get_exe_path(pid: u32) -> Option<String> {
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buf = vec![0u16; 1024];
            let mut size = buf.len() as u32;
            let res = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut size);
            let _ = windows::Win32::Foundation::CloseHandle(h);
            res.ok()?;
            Some(String::from_utf16_lossy(&buf[..size as usize]))
        }
    }

    fn extract_icon(exe_path: &str) -> Option<String> {
        unsafe {
            let path_w: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
            let mut sfi = SHFILEINFOW::default();
            let r = SHGetFileInfoW(
                PCWSTR(path_w.as_ptr()),
                Default::default(),
                Some(&mut sfi),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_SMALLICON,
            );
            if r == 0 || sfi.hIcon.is_invalid() {
                return None;
            }
            let hicon = sfi.hIcon;
            let icon_px: i32 = 16;

            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: icon_px,
                    biHeight: -icon_px,
                    biPlanes: 1,
                    biBitCount: 32,
                    ..Default::default()  // biCompression = BI_COMPRESSION(0) = BI_RGB
                },
                ..Default::default()
            };

            let hdc = CreateCompatibleDC(None);
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let hbmp = match CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, HANDLE::default(), 0) {
                Ok(h) => h,
                Err(_) => {
                    let _ = DeleteDC(hdc);
                    let _ = DestroyIcon(hicon);
                    return None;
                }
            };

            let old = SelectObject(hdc, hbmp);
            let _ = DrawIconEx(hdc, 0, 0, hicon, icon_px, icon_px, 0, HBRUSH::default(), DI_NORMAL);
            let _ = SelectObject(hdc, old);

            let n = (icon_px * icon_px) as usize;
            let mut pixels = std::slice::from_raw_parts(bits as *const u8, n * 4).to_vec();

            let _ = DeleteObject(hbmp);
            let _ = DeleteDC(hdc);
            let _ = DestroyIcon(hicon);

            // BGRA → RGBA; fix alpha for old-style masked icons
            let all_zero_alpha = pixels.chunks_exact(4).all(|p| p[3] == 0);
            for px in pixels.chunks_exact_mut(4) {
                px.swap(0, 2);
                if all_zero_alpha && (px[0] | px[1] | px[2]) != 0 {
                    px[3] = 255;
                }
            }

            let mut png_bytes: Vec<u8> = Vec::new();
            {
                let cursor = std::io::Cursor::new(&mut png_bytes);
                let mut enc = png::Encoder::new(cursor, icon_px as u32, icon_px as u32);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                let mut w = enc.write_header().ok()?;
                w.write_image_data(&pixels).ok()?;
            }

            use base64::Engine as _;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
            Some(format!("data:image/png;base64,{b64}"))
        }
    }

    pub fn own_name() -> String {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
            .unwrap_or_default()
    }

    pub fn get_by_hwnd(hwnd_raw: isize) -> Option<ProcessInfo> {
        unsafe {
            let h = HWND(hwnd_raw as *mut _);

            let len = GetWindowTextLengthW(h);
            if len == 0 { return None; }
            let mut buf = vec![0u16; (len + 1) as usize];
            GetWindowTextW(h, &mut buf);
            let title = String::from_utf16_lossy(&buf[..len as usize]);

            let mut pid = 0u32;
            GetWindowThreadProcessId(h, Some(&mut pid));
            if pid == 0 { return None; }

            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut name = String::new();
            if Process32FirstW(snap, &mut entry).is_ok() {
                loop {
                    if entry.th32ProcessID == pid {
                        let nul = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                        name = String::from_utf16_lossy(&entry.szExeFile[..nul]);
                        break;
                    }
                    if Process32NextW(snap, &mut entry).is_err() { break; }
                }
            }
            let _ = CloseHandle(snap);
            if name.is_empty() { return None; }

            let icon = get_exe_path(pid).and_then(|p| extract_icon(&p));
            Some(ProcessInfo { pid, name, window_title: title, hwnd: hwnd_raw, icon })
        }
    }

    pub fn collect(app_name: &str) -> Vec<ProcessInfo> {
        let mut state = EnumState { map: HashMap::new() };
        unsafe {
            let _ = EnumWindows(Some(enum_cb), LPARAM(&mut state as *mut _ as isize));
        }

        let mut results = Vec::new();
        let mut icon_cache: HashMap<String, Option<String>> = HashMap::new();

        unsafe {
            let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(h) => h,
                Err(_) => return results,
            };
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            if Process32FirstW(snap, &mut entry).is_ok() {
                loop {
                    let pid = entry.th32ProcessID;
                    if let Some(wins) = state.map.get(&pid) {
                        let nul = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                        let name = String::from_utf16_lossy(&entry.szExeFile[..nul]);

                        if !app_name.is_empty() && name.eq_ignore_ascii_case(app_name) {
                            if Process32NextW(snap, &mut entry).is_err() { break; }
                            continue;
                        }

                        let icon = icon_cache
                            .entry(name.clone())
                            .or_insert_with(|| get_exe_path(pid).and_then(|p| extract_icon(&p)))
                            .clone();

                        for (hwnd, title) in wins {
                            results.push(ProcessInfo { pid, name: name.clone(), window_title: title.clone(), hwnd: *hwnd, icon: icon.clone() });
                        }
                    }
                    if Process32NextW(snap, &mut entry).is_err() { break; }
                }
            }
            let _ = CloseHandle(snap);
        }

        results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        results
    }
}

#[tauri::command]
pub fn get_processes() -> Vec<ProcessInfo> {
    #[cfg(target_os = "windows")]
    { win::collect(&win::own_name()) }
    #[cfg(not(target_os = "windows"))]
    { vec![] }
}

#[tauri::command]
pub fn get_app_name() -> String {
    #[cfg(target_os = "windows")]
    { win::own_name() }
    #[cfg(not(target_os = "windows"))]
    { String::new() }
}

#[tauri::command]
pub fn get_process_by_hwnd(hwnd: isize) -> Option<ProcessInfo> {
    #[cfg(target_os = "windows")]
    { win::get_by_hwnd(hwnd) }
    #[cfg(not(target_os = "windows"))]
    { None }
}
