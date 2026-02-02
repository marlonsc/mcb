# Roadmap: MCB v0.2.0

**Created:** 2026-01-31
**Target:** Git-Aware Indexing + Session Memory + Advanced Browser
**Phases:** 10
**Requirements:** 38 mapped

## Phase Overview

| # | Phase | Goal | Requirements | Success Criteria |
|---|-------|------|--------------|------------------|
| 1 | Architecture Cleanup | Break dependency cycles, create shared crates | ARCH-01→05 | 5 |
| 2 | Git Foundation | Basic git repository indexing | GIT-01→03, GIT-07, GIT-08, GIT-11 | 6 |
| 3 | Git Advanced | Submodules and project detection | GIT-04, GIT-05 | 4 |
| 4 | Git Analysis | Impact analysis and branch comparison | GIT-06, GIT-09, GIT-10 | 4 |
| 5 | Memory Foundation | SQLite storage and basic tools | MEM-01, MEM-02, MEM-07, MEM-10 | 5 |
| 6 | Memory Search | Hybrid search and progressive disclosure | MEM-03, MEM-04, MEM-08, MEM-09 | 5 |
| 7 | Memory Integration | Context injection and git tagging | MEM-05, MEM-06, MEM-11 | 4 |
| 8 | Browser Foundation | Tree view and syntax highlighting | BROWSE-01, BROWSE-02, BROWSE-06 | 4 |
| 9 | Browser Polish | Search integration and keyboard nav | BROWSE-03→05 | 4 |
| 10 | Production Hardening | Health checks, metrics, validation | PROD-01→05 | 5 |

---

## Phase 1: Architecture Cleanup

**Goal:** Break dependency cycles and create shared infrastructure crates

**Requirements:**
- ARCH-01: Break mcb-infrastructure → mcb-validate dependency cycle
- ARCH-02: Create mcb-language-support crate
- ARCH-03: Create mcb-ast-utils crate
- ARCH-04: Define ValidationServiceInterface port
- ARCH-05: Define MetricsProvider port

**Success Criteria:**
1. mcb-validate is dev-only dependency (not required for production build)
2. Language detection unified in single crate (13 languages)
3. AST traversal utilities shared between mcb-validate and mcb-providers
4. ValidationServiceInterface trait in mcb-domain/src/ports/services.rs
5. MetricsProvider trait in mcb-domain/src/ports/providers/

**Estimated Effort:** ~1500 LOC

---

## Phase 2: Git Foundation

**Goal:** Basic git repository indexing with branch awareness

**Requirements:**
- GIT-01: Repository ID via root commit hash
- GIT-02: Multi-branch indexing
- GIT-03: Commit history indexing
- GIT-07: MCP tool: index_git_repository
- GIT-08: MCP tool: search_branch
- GIT-11: MCP tool: list_repositories

**Success Criteria:**
1. Repository uniquely identified by root commit hash
2. Can index main, HEAD, and current branch
3. Last 50 commits indexed with metadata
4. index_git_repository tool registers and works
5. search_branch tool filters by branch
6. list_repositories returns indexed repos

**Technical Details:**
- New dependency: git2 (libgit2 bindings)
- New files: ~8 source files
- Estimated LOC: ~1200

---

## Phase 3: Git Advanced

**Goal:** Submodule and monorepo support

**Requirements:**
- GIT-04: Submodule support with recursive indexing
- GIT-05: Project detection (Cargo, npm, Python, Go, Maven)

**Success Criteria:**
1. Submodules indexed as separate projects with parent link
2. Cargo.toml detected → Rust project markers applied
3. package.json detected → Node.js project markers applied
4. requirements.txt/pyproject.toml → Python project markers
5. go.mod → Go project markers
6. pom.xml → Maven project markers

**Plans:** 4 plans

Plans:
- [ ] 03-01-PLAN.md — Project detection infrastructure (5 detectors with linkme)
- [ ] 03-02-PLAN.md — Submodule traversal service (BFS with depth limit)
- [ ] 03-03-PLAN.md — File hash store for incremental indexing (SQLite)
- [ ] 03-04-PLAN.md — Integration (GitIndexingService + tool interface)

**Estimated Effort:** ~800 LOC

---

## Phase 4: Git Analysis

**Goal:** Change impact analysis between git refs

**Requirements:**
- GIT-06: Change impact analysis between refs
- GIT-09: MCP tool: compare_branches
- GIT-10: MCP tool: analyze_impact

**Success Criteria:**
1. Can analyze semantic impact of changes between two refs
2. compare_branches shows diff with semantic annotations
3. analyze_impact identifies affected code paths
4. Impact analysis completes in <5s for typical PR

**Estimated Effort:** ~1000 LOC

---

## Phase 5: Memory Foundation

**Goal:** SQLite-based observation storage

