# ADR 024: Shaku to dill DI Migration

## Status

**Proposed** (v0.1.2)

> Planned replacement for [ADR 002: Dependency Injection with Shaku](002-dependency-injection-shaku.md) using the dill runtime DI framework.

## Context

The current dependency injection system uses Shaku (version 0.6), a compile-time DI container that provides trait-based dependency resolution. While effective, this approach introduces substantial complexity that impacts development velocity and maintainability.

### Current Shaku Implementation

The current system uses a two-layer DI approach (ADR 012) with:

1.  **Shaku Components**: Services registered with `#[derive(Component)]` and `#[shaku(interface = dyn Trait)]`
2.  **Runtime Factories**: Production providers created via factory functions outside Shaku
3.  **Module Composition**: Services organized in Shaku modules with macro-generated wiring

**Example of current complexity:**

```rust
// Component definition with multiple attributes
#[derive(Component)]
#[shaku(interface = dyn EmbeddingProvider)]
pub struct OllamaEmbeddingProvider {
    #[shaku(inject)]  // Runtime injection point
    config: Arc<dyn ConfigProvider>,
    #[shaku(inject)]
    http_client: Arc<dyn HttpClient>,
}

// Module definition with macro
module! {
    pub EmbeddingModuleImpl: EmbeddingModule {
        components = [OllamaEmbeddingProvider],
        providers = []
    }
}

// Runtime resolution
let provider: Arc<dyn EmbeddingProvider> = container.resolve();
```

### Problems with Current Approach

#### Developer Experience Issues

1.  **Macro complexity**: `#[derive(Component)]`, `#[shaku(interface = ...)]`, `#[shaku(inject)]` everywhere
2.  **Build time impact**: Extensive macro expansion slows compilation
3.  **Learning curve**: Shaku API is complex for new team members
4.  **Debugging difficulty**: DI resolution happens through macro-generated code

#### Maintenance Issues

1.  **Module sync**: Manual maintenance of module definitions as services change
2.  **Trait bounds**: Complex trait bounds on component implementations
3.  **Testing overhead**: Need to understand Shaku to write unit tests
4.  **Refactoring friction**: Changes require updating multiple macro annotations

#### Architectural Issues

1.  **Over-engineering**: DI container complexity exceeds project needs
2.  **Runtime opacity**: Service resolution happens through generated code
3.  **Limited flexibility**: Hard to customize service creation per environment

### DI Library Research

We evaluated modern Rust DI alternatives:

| Library | Type | Cross-Crate | Async | Verdict |
|---------|------|-------------|-------|---------|
| **Shaku** (current) | Compile-time | Yes | No | High boilerplate |
| **nject** | Compile-time | **NO** | No | Rejected (cross-crate limitation) |
| **dill** | Runtime | Yes | Tokio | **SELECTED** |
| **dependency_injector** | Runtime | Yes | Optional | Viable alternative |
| Manual injection | N/A | N/A | N/A | Step backwards |

**Critical Requirement**: Cross-crate compatibility is essential for the 8-crate workspace architecture.

### Why dill

