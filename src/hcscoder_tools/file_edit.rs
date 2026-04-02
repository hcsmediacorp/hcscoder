//! hcscoder File Edit Tool
//!
//! Advanced file editing with diff-based modifications.
//! Zero telemetry, no phone-home logic.

use crate::hcscoder_tools::filesystem;
use anyhow::Result;

/// Apply a diff-based edit to a file
pub async fn apply_edit(path: &str, old_string: &str, new_string: &str) -> Result<String> {
    let content = filesystem::read_file(path).await?;

    // Find and replace the first occurrence
    let updated = content.replacen(old_string, new_string, 1);

    if updated == content {
        return Err(anyhow::anyhow!(
            "Could not find the specified text to replace in file: {}",
            path
        ));
    }

    filesystem::write_file(path, &updated).await?;

    Ok(format!("Successfully edited file: {}", path))
}

/// Apply multiple edits to a file atomically
pub async fn apply_edits(path: &str, edits: Vec<(String, String)>) -> Result<String> {
    let mut content = filesystem::read_file(path).await?;
    let n_edits = edits.len();

    for (old_str, new_str) in edits {
        let updated = content.replacen(&old_str, &new_str, 1);
        if updated == content {
            return Err(anyhow::anyhow!(
                "Could not find text to replace: \"{}...\"",
                old_str.chars().take(50).collect::<String>()
            ));
        }
        content = updated;
    }

    filesystem::write_file(path, &content).await?;

    Ok(format!("Applied {} edits to file: {}", n_edits, path))
}

/// Insert content at a specific line number
pub async fn insert_at_line(path: &str, line_number: usize, content: &str) -> Result<String> {
    let file_content = filesystem::read_file(path).await?;
    let mut lines: Vec<&str> = file_content.lines().collect();

    if line_number > lines.len() + 1 {
        return Err(anyhow::anyhow!(
            "Line number {} exceeds file length ({} lines)",
            line_number,
            lines.len()
        ));
    }

    lines.insert(line_number - 1, content);
    let updated = lines.join("\n");

    filesystem::write_file(path, &updated).await?;

    Ok(format!(
        "Inserted content at line {} in {}",
        line_number, path
    ))
}

/// Delete lines from a file
pub async fn delete_lines(path: &str, start_line: usize, end_line: usize) -> Result<String> {
    let file_content = filesystem::read_file(path).await?;
    let lines: Vec<&str> = file_content.lines().collect();

    if start_line > lines.len() || end_line < start_line {
        return Err(anyhow::anyhow!(
            "Invalid line range: {}-{} (file has {} lines)",
            start_line,
            end_line,
            lines.len()
        ));
    }

    let updated: Vec<&str> = lines[..start_line - 1]
        .iter()
        .chain(lines[end_line..].iter())
        .copied()
        .collect();

    let result = updated.join("\n");
    filesystem::write_file(path, &result).await?;

    Ok(format!(
        "Deleted lines {}-{} from {}",
        start_line, end_line, path
    ))
}

/// Replace a range of lines with new content
pub async fn replace_lines(
    path: &str,
    start_line: usize,
    end_line: usize,
    new_content: &str,
) -> Result<String> {
    let file_content = filesystem::read_file(path).await?;
    let lines: Vec<&str> = file_content.lines().collect();

    if start_line > lines.len() || end_line < start_line {
        return Err(anyhow::anyhow!(
            "Invalid line range: {}-{} (file has {} lines)",
            start_line,
            end_line,
            lines.len()
        ));
    }

    let mut updated: Vec<&str> = lines[..start_line - 1].to_vec();
    updated.push(new_content);
    updated.extend_from_slice(&lines[end_line..]);

    let result = updated.join("\n");
    filesystem::write_file(path, &result).await?;

    Ok(format!(
        "Replaced lines {}-{} in {}",
        start_line, end_line, path
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_apply_edit() {
        let temp_path = "/tmp/hcscoder_edit_test.txt";
        filesystem::write_file(temp_path, "Hello World")
            .await
            .unwrap();

        let result = apply_edit(temp_path, "World", "hcscoder").await.unwrap();
        assert!(result.contains("Successfully edited"));

        let content = filesystem::read_file(temp_path).await.unwrap();
        assert_eq!(content, "Hello hcscoder");

        let _ = filesystem::delete_file(temp_path).await;
    }
}
