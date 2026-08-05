<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 56
title: Multi-Tenant Isolation, Weaviate Vector Store, OIDC Identity, and Integration Boundary (v0.4.0)
status: PROPOSED
created: 2026-06-13
updated: 2026-06-13
related: [3, 34, 35, 36, 37, 38, 47, 48, 55]
supersedes: []
superseded_by: []
implementation_status: Proposed
---

# ADR 056: Multi-Tenant Isolation, Weaviate Vector Store, OIDC Identity, and Integration Boundary (v0.4.0)

## Status

**Proposed** (target v0.4.0)

> Scopes mcb to its mandated role in the cosmos MSP control plane — agent memory,
> semantic code context, and session metadata — under per-tenant isolation, OIDC
> identity, and a single platform vector store (Weaviate), and declares what mcb
> explicitly does **not** own.

## Context

cosmos-main `ADR-100` (APROVADO) assigns mcb a single, bounded capability (line 77):
**"memória de agentes, contexto semântico de código e metadados de sessão"**, and explicitly
**not** wiki/catalog content. cosmos `ADR-117` (PROPOSTO) adds two operator decisions that this
ADR implements on the mcb side: (a) **Weaviate is the single platform vector store** (RAG +
code index), retiring Milvus; (b) **native Keycloak OIDC** identity. Operating mcb as a shared
service across tenants requires per-organization isolation and Keycloak-issued identity, neither
of which is enforced today.

Verified current state (mcb v0.3.2):

- An `OrgContext` value object already exists
  (`crates/mcb-domain/src/value_objects/org_context.rs`): `{ org_id: OrgId, org_name }`.
- **Memory storage is already org-scoped (verified 2026-06-13).** `Observation` has `org_id` and
  `crates/mcb-providers/src/database/sqlite/memory_repository.rs` enforces `org_id` on every read
  (`get_observation`/`find_by_hash`/`search` FTS join/`get_observations_by_ids`/`get_timeline`/
  `get_session_summary`) and on INSERT — **no cross-tenant memory leak**. (An earlier review claimed
  `Observation` lacked `org_id`; that was inaccurate and is corrected here.)
- **The real isolation gap is identity binding:** `org_id` is **caller-asserted** via
  `resolve_org_id(args.org_id)` (e.g. `handlers/memory/observation.rs`) with a default fallback, not
  derived from an authenticated principal. No JWT/OIDC validation exists today; under untrusted
  multi-tenant operation a client could assert another org's id.
- `SearchServiceInterface` / `ContextService` / the vector-store providers carry no tenant context;
  the code index is keyed by collection name only (e.g. Pinecone `namespace=collection`), without org
  namespacing.
- Vector providers present: EdgeVec, Encrypted, Milvus, Pinecone, Qdrant — **no Weaviate provider**.
  Default (`config/deploy.toml`) is Milvus.
- `AgentSession` implements ~2 of the ~10 fields of cosmos `agent-session.schema.json`. The 12-state
  workflow FSM is `WorkflowState` (ADR-034), **distinct** from the 3-state `AgentSessionStatus`.

This ADR fixes the isolation/identity gaps at the root (no-bypass mandate), adopts Weaviate as the
mcb vector backend with native multi-tenancy, and pins the integration boundary.

## Decision

1. **Weaviate provider (single platform vector store).** Add
   `crates/mcb-providers/src/vector_store/weaviate.rs` (single file, mirroring `pinecone.rs` ~460 LOC;
   split if the 200-LOC cap requires) implementing the `VectorStoreProvider`/`VectorStoreAdmin`/
   `VectorStoreBrowser` traits from `crates/mcb-domain/src/ports/providers/vector_store/`, via REST
   (`reqwest`; no official Rust SDK); register with `register_vector_store_provider!`
   (`vector_store/macros.rs`) + `pub mod weaviate;` in `vector_store/mod.rs`; set
   `config/deploy.toml` `provider = "weaviate"` pointing at the cosmos Weaviate (HTTP :80). Other
   providers remain; Milvus is retired at the platform level (cosmos ADR-117). No new crate dep
   (reqwest + serde_json already present).

2. **Authenticated tenant principal = reuse `OrgContext` (no parallel type).** Today `org_id` is
   caller-asserted via `resolve_org_id(args.org_id)` (`crate::utils::mcp`) with a default fallback —
   the real isolation gap. Resolve `OrgContext` instead from credentials: ApiKey (`key_hash` match →
   `org_id`, reject `revoked_at`/expired) or **Keycloak Bearer JWT (validate signature/issuer, extract
   `org` claim, reject if absent)** — returning `r[OrgContext]`. A `RequestPrincipal { org: OrgContext,
   user_id, scopes }` composes `OrgContext` with the authenticated subject and parsed scopes. Handlers
   take `org_id` from the `RequestPrincipal`, **not** from `args.org_id`.

3. **Memory isolation: already enforced — add regression coverage.** `Observation.org_id` and
   `memory_repository.rs` already scope every read/write by org (verified). No schema/migration change
   needed. Work here = (a) cross-tenant negative tests as a CI gate; (b) source `org_id` from the
   `RequestPrincipal` (Decision 2) rather than `args`.

