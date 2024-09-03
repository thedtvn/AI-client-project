use std::sync::Arc;

use eventsource_stream::EventStream;
use futures_core::future::BoxFuture;
use futures_util::{FutureExt as _, StreamExt as _};
use reqwest::header;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::Mutex, Manager as _, State};

use crate::{serde_obj::MessageEventPayload, tokenizer::*};

async fn crate_client() -> reqwest::Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "User-Agent",
        header::HeaderValue::from_static("Fopilot/1.0"),
    );
    let client = reqwest::Client::builder().default_headers(headers);
    client.build().unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReqestEventID {
    pub data: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseEventID {
    pub event_id: String,
}

async fn get_event_id(client: reqwest::Client, promt: String) -> reqwest::Result<String> {
    let body = ReqestEventID { data: vec![promt] };
    let req = client
        .post("https://thedtvn-local-ai-helper.hf.space/call/predict")
        .json(&body)
        .send()
        .await?;
    let id: ResponseEventID = req.json().await?;
    Ok(id.event_id)
}

async fn get_response(
    client: reqwest::Client,
    event_id: String,
) -> reqwest::Result<impl futures_core::Stream<Item = reqwest::Result<bytes::Bytes>>> {
    let req = client
        .get(format!(
            "https://thedtvn-local-ai-helper.hf.space/call/predict/{event_id}"
        ))
        .send()
        .await?;
    Ok(req.bytes_stream())
}

pub(crate) fn get_response_text(promt: String, app: tauri::AppHandle, id: String) -> BoxFuture<'static, ()> {
    async move {
        get_response_text_async(promt, app, id).await;
    }.boxed()
}

async fn get_response_text_async(promt: String, app: tauri::AppHandle, messages_uuid: String) {
    let client = crate_client().await;
    let event_id = get_event_id(client.clone(), promt).await.unwrap();
    let res = get_response(client, event_id).await.unwrap();
    let mut stream = EventStream::new(res);
    let mut is_tool_call = false;
    let mut vec = Vec::new();
    let mut index = 0;
    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                if event.event != "generating" {
                    break;
                };
                let token = get_response_token(event.data);
                if token.special && index == 0 && token.text == "[TOOL_CALLS]" {
                    is_tool_call = true;
                } else if token.special && token.text == "</s>" {
                    break;
                }
                index += 1;
                vec.push(token.text);
                if is_tool_call {
                    continue;
                }
                app.emit_all(
                    "message",
                    MessageEventPayload {
                        data: vec.clone().join(""),
                        uuid: messages_uuid.clone(),
                    },
                )
                .unwrap();
            }
            Err(e) => break,
        }
    }
    let messages: State<Arc<Mutex<Vec<MessageType>>>> = app.state();
    let mut messages = messages.lock().await; 
    if !is_tool_call {
        messages.push(MessageType::Assistant(AssistantMessage { content: vec.join("") }));
        return;
    }
    messages.push(MessageType::ToolCall(ToolCall { content: vec.join("") }));
    let tool_response = get_tool_response(vec.join(""), app.clone());
    messages.push(MessageType::ToolResponse(ToolResponse { content: tool_response }));
    let clone_messages = messages.clone();
    drop(messages);
    let promt = tokenize_messages(clone_messages);
    get_response_text(promt, app, messages_uuid).await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub text: String,
    pub special: bool,
}

fn get_response_token(data: String) -> TokenResponse {
    let vec_data: Vec<String> = serde_json::from_str(&data).unwrap();
    let json_str_data = vec_data[0].clone();
    serde_json::from_str(&json_str_data).unwrap()
}

fn get_tool_response(input: String, app: tauri::AppHandle) -> String {
    "".to_string()
}