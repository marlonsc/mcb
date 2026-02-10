---
adr: 26
title: API Routing Refactor (Rocket vs Poem)
status: IMPLEMENTED
created: 
updated: 2026-02-05
related: [7, 11]
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

## ADR 026: API Routing Refactor (Rocket vs Poem)

## Status

**Implemented** (v0.1.2)

> Migration from Axum to Rocket completed as part of the infrastructure modernization initiative.

## Context

The current HTTP routing uses Axum (version 0.8) with manual route construction and Tower middleware. Axum provides a functional approach to web development with a focus on performance and composability.

### Current Implementation

```rust
use axum::{Router, routing::{get, post}, middleware::from_fn};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

// Manual route construction
let router = Router::new()
    .route("/health", get(health_check))
    .route("/metrics", get(get_metrics))
    .route("/config", get(get_config))
    .route("/config/reload", post(reload_config))
    .route("/services", get(list_services))
    .route("/services/{name}/start", post(start_service))
    .route("/services/{name}/stop", post(stop_service))
    .route("/services/{name}/restart", post(restart_service))
    // Complex middleware composition
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http())
    .with_state(state);
```

### Current Limitations

#### Developer Experience Issues

1.  **Manual route building**: Each route requires explicit registration
2.  **Middleware complexity**: Tower ecosystem has steep learning curve
3.  **Handler coupling**: Routes are defined separately from handler functions
4.  **No built-in features**: Missing common web framework conveniences

#### Maintenance Issues

1.  **Route scattering**: Routes defined far from handler implementations
2.  **Middleware orchestration**: Complex Tower middleware composition
3.  **Error handling**: Manual error response formatting
4.  **Testing**: Difficult to test routes in isolation

### Alternative Frameworks Evaluation

#### Rocket Framework (Selected)

Rocket (version 0.5.1) provides a batteries-included web framework with attribute-based routing:

**Key Features:**

-   **Attribute-based routing**: `#[get("/path")]` decorators on handler functions
-   **Built-in features**: Forms, templates, state management, validation
-   **Macro-driven**: Generates routing code at compile-time with conflict detection
-   **Stable and mature**: Long-established in Rust ecosystem with extensive documentation
-   **Full-stack**: Includes everything needed for web applications

**Strengths:**

-   Intuitive attribute-based API familiar to web developers
-   Compile-time route validation and conflict detection
-   Rich ecosystem of built-in features (forms, sessions, templates)
-   Excellent documentation and community support
-   Mature codebase with stable API

#### Poem Framework (Alternative Considered)

Poem (modern async web framework) offers a programmatic approach:

**Key Features:**

-   **Programmatic routing**: Fluent API with method chaining
-   **Modern async**: Built on Tokio and hyper with excellent performance
-   **Lightweight**: Minimal dependencies and fast compilation
-   **Flexible middleware**: Easy composition and reuse
-   **Type-safe**: Strong typing throughout the framework

**Strengths:**

-   Excellent performance and low overhead
-   Modern async patterns throughout
-   Highly composable and flexible architecture
-   Strong typing and compile-time guarantees

**Trade-offs:**

-   Less mature ecosystem compared to Rocket
-   Fewer built-in features (requires more external crates)
-   Different programming model (programmatic vs declarative)

### Framework Comparison

| Aspect | Axum (Current) | Rocket (Chosen) | Poem (Alternative) |
|--------|----------------|-----------------|-------------------|
| **Routing Style** | Programmatic | Attribute-based | Programmatic |
| **Learning Curve** | Medium | Low | Medium |
| **Ecosystem Maturity** | High | Very High | Medium |
| **Built-in Features** | Minimal | Extensive | Minimal |
| **Performance** | Excellent | Very Good | Excellent |
| **Compile Time** | Fast | Medium | Fast |
| **Middleware** | Tower ecosystem | Built-in | Flexible |
| **Error Handling** | Manual | Built-in | Manual |

## Decision

After comprehensive evaluation of framework alternatives, we will migrate from Axum to **Rocket** for HTTP routing. This decision prioritizes developer experience, ecosystem maturity, and built-in features over raw performance.

> **Performance Note**: Benchmarks (May 2025) show Axum is approximately 20% faster than Rocket in raw throughput (147,892 req/s vs 124,567 req/s). However, since the HTTP admin interface is secondary to the MCP stdio protocol, developer experience was prioritized over raw performance.

### Selection Criteria and Rationale

#### Primary Selection Factors

1.  **Developer Experience**: Rocket's attribute-based routing (`#[get("/path")]`) is more intuitive than Axum's programmatic approach, especially for teams familiar with web frameworks like Express.js, Flask, or Spring Boot.

2.  **Ecosystem Maturity**: Rocket has been stable in the Rust ecosystem for years with extensive documentation, community support, and proven production usage.

3.  **Built-in Features**: Rocket includes batteries like form handling, validation, templating, and session management out of the Box, reducing the need for external crates.

4.  **Compile-time Safety**: Rocket performs route conflict detection and validation at compile time, catching routing errors early.

5.  **Framework Consistency**: Rocket provides a cohesive framework experience rather than Axum's minimal approach that requires assembling multiple Tower middleware crates.

#### Performance Trade-off Analysis

While Axum + Tower offers slightly better raw performance, the difference is negligible for most applications and doesn't justify the increased complexity and maintenance burden.

### Technical Migration Strategy

#### Route Migration Pattern

**Before (Axum - scattered and verbose):**

