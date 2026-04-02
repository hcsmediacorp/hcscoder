//! Git operations via `git` CLI (parity with scripted TS workflows).

use anyhow::{Context, Result};
use std::path::Path;
use tokio::process::Command;

async fn run_git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .await
        .context("failed to spawn `git` — is it installed and on PATH?")?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("git exited {}: {}", out.status, stderr);
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// `git status --porcelain`
pub async fn git_status(repo: &str) -> Result<String> {
    let p = Path::new(repo);
    run_git(p, &["status", "--porcelain"]).await
}

/// `git diff` (unstaged)
pub async fn git_diff(repo: &str) -> Result<String> {
    let p = Path::new(repo);
    run_git(p, &["diff"]).await
}

/// Recent log
pub async fn git_log(repo: &str, max_entries: usize) -> Result<String> {
    let p = Path::new(repo);
    let n = max_entries.to_string();
    run_git(p, &["log", "--oneline", "-n", n.as_str()]).await
}

/// Current branch name
pub async fn git_branch_show_current(repo: &str) -> Result<String> {
    let p = Path::new(repo);
    let s = run_git(p, &["branch", "--show-current"]).await?;
    Ok(s.trim().to_string())
}

/// List branches
pub async fn git_branch_list(repo: &str) -> Result<String> {
    let p = Path::new(repo);
    run_git(p, &["branch", "-a"]).await
}
