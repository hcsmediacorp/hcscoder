//! hcscoder OpenRouter authentication module
//!
//! Handles API key management with secure storage.
//! Zero telemetry, no external tracking.

use crate::hcscoder_openrouter::HcscoderOpenRouterConfig;
use anyhow::{Context, Result};

/// Get API key from environment or config
pub fn get_api_key() -> Result<String> {
    HcscoderOpenRouterConfig::load_api_key().ok_or_else(|| {
        anyhow::anyhow!(
            "No OpenRouter API key found.\n\n\
             Please set one of the following:\n\
             1. Environment variable: OPENROUTER_API_KEY\n\
             2. Config file: ~/.hcscoder/openrouter_api_key\n\
             3. Run: hcscoder init or hcscoder-setup"
        )
    })
}

/// Validate API key format
pub fn validate_api_key(key: &str) -> bool {
    // OpenRouter keys start with "sk-or-" followed by alphanumeric and hyphens
    // Minimum length 20, maximum 512 characters
    if key.len() < 20 || key.len() > 512 {
        return false;
    }
    
    // Check for proper prefix pattern (sk-or-)
    if !key.starts_with("sk-or-") {
        // Allow legacy keys without prefix but warn
        tracing::warn!("API key does not start with 'sk-or-' prefix. Consider regenerating your key.");
    }
    
    // Ensure key contains only valid characters (alphanumeric, hyphen, underscore)
    // This prevents injection attacks and malformed keys
    key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Securely clear API key from config
pub fn clear_api_key() -> Result<()> {
    let path = HcscoderOpenRouterConfig::api_key_path()?;
    if path.exists() {
        std::fs::remove_file(&path).context("Failed to remove API key file")?;
        tracing::info!("API key cleared from {:?}", path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key() {
        assert!(validate_api_key("sk-or-valid-key-1234567890"));
        assert!(validate_api_key("abcdefghijklmnopqrstuvwxyz"));
        assert!(!validate_api_key(""));
        assert!(!validate_api_key("short"));
    }
}
