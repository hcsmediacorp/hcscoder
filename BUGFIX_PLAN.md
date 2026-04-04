# 🔧 hcscoder Bugfix & Verbesserungsplan - Vollständige Dokumentation

## ✅ Bereits implementierte Fixes

### 1. TUI Streaming Bug behoben
**Problem:** Der ursprüngliche Code spawned einen Task, der Client und Messages sofort dropped hat.
**Lösung:** 
- Komplette Neustrukturierung des TUI-Main-Loops mit `tokio::select!`
- Implementierung eines Kanalsystems (`mpsc::channel`) für die Kommunikation zwischen Streaming-Task und UI
- Korrekte Handhabung von Streaming-Chunks über `TuiMessage` Enum
- State-Management mit `TuiState` Struct

### 2. Termux/Android Support hinzugefügt
**Problem:** Termux wurde nicht erkannt, TUI funktionierte nicht richtig.
**Lösung:**
```rust
// Check for Termux (Android) - often has limited TUI support
if std::env::var("PREFIX")
    .map(|p| p.contains("com.termux"))
    .unwrap_or(false)
{
    return true; // Fallback zu Plain Mode
}
```

### 3. Modernes IsTerminal statt deprecated atty Crate
**Problem:** Verwendung der deprecated `atty` Crate.
**Lösung:** Migration zu `std::io::IsTerminal` (Rust Standard Library seit 1.70).

### 4. Theme-System implementiert
**Features:**
- 5 Themes: Default, Dracula, Gruvbox, Nord, HighContrast
- Environment Variable `HCSCODER_THEME` zur Steuerung
- Farbkonfiguration für Borders, Primary, Secondary, Error Colors

### 5. Verbesserte Plain Chat UX
**Neue Commands:**
- `theme` - Zeigt aktuelles Theme
- `themes` - Listet verfügbare Themes auf
- `model` - Zeigt aktuelles Model
- `status` - Verbindungsstatus

**Verbesserungen:**
- ASCII Logo beim Start
- "Made with ❤️ by hcsmedia" Branding
- Token-Usage-Anzeige nach jeder Antwort
- Emoji-Indikatoren (👤, 🤖, ❌, ✅, 🎨, 📖)

### 6. Scroll-Funktionalität im TUI
**Implementiert:**
- Up/Down Pfeile zum Scrollen durch Nachrichten
- PageUp/PageDown für schnelles Scrollen (10 Zeilen)
- Visueller Scrollbar mit ↑↓ Indikatoren
- Auto-Scroll bei neuen Nachrichten

### 7. Cursor-Indikator im Input
**Feature:** Blinkender Cursor `▌` im Eingabefeld wenn nicht streaming.

### 8. Verbesserte Fehleranzeige
- Status Messages mit ❌ Icon
- Theming-konforme Fehlerfarben
- Inline-Fehlermeldungen im Chat

---

## 📋 Ausstehende Bugs & Fixes (Priorisiert)

### KRITISCH (P0 - Sofort fixen)

#### 1. API Client: Exponentielles Backoff mit Jitter
**Datei:** `src/hcscoder_openrouter/client.rs`
**Aktuell:** Lineares Backoff (`200 * 2^attempt`)
**Sollte sein:** Exponentielles Backoff mit Jitter

```rust
// Neu implementieren in create_completion und create_stream
let base_delay = 100u64 << attempt.min(5); // Max 3.2s base
let jitter = rand::random::<u64>() % base_delay;
let delay = Duration::from_millis(base_delay + jitter);
```

#### 2. Shell Injection Protection
**Datei:** `src/hcscoder_tools/bash.rs`
**Problem:** User-Input wird direkt an Shell übergeben
**Fix:** Whitelist-Validierung oder Escaping

```rust
// Dangerous:
Command::new("sh").arg("-c").arg(user_input)

// Safe:
if !is_safe_command(&user_input) {
    return Err(anyhow!("Unsafe command detected"));
}
```

#### 3. API Key Validation stärken
**Datei:** `src/hcscoder_openrouter/auth.rs`
**Aktuell:** Nur Längenprüfung (≥20 Zeichen)
**Sollte:** Format-Validierung (Base64, alphanumerisch)

```rust
fn validate_api_key(key: &str) -> Result<()> {
    if key.len() < 20 {
        bail!("API key too short");
    }
    if !key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        bail!("API key contains invalid characters");
    }
    Ok(())
}
```

### HOCH (P1 - Diese Woche)

#### 4. Windows ACL für API Keys
**Datei:** `src/hcscoder_openrouter/auth.rs`
**Problem:** Windows bekommt keine expliziten ACLs gesetzt
**Fix:** Verwende `windows-acl` Crate oder `winapi`

