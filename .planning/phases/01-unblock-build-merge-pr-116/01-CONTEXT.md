# Phase 1: Unblock Build + Merge PR #116 - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix the two hard compile blockers (IndexingServiceInterface trait mismatch and linkme LTO stripping) and land PR #116 on `release/v0.3.1` with all tests passing. The workspace must compile, all 1700+ tests must pass, and the PR must be merged with review comments resolved.

</domain>

<decisions>
## Implementation Decisions

### LTO Fix
- **D-01:** Remove `[profile.release]` block entirely from `.cargo/config.toml` — let `Cargo.toml`'s `lto = "thin"` be the single source of truth for release profile settings
- **D-02:** Root cause: `.cargo/config.toml` overrides `Cargo.toml` per Cargo precedence rules; the `lto = "thin"` already in `Cargo.toml` was being silently overridden by `lto = true` in config.toml
- **D-03:** `codegen-units = 1` stays in `Cargo.toml` — no change needed there

### PR #116 Merge Approach
- **D-04:** Merge commit (`--no-ff`) — preserve the full individual commit history from the refactoring
- **D-05:** Self-approve from `marlonsc` admin account to unblock `REVIEW_REQUIRED` status
- **D-06:** AI reviewer comments (Copilot, Qodo, Gemini) are informational only — they posted `COMMENTED` not `APPROVED`, and CodeRabbit skipped (298 > 150 file limit)

### Test Baseline
- **D-07:** All tests green required before merge — zero failures, no exceptions
- **D-08:** This is stricter than the roadmap's Phase 1 criterion ("tests may fail on content, but compilation succeeds") — user decision overrides roadmap
- **D-09:** If specific tests require content updates due to the 298-file refactoring, those fixes are in-scope for Phase 1

### Claude's Discretion
- Order of operations for the fixes (compile fix vs LTO fix vs test fixes)
- How to batch test fixes (per-crate vs per-category)
- Whether to address AI reviewer comments before or after test fixes

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Build blockers
- `.cargo/config.toml` — Contains the `lto = true` override to remove
- `Cargo.toml` — Contains the correct `lto = "thin"` with `codegen-units = 1`
- `.planning/codebase/CONCERNS.md` §Compilation Errors — IndexingServiceInterface trait mismatch details and fix approach

### Architecture
- `docs/architecture/CLEAN_ARCHITECTURE.md` — Layer rules that PR #116 refactoring follows
- `docs/architecture/ARCHITECTURE_BOUNDARIES.md` — Dependency rules and violation codes

### DI / linkme
- `crates/mcb/src/main.rs` — `extern crate mcb_providers` force-link (must remain)
- `crates/mcb-infrastructure/src/di/bootstrap.rs` — AppContext composition root

### PR #116
- GitHub PR #116 (`gh pr view 116`) — 298-file refactoring: SeaORM shared CRUD macros, tool router decomposition, validator parallelization, build-script embedded rules, test migration, stdio rewrite, legacy modules deleted

</canonical_refs>

<code_context>
## Existing Code Insights

### Current Build State
- `cargo check --workspace` already passes — IndexingServiceInterface fix landed in `d2937f27`
- `cargo test --workspace --no-run` compiles all test binaries successfully
- Release build (`cargo build --release`) status pending verification with LTO fix

### Reusable Assets
- `Cargo.toml` already has correct `lto = "thin"` with explanatory comment — just need to unblock it
- CI pipeline in `.github/workflows/ci.yml` has `run_simplified` path for draft PRs

### Established Patterns
- Conventional Commits format used throughout (`feat:`, `fix:`, `refactor:`)
- `make test`, `make lint`, `make validate` as CI gates
- `make check` = full gate (fmt + lint + test + validate)

### Integration Points
- `.cargo/config.toml` — profile.release block removal
- GitHub branch protection ruleset (ID 12225448) — requires 1 approving review

</code_context>

<specifics>
## Specific Ideas

No specific requirements — straightforward build fix and merge workflow.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-unblock-build-merge-pr-116*
*Context gathered: 2026-03-23*
