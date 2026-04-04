# 🎯 hcscoder Roadmap to Perfection

## Executive Summary

**Current State:** Version 1.0.0 — First major Rust release with 40+ tools, OpenRouter integration, TUI/plain chat interfaces, and theming support.

**Vision:** Transform hcscoder into the gold standard for privacy-first, AI-powered CLI coding assistants — achieving production-grade security, performance, developer experience, and community adoption.

---

## 📊 Current State Analysis

### ✅ Strengths (What's Working Well)

1. **Solid Architecture**
   - Clean module separation (`hcscoder_*` crates)
   - Library-first design (`hcscoder` crate + binaries)
   - `#![deny(warnings)]` enforcement

2. **Feature-Rich**
   - 40+ autonomous tools
   - SSE streaming with robust error handling
   - 5 UI themes with proper abstraction
   - TUI + plain text modes
   - Buddy gacha system (unique feature)

3. **OpenRouter Compliance**
   - Proper attribution headers
   - Mid-stream error handling
   - Model fallback support
   - Tier-based model catalog

4. **Documentation**
   - Comprehensive README.md
   - BUGFIX_PLAN.md with prioritized issues
   - CHANGELOG.md following Keep a Changelog
   - SECURITY.md with responsible disclosure

### ❌ Critical Issues (Must Fix Before v1.1.0)

#### P0 - Security Vulnerabilities

1. **Shell Injection Risk** (`src/hcscoder_tools/bash.rs`)
   ```rust
   // Current: Direct shell execution
   Command::new("sh").arg("-c").arg(user_input)
   
   // Required: Command validation/whitelisting
   ```
   **Impact:** Remote code execution if AI generates malicious commands
   **Fix:** Implement command whitelist, argument escaping, dangerous command detection

2. **API Key Validation Weakness** (`src/hcscoder_openrouter/auth.rs`)
   - Only checks length (≥20 chars)
   - No format validation beyond prefix warning
   **Fix:** Strict regex validation, entropy checking

3. **Windows ACL Missing** (`src/hcscoder_openrouter/mod.rs:112-118`)
   - Unix: `chmod 600` implemented ✓
   - Windows: Inherits directory permissions only
   **Fix:** Use `windows-acl` crate for explicit DACL

4. **Path Traversal in File Tools** (`src/hcscoder_tools/filesystem.rs`)
   - No canonicalization before file operations
   - Potential for reading `/etc/passwd` via `../../../etc/passwd`
   **Fix:** `std::fs::canonicalize()` + chroot-like confinement

#### P1 - Reliability & Correctness

5. **SSE Parser Edge Cases** (`src/hcscoder_openrouter/client.rs:373-376`)
   - Empty data lines silently skipped (correct per RFC 8895)
   - But keep-alive detection could be clearer
   **Fix:** Explicit keep-alive comment handling, better logging

6. **Exponential Backoff Without Cap** (`client.rs:258-261`)
   ```rust
   let delay = Duration::from_millis(exponential_delay + jitter);
   // Can grow indefinitely on persistent failures
   ```
   **Fix:** Max delay cap (e.g., 30 seconds), circuit breaker pattern

7. **Token Usage Estimation Inaccurate** (`query_engine.rs`)
   - Simple character-count heuristic
   - No actual tokenization
   **Fix:** Integrate `tiktoken-rs` for accurate counts

8. **Memory Module Placeholder Code** (`src/hcscoder_memory/mod.rs`)
   - `consolidate_memories()` is stubbed
   - AutoDream not implemented
   **Fix:** Actual ML-lite consolidation algorithm

#### P2 - Developer Experience

9. **No Test Coverage** 
   - Few unit tests exist
   - Zero integration tests
   - No E2E test suite
   **Target:** >80% line coverage, critical path testing

10. **Missing Error Context**
    - Many `?` operators without `.context()`
    - Hard to debug production issues
    **Fix:** Comprehensive error contexts throughout

11. **Configuration Fragmentation**
    - API key: `~/.hcscoder/openrouter_api_key`
    - Model: `~/.hcscoder/openrouter_default_model`
    - Theme: `HCSCODER_THEME` env var only
    **Fix:** Unified `~/.hcscoder/config.toml`

12. **No Completion/History in Plain Mode**
    - Raw `stdin.read_line()` usage
    - No arrow-key history, no tab completion
    **Fix:** Integrate `reedline` or `rustyline`

