# RMCP in MCB: Professional, Complete, and Critical Analysis

Last updated: 2026-02-12
Scope: protocol layer, architecture fit, operational risk, and implementation guidance for `rmcp` usage in MCB.
Research basis: Context7, official docs, ADR corpus, and actual source usage.

---

## 1. Executive Summary

MCB uses RMCP as the protocol boundary for MCP tool exposure.
The implementation is architecturally sound in its core shape: protocol logic remains in `mcb-server`,
while business behavior is delegated to handler and service layers.

Key strengths:
- Typed protocol contracts and explicit capability declaration.
- Clear dispatch path from `ServerHandler` to handler routers.
- Good alignment with clean architecture and DI decisions in adjacent ADRs.

Key constraints:
- Router complexity rises as tool surface grows.
- Transport parity (stdio vs HTTP streamability) remains a strategic concern.
- Consistency of schema + runtime behavior requires strict governance.

Strategic recommendation:
- Keep RMCP boundary thin and deterministic.
- Formalize a protocol contract test suite per tool category.
- Treat ADR-033 consolidation principles as ongoing governance, not one-time migration.

---

## 2. Methodology and Evidence Sources

This document is based on four evidence channels:

1. Context7 library research
2. Official RMCP API references
3. MCB ADR corpus
4. Actual source code in current repository

Evidence quality policy:
- Prefer first-party docs and source repositories.
- Cross-check design intent (ADR) against implementation reality.
- Flag divergence explicitly.
- Separate observed facts from recommendations.

---

## 3. Context7 Findings

Resolved RMCP entries from Context7 during this session:
- `/websites/rs_rmcp_rmcp` (official Rust SDK profile)
- `/websites/rs-rmcp` (alternative profile entry)

Selected baseline:
- `/websites/rs_rmcp_rmcp` due to alignment with official SDK positioning and API shape.

Context7-backed capability themes:
- `ServerHandler` + tool registration patterns
- structured outputs
- task lifecycle support for long-running operations
- transport flexibility (stdio and streamable HTTP options)
- feature-flag-driven composition

Critical note:
- Context7 supports discovery and orientation but does not replace in-repo architectural truth.
- Final decisions must be anchored to MCB source and ADRs.

---

## 4. Official RMCP Surface (Condensed)

From official docs and examples, RMCP provides:

- Server building primitives (`ServerHandler`, service extensions)
- Tool macros (`#[tool]`, `#[tool_router]`, `#[tool_handler]`) where desired
- Typed protocol models (`CallToolRequest`, `CallToolResult`, capability structures)
- Error model (`ErrorData`) compatible with JSON-RPC response semantics
- Optional task system for asynchronous/lifecycle-aware execution
- Multiple transport strategies (stdio and streamable HTTP variants)

Reference:
- https://docs.rs/rmcp
- https://github.com/modelcontextprotocol/rust-sdk

---

## 5. Actual MCB Usage: Ground Truth

Primary code anchors:
- `crates/mcb-server/src/mcp_server.rs`
- `crates/mcb-server/src/tools/router.rs`
- `crates/mcb-server/src/transport/http.rs`

Observed architecture flow:
1. RMCP entrypoint via `impl ServerHandler for McpServer`
2. Tool metadata exposed via `list_tools()`
3. Tool calls routed through `route_tool_call()`
4. Handler layer parses typed args and delegates to domain/application services
5. Post-tool hooks execute memory-related follow-up in non-fatal mode

Critical architectural property:
- Protocol boundary remains in server crate and does not contaminate domain ports.

Implementation references:
- `crates/mcb-server/src/mcp_server.rs:262` (ServerHandler impl)
- `crates/mcb-server/src/mcp_server.rs:296` (`list_tools`)
- `crates/mcb-server/src/mcp_server.rs:310` (`call_tool`)
- `crates/mcb-server/src/tools/router.rs:58` (route entry)
- `crates/mcb-server/src/tools/router.rs:64` (dispatch match)
- `crates/mcb-server/src/tools/router.rs:139` (argument parse)

