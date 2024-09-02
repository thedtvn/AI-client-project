use std::sync::Arc;
use tauri::{async_runtime::Mutex, Manager, State};
use crate::tokenizer::*;

#[tauri::command]
pub fn md_to_html(text: String) -> Result<String, String> {
    Ok(markdown::to_html(&text))
}

#[tauri::command(async)]
pub async fn new_message(app: tauri::AppHandle, prompt: String) -> Result<(), String> {
    println!("Prompt: {}", prompt);
    let app_binding = app.clone();
    let messages: State<Arc<Mutex<Vec<MessageType>>>> = app_binding.state();
    println!("Messages: {:?}", messages);
    let mut messages = messages.blocking_lock();
    messages.push(MessageType::User(UserMessage { content: prompt }));
    let text = tokenize_messages(messages.clone());
    println!("Text: {}", text);
    app.emit_all("message", text).unwrap();
    tauri::async_runtime::spawn(async move {
        app.emit_all("message", "test").unwrap(); 
    });
    Ok(())
}
