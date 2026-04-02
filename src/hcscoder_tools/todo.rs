//! hcscoder Todo Write Tool
//!
//! Task list management with todo items.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Todo item status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

impl std::fmt::Display for TodoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoStatus::Pending => write!(f, "pending"),
            TodoStatus::InProgress => write!(f, "in_progress"),
            TodoStatus::Completed => write!(f, "completed"),
        }
    }
}

/// Todo item representation
#[derive(Debug, Clone)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: TodoStatus,
    pub priority: u8,
    pub parent_id: Option<String>,
}

impl TodoItem {
    pub fn new(id: String, content: String) -> Self {
        TodoItem {
            id,
            content,
            status: TodoStatus::Pending,
            priority: 1,
            parent_id: None,
        }
    }
}

/// Todo list manager
#[derive(Clone, Default)]
pub struct TodoManager {
    items: Arc<RwLock<HashMap<String, TodoItem>>>,
}

impl TodoManager {
    pub fn new() -> Self {
        TodoManager {
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Write/update todo list
    pub async fn write_todos(&self, todos: Vec<(String, String)>) -> Result<Vec<TodoItem>> {
        let mut items = self.items.write().await;
        let mut result = Vec::new();

        for (id, content) in todos {
            let item = if let Some(existing) = items.get_mut(&id) {
                existing.content = content;
                existing.clone()
            } else {
                let item = TodoItem::new(id.clone(), content);
                items.insert(id, item.clone());
                item
            };
            result.push(item);
        }

        Ok(result)
    }

    /// Get all todos
    pub async fn get_todos(&self) -> Result<Vec<TodoItem>> {
        let items = self.items.read().await;
        Ok(items.values().cloned().collect())
    }

    /// Update a single todo status
    pub async fn update_todo_status(&self, id: &str, status: TodoStatus) -> Result<TodoItem> {
        let mut items = self.items.write().await;
        let item = items
            .get_mut(id)
            .context(format!("Todo not found: {}", id))?;

        item.status = status;
        Ok(item.clone())
    }

    /// Delete a todo
    pub async fn delete_todo(&self, id: &str) -> Result<Option<TodoItem>> {
        let mut items = self.items.write().await;
        Ok(items.remove(id))
    }

    /// Clear all completed todos
    pub async fn clear_completed(&self) -> Result<usize> {
        let mut items = self.items.write().await;
        let before = items.len();
        items.retain(|_, item| item.status != TodoStatus::Completed);
        Ok(before - items.len())
    }
}

static GLOBAL_TODO_MANAGER: std::sync::OnceLock<TodoManager> = std::sync::OnceLock::new();

fn global_todos() -> &'static TodoManager {
    GLOBAL_TODO_MANAGER.get_or_init(TodoManager::new)
}

/// Write todos (convenience function)
pub async fn write_todos(todos: Vec<(String, String)>) -> Result<Vec<TodoItem>> {
    global_todos().write_todos(todos).await
}

/// Get todos (convenience function)
pub async fn get_todos() -> Result<Vec<TodoItem>> {
    global_todos().get_todos().await
}

/// Update todo status (convenience function)
pub async fn update_todo_status(id: &str, status: TodoStatus) -> Result<TodoItem> {
    global_todos().update_todo_status(id, status).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_and_get_todos() {
        let manager = TodoManager::new();

        let todos = vec![
            ("todo_1".to_string(), "First task".to_string()),
            ("todo_2".to_string(), "Second task".to_string()),
        ];

        let result = manager.write_todos(todos).await.unwrap();
        assert_eq!(result.len(), 2);

        let all = manager.get_todos().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_update_todo_status() {
        let manager = TodoManager::new();

        manager
            .write_todos(vec![("todo_1".to_string(), "Test".to_string())])
            .await
            .unwrap();

        let updated = manager
            .update_todo_status("todo_1", TodoStatus::InProgress)
            .await
            .unwrap();
        assert_eq!(updated.status, TodoStatus::InProgress);
    }
}
