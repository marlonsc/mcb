---
adr: 25
title: Figment Configuration Migration
status: IMPLEMENTED
created: 2024-06-01
updated: 2026-02-12
related: [13, 21]
supersedes: []
superseded_by: []
implementation_status: Complete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 025: Figment Configuration Migration

## Status

**Implemented** (v0.1.2)

> Migration from `config` crate to Figment completed as part of the infrastructure
> modernization initiative.

## Context

The previous configuration system used the `config` crate (version 0.15) with
manual source composition. The config crate provided a builder pattern for
aggregating configuration from multiple sources:

### Previous Implementation (config crate — removed in v0.1.2)

```rust
use config::{Config, Environment, File};

let mut builder = Config::builder();

// Step 1: Add defaults (serialized from AppConfig::default())
builder = builder.add_source(config::File::from_str(
    &toml::to_string(&AppConfig::default())?,
    config::FileFormat::Toml,
));

// Step 2: Add configuration file
builder = builder.add_source(File::from(config_path));

// Step 3: Add environment variables
builder = builder.add_source(
    Environment::with_prefix("APP")
        .prefix_separator("_")
        .separator("_")
        .try_parsing(true)
);

// Step 4: Build and deserialize
let config: AppConfig = builder.build()?.try_deserialize()?;
```

### Limitations of the config crate (motivation for migration)

1. **Manual orchestration**: Each source had to be explicitly added and
   configured
2. **Repetitive setup**: Similar patterns repeated across different configuration
   needs
3. **Format coupling**: Hard-coded format handling (TOML vs JSON vs YAML)
4. **Precedence complexity**: Source precedence depended on addition order
5. **Limited error context**: Basic deserialization errors without source
   attribution
6. **No profile support**: Manual handling of development/production
   environments

### Figment as Solution

Figment (version 0.10, workspace-managed) provides a unified configuration
approach with powerful composition and validation features:

#### Core Features

- **Provider system**: Modular sources (TOML, JSON, YAML, environment, custom)
- **Fluent composition**: Chainable `merge()` operations with clear precedence
- **Profile support**: Built-in development/production environment handling
- **Rich error handling**: Detailed error messages with source attribution
- **Type safety**: Compile-time guarantees through `extract<T>()` method
- **Extensible**: Easy to implement custom providers

#### Key Advantages Over Config Crate

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Feature | Config Crate | Figment |
| --------- | -------------- | --------- |
| **API Style** | Builder pattern with manual source addition | Fluent composition with `merge()` |
| **Error Messages** | Basic deserialization errors | Rich, contextual error messages |
| **Profile Support** | Manual implementation | Built-in profile system |
| **Provider Ecosystem** | Limited built-in providers | Extensive provider library |
| **Validation** | Basic deserialization | Rich validation with custom extractors |
| **Extensibility** | Custom sources via traits | Provider trait system |

## Decision

We will migrate from the `config` crate to Figment for all configuration loading
across the Memory Context Browser. This decision addresses the need for more
robust, maintainable, and feature-rich configuration handling.

### Migration Scope

The migration affected the following configuration loading points:

1. **Infrastructure configuration**
   (`mcb-infrastructure/src/config/loader.rs`) — **Migrated.** Central
   `ConfigLoader` uses Figment with TOML + Env providers.
2. **Validation configuration** (`mcb-validate/src/config/file_config.rs`) —
   **Migrated.** Separate Figment chain with embedded defaults, filesystem
   override, and `MCB_VALIDATE__` env.
3. **Rocket server configuration** — Rocket uses its own `rocket::figment`
   integration internally for template dirs and server settings. This is
   Rocket-managed, not MCB-managed.
4. **Server startup** (`mcb-server/src/init.rs`) — Receives `AppConfig` from
   infrastructure; does not load config directly.
5. **Admin configuration** (`mcb-server/src/admin/config.rs`) —
   Serialization/sanitization layer for admin API responses; does not load
   config.

### Technical Migration Strategy

#### Core Changes

1. **Replace `config::Config`** with `figment::Figment` as the central
   configuration type
2. **Use `Figment::new().merge()`** fluent API for source composition
3. **Leverage Figment's built-in providers** for TOML, environment variables,
   and custom sources
4. **Maintain the same `AppConfig` structure** for API compatibility
5. **Add profile support** for development/production environment switching

#### Provider Migration

Figment provides dedicated providers for common configuration sources:

| Source Type | Config Crate | Figment Provider |
| ------------- | -------------- | ------------------ |
| **TOML files** | `config::File` | `figment::providers::Toml` |
| **Environment vars** | `config::Environment` | `figment::providers::Env` |
| **JSON files** | `config::File` | `figment::providers::Json` |
| **YAML files** | Manual parsing | `figment::providers::Yaml` |

### Migration Pattern

Before (config crate — removed in v0.1.2):

```rust
use config::{Config, Environment, File};

let mut builder = Config::builder();

// Manual source addition with explicit format specification
builder = builder.add_source(File::from(config_path));

// Complex environment variable setup
builder = builder.add_source(
    Environment::with_prefix("APP")
        .prefix_separator("_")
        .separator("_")
        .try_parsing(true)
        .list_separator(" ")
        .with_list_parse_key("server.cors_origins")
);

// Manual error handling
let config = builder.build()?;
let app_config: AppConfig = config.try_deserialize()
    .context("Failed to deserialize configuration")?;
```

