<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
<!-- markdownlint-disable MD025 -->
adr: 51
title: SeaQL + Loco.rs Platform Rebuild
status: ACCEPTED
created: 2026-02-22
updated: 2026-02-22
related: [49, 50, 3, 8, 9, 10]
supersedes: []
superseded_by: []
implementation_status: In Progress
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 051: SeaQL + Loco.rs Platform Rebuild

## Status

**Accepted** (v0.3.0)

> Platform rebuild decision documenting migration from custom infrastructure to SeaQL ecosystem + Loco.rs framework.
> See ADR-049 for schema resolution decisions.

## Context

### Current State Crisis

MCB v0.2.1 is **alpha quality with critical infrastructure failures**:

| Metric | Value | Status |
|--------|-------|--------|
| MCP Tools | 2 broken, 2 partial, 1 stub | ❌ Critical |
| TODO/FIXME | 247 markers | ⚠️ High |
| Custom Infrastructure | ~9,000 LOC | ⚠️ Unmaintainable |
| Missing Tests | Database layer has ZERO tests | ❌ Critical |

**Broken Tools** (blocking production use):
- `mcb_memory list` — SQL syntax bug (confirmed)
- `mcb_session create` — Schema validation bug (confirmed)
- `mcb_agent` — SQL storage bug (confirmed)
- `mcb_vcs` — Repository discovery bug (confirmed)
- `mcb_project` — Stubs only, no implementation

### Infrastructure Debt Analysis

**Custom persistence layer** (`~3,827 LOC`):
- Raw SQL/SQLite via sqlx
- No migration system — schema changes manual
- Zero test coverage
- Repetitive boilerplate for CRUD operations

**Custom admin system** (`~5,062 LOC`):
- Handlebars-based CRUD interface
- No background job support
- No GraphQL API
- Ad-hoc routing and middleware

**Event bus** (2 implementations):
- Tokio broadcast (in-memory only, no persistence)
- NATS (external dependency, adds ops complexity)
- No consumer groups, no replay capability

### Product Reality

The user explicitly stated:
> "Product is completely alpha with broken tools — nothing worth 'preserving' from broken infra"
>
> "Focus on REAL product functionality, not creating a framework, but keeping project organization at the highest level"

This is not a conservative refactor. This is a **platform rebuild** to establish a maintainable foundation.

## Decision

### Platform Migration

Rebuild MCB v0.3.0 on the **SeaQL ecosystem** + **Loco.rs framework**:

| Component | Current | New | Rationale |
|-----------|---------|-----|-----------|
| Database | sqlx + raw SQL | **SeaORM 2.x** | Type-safe entities, migrations, relations |
| Query Building | String concatenation | **SeaQuery** | Structured, composable, safe |
| Admin Runtime | Custom Handlebars | **Loco.rs** | Background jobs, middleware, structured routing |
| Admin UI | Custom CRUD | **SeaORM Pro** (MIT) | Production-ready admin panel |
| API | REST only | **Seaography** GraphQL | Flexible querying, code generation |
| Events | Tokio broadcast + NATS | **SeaStreamer** | Persistence, consumer groups, multiple backends |

### Version Bumping

The scope of this rebuild redefines the version roadmap:

| Old Version | Content | New Version | Status |
|-------------|---------|-------------|--------|
| v0.3.0 | Workflow (FSM, Scout, Policies) | **v0.4.0** | Postponed |
| v0.4.0 | Integrated Context (Knowledge Graph) | **v0.5.0** | Postponed |
| v0.5.0 | Enterprise Features | **v1.0.0** | Unchanged |

**New v0.3.0** = This SeaQL + Loco.rs platform rebuild

### Architecture Decisions

#### 1. SeaORM 2.x Persistence

**Decision**: Use SeaORM 2.0.0-rc.34 (pinned exact version)

```rust
// Entity-first approach with schema-sync
derive_entity!(Observations);
derive_entity!(Sessions);
// ... all current schema tables
```

**Rationale**:
- Type-safe database operations (compile-time query validation)
- Built-in migration system (`sea-orm-cli migrate`)
- Entity relations with lazy/eager loading
- Database abstraction (SQLite now, Postgres later)

**Trade-off**: RC version carries pre-release risk, but `2.0.0-rc.34` is production-tested by SeaQL team.

#### 2. Loco.rs Foundation

**Decision**: Loco.rs as admin runtime and web framework

```rust
// Loco app structure
loco::app::App {
    controllers: vec![admin::routes(), graphql::routes()],
    workers: vec![indexing::Worker, cleanup::Worker],
    hooks: lifecycle::Hooks,
}
```

**Rationale**:
- Background jobs via `BackgroundAsync` (no Redis required)
- Scheduler with cron expressions
- Middleware stack (auth, logging, cors)
- Structured project layout (controllers, models, workers)

