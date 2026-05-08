//! llama-gguf inference engine
//!
//! Uses the llama-gguf Rust crate for local LLM inference.
//! This provides clean, silent inference with native Rust performance.

use std::path::Path as StdPath;
use std::pin::Pin;
use std::sync::Arc;

use crate::errors::{AppError, Result};
use super::{GenerationParams, TokenStream};

use futures::stream;

/// LLM inference engine using llama-gguf
pub struct LlmEngine {
    engine: Option<Arc<llama_gguf::Engine>>,
    loaded: bool,
}

unsafe impl Send for LlmEngine {}

impl LlmEngine {
    pub fn new() -> Self {
        Self {
            engine: None,
            loaded: false,
        }
    }

    pub fn is_loaded_inner(&self) -> bool {
        self.loaded
    }

    pub async fn load_model_inner(&mut self, path: &StdPath) -> Result<()> {
        let path_str = path.to_str()
            .ok_or_else(|| AppError::InferenceFailed("Invalid model path".to_string()))?;

        if !std::path::Path::new(path_str).exists() {
            return Err(AppError::ModelNotFound(path_str.to_string()));
        }

        // Load engine using llama-gguf
        // This is CPU-intensive, so we spawn to a blocking thread
        let model_path = path_str.to_string();
        let engine = tokio::task::spawn_blocking(move || {
            llama_gguf::Engine::load(llama_gguf::EngineConfig {
                model_path,
                temperature: 0.7,
                top_k: 40,
                top_p: 0.9,
                repeat_penalty: 1.1,
                max_tokens: 50,
                use_gpu: false,
                ..Default::default()
            })
        })
        .await
        .map_err(|e| AppError::InferenceFailed(format!("Engine loading task failed: {}", e)))?
        .map_err(|e| AppError::InferenceFailed(format!("Failed to load engine: {}", e)))?;

        self.engine = Some(Arc::new(engine));
        self.loaded = true;

        tracing::info!("LlmEngine: Model loaded from {}", path_str);
        Ok(())
    }

    pub async fn unload_inner(&mut self) -> Result<()> {
        self.engine = None;
        self.loaded = false;
        Ok(())
    }

    async fn generate_with_engine(&self, prompt: &str) -> Result<String> {
        let engine = self.engine.as_ref()
            .ok_or_else(|| AppError::InferenceFailed("Engine not loaded".to_string()))?
            .clone();

        let prompt = prompt.to_string();

        // Run generation in blocking thread
        let result = tokio::task::spawn_blocking(move || {
            engine.generate(&prompt, 50)
                .map_err(|e| format!("Generation failed: {}", e))
        })
        .await
        .map_err(|e| AppError::InferenceFailed(format!("Generation task failed: {}", e)))?
        .map_err(|e| AppError::InferenceFailed(e))?;

        Ok(result)
    }
}

impl Default for LlmEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Implement InferenceEngine trait
impl super::InferenceEngine for LlmEngine {
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
        let is_loaded = self.loaded;

        Box::pin(async move {
            if !is_loaded {
                return Err(AppError::InferenceFailed("Model not loaded".to_string()));
            }

            // Generate with timeout
            use tokio::time::{timeout, Duration};
            let response = match timeout(Duration::from_secs(30), self.generate_with_engine(&prompt)).await {
                Ok(Ok(resp)) => resp,
                Ok(Err(e)) => {
                    tracing::error!("Generation failed: {}", e);
                    format!("Sorry, er ging iets mis: {}", e)
                }
                Err(_) => "Het genereren duurde te lang. Probeer een kortere vraag.".to_string(),
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