---

## 🏆 Perfection Targets (v2.0.0 Vision)

### Category 1: Security Excellence ⭐⭐⭐⭐⭐

| Target | Current | Goal | Priority |
|--------|---------|------|----------|
| Shell injection protection | ❌ None | ✅ Whitelist + escaping | P0 |
| API key validation | ⚠️ Basic | ✅ Regex + entropy check | P0 |
| Windows ACL | ❌ Missing | ✅ Explicit DACL | P0 |
| Path traversal prevention | ❌ Missing | ✅ Canonicalization + sandbox | P0 |
| Secure memory wiping | ❌ None | ✅ `zeroize` crate | P1 |
| Rate limiting (local) | ❌ None | ✅ Token bucket | P2 |
| Audit logging | ❌ None | ✅ Structured audit trail | P2 |

**Acceptance Criteria:**
- Pass `cargo-audit` with zero vulnerabilities
- Pass `cargo-deny` license/dependency checks
- External security audit completed
- CVE response process documented

### Category 2: Performance Mastery ⭐⭐⭐⭐⭐

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Binary size | ~15MB | <8MB | `strip` + `mold` linker |
| Startup time | ~50ms | <15ms | `hyperfine` benchmark |
| First token latency | ~500ms | <250ms | Time-to-first-byte |
| Memory footprint | ~50MB | <25MB | RSS at idle |
| Throughput | N/A | 100 req/min | Stress test |

**Optimization Strategies:**
1. Enable LTO + `codegen-units = 1` ✓ (already done)
2. Replace `reqwest` with `hyper` + `rustls` for smaller binary
3. Lazy-load heavy dependencies (LSP, git)
4. Use `smallvec` for hot paths
5. Profile with `cargo flamegraph`

### Category 3: Developer Experience ⭐⭐⭐⭐⭐

#### 3.1 Testing Infrastructure
```toml
# Required test coverage
- Unit tests: >80% line coverage
- Integration tests: All public APIs
- E2E tests: Installation, setup, chat flows
- Fuzz tests: SSE parser, user input handling
- Property tests: Tool argument validation
```

**Test Structure:**
```
tests/
├── unit/           # Fast, isolated tests
├── integration/    # API client, tool registry
├── e2e/           # Full workflow tests
├── fixtures/      # Test data
└── mocks/         # OpenRouter mock server
```

#### 3.2 Documentation Standards
- **API Docs:** `cargo doc --no-deps --open` → 100% public items documented
- **Examples:** `/examples` directory with working use cases
- **Tutorials:** Step-by-step guides for common workflows
- **Architecture Decision Records (ADRs):** Document key design choices

#### 3.3 CLI Polish
```bash
# Add missing quality-of-life features
✅ Smart defaults
✅ Sensible --help messages (clap_derive)
⚠️ Shell completions (bash, zsh, fish, PowerShell)
⚠️ Man page generation
⚠️ Interactive config wizard (beyond hcscoder-setup)
⚠️ Plugin system for custom tools
```

### Category 4: Feature Completeness ⭐⭐⭐⭐

#### 4.1 Core Features (Must Have)
- [x] Chat interface (TUI + plain)
- [x] SSE streaming
- [x] Multi-model support
- [x] 40+ built-in tools
- [ ] **Conversation persistence** (save/load chat history)
- [ ] **Multi-file editing** (atomic batch edits)
- [ ] **Project context awareness** (auto-load `.hcscoder/context.md`)
- [ ] **Tool result caching** (avoid redundant API calls)

#### 4.2 Advanced Features (Should Have)
- [ ] **MCP (Model Context Protocol) support** — interop with Claude Desktop
- [ ] **Local LLM fallback** — ollama integration when offline
- [ ] **Voice input/output** — experimental accessibility feature
- [ ] **Collaborative sessions** — shared context across terminals
- [ ] **Custom tool definitions** — YAML/JSON tool specs

#### 4.3 Nice-to-Have (Could Have)
- [ ] VS Code extension (wrapper around CLI)
- [ ] Syntax highlighting in TUI (`syntect` integration)
- [ ] Diff viewer for file edits
- [ ] Session recording/playback
- [ ] Analytics dashboard (opt-in, privacy-preserving)

### Category 5: Community & Ecosystem ⭐⭐⭐⭐⭐

