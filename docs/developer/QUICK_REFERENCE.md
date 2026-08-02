<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->

# MCB Quick Reference

One-pager for daily MCB development.

**Version:** `0.4.0` | **Rust:** `1.92+` | **Edition:** `2024`

## Make verbs (90% of daily use)

| Task | Command |
|------|---------|
| Build | `make build` / `make build RELEASE=1` |
| Test all | `make test` |
| Unit tests | `make test SCOPE=unit` |
| Lint | `make check WHAT=lint` |
| Auto-format | `make check WHAT=fix ACT=fmt APPLY=Y` |
| Architecture check | `make check WHAT=validate QUICK=1` |
| Banned-pattern scan | `make check WHAT=guard` |
| Full CI gate | `make check WHAT=ci` |
| Lint docs | `make build WHAT=docs ACT=lint` |
| Validate docs | `make build WHAT=docs ACT=validate QUICK=1` |
| Git status | `make ship WHAT=status` |
| Commit | `make ship WHAT=commit MSG='...' APPLY=Y` |
| Push | `make ship WHAT=push APPLY=Y` |
| Bootstrap | `make boot` |

> Destructive verbs require `APPLY=Y`.

## Architecture

```text
mcb-server → mcb-infrastructure → mcb-providers → mcb-domain
mcb-utils (leaf, no mcb-* deps)
mcb-validate (developer tooling)
```

- **Domain** defines ports and entities.
- **Providers** implement domain ports.
- **Infrastructure** wires everything via DI.
- **Server** exposes handlers and admin UI.

## Good / bad in 10s

| ✅ Do this | ❌ Never this |
|-----------|--------------|
| `Result<T>` + `?` propagation | `unwrap()`/`expect()`/`panic!()` in prod |
| `Error::embedding("...")` | Raw `Error::ProviderError { ... }` |
| `tracing::info!(...)` | `println!()`/`eprintln!()` in prod |
| Domain ports in handlers | Concrete providers in handlers |
| `define_id!(SessionId)` | Raw `String`/`Uuid` as domain IDs |
| Tests in `tests/` directory | Inline `#[cfg(test)]` for integration tests |
| `make check WHAT=guard` before commit | `TODO`/`FIXME`/`todo!()` in committed code |

## Beads workflow

```bash
bd prime
bd ready
bd update <id> --claim
# edit
make check WHAT=lint && make test SCOPE=unit
bd close <id> --reason "lint + unit tests passed"
```

## Project skills

Load `/skill:mcb-patterns` (or `/skill:<name>`) as the central index, or the domain skill directly:

- `/skill:mcb-make-verbs`
- `/skill:mcb-architecture-layers`
- `/skill:mcb-error-handling`
- `/skill:mcb-import-rules`
- `/skill:mcb-testing-patterns`
- `/skill:mcb-quality-gates`

Project skills live under `.agents/skills/`.

## Pre-commit validation

```bash
make check WHAT=lint
make test SCOPE=unit
make check WHAT=validate QUICK=1
make check WHAT=guard
```

## Key links

- [AGENTS.md](../../AGENTS.md) — project agent law
- [CONTRIBUTING.md](./CONTRIBUTING.md) — full contribution guide
- [PATTERNS.md](../architecture/PATTERNS.md) — architecture patterns
- [FLEXT_TO_MCB_MAPPING.md](./FLEXT_TO_MCB_MAPPING.md) — FLEXT translation
- [SKILL_INDEX.md](./SKILL_INDEX.md) — project skills
- [MCP_TOOLS.md](../MCP_TOOLS.md) — public MCP API
