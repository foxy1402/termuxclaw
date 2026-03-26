# GitHub Copilot Instructions for ZeroClaw

This file contains essential context for Copilot (and other AI assistants) working in the ZeroClaw repository. For cross-tool instructions, see [`AGENTS.md`](../AGENTS.md).

## Quick Commands

| Task | Command |
|------|---------|
| **Format code** | `cargo fmt --all` |
| **Check formatting** | `cargo fmt --all -- --check` |
| **Lint** | `cargo clippy --all-targets -- -D warnings` |
| **Run all tests** | `cargo test --locked` |
| **Run unit tests only** | `cargo test --lib` |
| **Full CI validation** | `./dev/ci.sh all` |
| **Build release** | `cargo build --release --locked` |
| **Development run** | `cargo run -- <ARGS>` |
| **Check code (no build)** | `cargo check --all-targets` |

**Justfile shortcuts:** All commands above are available as `just <target>`. Run `just` to list all recipes.

## High-Level Architecture

ZeroClaw is a Rust-first autonomous agent runtime with a trait-driven, modular architecture. The core abstractions are:

### Key Architecture Traits

These are the extension points for the system. Implement these traits to add new functionality:

- **`Provider`** (`src/providers/traits.rs`) — Model providers (OpenAI, Anthropic, Gemini, etc.). Route and failover logic in `src/providers/router.rs`.
- **`Channel`** (`src/channels/traits.rs`) — Communication channels (Telegram, Discord, Slack, WhatsApp, Matrix, etc.). Session backends in `src/channels/session_*.rs`.
- **`Tool`** (`src/tools/traits.rs`) — Executable tools available to the agent (shell, file I/O, memory, browser, etc.). ~100+ tools registered in `src/tools/mod.rs`.
- **`Memory`** (`src/memory/traits.rs`) — Storage backends (markdown, SQLite, PostgreSQL, Qdrant, etc.). Supports embeddings, decay, consolidation, and retrieval.
- **`Observer`** (`src/observability/traits.rs`) — Observability/instrumentation (logging, Prometheus, OpenTelemetry, Dora, etc.).
- **`RuntimeAdapter`** (`src/runtime/traits.rs`) — Execution environments (native, WebAssembly, Docker).

### Repository Structure

```
src/
├── main.rs                      # CLI entrypoint and command routing
├── lib.rs                       # Module exports
├── agent/                       # Orchestration loop (dispatcher, classifier, thinking, memory)
├── gateway/                     # Webhook/REST API server (WebSocket, SSE, static files, auth)
├── security/                    # Policy, pairing, secrets, sandboxing (firejail, landlock, bubblewrap)
├── memory/                      # Backends + embeddings/vector merge/decay/consolidation
├── providers/                   # Model providers + resilient wrapper
├── channels/                    # Communication backends (50+ channels)
├── tools/                       # Tool execution surface (~100 tools)
├── runtime/                     # Runtime adapters (native/wasm/docker)
├── config/                      # Schema + config loading/merging
├── onboard/                     # Interactive setup wizard
├── plugins/                     # WASM plugin system
├── security/                    # Authorization, secrets, sandboxing
├── hooks/                       # Lifecycle hooks (command logger, audit)
├── cron/                        # Scheduled task execution
├── observability/               # Instrumentation backends
├── auth/                        # OAuth/token management
└── [15+ more subsystems]
```

### Factory Registration Pattern

New providers, channels, tools, and memory backends are registered in factory modules. Example:

- **Providers**: `src/providers/mod.rs` — add `fn_<name>()` factory and match arm in `factory()`.
- **Channels**: `src/channels/mod.rs` — same pattern.
- **Tools**: `src/tools/mod.rs` — register in `BUILTIN_TOOLS` vec.
- **Memory**: `src/memory/mod.rs` — register in `factory()`.

Always register new implementations before they can be instantiated.

## Key Conventions

### Branching & Contribution Workflow

- **Default branch**: `master` (NOT `main`). The `main` branch no longer exists (deleted March 2026).
- **Create branches from `master`**: Use naming `feat/*` or `fix/*`.
- **All PRs target `master`**: PRs targeting `main` will be rejected.
- **Enable pre-push hook**: `git config core.hooksPath .githooks` — runs fmt, clippy, and tests automatically.

See [`CONTRIBUTING.md`](../CONTRIBUTING.md) for branch migration instructions if you have an older fork.

### Risk Classification (from `AGENTS.md`)

Classify your changes to validate appropriately:

