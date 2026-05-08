//! llama-gguf inference engine
//!
//! Uses the llama-gguf Rust crate for local LLM inference.
//! This provides clean, silent inference with native Rust performance.

use std::path::Path as StdPath;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use crate::errors::{AppError, Result};
use super::{InferenceEngine, GenerationParams, TokenStream};

use futures::stream;

/// LLM inference engine using llama-gguf
pub struct LlmEngine {
    model: Option<Arc<StdMutex<llama_gguf::Model>>>,
    loaded: bool,
}

unsafe impl Send for LlmEngine {}

impl LlmEngine {
    pub fn new() -> Self {
        Self {
            model: None,
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

        // Load model using llama-gguf
        // This is CPU-intensive, so we spawn to a blocking thread
        let model_path = path_str.to_string();
        let model = tokio::task::spawn_blocking(move || {
            // Load GGUF model
            llama_gguf::Model::load(model_path)
                .map_err(|e| AppError::InferenceFailed(format!("Failed to load model: {}", e)))
        })
        .await
        .map_err(|e| AppError::InferenceFailed(format!("Model loading task failed: {}", e)))?
        .map_err(|e| e)?;

        self.model = Some(Arc::new(StdMutex::new(model)));
        self.loaded = true;

        tracing::info!("LlmEngine: Model loaded from {}", path_str);
        Ok(())
    }

    pub async fn unload_inner(&mut self) -> Result<()> {
        self.model = None;
        self.loaded = false;
        Ok(())
    }

    async fn generate_with_llama_gguf(&self, prompt: &str) -> Result<String> {
        let model = self.model.as_ref()
            .ok_or_else(|| AppError::InferenceFailed("Model not loaded".to_string()))?
            .clone();

        let prompt = prompt.to_string();

        // Run generation in blocking thread
        let result = tokio::task::spawn_blocking(move || {
            let model = model.lock().unwrap();

            // Create a session
            let mut session = model.create_session(llama_gguf::SessionConfig {
                context_size: 2048,
                ..Default::default()
            })
            .map_err(|e| format!("Failed to create session: {}", e))?;

            // Tokenize and feed prompt
            let tokens = model.tokenize(prompt, /* add_bos= */ true)
                .map_err(|e| format!("Tokenization failed: {}", e))?;

            for token in tokens {
                session.feed(token)
                    .map_err(|e| format!("Feed failed: {}", e))?;
            }

            // Generate response
            let mut response = String::new();
            let max_tokens = 50;
            let mut tokens_generated = 0;

            while tokens_generated < max_tokens {
                match session.next() {
                    Ok(Some(token)) => {
                        // Decode token
                        match model.decode_token(token) {
                            Some(text) => {
                                response.push_str(&text);

                                // Check for EOS
                                if text.contains("</s>") || text.contains("[INST]") {
                                    break;
                                }

                                // Stop at reasonable length
                                if response.len() > 400 {
                                    break;
                                }
                            }
                            None => break,
                        }
                        tokens_generated += 1;
                    }
                    Ok(None) => break,  // EOS
                    Err(e) => return Err(format!("Generation error: {}", e)),
                }
            }

            Ok(response)
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
impl InferenceEngine for LlmEngine {
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
            let response = match timeout(Duration::from_secs(30), self.generate_with_llama_gguf(&prompt)).await {
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
