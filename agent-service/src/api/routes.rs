use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::llm::{LLMProvider, OllamaProvider, OpenAIProvider, LLMRequest, route_backend};

#[derive(Clone)]
pub struct AppState {
    pub ollama: Arc<OllamaProvider>,
    pub openai: Arc<Option<OpenAIProvider>>,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub agent_mode: String,
    pub selected_agent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub content: String,
    pub agents_used: Vec<String>,
    pub tokens_used: u32,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub ollama_available: bool,
    pub openai_available: bool,
}

#[derive(Debug, Serialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let ollama_available = state.ollama.is_available().await;
    let openai_available = if let Some(openai) = state.openai.as_ref() {
        openai.is_available().await
    } else {
        false
    };

    Json(HealthResponse {
        status: "healthy".to_string(),
        ollama_available,
        openai_available,
    })
}

pub async fn list_agents() -> impl IntoResponse {
    let agents = vec![
        AgentInfo {
            id: "orchestrator".to_string(),
            name: "Orchestrator".to_string(),
            description: "Analyseert je vraag en kiest de juiste agents".to_string(),
        },
        AgentInfo {
            id: "code".to_string(),
            name: "Code Agent".to_string(),
            description: "Schrijft en analyseert code".to_string(),
        },
        AgentInfo {
            id: "schrijf".to_string(),
            name: "Schrijf Agent".to_string(),
            description: "Helpt met teksten schrijven en redigeren".to_string(),
        },
        AgentInfo {
            id: "tools".to_string(),
            name: "Tools Agent".to_string(),
            description: "Gebruikt tools zoals bestanden lezen en web search".to_string(),
        },
    ];

    Json(agents)
}

pub async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let backend = route_backend(&LLMRequest {
        prompt: req.message.clone(),
        max_tokens: None,
        temperature: None,
    });

    let response = match backend {
        crate::llm::LLMBackend::Ollama => {
            state.ollama.generate(LLMRequest {
                prompt: req.message,
                max_tokens: Some(1000),
                temperature: Some(0.7),
            }).await
        }
        crate::llm::LLMBackend::OpenAI => {
            if let Some(openai) = state.openai.as_ref() {
                openai.generate(LLMRequest {
                    prompt: req.message,
                    max_tokens: Some(1000),
                    temperature: Some(0.7),
                }).await
            } else {
                state.ollama.generate(LLMRequest {
                    prompt: req.message,
                    max_tokens: Some(1000),
                    temperature: Some(0.7),
                }).await
            }
        }
    };

    match response {
        Ok(resp) => Ok(Json(ChatResponse {
            content: resp.content,
            agents_used: vec!["single".to_string()],
            tokens_used: resp.tokens_used.unwrap_or(0),
        })),
        Err(e) => {
            tracing::error!("Chat error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
