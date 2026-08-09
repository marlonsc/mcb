# Project State

Last updated: 2026-02-15 (America/Sao_Paulo)

## Snapshot

- **Version**: 0.2.1-dev | **Rust**: Edition 2024, nightly, MSRV 1.92
- **Codebase**: 7 workspace crates, 1448 source files, ~159K lines, 1700+ tests
- **MCP Tools**: 9 stable tools (index, search, validate, memory, session, agent, project, vcs, entity)
- **Architecture**: Clean Architecture enforced at compile time via linkme + dill DI
- **Providers**: 6 embedding, 5 vector store, SQLite DB, Moka/Redis cache, Tokio/NATS events
- **Admin UI**: Rocket + Handlebars + Alpine.js with browse/highlight/SSE
- **Deployment**: systemd, Docker, full Kubernetes manifests (k8s/)
- **CI**: GitHub Actions (lint → test → smoke → validate → golden → audit → coverage → release)

## Quality Gates

- `make lint` — Zero clippy warnings, consistent rustfmt
- `make test` — 1700+ tests green (unit/integration/golden/e2e)
- `make validate` — Zero architecture violations (mcb-validate, 349+ validate tests)
- No `unwrap()`/`expect()` in production, `unsafe_code = "deny"`
- Playwright E2E for admin UI

## Active Signals

- Architecture validation (`mcb-validate`) is the core quality gate
- Beads (`bd`) is the canonical issue tracker
- Context system planning: ADR-034..037 (workflow) and ADR-041..046 (integrated context)
- Pre-v0.3.0 closure audit active

## Planned Versions

- **v0.3.0**: Workflow FSM, context scout, policy enforcement, orchestrator
- **v0.4.0**: Knowledge graph, hybrid search (semantic + BM25), time-travel queries

## Sources

- `README.md`, `Cargo.toml`, `docs/developer/ROADMAP.md`
- `.github/workflows/ci.yml`, `Makefile`, `make/*.mk`

## Update Notes

- 2026-02-15: Full harvest rewrite — comprehensive project state from codebase analysis.
- 2026-02-12: Added closure-audit snapshot.
