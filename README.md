# hcscoder

**hcscoder** is a privacy-first, **Rust-native** CLI coding assistant powered by [OpenRouter](https://openrouter.ai/). It runs on your machine, stores API keys locally, and does **not** ship telemetry or phone-home analytics.

| Topic | Details |
|--------|---------|
| **Version** | **1.1.0-security** — Security hardening release ([CHANGELOG](CHANGELOG.md)) |
| **Status** | ✅ **Stable Core Release** — Core CLI + OpenRouter + primary tools are production ready |
| **License** | MIT (c) 2026 **hcsmedia** — **attribution is mandatory** when you redistribute (see [LICENSE](LICENSE)). |
| **Contact** | Instagram [@timfromhcs](https://www.instagram.com/timfromhcs/) · Email [hcsmediagroup@gmail.com](mailto:hcsmediagroup@gmail.com) |
| **Repository** | [github.com/hcsmediacorp/hcscoder](https://github.com/hcsmediacorp/hcscoder) |

---

## ✨ Features

- 🔒 **Security First** — Enhanced API key validation, path traversal prevention, command injection protection
- 🚀 **High Performance** — Built with Rust for speed and reliability
- 🎨 **5 UI Themes** — Default, Dracula, Gruvbox, Nord, HighContrast
- 🛠️ **40+ Tools** — Shell execution, filesystem operations, web search, git introspection, and more
- 📡 **SSE Streaming** — Real-time AI responses with robust error handling
- 🌐 **OpenRouter Compliant** — Full attribution headers, model catalog support
- 🧠 **Smart Context** — Conversation memory with token estimation
- 🎯 **Privacy Focused** — Zero telemetry, local API key storage

---

## Capability status (implemented vs experimental)

| Area | Status | Notes |
|------|--------|-------|
| Core CLI (`chat`, `ask`, `run`, `review`, `status`, `init`) | ✅ Stable | Primary user flows are implemented and tested |
| OpenRouter API + SSE streaming | ✅ Stable | Includes attribution headers and streaming parsing |
| Filesystem / Bash / Git / Net tools | ✅ Stable | Security checks and tests included |
| Memory persistence | ✅ Stable (JSON persistence) | Markdown export + JSON roundtrip persistence |
| MCP integration | ⚠️ Experimental | Current module is a scaffold; runtime integration is pending |
| REPL execution | ⚠️ Experimental | Session/history exists; evaluation is currently simulated |
| Notebook cell execution | ⚠️ Experimental | Read/write works; execute requires kernel integration |

Advanced modules marked **experimental** are included for forward compatibility, but should not be treated as production-complete yet.

---

## One-click install (release binaries)

**Prebuilt archives:** On each [release](https://github.com/hcsmediacorp/hcscoder/releases), download **`hcscoder-vX.Y.Z-windows-x86_64.zip`** (exes + `install.ps1`) or **`hcscoder-vX.Y.Z-linux-x86_64.tar.gz`** (ELF binaries + `install.sh`). These are produced by the [Release](https://github.com/hcsmediacorp/hcscoder/actions/workflows/release.yml) workflow.

You can also grab **`hcscoder.exe`** / **`hcscoder-setup.exe`** from [`releases/`](releases/) on `main`, or use **latest** on the releases page.

When release assets are on GitHub, you can use the installers (PATH + config template) via raw scripts:

**Windows (PowerShell, run as Administrator only if you install system-wide):**

```powershell
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned -Force
iwr -useb https://raw.githubusercontent.com/hcsmediacorp/hcscoder/main/install.ps1 -OutFile install.ps1
.\install.ps1 -Local
# Optional: pin a release version (must exist on GitHub Releases):
# .\install.ps1 -Local -Version 1.1.0
```

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/hcsmediacorp/hcscoder/main/install.sh | bash
# User install (no sudo):
curl -fsSL https://raw.githubusercontent.com/hcsmediacorp/hcscoder/main/install.sh | bash -s -- --local
```

Then set your OpenRouter API key and run `hcscoder-setup` or `export OPENROUTER_API_KEY=...` as described below.

**Prefer building from source (100% Rust, always up to date)?** Use [Quick start](#quick-start-build-from-source) — no Node.js required.

---

## What you get

- **OpenRouter** chat completions and **SSE streaming** with line-buffered parsing, comment lines (`:`) ignored, and handling for **mid-stream error payloads** (HTTP 200 with `error` in JSON, per OpenRouter docs).
- **Attribution headers** on every request: `HTTP-Referer`, `X-Title`, `X-OpenRouter-Title`, optional `X-OpenRouter-Categories` (`cli-agent`), plus a clear `User-Agent`.
- **Configurable model**: CLI `--model` → `OPENROUTER_MODEL` → `~/.hcscoder/openrouter_default_model` (from **hcscoder-setup**) → built-in default.
- **Secure API key setup**: `hcscoder-setup` uses a **hidden password-style prompt** (key not echoed) and writes `~/.hcscoder/openrouter_api_key` (Unix: mode `600`).
- **Enhanced Security** (v1.1.0):
  - Advanced API key validation with entropy checking
  - Path traversal prevention for all filesystem operations
  - Command injection protection
  - Comprehensive audit logging
  - System file protection

---

## Prerequisites

1. **Rust toolchain** (stable): install from [rustup.rs](https://rustup.rs/).
2. An **OpenRouter API key** from [openrouter.ai/keys](https://openrouter.ai/keys).

---

## Quick start (build from source)

This is the recommended path for developers and for **100% Rust** workflows.

### Windows (PowerShell)

```powershell
# Install Rust if needed: https://rustup.rs/
rustup default stable

git clone https://github.com/hcsmediacorp/hcscoder.git
cd hcscoder

cargo build --release
.\target\release\hcscoder-setup.exe
.\target\release\hcscoder.exe --contact
.\target\release\hcscoder.exe chat --plain
```

The project builds two Windows executables: **`hcscoder-setup.exe`** (first-time key + model) and **`hcscoder.exe`** (all CLI commands). For interactive chat with full **SSE streaming**, prefer **`hcscoder chat --plain`** (line-oriented I/O). The full-screen **ratatui** UI is optional; you can also force plain mode with **`NO_COLOR=1`** or **`TERM=dumb`**.

Add `target\release` to your PATH if you want to run `hcscoder` from anywhere, or install into Cargo’s bin:

```powershell
cargo install --path . --locked
hcscoder-setup
hcscoder chat
```

### Linux (bash)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

git clone https://github.com/hcsmediacorp/hcscoder.git
cd hcscoder
cargo build --release
./target/release/hcscoder-setup
./target/release/hcscoder --contact
./target/release/hcscoder chat --plain
```

Optional global install:

```bash
cargo install --path . --locked
hcscoder-setup
hcscoder chat
```

### macOS (zsh or bash)

Same as Linux: install Rust via rustup, clone the repo, `cargo build --release`, then run `hcscoder-setup` and `hcscoder`.

**Apple Silicon note:** Rust targets `aarch64-apple-darwin` by default on M1/M2/M3; no extra steps when building locally.

---

## Configuration

| Method | Description |
|--------|-------------|
| **Environment** | `OPENROUTER_API_KEY`, `OPENROUTER_MODEL` |
| **Key file** | `~/.hcscoder/openrouter_api_key` (written by `hcscoder-setup` or `hcscoder init`) |
| **Default model file** | `~/.hcscoder/openrouter_default_model` (one line, model id — written by `hcscoder-setup`) |

Priority for the **model** id: **`--model`** → **`OPENROUTER_MODEL`** → **saved default file** → **catalog default** (`anthropic/claude-3.5-haiku`).

---

## Free vs paid models (OpenRouter)

- **Free** — OpenRouter lists models with **`:free`** in the id (no per-request metered charge on that tier; **rate limits** still apply). Example in hcscoder: `meta-llama/llama-3.1-8b-instruct:free`.
- **Paid / usage-based** — All other routed models bill per your OpenRouter balance and the model’s page pricing (no `:free` suffix).
- **How hcscoder picks a model** — `hcscoder-setup` tier choice writes `~/.hcscoder/openrouter_default_model`. At runtime: **`--model`** overrides **`OPENROUTER_MODEL`**, which overrides that file, which overrides the catalog default (`anthropic/claude-3.5-haiku`).

Curated reference list: `hcscoder_openrouter::models::MODEL_CATALOG`. **Confirm** every id on [openrouter.ai/models](https://openrouter.ai/models) before production.

---

## Commands (overview)

| Command | Purpose |
|---------|---------|
| `hcscoder` | Interactive chat (default) |
| `hcscoder chat` | Same, optional initial prompt |
| `hcscoder ask "<query>"` | Single-shot question |
| `hcscoder review <path>` | Code review helper |
| `hcscoder run "<cmd>"` | Shell command with AI commentary |
| `hcscoder status` | Show config / model source |
| `hcscoder init` | Async API key prompt (TTY) |
| `hcscoder-setup` | Interactive **secure** key + default model |
| `hcscoder --contact` | Attribution / contact info |

---

## Automated installers (optional)

If you use **prebuilt release binaries** from GitHub (when available):

- **Windows:** run `install.ps1` (see script header for `iwr ... | iex` style usage). It downloads the release, places the binary, and **fixes a user PATH** entry. A default TOML template is created under `%APPDATA%\hcscoder\config\`.
- **Linux / macOS:** run `install.sh` with optional `--local` or `--version X.Y.Z`.

Building **from source** with `cargo install --path .` remains the most portable approach when releases are missing or you need the latest `main`.

---

## Development

```bash
cargo check
cargo test --lib
cargo build --release
```

The library crate is built with **`#![deny(warnings)]`** — keep the tree warning-free.

### GitHub Releases (maintainers)

- **Automatic:** Pushing a new annotated tag `v*` on `main` triggers **Release** — it builds Windows + Linux binaries and creates/updates the GitHub Release with archives.
- **Manual (e.g. first time for an old tag):** [Actions → Release](https://github.com/hcsmediacorp/hcscoder/actions/workflows/release.yml) → **Run workflow** → set tag to `v1.0.0` (must already exist) → Run. This publishes the same zip/tar.gz assets without re-tagging.

### Git / VS Code: `Repository not found` on push

If `git push -u origin main` fails with **remote: Repository not found**, the problem is not your local tree—it is the **GitHub side or authentication**:

1. **Create the repository** on GitHub (e.g. [github.com/new](https://github.com/new)) under the account/org you use. Name it `hcscoder` (or your chosen name). **Do not** initialize with a README/license if you are pushing this repo’s existing history for the first time.
2. **Match the remote URL** to that repo:

   ```bash
   git remote set-url origin https://github.com/<OWNER>/hcscoder.git
   ```

3. **Authenticate:** On Windows, sign in when Git Credential Manager prompts you, or use a [personal access token](https://github.com/settings/tokens) with **repo** scope for HTTPS. Alternatively switch to SSH:

   ```bash
   git remote set-url origin git@github.com:<OWNER>/hcscoder.git
   ```

4. Push branch and tag:

   ```bash
   git push -u origin main
   git push origin v1.0.0
   ```

This repo sets **`push.autoSetupRemote = true`** locally so a plain `git push` can set upstream once the remote exists and credentials work.

---

## Architecture (Rust)

| Path | Role |
|------|------|
| `src/main.rs` | CLI entry, `OPENROUTER_MODEL` resolution |
| `src/lib.rs` | Library: buddy, engine, memory, openrouter, planner, tools, UI |
| `src/hcscoder_openrouter/client.rs` | HTTP + SSE, retries, attribution headers |
| `src/hcscoder_openrouter/models.rs` | Model catalog & tiers |
| `src/hcscoder_engine/tool_runtime.rs` | Tool registry → implementations |
| `src/hcscoder_tools/` | Tool implementations |

---

## OpenRouter compliance (summary)

- **POST** `https://openrouter.ai/api/v1/chat/completions` with `Authorization: Bearer …`, `Content-Type: application/json`.
- **Streaming:** `stream: true`; parse SSE `data:` lines; ignore `:` comments; handle `[DONE]`; tolerate **JSON parse errors** on noise; **surface errors** in the JSON payload (`error` and `finish_reason: error`).
- **Attribution:** `HTTP-Referer`, `X-OpenRouter-Title` / `X-Title`, optional `X-OpenRouter-Categories`.

See the official docs: [Chat completion](https://openrouter.ai/docs/api-reference/chat-completion), [Streaming](https://openrouter.ai/docs/api/reference/streaming), [App attribution](https://openrouter.ai/docs/app-attribution).

---

## Legal

MIT License — see [LICENSE](LICENSE). Redistributions must retain copyright and license text and credit **hcsmedia** as required.
