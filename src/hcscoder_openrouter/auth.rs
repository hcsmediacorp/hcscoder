//! hcscoder OpenRouter authentication module
//!
//! Handles API key management with secure storage.
//! Zero telemetry, no external tracking.
//!
//! ## Security Features
//! - Strict regex validation for API key format
//! - Entropy checking to detect weak keys
//! - Platform-accurate key-file permission handling notes

use crate::hcscoder_openrouter::HcscoderOpenRouterConfig;
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref API_KEY_REGEX: Regex = Regex::new(r"^sk-or-[a-zA-Z0-9_-]{14,}$").unwrap();
    static ref LEGACY_KEY_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]{20,}$").unwrap();
    static ref SK_LEGACY_REGEX: Regex = Regex::new(r"^sk-[a-zA-Z0-9_-]{17,}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyValidationError {
    Empty,
    Length {
        actual: usize,
        min: usize,
        max: usize,
    },
    InvalidCharacters,
    NullByte,
    Format,
    LowEntropy,
}

impl std::fmt::Display for ApiKeyValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "API key cannot be empty"),
            Self::Length { actual, min, max } => write!(
                f,
                "API key length is invalid: got {actual}, expected {min}-{max} characters"
            ),
            Self::InvalidCharacters => write!(f, "API key contains invalid characters"),
            Self::NullByte => write!(f, "API key contains null byte"),
            Self::Format => write!(
                f,
                "API key format is invalid (expected sk-or-..., legacy sk-..., or legacy alphanumeric key)"
            ),
            Self::LowEntropy => write!(f, "API key entropy appears too low"),
        }
    }
}

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

pub fn validate_api_key_strict(key: &str) -> std::result::Result<(), ApiKeyValidationError> {
    if key.is_empty() {
        return Err(ApiKeyValidationError::Empty);
    }

    if key.len() < 20 || key.len() > 512 {
        return Err(ApiKeyValidationError::Length {
            actual: key.len(),
            min: 20,
            max: 512,
        });
    }

    if key.contains('\0') {
        return Err(ApiKeyValidationError::NullByte);
    }

    if !key
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ApiKeyValidationError::InvalidCharacters);
    }

    if !has_sufficient_entropy(key) {
        return Err(ApiKeyValidationError::LowEntropy);
    }

    if API_KEY_REGEX.is_match(key) {
        return Ok(());
    }
    if SK_LEGACY_REGEX.is_match(key) {
        tracing::warn!("API key uses deprecated 'sk-' format.");
        return Ok(());
    }
    if LEGACY_KEY_REGEX.is_match(key) {
        tracing::warn!("API key uses legacy format without 'sk-or-' prefix.");
        return Ok(());
    }

    Err(ApiKeyValidationError::Format)
}

pub fn validate_api_key(key: &str) -> bool {
    validate_api_key_strict(key).is_ok()
}

fn has_sufficient_entropy(key: &str) -> bool {
    if key
        .chars()
        .all(|c| c == key.chars().next().unwrap_or_default())
    {
        return false;
    }

    let unique_chars: std::collections::HashSet<char> = key.chars().collect();
    if unique_chars.len() <= 2 {
        return false;
    }

    let normalized = key.strip_prefix("sk-or-").unwrap_or(key).to_lowercase();
    let starts_alpha_seq = normalized.starts_with("abcdef");
    let starts_digit_seq = normalized.starts_with("123456");
    let has_alpha = normalized.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = normalized.chars().any(|c| c.is_ascii_digit());
    if (starts_alpha_seq && has_digit) || (starts_digit_seq && has_alpha) {
        return false;
    }

    let mut char_counts = std::collections::HashMap::new();
    for c in key.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }

    if let Some(max_count) = char_counts.values().max() {
        if *max_count > key.len() / 2 {
            return false;
        }
    }

    true
}

pub fn clear_api_key() -> Result<()> {
    let path = HcscoderOpenRouterConfig::api_key_path()?;
    if path.exists() {
        std::fs::remove_file(&path).context("Failed to remove API key file")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_validator_reports_length() {
        let err = validate_api_key_strict("short").unwrap_err();
        assert!(matches!(err, ApiKeyValidationError::Length { .. }));
    }

    #[test]
    fn strict_validator_reports_format() {
        let err = validate_api_key_strict("abc-def-ghijklmno123456789").unwrap_err();
        assert_eq!(err, ApiKeyValidationError::Format);
    }

    #[test]
    fn strict_validator_reports_entropy() {
        let err = validate_api_key_strict("sk-or-aaaaaaaaaaaaaaaaaaaaaa").unwrap_err();
        assert_eq!(err, ApiKeyValidationError::LowEntropy);
    }

    #[test]
    fn strict_validator_accepts_valid_key() {
        assert!(validate_api_key_strict("sk-or-valid-key-1234567890").is_ok());
    }
}
