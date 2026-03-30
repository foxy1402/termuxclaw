# ZeroClaw Codebase Index

**Termux-Only Runtime** — Android AI agent optimized for mobile constraints.

> **For AI Agents**: This file provides a comprehensive map of the codebase structure, module responsibilities, and entry points for development and maintenance tasks.

---

## Quick Stats

- **Total Source Files**: 349 Rust files (.rs)
- **Documentation**: 66 Markdown files
- **Tests**: 52 test files
- **Core Modules**: 31 subsystems in `src/`
- **Tools**: 75 tool implementations
- **Channels**: 42 communication backends
- **Providers**: 16 model provider integrations

---

## 🎯 Entry Points for AI Agents

### For Bug Fixes
1. Read error message and identify the module (e.g., `src/providers/anthropic.rs`)
2. Check `src/lib.rs` for module exports
3. Check factory files: `src/providers/mod.rs`, `src/channels/mod.rs`, `src/tools/mod.rs`
4. Review related tests in same directory or `tests/`

### For New Features
1. **New Tool**: Add to `src/tools/`, implement `Tool` trait, register in `src/tools/mod.rs`
2. **New Channel**: Add to `src/channels/`, implement `Channel` trait, register in `src/channels/mod.rs`
3. **New Provider**: Add to `src/providers/`, implement `Provider` trait, register in `src/providers/mod.rs`
4. **New Config**: Update `src/config/schema.rs` and add schema validation

### For Documentation Updates
- User guides: `docs/setup-guides/`
- API reference: `docs/reference/`
- Architecture: `docs/architecture/`
- Termux-specific: `TERMUX_SYSTEM_PROMPT_GUIDE.md`, `TERMUX-TRIM-SUMMARY.md`

---

## 📁 Root Directory Structure

```
termuxclaw/
├── src/                    # Main source code (307 files)
├── docs/                   # Documentation (50 files)
├── tests/                  # Integration tests (52 files)
├── scripts/                # Utility scripts (12 files)
├── skills/                 # Agent skills/prompts
├── .github/                # CI/CD workflows (29 files)
├── dev/                    # Development tools
├── benches/                # Performance benchmarks
├── Cargo.toml              # Rust project manifest
├── install.sh              # Termux-only installer
└── backup_install.sh       # Cross-platform installer backup
```

---

## 🔧 Core Subsystems (`src/`)

### Module Overview (31 modules, 307 files)

| Module | Files | Purpose |
|--------|-------|---------|
| **tools** | 75 | Tool execution surface (shell, file I/O, memory, browser, etc.) |
| **channels** | 42 | Communication backends (Telegram, Discord, Slack, WhatsApp, etc.) |
| **memory** | 26 | Storage backends (markdown, SQLite, PostgreSQL, Qdrant, etc.) |
| **security** | 18 | Policy, pairing, secrets, sandboxing (Termux: noop sandbox) |
| **providers** | 16 | Model providers (Anthropic, OpenAI, Gemini, Cohere, etc.) |
| **agent** | 15 | Orchestration loop (dispatcher, classifier, thinking, memory) |
| **gateway** | 11 | Webhook/REST API server (WebSocket, SSE, static files, auth) |
| **observability** | 10 | Instrumentation backends (logging, Prometheus, OpenTelemetry) |
| **tunnel** | 8 | NAT traversal for webhooks (ngrok, localhost.run, etc.) |
| **sop** | 7 | Standard Operating Procedures engine |
| **auth** | 6 | OAuth/token management |
| **skills** | 6 | Skill loader and executor |
| **verifiable_intent** | 6 | Intent verification system |
| **plugins** | 6 | WASM plugin system |
| **cron** | 5 | Scheduled task execution |
| **runtime** | 4 | Runtime adapters (native only for Termux) |
| **skillforge** | 4 | Skill creation/management |
| **config** | 4 | Schema + config loading/merging |
| **cost** | 3 | Usage tracking and billing |
| **commands** | 3 | CLI command implementations |
| **heartbeat** | 3 | Health monitoring |
| **hooks** | 3 | Lifecycle hooks (command logger, audit) |
| **routines** | 3 | Recurring task management |
| **hands** | 2 | Human-in-the-loop approval |
| **integrations** | 2 | Third-party integrations |
| **nodes** | 2 | Node/cluster management |
| **onboard** | 2 | Interactive setup wizard |
| **approval** | 1 | Approval flow management |
| **daemon** | 1 | Background service mode |
| **doctor** | 1 | System diagnostics |
| **health** | 1 | Health check endpoints |

