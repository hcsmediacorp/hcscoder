//! hcscoder REPL Tool
//!
//! Read-Eval-Print Loop for interactive code execution.
//! NOTE: Code execution is currently simulated (experimental).
//! Zero telemetry, no phone-home logic.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// REPL session state
#[derive(Debug, Clone, Default)]
pub struct ReplSession {
    pub history: Vec<String>,
    pub variables: std::collections::HashMap<String, String>,
    pub active: bool,
}

/// Global REPL session
static REPL_SESSION: std::sync::OnceLock<Arc<RwLock<ReplSession>>> = std::sync::OnceLock::new();

fn get_session() -> Arc<RwLock<ReplSession>> {
    REPL_SESSION
        .get_or_init(|| Arc::new(RwLock::new(ReplSession::default())))
        .clone()
}

/// Start a REPL session
pub async fn start_repl() -> Result<String> {
    let session = get_session();
    let mut s = session.write().await;

    if s.active {
        return Ok("REPL session already active".to_string());
    }

    s.active = true;
    s.history.clear();
    s.variables.clear();

    Ok("REPL session started. Use /exit to end.".to_string())
}

/// Execute code in REPL
pub async fn repl_eval(code: &str) -> Result<String> {
    let session = get_session();
    let mut s = session.write().await;

    if !s.active {
        return Err(anyhow::anyhow!(
            "REPL session not active. Use /start-repl first."
        ));
    }

    s.history.push(code.to_string());

    // Simulated execution until a sandboxed evaluator is integrated.
    Ok(format!(
        "Simulated eval (experimental): {} ({} chars)",
        code,
        code.len()
    ))
}

/// Get REPL history
pub async fn repl_history() -> Result<Vec<String>> {
    let session = get_session();
    let s = session.read().await;
    Ok(s.history.clone())
}

/// End REPL session
pub async fn end_repl() -> Result<String> {
    let session = get_session();
    let mut s = session.write().await;

    if !s.active {
        return Ok("No active REPL session".to_string());
    }

    s.active = false;
    let history_len = s.history.len();

    Ok(format!(
        "REPL session ended. {} commands executed.",
        history_len
    ))
}

/// Set a variable in REPL
pub async fn repl_set_var(name: &str, value: &str) -> Result<String> {
    let session = get_session();
    let mut s = session.write().await;

    s.variables.insert(name.to_string(), value.to_string());
    Ok(format!("Set {} = {}", name, value))
}

/// Get a variable from REPL
pub async fn repl_get_var(name: &str) -> Result<Option<String>> {
    let session = get_session();
    let s = session.read().await;
    Ok(s.variables.get(name).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repl_lifecycle() {
        let start_result = start_repl().await.unwrap();
        assert!(start_result.contains("started"));

        let eval_result = repl_eval("let x = 5").await.unwrap();
        assert!(eval_result.contains("Simulated eval"));

        let history = repl_history().await.unwrap();
        assert_eq!(history.len(), 1);

        let end_result = end_repl().await.unwrap();
        assert!(end_result.contains("ended"));
    }
}
