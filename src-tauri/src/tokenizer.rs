use serde::{Deserialize, Serialize};
use serde_json::Value;


#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MessageType {
    User(UserMessage),
    Assistant(AssistantMessage),
    System(SystemMessage),
    ToolResponse(ToolResponse),
    ToolCall(ToolCall)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UserMessage {
    pub content: String
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AssistantMessage {
    pub content: String
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SystemMessage {
    pub content: String
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ToolResponse {
    pub content: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ToolCall {
    pub content: String
}

fn find_first_last_user(messages: Vec<MessageType>) -> (i32, i32) {
    let mut last_user_idx = -1;
    let mut first_user_idx = -1;

    for (idx, message) in messages.iter().enumerate() {
        if let MessageType::User(_) = message {
            if first_user_idx == -1 {
                first_user_idx = idx as i32;
            }
            last_user_idx = idx as i32;
        }
    }
    (first_user_idx, last_user_idx)
}

fn get_filtered_messages(messages: Vec<MessageType>) -> (Vec<SystemMessage>, Vec<MessageType>) {
    let mut system_messages = Vec::new();
    let mut filtered_messages = Vec::new();
    for message in messages {
        if let MessageType::System(system_message) = message {
            system_messages.push(system_message);
        } else {
            filtered_messages.push(message);
        }
    }
    (system_messages, filtered_messages)
}

pub fn tokenize_messages(mut messages: Vec<MessageType>, app: tauri::AppHandle) -> String {
    inject_system_prompt(&mut messages);
    let mut text = String::new();
    text.push_str("<s>");
    let (system_messages, filtered_messages) = get_filtered_messages(messages.clone());
    let (first_user_idx, last_user_idx) = find_first_last_user(messages.clone());
    for (idx, message) in filtered_messages.iter().enumerate() {
        let idx = idx as i32;
        if let MessageType::User(user_message) = message {
            text.push_str("[INST]");
            if idx <= last_user_idx {
                if !system_messages.is_empty() {
                    let system_messages_str = system_messages.iter().map(|s| s.content.clone()).collect::<Vec<String>>().join("<0x0A><0x0A>");
                    text.push_str(&system_messages_str);
                    text.push_str("<0x0A><0x0A>");
                }
            }
            text.push_str(&user_message.content);
            text.push_str("[/INST]");
        } else if let MessageType::Assistant(assistant_message) = message {
            text.push_str(&assistant_message.content);
            text.push_str("</s><s>");
        } else if let MessageType::ToolCall(tool_call) = message {
            text.push_str("[TOOL_CALLS]");
            text.push_str(&tool_call.content);
            text.push_str("</s><s>");
        } else if let MessageType::ToolResponse(tool_response) = message {
            text.push_str("[TOOL_RESULTS]");
            text.push_str(serde_json::to_string(&tool_response).unwrap().as_str());
            text.push_str("[/TOOL_RESULTS]");
        }
    }
    text.replace(" ", "▁")
}

fn inject_system_prompt(messages: &mut Vec<MessageType>) {
    let sys_mess = "Always assist with care, respect, and truth. Respond with utmost utility yet securely. Markdown is allowed.
Avoid harmful, unethical, prejudiced, or negative content. Ensure replies promote fairness and positivity.
You're a helpful assistant Name \"Rasast\".";
    messages.insert( 0, MessageType::System(SystemMessage { content: sys_mess.to_string() })); 
}
