use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const AGENT_SERVICE_URL: &str = "http://127.0.0.1:8080";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatRequest {
    pub message: String,
    pub selected_agent: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatResponse {
    pub content: String,
    pub agents_used: Vec<String>,
    pub tokens_used: u32,
    pub model_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub selectable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealthResponse {
    pub status: String,
    pub ollama_available: bool,
    pub openai_available: bool,
}

#[tauri::command]
pub async fn agent_chat(request: AgentChatRequest) -> Result<AgentChatResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/chat", AGENT_SERVICE_URL);

    let resp = tokio::time::timeout(
        Duration::from_secs(120),
        client
            .post(&url)
            .json(&request)
            .send()
    ).await
    .map_err(|_| crate::errors::AppError::InferenceFailed(
        "Agent service reageert niet (timeout)".to_string()
    ))?
    .map_err(|e| crate::errors::AppError::InferenceFailed(
        format!("Agent service request failed: {}", e)
    ))?;

    if !resp.status().is_success() {
        return Err(crate::errors::AppError::InferenceFailed(
            format!("Agent service error: {}", resp.status())
        ));
    }

    let response = resp
        .json::<AgentChatResponse>()
        .await
        .map_err(|e| crate::errors::AppError::InferenceFailed(
            format!("Failed to parse agent response: {}", e)
        ))?;

    Ok(response)
}

#[tauri::command]
pub async fn list_agents() -> Result<Vec<AgentInfo>> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/agents", AGENT_SERVICE_URL);

    let resp = tokio::time::timeout(
        Duration::from_secs(5),
        client.get(&url).send()
    ).await
    .map_err(|_| crate::errors::AppError::InferenceFailed(
        "Agent service niet bereikbaar".to_string()
    ))?
    .map_err(|e| crate::errors::AppError::InferenceFailed(
        format!("Agent service request failed: {}", e)
    ))?;

    if !resp.status().is_success() {
        return Err(crate::errors::AppError::InferenceFailed(
            format!("Agent service error: {}", resp.status())
        ));
    }

    let agents = resp
        .json::<Vec<AgentInfo>>()
        .await
        .map_err(|e| crate::errors::AppError::InferenceFailed(
            format!("Failed to parse agents response: {}", e)
        ))?;

    Ok(agents)
}

