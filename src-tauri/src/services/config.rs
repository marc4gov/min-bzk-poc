use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub model_path: PathBuf,
    pub hotkey: String,
    pub window_opacity: f32,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("~/.localassistant/models"),
            hotkey: "Cmd+Shift+Space".to_string(),
            window_opacity: 0.95,
            theme: "dark".to_string(),
        }
    }
}

pub struct ConfigService {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigService {
    pub async fn new(config_dir: &Path) -> Result<Self> {
        // Ensure config directory exists
        fs::create_dir_all(config_dir).await?;

        let config_path = config_dir.join("config.json");

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            serde_json::from_str(&content)?
        } else {
            let default_config = AppConfig::default();
            let content = serde_json::to_string_pretty(&default_config)?;
            fs::write(&config_path, content).await?;
            default_config
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
        self.save().await?;
        Ok(())
    }

    async fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content).await?;
        Ok(())
    }

    pub async fn set_model_path(&mut self, path: &Path) -> Result<()> {
        self.config.model_path = path.to_path_buf();
        self.save().await?;
        Ok(())
    }

    pub async fn set_hotkey(&mut self, hotkey: &str) -> Result<()> {
        self.config.hotkey = hotkey.to_string();
        self.save().await?;
        Ok(())
    }
}
