<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 49
title: Axum Return for rmcp Tower Compatibility
status: ACCEPTED
created: 2026-02-21
updated: 2026-02-21
related: [26, 33]
supersedes: [26]
superseded_by: []
implementation_status: Planned
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 049: Axum Return for rmcp Tower Compatibility

## Status

**Accepted** (v0.2.2)

> Supersedes [ADR 026: API Routing Refactor (Rocket vs Poem)]
> (026-routing-refactor-rocket-poem.md)
>
> The decision to migrate from Axum to Rocket (ADR-026) is reversed due to
> rmcp's `StreamableHttpService` requiring Tower compatibility, which Rocket
> does not provide.

## Context

In ADR-026, we migrated from Axum to Rocket, prioritizing developer experience
and built-in features over raw performance. The rationale was that the HTTP
admin interface was secondary to the MCP stdio protocol, making framework
ergonomics more important than performance.

### The rmcp Integration Challenge

The [rmcp](https://github.com/modelcontextprotocol/rust-sdk) crate (v0.16+)
provides a `StreamableHttpService` that implements the MCP protocol over HTTP.
This service is built on the Tower ecosystem:

```rust
// rmcp's StreamableHttpService is a Tower Service
use rmcp::transport::http::StreamableHttpService;
use tower::Service;

// StreamableHttpService implements Tower's Service trait
impl Service<Request<Body>> for StreamableHttpService {
    type Response = Response<Body>;
    type Error = rmcp::Error;
    type Future = ...;
    
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // MCP protocol handling
    }
}
```

### Tower Compatibility Requirement

To integrate rmcp's HTTP transport, we need to mount `StreamableHttpService`
as a route in our web framework. This requires:

1. **Tower `Service` compatibility**: The framework must be able to wrap or
   route to Tower services
2. **Hyper integration**: Tower services use Hyper's request/response types
3. **Middleware compatibility**: CORS, tracing, and auth must work with
   Tower-based handlers

### Rocket Limitations

Rocket (v0.5) does not support Tower integration:

| Feature | Rocket Support | Tower Requirement |
|---------|----------------|-------------------|
| `Service` trait | ❌ No native support | ✅ Required by rmcp |
| Hyper types | ❌ Uses own request types | ✅ Required by Tower |
| Tower middleware | ❌ Incompatible | ✅ CORS, tracing, auth |
| Async ecosystem | ⚠️ Custom runtime | ✅ Tokio-native |

While Rocket provides excellent developer experience through attribute-based
routing, it uses its own request/response abstractions that are incompatible
with Tower's `Service` trait.

### Axum-Tower Ecosystem Compatibility

Axum (v0.8) is built on Tower and provides seamless integration:

```rust
use axum::{Router, routing::post};
use rmcp::transport::http::StreamableHttpService;
use tower_http::cors::CorsLayer;

// StreamableHttpService can be mounted directly
let mcp_service = StreamableHttpService::new(sse_transport);

let router = Router::new()
    .route("/mcp", post(mcp_service))
    .layer(CorsLayer::permissive());
```

## Decision

We will migrate back to Axum (v0.8) from Rocket to enable rmcp's
`StreamableHttpService` integration.

### Selection Criteria

| Factor | Weight | Rocket | Axum |
|--------|--------|--------|------|
| rmcp Tower compatibility | **Critical** | ❌ No | ✅ Yes |
| Developer experience | High | ✅ Excellent | ⚠️ Good |
| Ecosystem maturity | Medium | ✅ Very High | ✅ High |
| Performance | Low | ✅ Very Good | ✅ Excellent |

### Technical Rationale

1. **MCP Protocol Support**: rmcp's HTTP transport is the primary driver.
   Without Tower compatibility, we cannot support MCP over HTTP, which is
   becoming the standard transport for IDE integrations (Claude Desktop,
   Cursor, Windsurf).

2. **Ecosystem Alignment**: The broader Rust async ecosystem is converging
   on Tower as the standard abstraction. Axum's Tower-native design aligns
   with this direction.

3. **Middleware Investment**: Our existing Tower middleware (CORS, tracing,
   auth) can be reused without rewrite.

### What We Lose from Rocket

| Rocket Feature | Impact | Mitigation |
|----------------|--------|------------|
| Attribute-based routing | Medium | Axum's `#[axum::debug_handler]` + macros |
| Built-in forms/validation | Low | Use `validator` + `serde` crates |
| Compile-time route validation | Low | Axum's type-safe handlers catch errors |
| Rich documentation | Low | Axum docs are excellent |

### What We Gain from Tower Compatibility

| Tower Capability | Benefit |
|------------------|---------|
| `StreamableHttpService` integration | Native MCP over HTTP |
| Middleware composition | CORS, auth, tracing work out-of-box |
| Ecosystem interoperability | Works with any Tower service |
| Future-proofing | Industry standard abstraction |

## Consequences

### Positive

- **MCP over HTTP**: Full support for rmcp's HTTP transport
- **Tower ecosystem**: Access to rich middleware ecosystem
- **Standard abstractions**: Aligns with industry direction
- **Simpler integration**: No adapter layers needed for rmcp

### Negative

- **Migration effort**: Revert Rocket routes back to Axum patterns
- **Learning curve**: Team re-acclimation to Axum patterns
- **Lost Rocket features**: Attribute macros, built-in validation
- **Boilerplate increase**: More explicit route registration

### Neutral

- **Performance**: Axum is slightly faster (as noted in ADR-026)
- **Compile times**: Similar to pre-Rocket state

## Migration Strategy

### Phase 1: Axum Reintroduction

1. Replace Rocket dependency with Axum in workspace `Cargo.toml`
2. Add `rmcp` with HTTP transport features
3. Restore Axum application bootstrap

### Phase 2: Route Migration

1. Convert Rocket attribute-based handlers to Axum functions
2. Restore manual route registration
3. Migrate state management to Axum's `State` extractor

### Phase 3: MCP Integration

1. Configure `StreamableHttpService` from rmcp
2. Mount MCP routes alongside admin API
3. Implement SSE transport for MCP streaming

### Phase 4: Middleware Restoration

1. Restore Tower middleware stack (CORS, tracing, auth)
2. Validate middleware works with MCP routes
3. Performance testing

## Comparison: Rocket vs Tower/rmcp Benefits

| Aspect | Rocket Provided | Tower/rmcp Provides |
|--------|-----------------|---------------------|
| **Routing ergonomics** | Attribute macros | Function-based with macros |
| **MCP HTTP support** | ❌ Not possible | ✅ Native via StreamableHttpService |
| **Middleware** | Built-in fairings | Tower ecosystem |
| **Type safety** | Compile-time validation | Handler type checking |
| **Community** | Mature, stable | Growing, standard-aligned |
| **IDE integration** | N/A | MCP over HTTP enables all IDEs |

## Related ADRs

- [ADR 026: API Routing Refactor (Rocket vs Poem)]
  (026-routing-refactor-rocket-poem.md) — **SUPERSEDED** by this ADR
- [ADR 033: MCP Handler Consolidation]
  (033-mcp-handler-consolidation.md) — MCP integration patterns

## References

- [rmcp crate](https://github.com/modelcontextprotocol/rust-sdk) —
  Official Rust MCP SDK
- [Tower Service trait](https://docs.rs/tower/latest/tower/trait.Service.html) —
  Core abstraction
- [Axum documentation](https://docs.rs/axum/latest/axum/) — Web framework
- [ADR 026](026-routing-refactor-rocket-poem.md) — Original Rocket migration
