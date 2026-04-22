//! Inference engine bridges for local LLM execution
//!
//! This module provides abstraction over different inference backends:
//! - **llama.cpp**: Stable, cross-platform, uses GGUF quantized models
//! - **MLX**: Apple's native ML framework (experimental, via Swift bridge)

use std::ffi::CString;
use std::os::raw::c_char;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;
use futures::stream::{self, Stream};

// Link to llama.cpp static library (when available)
#[cfg(feature = "llama")]
#[link(name = "llama", kind = "static")]
extern "C" {
    fn llama_load_model(path: *const c_char) -> *mut std::ffi::c_void;
    fn llama_generate(
        model: *mut std::ffi::c_void,
        prompt: *const c_char,
        temp: f32,
        top_p: f32,
        max_tokens: i32,
        callback: extern "C" fn(*const c_char),
    );
    fn llama_unload_model(model: *mut std::ffi::c_void);
}

// Link to MLX Swift bridge (when available)
#[cfg(all(target_os = "macos", feature = "mlx"))]
#[link(name = "mlx_bridge", kind = "static")]
extern "C" {
    fn mlx_load_model(path: *const c_char) -> *mut std::ffi::c_void;
    fn mlx_generate(
        model: *mut std::ffi::c_void,
        prompt: *const c_char,
        temp: f32,
        top_p: f32,
        max_tokens: i32,
        callback: extern "C" fn(*const c_char),
    );
    fn mlx_unload_model(model: *mut std::ffi::c_void);
}

use crate::errors::{AppError, Result};

/// Inference backend selection
#[derive(Debug, Clone, Copy)]
pub enum Backend {
    LlamaCpp,
    MLX,
}

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
    fn load_model(&mut self, path: &std::path::Path) -> impl std::future::Future<Output = Result<()>> + Send;
    fn generate(
        &self,
        prompt: &str,
        params: GenerationParams,
    ) -> impl std::future::Future<Output = Result<TokenStream>> + Send;
    fn is_loaded(&self) -> bool;
    fn unload(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;
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

/// Llama.cpp inference engine (recommended for production)
#[cfg(feature = "llama")]
pub struct LlamaEngine {
    model: Option<*mut std::ffi::c_void>,
    is_loaded: bool,
}

#[cfg(feature = "llama")]
unsafe impl Send for LlamaEngine {}

#[cfg(feature = "llama")]
impl LlamaEngine {
    pub fn new() -> Self {
        Self {
            model: None,
            is_loaded: false,
        }
    }

    pub async fn load_model_inner(&mut self, path: &std::path::Path) -> Result<()> {
        let path_str = path.to_str().ok_or_else(|| {
            AppError::InferenceFailed("Invalid model path".to_string())
        })?;
        let c_path = CString::new(path_str).map_err(|e| AppError::Io(e.into()))?;

        let model = unsafe { llama_load_model(c_path.as_ptr()) };

        if model.is_null() {
            return Err(AppError::ModelNotFound(path_str.to_string()));
        }

        self.model = Some(model);
        self.is_loaded = true;
        tracing::info!("Llama.cpp model loaded from: {}", path_str);
        Ok(())
    }

    pub async fn generate_inner(&self, prompt: &str, params: GenerationParams) -> Result<TokenStream> {
        if !self.is_loaded {
            return Err(AppError::InferenceFailed("Model not loaded".to_string()));
        }

        if let Some(model) = self.model {
            let c_prompt = CString::new(prompt).map_err(|e| AppError::Io(e.into()))?;

            // Create a channel for token streaming
            let (tx, rx) = std::sync::mpsc::channel();

            // Spawn thread for generation
            let params_clone = params.clone();
            std::thread::spawn(move || {
                extern "C" fn trampoline(
                    ctx: *const c_char,
                    tx: *mut std::ffi::c_void,
                ) {
                    unsafe {
                        if !ctx.is_null() {
                            let s = std::ffi::CStr::from_ptr(ctx).to_string_lossy().to_string();
                            let tx = &*(tx as *const std::sync::mpsc::Sender<String>);
                            let _ = tx.send(s);
                        }
                    }
                    // Signal end
                    let tx = &*(tx as *const std::sync::mpsc::Sender<String>);
                    let _ = tx.send(String::new()); // Empty string signals end
                }

                unsafe {
                    llama_generate(
                        model,
                        c_prompt.as_ptr(),
                        params_clone.temperature,
                        params_clone.top_p,
                        params_clone.max_tokens as i32,
                        trampoline,
                    );
                }
            });

            // Create stream from receiver
            let stream = stream::repeat_with(move || rx.recv().ok().unwrap_or_default())
                .take_while(|s| !s.is_empty());

            Ok(TokenStream {
                inner: stream,
            })
        } else {
            Err(AppError::InferenceFailed("No model loaded".to_string()))
        }
    }

    pub fn is_loaded_inner(&self) -> bool {
        self.is_loaded
    }

    pub async fn unload_inner(&mut self) -> Result<()> {
        if let Some(model) = self.model {
            unsafe { llama_unload_model(model) };
        }
        self.model = None;
        self.is_loaded = false;
        Ok(())
    }
}

#[cfg(feature = "llama")]
impl Default for LlamaEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "llama")]
impl Drop for LlamaEngine {
    fn drop(&mut self) {
        if let Some(model) = self.model {
            unsafe { llama_unload_model(model) };
        }
    }
}

