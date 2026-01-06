# CLAUDE.md - MCP Context Browser Development Guide

## 🤖 Claude Code Assistant Configuration

**This file contains specific instructions for Claude Code when working with the MCP Context Browser project.**

---

## 📋 Project Overview

**MCP Context Browser** is a high-performance Rust-based Model Context Protocol (MCP) server that provides semantic code search capabilities using vector embeddings.

### 🎯 Core Purpose

- **Semantic Code Search**: Natural language to code search using AI embeddings
- **MCP Protocol Server**: Standardized interface for AI assistants (Claude Desktop, etc.)
- **Provider Architecture**: Extensible system supporting multiple AI and vector storage providers
- **Enterprise Ready**: Production-grade async Rust implementation with comprehensive testing

### 🏗️ Architecture Highlights

- **Async-First Design**: Tokio runtime throughout for high concurrency
- **Provider Pattern**: Clean abstraction for embeddings (OpenAI, Ollama) and vector stores (Milvus, Pinecone)
- **SOLID Principles**: Clean separation of concerns with dependency injection
- **Comprehensive Testing**: 60+ tests covering all major functionality
- **Automated Documentation**: PlantUML diagrams, ADR tracking, validation pipelines

---

## 🚀 Development Workflow

### Essential Commands (Use Make!)

```bash
# Core development cycle (VALIDATED ✅)
make build          # Build project (cargo build)
make test           # Run all tests (60 tests, 100% pass rate)
make docs           # Generate documentation + diagrams + index
make validate       # Validate diagrams, docs, links, ADRs, sync
make ci             # Full CI pipeline: clean + validate + test + build + docs

# Development (VALIDATED ✅)
make dev            # Run with auto-reload (cargo watch -x run)
make fmt            # Format code (cargo fmt)
make lint           # Lint code (cargo clippy)
make setup          # Install dev tools (cargo-watch, tarpaulin, audit)

# Documentation (VALIDATED ✅)
make adr-new        # Create new ADR interactively
make adr-list       # List all ADRs
make diagrams       # Generate PlantUML diagrams only

# Git Operations (VALIDATED ✅ - Added for force commits)
make git-status     # Show git repository status
make git-add-all    # Add all changes to git
make git-commit-force # Force commit with timestamp
make git-push-force   # Force push to remote
make git-force-all    # Complete force workflow: add + commit + push
make force-commit     # Alternative force commit via script

# Quality & Security (VALIDATED ✅)
make quality        # Run all quality checks: fmt + lint + test + audit + validate
make audit          # Security audit (⚠️ Known vulnerabilities in dependencies)
make bench          # Run benchmarks (0 defined)
make coverage       # Generate test coverage report

# Release (VALIDATED ✅)
make release        # Create full release: test + build-release + package
make build-release  # Build optimized release binary
make package        # Create distribution package (tar.gz)
```

### 🚫 NEVER Use These Commands Directly

**Cargo Commands (BLOCKED):**
- `cargo test` → Use `make test`
- `cargo build` → Use `make build`
- `cargo fmt` → Use `make fmt`
- `cargo clippy` → Use `make lint`
- `cargo doc` → Use `make docs`

**Git Commands (BLOCKED):**
- `git add . && git commit -m "msg" && git push` → Use `make git-force-all`
- `git status` → Use `make git-status`
- `git add -A` → Use `make git-add-all`

**Reason**: Make commands integrate validation, automation, and prevent direct usage of blocked operations.

---

## 📁 Project Structure