The [dill](https://github.com/sergiimk/dill-rs) crate (version 0.15.0, as of January 2026) provides runtime DI designed specifically for Clean/Onion Architecture:

**Key Benefits:**

1.  **Clean Architecture alignment**: Explicitly designed for Onion Architecture patterns
2.  **Native `Arc<dyn Trait>`**: First-class support for trait-based DI
3.  **Cross-crate compatible**: Works seamlessly across all 8 workspace crates
4.  **Tokio-compatible**: Task-local scoping for async contexts
5.  **Production-proven**: Used in [kamu-cli](https://github.com/kamu-data/kamu-cli), a large-scale Rust project with similar architecture
6.  **Lower boilerplate**: Simple `#[component]` + `#[interface]` attributes

**dill Injection Specifications:**

-   `OneOf<T>` - Single implementation
-   `AllOf<T>` - Collection of all implementations
-   `Maybe<T>` - Optional dependency (returns None if missing)
-   `Lazy<T>` - Deferred resolution

**dill Scopes:**

-   `Transient` - New instance per call
-   `Singleton` - Reused after first creation
-   `Transaction` - Cached during transaction

## Decision

We will replace Shaku-based DI with dill runtime DI:

1.  **Replace Shaku with dill** across all crates
2.  **Use Catalog pattern** for service registration and resolution
3.  **Maintain trait-based abstraction** for testability
4.  **Keep dependency inversion** through trait objects (`Arc<dyn Trait>`)

### Implementation Pattern

**Before (Shaku - verbose macros):**

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

// Resolution
let service: Arc<dyn MyService> = container.resolve();
```

**After (dill - clean attributes):**

```rust
use dill::{component, interface, Catalog};

#[component]
#[interface(dyn MyService)]
pub struct MyServiceImpl {
    dependency: Arc<dyn OtherService>,
}

// Catalog composition
let catalog = Catalog::builder()
    .add::<MyServiceImpl>()
    .add::<OtherServiceImpl>()
    .build();

// Resolution
let service: Arc<dyn MyService> = catalog.get_one().unwrap();
```

### Bootstrap Pattern

Service composition will be handled in `mcb-infrastructure/src/di/bootstrap.rs`:

```rust
use dill::Catalog;

pub struct AppContext {
    pub catalog: Catalog,
    pub config: AppConfig,
}

impl AppContext {
    pub fn new(config: AppConfig) -> Result<Self> {
        let catalog = Catalog::builder()
            // Infrastructure services
            .add::<MokaCacheProvider>()
            .add::<TokioBroadcastEventBus>()
            .add::<NullAuthService>()
            // Add more services...
            .build();

        Ok(Self { catalog, config })
    }

    pub fn get<T: ?Sized + 'static>(&self) -> Arc<T> {
        self.catalog.get_one().expect("Service not registered")
    }
}
```

### Comparative Analysis

| Aspect | Shaku (Current) | dill (Proposed) |
|--------|----------------|-----------------|
| **API Complexity** | High (macros, modules, components) | Low (#[component], Catalog) |
| **Build Time** | Slow (extensive macro expansion) | Faster (simpler macros) |
| **Learning Curve** | Steep (Shaku-specific) | Moderate (catalog pattern) |
| **Testability** | Good (but requires Shaku setup) | Excellent (catalog builder) |
| **Cross-Crate** | Yes | Yes |
| **Async Support** | No | Tokio task-local |
| **Production Use** | Many projects | kamu-cli (similar architecture) |

## Consequences

### Positive

-   **Reduced complexity**: Simpler attribute-based registration
-   **Better readability**: Clear catalog-based composition
-   **Faster compilation**: Less macro expansion overhead
-   **Easier debugging**: Direct catalog resolution
-   **Architecture alignment**: dill designed for Clean Architecture
-   **Maintained automation**: Unlike manual injection, keeps DI benefits
-   **Optional dependencies**: `Maybe<T>` pattern for optional services

### Negative

-   **New dependency**: Adds dill to the dependency tree
-   **Runtime resolution**: Dependencies resolved at runtime, not compile-time
-   **API change**: Different syntax from Shaku
-   **Learning curve**: Team needs to learn dill patterns

### Risks

-   **Runtime errors**: Missing dependencies caught at runtime
-   **Catalog misconfiguration**: Must register all components
-   **Version stability**: dill is relatively newer than Shaku

## Migration Strategy

### Phase 1: Preparation

1.  Add dill dependency alongside Shaku
2.  Create new dill-based implementations in parallel
3.  Add integration tests for both approaches

### Phase 2: Gradual Migration

1.  Migrate infrastructure services first (mcb-infrastructure)
2.  Migrate application services (mcb-application)
3.  Migrate server bootstrap (mcb-server)
4.  Keep both systems running during transition

### Phase 3: Cleanup

1.  Remove Shaku dependencies from all crates
2.  Delete old Shaku module definitions
3.  Update all documentation
4.  Run comprehensive testing

## Validation Criteria

-   [ ] All services registered in dill Catalog
-   [ ] Test mocking works with catalog.get_one()
-   [ ] Compile time improves or stays the same
-   [ ] All integration tests pass
-   [ ] No Shaku references remain in codebase
-   [ ] Binary size remains stable

## Related ADRs

-   [ADR 002: Dependency Injection with Shaku](002-dependency-injection-shaku.md) - **SUPERSEDED** by this ADR
-   [ADR 012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - **SUPERSEDED** (dill simplifies to single layer)
-   [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Multi-crate organization

## References

-   [dill-rs GitHub](https://github.com/sergiimk/dill-rs) - dill source and documentation (v0.15.0)
-   [dill on crates.io](https://crates.io/crates/dill) - 56,675 downloads, actively maintained
-   [kamu-cli](https://github.com/kamu-data/kamu-cli) - Production example using dill
-   [Rust DI Libraries Comparison](https://users.rust-lang.org/t/comparing-dependency-injection-libraries-shaku-nject/102619) - Community discussion

## Migration Scope (Verified 2026-01-18)

**Actual Shaku usage in production code:**

| Category | Count | Files |
|----------|-------|-------|
| `#[derive(Component)]` | 1 | `mcb-providers/src/embedding/openai.rs` |
| `module!` macro | 4 | `di/modules/{admin,server,infrastructure,traits}.rs` |
| `use shaku::Interface` | ~20 | All port traits in `mcb-application/src/ports/**` |
| `: Interface` trait bound | ~20 | Same files as above |

**Total files requiring changes**: ~25 files (not 5-6 as originally estimated)
