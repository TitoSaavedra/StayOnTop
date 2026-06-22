use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedPin {
    pub hwnd: isize,
    pub process_name: String,
    pub window_title: String,
    pub opacity: f32,
    pub click_through: bool,
    pub icon: Option<String>,
}

pub fn pinned_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("StayOnTop").join("pinned.json")
}

pub fn read_pinned() -> Vec<PersistedPin> {
    std::fs::read_to_string(pinned_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_pinned() -> Vec<PersistedPin> {
    read_pinned()
}

#[tauri::command]
pub fn save_pinned(pins: Vec<PersistedPin>) -> Result<(), String> {
    let path = pinned_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&pins).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}
