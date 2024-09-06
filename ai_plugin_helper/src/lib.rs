#![allow(dead_code)]
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Function {
    name: String,
    description: String,
    parameters: Vec<ArgsInfo>,
    callback: fn(HashMap<String, Value>) -> String,
}

impl Function {
    pub fn new(
        name: String,
        description: String,
        parameters: Vec<ArgsInfo>,
        callback: fn(HashMap<String, Value>) -> String,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            callback,
        }
    }

    pub fn to_value(&self) -> Value {
        let mut args_required = Vec::new();
        let mut args_info = HashMap::new();
        for arg in self.parameters.clone() {
            let (name, required, obj) = arg.to_value();
            if required {
                args_required.push(name.clone());
            }
            args_info.insert(name, obj);
        }
        let parameters = serde_json::json!({
            "type": "object",
            "properties": args_info,
            "required": args_required
        });
        let command = serde_json::json!({
            "name": self.name,
            "description": self.description,
            "parameters": parameters
        });
        serde_json::json!({
            "type": "function",
            "function": command
        })
    }
}

#[derive(Clone, Debug)]
pub struct ArgsInfo {
    type_input: String,
    description: String,
    name: String,
    required: bool,
}

impl ArgsInfo {
    fn new(
        type_input: String,
        name: String,
        description: String,
        required: bool,
    ) -> Self {
        let types = vec![
            "array", "boolean", "integer", "number", "object", "string",
        ];
        if !types.contains(&type_input.as_str()) {
            panic!("Invalid type input must be one of those: array, boolean, integer, number, object, string");
        }
        Self {
            type_input,
            description,
            name,
            required,
        }
    }

    fn to_value(&self) -> (String, bool, Value) {
        let obj = serde_json::json!({
            "type": self.type_input,
            "description": self.description
        });
        (self.name.clone(), self.required, obj)
    }
}

#[derive(Clone, Debug)]
pub struct PluginManager {
    pub id: String,
    commands: Vec<Function>,
}

impl PluginManager {
    pub fn new(id: String) -> Self {
        let check_rg = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        if !check_rg.is_match(&id) {
            panic!("Invalid plugin id: {} (only a-z, A-Z, 0-9 and _ allowed)", id);
        }
        Self {
            id,
            commands: vec![],
        }
    }

    pub fn add_command(&mut self, command: Function) {
        self.commands.push(command);
    }

    pub fn get_commands(
        &self,
    ) -> (
        Vec<Value>,
        HashMap<String, fn(HashMap<String, Value>) -> String>,
    ) {
        let mut commands = Vec::new();
        let mut callbacks = HashMap::new();
        for command in self.commands.clone() {
            callbacks.insert(command.name.clone(), command.callback);
            commands.push(command.to_value());
        }
        (commands, callbacks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn callback_test(input: HashMap<String, Value>) -> String {
        let _ = input;
        "test".to_string()
    }

    fn callback_test2(input: HashMap<String, Value>) -> String {
        let _ = input;
        "test2".to_string()
    }

    #[test]
    fn it_works() {
        let mut manager = PluginManager::new("test".to_string()).unwrap();
        let mut parameters = Vec::new();
        parameters.push(ArgsInfo::new(
            "string".to_string(),
            "test".to_string(),
            "test".to_string(),
            false,
        ).unwrap());
        let command = Function::new(
            "test".to_string(),
            "test".to_string(),
            parameters,
            callback_test,
        );
        manager.add_command(command);
        let mut parameters2 = Vec::new();
        parameters2.push(ArgsInfo::new(
            "string".to_string(),
            "test2".to_string(),
            "test".to_string(),
            false,
        ).unwrap());
        let commamd2 = Function::new(
            "test2".to_string(),
            "test2".to_string(),
            parameters2,
            callback_test2,
        );
        manager.add_command(commamd2);
        let (commands, callbacks) = manager.get_commands();
        println!("{}", serde_json::to_string(&commands).unwrap());
        println!("{:?}", callbacks);
    }
}
