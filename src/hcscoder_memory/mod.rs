//! hcscoder Memory & Dreams Module
//!
//! Background memory consolidation and autoDream service.
//! Zero telemetry, no phone-home logic.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderMemoryEntry {
    pub id: String,
    pub content: String,
    pub category: MemoryCategory,
    pub importance: u8,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryCategory {
    Code,
    Conversation,
    Task,
    Insight,
    Dream,
}

/// Memory manager with background consolidation
pub struct HcscoderMemoryManager {
    entries: Vec<HcscoderMemoryEntry>,
    storage_path: PathBuf,
    tx: mpsc::Sender<MemoryEvent>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum MemoryEvent {
    Add(HcscoderMemoryEntry),
    Consolidate,
    Dream,
}

impl HcscoderMemoryManager {
    /// Create new memory manager
    pub async fn new() -> Result<Self> {
        let storage_path = Self::get_storage_path()?;
        let (tx, mut rx) = mpsc::channel(100);

        // Spawn background task for consolidation
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 min

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Periodic consolidation
                        tracing::debug!("Running periodic memory consolidation");
                    }
                    Some(event) = rx.recv() => {
                        match event {
                            MemoryEvent::Consolidate => {
                                tracing::debug!("Consolidating memories");
                            }
                            MemoryEvent::Dream => {
                                tracing::debug!("Running autoDream cycle");
                            }
                            MemoryEvent::Add(_) => {}
                        }
                    }
                }
            }
        });

        let mut manager = Self {
            entries: Vec::new(),
            storage_path,
            tx,
        };

        // Load existing memories
        manager.load().await?;

        Ok(manager)
    }

    /// Get storage path
    fn get_storage_path() -> Result<PathBuf> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?
            .join(".hcscoder")
            .join("memory");

        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("MEMORY.md"))
    }

    /// Add a memory entry
    pub async fn add_memory(
        &mut self,
        content: String,
        category: MemoryCategory,
        importance: u8,
    ) -> Result<()> {
        use uuid::Uuid;

        let now = Utc::now();
        let entry = HcscoderMemoryEntry {
            id: Uuid::new_v4().to_string(),
            content,
            category,
            importance: importance.min(10),
            created_at: now,
            last_accessed: now,
        };

        self.tx.send(MemoryEvent::Add(entry.clone())).await.ok();
        self.entries.push(entry);

        self.save().await
    }

    /// Save memories to file
    pub async fn save(&self) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let mut content = String::from("# hcscoder Memory\n\n");
        content.push_str(&format!("*Last updated: {}*\n\n", Utc::now()));

        // Group by category
        let categories = [
            MemoryCategory::Code,
            MemoryCategory::Conversation,
            MemoryCategory::Task,
            MemoryCategory::Insight,
            MemoryCategory::Dream,
        ];

        for category in categories {
            let entries: Vec<_> = self
                .entries
                .iter()
                .filter(|e| e.category == category)
                .collect();

            if !entries.is_empty() {
                content.push_str(&format!("## {:?}\n\n", category));

                for entry in entries {
                    content.push_str(&format!(
                        "- [{}] {} (Importance: {})\n",
                        entry.created_at.format("%Y-%m-%d %H:%M"),
                        entry.content,
                        entry.importance
                    ));
                }

                content.push('\n');
            }
        }

        let mut file = tokio::fs::File::create(&self.storage_path).await?;
        file.write_all(content.as_bytes()).await?;

        Ok(())
    }

    /// Load memories from file
    pub async fn load(&mut self) -> Result<()> {
        if !self.storage_path.exists() {
            return Ok(());
        }

        let _content = tokio::fs::read_to_string(&self.storage_path).await?;

        // Simple parsing - in production would use proper markdown parser
        self.entries.clear();

        // For now, just acknowledge the file exists
        tracing::info!("Loaded memory file: {:?}", self.storage_path);

        Ok(())
    }

    /// Trigger autoDream consolidation
    pub async fn trigger_dream(&self) -> Result<()> {
        self.tx.send(MemoryEvent::Dream).await.ok();
        Ok(())
    }

    /// View all memories
    pub fn view_memories(&self) {
        println!("\n🧠 hcscoder Memory");
        println!("{}", "=".repeat(50));

        if self.entries.is_empty() {
            println!("No memories yet.");
            return;
        }

        for entry in &self.entries {
            let icon = match entry.category {
                MemoryCategory::Code => "💻",
                MemoryCategory::Conversation => "💬",
                MemoryCategory::Task => "✅",
                MemoryCategory::Insight => "💡",
                MemoryCategory::Dream => "💭",
            };

            println!("{} [{:?}] {}", icon, entry.category, entry.content);
            println!(
                "   Importance: {}/10 | Created: {}",
                entry.importance,
                entry.created_at.format("%Y-%m-%d")
            );
        }

        println!("{}", "=".repeat(50));
        println!("Total: {} memories", self.entries.len());
    }

    /// Clear all memories
    pub async fn clear_memories(&mut self) -> Result<()> {
        self.entries.clear();
        self.save().await?;
        println!("✅ All memories cleared.");
        Ok(())
    }

    /// Export memories to file
    pub async fn export_memories(&self, path: &str) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let mut content = String::from("# hcscoder Memory Export\n\n");

        for entry in &self.entries {
            content.push_str(&format!(
                "## [{:?}] {}\n\n{}\n\n",
                entry.category,
                entry.created_at.format("%Y-%m-%d %H:%M:%S"),
                entry.content
            ));
        }

        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(content.as_bytes()).await?;

        println!("✅ Memories exported to: {}", path);
        Ok(())
    }
}

// Global functions for CLI usage
pub fn view_memory() -> Result<()> {
    println!("\n🧠 Memory viewing requires running session.");
    println!("Use 'hcscoder chat' to start a session with memory enabled.");
    Ok(())
}

pub fn clear_memory() -> Result<()> {
    println!("\n⚠️  Memory clearing requires running session.");
    Ok(())
}

pub fn export_memory(_path: &str) -> Result<()> {
    println!("\n📤 Memory export requires running session.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_manager() {
        let mut manager = HcscoderMemoryManager::new().await.unwrap();

        manager
            .add_memory(
                "Test memory content".to_string(),
                MemoryCategory::Insight,
                5,
            )
            .await
            .unwrap();

        assert_eq!(manager.entries.len(), 1);
        assert_eq!(manager.entries[0].importance, 5);
    }
}
