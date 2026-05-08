//! Mock inference engine for testing without models
//!
//! This provides instant responses without needing any model or external tool.

use std::path::Path as StdPath;
use std::pin::Pin;

use crate::errors::{AppError, Result};
use super::{GenerationParams, TokenStream};

use futures::stream;

/// Mock inference engine for testing
pub struct MockEngine {
    loaded: bool,
}

unsafe impl Send for MockEngine {}

impl MockEngine {
    pub fn new() -> Self {
        Self { loaded: false }
    }

    pub fn is_loaded_inner(&self) -> bool {
        self.loaded
    }

    pub async fn load_model_inner(&mut self, _path: &StdPath) -> Result<()> {
        self.loaded = true;
        tracing::info!("MockEngine: Ready");
        Ok(())
    }

    pub async fn unload_inner(&mut self) -> Result<()> {
        self.loaded = false;
        Ok(())
    }

    /// Generate a mock response
    fn generate_mock_response(prompt: &str) -> String {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("hallo") || prompt_lower.contains("hoi") {
            return "Hallo! Ik ben je lokale AI assistent. Ik kan je helpen met vragen, documenten analyseren, en meer. Waarmee kan ik je helpen?".to_string();
        }

        if prompt_lower.contains("hoe gaat") {
            return "Ik gaat goed, dank je! Ik ben een AI assistent die volledig lokaal op jouw computer draait. Hoe kan ik je vandaag helpen?".to_string();
        }

        if prompt_lower.contains("wat kun je") {
            return "Ik kan je helpen met:\n• Vragen beantwoorden\n• Tekst schrijven\n• Documenten analyseren\n• Samenvattingen maken\n\nAlles gebeurt lokaal op jouw device!".to_string();
        }

        if prompt_lower.contains("document") || prompt_lower.contains("analyseer") {
            return "Ik kan documenten analyseren. Upload een document via het Documenten tab en ik zal het voor je lezen en samenvatten.".to_string();
        }

        // Default response
        "Dat is een interessante vraag! Ik ben een demo assistent die lokaal draait. Download een echt AI model via de Modellen tab voor betere antwoorden.".to_string()
    }
}

impl Default for MockEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl super::InferenceEngine for MockEngine {
    fn load_model(&mut self, path: &StdPath) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        let path = path.to_path_buf();
        Box::pin(async move {
            // Accept any path for mock
            tracing::info!("MockEngine: Loading from {}", path.display());
            self.load_model_inner(&path).await
        })
    }

    fn generate(
        &self,
        prompt: &str,
        _params: GenerationParams,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<TokenStream>> + Send + '_>> {
        let prompt = prompt.to_string();

        Box::pin(async move {
            // Generate instant response
            let response = Self::generate_mock_response(&prompt);

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
