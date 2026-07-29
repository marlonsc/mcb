# CLAUDE.md — MCB Project Instructions

Reference [`./AGENTS.md`](./AGENTS.md) for all project rules, architecture,
commands, beads workflow, validation, and Git policy. Do not duplicate those
rules here.

<!-- bd-doctor-divergence: ok -->
<!-- Intentional: this file is a thin pointer to AGENTS.md (the SSOT); their
     content legitimately differs, so the bd-doctor Agent-Doc-Divergence check
     is opted out here rather than syncing the two. -->

---

## Quick Reference

### Essentials

- **Language / toolchain**: Rust 1.92+, edition 2024. Toolchain pinned in
  `rust-toolchain.toml`.
- **Build interface**: `make <verb> [WHAT=phase] [SCOPE=...] [APPLY=Y]`.
  Do not call `cargo` or `git` directly for canonical workflows.
- **SSOT**: `AGENTS.md` > `Cargo.toml` / `Makefile` / `config/*.yaml` > static
  docs.

### Common Commands

| Task | Command |
| ----- | ------- |
| Build release | `make build RELEASE=1` |
| Run dev server | `make dev WHAT=run` |
| Run unit tests | `make test SCOPE=unit` |
| Run all tests | `make test` |
| Lint + format check | `make check WHAT=lint` |
| Architecture validation | `make check WHAT=validate` |
| Full CI gate | `make ci` |
| Banned-pattern scan | `make guard` |
| Docs lint | `make docs WHAT=lint` |
| Pre-commit hook | `make hook WHAT=pre-commit` |

### Workspace Crates

```text
mcb              CLI / Loco app
mcb-server       MCP protocol, handlers, transport
mcb-infrastructure  DI, config, cache, logging, AppContext
mcb-domain       entities, value objects, port traits, errors
mcb-providers    adapters (DB, embeddings, vector store, git, parsers)
mcb-validate     architecture rule engine
mcb-utils        leaf utilities
```

### Must-Know Conventions

- Clean Architecture: inward-only dependencies; `mcb-domain` has no `mcb-*`
  deps; handlers use ports, not concrete providers.
- Error handling: `mcb_domain::error::Error` (`thiserror`) + `Result<T>`;
  no `unwrap`/`expect`/`panic`/`todo` in production paths.
- Provider discovery via `linkme` distributed slices.
- Conventional commits (`feat`, `fix`, `refactor`, `docs`, `test`, `chore`,
  `perf`, `ci`).
- Work is tracked in **beads** (`bd`): `bd ready`, `bd update <id> --claim`,
  `bd close <id> --reason "evidence"`.
- `third-party/` is excluded from the workspace — do not edit unless explicitly
  asked.

### First-Time Onboarding

See [`ONBOARDING.md`](./ONBOARDING.md) for a structured walkthrough of the
stack, architecture, request lifecycle, and where to find things.
