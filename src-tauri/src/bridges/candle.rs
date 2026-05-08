//! Candle ML framework inference engine - simplified for stability

#![allow(dead_code)]

use std::path::Path as StdPath;
use std::pin::Pin;
use std::process::Command;
use std::io::Write;

use crate::errors::{AppError, Result};
use super::{GenerationParams, TokenStream};

use futures::stream;

/// Candle inference engine
pub struct CandleEngine {
    model_path: Option<String>,
    loaded: bool,
}

unsafe impl Send for CandleEngine {}

impl CandleEngine {
    pub fn new() -> Self {
        Self {
            model_path: None,
            loaded: false,
        }
    }

    pub fn is_loaded_inner(&self) -> bool {
        self.loaded
    }

    pub async fn load_model_inner(&mut self, path: &StdPath) -> Result<()> {
        let path_str = path.to_str()
            .ok_or_else(|| AppError::InferenceFailed("Invalid model path".to_string()))?;

        if !StdPath::new(path_str).exists() {
            return Err(AppError::ModelNotFound(path_str.to_string()));
        }

        self.model_path = Some(path_str.to_string());
        self.loaded = true;

        tracing::info!("CandleEngine: Model loaded from {}", path_str);
        Ok(())
    }

    pub async fn unload_inner(&mut self) -> Result<()> {
        self.model_path = None;
        self.loaded = false;
        Ok(())
    }

    /// Run llama-cli in blocking thread and capture output
    async fn generate_with_llama_cli(model_path: &str, prompt: &str) -> Result<String> {
        let model_path = model_path.to_string();
        let prompt = prompt.to_string();

        // Run in blocking thread since llama-cli can take a while
        let result = tokio::task::spawn_blocking(move || {
            Self::run_llama_cli_sync(&model_path, &prompt)
        })
        .await
        .map_err(|e| AppError::InferenceFailed(format!("Spawn error: {}", e)))??;

        Ok(result)
    }

    /// Synchronous llama-cli execution
    fn run_llama_cli_sync(model_path: &str, prompt: &str) -> Result<String> {
        // TEMPORARY: Return test response to verify flow works
        eprintln!("!!! run_llama_cli_sync called with prompt length: {}", prompt.len());
        let test_response = "Dit is een TEST antwoord om te zien of de flow werkt. Als je dit leest, werkt de backend maar niet llama-cli output parsing.";
        eprintln!("!!! Returning test response: {}", test_response);
        return Ok(test_response.to_string());

        let temp_dir = std::env::temp_dir();
        let prompt_file = temp_dir.join("llama_prompt.txt");

        // Write prompt
        {
            let mut f = std::fs::File::create(&prompt_file)
                .map_err(|e| AppError::InferenceFailed(format!("Cannot create prompt file: {}", e)))?;
            f.write_all(prompt.as_bytes())
                .map_err(|e| AppError::InferenceFailed(format!("Cannot write prompt: {}", e)))?;
        }

        // Find llama-cli
        let llama_cli = [
            "./third-party/llama.cpp/build/bin/llama-cli",
            "../third-party/llama.cpp/build/bin/llama-cli",
            "/usr/local/bin/llama-cli",
        ]
        .iter()
        .find(|p| StdPath::new(p).exists())
        .ok_or_else(|| AppError::InferenceFailed("llama-cli not found".to_string()))?;

        // Run llama-cli
        let output = Command::new(llama_cli)
            .arg("-m")
            .arg(model_path)
            .arg("-f")
            .arg(&prompt_file)
            .arg("-n")
            .arg("100")
            .arg("--temp")
            .arg("0.7")
            .arg("--top-p")
            .arg("0.9")
            .arg("-ngl")
            .arg("0")
            .arg("-c")
            .arg("2048")
            .arg("--no-display-prompt")
            // Capture all output, suppress stderr
            .stderr(std::process::Stdio::null())
            .output()
            .map_err(|e| AppError::InferenceFailed(format!("Failed to run llama-cli: {}", e)))?;

        // Clean up
        let _ = std::fs::remove_file(&prompt_file);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::InferenceFailed(format!("llama-cli failed: {}", stderr)));
        }

        // Parse stdout - llama-cli outputs the response here
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        tracing::info!("Raw stdout from llama-cli: {}", stdout);
        tracing::info!("Raw stdout length: {}", stdout.len());

        // TEMP: No filtering - return everything to debug
        let response = stdout.trim().to_string();

        Ok(response)
    }
}

impl Default for CandleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl super::InferenceEngine for CandleEngine {
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
        let model_path = self.model_path.clone();
        let is_loaded = self.loaded;

        Box::pin(async move {
            if !is_loaded {
                return Err(AppError::InferenceFailed("Model not loaded".to_string()));
            }

            let response = if let Some(path) = model_path {
                use tokio::time::{timeout, Duration};
                match timeout(Duration::from_secs(300), Self::generate_with_llama_cli(&path, &prompt)).await {
                    Ok(Ok(resp)) => {
                        tracing::info!("Raw response length: {}", resp.len());
                        tracing::info!("Raw response: {}", resp);
                        if resp.is_empty() {
                            "Ik begrijp je vraag. Probeer het anders te formuleren.".to_string()
                        } else {
                            resp
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Generation failed: {}", e);
                        "Sorry, er ging iets mis bij het genereren.".to_string()
                    }
                    Err(_) => "Het genereren duurde te lang.".to_string(),
                }
            } else {
                "Hallo! Ik ben je lokale AI assistent.".to_string()
            };

            tracing::info!("Final response to send: '{}'", response);

            let stream = stream::iter(vec![response.clone()]);
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
