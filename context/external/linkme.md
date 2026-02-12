# linkme

Last updated: 2026-02-12

## Executive Summary

`linkme` is the compile-time registration mechanism used by MCB to discover providers without runtime plugin scanning. MCB declares distributed slices in `mcb-domain`, registers entries in `mcb-providers`, and resolves selected providers through infrastructure resolvers.

Primary implementation paths:

- `crates/mcb-domain/src/registry/mod.rs`
- `crates/mcb-providers/src/embedding/openai.rs`
- `crates/mcb-infrastructure/src/di/provider_resolvers.rs`

## Context7 + External Research

Context7 resolution for this document:

- `/websites/rs_linkme_0_3_35`

Validated capability surface from Context7/docs.rs:

- `#[distributed_slice]` declares a distributed slice.
- `#[distributed_slice(MY_SLICE)]` registers static elements into that slice.
- Registration initializers must be const-compatible static values.
- Elements from downstream crates appear in the final linked binary if crates are linked.

Cross-check references:

- https://docs.rs/linkme/0.3.35/linkme/
- https://github.com/dtolnay/linkme

## Actual MCB Usage (Current Source of Truth)

### 1) Registry declaration in domain

MCB defines domain-specific registries and documents the full registration flow in:

- `crates/mcb-domain/src/registry/mod.rs:14`
- `crates/mcb-domain/src/registry/mod.rs:17`

This file describes the exact lifecycle: provider static entry -> distributed slice -> resolver iteration -> config-based selection.

### 2) Provider auto-registration in adapters

Concrete providers register themselves via distributed slices in `mcb-providers`, for example:

- `crates/mcb-providers/src/embedding/openai.rs:208`
- `crates/mcb-providers/src/embedding/ollama.rs:208`
- `crates/mcb-providers/src/vector_store/qdrant.rs:511`
- `crates/mcb-providers/src/cache/redis.rs:271`

Pattern used in production:

```rust
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OPENAI_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "openai",
    description: "OpenAI embedding provider (...)",
    factory: openai_factory,
};
```

### 3) Runtime selection through resolvers

Infrastructure resolvers convert config into registry configs and call domain resolve functions:

- `crates/mcb-infrastructure/src/di/provider_resolvers.rs:33`
- `crates/mcb-infrastructure/src/di/provider_resolvers.rs:62`
- `crates/mcb-infrastructure/src/di/provider_resolvers.rs:114`

### 4) Link-time inclusion guard

MCB explicitly force-links provider crates in runtime and tests to ensure distributed entries are present in the linked binary:

- `crates/mcb/src/main.rs:21`
- `crates/mcb-application/tests/unit/registry_tests.rs:11`
- `crates/mcb-infrastructure/tests/di/catalog_tests.rs:10`

## ADR Alignment (Critical)

### ADR-023 (core architectural decision)

`ADR-023` is the authoritative migration/architecture decision for this mechanism:

- `docs/adr/023-inventory-to-linkme-migration.md`

It records:

- Migration from `inventory` to `linkme`.
- Distributed-slice declaration and registration pattern.
- Completed migration status for provider families.

### ADR relationships to keep in mind

While ADR-023 defines registration mechanics, the resulting provider wiring is consumed inside the broader architecture:

- `docs/adr/013-clean-architecture-crate-separation.md`
- `docs/adr/029-hexagonal-architecture-dill.md`

Implication: keep registry contracts in domain, entries in providers, and wiring/selection in infrastructure.

## GitHub Evidence (Upstream + In-Repo)

Upstream `linkme` reference:

- https://github.com/dtolnay/linkme

Representative external usage examples:

- https://github.com/risingwavelabs/risingwave/blob/main/src/expr/core/src/sig/udf.rs
- https://github.com/neon-bindings/neon/blob/main/crates/neon/src/macro_internal/mod.rs

MCB production usage anchors:

- https://github.com/marlonsc/mcb/blob/main/crates/mcb-domain/src/registry/mod.rs
- https://github.com/marlonsc/mcb/blob/main/crates/mcb-providers/src/embedding/openai.rs
- https://github.com/marlonsc/mcb/blob/main/crates/mcb-infrastructure/src/di/provider_resolvers.rs
- https://github.com/marlonsc/mcb/blob/main/docs/adr/023-inventory-to-linkme-migration.md

## Practical Contributor Checklist

When adding a new provider family or provider implementation:

1. Declare/extend the correct distributed slice in `mcb-domain` registry module.
2. Add a static entry with `#[linkme::distributed_slice(TARGET_SLICE)]` in `mcb-providers`.
3. Ensure resolver/config mapping exists in `mcb-infrastructure` resolver layer.
4. Verify provider crate is linked in binaries/tests that depend on discovery.
5. Add or update integration tests that validate discovery and resolution.

## Common Pitfalls

- **Crate not linked**: entries do not appear at runtime if the registering crate is not linked into the target binary.
- **Type mismatch**: entry type must match the slice element type exactly.
- **Boundary drift**: putting registration/wiring logic in the wrong crate violates ADR-013/029 layering.
- **Config mismatch**: registry entry exists but resolver config maps to an unknown provider name.

## References

- Linkme docs: https://docs.rs/linkme/0.3.35/linkme/
- Linkme repository: https://github.com/dtolnay/linkme
- ADR-023: `docs/adr/023-inventory-to-linkme-migration.md`
- Domain registry docs: `crates/mcb-domain/src/registry/mod.rs`