```
├── src/                           # Source code (Rust)
│   ├── main.rs                   # Application entry point
│   ├── lib.rs                    # Library exports
│   ├── core/                     # Core types and error handling
│   │   ├── mod.rs               # Core module exports
│   │   ├── error.rs             # Custom error types (thiserror)
│   │   └── types.rs             # Data structures (Embedding, CodeChunk, etc.)
│   ├── providers/               # Provider implementations
│   │   ├── mod.rs               # Provider traits
│   │   ├── embedding/           # Embedding providers (OpenAI, Ollama, Mock)
│   │   └── vector_store/        # Vector storage (Milvus, InMemory)
│   ├── services/                # Business logic
│   │   ├── mod.rs               # Service exports
│   │   ├── context.rs           # ContextService (embedding + storage orchestration)
│   │   ├── indexing.rs          # IndexingService (codebase processing)
│   │   └── search.rs            # SearchService (semantic search)
│   ├── server/                  # MCP protocol implementation
│   │   └── mod.rs               # MCP server with stdio transport
│   ├── registry/                # Provider registration system
│   ├── factory/                 # Service instantiation
│   └── config.rs                # Configuration handling
├── tests/                        # Test suites
│   ├── core_types.rs            # Core data structure tests (18 tests)
│   ├── services.rs              # Business logic tests (16 tests)
│   ├── mcp_protocol.rs          # MCP protocol tests (15 tests)
│   └── integration.rs           # End-to-end tests (11 tests)
├── docs/                        # Documentation (AUTOMATED)
│   ├── user-guide/              # User documentation
│   ├── developer/               # Developer guides
│   ├── architecture/            # Technical architecture
│   │   ├── ARCHITECTURE.md      # System architecture
│   │   ├── adr/                 # Architecture Decision Records
│   │   └── diagrams/            # PlantUML diagrams (auto-generated)
│   ├── operations/              # Deployment & operations
│   └── templates/               # Documentation templates
├── scripts/docs/                # Documentation automation
│   ├── generate-diagrams.sh     # PlantUML diagram generation
│   ├── validate-*.sh           # Various validation scripts
│   └── create-adr.sh           # ADR creation tool
└── Makefile                    # Build automation (PRIMARY INTERFACE)
```

---

## 🛠️ Tool Usage Guidelines

### ✅ ALLOWED: Direct Tool Usage

- **Read/Edit/Write**: For file operations
- **Grep**: For pattern matching and searching
- **Run Terminal**: For `make` commands and verified scripts

### ⚠️ CAUTION: MCP and External Tools

- **No untrusted MCP servers**: Only use approved, audited MCP servers
- **Verify before install**: Check source code and security
- **Local tools only**: Prefer local processing over external APIs

### 🚫 FORBIDDEN: Direct Cargo Usage

```
❌ cargo test        → ✅ make test
❌ cargo build       → ✅ make build
❌ cargo fmt         → ✅ make fmt
❌ cargo clippy      → ✅ make lint
❌ cargo doc         → ✅ make docs
```

---

## 🧪 Testing Strategy

### Test Categories & Expectations

| Test Suite | Location | Tests | Purpose | Pass Rate |
|------------|----------|-------|---------|-----------|
| **Core Types** | `tests/core_types.rs` | 18 | Data structure validation, serialization | 100% |
| **Services** | `tests/services.rs` | 16 | Business logic (Context, Index, Search) | 100% |
| **MCP Protocol** | `tests/mcp_protocol.rs` | 15 | Protocol compliance, message handling | 100% |
| **Integration** | `tests/integration.rs` | 11 | End-to-end functionality | 100% |
| **Total** | - | **60** | Full coverage | **100%** |

### Quality Gates (MANDATORY)

- **✅ All tests must pass**: `make test` = 0 failures (60/60 tests passing)
- **✅ No warnings**: `make lint` = clean clippy output (minor test warnings allowed)
- **✅ Format compliance**: `make fmt` = no changes
- **✅ Documentation sync**: `make validate` = all checks pass
- **⚠️ Security audit**: `make audit` = monitor known vulnerabilities (currently 3 in dependencies)
- **✅ Git operations**: Use `make git-force-all` for all commits

### Test Coverage Target

- **Current**: ~36% (acceptable for v0.0.2-alpha MVP)
- **Target**: >80% for production releases
- **Focus**: Core business logic, error paths, edge cases

---

## 🏗️ Architecture Patterns

### Provider Pattern (MANDATORY)

```rust
// CORRECT: Use traits for abstraction
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}

// CORRECT: Constructor injection
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextService {
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self { embedding_provider, vector_store_provider }
    }
}
```

### Async-First Design (MANDATORY)

```rust
// CORRECT: Async throughout
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // All operations are async
    let result = context_service.embed_text("query").await?;
    Ok(())
}
```

### Error Handling (MANDATORY)

```rust
// CORRECT: Custom error types with thiserror
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {source}")]
    Io { #[from] source: std::io::Error },

    #[error("Provider error: {message}")]
    Provider { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },
}

// CORRECT: Result type alias
pub type Result<T> = std::result::Result<T, Error>;
```

---

## 📚 Documentation Standards

### ADR (Architecture Decision Record) Process