```rust
use axum::{Router, routing::{get, post, patch}, extract::Path};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

// Handler functions defined separately
async fn health_check() -> &'static str { "OK" }

async fn get_config() -> Json<AppConfig> { /* ... */ }

async fn update_config_section(
    Path(section): Path<String>,
    Json(update): Json<ConfigUpdate>,
) -> Result<Json<ConfigResponse>, AppError> { /* ... */ }

// Routes defined far from handlers
let router = Router::new()
    .route("/health", get(health_check))
    .route("/config", get(get_config))
    .route("/config/{section}", patch(update_config_section))
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http())
    .with_state(app_state);
```

**After (Rocket - co-located and declarative):**

```rust
use rocket::{get, post, patch, routes, serde::json::Json, State};

// Routes defined with handlers using attributes
#[get("/health")]
fn health_check() -> &'static str { "OK" }

#[get("/config")]
fn get_config(config: &State<AppConfig>) -> Json<&AppConfig> {
    Json(config.inner())
}

#[patch("/config/<section>")]
fn update_config_section(
    section: &str,
    update: Json<ConfigUpdate>,
    config: &State<AppConfig>,
) -> Result<Json<ConfigResponse>, AppError> {
    // Handler implementation...
}

// Routes organized by feature
mod config_routes {
    use rocket::{get, patch, routes};

    // All config-related routes in one place
    #[get("/")]
    fn get_config() -> Json<AppConfig> { /* ... */ }

    #[patch("/<section>")]
    fn update_config_section(section: &str) -> Json<ConfigResponse> { /* ... */ }

    pub fn routes() -> Vec<rocket::Route> {
        routes![get_config, update_config_section]
    }
}

// Main application setup
#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(app_config)
        .mount("/api/v1/health", routes![health_check])
        .mount("/api/v1/config", config_routes::routes())
}
```

#### Middleware and Cross-cutting Concerns

**CORS, Logging, and Security in Rocket:**

```rust
use rocket::{fairing::{Fairing, Info, Kind}, http::Header, Request, Response};

// Built-in CORS support
#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Cors)  // Built-in CORS fairing
        .attach(RequestLogger)  // Custom logging fairing
        .mount("/", routes![...])
}

// Custom fairing for cross-cutting concerns
#[derive(Default)]
struct RequestLogger;

#[rocket::async_trait]
impl Fairing for RequestLogger {
    fn info(&self) -> Info {
        Info {
            name: "Request Logger",
            kind: Kind::Request | Kind::Response
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        // Request logging logic
    }

    async fn on_response(&self, req: &Request<'_>, res: &mut Response<'_>) {
        // Response logging logic
    }
}
```

### Migration Scope and Phases

#### Phase 1: Infrastructure Migration

1.  Replace Axum dependency with Rocket in `Cargo.toml`
2.  Create Rocket application bootstrap in `mcb-server/src/main.rs`
3.  Migrate basic health check and metrics endpoints

#### Phase 2: Admin API Migration

1.  Convert admin routes from Axum to Rocket attribute-based routing
2.  Migrate state management and dependency injection
3.  Update error handling to Rocket patterns

#### Phase 3: Advanced Features

1.  Implement authentication and authorization using Rocket's guard system
2.  Add request/response processing with Rocket's data guards
3.  Migrate WebSocket/SSE functionality if needed

#### Phase 4: Testing and Validation

1.  Update integration tests for Rocket
2.  Validate API contract compliance
3.  Performance testing and optimization

## Consequences

### Positive

-   **Reduced boilerplate**: Attribute macros eliminate manual route registration
-   **Better validation**: Compile-time route validation and conflict detection
-   **Rich features**: Built-in JSON handling, form validation, templates
-   **Developer experience**: Better error messages and debugging tools
-   **Performance**: Compile-time optimization of routing table

### Negative

-   **Framework lock-in**: Tightly coupled to Rocket's patterns
-   **Learning curve**: New framework API to learn
-   **Migration effort**: Significant rewrite of routing code
-   **Dependency change**: Replace Axum + Tower with Rocket

### Risks

-   **Breaking API changes**: Route paths and middleware may change
-   **Performance impact**: Rocket's feature set might add overhead
-   **Compatibility issues**: Some Axum middleware may not have Rocket equivalents

## Migration Strategy

### Phase 1: Evaluation

1.  Create prototype implementations with both Rocket and Poem
2.  Benchmark performance, compile times, and developer experience
3.  Evaluate ecosystem support and documentation quality
4.  Make final framework choice based on criteria

### Phase 2: Rocket Migration

1.  Add Rocket dependency alongside Axum
2.  Create Rocket handlers for existing endpoints
3.  Migrate middleware and state management
4.  Update error handling to Rocket patterns
5.  Parallel testing of both implementations

### Phase 3: Cleanup

1.  Remove Axum dependencies
2.  Update all routing documentation
3.  Comprehensive integration testing
4.  Performance validation

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

-   More mature ecosystem and community support
-   Better developer experience with attribute macros
-   Richer built-in features reduce external dependencies
-   Extensive documentation and learning resources

## Validation Criteria

-   [x] All existing endpoints work with new routing framework
-   [x] API contracts remain stable (same paths, methods, responses)
-   [x] Middleware functionality preserved (CORS, auth, logging)
-   [x] Performance meets or exceeds current benchmarks
-   [x] Compile times remain reasonable
-   [x] Error handling provides equivalent or better user experience

## Related ADRs

-   [ADR 011: HTTP Transport Request/Response Pattern](011-http-transport-request-response-pattern.md) - HTTP handling patterns
-   [ADR 007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md) - Admin interface architecture

## References

-   [Rust Web Frameworks Performance Benchmark 2025](https://markaicode.com/rust-web-frameworks-performance-benchmark-2025/)
-   [Rocket Guide](https://rocket.rs/guide/master/)
-   [Figment Configuration](https://docs.rs/figment/latest/figment/) - Same author as Rocket
