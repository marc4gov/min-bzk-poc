use super::provider::{LLMProvider, LLMRequest, LLMResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

#[derive(Clone)]
pub struct OllamaProvider {
    model: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    #[serde(default)]
    prompt_eval_count: u32,
    #[serde(default)]
    eval_count: u32,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            model: "llama3.2:latest".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    async fn check_available(&self) -> bool {
        let url = format!("{}/api/tags", OLLAMA_BASE_URL);
        self.client.get(&url).send().await.is_ok()
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn generate(&self, request: LLMRequest) -> anyhow::Result<LLMResponse> {
        let ollama_req = OllamaRequest {
            model: self.model.clone(),
            prompt: request.prompt,
            stream: false,
            options: OllamaOptions {
                temperature: request.temperature.unwrap_or(0.7),
                num_predict: request.max_tokens.unwrap_or(2000),
            },
        };

        let url = format!("{}/api/generate", OLLAMA_BASE_URL);
        let resp = self
            .client
            .post(&url)
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("Ollama returned status: {}", resp.status()));
        }

        let ollama_resp: OllamaResponse = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse Ollama response: {}", e))?;

        Ok(LLMResponse {
            content: ollama_resp.response,
            tokens_used: Some(ollama_resp.prompt_eval_count + ollama_resp.eval_count),
        })
    }

    fn backend_name(&self) -> &str {
        "ollama"
    }

    async fn is_available(&self) -> bool {
        self.check_available().await
    }
}