---

## 🔑 Key Files

### Core Entry Points

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entrypoint and command routing |
| `src/lib.rs` | Module exports and shared command enums |
| `src/agent/mod.rs` | Main agent orchestration loop |
| `src/gateway/mod.rs` | HTTP gateway server |

### Factory Registration

| File | Registers |
|------|-----------|
| `src/providers/mod.rs` | Model provider factories (Anthropic, OpenAI, Gemini, etc.) |
| `src/channels/mod.rs` | Channel factories (Telegram, Discord, Slack, etc.) |
| `src/tools/mod.rs` | Tool registry (~100 tools) |
| `src/memory/mod.rs` | Memory backend factories |
| `src/runtime/mod.rs` | Runtime adapter factories (native only) |

### Configuration

| File | Purpose |
|------|---------|
| `src/config/schema.rs` | Complete config schema (7000+ lines) |
| `src/config/load.rs` | Config loading and merging |
| `src/config/mod.rs` | Config factory and defaults |
| `.env.example` | Environment variable template |

### Security

| File | Purpose |
|------|---------|
| `src/security/policy.rs` | Security policy (autonomy, forbidden paths, rate limits) |
| `src/security/detect.rs` | Sandbox detection (always returns NoopSandbox for Termux) |
| `src/security/pairing.rs` | Channel pairing and authorization |
| `src/security/secret_store.rs` | Secret storage backend |

---

## 🛠️ Tool Categories (`src/tools/`)

### File Operations (20+ tools)
- `file_read.rs`, `file_write.rs`, `file_edit.rs`, `file_delete.rs`
- `glob.rs`, `grep.rs`, `tree.rs`
- `compress.rs`, `decompress.rs`, `download.rs`

### Shell & Process (10+ tools)
- `shell.rs` — Shell command execution
- `shell_which.rs` — Command lookup
- `process_list.rs`, `process_kill.rs`

### Memory & Knowledge (10+ tools)
- `memory_*.rs` — Memory operations (add, search, consolidate, decay)
- `sql.rs` — SQL query execution
- `web_fetch.rs` — HTTP requests

### Agent Control (15+ tools)
- `task.rs` — Sub-agent spawning
- `ask_user.rs` — User input prompting
- `thinking.rs` — Thinking protocol
- `report_intent.rs` — Intent reporting

### Scheduling (5+ tools)
- `cron_add.rs`, `cron_update.rs`, `cron_cancel.rs`
- `schedule.rs` — Shell-only scheduling
- `delay.rs` — Delayed execution

### Integration Tools (15+ tools)
- GitHub API tools (`github_*.rs`)
- Google Workspace (`google_docs.rs`, `google_sheets.rs`, `gmail_*.rs`)
- Slack, Discord, Telegram direct messaging

---

## 📡 Channel Implementations (`src/channels/`)

### Messaging Platforms (42 implementations)
- **Major**: Telegram, Discord, Slack, WhatsApp, QQ, WeChat
- **Enterprise**: Microsoft Teams, Mattermost, Rocket.Chat, Zulip
- **Social**: Facebook Messenger, Instagram, Twitter/X, Snapchat
- **Asian Markets**: LINE, KakaoTalk, Viber, Zalo
- **Niche**: Signal, IRC, XMPP/Jabber, Nextcloud Talk
- **Email**: Gmail, Outlook, IMAP/SMTP

### Session Management
- `session_sqlite.rs` — SQLite session backend
- `session_postgres.rs` — PostgreSQL session backend
- `session_redis.rs` — Redis session backend

### Voice & TTS
- `tts.rs` — Text-to-speech (Termux: Edge TTS gated for desktop)
- `voice.rs` — Voice message handling

---

## 🤖 Model Providers (`src/providers/`)

