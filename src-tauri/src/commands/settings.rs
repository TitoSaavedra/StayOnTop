use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_hotkey_pin_toggle() -> String {
    "Alt+P".into()
}

fn default_opacity() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub start_with_windows: bool,
    #[serde(default)]
    pub keep_app_on_top: bool,
    pub refresh_interval_ms: u32,
    pub excluded_processes: Vec<String>,
    #[serde(default = "default_hotkey_pin_toggle")]
    pub hotkey_pin_toggle: String,
    #[serde(default = "default_opacity")]
    pub default_opacity: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            start_with_windows: false,
            keep_app_on_top: false,
            refresh_interval_ms: 5000,
            excluded_processes: vec![],
            hotkey_pin_toggle: default_hotkey_pin_toggle(),
            default_opacity: default_opacity(),
        }
    }
}

fn settings_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("StayOnTop").join("settings.json")
}

#[tauri::command]
pub fn get_settings() -> AppSettings {
    let path = settings_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn register_startup(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegSetValueExW, RegDeleteValueW,
            HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
        };
        use windows::core::PCWSTR;

        let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16()
            .collect::<Vec<u16>>();
        let value_name = "StayOnTop\0".encode_utf16().collect::<Vec<u16>>();

        unsafe {
            let mut hkey = windows::Win32::System::Registry::HKEY::default();
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(key_path.as_ptr()),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            )
            .ok()
            .map_err(|e| e.to_string())?;

            if enabled {
                let exe = std::env::current_exe()
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .to_string();
                let exe_w: Vec<u16> = exe.encode_utf16().chain(std::iter::once(0)).collect();
                let data = std::slice::from_raw_parts(
                    exe_w.as_ptr() as *const u8,
                    exe_w.len() * 2,
                );
                RegSetValueExW(
                    hkey,
                    PCWSTR(value_name.as_ptr()),
                    0,
                    REG_SZ,
                    Some(data),
                )
                .ok()
                .map_err(|e| e.to_string())?;
            } else {
                let _ = RegDeleteValueW(hkey, PCWSTR(value_name.as_ptr()));
            }

            RegCloseKey(hkey).ok().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
