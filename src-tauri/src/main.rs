// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_variables)]
mod api_req;
mod commands;
mod serde_obj;
mod tokenizer;

use std::{collections::HashMap, sync::Arc};

use serde_json::Value;
use serde_obj::NewInstancePayload;
use tauri::{
    async_runtime::Mutex, AppHandle, CustomMenuItem, Manager as _, State, SystemTray,
    SystemTrayEvent, SystemTrayMenu, WindowBuilder
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_positioner::WindowExt as _;
use tokenizer::MessageType;

fn get_dir() -> std::path::PathBuf {
    #[cfg(dev)]
    {
        let workking_test_dir = std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("test_workdir");
        if !workking_test_dir.exists() {
            std::fs::create_dir(&workking_test_dir).unwrap();
        }
        return workking_test_dir;
    }
    #[cfg(not(dev))]
    {
        return std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
    }
}

fn get_config(plugins_config_default: HashMap<String, Value>, app_handle: Option<AppHandle>) -> serde_obj::ConfigFile {
    let file_path = get_dir().join("config.json");
    let mut config: serde_obj::ConfigFile = if !file_path.exists() {
        serde_json::from_slice(include_bytes!("cdn/config.json")).unwrap()
    } else {
        serde_json::from_str(&std::fs::read_to_string(file_path.clone()).unwrap()).unwrap()
    };
    for (k, v) in plugins_config_default {
        if !config.plugins.contains_key(&k) {
            config.set_plugin_config(k, v);
        }
    }
    config.clone().save_to_file(&file_path, app_handle);
    config
}

fn create_main_window(app: AppHandle) {
    let window_r = app.get_window("main");
    let window = if window_r.is_none() {
        WindowBuilder::new(&app, "main", tauri::WindowUrl::App("".into()))
            .transparent(true)
            .skip_taskbar(true)
            .always_on_top(true)
            .decorations(false)
            .resizable(true)
            .enable_clipboard_access()
            .build()
            .unwrap()
    } else {
        window_r.unwrap()
    };
    let binding = window.current_monitor().unwrap().unwrap();
    let monitor = binding.size();
    window
        .set_size(tauri::PhysicalSize::new(
            monitor.width / 4,
            monitor.height - 60,
        ))
        .unwrap();
    window.show().unwrap();
    window
        .move_window(tauri_plugin_positioner::Position::TopRight)
        .unwrap();
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
    let plugins_config_default = HashMap::new();
    let config = get_config(plugins_config_default, None);
    let mess_list: Arc<Mutex<Vec<MessageType>>> = Arc::new(Mutex::new(Vec::new()));
    tauri::Builder::default()
        .manage(mess_list)
        .manage(Arc::new(Mutex::new(config)))
        .on_window_event(|event| {
            let config: State<Arc<Mutex<serde_obj::ConfigFile>>> = event.window().state();
            match event.event() {
                tauri::WindowEvent::Destroyed => {
                    if event.window().label() == "main" && config.blocking_lock().save_on_close {
                        let messages: State<Arc<Mutex<Vec<MessageType>>>> = event.window().state();
                        messages.blocking_lock().clear();
                    }
                }
                tauri::WindowEvent::Resized(size) => {
                    if event.window().label() == "main" {
                        let x_size = size.width;
                        let y_size = size.height;
                        let window = event.window();
                        let binding = window.current_monitor().unwrap().unwrap();
                        let monitor = binding.size();
                        if (monitor.height - 60) != y_size {
                            window
                                .set_size(tauri::PhysicalSize::new(x_size, monitor.height - 60))
                                .unwrap();
                        }
                    }
                }
                tauri::WindowEvent::Moved(position) => {
                    if event.window().label() == "main" && position.y != 0 {
                        let _ = event
                            .window()
                            .set_position(tauri::PhysicalPosition::new(position.x, 0));
                    }
                }
                _ => {}
            }
        })
        .system_tray(create_sys_tray())
        .on_system_tray_event(|app: &AppHandle, event: SystemTrayEvent| tray_event(app, event))
        // plugins
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            app.emit_all("new-instance", NewInstancePayload { args, cwd })
                .unwrap();
        }))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![])))
        // setup
        .setup(|app| {
            let hendler = app.handle();
            let config = hendler.state::<Arc<Mutex<serde_obj::ConfigFile>>>();
            if config.blocking_lock().run_on_startup {
                let _ = hendler.autolaunch().enable();
            } else {
                let _ = hendler.autolaunch().disable();
            }
            Ok(())
        })
        .setup(|app| {
            let handle = app.handle();
            let _ = tauri_plugin_deep_link::unregister("aihelper");
            tauri_plugin_deep_link::prepare("aihelper");
            tauri_plugin_deep_link::register("aihelper", move |request| {
                handle.emit_all("scheme-request-received", request).unwrap();
            })
            .unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::md_to_html,
            crate::commands::new_message,
            crate::commands::generate_uuid
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
