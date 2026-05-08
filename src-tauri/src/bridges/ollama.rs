//! Ollama integration - stable local LLM via HTTP API

use std::path::Path as StdPath;
use std::pin::Pin;
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, Result};
use super::{GenerationParams, TokenStream};

use futures::stream;

/// Ollama inference engine
pub struct OllamaEngine {
    model: String,
    loaded: bool,
}

unsafe impl Send for OllamaEngine {}

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
    top_p: f32,
    num_predict: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

impl OllamaEngine {
    pub fn new() -> Self {
        Self {
            model: "llama3.2:latest".to_string(), // Default model
            loaded: false,  // Ollama doesn't need "loading"
        }
    }

    pub fn with_model(model: String) -> Self {
        Self {
            model,
            loaded: false,
        }
    }

    pub fn is_loaded_inner(&self) -> bool {
        self.loaded
    }

    pub async fn load_model_inner(&mut self, path: &StdPath) -> Result<()> {
        // Use digitsflow/bonsai-8b:latest as default model
        self.model = "digitsflow/bonsai-8b:latest".to_string();
        self.loaded = true;

        // Verify Ollama is running
        let _ = Self::check_ollama_running().await?;

        tracing::info!("OllamaEngine: Ready with model {}", self.model);
        Ok(())
    }

    pub async fn unload_inner(&mut self) -> Result<()> {
        self.loaded = false;
        Ok(())
    }

    async fn check_ollama_running() -> Result<()> {
        let resp = reqwest::get("http://localhost:11434/api/tags")
            .await
            .map_err(|e| AppError::InferenceFailed(format!("Ollama niet gevonden. Start met: ollama serve. Error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InferenceFailed("Ollama reageert niet correct".to_string()));
        }

        Ok(())
    }

    async fn generate_with_ollama(model: &str, prompt: &str) -> Result<String> {
        let req = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature: 0.7,
                top_p: 0.9,
                num_predict: 200,
            },
        };

        eprintln!("!!! Ollama request: model={}, prompt_len={}", model, prompt.len());

        let resp = reqwest::Client::new()
            .post("http://localhost:11434/api/generate")
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::InferenceFailed(format!("Ollama request failed: {}", e)))?;

        let status = resp.status();
        eprintln!("!!! Ollama response status: {}", status);

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            eprintln!("!!! Ollama error body: {}", body);
            return Err(AppError::InferenceFailed(format!("Ollama error: {}", status)));
        }

        // Get raw response body first
        let body = resp.text().await.unwrap_or_default();
        eprintln!("!!! Raw Ollama response body: {}", body);

        // Then parse as JSON
        let ollama_resp: OllamaResponse = serde_json::from_str(&body)
            .map_err(|e| AppError::InferenceFailed(format!("Ollama JSON parse failed: {}. Body: {}", e, body)))?;

        eprintln!("!!! Parsed Ollama response: '{}'", ollama_resp.response);
        eprintln!("!!! Response length: {}", ollama_resp.response.len());

        Ok(ollama_resp.response)
    }
}

impl Default for OllamaEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl super::InferenceEngine for OllamaEngine {
    fn load_model(&mut self, path: &StdPath) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        let path = path.to_path_buf();
        Box::pin(async move {
            self.load_model_inner(&path).await
        })
    }

    fn generate(
        &self,
        prompt: &str,
        _params: GenerationParams,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<TokenStream>> + Send + '_>> {
        let prompt = prompt.to_string();
        let model = self.model.clone();
        let is_loaded = self.loaded;

        Box::pin(async move {
            if !is_loaded {
                return Err(AppError::InferenceFailed("Ollama not loaded".to_string()));
            }

            use tokio::time::{timeout, Duration};
            let response = match timeout(Duration::from_secs(120), Self::generate_with_ollama(&model, &prompt)).await {
                Ok(Ok(resp)) => resp,
                Ok(Err(e)) => {
                    tracing::error!("Ollama generation failed: {}", e);
                    format!("Sorry, er ging iets mis: {}", e)
                }
                Err(_) => "Het genereren duurde te lang.".to_string(),
            };

            let stream = stream::iter(vec![response]);
            Ok(TokenStream {
                inner: Box::pin(stream),
            })
        })
    }

    fn is_loaded(&self) -> bool {
        self.is_loaded_inner()
    }

    fn unload(&mut self) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.unload_inner().await
        })
    }
}