**Requirements:**
- MEM-01: SQLite observation storage with metadata
- MEM-02: Session summaries with auto-generation
- MEM-07: MCP tool: search (memory index)
- MEM-10: MCP tool: store_observation

**Success Criteria:**
1. Observations persisted to ~/.mcb/memory.db
2. Session summaries auto-generated on session end
3. search tool returns memory index (token-efficient)
4. store_observation persists with metadata

**Technical Details:**
- New dependency: sqlx (SQLite)
- Database: ~/.mcb/memory.db
- Estimated LOC: ~1200

---

## Phase 6: Memory Search

**Goal:** Hybrid search with progressive disclosure

**Requirements:**
- MEM-03: BM25 + vector hybrid search
- MEM-04: Progressive disclosure (3-layer workflow)
- MEM-08: MCP tool: timeline
- MEM-09: MCP tool: get_observations

**Success Criteria:**
1. Hybrid search combines BM25 and vector similarity
2. 3-layer workflow: search → timeline → get_observations
3. timeline returns chronological context around anchor
4. get_observations fetches full details for filtered IDs
5. Token savings: 10x vs fetching all details

**Plans:** 3 plans

Plans:
- [ ] 06-01-PLAN.md — FTS5 and Hybrid Search Infrastructure
- [ ] 06-02-PLAN.md — Hybrid Search Implementation
- [ ] 06-03-PLAN.md — Progressive Disclosure Tools

**Estimated Effort:** ~1000 LOC

---

## Phase 7: Memory Integration

**Goal:** Context injection and git tagging

**Requirements:**
- MEM-05: Context injection for SessionStart hooks
- MEM-06: Git-tagged observations
- MEM-11: MCP tool: inject_context

**Success Criteria:**
1. inject_context generates context for SessionStart
2. Observations tagged with branch and commit
3. Can filter memory by git context
4. SessionStart hook integration documented

**Estimated Effort:** ~600 LOC

---

## Phase 8: Browser Foundation

**Goal:** Tree view navigation and syntax highlighting

**Requirements:**
- BROWSE-01: Tree view with collapsible directories
- BROWSE-02: Tree-sitter syntax highlighting with chunk markers
- BROWSE-06: Dark mode with CSS variables

**Success Criteria:**
1. Directory tree renders with collapse/expand
2. Code displayed with syntax highlighting
3. Chunk boundaries visible in display
4. Dark mode toggle works
5. CSS variables enable theming

**Technical Details:**
- Foundation: Existing v0.1.2 browse UI
- New dependency: Alpine.js (CDN)
- Estimated LOC: ~800

---

## Phase 9: Browser Polish

**Goal:** Search integration and keyboard navigation

**Requirements:**
- BROWSE-03: Inline search result highlighting
- BROWSE-04: Keyboard navigation (Vim-like)
- BROWSE-05: Real-time SSE updates

**Success Criteria:**
1. Search results highlighted inline with similarity scores
2. j/k scrolls, Enter opens files
3. SSE shows indexing progress in real-time
4. Updates appear without page refresh

**Estimated Effort:** ~700 LOC

---

## Phase 10: Production Hardening

**Goal:** Production readiness features

**Requirements:**
- PROD-01: Provider health check endpoints
- PROD-02: Configuration schema validation
- PROD-03: Metrics export to Prometheus
- PROD-04: Config migration guide
- PROD-05: Fix Windows file locking

**Success Criteria:**
1. GET /admin/health/providers returns provider status
2. Invalid config fails fast with clear errors
3. Prometheus metrics at /metrics endpoint
4. Migration guide for v0.1.x → v0.2.0 config
5. Windows file locking uses LockFileEx

**Estimated Effort:** ~800 LOC

---

## Total Estimated Effort

| Phase | LOC | Cumulative |
|-------|-----|------------|
| 1 | 1500 | 1500 |
| 2 | 1200 | 2700 |
| 3 | 800 | 3500 |
| 4 | 1000 | 4500 |
| 5 | 1200 | 5700 |
| 6 | 1000 | 6700 |
| 7 | 600 | 7300 |
| 8 | 800 | 8100 |
| 9 | 700 | 8800 |
| 10 | 800 | 9600 |

**Total:** ~9600 LOC for v0.2.0

---

## Dependencies

```
Phase 1 (Architecture) → unlocks all others
    ↓
Phase 2 (Git Foundation) → Phase 3, 4
    ↓
Phase 3 (Git Advanced) → Phase 4
    ↓
Phase 4 (Git Analysis)

Phase 5 (Memory Foundation) → Phase 6, 7
    ↓
Phase 6 (Memory Search) → Phase 7
    ↓
Phase 7 (Memory Integration)

Phase 8 (Browser Foundation) → Phase 9
    ↓
Phase 9 (Browser Polish)

Phase 10 (Production) — independent, can run parallel
```

---
*Roadmap created: 2026-01-31*
*Last updated: 2026-01-31 after initial definition*
