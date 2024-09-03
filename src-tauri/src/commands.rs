use std::sync::Arc;
use tauri::{async_runtime::Mutex, Manager, State};
use crate::tokenizer::*;
use crate::api_req::get_response_text;

#[tauri::command]
pub fn md_to_html(text: String) -> Result<String, String> {
    Ok(markdown::to_html(&text))
}

#[tauri::command(async)]
pub async fn new_message(app: tauri::AppHandle, prompt: String) -> Result<(), String> {
    let app_binding = app.clone();
    let messages_mutex: State<Arc<Mutex<Vec<MessageType>>>> = app_binding.state();
    messages_mutex.lock().await.push(MessageType::User(UserMessage { content: prompt }));
    println!("{:#?}", messages_mutex.lock().await);
    let messeges = messages_mutex.lock().await.clone();
    let text = tokenize_messages(messeges);
    get_response_text(text, app).await;
    Ok(())
}
