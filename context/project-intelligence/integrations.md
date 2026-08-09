# Integrations

Last updated: 2026-02-15 (America/Sao_Paulo)

## MCP Protocol (Core Integration)

- **SDK**: `rmcp` 0.15 (server + client + macros + transport-io + transport-child-process)
- **Transport**: stdio (default) or HTTP (Rocket 0.5)
- **Tools**: 9 MCP tools exposed: `index`, `search`, `validate`, `memory`, `session`, `agent`, `project`, `vcs`, `entity`
- **Clients**: Claude Desktop, Claude Code, any MCP-compatible client

## Embedding Providers

| Provider | Crate/API | Config Key |
|----------|-----------|------------|
| FastEmbed | `fastembed` 5.9 (local, default) | `EMBEDDING_PROVIDER=fastembed` |
| Ollama | HTTP API to `OLLAMA_BASE_URL` | `EMBEDDING_PROVIDER=ollama` |
| OpenAI | `reqwest` → `api.openai.com` | `OPENAI_API_KEY` |
| VoyageAI | `reqwest` → VoyageAI API | `VOYAGEAI_API_KEY` |
| Gemini | `reqwest` → Google AI API | `GEMINI_API_KEY` |
| Anthropic | `reqwest` → Anthropic API | `ANTHROPIC_API_KEY` |

Implementation: `crates/mcb-providers/src/embedding/`

## Vector Store Providers

| Provider | Crate | Notes |
|----------|-------|-------|
| EdgeVec | `edgevec` 0.8 (local, default) | In-memory HNSW |
| Milvus | `milvus-sdk-rust` (git) | gRPC client |
| Qdrant | HTTP via `reqwest` | REST API |
| Pinecone | HTTP via `reqwest` | REST API |
| Encrypted | Wraps any store with AES-GCM | `aes-gcm` 0.10 |

Implementation: `crates/mcb-providers/src/vector_store/`

## Database

- **Primary**: SQLite via `sqlx` 0.8 (async, compile-time checked)
- **Schema**: DDL generated from domain schema (`crates/mcb-providers/src/database/sqlite/ddl.rs`)
- **Connection pool**: r2d2 (for PostgreSQL path, currently unused)
- **File hash storage**: SQLite for incremental indexing

Implementation: `crates/mcb-providers/src/database/sqlite/`

## Caching

- **Moka** 0.12 (default, in-process, async futures)
- **Redis** via `redis` 1.0 (tokio-comp + connection-manager)
- Config: `system.infrastructure.cache` section in `default.toml`

## Event Bus

- **Tokio** channels (default, in-process)
- **NATS** via `async-nats` 0.46 (distributed)
- Config: `system.infrastructure.event_bus` section

## VCS (Git)

- **git2** 0.20 for repository operations (branches, commits, diffs, submodules)
- **Project detection**: Auto-detect Cargo, npm, Python, Go, Maven projects
- Implementation: `crates/mcb-providers/src/git/`

## AST / Language Parsing

- **tree-sitter** 0.26 with 14 language grammars
- **rust-code-analysis** (forked) for metrics: cognitive complexity, cyclomatic, Halstead
- **tree-sitter-highlight** 0.26.3 for syntax highlighting in admin UI
- Implementation: `crates/mcb-providers/src/language/`

## Admin Web UI

- **Rocket** 0.5 web framework (migrated from Axum, ADR-026)
- **Handlebars** 6.0 templates (`crates/mcb-server/templates/admin/`)
- **Alpine.js** for client-side interactivity (in templates)
- SSE (Server-Sent Events) for real-time updates
- Browse UI for file tree, code viewer with syntax highlighting

## Configuration

- **Figment** 0.10 (TOML + env merge, ADR-025)
- **notify** 8.2 + `arc-swap` 1.8 for hot-reload
- Files: `config/default.toml`, `config/deploy.toml`, `config/mcb-validate.toml`

## Authentication / Crypto

- **Argon2** 0.5 / **bcrypt** 0.18 for password hashing
- **AES-GCM** 0.10 for encryption (vector store encryption wrapper)
- **JWT** planned but removed (RUSTSEC-2023-0071)
- API key auth via `X-API-Key` header

## CI/CD (`.github/workflows/ci.yml`)

- GitHub Actions: lint → test → startup-smoke → validate → golden-tests → audit → coverage → release-build
- Matrix: Ubuntu + macOS + Windows, Rust stable + beta
- CodeQL analysis, Dependabot auto-merge, Codecov
- Makefile-driven: `make lint`, `make test`, `make validate`, `make audit`

## Deployment

- **systemd** user service (`systemd/mcb.service`)
- **Kubernetes**: Full k8s manifests (`k8s/`: deployment, service, ingress, HPA, RBAC, network policy)
- **Docker**: `tests/docker/Dockerfile.test` for CI
- **Monitoring**: Prometheus alerts + Grafana dashboard (`monitoring/`)

## Dev Tooling

- **Beads** (`bd`): Git-native issue tracking
- **Playwright** E2E tests for admin UI (`tests/e2e/`)
- **qlty**: Code quality and smells analysis
- **cargo-audit**, **cargo-tarpaulin**, **cargo-udeps**
- **markdownlint**, **shellcheck**, **hadolint**

## Sources

- `Cargo.toml` (all dependency versions), `config/default.toml`
- `crates/mcb-providers/src/` (provider implementations)
- `.github/workflows/ci.yml`, `k8s/`, `monitoring/`

## Update Notes

- 2026-02-15: Full harvest rewrite — complete integration map from codebase.
- 2026-02-12: Pointed to docs/modules/providers.md as source of truth.
