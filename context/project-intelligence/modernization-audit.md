# Modernization Audit

Last updated: 2026-02-12
Scope: `crates/`, `docs/adr/`, `scripts/`

## Purpose

This audit consolidates internal modernization findings for legacy/incomplete code, duplication hotspots, migration leftovers, and architecture inconsistencies.

The goal is to produce a prioritized backlog that reduces maintenance cost without destabilizing runtime behavior.

## Method

- Internal scans: `rg`, `ast-grep`, targeted file reads.
- External references: official docs and ecosystem tooling research.
- Prioritization model: production risk first, then maintainability burden.
- Validation principle: no claim without concrete file evidence.

## Critical Findings (P0)

### 1) Multi-tenant context fallback in admin browse handlers

- `crates/mcb-server/src/admin/handlers.rs:180`
- `crates/mcb-server/src/admin/handlers.rs:253`
- `crates/mcb-server/src/admin/handlers.rs:298`
- `crates/mcb-server/src/admin/handlers.rs:336`

Current behavior uses `OrgContext::default()` with TODO markers for auth-derived org extraction. This creates cross-tenant correctness risk and blocks full multi-tenant behavior.

Modernization target:

- Extract org context from authenticated admin identity in these handlers.
- Fail closed when org cannot be resolved (no implicit default in protected paths).

## High Findings (P1)

### 2) Production dead-code suppression in providers

- `crates/mcb-providers/src/database/sqlite/mod.rs:11`

`#[allow(dead_code)]` is used in production module wiring. This suppresses useful compiler signals.

Modernization target:

- Remove suppression and either use or delete dead paths.
- If temporary, isolate behind feature flags with explicit sunset date.

### 3) ADR metadata inconsistency (implemented vs incomplete)

- `docs/adr/030-multi-provider-strategy.md:4`
- `docs/adr/030-multi-provider-strategy.md:10`

The ADR is marked `status: IMPLEMENTED` while also declaring `implementation_status: Incomplete`. This weakens architecture governance and creates planning ambiguity.

Modernization target:

- Harmonize status fields and add objective completion criteria.
- Apply same metadata review to all ADRs marked incomplete.

### 4) Superseded ADR lineage still creates decision noise

- `docs/adr/012-di-strategy-two-layer-approach.md:9`
- `docs/adr/024-simplified-dependency-injection.md:9`
- `docs/adr/032-agent-quality-domain-extension.md:9`

Supersession is documented but still creates confusion in active planning when historical ADRs are mixed with current policy docs.

Modernization target:

- Keep archived ADRs for history, but add stronger "active replacement" headers and link to active ADRs at top.

## Medium Findings (P2)

### 5) Duplicated resolver/mocks across handlers

- `crates/mcb-server/src/handlers/vcs_entity.rs:239`
- `crates/mcb-server/src/handlers/project.rs:88`
- `crates/mcb-server/src/handlers/plan_entity.rs:176`
- `crates/mcb-server/src/handlers/issue_entity.rs:228`
- `crates/mcb-server/src/handlers/org_entity.rs:30`

Multiple handlers duplicate test resolver setup and mock service patterns. This increases churn cost and weakens consistency.

Modernization target:

- Consolidate shared test resolver builders in `crates/mcb-server/tests/test_utils/`.
- Keep handler tests behavior-focused, not wiring-focused.

### 6) Commented-out tests without explicit retirement policy

- `crates/mcb-server/tests/integration.rs:54`
- `crates/mcb-server/tests/integration.rs:83`
- `crates/mcb-server/tests/unit.rs:24`
- `crates/mcb-server/tests/unit.rs:33`
- `crates/mcb-server/tests/unit.rs:38`

Disabled modules are tracked in comments, but there is no enforced restore/remove deadline.

Modernization target:

- Convert commented modules into tracked issues with due criteria.
- Either restore with passing coverage or delete with rationale.

### 7) Handle extension trait duplication

- `crates/mcb-infrastructure/src/di/handles.rs:63`
- `crates/mcb-infrastructure/src/di/handles.rs:85`

`EmbeddingHandleExt` and `CacheHandleExt` implement near-identical logic for provider name extraction.

Modernization target:

- Replace repetitive extension traits with one generic trait or utility helper.

## Lower Findings (P3)

### 8) Historical comments and allowances in tests

