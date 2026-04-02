//! hcscoder Messaging Tool
//!
//! Send messages and communicate with users.
//! Zero telemetry, no phone-home logic.

use anyhow::Result;

/// Send a message to the user
pub async fn send_message(content: &str) -> Result<String> {
    // In a real implementation, this would queue/display the message
    println!("{}", content);
    Ok(format!("Message sent: {} chars", content.len()))
}

/// Ask user a question and wait for response
pub async fn ask_user_question(question: &str) -> Result<String> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    println!("{}", question);

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    Ok(response.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        let result = send_message("Hello!").await.unwrap();
        assert!(result.contains("Message sent"));
    }
}
