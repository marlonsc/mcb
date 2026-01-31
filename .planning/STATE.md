# Project State: MCB v0.2.0

**Updated:** 2026-01-31T20:50:00Z
**Branch:** release/v0.1.5 (will create feature/v0.2.0)

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-31)

**Core value:** Semantic code search with git awareness and session memory
**Current focus:** Phase 1 - Architecture Cleanup

## State

- Done:
  - [x] v0.1.5 release completed
  - [x] Codebase mapping (.planning/codebase/)
  - [x] PROJECT.md initialized
  - [x] REQUIREMENTS.md defined (38 requirements)
  - [x] ROADMAP.md created (10 phases)
  - [x] Beads tasks created for tracking
- Now: [→] Phase 1: Architecture Cleanup
- Next: Phase 2: Git Foundation
- Remaining:
  - [ ] Phase 1: Architecture Cleanup (ARCH-01→05)
  - [ ] Phase 2: Git Foundation (GIT-01→03, 07, 08, 11)
  - [ ] Phase 3: Git Advanced (GIT-04, 05)
  - [ ] Phase 4: Git Analysis (GIT-06, 09, 10)
  - [ ] Phase 5: Memory Foundation (MEM-01, 02, 07, 10)
  - [ ] Phase 6: Memory Search (MEM-03, 04, 08, 09)
  - [ ] Phase 7: Memory Integration (MEM-05, 06, 11)
  - [ ] Phase 8: Browser Foundation (BROWSE-01, 02, 06)
  - [ ] Phase 9: Browser Polish (BROWSE-03→05)
  - [ ] Phase 10: Production Hardening (PROD-01→05)

## Key Metrics

| Metric | v0.1.5 | v0.2.0 Target |
|--------|--------|---------------|
| Tests | 1670+ | 2500+ |
| Embedding Providers | 7 | 7 |
| Vector Stores | 8 | 8 |
| MCP Tools | 5 | 16 (+11) |
| Languages | 14 | 14 |
| LOC Added | — | ~9600 |

## Open Questions

- UNCONFIRMED: Should git2 be optional feature flag or required?
- UNCONFIRMED: SQLite vs PostgreSQL for session memory?
- UNCONFIRMED: Alpine.js vs vanilla JS for browser UI?

## Working Set

- **Branch:** `release/v0.1.5` → will create `feature/v0.2.0-arch`
- **Key files for Phase 1:**
  - `crates/mcb-domain/src/ports/services.rs` (ValidationServiceInterface)
  - `crates/mcb-domain/src/ports/providers/` (MetricsProvider)
  - `crates/mcb-language-support/` (new crate)
  - `crates/mcb-ast-utils/` (new crate)
- **Validation:** `make validate` → 0 violations
- **Tests:** `make test` → 1670+ passing

---
*State initialized: 2026-01-31*
