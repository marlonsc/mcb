# ADR 025: Figment Configuration Migration

## Status

**Proposed** (v0.2.0)

> Planned replacement for manual configuration loading as part of the simplification initiative.

## Context

The current configuration system uses the `config` crate with manual source composition:

```rust
let mut builder = Config::builder();
builder = builder.add_source(config::File::from_str(&toml::to_string(&AppConfig::default())?, config::FileFormat::Toml));
builder = builder.add_source(File::from(config_path));
builder = builder.add_source(Environment::with_prefix("APP").separator("_"));
let config = builder.build()?.try_deserialize::<AppConfig>()?;
```

This approach requires:
1. **Manual source management**: Explicitly adding each configuration source
2. **Format specification**: Hard-coded format handling for TOML/JSON
3. **Error-prone composition**: Easy to miss sources or get precedence wrong
4. **Limited validation**: Basic deserialization without rich error context
5. **Repetitive code**: Similar patterns across different configuration needs

Figment provides a unified configuration approach that:
1. **Unifies sources**: Single API for files, environment variables, and defaults
2. **Rich error handling**: Better error messages and validation
3. **Type safety**: Compile-time guarantees for configuration structure
4. **Extensible**: Easy to add new configuration sources
5. **Profile support**: Built-in development/production profile handling

## Decision

We will migrate from the `config` crate to Figment for all configuration loading:

1. **Replace `config::Config`** with `figment::Figment`
2. **Use `Figment::new().merge()`** pattern for source composition
3. **Leverage Figment's providers** for TOML, environment variables, etc.
4. **Maintain the same `AppConfig` structure** for compatibility
5. **Add profile support** for development/production environments

### Migration Pattern

**Before (config crate):**
```rust
use config::{Config, Environment, File};

let mut builder = Config::builder();
builder = builder.add_source(File::from(config_path));
builder = builder.add_source(Environment::with_prefix("APP"));
let config: AppConfig = builder.build()?.try_deserialize()?;
```

**After (Figment):**
```rust
use figment::{Figment, providers::{Toml, Env}};

let config: AppConfig = Figment::new()
    .merge(Toml::file(config_path))
    .merge(Env::prefixed("APP_"))
    .extract()?;
```

### Profile Support

Figment enables profile-based configuration:

```rust
// Development config
Figment::new()
    .merge(Toml::file("config/default.toml"))
    .merge(Toml::file("config/development.toml"))
    .merge(Env::prefixed("APP_").split("_"))

// Production config
Figment::new()
    .merge(Toml::file("config/default.toml"))
    .merge(Toml::file("config/production.toml"))
    .merge(Env::prefixed("APP_").split("_"))
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
- **Source precedence confusion**: Figment's merge order might differ from config crate
- **Provider compatibility**: Not all config crate sources have Figment equivalents
- **Validation differences**: Figment's extraction might behave differently

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

- [ ] All configuration sources load correctly (TOML, environment, defaults)
- [ ] Error messages are more helpful than before
- [ ] Profile-based configuration works
- [ ] All existing configuration values are preserved
- [ ] Performance is maintained or improved
- [ ] Integration tests pass with new configuration system

## Related ADRs

- [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Configuration across crates
- [ADR 021: Dependency Management](021-dependency-management.md) - Workspace dependency strategy