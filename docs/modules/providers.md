<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Providers Module

**Source**: `crates/mcb-providers/src/`
**Crate**: `mcb-providers`

**Project links**: See `docs/architecture/ARCHITECTURE.md` and `docs/developer/ROADMAP.md` for provider architecture and v0.2.1 roadmap alignment.

## Overview

The providers module implements a trait-based abstraction layer for AI and storage services. All integrations follow port/adapter — the domain defines traits, providers implement. This enables flexible deployment with multiple providers, intelligent routing, and automatic failover.

All port traits are resolved via**dill Catalog** (ADR-029) for dependency injection.

## Embedding Providers (`embedding/`)

Transform text into vector embeddings.

**Port:** `EmbeddingProvider` — `embed_batch()`, `dimensions()`, `health_check()` (`Send + Sync`)

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}
```

| Provider | Protocol | Auth | Models (dimensions) | Env Key Suffix | Use Case |
| ---------- | ---------- | ------ | --------------------- | ---------------- | ---------- |
| OpenAI | HTTP REST | Bearer | 3-small (1536), 3-large (3072), ada-002 (1536) | `OPENAI__API_KEY` | Production |
| VoyageAI | HTTP REST | Bearer | voyage-code-3 (1024), voyage-3 (1024) | `VOYAGEAI__API_KEY` | Code-specialized |
| Ollama | HTTP REST | None | nomic (768), minilm (384), mxbai (1024) | `OLLAMA__BASE_URL` | Self-hosted |
| Gemini | HTTP REST | API key | text-embedding-004 (768) | `GEMINI__API_KEY` | Alternative |
| FastEmbed | Local ONNX | None | AllMiniLML6V2 (384) — Actor pattern | Model enum | Privacy-first |
| Anthropic | HTTP REST | x-api-key | claude-3-5-sonnet (optional) | `ANTHROPIC__API_KEY` | Optional |
| Null | — | — | fixed vectors (128) | — | Testing |

All env keys are prefixed with `MCP__PROVIDERS__EMBEDDING__CONFIGS__`. Default timeout: 30s.

## Vector Store Providers (`vector_store/`)

Store and search vector embeddings.

**Port:** `VectorStoreProvider` + `VectorStoreAdmin` + `VectorStoreBrowser` (`Send + Sync`)

```rust
#[async_trait]
pub trait VectorStoreProvider: Send + Sync {
    async fn store(&self, collection: &str, embeddings: &[Embedding]) -> Result<()>;
    async fn search(&self, collection: &str, query: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    async fn delete_collection(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;
}
```

| Provider | Protocol | Auth | Algorithm | Use Case |
| ---------- | ---------- | ------ | ----------- | ---------- |
| EdgeVec | In-process | None | HNSW (M=16, EF=100) | Dev/test, single-instance |
| Milvus | gRPC | Optional | IVF_FLAT (NLIST=128) | Production cloud |
| Qdrant | HTTP REST | API key | HNSW configurable | Production cloud |
| Pinecone | HTTP REST | API key | Pre-created index | Managed cloud |
| Encrypted | Wraps any | N/A | AES-256-GCM decorator | Security-sensitive |

## Database (`database/sqlite/`)

- **Engine**: SQLite via sqlx v0.8 — primary persistence
- **Full-text search**: FTS5 for lexical search
- **Schema**: 48+ foreign keys, 69+ indexes, SHA256 dedup triggers
- **Path**: `MCP__DATA__DATABASE__PATH` (default: `~/.mcb/data/mcb.db`)

**Repository implementations** (7 total):

| Repository | Domain Port | Purpose |
| ----------- | ------------- | --------- |
| MemoryRepo | `MemoryRepository` | Observation storage + FTS search |
| AgentRepo | `AgentRepository` | Agent session persistence + query |
| OrgRepo | `OrgEntityRepository` | Multi-tenant org data |
| VcsRepo | `VcsEntityRepository` | Repository/branch persistence |
| PlanRepo | `PlanEntityRepository` | Plan version/review persistence |
| IssueRepo | `IssueEntityRepository` | Issue tracking persistence |
| ProjectRepo | `ProjectRepository` | Project CRUD |

## Hybrid Search (`hybrid_search/`)

Combines BM25 lexical search (via FTS5) with semantic vector search for improved recall and precision.

**Port:** `HybridSearchProvider`

## Cache Providers (`cache/`)

| Provider | Backend | Protocol | Use Case |
| ---------- | --------- | ---------- | ---------- |
| Moka | In-memory LRU (v0.12) | In-process | Default, high-performance |
| Redis | Redis server (v1.0) | TCP | Distributed caching |

## Events (`events/`)

| Type | Local | Distributed |
| ------ | ------- | ------------- |
| Events | Tokio broadcast channels | NATS v0.46 (TCP) |

## Git Provider (`git/`)

- **Library**: git2 v0.20 (libgit2)
- **Operations**: clone, fetch, branches, commits, diffs
- **Project detection**: Cargo, npm, Python, Go, Maven manifest parsing
- **Submodules**: list, initialize, update

## Language Processors (`language/`)

AST-based code chunking via**tree-sitter v0.26**for**13 languages (12 parsers; JavaScript handles both JS and TS)**. Language-specific processors with fallback to generic chunking. File extension → language detection.

| Language | Parser | Status |
| ---------- | -------- | -------- |
| Rust | tree-sitter-Rust | Production |
| Python | tree-sitter-python | Production |
| JavaScript | tree-sitter-JavaScript | Production |
| TypeScript | JavaScript processor (TS mode) | Production |
| Go | tree-sitter-go | Production |
| Java | tree-sitter-java | Production |
| C | tree-sitter-c | Production |
| C++ | tree-sitter-cpp | Production |
| C# | tree-sitter-c-sharp | Production |
| Ruby | tree-sitter-ruby | Production |
| PHP | tree-sitter-php | Production |
| Swift | tree-sitter-swift | Production |
| Kotlin | tree-sitter-kotlin-ng | Production |

## Analysis (`analysis/`)

Native code analysis using Rust-code-analysis integration:

- `native.rs` — RCA-based metrics analysis

## Workflow (`workflow/`)

Workflow engine implementations for agent session state management.

## Configuration

**Library**: figment v0.10 — Sources: Default TOML → Override TOML → `MCP__*` env vars (highest priority)

```text
MCP__PROVIDERS__EMBEDDING__PROVIDER=openai
MCP__PROVIDERS__VECTOR_STORE__PROVIDER=edgevec
MCP__SERVER__NETWORK__PORT=8080
MCP__INFRASTRUCTURE__CACHE__PROVIDER=moka
```

## Integration Summary

| Integration | Type | Protocol | Auth | Source File |
| ------------- | ------ | ---------- | ------ | ------------- |
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
| Redis | Cache | TCP | Optional | `cache/redis.rs` |
| NATS | Events | TCP | Optional | `events/nats.rs` |
| Tree-sitter | AST | In-proc | None | `language/` |

## File Structure (Actual)

```text
crates/mcb-providers/src/
├── analysis/           # Code analysis
│   ├── native.rs       # Rust-code-analysis integration
│   └── mod.rs
├── cache/
│   ├── moka.rs         # Moka cache (feature-gated)
│   ├── redis.rs        # Redis cache (feature-gated)
│   └── mod.rs
├── database/
│   └── sqlite/         # SQLite + FTS5 repositories
├── embedding/
│   ├── anthropic.rs    # Anthropic API
│   ├── fastembed.rs    # Local ONNX embeddings (feature-gated)
│   ├── gemini.rs       # Google Gemini
│   ├── helpers.rs      # Shared embedding utilities
│   ├── ollama.rs       # Self-hosted
│   ├── openai.rs       # OpenAI API
│   ├── voyageai.rs     # VoyageAI
│   └── mod.rs
├── events/             # Event bus implementations
├── git/                # git2 VCS provider
├── hybrid_search/      # BM25 + semantic combined search
├── language/
│   ├── common/         # Shared language utilities
│   │   ├── config.rs   # Language configuration
│   │   ├── constants.rs # Language constants
│   │   ├── processor.rs # Common processor logic
│   │   ├── traverser.rs # AST traversal utilities
│   │   └── mod.rs
│   ├── detection.rs    # Language detection
│   ├── engine.rs       # Chunking engine
│   ├── rust.rs         # Rust processor
│   ├── python.rs       # Python processor
│   ├── javascript.rs   # JavaScript + TypeScript (mode)
│   ├── go.rs           # Go processor
│   ├── java.rs         # Java processor
│   ├── c.rs            # C processor
│   ├── cpp.rs          # C++ processor
│   ├── csharp.rs       # C# processor
│   ├── ruby.rs         # Ruby processor
│   ├── php.rs          # PHP processor
│   ├── swift.rs        # Swift processor
│   ├── kotlin.rs       # Kotlin processor
│   └── mod.rs
├── utils/              # Shared utilities
├── vector_store/
│   ├── edgevec.rs      # In-process HNSW
│   ├── encrypted.rs    # AES-GCM encrypted decorator (feature-gated)
│   ├── helpers.rs      # Shared vector utilities
│   ├── milvus.rs       # Milvus gRPC client
│   ├── pinecone.rs     # Pinecone REST client
│   ├── qdrant.rs       # Qdrant REST client
│   └── mod.rs
├── workflow/
│   ├── mod.rs          # Workflow module
│   └── transitions.rs  # State transitions
├── constants.rs        # Provider constants
├── provider_utils.rs   # Provider utilities
└── lib.rs              # Crate root
```

## Testing

Provider tests are located in `crates/mcb-providers/tests/`.

---

### Updated 2026-02-12 — Enriched with Anthropic embedding, external vector stores (Milvus/Qdrant/Pinecone), SQLite/FTS5, hybrid search, git, events, config, and 13-language support via 12 parser implementations (v0.2.1)