### Supported Providers (16 total)
1. **anthropic.rs** — Anthropic Claude (Sonnet, Opus, Haiku)
2. **openai.rs** — OpenAI GPT models
3. **gemini.rs** — Google Gemini (Termux: API key only, no CLI OAuth)
4. **cohere.rs** — Cohere Command models
5. **deepseek.rs** — DeepSeek models
6. **mistral.rs** — Mistral AI
7. **groq.rs** — Groq inference
8. **together.rs** — Together AI
9. **fireworks.rs** — Fireworks AI
10. **bedrock.rs** — AWS Bedrock (Termux: 1s IMDSv2 timeout)
11. **azure.rs** — Azure OpenAI
12. **copilot.rs** — GitHub Copilot (Termux: requires GITHUB_TOKEN env var)
13. **perplexity.rs** — Perplexity AI
14. **openrouter.rs** — OpenRouter
15. **xai.rs** — xAI Grok
16. **custom.rs** — Custom API endpoints

### Provider Infrastructure
- `router.rs` — Provider routing and failover
- `resilient.rs` — Retry logic and error handling
- `traits.rs` — Provider trait definition

---

## 💾 Memory Backends (`src/memory/`)

### Storage Implementations
- `markdown.rs` — Markdown file storage
- `sqlite.rs` — SQLite backend
- `postgres.rs` — PostgreSQL backend
- `qdrant.rs` — Vector database (Qdrant)

### Memory Features
- `embeddings.rs` — Embedding generation
- `vector_merge.rs` — Vector merging/deduplication
- `decay.rs` — Memory decay over time
- `consolidate.rs` — Memory consolidation
- `audit.rs` — Audit trail support

---

## 🔐 Security Subsystem (`src/security/`)

### Components
- `policy.rs` — Security policy enforcement
- `detect.rs` — Sandbox detection (Termux: always NoopSandbox)
- `pairing.rs` — Channel pairing authorization
- `secret_store.rs` — Secret storage
- `sandbox/noop.rs` — No-op sandbox (Android app sandbox provides isolation)

### Termux-Specific
- **Forbidden paths**: `/data/data`, `/system`, `/vendor`, `/product` (Android-specific)
- **No desktop sandboxes**: Firejail, Docker, Bubblewrap, Landlock, Seatbelt all removed

---

## 🌐 Gateway Server (`src/gateway/`)

### Components
- `mod.rs` — Main gateway server and routing
- `api.rs` — REST API endpoints
- `websocket.rs` — WebSocket handler
- `sse.rs` — Server-Sent Events
- `static_files.rs` — Static file serving
- `auth.rs` — Authentication middleware
- `rate_limit.rs` — Rate limiting
- `idempotency.rs` — Idempotency store

### Webhook Handlers
- 50+ webhook endpoints for various platforms
- Telegram, Discord, Slack, WhatsApp, etc.
- GitHub, Gmail push notifications

---

## 📚 Documentation Structure (`docs/`)

```
docs/
├── setup-guides/           # Installation and onboarding
│   ├── quick-start.md
│   ├── termux-setup.md
│   └── configuration.md
├── reference/              # API and config reference
│   ├── api/
│   ├── config-schema.md
│   ├── providers-reference.md
│   └── tools-reference.md
├── ops/                    # Operations and deployment
│   ├── deployment.md
│   ├── monitoring.md
│   └── troubleshooting.md
├── security/               # Security documentation
│   ├── policy.md
│   ├── secrets.md
│   └── sandboxing.md
├── contributing/           # Contributor guides
│   ├── change-playbooks.md
│   ├── pr-discipline.md
│   └── docs-contract.md
├── maintainers/            # Maintainer documentation
│   ├── release-process.md
│   └── governance.md
├── architecture/           # Architecture deep-dives
│   └── design-decisions.md
└── superpowers/            # Advanced features
    └── agent-orchestration.md
```

---

## 🧪 Testing Structure (`tests/`)

### Test Categories
- **Integration tests**: `tests/*.rs` (52 files)
- **Unit tests**: Inline in `src/**/*.rs` files
- **Benchmarks**: `benches/agent_benchmarks.rs`

