# Contributing to hcscoder

First off, thank you for considering contributing to hcscoder! It's people like you that make hcscoder such a great tool.

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

* **Use a clear and descriptive title**
* **Describe the exact steps to reproduce the problem**
* **Provide specific examples to demonstrate the steps**
* **Describe the behavior you observed and what behavior you expected**
* **Include screenshots if possible**
* **Include environment details** (OS, Rust version, hcscoder version)

Bug reports should be created using the [Bug Report Template](.github/ISSUE_TEMPLATE/bug_report.md).

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

* **Use a clear and descriptive title**
* **Provide a detailed description of the suggested enhancement**
* **Explain why this enhancement would be useful**
* **List some examples of how this enhancement would be used**

Enhancement suggestions should be created using the [Feature Request Template](.github/ISSUE_TEMPLATE/feature_request.md).

### Pull Requests

* Fill in the required template
* Follow the Rust style guidelines (see below)
* Include tests for new functionality
* Update documentation if needed
* Ensure all tests pass and there are no clippy warnings

## Development Setup

### Prerequisites

* Rust 1.70 or later (edition 2021)
* Git
* A text editor (VS Code, Vim, Emacs, etc.)

### Setting Up Your Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/hcscoder.git
   cd hcscoder
   ```

3. Create a branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

4. Install development dependencies:
   ```bash
   cargo install cargo-audit
   cargo install cargo-nextest
   ```

5. Build the project:
   ```bash
   cargo build
   ```

6. Run tests:
   ```bash
   cargo test
   cargo nextest run
   ```

7. Run linters:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo audit
   ```

## Coding Guidelines

### Style Guide

We follow standard Rust style guidelines. Please ensure your code is formatted with `rustfmt`:

```bash
cargo fmt
```

Configuration is in [`rustfmt.toml`](rustfmt.toml).

### Clippy Lints

We use `clippy` for additional linting. All warnings must be fixed before merging:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Configuration is in [`clippy.toml`](clippy.toml).

### Error Handling

* Use `anyhow::Result` for application-level errors
* Use `thiserror` for library-level error types
* Never use `unwrap()` in production code - use `?` operator or proper error handling
* Provide meaningful error messages with context using `.context()`

Example:
```rust
// ❌ Bad
let file = std::fs::File::open(path).unwrap();

// ✅ Good
let file = std::fs::File::open(path)
    .with_context(|| format!("Failed to open file: {}", path.display()))?;
```

### Documentation

* All public items should be documented with rustdoc comments (`///`)
* Include examples in documentation where appropriate
* Keep documentation up-to-date with code changes

Example:
```rust
/// Validates an OpenRouter API key.
///
/// # Arguments
///
/// * `key` - The API key string to validate
///
/// # Returns
///
/// * `bool` - `true` if the key is valid, `false` otherwise
///
/// # Examples
///
/// ```
/// assert!(validate_api_key("sk-or-validkey123"));
/// assert!(!validate_api_key("invalid-key"));
/// ```
pub fn validate_api_key(key: &str) -> bool {
    // ...
}
```

### Testing

* Write unit tests for all public functions
* Write integration tests for critical paths
* Aim for >80% code coverage
* Use `#[cfg(test)]` modules for tests
* Use descriptive test names: `test_function_name_scenario_expected_result`

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key_valid_format() {
        assert!(validate_api_key("sk-or-validkey123456"));
    }

    #[test]
    fn test_validate_api_key_invalid_format() {
        assert!(!validate_api_key("invalid-key"));
    }
}
```

### Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
feat: add new feature X
fix: fix bug in module Y
docs: update README
style: format code
refactor: refactor module Z
test: add tests for feature A
chore: update dependencies
```

## Architecture Overview

hcscoder is organized into several modules:

* `hcscoder_openrouter` - OpenRouter API client and authentication
* `hcscoder_engine` - Core engine and tool coordination
* `hcscoder_tools` - Individual tool implementations (40+ tools)
* `hcscoder_ui` - TUI interface using ratatui
* `hcscoder_memory` - Conversation memory management
* `hcscoder_planner` - Planning logic

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for detailed documentation.

## Security Considerations

When contributing security-related code:

* Follow the principle of least privilege
* Validate all user inputs
* Use secure defaults
* Log security-relevant events
* Never commit secrets or API keys
* Follow responsible disclosure (see [`SECURITY.md`](SECURITY.md))

## Code Review Process

All submissions require review by maintainers:

1. Create a pull request
2. Ensure all CI checks pass
3. Address reviewer feedback
4. Squash commits if necessary
5. Maintain rebase capability until merge

## Questions?

Feel free to open an issue with the "question" label or reach out to the maintainers.

Thank you for contributing to hcscoder! 🎉
