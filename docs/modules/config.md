<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# config Module

**Source**: `crates/mcb-infrastructure/src/config/`
**Crate**: `mcb-infrastructure`
**Files**: 3+
**Lines of Code**: ~1,000

## ↔ Code ↔ Docs cross-reference

| Direction | Link |
| --------- | ---- |
| Code → Docs | [`crates/mcb-infrastructure/src/config/mod.rs`](../../crates/mcb-infrastructure/src/config/mod.rs) links here |
| Docs → Code | [`crates/mcb-infrastructure/src/config/`](../../crates/mcb-infrastructure/src/config/) — Config logic |
| Env Vars | [`ENVIRONMENT_VARIABLES.md`](../configuration/ENVIRONMENT_VARIABLES.md) (**Canonical SSOT**) |
| ADR | [`ADR-041`](../adr/041-modular-config-system.md) |

## Overview

Application configuration management with type-safe structures, environment variable overrides, and validation.

### Key Components

### Configuration Types ([`types.rs`](../../crates/mcb-infrastructure/src/config/types.rs))

Hierarchical configuration structures:

- `AppConfig` - Root configuration
- `ServerConfig` - Server settings (network, SSL, CORS, timeouts)
- `AuthConfig` - Authentication (JWT settings)
- `CacheConfig` - Cache configuration
- `ProviderConfig` - Provider settings

### Configuration Loader ([`loader.rs`](../../crates/mcb-infrastructure/src/config/loader.rs))

Multi-source configuration loading.

## File Structure

```text
crates/mcb-infrastructure/src/config/
├── types.rs              # Configuration types
├── loader.rs             # Configuration loading
└── mod.rs                # Module exports
```

## Configuration Structure

```rust
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub cache: CacheConfig,
    pub providers: ProviderConfig,
}

pub struct ServerConfig {
    pub network: ServerNetworkConfig,  // port, host
    pub ssl: ServerSslConfig,          // https, certs
    pub cors: CorsConfig,              // allowed origins
    pub timeouts: TimeoutConfig,       // request timeouts
}

pub struct AuthConfig {
    pub jwt: JwtConfig,                // secret, expiration
    pub rate_limit: RateLimitConfig,   // request limits
}
```

## Environment Variables

The full matrix of environment variables is documented in the canonical **Single Source of Truth**:

👉 **[docs/configuration/ENVIRONMENT_VARIABLES.md](../configuration/ENVIRONMENT_VARIABLES.md)**

## Key Exports

```rust
pub use types::{AppConfig, ServerConfig, AuthConfig, CacheConfig};
pub use loader::ConfigLoader;
```

## Cross-References

- **Infrastructure**: [infrastructure.md](./infrastructure.md) (parent module)
- **Server**: [server.md](./server.md) (uses config)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

### Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)
