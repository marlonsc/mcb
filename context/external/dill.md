# dill

Last updated: 2026-02-12

## Executive Summary

`dill` is the IoC backbone of MCB's composition root. In this repository, it is used in a deliberately explicit style (`CatalogBuilder::new().add_value(...).build()`) to preserve architectural clarity, runtime provider switching, and predictable dependency wiring.

Primary implementation:

- `crates/mcb-infrastructure/src/di/catalog.rs:38`
- `crates/mcb-infrastructure/src/di/catalog.rs:128`

Primary architecture decision:

- `docs/adr/029-hexagonal-architecture-dill.md`

## Context7 + External Research Notes

Context7 lookup for a dedicated `dill` package did not return a direct dill index entry during this session. Therefore, this document uses authoritative primary sources:

- Official Rust docs: `docs.rs/dill`
- Upstream repository: `sergiimk/dill-rs`
- In-repo ADRs and production usage

This keeps the analysis evidence-based even when Context7 coverage is incomplete.

## What dill Provides (Official Capability Surface)

Based on `dill` 0.15 documentation and examples:

- `Catalog` and `CatalogBuilder` for dependency registration/resolution
- Interface binding patterns (`bind::<dyn Trait, Impl>()`)
- Scope control (`Singleton`, `Transient`)
- Macro-assisted wiring (`#[component]`, `#[interface]`, `#[scope]`)
- Delayed construction patterns (`Lazy`, builder parametrization)

Reference: https://docs.rs/dill/latest/dill/

## Actual MCB Usage (Current Source of Truth)

MCB does **not** rely primarily on macro-driven auto-registration for service composition. Instead, it uses explicit value registration in the infrastructure composition root.

### Registered dependency groups in catalog

- Config and environment-backed app settings (`Arc<AppConfig>`)
- Provider trait objects (`Arc<dyn EmbeddingProvider>`, etc.)
- Runtime-switch handles (`EmbeddingProviderHandle`, `VectorStoreProviderHandle`, ...)
- Resolver layer (`EmbeddingProviderResolver`, `CacheProviderResolver`, ...)
- Admin service layer (`EmbeddingAdminInterface`, `VectorStoreAdminInterface`, ...)
- Infrastructure services (`EventBusProvider`, `ShutdownCoordinator`, metrics/indexing interfaces)

### Why this explicit style is intentional

- Enforces clean ownership of composition in `mcb-infrastructure`
- Keeps runtime provider switching behavior explicit and auditable
- Avoids implicit construction surprises in multi-crate boundaries

## ADR-Critical Analysis

### ADR-029 (implemented architecture)

- Declares dill-based IoC in a hexagonal architecture
- Keeps ports in `mcb-domain`, implementations in `mcb-providers`, composition in infrastructure
- Aligns with linkme discovery + handle-based runtime switching

File: `docs/adr/029-hexagonal-architecture-dill.md`

### ADR-024 (superseded but still instructive)

- Documents migration constraints and early friction points
- Explicitly notes limitations around certain patterns and why handle-based composition remained central

File: `docs/adr/024-simplified-dependency-injection.md`

### Practical consequence for contributors

Do not treat dill as a generic "auto DI magic" layer in MCB. Treat it as a controlled container in a strict architecture with explicit boundaries.

## GitHub Evidence (Upstream + In-Repo)

Upstream dill examples (capability reference):

- https://github.com/sergiimk/dill-rs
- https://docs.rs/dill/latest/dill/

MCB production usage (actual behavior):

- https://github.com/marlonsc/mcb/blob/main/crates/mcb-infrastructure/src/di/catalog.rs
- `crates/mcb-infrastructure/src/di/mod.rs:13`

## Strengths, Risks, and Tradeoffs

### Strengths

- Predictable composition and traceable dependency graph
- Strong alignment with clean architecture boundaries
- Supports runtime provider replacement through handles/admin services

### Risks

- Boilerplate growth when adding provider families
- Registration drift if related objects (provider + handle + resolver + admin) are not updated together
- Potential confusion for new contributors expecting macro-centric DI patterns

### Tradeoff chosen by MCB

MCB prefers explicitness and operability over reduced boilerplate. This is a valid choice for a multi-crate system with runtime switching requirements.

## Contributor Checklist (When Adding/Changing Services)

1. Add/adjust domain port in `mcb-domain` if the contract changes.
2. Update concrete adapter in `mcb-providers`.
3. Update resolver + handle + admin service in infrastructure.
4. Register all required values in `build_catalog()`.
5. Add integration tests validating catalog build and retrieval.
6. Verify ADR consistency if architecture behavior changes.

## Anti-Patterns to Avoid

- Putting business logic inside DI/bootstrap modules
- Registering concrete-only interfaces where trait objects are required by architecture
- Adding service entries without corresponding runtime handle/admin plumbing
- Introducing cross-layer imports that violate ADR-029 boundaries

## References

- Dill docs: https://docs.rs/dill/latest/dill/
- Dill crate page: https://crates.io/crates/dill
- Dill repository: https://github.com/sergiimk/dill-rs
- ADR-029: `docs/adr/029-hexagonal-architecture-dill.md`
- ADR-024: `docs/adr/024-simplified-dependency-injection.md`
- ADR-023: `docs/adr/023-inventory-to-linkme-migration.md`
