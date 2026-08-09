<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Module Structure

This document shows the current module hierarchy for the Memory Context Browser workspace.

## Crate Structure

```text
mcb/
├── Cargo.toml (workspace root)
├── crates/
│   ├── mcb/                          # Facade crate (public re-exports)
│   │   └── src/lib.rs
│   │
│   ├── mcb-domain/                   # Domain layer (entities, value objects, ports)
│   │   └── src/
│   │       ├── ports/
│   │       │   ├── providers/
│   │       │   ├── infrastructure/
│   │       │   ├── repositories/
│   │       │   ├── services/
│   │       │   └── *.rs
│   │       ├── entities/
│   │       ├── value_objects/
│   │       ├── events/
│   │       └── error.rs
│   │
│   ├── mcb-infrastructure/           # Infrastructure layer (DI, config, utilities)
│   │   └── src/
│   │       ├── di/
│   │       ├── config/
│   │       ├── constants/
│   │       ├── project/
│   │       ├── services/
│   │       ├── validation/
│   │       ├── routing/
│   │       ├── utils/
│   │       ├── cache/
│   │       ├── crypto/
│   │       └── infrastructure/
│   │
│   ├── mcb-providers/                # Adapter implementations
│   │   └── src/
│   │       ├── embedding/
│   │       ├── vector_store/
│   │       ├── database/sqlite/
│   │       ├── cache/
│   │       ├── events/
│   │       ├── git/
│   │       ├── hybrid_search/
│   │       ├── language/
│   │       │   ├── rust.rs
│   │       │   ├── python.rs
│   │       │   ├── javascript.rs
│   │       │   ├── go.rs
│   │       │   ├── java.rs
│   │       │   ├── c.rs
│   │       │   ├── cpp.rs
│   │       │   ├── csharp.rs
│   │       │   ├── ruby.rs
│   │       │   ├── php.rs
│   │       │   ├── swift.rs
│   │       │   ├── kotlin.rs
│   │       │   ├── detection.rs
│   │       │   ├── engine.rs
│   │       │   └── mod.rs
│   │       ├── analysis/
│   │       ├── project_detection/
│   │       ├── vcs/
│   │       ├── workflow/
│   │       ├── utils/
│   │       └── constants/
│   │
│   ├── mcb-server/                   # MCP server and web/admin APIs
│   │   └── src/
│   │       ├── handlers/
│   │       ├── tools/
│   │       ├── transport/
│   │       ├── admin/
│   │       │   └── web/
│   │       ├── hooks/
│   │       ├── session/
│   │       ├── templates/
│   │       ├── utils/
│   │       └── main.rs
│   │
│   └── mcb-validate/                 # Architecture and quality validation
│       └── src/
│           ├── validators/
│           └── report.rs
```

## Architecture Layers

| Layer | Crate | Purpose |
| ------- | ------- | --------- |
| Domain | `mcb-domain` | Entities, value objects, domain ports |
| Infrastructure | `mcb-infrastructure` | DI, config, routing, technical services |
| Providers | `mcb-providers` | External adapters and implementations |
| Server | `mcb-server` | MCP protocol handlers and admin/web surfaces |
| Validation | `mcb-validate` | Architecture and quality gates |
| Facade | `mcb` | Public API re-exports |

## Feature Flags

| Feature | Default | Description |
| --------- | --------- | ------------- |
| `embedding-ollama` | Yes | Ollama embedding provider |
| `embedding-openai` | No | OpenAI embedding provider |
| `embedding-voyageai` | No | VoyageAI embedding provider |
| `embedding-gemini` | No | Google Gemini embedding provider |
| `embedding-fastembed` | No | FastEmbed local embeddings |
| `vectorstore-memory` | Yes | In-memory vector store |
| `vectorstore-encrypted` | No | AES-GCM encrypted store |
| `cache-moka` | Yes | Moka cache provider |
| `cache-redis` | No | Redis cache provider |
| `lang-all` | Yes | 13 languages (12 parsers; JavaScript processor handles TypeScript mode) |

### Updated: 2026-02-12 - Reflects modular crate architecture (v0.2.1)
