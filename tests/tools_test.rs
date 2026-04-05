//! Unit tests for hcscoder_tools module

use hcscoder::hcscoder_tools::{bash, filesystem};
use tempfile::tempdir;

#[tokio::test]
async fn test_filesystem_write_and_read_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("note.txt");
    let file_str = file_path.to_string_lossy().to_string();

    filesystem::write_file(&file_str, "hello")
        .await
        .expect("write file");
    let content = filesystem::read_file(&file_str).await.expect("read file");

    assert_eq!(content, "hello");
}

#[tokio::test]
async fn test_filesystem_rejects_null_byte_path() {
    let result = filesystem::read_file("bad\0path").await;
    assert!(result.is_err());
    let err = format!("{}", result.err().unwrap());
    assert!(err.contains("null byte"));
}

#[tokio::test]
async fn test_filesystem_search_files_matches_pattern() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    tokio::fs::write(root.join("a.rs"), "fn main() {}")
        .await
        .expect("write a.rs");
    tokio::fs::write(root.join("b.txt"), "notes")
        .await
        .expect("write b.txt");

    let matches = filesystem::search_files(&root.to_string_lossy(), "*.rs")
        .await
        .expect("search files");

    assert_eq!(matches.len(), 1);
    assert!(matches[0].to_string_lossy().ends_with("a.rs"));
}

#[tokio::test]
async fn test_bash_blocks_dangerous_command_injection_pattern() {
    let result = bash::execute_command("echo ok && rm -rf /").await;
    assert!(result.is_err());

    let err = format!("{}", result.err().unwrap());
    assert!(err.contains("Command blocked"));
}

#[tokio::test]
async fn test_bash_executes_safe_command() {
    let output = bash::execute_command("echo hcscoder")
        .await
        .expect("safe command should run");

    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.contains("hcscoder"));
}

#[test]
fn test_escape_shell_arg_quotes_unsafe_input() {
    let escaped = bash::escape_shell_arg("hello world");
    assert_eq!(escaped, "'hello world'");

    let safe = bash::escape_shell_arg("abc-123_/test");
    assert_eq!(safe, "abc-123_/test");
}
