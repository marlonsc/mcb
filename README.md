# MCP Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.89%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.1.1-blue)](https://github.com/marlonsc/mcp-context-browser/releases)

**High-performance MCP server for semantic code search** - AI-powered code analysis using vector embeddings. Provides intelligent, natural language code discovery with AST-based parsing for 12+ languages, supporting Claude Desktop and other AI assistants through the Model Context Protocol.

## 🏗️ Architecture

This project follows **Clean Architecture** principles with **Shaku dependency injection**:

```
📦 mcb-domain/        # Domain layer (entities, ports, business rules)
📦 mcb-application/   # Application layer (use cases, orchestration)
📦 mcb-providers/     # Adapters (external integrations, implementations)
📦 mcb-infrastructure/ # Infrastructure (DI, composition, cross-cutting)
📦 mcb-server/        # Server layer (MCP protocol, HTTP handlers)
📦 mcb-validate/      # Development tools (architecture validation)
```

### Clean Architecture Benefits

- **Dependency Inversion**: Business logic doesn't depend on infrastructure
- **Testability**: Easy mocking with DI and interface-based design
- **Maintainability**: Clear separation of concerns and responsibilities
- **Scalability**: Easy to add new providers or change implementations

## Why Switch from Claude-context?

| | Claude-context | MCP-context-browser |
|---|----------------|---------------------|
| **Runtime** | Node.js 20-23 | Single binary |
| **Startup** | npm/npx overhead | Instant |
| **Memory** | Node.js interpreter | Native efficiency |
| **Providers** | 4 embedding | 6 embedding |
| **Vector Stores** | 2 (Milvus/Zilliz) | 6 options |
| **Languages** | 13+ | 13 with AST parsing |

**Same environment variables work!** No configuration changes needed.

## Quick Start

### Claude Desktop Integration

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "context": {
      "command": "/path/to/mcp-context-browser",
      "args": [],
      "env": {
        "OPENAI_API_KEY": "sk-...",
        "MILVUS_ADDRESS": "http://localhost:19530"
      }
    }
  }
}
```

### From Source

```bash
# Build
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser
make build-release

# Run with your existing env vars
export OPENAI_API_KEY=sk-...
./target/release/mcp-context-browser
```

### Environment Variables (Claude-context compatible)

```bash
# Embedding providers
EMBEDDING_PROVIDER=openai|voyageai|ollama|gemini|fastembed

# API keys (same as claude-context)
OPENAI_API_KEY=sk-...
VOYAGE_API_KEY=...
GEMINI_API_KEY=...
OLLAMA_BASE_URL=http://localhost:11434

# Vector store
VECTOR_STORE_PROVIDER=milvus|in-memory|filesystem|edgevec
MILVUS_ADDRESS=http://localhost:19530
MILVUS_TOKEN=...
```

## Installation

### Prerequisites

- Rust 1.89+ ([install Rust](https://rustup.rs/))
- (Optional) Milvus vector database for production use

### Build from Source

```bash
# Clone the repository
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser

# Build release binary
make build-release

# Binary will be available at ./target/release/mcp-context-browser
```

### Docker Installation

```bash
# Build Docker image
make docker-build

# Run with Docker Compose
docker-compose up -d
```

### Systemd Service (Linux)

```bash
# Install as system service
sudo make install-service

# Start the service
sudo systemctl start mcp-context-browser
```

## Usage

### Basic Usage

After installation, start the MCP server:

```bash
# Set required environment variables
export OPENAI_API_KEY="your-api-key-here"
export MILVUS_ADDRESS="http://localhost:19530"

# Start the server
./target/release/mcp-context-browser
```

### MCP Tools

The server provides 4 MCP tools for AI assistants:

1. **`index_codebase`** - Index your codebase for semantic search
2. **`search_code`** - Perform natural language code search
3. **`get_indexing_status`** - Check indexing progress and system health
4. **`clear_index`** - Clear the search index

### Claude Desktop Integration

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "context": {
      "command": "/path/to/mcp-context-browser",
      "args": [],
      "env": {
        "OPENAI_API_KEY": "sk-...",
        "MILVUS_ADDRESS": "http://localhost:19530"
      }
    }
  }
}
```

### Advanced Configuration

See [DEPLOYMENT.md](docs/operations/DEPLOYMENT.md) for production deployment guides and advanced configuration options.

## Core Features

-   **Semantic Search**: Find code by meaning, not just keywords
-   **Real-Time Sync**: Automatic background updates keep results current
-   **Multi-Provider**: Support for OpenAI, Ollama, Gemini, VoyageAI
-   **Production Ready**: JWT auth, rate limiting, encryption, audit trails
-   **Comprehensive Monitoring**: Metrics API, health checks, performance tracking

## How It Works

**AST-Based Analysis** - Analyzes code structure and relationships to provide contextually relevant results.

**Intelligent Routing** - Automatically routes requests to optimal AI providers with health monitoring and failover.

**MCP Integration** - Connects directly with Claude Desktop and other AI assistants through the Model Context Protocol.

## MCP Tools

| Tool | Purpose | Implementation |
|------|---------|----------------|
| `index_codebase` | Ingest codebase | AST chunking, incremental sync |
| `search_code` | Natural language search | Hybrid BM25 + semantic vectors |
| `get_indexing_status` | System monitoring | Real-time health and progress |
| `clear_index` | Index management | Professional cleanup operations |

