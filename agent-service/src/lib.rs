pub mod agents;
pub mod llm;
pub mod tools;
pub mod api;
pub mod events;

use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use api::routes::{AppState, health_check, list_agents, chat};
use api::websocket::websocket_handler;
use llm::{OllamaProvider, OpenAIProvider};

pub async fn create_app() -> Router {
    let ollama = Arc::new(OllamaProvider::new());
    let openai = std::env::var("OPENAI_API_KEY")
        .ok()
        .and_then(|key| OpenAIProvider::from_key(key).into());

    let state = AppState {
        ollama,
        openai: Arc::new(openai),
    };

    Router::new()
        .route("/health", get(health_check))
        .route("/api/agents", get(list_agents))
        .route("/api/chat", post(chat))
        .route("/ws", get(websocket_handler))
        .with_state(state)
}
