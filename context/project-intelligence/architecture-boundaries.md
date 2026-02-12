# Architecture Boundaries

Last updated: 2026-02-12
Source: `docs/architecture/ARCHITECTURE_BOUNDARIES.md` (v0.2.1)

## Dependency Rules

**Critical:** Dependencies ALWAYS point inward.

*   `mcb-domain`: Zero internal dependencies. No `sqlx`, no `rocket`, no `git2`.
*   `mcb-application`: Orchestrates Domain. No direct usage of `mcb-providers` (use Ports).
*   `mcb-providers`: Adapters only. No dependency on `mcb-infrastructure` or `mcb-server`.
*   `mcb-infrastructure`: Wires everything. Can see all layers except Server.
*   `mcb-server`: Entry point. Can see all layers (via DI).

## Module Ownership

| Concept | Owner | Importers |
| :--- | :--- | :--- |
| **Port Traits** | `mcb-domain` | App, Providers |
| **Entities** | `mcb-domain` | All Layers |
| **Services** | `mcb-application` | Infra, Server |
| **Providers** | `mcb-providers` | Infra (DI) |
| **DI Catalog** | `mcb-infrastructure` | Server |
| **Tool Handlers** | `mcb-server` | None |

## Common Violations (to avoid)

*   **CA001 (Layer Violation):** Domain importing Application types.
*   **CA002 (Circular Dep):** Application -> Infrastructure -> Application. Use DI/Ports to break.
*   **CA007 (Port Duplication):** Defining ports in Application. Move to Domain.
*   **Leakage:** Exposing `sqlx` types in Domain entities.

## Validation

*   Use `mcb-validate` (if available) or `cargo deny` rules.
*   Manual check: `grep` imports in `crates/mcb-domain` to ensure cleanliness.
