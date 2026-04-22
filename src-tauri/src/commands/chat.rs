use crate::services::{StorageService, MLXEngine};
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub engine: Arc<Mutex<MLXEngine>>,
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

    // TODO: Implement actual inference streaming
    let response = "Dit is een tijdelijke reactie. MLX integratie volgt.";
    state.storage.add_chat_message(&history_id, "assistant", response).await?;

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
