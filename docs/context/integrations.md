# Integration Points Context

**Last updated:** 2026-02-11
**Source:** `mcb-providers/src/`, `mcb-server/src/`, `mcb-infrastructure/src/`

## Overview

MCB composes AI embeddings, vector stores, SQLite, MCP protocol, git, tree-sitter, events, and an admin web UI. All integrations follow port/adapter — domain defines traits, providers implement.

## Embedding Providers (`mcb-providers/src/embedding/`)

**Port:** `EmbeddingProvider` — `embed_batch()`, `dimensions()`, `health_check()`

| Provider | Protocol | Auth | Models (dims) | Env Key Suffix |
|----------|----------|------|---------------|----------------|
| OpenAI | HTTP REST | Bearer | 3-small(1536), 3-large(3072), ada-002(1536) | `OPENAI__API_KEY` |
| VoyageAI | HTTP REST | Bearer | voyage-code-3(1024), voyage-3(1024) | `VOYAGEAI__API_KEY` |
| Ollama | HTTP REST | None | nomic(768), minilm(384), mxbai(1024) | `OLLAMA__BASE_URL` |
| Gemini | HTTP REST | API key | text-embedding-004(768) | `GEMINI__API_KEY` |
| FastEmbed | Local ONNX | None | AllMiniLML6V2(384) — Actor pattern | Model enum |
| Anthropic | HTTP REST | x-api-key | claude-3-5-sonnet (optional) | `ANTHROPIC__API_KEY` |

All env keys prefixed with `MCP__PROVIDERS__EMBEDDING__CONFIGS__`. Default timeout: 30s.

## Vector Stores (`mcb-providers/src/vector_store/`)

**Port:** `VectorStoreProvider` + `VectorStoreAdmin` + `VectorStoreBrowser`

| Store | Protocol | Auth | Algorithm | Use Case |
|-------|----------|------|-----------|----------|
| EdgeVec | In-process | None | HNSW (M=16, EF=100) | Dev/test, single-instance |
| Milvus | gRPC | Optional | IVF_FLAT (NLIST=128) | Production cloud |
| Qdrant | HTTP REST | API key | HNSW configurable | Production cloud |
| Pinecone | HTTP REST | API key | Pre-created index | Managed cloud |
| Encrypted | Wraps any | N/A | AES-256-GCM decorator | Security-sensitive |

## Database (`mcb-providers/src/database/sqlite/`)

- **SQLite** via sqlx v0.8 — primary persistence, FTS5 full-text search
- **Schema**: 48+ FKs, 69+ indexes, SHA256 dedup triggers
- **Repositories**: Memory, Agent, Org, VCS, Plan, Issue, Project (7 total)
- **Path**: `MCP__DATA__DATABASE__PATH` (default: `~/.mcb/data/mcb.db`)

## MCP Protocol (`mcb-server/src/`)

- **SDK**: rmcp v0.15 — `McpServer` implements `ServerHandler`
- **Tools**: 12+ (index, search, validate, memory, session, agent, project, vcs, vcs_entity, org_entity, issue_entity, plan_entity)
- **Transport**: stdio (`transport/stdio.rs`) + HTTP/Rocket (`transport/http.rs`)
- **Endpoints**: `POST /mcp`, `GET /events` (SSE), `/healthz`, `/metrics`

## Admin Web UI (`mcb-server/src/admin/`)

- **Template**: Handlebars 6.x — `templates/engine/handlebars_engine.rs`
- **Framework**: Rocket v0.5 — `admin/web/router.rs`
- **CRUD**: Generic `EntityCrudAdapter` pattern — `admin/crud_adapter.rs`
- **Features**: LOV dropdowns, SSE updates, entity filtering, auth

## Git (`mcb-providers/src/git/`)

- **Library**: git2 v0.20 (libgit2) — clone, fetch, branches, commits, diffs
- **Project detection**: Cargo, npm, Python, Go, Maven manifest parsing
- **Submodules**: List, initialize, update

## Language Support (`mcb-providers/src/language/`)

**13 languages** via tree-sitter v0.26: Rust, Python, JS/TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin
- Language-specific processors with fallback to generic chunking
- File extension → language detection

## Cache & Events

| Type | Local | Distributed |
|------|-------|------------|
| Cache | Moka v0.12 (in-memory) | Redis v1.0 (TCP) |
| Events | Tokio broadcast channels | NATS v0.46 (TCP) |

## Configuration (`mcb-infrastructure/src/config/`)

**Library**: figment v0.10 — Sources: Default TOML → Override TOML → `MCP__*` env vars (highest priority)

```
MCP__PROVIDERS__EMBEDDING__PROVIDER=openai
MCP__PROVIDERS__VECTOR_STORE__PROVIDER=edgevec
MCP__SERVER__NETWORK__PORT=8080
MCP__INFRASTRUCTURE__CACHE__PROVIDER=moka
```

## Crypto (`mcb-infrastructure/src/crypto/`)

- **Encryption**: AES-256-GCM (vector store encryption)
- **Password**: Argon2 (preferred), Bcrypt (legacy)
- **Tokens**: JWT-like generation/validation

## Summary Table

| Integration | Type | Protocol | Auth | File |
|-------------|------|----------|------|------|
| OpenAI | Embedding | HTTP | Bearer | `embedding/openai.rs` |
| VoyageAI | Embedding | HTTP | Bearer | `embedding/voyageai.rs` |
| Ollama | Embedding | HTTP | None | `embedding/ollama.rs` |
| Gemini | Embedding | HTTP | API key | `embedding/gemini.rs` |
| FastEmbed | Embedding | Local | None | `embedding/fastembed.rs` |
| Anthropic | Embedding | HTTP | API key | `embedding/anthropic.rs` |
| EdgeVec | Vector | In-proc | None | `vector_store/edgevec.rs` |
| Milvus | Vector | gRPC | Optional | `vector_store/milvus.rs` |
| Qdrant | Vector | HTTP | API key | `vector_store/qdrant.rs` |
| Pinecone | Vector | HTTP | API key | `vector_store/pinecone.rs` |
| SQLite | Database | File | None | `database/sqlite/` |
| Git2 | VCS | File | SSH | `git/git2_provider.rs` |
| MCP | Server | HTTP/Stdio | Optional | `mcp_server.rs` |
| Redis | Cache | TCP | Optional | `cache/redis.rs` |
| NATS | Events | TCP | Optional | `events/nats.rs` |
| Tree-sitter | AST | In-proc | None | `language/` |

## Related Context

- `docs/context/domain-concepts.md` — entity model and ports
- `docs/context/technical-patterns.md` — provider registration
- `docs/CONFIGURATION.md` — full env var reference

## Mirror Context

- `context/project-intelligence/integrations.md` — compact operational mirror

## Change Notes

- 2026-02-11T23:26:00-03:00 - Reconciled with `context/project-intelligence/integrations.md` and added mirror reference.
