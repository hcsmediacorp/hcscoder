//! hcscoder Core Engine Module
//!
//! Handles query processing, multi-agent orchestration, and swarm logic.
//! Zero telemetry, no phone-home logic.

pub mod coordinator;
pub mod query_engine;
pub mod tool_runtime;

use crate::hcscoder_openrouter::client::{ChatMessage, HcscoderApiClient, MessageRole};
use anyhow::Result;

/// Handle a single query and return response
pub async fn handle_single_query(
    api_key: Option<String>,
    model: String,
    query: &str,
    plain: bool,
) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};

    if !plain {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner().template("{spinner} 🤔 Processing... {msg}")?,
        );
        pb.set_message("Initializing");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
    }

    let client = if let Some(key) = api_key {
        HcscoderApiClient::with_config(model, key, None)?
    } else {
        HcscoderApiClient::new(model)?
    };

    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: query.to_string(),
    }];

    let response = client.create_completion(messages, None, None).await?;

    if let Some(choice) = response.choices.first() {
        println!("\n{}\n", choice.message.content);

        if let Some(usage) = &response.usage {
            eprintln!(
                "Tokens: {} prompt + {} completion = {} total",
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
            );
        }
    }

    Ok(())
}

/// Review code at a given path
pub async fn review_code(
    api_key: Option<String>,
    model: String,
    path: &str,
    _plain: bool,
) -> Result<()> {
    use std::fs;
    use std::path::Path as StdPath;

    let std_path = StdPath::new(path);

    if !std_path.exists() {
        anyhow::bail!("Path does not exist: {}", path);
    }

    let content = if std_path.is_file() {
        fs::read_to_string(std_path)?
    } else if std_path.is_dir() {
        // Read first few files in directory
        let mut combined = String::new();
        let mut count = 0;
        for entry in fs::read_dir(std_path)? {
            if count >= 5 {
                break;
            }
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    combined.push_str(&format!("=== {} ===\n{}\n\n", path.display(), content));
                    count += 1;
                }
            }
        }
        combined
    } else {
        anyhow::bail!("Invalid path type: {}", path);
    };

    let client = if let Some(key) = api_key {
        HcscoderApiClient::with_config(model, key, None)?
    } else {
        HcscoderApiClient::new(model)?
    };

    let prompt = format!(
        "Please review the following code and provide:\n\
         1. Code quality assessment\n\
         2. Potential bugs or issues\n\
         3. Suggestions for improvement\n\
         4. Security considerations\n\n\
         Code:\n{}",
        content
    );

    let messages = vec![
        ChatMessage {
            role: MessageRole::System,
            content: "You are an expert code reviewer. Provide concise, actionable feedback."
                .to_string(),
        },
        ChatMessage {
            role: MessageRole::User,
            content: prompt,
        },
    ];

    println!("🔍 Analyzing code...\n");

    let response = client
        .create_completion(messages, Some(0.3), Some(2000))
        .await?;

    if let Some(choice) = response.choices.first() {
        println!("{}", choice.message.content);
    }

    Ok(())
}

/// Get model recommendations based on task type
pub fn recommend_model(task_type: &str) -> &'static str {
    match task_type {
        "quick" | "simple" => "meta-llama/llama-3.1-8b-instruct:free",
        "code" | "programming" => "anthropic/claude-3.5-haiku",
        "complex" | "reasoning" => "anthropic/claude-3.5-sonnet",
        "creative" | "writing" => "google/gemini-2.0-flash-001",
        _ => "anthropic/claude-3.5-haiku",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_recommendation() {
        assert_eq!(
            recommend_model("quick"),
            "meta-llama/llama-3.1-8b-instruct:free"
        );
        assert_eq!(recommend_model("code"), "anthropic/claude-3.5-haiku");
        assert_eq!(recommend_model("complex"), "anthropic/claude-3.5-sonnet");
    }
}
