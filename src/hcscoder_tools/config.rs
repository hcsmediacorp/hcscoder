//! hcscoder Config Tool
//!
//! Configuration management for hcscoder settings.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Supported configuration keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigKey {
    Model,
    Temperature,
    MaxTokens,
    Theme,
    Editor,
    AutoApproveTools,
    Verbosity,
}

impl std::fmt::Display for ConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigKey::Model => write!(f, "model"),
            ConfigKey::Temperature => write!(f, "temperature"),
            ConfigKey::MaxTokens => write!(f, "max_tokens"),
            ConfigKey::Theme => write!(f, "theme"),
            ConfigKey::Editor => write!(f, "editor"),
            ConfigKey::AutoApproveTools => write!(f, "auto_approve_tools"),
            ConfigKey::Verbosity => write!(f, "verbosity"),
        }
    }
}

/// Configuration storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HcscoderConfig {
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub theme: Option<String>,
    pub editor: Option<String>,
    pub auto_approve_tools: Option<Vec<String>>,
    pub verbosity: Option<String>,
    pub custom: HashMap<String, serde_json::Value>,
}

impl HcscoderConfig {
    pub fn new() -> Self {
        HcscoderConfig::default()
    }

    /// Get config file path
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("could not resolve home directory")?;
        Ok(home.join(".hcscoder").join("config.json"))
    }

    /// Load configuration from disk
    pub async fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(HcscoderConfig::new());
        }

        let content = fs::read_to_string(&path).await?;
        serde_json::from_str(&content).context("Failed to parse config file")
    }

    /// Save configuration to disk
    pub async fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content).await?;

        Ok(())
    }

    /// Get a configuration value (owned JSON for primitive fields).
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        match key {
            "model" => self.model.as_ref().map(|v| serde_json::json!(v)),
            "temperature" => self.temperature.and_then(|v| {
                serde_json::Number::from_f64(v as f64).map(serde_json::Value::Number)
            }),
            "max_tokens" => self.max_tokens.map(|v| serde_json::Value::Number(v.into())),
            "theme" => self.theme.as_ref().map(|v| serde_json::json!(v)),
            "editor" => self.editor.as_ref().map(|v| serde_json::json!(v)),
            "verbosity" => self.verbosity.as_ref().map(|v| serde_json::json!(v)),
            _ => self.custom.get(key).cloned(),
        }
    }

    /// Set a configuration value
    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        match key {
            "model" => self.model = value.as_str().map(|s| s.to_string()),
            "temperature" => self.temperature = value.as_f64().map(|f| f as f32),
            "max_tokens" => self.max_tokens = value.as_u64().map(|n| n as u32),
            "theme" => self.theme = value.as_str().map(|s| s.to_string()),
            "editor" => self.editor = value.as_str().map(|s| s.to_string()),
            "verbosity" => self.verbosity = value.as_str().map(|s| s.to_string()),
            _ => {
                self.custom.insert(key.to_string(), value);
            }
        }
    }
}

/// Get current configuration
pub async fn get_config() -> Result<HcscoderConfig> {
    HcscoderConfig::load().await
}

/// Update configuration
pub async fn update_config(key: &str, value: serde_json::Value) -> Result<HcscoderConfig> {
    let mut config = HcscoderConfig::load().await?;
    config.set(key, value);
    config.save().await?;
    Ok(config)
}

/// List all configuration values
pub async fn list_config() -> Result<HcscoderConfig> {
    HcscoderConfig::load().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_set_get() {
        let mut config = HcscoderConfig::new();
        config.set("model", serde_json::Value::String("claude-3.5".to_string()));

        assert!(config.get("model").is_some());
    }
}