4. **Tenant threading + Weaviate native multi-tenancy.** Thread `org_id` end-to-end through
   `SearchServiceInterface`, `SearchFilters`, `ContextService` store/search, and the
   `EmbeddingProvider` call-chain. Because `org_id` is already threaded here, the **Weaviate provider
   uses Weaviate's native multi-tenancy (one tenant per `org`, physical shard isolation)** rather than
   a metadata filter; legacy providers map `org_id` → collection namespacing `org:{org_id}:{name}`.
   Handlers derive `org_id` from the `RequestPrincipal`, never from caller-supplied args.

5. **Scope enforcement.** Parse `ApiKey.scopes_json` and enforce required scopes before tool dispatch
   (deny-wins). An `AuditEvent { org_id, user_id, action, resource, timestamp }` entity records access
   decisions (emitted via OTEL; no ClickHouse driver here).

6. **`AgentSession` conforms to `agent-session.schema.json`.** Add `kind`, `work_order_ref`, `owner`,
   `workspace { runtime, repo, mode, ide }`, `identity { sso_subject, vault_role, teleport_user? }`,
   `scope { tenant_key, service, paths }`, `expires_at` (+TTL), `allowed_tools`, with migration.
   `AgentSessionStatus` stays 3-state; the 12-state FSM remains `WorkflowState` (ADR-034).

7. **Evidence by emission, not persistence.** Instrument session lifecycle and tool calls with OTEL
   spans (ADR-048) shaped to `EvidenceBundle` / `ToolCallEvent`. mcb does **not** write a ClickHouse
   driver and does **not** implement the `VerifierRun` / `PhaseGate` engine — cosmos Fase-3 work, gated
   by WorkOrder `adr100-p3`. Schema-only entities optional.

8. **Webhook ingress for agent lifecycle.** A signed/OIDC `POST /webhook` endpoint parses a
   `WebhookEvent`, creates an `AgentSession`, populates `identity`/`workspace` from the payload (so mcb
   is not coupled to the Coder API), idempotent via a dedup key.

9. **Integration boundary — mcb does NOT own:** the MCP gateway / multi-tenant fan-out (OSS gateway —
   ContextForge; mcb is one upstream MCP server); identity issuance (Keycloak is the SSOT; mcb only
   validates tokens); client knowledge RAG / connectors / ACL ingestion (PipesHub, ADR-100 l.69);
   reverse-ETL / outbound write-back. A graph layer (e.g. cognee) may later sit as an **overlay** over
   mcb episodic storage, not a replacement (mcb remains the memory SSOT per ADR-100 l.77).

## Consequences

- Closes an active cross-tenant data-leak path; isolation becomes a CI-enforced invariant, backed by
  Weaviate's physical shard-per-tenant model.
- Consolidates on a single platform vector store (Weaviate), enabling Milvus retirement.
- **Net code is NOT negative this cycle**: the Weaviate provider adds ~800 LOC (REST, no official Rust
  SDK). This is an authorized consequence of the operator's "single vector store = Weaviate" decision
  (cosmos ADR-117), recorded here rather than hidden.
- Code-index on Weaviate is comparatively unproven (the code-search ecosystem favors Qdrant) — validate
  with benchmarks before retiring Milvus.
- mcb remains deployable behind an OSS MCP gateway and Keycloak, emitting ADR-100-shaped
  session/evidence signals.
- Cost: `org_id` migration across `observations` (+ chunk metadata) with a backfill to the bootstrap org.

## Alternatives considered

- **Keep Milvus/Qdrant for the code index; do not add Weaviate.** This was the prior recommendation
  (net-LOC negative, no new provider). **Superseded** by the operator decision to minimize tools via a
  single platform vector store (Weaviate). Recorded for traceability.
- **Carve memory out to an OSS product (cognee/Mem0) as a replacement.** Rejected: ADR-100 assigns
  memory to mcb, the storage exists, and the defect is a missing `org_id` (root-fix). cognee remains
  viable as a future graph **overlay**.
- **New `AuthContext` type independent of `OrgContext`.** Rejected (SSOT / no-wrapper): compose the
  existing `OrgContext`.
- **Weaviate via metadata filter instead of native tenants.** Rejected for Weaviate: since `org_id` is
  threaded anyway, native multi-tenancy gives stronger (physical) isolation at low marginal cost.

## References

- cosmos-main ADR-100 (capability matrix, l.72 Weaviate, l.77 mcb boundary, l.86 Keycloak SSOT;
  object model; delivery gates; Fases 3/9/11)
- cosmos-main ADR-117 (vetor único Weaviate, mcp-gateway, amendments) — PROPOSTO
- cosmos-main ADR-112 (no-bypass / root-cause mandate)
- ADR-003 (unified provider architecture), ADR-034..038 (workflow), ADR-047 (project architecture),
  ADR-048 (OTEL), ADR-055 (constants SSOT)
