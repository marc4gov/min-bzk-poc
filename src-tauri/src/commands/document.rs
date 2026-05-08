use crate::services::{StorageService, extract_text_from_file, DocumentError};
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

pub struct DocumentState {
    pub storage: Arc<StorageService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocumentRequest {
    pub file_path: String,
    pub instructions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocumentResponse {
    pub filename: String,
    pub content: String,
    pub file_type: String,
    pub preview: String,
}

#[tauri::command]
pub async fn upload_document_for_improvement(
    request: UploadDocumentRequest,
) -> std::result::Result<UploadDocumentResponse, String> {
    let extracted = extract_text_from_file(&request.file_path)
        .map_err(|e| format!("Kon tekst niet extraheren: {}", e))?;

    // Create preview (first 500 chars)
    let preview = if extracted.content.len() > 500 {
        format!("{}...", &extracted.content[..500])
    } else {
        extracted.content.clone()
    };

    Ok(UploadDocumentResponse {
        filename: extracted.filename,
        content: extracted.content,
        file_type: extracted.file_type,
        preview,
    })
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
