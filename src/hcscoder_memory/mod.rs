//! hcscoder Memory & Dreams Module
//!
//! Background memory consolidation and autoDream service.
//! Zero telemetry, no phone-home logic.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;

const MEMORY_MARKDOWN_FILE: &str = "MEMORY.md";
const MEMORY_JSON_FILE: &str = "memory.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderMemoryEntry {
    pub id: String,
    pub content: String,
    pub category: MemoryCategory,
    pub importance: u8,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryCategory {
    Code,
    Conversation,
    Task,
    Insight,
    Dream,
}

pub struct HcscoderMemoryManager {
    entries: Vec<HcscoderMemoryEntry>,
    storage_path: PathBuf,
    json_storage_path: PathBuf,
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
    pub async fn new() -> Result<Self> {
        Self::new_with_override(None).await
    }

    async fn new_with_override(base_dir: Option<PathBuf>) -> Result<Self> {
        let (storage_path, json_storage_path) = Self::get_storage_paths(base_dir)?;
        let (tx, mut rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
            loop {
                tokio::select! {
                    _ = interval.tick() => tracing::debug!("Periodic memory maintenance tick"),
                    Some(_event) = rx.recv() => {}
                }
            }
        });

        let mut manager = Self {
            entries: Vec::new(),
            storage_path,
            json_storage_path,
            tx,
        };
        manager.load().await?;
        Ok(manager)
    }

    fn get_storage_paths(base_dir: Option<PathBuf>) -> Result<(PathBuf, PathBuf)> {
        let config_dir = if let Some(path) = base_dir {
            path
        } else if let Ok(path) = std::env::var("HCSCODER_MEMORY_DIR") {
            PathBuf::from(path)
        } else {
            dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?
                .join(".hcscoder")
                .join("memory")
        };
        std::fs::create_dir_all(&config_dir)?;
        Ok((
            config_dir.join(MEMORY_MARKDOWN_FILE),
            config_dir.join(MEMORY_JSON_FILE),
        ))
    }

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

    pub async fn save(&self) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let mut content = String::from("# hcscoder Memory\n\n");
        content.push_str(&format!("*Last updated: {}*\n\n", Utc::now()));

        for category in [
            MemoryCategory::Code,
            MemoryCategory::Conversation,
            MemoryCategory::Task,
            MemoryCategory::Insight,
            MemoryCategory::Dream,
        ] {
            let entries: Vec<_> = self
                .entries
                .iter()
                .filter(|e| e.category == category)
                .collect();
            if entries.is_empty() {
                continue;
            }

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

        let mut file = tokio::fs::File::create(&self.storage_path).await?;
        file.write_all(content.as_bytes()).await?;

        let json = serde_json::to_string_pretty(&self.entries)?;
        tokio::fs::write(&self.json_storage_path, json).await?;
        Ok(())
    }

    pub async fn load(&mut self) -> Result<()> {
        if !self.json_storage_path.exists() {
            return Ok(());
        }
        let content = tokio::fs::read_to_string(&self.json_storage_path).await?;
        self.entries = serde_json::from_str(&content)?;
        Ok(())
    }

    pub async fn run_consolidation_cycle(&mut self) -> Result<()> {
        self.consolidate_entries();
        self.generate_dream_entries();
        self.tx.send(MemoryEvent::Consolidate).await.ok();
        self.save().await
    }

    fn consolidate_entries(&mut self) {
        let now = Utc::now();
        let mut dedup: HashMap<(MemoryCategory, String), HcscoderMemoryEntry> = HashMap::new();

        for entry in self.entries.drain(..) {
            let key = (entry.category, entry.content.trim().to_lowercase());
            match dedup.get_mut(&key) {
                Some(existing) => {
                    existing.importance = existing.importance.max(entry.importance);
                    existing.last_accessed = existing.last_accessed.max(entry.last_accessed);
                }
                None => {
                    dedup.insert(key, entry);
                }
            }
        }

        self.entries = dedup
            .into_values()
            .filter(|e| !(e.importance <= 2 && now - e.last_accessed > Duration::days(30)))
            .collect();

        self.entries.sort_by(|a, b| {
            b.importance
                .cmp(&a.importance)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
    }

    fn generate_dream_entries(&mut self) {
        let mut repeated = HashMap::<String, (u8, usize)>::new();
        for entry in self
            .entries
            .iter()
            .filter(|e| matches!(e.category, MemoryCategory::Insight | MemoryCategory::Task))
        {
            let normalized = entry.content.trim().to_lowercase();
            let bucket = repeated.entry(normalized).or_insert((0, 0));
            bucket.0 = bucket.0.max(entry.importance);
            bucket.1 += 1;
        }

        let mut synth = repeated
            .into_iter()
            .filter(|(_, (_, count))| *count >= 2)
            .collect::<Vec<_>>();
        synth.sort_by(|a, b| a.0.cmp(&b.0));

        for (content, (importance, count)) in synth {
            let dream_content = format!("Pattern noticed {} times: {}", count, content);
            if self
                .entries
                .iter()
                .any(|e| e.category == MemoryCategory::Dream && e.content == dream_content)
            {
                continue;
            }

            let now = Utc::now();
            self.entries.push(HcscoderMemoryEntry {
                id: format!("dream-{}", uuid::Uuid::new_v4()),
                content: dream_content,
                category: MemoryCategory::Dream,
                importance: importance.saturating_add(1).min(10),
                created_at: now,
                last_accessed: now,
            });
        }
    }

    pub fn view_memories(&self) {
        println!("\n🧠 hcscoder Memory");
        println!("{}", "=".repeat(50));
        if self.entries.is_empty() {
            println!("No memories yet.");
            return;
        }
        for entry in &self.entries {
            println!("[{:?}] {}", entry.category, entry.content);
        }
        println!("{}", "=".repeat(50));
        println!("Total: {} memories", self.entries.len());
    }

    pub async fn clear_memories(&mut self) -> Result<()> {
        self.entries.clear();
        self.save().await
    }

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
        Ok(())
    }

    #[cfg(test)]
    async fn new_for_tests(base_dir: PathBuf) -> Result<Self> {
        Self::new_with_override(Some(base_dir)).await
    }

    #[cfg(test)]
    fn entries(&self) -> &[HcscoderMemoryEntry] {
        &self.entries
    }
}

