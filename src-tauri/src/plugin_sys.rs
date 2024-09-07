use std::collections::HashMap;

use dlopen2::wrapper::{Container, WrapperApi};
use rasast_plugin::PluginManager;
use serde_json::Value;

use crate::get_dir;

fn get_plugin_file_ext() -> String {
    let os_name = std::env::consts::OS;
    let file_extension;
    if os_name == "windows" {
        file_extension = ".dll";
    } else if os_name == "macos" {
        file_extension = ".dylib";
    } else if os_name == "linux" {
        file_extension = ".so";
    } else {
        file_extension = "";
    }
    file_extension.to_string()
}

#[derive(WrapperApi)]
struct Plugin {
    init: fn() -> PluginManager
}

pub fn load_plugin() -> (
    Vec<Value>,
    HashMap<String, fn(HashMap<String, Value>) -> Value>,
) {
    let plugin_dir = get_dir().join("plugins");
    let mut plugin_info = Vec::new();
    let mut plugin_callback = HashMap::new();
    if !plugin_dir.exists() {
        eprintln!("create plugin dir");
        let create_dir_r = std::fs::create_dir(&plugin_dir);
        if create_dir_r.is_err() {
            eprintln!("create plugin dir error skip load plugin");
            return (plugin_info, plugin_callback);
        }
    }
    let plugins_dir_list = std::fs::read_dir(plugin_dir);
    if plugins_dir_list.is_err() {
        eprintln!("read plugin dir error skip load plugin");
        return (plugin_info, plugin_callback);
    }
    let file_extension = get_plugin_file_ext();
    for file in plugins_dir_list.unwrap() {
        let path = file.unwrap().path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        if !file_name.ends_with(file_extension.as_str()) { continue; }

        let plugin_r = unsafe { Container::load(path.clone()) };
        if plugin_r.is_err() {
            eprintln!("load plugin {} error", file_name);
            continue;
        }
        let plugin: Container<Plugin> = plugin_r.unwrap();
        let plugin_manager = plugin.init();
        let (plugin_info_r, plugin_callback_r) = plugin_manager.get_commands();
        plugin_info.extend(plugin_info_r);
        plugin_callback.extend(plugin_callback_r);
    }
    println!("load plugin {}", plugin_info.len());
    (plugin_info, plugin_callback)
}
