//! hcscoder OpenRouter API client
//!
//! Handles all communication with OpenRouter API.
//! Zero telemetry, no phone-home logic.

pub mod auth;
pub mod client;
pub mod models;

use crate::hcscoder_tools::config::HcscoderConfig;
use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for OpenRouter API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderOpenRouterConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
    pub timeout_secs: u64,
}

impl Default for HcscoderOpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: "anthropic/claude-3.5-haiku".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            timeout_secs: 60,
        }
    }
}

impl HcscoderOpenRouterConfig {
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let home = home_dir().context("Failed to get home directory")?;
        let config_dir = home.join(".hcscoder");

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .context("Failed to create hcscoder config directory")?;
        }

        // Unix: `save_api_key` sets key file mode to 0o600.
        // Windows: explicit ACL hardening is not implemented in this crate yet; file privacy
        // depends on the user profile ACL defaults.

        Ok(config_dir)
    }

    /// Get the API key file path
    pub fn api_key_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("openrouter_api_key"))
    }

    /// Default model id file (written by `hcscoder-setup`, lower priority than `OPENROUTER_MODEL`).
    pub fn default_model_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("openrouter_default_model"))
    }

    /// Load saved default model from `~/.hcscoder/openrouter_default_model`.
    pub fn load_saved_model() -> Option<String> {
        Self::default_model_path()
            .ok()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Persist default model id (single line).
    pub fn save_default_model(model: &str) -> Result<()> {
        let path = Self::default_model_path()?;
        std::fs::write(&path, model.trim())
            .with_context(|| format!("failed to write default model to {:?}", path))?;
        tracing::info!("Default model saved to {:?}", path);
        Ok(())
    }

    /// Load API key from environment or config file
    pub fn load_api_key() -> Option<String> {
        // Priority 1: Environment variable
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            if !key.is_empty() {
                tracing::debug!("Loaded API key from OPENROUTER_API_KEY environment variable");
                return Some(key);
            }
        }

        // Priority 2: Config file
        if let Ok(path) = Self::api_key_path() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let key = content.trim().to_string();
                    if !key.is_empty() {
                        tracing::debug!("Loaded API key from config file: {:?}", path);
                        return Some(key);
                    }
                }
            }
        }

        None
    }

    /// Save API key to config file
    pub fn save_api_key(key: &str) -> Result<()> {
        let path = Self::api_key_path()?;
        std::fs::write(&path, key.trim())
            .context(format!("Failed to save API key to {:?}", path))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o600); // Owner read/write only
            std::fs::set_permissions(&path, perms)?;
        }

        tracing::info!("API key saved to {:?}", path);
        Ok(())
    }

    pub fn validate_init_api_key(key: &str) -> Result<()> {
        crate::hcscoder_openrouter::auth::validate_api_key_strict(key)
            .map_err(|e| anyhow::anyhow!("Invalid API key: {}", e))
    }

    /// Initialize configuration interactively
    pub async fn init_config() -> Result<()> {
        use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

        let mut stdin = BufReader::new(io::stdin());
        let mut stdout = io::stdout();

        stdout.write_all(b"Enter your OpenRouter API key: ").await?;
        stdout.flush().await?;

        let mut input = String::new();
        stdin.read_line(&mut input).await?;
        let key = input.trim().to_string();

        if key.is_empty() {
            anyhow::bail!("API key cannot be empty");
        }

        Self::validate_init_api_key(&key)?;

        Self::save_api_key(&key)?;

        stdout
            .write_all("\nAPI key saved successfully.\n".as_bytes())
            .await?;
        stdout.flush().await?;

        Ok(())
    }

    /// Show current configuration status
    pub fn show_status(_api_key: Option<String>, model: String, config: &HcscoderConfig) {
        println!("🔧 hcscoder Configuration Status");
        println!("================================");
        println!();

        // API Key status
        let key_source = if std::env::var("OPENROUTER_API_KEY").is_ok() {
            "Environment variable (OPENROUTER_API_KEY)"
        } else if Self::load_api_key().is_some() {
            "Config file (~/.hcscoder/openrouter_api_key)"
        } else {
            "Not configured"
        };

        println!("API Key Source: {}", key_source);
        println!("Current Model:  {}", model);
        println!("Model Source:   {}", resolve_model_source(config));
        println!("Base URL:       https://openrouter.ai/api/v1");
        println!("Theme Source:   {}", resolve_theme_source(config));
        println!();

        // Model tier info
        let tier = if model.contains(":free") || model.contains("/free") {
            "Free Tier"
        } else if model.contains("sonnet")
            || model.contains("opus")
            || model.contains("o1")
            || model.contains("gpt-4")
        {
            "Premium Tier"
        } else {
            "Standard Tier"
        };
        println!("Model Tier:     {}", tier);
        println!();

        println!("💡 Tip: Change model with --model or OPENROUTER_MODEL env var");
    }
}

fn resolve_model_source(config: &HcscoderConfig) -> &'static str {
    if std::env::var("OPENROUTER_MODEL").is_ok() {
        "Environment variable (OPENROUTER_MODEL)"
    } else if config.model.is_some() && HcscoderConfig::exists() {
        "Config file (~/.hcscoder/config.toml -> model)"
    } else if HcscoderConfig::exists() {
        "Config file (~/.hcscoder/config.toml -> openrouter.default_model)"
    } else if HcscoderOpenRouterConfig::load_saved_model().is_some() {
        "Saved model file (~/.hcscoder/openrouter_default_model)"
    } else {
        "Built-in default"
    }
}

fn resolve_theme_source(config: &HcscoderConfig) -> &'static str {
    if std::env::var("HCSCODER_THEME").is_ok() {
        "Environment variable (HCSCODER_THEME)"
    } else if config.theme.is_some() && HcscoderConfig::exists() {
        "Config file (~/.hcscoder/config.toml -> theme)"
    } else {
        "Built-in or config default (~/.hcscoder/config.toml -> ui.theme)"
    }
}

/// Initialize configuration (called from CLI)
pub async fn init_config() -> Result<()> {
    HcscoderOpenRouterConfig::init_config().await
}

/// Show status (called from CLI)
pub fn show_status(api_key: Option<String>, model: String, config: &HcscoderConfig) {
    HcscoderOpenRouterConfig::show_status(api_key, model, config);
}
