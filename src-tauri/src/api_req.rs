use std::sync::Arc;

use eventsource_stream::EventStream;
use futures_core::future::BoxFuture;
use futures_util::{FutureExt as _, StreamExt as _};
use reqwest::header;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::Mutex, Manager as _, State};

use crate::{plugin_sys::PluginCore, serde_obj::MessageEventPayload, tokenizer::*, utility::{get_response_token, prase_tool_call}};

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

// this is to make it can recursion async
pub fn get_response_text(promt: String, app: tauri::AppHandle, id: String) -> BoxFuture<'static, ()> {
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
                    continue;
                } else if token.special && token.text == "</s>" {
                    break;
                } else if token.special && vec!["<unk>"].contains(&token.text.as_str()) {
                    let _ = app.emit_all(
                        "message",
                        MessageEventPayload {
                            data: "Unknown ?".to_string(),
                            uuid: messages_uuid.clone(),
                        },
                    );
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
    let tool_calls_r = prase_tool_call(vec.join(""));
    if tool_calls_r.is_err() {
        return;
    }
    let tool_calls = tool_calls_r.unwrap();
    let tool_call_str = serde_json::to_string(&tool_calls).unwrap();
    messages.push(MessageType::ToolCall(ToolCall { content: tool_call_str }));
    let p_callbacks: State<PluginCore> = app.state();
    for tool_call in tool_calls {
        let tool_response = p_callbacks.call_fn(&tool_call.name, tool_call.arguments.clone());
        messages.push(MessageType::ToolResponse(ToolResponse { content: tool_response.to_value(), call_id: tool_call.call_id }));
    }
    let promt = tokenize_messages(messages.clone(), app.clone());
    drop(messages);
    get_response_text(promt, app.clone(), messages_uuid).await
}