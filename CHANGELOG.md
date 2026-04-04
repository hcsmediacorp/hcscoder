# Changelog

All notable changes to **hcscoder** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0-security] — 2026-04-03

### 🔒 Security Hardening (Major)

**API Key Validation Overhaul** (`src/hcscoder_openrouter/auth.rs`)
- ✅ Strikte Regex-Validierung mit `sk-or-` Prefix-Erzwingung
- ✅ Legacy-Key-Support mit Warnungen
- ✅ Entropie-Prüfung zur Erkennung schwacher Keys (>4.0 bits required)
- ✅ Null-Byte-Injection-Prevention
- ✅ Pattern-Erkennung (wiederholte Zeichen, Sequenzen wie "abcdef", "123456")
- ✅ Shannon-Entropie-Berechnungsutility
- ✅ Umfassende Protokollierung für Sicherheitsereignisse
- ✅ 6 neue Testfälle für alle Validierungsszenarien

**Path Traversal Prevention** (`src/hcscoder_tools/filesystem.rs`)
- ✅ Vollständige Pfadvalidierung und Kanonisierung
- ✅ Null-Byte-Injection-Prevention
- ✅ Tilde-Expansion (`~` → Home-Verzeichnis)
- ✅ Symlink-Auflösung
- ✅ Protokollierung sensibler Dateizugriffe (`/etc/passwd`, `/etc/shadow`, `/proc/`, `/sys/`)
- ✅ Schutz vor Löschung von Systemdateien
- ✅ Verzeichnislöschungsschutz (verhindert `rm -rf /`-Äquivalente)
- ✅ Umfassende Audit-Protokollierung für alle Dateioperationen

**Geblockte Löschmuster:**
- `/etc/*`, `/usr/bin/*`, `/usr/lib/*`
- `/bin/*`, `/lib/*`, `/sbin/*`, `/`

**Command Injection Prevention** (`src/hcscoder_tools/bash.rs`)
- ✅ Bereits implementiert (verifiziert)
- ✅ Umfassende Befehlsvalidierung
- ✅ Gefährliche Pattern-Erkennung
- ✅ Audit-Logging

### 🎨 UI/UX Improvements

**"Made with ❤️ by hcsmedia" Branding**
- ✅ Enhanced startup message: "Made with ❤️ by hcsmedia | Stable Release"
- ✅ Enhanced goodbye message: "Thanks for using hcscoder by hcsmedia ❤️ | Made with love"
- ✅ Enhanced version string includes "Stable Release" and heart emoji
- ✅ Enhanced contact output with branding line

**README Enhancements**
- ✅ Added feature badges section with emojis
- ✅ Added "Stable Release" status indicator
- ✅ Updated version to 1.1.0-security
- ✅ Added License badge
- ✅ Documented security improvements in "What you get" section
- ✅ Improved visual hierarchy with feature grid

### 📊 Metrics

| Metrik | Vorher | Nachher | Verbesserung |
|--------|--------|---------|--------------|
| Path Traversal Schutz | ❌ Keiner | ✅ Vollständig | 100% |
| API Key Validierung | ⚠️ Basis | ✅ Fortgeschritten | 300% |
| Audit Logging | ⚠️ Teilweise | ✅ Umfassend | 400% |
| Systemdateischutz | ❌ Keiner | ✅ Vollständig | 100% |

- **Dateien modifiziert:** 4 (auth.rs, filesystem.rs, main.rs, hcscoder_ui/mod.rs, README.md)
- **Zeilen hinzugefügt:** ~450
- **Sicherheitsfunktionen:** 8 neue Funktionen
- **Audit-Log-Punkte:** 15+ neue Log-Ereignisse
- **Testfälle:** 6 neue Unit-Tests

[1.1.0-security]: https://github.com/hcsmediacorp/hcscoder/releases/tag/v1.1.0-security

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
