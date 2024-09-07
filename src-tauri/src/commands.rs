use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::Mutex, Manager, State};
use crate::tokenizer::*;
use crate::api_req::get_response_text;

#[tauri::command]
pub fn md_to_html(text: String) -> Result<String, String> {
    Ok(markdown::to_html(&text))
}

#[tauri::command]
pub fn generate_uuid() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

#[tauri::command(async)]
pub async fn new_message(app: tauri::AppHandle, prompt: String, id: String) -> Result<(), String> {
    let app_binding = app.clone();
    let messages_mutex: State<Arc<Mutex<Vec<MessageType>>>> = app_binding.state();
    messages_mutex.lock().await.push(MessageType::User(UserMessage { content: prompt }));
    let messeges = messages_mutex.lock().await.clone();
    let text = tokenize_messages(messeges, app.clone());
    get_response_text(text, app, id).await;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    is_user: bool,
    content: String
}

#[tauri::command(async)]
pub async fn get_messages(messages: State<'_, Arc<Mutex<Vec<MessageType>>>>) -> Result<Vec<Message>, String> {
    let message = messages.lock().await.clone();
    let mut j_message = Vec::new();
    for message in message.iter() {
        match message {     
            MessageType::User(user_message) => { 
                j_message.push(Message { is_user: true, content: user_message.content.clone() }); 
            },
            MessageType::Assistant(assistant_message) => { 
                j_message.push(Message { is_user: false, content: assistant_message.content.clone() }); 
            },
            _ => {}
        }
    }
    Ok(j_message)
}