**Coexistence Pattern**: Loco.rs + MCP (`rmcp`) run as separate Tokio tasks, sharing the database connection pool.

#### 3. SeaORM Pro Admin Panel

**Decision**: Include SeaORM Pro (MIT licensed)

```bash
# Build-time download
./scripts/download_frontend.sh  # Downloads SeaORM Pro v2.0.0-rc.1
```

**Rationale**:
- Production-ready admin UI for all SeaORM entities
- Built-in Seaography GraphQL integration
- MIT license allows commercial use
- Mounts at `/admin` with configurable auth

#### 4. SeaStreamer Events

**Decision**: Replace Tokio broadcast + NATS with SeaStreamer

```rust
// SeaStreamer backend selection
#[cfg(dev)]
const BACKEND: &str = "file:///tmp/mcb-events";  // Dev

#[cfg(prod)]
const BACKEND: &str = "redis://localhost:6379";  // Prod
```

**Rationale**:
- Unified API across backends (Redis Streams, Kafka, file)
- Consumer groups for load balancing
- Message persistence and replay
- `file://` backend for zero-dependency development

**Semantic Change**: Broadcast → Consumer Groups. Event handlers must be idempotent.

#### 5. Port Codegen

**Decision**: Code generation for CRUD registry traits (~20 of 77 traits)

```rust
// Generated by build.rs or proc-macro
#[derive(PortCodegen)]
trait ObservationRepository {
    async fn create(&self, obs: Observation) -> Result<Observation>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Observation>>;
    // ... 18 more methods auto-generated
}
```

**Rationale**:
- Eliminates ~2,000 LOC of repetitive trait implementations
- Maintains Clean Architecture (domain traits, infrastructure implementations)
- Focuses developer effort on business logic, not boilerplate

**Scope**: Only CRUD registry traits. Complex business logic traits remain manual.

### What Stays Unchanged

| Component | Reason |
|-----------|--------|
| **Vector Stores** | EdgeVec, Milvus, Qdrant, Pinecone work correctly — semantic search is the primary value proposition |
| **Provider Registration** | linkme distributed slices (42 registrations) are stable and efficient |
| **mcb-validate** | 349 tests, working — out of scope for this rebuild |
| **MCP Protocol** | Tool contracts preserved, JSON-RPC schema unchanged |
| **Clean Architecture** | Inward-only dependency flow maintained |

### What Gets Deleted

- `crates/mcb-providers/src/database/sqlx/` (~3,827 LOC)
- `crates/mcb-infrastructure/src/admin/` custom Handlebars admin (~5,062 LOC)
- `crates/mcb-infrastructure/src/events/broadcast.rs`
- `crates/mcb-providers/src/events/nats/`
- All raw SQL string concatenation

## Consequences

### Positive Consequences

1. **Fix All Broken Tools**: Contract tests + type-safe SeaORM eliminate SQL bugs
2. **Zero Custom Persistence**: ~3,800 LOC deleted, replaced with battle-tested SeaORM
3. **Production Admin**: SeaORM Pro provides enterprise-grade admin UI
4. **Background Jobs**: Loco.rs enables async processing (indexing, cleanup, notifications)
5. **GraphQL API**: Flexible querying without versioned REST endpoints
6. **Event Persistence**: SeaStreamer enables replay, audit trails, and recovery
7. **Developer Velocity**: Port codegen eliminates repetitive CRUD implementations
8. **Test Coverage**: SeaORM + Loco have established testing patterns; contract tests catch regressions
9. **Future-Proof**: SQLite → Postgres migration path; event backend swaps

### Negative Consequences

1. **Migration Risk**: Data migration from raw SQLite to SeaORM entities
2. **Learning Curve**: Team must learn SeaORM, Loco.rs, SeaStreamer patterns
3. **RC Dependency**: SeaORM 2.0.0-rc.34 is pre-release (migration path to stable exists)
4. **Binary Size**: Loco.rs + SeaORM Pro adds ~15-20% to binary (acceptable under 1.2x limit)
5. **Compile Time**: More dependencies increase `cargo check` time (target: <2x current)
6. **Event Semantics**: Broadcast → Consumer Groups requires idempotent handlers
7. **Coexistence Complexity**: Loco.rs + MCP as separate Tokio tasks adds coordination overhead

### Neutral Consequences

1. **API Surface**: Public MCP tool contracts unchanged
2. **Clean Architecture**: Layer boundaries preserved
3. **Provider Pattern**: linkme registration unchanged
4. **Configuration**: Figment/TOML config continues to work

## Alternatives Considered

### Alternative 1: Fix Existing Infrastructure

**Description**: Debug and fix the 247 TODO/FIXMEs, add tests to sqlx layer, improve custom admin.

**Pros**:
- No migration risk
- Keeps existing codebase

