# Integrations

Last updated: 2026-02-11
Source baseline: `docs/context/integrations.md`

Primary integration surfaces:

- Embedding providers: OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Anthropic.
- Vector stores: EdgeVec, Milvus, Qdrant, Pinecone, encrypted wrapper.
- Database: SQLite via `sqlx` with FTS5 and repository implementations.
- MCP server: stdio and HTTP transports in `mcb-server/src/transport/`.
- Git integration: `git2` provider and repository discovery.
- Language parsing: tree-sitter processors for multi-language chunking.
- Runtime systems: Moka/Redis cache and Tokio/NATS event pathways.

Configuration convention:

- Hierarchical config with `MCP__*` env var overrides.
- Provider choice and network settings are env-driven.

Operational notes:

- Keep provider docs aligned with `docs/CONFIGURATION.md`.
- New integration adapters must map to existing domain ports.

Related:

- `context/project-intelligence/technical-patterns.md`
- `context/project-intelligence/domain-concepts.md`