- `crates/mcb-server/tests/test_utils/mock_services/mod.rs:16`
- `crates/mcb-server/tests/test_utils/mock_services/mod.rs:21`
- `crates/mcb-server/tests/test_utils/mock_services/mod.rs:23`

Test-only allowances are less risky but should still be trimmed to keep signal quality high.

Modernization target:

- Remove unnecessary `allow` attributes and stale notes after test utility cleanup.

## Important Validation Notes

### Not a production panic finding

- `crates/mcb-validate/src/implementation/validator.rs:692`
- `crates/mcb-validate/src/implementation/validator.rs:695`

`todo!()` and `unimplemented!()` here occur inside validator test fixture strings in test code, not active production runtime paths.

Action: keep as fixture data; do not prioritize as production panic remediation.

## Architecture Duplication Hotspots (Large Refactor Candidates)

The following clusters show high structural duplication and should be modernized incrementally behind tests:

- Entity handlers:
  - `crates/mcb-server/src/handlers/issue_entity.rs`
  - `crates/mcb-server/src/handlers/org_entity.rs`
  - `crates/mcb-server/src/handlers/plan_entity.rs`
  - `crates/mcb-server/src/handlers/vcs_entity.rs`
- Memory/session action routing families:
  - `crates/mcb-server/src/handlers/memory/`
  - `crates/mcb-server/src/handlers/session/`
- Provider families with repeated HTTP/request flow patterns:
  - `crates/mcb-providers/src/embedding/`
  - `crates/mcb-providers/src/vector_store/`

Modernization strategy:

1. Extract shared dispatch/template logic into internal generic helpers.
2. Preserve public behavior and schema contracts.
3. Migrate one family at a time with golden tests as regression guard.

## Additional Duplication Intake (bg_e83b2ab3)

### CRUD adapter concentration

- `crates/mcb-server/src/admin/crud_adapter.rs:26`
- `crates/mcb-server/src/admin/crud_adapter.rs:55`
- `crates/mcb-server/src/admin/crud_adapter.rs:246`

This module centralizes a large set of entity adapters with repeated patterns (`list_all/get_by_id/create/update/delete`). It is functional but high-churn and difficult to evolve safely.

Modernization target:

- Keep behavior and slug contract stable.
- Incrementally extract common adapter plumbing and JSON conversion helpers into reusable generic primitives.

### Search abstraction overlap

- `crates/mcb-application/src/use_cases/search_service.rs:12`
- `crates/mcb-domain/src/repositories/search_repository.rs:35`

`SearchServiceImpl` delegates search to context service while a separate domain search repository abstraction exists for semantic/hybrid operations. This overlap should be reviewed for boundary clarity.

Modernization target:

- Define one canonical path for runtime search orchestration.
- Keep other interfaces only when they serve a distinct boundary (for example, provider-level retrieval vs application-level filtering).

### Parallel indexing orchestration

- `crates/mcb-application/src/use_cases/indexing_service.rs`
- `crates/mcb-application/src/use_cases/vcs_indexing.rs:79`

There are two orchestration paths for indexing (generic and VCS-aware). This may be intentional, but currently increases cognitive load for contributors.

Modernization target:

- Document explicit responsibility split and decision criteria.
- If overlap remains high after clarification, merge orchestration into a single configurable flow.

## External Tooling Recommendations (Adopt Carefully)

Prioritized tools for audit and cleanup support:

- `cargo-udeps` (authoritative but nightly) for unused dependency verification.
- `cargo-machete` (fast heuristic) for CI trend checks, not auto-removal.
- `cargo-modules` for module graph visibility in overengineered areas.

Do not auto-adopt broad runtime/framework swaps (for example async runtime replacement) in this cycle. Focus on internal simplification first.

## Proposed Backlog (Issue Seeds)

1. P0: Replace admin browse `OrgContext::default()` with auth-derived org context.
2. P1: Remove production `#[allow(dead_code)]` in sqlite provider module.
3. P1: Reconcile ADR-030 status fields and define completion criteria.
4. P1: Add explicit active-replacement banners to superseded ADRs.
5. P2: Consolidate duplicated handler test resolver/mocks into shared test utilities.
6. P2: Resolve commented-out test modules via restore-or-remove policy.
7. P2: Consolidate duplicate provider-handle extension traits.

## Exit Criteria for This Audit

- Every P0/P1 item converted into tracked issues.
- No unresolved status conflicts in active ADR metadata.
- First duplication family migrated with tests unchanged in behavior.
- Context docs remain in English and pass validation checks.
