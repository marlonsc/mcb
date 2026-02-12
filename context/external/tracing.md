# tracing in MCB: Structured Telemetry, Runtime Diagnostics, and Logging Safety

Last updated: 2026-02-12  
Scope: tracing usage patterns in MCB, initialization strategy, instrumentation quality, and risk controls.  
Cross-reference: `context/external/tokio.md`, `context/external/rmcp.md`, `context/external/rocket.md`, `context/external/thiserror.md`.

---

## 1. Executive Summary

Tracing is a first-class operational dependency in MCB, used across server, infrastructure, providers, and validation paths.

Current strengths:

- widespread structured event usage
- explicit handler instrumentation in multiple server paths
- centralized logging initialization in infrastructure

Key risks to manage:

- cardinality/noise in hot paths
- sensitive field leakage
- initialization/filter misconfiguration at runtime

---

## 2. Primary Internal Anchors

Initialization and subscriber configuration:

- `crates/mcb-infrastructure/src/logging.rs`

High-traffic server and admin paths:

- `crates/mcb-server/src/handlers/*`
- `crates/mcb-server/src/admin/*`
- `crates/mcb-server/src/transport/http.rs`

Provider/infrastructure examples:

- `crates/mcb-providers/src/git/submodule.rs`
- `crates/mcb-providers/src/database/sqlite/provider.rs`
- `crates/mcb-application/src/use_cases/indexing_service.rs`

The platform/integration exploration also flagged broad `tracing` usage (including `#[instrument]` in handler flows), which aligns with observability expectations for this architecture.

---

## 3. Core Tracing Patterns in MCB

### 3.1 Event-level structured logging

Common macros in heavy use:

- `tracing::error!`
- `tracing::warn!`
- `tracing::info!`
- `tracing::debug!`

Preferred style:

- structured key-value fields over opaque formatted strings

### 3.2 Span instrumentation on handler entrypoints

Server handlers use `#[tracing::instrument(skip_all)]` (or similar variants) in multiple modules.

Operational value:

- request-level execution context
- consistent span boundaries for tool handling

### 3.3 Environment filter configuration

`tracing-subscriber` with env-driven filtering is used to control verbosity by deployment context.

---

## 4. Architecture Fit

Tracing in MCB is infrastructure-aligned, not business-logic-owned.

Expected layering behavior:

- domain describes errors/contracts
- transport/infrastructure emits diagnostics context
- providers emit external-system operation details

This is consistent with clean architecture separation and error mapping patterns.

---

## 5. Project-Specific Best Practices

### 5.1 Log meaningful context, not just text

Required fields should include operation-specific identifiers where available (session/tool/resource/action identifiers).

### 5.2 Keep instrumentation selective in hot paths

Use `skip_all` or explicit `skip(...)` where payload capture is expensive or sensitive.

### 5.3 Preserve failure context at mapping boundaries

When errors are mapped (e.g., server error mapping), log source/variant context so diagnosis remains possible.

### 5.4 Keep log volume bounded

Tight loops/retry loops should avoid high-cardinality unbounded logs.

### 5.5 Align tracing with async lifecycle semantics

Spawned tasks and background operations should preserve enough context to correlate with origin operations.

---

## 6. Failure Modes and Fragile Areas

### 6.1 Invalid filter configuration behavior

Risk:

- invalid env filter values can degrade logging behavior at startup/runtime.

Mitigation:

- validate configuration defaults and fail clearly when filter parsing is invalid.

### 6.2 Sensitive data leakage

Risk:

- debug/info events accidentally include secrets or user-sensitive payload fields.

Mitigation:

- enforce skip/redaction patterns in instrumented functions and helper utilities.

### 6.3 Signal-to-noise degradation

Risk:

- overlogging in operational loops obscures actionable incidents.

Mitigation:

- cap repetitive logs, use appropriate levels, and consolidate repeated warnings.

### 6.4 Incomplete span propagation in async boundaries

Risk:

- detached tasks lose request/tool context, weakening incident traceability.

Mitigation:

- ensure spawned tasks carry explicit context fields or inherited span linkage where needed.

---

## 7. Contributor Guidance

Do:

- prefer structured key-value logs
- instrument boundaries (handlers, adapters, long-running tasks)
- keep log levels meaningful and consistent
- preserve error-source information at conversion boundaries
- review logs for confidentiality before merging

Do not:

- log raw sensitive config/secrets/tokens
- add high-frequency info/debug logs in tight loops without controls
- rely only on message text where structured fields should exist

---

## 8. Verification Checklist

When changing tracing behavior:

1. Verify log fields are structured and relevant.
2. Verify sensitive inputs are skipped/redacted.
3. Verify instrumentation remains present on new high-value boundaries.
4. Verify verbosity is acceptable in normal and failure scenarios.
5. Verify async task logs still correlate to initiating context.

Suggested commands:

```bash
rg -n "tracing::|#\[tracing::instrument" crates
cargo test
```

---

## 9. Cross-Document Map

- Async/task lifecycle and spawn semantics: `context/external/tokio.md`
- MCP handler and tool protocol context: `context/external/rmcp.md`
- Transport/web boundary behavior: `context/external/rocket.md`
- Typed error taxonomy feeding logs: `context/external/thiserror.md`

---

## 10. References

Official:

- https://docs.rs/tracing
- https://docs.rs/tracing-subscriber
- https://tokio.rs/tokio/topics/tracing

Repository anchors:

- `crates/mcb-infrastructure/src/logging.rs`
- `crates/mcb-server/src/error_mapping.rs`
- `crates/mcb-server/src/handlers/`
- `crates/mcb-server/src/admin/`

External examples:

- https://github.com/availproject/avail-light/blob/main/fat/src/main.rs
- https://github.com/netdata/netdata/blob/master/src/crates/netdata-log-viewer/journal-viewer-plugin/src/main.rs
