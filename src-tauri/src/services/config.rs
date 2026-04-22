use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub model_path: PathBuf,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: usize,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/mistral-7b-q4"),
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 2048,
            theme: "dark".to_string(),
        }
    }
}

pub struct ConfigService {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigService {
    pub async fn new(config_dir: PathBuf) -> Result<Self> {
        let config_path = config_dir.join("config.json");
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            serde_json::from_str(&content)?
        } else {
            let default = AppConfig::default();
            let content = serde_json::to_string_pretty(&default)?;
            fs::write(&config_path, content).await?;
            default
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn get(&self) -> &AppConfig {
        &self.config
    }

    pub async fn update<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        updater(&mut self.config);
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content).await?;
        Ok(())
    }
}
