# Domain Concepts

Last updated: 2026-02-11
Source baseline: `docs/context/domain-concepts.md`

Core domain model:

- `Project`, `Repository`, and `CodeChunk` drive indexing and search.
- `Organization` is the tenant isolation root (`org_id` everywhere).
- `Observation` stores memory artifacts (code, decisions, context, errors, summaries).
- `AgentSession` and workflow entities track execution lifecycle.

Important value objects:

- `Embedding`, `SearchResult`, and strong-typed IDs.
- Browse value objects under `mcb-domain/src/value_objects/browse/`.

Domain boundaries (ports):

- Provider ports: embedding, vector store, chunking, vcs, cache, crypto.
- Repository ports: memory, project, plan, issue, org, vcs entities.
- Service ports: indexing, search, context assembly, validation, memory, sessions.

Invariants:

1. Multi-tenant isolation is mandatory.
2. Search combines semantic and hybrid strategies.
3. Memory observations remain queryable for context recall.

Related:

- `context/project-intelligence/technical-patterns.md`
- `context/project-intelligence/integrations.md`
