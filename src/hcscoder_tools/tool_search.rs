//! ToolSearchTool - Search and fetch tool schemas
//!
//! Provides functionality to search for tools and fetch their complete schema definitions.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// Tool schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// ToolSearchTool for finding and fetching tool schemas
pub struct ToolSearchTool {
    /// Registry of available tools (in real impl, this would be populated dynamically)
    tool_registry: HashMap<String, ToolSchema>,
}

impl ToolSearchTool {
    /// Create a new ToolSearchTool instance
    pub fn new() -> Self {
        let mut registry = HashMap::new();

        // Register built-in tools with their schemas
        registry.insert(
            "read_file".to_string(),
            ToolSchema {
                name: "read_file".to_string(),
                description: "Read contents of a file".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Path to the file"}
                    },
                    "required": ["path"]
                }),
            },
        );

        registry.insert(
            "write_file".to_string(),
            ToolSchema {
                name: "write_file".to_string(),
                description: "Write content to a file".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Path to the file"},
                        "content": {"type": "string", "description": "Content to write"}
                    },
                    "required": ["path", "content"]
                }),
            },
        );

        registry.insert(
            "bash".to_string(),
            ToolSchema {
                name: "bash".to_string(),
                description: "Execute a shell command".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "command": {"type": "string", "description": "Command to execute"},
                        "timeout_ms": {"type": "integer", "description": "Timeout in milliseconds"}
                    },
                    "required": ["command"]
                }),
            },
        );

        registry.insert(
            "grep".to_string(),
            ToolSchema {
                name: "grep".to_string(),
                description: "Search for patterns in files".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string", "description": "Pattern to search for"},
                        "path": {"type": "string", "description": "Directory or file to search"}
                    },
                    "required": ["pattern"]
                }),
            },
        );

        registry.insert(
            "glob".to_string(),
            ToolSchema {
                name: "glob".to_string(),
                description: "Find files matching a glob pattern".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string", "description": "Glob pattern to match"}
                    },
                    "required": ["pattern"]
                }),
            },
        );

        Self {
            tool_registry: registry,
        }
    }

    /// Search for tools by query
    pub async fn search(&self, query: &str, max_results: usize) -> Result<Vec<ToolSchema>> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(&String, &ToolSchema, u32)> = Vec::new();

        for (name, schema) in &self.tool_registry {
            let mut score = 0u32;

            // Check if query matches name
            if name.to_lowercase().contains(&query_lower) {
                score += 10;
            }

            // Check if query matches description
            if schema.description.to_lowercase().contains(&query_lower) {
                score += 5;
            }

            // Handle select: syntax — keywords must appear in tool name (e.g. select:Read,Write → read_file, write_file)
            if let Some(rest) = query.strip_prefix("select:") {
                let targets: Vec<String> = rest
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                let n = name.to_lowercase();
                if targets.iter().any(|t| n.contains(t)) {
                    score = 100;
                }
            }

            // Handle +prefix syntax for name requirements
            if let Some(rest) = query.strip_prefix('+') {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if !parts.is_empty() && name.to_lowercase().contains(&parts[0].to_lowercase()) {
                    score += 20;
                }
            }

            if score > 0 {
                results.push((name, schema, score));
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.2.cmp(&a.2));

        // Return top results
        Ok(results
            .into_iter()
            .take(max_results)
            .map(|(_, schema, _)| schema.clone())
            .collect())
    }

    /// Get a specific tool by name
    pub async fn get_tool(&self, name: &str) -> Result<Option<ToolSchema>> {
        Ok(self.tool_registry.get(name).cloned())
    }

    /// List all available tools
    pub async fn list_all(&self) -> Result<Vec<ToolSchema>> {
        Ok(self.tool_registry.values().cloned().collect())
    }

    /// Execute tool search and return JSON result
    pub async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let action = params["action"].as_str().unwrap_or("search");
        let max_results = params["max_results"].as_u64().unwrap_or(10) as usize;

        match action {
            "search" => {
                let query = params["query"]
                    .as_str()
                    .context("Missing 'query' parameter")?;
                let tools = self.search(query, max_results).await?;
                Ok(json!({
                    "action": "search",
                    "query": query,
                    "count": tools.len(),
                    "tools": tools,
                }))
            }
            "get" => {
                let name = params["name"]
                    .as_str()
                    .context("Missing 'name' parameter")?;
                match self.get_tool(name).await? {
                    Some(tool) => Ok(json!({
                        "action": "get",
                        "tool": tool,
                    })),
                    None => Ok(json!({
                        "action": "get",
                        "error": format!("Tool not found: {}", name),
                    })),
                }
            }
            "list" => {
                let tools = self.list_all().await?;
                Ok(json!({
                    "action": "list",
                    "count": tools.len(),
                    "tools": tools,
                }))
            }
            _ => anyhow::bail!("Unknown action: {}", action),
        }
    }
}

impl Default for ToolSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_by_name() {
        let tool = ToolSearchTool::new();
        let results = tool.search("read", 10).await.unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|t| t.name == "read_file"));
    }

    #[tokio::test]
    async fn test_select_syntax() {
        let tool = ToolSearchTool::new();
        let results = tool.search("select:Read,Write", 10).await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|t| t.name == "read_file"));
        assert!(results.iter().any(|t| t.name == "write_file"));
    }

    #[tokio::test]
    async fn test_get_specific_tool() {
        let tool = ToolSearchTool::new();
        let result = tool.get_tool("bash").await.unwrap().unwrap();

        assert_eq!(result.name, "bash");
        assert!(result.description.contains("shell"));
    }

    #[tokio::test]
    async fn test_list_all_tools() {
        let tool = ToolSearchTool::new();
        let tools = tool.list_all().await.unwrap();

        assert!(tools.len() >= 5);
    }
}
