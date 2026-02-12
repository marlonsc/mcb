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

## Best Practices in MCB

### Port design with async-trait

MCB defines all domain service ports as async traits in `crates/mcb-domain/src/ports/`. These ports are the contracts that provider implementations fulfill. The `#[async_trait]` macro makes these traits object-safe, enabling `Arc<dyn PortTrait>` injection through DI.

Pattern: domain defines the contract → infrastructure/providers implement it → DI wires the concrete to the abstract.

Cross-reference: `context/external/dill.md` for IoC wiring, `context/external/linkme.md` for auto-registration.

### Narrow trait surfaces

Each port trait should cover a single responsibility. MCB separates `SearchService`, `IndexingService`, `MemoryService`, `ValidationService`, etc. rather than bundling operations into a single omnibus trait.

This makes mocking easier (each test only needs to implement the methods it uses) and keeps adapter implementations focused.

### Send + Sync awareness

All async trait objects in MCB must be `Send + Sync` because they're shared across Tokio tasks via `Arc<dyn Trait>`. The `#[async_trait]` macro generates `Send`-bounded futures by default. Use `#[async_trait(?Send)]` only when explicitly needed (none currently in MCB).

### Mock implementation pattern

Test mocks implement the same async trait ports. MCB provides mock implementations in `crates/mcb-server/tests/test_utils/mock_services/`. Each mock stores canned responses and tracks call counts, making assertions straightforward.

## Performance and Safety Considerations

### Allocation overhead

`#[async_trait]` converts async methods to `Box<dyn Future>`, which allocates on the heap. For high-frequency, hot-path operations, this overhead matters. MCB mitigates by:
- Using async traits only at boundary/DI injection points (not inner loops)
- Keeping trait method granularity appropriate (one call per service operation, not per-item)

For performance-critical internal paths, prefer generic `impl Future` over trait objects.

### Rust native async traits (future consideration)

Rust 1.75+ supports native async fn in traits for some cases. However, `dyn Trait` support for native async traits is still limited. MCB continues using `#[async_trait]` because all ports are consumed as `Arc<dyn Trait>`. Monitor RFC progress for eventual migration.

### Trait object downcast limitations

`dyn Trait` objects cannot be downcast without additional machinery (e.g., `downcast-rs`). MCB's architecture avoids downcasting — services interact through the trait interface only.

## Testing and Verification Guidance

### Testing against trait ports

All service tests should test against the trait interface, not the concrete implementation. This validates that the contract is correct and that mocks remain in sync with real implementations.

### Mock constructibility test

MCB has an explicit constructibility test (`crates/mcb-server/tests/test_utils/mock_services/mod.rs:49`) that ensures all mock implementations can be instantiated. This catches compilation regressions when port traits change.

### Verifying trait implementations

When a port trait adds a method, all implementations (real and mock) must be updated. The compiler enforces this — a missing method causes a compile error. MCB leverages this guarantee.

## Operational Risk and Monitoring

| Risk | Impact | Mitigation |
|---|---|---|
| Missing `Send` bound on trait object | Compile error when shared across tasks | Use default `#[async_trait]` (Send-bounded) |
| Broad trait surface | Hard to mock, hard to evolve | Keep traits narrow and focused |
| Allocation overhead on hot path | Performance regression | Use async traits at boundaries only |
| Trait/mock implementation drift | Tests pass but production fails | Constructibility test + integration tests |

## Migration and Version Notes

- MCB uses async-trait (dtolnay) for all domain port definitions.
- ADR-002 (`docs/adr/002-async-first-architecture.md`) mandates async-first architecture.
- ADR-013 (`docs/adr/013-clean-architecture-crate-separation.md`) requires domain ports as trait interfaces.
- When Rust stabilizes `dyn` support for native async traits, MCB can migrate away from the macro. The trait signatures would remain identical — only the `#[async_trait]` annotation would be removed.

## Verification Checklist

- [ ] New port trait uses `#[async_trait]` and lives in `crates/mcb-domain/src/ports/`
- [ ] Trait surface is narrow (single responsibility)
- [ ] All implementations (real + mock) compile after trait changes
- [ ] Mock implementation added in `test_utils/mock_services/`
- [ ] `Send + Sync` bound preserved (default `#[async_trait]`, not `?Send`)
- [ ] No async trait used in performance-critical inner loops
- [ ] Constructibility test updated if new mock added

## Common Pitfalls

- Using async traits for high-frequency, allocation-sensitive internals where generics would be cheaper.
- Hiding broad method surfaces behind one trait; narrow contracts are easier to test and evolve.
- Forgetting `Send + Sync` expectations on trait objects passed across tasks/threads.
- Adding methods to a port trait without updating all mock implementations.
- Attempting to downcast trait objects instead of interacting through the trait interface.

## References

- https://docs.rs/async-trait
- https://github.com/dtolnay/async-trait
- `docs/adr/002-async-first-architecture.md`
- `docs/adr/013-clean-architecture-crate-separation.md`
- `docs/adr/021-dependency-management.md`
- `context/external/dill.md`
- `context/external/linkme.md`
- `context/external/tokio.md`
