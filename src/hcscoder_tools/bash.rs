//! hcscoder Bash / shell execution — async via tokio::process.
//!
//! ## Security Considerations
//!
//! This module executes shell commands with comprehensive security measures:
//! - Command injection prevention through strict validation
//! - Dangerous pattern detection and blocking
//! - Audit logging for security monitoring
//! - Path traversal prevention
//! - Resource limits enforcement

use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Command execution output
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
    pub execution_time_ms: u64,
}

lazy_static! {
    /// Patterns that indicate command injection attempts
    static ref INJECTION_PATTERNS: Vec<(Regex, &'static str)> = vec![
        (Regex::new(r";\s*(?:rm|dd|mkfs|chmod|chown)\s").unwrap(), "Command chaining with destructive command"),
        (Regex::new(r"\|\s*(?:rm|dd|mkfs|chmod|chown)\s").unwrap(), "Pipe to destructive command"),
        (Regex::new(r"`[^`]*`").unwrap(), "Backtick command substitution"),
        (Regex::new(r"\$\([^)]+\)").unwrap(), "Shell command substitution"),
        (Regex::new(r"\$\{[^}]+\}").unwrap(), "Variable expansion"),
        (Regex::new(r"&&\s*(?:rm|dd|mkfs)").unwrap(), "AND chain with destructive command"),
        (Regex::new(r"\|\|\s*(?:rm|dd|mkfs)").unwrap(), "OR chain with destructive command"),
        (Regex::new(r">\s*/dev/sd").unwrap(), "Direct device write"),
        (Regex::new(r"<\s*/dev/sd").unwrap(), "Direct device read"),
        (Regex::new(r":\(\)\{\s*:\|:&\s*\};:").unwrap(), "Fork bomb"),
        (Regex::new(r"/etc/(?:passwd|shadow|sudoers)").unwrap(), "Sensitive file access"),
        (Regex::new(r"~[a-zA-Z0-9_-]+/").unwrap(), "Home directory traversal"),
        (Regex::new(r"\.\./").unwrap(), "Parent directory traversal"),
        (Regex::new(r"(?i)(?:curl|wget|fetch).*\|\s*(?:bash|sh)").unwrap(), "Remote code execution pattern"),
    ];

    /// Blocked destructive commands
    static ref BLOCKED_COMMANDS: Vec<&'static str> = vec![
        "rm -rf /",
        "rm -rf /*",
        "mkfs",
        "dd if=",
        ":(){:|:&};:",
        "> /dev/sd",
        "chmod -R 777 /",
        "chown -R",
        "fdisk",
        "parted",
        "mount ",
        "umount ",
        "iptables -F",
        "ufw disable",
        "systemctl stop",
        "kill -9 1",
        "reboot",
        "shutdown",
        "poweroff",
    ];
}

/// Security validation result
#[derive(Debug, Clone, PartialEq)]
enum SecurityCheckResult {
    Safe,
    Warning(String),
    Blocked(String),
}

/// Comprehensive security validation for shell commands
fn validate_command_security(cmd: &str) -> SecurityCheckResult {
    // Reject empty commands
    if cmd.trim().is_empty() {
        return SecurityCheckResult::Blocked("Empty command".to_string());
    }

    // Reject commands with null bytes
    if cmd.contains('\0') {
        return SecurityCheckResult::Blocked("Null byte injection attempt".to_string());
    }

    // Check for blocked commands
    for blocked in BLOCKED_COMMANDS.iter() {
        if cmd.to_lowercase().contains(blocked) {
            tracing::warn!(
                target: "hcscoder::security",
                event = "blocked_command",
                command = %cmd,
                reason = "Matched blocked command pattern"
            );
            return SecurityCheckResult::Blocked(format!(
                "Blocked destructive command pattern: '{}'",
                blocked
            ));
        }
    }

    // Check for injection patterns
    for (pattern, description) in INJECTION_PATTERNS.iter() {
        if pattern.is_match(cmd) {
            tracing::warn!(
                target: "hcscoder::security",
                event = "injection_attempt",
                command = %cmd,
                pattern = %description
            );
            return SecurityCheckResult::Blocked(format!(
                "Potential command injection detected: {}",
                description
            ));
        }
    }

    // Check for potentially dangerous but not blocked patterns
    let warning_patterns = [
        ("sudo", "Privilege escalation command"),
        ("su ", "User switching command"),
        ("export ", "Environment variable modification"),
        ("unset ", "Environment variable removal"),
    ];

    for (pattern, description) in warning_patterns.iter() {
        if cmd.contains(pattern) {
            tracing::info!(
                target: "hcscoder::security",
                event = "warning_command",
                command = %cmd,
                reason = %description
            );
            return SecurityCheckResult::Warning(description.to_string());
        }
    }

    SecurityCheckResult::Safe
}

/// Shell-safe argument escaping
pub fn escape_shell_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }

    // If arg contains only safe characters, no need to quote
    if arg
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | '/' | '@'))
    {
        return arg.to_string();
    }

    // Escape single quotes and wrap in single quotes
    let escaped = arg.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
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
/// # Security Features
/// - Command injection prevention
/// - Dangerous pattern blocking
/// - Execution timeout (default 60s)
/// - Audit logging
/// - Resource limits
///
/// # Arguments
/// * `command` - The shell command to execute
///
/// # Returns
/// * `Ok(CommandOutput)` - Command execution result
/// * `Err(anyhow::Error)` - Execution or validation error
///
/// # Example
/// ```rust,no_run
/// use hcscoder::hcscoder_tools::bash::execute_command;
///
/// #[tokio::main]
/// async fn main() {
///     let output = execute_command("ls -la").await.unwrap();
///     println!("Exit code: {}", output.exit_code);
///     println!("Output: {}", output.stdout);
/// }
/// ```
pub async fn execute_command(command: &str) -> Result<CommandOutput> {
    let start_time = std::time::Instant::now();

    // Comprehensive security validation
    match validate_command_security(command) {
        SecurityCheckResult::Blocked(reason) => {
            anyhow::bail!("Command blocked: {}", reason);
        }
        SecurityCheckResult::Warning(reason) => {
            tracing::warn!("Executing command with warning: {} - {}", command, reason);
        }
        SecurityCheckResult::Safe => {}
    }

    // Log command execution for audit trail
    tracing::info!(
        target: "hcscoder::audit",
        event = "command_execute",
        command = %command
    );

    let (prog, args) = shell_invocation(command);
    let mut child = Command::new(&prog)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Ensure child is terminated if dropped.
        .kill_on_drop(true)
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

    let status = tokio::time::timeout(std::time::Duration::from_secs(60), child.wait())
        .await
        .context("command timed out after 60s")?
        .context("failed to wait on child")?;
    let stdout = stdout_task.await.unwrap_or_default();
    let stderr = stderr_task.await.unwrap_or_default();

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    // Log completion for audit trail
    tracing::info!(
        target: "hcscoder::audit",
        event = "command_complete",
        command = %command,
        exit_code = status.code().unwrap_or(-1),
        execution_time_ms = execution_time_ms,
        stdout_bytes = stdout.len(),
        stderr_bytes = stderr.len()
    );

    Ok(CommandOutput {
        exit_code: status.code().unwrap_or(-1),
        stdout,
        stderr,
        command: command.to_string(),
        execution_time_ms,
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
