# MCB — Gemini Code Assist Style Guide

> Single source of truth: [`CLAUDE.md`](../CLAUDE.md). This file adds Gemini-specific
> PR review priorities. See [`AGENTS.md`](../AGENTS.md) for the full agent configuration index.

## PR Review Priorities (ordered)

1. **Architecture boundaries** — Does this respect `server -> infra -> app -> domain`? No inner-to-outer imports.
2. **Safety** — No unsafe, no `unwrap()` outside tests, proper `?` error propagation.
3. **Correctness** — Logic errors, edge cases, race conditions.
4. **Performance** — Unnecessary allocations, O(n^2) patterns, blocking on async runtime.
5. **Maintainability** — Naming, modularity, idiomatic Rust.
6. **Security** — Input validation, credential handling, injection vectors.
7. **Tests** — New code has tests. Existing tests not deleted to "pass".

## Architecture Violation Codes

| Code | Rule | Fix |
|------|------|-----|
| CA001 | Layer violation | Move import to correct layer |
| CA002 | Circular dependency | Use DI/ports to break cycle |
| CA007 | Port duplication | Move trait to `mcb-domain` |
| ORG016 | File placement | Move file to correct directory |
| ORG019 | Ignore pattern | Update validation config |

## Forbidden Patterns (flag in review)

- `unsafe` code (workspace `deny`)
- `unwrap()` / `expect()` outside tests
- `#[allow(unused_*)]` hiding dead code
- `todo!()` / `unimplemented!()` in non-draft code
- Empty catch blocks / error-swallowing
- Hardcoded paths or fallback values (use `ConfigLoader`)

## Quality Gate Checklist

- `make lint` passes (clippy + fmt)
- `make test` passes (1700+ tests)
- `make validate` passes (zero architecture violations)
- Conventional Commits format used

## Key Documentation

- [`CLAUDE.md`](../CLAUDE.md) — all rules, patterns, commands (single source of truth)
- [`docs/architecture/ARCHITECTURE.md`](../docs/architecture/ARCHITECTURE.md) — architecture details
- [`docs/developer/CONTRIBUTING.md`](../docs/developer/CONTRIBUTING.md) — contributor guide
- [`docs/adr/`](../docs/adr/) — 48 Architecture Decision Records