pub fn view_memory() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let manager = rt.block_on(HcscoderMemoryManager::new())?;
    manager.view_memories();
    Ok(())
}

pub fn clear_memory() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let mut manager = rt.block_on(HcscoderMemoryManager::new())?;
    rt.block_on(manager.clear_memories())
}

pub fn export_memory(path: &str) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let manager = rt.block_on(HcscoderMemoryManager::new())?;
    rt.block_on(manager.export_memories(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn add_save_load_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = HcscoderMemoryManager::new_for_tests(temp.path().to_path_buf())
            .await
            .unwrap();
        manager
            .add_memory("Remember this".to_string(), MemoryCategory::Insight, 6)
            .await
            .unwrap();

        let manager2 = HcscoderMemoryManager::new_for_tests(temp.path().to_path_buf())
            .await
            .unwrap();
        assert_eq!(manager2.entries().len(), 1);
        assert_eq!(manager2.entries()[0].content, "Remember this");
    }

    #[tokio::test]
    async fn clear_operation_works() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = HcscoderMemoryManager::new_for_tests(temp.path().to_path_buf())
            .await
            .unwrap();
        manager
            .add_memory("temp".to_string(), MemoryCategory::Task, 4)
            .await
            .unwrap();
        manager.clear_memories().await.unwrap();

        let manager2 = HcscoderMemoryManager::new_for_tests(temp.path().to_path_buf())
            .await
            .unwrap();
        assert!(manager2.entries().is_empty());
    }

    #[tokio::test]
    async fn export_operation_writes_file() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = HcscoderMemoryManager::new_for_tests(temp.path().to_path_buf())
            .await
            .unwrap();
        manager
            .add_memory("export me".to_string(), MemoryCategory::Conversation, 5)
            .await
            .unwrap();

        let export_path = temp.path().join("export.md");
        manager
            .export_memories(export_path.to_str().unwrap())
            .await
            .unwrap();

        let exported = tokio::fs::read_to_string(export_path).await.unwrap();
        assert!(exported.contains("export me"));
    }

    #[tokio::test]
    async fn consolidation_dedupes_prunes_and_dreams_deterministically() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = HcscoderMemoryManager::new_for_tests(temp.path().to_path_buf())
            .await
            .unwrap();
        let old = Utc::now() - Duration::days(40);
        manager.entries = vec![
            HcscoderMemoryEntry {
                id: "1".into(),
                content: "Refactor parser".into(),
                category: MemoryCategory::Insight,
                importance: 7,
                created_at: old,
                last_accessed: Utc::now(),
            },
            HcscoderMemoryEntry {
                id: "2".into(),
                content: "refactor parser".into(),
                category: MemoryCategory::Insight,
                importance: 8,
                created_at: old,
                last_accessed: Utc::now(),
            },
            HcscoderMemoryEntry {
                id: "3".into(),
                content: "stale low signal".into(),
                category: MemoryCategory::Task,
                importance: 1,
                created_at: old,
                last_accessed: old,
            },
            HcscoderMemoryEntry {
                id: "4".into(),
                content: "Refactor parser".into(),
                category: MemoryCategory::Task,
                importance: 6,
                created_at: old,
                last_accessed: Utc::now(),
            },
        ];

        manager.run_consolidation_cycle().await.unwrap();
        assert!(manager
            .entries()
            .iter()
            .all(|e| e.content != "stale low signal"));

        let insight_count = manager
            .entries()
            .iter()
            .filter(|e| e.category == MemoryCategory::Insight)
            .count();
        assert_eq!(insight_count, 1);

        assert!(manager.entries().iter().any(|e| {
            e.category == MemoryCategory::Dream && e.content.starts_with("Pattern noticed")
        }));
    }
}
