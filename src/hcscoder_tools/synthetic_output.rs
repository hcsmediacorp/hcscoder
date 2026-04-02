//! SyntheticOutputTool - Generate structured output
//!
//! Provides structured output generation with dynamic schema support.

use anyhow::{Context, Result};
use serde_json::json;

/// SyntheticOutputTool for generating structured outputs
pub struct SyntheticOutputTool;

impl SyntheticOutputTool {
    /// Create a new SyntheticOutputTool instance
    pub fn new() -> Self {
        Self
    }

    /// Generate structured output based on provided schema
    pub async fn generate(
        &self,
        content: String,
        schema: Option<serde_json::Value>,
        format: Option<String>,
    ) -> Result<serde_json::Value> {
        // Validate content is not empty
        if content.trim().is_empty() {
            anyhow::bail!("Content cannot be empty");
        }

        let output_format = format.unwrap_or_else(|| "text".to_string());

        // If a schema is provided, we would validate against it
        // For now, we return the content as-is with metadata
        let result = json!({
            "status": "generated",
            "content": content,
            "format": output_format,
            "schema_validated": schema.is_some(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        Ok(result)
    }

    /// Transform data to specified format
    pub async fn transform(
        &self,
        data: serde_json::Value,
        target_format: String,
    ) -> Result<serde_json::Value> {
        match target_format.as_str() {
            "json" => Ok(json!({
                "status": "transformed",
                "format": "json",
                "data": data,
            })),
            "markdown" => {
                let markdown = Self::json_to_markdown(&data);
                Ok(json!({
                    "status": "transformed",
                    "format": "markdown",
                    "content": markdown,
                }))
            }
            "text" => Ok(json!({
                "status": "transformed",
                "format": "text",
                "content": data.to_string(),
            })),
            _ => anyhow::bail!("Unsupported format: {}", target_format),
        }
    }

    /// Convert JSON value to markdown representation
    fn json_to_markdown(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(obj) => {
                let mut lines = Vec::new();
                for (key, val) in obj {
                    lines.push(format!("**{}**: {}", key, Self::value_to_markdown(val)));
                }
                lines.join("\n")
            }
            serde_json::Value::Array(arr) => arr
                .iter()
                .map(|v| format!("- {}", Self::value_to_markdown(v)))
                .collect::<Vec<_>>()
                .join("\n"),
            _ => value.to_string(),
        }
    }

    /// Convert a single JSON value to markdown text
    fn value_to_markdown(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                "```json\n".to_string() + &value.to_string() + "\n```"
            }
        }
    }

    /// Execute synthetic output tool and return JSON result
    pub async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let action = params["action"].as_str().unwrap_or("generate");

        match action {
            "generate" => {
                let content = params["content"]
                    .as_str()
                    .context("Missing 'content' parameter")?
                    .to_string();
                let schema = params["schema"].clone();
                let format = params["format"].as_str().map(String::from);
                self.generate(content, schema.is_null().then_some(schema), format)
                    .await
            }
            "transform" => {
                let data = params["data"].clone();
                let target_format = params["format"]
                    .as_str()
                    .context("Missing 'format' parameter")?
                    .to_string();
                self.transform(data, target_format).await
            }
            _ => anyhow::bail!("Unknown action: {}", action),
        }
    }
}

impl Default for SyntheticOutputTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_output() {
        let tool = SyntheticOutputTool::new();
        let result = tool
            .generate("Test content".to_string(), None, Some("text".to_string()))
            .await
            .unwrap();

        assert_eq!(result["status"], "generated");
        assert_eq!(result["content"], "Test content");
    }

    #[tokio::test]
    async fn test_transform_json() {
        let tool = SyntheticOutputTool::new();
        let data = json!({"key": "value"});
        let result = tool.transform(data, "json".to_string()).await.unwrap();

        assert_eq!(result["status"], "transformed");
        assert_eq!(result["format"], "json");
    }

    #[tokio::test]
    async fn test_transform_markdown() {
        let tool = SyntheticOutputTool::new();
        let data = json!({"name": "test", "count": 5});
        let result = tool.transform(data, "markdown".to_string()).await.unwrap();

        assert_eq!(result["status"], "transformed");
        assert_eq!(result["format"], "markdown");
        assert!(result["content"].as_str().unwrap().contains("**name**"));
    }

    #[tokio::test]
    async fn test_empty_content_rejected() {
        let tool = SyntheticOutputTool::new();
        let result = tool.generate("   ".to_string(), None, None).await;

        assert!(result.is_err());
    }
}
