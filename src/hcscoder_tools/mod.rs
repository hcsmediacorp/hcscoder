//! hcscoder Tool Belt Module
//!
//! Comprehensive tool implementations for file operations, shell execution, LSP, and more.
//! Zero telemetry, no phone-home logic.

pub mod agent;
pub mod bash;
pub mod brief;
pub mod config;
pub mod cron;
pub mod file_edit;
pub mod filesystem;
pub mod git;
pub mod glob;
pub mod grep;
pub mod lsp;
pub mod mcp;
pub mod messaging;
pub mod net;
pub mod notebook;
pub mod plan_mode;
pub mod repl;
pub mod skill;
pub mod sleep;
pub mod synthetic_output;
pub mod sys;
pub mod task;
pub mod team;
pub mod todo;
pub mod tool_search;
pub mod utility;
pub mod web;
pub mod worktree;

use crate::hcscoder_openrouter::client::HcscoderApiClient;
use anyhow::Result;

/// Execute a command with AI assistance
pub async fn execute_with_assistance(
    api_key: Option<String>,
    model: String,
    command: &str,
    _plain: bool,
) -> Result<()> {
    use crate::hcscoder_openrouter::client::{ChatMessage, MessageRole};

    let client = if let Some(key) = api_key {
        HcscoderApiClient::with_config(model, key, None)?
    } else {
        HcscoderApiClient::new(model)?
    };

    // First, get AI analysis of the command
    let analysis_prompt = format!(
        "Analyze this shell command and provide:\n\
         1. What it does\n\
         2. Any safety concerns\n\
         3. Expected output\n\
         4. Suggested improvements if any\n\n\
         Command: {}",
        command
    );

    let messages = vec![
        ChatMessage {
            role: MessageRole::System,
            content: "You are a shell command safety analyzer. Be concise and practical."
                .to_string(),
        },
        ChatMessage {
            role: MessageRole::User,
            content: analysis_prompt,
        },
    ];

    println!("🔍 Analyzing command...\n");
    let analysis = client
        .create_completion(messages, Some(0.2), Some(500))
        .await?;

    if let Some(choice) = analysis.choices.first() {
        println!("{}", choice.message.content);
    }

    // Ask for confirmation before executing
    println!("\n⚠️  Execute this command? (y/N): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Command execution cancelled.");
        return Ok(());
    }

    // Execute the command
    println!("\n▶️  Executing...\n");
    let output = bash::execute_command(command).await?;

    println!("Exit code: {}", output.exit_code);
    if !output.stdout.is_empty() {
        println!("STDOUT:\n{}", output.stdout);
    }
    if !output.stderr.is_empty() {
        eprintln!("STDERR:\n{}", output.stderr);
    }

    Ok(())
}

/// Available tools registry
#[derive(Debug, Clone)]
pub struct HcscoderTool {
    pub name: &'static str,
    pub description: &'static str,
    pub category: ToolCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    Shell,
    FileSystem,
    LSP,
    Web,
    Git,
    Sys,
    Net,
    Utility,
}

