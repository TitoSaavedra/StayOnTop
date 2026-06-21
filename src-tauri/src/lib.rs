mod commands;

use commands::{
    highlight::{clear_highlight, show_highlight},
    process::{get_app_name, get_processes},
    settings::{get_settings, register_startup, save_settings},
    window::{
        get_foreground_window, pin_window, set_window_click_through,
        set_window_opacity, unpin_window,
    },
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_processes,
            get_app_name,
            pin_window,
            unpin_window,
            set_window_opacity,
            set_window_click_through,
            get_foreground_window,
            get_settings,
            save_settings,
            register_startup,
            show_highlight,
            clear_highlight,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
