# Requirements: MCB v0.2.0

**Defined:** 2026-01-31
**Core Value:** Semantic code search with git awareness and session memory

## v0.2.0 Requirements

### Architecture Cleanup (ARCH)

- [ ] **ARCH-01**: Break mcb-infrastructure → mcb-validate dependency cycle
- [ ] **ARCH-02**: Create mcb-language-support crate for unified language handling
- [ ] **ARCH-03**: Create mcb-ast-utils crate for shared AST abstractions
- [ ] **ARCH-04**: Define ValidationServiceInterface port in mcb-domain
- [ ] **ARCH-05**: Define MetricsProvider port in mcb-domain

### Git Integration (GIT)

- [ ] **GIT-01**: Repository ID via root commit hash for portable identification
- [ ] **GIT-02**: Multi-branch indexing (main, HEAD, current branch)
- [ ] **GIT-03**: Commit history indexing (last 50 commits default)
- [ ] **GIT-04**: Submodule support with recursive indexing
- [ ] **GIT-05**: Project detection (Cargo, npm, Python, Go, Maven)
- [ ] **GIT-06**: Change impact analysis between refs
- [ ] **GIT-07**: MCP tool: index_git_repository
- [ ] **GIT-08**: MCP tool: search_branch
- [ ] **GIT-09**: MCP tool: compare_branches
- [ ] **GIT-10**: MCP tool: analyze_impact
- [ ] **GIT-11**: MCP tool: list_repositories

### Session Memory (MEM)

- [ ] **MEM-01**: SQLite observation storage with metadata
- [ ] **MEM-02**: Session summaries with auto-generation
- [ ] **MEM-03**: BM25 + vector hybrid search over memory
- [ ] **MEM-04**: Progressive disclosure (3-layer workflow)
- [ ] **MEM-05**: Context injection for SessionStart hooks
- [ ] **MEM-06**: Git-tagged observations (branch, commit)
- [ ] **MEM-07**: MCP tool: search (memory index)
- [ ] **MEM-08**: MCP tool: timeline (chronological context)
- [ ] **MEM-09**: MCP tool: get_observations (full details)
- [ ] **MEM-10**: MCP tool: store_observation
- [ ] **MEM-11**: MCP tool: inject_context

### Code Browser (BROWSE)

- [ ] **BROWSE-01**: Tree view navigation with collapsible directories
- [ ] **BROWSE-02**: Tree-sitter syntax highlighting with chunk markers
- [ ] **BROWSE-03**: Inline search result highlighting with similarity scores
- [ ] **BROWSE-04**: Keyboard navigation (Vim-like: j/k scroll, Enter open)
- [ ] **BROWSE-05**: Real-time SSE updates during indexing
- [ ] **BROWSE-06**: Dark mode with CSS variable theming

### Production Readiness (PROD)

- [ ] **PROD-01**: Provider health check endpoints
- [ ] **PROD-02**: Configuration schema validation
- [ ] **PROD-03**: Metrics export to Prometheus
- [ ] **PROD-04**: Config migration guide between versions
- [ ] **PROD-05**: Fix Windows file locking edge cases

## v0.3.0 Requirements (Deferred)

### Code Intelligence

- **INTEL-01**: Symbol extraction and indexing
- **INTEL-02**: Cross-referencing and usage graph
- **INTEL-03**: Call graph analysis
- **INTEL-04**: Dependency mapping
- **INTEL-05**: Code similarity detection
- **INTEL-06**: Refactoring suggestions

## Out of Scope

| Feature | Reason |
|---------|--------|
| Multi-tenancy | Enterprise feature, defer to v0.4.0 |
| SSO/SAML | Enterprise feature, defer to v0.4.0 |
| Real-time collaboration | Complexity, single-user focus |
| Mobile app | Web-first strategy |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| ARCH-01 | Phase 1 | Pending |
| ARCH-02 | Phase 1 | Pending |
| ARCH-03 | Phase 1 | Pending |
| ARCH-04 | Phase 1 | Pending |
| ARCH-05 | Phase 1 | Pending |
| GIT-01 | Phase 2 | Pending |
| GIT-02 | Phase 2 | Pending |
| GIT-03 | Phase 2 | Pending |
| GIT-04 | Phase 3 | Pending |
| GIT-05 | Phase 3 | Pending |
| GIT-06 | Phase 4 | Pending |
| GIT-07 | Phase 2 | Pending |
| GIT-08 | Phase 2 | Pending |
| GIT-09 | Phase 4 | Pending |
| GIT-10 | Phase 4 | Pending |
| GIT-11 | Phase 2 | Pending |
| MEM-01 | Phase 5 | Pending |
| MEM-02 | Phase 5 | Pending |
| MEM-03 | Phase 6 | Pending |
| MEM-04 | Phase 6 | Pending |
| MEM-05 | Phase 7 | Pending |
| MEM-06 | Phase 7 | Pending |
| MEM-07 | Phase 5 | Pending |
| MEM-08 | Phase 6 | Pending |
| MEM-09 | Phase 6 | Pending |
| MEM-10 | Phase 5 | Pending |
| MEM-11 | Phase 7 | Pending |
| BROWSE-01 | Phase 8 | Pending |
| BROWSE-02 | Phase 8 | Pending |
| BROWSE-03 | Phase 9 | Pending |
| BROWSE-04 | Phase 9 | Pending |
| BROWSE-05 | Phase 9 | Pending |
| BROWSE-06 | Phase 8 | Pending |
| PROD-01 | Phase 10 | Pending |
| PROD-02 | Phase 10 | Pending |
| PROD-03 | Phase 10 | Pending |
| PROD-04 | Phase 10 | Pending |
| PROD-05 | Phase 10 | Pending |

**Coverage:**
- v0.2.0 requirements: 38 total
- Mapped to phases: 38
- Unmapped: 0 ✓

---
*Requirements defined: 2026-01-31*
*Last updated: 2026-01-31 after initial definition*