## Architecture

Built on production-grade foundations:

-   **Tokio async runtime** - Concurrent performance (1000+ users)
-   **Provider registry** - Thread-safe management with health monitoring
-   **Circuit breakers** - Automatic failover between providers
-   **Background processing** - Non-blocking indexing and sync
-   **Metrics collection** - Comprehensive system and performance monitoring

## Testing

790+ automated tests covering all critical functionality:

```bash
make test           # Run full test suite (790+ tests)
make quality        # Complete quality check (fmt + lint + test + audit)
make validate       # Documentation and configuration validation
```

Test organization (Clean Architecture layers):

-   **Domain tests**: Types, validation, chunking
-   **Application tests**: Services (indexing, search, context)
-   **Adapter tests**: Providers, repositories, hybrid search
-   **Infrastructure tests**: Auth, cache, events, sync, daemon
-   **Server tests**: Handlers, admin, protocol
-   **Integration tests**: End-to-end workflows, Docker
-   **Unit tests**: Property-based testing, security

See [tests/README.md](tests/README.md) for detailed test structure.

## Performance

-   **Response time**: <500ms average query response
-   **Indexing**: <30s for 1000+ files
-   **Scalability**: Handles millions of lines efficiently
-   **Concurrency**: 1000+ simultaneous users

## Documentation

-   [**Migration Guide**](docs/migration/FROM_CLAUDE_CONTEXT.md) - Migrating from Claude-context
-   [**QUICKSTART.md**](docs/user-guide/QUICKSTART.md) - Get started in 5 minutes
-   [**Claude.md**](CLAUDE.md) - Development guide and project rules
-   [**ARCHITECTURE.md**](docs/architecture/ARCHITECTURE.md) - Technical architecture
-   [**DEPLOYMENT.md**](docs/operations/DEPLOYMENT.md) - Deployment guides
-   [**CONTRIBUTING.md**](docs/developer/CONTRIBUTING.md) - Contribution guidelines
-   [**ADR Index**](docs/adr/README.md) - Architectural decisions
-   [**VERSION_HISTORY.md**](docs/VERSION_HISTORY.md) - Complete version history

## Use Cases

**Development Teams:**

-   Instant code discovery and understanding
-   Fast onboarding (days instead of weeks)
-   Identify refactoring opportunities

**AI Integration:**

-   Claude Desktop direct codebase access
-   Custom assistant development
-   Automated code review assistance

**Enterprise:**

-   Large codebase search (millions of lines)
-   Multi-language support (Rust, Python, JavaScript, etc.)
-   Security compliance with audit trails

## Current Status: v0.1.1 ✅ RELEASED

First stable release - drop-in replacement for Claude-context:

-   ✅ Full MCP protocol implementation (4 tools)
-   ✅ 12 languages with AST parsing (Rust, Python, JS/TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin)
-   ✅ 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
-   ✅ 6 vector stores (Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null)
-   ✅ Claude-context environment variable compatibility
-   ✅ 790+ tests with comprehensive coverage
-   ✅ JWT authentication and rate limiting
-   ✅ Clean architecture with 14 domain port traits and full DI wiring
-   ✅ HTTP transport foundation
-   ✅ Systemd service integration

**Migrating from Claude-context?** See [Migration Guide](docs/migration/FROM_CLAUDE_CONTEXT.md)

## Coming in v0.2.0: Git-Aware Indexing + Persistent Session Memory

Planning complete ([ADR-008](docs/adr/008-git-aware-semantic-indexing-v0.2.0.md), [ADR-009](docs/adr/009-persistent-session-memory-v0.2.0.md)):

**Git Integration:**

-   🚧 **Project-relative indexing**: Indexes remain valid if directory moves
-   🚧 **Multi-branch support**: Search specific branches or across all branches
-   🚧 **Commit history**: Index last 50 commits (configurable)
-   🚧 **Submodule support**: Recursive indexing as separate projects
-   🚧 **Monorepo detection**: Auto-detect Cargo, npm, Python, Go projects
-   🚧 **Impact analysis**: Understand change impact between commits/branches

**Session Memory:**

-   🚧 **Cross-session memory**: Persistent storage of tool observations and decisions
-   🚧 **Session summaries**: Comprehensive tracking of work completed per session
-   🚧 **Semantic search**: Search past work and decisions using natural language
-   🚧 **Progressive disclosure**: 3-layer workflow for 10x token savings
-   🚧 **Context injection**: Automatic context generation for session continuity
-   🚧 **Git-tagged memory**: Observations linked to branches and commits

## Contributing

Contributions welcome! See [CONTRIBUTING.md](docs/developer/CONTRIBUTING.md) for guidelines.

**Development philosophy:**

-   Quality first: comprehensive testing before changes
-   Documentation driven: features documented before implementation
-   Community focused: production-grade solutions for development teams

## License

MIT Licensed - Open source and free for commercial and personal use.

## Support

-   Issues: [GitHub Issues](https://github.com/marlonsc/mcp-context-browser/issues)
-   Documentation: [docs/](docs/)
-   Architecture: [ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md)