### Key Test Files
- `test_agent.rs` — Agent orchestration tests
- `test_providers.rs` — Provider integration tests
- `test_channels.rs` — Channel backend tests
- `test_tools.rs` — Tool execution tests
- `test_memory.rs` — Memory backend tests

---

## 🔨 Build & Development

### Key Files
- `Cargo.toml` — Rust project manifest, dependencies, features
- `Cargo.lock` — Locked dependency versions
- `build.rs` — Build script
- `Justfile` — Task runner recipes
- `rustfmt.toml` — Code formatting rules
- `clippy.toml` — Lint configuration
- `taplo.toml` — TOML formatter config
- `deny.toml` — Dependency license/security checks

### Feature Flags (Termux-focused)
```toml
default = ["termux"]  # Termux is now default
termux = [...]        # Termux-specific features
```

### Build Commands
```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets -- -D warnings

# Test
cargo test --locked

# Build release
cargo build --release --locked

# Full CI validation
./dev/ci.sh all
```

---

## 📦 Scripts & Tools (`scripts/`, `dev/`)

### Installation
- `install.sh` — Termux-only installer (new default)
- `backup_install.sh` — Cross-platform installer (backup)

### Development
- `dev/ci.sh` — CI validation script
- `dev/termux-boot.sh` — Termux:Boot integration
- `dev/test-termux-release.sh` — Termux release testing

### Utilities
- `scripts/*.sh` — Various utility scripts (12 files)

---

## 🤖 GitHub Workflows (`.github/workflows/`)

### CI/CD Pipelines
- `ci-run.yml` — Main CI pipeline
- `checks-on-pr.yml` — PR validation
- `master-branch-flow.md` — Branch workflow documentation
- `release-stable-manual.yml` — Stable release
- `release-beta-on-push.yml` — Beta release
- `publish-crates.yml` — Crates.io publishing

### Platform Distribution
- `pub-aur.yml` — Arch User Repository
- `pub-homebrew-core.yml` — Homebrew
- `pub-scoop.yml` — Scoop (Windows)

### Notifications
- `discord-release.yml` — Discord announcements
- `tweet-release.yml` — Twitter announcements

---

## 🎯 Termux-Specific Notes

### What's Different in Termux Build

#### Removed (Desktop-Only)
- ❌ Docker runtime (`src/runtime/docker.rs`)
- ❌ Sandboxing backends (Firejail, Landlock, Bubblewrap, Seatbelt)
- ❌ Service management (`src/service/`)
- ❌ 14 desktop tools (browser, screenshot, browser_delegate, etc.)
- ❌ 3 CLI-based providers (claude_code, gemini_cli, codex_cli)
- ❌ Matrix channel
- ❌ Edge TTS subprocess (gated for desktop)
- ❌ Gemini CLI OAuth (API key only)
- ❌ GitHub Copilot device flow (requires GITHUB_TOKEN)

#### Modified for Termux
- ✅ Bedrock IMDSv2: 1s timeout (down from 3s)
- ✅ Security policy: Added Android forbidden paths
- ✅ Sandbox: Always uses NoopSandbox (Android app sandbox provides isolation)
- ✅ Daemon management: Gated behind `#[cfg(not(target_os = "android"))]`
- ✅ Install script: Termux-first, mobile-optimized

#### Termux Documentation
- `TERMUX_SYSTEM_PROMPT_GUIDE.md` — System prompt for AI agents
- `TERMUX-TRIM-SUMMARY.md` — Trim summary and rationale
- `termux-trim.md` — Detailed trim planning
- `INSTALL-TRIM-GUIDE.md` — Installation trim guide

---

## 📝 Change Workflow for AI Agents

### Before Making Changes
1. Read `AGENTS.md` — Cross-tool agent instructions
2. Read `CLAUDE.md` — Claude Code-specific directives
3. Check `CONTRIBUTING.md` — Full contributing guide
4. Review `docs/contributing/change-playbooks.md` — Module-specific guides

### Risk Classification
- **Low**: docs, tests-only, chores → lightweight checks
- **Medium**: most `src/` changes → full test suite
- **High**: `src/security/`, `src/runtime/`, `src/gateway/`, `.github/workflows/` → full CI + review