#### 5.1 Contribution Guidelines
```markdown
# CONTRIBUTING.md checklist
- Clear PR template
- Code style guide (rustfmt.toml ✓)
- Commit message convention (Conventional Commits)
- Issue templates (bug, feature, question)
- Code of Conduct
- Development setup guide
```

#### 5.2 Release Engineering
```yaml
# GitHub Actions automation
- CI: cargo test, clippy, fmt on every PR
- Release: Automatic on git tag (existing ✓)
- Benchmarks: Nightly performance tracking
- Dependency updates: Dependabot (existing ✓)
- Security scanning: cargo-audit in CI
```

#### 5.3 Distribution Channels
| Channel | Status | Target |
|---------|--------|--------|
| GitHub Releases | ✅ Done | Maintain |
| crates.io | ❌ Missing | Publish library |
| Homebrew | ❌ Missing | `brew install hcscoder` |
| Scoop (Windows) | ❌ Missing | Community bucket |
| AUR (Arch Linux) | ❌ Missing | Community package |
| Docker | ❌ Missing | Multi-arch image |
| Nix | ❌ Missing | Flake integration |

---

## 📅 Implementation Timeline

### Phase 1: Foundation (Weeks 1-2) — v1.1.0
**Theme:** Security hardening + critical bug fixes

