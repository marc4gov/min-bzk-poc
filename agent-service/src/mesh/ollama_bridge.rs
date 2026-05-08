//! Ollama HTTP (`/api/chat`) voor mesh-experts; zelfde base/model als `crate::registry`.

use serde::Deserialize;
use serde_json::json;
use std::sync::RwLock;
use tokio::sync::broadcast;

use crate::registry::{default_ollama_model, ollama_base_url};

/// Global model override for mesh operations (set via HTTP request).
/// Uses RwLock for thread-safe access.
static MESH_MODEL_OVERRIDE: RwLock<Option<String>> = RwLock::new(None);

/// Set the model to use for all mesh operations.
pub fn set_mesh_model(model: Option<String>) {
    let mut guard = MESH_MODEL_OVERRIDE.write().unwrap();
    *guard = model;
}

/// Get the model override if set, otherwise None.
pub fn get_mesh_model() -> Option<String> {
    let guard = MESH_MODEL_OVERRIDE.read().unwrap();
    guard.clone()
}

/// Emit een model-event naar de mesh logs.
pub fn emit_model_event(
    tx: &Option<broadcast::Sender<crate::AgentEvent>>,
    expert_name: &str,
    model: &str,
) {
    let Some(sender) = tx else {
        return;
    };
    let _ = sender.send(crate::AgentEvent::Mesh {
        actor: expert_name.to_string(),
        phase: "model".to_string(),
        detail: format!("Gebruikt model: {}", model),
        model: Some(model.to_string()),
        timestamp: chrono::Utc::now().timestamp(),
    });
}

/// Standaard `true`; zet `MESH_USE_OLLAMA=0` om aleen heuristiek te gebruiken (snellere/tests).
pub fn mesh_ollama_enabled() -> bool {
    match std::env::var("MESH_USE_OLLAMA") {
        Ok(v) => {
            let t = v.to_lowercase();
            !(t.is_empty() || t == "0" || t == "false" || t == "off")
        }
        Err(_) => true,
    }
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaMessage>,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

fn system_prompt_for_expert(expert_name: &str) -> &'static str {
    match expert_name {
        "RustExpert" => {
            "Je bent een beknopte Rust-programmeur-assistent. Antwoord helder en kort, desnoods in het Nederlands."
        }
        "FrontendExpert" => {
            "Je bent een beknopte frontend/UI-assistent (React, CSS, WASM). Antwoord helder en kort, desnoods in het Nederlands."
        }
        "SchrijverExpert" => {
            "Je bent Nederlandstalig redacteur. Je antwoordt en schrijft ALTIJD volledig in het Nederlands: \
             geen Engelse zinnen, geen door elkaar gehakte talen, behalve vast product- of bedrijfsnamen \
             die letterlijk in de bron staan. Compact en feitelijk, zonder excuses of meta-commentaar. \
             Samenvattingen: hooguit tien tekstregels in de output (regeleinden als harde grens). \
             Verzin geen feiten die niet in de bron staan."
        }
        _ => "Je bent een beknopte technische assistent.",
    }
}

/// Eén non-stream chat-call naar lokale Ollama.
pub async fn chat_via_ollama_for_mesh(
    expert_name: &str,
    user_prompt: &str,
    model_override: Option<&str>,
) -> Result<String, String> {
    let base_owned = ollama_base_url();
    let base = base_owned.trim_end_matches('/');

    // Priority: 1. model_override parameter, 2. global MESH_MODEL_OVERRIDE, 3. default
    // We need to own the model string for the json body, so we collect it here
    let model_owned = if let Some(m) = model_override {
        m.to_string()
    } else if let Some(global_model) = get_mesh_model() {
        global_model
    } else {
        default_ollama_model()
    };

    let url = format!("{}/api/chat", base);

    // Log which model is being used for this mesh operation
    tracing::info!(
        expert = expert_name,
        model = %model_owned,
        "Mesh calling Ollama"
    );

    let body = json!({
        "model": model_owned,
        "messages": [
            { "role": "system", "content": system_prompt_for_expert(expert_name) },
            { "role": "user", "content": user_prompt.trim() },
        ],
        "stream": false,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(240))
        .build()
        .map_err(|e| format!("reqwest builder: {}", e))?;

    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama chat request: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Ollama HTTP {} — {}",
            status,
            txt.chars().take(400).collect::<String>()
        ));
    }

    let parsed: OllamaChatResponse = resp
        .json()
        .await
        .map_err(|e| format!("parse json: {}", e))?;

    let content = parsed
        .message
        .map(|m| m.content)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "Geen assistant-tekst in Ollama-response".to_string())?;

    Ok(content.trim().to_owned())
}

/// Chat-call met live event emission (voor experts die toegang hebben tot mesh_events).
pub async fn chat_via_ollama_for_mesh_with_events(
    expert_name: &str,
    user_prompt: &str,
    model_override: Option<&str>,
    mesh_events: &Option<broadcast::Sender<crate::AgentEvent>>,
) -> Result<String, String> {
    let base_owned = ollama_base_url();
    let base = base_owned.trim_end_matches('/');

    // Priority: 1. model_override parameter, 2. global MESH_MODEL_OVERRIDE, 3. default
    let model_owned = if let Some(m) = model_override {
        m.to_string()
    } else if let Some(global_model) = get_mesh_model() {
        global_model
    } else {
        default_ollama_model()
    };

    let url = format!("{}/api/chat", base);

    // Emit model event to logs
    emit_model_event(mesh_events, expert_name, &model_owned);

    let body = json!({
        "model": model_owned,
        "messages": [
            { "role": "system", "content": system_prompt_for_expert(expert_name) },
            { "role": "user", "content": user_prompt.trim() },
        ],
        "stream": false,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(240))
        .build()
        .map_err(|e| format!("reqwest builder: {}", e))?;

    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama chat request: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Ollama HTTP {} — {}",
            status,
            txt.chars().take(400).collect::<String>()
        ));
    }

    let parsed: OllamaChatResponse = resp
        .json()
        .await
        .map_err(|e| format!("parse json: {}", e))?;

    let content = parsed
        .message
        .map(|m| m.content)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "Geen assistant-tekst in Ollama-response".to_string())?;

    Ok(content.trim().to_owned())
}