### Adding New Components

#### New Tool
1. Create `src/tools/new_tool.rs`
2. Implement `Tool` trait from `src/tools/traits.rs`
3. Register in `src/tools/mod.rs` `BUILTIN_TOOLS` vec
4. Add tests in same file or `tests/`
5. Update `docs/reference/tools-reference.md`

#### New Channel
1. Create `src/channels/new_channel.rs`
2. Implement `Channel` trait from `src/channels/traits.rs`
3. Add factory `fn fn_new_channel()` 
4. Register in `src/channels/mod.rs` `factory()` match arm
5. Add session backend if needed: `src/channels/session_new_channel.rs`
6. Update webhook route in `src/gateway/mod.rs` if needed

#### New Provider
1. Create `src/providers/new_provider.rs`
2. Implement `Provider` trait from `src/providers/traits.rs`
3. Add factory `fn fn_new_provider()`
4. Register in `src/providers/mod.rs`:
   - Import module and factory
   - Add match arm in `factory()`
   - Add `ProviderInfo` entry
   - Add test function
5. Update `docs/reference/providers-reference.md`

#### New Config Field
1. Update `src/config/schema.rs` with new struct/field
2. Add serde defaults and validation
3. Update `Config` struct to include new field
4. Add tests for config loading
5. Document in config schema docs

---

## 🔍 Common Debugging Paths

### Compilation Errors
1. Check `src/lib.rs` for module exports
2. Check factory files for registration
3. Run `cargo check --lib` to isolate library issues
4. Review recent git changes: `git diff HEAD~1`

### Runtime Errors
1. Check logs with `RUST_LOG=debug`
2. Review error in context (check calling code)
3. Check config schema validation
4. Check security policy constraints

### Test Failures
1. Run single test: `cargo test --lib test_name -- --nocapture`
2. Check test setup (temp dirs, mock configs)
3. Review test assertions
4. Check for environmental dependencies

### Performance Issues
1. Run benchmarks: `cargo bench`
2. Profile with `cargo flamegraph`
3. Check memory allocations
4. Review async task spawning

---

## 📋 Pre-PR Checklist

- [ ] Branched from `master` (NOT `main`)
- [ ] Ran `cargo fmt --all`
- [ ] Ran `cargo clippy --all-targets -- -D warnings`
- [ ] Ran `cargo test --locked` (all pass)
- [ ] Filled PR template completely
- [ ] No secrets or personal data in commits
- [ ] Backward compatible OR migration steps documented
- [ ] i18n follow-through if user-facing wording changed
- [ ] Security impact assessed
- [ ] Code follows existing patterns
- [ ] Targeted `master` branch

---

## 🔗 Quick Reference Links

### Internal Documentation
- Architecture: `docs/architecture/`
- Change Playbooks: `docs/contributing/change-playbooks.md`
- PR Discipline: `docs/contributing/pr-discipline.md`
- Docs Contract: `docs/contributing/docs-contract.md`

### External Resources
- Rust Book: https://doc.rust-lang.org/book/
- Tokio Docs: https://tokio.rs/
- Serde Guide: https://serde.rs/

---

## 📊 Codebase Metrics

| Metric | Count |
|--------|-------|
| Total Lines of Rust | ~90,000+ |
| Total Lines of Docs | ~15,000+ |
| Source Files (.rs) | 349 |
| Test Files | 52 |
| Modules | 31 |
| Tools | 75 |
| Channels | 42 |
| Providers | 16 |
| Dependencies | ~150 |

---

## 🏷️ Module Tags (for AI search)

**Core**: agent, config, security, runtime  
**I/O**: tools, channels, gateway  
**Storage**: memory, cron  
**Auth**: auth, pairing, secret_store  
**Infra**: observability, tunnel, nodes  
**Advanced**: plugins, sop, skills, verifiable_intent  
**Meta**: doctor, health, heartbeat, onboard  

---

**Last Updated**: 2026-03-29  
**Termux Version**: 1.0 (Android-first runtime)  
**Branch**: `master`

---

*For questions or improvements to this index, see `docs/contributing/docs-contract.md` for documentation standards.*
