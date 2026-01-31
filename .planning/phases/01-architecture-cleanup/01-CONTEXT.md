# Phase 1: Architecture Cleanup - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Break dependency cycles and create shared infrastructure crates. Specifically:
- Break mcb-infrastructure → mcb-validate dependency cycle
- Create mcb-language-support crate (unified language abstraction)
- Create mcb-ast-utils crate (AST traversal and analysis)
- Define ValidationServiceInterface port
- Define MetricsProvider port

This phase reorganizes internal architecture. No new user-facing features.

</domain>

<decisions>
## Implementation Decisions

### Crate Boundaries

**mcb-language-support:**
- Full language abstraction: detection, parsing, and language-specific chunking strategies
- Built on Mozilla RCA fork as direct Cargo dependency
- Full RCA suite: detection, parsing, metrics, complexity analysis, code smells
- RCA always compiled (not optional feature)
- Full abstraction over RCA — no RCA types leak to consumers
- Async-first APIs (aligned with Tokio runtime)
- Semi-open visibility: core types public, implementation helpers private
- Extension points via traits for languages RCA doesn't support

**mcb-ast-utils:**
- Traversal + analysis: tree walking, node visiting, cursor utilities, complexity metrics, symbol extraction
- Depends on mcb-language-support (natural dependency for language context)

**mcb-validate:**
- Uses mcb-language-support for common language operations
- Adds validation-specific extensions on top of shared core
- Component interno do MCB (não é dev-only separado)

### Port Trait Design

**ValidationServiceInterface:**
- Streaming validation: Iterator/Stream of violations as found (memory efficient)
- Location: Follow Clean Architecture and ADR patterns (ports in mcb-domain)
- Error type: Shared DomainError
- Optional default implementations where sensible
- Type design and thread safety bounds: Follow project patterns and ADRs

**MetricsProvider:**
- Both levels: generic primitives (increment, gauge, histogram) + domain convenience methods (index_time, search_latency)
- Error type: Shared DomainError
- Optional default implementations

### Claude's Discretion
- Labels/tags support for MetricsProvider (based on Prometheus/OpenTelemetry patterns)
- Associated types vs generics (evaluate based on project scope, ADRs, libraries)
- Send + Sync bounds (apply what fits Clean Architecture and existing patterns)

### Migration Strategy

- Big bang migration: move all code at once, fix imports in single commit
- Git history: mv files to .bak for local filesystem check, use `git mv` when moving between locations
- Breaking changes: just change internal APIs freely
- Tests: move to new crate location inside test directory

### MCB Validate as Internal Component

- Standard feature of MCB (not dev-only separated)
- Offers MCP and CLI commands to validate code repositories in general
- Custom configuration and rules via existing YAML rules system
- Generalizable for use in other projects
- Auto-validation triggers: build (dev), commit hook, and manual `make validate`

</decisions>

<specifics>
## Specific Ideas

- "Use unified language base within Clean Architecture that maximizes Mozilla RCA fork as library base"
- Validation functionality should be a standard MCB feature usable in dev for auto-validation
- Rules system already exists in YAML — reuse same pattern

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-architecture-cleanup*
*Context gathered: 2026-01-31*
