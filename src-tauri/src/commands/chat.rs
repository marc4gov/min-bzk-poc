use crate::services::{StorageService, DefaultEngine, InferenceEngine, GenerationParams};
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub history_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub history_id: String,
    pub message_id: String,
}

pub struct ChatState {
    pub storage: Arc<StorageService>,
    pub engine: Arc<Mutex<DefaultEngine>>,
}

#[tauri::command]
pub async fn send_message(
    request: ChatRequest,
    state: tauri::State<'_, ChatState>,
) -> Result<ChatResponse> {
    let history_id = match request.history_id {
        Some(id) => id,
        None => {
            let history = state.storage.create_chat_history("Nieuw gesprek").await?;
            history.id
        }
    };

    // Save user message
    state.storage.add_chat_message(&history_id, "user", &request.message).await?;

    // Build prompt from conversation history
    let history = state.storage.get_chat_history(&history_id).await?.unwrap();
    let mut prompt = String::from("<|im_start|>system\nJe bent een behulpzame AI assistent die Nederlands spreekt.<|im_end|>\n");

    for msg in history.messages {
        match msg.role.as_str() {
            "user" => {
                prompt.push_str(&format!("<|im_start|>user\n{}<|im_end|>\n", msg.content));
            }
            "assistant" => {
                prompt.push_str(&format!("<|im_start|>assistant\n{}<|im_end|>\n", msg.content));
            }
            _ => {}
        }
    }
    prompt.push_str("<|im_start|>assistant\n");

    // Generate response using inference engine
    let engine = state.engine.lock().await;
    let params = GenerationParams::default();

    let mut response = String::new();
    let mut token_stream = engine.generate(&prompt, params).await?;

    while let Some(token) = token_stream.next().await {
        response.push_str(&token);
    }

    // Save assistant response
    state.storage.add_chat_message(&history_id, "assistant", &response).await?;

    Ok(ChatResponse {
        history_id,
        message_id: uuid::Uuid::new_v4().to_string(),
    })
}

#[tauri::command]
pub async fn get_histories(
    state: tauri::State<'_, ChatState>,
) -> Result<Vec<crate::services::ChatHistory>> {
    state.storage.get_chat_histories().await
}

#[tauri::command]
pub async fn get_history(
    id: String,
    state: tauri::State<'_, ChatState>,
) -> Result<Option<crate::services::ChatHistory>> {
    state.storage.get_chat_history(&id).await
}

#[tauri::command]
pub async fn delete_history(
    id: String,
    state: tauri::State<'_, ChatState>,
) -> Result<()> {
    state.storage.delete_chat_history(&id).await
}