**Cons**:
- ~9,000 LOC of custom infra to maintain forever
- No migration system — schema changes remain manual
- No background job support
- No GraphQL API
- No event persistence

**Rejection**: "Nothing worth preserving from broken infra" — the custom infrastructure is fundamentally flawed, not just buggy.

### Alternative 2: Use Diesel Instead of SeaORM

**Description**: Diesel is mature, stable, widely used.

**Pros**:
- Stable 2.x release (no RC dependency)
- Excellent compile-time query checking
- Mature ecosystem

**Cons**:
- No built-in async support (requires `diesel-async`)
- No GraphQL/codegen ecosystem like Seaography
- No admin panel like SeaORM Pro
- Sync-first design clashes with async-first MCB

**Rejection**: SeaORM's async-native design + Seaography + SeaORM Pro ecosystem is purpose-built for this use case.

### Alternative 3: Use Axum Directly Instead of Loco.rs

**Description**: Axum is the standard Rust web framework, more flexible than Loco.rs.

**Pros**:
- Maximum flexibility
- Smaller dependency tree
- Direct Tower integration

**Cons**:
- No structured project layout
- No background job system
- No scheduler
- Build admin panel from scratch

**Rejection**: Loco.rs provides the admin runtime, jobs, and structure needed — rebuilding these on raw Axum recreates the custom infra problem.

### Alternative 4: Keep NATS, Drop Tokio Broadcast

**Description**: Use NATS as the sole event backend.

**Pros**:
- Single event system
- Production-proven

**Cons**:
- External dependency for development
- Adds ops complexity (NATS server)
- No file-based backend for testing

**Rejection**: SeaStreamer's multi-backend (file/Redis/Kafka) enables zero-dependency dev and production flexibility.

### Alternative 5: Exclude SeaORM Pro (Build Custom Admin)

**Description**: Build custom admin UI on Loco.rs instead of using SeaORM Pro.

**Pros**:
- Full control over UI
- Smaller bundle size

**Cons**:
- ~2,000+ LOC of admin UI to write and maintain
- No GraphQL integration
- Months of development

**Rejection**: SeaORM Pro is MIT licensed and production-ready. "Focus on REAL product functionality, not creating a framework."

## Implementation Notes

### Critical Path

1. **Contract Tests First**: Snapshot tests for all 9 MCP tools before any migration
2. **Loco+rmcp Spike**: Proof-of-concept for coexistence pattern
3. **Schema Resolution**: Domain `Schema` model vs SeaORM entities (ADR-049)
4. **Entity Generation**: SeaORM entities for all current schema tables
5. **Repository Migration**: Port one entity at a time with tests
6. **Admin Migration**: Loco.rs scaffolding + SeaORM Pro integration
7. **Event Migration**: SeaStreamer implementation + handler migration
8. **Cleanup**: Delete old sqlx, custom admin, broadcast/NATS code

### Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Data Loss | Full backup before migration; rollback script |
| SeaORM RC bugs | Pin exact version; upgrade path to 2.0 stable |
| Loco+rmcp incompatibility | Mandatory spike before main work |
| Compile time explosion | Feature flags; workspace dependency optimization |
| Binary size | Strip symbols; UPX compression if needed |

### Migration Strategy

```
Phase 1: Validation (Contract tests, spike)
Phase 2: Foundation (Dependencies, entities, migrations)
Phase 3: Persistence (SeaORM repos, fix bugs)
Phase 4: Admin (Loco.rs, SeaORM Pro, GraphQL)
Phase 5: Events (SeaStreamer, handler migration)
Phase 6: Cleanup (Delete old code, final validation)
```

## Canonical References

- [ROADMAP.md](../developer/ROADMAP.md) — Version roadmap with bumped versions (normative)
- [CHANGELOG.md](../operations/CHANGELOG.md) — v0.3.0 release notes (normative)
- [PLAN: v030-seaql-loco-rebuild.md](../../../.sisyphus/plans/v030-seaql-loco-rebuild.md) — Detailed execution plan
- [ADR 049: Schema Resolution](049-axum-return-rmcp-tower-compatibility.md) — Schema model decisions

## References

- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [SeaORM Pro](https://www.sea-ql.org/sea-orm-pro/) — MIT licensed admin panel
- [Loco.rs](https://loco.rs/) — Rust web framework
- [SeaStreamer](https://github.com/SeaQL/sea-streamer) — Streaming processor
- [Seaography](https://github.com/SeaQL/seaography) — GraphQL framework
- [ADR 003: Unified Provider Architecture](003-unified-provider-architecture.md)
- [ADR 008: Git-Aware Semantic Indexing](008-git-aware-semantic-indexing-v0.2.0.md)
- [ADR 010: Hooks Subsystem](010-hooks-subsystem-agent-backed.md)
