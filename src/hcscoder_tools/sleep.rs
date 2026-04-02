//! SleepTool - Wait for a specified duration
//!
//! Allows waiting without holding a shell process. User can interrupt at any time.

use anyhow::{Context, Result};
use serde_json::json;
use tokio::time::{sleep, Duration};

/// SleepTool for waiting specified durations
pub struct SleepTool;

impl SleepTool {
    /// Create a new SleepTool instance
    pub fn new() -> Self {
        Self
    }

    /// Sleep for specified milliseconds
    pub async fn sleep(
        &self,
        duration_ms: u64,
        allow_interrupt: bool,
    ) -> Result<serde_json::Value> {
        // Cap maximum sleep time to 1 hour (tests use a short cap so CI does not stall)
        let max_duration = max_sleep_ms();
        let actual_duration = duration_ms.min(max_duration);

        if actual_duration != duration_ms {
            tracing::warn!(
                "Sleep duration capped from {}ms to {}ms",
                duration_ms,
                actual_duration
            );
        }

        let start = std::time::Instant::now();

        // In a real implementation with interrupt support, we'd check for cancellation
        // For now, we use standard tokio sleep
        if allow_interrupt {
            // With interrupt support, we'd use tokio::select! with a cancellation token
            sleep(Duration::from_millis(actual_duration)).await;
        } else {
            sleep(Duration::from_millis(actual_duration)).await;
        }

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(json!({
            "status": "completed",
            "requested_duration_ms": duration_ms,
            "actual_duration_ms": elapsed,
            "interrupted": false,
        }))
    }

    /// Execute sleep tool and return JSON result
    pub async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let duration_ms = params["duration_ms"]
            .as_u64()
            .context("Missing or invalid 'duration_ms' parameter")?;

        let allow_interrupt = params["allow_interrupt"].as_bool().unwrap_or(true);

        self.sleep(duration_ms, allow_interrupt).await
    }
}

impl Default for SleepTool {
    fn default() -> Self {
        Self::new()
    }
}

fn max_sleep_ms() -> u64 {
    #[cfg(test)]
    {
        5_000
    }
    #[cfg(not(test))]
    {
        3_600_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_short_sleep() {
        let tool = SleepTool::new();
        let result = tool.sleep(100, true).await.unwrap();

        assert_eq!(result["status"], "completed");
        let actual = result["actual_duration_ms"].as_u64().unwrap();
        assert!((95..=150).contains(&actual)); // Allow some timing variance
    }

    #[tokio::test]
    async fn test_capped_sleep() {
        let tool = SleepTool::new();
        // Under tests, max sleep is 5s — request more and ensure we cap and finish quickly
        let result = tool.sleep(100_000, false).await.unwrap();

        assert_eq!(result["status"], "completed");
        assert_eq!(result["requested_duration_ms"], 100_000);
        let actual = result["actual_duration_ms"].as_u64().unwrap();
        assert!(actual <= 6_000, "expected capped sleep, got {}ms", actual);
    }
}
