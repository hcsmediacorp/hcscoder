//! hcscoder Task Management Tool
//!
//! Task creation, listing, updating, and tracking.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Task status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Task representation
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub parent_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Task {
    pub fn new(id: String, title: String) -> Self {
        let now = chrono::Utc::now();
        Task {
            id,
            title,
            description: None,
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            created_at: now,
            updated_at: now,
            completed_at: None,
            parent_id: None,
            metadata: HashMap::new(),
        }
    }
}

/// Task manager with thread-safe storage
#[derive(Clone, Default)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new task
    pub async fn create_task(&self, title: String, description: Option<String>) -> Result<Task> {
        let id = format!("task_{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let mut task = Task::new(id.clone(), title);
        task.description = description;

        let mut tasks = self.tasks.write().await;
        tasks.insert(id.clone(), task.clone());

        Ok(task)
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &str) -> Result<Option<Task>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(task_id).cloned())
    }

    /// List all tasks
    pub async fn list_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    /// List tasks by status
    pub async fn list_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;
        Ok(tasks
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect())
    }

    /// Update task status
    pub async fn update_status(&self, task_id: &str, status: TaskStatus) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .context(format!("Task not found: {}", task_id))?;

        task.status = status;
        task.updated_at = chrono::Utc::now();

        if status == TaskStatus::Completed
            || status == TaskStatus::Failed
            || status == TaskStatus::Cancelled
        {
            task.completed_at = Some(task.updated_at);
        }

        Ok(task.clone())
    }

    /// Update task details
    pub async fn update_task(
        &self,
        task_id: &str,
        title: Option<String>,
        description: Option<String>,
        priority: Option<TaskPriority>,
    ) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .context(format!("Task not found: {}", task_id))?;

        if let Some(t) = title {
            task.title = t;
        }
        if let Some(d) = description {
            task.description = Some(d);
        }
        if let Some(p) = priority {
            task.priority = p;
        }
        task.updated_at = chrono::Utc::now();

        Ok(task.clone())
    }

    /// Delete a task
    pub async fn delete_task(&self, task_id: &str) -> Result<Option<Task>> {
        let mut tasks = self.tasks.write().await;
        Ok(tasks.remove(task_id))
    }

    /// Stop/cancel a task
    pub async fn stop_task(&self, task_id: &str) -> Result<Task> {
        self.update_status(task_id, TaskStatus::Cancelled).await
    }

    /// Get task output/result
    pub async fn get_output(&self, task_id: &str) -> Result<Option<String>> {
        let tasks = self.tasks.read().await;
        let task = tasks.get(task_id);

        task.and_then(|t| t.metadata.get("output").cloned())
            .map_or(Ok(None), |o| Ok(Some(o)))
    }

    /// Set task output
    pub async fn set_output(&self, task_id: &str, output: String) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .context(format!("Task not found: {}", task_id))?;

        task.metadata.insert("output".to_string(), output);
        task.updated_at = chrono::Utc::now();

        Ok(())
    }
}

static GLOBAL_TASK_MANAGER: std::sync::OnceLock<TaskManager> = std::sync::OnceLock::new();

fn global_tasks() -> &'static TaskManager {
    GLOBAL_TASK_MANAGER.get_or_init(TaskManager::new)
}

/// Create a task (convenience function)
pub async fn create_task(title: String, description: Option<String>) -> Result<Task> {
    global_tasks().create_task(title, description).await
}

/// List all tasks (convenience function)
pub async fn list_tasks() -> Result<Vec<Task>> {
    global_tasks().list_tasks().await
}

/// Update task status (convenience function)
pub async fn update_task_status(task_id: &str, status: TaskStatus) -> Result<Task> {
    global_tasks().update_status(task_id, status).await
}

/// Stop a task (convenience function)
pub async fn stop_task(task_id: &str) -> Result<Task> {
    global_tasks().stop_task(task_id).await
}

/// Get task output (convenience function)
pub async fn get_task_output(task_id: &str) -> Result<Option<String>> {
    global_tasks().get_output(task_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_list_tasks() {
        let manager = TaskManager::new();

        let _task1 = manager
            .create_task("Test Task 1".to_string(), None)
            .await
            .unwrap();
        let _task2 = manager
            .create_task("Test Task 2".to_string(), Some("Description".to_string()))
            .await
            .unwrap();

        let tasks = manager.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 2);

        let pending = manager
            .list_tasks_by_status(TaskStatus::Pending)
            .await
            .unwrap();
        assert_eq!(pending.len(), 2);

        // Cleanup would happen automatically as manager is local
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let manager = TaskManager::new();
        let task = manager.create_task("Test".to_string(), None).await.unwrap();

        let updated = manager
            .update_status(&task.id, TaskStatus::InProgress)
            .await
            .unwrap();
        assert_eq!(updated.status, TaskStatus::InProgress);

        let completed = manager
            .update_status(&task.id, TaskStatus::Completed)
            .await
            .unwrap();
        assert_eq!(completed.status, TaskStatus::Completed);
        assert!(completed.completed_at.is_some());
    }
}