```bash
# Create new ADR
make adr-new

# Follow template structure:
# - Status: Proposed/Accepted/Rejected/Deprecated/Superseded
# - Context: Problem description
# - Decision: What was chosen
# - Consequences: Positive/negative impacts
# - Alternatives: Other options considered
```

### Diagram Standards

- **PlantUML C4 Model**: Context → Container → Component → Code
- **Auto-generated**: Use `make diagrams`
- **Validation**: `make validate` checks syntax
- **Location**: `docs/architecture/diagrams/`

### Documentation Automation

```bash
make docs          # Generate all docs + diagrams + index
make validate      # Validate structure, links, sync
make docs-ci       # Full documentation CI pipeline
```

---

## 🔧 Development Rules

### Code Quality (MANDATORY)

1. **SOLID Principles**: Single responsibility, open/closed, etc.
2. **Async Throughout**: No blocking operations in async contexts
3. **Error Propagation**: Use `?` operator and custom error types
4. **Dependency Injection**: Constructor injection for testability
5. **Comprehensive Tests**: Every feature must have tests

### Git Workflow (MANDATORY - Always Force Commits)

```bash
# PRIMARY: Complete force workflow (recommended)
make git-force-all     # Add all + commit + push with force

# Individual steps (when needed)
make git-status        # Check repository status
make git-add-all       # Stage all changes
make git-commit-force  # Commit with timestamp (allow empty)
make git-push-force    # Push with force-with-lease/fallback to force

# Alternative method
make force-commit      # Use script-based force commit
```

**Force Commit Policy:**
- Always use `make git-force-all` for commits
- Commits include automatic timestamp: "Force commit: YYYY-MM-DD HH:MM:SS - Automated update"
- Push uses `--force-with-lease` first, `--force` as fallback
- No manual git commands allowed

### CI/CD Integration (MANDATORY)

```bash
# Local CI simulation (VALIDATED ✅)
make ci            # Full pipeline: clean + validate + test + build + docs

# Quality assurance (VALIDATED ✅)
make quality       # Complete quality: fmt + lint + test + audit + validate
make audit         # Security audit (⚠️ 3 known vulnerabilities in dependencies)
make coverage      # Generate coverage report (tarpaulin)

# Release process (VALIDATED ✅)
make release       # Production release: test + build-release + package
make build-release # Optimized release build
make package       # Create distribution package (tar.gz in dist/)
```

---

## 🚨 Critical Rules & Blockers

### 🚫 ABSOLUTELY FORBIDDEN

1. **Direct Cargo Commands**: Always use `make` equivalents (BLOCKED by hooks)
2. **Direct Git Commands**: Never use `git add/commit/push` directly (use `make git-force-all`)
3. **Mock Infrastructure**: Never mock databases, APIs, or external services
4. **Bypass Permissions**: Never use workarounds for permission issues
5. **Skip Tests**: All 60 tests must pass before commits
6. **Manual Documentation**: Always use automated documentation generation
7. **Bypass Make**: All operations must go through validated make commands

### ⚠️ HIGH RISK (Require Approval)

1. **New Dependencies**: Must be vetted for security and maintenance
2. **Breaking Changes**: Require ADR and impact analysis
3. **Configuration Changes**: Must update validation and tests
4. **External APIs**: Must have proper error handling and retries

### ✅ SAFE Operations

1. **Test Creation**: Add tests for new functionality
2. **Documentation Updates**: Use automated tools
3. **Code Refactoring**: Within existing patterns
4. **Bug Fixes**: Following existing error handling patterns

---

## 🎯 Task Execution Protocol

### For New Features

1. **Plan First**: Create ADR if architectural impact
2. **Test-Driven**: Write tests before implementation
3. **Incremental**: Small, testable changes
4. **Validate**: `make validate` after each change
5. **Document**: Update docs if user-facing changes

### For Bug Fixes

1. **Reproduce**: Confirm the bug exists
2. **Test First**: Write test that demonstrates the bug
3. **Fix**: Implement minimal fix
4. **Verify**: Ensure fix works and doesn't break existing tests
5. **Regression**: Add test to prevent future regression

### For Refactoring

1. **Preserve Behavior**: Ensure no functional changes
2. **Tests Pass**: All existing tests must continue passing
3. **Incremental**: Small changes with validation at each step
4. **Performance**: Verify no performance regressions

---

## 🔍 Verification Checklist

**Before marking any task complete:**

