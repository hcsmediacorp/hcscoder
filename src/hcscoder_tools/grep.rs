//! hcscoder Grep Tool
//!
//! Pattern searching in files using grep-like functionality.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
use std::path::Path;
#[cfg(test)]
use tokio::fs;

/// Search result entry
#[derive(Debug, Clone)]
pub struct GrepMatch {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub match_start: Option<usize>,
    pub match_end: Option<usize>,
}

/// Search for a literal pattern in files (walkdir + blocking read).
pub async fn grep(
    pattern: &str,
    root_path: &str,
    include_pattern: Option<&str>,
) -> Result<Vec<GrepMatch>> {
    let root = Path::new(root_path).to_path_buf();
    let pattern = pattern.to_string();
    let include_pattern = include_pattern.map(|s| s.to_string());
    tokio::task::spawn_blocking(move || {
        let mut results = Vec::new();
        use walkdir::WalkDir;
        for entry in WalkDir::new(&root).into_iter().filter_entry(|e| {
            let n = e.file_name().to_string_lossy();
            n != ".git" && n != "node_modules" && n != "target"
        }) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if let Some(include) = &include_pattern {
                if let Some(name) = path.file_name() {
                    if !glob_match(include, &name.to_string_lossy()) {
                        continue;
                    }
                }
            }
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for (line_num, line) in content.lines().enumerate() {
                if let Some(pos) = line.find(&pattern) {
                    results.push(GrepMatch {
                        file_path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        match_start: Some(pos),
                        match_end: Some(pos + pattern.len()),
                    });
                }
            }
        }
        Ok::<_, anyhow::Error>(results)
    })
    .await
    .context("grep join")?
}

/// Search with regex pattern
pub async fn grep_regex(
    pattern: &str,
    root_path: &str,
    include_pattern: Option<&str>,
) -> Result<Vec<GrepMatch>> {
    let regex = regex::Regex::new(pattern).context("Invalid regex pattern")?;
    let root = Path::new(root_path).to_path_buf();
    let include_pattern = include_pattern.map(|s| s.to_string());
    tokio::task::spawn_blocking(move || {
        let mut results = Vec::new();
        use walkdir::WalkDir;
        for entry in WalkDir::new(&root).into_iter().filter_entry(|e| {
            let n = e.file_name().to_string_lossy();
            n != ".git" && n != "node_modules" && n != "target"
        }) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if let Some(include) = &include_pattern {
                if let Some(name) = path.file_name() {
                    if !glob_match(include, &name.to_string_lossy()) {
                        continue;
                    }
                }
            }
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for (line_num, line) in content.lines().enumerate() {
                if let Some(m) = regex.find(line) {
                    results.push(GrepMatch {
                        file_path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line.to_string(),
                        match_start: Some(m.start()),
                        match_end: Some(m.end()),
                    });
                }
            }
        }
        Ok::<_, anyhow::Error>(results)
    })
    .await
    .context("grep_regex join")?
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return text.starts_with(parts[0]) && text.ends_with(parts[1]);
        }
    }

    text == pattern
}

/// Count matches without returning them
pub async fn grep_count(
    pattern: &str,
    root_path: &str,
    include_pattern: Option<&str>,
) -> Result<usize> {
    let results = grep(pattern, root_path, include_pattern).await?;
    Ok(results.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grep() {
        let temp_dir = "/tmp/hcscoder_grep_test";
        let _ = fs::create_dir_all(temp_dir).await;

        fs::write(
            format!("{}/test.txt", temp_dir),
            "Hello World\nHello hcscoder\nGoodbye",
        )
        .await
        .unwrap();

        let results = grep("Hello", temp_dir, None).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].line_content.contains("Hello"));

        let _ = fs::remove_dir_all(temp_dir).await;
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("*.txt", "test.txt"));
        assert!(!glob_match("*.rs", "main.ts"));
    }
}
