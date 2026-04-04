# hcscoder FAQ

Frequently Asked Questions about hcscoder.

## General

### What is hcscoder?

hcscoder is a privacy-first, Rust-native CLI coding assistant that uses OpenRouter to access various AI models. It provides 40+ autonomous tools for code generation, file manipulation, git operations, and more.

### How does hcscoder differ from other AI coding assistants?

- **Privacy-First**: No data sent to hcscoder servers - only to OpenRouter
- **Open Source**: Full transparency, MIT licensed
- **CLI-Native**: Designed for terminal workflows
- **Tool-Based**: 40+ autonomous tools for specific tasks
- **Cross-Platform**: Works on Windows, Linux, macOS, and Termux

### Is hcscoder free to use?

hcscoder itself is free (MIT license), but you need an OpenRouter API key which charges based on model usage. See [OpenRouter Pricing](https://openrouter.ai/pricing) for details.

## Installation

### How do I install hcscoder?

**Windows:**
```powershell
irm https://raw.githubusercontent.com/hcsmediacorp/hcscoder/main/install.ps1 | iex
```

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/hcsmediacorp/hcscoder/main/install.sh | bash
```

See README.md for detailed installation instructions.

### Can I build from source?

Yes! Requirements:
- Rust 1.70 or later
- Git

```bash
git clone https://github.com/hcsmediacorp/hcscoder.git
cd hcscoder
cargo build --release
```

### Does hcscoder work on Termux/Android?

Yes! hcscoder has explicit Termux support with automatic detection and plain mode fallback.

## Configuration

### How do I configure my API key?

Set the `OPENROUTER_API_KEY` environment variable:

**Linux/macOS:**
```bash
export OPENROUTER_API_KEY="sk-or-your-key-here"
```

**Windows PowerShell:**
```powershell
$env:OPENROUTER_API_KEY = "sk-or-your-key-here"
```

Or create `~/.hcscoder/api_key` file (permissions: 600).

### How do I change the UI theme?

Set the `HCSCODER_THEME` environment variable:

```bash
export HCSCODER_THEME=dracula  # Options: default, dracula, gruvbox, nord, highcontrast
```

### Can I use a custom config file?

Config file support is planned for v2.0. Currently, configuration is via environment variables only. See PERFECTION_ROADMAP.md for timeline.

## Usage

### What models does hcscoder support?

hcscoder supports all models available through OpenRouter, including:
- Claude 3 family
- GPT-4 family
- Llama family
- Mistral family
- And many more!

Use `hcscoder models` to see available models.

### How do I run hcscoder in non-interactive mode?

Use plain mode automatically when piping input or redirecting output:

```bash
echo "Review this code" | hcscoder
hcscoder < input.txt > output.txt
```

### Can I use hcscoder in CI/CD pipelines?

Yes! Plain mode is designed for scripting:

```bash
#!/bin/bash
export OPENROUTER_API_KEY="$SECRET_KEY"
hcscoder "Review src/main.rs for security issues" >> review.txt
```

## Security

### Is my code sent to hcscoder servers?

**No.** hcscoder has no servers. Your code is only sent to:
1. OpenRouter API (for AI processing)
2. Local filesystem (for file operations)

### How is my API key stored?

- Environment variable: In memory only
- File storage: `~/.hcscoder/api_key` with 600 permissions (Unix)
- Validation: Multi-layer validation prevents compromised keys

### What security features does hcscoder have?

- API key validation (format, entropy, pattern detection)
- Path traversal prevention
- Command injection protection
- System file protection
- Audit logging
- Secure file permissions

See SECURITY.md and ARCHITECTURE.md for details.

### Can hcscoder modify system files?

No. System paths are blocked by default:
- `/etc/*`, `/usr/bin/*`, `/bin/*` (Unix)
- `C:\Windows\*`, `C:\Program Files\*` (Windows)

See `hcscoder_tools/filesystem.rs` for blocked patterns.

## Troubleshooting

### "Authentication failed" error

1. Check API key format: Should start with `sk-or-`
2. Verify key is not expired: Test at openrouter.ai
3. Ensure environment variable is set correctly
4. Check for trailing whitespace in key

### "Path not found" errors

1. Use absolute paths when possible
2. Check file permissions
3. Ensure path doesn't contain blocked patterns
4. Try tilde expansion: `~/path/to/file`

### TUI not displaying correctly

1. Ensure terminal supports UTF-8
2. Try different themes: `HCSCODER_THEME=highcontrast`
3. Update terminal emulator
4. Check `TERM` environment variable

### Slow response times

1. Check internet connection
2. Try different models (some are faster)
3. Reduce context size if possible
4. Check OpenRouter status

### Memory usage concerns

hcscoder is designed to be lightweight:
- Binary size: ~15MB
- RAM usage: Typically <100MB
- No background processes

Run `hcscoder --version` to verify you're using the latest optimized build.

## Development

### How can I contribute?

See CONTRIBUTING.md for:
- Development setup
- Coding guidelines
- Pull request process
- Issue templates

### Where can I find API documentation?

Generate local docs:
```bash
cargo doc --open
```

Or read ARCHITECTURE.md for module overview.

### How do I report a bug?

Use the [Bug Report Template](.github/ISSUE_TEMPLATE/bug_report.md) with:
- Steps to reproduce
- Expected vs actual behavior
- Environment details
- Logs (run with `RUST_LOG=debug`)

### How do I request a feature?

Use the [Feature Request Template](.github/ISSUE_TEMPLATE/feature_request.md) with:
- Problem description
- Proposed solution
- Use cases
- Willingness to contribute

## Licensing

### What license is hcscoder under?

MIT License - see LICENSE file for details.

### Can I use hcscoder commercially?

Yes! MIT license allows commercial use, modification, and distribution.

### Do I need to attribute hcscoder?

Attribution is appreciated but not required. If you use hcscoder in a project, please mention it in your documentation.

## Contact

### How can I reach the maintainers?

- GitHub Issues: For bugs and feature requests
- Email: hcsmediagroup@gmail.com
- Website: https://github.com/hcsmediacorp/hcscoder

### Is there a community chat?

Community chat is planned for future releases. Watch the repository for updates.

## More Resources

- [README.md](README.md) - Getting started guide
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [SECURITY.md](SECURITY.md) - Security policy
- [PERFECTION_ROADMAP.md](PERFECTION_ROADMAP.md) - Future plans

---

**Last Updated:** 2024  
**Version:** 1.1.0-security
