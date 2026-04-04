//! hcscoder Bash / shell execution — async via tokio::process.
//! 
//! ## Security Considerations
//! 
//! This module executes shell commands. Users should:
//! - Never pass untrusted input directly to execute_command()
//! - Use shell escaping for user-provided arguments
//! - Consider using direct binary invocation for sensitive operations

use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Command execution output
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Check if command contains potentially dangerous patterns
/// Returns true if the command appears safe to execute
fn is_command_safe(cmd: &str) -> bool {
    // Reject commands with null bytes (should never happen in valid strings)
    if cmd.contains('\0') {
        return false;
    }
    
    // Log potentially dangerous patterns for audit trail
    // Note: We don't block these, just log them for security monitoring
    let dangerous_patterns = ["rm -rf /", "mkfs", "dd if=", ":(){:|:&};:", "> /dev/sd"];
    for pattern in dangerous_patterns.iter() {
        if cmd.contains(pattern) {
            tracing::warn!("Potentially dangerous command detected: {}", cmd);
            break;
        }
    }
    
    true
}

fn shell_invocation(cmd: &str) -> (String, Vec<String>) {
    if cfg!(windows) {
        ("cmd".to_string(), vec!["/C".to_string(), cmd.to_string()])
    } else {
        ("sh".to_string(), vec!["-c".to_string(), cmd.to_string()])
    }
}

/// Execute a shell command asynchronously (cross-platform)
/// 
/// # Security Warning
/// This function executes arbitrary shell commands. Ensure that:
/// - The `command` parameter comes from a trusted source
/// - User input is properly sanitized before being passed
/// - For file operations, consider using the filesystem module instead
pub async fn execute_command(command: &str) -> Result<CommandOutput> {
    // Safety check and logging
    if !is_command_safe(command) {
        anyhow::bail!("Command rejected for security reasons");
    }
    
    let (prog, args) = shell_invocation(command);
    let mut child = Command::new(&prog)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn {:?} {:?}", prog, args))?;

    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();

    let stdout_task = tokio::spawn(async move {
        let mut out = String::new();
        if let Some(stdout) = stdout_handle {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                out.push_str(&line);
                line.clear();
            }
        }
        out
    });

    let stderr_task = tokio::spawn(async move {
        let mut out = String::new();
        if let Some(stderr) = stderr_handle {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                out.push_str(&line);
                line.clear();
            }
        }
        out
    });

    let status = child.wait().await.context("failed to wait on child")?;
    let stdout = stdout_task.await.unwrap_or_default();
    let stderr = stderr_task.await.unwrap_or_default();

    Ok(CommandOutput {
        exit_code: status.code().unwrap_or(-1),
        stdout,
        stderr,
    })
}

/// Execute with wall-clock timeout
pub async fn execute_command_with_timeout(
    command: &str,
    timeout_secs: u64,
) -> Result<CommandOutput> {
    tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        execute_command(command),
    )
    .await
    .context("command timed out")?
}

/// Spawn background process (returns PID when available)
pub async fn execute_background(command: &str) -> Result<u32> {
    let (prog, args) = shell_invocation(command);
    let child = Command::new(&prog)
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to spawn background {:?}", prog))?;

    Ok(child.id().unwrap_or(0))
}

/// Kill a process by PID (best-effort)
pub async fn kill_process(pid: u32) -> Result<()> {
    if pid == 0 {
        return Ok(());
    }
    if cfg!(windows) {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()
            .await
            .context("taskkill")?;
    } else {
        Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status()
            .await
            .context("kill")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo() -> Result<()> {
        let output = execute_command("echo hcscoder").await?;
        assert!(output.exit_code == 0 || cfg!(windows));
        assert!(
            output.stdout.to_lowercase().contains("hcscoder") || output.stdout.contains("hcscoder")
        );
        Ok(())
    }
}
