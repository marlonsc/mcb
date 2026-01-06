# MCP Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.0.1--alpha-blue)](https://github.com/marlonsc/mcp-context-browser/releases)

**Model Context Protocol Server** - Provides semantic code search and analysis capabilities to AI assistants through a standardized MCP interface.

## 🎯 Current Capabilities

- **🔍 Semantic Code Search**: Vector-based similarity search using embeddings
- **📁 Codebase Indexing**: Process and index code files for efficient retrieval
- **🔗 MCP Protocol Support**: Standardized interface for AI assistant integration
- **💾 Extensible Storage**: Pluggable storage backends (file system, vector databases)
- **🎨 Modular Architecture**: Clean separation of concerns with provider pattern

## 📋 Documentation

- [**ARCHITECTURE.md**](ARCHITECTURE.md) - Technical architecture and design
- [**ROADMAP.md**](ROADMAP.md) - Development roadmap and milestones
- [**DEPLOYMENT.md**](DEPLOYMENT.md) - Deployment guides and configurations
- [**CONTRIBUTING.md**](CONTRIBUTING.md) - Contribution guidelines

## 📋 Documentation

- [**ARCHITECTURE.md**](ARCHITECTURE.md) - Technical architecture and design
- [**ROADMAP.md**](ROADMAP.md) - Development roadmap and milestones
- [**DEPLOYMENT.md**](DEPLOYMENT.md) - Deployment guides and configurations
- [**CONTRIBUTING.md**](CONTRIBUTING.md) - Contribution guidelines

## 📦 Quick Start

```bash
# Install Rust and clone
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser

# Run development setup
make dev
```

## 🤝 Contributing

See [**CONTRIBUTING.md**](CONTRIBUTING.md) for detailed contribution guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
