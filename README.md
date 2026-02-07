# MCP Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.89%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.2.0-blue)](https://github.com/marlonsc/mcb/releases/tag/v0.2.0)
[![Roadmap](https://img.shields.io/badge/roadmap-v0.3.0%20%2B%20v0.4.0-lightgreen)](./RELEASE_ROADMAP_v0.3.0-v0.4.0.md)

**High-performance MCP server for semantic code search using vector embeddings**

## Overview

MCP Context Browser is a Model Context Protocol (MCP) server that provides semantic code search capabilities using vector embeddings. Transform natural language queries into code search across indexed codebases, enabling intelligent code discovery and analysis. Built with Clean Architecture principles in Rust with comprehensive provider support.

**Current Version**: 0.2.0  
**In Development**: v0.3.0 (Workflow System), v0.4.0 (Integrated Context System)

See [`CLAUDE.md`](./CLAUDE.md) for development guide and [`docs/architecture/ARCHITECTURE.md`](./docs/architecture/ARCHITECTURE.md) for complete architecture documentation.

## Installation

### From source (recommended)

Prerequisites: Rust toolchain (1.89+), `make`, and a POSIX shell.

```bash
# Build release binary
make build-release

# Install as a user systemd service (installs to ~/.claude/servers/claude-context-mcp)
make install
```

For a faster dev install, use `make install-debug`. If you prefer to run without systemd, build with `make build-release` and run `target/release/mcb` directly.

### Main Features

-   **Semantic Code Search**: Natural language queries → code discovery using vector embeddings
-   **Clean Architecture**: 8 crates (domain, application, infrastructure, providers, server, validate) per Clean Architecture layers
-   **Provider Ecosystem**: 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null), 6 vector stores (In-Memory, Encrypted, Filesystem, Milvus, EdgeVec, Null)
-   **Multi-Language Support**: AST-based parsing for 14 languages (Rust, Python, JS/TS, Go, Java, C/C++/C#, Ruby, PHP, Swift, Kotlin)
-   **Architecture Validation**: mcb-validate crate, Phases 1–7 (CA001–CA009, metrics, duplication); 2209+ tests project-wide
-   **Linkme Provider Registration**: Compile-time provider discovery (zero runtime overhead)
-   **Workflow System** (v0.3.0): FSM-based task orchestration with context awareness and policy enforcement
-   **Integrated Context** (v0.4.0 - Planned): Knowledge graph, hybrid search, freshness tracking, time-travel queries (blocked on v0.3.0)

## Architecture

MCP Context Browser follows **Clean Architecture** with strict layer separation across 8 Cargo workspace crates:

```
crates/
├── mcb/                 # Facade crate (re-exports public API)
├── mcb-domain/          # Layer 1: Entities, ports (traits), errors
├── mcb-application/     # Layer 2: Use cases, services orchestration
├── mcb-providers/       # Layer 3: Provider implementations (embedding, vector stores)
├── mcb-infrastructure/  # Layer 4: DI, config, cache, crypto, health, logging
├── mcb-server/          # Layer 5: MCP protocol, handlers, transport
└── mcb-validate/        # Dev tooling: architecture validation (Phases 1–7)
```

**Dependency Direction** (inward only):

```
mcb-server → mcb-infrastructure → mcb-application → mcb-domain
                    ↓
              mcb-providers
```

### Key Architectural Decisions

**Foundation (v0.1.0+)**:

-   **ADR-001**: Modular Crates Architecture – 8 crates, separation of concerns
-   **ADR-002**: Async-First Architecture – Tokio throughout
-   **ADR-013**: Clean Architecture Crate Separation – Port/Adapter pattern

**Dependency Injection (v0.1.2+)**:

-   **ADR-029**: Hexagonal Architecture with dill – DI IoC container, handles, linkme registry (replaces Shaku)
-   **ADR-023**: Inventory to Linkme Migration – Compile-time provider registration

**Provider Architecture (v0.2.0+)**:

-   **ADR-003**: Unified Provider Architecture & Routing – Consolidated embedding and vector store strategies
-   **ADR-030**: Multi-Provider Strategy (superseded by ADR-003)

**Workflow System (v0.3.0 - In Development)**:

-   **ADR-034**: Workflow Core FSM – State machine for task orchestration
-   **ADR-035**: Context Scout Architecture – Context gathering and search
-   **ADR-036**: Enforcement Policies – Policy engine for workflow validation
-   **ADR-037**: Orchestrator Pattern – Multi-layer task coordination
-   **ADR-038**: Multi-Tier Execution – Hierarchical execution tiers

**Integrated Context (v0.4.0 - Planned)**:

-   **ADR-041**: Context System Architecture – 5-layer integrated context (graph → search → versioning → policies)
-   **ADR-042**: Knowledge Graph – petgraph-based relationships, tree-sitter semantic extraction
-   **ADR-043**: Hybrid Search – RRF fusion of semantic + BM25 ranking with freshness weighting
-   **ADR-044**: Lightweight Discovery Models – AST-based routing, rhai rules, optional ML
-   **ADR-045**: Context Versioning – Immutable snapshots, time-travel queries, TTL garbage collection
-   **ADR-046**: FSM & Policy Integration – Workflow gating, scope boundaries, compensation rollback

**Planned (v0.2.0+)**:

-   **ADR-008**: Git-Aware Semantic Indexing – Repository context and multi-branch support
-   **ADR-009**: Persistent Session Memory – Cross-session observation storage
-   **ADR-034-038**: Workflow System – FSM, context discovery, policy enforcement, orchestration, multi-tier

See [`docs/adr/`](./docs/adr/) for complete Architecture Decision Records (46 total) and [`docs/architecture/ARCHITECTURE.md`](./docs/architecture/ARCHITECTURE.md) for detailed architecture documentation.

## Usage

### Requirements

-   Rust 1.89+ (edition 2024)
-   For embedding providers: API keys (OpenAI, VoyageAI, Gemini) or local Ollama instance
-   For vector stores: Milvus/Qdrant instance (or use in-memory for development)

### Build and Run

```bash
# Build
make build-release

# Run tests
make test

# Validate architecture
make validate
```

### MCP Tools

The server exposes 8 consolidated MCP tools:

| Tool | Purpose |
|------|---------|
| `index` | Index operations (start/status/clear) |
| `search` | Unified search for code and memory |
| `validate` | Validation and complexity analysis |
| `memory` | Memory storage, retrieval, timeline, inject |
| `session` | Session lifecycle + summaries |
| `agent` | Agent activity logging |
| `project` | Project workflow operations |
| `vcs` | Repository operations |

### Configuration

Configure via environment variables (see [`CLAUDE.md`](./CLAUDE.md) for details):

```bash
# Embedding provider (openai, voyageai, ollama, gemini, fastembed)
export EMBEDDING_PROVIDER=ollama
export OLLAMA_MODEL=nomic-embed-text

# Vector store (in-memory, encrypted, null)
export VECTOR_STORE_PROVIDER=in-memory
```

See [`docs/CONFIGURATION.md`](./docs/CONFIGURATION.md) for complete configuration guide.

## Development

### Commands

Always use `make` commands (see [`CLAUDE.md`](./CLAUDE.md)):

```bash
make build          # Debug build
make build-release  # Release build
make test           # All tests (950+)
make quality        # Full check: fmt + lint + test
make validate       # Architecture validation
```

### Quality Gates

-   All tests pass (`make test`)
-   Clean Rust lint (`make lint`); clean Markdown lint (`make docs-lint`)
-   Zero architecture violations (`make validate`)
-   No new `unwrap/expect` in code

See [`docs/developer/CONTRIBUTING.md`](./docs/developer/CONTRIBUTING.md) for contribution guidelines.

## Testing

2209+ tests covering all layers:

```bash
make test           # All tests
make test-unit      # Unit tests only
cargo test test_name -- --nocapture  # Single test
```

Test organization:

-   **Domain layer**: Entity and value object tests
-   **Application layer**: Service and use case tests
-   **Infrastructure layer**: DI, config, cache tests
-   **Providers**: Embedding and vector store provider tests
-   **mcb-validate**: Architecture validation (Phases 1–7, 2209+ tests)

See [`docs/INTEGRATION_TESTS.md`](./docs/INTEGRATION_TESTS.md) for testing documentation.

## Documentation

-   **Quick Start**: [`docs/user-guide/QUICKSTART.md`](./docs/user-guide/QUICKSTART.md)
-   **Architecture**: [`docs/architecture/ARCHITECTURE.md`](./docs/architecture/ARCHITECTURE.md)
-   **Development**: [`CLAUDE.md`](./CLAUDE.md) and [`docs/developer/CONTRIBUTING.md`](./docs/developer/CONTRIBUTING.md)
-   **Roadmap**: [`docs/developer/ROADMAP.md`](./docs/developer/ROADMAP.md)
-   **Changelog**: [`docs/operations/CHANGELOG.md`](./docs/operations/CHANGELOG.md)
-   **ADRs**: [`docs/adr/`](./docs/adr/) - Architecture Decision Records
-   **Migration**: [`docs/migration/FROM_CLAUDE_CONTEXT.md`](./docs/migration/FROM_CLAUDE_CONTEXT.md)
-   **API (docs.rs)**: [mcb](https://docs.rs/mcb) (when published)

## Contributing

Contributions welcome! See [`docs/developer/CONTRIBUTING.md`](./docs/developer/CONTRIBUTING.md) for guidelines.

Quality requirements:

-   Follow Clean Architecture principles
-   Add tests for new features
-   Update ADRs for architectural changes
-   Run `make quality` before committing

## License

MIT Licensed - Open source and free for commercial and personal use.

---

**Last Updated**: 2026-01-28
