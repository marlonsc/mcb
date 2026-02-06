# Continuation Prompt — Quality Refactoring Beads Issue Creation

> **Copy everything below the line and paste into the next session.**

---

## Context

We are on branch `release/v0.2.0` of the MCB project. Compilation is clean (`cargo check --workspace` passes).

We created a unified quality refactoring plan at `.planning/QUALITY-REFACTOR-PLAN.md` with 4 waves, 25 tasks, ~7 hours of work.

We then created Beads issues (`bd create`) for each task in the plan. The Beads CLI is `bd` at `/home/marlonsc/.local/bin/bd`.

### Issues Already Created

**Wave 1 — Strong Typing Completion (6 issues):**

-   `mcb-4t5` — Strong Typing: Fix MemoryRepository trait to use value objects (P1)
-   `mcb-a9b` — Strong Typing: Update MemoryRepository SQLite implementation (P1)
-   `mcb-9u9` — Strong Typing: Migrate server argument structs to value objects (P1)
-   `mcb-2pt` — Strong Typing: Migrate transport types to value objects (P1)
-   `mcb-bux` — Strong Typing: Migrate hook types to value objects (P1)
-   `mcb-ouf` — Strong Typing: Remove unnecessary String→ID conversions in handlers (P1)

**Wave 2 — Architecture Violations (4 issues):**

-   `mcb-s35` — CA Violation: Inject HighlightService via DI instead of direct creation (P1)
-   `mcb-95d` — CA Violation: Wire HighlightService in DI catalog (P1)
-   `mcb-xme` — CA Violation: FileTreeNode immutability CA005 (P1)
-   `mcb-o1z` — Verify zero CA violations with make validate (P1)

**Wave 3 — Magic Strings → Constants (9 issues):**

-   `mcb-hih` — Constants: Create vector store field constants in mcb-domain (P2)
-   `mcb-b3l` — Constants: Migrate Milvus provider magic strings ~40 (P2)
-   `mcb-eq3` — Constants: Migrate EdgeVec provider magic strings ~20 (P2)
-   `mcb-bob` — Constants: Migrate Pinecone provider magic strings ~10 (P2)
-   `mcb-d0n` — Constants: Migrate Qdrant provider magic strings ~5 (P2)
-   `mcb-h5w` — Constants: Migrate Encrypted provider magic strings ~3 (P2)
-   `mcb-kz8` — Constants: Migrate ContextService metadata strings (P2)
-   `mcb-gei` — Constants: Migrate server handler magic strings (P2)
-   `mcb-c3k` — Constants: Consolidate infrastructure metadata constants to domain layer (P2)

**Wave 4 — Documentation & Cleanup (3 issues):**

-   `mcb-7h1` — Docs: Add module documentation to value_objects/browse and ports/services (P3)
-   `mcb-7u3` — Docs: Add enum documentation to ConfigError (P3)
-   `mcb-83t` — Cleanup: Triage and reduce compiler warnings to <100 (P3)

### What Still Needs To Be Done

1.  **Add dependencies between issues** — Use `bd edit <id> --deps <dep-id>` to wire:
   -   `mcb-a9b` depends on `mcb-4t5` (impl depends on trait change)
   -   `mcb-95d` depends on `mcb-s35` (wiring depends on DI refactor)
   -   `mcb-o1z` depends on `mcb-s35`, `mcb-95d`, `mcb-xme` (verification after all fixes)
   -   All Wave 3 provider issues (`mcb-b3l` through `mcb-gei`) depend on `mcb-hih` (constants must exist first)

2.  **Update `.planning/QUALITY-REFACTOR-PLAN.md`** with the Beads issue IDs next to each task

3.  **Close `mcb-037`** (P0 Fix Compilation) — compilation is already clean, this can be closed: `bd close mcb-037`

4.  **Run `make validate`** to establish a baseline of current architecture violations (it was timing out before — try with longer timeout or run `cargo test -p mcb-validate` directly)

5.  **Start executing Wave 1 tasks** (strongest impact, P1 priority)
