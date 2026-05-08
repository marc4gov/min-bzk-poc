use crate::services::{StorageService, DefaultEngine, InferenceEngine, GenerationParams};
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
    let mut prompt = String::from("<s>[INST] Je bent een behulpzame AI assistent die Nederlands spreekt. Geef KORTE en directe antwoorden. ");

    // Add conversation history
    for msg in history.messages.iter().take(5) {  // Laatste 5 berichten
        match msg.role.as_str() {
            "user" => {
                prompt.push_str(&format!("{} [/INST]", msg.content));
            }
            "assistant" => {
                prompt.push_str(&format!(" {} </s><s>[INST]", msg.content));
            }
            _ => {}
        }
    }

    // Close the last instruction and prepare for response
    if !prompt.ends_with("[INST] ") {
        prompt.push_str(" </s><s>[INST]");
    }
    prompt.push_str(" ");

    // Generate response using inference engine
    let response = {
        let mut engine = state.engine.lock().await;

        // Ensure model is loaded
        if !engine.is_loaded() {
            // Find any .gguf model file
            let model_dir = std::path::Path::new("models");
            let mut found_model = None;

            // Try different relative paths
            for base in &[".", "..", "../.."] {
                let base_path = std::path::Path::new(base).join("models");
                if let Ok(entries) = std::fs::read_dir(&base_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "gguf") {
                            found_model = Some(path.clone());
                            tracing::info!("Found model at: {}", path.display());
                            break;
                        }
                    }
                    if found_model.is_some() {
                        break;
                    }
                }
            }

            if let Some(model_path) = found_model {
                match engine.load_model(&model_path).await {
                    Ok(_) => {
                        tracing::info!("Model loaded successfully");
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load model: {}", e);
                    }
                }
            } else {
                tracing::warn!("No GGUF model found in models/ directory");
            }
        }

        let params = GenerationParams::default();

        match engine.generate(&prompt, params).await {
            Ok(token_stream) => {
                use futures::StreamExt;
                use tokio::time::{timeout, Duration};
                futures::pin_mut!(token_stream);

                // Collect all tokens from the stream with timeout
                let mut tokens = vec![];
                let collect_timeout = Duration::from_secs(120); // 2 minutes total

                let collect_result = timeout(collect_timeout, async {
                    while let Some(token) = token_stream.next().await {
                        tokens.push(token);
                        if tokens.len() > 2000 {
                            tracing::warn!("Token stream exceeded limit, breaking");
                            break;
                        }
                    }
                }).await;

                match collect_result {
                    Ok(_) => {
                        let resp = tokens.join("");
                        tracing::info!("Collected {} tokens, total length: {}", tokens.len(), resp.len());
                        resp
                    }
                    Err(_) => {
                        tracing::error!("Token collection timed out after 120s");
                        "Sorry, het genereren van een antwoord duurde te lang. Probeer een kortere vraag.".to_string()
                    }
                }
            }
            Err(e) => {
                tracing::error!("Generation failed: {}", e);
                format!("Sorry, er ging iets mis: {}", e)
            }
        }
    };

    // Ensure we always have a response
    let response = if response.is_empty() {
        tracing::warn!("Empty response, using fallback");
        String::from("Hallo! Ik ben je lokale AI assistent. Hoe kan ik je helpen?")
    } else {
        response
    };

    tracing::info!("Final response length: {}", response.len());

    // Save assistant response
    let saved_message = state.storage.add_chat_message(&history_id, "assistant", &response).await?;
    tracing::info!("Saved assistant message with id: {}", saved_message.id);

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

#[tauri::command]
pub async fn delete_all_histories(
    state: tauri::State<'_, ChatState>,
) -> Result<usize> {
    state.storage.delete_all_chat_histories().await
}
