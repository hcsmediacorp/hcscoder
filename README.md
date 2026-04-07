# hcscoder

Rust CLI for OpenRouter-based prompts with optional interactive chat UI.

## Scope of this repository

`hcscoder` currently builds two binaries:

- `hcscoder` (main CLI)
- `hcscoder-setup` (interactive setup helper for API key + default model)

The codebase includes additional modules (`memory`, `planner`, `buddy`, `tools`, `mcp`, `notebook`, `repl`) with unit tests. Some of these modules are scaffolding-oriented and not full external integrations yet.

## Requirements

- Rust stable toolchain
- OpenRouter API key for real model requests (`OPENROUTER_API_KEY`)

## Build and test

Run exactly the same checks used in CI:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

## Install (local)

```bash
cargo install --path . --locked
```

This installs:

- `hcscoder`
- `hcscoder-setup`

## Configuration

### API key

Priority:

1. `--api-key` CLI flag
2. `OPENROUTER_API_KEY` environment variable
3. `~/.hcscoder/openrouter_api_key` file

### Model

Priority:

1. `--model` CLI flag
2. `OPENROUTER_MODEL` environment variable
3. `~/.hcscoder/openrouter_default_model` file
4. Built-in default model from catalog

## CLI quick reference

```text
hcscoder [OPTIONS] [COMMAND]

Commands:
  chat    Interactive coding session
  ask     Run a single query and exit
  run     Execute a shell command with AI assistance
  review  Analyze and improve code
  init    Initialize hcscoder configuration
  buddy   Manage Buddy companions
  memory  View and manage memory
  status  Display system status and configuration
```

Examples:

```bash
hcscoder status
hcscoder ask "Explain ownership in Rust"
hcscoder --plain chat "Say hello in one sentence"
hcscoder --model meta-llama/llama-3.1-8b-instruct:free ask "Give me a short tip"
```

## OpenRouter usage notes

- Requests use OpenRouter chat completions API.
- Streaming is supported in chat flows.
- Provider/rate-limit errors are surfaced as CLI errors.
- Use `:free` models if you only want free-tier catalog entries.

## Repository structure

- `src/main.rs` – main CLI and command dispatch
- `src/bin/hcscoder-setup.rs` – interactive setup binary
- `src/hcscoder_openrouter/` – auth, model catalog, HTTP + streaming client
- `src/hcscoder_ui/` – plain and TUI chat interfaces
- `src/hcscoder_tools/` – tool modules (filesystem, bash, web, etc.)
- `.github/workflows/ci.yml` – lint/test/build pipeline
- `.github/workflows/release.yml` – tag-based release pipeline

## CI workflows

### CI (`.github/workflows/ci.yml`)

On pushes/PRs to `main` and `master`:

- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo build --release`
- `cargo audit`
- extra cross-platform release builds on Windows/macOS

### Release (`.github/workflows/release.yml`)

On `v*` tags or manual dispatch:

- runs the same Rust quality gate checks
- builds release binaries on Linux + Windows
- uploads release archives to GitHub Releases

## Security and sandboxing

Filesystem helpers enforce sandbox-style path validation relative to a sandbox root (`HCSCODER_SANDBOX_ROOT`, default: current working directory). Sensitive system paths are explicitly guarded.

## License

MIT. See `LICENSE`.
