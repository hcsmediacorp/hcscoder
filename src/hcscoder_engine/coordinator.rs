//! hcscoder Multi-Agent Coordinator
//!
//! Handles swarm logic and multi-agent orchestration.
//! Zero telemetry, no phone-home logic.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent role in the swarm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HcscoderAgentRole {
    Planner,
    Coder,
    Reviewer,
    Tester,
    Optimizer,
}

impl std::fmt::Display for HcscoderAgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planner => write!(f, "Planner"),
            Self::Coder => write!(f, "Coder"),
            Self::Reviewer => write!(f, "Reviewer"),
            Self::Tester => write!(f, "Tester"),
            Self::Optimizer => write!(f, "Optimizer"),
        }
    }
}

/// Agent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderAgent {
    pub id: String,
    pub role: HcscoderAgentRole,
    pub model: String,
    pub status: AgentStatus,
    pub tasks_completed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Working,
    Waiting,
    Completed,
}

/// Task assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderTask {
    pub id: String,
    pub description: String,
    pub assigned_to: Option<String>,
    pub status: TaskStatus,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Swarm coordinator for multi-agent orchestration
pub struct HcscoderSwarmCoordinator {
    agents: HashMap<String, HcscoderAgent>,
    tasks: Vec<HcscoderTask>,
    default_model: String,
}

impl HcscoderSwarmCoordinator {
    /// Create a new swarm coordinator
    pub fn new(default_model: String) -> Self {
        Self {
            agents: HashMap::new(),
            tasks: Vec::new(),
            default_model,
        }
    }

    /// Initialize default agent swarm
    pub fn initialize_default_swarm(&mut self) {
        let roles = [
            HcscoderAgentRole::Planner,
            HcscoderAgentRole::Coder,
            HcscoderAgentRole::Reviewer,
            HcscoderAgentRole::Tester,
        ];

        for (i, role) in roles.iter().enumerate() {
            let agent = HcscoderAgent {
                id: format!("agent-{}", i + 1),
                role: *role,
                model: self.default_model.clone(),
                status: AgentStatus::Idle,
                tasks_completed: 0,
            };
            self.agents.insert(agent.id.clone(), agent);
        }

        tracing::info!("Initialized {} agents in swarm", self.agents.len());
    }

    /// Add a task to the swarm
    pub fn add_task(&mut self, description: String) -> &str {
        use uuid::Uuid;

        let idx = self.tasks.len();
        self.tasks.push(HcscoderTask {
            id: Uuid::new_v4().to_string(),
            description,
            assigned_to: None,
            status: TaskStatus::Pending,
            result: None,
        });
        self.tasks[idx].id.as_str()
    }

    /// Assign pending tasks to idle agents
    pub fn assign_tasks(&mut self) {
        let pending_tasks: Vec<usize> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status == TaskStatus::Pending && t.assigned_to.is_none())
            .map(|(i, _)| i)
            .collect();

        let idle_ids: Vec<String> = self
            .agents
            .iter()
            .filter(|(_, a)| a.status == AgentStatus::Idle)
            .map(|(id, _)| id.clone())
            .collect();

        for (task_idx, agent_id) in pending_tasks.into_iter().zip(idle_ids.into_iter()) {
            if let Some(task) = self.tasks.get_mut(task_idx) {
                task.assigned_to = Some(agent_id.clone());
                task.status = TaskStatus::InProgress;
            }

            if let Some(agent) = self.agents.get_mut(&agent_id) {
                agent.status = AgentStatus::Working;
            }
        }
    }

    /// Mark a task as completed
    pub fn complete_task(&mut self, task_id: &str, result: String) -> Result<()> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

        task.status = TaskStatus::Completed;
        task.result = Some(result);

        if let Some(ref agent_id) = task.assigned_to {
            if let Some(agent) = self.agents.get_mut(agent_id) {
                agent.status = AgentStatus::Idle;
                agent.tasks_completed += 1;
            }
        }

        Ok(())
    }

    /// Get swarm status summary
    pub fn get_status(&self) -> SwarmStatus {
        let total_agents = self.agents.len();
        let idle_agents = self
            .agents
            .values()
            .filter(|a| a.status == AgentStatus::Idle)
            .count();
        let working_agents = self
            .agents
            .values()
            .filter(|a| a.status == AgentStatus::Working)
            .count();

        let total_tasks = self.tasks.len();
        let pending_tasks = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count();
        let completed_tasks = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();

        SwarmStatus {
            total_agents,
            idle_agents,
            working_agents,
            total_tasks,
            pending_tasks,
            completed_tasks,
        }
    }

    /// Get agents by role
    pub fn get_agents_by_role(&self, role: HcscoderAgentRole) -> Vec<&HcscoderAgent> {
        self.agents.values().filter(|a| a.role == role).collect()
    }
}

/// Swarm status summary
#[derive(Debug, Clone)]
pub struct SwarmStatus {
    pub total_agents: usize,
    pub idle_agents: usize,
    pub working_agents: usize,
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub completed_tasks: usize,
}

impl std::fmt::Display for SwarmStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "🐝 Swarm Status")?;
        writeln!(
            f,
            "  Agents: {}/{} working",
            self.working_agents, self.total_agents
        )?;
        writeln!(
            f,
            "  Tasks: {}/{} completed",
            self.completed_tasks, self.total_tasks
        )?;
        writeln!(f, "  Pending: {}", self.pending_tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_initialization() {
        let mut coordinator = HcscoderSwarmCoordinator::new("claude-3.5-haiku".to_string());
        coordinator.initialize_default_swarm();

        assert_eq!(coordinator.agents.len(), 4);
        assert!(!coordinator
            .get_agents_by_role(HcscoderAgentRole::Planner)
            .is_empty());
    }

    #[test]
    fn test_task_assignment() {
        let mut coordinator = HcscoderSwarmCoordinator::new("claude-3.5-haiku".to_string());
        coordinator.initialize_default_swarm();

        coordinator.add_task("Test task".to_string());
        coordinator.assign_tasks();

        let status = coordinator.get_status();
        assert_eq!(status.pending_tasks, 0);
        assert!(status.working_agents >= 1);
    }
}
