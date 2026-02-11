---
adr: 25
title: Figment Configuration Migration
status: IMPLEMENTED
created:
updated: 2026-02-05
related: [13, 21]
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

## ADR 025: Figment Configuration Migration

## Status

**Implemented** (v0.1.2)

> Migration from `config` crate to Figment completed as part of the infrastructure modernization initiative.

## Context

The current configuration system uses the `config` crate (version 0.15) with manual source composition. The config crate provides a builder pattern for aggregating configuration from multiple sources:

### Current Implementation

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

### Current Limitations

1. **Manual orchestration**: Each source must be explicitly added and configured
2. **Repetitive setup**: Similar patterns repeated across different configuration needs
3. **Format coupling**: Hard-coded format handling (TOML vs JSON vs YAML)
4. **Precedence complexity**: Source precedence depends on addition order
5. **Limited error context**: Basic deserialization errors without source attribution
6. **No profile support**: Manual handling of development/production environments

### Figment as Solution

Figment (version 0.10.19) provides a unified configuration approach with powerful composition and validation features:

#### Core Features

-   **Provider system**: Modular sources (TOML, JSON, YAML, environment, custom)
-   **Fluent composition**: Chainable `merge()` operations with clear precedence
-   **Profile support**: Built-in development/production environment handling
-   **Rich error handling**: Detailed error messages with source attribution
-   **Type safety**: Compile-time guarantees through `extract<T>()` method
-   **Extensible**: Easy to implement custom providers

#### Key Advantages Over Config Crate

| Feature | Config Crate | Figment |
|---------|--------------|---------|
| **API Style** | Builder pattern with manual source addition | Fluent composition with `merge()` |
| **Error Messages** | Basic deserialization errors | Rich, contextual error messages |
| **Profile Support** | Manual implementation | Built-in profile system |
| **Provider Ecosystem** | Limited built-in providers | Extensive provider library |
| **Validation** | Basic deserialization | Rich validation with custom extractors |
| **Extensibility** | Custom sources via traits | Provider trait system |

## Decision

We will migrate from the `config` crate to Figment for all configuration loading across the Memory Context Browser. This decision addresses the need for more robust, maintainable, and feature-rich configuration handling.

### Migration Scope

The migration will affect all configuration loading points:

1. **Infrastructure configuration** (`mcb-infrastructure/src/config/loader.rs`)
2. **Server startup configuration** (`mcb-server/src/init.rs`)
3. **Admin interface configuration** (`mcb-server/src/admin/config.rs`)
4. **Provider-specific configuration** (embedding, vector store, cache providers)

### Technical Migration Strategy

#### Core Changes

1. **Replace `config::Config`** with `figment::Figment` as the central configuration type
2. **Use `Figment::new().merge()`** fluent API for source composition
3. **Leverage Figment's built-in providers** for TOML, environment variables, and custom sources
4. **Maintain the same `AppConfig` structure** for API compatibility
5. **Add profile support** for development/production environment switching

#### Provider Migration

Figment provides dedicated providers for common configuration sources:

| Source Type | Config Crate | Figment Provider |
|-------------|--------------|------------------|
| **TOML files** | `config::File` | `figment::providers::Toml` |
| **Environment vars** | `config::Environment` | `figment::providers::Env` |
| **JSON files** | `config::File` | `figment::providers::Json` |
| **YAML files** | Manual parsing | `figment::providers::Yaml` |

### Migration Pattern

**Before (config crate - verbose and error-prone):**

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

**After (Figment - clean and robust):**

```rust
use figment::{Figment, providers::{Toml, Env}};

// Fluent composition with clear precedence
let figment = Figment::new()
    .merge(Toml::file("config/default.toml"))  // Defaults
    .merge(Toml::file(config_path))            // User config (overrides defaults)
    .merge(Env::prefixed("APP_").split("_")); // Environment (highest precedence)

// Rich error handling with source attribution
let app_config: AppConfig = figment.extract()
    .map_err(|e| format!("Configuration error in {}: {}", e.path, e.value))?;
```

### Profile Support Implementation

Figment enables environment-specific configuration through profile-based composition:

```rust
use figment::Profile;

// Development configuration
let dev_config: AppConfig = Figment::new()
    .merge(Toml::file("config/default.toml"))
    .merge(Toml::file("config/development.toml").profile(Profile::new("dev")))
    .merge(Env::prefixed("APP_").profile(Profile::new("dev")))
    .extract()?;

// Production configuration
let prod_config: AppConfig = Figment::new()
    .merge(Toml::file("config/default.toml"))
    .merge(Toml::file("config/production.toml").profile(Profile::new("prod")))
    .merge(Env::prefixed("APP_").profile(Profile::new("prod")))
    .extract()?;
```

### Error Handling Improvements

Figment provides significantly better error messages:

**Config crate error:**

```
Error: TOML parse error: invalid type: integer `123`, expected a string for key `app.port` at line 10 column 15
```

**Figment error:**

```
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

-   **Unified API**: Single approach for all configuration sources
-   **Better error messages**: Figment provides more detailed configuration errors
-   **Profile support**: Easy development/production configuration switching
-   **Type safety**: Compile-time validation of configuration structure
-   **Extensibility**: Easy to add new configuration sources
-   **Less boilerplate**: Simpler source composition code

### Negative

-   **New dependency**: Adds Figment to the dependency tree
-   **API changes**: Different method names and patterns
-   **Migration effort**: Need to update all configuration loading code
-   **Learning curve**: New API to understand

### Risks

-   **Source precedence confusion**: Figment's merge order might differ from config crate
-   **Provider compatibility**: Not all config crate sources have Figment equivalents
-   **Validation differences**: Figment's extraction might behave differently

## Migration Strategy

### Phase 1: Compatibility Layer

1. Add Figment dependency alongside existing config crate
2. Create compatibility functions that wrap Figment with config crate API
3. Update tests to work with both systems

### Phase 2: Gradual Migration

1. Migrate infrastructure configuration first
2. Update application configuration loading
3. Migrate server startup configuration
4. Add profile support for development/production

### Phase 3: Cleanup

1. Remove config crate dependency
2. Delete compatibility layer
3. Update all documentation
4. Comprehensive testing

## Validation Criteria

-   [x] All configuration sources load correctly (TOML, environment, defaults)
-   [x] Error messages are more helpful than before
-   [x] Profile-based configuration works
-   [x] All existing configuration values are preserved
-   [x] Performance is maintained or improved
-   [x] Integration tests pass with new configuration system

## Related ADRs

-   [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Configuration across crates
-   [ADR 021: Dependency Management](021-dependency-management.md) - Workspace dependency strategy
