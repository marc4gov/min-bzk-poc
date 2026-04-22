use crate::services::StorageService;
use crate::errors::Result;
use std::path::PathBuf;
use std::sync::Arc;

pub struct DocumentState {
    pub storage: Arc<StorageService>,
}

#[tauri::command]
pub async fn extract_text(
    file_path: String,
    state: tauri::State<'_, DocumentState>,
) -> Result<String> {
    let path = PathBuf::from(&file_path);

    // For now, just read text files directly
    let content = tokio::fs::read_to_string(&path).await
        .map_err(|e| crate::errors::AppError::Io(e))?;

    // Get filename
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Create preview
    let preview = content.chars().take(500).collect::<String>();

    // Save document
    state.storage.save_document(&filename, path.clone(), &preview).await?;

    Ok(content)
}

#[tauri::command]
pub async fn get_documents(
    state: tauri::State<'_, DocumentState>,
) -> Result<Vec<crate::services::Document>> {
    state.storage.get_documents().await
}
