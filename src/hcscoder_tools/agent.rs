//! AgentTool - Manage AI agent sessions
//!
//! Provides functionality for creating, managing, and interacting with AI agent sessions.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Agent session state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Running,
    Paused,
    Completed,
    Error,
}

/// Agent session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub name: String,
    pub state: AgentState,
    pub created_at: i64,
    pub updated_at: i64,
    pub context: HashMap<String, serde_json::Value>,
}

/// AgentTool for managing AI agent sessions
pub struct AgentTool {
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
}

impl AgentTool {
    /// Create a new AgentTool instance
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new agent session
    pub async fn create_session(&self, name: String) -> Result<String> {
        use chrono::Utc;
        use uuid::Uuid;

        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        let session = AgentSession {
            id: session_id.clone(),
            name,
            state: AgentState::Idle,
            created_at: now,
            updated_at: now,
            context: HashMap::new(),
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Get agent session info
    pub async fn get_session(&self, session_id: &str) -> Result<Option<AgentSession>> {
        let sessions = self.sessions.lock().await;
        Ok(sessions.get(session_id).cloned())
    }

    /// List all agent sessions
    pub async fn list_sessions(&self) -> Result<Vec<AgentSession>> {
        let sessions = self.sessions.lock().await;
        Ok(sessions.values().cloned().collect())
    }

    /// Update agent state
    pub async fn update_state(&self, session_id: &str, state: AgentState) -> Result<()> {
        use chrono::Utc;

        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.state = state;
            session.updated_at = Utc::now().timestamp();
            Ok(())
        } else {
            anyhow::bail!("Session not found: {}", session_id)
        }
    }

    /// Add context to agent session
    pub async fn add_context(
        &self,
        session_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> Result<()> {
        use chrono::Utc;

        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.context.insert(key, value);
            session.updated_at = Utc::now().timestamp();
            Ok(())
        } else {
            anyhow::bail!("Session not found: {}", session_id)
        }
    }

    /// Delete an agent session
    pub async fn delete_session(&self, session_id: &str) -> Result<bool> {
        let mut sessions = self.sessions.lock().await;
        Ok(sessions.remove(session_id).is_some())
    }

    /// Execute agent tool and return JSON result
    pub async fn execute(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match action {
            "create" => {
                let name = params["name"]
                    .as_str()
                    .context("Missing 'name' parameter")?
                    .to_string();
                let session_id = self.create_session(name).await?;
                Ok(json!({"session_id": session_id, "status": "created"}))
            }
            "get" => {
                let session_id = params["session_id"]
                    .as_str()
                    .context("Missing 'session_id' parameter")?;
                match self.get_session(session_id).await? {
                    Some(session) => Ok(json!(session)),
                    None => Ok(json!({"error": "Session not found"})),
                }
            }
            "list" => {
                let sessions = self.list_sessions().await?;
                Ok(json!({"sessions": sessions}))
            }
            "update_state" => {
                let session_id = params["session_id"]
                    .as_str()
                    .context("Missing 'session_id' parameter")?;
                let state_str = params["state"]
                    .as_str()
                    .context("Missing 'state' parameter")?;
                let state = match state_str {
                    "idle" => AgentState::Idle,
                    "running" => AgentState::Running,
                    "paused" => AgentState::Paused,
                    "completed" => AgentState::Completed,
                    "error" => AgentState::Error,
                    _ => anyhow::bail!("Invalid state: {}", state_str),
                };
                self.update_state(session_id, state).await?;
                Ok(json!({"status": "updated", "session_id": session_id}))
            }
            "delete" => {
                let session_id = params["session_id"]
                    .as_str()
                    .context("Missing 'session_id' parameter")?;
                let deleted = self.delete_session(session_id).await?;
                Ok(json!({"deleted": deleted, "session_id": session_id}))
            }
            _ => anyhow::bail!("Unknown action: {}", action),
        }
    }
}

impl Default for AgentTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let tool = AgentTool::new();
        let session_id = tool.create_session("test".to_string()).await.unwrap();
        assert!(!session_id.is_empty());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let tool = AgentTool::new();
        tool.create_session("test1".to_string()).await.unwrap();
        tool.create_session("test2".to_string()).await.unwrap();
        let sessions = tool.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);
    }
}
