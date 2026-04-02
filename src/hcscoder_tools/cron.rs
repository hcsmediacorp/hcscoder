//! hcscoder Cron/Schedule Tool
//!
//! Scheduled task and cron job management.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cron job representation
#[derive(Debug, Clone)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub schedule: String, // Cron expression: "*/5 * * * *"
    pub command: String,
    pub enabled: bool,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run: Option<chrono::DateTime<chrono::Utc>>,
}

/// Cron manager
#[derive(Clone, Default)]
pub struct CronManager {
    jobs: Arc<RwLock<HashMap<String, CronJob>>>,
}

impl CronManager {
    pub fn new() -> Self {
        CronManager {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new cron job
    pub async fn create_cron(
        &self,
        name: String,
        schedule: String,
        command: String,
    ) -> Result<CronJob> {
        let id = format!("cron_{}", &uuid::Uuid::new_v4().to_string()[..8]);

        // Validate cron expression (basic validation)
        validate_cron_expression(&schedule)?;

        let job = CronJob {
            id: id.clone(),
            name,
            schedule,
            command,
            enabled: true,
            last_run: None,
            next_run: None, // Would calculate based on schedule
        };

        let mut jobs = self.jobs.write().await;
        jobs.insert(id, job.clone());

        Ok(job)
    }

    /// List all cron jobs
    pub async fn list_crons(&self) -> Result<Vec<CronJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.values().cloned().collect())
    }

    /// Delete a cron job
    pub async fn delete_cron(&self, job_id: &str) -> Result<Option<CronJob>> {
        let mut jobs = self.jobs.write().await;
        Ok(jobs.remove(job_id))
    }

    /// Enable/disable a cron job
    pub async fn toggle_cron(&self, job_id: &str, enabled: bool) -> Result<CronJob> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get_mut(job_id)
            .context(format!("Cron job not found: {}", job_id))?;

        job.enabled = enabled;
        Ok(job.clone())
    }
}

/// Validate cron expression (basic)
fn validate_cron_expression(expr: &str) -> Result<()> {
    let parts: Vec<&str> = expr.split_whitespace().collect();

    if parts.len() != 5 {
        return Err(anyhow::anyhow!(
            "Invalid cron expression: expected 5 fields (minute hour day month weekday), got {}",
            parts.len()
        ));
    }

    // Basic validation - would need full cron parser for production
    for (i, part) in parts.iter().enumerate() {
        if *part != "*"
            && !part
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, ',' | '-' | '/' | '*'))
        {
            return Err(anyhow::anyhow!(
                "Invalid character in cron field {}: {}",
                i + 1,
                part
            ));
        }
    }

    Ok(())
}

static GLOBAL_CRON_MANAGER: std::sync::OnceLock<CronManager> = std::sync::OnceLock::new();

fn global_crons() -> &'static CronManager {
    GLOBAL_CRON_MANAGER.get_or_init(CronManager::new)
}

/// Create a cron job (convenience function)
pub async fn create_cron(name: String, schedule: String, command: String) -> Result<CronJob> {
    global_crons().create_cron(name, schedule, command).await
}

/// List cron jobs (convenience function)
pub async fn list_crons() -> Result<Vec<CronJob>> {
    global_crons().list_crons().await
}

/// Delete a cron job (convenience function)
pub async fn delete_cron(job_id: &str) -> Result<Option<CronJob>> {
    global_crons().delete_cron(job_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_list_crons() {
        let manager = CronManager::new();

        let job = manager
            .create_cron(
                "Test Job".to_string(),
                "*/5 * * * *".to_string(),
                "echo hello".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(job.name, "Test Job");
        assert!(job.enabled);

        let jobs = manager.list_crons().await.unwrap();
        assert_eq!(jobs.len(), 1);
    }

    #[test]
    fn test_validate_cron() {
        assert!(validate_cron_expression("*/5 * * * *").is_ok());
        assert!(validate_cron_expression("0 0 * * *").is_ok());
        assert!(validate_cron_expression("invalid").is_err());
        assert!(validate_cron_expression("0 0 * *").is_err()); // Only 4 fields
    }
}