- [ ] `make test` passes all 60 tests (100% success rate)
- [ ] `make lint` has no critical warnings
- [ ] `make fmt` makes no changes
- [ ] `make validate` passes all validation checks
- [ ] `make docs` generates documentation without errors
- [ ] `make git-force-all` commits all changes successfully
- [ ] Code follows established patterns (Provider, Async-First, SOLID)
- [ ] Tests cover new functionality (add to existing test suites)
- [ ] Documentation is updated and validated
- [ ] No breaking changes to public APIs

---

## 📞 Getting Help

### Documentation Resources

- **Architecture**: `docs/architecture/ARCHITECTURE.md`
- **Contributing**: `docs/developer/CONTRIBUTING.md`
- **ADRs**: `docs/architecture/adr/`
- **Diagrams**: `docs/architecture/diagrams/generated/`

### Emergency Procedures

1. **If tests fail**: Run `make validate` to diagnose
2. **If build breaks**: Check for missing dependencies
3. **If docs fail**: Run `make clean-docs && make docs`
4. **If confused**: Re-read this CLAUDE.md file

### Communication

- **Issues**: Document in ADRs or commit messages
- **Decisions**: Use ADR process for architectural changes
- **Blockers**: Stop and ask user immediately

---

## ⚠️ Known Issues & Monitoring

### Security Vulnerabilities (TRACKED)

**Current Status:** 3 known vulnerabilities in dependencies (`make audit`)

| Vulnerability | Severity | Package | Status |
|---------------|----------|---------|--------|
| AES panic with overflow checking | High | `ring` 0.16.20/0.17.9 | Upgrade to >=0.17.12 |
| Infinite loop in rustls | High | `rustls` 0.20.9 | Upgrade to >=0.23.5 |
| Unmaintained packages | Medium | `ring` 0.16.20, `rustls-pemfile` 1.0.4 | Monitor for updates |

**Action Required:** Update dependencies when compatible versions are available.

### Project Validation Status (COMPLETED ✅)

**Comprehensive Make Command Audit:**
- **Core Commands:** 5/5 validated (build, test, clean, docs, validate)
- **Development Commands:** 4/4 validated (dev, fmt, lint, setup)
- **Documentation Commands:** 3/3 validated (adr-new, adr-list, diagrams)
- **Git Commands:** 6/6 validated (git-status, git-add-all, git-commit-force, git-push-force, git-force-all, force-commit)
- **Quality Commands:** 4/4 validated (quality, audit, bench, coverage)
- **Release Commands:** 3/3 validated (release, build-release, package)

**Test Coverage Verified:**
- Core Types: 18 tests ✅
- Services: 16 tests ✅
- MCP Protocol: 15 tests ✅
- Integration: 11 tests ✅
- **Total: 60 tests, 100% pass rate** ✅

**Security & Quality Gates:**
- Linting: Clean (minor test warnings allowed) ✅
- Formatting: Compliant ✅
- Documentation: Auto-generated and validated ✅
- CI Pipeline: Full pipeline working ✅
- Force Commits: Working and validated ✅

### Validation Results (VERIFIED ✅)

**All Make Commands Validated:**
- ✅ `make build` - Compiles successfully
- ✅ `make test` - 60/60 tests pass
- ✅ `make docs` - Generates documentation + diagrams
- ✅ `make validate` - All validation checks pass
- ✅ `make ci` - Full pipeline completes
- ✅ `make git-force-all` - Force commits work
- ✅ `make audit` - Security scan runs (finds known vulns)
- ✅ `make release` - Creates distribution packages

**Makefile Fixes Applied:**
- ✅ Fixed `package` command (was including itself in tar)
- ✅ Added complete git workflow commands
- ✅ Updated .PHONY declarations
- ✅ Verified all command dependencies

---

## 🎯 Success Criteria

**Task is complete when:**

- ✅ All tests pass (`make test` - 60/60 tests)
- ✅ Code quality verified (`make lint` - clippy clean)
- ✅ Documentation current (`make docs` - auto-generated)
- ✅ Validation clean (`make validate` - all checks pass)
- ✅ CI pipeline passes (`make ci` - full pipeline)
- ✅ Changes committed (`make git-force-all` - force push successful)
- ✅ User requirements satisfied
- ✅ No regressions introduced
- ✅ Security audit monitored (`make audit` - known vulns tracked)

**Remember**: Quality over speed. Automated validation catches issues before they become problems.
