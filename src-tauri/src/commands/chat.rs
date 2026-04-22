use crate::errors::Result;
use crate::services::{Conversation, StorageService, Template};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn create_conversation(
    storage: State<'_, Arc<StorageService>>,
    title: String,
) -> Result<Conversation> {
    storage.create_conversation(&title).await
}

#[tauri::command]
pub async fn get_conversations(
    storage: State<'_, Arc<StorageService>>,
) -> Result<Vec<Conversation>> {
    storage.get_conversations().await
}

#[tauri::command]
pub async fn get_conversation(
    storage: State<'_, Arc<StorageService>>,
    id: String,
) -> Result<Option<Conversation>> {
    storage.get_conversation(&id).await
}

#[tauri::command]
pub async fn add_message(
    storage: State<'_, Arc<StorageService>>,
    conversation_id: String,
    role: String,
    content: String,
) -> Result<()> {
    storage.add_message(&conversation_id, &role, &content).await
}

#[tauri::command]
pub async fn delete_conversation(
    storage: State<'_, Arc<StorageService>>,
    id: String,
) -> Result<()> {
    storage.delete_conversation(&id).await
}

#[tauri::command]
pub async fn get_templates(
    storage: State<'_, Arc<StorageService>>,
) -> Result<Vec<Template>> {
    storage.get_templates().await
}
