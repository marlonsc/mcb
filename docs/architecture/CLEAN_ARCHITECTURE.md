<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Clean Architecture in MCB

> **SSOT**: The full architecture specification lives in
> [ARCHITECTURE.md](./ARCHITECTURE.md). Implementation patterns are in
> [PATTERNS.md](./PATTERNS.md). This file exists as a stable link target only.

## Quick Reference

| Topic | Document |
| --- | --- |
| Layer definitions (Domain → Server) | [ARCHITECTURE.md § Layers](./ARCHITECTURE.md#the-clean-architecture-layers) |
| Dependency direction & crate graph | [ARCHITECTURE.md § Dependency Rules](./ARCHITECTURE.md#dependency-rules) |
| Port categories & trait locations | [ARCHITECTURE.md § Port Categories](./ARCHITECTURE.md#port-categories) |
| Extension patterns (new providers) | [ARCHITECTURE.md § Extensibility](./ARCHITECTURE.md#extensibility---adding-new-providers) |
| Two-layer DI (linkme → Handle) | [PATTERNS.md § Two-Layer DI](./PATTERNS.md#two-layer-di-linkme--handle-adr-050) |
| Error factory pattern | [PATTERNS.md § Error Handling](./PATTERNS.md#error-handling) |
| Boundary enforcement rules | [ARCHITECTURE_BOUNDARIES.md](./ARCHITECTURE_BOUNDARIES.md) |
