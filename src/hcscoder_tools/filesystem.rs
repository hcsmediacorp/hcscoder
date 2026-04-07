//! hcscoder Filesystem Tool
//!
//! Safe file and directory operations with security hardening.
//! Zero telemetry, no phone-home logic.

use anyhow::{Context, Result};
use std::path::{Component, Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

const SANDBOX_ROOT_ENV: &str = "HCSCODER_SANDBOX_ROOT";
const ALLOW_SENSITIVE_READS_ENV: &str = "HCSCODER_ALLOW_SENSITIVE_READS";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PathAccessMode {
    ReadOnly,
    ReadWrite,
}

/// Get the configured sandbox root for filesystem confinement.
/// Defaults to current working directory, with optional explicit env override.
fn get_sandbox_root() -> Result<PathBuf> {
    if let Ok(raw) = std::env::var(SANDBOX_ROOT_ENV) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            anyhow::bail!("{} is set but empty", SANDBOX_ROOT_ENV);
        }

        let root = PathBuf::from(trimmed);
        let absolute = if root.is_absolute() {
            root
        } else {
            std::env::current_dir()
                .context("Failed to get current directory")?
                .join(root)
        };

        return absolute
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize {}", SANDBOX_ROOT_ENV));
    }

    std::env::current_dir()
        .context("Failed to get current directory")?
        .canonicalize()
        .context("Failed to canonicalize current working directory")
}

fn allows_sensitive_reads() -> bool {
    matches!(std::env::var(ALLOW_SENSITIVE_READS_ENV), Ok(v) if v == "1" || v.eq_ignore_ascii_case("true"))
}

#[cfg(unix)]
fn is_sensitive_path(path: &Path) -> bool {
    let mut components = path.components();
    if components.next() != Some(Component::RootDir) {
        return false;
    }

    matches!(
        components.next(),
        Some(Component::Normal(part))
            if part == "etc"
                || part == "proc"
                || part == "sys"
                || part == "usr"
                || part == "bin"
                || part == "lib"
                || part == "lib64"
                || part == "sbin"
    )
}

#[cfg(windows)]
fn is_sensitive_path(path: &Path) -> bool {
    let path_lc = path.to_string_lossy().to_ascii_lowercase();
    path_lc.starts_with(r"c:\\windows")
        || path_lc.starts_with(r"c:\\windows\\system32")
        || path_lc.starts_with(r"c:\\program files")
        || path_lc.starts_with(r"c:\\programdata")
        || path_lc.starts_with(r"c:\\users\\public")
}

fn resolve_candidate_path(path: &str) -> Result<PathBuf> {
    if path.contains('\0') {
        anyhow::bail!("Path contains null byte injection attempt");
    }

    let expanded = shellexpand::tilde(path);
    let path_buf = PathBuf::from(expanded.as_ref());
    if path_buf.is_absolute() {
        Ok(path_buf)
    } else {
        Ok(get_sandbox_root()?.join(path_buf))
    }
}

/// Validate and canonicalize a path to prevent traversal attacks and sandbox escape.
fn validate_and_canonicalize_path(path: &str, mode: PathAccessMode) -> Result<PathBuf> {
    let sandbox_root = get_sandbox_root()?;
    let candidate = resolve_candidate_path(path)?;

    let canonical = if candidate.exists() {
        candidate
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", path))?
    } else {
        let mut existing_ancestor = candidate.as_path();
        let mut suffix = Vec::new();

        while !existing_ancestor.exists() {
            let name = existing_ancestor
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid path"))?
                .to_os_string();
            suffix.push(name);
            existing_ancestor = existing_ancestor
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        }

        let mut rebuilt = existing_ancestor.canonicalize().with_context(|| {
            format!(
                "Failed to canonicalize parent directory: {}",
                existing_ancestor.display()
            )
        })?;
        for part in suffix.iter().rev() {
            rebuilt.push(part);
        }
        rebuilt
    };

    if !canonical.starts_with(&sandbox_root) {
        anyhow::bail!(
            "Path '{}' escapes sandbox root '{}'",
            canonical.display(),
            sandbox_root.display()
        );
    }

    if is_sensitive_path(&canonical) {
        let allowed = mode == PathAccessMode::ReadOnly && allows_sensitive_reads();
        if !allowed {
            anyhow::bail!(
                "Access to sensitive system path '{}' is blocked",
                canonical.display()
            );
        }
    }

    Ok(canonical)
}

pub async fn read_file(path: &str) -> Result<String> {
    let safe_path = validate_and_canonicalize_path(path, PathAccessMode::ReadOnly)?;
    fs::read_to_string(&safe_path)
        .await
        .context(format!("Failed to read file: {}", path))
}

pub async fn write_file(path: &str, content: &str) -> Result<()> {
    let safe_path = validate_and_canonicalize_path(path, PathAccessMode::ReadWrite)?;
    if let Some(parent) = safe_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }
    }

    let mut file = fs::File::create(&safe_path)
        .await
        .context(format!("Failed to create file: {}", path))?;
    file.write_all(content.as_bytes())
        .await
        .context(format!("Failed to write to file: {}", path))
}

pub async fn append_file(path: &str, content: &str) -> Result<()> {
    let safe_path = validate_and_canonicalize_path(path, PathAccessMode::ReadWrite)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&safe_path)
        .await
        .context(format!("Failed to open file for appending: {}", path))?;

    file.write_all(content.as_bytes())
        .await
        .context(format!("Failed to append to file: {}", path))
}

