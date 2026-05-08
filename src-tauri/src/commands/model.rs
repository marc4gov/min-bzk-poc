//! Model download and management commands

use crate::errors::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::fs::File;
use tauri::Emitter;

/// Available models for download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub size_mb: u64,
    pub description: String,
}

/// Download progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f32,
}

/// Get list of available models
#[tauri::command]
pub async fn get_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "mistral-7b-instruct-q4".to_string(),
            name: "Mistral 7B Instruct (Q4_K_M)".to_string(),
            url: "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string(),
            size_mb: 4371,
            description: "Balanced quality/speed, ~4.3GB".to_string(),
        },
        ModelInfo {
            id: "mistral-7b-instruct-q3".to_string(),
            name: "Mistral 7B Instruct (Q3_K_M)".to_string(),
            url: "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q3_K_M.gguf".to_string(),
            size_mb: 3509,
            description: "Faster, lower quality, ~3.5GB".to_string(),
        },
        ModelInfo {
            id: "phi-3-mini-q4".to_string(),
            name: "Phi-3 Mini (Q4_K_M)".to_string(),
            url: "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf".to_string(),
            size_mb: 2343,
            description: "Small & fast, ~2.3GB".to_string(),
        },
        ModelInfo {
            id: "gemma-2b-q4".to_string(),
            name: "Gemma 2B (Q4_K_M)".to_string(),
            url: "https://huggingface.co/lmstudio-community/Gemma-2B-GGUF/resolve/main/gemma-2b-q4_k_m.gguf".to_string(),
            size_mb: 1626,
            description: "Tiny model, ~1.6GB".to_string(),
        },
    ]
}

/// Get local models directory
fn get_models_dir() -> Result<PathBuf> {
    let mut models_dir = std::env::current_exe()
        .map_err(|e| AppError::InferenceFailed(format!("Failed to get exe path: {}", e)))?;

    // Navigate from .app bundle on macOS
    if cfg!(target_os = "macos") {
        if let Some(parent) = models_dir.parent() {
            if let Some(grandparent) = parent.parent() {
                models_dir = grandparent.to_path_buf();
            }
        }
    }

    models_dir.push("models");
    Ok(models_dir)
}

/// Get list of downloaded models
#[tauri::command]
pub async fn get_downloaded_models() -> Vec<String> {
    let models_dir = match get_models_dir() {
        Ok(dir) => dir,
        Err(_) => return vec![],
    };

    if !models_dir.exists() {
        let _ = std::fs::create_dir_all(&models_dir);
        return vec![];
    }

    let mut models = vec![];
    if let Ok(entries) = std::fs::read_dir(&models_dir) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".gguf") {
                    models.push(name);
                }
            }
        }
    }
    models
}

/// Download a model with progress reporting
#[tauri::command]
pub async fn download_model(
    model_id: String,
    app_handle: tauri::AppHandle,
) -> Result<String> {
    let models = get_available_models().await;
    let model = models.iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| AppError::InferenceFailed("Model not found".to_string()))?;

    let models_dir = get_models_dir()?;
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| AppError::InferenceFailed(format!("Failed to create models dir: {}", e)))?;

    let filename = format!("{}.gguf", model_id);
    let file_path = models_dir.join(&filename);

    if file_path.exists() {
        return Ok(filename);
    }

    // Download with progress
    let response = reqwest::get(&model.url)
        .await
        .map_err(|e| AppError::InferenceFailed(format!("Download failed: {}", e)))?;

    let total_bytes = response.content_length().unwrap_or(model.size_mb * 1024 * 1024);
    let mut downloaded_bytes: u64 = 0;
    let mut file = File::create(&file_path)
        .await
        .map_err(|e| AppError::InferenceFailed(format!("Failed to create file: {}", e)))?;

    let mut stream = response.bytes_stream();

    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(|e| AppError::InferenceFailed(format!("Download chunk error: {}", e)))?;

        file.write_all(&chunk)
            .await
            .map_err(|e| AppError::InferenceFailed(format!("Write error: {}", e)))?;

        downloaded_bytes += chunk.len() as u64;

        // Emit progress event
        let progress = DownloadProgress {
            model_id: model_id.clone(),
            downloaded_bytes,
            total_bytes,
            percentage: (downloaded_bytes as f32 / total_bytes as f32) * 100.0,
        };

        let _ = app_handle.emit("model-download-progress", progress);
    }

    file.flush()
        .await
        .map_err(|e| AppError::InferenceFailed(format!("Flush error: {}", e)))?;

    Ok(filename)
}

/// Delete a model
#[tauri::command]
pub async fn delete_model(model_id: String) -> Result<()> {
    let models_dir = get_models_dir()?;
    let filename = format!("{}.gguf", model_id);
    let file_path = models_dir.join(&filename);

    if file_path.exists() {
        std::fs::remove_file(&file_path)
            .map_err(|e| AppError::InferenceFailed(format!("Failed to delete model: {}", e)))?;
    }

    Ok(())
}
