# tokio

Last updated: 2026-02-12

> Scope: project-specific Tokio usage in MCB, operational risk, and verification guidance.  
> Cross-reference: `context/external/tracing.md`, `context/external/git2.md`, `context/external/rocket.md`, `context/external/sqlx.md`.

## Executive Summary

Tokio is the execution substrate for nearly every asynchronous workflow in MCB:

- Server runtime and tool execution flow
- Background indexing and maintenance jobs
- Event streaming and pub/sub adapters
- Async file and process operations
- Integration and end-to-end test harnesses

The architecture depends on Tokio for both throughput and correctness. If Tokio patterns degrade, cross-crate behavior degrades with it.

This aligns with architecture boundary documents in `context/project-intelligence/clean-architecture.md` and `context/project-intelligence/architecture-boundaries.md`.

## Actual MCB Usage (Current Source of Truth)

### Where Tokio Is Used (Code-Grounded Coverage)

Representative hotspots:

- Binary entrypoint: `crates/mcb/src/main.rs:52` (`#[tokio::main]`)
- Server boot and transport orchestration: `crates/mcb-server/src/init.rs`
- HTTP transport and request lifecycle: `crates/mcb-server/src/transport/http.rs`
- Background indexing task spawn: `crates/mcb-application/src/use_cases/indexing_service.rs:214`
- Hybrid memory search concurrency: `crates/mcb-application/src/use_cases/memory_service.rs:228`
- Config watcher and shared state: `crates/mcb-infrastructure/src/config/watcher.rs`
- Event bus implementations: `crates/mcb-providers/src/events/tokio.rs`, `crates/mcb-providers/src/events/nats.rs`
- Blocking adapter boundaries:
  - `crates/mcb-providers/src/git/submodule.rs:48`
  - `crates/mcb-providers/src/language/engine.rs:130`
  - `crates/mcb-infrastructure/src/services/highlight_service.rs:228`

Tests also rely heavily on Tokio (`#[tokio::test]`) across `crates/*/tests` and `tests/golden/*`.

### Key Tokio APIs in Real Use

#### Runtime bootstrap

- `#[tokio::main]` in `crates/mcb/src/main.rs:52`

Rationale: single runtime bootstrap at binary boundary, with async orchestration delegated to server/init modules.

#### Task orchestration

- `tokio::spawn(...)` for background or parallel flows
  - `crates/mcb-server/src/init.rs`
  - `crates/mcb-application/src/use_cases/indexing_service.rs:214`
  - `crates/mcb-validate/src/engines/hybrid_engine.rs`
- `tokio::join!(...)` for coordinated parallel await
   - `crates/mcb-server/src/init.rs`
   - `crates/mcb-application/src/use_cases/memory_service.rs:228`

#### Async synchronization primitives

- `tokio::sync::RwLock` in shared state components
  - `crates/mcb-infrastructure/src/health.rs`
  - `crates/mcb-infrastructure/src/config/watcher.rs`
  - `crates/mcb-providers/src/hybrid_search/engine.rs`
- `tokio::sync::Mutex` for async-safe lock boundaries
  - `crates/mcb-infrastructure/src/services/highlight_service.rs`
- `tokio::sync::broadcast` for event fan-out
   - `crates/mcb-providers/src/events/tokio.rs`

#### Async I/O and time

- `tokio::fs::*` for filesystem operations
  - `crates/mcb-infrastructure/src/utils/file.rs`
  - `crates/mcb-language-support/src/parser.rs:221`
  - `crates/mcb-application/src/use_cases/indexing_service.rs:288`
- `tokio::time::sleep(...)` in controlled retries/watch loops
   - `crates/mcb-infrastructure/src/config/watcher.rs`
   - `crates/mcb-providers/src/vector_store/milvus.rs`

#### Blocking boundary handling

