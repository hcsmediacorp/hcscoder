//! hcscoder Planner Module (Ultraplan & Kairos)
//!
//! Deep planning logic and proactive event-loop monitoring.
//! Zero telemetry, no phone-home logic.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Planning step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderPlanStep {
    pub id: usize,
    pub description: String,
    pub status: StepStatus,
    pub dependencies: Vec<usize>,
    pub estimated_tokens: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
    Skipped,
}

/// Complete plan structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderPlan {
    pub id: String,
    pub objective: String,
    pub steps: Vec<HcscoderPlanStep>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl HcscoderPlan {
    /// Create a new plan from objective
    pub fn new(objective: String) -> Self {
        use uuid::Uuid;

        Self {
            id: Uuid::new_v4().to_string(),
            objective,
            steps: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, description: String, dependencies: Vec<usize>) -> usize {
        let id = self.steps.len();

        self.steps.push(HcscoderPlanStep {
            id,
            description,
            status: if dependencies.is_empty() {
                StepStatus::Pending
            } else {
                StepStatus::Blocked
            },
            dependencies,
            estimated_tokens: 500, // Default estimate
        });

        self.updated_at = Utc::now();
        id
    }

    /// Get progress percentage
    pub fn progress(&self) -> f64 {
        if self.steps.is_empty() {
            return 0.0;
        }

        let completed = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .count();

        (completed as f64 / self.steps.len() as f64) * 100.0
    }

    /// Get next actionable step
    pub fn next_step(&self) -> Option<&HcscoderPlanStep> {
        self.steps.iter().find(|s| {
            s.status == StepStatus::Pending
                && s.dependencies.iter().all(|dep| {
                    self.steps
                        .get(*dep)
                        .map(|d| d.status == StepStatus::Completed)
                        .unwrap_or(false)
                })
        })
    }

    /// Mark step as completed
    pub fn complete_step(&mut self, step_id: usize) -> Result<()> {
        let step = self
            .steps
            .get_mut(step_id)
            .ok_or_else(|| anyhow::anyhow!("Step {} not found", step_id))?;

        step.status = StepStatus::Completed;
        self.updated_at = Utc::now();

        // Update blocked steps that depend on this one (avoid aliasing self.steps)
        let mut unblock: Vec<usize> = Vec::new();
        for (i, step) in self.steps.iter().enumerate() {
            if step.status == StepStatus::Blocked {
                let deps_ok = step.dependencies.iter().all(|dep| {
                    self.steps
                        .get(*dep)
                        .map(|d| d.status == StepStatus::Completed)
                        .unwrap_or(false)
                });
                if deps_ok {
                    unblock.push(i);
                }
            }
        }
        for i in unblock {
            if let Some(s) = self.steps.get_mut(i) {
                s.status = StepStatus::Pending;
            }
        }

        Ok(())
    }
}

/// Kairos Event Monitor for proactive monitoring
pub struct HcscoderKairosMonitor {
    events: Vec<KairosEvent>,
    monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KairosEvent {
    pub id: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub data: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    FileChange,
    CommandExecuted,
    ErrorOccurred,
    TaskCompleted,
    UserInput,
    SystemState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

impl HcscoderKairosMonitor {
    /// Create new monitor
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            monitoring: false,
        }
    }

    /// Start monitoring
    pub fn start(&mut self) {
        self.monitoring = true;
        tracing::info!("Kairos monitor started");
    }

    /// Stop monitoring
    pub fn stop(&mut self) {
        self.monitoring = false;
        tracing::info!("Kairos monitor stopped");
    }

    /// Record an event
    pub fn record_event(&mut self, event_type: EventType, data: String, priority: Priority) {
        use uuid::Uuid;

        let event = KairosEvent {
            id: Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            data,
            priority,
        };

        self.events.push(event);

        // Keep only last 1000 events
        if self.events.len() > 1000 {
            self.events.remove(0);
        }
    }

    /// Get recent events by priority
    pub fn get_recent_events(&self, min_priority: Priority, limit: usize) -> Vec<&KairosEvent> {
        self.events
            .iter()
            .filter(|e| e.priority >= min_priority)
            .rev()
            .take(limit)
            .collect()
    }

    /// Analyze patterns in events
    pub fn analyze_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        // Count event types
        let mut type_counts = std::collections::HashMap::new();
        for event in &self.events {
            *type_counts.entry(event.event_type).or_insert(0) += 1;
        }

        // Find dominant patterns
        for (event_type, count) in type_counts {
            if count > 10 {
                patterns.push(format!(
                    "High frequency of {:?} events ({})",
                    event_type, count
                ));
            }
        }

        // Check for error spikes
        let recent_errors = self
            .events
            .iter()
            .filter(|e| e.event_type == EventType::ErrorOccurred && e.priority >= Priority::High)
            .count();

        if recent_errors > 5 {
            patterns.push(format!(
                "Error spike detected: {} recent errors",
                recent_errors
            ));
        }

        patterns
    }

    /// Get event summary
    pub fn get_summary(&self) -> String {
        format!(
            "Kairos Monitor: {} events recorded | Monitoring: {}",
            self.events.len(),
            if self.monitoring {
                "Active"
            } else {
                "Inactive"
            }
        )
    }
}

