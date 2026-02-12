# Clean Architecture in MCB

Last updated: 2026-02-12
Source: `docs/architecture/CLEAN_ARCHITECTURE.md` (v0.2.2)

## Overview

MCB follows strict **Clean Architecture** with 6 layers across 8 crates.
Dependencies flow **INWARD**: `server -> infrastructure -> application -> domain`.

## The 6 Layers

1.  **Layer 6: Server** (`mcb-server`)
    *   **Role:** MCP protocol, HTTP/Stdio transport, tool handlers.
    *   **Deps:** Domain, Application, Infrastructure.
2.  **Layer 5: Infrastructure** (`mcb-infrastructure`)
    *   **Role:** DI container (`dill`), Config (`figment`), Logging (`tracing`), Metrics.
    *   **Deps:** Domain, Application, Providers.
3.  **Layer 4: Application** (`mcb-application`)
    *   **Role:** Use cases, Orchestration, Service implementation.
    *   **Registry:** Defines `linkme` slices (`EMBEDDING_PROVIDERS`).
    *   **Deps:** Domain.
4.  **Layer 3: Providers** (`mcb-providers`)
    *   **Role:** Adapters for external systems (OpenAI, SQLite, Git2).
    *   **Pattern:** Implements Domain Ports; registers via `linkme`.
    *   **Deps:** Domain, Application (for registry).
5.  **Layer 2: Domain** (`mcb-domain`)
    *   **Role:** Pure business rules, Entities, Port Traits (Interfaces).
    *   **Deps:** `thiserror`, `serde` (std only). NO external I/O.
6.  **Layer 1: Facade** (`mcb`)
    *   **Role:** Public API re-exports.

## Key Patterns

*   **Port/Adapter:** Domain defines traits (Ports); Providers implement them (Adapters).
*   **Dependency Injection:** `mcb-infrastructure` builds `dill::Catalog` to wire Services with Providers at runtime.
*   **Auto-Registration:** Providers self-register into `distributed_slice`s defined in Application.

## Extension Guide

*   **New Provider:** Implement Port (Domain) in `mcb-providers`, register via `#[linkme]`.
*   **New Service:** Define in `mcb-application`, register in DI catalog.
*   **New Transport:** Add handler in `mcb-server`, use Services via DI.
