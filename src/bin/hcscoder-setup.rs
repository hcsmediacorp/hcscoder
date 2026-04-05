//! hcscoder Setup CLI Binary
//!
//! Interactive setup utility for first-time configuration.
//! Zero telemetry, no phone-home logic.

#![deny(warnings)]

use anyhow::{Context, Result};
use hcscoder::hcscoder_openrouter::models;
use hcscoder::hcscoder_openrouter::HcscoderOpenRouterConfig;

fn main() -> Result<()> {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        println!("hcscoder-setup - Interactive setup for hcscoder");
        println!();
        println!("Usage:");
        println!("  hcscoder-setup");
        println!();
        println!("Options:");
        println!("  -h, --help    Show this help message and exit");
        return Ok(());
    }

    println!("🚀 hcscoder Setup Utility");
    println!("{}", "=".repeat(50));
    println!();

    // Get API key (hidden input; not echoed to terminal)
    let api_key = rpassword::prompt_password("Enter your OpenRouter API key: ")
        .context("failed to read API key from terminal")?;

    if api_key.len() < 20 {
        eprintln!("❌ API key appears too short. Please check and try again.");
        std::process::exit(1);
    }

    HcscoderOpenRouterConfig::save_api_key(&api_key)?;

    println!();
    println!("✅ API key saved securely to ~/.hcscoder/openrouter_api_key");
    println!();

    let model = select_model()?;
    HcscoderOpenRouterConfig::save_default_model(&model)?;

    println!();
    println!("✅ Default model saved to ~/.hcscoder/openrouter_default_model");
    println!();
    println!("🎉 Setup complete!");
    println!();
    println!("You can now use hcscoder:");
    println!("  hcscoder chat          - Start interactive chat");
    println!("  hcscoder ask <query>   - Ask a single question");
    println!("  hcscoder status        - View configuration");
    println!();
    println!("Or set environment variables:");
    println!("  export OPENROUTER_API_KEY='your-key'");
    println!("  export OPENROUTER_MODEL='{}'", model);
    println!();

    Ok(())
}

fn select_model() -> Result<String> {
    use std::io::{self, Write};

    println!("Select your preferred model tier:");
    println!();
    println!("  1) Free Tier      - No cost, rate limited (OpenRouter :free models)");
    println!("  2) Standard Tier  - Balanced performance/cost");
    println!("  3) Performance    - Complex reasoning tasks");
    println!("  4) Premium        - Maximum capabilities");
    println!();
    print!("Enter choice (1-4, default: 2): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    let model = match choice.trim() {
        "1" => "meta-llama/llama-3.1-8b-instruct:free",
        "3" => "anthropic/claude-3.5-sonnet",
        "4" => "anthropic/claude-sonnet-4-20250514",
        _ => "anthropic/claude-3.5-haiku",
    };

    println!();
    println!("Selected model: {}", model);
    println!();
    println!("💡 You can change this anytime with:");
    println!("   --model <model_id> or OPENROUTER_MODEL env var");
    println!();

    models::display_model_menu();

    Ok(model.to_string())
}
