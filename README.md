# hcscoder

**hcscoder** is a Rust-native CLI coding assistant focused on local-first workflows, hardened defaults, and fast terminal UX.  
**Made with ❤️ by hcsmedia.**

[![Release](https://img.shields.io/github/v/release/hcsmediacorp/hcscoder?label=release)](https://github.com/hcsmediacorp/hcscoder/releases/latest)
[![CI](https://github.com/hcsmediacorp/hcscoder/actions/workflows/ci.yml/badge.svg)](https://github.com/hcsmediacorp/hcscoder/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Why hcscoder

- **Privacy-first**: no telemetry, no phone-home analytics.
- **Rust performance**: single native binary, fast startup, low overhead.
- **Hardened tool runtime**: command safety checks, path traversal prevention, key validation.
- **Clean UX**: plain mode and themed TUI mode with streaming responses.

---

## Security & hardening overview

hcscoder ships with secure-by-default behavior in core areas:

1. **API key validation**
   - Prefix and pattern checks for OpenRouter-style keys.
   - Entropy/pattern validation to block weak or malformed inputs.

2. **Filesystem safety**
   - Canonical path validation.
   - Null-byte rejection.
   - Path traversal protections and sensitive-path checks.

3. **Shell safety**
   - High-risk command pattern blocking.
   - Injection pattern detection.
   - Audit logging.
   - **Default 60s command timeout** for shell execution.

4. **Streaming robustness**
   - SSE parsing with error propagation and `[DONE]` handling.

---

## Install

### Option A: Build from source (recommended)

```bash
git clone https://github.com/hcsmediacorp/hcscoder.git
cd hcscoder
cargo build --release --locked
```

Binaries:
- `target/release/hcscoder`
- `target/release/hcscoder-setup`

### Option B: Installer scripts

- Linux/macOS: `install.sh`
- Windows: `install.ps1`

---

## Quick start

1. Configure key/model:

```bash
./target/release/hcscoder-setup
```

2. Run chat:

```bash
./target/release/hcscoder chat
```

3. Useful commands:

```bash
hcscoder --help
hcscoder status
hcscoder ask "review this function"
hcscoder run "cargo test -q"
```

---

## UI/UX

### Plain mode

Use plain mode for CI, remote shells, or minimal terminals:

```bash
hcscoder chat --plain
```

Runtime helper commands inside plain chat:
- `help`
- `status`
- `security`
- `theme` / `themes`
- `clear`
- `quit`

### Themed TUI

Set theme via env:

```bash
export HCSCODER_THEME=dracula
hcscoder chat
```

Available themes:
- `default`
- `dracula`
- `gruvbox`
- `nord`
- `highcontrast`

---

## Prebuilt package generation (local)

To generate a distributable archive from your current machine:

```bash
./scripts/prebuild-packages.sh
```

This script:
- builds release binaries with `--locked`
- creates `dist/hcscoder-v<version>-<target-triple>/`
- outputs either:
  - `.tar.gz` (Linux/macOS)
  - `.zip` (Windows target hosts)

---

## Developer workflow

```bash
cargo fmt --all
cargo test -q
cargo build --release --locked
```

---

## Branding & attribution

- Project name: **hcscoder**
- Brand line: **Made with ❤️ by hcsmedia**
- License: MIT (see [LICENSE](LICENSE))

---

## Contact

- Instagram: [@timfromhcs](https://www.instagram.com/timfromhcs/)
- Email: [hcsmediagroup@gmail.com](mailto:hcsmediagroup@gmail.com)