#[tauri::command]
pub async fn agent_health() -> Result<AgentHealthResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}/health", AGENT_SERVICE_URL);

    let resp = tokio::time::timeout(
        Duration::from_secs(10),
        client.get(&url).send()
    ).await
    .map_err(|_| crate::errors::AppError::InferenceFailed(
        "Agent service niet bereikbaar".to_string()
    ))?
    .map_err(|e| crate::errors::AppError::InferenceFailed(
        format!("Agent service request failed: {}", e)
    ))?;

    if !resp.status().is_success() {
        return Ok(AgentHealthResponse {
            status: "unavailable".to_string(),
            ollama_available: false,
            openai_available: false,
        });
    }

    let health = resp
        .json::<AgentHealthResponse>()
        .await
        .map_err(|e| crate::errors::AppError::InferenceFailed(
            format!("Failed to parse health response: {}", e)
        ))?;

    Ok(health)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDemoResult {
    pub ok: bool,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDemoRequest {
    pub query: String,
    #[serde(default = "mesh_demo_default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub full_mesh: bool,
}

fn mesh_demo_default_timeout() -> u64 {
    90
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDocumentRequest {
    pub urls: Vec<String>,
    #[serde(default = "mesh_document_default_timeout")]
    pub timeout_secs: u64,
}

fn mesh_document_default_timeout() -> u64 {
    120
}

#[tauri::command]
pub async fn mesh_demo(request: MeshDemoRequest) -> Result<MeshDemoResult> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/mesh/demo", AGENT_SERVICE_URL);

    let wall_secs = request.timeout_secs.max(5).min(240).saturating_add(60);

    let resp = tokio::time::timeout(
        Duration::from_secs(wall_secs),
        client.post(&url).json(&request).send(),
    )
    .await
    .map_err(|_| {
        crate::errors::AppError::InferenceFailed(
            "Agent service (mesh demo) reageert niet (timeout)".into(),
        )
    })?
    .map_err(|e| {
        crate::errors::AppError::InferenceFailed(format!("Mesh demo request failed: {}", e))
    })?;

    if !resp.status().is_success() {
        return Err(crate::errors::AppError::InferenceFailed(format!(
            "Mesh demo error: {}",
            resp.status()
        )));
    }

    let out = resp
        .json::<MeshDemoResult>()
        .await
        .map_err(|e| crate::errors::AppError::InferenceFailed(format!("Parse mesh demo: {}", e)))?;

    Ok(out)
}

#[tauri::command]
pub async fn mesh_document(request: MeshDocumentRequest) -> Result<MeshDemoResult> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/mesh/demo/document", AGENT_SERVICE_URL);

    let wall_secs = request.timeout_secs.max(15).min(600).saturating_add(120);

    let resp = tokio::time::timeout(
        Duration::from_secs(wall_secs),
        client.post(&url).json(&request).send(),
    )
    .await
    .map_err(|_| {
        crate::errors::AppError::InferenceFailed(
            "Agent service (mesh document) reageert niet (timeout)".into(),
        )
    })?
    .map_err(|e| {
        crate::errors::AppError::InferenceFailed(format!(
            "Mesh document request failed: {}",
            e
        ))
    })?;

    if !resp.status().is_success() {
        return Err(crate::errors::AppError::InferenceFailed(format!(
            "Mesh document error: {}",
            resp.status()
        )));
    }

    let out = resp.json::<MeshDemoResult>().await.map_err(|e| {
        crate::errors::AppError::InferenceFailed(format!("Parse mesh document: {}", e))
    })?;

    Ok(out)
}
#[tauri::command]
pub async fn ollama_models() -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/ollama/models", AGENT_SERVICE_URL);

    let resp = tokio::time::timeout(
        Duration::from_secs(10),
        client.get(&url).send()
    ).await
    .map_err(|_| crate::errors::AppError::InferenceFailed(
        "Agent service reageert niet (timeout)".to_string()
    ))?
    .map_err(|e| crate::errors::AppError::InferenceFailed(
        format!("Agent service request failed: {}", e)
    ))?;

    if !resp.status().is_success() {
        return Err(crate::errors::AppError::InferenceFailed(
            format!("Agent service error: {}", resp.status())
        ));
    }

    let v = resp.json::<serde_json::Value>().await.map_err(|e| {
        crate::errors::AppError::InferenceFailed(format!("Failed to parse ollama models: {}", e))
    })?;

    Ok(v)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshImproveRequest {
    pub content: String,
    pub filename: String,
    pub instructions: String,
    #[serde(default = "mesh_improve_default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub model: Option<String>,
}

fn mesh_improve_default_timeout() -> u64 {
    120
}

#[tauri::command]
pub async fn mesh_improve_document(request: MeshImproveRequest) -> Result<MeshDemoResult> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/mesh/improve", AGENT_SERVICE_URL);

    let wall_secs = request.timeout_secs.max(15).min(600).saturating_add(120);

    let resp = tokio::time::timeout(
        Duration::from_secs(wall_secs),
        client.post(&url).json(&request).send(),
    )
    .await
    .map_err(|_| {
        crate::errors::AppError::InferenceFailed(
            "Agent service (mesh improve) reageert niet (timeout)".into(),
        )
    })?
    .map_err(|e| {
        crate::errors::AppError::InferenceFailed(format!("Mesh improve request failed: {}", e))
    })?;

    if !resp.status().is_success() {
        return Err(crate::errors::AppError::InferenceFailed(format!(
            "Mesh improve error: {}",
            resp.status()
        )));
    }

    let out = resp.json::<MeshDemoResult>().await.map_err(|e| {
        crate::errors::AppError::InferenceFailed(format!("Parse mesh improve: {}", e))
    })?;

    Ok(out)
}
