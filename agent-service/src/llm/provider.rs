use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub tokens_used: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMBackend {
    Ollama,
    OpenAI,
}

pub fn route_backend(request: &LLMRequest) -> LLMBackend {
    if request.prompt.len() < 500 {
        return LLMBackend::Ollama;
    }
    let code_keywords = ["code", "function", "bug", "debug", "python", "rust", "javascript"];
    let prompt_lower = request.prompt.to_lowercase();
    if code_keywords.iter().any(|kw| prompt_lower.contains(kw)) {
        return LLMBackend::Ollama;
    }
    let writing_keywords = ["schrijf", "write", "essay", "verhaal", "story"];
    if writing_keywords.iter().any(|kw| prompt_lower.contains(kw)) {
        return LLMBackend::OpenAI;
    }
    LLMBackend::Ollama
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate(&self, request: LLMRequest) -> anyhow::Result<LLMResponse>;
    fn backend_name(&self) -> &str;
    async fn is_available(&self) -> bool;
}