---

## 6. ADR Crosswalk and Consistency Review

### ADR-033: MCP Handler Consolidation

Assessment: Supports centralized and parameterized handler patterns; directly relevant to router governance.

Alignment level:
- Intent: aligned
- Current implementation: mostly aligned
- Governance maturity: moderate

### ADR-011: HTTP Transport Pattern

Assessment: Documents transport behavior tradeoffs and deferred streaming complexity.

Alignment level:
- Intent: aligned
- Current implementation: mostly aligned
- Governance maturity: moderate

### ADR-037: Workflow Orchestrator

Assessment: Confirms MCP as orchestration interface and emphasizes coherent tool semantics.

Alignment level:
- Intent: aligned
- Current implementation: mostly aligned
- Governance maturity: moderate

### ADR-046: Integration Policies (034-037)

Assessment: Reinforces unified MCP interface and integration discipline.

Alignment level:
- Intent: aligned
- Current implementation: mostly aligned
- Governance maturity: moderate

### ADR-013: Clean Architecture Separation

Assessment: Supports keeping protocol concerns in server layer only.

Alignment level:
- Intent: aligned
- Current implementation: mostly aligned
- Governance maturity: moderate

Cross-ADR conclusion:
- RMCP usage in MCB is directionally consistent with architectural decisions.
- Main residual risk is operational consistency under expanding tool surface.

---

## 7. Tool Surface Analysis (Deep Dive)

MCB currently exposes a consolidated but broad tool set through RMCP.
Below is a critical review per tool family.

### 7.1 Tool: `index`

Functional scope: Index lifecycle operations

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `index`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- Strongly operational; needs idempotency guarantees and clear status states.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.2 Tool: `search`

Functional scope: Code and memory retrieval

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `search`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- High user visibility; schema consistency and ranking transparency are critical.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.3 Tool: `validate`

Functional scope: Architecture and code validation

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `validate`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- Can become policy-heavy; should separate advisory vs blocking findings.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.4 Tool: `memory`

Functional scope: Long-term memory operations

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `memory`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- Most sensitive data path; requires strict scoping and schema rigor.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.5 Tool: `session`

Functional scope: Agent session lifecycle

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `session`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- State model must remain deterministic under concurrency.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.6 Tool: `agent`

Functional scope: Agent activity logging

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `agent`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- Observability-critical; can produce high write volume and noisy telemetry.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.7 Tool: `project`

Functional scope: Project workflow resources

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `project`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- Business workflow semantics; demands stable action/resource contracts.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.8 Tool: `vcs`

Functional scope: Repository operations

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `vcs`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- Potentially expensive and safety-sensitive; bounded scope required.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.9 Tool: `vcs_entity`

Functional scope: VCS entities CRUD

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `vcs_entity`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- Entity consistency and referential integrity are central risks.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.10 Tool: `plan_entity`

Functional scope: Plan/version/review CRUD

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `plan_entity`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- Schema drift risk across versions and review lifecycle.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.11 Tool: `issue_entity`

Functional scope: Issue/comment/label CRUD

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `issue_entity`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- High-volume workflow object; pagination and filtering behavior must be stable.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

### 7.12 Tool: `org_entity`

Functional scope: Org/user/team/api key CRUD

Current router behavior:
- Dispatched by explicit match arm in `route_tool_call` for `org_entity`.
- Arguments parsed from JSON object into typed args struct.
- Handler call is awaited and converted into protocol result.
- Post-tool hook path executes after success, failures are logged as non-fatal.

Protocol contract expectations:
- Deterministic argument schema
- Stable error code mapping
- Content payload shape compatibility across releases
- Backward-safe optional field evolution

Critical analysis:
- Security and authorization constraints are primary concern.
- Ensure semantic validation goes beyond JSON shape validation.
- Avoid adding implicit behavior that is not reflected in schema docs.

