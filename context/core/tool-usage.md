# Tool Usage

Last updated: 2026-02-15 (America/Sao_Paulo)

## Purpose

MCB-specific tool selection rules for efficient repository work.

## File Operations

| Task | Tool | Notes |
|------|------|-------|
| Read file | `read` | Always first — never guess contents |
| Find files | `glob` | Pattern-based (`**/*.rs`, `crates/*/src/**`) |
| Search content | `grep` | Regex-capable, filter by `*.rs` etc. |
| AST patterns | `ast-grep` | Structural search (`fn $NAME($$$) { $$$ }`) |
| Semantic search | `mcb_search` | Concept-level ("error handling in providers") |
| Edit file | `edit` | Surgical replacement with context |

## Build & Quality Commands

| Command | Purpose | When |
|---------|---------|------|
| `make build` | Debug build | Before running locally |
| `make build RELEASE=1` | Release build | Before E2E or benchmarks |
| `make lint` | fmt --check + clippy | After any code edit |
| `make lint FIX=1` | Auto-fix fmt + clippy | Before committing |
| `make lint MCB_CI=1` | CI-strict (Rust 2024 lints) | Match CI behavior |
| `make test` | All workspace tests | Before completion |
| `make test SCOPE=unit` | Unit tests only | Quick feedback loop |
| `make test SCOPE=golden` | Golden acceptance tests | After behavioral changes |
| `make test SCOPE=startup` | DDL/init smoke test | After config/DI changes |
| `make test SCOPE=integration` | Integration tests | After provider changes |
| `make test SCOPE=e2e` | Playwright browser tests | After UI/admin changes |
| `make validate` | Architecture validation (mcb-validate) | After structural changes |
| `make validate QUICK=1` | Fast architecture check | Quick boundary verify |
| `make audit` | Security audit (cargo-audit + udeps) | Before release |
| `make coverage` | HTML coverage report | On demand |
| `make fmt` | Format Rust + Markdown | Mutating — pre-commit |

## MCB CLI Commands

```bash
# Validate codebase architecture
cargo run --package mcb -- validate . --format json

# Quick validate
cargo run --package mcb -- validate . --quick --format json
```

## Search Strategy (MCB-Specific)

1. **Known pattern**: `grep -r "pattern" crates/` or `ast-grep`.
2. **Concept search**: `mcb_search(resource="code", query="...")`.
3. **Cross-crate flow**: `explore` agent with specific crate boundaries.
4. **External API**: `librarian` agent for crate docs.

## Beads (Task Tracking)

- All `bd` commands via `bash` tool (no direct MCP tool).
- Always use `--json` flag for structured output.
- Key commands: `bd ready`, `bd create`, `bd update`, `bd close`, `bd sync`.

## Anti-Patterns

- Do not use `cat` / `head` / `tail` — use `read` tool instead.
- Do not use `find` — use `glob` tool instead.
- Do not use `grep` in bash — use `grep` tool instead.
- Do not run `cargo test` directly — use `make test SCOPE=...` for consistency.
- Do not edit without reading first — `edit` tool will reject.

## Sources

- `Makefile`, `make/dev.mk`, `make/quality.mk`
- `crates/mcb/src/main.rs` (CLI entry)
- `.github/workflows/ci.yml` (CI pipeline)

## Update Notes

- 2026-02-15: Full rewrite with MCB Make targets, CLI commands, and search strategy.
- 2026-02-11: Initial generic baseline.
