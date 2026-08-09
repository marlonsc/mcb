# Technical Patterns

Last updated: 2026-02-15 (America/Sao_Paulo)

## Project Identity

- **Name**: MCB (Memory Context Browser)
- **Type**: MCP server — semantic code search, persistent AI memory, architecture validation
- **Version**: 0.2.1-dev | **Lang**: Rust Edition 2024 | **Toolchain**: nightly, MSRV 1.92
- **Binary**: `mcb serve` (MCP server) | `mcb validate` (architecture validation)

## Architecture: Clean Architecture (Hexagonal)

Strict inward-only dependency flow enforced at compile time:

```
mcb (facade/CLI binary)
  → mcb-server       (MCP protocol, transport, handlers, admin UI, Rocket)
    → mcb-infrastructure (DI, config/Figment, cache/Moka, logging/tracing)
      → mcb-application  (use cases, service orchestration, decorators)
        → mcb-domain     (entities, ports, errors, value objects, schema — ZERO infra deps)
  → mcb-providers    (concrete: embedding, vector, DB, git, language parsers)
  → mcb-validate     (rule engine, AST analysis, metrics, duplication detection)
```

## Workspace Crates (7)

| Crate | Role | Key Modules |
|-------|------|-------------|
| `mcb` | CLI facade, binary entry | `cli/{serve,validate}.rs`, `main.rs` |
| `mcb-domain` | Pure domain layer | `entities/`, `ports/`, `error/`, `value_objects/`, `schema/`, `events/` |
| `mcb-application` | Use cases | `use_cases/{indexing,search,memory,context,agent_session}_service.rs` |
| `mcb-infrastructure` | DI + infra | `di/`, `config/`, `cache/`, `routing/`, `validation/`, `project/` |
| `mcb-server` | Protocol + transport | `handlers/`, `tools/`, `admin/`, `transport/`, `hooks/`, `templates/` |
| `mcb-providers` | Implementations | `embedding/`, `vector_store/`, `database/sqlite/`, `git/`, `language/`, `hybrid_search/` |
| `mcb-validate` | Validation engine | `validators/`, `engines/`, `rules/` (YAML), `ast/`, `metrics/`, `linters/` |

## Dependency Injection (3-layer, ADR-024)

1. **linkme distributed slices** (compile-time): `#[linkme::distributed_slice]` for auto-discovery
2. **dill Catalog** (runtime IoC): `catalog.rs::build_catalog()` composition root
3. **Handle pattern** (hot-swap): `Handle<Arc<dyn Trait>>` with `RwLock` for live provider switching

`extern crate mcb_providers` in `main.rs` forces linkme registrations to link.

## MCP Tool Registration

9 tools via `linkme::distributed_slice(TOOL_DESCRIPTORS)`:
`index`, `search`, `validate`, `memory`, `session`, `agent`, `project`, `vcs`, `entity`

Each tool = Args struct (`args/`) + Handler (`handlers/`) + Schema (`schemars::schema_for!`)

## Provider Architecture (Multi-Provider, ADR-030)

| Domain | Providers | Default |
|--------|-----------|---------|
| Embedding | FastEmbed, Ollama, OpenAI, VoyageAI, Gemini, Anthropic | FastEmbed |
| Vector Store | EdgeVec, Milvus, Qdrant, Pinecone, Encrypted | EdgeVec |
| Database | SQLite (sqlx) | SQLite |
| Cache | Moka, Redis | Moka |
| Event Bus | Tokio channels, NATS | Tokio |
| VCS | git2 | git2 |
| Language | 14 tree-sitter parsers (Rust, Python, JS/TS, Go, Java, C/C++/C#, Ruby, PHP, Swift, Kotlin) | All built-in |

Contracts: `crates/mcb-domain/src/ports/providers/`

## Configuration (Figment, ADR-025)

`config/default.toml` → env vars (`MCB_*`) → CLI args. Hot-reload: `notify` + `arc-swap`.

## Async Pattern

Tokio full + `#[async_trait]` + `futures`/`rayon` (CPU-bound). All I/O is async.

## Key ADR Series

- **Foundation** (001–013): Crate structure, async, providers, events, DI, clean arch
- **Modernization** (024–026): Simplified DI (dill), Figment config, Rocket (from Axum)
- **Consolidation** (033): MCP handler consolidation
- **Workflow** (034–037): FSM, Context Scout, Policies, Orchestrator
- **Context System** (041–046): Knowledge graph, hybrid search, freshness, versioning

## Sources

- `Cargo.toml`, `crates/*/src/lib.rs`, `crates/mcb-infrastructure/src/di/mod.rs`
- `crates/mcb-server/src/tools/registry.rs`, `docs/architecture/ARCHITECTURE.md`

## Update Notes

- 2026-02-15: Full harvest rewrite — comprehensive patterns from codebase analysis.
- 2026-02-12: Pointed to docs/architecture/PATTERNS.md as source of truth.
