# Project State Context

**Last updated:** 2026-02-05
**Source:** `docs/developer/ROADMAP.md`, `docs/adr/phase-9/README.md`, `docs/implementation/phase-9-roadmap.md`

## Overview

Phase 8-9 "Workflow FSM + Integrated Context System" is planned for v0.3.0-v0.4.0 (Feb 17 - Mar 16, 2026). The project has completed v0.1.5 documentation updates and is preparing for Phase 8 implementation with workflow state machines, freshness tracking, and policy enforcement.

## Current Phase

-   **Phase:** 8-9 of 10 — Workflow FSM (Phase 8) + Integrated Context System (Phase 9)
-   **Version:** v0.3.0 (Phase 8) → v0.4.0 (Phase 9)
-   **Timeline:** Feb 17 - Mar 16, 2026 (4 weeks for Phase 9)
-   **ADRs:** Phase 8 (ADR-034-037), Phase 9 (ADR-041-046)
-   **Next action:** Implement Phase 8 workflow FSM, then Phase 9 context system with 70+ tests.

## Requirements & Debt

-   **Validated requirements:** Full MCP protocol, 14 languages supported, 7 embedding providers, 8 vector stores, clean architecture with linkme/dill DI, architecture validation (mcb-validate), health endpoints, instrumentation, 2040+ tests.
-   **Phase 8 work:** Workflow FSM (ADR-034), Freshness Tracking (ADR-035), Policies & Validation (ADR-036), Compensation & Orchestration (ADR-037)
-   **Phase 9 work:** Context Architecture (ADR-041), Knowledge Graph (ADR-042), Hybrid Search (ADR-043), Model Selection (ADR-044), Context Versioning (ADR-045), Integration Patterns (ADR-046)
-   **Technical debt to track:** mcb-validate currently coupled to runtime, duplicate Tree-sitter logic, missing centralized provider health/config validation.

## Roadmap Signals

-   **v0.1.5** (Current): Documentation updates for v0.4.0 planning
-   **v0.3.0** (Phase 8): Workflow FSM, Freshness Tracking, Policies, Compensation (ADR-034-037)
-   **v0.4.0** (Phase 9): Integrated Context System with Knowledge Graph, Hybrid Search, Versioning (ADR-041-046)
-   **v0.2.0** (Future): Git-aware indexing, session memory, advanced code browser
-   Key upcoming capabilities: Workflow state machines, freshness-aware search, time-travel queries, knowledge graphs, policy-driven context discovery.

## Metrics Snapshot

-   **Tests:** 2040+ (unit, integration, end-to-end)
-   **Providers:** 7 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
-   **Vector stores:** 8 vector stores (Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null)
-   **ADRs:** 46 total (ADR-001-046, including Phase 8-9)
-   **Crates:** 8 (mcb, mcb-domain, mcb-application, mcb-providers, mcb-infrastructure, mcb-server, mcb-validate, facade)

## Related Context

-   `docs/context/technical-patterns.md`
-   `docs/context/domain-concepts.md`
-   `docs/developer/ROADMAP.md`
-   `docs/user-guide/README.md` (contextual user-facing summary)