```rust
#[cfg(target_os = "windows")]
fn set_secure_permissions(path: &Path) -> Result<()> {
    use std::os::windows::fs::PermissionsExt;
    // Set owner-only permissions
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}
```

#### 5. SSE Parser: Leere Data Lines korrekt behandeln
**Datei:** `src/hcscoder_openrouter/client.rs:352`
**Problem:** Leere data: Lines werden ignoriert, könnten aber Keep-Alive sein
**Fix:** RFC 8895 konforme Behandlung

```rust
if let Some(data) = line.strip_prefix("data: ") {
    let data = data.trim_end(); // Trim nur rechts!
    if data.is_empty() {
        // Keep-alive message per SSE spec
        continue;
    }
    // ...
}
```

#### 6. Token Usage in TUI anzeigen
**Datei:** `src/hcscoder_ui/mod.rs`
**Problem:** Token-Usage wird nur im Plain Chat angezeigt
**Fix:** In TUI als Footer oder Status-Bar

```rust
// In TuiState hinzufügen:
token_usage: Option<HcscoderUsage>,

// In ui() rendern als kleine Zeile unten
```

### MITTEL (P2 - Nächste 2 Wochen)

#### 7. Model-Fallback testen
**Datei:** `src/hcscoder_openrouter/client.rs`
**Problem:** Fallback-Logik wurde nie getestet
**Fix:** Integration Test schreiben

```rust
#[test]
fn test_fallback_model_routing() {
    let client = HcscoderApiClient::new("primary-model".to_string())
        .with_fallback_models(vec!["fallback-1".to_string(), "fallback-2".to_string()]);
    
    // Mock HTTP response mit 503 für primary
    // Verify dass fallback-1 versucht wird
}
```

#### 8. Memory Load Parser verbessern
**Datei:** `src/hcscoder_memory/mod.rs`
**Problem:** Parser ist vereinfacht ("just acknowledge file exists")
**Fix:** Echten Markdown-Parser implementieren

```rust
use pulldown_cmark::{Parser, Event};

fn parse_memory_file(content: &str) -> Vec<MemoryEntry> {
    let parser = Parser::new(content);
    // Parse headings as categories, paragraphs as entries
}
```

#### 9. AutoDream Implementierung
**Datei:** `src/hcscoder_memory/mod.rs`
**Status:** Placeholder
**Fix:** Echte Konsolidierungslogik

```rust
async fn consolidate_memories(entries: &[MemoryEntry]) -> Vec<MemoryEntry> {
    // Group by category
    // Merge similar entries
    // Prune low-importance old entries
}
```

### NIEDRIG (P3 - Nice to have)

#### 10. Syntax Highlighting im Chat
**Crate:** `syntect` oder `two-face`
**Implementierung:** Code-Blöcke erkennen und farblich hervorheben

#### 11. Command Auto-Completion
**Crate:** `reedline` oder `rustyline`
**Features:** Tab-Completion für Commands, Dateipfade, Models

#### 12. Config-File für Themes
**Format:** TOML in `~/.hcscoder/config.toml`
```toml
[ui]
theme = "dracula"
show_token_usage = true
scrollback_limit = 1000
```

---

## 🎨 UI/UX Verbesserungen (Alle umgesetzt)

### Design-Elemente
1. **ASCII Logo** - Beim Start im Plain Mode
2. **"Made with ❤️ by hcsmedia"** - Durchgängiges Branding
3. **Emoji-Indikatoren** - Visuelle Feedback für verschiedene Aktionen
4. **Theme-System** - 5 vordefinierte Themes
5. **Scrollbar** - Visuelles Feedback für scrollbaren Content
6. **Cursor** - Blinkender Indikator im Input-Feld

### Accessibility
- HighContrast Theme für sehbehinderte Nutzer
- Klare Fehlermeldungen mit Kontext
- Keyboard-Only Navigation (keine Maus erforderlich)

---

## 📦 Cross-Platform Installation

### install.sh Verbesserungen
```bash
#!/bin/bash
# Automatische Plattform-Erkennung
detect_platform() {
    case "$(uname -s)" in
        Linux*)     
            if [ -n "$PREFIX" ] && echo "$PREFIX" | grep -q "com.termux"; then
                echo "termux-aarch64"
            else
                echo "linux-$(uname -m)"
            fi
            ;;
        Darwin*)    echo "macos-$(uname -m)" ;;
        *)          echo "unknown" ;;
    esac
}

# Auto-Dependency Installation
install_dependencies() {
    if command -v apt-get &> /dev/null; then
        sudo apt-get install -y pkg-config libssl-dev
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y openssl-devel
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm openssl
    fi
}

# Build-from-Source Fallback
if [ ! -f "$BINARY_PATH" ]; then
    echo "Prebuilt binary not available, building from source..."
    cargo build --release
fi
```

