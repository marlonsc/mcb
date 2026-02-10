# MCB — Memory Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.2.0-green)](https://github.com/marlonsc/mcb/releases/tag/v0.2.0)

**Memory Context Browser** (MCB) is a high-performance [MCP](https://modelcontextprotocol.io/) server
that gives AI coding agents persistent memory, semantic code search, and deep codebase
understanding — all through the standard Model Context Protocol.

## Features

- 🔍 **Semantic Code Search** — Natural language queries over indexed codebases using vector embeddings
- 🧠 **Persistent Memory** — Cross-session observation storage with timeline, tagging, and context injection
- 🏗️ **Multi-Provider Architecture** — 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed) and 6 vector stores (Milvus, EdgeVec)
- 🌳 **AST-Aware Analysis** — Tree-sitter parsing for 14 languages (Rust, Python, JS/TS, Go, Java, C/C++/C#, Ruby, PHP, Swift, Kotlin)
- ✅ **Architecture Validation** — Built-in Clean Architecture rule enforcement (9 rules, 7 phases, 3083+ tests)
- ✅ **Architecture Validation** — Built-in Clean Architecture rule enforcement (9 rules, 7 phases, 3083+ tests)
- 🔌 **MCP Protocol Native** — Seamless integration with Claude Desktop, Claude Code, and any MCP-compatible client
- 🔒 **Git-Aware Indexing** — Repository-level context with branch comparison and impact analysis

## Quick Start

### Prerequisites

- Rust 1.92+ (`rustup` recommended)
- `make` and a POSIX shell
- An embedding provider: [Ollama](https://ollama.ai/) (local, free) or an API key (OpenAI, VoyageAI, Gemini)

### Build & Install

```bash
git clone https://github.com/marlonsc/mcb.git
cd mcb

# Build release binary
make build-release

# Install as a systemd user service
make install
```

### Configure

```bash
# Option A: Local embeddings (free, no API key)
export EMBEDDING_PROVIDER=fastembed

# Option B: Ollama (local, more models)
export EMBEDDING_PROVIDER=ollama
export OLLAMA_BASE_URL=http://localhost:11434

# Option C: Cloud embeddings
export EMBEDDING_PROVIDER=openai
export OPENAI_API_KEY=sk-your-key
```

See [Configuration Guide](./docs/CONFIGURATION.md) for all options.

### Integrate with Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "mcb": {
      "command": "mcb",
      "args": ["serve", "--stdio"]
    }
  }
}
```

## MCP Tools

MCB exposes 8 tools through the MCP protocol:

| Tool | Description | Status |
|------|-------------|--------|
| `mcb_index` | Index codebases for semantic search (start, status, clear) | ✅ Stable |
| `mcb_search` | Semantic search across code and memory | ✅ Stable |
| `mcb_memory` | Store, retrieve, and query persistent observations | ✅ Stable |
| `mcb_validate` | Architecture validation and complexity analysis | ✅ Stable |
| `mcb_vcs` | Git operations — repo indexing, branch comparison, impact analysis | ✅ Stable |
| `mcb_session` | Session lifecycle management and summaries | ✅ Stable |
| `mcb_agent` | Agent activity logging and tracking | ✅ Stable |
| `mcb_project` | Project workflow management (phases, issues, decisions) | 🚧 Preview |

See [MCP Tools Documentation](./docs/MCP_TOOLS.md) for full schemas and examples.

## Architecture

MCB follows **Clean Architecture** with strict inward-only dependency flow:

```ascii
┌─────────────────────────────────────────────────┐
│                  mcb-server                      │
│           (MCP protocol, transport)              │
├─────────────────────────────────────────────────┤
│              mcb-infrastructure                  │
│        (DI, config, cache, logging)              │
├─────────────────────────────────────────────────┤
│              mcb-application                     │
│          (use cases, orchestration)              │
├─────────────────────────────────────────────────┤
│                mcb-domain                        │
│         (entities, ports, errors)                │
└─────────────────────────────────────────────────┘
         ▲                        ▲
    mcb-providers            mcb-validate
  (embeddings, stores)    (architecture rules)
```

8 workspace crates enforce layer boundaries at compile time via
[linkme](https://crates.io/crates/linkme) provider registration (zero runtime overhead).

See [Architecture Documentation](./docs/architecture/ARCHITECTURE.md) for detailed design
and [ADR index](./docs/adr/) for all 46 Architecture Decision Records.

## Documentation

### Getting Started

- [Quick Start Guide](./docs/user-guide/QUICKSTART.md) — Build, configure, and run in 5 minutes
- [Configuration Reference](./docs/CONFIGURATION.md) — All environment variables and config file options

### Architecture & Design

- [Architecture Overview](./docs/architecture/ARCHITECTURE.md) — Clean Architecture layers, crate map, dependency flow
- [Architecture Decision Records](./docs/adr/) — 46 ADRs documenting every major design choice
- [MCP Tools Schema](./docs/MCP_TOOLS.md) — Full tool API documentation

### Developer Guide

- [Contributing](./docs/developer/CONTRIBUTING.md) — Development setup, coding standards, PR process
- [Roadmap](./docs/developer/ROADMAP.md) — Version plans and feature timeline
- [Integration Tests](./docs/INTEGRATION_TESTS.md) — Test infrastructure and patterns

### Operations

- [Changelog](./docs/operations/CHANGELOG.md) — Release history and migration notes
- [Migration Guide](./docs/migration/FROM_CLAUDE_CONTEXT.md) — Upgrading from the previous project version

## Development

```bash
make build          # Debug build
make build-release  # Optimized release build
make test           # Run all tests (~2900+)
make lint           # Clippy + format check
make validate       # Architecture rule enforcement
make quality        # Full pipeline: fmt + lint + test + validate
```

### Quality Gates

All contributions must pass:

- `make lint` — Zero Clippy warnings, consistent formatting
- `make test` — All tests green
- `make validate` — Zero architecture violations
- No `unwrap()`/`expect()` in production code paths

## Planned

- **v0.3.0** — Workflow system: FSM-based task orchestration, context scout, policy enforcement
- **v0.4.0** — Integrated context: knowledge graph, hybrid search (semantic + BM25), time-travel queries

See [Roadmap](./docs/developer/ROADMAP.md) for details.

## Contributing

Contributions are welcome! Please read the [Contributing Guide](./docs/developer/CONTRIBUTING.md)
for development setup, coding standards, and the PR process.

## License

[MIT](./LICENSE) — Open source, free for commercial and personal use.