/// MLX-based inference engine (experimental, Apple Silicon only)
#[cfg(all(target_os = "macos", feature = "mlx"))]
pub struct MLXEngine {
    model: Option<*mut std::ffi::c_void>,
    is_loaded: bool,
}

#[cfg(all(target_os = "macos", feature = "mlx"))]
impl MLXEngine {
    pub fn new() -> Self {
        Self {
            model: None,
            is_loaded: false,
        }
    }

    pub async fn load_model_inner(&mut self, path: &std::path::Path) -> Result<()> {
        let path_str = path.to_str().ok_or_else(|| {
            AppError::InferenceFailed("Invalid model path".to_string())
        })?;
        let c_path = CString::new(path_str).map_err(|e| AppError::Io(e.into()))?;

        let model = unsafe { mlx_load_model(c_path.as_ptr()) };

        if model.is_null() {
            return Err(AppError::ModelNotFound(path_str.to_string()));
        }

        self.model = Some(model);
        self.is_loaded = true;
        tracing::info!("MLX model loaded from: {}", path_str);
        Ok(())
    }

    // Similar implementation for generate_inner, etc.
    pub fn is_loaded_inner(&self) -> bool {
        self.is_loaded
    }

    pub async fn unload_inner(&mut self) -> Result<()> {
        if let Some(model) = self.model {
            unsafe { mlx_unload_model(model) };
        }
        self.model = None;
        self.is_loaded = false;
        Ok(())
    }
}

#[cfg(all(target_os = "macos", feature = "mlx"))]
impl Default for MLXEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Fallback engine that returns mock responses (for testing)
pub struct MockEngine {
    loaded: bool,
}

impl MockEngine {
    pub fn new() -> Self {
        Self { loaded: false }
    }

    pub async fn load_model_inner(&mut self, _path: &std::path::Path) -> Result<()> {
        self.loaded = true;
        tracing::info!("Mock engine loaded");
        Ok(())
    }

    pub async fn generate_inner(&self, prompt: &str, _params: GenerationParams) -> Result<TokenStream> {
        if !self.loaded {
            return Err(AppError::InferenceFailed("Model not loaded".to_string()));
        }

        let response = format!("This is a mock response to: {}\n\nTo enable real inference:\n1. Add llama.cpp library to the project\n2. Enable the 'llama' feature in Cargo.toml\n3. Download a GGUF model file", prompt);

        let stream = Box::pin(stream::iter(vec![response]));
        Ok(TokenStream { inner: stream })
    }

    pub fn is_loaded_inner(&self) -> bool {
        self.loaded
    }

    pub async fn unload_inner(&mut self) -> Result<()> {
        self.loaded = false;
        Ok(())
    }
}

impl Default for MockEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Export the appropriate engine based on features
#[cfg(feature = "llama")]
pub type DefaultEngine = LlamaEngine;

#[cfg(all(not(feature = "llama"), all(target_os = "macos", feature = "mlx")))]
pub type DefaultEngine = MLXEngine;

#[cfg(all(not(feature = "llama"), not(feature = "mlx")))]
pub type DefaultEngine = MockEngine;
