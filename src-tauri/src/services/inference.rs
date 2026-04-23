//! Inference service for local LLM execution
//!
//! This module re-exports the inference engines from the bridges module.

// Re-export everything from bridges
pub use crate::bridges::{
    InferenceEngine, GenerationParams, TokenStream,
    LlamaCppEngine,
};

/// Type alias for the default inference engine
///
/// Uses LlamaCppEngine for real local inference with GGUF models
pub type DefaultEngine = LlamaCppEngine;

/// Helper function to create the default engine
pub fn create_engine() -> DefaultEngine {
    DefaultEngine::default()
}
