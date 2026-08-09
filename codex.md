# Codex Instructions — MCB

All project rules, architecture, conventions, and commands are defined in
[`CLAUDE.md`](CLAUDE.md) at the repository root. Follow it as the single source of truth.

See [`AGENTS.md`](AGENTS.md) for the full agent configuration index.

## Essential Rules

- **Architecture**: Clean Architecture — dependencies flow inward only. Run `make validate` to verify.
- **Error handling**: Use `Error::vcs("msg")` constructors, never `unwrap()`/`expect()` in production.
- **Lints**: `unsafe_code = "deny"`, `dead_code = "deny"`. Zero clippy warnings required.
- **Testing**: `make test` (1700+ tests). New logic must include tests.
- **Build**: Always use `make` targets (`make build`, `make lint`, `make test`, `make check`).
- **Commits**: Conventional Commits format — `feat(scope): description`.
- **Change philosophy**: Surgical edits, maximum reuse, no bypasses. Fix all warnings every cycle.
