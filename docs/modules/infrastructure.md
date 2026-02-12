# infrastructure Module

**Source**: `crates/mcb-infrastructure/src/`
**Crate**: `mcb-infrastructure`
**Files**: 40+
**Lines of Code**: ~6,000

**Project links**: See `docs/architecture/ARCHITECTURE.md` (DI/provider patterns), `docs/modules/domain.md`, and `docs/developer/ROADMAP.md` for infrastructure architecture and v0.2.1 roadmap alignment.

## Overview

The infrastructure module provides shared technical services and cross-cutting concerns for the Memory Context Browser system. It implements dill-based dependency injection (ADR-029), Figment configuration, caching, health checks, and null adapters for testing.

## Key Components

### Dependency Injection (`di/`)

dill IoC Container with handle-based runtime switching (ADR-024 → ADR-029):

- `catalog.rs` - dill Catalog configuration and service resolution
- `bootstrap.rs` - Application initialization and AppContext creation
- `handles.rs` - RwLock provider handles for runtime switching
- `admin.rs` - Admin services for provider switching via API
- `provider_resolvers.rs` - linkme registry access

### Configuration (`config/`)

Application configuration management:

- Type-safe configuration with nested structures
- Environment variable overrides
- Server, auth, cache, and provider configurations

### Cache (`cache/`)

Caching infrastructure:

- Cache configuration and management
- Integration with mcb-providers cache implementations

### Crypto (`crypto/`)

Encryption and hashing utilities:

- AES-GCM encryption support
- Hash computation utilities

### Health (`health/`)

Health check infrastructure:

- Component health monitoring
- Readiness and liveness checks

### Logging (`logging/`)

Structured logging configuration:

- Tracing integration
- Log level management

### Adapters (`adapters/`)

Null implementations for DI testing:

- `infrastructure/` - Null adapters for infrastructure ports
- `NullAuthService`
- `NullEventBus`
- `NullSyncProvider`
- `NullLockProvider`
- `NullSnapshotProvider`
- `NullStateStoreProvider`
- `NullPerformanceMetrics`
- `NullIndexingOperations`
- `NullSystemMetricsCollector`
- `providers/` - Provider adapter bindings
- `repository/` - Repository adapters
- `NullChunkRepository`
- `NullSearchRepository`

## File Structure

```text
crates/mcb-infrastructure/src/
├── di/
│   ├── bootstrap.rs        # DI container setup
│   ├── modules/
│   │   ├── infrastructure.rs
│   │   ├── server.rs
│   │   ├── providers.rs
│   │   ├── traits.rs
│   │   └── mod.rs
│   └── mod.rs
├── config/
│   ├── types.rs            # Configuration types
│   └── mod.rs
├── cache/
│   └── mod.rs
├── crypto/
│   └── mod.rs
├── health/
│   └── mod.rs
├── logging/
│   └── mod.rs
├── adapters/
│   ├── infrastructure/     # Null infrastructure adapters
│   │   ├── auth.rs
│   │   ├── events.rs
│   │   ├── metrics.rs
│   │   ├── snapshot.rs
│   │   ├── sync.rs
│   │   ├── admin.rs
│   │   └── mod.rs
│   ├── providers/          # Provider bindings
│   │   └── mod.rs
│   ├── repository/         # Repository adapters
│   │   └── mod.rs
│   └── mod.rs
├── infrastructure/         # Re-exports
│   └── mod.rs
└── lib.rs                  # Crate root
```

## Key Exports

```rust
// DI
pub use di::{bootstrap, McpModule};

// Configuration
pub use config::{AppConfig, ServerConfig, AuthConfig};

// Null Adapters (for testing)
pub use adapters::infrastructure::{
    NullAuthService, NullEventBus, NullSyncProvider,
    NullSnapshotProvider, NullPerformanceMetrics,
};
```

## Testing

Infrastructure tests are located in `crates/mcb-infrastructure/tests/`.

## Project Alignment

- **Architecture guidance**: `docs/architecture/ARCHITECTURE.md` explains the layered wiring and documents linkme/provider registration so every adapter matches compiled routing expectations.
- **Roadmap signals**: Anchor infrastructure decisions in `docs/developer/ROADMAP.md` (validated requirements, debt) so features like provider health checks and session memory inherit the correct dependencies.
- **Operational metrics**: Sync with `docs/operations/CHANGELOG.md`/`docs/operations/CI_OPTIMIZATION_VALIDATION.md` for metrics when adjusting caches, health, or DI to maintain the declared `0 architecture violations` and `~1805 tests` commitments.

---

*Updated 2026-01-28 - Reflects dill IoC, Figment config (v0.2.1)*
