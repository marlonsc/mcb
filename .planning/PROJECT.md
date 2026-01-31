# MCB - MCP Context Browser

## What This Is

MCP Context Browser is a high-performance MCP server for semantic code search using vector embeddings. It serves as a drop-in replacement for claude-context with enhanced capabilities including multi-provider support, clean architecture, and comprehensive validation tooling.

## Core Value

Enable developers to find code by meaning, not just keywords - providing semantic understanding of codebases through AI-powered embeddings and vector search.

## Requirements

### Validated

- [x] Full MCP protocol implementation (5 tools) — v0.1.0
- [x] 14 languages with AST parsing support — v0.1.0
- [x] 7 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Anthropic, Null) — v0.1.5
- [x] 8 vector stores (Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Pinecone, Qdrant, Null) — v0.1.5
- [x] Clean architecture with dill-based DI (ADR-029) — v0.1.2
- [x] Linkme compile-time provider registration — v0.1.2
- [x] Architecture validation tooling (mcb-validate) — v0.1.2
- [x] Health endpoints (/healthz, /readyz) — v0.1.5
- [x] InstrumentedEmbeddingProvider decorator (SOLID) — v0.1.5
- [x] 1670+ tests passing — v0.1.5

### Active (v0.2.0)

- [ ] Git-aware semantic indexing (ADR-008)
- [ ] Persistent session memory (ADR-009)
- [ ] Advanced code browser UI (ADR-028)
- [ ] Break mcb-infrastructure → mcb-validate dependency cycle
- [ ] Consolidate language support into shared crate
- [ ] Provider health checks and config validation

### Out of Scope

- Mobile app — Web-first, defer to post-v1.0
- Real-time collaboration — Single-user focus for v0.x
- Hosted SaaS offering — Self-hosted only until v1.0

## Context

**Current State (v0.1.5):**
- Production-ready foundation with comprehensive provider ecosystem
- 1670+ tests, 0 architecture violations
- DRY/SOLID improvements completed
- Ready for major feature development (v0.2.0)

**Technical Debt:**
- mcb-validate coupled to production code (should be dev-only)
- Duplicate Tree-sitter integration across crates
- No centralized language support infrastructure
- Missing provider health checks and config validation

## Constraints

- **Tech Stack**: Rust 1.92+, Tokio async runtime, rmcp 0.14
- **Architecture**: Clean Architecture with hexagonal ports/adapters
- **Quality**: 0 architecture violations, 85%+ test coverage
- **Backwards Compat**: Environment variable compatibility with claude-context

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rocket over Axum (ADR-026) | Simpler routing, built-in templates | — Pending |
| dill IoC Container (ADR-029) | Runtime provider switching, clean DI | ✓ Good |
| Linkme for registration (ADR-023) | Compile-time provider discovery | ✓ Good |
| RCA for metrics (v0.1.4) | Comprehensive code analysis | ✓ Good |

---
*Last updated: 2026-01-31 after v0.1.5 release*