### install.ps1 Verbesserungen
```powershell
# Execution Policy Handling
$currentPolicy = Get-ExecutionPolicy -Scope Process
if ($currentPolicy -eq "Restricted") {
    Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force
}

# Winget Fallback
if (!(Test-Path $binaryPath)) {
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install Rustlang.Rustup -e --silent
    }
}
```

---

## 📄 README.md Best Practices

### Neue Struktur
```markdown
# hcscoder 🚀

High-performance AI coding assistant by hcsmedia  
Made with ❤️ using Rust

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS%20%7C%20Android-lightgrey.svg)

## 🚀 One-Line Installation

### Linux/macOS/Termux
```bash
curl -fsSL https://raw.githubusercontent.com/hcsmediacorp/hcscoder/main/install.sh | bash
```

### Windows PowerShell
```powershell
irm https://raw.githubusercontent.com/hcsmediacorp/hcscoder/main/install.ps1 | iex
```

## ✨ Features

- 🎨 **5 Themes**: Default, Dracula, Gruvbox, Nord, HighContrast
- 📜 **Scrollable Chat**: Navigate through long conversations
- 🔒 **Privacy-First**: Zero telemetry, no phone-home
- 🛠️ **40+ Tools**: Shell, FileSystem, LSP, Git, Web, and more
- 🤖 **Smart Models**: Auto-recommendation based on task type

## 🎯 Quick Start

1. Set your API key:
   ```bash
   export OPENROUTER_API_KEY=your_key_here
   ```

2. Run hcscoder:
   ```bash
   hcscoder
   ```

3. Customize theme:
   ```bash
   export HCSCODER_THEME=dracula
   ```

## 📖 Commands

| Command | Description |
|---------|-------------|
| `chat` | Interactive chat session |
| `ask "<query>"` | Single question |
| `review <path>` | Code review |
| `buddy summon` | Get a companion |
| `memory view` | View memories |

## 🎨 Themes

Set via environment variable:
```bash
export HCSCODER_THEME=dracula  # Options: default, dracula, gruvbox, nord, highcontrast
```

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md).

## 📜 License

MIT License (c) 2026 hcsmedia — attribution required when redistributing.

## 👤 Contact

- **Author**: Tim from hcsmedia
- **Instagram**: [@timfromhcs](https://instagram.com/timfromhcs)
- **Email**: hcsmediagroup@gmail.com

---

Made with ❤️ by hcsmedia
```

---

## 🔒 Security Checklist

- [x] API-Key Validierung (Länge)
- [ ] API-Key Format-Validierung
- [ ] Windows ACL Implementation
- [ ] Shell-Injection Protection
- [ ] Path-Traversal Prevention in File-Tools
- [ ] Rate-Limiting für lokale Commands
- [ ] Secure Memory Wiping für API Keys

---

## 🧪 Test Coverage Plan

### Unit Tests (>80% Coverage)
- [ ] OpenRouter Client (SSE Parsing, Error Handling)
- [ ] Tool Registry (Dispatch, Argument Parsing)
- [ ] Theme System (Color Selection)
- [ ] Memory Operations (CRUD)

### Integration Tests
- [ ] Full Chat Session (Plain & TUI)
- [ ] Model Fallback Chain
- [ ] File Operations (Read/Write/Edit)
- [ ] Git Commands

### E2E Tests
- [ ] Installation Script (All Platforms)
- [ ] Setup Flow (API-Key + Model Selection)
- [ ] Buddy Gacha (Determinismus)

---

## 📈 Performance Targets

| Metric | Current | Target |
|--------|---------|--------|
| Startup Time | ~50ms | <20ms |
| First Token | ~500ms | <300ms |
| Memory Usage | ~50MB | <30MB |
| Binary Size | ~15MB | <10MB |

---

## 🔄 Release Checklist

Before each release:
- [ ] All tests passing
- [ ] No warnings (`#![deny(warnings)]`)
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Git tag created (`vX.Y.Z`)
- [ ] GitHub Release published
- [ ] Binaries attached (Windows, Linux, macOS)
- [ ] Installation scripts tested

---

## 📞 Support

For issues and questions:
1. Check existing [Issues](https://github.com/hcsmediacorp/hcscoder/issues)
2. Read [FAQ](docs/FAQ.md)
3. Contact: hcsmediagroup@gmail.com

---

**Last Updated:** 2026-01-XX  
**Version:** 1.0.0  
**Status:** Active Development
