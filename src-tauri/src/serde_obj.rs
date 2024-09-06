use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri_plugin_autostart::ManagerExt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigFile {
    #[serde(default)]
    pub run_on_startup: bool,
    pub save_on_close: bool,
    pub plugins: HashMap<String, Value>,
}

impl ConfigFile {

    pub fn set_plugin_config(&mut self, id: String, value: Value) {
        self.plugins.insert(id, value);
    }

    pub fn save_to_file(self, path: &PathBuf, app: Option<tauri::AppHandle>) {
        if let Some(app) = app {
            if self.run_on_startup {
                let _ = app.autolaunch().enable();
            } else {
                let _ = app.autolaunch().disable();
            }
        }
        std::fs::write(path, serde_json::to_string(&self).unwrap()).unwrap();
    }
}

#[derive(Clone, serde::Serialize)]
pub struct NewInstancePayload {
    pub args: Vec<String>,
    pub cwd: String,
}

#[derive(Clone, serde::Serialize)]
pub struct MessageEventPayload {
    pub data: String,
    pub uuid: String,
}