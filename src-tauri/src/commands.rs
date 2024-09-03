use std::sync::Arc;
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
    let text = tokenize_messages(messeges);
    get_response_text(text, app, id).await;
    Ok(())
}
