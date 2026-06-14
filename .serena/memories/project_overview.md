# MCB Project Overview

## Identity
- **Name**: Memory Context Browser (MCB)
- **Version**: 0.3.2
- **Repository**: https://github.com/marlonsc/mcb
- **License**: MIT
- **Author**: Marlon Costa <marlonsc@gmail.com>

## Purpose
MCB is a high-performance, extensible Model Context Protocol (MCP) server that provides semantic code search, persistent agent memory, and architecture validation capabilities to AI assistants.

## Core Capabilities
- **Semantic Code Search**: Natural language to code search using vector embeddings
- **Persistent Agent Memory**: Store and retrieve observations, sessions, and context
- **Architecture Validation**: Enforce Clean Architecture rules and code quality gates
- **Multi-Language Support**: AST-based parsing for Rust, Python, JavaScript, TypeScript, and more

## Technology Stack
- **Language**: Rust 2024 edition, MSRV 1.92
- **Async Runtime**: Tokio
- **Web Framework**: Axum (HTTP), Loco.rs (application framework)
- **ORM**: SeaORM 2.0.0-rc.38 (SQLite/PostgreSQL)
- **Vector Stores**: EdgeVec (default), Milvus
- **Embeddings**: FastEmbed (default), Ollama, OpenAI
- **DI Pattern**: linkme distributed slices + AppContext composition root

## Workspace Structure
7 crates in Clean Architecture layers:
1. `mcb-utils` — pure utilities, zero internal deps
2. `mcb-domain` — entities, ports, errors
3. `mcb-providers` — port adapters (embedding, vector store, cache, etc.)
4. `mcb-infrastructure` — composition root, config, DI wiring
5. `mcb-server` — MCP protocol handlers, HTTP API
6. `mcb-validate` — architecture rule engine
7. `mcb` — CLI facade binary

## Key Files
- `Cargo.toml` — workspace definition, dependencies, lint policy
- `Makefile` — canonical developer commands
- `AGENTS.md` — project rules and conventions
- `config/development.yaml` — runtime configuration
- `docs/architecture/ARCHITECTURE.md` — comprehensive architecture docs
