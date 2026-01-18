# ADR 026: API Routing Refactor (Rocket vs Poem)

## Status

**Proposed** (v0.2.0)

> Evaluation and migration decision for HTTP routing framework as part of the simplification initiative.

## Context

The current HTTP routing uses Axum with manual route construction:

```rust
let router = Router::new()
    .route("/health", get(health_check))
    .route("/metrics", get(get_metrics))
    .route("/config", get(get_config))
    .route("/config/reload", post(reload_config))
    .with_state(state);
```

This approach has limitations:
1. **Manual route building**: Verbose route definitions
2. **Middleware complexity**: Tower middleware composition is verbose
3. **Handler coupling**: Routes tightly coupled to handler functions
4. **No built-in features**: Missing common web framework features

Two alternatives are being evaluated:

### Rocket Framework
- **Attribute-based routing**: `#[get("/path")]` decorators
- **Built-in features**: Forms, templates, state management
- **Macro-driven**: Generates routing code at compile-time
- **Stable and mature**: Long-established in Rust ecosystem

### Poem Framework
- **Programmatic routing**: Fluent API for route construction
- **Modern async**: Built on Tokio and hyper
- **Lightweight**: Minimal overhead, fast compilation
- **Flexible middleware**: Easy composition and reuse

## Decision

After evaluation, we will migrate to **Rocket** for the following reasons:

1. **Familiarity**: Attribute-based routing is more intuitive
2. **Ecosystem maturity**: Extensive community support and documentation
3. **Built-in features**: Forms, validation, templates reduce boilerplate
4. **Code generation**: Compile-time route validation and optimization
5. **Developer experience**: Better error messages and debugging

### Rocket Migration Pattern

**Before (Axum):**
```rust
use axum::{Router, routing::get};

fn health_check() -> &'static str { "OK" }

let router = Router::new().route("/health", get(health_check));
```

**After (Rocket):**
```rust
use rocket::{get, routes};

#[get("/health")]
fn health_check() -> &'static str { "OK" }

let rocket = rocket::build().mount("/", routes![health_check]);
```

### Route Organization

Routes will be organized by feature area:

```rust
mod health {
    use rocket::{get, post};

    #[get("/health")]
    fn check() -> &'static str { "OK" }

    #[get("/health/extended")]
    fn extended() -> Json<HealthStatus> { /* ... */ }

    pub fn routes() -> Vec<rocket::Route> {
        routes![check, extended]
    }
}

// In main router
.mount("/api/v1", health::routes())
```

## Consequences

### Positive
- **Reduced boilerplate**: Attribute macros eliminate manual route registration
- **Better validation**: Compile-time route validation and conflict detection
- **Rich features**: Built-in JSON handling, form validation, templates
- **Developer experience**: Better error messages and debugging tools
- **Performance**: Compile-time optimization of routing table

### Negative
- **Framework lock-in**: Tightly coupled to Rocket's patterns
- **Learning curve**: New framework API to learn
- **Migration effort**: Significant rewrite of routing code
- **Dependency change**: Replace Axum + Tower with Rocket

### Risks
- **Breaking API changes**: Route paths and middleware may change
- **Performance impact**: Rocket's feature set might add overhead
- **Compatibility issues**: Some Axum middleware may not have Rocket equivalents

## Migration Strategy

### Phase 1: Evaluation
1. Create prototype implementations with both Rocket and Poem
2. Benchmark performance, compile times, and developer experience
3. Evaluate ecosystem support and documentation quality
4. Make final framework choice based on criteria

### Phase 2: Rocket Migration
1. Add Rocket dependency alongside Axum
2. Create Rocket handlers for existing endpoints
3. Migrate middleware and state management
4. Update error handling to Rocket patterns
5. Parallel testing of both implementations

### Phase 3: Cleanup
1. Remove Axum dependencies
2. Update all routing documentation
3. Comprehensive integration testing
4. Performance validation

### Alternative: Poem Evaluation

If Poem had been chosen, the migration would look like:

```rust
use poem::{get, Route, Server};

let app = Route::new()
    .at("/health", get(health_check))
    .at("/metrics", get(get_metrics));

Server::new(TcpListener::bind("127.0.0.1:3000")).run(app).await;
```

**Why Rocket over Poem:**
- More mature ecosystem and community support
- Better developer experience with attribute macros
- Richer built-in features reduce external dependencies
- Extensive documentation and learning resources

## Validation Criteria

- [ ] All existing endpoints work with new routing framework
- [ ] API contracts remain stable (same paths, methods, responses)
- [ ] Middleware functionality preserved (CORS, auth, logging)
- [ ] Performance meets or exceeds current benchmarks
- [ ] Compile times remain reasonable
- [ ] Error handling provides equivalent or better user experience

## Related ADRs

- [ADR 011: HTTP Transport Request/Response Pattern](011-http-transport-request-response-pattern.md) - HTTP handling patterns
- [ADR 007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md) - Admin interface architecture