//! llama.cpp bridge (placeholder - in development)

#![allow(dead_code)]

use crate::errors::Result;
use super::{GenerationParams, TokenStream};
use std::path::Path as StdPath;
use std::pin::Pin;
use futures::stream;

/// Llama.cpp inference engine (placeholder)
pub struct LlamaCppEngine {
    loaded: bool,
}

impl LlamaCppEngine {
    pub fn new() -> Self {
        Self { loaded: false }
    }
}

impl Default for LlamaCppEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl super::InferenceEngine for LlamaCppEngine {
    fn load_model(&mut self, _path: &StdPath) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            Ok(())
        })
    }

    fn generate(
        &self,
        _prompt: &str,
        _params: GenerationParams,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<TokenStream>> + Send + '_>> {
        Box::pin(async move {
            let stream = stream::iter(vec!["llama.cpp integratie wordt ontwikkeld.".to_string()]);
            Ok(TokenStream { inner: Box::pin(stream) })
        })
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn unload(&mut self) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            Ok(())
        })
    }
}
