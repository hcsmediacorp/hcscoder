//! BriefTool - Send messages to the user
//!
//! Primary communication channel for sending messages that users will actually read.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Message status for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Normal,
    Proactive,
}

/// Attachment for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub path: String,
    pub description: Option<String>,
}

/// BriefTool for sending messages to users
pub struct BriefTool;

impl BriefTool {
    /// Create a new BriefTool instance
    pub fn new() -> Self {
        Self
    }

    /// Send a message to the user
    pub async fn send_message(
        &self,
        message: String,
        attachments: Option<Vec<MessageAttachment>>,
        status: Option<MessageStatus>,
    ) -> Result<serde_json::Value> {
        // Validate message is not empty
        if message.trim().is_empty() {
            anyhow::bail!("Message cannot be empty");
        }

        // Process attachments if provided
        let processed_attachments = if let Some(atts) = attachments {
            let mut validated = Vec::new();
            for att in atts {
                // Verify file exists (basic check)
                if !std::path::Path::new(&att.path).exists() {
                    anyhow::bail!("Attachment file not found: {}", att.path);
                }
                validated.push(att);
            }
            Some(validated)
        } else {
            None
        };

        let result = json!({
            "status": "sent",
            "message": message,
            "attachments": processed_attachments.map(|a| a.iter().map(|att| json!({
                "path": att.path,
                "description": att.description
            })).collect::<Vec<_>>()),
            "status_label": status.map(|s| match s {
                MessageStatus::Normal => "normal",
                MessageStatus::Proactive => "proactive",
            }),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        // In a real implementation, this would display to the user
        // For now, we log it and return success
        tracing::info!(
            "Brief message sent: {}",
            message.chars().take(100).collect::<String>()
        );

        Ok(result)
    }

    /// Execute brief tool and return JSON result
    pub async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let message = params["message"]
            .as_str()
            .context("Missing 'message' parameter")?
            .to_string();

        let attachments = params["attachments"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let path = v["path"].as_str()?.to_string();
                    let description = v["description"].as_str().map(String::from);
                    Some(MessageAttachment { path, description })
                })
                .collect()
        });

        let status = params["status"].as_str().map(|s| match s {
            "normal" => MessageStatus::Normal,
            "proactive" => MessageStatus::Proactive,
            _ => MessageStatus::Normal,
        });

        self.send_message(message, attachments, status).await
    }
}

impl Default for BriefTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        let tool = BriefTool::new();
        let result = tool
            .send_message(
                "Test message".to_string(),
                None,
                Some(MessageStatus::Normal),
            )
            .await
            .unwrap();

        assert_eq!(result["status"], "sent");
        assert_eq!(result["message"], "Test message");
    }

    #[tokio::test]
    async fn test_empty_message_rejected() {
        let tool = BriefTool::new();
        let result = tool.send_message("   ".to_string(), None, None).await;

        assert!(result.is_err());
    }
}
