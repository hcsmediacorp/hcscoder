//! hcscoder Filesystem Tool
//!
//! Safe file and directory operations.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Read file contents
pub async fn read_file(path: &str) -> Result<String> {
    fs::read_to_string(path)
        .await
        .context(format!("Failed to read file: {}", path))
}

/// Write content to file (creates if not exists, overwrites if exists)
pub async fn write_file(path: &str, content: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).await?;
        }
    }

    let mut file = fs::File::create(path)
        .await
        .context(format!("Failed to create file: {}", path))?;

    file.write_all(content.as_bytes())
        .await
        .context(format!("Failed to write to file: {}", path))?;

    Ok(())
}

/// Append content to file
pub async fn append_file(path: &str, content: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .context(format!("Failed to open file for appending: {}", path))?;

    file.write_all(content.as_bytes())
        .await
        .context(format!("Failed to append to file: {}", path))?;

    Ok(())
}

/// List directory contents
pub async fn list_directory(path: &str) -> Result<Vec<PathEntry>> {
    let mut entries = Vec::new();
    let mut dir = fs::read_dir(path)
        .await
        .context(format!("Failed to read directory: {}", path))?;

    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        let metadata = fs::metadata(&path).await.ok();

        let entry_type = if let Some(meta) = &metadata {
            if meta.is_dir() {
                EntryType::Directory
            } else if meta.is_file() {
                EntryType::File
            } else {
                EntryType::Other
            }
        } else {
            EntryType::Unknown
        };

        entries.push(PathEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.to_string_lossy().to_string(),
            entry_type,
            size: metadata.map(|m| m.len()).unwrap_or(0),
        });
    }

    Ok(entries)
}

/// Directory entry information
#[derive(Debug, Clone)]
pub struct PathEntry {
    pub name: String,
    pub path: String,
    pub entry_type: EntryType,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Other,
    Unknown,
}

/// Search for files matching a pattern (walkdir; skips `.git` / `node_modules` / `target`).
pub async fn search_files(root: &str, pattern: &str) -> Result<Vec<PathBuf>> {
    let root = Path::new(root).to_path_buf();
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
                    results.push(path.to_path_buf());
                }
            }
        }
        Ok::<_, anyhow::Error>(results)
    })
    .await
    .context("search_files join")?
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

/// Create a new directory
pub async fn create_directory(path: &str) -> Result<()> {
    fs::create_dir_all(path)
        .await
        .context(format!("Failed to create directory: {}", path))
}

/// Delete a file
pub async fn delete_file(path: &str) -> Result<()> {
    fs::remove_file(path)
        .await
        .context(format!("Failed to delete file: {}", path))
}

/// Delete a directory recursively
pub async fn delete_directory(path: &str) -> Result<()> {
    fs::remove_dir_all(path)
        .await
        .context(format!("Failed to delete directory: {}", path))
}

/// Move/rename a file or directory
pub async fn move_path(from: &str, to: &str) -> Result<()> {
    fs::rename(from, to)
        .await
        .context(format!("Failed to move {} to {}", from, to))
}

/// Copy a file
pub async fn copy_file(from: &str, to: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(to).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).await?;
        }
    }

    fs::copy(from, to)
        .await
        .context(format!("Failed to copy {} to {}", from, to))?;

    Ok(())
}

/// Get file metadata
pub async fn get_metadata(path: &str) -> Result<FileMetadata> {
    let metadata = fs::metadata(path)
        .await
        .context(format!("Failed to get metadata for: {}", path))?;

    Ok(FileMetadata {
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        modified: metadata.modified().ok(),
        accessed: metadata.accessed().ok(),
        created: metadata.created().ok(),
    })
}

/// File metadata information
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub is_file: bool,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
    pub accessed: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
}

/// Check if path exists
pub async fn exists(path: &str) -> bool {
    fs::metadata(path).await.is_ok()
}

/// Get absolute path
pub fn absolute_path(path: &str) -> Result<String> {
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .map(|p| p.to_string_lossy().to_string())
        .context("Failed to get absolute path")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp_path = "/tmp/hcscoder_test.txt";
        let content = "Hello, hcscoder!";

        write_file(temp_path, content).await.unwrap();
        let read_content = read_file(temp_path).await.unwrap();

        assert_eq!(read_content, content);

        // Cleanup
        let _ = delete_file(temp_path).await;
    }

    #[tokio::test]
    async fn test_glob_match() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("*.rs", "lib.rs"));
        assert!(!glob_match("*.rs", "main.ts"));
        assert!(glob_match("*", "anything"));
    }
}
