#![allow(dead_code)]
use serde_json::Value;
use std::collections::HashMap;
use std::marker::Sized;

#[derive(Clone, Debug)]
pub struct Function {
    name: String,
    description: String,
    parameters: Vec<ArgsInfo>
}

impl Function {
    pub fn new(
        name: &str,
        description: &str,
        parameters: Vec<ArgsInfo>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters
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
    pub fn new(
        type_input: &str,
        name: &str,
        description: &str,
        required: bool,
    ) -> Self {
        let types = vec![
            "array", "boolean", "integer", "number", "object", "string",
        ];
        if !types.contains(&type_input) {
            panic!("Invalid type input must be one of those: array, boolean, integer, number, object, string");
        }
        Self {
            type_input: type_input.to_string(),
            description: description.to_string(),
            name: name.to_string(),
            required,
        }
    }

    pub fn to_value(&self) -> (String, bool, Value) {
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
    pub fn new(id: &str) -> Self {
        let check_rg = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        if !check_rg.is_match(&id) {
            panic!("Invalid plugin id: {} (only a-z, A-Z, 0-9 and _ allowed)", id);
        }
        Self {
            id: id.to_string(),
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
        Vec<String>,
    ) {
        let mut commands = Vec::new();
        let mut callbacks = Vec::new();
        for command in self.commands.clone() {
            callbacks.push(command.name.clone());
            commands.push(command.to_value());
        }
        (commands, callbacks)
    }
}

// this help move value from dll to app
#[derive(Clone, Debug)]
pub struct ResultValue {
    value: String
}

impl ResultValue {
    pub fn new<T>(value: T) -> Self 
    where
     T: Sized + serde::Serialize {
        Self {
            value: serde_json::to_string(&value).unwrap()
        }
    }

    pub fn to_value(&self) -> Value {
        serde_json::json!(self.value)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut manager = PluginManager::new("test");
        let mut parameters = Vec::new();
        parameters.push(ArgsInfo::new(
            "string",
            "test",
            "test",
            false,
        ));
        let command = Function::new(
            "test",
            "test",
            parameters
        );
        manager.add_command(command);
        let mut parameters2 = Vec::new();
        parameters2.push(ArgsInfo::new(
            "string",
            "test2",
            "test",
            false,
        ));
        let commamd2 = Function::new(
            "test2",
            "test2",
            parameters2
        );
        manager.add_command(commamd2);
        let (commands, callbacks) = manager.get_commands();
        println!("{}", serde_json::to_string(&commands).unwrap());
        println!("{:?}", callbacks);
    }
}
