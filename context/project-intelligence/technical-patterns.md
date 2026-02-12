# Technical Patterns

Last updated: 2026-02-11
Source baseline: `docs/context/technical-patterns.md`

Architecture:

- Clean Architecture flow: `mcb-server -> mcb-infrastructure -> mcb-application -> mcb-domain`
- Provider implementations in `mcb-providers` depend on domain ports only.

Dependency injection stack:

1. `linkme` distributed slices for compile-time discovery
2. `dill` catalog wiring at runtime
3. `Handle<T>` for runtime provider switching

High-value patterns:

- Registry macros in `mcb-domain/src/registry/` for provider discovery.
- Error factories in `mcb-domain/src/error/mod.rs` (`Error::io`, `Error::embedding`, etc.).
- `define_id!` macro for strong typed IDs in `mcb-domain/src/value_objects/ids.rs`.
- Decorator pattern in `mcb-application/src/decorators/`.

Implementation guidance:

- New features follow existing crate layering; avoid cross-layer leakage.
- New providers should self-register via distributed slices.
- Avoid `unwrap()` and `expect()` outside tests.

Related:

- `context/project-intelligence/domain-concepts.md`
- `context/project-intelligence/integrations.md`
- `context/project-intelligence/conventions.md`
