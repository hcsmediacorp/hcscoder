//! Example: Basic usage of hcscoder as a library
//!
//! This example demonstrates how to use hcscoder's OpenRouter client
//! to interact with AI models programmatically.

use hcscoder::hcscoder_openrouter::{auth, client::OpenRouterClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the OpenRouter client
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .unwrap_or_else(|_| "sk-or-example-key".to_string());

    if !auth::validate_api_key(&api_key) {
        eprintln!("Invalid API key format");
        std::process::exit(1);
    }

    println!("API key validated successfully!");

    // Create the client (example only - requires valid API key)
    // let client = OpenRouterClient::new(&api_key)?;

    println!("Example complete. Set OPENROUTER_API_KEY to run with actual API.");

    Ok(())
}
