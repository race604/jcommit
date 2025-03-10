use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api_endpoint: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub is_azure: Option<bool>,
    pub api_version: Option<String>,
    pub prompt: Option<String>,
}

impl Config {
    pub fn new() -> Result<Self> {
        // read cofig file
        let config = Self::from_file().unwrap_or_default();
        
        // read env vars to override config
        let config = Self {
            api_endpoint: std::env::var("JCOMMIT_API_ENDPOINT").ok().or(config.api_endpoint),
            model: std::env::var("JCOMMIT_MODEL").ok().or(config.model),
            api_key: std::env::var("OPENAI_API_KEY").ok().or(config.api_key),
            is_azure: std::env::var("JCOMMIT_IS_AZURE")
                .ok()
                .and_then(|v| v.parse::<bool>().ok())
                .or(config.is_azure),
            api_version: std::env::var("JCOMMIT_API_VERSION").ok().or(config.api_version),
            prompt: std::env::var("JCOMMIT_PROMPT").ok().or(config.prompt),
        };

        Ok(config)
    }

    fn from_file() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(config_path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    fn config_file_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home).join(".jcommit.toml"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_endpoint: None,
            model: None,
            api_key: None,
            is_azure: None,
            api_version: None,
            prompt: None,
        }
    }
}