impl Default for HcscoderKairosMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Ultraplan deep planning system
pub struct HcscoderUltraplan {
    plans: Vec<HcscoderPlan>,
    active_plan_id: Option<String>,
}

impl HcscoderUltraplan {
    /// Create new Ultraplan system
    pub fn new() -> Self {
        Self {
            plans: Vec::new(),
            active_plan_id: None,
        }
    }

    /// Create a new plan
    pub fn create_plan(&mut self, objective: String) -> &HcscoderPlan {
        let plan = HcscoderPlan::new(objective);
        self.active_plan_id = Some(plan.id.clone());
        self.plans.push(plan);
        let idx = self.plans.len() - 1;
        &self.plans[idx]
    }

    /// Get active plan
    pub fn get_active_plan(&self) -> Option<&HcscoderPlan> {
        self.active_plan_id
            .as_ref()
            .and_then(|id| self.plans.iter().find(|p| p.id == *id))
    }

    /// Get mutable active plan
    pub fn get_active_plan_mut(&mut self) -> Option<&mut HcscoderPlan> {
        let id = self.active_plan_id.clone()?;
        self.plans.iter_mut().find(|p| p.id == id)
    }

    /// List all plans
    pub fn list_plans(&self) {
        println!("\n📋 hcscoder Plans");
        println!("{}", "=".repeat(50));

        if self.plans.is_empty() {
            println!("No plans yet.");
            return;
        }

        for plan in &self.plans {
            let active_marker = if Some(&plan.id) == self.active_plan_id.as_ref() {
                "🔴"
            } else {
                "  "
            };

            println!(
                "{} [{}] {}",
                active_marker,
                plan.id.chars().take(8).collect::<String>(),
                plan.objective
            );
            println!(
                "   Progress: {:.1}% | Steps: {}",
                plan.progress(),
                plan.steps.len()
            );
        }

        println!("{}", "=".repeat(50));
    }
}

impl Default for HcscoderUltraplan {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_creation() {
        let mut plan = HcscoderPlan::new("Test objective".to_string());

        let step1 = plan.add_step("First step".to_string(), vec![]);
        let step2 = plan.add_step("Second step".to_string(), vec![step1]);

        assert_eq!(plan.steps.len(), 2);
        assert_eq!(step1, 0);
        assert_eq!(step2, 1);
    }

    #[test]
    fn test_plan_progress() {
        let mut plan = HcscoderPlan::new("Test".to_string());
        plan.add_step("Step 1".to_string(), vec![]);
        plan.add_step("Step 2".to_string(), vec![]);

        assert!((plan.progress() - 0.0).abs() < 0.01);

        plan.complete_step(0).expect("step 0 exists");
        assert!((plan.progress() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_kairos_monitor() {
        let mut monitor = HcscoderKairosMonitor::new();

        monitor.record_event(
            EventType::FileChange,
            "file.rs modified".to_string(),
            Priority::Normal,
        );

        monitor.record_event(
            EventType::ErrorOccurred,
            "Compilation error".to_string(),
            Priority::High,
        );

        let high_priority = monitor.get_recent_events(Priority::High, 10);
        assert_eq!(high_priority.len(), 1);
    }
}
