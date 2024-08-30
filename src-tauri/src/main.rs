// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_variables)]
mod commands;
mod tokenizer;
mod serde_obj;

use std::sync::Arc;

use tauri::{
    async_runtime::Mutex, AppHandle, CustomMenuItem, Manager as _, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowBuilder
};
use tauri_plugin_positioner::WindowExt as _;
use tokenizer::MessageType;

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

fn get_dir() -> std::path::PathBuf {
    std::env::current_exe().unwrap().parent().unwrap().to_path_buf()
}

fn get_config() -> serde_obj::ConfigFile {
    let file_path = get_dir().join("config.json");
    if !file_path.exists() {
        std::fs::write(file_path.clone(), include_bytes!("cdn/config.json")).unwrap();
    }   
    let json = std::fs::read_to_string(file_path.clone()).unwrap();
    serde_json::from_str(&json).unwrap()
}

fn create_main_window(app: AppHandle) {
    let window_r = app.get_window("main");
    let window = if window_r.is_none() {
        WindowBuilder::new(&app, "main", tauri::WindowUrl::App("".into()))
            .transparent(true)
            .skip_taskbar(true)
            .always_on_top(true)
            .decorations(false)
            .resizable(false)
            .enable_clipboard_access()
            .build()
            .unwrap()
    } else {
        window_r.unwrap()
    };
    let binding = window.current_monitor().unwrap().unwrap();
    let monitor = binding.size();
    window.set_size(tauri::PhysicalSize::new(400, monitor.height - 60)).unwrap();
    window.show().unwrap();
    window.move_window(tauri_plugin_positioner::Position::TopRight).unwrap();
    window.set_focus().unwrap();
}


fn create_setting_window(app: AppHandle) {
    let window_r = app.get_window("settings");
    let window = if window_r.is_none() {
        WindowBuilder::new(&app, "settings", tauri::WindowUrl::App("setting".into()))
            .transparent(true)
            .decorations(false)
            .enable_clipboard_access()
            .build()
            .unwrap()
    } else {
        window_r.unwrap()
    };
    window.show().unwrap();
    window.set_focus().unwrap();
}

fn tray_event(app: &AppHandle, event: tauri::SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            create_main_window(app.clone());
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "settings" => {
                create_setting_window(app.clone());
            }
            "quit" => {
                for window in app.windows().values() {
                    window.close().unwrap();
                    while !window.is_closable().unwrap() {}
                }
                app.exit(0);
            }
            _ => {}
        },
        _ => {}
    }
}

#[allow(unused_mut)]
fn create_sys_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let tray_menu = SystemTrayMenu::new().add_item(settings).add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

fn main() {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();
    let config = get_config();
    let mess_list: Arc<Mutex<Vec<MessageType>>> = Arc::new(Mutex::new(Vec::new()));
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(config)))
        .manage(mess_list)
        .system_tray(create_sys_tray())
        .on_system_tray_event(|app: &AppHandle, event: SystemTrayEvent| tray_event(app, event))
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            app.emit_all("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            // If you need macOS support this must be called in .setup() !
            // Otherwise this could be called right after prepare() but then you don't have access to tauri APIs
            let handle = app.handle();
            let _ = tauri_plugin_deep_link::unregister("aihelper");
            tauri_plugin_deep_link::prepare("aihelper");
            tauri_plugin_deep_link::register("aihelper", move |request| {
                handle.emit_all("scheme-request-received", request).unwrap();
            })
            .unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![crate::commands::md_to_html])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