Recommended hardening actions:
1. Add contract tests for valid/invalid argument permutations.
2. Add snapshot tests for successful result envelopes.
3. Add golden tests for error mapping and failure modes.
4. Document latency expectations and timeout behavior.

Suggested observability fields:
- tool_name
- action/resource (if applicable)
- result_status
- elapsed_ms
- correlation/session identifiers

---

## 8. Router and Dispatch Critical Review

Strengths:
- Explicit match-based dispatch is auditable and straightforward.
- Typed argument parsing centralization lowers duplicated parse logic.
- Non-fatal post-hook behavior avoids cascading failures.

Weak points:
- Match-arm growth can become a hotspot for merge conflicts and drift.
- Central parser path can hide tool-specific semantic preconditions.
- Non-fatal hooks may mask data quality regressions if not monitored.

Recommendations:
- Introduce a lightweight dispatch registration map with compile-time checks.
- Keep shared parser but enforce per-tool semantic validators.
- Add hook success-rate SLO and alerting.

---

## 9. Transport Layer Review (stdio + HTTP)

Observed HTTP behavior in current source:
- `/mcp` handles JSON-RPC methods including initialize, tools/list, tools/call, ping.
- Health probes are exposed via `/healthz` and `/readyz`.
- Admin and browse routes can be mounted in the same Rocket application.
- CORS support is attachable via fairing.

Critical transport notes:
- HTTP transport complexity is materially higher than stdio and needs dedicated test depth.
- Method-level consistency between stdio and HTTP must be continuously verified.
- Error code parity and payload parity are mandatory for client predictability.

ADR tie-in:
- ADR-011 already frames request-response and streaming tradeoffs.
- Operational documentation must reflect actual implemented behavior, not aspirational states.

---

## 10. Error Model and Contract Stability

Current pattern:
- RMCP errors are represented via `ErrorData` and mapped from internal failures.

Critical requirement:
- Error mapping must be deterministic across handlers and transports.

Recommended error policy:
- Invalid arguments -> invalid params category
- Unknown tool/method -> method not found category
- Internal service failure -> internal error category
- Authorization failures -> explicit auth-related category if applicable

Anti-patterns:
- Returning implementation detail strings as user-facing protocol messages.
- Inconsistent status envelope fields across tool families.

---

## 11. Security and Trust Boundary Analysis

Trust boundaries:
1. MCP client input
2. RMCP server protocol boundary
3. Handler/service execution
4. Data stores and external adapters

Security concerns by boundary:
- Input tampering and schema abuse
- Overly permissive argument handling
- High-cost operations without quotas
- Data exfiltration through broad query interfaces

Hardening recommendations:
- Add explicit per-tool authorization checks where needed
- Enforce maximum limits (`limit`, payload size, time budget)
- Use allow-list semantics for enum/action/resource fields
- Redact sensitive fields in logs

---

## 12. Performance and Scalability Considerations

Performance hotspots:
- Heavy search/validation tasks
- VCS analysis over large repositories
- Memory operations with large payloads
- HTTP serialization overhead under concurrent load

Recommended controls:
- Timeouts per tool category
- Pagination and bounded result sets
- Cancellable long-running operations
- Optional task queue usage for expensive workflows

Critical metric set:
- p50/p95 latency by tool
- error rate by tool and error class
- timeout/cancellation counts
- queue depth (if task system used)
- response payload size distribution

---

## 13. Testing Strategy (Protocol + Behavior)

Test layers:
- Unit tests for parser and router branch behavior
- Contract tests for schema and response envelope stability
- Integration tests for end-to-end tool invocation
- Transport parity tests (stdio vs HTTP)
- Failure-mode tests (invalid params, internal errors, unknown tools)