pub async fn list_directory(path: &str) -> Result<Vec<PathEntry>> {
    let safe_path = validate_and_canonicalize_path(path, PathAccessMode::ReadOnly)?;
    let mut entries = Vec::new();
    let mut dir = fs::read_dir(&safe_path)
        .await
        .context(format!("Failed to read directory: {}", path))?;

    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).await.ok();

        let entry_type = if let Some(meta) = &metadata {
            if meta.file_type().is_symlink() {
                EntryType::Symlink
            } else if meta.is_dir() {
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

pub async fn search_files(root: &str, pattern: &str) -> Result<Vec<PathBuf>> {
    let safe_root = validate_and_canonicalize_path(root, PathAccessMode::ReadOnly)?;
    let pattern = pattern.to_string();
    tokio::task::spawn_blocking(move || {
        let mut results = Vec::new();
        use walkdir::WalkDir;
        for entry in WalkDir::new(&safe_root).into_iter().filter_entry(|e| {
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
                if glob_match(&pattern, &name.to_string_lossy()) {
                    results.push(path.to_path_buf());
                }
            }
        }
        Ok::<_, anyhow::Error>(results)
    })
    .await
    .context("search_files join")?
}

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

pub async fn create_directory(path: &str) -> Result<()> {
    let safe_path = validate_and_canonicalize_path(path, PathAccessMode::ReadWrite)?;
    fs::create_dir_all(&safe_path)
        .await
        .context(format!("Failed to create directory: {}", path))
}

pub async fn delete_file(path: &str) -> Result<()> {
    let safe_path = validate_and_canonicalize_path(path, PathAccessMode::ReadWrite)?;
    fs::remove_file(&safe_path)
        .await
        .context(format!("Failed to delete file: {}", path))
}

pub async fn delete_directory(path: &str) -> Result<()> {
    let safe_path = validate_and_canonicalize_path(path, PathAccessMode::ReadWrite)?;
    fs::remove_dir_all(&safe_path)
        .await
        .context(format!("Failed to delete directory: {}", path))
}

pub async fn move_path(from: &str, to: &str) -> Result<()> {
    let safe_from = validate_and_canonicalize_path(from, PathAccessMode::ReadWrite)?;
    let safe_to = validate_and_canonicalize_path(to, PathAccessMode::ReadWrite)?;
    fs::rename(&safe_from, &safe_to)
        .await
        .context(format!("Failed to move {} to {}", from, to))
}

pub async fn copy_file(from: &str, to: &str) -> Result<()> {
    let safe_from = validate_and_canonicalize_path(from, PathAccessMode::ReadOnly)?;
    let safe_to = validate_and_canonicalize_path(to, PathAccessMode::ReadWrite)?;
    if let Some(parent) = safe_to.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }
    }

    fs::copy(&safe_from, &safe_to)
        .await
        .context(format!("Failed to copy {} to {}", from, to))?;
    Ok(())
}

pub async fn get_metadata(path: &str) -> Result<FileMetadata> {
    let safe_path = validate_and_canonicalize_path(path, PathAccessMode::ReadOnly)?;
    let metadata = fs::metadata(&safe_path)
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

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub is_file: bool,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
    pub accessed: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
}

pub async fn exists(path: &str) -> bool {
    validate_and_canonicalize_path(path, PathAccessMode::ReadOnly)
        .ok()
        .and_then(|p| p.try_exists().ok())
        .unwrap_or(false)
}

pub fn absolute_path(path: &str) -> Result<String> {
    Ok(
        validate_and_canonicalize_path(path, PathAccessMode::ReadOnly)?
            .to_string_lossy()
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn safe_in_sandbox_read_write() {
        let base = std::env::current_dir().unwrap().join("target/test-fs-safe");
        tokio::fs::create_dir_all(&base).await.unwrap();
        let rel = "target/test-fs-safe/notes/test.txt";
        write_file(rel, "hello").await.unwrap();
        let read = read_file(rel).await.unwrap();
        assert_eq!(read, "hello");
    }

    #[tokio::test]
    async fn rejects_parent_traversal() {
        let err = read_file("../outside.txt").await.unwrap_err().to_string();
        assert!(err.contains("escapes sandbox root"));
    }

    #[tokio::test]
    async fn rejects_absolute_outside_sandbox() {
        let outside = if cfg!(windows) {
            r"C:\\Windows\\win.ini"
        } else {
            "/tmp"
        };
        let err = list_directory(outside).await.unwrap_err().to_string();
        assert!(err.contains("escapes sandbox root"));
    }

    #[test]
    fn detects_sensitive_system_path() {
        if cfg!(windows) {
            return;
        }

        assert!(is_sensitive_path(Path::new("/etc/passwd")));
        assert!(!is_sensitive_path(Path::new("/tmp/not-sensitive")));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn rejects_symlink_escape() {
        let base = std::env::current_dir()
            .unwrap()
            .join("target/test-fs-symlink");
        let outside = std::env::current_dir()
            .unwrap()
            .join("target/test-fs-outside");
        tokio::fs::create_dir_all(&base).await.unwrap();
        let outside_tmp = tempfile::tempdir().unwrap();
        tokio::fs::create_dir_all(&outside).await.unwrap();
        let outside_file = outside_tmp.path().join("secret.txt");
        tokio::fs::write(&outside_file, "secret").await.unwrap();

        let link_path = base.join("leak.txt");
        let _ = std::fs::remove_file(&link_path);
        std::os::unix::fs::symlink(&outside_file, &link_path).unwrap();

        let err = read_file("target/test-fs-symlink/leak.txt")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("escapes sandbox root"));
    }

    #[tokio::test]
    async fn test_glob_match() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "main.ts"));
        assert!(glob_match("*", "anything"));
    }
}
