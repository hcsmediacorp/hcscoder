//! IDE-style helpers without embedding a full LSP client: text search + optional `cargo check`.

use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

/// Heuristic “diagnostics”: run `cargo check` when `path` is inside a Rust project.
pub async fn get_diagnostics(file_path: &str) -> Result<Vec<String>> {
    let path = Path::new(file_path);
    let mut dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| Path::new(".").to_path_buf());

    for _ in 0..8 {
        if dir.join("Cargo.toml").exists() {
            let out = Command::new("cargo")
                .current_dir(&dir)
                .args(["check", "--message-format", "short"])
                .output()
                .await
                .context("failed to run cargo check — is Rust installed?")?;
            let mut lines = Vec::new();
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            for line in combined.lines().take(50) {
                if !line.trim().is_empty() {
                    lines.push(line.to_string());
                }
            }
            if lines.is_empty() {
                lines.push("(cargo check produced no text output)".to_string());
            }
            return Ok(lines);
        }
        if !dir.pop() {
            break;
        }
    }

    Ok(vec![format!(
        "No Cargo.toml found near {}; skipped cargo check.",
        file_path
    )])
}

/// Find `fn name` / `struct name` / … in file (regex).
pub async fn find_definitions(file_path: &str, symbol: &str) -> Result<Vec<String>> {
    let content = fs::read_to_string(file_path)
        .await
        .with_context(|| format!("read {}", file_path))?;
    let pattern = format!(
        r"(?m)^\s*(pub\s+)?(async\s+)?(fn|struct|enum|trait|type)\s+{}\b",
        regex::escape(symbol)
    );
    let re = Regex::new(&pattern).context("invalid symbol regex")?;
    let mut hits = Vec::new();
    for cap in re.find_iter(&content) {
        let line = content[..cap.start()].lines().count() + 1;
        hits.push(format!("{}:{} — {}", file_path, line, cap.as_str().trim()));
    }
    Ok(hits)
}

/// Simple reference search: occurrences of symbol in file.
pub async fn find_references(file_path: &str, symbol: &str) -> Result<Vec<String>> {
    let content = fs::read_to_string(file_path)
        .await
        .with_context(|| format!("read {}", file_path))?;
    let mut hits = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if line.contains(symbol) {
            hits.push(format!("{}:{} — {}", file_path, i + 1, line.trim()));
        }
    }
    Ok(hits)
}

/// Heuristic “hover”: reuse definition search (no LSP server).
pub async fn hover_symbol(file_path: &str, symbol: &str) -> Result<String> {
    let defs = find_definitions(file_path, symbol).await?;
    Ok(if defs.is_empty() {
        format!("No definition-like match for `{}` in {}", symbol, file_path)
    } else {
        defs.join("\n")
    })
}

/// Heuristic completions: tokens in file starting with `prefix`.
pub async fn completion_prefix(file_path: &str, prefix: &str) -> Result<Vec<String>> {
    let content = fs::read_to_string(file_path)
        .await
        .with_context(|| format!("read {}", file_path))?;
    let mut out = Vec::new();
    for word in content.split(|c: char| {
        c.is_whitespace() || matches!(c, '{' | '}' | '(' | ')' | ';' | ',' | '.' | ':')
    }) {
        let w = word.trim();
        if w.len() > prefix.len() && w.starts_with(prefix) {
            out.push(w.to_string());
        }
    }
    out.sort();
    out.dedup();
    Ok(out.into_iter().take(80).collect())
}
