# hcscoder Examples

This directory contains example usage of hcscoder as a library and CLI.

## Examples Overview

### CLI Examples

#### Basic Chat
```bash
# Start interactive chat
hcscoder

# Chat with specific model
hcscoder --model "anthropic/claude-3-haiku"

# Plain mode (non-interactive)
echo "Explain this code" | hcscoder
```

#### Code Review
```bash
# Review a single file
hcscoder "Review src/main.rs for security issues"

# Review entire directory
hcscoder "Find all TODO comments in src/"

# Generate documentation
hcscoder "Generate Rust doc comments for src/lib.rs"
```

#### File Operations
```bash
# Create new file with AI-generated content
hcscoder "Create a new Rust module for user authentication"

# Refactor code
hcscoder "Refactor src/utils.rs to use Result types"

# Find and fix bugs
hcscoder "Find potential null pointer dereferences in src/"
```

#### Git Integration
```bash
# Generate commit message
hcscoder "Generate a commit message for current changes"

# Review pull request
hcscoder "Review PR #42 for potential issues"

# Create changelog entry
hcscoder "Write a changelog entry for the new features"
```

### Library Usage Examples

#### Example 1: API Key Validation
```rust
use hcscoder::hcscoder_openrouter::auth;

fn main() {
    let api_key = "sk-or-your-key-here";
    
    if auth::validate_api_key(api_key) {
        println!("Valid API key!");
    } else {
        eprintln!("Invalid API key format");
    }
}
```

#### Example 2: Using OpenRouter Client
```rust
use hcscoder::hcscoder_openrouter::client::OpenRouterClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENROUTER_API_KEY")?;
    let client = OpenRouterClient::new(&api_key)?;
    
    // Get available models
    let models = client.list_models().await?;
    println!("Available models: {:?}", models);
    
    Ok(())
}
```

#### Example 3: Tool Execution
```rust
use hcscoder::hcscoder_tools::filesystem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read a file
    let content = filesystem::read_file("./src/main.rs").await?;
    println!("File content: {}", content);
    
    Ok(())
}
```

#### Example 4: Theme System
```rust
use hcscoder::hcscoder_ui::theme::UiTheme;

fn main() {
    // Get theme from environment
    let theme = UiTheme::from_env();
    
    // Use theme colors
    println!("Primary color: {:?}", theme.primary_color());
}
```

## Running Examples

### Prerequisites
- Rust 1.70 or later
- OpenRouter API key (for examples that use the API)

### Run All Examples
```bash
cargo run --example basic_usage
cargo run --example tool_usage
```

### Run Specific Example
```bash
# Basic API usage
cargo run --example basic_usage

# Tool usage
cargo run --example tool_usage
```

### With Custom API Key
```bash
OPENROUTER_API_KEY="sk-or-your-key" cargo run --example basic_usage
```

## Example Output

### Basic Usage Example
```
$ cargo run --example basic_usage
API key validated successfully!
Example complete. Set OPENROUTER_API_KEY to run with actual API.
```

### Tool Usage Example
```
$ cargo run --example tool_usage
Testing path validation for: ./examples/basic_usage.rs
Path validation example complete.
```

## Contributing Examples

We welcome example contributions! When adding examples:

1. Place in `examples/` directory
2. Use descriptive names (e.g., `advanced_chat.rs`)
3. Include comprehensive comments
4. Test with `cargo run --example <name>`
5. Update this README

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## Additional Resources

- [README.md](../README.md) - Main documentation
- [ARCHITECTURE.md](../ARCHITECTURE.md) - Architecture overview
- [API Documentation](https://docs.rs/hcscoder) - Rust API docs
- [FAQ.md](../FAQ.md) - Frequently asked questions

---

**Note:** Examples that require API calls will need a valid OpenRouter API key.
Set the `OPENROUTER_API_KEY` environment variable before running.
