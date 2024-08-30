use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::get_dir;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigFile {
    #[serde(default)]
    pub run_on_startup: bool,
    pub plugins: HashMap<String, HashMap<String, Value>>,
}

impl ConfigFile {
    pub fn save_to_file(self) {
        let file_path = get_dir().join("config.json");
        std::fs::write(file_path, serde_json::to_string(&self).unwrap()).unwrap();
    }
}