/// Get all available tools
pub fn get_all_tools() -> Vec<HcscoderTool> {
    vec![
        // Shell tools
        HcscoderTool {
            name: "bash",
            description: "Execute shell commands safely",
            category: ToolCategory::Shell,
        },
        HcscoderTool {
            name: "bash_background",
            description: "Run long-running commands in background",
            category: ToolCategory::Shell,
        },
        // File system tools
        HcscoderTool {
            name: "read_file",
            description: "Read file contents",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "write_file",
            description: "Write content to file",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "list_directory",
            description: "List directory contents",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "search_files",
            description: "Search for files by pattern",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "create_directory",
            description: "Create new directories",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "delete_file",
            description: "Delete files safely",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "move_file",
            description: "Move or rename files",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "copy_file",
            description: "Copy files",
            category: ToolCategory::FileSystem,
        },
        // LSP tools
        HcscoderTool {
            name: "lsp_diagnostics",
            description: "Get LSP diagnostics for a file",
            category: ToolCategory::LSP,
        },
        HcscoderTool {
            name: "lsp_definitions",
            description: "Find symbol definitions",
            category: ToolCategory::LSP,
        },
        HcscoderTool {
            name: "lsp_references",
            description: "Find symbol references",
            category: ToolCategory::LSP,
        },
        HcscoderTool {
            name: "lsp_hover",
            description: "Get hover information for symbol",
            category: ToolCategory::LSP,
        },
        HcscoderTool {
            name: "lsp_completion",
            description: "Get code completions",
            category: ToolCategory::LSP,
        },
        // Web tools
        HcscoderTool {
            name: "web_search",
            description: "Search the web",
            category: ToolCategory::Web,
        },
        HcscoderTool {
            name: "fetch_url",
            description: "Fetch URL content",
            category: ToolCategory::Web,
        },
        // Git
        HcscoderTool {
            name: "git_status",
            description: "Git working tree status (porcelain)",
            category: ToolCategory::Git,
        },
        HcscoderTool {
            name: "git_diff",
            description: "Git unstaged diff",
            category: ToolCategory::Git,
        },
        HcscoderTool {
            name: "git_log",
            description: "Git recent commits",
            category: ToolCategory::Git,
        },
        HcscoderTool {
            name: "git_branch",
            description: "List or show current branch",
            category: ToolCategory::Git,
        },
        // System
        HcscoderTool {
            name: "system_snapshot",
            description: "Host, OS, memory, CPU, load average",
            category: ToolCategory::Sys,
        },
        HcscoderTool {
            name: "env_dump",
            description: "Filtered environment variables",
            category: ToolCategory::Sys,
        },
        // Network
        HcscoderTool {
            name: "net_resolve",
            description: "Resolve hostname to addresses",
            category: ToolCategory::Net,
        },
        HcscoderTool {
            name: "tcp_probe",
            description: "TCP connect probe with timeout",
            category: ToolCategory::Net,
        },
        // Utility tools
        HcscoderTool {
            name: "think",
            description: "Think step-by-step about a problem",
            category: ToolCategory::Utility,
        },
        HcscoderTool {
            name: "summarize",
            description: "Summarize content",
            category: ToolCategory::Utility,
        },
        HcscoderTool {
            name: "explain",
            description: "Explain code or concepts",
            category: ToolCategory::Utility,
        },
        HcscoderTool {
            name: "grep",
            description: "Search for literal text in files under a directory",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "grep_regex",
            description: "Search with a regex pattern in files under a directory",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "grep_count",
            description: "Count literal matches without returning lines",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "glob",
            description: "List paths matching a glob pattern",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "apply_edit",
            description: "Replace first occurrence of text in a file",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "append_file",
            description: "Append content to an existing file",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "file_metadata",
            description: "File size, modified time, type",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "delete_directory",
            description: "Remove a directory tree",
            category: ToolCategory::FileSystem,
        },
        HcscoderTool {
            name: "list_skills",
            description: "List loaded skill plugins",
            category: ToolCategory::Utility,
        },
        HcscoderTool {
            name: "execute_skill",
            description: "Run a skill by id with JSON args",
            category: ToolCategory::Utility,
        },
        HcscoderTool {
            name: "create_task",
            description: "Create a tracked background task",
            category: ToolCategory::Utility,
        },
        HcscoderTool {
            name: "list_tasks",
            description: "List tracked tasks",
            category: ToolCategory::Utility,
        },
        HcscoderTool {
            name: "find_code_files",
            description: "Find source files (rs, ts, py, …) under a root",
            category: ToolCategory::FileSystem,
        },
    ]
}

/// Get tools by category
pub fn get_tools_by_category(category: ToolCategory) -> Vec<HcscoderTool> {
    get_all_tools()
        .into_iter()
        .filter(|t| t.category == category)
        .collect()
}

/// Display available tools
pub fn display_tools() {
    println!("\n🛠️  hcscoder Tool Belt");
    println!("===================\n");

    for category in [
        ToolCategory::Shell,
        ToolCategory::FileSystem,
        ToolCategory::LSP,
        ToolCategory::Web,
        ToolCategory::Git,
        ToolCategory::Sys,
        ToolCategory::Net,
        ToolCategory::Utility,
    ] {
        let tools = get_tools_by_category(category);
        if tools.is_empty() {
            continue;
        }

        println!("【{:?}】", category);
        println!("{}", "-".repeat(40));

        for tool in tools {
            println!("  • {} - {}", tool.name, tool.description);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_tools() {
        let tools = get_all_tools();
        assert!(tools.len() >= 40);
    }

    #[test]
    fn test_get_tools_by_category() {
        let fs_tools = get_tools_by_category(ToolCategory::FileSystem);
        assert!(fs_tools.len() >= 5);

        let shell_tools = get_tools_by_category(ToolCategory::Shell);
        assert!(!shell_tools.is_empty());
    }
}
