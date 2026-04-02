# Changelog

All notable changes to **hcscoder** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] — 2026-04-02

First major release on GitHub: **hcscoder** as a **Rust-native** OpenRouter CLI.

### The massive 2026 migration

- **Complete rewrite from TypeScript to high-performance Rust** — smaller attack surface, a single `cargo build --release` artifact, and library-first design (`hcscoder` crate + `hcscoder` / `hcscoder-setup` binaries).
- **40+ autonomous tools** in the engine tool belt: shell execution, filesystem (read/write/grep/glob/LSP-adjacent helpers), web search and fetch, git introspection, system snapshot, networking probes, and utility hooks (summarize, skills, background tasks, and more).
- **Advanced OpenRouter API compliance (2026)**:
  - **SSE** streaming with robust line buffering, `[DONE]` handling, and tolerance for comment / noise lines.
  - **Mid-stream and billing errors** surfaced from JSON payloads (including contexts aligned with **402** / insufficient credits behavior as returned by the provider).
  - **App attribution**: `HTTP-Referer`, `X-Title` / `X-OpenRouter-Title`, and **CLI-focused** `X-OpenRouter-Categories` (e.g. `cli-agent`) where applicable.
- **New interactive setup** — `hcscoder-setup` with **secure key entry** (hidden prompt), optional default model tier, and persisted config under `~/.hcscoder/` (including default model file resolution: `--model` → `OPENROUTER_MODEL` → saved default → catalog default).

### Documentation & distribution

- **README** — one-click **Windows** (`install.ps1`) and **Linux/macOS** (`install.sh`) flows; build-from-source instructions for all platforms.
- **MIT License** — copyright **hcsmedia**; attribution required on redistribution (see `LICENSE`).

[1.0.0]: https://github.com/hcsmediacorp/hcscoder/releases/tag/v1.0.0