Minimum regression pack per tool:
- `index`: success case, invalid arguments case, internal failure case, envelope snapshot
- `search`: success case, invalid arguments case, internal failure case, envelope snapshot
- `validate`: success case, invalid arguments case, internal failure case, envelope snapshot
- `memory`: success case, invalid arguments case, internal failure case, envelope snapshot
- `session`: success case, invalid arguments case, internal failure case, envelope snapshot
- `agent`: success case, invalid arguments case, internal failure case, envelope snapshot
- `project`: success case, invalid arguments case, internal failure case, envelope snapshot
- `vcs`: success case, invalid arguments case, internal failure case, envelope snapshot
- `vcs_entity`: success case, invalid arguments case, internal failure case, envelope snapshot
- `plan_entity`: success case, invalid arguments case, internal failure case, envelope snapshot
- `issue_entity`: success case, invalid arguments case, internal failure case, envelope snapshot
- `org_entity`: success case, invalid arguments case, internal failure case, envelope snapshot

Recommended CI gates:
1. Contract snapshot diff gate
2. Error mapping conformance gate
3. Transport parity gate for shared methods
4. Backward-compatibility schema gate

---

## 14. Operational Runbook Guidance

When a tool fails unexpectedly:
1. Confirm tool exists in `list_tools` output.
2. Verify argument schema and request payload shape.
3. Check router branch and parsed type for that tool.
4. Inspect handler logs with correlation identifiers.
5. Validate downstream service availability and dependencies.
6. Confirm error mapping category is correct.

When latency spikes:
1. Isolate by tool and action/resource dimension.
2. Check payload size and result size trends.
3. Check backend dependency saturation (DB, VCS, network).
4. Apply short-term bounds (limits/timeouts) before scaling changes.

---

## 15. Governance Model for Tool Growth

Every new tool or action should pass a governance checklist:
- Is this a new tool or an action on an existing resource?
- Does it follow existing error and payload conventions?
- Is transport behavior consistent with existing contracts?
- Is observability instrumentation complete?
- Are ADR updates required?

Gate policy recommendation:
- No new MCP action merges without contract tests and ADR impact review.

---

## 16. Critical Gap Register

The following gaps should be treated as active quality debt until proven closed:

- G01: Router scalability under sustained tool expansion
- G02: Guaranteed parity between stdio and HTTP behavior
- G03: Formalized long-running task policy per tool family
- G04: Error taxonomy consistency across handlers
- G05: Schema-versioning policy for backward compatibility
- G06: Security policy for sensitive org/session/memory operations
- G07: Operational SLOs and alerting for hook failures
- G08: Comprehensive contract snapshots for all tools
- G09: Cross-ADR drift detection automation
- G10: Client compatibility matrix for transport capabilities

---

## 17. Recommended 30-60-90 Day Improvement Plan

### Day 0-30
- Build contract snapshots for all current tools.
- Add transport parity tests for initialize/list/call/ping.
- Standardize error mapping table and enforce in code review.

### Day 31-60
- Introduce semantic validators per tool argument model.
- Add per-tool latency/error dashboards.
- Add hook reliability SLO and alerting.

### Day 61-90
- Refactor router registration model for maintainability.
- Add schema-version compatibility tests for major tools.
- Publish operator-facing protocol stability guide.

---

## 18. Appendix A - Tool Contract Baseline Matrix

Legend: Y = required, O = optional

| Tool | Typed Args | Semantic Validation | Snapshot Test | Transport Parity | Error Map Policy |
|---|---|---|---|---|---|
| `index` | Y | Y | Y | Y | Y |
| `search` | Y | Y | Y | Y | Y |
| `validate` | Y | Y | Y | Y | Y |
| `memory` | Y | Y | Y | Y | Y |
| `session` | Y | Y | Y | Y | Y |
| `agent` | Y | Y | Y | Y | Y |
| `project` | Y | Y | Y | Y | Y |
| `vcs` | Y | Y | Y | Y | Y |
| `vcs_entity` | Y | Y | Y | Y | Y |
| `plan_entity` | Y | Y | Y | Y | Y |
| `issue_entity` | Y | Y | Y | Y | Y |
| `org_entity` | Y | Y | Y | Y | Y |

---

## 19. Appendix B - OSS RMCP Usage References

