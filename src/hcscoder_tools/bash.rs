//! hcscoder Bash / shell execution — async via tokio::process.

use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
    pub execution_time_ms: u64,
}

lazy_static! {
    static ref INJECTION_PATTERNS: Vec<(Regex, &'static str)> = vec![
        (
            Regex::new(r"`[^`]*`").unwrap(),
            "Backtick command substitution"
        ),
        (
            Regex::new(r"\$\([^)]+\)").unwrap(),
            "Shell command substitution"
        ),
        (
            Regex::new(r"\$\{[^}]+\}").unwrap(),
            "Brace variable expansion"
        ),
        (
            Regex::new(r"(^|[^\\])\$[A-Za-z_][A-Za-z0-9_]*").unwrap(),
            "Variable expansion"
        ),
        (
            Regex::new(r"\|\||&&|;|\|&").unwrap(),
            "Command chaining/metacharacter"
        ),
        (
            Regex::new(r">\s*/dev/(?:sd|null|zero|random|tty)").unwrap(),
            "Redirection to device"
        ),
        (
            Regex::new(r"<\s*/dev/(?:sd|null|zero|random|tty)").unwrap(),
            "Redirection from device"
        ),
        (
            Regex::new(r"(?i)\b(?:export|unset)\s+[A-Za-z_][A-Za-z0-9_]*").unwrap(),
            "Environment mutation"
        ),
        (Regex::new(r"\.{2}/").unwrap(), "Path traversal"),
        (
            Regex::new(r"(?i)(?:curl|wget|fetch).*(?:\||>)\s*(?:bash|sh)").unwrap(),
            "Remote code execution pattern"
        ),
    ];
    static ref BLOCKED_COMMANDS: Vec<&'static str> = vec![
        "rm -rf /",
        "rm -rf /*",
        "mkfs",
        "dd if=",
        ":(){:|:&};:",
        "fdisk",
        "parted",
        "mount ",
        "umount ",
        "iptables -f",
        "ufw disable",
        "systemctl stop",
        "kill -9 1",
        "reboot",
        "shutdown",
        "poweroff",
    ];
    static ref SIMPLE_TOKEN_RE: Regex = Regex::new(r"^[A-Za-z0-9_./:=@,+%-]+$").unwrap();
}

#[derive(Debug, Clone, PartialEq)]
enum SecurityCheckResult {
    Safe,
    Warning(String),
    Blocked(String),
}

fn validate_command_security(cmd: &str) -> SecurityCheckResult {
    if cmd.trim().is_empty() {
        return SecurityCheckResult::Blocked("Empty command".to_string());
    }
    if cmd.contains('\0') {
        return SecurityCheckResult::Blocked("Null byte injection attempt".to_string());
    }

    let lowered = cmd.to_lowercase();
    for blocked in &*BLOCKED_COMMANDS {
        if lowered.contains(blocked) {
            return SecurityCheckResult::Blocked(format!(
                "Blocked destructive command pattern: '{}'",
                blocked
            ));
        }
    }

    for (pattern, description) in &*INJECTION_PATTERNS {
        if pattern.is_match(cmd) {
            return SecurityCheckResult::Blocked(format!(
                "Potential command injection detected: {description}"
            ));
        }
    }

    for pattern in ["sudo", " su "] {
        if lowered.contains(pattern) {
            return SecurityCheckResult::Warning("Privileged execution requested".to_string());
        }
    }

    SecurityCheckResult::Safe
}

pub fn escape_shell_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    if arg
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | '/' | '@'))
    {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', "'\"'\"'"))
}

fn parse_simple_command(command: &str) -> Result<(String, Vec<String>)> {
    let mut parts = command.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty command"))?
        .to_string();

    if !SIMPLE_TOKEN_RE.is_match(&program) {
        anyhow::bail!("Program contains unsupported characters for safe mode");
    }

    let mut args = Vec::new();
    for part in parts {
        if !SIMPLE_TOKEN_RE.is_match(part) {
            anyhow::bail!("Argument '{part}' contains unsupported characters for safe mode");
        }
        args.push(part.to_string());
    }

    Ok((program, args))
}

fn shell_invocation(cmd: &str) -> (String, Vec<String>) {
    if cfg!(windows) {
        ("cmd".to_string(), vec!["/C".to_string(), cmd.to_string()])
    } else {
        ("sh".to_string(), vec!["-c".to_string(), cmd.to_string()])
    }
}

async fn execute_spawned_command(
    mut child: tokio::process::Child,
    command: &str,
    start_time: std::time::Instant,
) -> Result<CommandOutput> {
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
        command: command.to_string(),
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    })
}

pub async fn execute_simple_command(command: &str) -> Result<CommandOutput> {
    match validate_command_security(command) {
        SecurityCheckResult::Blocked(reason) => anyhow::bail!("Command blocked: {}", reason),
        SecurityCheckResult::Warning(_) | SecurityCheckResult::Safe => {}
    }

    let start = std::time::Instant::now();
    let (program, args) = parse_simple_command(command)?;

    let child = Command::new(&program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("failed to spawn simple command: {program}"))?;

    execute_spawned_command(child, command, start).await
}

/// Executes in safe simple mode first; only falls back to shell parsing when explicitly requested.
pub async fn execute_command(command: &str) -> Result<CommandOutput> {
    execute_simple_command(command).await
}

pub async fn execute_raw_shell(command: &str) -> Result<CommandOutput> {
    match validate_command_security(command) {
        SecurityCheckResult::Blocked(reason) => anyhow::bail!("Command blocked: {}", reason),
        SecurityCheckResult::Warning(_) | SecurityCheckResult::Safe => {}
    }

    let start = std::time::Instant::now();
    let (prog, args) = shell_invocation(command);
    let child = Command::new(&prog)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("failed to spawn {:?} {:?}", prog, args))?;

    execute_spawned_command(child, command, start).await
}

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

pub async fn execute_background(command: &str) -> Result<u32> {
    match validate_command_security(command) {
        SecurityCheckResult::Blocked(reason) => anyhow::bail!("Command blocked: {}", reason),
        SecurityCheckResult::Warning(_) | SecurityCheckResult::Safe => {}
    }

    let (prog, args) = shell_invocation(command);
    let child = Command::new(&prog)
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to spawn background {:?}", prog))?;
    Ok(child.id().unwrap_or(0))
}

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
    async fn safe_command_success() {
        let output = execute_command("echo hcscoder").await.unwrap();
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.to_lowercase().contains("hcscoder"));
    }

    #[tokio::test]
    async fn blocked_chained_destructive_command() {
        let err = execute_command("echo ok && rm -rf /")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("Command blocked"));
    }

    #[tokio::test]
    async fn blocked_command_substitution() {
        let err = execute_command("echo $(whoami)")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("Command blocked"));
    }

    #[tokio::test]
    async fn blocked_variable_expansion() {
        let err = execute_command("echo $HOME").await.unwrap_err().to_string();
        assert!(err.contains("Command blocked"));
    }

    #[tokio::test]
    async fn blocked_path_traversal_input() {
        let err = execute_command("cat ../secret.txt")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("Command blocked"));
    }
}
