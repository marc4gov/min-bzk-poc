//! llama.cpp bridge for local LLM inference
//!
//! This module provides a safe(ish) wrapper around the llama.cpp C API.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int};
use std::path::Path as StdPath;
use std::pin::Pin;
use std::sync::Arc;
use std::ptr::NonNull;

use crate::errors::{AppError, Result};
use super::{InferenceEngine, GenerationParams, TokenStream};

// Opaque types from llama.cpp
enum llama_model {}
enum llama_context {}
type llama_token = c_int;

// Parameter structures
#[repr(C)]
struct llama_model_params {
    n_gpu_layers: c_int,
    main_gpu: c_int,
    tensor_split: *const c_float,
    use_mmap: bool,
    use_mlock: bool,
}

#[repr(C)]
struct llama_context_params {
    n_ctx: c_int,
    n_batch: c_int,
    n_threads: c_int,
    n_threads_batch: c_int,
    rope_freq_base: c_float,
    rope_freq_scale: c_float,
    f16_kv: bool,
    logits_all: bool,
}

impl Default for llama_model_params {
    fn default() -> Self {
        Self {
            n_gpu_layers: -1,
            main_gpu: 0,
            tensor_split: std::ptr::null(),
            use_mmap: true,
            use_mlock: false,
        }
    }
}

impl Default for llama_context_params {
    fn default() -> Self {
        Self {
            n_ctx: 2048,
            n_batch: 512,
            n_threads: 4,
            n_threads_batch: 4,
            rope_freq_base: 10000.0,
            rope_freq_scale: 1.0,
            f16_kv: true,
            logits_all: false,
        }
    }
}

// FFI declarations (stub implementation for now)
// Note: These are only used when llama.cpp library is actually linked
#[cfg_attr(feature = "llama", link(name = "llama"))]
extern "C" {
    fn llama_load_model_from_file(
        path: *const c_char,
        params: *const llama_model_params,
    ) -> *mut llama_model;

    fn llama_free_model(model: *mut llama_model);

    fn llama_init_from_model(
        model: *mut llama_model,
        params: *const llama_context_params,
    ) -> *mut llama_context;

    fn llama_free(ctx: *mut llama_context);

    fn llama_n_ctx(ctx: *const llama_context) -> c_int;
}

/// Inner state for LlamaCppEngine (handles raw pointers)
struct LlamaCppInner {
    model: Option<NonNull<llama_model>>,
    ctx: Option<NonNull<llama_context>>,
    loaded: bool,
}

unsafe impl Send for LlamaCppInner {}

/// Llama.cpp inference engine
///
/// Uses the llama.cpp library for GGUF model inference.
/// Currently a placeholder that returns mock responses until
/// the library is properly linked.
pub struct LlamaCppEngine {
    inner: Arc<std::sync::Mutex<LlamaCppInner>>,
}

impl LlamaCppEngine {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(LlamaCppInner {
                model: None,
                ctx: None,
                loaded: false,
            })),
        }
    }

    pub fn is_loaded_inner(&self) -> bool {
        self.inner.lock().unwrap().loaded
    }

    pub async fn load_model_inner(&mut self, path: &StdPath) -> Result<()> {
        let path_str = path.to_str()
            .ok_or_else(|| AppError::InferenceFailed("Invalid model path".to_string()))?;

        // For now, just mark as loaded without calling the actual FFI
        // TODO: Call actual llama_load_model_from_file when library is linked
        {
            let mut inner = self.inner.lock().unwrap();
            inner.loaded = true;
            tracing::info!("LlamaCppEngine: Model marked as loaded from: {}", path_str);
            tracing::warn!("LlamaCppEngine: Using mock implementation - FFI not yet linked");
        }

        Ok(())
    }

    pub async fn unload_inner(&mut self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.loaded = false;
        inner.model = None;
        inner.ctx = None;
        Ok(())
    }
}

impl Default for LlamaCppEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Implement InferenceEngine trait
impl InferenceEngine for LlamaCppEngine {
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
        let is_loaded = self.is_loaded_inner();

        Box::pin(async move {
            if !is_loaded {
                return Err(AppError::InferenceFailed("Model not loaded".to_string()));
            }

            // Mock response for now
            let response = format!(
                "# Llama.cpp Response\n\nGebruiker vroeg: {}\n\n[Dit is een placeholder antwoord. \
                Echte llama.cpp integratie volgt wanneer de C library correct is gelinkt.]\n\n\
                Om dit werkend te krijgen:\n\
                1. Build llama.cpp: `cd third-party/llama.cpp && cmake -B build && cmake --build build --parallel`\n\
                2. Download een GGUF model naar models/\n\
                3. Enable de 'llama' feature in Cargo.toml",
                prompt.chars().take(80).collect::<String>()
            );

            let stream = futures::stream::iter(vec![response]);
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
