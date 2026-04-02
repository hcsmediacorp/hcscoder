//! hcscoder OpenRouter model selection utilities
//!
//! Provides model discovery and tier-based selection.
//! Zero telemetry, no external API calls for model listing.

use serde::{Deserialize, Serialize};

/// Model tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HcscoderModelTier {
    Free,
    Standard,
    Performance,
    Premium,
}

impl std::fmt::Display for HcscoderModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free => write!(f, "Free"),
            Self::Standard => write!(f, "Standard"),
            Self::Performance => write!(f, "Performance"),
            Self::Premium => write!(f, "Premium"),
        }
    }
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub tier: HcscoderModelTier,
    pub context_length: u32,
    pub description: &'static str,
}

/// Pre-defined model catalog (curated; verify current ids at [openrouter.ai/models](https://openrouter.ai/models)).
/// **Free** models include `:free` in the id and draw from OpenRouter’s free tier (rate limits apply).
/// **Paid** models bill per OpenRouter usage; pricing is shown on the model page.
pub const MODEL_CATALOG: &[HcscoderModelInfo] = &[
    // Free Tier (OpenRouter `:free` — no per-request charge; subject to rate limits)
    HcscoderModelInfo {
        id: "meta-llama/llama-3.1-8b-instruct:free",
        name: "Llama 3.1 8B (Free)",
        tier: HcscoderModelTier::Free,
        context_length: 8192,
        description: "Fast, lightweight model for simple tasks",
    },
    HcscoderModelInfo {
        id: "google/gemma-2-9b-it:free",
        name: "Gemma 2 9B (Free)",
        tier: HcscoderModelTier::Free,
        context_length: 8192,
        description: "Google's efficient open model",
    },
    HcscoderModelInfo {
        id: "microsoft/phi-3-mini-128k-instruct:free",
        name: "Phi-3 Mini (Free)",
        tier: HcscoderModelTier::Free,
        context_length: 128000,
        description: "Microsoft's compact model with large context",
    },
    // Standard Tier (paid / usage-based on OpenRouter)
    HcscoderModelInfo {
        id: "anthropic/claude-3.5-haiku",
        name: "Claude 3.5 Haiku",
        tier: HcscoderModelTier::Standard,
        context_length: 200000,
        description: "Fast and responsive for everyday tasks",
    },
    HcscoderModelInfo {
        id: "google/gemini-2.0-flash-001",
        name: "Gemini 2.0 Flash",
        tier: HcscoderModelTier::Standard,
        context_length: 1048576,
        description: "Google's balanced performance model",
    },
    HcscoderModelInfo {
        id: "meta-llama/llama-3.1-70b-instruct",
        name: "Llama 3.1 70B",
        tier: HcscoderModelTier::Standard,
        context_length: 131072,
        description: "Powerful open model for complex tasks",
    },
    // Performance Tier
    HcscoderModelInfo {
        id: "anthropic/claude-3.5-sonnet",
        name: "Claude 3.5 Sonnet",
        tier: HcscoderModelTier::Performance,
        context_length: 200000,
        description: "Excellent balance of speed and intelligence",
    },
    HcscoderModelInfo {
        id: "deepseek/deepseek-chat",
        name: "DeepSeek Chat",
        tier: HcscoderModelTier::Performance,
        context_length: 128000,
        description: "Advanced reasoning capabilities",
    },
    HcscoderModelInfo {
        id: "mistralai/mistral-large",
        name: "Mistral Large",
        tier: HcscoderModelTier::Performance,
        context_length: 131072,
        description: "Mistral's most capable model",
    },
    // Premium Tier
    HcscoderModelInfo {
        id: "anthropic/claude-sonnet-4-20250514",
        name: "Claude Sonnet 4",
        tier: HcscoderModelTier::Premium,
        context_length: 200000,
        description: "Latest Claude model with maximum capabilities",
    },
    HcscoderModelInfo {
        id: "openai/o1",
        name: "OpenAI o1",
        tier: HcscoderModelTier::Premium,
        context_length: 200000,
        description: "OpenAI's reasoning-focused model",
    },
    HcscoderModelInfo {
        id: "openai/gpt-4o",
        name: "GPT-4o",
        tier: HcscoderModelTier::Premium,
        context_length: 128000,
        description: "OpenAI's flagship multimodal model",
    },
    HcscoderModelInfo {
        id: "anthropic/claude-3-opus",
        name: "Claude 3 Opus",
        tier: HcscoderModelTier::Premium,
        context_length: 200000,
        description: "Most powerful Claude 3 model",
    },
];

/// Get model by ID
pub fn get_model(model_id: &str) -> Option<&'static HcscoderModelInfo> {
    MODEL_CATALOG.iter().find(|m| m.id == model_id)
}

/// Get models by tier
pub fn get_models_by_tier(tier: HcscoderModelTier) -> Vec<&'static HcscoderModelInfo> {
    MODEL_CATALOG.iter().filter(|m| m.tier == tier).collect()
}

/// Display interactive model selection menu
pub fn display_model_menu() {
    println!("\n📦 hcscoder Model Selection");
    println!("=========================\n");

    for tier in [
        HcscoderModelTier::Free,
        HcscoderModelTier::Standard,
        HcscoderModelTier::Performance,
        HcscoderModelTier::Premium,
    ] {
        let models = get_models_by_tier(tier);
        if models.is_empty() {
            continue;
        }

        println!("【{}】", tier);
        println!("{}", "-".repeat(50));

        for model in models {
            println!("  • {} ({})", model.name, model.id);
            println!(
                "    Context: {} tokens | {}",
                format_context(model.context_length),
                model.description
            );
        }
        println!();
    }

    println!("💡 Set model via:");
    println!("   --model <model_id>");
    println!("   or OPENROUTER_MODEL=<model_id>");
}

fn format_context(tokens: u32) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.0}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

/// Get default model
pub fn get_default_model() -> &'static str {
    "anthropic/claude-3.5-haiku"
}

/// Validate model ID
pub fn is_valid_model(model_id: &str) -> bool {
    get_model(model_id).is_some() || model_id.contains('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model() {
        assert!(get_model("anthropic/claude-3.5-haiku").is_some());
        assert!(get_model("nonexistent/model").is_none());
    }

    #[test]
    fn test_get_models_by_tier() {
        let free_models = get_models_by_tier(HcscoderModelTier::Free);
        assert!(!free_models.is_empty());

        let premium_models = get_models_by_tier(HcscoderModelTier::Premium);
        assert!(!premium_models.is_empty());
    }
}
