# hcscoder Architecture

This document provides a detailed overview of the hcscoder architecture, module structure, and design decisions.

## Overview

hcscoder is a privacy-first, Rust-native CLI coding assistant that interfaces with OpenRouter AI models. It's designed with modularity, security, and performance in mind.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      User Interface                          │
│  ┌─────────────────────┐  ┌──────────────────────────────┐  │
│  │   TUI (ratatui)     │  │   Plain Mode (CLI)           │  │
│  │   - Chat interface  │  │   - Simple stdin/stdout      │  │
│  │   - Themes          │  │   - Scriptable               │  │
│  │   - Streaming       │  │                              │  │
│  └─────────────────────┘  └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Core Engine                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Query Engine                                         │   │
│  │  - Parse user input                                   │   │
│  │  - Route to appropriate tools                         │   │
│  │  - Manage conversation context                        │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Tool Coordinator                                     │   │
│  │  - Tool discovery                                     │   │
│  │  - Tool execution                                     │   │
│  │  - Result aggregation                                 │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Tools Layer                             │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │
│  │ Bash    │ │ File    │ │ Git     │ │ Web     │ │ LSP    │ │
│  │ Runner  │ │ System  │ │ Tools   │ │ Search  │ │ Tools  │ │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └────────┘ │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │
│  │ Memory  │ │ Planner │ │ Skills  │ │ Team    │ │ ...    │ │
│  │ Tools   │ │ Tools   │ │         │ │         │ │        │ │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 External Services                            │
│  ┌─────────────────────┐  ┌──────────────────────────────┐  │
│  │  OpenRouter API     │  │  Local Filesystem            │  │
│  │  - Chat completion  │  │  - Read/Write files          │  │
│  │  - Model selection  │  │  - Watch changes             │  │
│  │  - Streaming        │  │                              │  │
│  └─────────────────────┘  └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Module Structure

### `hcscoder_openrouter`
**Purpose:** OpenRouter API client and authentication

**Key Components:**
- `auth.rs` - API key validation and management
- `client.rs` - HTTP client for OpenRouter API
- `models.rs` - Model listing and selection

**Security Features:**
- API key format validation (regex + entropy check)
- Null-byte prevention
- Pattern detection for compromised keys
- Secure file permissions (600) on key storage

### `hcscoder_engine`
**Purpose:** Core engine and tool coordination

**Key Components:**
- `coordinator.rs` - Orchestrates tool execution
- `query_engine.rs` - Parses and routes user queries
- `tool_runtime.rs` - Runtime environment for tools

### `hcscoder_tools`
**Purpose:** Individual tool implementations (40+ tools)

**Tool Categories:**
1. **System Tools**: bash, filesystem, process management
2. **Development Tools**: git, lsp, code analysis
3. **AI Tools**: memory, planner, skills
4. **Utility Tools**: web search, sleep, notebook

**Design Pattern:** Each tool implements a common trait for:
- Name and description
- Parameter schema
- Execution logic
- Error handling

### `hcscoder_ui`
**Purpose:** Terminal user interface

**Key Components:**
- TUI implementation using ratatui
- Theme system (5 themes)
- Streaming output handler
- Scroll management

### `hcscoder_memory`
**Purpose:** Conversation memory management

**Features:**
- Short-term memory (current session)
- Long-term memory (persistent across sessions via JSON roundtrip storage)
- Memory consolidation (planned)

### `hcscoder_planner`
**Purpose:** Planning and task decomposition

**Features:**
- Task breakdown
- Progress tracking
- Dependency management

### Experimental modules (explicitly non-stable)

- `hcscoder_tools::mcp` — scaffold for MCP server integration.
- `hcscoder_tools::repl` — session + history implemented, code execution currently simulated.
- `hcscoder_tools::notebook::execute_cell` — notebook execution requires kernel integration.

These modules are intentionally shipped for forward compatibility and iterative hardening, but are not part of the stable core contract.

## Data Flow

### Request Flow
1. User inputs query via TUI or plain mode
2. Query engine parses input
3. Coordinator determines required tools
4. Tools execute with proper security checks
5. Results aggregated and sent to OpenRouter
6. AI response streamed back to user

### Security Flow
1. All inputs validated (API keys, paths, commands)
2. Path canonicalization and traversal prevention
3. Command injection detection
4. Audit logging for security events
5. Principle of least privilege enforced

## Design Decisions

### Why Rust?
- Memory safety without garbage collection
- Strong type system prevents runtime errors
- Excellent performance for CLI applications
- Growing ecosystem for terminal UIs

### Why OpenRouter?
- Access to multiple AI models through single API
- Privacy-focused (no training on user data)
- Competitive pricing
- SSE streaming support

### Why Modular Architecture?
- Separation of concerns
- Easier testing and maintenance
- Potential for plugin system in future
- Library-first design allows embedding

### Why TUI + Plain Mode?
- TUI for interactive usage
- Plain mode for scripting and CI/CD
- Automatic detection based on terminal capabilities

## Security Model

### Threat Model
- Malicious user input (injection attacks)
- Compromised API keys
- Path traversal attempts
- Privilege escalation

### Mitigations
1. **Input Validation**: All inputs sanitized and validated
2. **Path Security**: Canonicalization, blocked patterns
3. **Command Security**: Dangerous pattern detection
4. **Audit Logging**: All security events logged
5. **Least Privilege**: Minimal permissions required

## Performance Considerations

- Async I/O using Tokio for non-blocking operations
- Streaming responses to reduce latency
- LTO (Link-Time Optimization) for release builds
- Minimal dependencies to reduce binary size

## Future Architecture Plans

See `PERFECTION_ROADMAP.md` for detailed plans including:
- Plugin system for extensibility
- Config file support
- Enhanced memory consolidation
- Improved error context propagation
- Windows ACL support

## Testing Strategy

- Unit tests for individual functions
- Integration tests for CLI behavior
- Benchmark suite for performance tracking
- Security-focused test cases

## Contributing

When adding new modules:
1. Follow existing module structure
2. Implement appropriate traits
3. Add comprehensive tests
4. Update this documentation
5. Consider security implications

See `CONTRIBUTING.md` for detailed guidelines.
