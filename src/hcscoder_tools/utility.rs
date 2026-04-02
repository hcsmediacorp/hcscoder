//! Local reasoning helpers (no remote LLM). Used when tools are invoked without an API client.

/// Structured placeholder for chain-of-thought style output.
pub fn think(text: &str) -> String {
    format!(
        "Thought:\n{}\n\nNext: break the problem into smaller steps and verify assumptions.",
        text.trim()
    )
}

/// Rough summarization by truncation with length hint.
pub fn summarize(text: &str) -> String {
    let t = text.trim();
    const MAX: usize = 800;
    if t.len() <= MAX {
        t.to_string()
    } else {
        format!("{}…\n[truncated from {} chars]", &t[..MAX], t.len())
    }
}

/// Short explanatory stub (full teaching requires the chat model).
pub fn explain(subject: &str) -> String {
    format!(
        "Topic: {}\nProvide definitions, examples, and common pitfalls when using the full assistant.",
        subject.trim()
    )
}
