<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Module Documentation Index

MCB uses a 4-layer architecture. Each layer is documented in its core module file.

## 🏗️ Core Layers (The Pillars)

- **[Domain Layer](./domain.md)** (`mcb-domain`) - Entities, Value Objects, and Port Traits (**SSOT for Logic**)
- **[Application Layer](./application.md)** (superseded — see infrastructure) - Historical use-case documentation
- **[Providers Layer](./providers.md)** (`mcb-providers`) - External integrations (Adapters) and Storage (SQLite/Vector)
- **[Infrastructure Layer](./infrastructure.md)** (`mcb-infrastructure`) - DI Bootstrap, Config, Logging, and Plumbing

## 🛠️ Specialized Modules

- **[Validation Layer](./validate.md)** (`mcb-validate`) - Architecture and Code Quality Validation Engine
- **[Project & Issue Management](./project.md)** - Deep dive into Beads (Issue tracking & Project coordination)
- **[Server & API](./server.md)** (`mcb-server`) - HTTP/MCP Transport, Admin API, and Handlers
- **[Admin Interface](./admin.md)** - Admin-specific service logic

## 📖 How to navigate this documentation

1. **Top-Down**: Start here to understand the 4 logical pillars of the system.
2. **Bottom-Up**: Every source file in the `crates/` directory contains a `//! **Documentation**` header that links directly back to the relevant section in these documents.
3. **Traceability**: Decisions are recorded in [ADRs](../adr/README.md). Cross-references between ADRs, Docs, and Code ensure a Single Source of Truth (SSOT).

### SSOT Verification
Run the automated documentation audit regularly to ensure integrity:
```bash
make docs-validate
```

---

### Updated 2026-02-20 - Consolidated 7+ redundant files into the 4 Core Pillar documents
