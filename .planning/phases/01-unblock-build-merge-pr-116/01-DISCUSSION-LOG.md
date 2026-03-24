# Phase 1: Unblock Build + Merge PR #116 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 01-unblock-build-merge-pr-116
**Areas discussed:** LTO fix strategy, PR #116 merge approach, Test baseline for merge

---

## LTO Fix Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| lto = "thin" in config.toml | Change the single line. Both config files agree, self-documenting, ~90% of fat LTO performance, safe with linkme. | |
| Remove [profile.release] from config.toml | Delete the release profile block entirely. Cargo.toml's lto = "thin" takes effect. Slightly less self-documenting. | ✓ |
| lto = "off" | Disable LTO entirely. Fastest builds, zero risk, but larger binary and no cross-crate optimization. | |

**User's choice:** Remove [profile.release] from config.toml
**Notes:** Let Cargo.toml be the single source of truth for release profile settings.

---

## PR #116 Merge Method

| Option | Description | Selected |
|--------|-------------|----------|
| Squash merge | 100 WIP commits → 1 clean Conventional Commit. PR description becomes commit body. | |
| Merge commit (--no-ff) | Preserves all 100 individual commits. Full blame chain intact. | ✓ |
| Rebase merge | Linear history with all commits retained. No merge commit but 100 entries on main. | |

**User's choice:** Merge commit (--no-ff)
**Notes:** Preserve the full individual commit history from the refactoring.

## PR #116 Review Gate Unblock

| Option | Description | Selected |
|--------|-------------|----------|
| Approve from marlonsc account | Self-approve using the admin/owner account. Simplest unblock for solo developer repo. | ✓ |
| Edit ruleset to add bypass actor | Add marlonsc as bypass actor in branch protection. Permanent fix. | |
| Temporarily disable ruleset | Remove protection, merge, re-enable. Quick but leaves a gap. | |

**User's choice:** Approve from marlonsc account
**Notes:** None.

---

## Test Baseline for Merge

| Option | Description | Selected |
|--------|-------------|----------|
| Compile-only gate | All tests must compile. Content failures accepted and tracked. Matches roadmap Phase 1 criterion. | |
| Percentage threshold (>=85%) | Set minimum pass rate. Creates documented baseline. Mirrors v0.3.0 precedent (91%). | |
| All green required | Zero failures before merge. Forces all content fixes in Phase 1. | ✓ |

**User's choice:** All green required
**Notes:** User chose stricter threshold than roadmap suggested. All 1700+ tests must pass before merge.

---

## Claude's Discretion

- Order of operations for fixes
- How to batch test fixes
- Whether to address AI reviewer comments before or after test fixes

## Deferred Ideas

None — discussion stayed within phase scope.
