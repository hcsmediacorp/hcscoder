//! hcscoder Plan Mode Tool
//!
//! Enter and exit planning mode for structured development.
//! Zero telemetry, no phone-home logic.

use anyhow::Result;

/// Plan mode state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanModeState {
    Active,
    Inactive,
}

/// Global plan mode state
static PLAN_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Enter plan mode
pub async fn enter_plan_mode() -> Result<String> {
    PLAN_MODE.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok("Plan mode activated. Use /exit-plan to return to coding mode.".to_string())
}

/// Exit plan mode
pub async fn exit_plan_mode() -> Result<String> {
    PLAN_MODE.store(false, std::sync::atomic::Ordering::SeqCst);
    Ok("Plan mode deactivated. Ready to code!".to_string())
}

/// Check if plan mode is active
pub fn is_plan_mode_active() -> bool {
    PLAN_MODE.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enter_exit_plan_mode() {
        assert!(!is_plan_mode_active());

        enter_plan_mode().await.unwrap();
        assert!(is_plan_mode_active());

        exit_plan_mode().await.unwrap();
        assert!(!is_plan_mode_active());
    }
}
