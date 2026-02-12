# async-trait

Last updated: 2026-02-12

## Executive Summary

`async-trait` is the core bridge that lets MCB keep async port interfaces in the domain layer while still using trait objects (`Arc<dyn Trait>`) across DI and adapter boundaries.

## Context7 + External Research

- Context7 ID: `/dtolnay/async-trait`
- Primary docs: https://docs.rs/async-trait
- Upstream: https://github.com/dtolnay/async-trait
- Why it matters in MCB: clean-architecture ports are async-first (ADR-002), and many provider/service contracts are injected behind trait objects.

## Actual MCB Usage (Current Source of Truth)

### 1) Domain ports are async traits

- `crates/mcb-domain/src/ports/services/search.rs:1`
- `crates/mcb-domain/src/ports/services/memory.rs:1`
- `crates/mcb-domain/src/ports/services/indexing.rs:3`
- `crates/mcb-domain/src/ports/providers/vcs.rs:5`

Pattern: domain defines async contracts in ports; infrastructure/providers implement them.

### 2) Providers implement async port contracts

- `crates/mcb-providers/src/git/git2_provider.rs:5`
- `crates/mcb-providers/src/cache/moka.rs:22`
- `crates/mcb-providers/src/vector_store/qdrant.rs:13`
- `crates/mcb-providers/src/embedding/openai.rs:8`

Pattern: adapter implementations stay outside domain and fulfill async contracts from ports.

### 3) Server and infra extension points rely on async trait boundaries

- `crates/mcb-infrastructure/src/routing/router.rs:8`
- `crates/mcb-infrastructure/src/project/service.rs:7`
- `crates/mcb-server/src/admin/web/router.rs:8`

Pattern: routing, project services, and admin components remain swappable through async interfaces.

## ADR Alignment (Critical)

- ADR-002 (`docs/adr/002-async-first-architecture.md`): async-first architecture is mandatory; trait-based async contracts are part of the baseline.
- ADR-013 (`docs/adr/013-clean-architecture-crate-separation.md`): domain stays dependency-light while ports define boundaries and adapters implement them.
- ADR-021 (`docs/adr/021-dependency-management.md`): `async-trait` is a workspace-managed foundational dependency.

## GitHub Evidence (Upstream + In-Repo)

- Upstream crate: https://github.com/dtolnay/async-trait
- Canonical docs/tests: https://github.com/dtolnay/async-trait/tree/master/tests
- In-repo anchor: `crates/mcb-domain/src/ports/services/validation.rs:3`
- In-repo anchor: `crates/mcb-providers/src/hybrid_search/engine.rs:30`

## Common Pitfalls

- Using async traits for high-frequency, allocation-sensitive internals where generics would be cheaper.
- Hiding broad method surfaces behind one trait; narrow contracts are easier to test and evolve.
- Forgetting `Send + Sync` expectations on trait objects passed across tasks/threads.

## References

- https://docs.rs/async-trait
- https://github.com/dtolnay/async-trait
- `docs/adr/002-async-first-architecture.md`
- `docs/adr/013-clean-architecture-crate-separation.md`
- `docs/adr/021-dependency-management.md`