- https://github.com/modelcontextprotocol/rust-sdk
- https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples
- https://github.com/openai/codex/blob/main/codex-rs/exec-server/src/posix/mcp.rs
- https://github.com/microsoft/wassette/blob/main/src/server.rs
- https://github.com/mozilla-ai/encoderfile/blob/main/encoderfile/src/transport/mcp/mod.rs
- https://github.com/nushell/nushell/blob/main/crates/nu-mcp/src/server.rs
- https://github.com/0xPlaygrounds/rig/blob/main/rig/rig-core/examples/rmcp.rs

---

## 20. Appendix C - ADR Reference Index

- `docs/adr/033-mcp-handler-consolidation.md`
- `docs/adr/011-http-transport-request-response-pattern.md`
- `docs/adr/037-workflow-orchestrator.md`
- `docs/adr/046-integration-adr-034-037-policies.md`
- `docs/adr/013-clean-architecture-crate-separation.md`

---

## 21. Appendix D - Detailed Per-Tool Review Templates

### D.1 `index` Detailed Template

Purpose
- Index lifecycle operations

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- Strongly operational; needs idempotency guarantees and clear status states.

### D.2 `search` Detailed Template

Purpose
- Code and memory retrieval

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- High user visibility; schema consistency and ranking transparency are critical.

### D.3 `validate` Detailed Template

Purpose
- Architecture and code validation

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- Can become policy-heavy; should separate advisory vs blocking findings.

### D.4 `memory` Detailed Template

Purpose
- Long-term memory operations

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- Most sensitive data path; requires strict scoping and schema rigor.

### D.5 `session` Detailed Template

Purpose
- Agent session lifecycle

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- State model must remain deterministic under concurrency.

### D.6 `agent` Detailed Template

Purpose
- Agent activity logging

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- Observability-critical; can produce high write volume and noisy telemetry.

### D.7 `project` Detailed Template

Purpose
- Project workflow resources

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- Business workflow semantics; demands stable action/resource contracts.

### D.8 `vcs` Detailed Template

Purpose
- Repository operations

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- Potentially expensive and safety-sensitive; bounded scope required.

### D.9 `vcs_entity` Detailed Template

Purpose
- VCS entities CRUD

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- Entity consistency and referential integrity are central risks.

### D.10 `plan_entity` Detailed Template

Purpose
- Plan/version/review CRUD

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- Schema drift risk across versions and review lifecycle.

### D.11 `issue_entity` Detailed Template

Purpose
- Issue/comment/label CRUD

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- High-volume workflow object; pagination and filtering behavior must be stable.

### D.12 `org_entity` Detailed Template

Purpose
- Org/user/team/api key CRUD

Required input constraints
- Type-safe field validation
- Semantic precondition checks
- Bounded cardinality and payload size

Expected output invariants
- Deterministic envelope shape
- Stable error/success signaling
- Backward-safe optional additions

Failure modes
- Invalid params
- Downstream dependency failure
- Timeout/cancellation
- Partial data availability

Mitigations
- Contract tests
- Retry/circuit-breaker policy where applicable
- Structured diagnostics

Critical note
- Security and authorization constraints are primary concern.

---

## 22. Appendix E - Quality Rubric for RMCP Changes

- Q01: Protocol correctness
- Q02: Schema stability
- Q03: Error mapping quality
- Q04: Transport parity
- Q05: Observability completeness
- Q06: Security posture
- Q07: ADR consistency
- Q08: Operational readiness
- Q09: Backward compatibility
- Q10: Test coverage depth

Scoring model suggestion:
- 0 = missing
- 1 = partial
- 2 = complete
- Merge threshold recommendation: >= 16/20 for material protocol changes

---

## 23. Conclusion

RMCP usage in MCB is solid in foundational architecture and boundary placement.
The main strategic challenge is scale governance: as tool surface and workflow depth grow,
contract consistency, transport parity, and operational controls must be treated as first-class engineering work.

This document should be maintained as a living protocol governance reference and updated with each major MCP/ADR evolution.
