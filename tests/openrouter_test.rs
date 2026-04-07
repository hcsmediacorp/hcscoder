//! Unit tests for hcscoder_openrouter module

use hcscoder::hcscoder_openrouter::{auth, HcscoderOpenRouterConfig};

#[test]
fn test_validate_api_key_valid_format() {
    assert!(auth::validate_api_key("sk-or-a1b2c3d4e5f6g7h8"));
}

#[test]
fn test_validate_api_key_invalid_prefix() {
    assert!(!auth::validate_api_key("invalid-prefix-a1b2c3d4e5f6g7h8"));
}

#[test]
fn test_validate_api_key_too_short() {
    assert!(!auth::validate_api_key("sk-or-short"));
}

#[test]
fn test_validate_api_key_empty() {
    assert!(!auth::validate_api_key(""));
}

#[test]
fn test_validate_api_key_null_byte() {
    assert!(!auth::validate_api_key("sk-or-key\0with\0nulls"));
}

#[test]
fn test_validate_api_key_special_characters() {
    assert!(!auth::validate_api_key("sk-or-key@with#special$chars"));
}

#[test]
fn test_validate_api_key_legacy_format() {
    assert!(auth::validate_api_key("sk-legacy-format-key-12345"));
}

#[test]
fn test_init_validation_rejects_low_entropy_key() {
    let err = HcscoderOpenRouterConfig::validate_init_api_key("sk-or-aaaaaaaaaaaaaaaaaaaa")
        .unwrap_err()
        .to_string();
    assert!(err.contains("entropy"));
}

#[test]
fn test_init_validation_reports_format_failure() {
    let err = HcscoderOpenRouterConfig::validate_init_api_key("not-openrouter-format-123456789")
        .unwrap_err()
        .to_string();
    assert!(err.contains("format"));
}
