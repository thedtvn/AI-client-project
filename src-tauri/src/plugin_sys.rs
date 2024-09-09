use std::{collections::HashMap, path::PathBuf, sync::Arc};

use dlopen2::symbor::Library;
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

#[derive(Clone)]
pub struct PluginCore {
    plugin_lib: HashMap<String, Arc<Library>>,
    map_func: HashMap<String, String>,
    plugin_info: Vec<Value>
}

impl PluginCore {
    pub fn new() -> Self {
        Self {
            plugin_lib: HashMap::new(),
            map_func: HashMap::new(),
            plugin_info: Vec::new(),
        }
    }

    pub fn add_plugin(&mut self, file_path: PathBuf, file_name: String) -> Result<(), String> {
        let plugin_r = Library::open(file_path);
        if plugin_r.is_err() {
            eprintln!("load plugin {:?} error", plugin_r.err().unwrap());
            return Err(format!("load plugin {} error", file_name));
        }
        let plugin = plugin_r.unwrap();
        let init_func_r = unsafe { plugin.symbol::<fn() -> PluginManager>("init") };
        if init_func_r.is_err() {
            return Err(format!("load plugin {} error init func not found", file_name));
        }
        let init_func = init_func_r.unwrap()();
        self.plugin_lib.insert(init_func.id.clone(), Arc::new(plugin));
        let (info_vec, callback_list) = init_func.get_commands();
        self.plugin_info.extend(info_vec);
        for i in callback_list {
            self.map_func.insert(i, init_func.id.clone());
        }
        Ok(())
    }

    pub fn get_plugin_info(&self) -> Vec<Value> {
        self.plugin_info.clone()
    }

    pub fn call_fn(&self, name: &str, args: HashMap<String, Value>) -> Value {
        let id = self.map_func.get(name).unwrap();
        let plugin = self.plugin_lib.get(id).unwrap();
        let func_r = unsafe { plugin.symbol::<fn(HashMap<String, Value>) -> Value>(name) };
        if func_r.is_err() {
            panic!("func not found");
        }
        let func = func_r.unwrap();
        let r = func(args);
        r
    }
}


pub fn load_plugin() -> PluginCore {
    let plugin_dir = get_dir().join("plugins");
    let mut plugin_core = PluginCore::new();
    if !plugin_dir.exists() {
        eprintln!("create plugin dir");
        let create_dir_r = std::fs::create_dir(&plugin_dir);
        if create_dir_r.is_err() {
            eprintln!("create plugin dir error skip load plugin");
            return plugin_core;
        }
    }
    let plugins_dir_list = std::fs::read_dir(plugin_dir);
    if plugins_dir_list.is_err() {
        eprintln!("read plugin dir error skip load plugin");
        return plugin_core;
    }
    let file_extension = get_plugin_file_ext();
    for file in plugins_dir_list.unwrap() {
        let path = file.unwrap().path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        if !file_name.ends_with(file_extension.as_str()) { continue; }
        let err = plugin_core.add_plugin(path.clone(), file_name.to_string());
        if err.is_err() {
            eprintln!("{}", err.err().unwrap());
            continue;
        }
    }
    println!("load plugin {} success", plugin_core.get_plugin_info().len());
    plugin_core
}