| Risk Level | Examples | Validation |
|-----------|----------|-----------|
| **Low** | docs, tests-only, chores | lightweight checks (formatting, basic lint) |
| **Medium** | most src/ behavior changes (no boundary/security impact) | full test suite + clippy |
| **High** | src/security/*, src/runtime/*, src/gateway/*, src/tools/*, .github/workflows/*, access-control boundaries | full CI suite + design review |

When uncertain, classify higher.

### PR Template & Validation

Use the [PR template](./pull_request_template.md). Key sections:

- **Label Snapshot**: risk (`low|medium|high`), size (`XS|S|M|L|XL`), scope, module labels.
- **Change Metadata**: change type (`bug|feature|refactor|docs|security|chore`), primary scope.
- **Validation Evidence**: Include output from `cargo fmt`, `cargo clippy`, `cargo test`.
- **Security Impact**: Declare any new permissions, network calls, secret handling, or file system scope changes.
- **Compatibility**: Explain if backward compatible, config/env changes, migration steps.
- **i18n Follow-Through** (if user-facing): Ensure locale parity for `en`, `zh-CN`, `ja`, `ru`, `fr`, `vi`.

**Supersede Attribution** (if replacing an older PR): Include `Co-authored-by` trailers for materially incorporated contributors.

### Code Organization

- **One concern per PR**: Avoid mixing feature + refactor + infra patches.
- **Minimal patch**: No speculative abstractions or config keys without concrete use cases.
- **Read before write**: Inspect existing module, factory wiring, and adjacent tests before editing.
- **No speculative dependencies**: Dependencies must solve a concrete problem.
- **Weak constraints**: Do not weaken security policy or access constraints.
- **Privacy**: Never commit secrets, personal data, or real identity information (see `docs/contributing/pr-discipline.md`).

## Testing & Validation

### Running Tests

```bash
# Full suite (required before PR)
cargo test --locked

# Unit tests only (faster during development)
cargo test --lib

# Specific test (replace MODULE with module name)
cargo test --lib MODULE -- --nocapture

# Run with logging
RUST_LOG=debug cargo test --lib -- --nocapture
```

### Pre-PR Quality Gate

```bash
# All at once (replicates CI)
./dev/ci.sh all

# Or individually
cargo fmt --all -- --check      # Format check
cargo clippy --all-targets -- -D warnings  # Lint
cargo test --locked             # Tests
```

**Pre-push hook** (recommended): Automatically enforces these before every push:

```bash
git config core.hooksPath .githooks
```

### Local CI Against Docker (if available)

```bash
./dev/ci.sh all  # Runs full CI suite in Docker (if configured)
```

## Module-Specific Patterns

### Adding a New Provider

1. Create `src/providers/<name>.rs` implementing `Provider` trait.
2. Add factory function `pub fn fn_<name>(...) -> Result<Box<dyn Provider>>`.
3. Register in `src/providers/mod.rs`:
   - Import module and factory.
   - Add match arm in `factory()` function.
4. Add tests in same file (or `src/providers/<name>_tests.rs`).
5. Update `docs/reference/api/providers-reference.md` if user-facing.

See `src/providers/anthropic.rs`, `src/providers/openai.rs` for examples.

### Adding a New Channel

1. Create `src/channels/<name>.rs` implementing `Channel` trait.
2. Add factory function `pub fn fn_<name>(...) -> Result<Box<dyn Channel>>`.
3. Register in `src/channels/mod.rs`:
   - Import module and factory.
   - Add match arm in `factory()` function.
4. If session state needed, create `src/channels/session_<name>.rs`.
5. Add tests and update documentation.

See `src/channels/telegram.rs`, `src/channels/discord.rs` for examples.

### Adding a New Tool

1. Create `src/tools/<name>.rs` implementing `Tool` trait.
2. Add to `BUILTIN_TOOLS` vec in `src/tools/mod.rs`.
3. Document parameters, returns, and error cases in the `Tool::schema()` method.
4. Add integration tests in `src/tools/<name>_tests.rs` if complex.

See `src/tools/shell.rs`, `src/tools/file_read.rs` for examples.

### Adding a New Memory Backend

1. Create `src/memory/<name>.rs` implementing `Memory` trait.
2. Add factory function `pub fn fn_<name>(...) -> Result<Box<dyn Memory>>`.
3. Register in `src/memory/mod.rs` factory.
4. Support embeddings retrieval if applicable (see `src/memory/embeddings.rs`).
5. Add audit trail support (see `src/memory/audit.rs`).

See `src/memory/sqlite.rs`, `src/memory/postgres.rs` for examples.

## Security Considerations

### High-Risk Modules (require extra scrutiny)

- **`src/security/`** — Policy, pairing, secrets, sandboxing.
- **`src/runtime/`** — Execution environments.
- **`src/gateway/`** — Public-facing REST API and webhooks.
- **`src/tools/`** — Access to shell, file system, and external APIs.
- **`.github/workflows/`** — CI/CD pipeline.

### Anti-Patterns

- ❌ Do not add heavy dependencies for minor convenience.
- ❌ Do not silently weaken security policy or access constraints.
- ❌ Do not add speculative config/feature flags "just in case".
- ❌ Do not modify unrelated modules "while here".
- ❌ Do not bypass failing checks without explicit explanation.
- ❌ Do not hide behavior-changing side effects in refactor commits.
- ❌ Do not include personal identity or sensitive information in test data, examples, or commits.

## Documentation

- **Setup guides**: `docs/setup-guides/` — installation and onboarding.
- **Reference**: `docs/reference/` — API, config schema, provider/channel reference.
- **Operations**: `docs/ops/` — deployment, monitoring, troubleshooting.
- **Security**: `docs/security/` — policy, secrets, sandboxing.
- **Contributing**: `docs/contributing/` — change playbooks, PR discipline, docs contract.
- **Maintainers**: `docs/maintainers/` — release process, governance.
- **Architecture**: `docs/architecture/` — design deep-dives.
- **Superpowers**: `docs/superpowers/` — advanced features.

Docs follow a **locale parity** contract (see `docs/contributing/docs-contract.md`). English (`en`) is canonical; maintain parity for `zh-CN`, `ja`, `ru`, `fr`, `vi`.

## Workspace Structure

This is a Cargo workspace with:

- **Root crate** (`zeroclawlabs`) — main binary and library.

Build against the root: `cargo build --release --locked`.

## Checklist Before Opening a PR

- [ ] Branched from `master` (NOT `main`)
- [ ] Ran `cargo fmt --all`
- [ ] Ran `cargo clippy --all-targets -- -D warnings`
- [ ] Ran `cargo test --locked` (all pass)
- [ ] Filled PR template completely (risk, size, labels, scope)
- [ ] No secrets or personal data in commits
- [ ] Backward compatible OR migration steps documented
- [ ] i18n follow-through if user-facing wording changed
- [ ] Security impact assessed (new permissions, network calls, etc.)
- [ ] Code follows existing patterns (trait-driven, factory registration, etc.)
- [ ] Targeted `master` branch (not `main`)

## Useful References

- **`AGENTS.md`** — Cross-tool agent instructions (workflow, risk tiers, anti-patterns).
- **`CONTRIBUTING.md`** — Full contributing guide, branch migration, first-time contributor flow.
- **`docs/contributing/change-playbooks.md`** — Detailed guides for adding providers, channels, tools; security/gateway changes.
- **`docs/contributing/pr-discipline.md`** — Privacy rules, superseded-PR attribution templates.
- **`docs/contributing/docs-contract.md`** — Documentation system contract, i18n rules.

## Rust Version & Tooling

- **Rust**: 1.87 stable (MSRV in `Cargo.toml`).
- **Code formatter**: `cargo fmt` (rustfmt, configured in `rustfmt.toml`).
- **Linter**: `cargo clippy` (rules in `clippy.toml`).
- **TOML formatter**: `taplo format` (if you modify `.toml` files).
- **Dependency audit**: `cargo audit` (check for vulnerabilities).
- **License check**: `cargo deny` (ensure compliant dependencies).

All checks must pass before merge.

## MCP Server Configuration

Model Context Protocol (MCP) servers enhance Copilot's capabilities for this repository:

### Filesystem MCP

Enables advanced file exploration across the large ZeroClaw codebase.

**Configuration (Claude Code, Cline, or Windsurf):**
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/path/to/zeroclaw"],
      "disabled": false
    }
  }
}
```

Useful for:
- Navigating large module hierarchies (50+ channels, 100+ tools).
- Understanding cross-module dependencies.
- Exploring factory registration patterns.

### Git MCP

Provides commit history, branch tracking, and change context.

**Configuration:**
```json
{
  "mcpServers": {
    "git": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-git", "/path/to/zeroclaw"],
      "disabled": false
    }
  }
}
```

Useful for:
- Reviewing historical context before modifying a module.
- Understanding why a pattern was chosen (check commit history).
- Validating that your change follows established conventions.
- Cross-referencing related PRs and issues.

Both MCP servers are optional but recommended for projects of ZeroClaw's complexity.