**Deliverables:**
- [ ] Shell injection protection (#1)
- [ ] API key validation overhaul (#3)
- [ ] Windows ACL implementation (#4)
- [ ] Path traversal prevention (#5)
- [ ] Exponential backoff with cap (#6)
- [ ] Error context improvements (#10)

**Success Metrics:**
- Zero P0 security issues remaining
- All existing tests passing
- No new compiler warnings

### Phase 2: Reliability (Weeks 3-4) — v1.2.0
**Theme:** Testing infrastructure + stability

**Deliverables:**
- [ ] Unit test suite (>50% coverage)
- [ ] Integration tests for OpenRouter client
- [ ] Mock server for offline testing
- [ ] SSE parser fuzz testing
- [ ] Token usage accuracy (`tiktoken-rs`)
- [ ] Memory module implementation

**Success Metrics:**
- >50% code coverage
- CI pipeline <10 minutes
- Zero flaky tests

### Phase 3: DX Enhancement (Weeks 5-6) — v1.3.0
**Theme:** Developer experience polish

**Deliverables:**
- [ ] Shell completions (all major shells)
- [ ] Man page generation
- [ ] Unified config.toml
- [ ] Input history + completion (reedline)
- [ ] Conversation persistence
- [ ] Example gallery

**Success Metrics:**
- `--help` scores 10/10 on clarity survey
- First-time setup <2 minutes
- Zero configuration-related issues

### Phase 4: Performance (Weeks 7-8) — v1.4.0
**Theme:** Speed + efficiency

**Deliverables:**
- [ ] Binary size optimization (<10MB)
- [ ] Startup time profiling + fixes
- [ ] Memory usage reduction
- [ ] Caching layer for tool results
- [ ] Lazy loading for heavy modules

**Success Metrics:**
- All performance targets met
- Benchmark suite in CI
- Profile-guided optimization (PGO) build option

### Phase 5: Feature Expansion (Weeks 9-12) — v2.0.0
**Theme:** Advanced capabilities

**Deliverables:**
- [ ] MCP protocol support
- [ ] Local LLM integration (ollama)
- [ ] Multi-file atomic edits
- [ ] Project context auto-loading
- [ ] Plugin system alpha
- [ ] VS Code extension prototype

**Success Metrics:**
- Feature parity with leading alternatives
- Plugin API documentation complete
- Community contributions accepted

---

## 🔧 Technical Debt Register

| ID | Description | Impact | Effort | Priority |
|----|-------------|--------|--------|----------|
| TD001 | Deprecated `atty` crate usage | Low | Low | ✅ Fixed (using `IsTerminal`) |
| TD002 | Simplified memory parser | Medium | Medium | P2 |
| TD003 | No structured logging config | Low | Low | P3 |
| TD004 | Hardcoded system prompt | Medium | Low | P2 |
| TD005 | Magic numbers in backoff calc | Low | Low | ✅ Fixed in plan |
| TD006 | Unwrap() in non-test code | High | Medium | P1 |
| TD007 | No graceful shutdown handling | Medium | Medium | P2 |
| TD008 | Blocking I/O in async context | High | High | P1 |

---

## 📈 Success Metrics Dashboard

### Quantitative KPIs
| Metric | Baseline | Target | Tracking Method |
|--------|----------|--------|-----------------|
| GitHub Stars | Current | 1,000+ | GitHub API |
| Monthly Downloads | 0 | 10,000+ | crates.io API |
| Issue Resolution Time | N/A | <48h avg | GitHub Insights |
| PR Merge Time | N/A | <7 days | GitHub Insights |
| Test Coverage | ~10% | >80% | `cargo-tarpaulin` |
| Binary Size | 15MB | <8MB | `ls -lh` |
| Startup Time | 50ms | <15ms | `hyperfine` |
| Crash Rate | Unknown | <0.1% | Sentry (opt-in) |

### Qualitative Goals
- [ ] Featured on [This Week in Rust](https://this-week-in-rust.org/)
- [ ] Positive review from influential Rustacean
- [ ] Adopted by at least 3 open-source projects
- [ ] Speaking opportunity at Rust meetup/conference
- [ ] Community-maintained packages (Homebrew, AUR, etc.)

---

## 🛡️ Risk Mitigation

### Technical Risks
| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| OpenRouter API changes | Medium | High | Abstraction layer, version pinning |
| Dependency vulnerability | High | Medium | cargo-audit in CI, minimal deps |
| Performance regression | Medium | Medium | Benchmark suite, perf budgets |
| Breaking API changes | Low | High | SemVer discipline, changelog |

### Business Risks
| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| OpenRouter service discontinuation | Low | Critical | Multi-provider support (Anthropic, Ollama) |
| License violation by users | Medium | Low | Clear LICENSE, attribution enforcement |
| Competitor feature parity | High | Medium | Rapid iteration, community focus |
| Maintainer burnout | Medium | High | Documentation, co-maintainers, scope management |

---

## 🎓 Learning & Growth

### Skills to Develop
1. **Advanced Rust Patterns**
   - Async trait objects
   - Custom allocators for performance
   - FFI for potential C bindings

2. **Security Engineering**
   - Threat modeling
   - Secure coding practices
   - Penetration testing basics

3. **DevOps Excellence**
   - Multi-platform CI/CD
   - Container optimization
   - Observability (metrics, tracing)

4. **Community Building**
   - Open-source governance
   - Conflict resolution
   - Mentorship

### Knowledge Base Creation
- Architecture decision records (ADRs)
- Performance tuning guide
- Security best practices doc
- Contributor onboarding guide

---

## 🏁 Definition of "Perfection"

**hcscoder achieves perfection when:**

1. ✅ **Security:** Zero known vulnerabilities, audited, best-in-class key management
2. ✅ **Performance:** Sub-15ms startup, <25MB RAM, <8MB binary
3. ✅ **Reliability:** >99.9% uptime in production use, comprehensive test coverage
4. ✅ **Usability:** Intuitive CLI, excellent docs, helpful error messages
5. ✅ **Maintainability:** Clean code, full test coverage, easy to extend
6. ✅ **Community:** Active contributors, regular releases, welcoming environment
7. ✅ **Ecosystem:** Packaged for all major platforms, plugin ecosystem thriving
8. ✅ **Innovation:** Unique features competitors lack, pushing CLI AI assistant boundaries

**Philosophy:** Perfection is not a destination but a direction. Every release should move measurably closer to these ideals while maintaining backward compatibility and user trust.

---

## 📞 Call to Action

### For Maintainers
1. Review and prioritize this roadmap
2. Create GitHub issues for each deliverable
3. Set up project board with milestones
4. Establish regular release cadence (e.g., every 2 weeks)

### For Contributors
1. Pick an issue matching your skill level
2. Read CONTRIBUTING.md before first PR
3. Join community discussions (Discord/Matrix)
4. Share your use cases and feedback

### For Users
1. Star the repository to show support
2. Report bugs with detailed reproduction steps
3. Suggest features via GitHub Discussions
4. Spread the word in your network

---

**Document Version:** 1.0  
**Created:** 2026-01-XX  
**Last Updated:** 2026-01-XX  
**Status:** Draft for Review  
**Owner:** hcsmedia (Tim)  

---

*Made with ❤️ by hcsmedia — Attribution required when redistributing.*
