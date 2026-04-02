//! hcscoder Team Management Tool
//!
//! Team creation, deletion, and management for multi-agent collaboration.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Team representation
#[derive(Debug, Clone)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub agent_ids: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Team {
    pub fn new(id: String, name: String) -> Self {
        Team {
            id,
            name,
            description: None,
            agent_ids: Vec::new(),
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

/// Team manager with thread-safe storage
#[derive(Clone, Default)]
pub struct TeamManager {
    teams: Arc<RwLock<HashMap<String, Team>>>,
}

impl TeamManager {
    pub fn new() -> Self {
        TeamManager {
            teams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new team
    pub async fn create_team(&self, name: String, description: Option<String>) -> Result<Team> {
        let id = format!("team_{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let mut team = Team::new(id.clone(), name);
        team.description = description;

        let mut teams = self.teams.write().await;
        teams.insert(id.clone(), team.clone());

        Ok(team)
    }

    /// Get a team by ID
    pub async fn get_team(&self, team_id: &str) -> Result<Option<Team>> {
        let teams = self.teams.read().await;
        Ok(teams.get(team_id).cloned())
    }

    /// List all teams
    pub async fn list_teams(&self) -> Result<Vec<Team>> {
        let teams = self.teams.read().await;
        Ok(teams.values().cloned().collect())
    }

    /// Add an agent to a team
    pub async fn add_agent(&self, team_id: &str, agent_id: String) -> Result<Team> {
        let mut teams = self.teams.write().await;
        let team = teams
            .get_mut(team_id)
            .context(format!("Team not found: {}", team_id))?;

        if !team.agent_ids.contains(&agent_id) {
            team.agent_ids.push(agent_id);
        }

        Ok(team.clone())
    }

    /// Remove an agent from a team
    pub async fn remove_agent(&self, team_id: &str, agent_id: &str) -> Result<Team> {
        let mut teams = self.teams.write().await;
        let team = teams
            .get_mut(team_id)
            .context(format!("Team not found: {}", team_id))?;

        team.agent_ids.retain(|id| id != agent_id);

        Ok(team.clone())
    }

    /// Delete a team
    pub async fn delete_team(&self, team_id: &str) -> Result<Option<Team>> {
        let mut teams = self.teams.write().await;
        Ok(teams.remove(team_id))
    }
}

static GLOBAL_TEAM_MANAGER: std::sync::OnceLock<TeamManager> = std::sync::OnceLock::new();

fn global_teams() -> &'static TeamManager {
    GLOBAL_TEAM_MANAGER.get_or_init(TeamManager::new)
}

/// Create a team (convenience function)
pub async fn create_team(name: String, description: Option<String>) -> Result<Team> {
    global_teams().create_team(name, description).await
}

/// List all teams (convenience function)
pub async fn list_teams() -> Result<Vec<Team>> {
    global_teams().list_teams().await
}

/// Delete a team (convenience function)
pub async fn delete_team(team_id: &str) -> Result<Option<Team>> {
    global_teams().delete_team(team_id).await
}

/// Add agent to team (convenience function)
pub async fn add_agent_to_team(team_id: &str, agent_id: String) -> Result<Team> {
    global_teams().add_agent(team_id, agent_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_list_teams() {
        let manager = TeamManager::new();

        let _team1 = manager
            .create_team("Dev Team".to_string(), None)
            .await
            .unwrap();
        let _team2 = manager
            .create_team("QA Team".to_string(), Some("Quality Assurance".to_string()))
            .await
            .unwrap();

        let teams = manager.list_teams().await.unwrap();
        assert_eq!(teams.len(), 2);
    }

    #[tokio::test]
    async fn test_add_remove_agent() {
        let manager = TeamManager::new();
        let team = manager
            .create_team("Test Team".to_string(), None)
            .await
            .unwrap();

        let updated = manager
            .add_agent(&team.id, "agent_1".to_string())
            .await
            .unwrap();
        assert_eq!(updated.agent_ids.len(), 1);

        let removed = manager.remove_agent(&team.id, "agent_1").await.unwrap();
        assert_eq!(removed.agent_ids.len(), 0);
    }
}
