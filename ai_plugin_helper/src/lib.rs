#![allow(dead_code)]
use std::collections::HashMap;

use regex::Regex;
use serde_json::Value;

#[derive(Debug)]
struct PluginHelper {
    name: String,
    id: String,
    callback: Option<fn (id: &str, name: &str, kargs: HashMap<&str, Value>) -> Value>,
}

impl PluginHelper {
    pub fn new(name: &str, id: &str) -> Result<Self, &'static str> {
        let re = Regex::new(r"^[a-z0-9_]+$").unwrap();
        if !re.is_match(&id) {
            return Err("Invalid ID");
        }
        Ok(Self { name: name.to_owned(), id: id.to_owned(), callback: None })
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::PluginHelper;

    #[test]
    fn it_works() {
        let a = PluginHelper::new("Google", "googe_12321");
        println!("{:?}", a);
        assert!(a.is_ok());
    }
}