- `tokio::task::spawn_blocking(...)`
  - Git traversal in `crates/mcb-providers/src/git/submodule.rs:48`
  - Syntax-heavy work in `crates/mcb-infrastructure/src/services/highlight_service.rs:228`
  - Language chunking in `crates/mcb-providers/src/language/engine.rs:130`

This is a critical clean-boundary behavior; see also `context/external/git2.md` and `context/external/tree-sitter.md`.

## Project-Specific Best Practices

### Keep hot paths non-blocking

Rule:

- No sync filesystem, no blocking library calls in request/task hot paths.
- Use `spawn_blocking` for unavoidable blocking work.

Evidence:

- `crates/mcb-providers/src/git/submodule.rs:41-48` explicitly documents this.

### Prefer explicit concurrency composition

Rule:

- Use `join!` only when both branches are expected and meaningful.
- Use `spawn` only when detached/asynchronous lifecycle is intended.

Evidence:

- `crates/mcb-application/src/use_cases/memory_service.rs:228` runs FTS + vector search in parallel with controlled fallback.

### Treat task lifecycle as part of API behavior

Rule:

- For detached tasks, make lifecycle visible through operation IDs, logs, status endpoints, or events.

Evidence:

- Background indexing in `crates/mcb-application/src/use_cases/indexing_service.rs` creates async work decoupled from caller completion.

### Use async-safe synchronization

Rule:

- Use Tokio primitives (`tokio::sync::*`) in async contexts.
- Avoid `std::sync::Mutex` in async execution paths.

Evidence:

- Production code uses Tokio sync primitives, and validation fixtures intentionally capture anti-pattern cases under `crates/mcb-validate/tests/fixtures`.

### Log with context around async boundaries

Rule:

- Every non-trivial spawn/retry/degradation path should emit structured logs.

Evidence:

- Extensive `tracing::*` integration across async flows (see `context/external/tracing.md`).

## Common Pitfalls

### Fire-and-forget orphan risk

Pattern:

- Detached spawned task with no external cancellation or ownership.

Observed in:

- `crates/mcb-application/src/use_cases/indexing_service.rs:214`

Risk:

- Task can outlive caller context; failures may become indirect.

Mitigation:

- Ensure task state is externally visible and errors are surfaced via events/logs.

### Broadcast lag and dropped messages

Pattern:

- Slow subscriber with `broadcast` channel.

Observed in:

- `crates/mcb-providers/src/events/tokio.rs` with lag handling.

Risk:

- Missed events for slow consumers.

Mitigation:

- Capacity tuning + explicit lag warnings + consumer resilience.

### Misplaced blocking work

Pattern:

- Sync-heavy operation on async worker thread.

Risk:

- Scheduler starvation and latency spikes.

Mitigation:

- Continue explicit `spawn_blocking` policy; test critical paths under load.

### Timeout without cancellation semantics

Pattern:

- Time-based wait used as supervisory mechanism.

Risk:

- Work may continue after timeout boundary from caller perspective.

Mitigation:

- Prefer `tokio::time::timeout` with explicit cancellation strategy where semantics require strict abort behavior.

### Runtime mixing or nested runtime pitfalls

Pattern:

- Constructing runtimes in already-async contexts.

Observed edge case:

- Validator fallback path uses runtime checks in `crates/mcb-validate/src/declarative_validator.rs`.

Risk:

- Complexity and subtle deadlock/panic behavior if misuse expands.

Mitigation:

- Keep this pattern constrained; avoid nested runtime creation in request flow.

### Do/Do-Not Guidance for Contributors

Do:

- Use Tokio primitives in async code.
- Offload blocking external library calls.
- Keep async execution ownership explicit.
- Add `#[tokio::test]` coverage for concurrency-sensitive changes.
- Ensure retries include bounded waits and logs.

Do not:

- Introduce blocking code in MCP handlers or server hot paths.
- Spawn detached tasks for core user-visible flows without status observability.
- Use sleep as synchronization substitute when deterministic signaling is available.
- Mix sync mutexes into async flow code.

