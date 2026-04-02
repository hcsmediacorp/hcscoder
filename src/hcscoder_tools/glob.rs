//! hcscoder Glob Tool
//!
//! File pattern matching and globbing operations.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
#[cfg(test)]
use tokio::fs;

/// Glob result entry
#[derive(Debug, Clone)]
pub struct GlobEntry {
    pub path: String,
    pub is_file: bool,
    pub is_dir: bool,
}

/// Find files matching a glob pattern
pub async fn glob(pattern: &str, root_path: &str) -> Result<Vec<GlobEntry>> {
    let root = std::path::Path::new(root_path).to_path_buf();
    let pattern = pattern.to_string();
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
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if glob_match(&pattern, &name_str) {
                    results.push(GlobEntry {
                        path: path.to_string_lossy().to_string(),
                        is_file: true,
                        is_dir: false,
                    });
                }
            }
        }
        Ok::<_, anyhow::Error>(results)
    })
    .await
    .context("glob join")?
}

/// Advanced glob pattern matching with ** support
pub fn glob_match(pattern: &str, text: &str) -> bool {
    // Handle ** (recursive directory match)
    if pattern.contains("**") {
        return glob_match_recursive(pattern, text);
    }

    // Simple single * matching
    if pattern == "*" {
        return true;
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return text.starts_with(parts[0]) && text.ends_with(parts[1]);
        }
        // Multiple wildcards
        return glob_multiple_wildcards(&parts, text);
    }

    // Exact match
    text == pattern
}

fn glob_match_recursive(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split("**").collect();

    if parts.len() == 2 {
        let path = text.replace('\\', "/");
        let prefix = parts[0].trim_end_matches('/');
        let suffix = parts[1].trim_start_matches('/');

        if !prefix.is_empty()
            && !path.starts_with(prefix)
            && !path.starts_with(&format!("{}/", prefix.trim_end_matches('/')))
        {
            return false;
        }

        if suffix.is_empty() {
            return true;
        }

        // e.g. "*.rs" — match file segment
        if suffix.contains('*') {
            let file = path.rsplit('/').next().unwrap_or(&path);
            return glob_match(suffix, file);
        }

        return path.ends_with(suffix);
    }

    glob_match(&pattern.replace("**", "*"), text)
}

fn glob_multiple_wildcards(parts: &[&str], text: &str) -> bool {
    if parts.is_empty() {
        return text.is_empty();
    }

    let mut pos = 0;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 {
            // First part must match at start
            if !text.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if i == parts.len() - 1 {
            // Last part must match at end
            if !text.ends_with(part) {
                return false;
            }
        } else {
            // Middle parts can be anywhere after current position
            if let Some(found_pos) = text[pos..].find(part) {
                pos = pos + found_pos + part.len();
            } else {
                return false;
            }
        }
    }

    true
}

/// Find files by extension
pub async fn find_by_extension(root_path: &str, extension: &str) -> Result<Vec<String>> {
    let pattern = format!("*.{}", extension.trim_start_matches('.'));
    let entries = glob(&pattern, root_path).await?;
    Ok(entries.into_iter().map(|e| e.path).collect())
}

/// Find all files of specific types
pub async fn find_code_files(root_path: &str) -> Result<Vec<GlobEntry>> {
    let code_extensions = [
        "rs", "ts", "tsx", "js", "jsx", "py", "go", "c", "cpp", "h", "hpp", "java", "cs", "rb",
        "php", "swift", "kt", "scala", "sh", "bash", "zsh", "fish", "ps1", "sql", "html", "css",
        "scss", "sass", "less", "json", "yaml", "yml", "toml", "xml", "md", "rst",
    ];

    let mut results = Vec::new();
    for ext in code_extensions {
        let pattern = format!("*.{}", ext);
        let mut entries = glob(&pattern, root_path).await?;
        results.append(&mut entries);
    }

    Ok(results)
}

/// Match against multiple patterns
pub async fn glob_many(patterns: &[&str], root_path: &str) -> Result<Vec<GlobEntry>> {
    let mut results = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for pattern in patterns {
        let entries = glob(pattern, root_path).await?;
        for entry in entries {
            if seen_paths.insert(entry.path.clone()) {
                results.push(entry);
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_simple() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("*.txt", "test.txt"));
        assert!(!glob_match("*.rs", "main.ts"));
    }

    #[test]
    fn test_glob_match_wildcard() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("test.*", "test.txt"));
        assert!(glob_match("test.*", "test.rs"));
        assert!(!glob_match("test.*", "other.txt"));
    }

    #[test]
    fn test_glob_match_double_star() {
        assert!(glob_match("**/*.rs", "src/main.rs"));
        assert!(glob_match("**/*.rs", "lib/utils/helpers/test.rs"));
        assert!(glob_match("src/**/*.rs", "src/lib/mod.rs"));
    }

    #[tokio::test]
    async fn test_find_by_extension() {
        let temp_dir = "/tmp/hcscoder_glob_test";
        let _ = fs::create_dir_all(temp_dir).await;

        fs::write(format!("{}/test.rs", temp_dir), "// test")
            .await
            .unwrap();
        fs::write(format!("{}/main.rs", temp_dir), "// main")
            .await
            .unwrap();
        fs::write(format!("{}/test.txt", temp_dir), "text")
            .await
            .unwrap();

        let results = find_by_extension(temp_dir, "rs").await.unwrap();
        assert_eq!(results.len(), 2);

        let _ = fs::remove_dir_all(temp_dir).await;
    }
}