After (Figment — current production pattern):

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
```rust
use figment::Figment;
use figment::providers::{Env, Format, Toml};

// Fluent composition with clear precedence (see loader.rs)
let mut figment = Figment::new()
    .merge(Toml::file("config/default.toml"))  // Defaults (required)
    .merge(Toml::file(config_path));           // User config (overrides defaults)

// Environment overrides — MCP__ prefix with double-underscore nesting
figment = figment.merge(
    Env::prefixed("MCP__").split("__").lowercase(true)
);

// Typed extraction followed by semantic validation
let app_config: AppConfig = figment.extract()
    .context("Failed to extract configuration")?;
validate_app_config(&app_config)?;
```

### Profile Support (available but not currently used)

Figment enables environment-specific configuration through profile-based
composition. This capability is available but**not currently used** in MCB's
`ConfigLoader` pipeline. MCB achieves environment differentiation via the
optional override file and `MCP__` environment variables instead.

The pattern below illustrates what is possible if profile-based switching is
needed in the future:

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
```rust
use figment::Profile;

// Development configuration
let dev_config: AppConfig = Figment::new()
    .merge(Toml::file("config/default.toml"))
    .merge(Toml::file("config/development.toml").profile(Profile::new("dev")))
    .merge(Env::prefixed("MCP__").split("__").lowercase(true).profile(Profile::new("dev")))
    .extract()?;

// Production configuration
let prod_config: AppConfig = Figment::new()
    .merge(Toml::file("config/default.toml"))
    .merge(Toml::file("config/production.toml").profile(Profile::new("prod")))
    .merge(Env::prefixed("MCP__").split("__").lowercase(true).profile(Profile::new("prod")))
    .extract()?;
```

### Error Handling Improvements

Figment provides significantly better error messages:

Config crate error:

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
```text
Error: TOML parse error: invalid type: integer `123`, expected a string for key `app.port` at line 10 column 15
```

Figment error:

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
```text
Error: `app.port` is not a string (integer `123`) in config/default.toml:10:15, but expected a string

Caused by:
    expected a string
    at config/default.toml:10:15
    8 | [app]
    9 | name = "myapp"
    10| port = 123  # <-- integer, expected string
```

## Consequences

### Positive

- **Unified API**: Single approach for all configuration sources
- **Better error messages**: Figment provides more detailed configuration errors
- **Profile support**: Easy development/production configuration switching
- **Type safety**: Compile-time validation of configuration structure
- **Extensibility**: Easy to add new configuration sources
- **Less boilerplate**: Simpler source composition code

### Negative

- **New dependency**: Adds Figment to the dependency tree
- **API changes**: Different method names and patterns
- **Migration effort**: Need to update all configuration loading code
- **Learning curve**: New API to understand

### Risks

- **Source precedence confusion**: Figment's merge order might differ from
  config crate
- **Provider compatibility**: Not all config crate sources have Figment
  equivalents
- **Validation differences**: Figment's extraction might behave differently

## Migration Strategy (all phases complete)

### Phase 1: Compatibility Layer ✅

1. ~~Add Figment dependency alongside existing config crate~~
2. ~~Create compatibility functions that wrap Figment with config crate API~~
3. ~~Update tests to work with both systems~~

### Phase 2: Gradual Migration ✅

1. ~~Migrate infrastructure configuration first~~
2. ~~Update application configuration loading~~
3. ~~Migrate server startup configuration~~
4. Profile support for development/production — **Deferred** (available in
   Figment but not currently needed; env overrides suffice)

### Phase 3: Cleanup ✅

1. ~~Remove config crate dependency~~ — `config` crate is no longer in
   `Cargo.toml`
2. ~~Delete compatibility layer~~
3. ~~Update all documentation~~ — See `docs/CONFIGURATION.md` and
   `context/external/figment.md`
4. ~~Comprehensive testing~~ — See
   `crates/mcb-infrastructure/tests/unit/config_figment_tests.rs`

## Validation Criteria

- [x] All configuration sources load correctly (TOML, environment, defaults)
- [x] Error messages are more helpful than before
- [ ] Profile-based configuration works — **Deferred**: Figment profiles are
  available but not used in `ConfigLoader`. Environment differentiation is
  achieved via override files and `MCP__` env vars instead.
- [x] All existing configuration values are preserved
- [x] Performance is maintained or improved
- [x] Integration tests pass with new configuration system
- [x] Integration tests pass with new configuration system
- [x] Legacy `MCB_` prefix is rejected (verified by `config_figment_tests.rs`)

## Related ADRs

- [ADR 013: Clean Architecture Crate Separation]
(013-clean-architecture-crate-separation.md) - Configuration across crates
- [ADR 021: Dependency Management]
(021-dependency-management.md) - Workspace dependency strategy

## Related Documentation

- [Configuration Guide](../../docs/CONFIGURATION.md) - Operator-facing
  configuration reference
- [Figment External Context](../../context/external/figment.md) - Library
  analysis with Context7/GitHub evidence
- [Architecture Boundaries](../architecture/ARCHITECTURE_BOUNDARIES.md) - Where
  Figment fits in the infrastructure layer