### Verification Checklist (Tokio Changes)

When changing Tokio-related code in MCB, verify:

1. No new blocking calls were introduced in async hot paths.
2. `spawn_blocking` is used for non-async-safe libraries.
3. Spawned task ownership and lifecycle are intentional and visible.
4. Concurrency branches (`join!`/spawn) have explicit error handling strategy.
5. Broadcast/queue capacities are sufficient for expected load.
6. `#[tokio::test]` coverage exists for new concurrency behavior.
7. Tracing spans/logs include enough context for async diagnostics.

Suggested commands:

```bash
rg -n "tokio::spawn|spawn_blocking|tokio::join!|tokio::sync|tokio::time" crates
rg -n "std::sync::Mutex" crates
cargo test
```

## Context7 + External Research

- **Context7 ID**: `/websites/rs_tokio_tokio`
- **Official Docs**: https://docs.rs/tokio
- **Upstream Repository**: https://github.com/tokio-rs/tokio

## ADR Alignment (Critical)

This document aligns with the following Architecture Decision Records:

- **ADR-002**: Async runtime selection and Tokio as the standard runtime for MCB
- **ADR-035**: Concurrency patterns and task lifecycle management
- **ADR-021**: Blocking boundary handling and `spawn_blocking` policy

Key architectural decisions:
- Tokio is the sole async runtime for MCB (no runtime mixing).
- Domain crates define async contracts via `async-trait` without runtime coupling.
- Blocking third-party APIs are explicitly isolated with `spawn_blocking`.
- Async synchronization primitives (`tokio::sync::*`) are mandatory in async contexts.

## GitHub Evidence (Upstream + In-Repo)

**Upstream Repository**:
- https://github.com/tokio-rs/tokio (primary source)
- https://github.com/tokio-rs/tokio/tree/master/examples (reference implementations)

**In-Repository Anchors** (hotspots and evidence):
- `crates/mcb/src/main.rs:52` — Binary entrypoint with `#[tokio::main]`
- `crates/mcb-server/src/init.rs` — Server boot and transport orchestration
- `crates/mcb-application/src/use_cases/indexing_service.rs:214` — Background task spawn
- `crates/mcb-application/src/use_cases/memory_service.rs:228` — Hybrid concurrency (FTS + vector search)
- `crates/mcb-providers/src/git/submodule.rs:48` — Blocking boundary for Git traversal
- `crates/mcb-providers/src/events/tokio.rs` — Event bus with broadcast channels
- `crates/mcb-infrastructure/src/config/watcher.rs` — Config watcher and shared state
- `crates/mcb-infrastructure/src/services/highlight_service.rs:228` — Syntax-heavy blocking work
- `crates/mcb-providers/src/language/engine.rs:130` — Language chunking with blocking boundary

**Related External Documentation**:
- For logging and span discipline: `context/external/tracing.md`
- For blocking Git adapter boundaries: `context/external/git2.md`
- For transport request lifecycle: `context/external/rocket.md`
- For persistence in async handlers: `context/external/sqlx.md`
- For protocol handler orchestration: `context/external/rmcp.md`

## References

**Official Documentation**:
- https://docs.rs/tokio
- https://github.com/tokio-rs/tokio
- https://tokio.rs/tokio/topics/tracing

**Implementation Examples**:
- https://github.com/tokio-rs/tokio/tree/master/examples
- https://github.com/hyperium/hyper/tree/master/examples

**Cross-Document Map (Avoid Duplication)**:

- For logging and span discipline: `context/external/tracing.md`
- For blocking Git adapter boundaries: `context/external/git2.md`
- For transport request lifecycle and server boundaries: `context/external/rocket.md`
- For persistence behavior inside async handlers: `context/external/sqlx.md`
- For protocol handler orchestration: `context/external/rmcp.md`
