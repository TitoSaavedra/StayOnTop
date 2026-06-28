mod commands;

use tauri::Manager;
use commands::{
    highlight::{clear_highlight, show_highlight},
    image_window::{
        close_all_image_windows, get_image_position, get_pinned_images, pin_image,
        save_pinned_images, set_image_click_through, set_image_opacity, set_image_scale,
        unpin_image,
    },
    pinned::{get_pinned, read_pinned, save_pinned},
    process::{get_app_name, get_process_by_hwnd, get_processes},
    settings::{get_settings, register_startup, save_settings},
    window::{
        do_unpin, get_foreground_window, get_window_rect, is_window_valid, pin_window,
        set_window_click_through, set_window_opacity, set_window_pos_size, unpin_window,
    },
};

fn shutdown(app: &tauri::AppHandle) {
    for pin in read_pinned() {
        let _ = do_unpin(pin.hwnd);
    }
    close_all_image_windows();
    app.exit(0);
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    shutdown(&app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

            let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(tauri::include_image!("icons/32x32.png"))
                .tooltip("StayOnTop")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => shutdown(app),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_processes,
            get_app_name,
            get_process_by_hwnd,
            pin_window,
            unpin_window,
            set_window_opacity,
            set_window_click_through,
            get_foreground_window,
            is_window_valid,
            get_window_rect,
            set_window_pos_size,
            get_settings,
            save_settings,
            register_startup,
            show_highlight,
            clear_highlight,
            get_pinned,
            save_pinned,
            pin_image,
            unpin_image,
            set_image_opacity,
            set_image_scale,
            set_image_click_through,
            get_image_position,
            get_pinned_images,
            save_pinned_images,
            quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
