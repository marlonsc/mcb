# ADR 024: Simplified Dependency Injection

## Status

**Proposed** (v0.2.0)

> Planned replacement for [ADR 002: Dependency Injection with Shaku](002-dependency-injection-shaku.md) as part of the refatoração and simplification initiative.

## Context

The current dependency injection system using Shaku provides compile-time guarantees but introduces significant complexity:

1. **Macro overhead**: Extensive use of `#[derive(Component)]`, `module!`, and `HasComponent` traits
2. **Build complexity**: Compile-time DI resolution adds to build times
3. **Learning curve**: Shaku's API is complex for new contributors
4. **Maintenance burden**: Module definitions must be kept in sync manually
5. **Limited flexibility**: Runtime service swapping requires container rebuilding

While Shaku provides good decoupling, the complexity outweighs the benefits for this project's scale and needs. A simpler approach using constructor injection with manual service composition would:

1. **Reduce boilerplate**: Eliminate macro-heavy DI declarations
2. **Improve readability**: Clear dependency flow through constructor parameters
3. **Maintain testability**: Constructor injection still enables dependency mocking
4. **Simplify debugging**: Direct object construction is easier to trace
5. **Reduce build times**: Less compile-time macro expansion

## Decision

We will replace Shaku-based DI with a simplified constructor injection pattern:

1. **Remove Shaku dependency** from all crates
2. **Use constructor injection** for all service dependencies
3. **Maintain trait-based abstraction** for testability
4. **Use manual service composition** in the application bootstrap
5. **Keep dependency inversion** through trait objects (`Arc<dyn Trait>`)

### Implementation Pattern

**Before (Shaku):**
```rust
#[derive(Component)]
#[shaku(interface = dyn MyService)]
pub struct MyServiceImpl {
    #[shaku(inject)]
    dependency: Arc<dyn OtherService>,
}

module! {
    pub MyModule: MyModuleTrait {
        components = [MyServiceImpl],
        providers = []
    }
}
```

**After (Constructor Injection):**
```rust
pub struct MyServiceImpl {
    dependency: Arc<dyn OtherService>,
}

impl MyServiceImpl {
    pub fn new(dependency: Arc<dyn OtherService>) -> Self {
        Self { dependency }
    }
}

// Manual composition in bootstrap
let other_service = Arc::new(OtherServiceImpl::new());
let my_service = Arc::new(MyServiceImpl::new(other_service));
```

### Bootstrap Pattern

Service composition will be handled in `mcb-infrastructure/src/di/bootstrap.rs`:

```rust
pub struct ServiceContainer {
    pub my_service: Arc<dyn MyService>,
    pub other_service: Arc<dyn OtherService>,
}

impl ServiceContainer {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let other_service = Arc::new(OtherServiceImpl::new(config));
        let my_service = Arc::new(MyServiceImpl::new(other_service.clone()));

        Ok(Self {
            my_service,
            other_service,
        })
    }
}
```

## Consequences

### Positive
- **Reduced complexity**: Eliminates macro-heavy DI infrastructure
- **Better readability**: Clear dependency chains through constructors
- **Faster compilation**: Less macro expansion overhead
- **Easier debugging**: Direct object construction is straightforward
- **Simplified testing**: Constructor injection still enables mocking
- **Smaller dependency tree**: Removes Shaku and related macro dependencies

### Negative
- **Manual composition**: Service wiring becomes explicit code
- **Runtime errors**: Missing dependencies caught at runtime, not compile-time
- **Boilerplate**: More verbose service instantiation
- **Refactoring impact**: Changes to dependencies require updating multiple constructors

### Risks
- **Service creation errors**: Runtime failures if dependencies are missing
- **Constructor parameter creep**: Large constructors with many dependencies
- **Testing complexity**: Need to manually create all dependency chains in tests

## Migration Strategy

### Phase 1: Preparation
1. Create new constructor injection implementations alongside Shaku code
2. Add integration tests to verify both approaches work
3. Update documentation and examples

### Phase 2: Gradual Migration
1. Migrate infrastructure services first (least dependent)
2. Migrate application services
3. Migrate server bootstrap code
4. Keep both systems running in parallel during transition

### Phase 3: Cleanup
1. Remove Shaku dependencies
2. Delete old Shaku module definitions
3. Update all documentation
4. Run comprehensive testing

## Validation Criteria

- [ ] All services can be instantiated through constructor injection
- [ ] Test mocking still works with trait objects
- [ ] Application startup time improves or stays the same
- [ ] Compile time reduces significantly
- [ ] All integration tests pass
- [ ] Binary size remains stable or decreases
- [ ] Code coverage maintained

## Related ADRs

- [ADR 002: Dependency Injection with Shaku](002-dependency-injection-shaku.md) - **SUPERSEDED** by this ADR
- [ADR 012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - Related DI layering approach
- [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Multi-crate organization