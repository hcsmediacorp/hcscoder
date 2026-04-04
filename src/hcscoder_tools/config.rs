//! hcscoder Configuration Module
//! 
//! Configuration file management for hcscoder.
//! Supports loading from ~/.hcscoder/config.toml
//! 
//! ## Priority Order (highest to lowest):
//! 1. CLI arguments (--model, --theme, etc.)
//! 2. Environment variables (OPENROUTER_API_KEY, HCSCODER_THEME, etc.)
//! 3. Config file (~/.hcscoder/config.toml)
//! 4. Built-in defaults

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Color support mode
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorSupport {
    #[default]
    Auto,
    Always,
    Never,
}

/// OpenRouter API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_true")]
    pub streaming: bool,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub syntax_highlighting: bool,
    #[serde(default = "default_true")]
    pub show_token_usage: bool,
    #[serde(default)]
    pub color_support: ColorSupport,
}

/// Chat configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    #[serde(default = "default_true")]
    pub history_enabled: bool,
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    #[serde(default = "default_true")]
    pub auto_completion: bool,
    #[serde(default = "default_true")]
    pub typing_indicator: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_true")]
    pub audit_logging: bool,
    pub log_file: Option<String>,
    #[serde(default = "default_true")]
    pub path_traversal_protection: bool,
    #[serde(default = "default_true")]
    pub command_injection_protection: bool,
    #[serde(default = "default_true")]
    pub block_sensitive_files: bool,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_max_history")]
    pub max_history_size: usize,
    #[serde(default = "default_context_limit")]
    pub context_token_limit: usize,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub worker_threads: Option<usize>,
    #[serde(default)]
    pub response_caching: bool,
    pub cache_dir: Option<String>,
}

/// Buddy system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuddyConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub auto_summon: bool,
    pub preferred_buddy: Option<String>,
}

// Default value functions
fn default_model() -> String {
    "anthropic/claude-3.5-haiku".to_string()
}

fn default_timeout() -> u64 {
    60
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_history_size() -> usize {
    1000
}

fn default_max_history() -> usize {
    50
}

fn default_context_limit() -> usize {
    4000
}

fn default_true() -> bool {
    true
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderConfig {
    #[serde(default)]
    pub openrouter: OpenRouterConfig,
    
    #[serde(default)]
    pub ui: UiConfig,
    
    #[serde(default)]
    pub chat: ChatConfig,
    
    #[serde(default)]
    pub security: SecurityConfig,
    
    #[serde(default)]
    pub memory: MemoryConfig,
    
    #[serde(default)]
    pub performance: PerformanceConfig,
    
    #[serde(default)]
    pub buddy: BuddyConfig,
    
    // Legacy fields for backward compatibility
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub theme: Option<String>,
    pub editor: Option<String>,
    pub auto_approve_tools: Option<Vec<String>>,
    pub verbosity: Option<String>,
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for HcscoderConfig {
    fn default() -> Self {
        Self {
            openrouter: OpenRouterConfig::default(),
            ui: UiConfig::default(),
            chat: ChatConfig::default(),
            security: SecurityConfig::default(),
            memory: MemoryConfig::default(),
            performance: PerformanceConfig::default(),
            buddy: BuddyConfig::default(),
            model: None,
            temperature: None,
            max_tokens: None,
            theme: None,
            editor: None,
            auto_approve_tools: None,
            verbosity: None,
            custom: HashMap::new(),
        }
    }
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            default_model: default_model(),
            timeout_secs: default_timeout(),
            streaming: default_true(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            syntax_highlighting: default_true(),
            show_token_usage: default_true(),
            color_support: ColorSupport::default(),
        }
    }
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            history_enabled: default_true(),
            history_size: default_history_size(),
            auto_completion: default_true(),
            typing_indicator: default_true(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            audit_logging: default_true(),
            log_file: None,
            path_traversal_protection: default_true(),
            command_injection_protection: default_true(),
            block_sensitive_files: default_true(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            max_history_size: default_max_history(),
            context_token_limit: default_context_limit(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: None,
            response_caching: false,
            cache_dir: None,
        }
    }
}

impl Default for BuddyConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            auto_summon: false,
            preferred_buddy: None,
        }
    }
}

impl HcscoderConfig {
    pub fn new() -> Self {
        HcscoderConfig::default()
    }

    /// Get config file path
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("could not resolve home directory")?;
        Ok(home.join(".hcscoder").join("config.toml"))
    }

    /// Load configuration from disk (TOML)
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            debug!("Config file not found, using defaults");
            return Ok(HcscoderConfig::default());
        }

        let content = std::fs::read_to_string(&path)
            .context("Failed to read config file")?;
        
        let config: HcscoderConfig = toml::from_str(&content)
            .context("Failed to parse config file (TOML)")?;
        
        info!("Configuration loaded from {:?}", path);
        Ok(config)
    }

    /// Save configuration to disk (TOML)
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;
        
        std::fs::write(&path, content)?;

        // Set secure permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, PermissionsExt::from_mode(0o600))?;
        }

        info!("Configuration saved to {:?}", path);
        Ok(())
    }

    /// Create default config file
    pub fn create_default() -> Result<Self> {
        let config = HcscoderConfig::default();
        config.save()?;
        Ok(config)
    }

    /// Check if config file exists
    pub fn exists() -> bool {
        Self::config_path().map(|p| p.exists()).unwrap_or(false)
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
