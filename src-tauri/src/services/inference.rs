//! Inference service for local LLM execution
//!
//! This module re-exports the inference engines from the bridges module.

// Re-export everything from bridges
pub use crate::bridges::{
    InferenceEngine, GenerationParams, TokenStream, DefaultEngine,
};

/// Helper function to create the default engine
pub fn create_engine() -> DefaultEngine {
    DefaultEngine::default()
}
