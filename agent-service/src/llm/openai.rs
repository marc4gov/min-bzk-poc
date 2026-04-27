use super::provider::{LLMProvider, LLMRequest, LLMResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    total_tokens: u32,
}

impl OpenAIProvider {
    pub fn from_key(api_key: String) -> Self {
        Self {
            api_key,
            model: "gpt-4o-mini".to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn check_available(&self) -> bool {
        let url = "https://api.openai.com/v1/models";
        self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn generate(&self, request: LLMRequest) -> anyhow::Result<LLMResponse> {
        let openai_req = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: request.prompt,
            }],
            max_tokens: request.max_tokens.unwrap_or(2000),
            temperature: request.temperature.unwrap_or(0.7),
        };

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("OpenAI request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI error {}: {}", status, body));
        }

        let openai_resp: OpenAIResponse = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse OpenAI response: {}", e))?;

        let content = openai_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(LLMResponse {
            content,
            tokens_used: Some(openai_resp.usage.total_tokens),
        })
    }

    fn backend_name(&self) -> &str {
        "openai"
    }

    async fn is_available(&self) -> bool {
        self.check_available().await
    }
}
