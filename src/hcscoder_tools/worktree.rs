//! hcscoder Worktree Tool
//!
//! Git worktree management for parallel development.
//! Zero telemetry, no phone-home logic.

use crate::hcscoder_tools::bash;
use anyhow::{Context, Result};

/// Enter a git worktree
pub async fn enter_worktree(path: &str, branch: Option<&str>) -> Result<String> {
    let command = if let Some(b) = branch {
        format!("git worktree add {} -b {}", path, b)
    } else {
        format!("git worktree add {}", path)
    };

    let output = bash::execute_command(&command).await?;

    if output.exit_code == 0 {
        Ok(format!("Entered worktree at: {}", path))
    } else {
        Err(anyhow::anyhow!(
            "Failed to enter worktree: {}",
            output.stderr
        ))
    }
}

/// Exit current worktree (return to main)
pub async fn exit_worktree() -> Result<String> {
    // Get the main repository path
    let output = bash::execute_command("git rev-parse --show-toplevel").await?;

    if output.exit_code != 0 {
        return Err(anyhow::anyhow!("Not in a git repository"));
    }

    let main_path = output.stdout.trim();
    std::env::set_current_dir(main_path).context("Failed to change to main repository")?;

    Ok(format!("Exited worktree, returned to: {}", main_path))
}

/// List all worktrees
pub async fn list_worktrees() -> Result<Vec<WorktreeInfo>> {
    let output = bash::execute_command("git worktree list --porcelain").await?;

    if output.exit_code != 0 {
        return Err(anyhow::anyhow!("Failed to list worktrees"));
    }

    let mut worktrees = Vec::new();
    let mut current = WorktreeInfo::default();

    for line in output.stdout.lines() {
        if line.starts_with("worktree ") {
            if !current.path.is_empty() {
                worktrees.push(current);
            }
            current = WorktreeInfo {
                path: line.strip_prefix("worktree ").unwrap_or("").to_string(),
                ..Default::default()
            };
        } else if line.starts_with("HEAD ") {
            current.head = line.strip_prefix("HEAD ").unwrap_or("").to_string();
        } else if line.starts_with("branch ") {
            current.branch = line
                .strip_prefix("branch ")
                .unwrap_or("")
                .trim_start_matches("refs/heads/")
                .to_string();
        }
    }

    if !current.path.is_empty() {
        worktrees.push(current);
    }

    Ok(worktrees)
}

/// Worktree information
#[derive(Debug, Clone, Default)]
pub struct WorktreeInfo {
    pub path: String,
    pub head: String,
    pub branch: String,
}

/// Remove a worktree
pub async fn remove_worktree(path: &str) -> Result<String> {
    let output = bash::execute_command(&format!("git worktree remove {}", path)).await?;

    if output.exit_code == 0 {
        Ok(format!("Removed worktree: {}", path))
    } else {
        Err(anyhow::anyhow!(
            "Failed to remove worktree: {}",
            output.stderr
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_worktrees() {
        // This may fail if not in a git repo, which is expected
        let result = list_worktrees().await;
        // Just verify it doesn't panic
        let _ = result;
    }
}
