//! hcscoder Query Engine
//!
//! Core query processing and response handling.
//! Zero telemetry, no phone-home logic.

use crate::hcscoder_openrouter::client::{ChatMessage, HcscoderApiClient, MessageRole};
use anyhow::Result;

/// System prompt for coding assistance
pub const HCS_CODER_SYSTEM_PROMPT: &str = r#"You are hcscoder, a high-performance AI coding assistant by hcsmedia.

Guidelines:
- Provide concise, accurate, and practical solutions
- Include code examples when relevant
- Explain complex concepts clearly
- Follow best practices and security considerations
- Respect user's existing code style and patterns
- No telemetry or data collection - privacy-first design

Remember: You are running locally with zero tracking. Focus on helping the user efficiently."#;

/// Process a query with conversation history
pub async fn process_query(
    client: &HcscoderApiClient,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
) -> Result<String> {
    let response = client
        .create_completion(messages, temperature, max_tokens)
        .await?;

    response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow::anyhow!("No response from model"))
}

/// Stream a query response
pub async fn stream_query(
    client: &HcscoderApiClient,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
) -> Result<tokio_stream::wrappers::ReceiverStream<Result<String>>> {
    use futures_util::stream::StreamExt;
    use tokio_stream::wrappers::ReceiverStream;

    let (tx, rx) = tokio::sync::mpsc::channel(32);

    let stream = client
        .create_stream(messages, temperature, max_tokens)
        .await?;

    tokio::spawn(async move {
        let mut stream = stream;
        while let Some(chunk) = stream.next().await {
            if tx.send(chunk).await.is_err() {
                break;
            }
        }
    });

    Ok(ReceiverStream::new(rx))
}

/// Create a conversation context with system prompt
pub fn create_conversation(user_message: &str) -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: MessageRole::System,
            content: HCS_CODER_SYSTEM_PROMPT.to_string(),
        },
        ChatMessage {
            role: MessageRole::User,
            content: user_message.to_string(),
        },
    ]
}

/// Add message to conversation history
pub fn add_to_conversation(messages: &mut Vec<ChatMessage>, role: MessageRole, content: String) {
    messages.push(ChatMessage { role, content });
}

/// Truncate conversation history to fit within token limits
pub fn truncate_conversation(messages: &mut Vec<ChatMessage>, max_messages: usize) {
    // Keep system prompt and last N messages
    if messages.len() > max_messages + 1 {
        // Remove oldest messages (keeping system prompt at index 0)
        let drain_start = std::cmp::max(1, messages.len() - max_messages);
        messages.drain(1..drain_start);
    }
}

/// Estimate token count (rough approximation)
pub fn estimate_tokens(text: &str) -> usize {
    // Rough estimate: ~4 characters per token for English
    text.chars().count() / 4
}

/// Check if message exceeds token limit
pub fn would_exceed_limit(messages: &[ChatMessage], new_message: &str, limit: usize) -> bool {
    let current_tokens: usize = messages.iter().map(|m| estimate_tokens(&m.content)).sum();

    let new_tokens = estimate_tokens(new_message);

    current_tokens + new_tokens > limit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_conversation() {
        let messages = create_conversation("Hello!");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, MessageRole::System);
        assert_eq!(messages[1].role, MessageRole::User);
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "This is a test message with approximately 10 tokens.";
        let tokens = estimate_tokens(text);
        assert!(tokens > 5 && tokens < 20);
    }

    #[test]
    fn test_truncate_conversation() {
        let mut messages = create_conversation("First");
        add_to_conversation(
            &mut messages,
            MessageRole::Assistant,
            "Response 1".to_string(),
        );
        add_to_conversation(&mut messages, MessageRole::User, "Second".to_string());
        add_to_conversation(
            &mut messages,
            MessageRole::Assistant,
            "Response 2".to_string(),
        );

        assert_eq!(messages.len(), 5);

        truncate_conversation(&mut messages, 2);
        assert!(messages.len() <= 3); // System + 2 messages
    }
}
