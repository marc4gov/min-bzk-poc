//! Inference engine bridges for local LLM execution
//!
//! This module provides abstraction over different inference backends:
//! - **OllamaEngine**: Ollama HTTP API (stable, recommended)
//! - **MockEngine**: Instant demo responses (no model needed)
//! - **CandleEngine**: llama-cli subprocess (requires llama-cli build)
//! - **LlmEngine**: llama-gguf Rust crate (experimental, may hang)

pub mod ollama;
pub mod mock;
pub mod candle;
pub mod llm_engine;
pub mod llama_cpp;

// Use OllamaEngine as default - most stable local LLM option
pub use ollama::OllamaEngine as DefaultEngine;

use std::pin::Pin;
use std::task::{Context, Poll};
use futures::stream::Stream;

use crate::errors::Result;

/// Token stream for streaming generation
pub struct TokenStream {
    inner: Pin<Box<dyn Stream<Item = String> + Send>>,
}

impl Stream for TokenStream {
    type Item = String;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Main inference engine abstraction
pub trait InferenceEngine: Send + Sync {
    fn load_model(&mut self, path: &std::path::Path) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>;
    fn generate(
        &self,
        prompt: &str,
        params: GenerationParams,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<TokenStream>> + Send + '_>>;
    fn is_loaded(&self) -> bool;
    fn unload(&mut self) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>;
}

/// Generation parameters
#[derive(Debug, Clone)]
pub struct GenerationParams {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 2048,
            stop_sequences: vec!["<|im_end|>".to_string(), "</s>".to_string()],
        }
    }
}
