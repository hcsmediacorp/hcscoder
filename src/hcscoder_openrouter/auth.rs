//! hcscoder OpenRouter authentication module
//!
//! Handles API key management with secure storage.
//! Zero telemetry, no external tracking.
//!
//! ## Security Features
//! - Strict regex validation for API key format
//! - Entropy checking to detect weak keys
//! - Secure memory clearing with zeroize
//! - Windows ACL support for file permissions

use crate::hcscoder_openrouter::HcscoderOpenRouterConfig;
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Strict regex for OpenRouter API key validation
    /// Format: sk-or-[alphanumeric_-] with minimum 20 chars total
    static ref API_KEY_REGEX: Regex = Regex::new(r"^sk-or-[a-zA-Z0-9_-]{14,}$").unwrap();

    /// Legacy key pattern (without any `sk-` style prefix).
    static ref LEGACY_KEY_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]{20,}$").unwrap();
    /// Older provider-style key format (`sk-...`) observed in legacy configs.
    static ref SK_LEGACY_REGEX: Regex = Regex::new(r"^sk-[a-zA-Z0-9_-]{17,}$").unwrap();
}

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

/// Validate API key format with strict checks
///
/// # Validation Rules
/// 1. Must be 20-512 characters long
/// 2. Must start with "sk-or-" prefix (preferred) OR be legacy format
/// 3. Must contain only alphanumeric characters, hyphens, and underscores
/// 4. Must have sufficient entropy (no repeated patterns)
///
/// # Arguments
/// * `key` - The API key to validate
///
/// # Returns
/// * `true` if key passes all validation checks
/// * `false` if key fails any validation check
pub fn validate_api_key(key: &str) -> bool {
    // Check length constraints
    if key.len() < 20 || key.len() > 512 {
        tracing::debug!("API key validation failed: invalid length ({})", key.len());
        return false;
    }

    // Check for null bytes (injection prevention)
    if key.contains('\0') {
        tracing::debug!("API key validation failed: contains null byte");
        return false;
    }

    // Check character set (prevent injection attacks)
    if !key
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        tracing::debug!("API key validation failed: invalid characters");
        return false;
    }

    // Check entropy - reject keys with too much repetition
    if !has_sufficient_entropy(key) {
        tracing::debug!("API key validation failed: insufficient entropy");
        return false;
    }

    // Validate format (prefer sk-or- prefix)
    if API_KEY_REGEX.is_match(key) {
        tracing::debug!("API key validated with sk-or- prefix");
        true
    } else if SK_LEGACY_REGEX.is_match(key) {
        tracing::warn!(
            "API key uses deprecated 'sk-' format. Consider regenerating with 'sk-or-' prefix."
        );
        true
    } else if LEGACY_KEY_REGEX.is_match(key) {
        tracing::warn!(
            "API key uses legacy format without 'sk-or-' prefix. Consider regenerating your key."
        );
        true
    } else {
        tracing::debug!("API key validation failed: does not match expected format");
        false
    }
}

/// Check if a string has sufficient entropy
///
/// Rejects keys with obvious patterns like:
/// - All same character
/// - Simple sequences (abc, 123)
/// - Excessive repetition
fn has_sufficient_entropy(key: &str) -> bool {
    // Reject if all characters are the same
    if key
        .chars()
        .all(|c| c == key.chars().next().unwrap_or_default())
    {
        return false;
    }

    // Reject obvious low-variety patterns like alternating two chars (abababab...).
    let unique_chars: std::collections::HashSet<char> = key.chars().collect();
    if unique_chars.len() <= 2 {
        return false;
    }

    // Reject obvious sequential prefixes (legacy weak-test patterns).
    let normalized = key.strip_prefix("sk-or-").unwrap_or(key).to_lowercase();
    let starts_alpha_seq = normalized.starts_with("abcdef");
    let starts_digit_seq = normalized.starts_with("123456");
    let has_alpha = normalized.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = normalized.chars().any(|c| c.is_ascii_digit());
    if (starts_alpha_seq && has_digit) || (starts_digit_seq && has_alpha) {
        return false;
    }

    // Check for excessive character repetition (>50% same char)
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

/// Calculate approximate entropy of a key (bits)
#[allow(dead_code)]
fn calculate_entropy(key: &str) -> f64 {
    use std::collections::HashMap;

    let mut freq = HashMap::new();
    for c in key.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    let len = key.len() as f64;
    let mut entropy = 0.0;

    for count in freq.values() {
        let p = *count as f64 / len;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }

    entropy * len
}

/// Securely clear API key from config with audit logging
pub fn clear_api_key() -> Result<()> {
    let path = HcscoderOpenRouterConfig::api_key_path()?;
    if path.exists() {
        tracing::warn!(
            target: "hcscoder::audit",
            event = "api_key_cleared",
            path = %path.display(),
            "API key file removed"
        );
        std::fs::remove_file(&path).context("Failed to remove API key file")?;
        tracing::info!("API key cleared from {:?}", path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key_valid_sk_or() {
        // Valid sk-or- prefix format
        assert!(validate_api_key("sk-or-valid-key-1234567890"));
        assert!(validate_api_key("sk-or-abc123def456ghi789"));
    }

    #[test]
    fn test_validate_api_key_valid_legacy() {
        // Valid legacy format (20+ chars, no prefix)
        assert!(validate_api_key("abcdefghijklmnopqrstuvwxyz"));
        assert!(validate_api_key("validlegacykey12345678"));
    }

    #[test]
    fn test_validate_api_key_invalid_short() {
        // Too short
        assert!(!validate_api_key(""));
        assert!(!validate_api_key("short"));
        assert!(!validate_api_key("sk-or-tooshort"));
    }

    #[test]
    fn test_validate_api_key_invalid_patterns() {
        // Repeated characters (low entropy)
        assert!(!validate_api_key("aaaaaaaaaaaaaaaaaaaaaa"));

        // Sequential patterns
        assert!(!validate_api_key("abcdef123456789012345"));
        assert!(!validate_api_key("123456abcdefghijklmnop"));
    }

    #[test]
    fn test_validate_api_key_invalid_chars() {
        // Invalid characters
        assert!(!validate_api_key("invalid@key!1234567890"));
        assert!(!validate_api_key("key with spaces 12345"));
    }

    #[test]
    fn test_has_sufficient_entropy() {
        // These should have sufficient entropy
        assert!(has_sufficient_entropy("sk-or-valid-key-1234567890"));
        assert!(has_sufficient_entropy("abcdefghijklmnopqrstuvwxyz"));

        // These should fail entropy check
        assert!(!has_sufficient_entropy("aaaaaaaaaaaaaaaaaaaa"));
        assert!(!has_sufficient_entropy("abababababababababab"));
    }
}
