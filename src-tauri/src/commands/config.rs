use crate::services::ConfigService;
use crate::errors::Result;
use std::sync::Arc;

pub struct ConfigState {
    pub config: Arc<tokio::sync::Mutex<ConfigService>>,
}

#[tauri::command]
pub async fn get_config(
    state: tauri::State<'_, ConfigState>,
) -> Result<crate::services::AppConfig> {
    let config = state.config.lock().await;
    Ok(config.get().clone())
}

#[tauri::command]
pub async fn update_config(
    updates: serde_json::Value,
    state: tauri::State<'_, ConfigState>,
) -> Result<()> {
    let mut config = state.config.lock().await;
    config.update(|c| {
        if let Some(temp) = updates.get("temperature").and_then(|v| v.as_f64()) {
            c.temperature = temp as f32;
        }
        if let Some(top_p) = updates.get("top_p").and_then(|v| v.as_f64()) {
            c.top_p = top_p as f32;
        }
        if let Some(max_tokens) = updates.get("max_tokens").and_then(|v| v.as_u64()) {
            c.max_tokens = max_tokens as usize;
        }
    }).await
}

#[tauri::command]
pub async fn get_templates(
    _state: tauri::State<'_, ConfigState>,
) -> Result<Vec<crate::services::Template>> {
    // Templates are in storage, need to refactor this
    // For now, return empty
    Ok(vec![])
}
