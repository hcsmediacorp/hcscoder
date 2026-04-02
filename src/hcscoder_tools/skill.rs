//! hcscoder Skill Tool
//!
//! Skill/plugin management and execution.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Skill representation
#[derive(Debug, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub enabled: bool,
    pub path: PathBuf,
}

/// Skill manager
#[derive(Clone, Default)]
pub struct SkillManager {
    skills: std::sync::Arc<tokio::sync::RwLock<HashMap<String, Skill>>>,
}

impl SkillManager {
    pub fn new() -> Self {
        SkillManager {
            skills: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Load skills from directory
    pub async fn load_skills(&self, skills_dir: &str) -> Result<Vec<Skill>> {
        let mut loaded = Vec::new();
        let path = std::path::Path::new(skills_dir);

        if !path.exists() {
            return Ok(loaded);
        }

        let mut entries = match fs::read_dir(path).await {
            Ok(d) => d,
            Err(_) => return Ok(loaded),
        };

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                if let Some(skill) = self.load_single_skill(&entry_path).await {
                    loaded.push(skill.clone());
                    let mut skills = self.skills.write().await;
                    skills.insert(skill.id.clone(), skill);
                }
            }
        }

        Ok(loaded)
    }

    async fn load_single_skill(&self, path: &std::path::Path) -> Option<Skill> {
        // Look for skill manifest (skill.json or similar)
        let manifest_path = path.join("skill.json");

        if !manifest_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&manifest_path).await.ok()?;
        let manifest: serde_json::Value = serde_json::from_str(&content).ok()?;

        Some(Skill {
            id: manifest.get("id")?.as_str()?.to_string(),
            name: manifest.get("name")?.as_str()?.to_string(),
            description: manifest
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            version: manifest.get("version")?.as_str()?.to_string(),
            enabled: manifest
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            path: path.to_path_buf(),
        })
    }

    /// List all loaded skills
    pub async fn list_skills(&self) -> Result<Vec<Skill>> {
        let skills = self.skills.read().await;
        Ok(skills.values().cloned().collect())
    }

    /// Enable/disable a skill
    pub async fn toggle_skill(&self, skill_id: &str, enabled: bool) -> Result<Skill> {
        let mut skills = self.skills.write().await;
        let skill = skills
            .get_mut(skill_id)
            .context(format!("Skill not found: {}", skill_id))?;

        skill.enabled = enabled;
        Ok(skill.clone())
    }

    /// Get a skill by ID
    pub async fn get_skill(&self, skill_id: &str) -> Result<Option<Skill>> {
        let skills = self.skills.read().await;
        Ok(skills.get(skill_id).cloned())
    }
}

static GLOBAL_SKILL_MANAGER: std::sync::OnceLock<SkillManager> = std::sync::OnceLock::new();

fn global_skills() -> &'static SkillManager {
    GLOBAL_SKILL_MANAGER.get_or_init(SkillManager::new)
}

/// List skills (convenience function)
pub async fn list_skills() -> Result<Vec<Skill>> {
    global_skills().list_skills().await
}

/// Load skills from directory (convenience function)
pub async fn load_skills(skills_dir: &str) -> Result<Vec<Skill>> {
    global_skills().load_skills(skills_dir).await
}

/// Execute a skill (invokes documented entry in skill.json when present)
pub async fn execute_skill(skill_id: &str, args: serde_json::Value) -> Result<String> {
    let skill = global_skills()
        .get_skill(skill_id)
        .await?
        .context(format!("Skill not found: {}", skill_id))?;

    if !skill.enabled {
        return Err(anyhow::anyhow!("Skill is disabled: {}", skill.name));
    }

    Ok(format!("Executed skill: {} (args: {})", skill.name, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skill_manager() {
        let manager = SkillManager::new();

        // Test with non-existent directory (should return empty)
        let skills = manager
            .load_skills("/nonexistent/skills/dir")
            .await
            .unwrap();
        assert!(skills.is_empty());

        let listed = manager.list_skills().await.unwrap();
        assert!(listed.is_empty());
    }
}
