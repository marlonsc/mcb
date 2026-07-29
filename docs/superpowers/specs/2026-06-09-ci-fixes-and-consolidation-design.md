# Design: CI Fixes & Workflow Consolidation

**Date:** 2026-06-09
**Branch:** `feat/v0.3.2-ci-gates`
**Related:** Epic `mcb-v5an` (v0.3.2 — CI/CD gates, compilation-cache efficiency & release reliability)

---

## 1. Context

CI run #1010 validated sccache optimizations with 14 jobs passing. Two failures remain:

| Job | Failure | Root Cause |
|-----|---------|------------|
| `Coverage` | `Test failed during run` | `cargo-tarpaulin` ptrace engine slows test `highlighting_50x_file_completes_under_2s` from 2000ms limit to 2304ms |
| `Test (windows-latest)` | Cancelled at 90min | Cold cache → ~20min compile + ~69min test execution exceeds timeout |

Additionally, the workflow compiles the Rust workspace **9 times independently** across jobs with zero artifact sharing.

---

## 2. Phase 1 — Immediate Fixes

### 2.1 Windows Test Timeout

**File:** `.github/workflows/ci.yml`
**Current:** `timeout-minutes: 90` (line ~325)
**Problem:** Measured cold-cache run: compile ~20min + 1715 tests ~69min = ~89min. Zero margin.
**Fix:** Increase to `timeout-minutes: 120`.
**Rationale:** The workflow already documented that 60min was insufficient. 90min was a guess. With 120min, cold-cache Windows runs finish safely, and warm-cache runs (target: <30min) are unaffected.

### 2.2 Coverage Timing-Sensitive Test

**File:** `crates/mcb-server/tests/unit/services/highlight_service_tests.rs:126`
**Current:**
```rust
assert!(elapsed.as_millis() < 2000, "took {}ms", elapsed.as_millis());
```
**Problem:** `cargo-tarpaulin` ptrace instrumentation adds ~15% overhead. Test panics at 2304ms.

**Approach A (recommended):** Switch tarpaulin to LLVM engine.
- `dispatch.mk:88` — change `cargo tarpaulin --out Lcov ...` to `cargo tarpaulin --engine llvm --out Lcov ...`
- LLVM engine has ~2-5% overhead vs ptrace's ~15%.
- May also improve cache reuse with `rust-cache` (though coverage still uses isolated key `ci-coverage`).
- Risk: LLVM engine requires `llvm-tools-preview` component. Must verify availability on CI runners.

**Approach B (fallback):** Skip the timing-sensitive test under tarpaulin.
- Add `#[cfg(not(tarpaulin_include))]` or `#[ignore = "timing-sensitive under instrumentation"]` to the test.
- Tarpaulin respects `#[ignore]` with `--ignored` flag (which we don't use), so the test is naturally excluded.
- Risk: Reduces coverage of highlight service by one test case.

**Decision:** Try Approach A (LLVM engine). If CI fails due to missing LLVM tools, fallback to Approach B in the same PR.

---

## 3. Phase 2 — Workflow Consolidation

### 3.1 Problem Statement

Current workflow compiles the workspace independently in 9 jobs:

| # | Job | Line | Command |
|---|-----|------|---------|
| 1 | `lint` | ~152 | `cargo clippy --all-targets` |
| 2 | `test-linux` startup | ~301 | `cargo test -p mcb --test integration` |
| 3 | `test-linux` full | ~305 | `cargo nextest run --workspace` |
| 4 | `test-cross` (×3 OS) | ~427 | `cargo nextest run --workspace` |
| 5 | `validate` | ~458 | `cargo run --package mcb -- validate .` |
| 6 | `golden-tests` | ~521 | `cargo test --workspace --tests golden` |
| 7 | `coverage` | ~587 | `cargo tarpaulin ...` |
| 8 | `release-build` (×3 OS) | ~682 | `cargo build --release` |
| 9 | `analyze` (CodeQL) | ~849 | `github/codeql-action/autobuild` |

No artifacts are shared. `rust-cache` mitigates this but keys are partitioned (`ci-ubuntu-stable`, `ci-cross-<os>`, `ci-coverage`, `ci-release-<os>`), so cold-cache runs rebuild everything.

### 3.2 Design: "Compile Once, Verify Many" for Ubuntu

Create a new job `build-ubuntu-debug` that compiles the workspace once and shares the `target/` directory.

```yaml
  build-ubuntu-debug:
    name: Build Ubuntu Debug
    runs-on: ubuntu-latest
    steps:
      - checkout
      - sccache-action
      - rust-toolchain
      - setup-ci.sh
      - rust-cache (key: ci-ubuntu-build-debug)
      - run: cargo build --workspace
      - run: cargo test --no-run --workspace
      - uses: actions/upload-artifact@v4
        with:
          name: target-debug-ubuntu
          path: target/
          retention-days: 1
```

Jobs `lint`, `test-linux`, `validate`, `golden-tests` become dependent on `build-ubuntu-debug` and download the artifact:

```yaml
  lint:
    needs: [build-ubuntu-debug, ...]
    steps:
      - checkout
      - download-artifact: target-debug-ubuntu
      - sccache-action
      - rust-toolchain
      - run: cargo clippy --all-targets
```

**Trade-offs:**
- **Pro:** Eliminates 4 redundant full-workspace compilations on Ubuntu.
- **Con:** `target/` artifact is large (2-5 GB compressed). Upload/download takes 1-3 minutes each.
- **Con:** Jobs become sequential (lint waits for build). Total wall-clock for Ubuntu pipeline may increase slightly if jobs were previously parallel.
- **Mitigation:** The `build-ubuntu-debug` job runs in parallel with `test-cross` (macOS/Windows) and `release-build`. Only the Ubuntu-dependent jobs are serialized.

### 3.3 Coverage — Keep Isolated

Coverage **must** remain isolated because `cargo-tarpaulin` sets `RUSTFLAGS="--cfg=tarpaulin -Clink-dead-code"`, which produces different artifacts than debug builds. Sharing `target/` with coverage would cause cache thrashing or wrong binaries.

With `--engine llvm`, the coverage build may be faster (~15-20% improvement expected), reducing the pain point.

### 3.4 Release Builds — No Change

Release builds (`cargo build --release`) produce different artifacts than debug builds. No sharing possible. Keep as-is.

### 3.5 Cross-Platform Tests — Cache Warmth

The `test-cross` jobs (macOS, Windows, beta) each have their own `rust-cache` key (`ci-cross-<os>`). After the first warm run, subsequent runs should be fast.

The Windows timeout fix (120min) ensures the initial cold-cache run completes and populates the cache.

### 3.6 Alternative: Sequential Ubuntu Mega-Job

Instead of artifact sharing, run `build`, `lint`, `test`, `validate`, `golden-tests` sequentially in a single Ubuntu job.

```yaml
  ubuntu-gates:
    name: Ubuntu Gates (build + lint + test + validate + golden)
    steps:
      - checkout + setup
      - run: cargo build --workspace
      - run: cargo test --no-run --workspace
      - run: make check WHAT=lint
      - run: make test SCOPE=startup THREADS=4
      - run: make test THREADS=4
      - run: cargo run --package mcb -- validate .
      - run: make test SCOPE=golden THREADS=2
```

**Trade-offs:**
- **Pro:** Zero artifact upload/download overhead. Simpler workflow.
- **Pro:** Incremental compilation between steps means each subsequent step is fast.
- **Con:** Loses parallelism — if lint fails fast, we don't know if tests pass until the job completes.
- **Con:** Harder to read CI status (one big job vs granular jobs).

**Decision:** Use artifact sharing (3.2) for better granularity and failure isolation. If artifact overhead proves problematic (>5min), revisit sequential mega-job.

---

## 4. Success Criteria

### Phase 1
- [ ] CI run passes with `Coverage` green (or advisory with clear justification)
- [ ] CI run passes with `Test (windows-latest)` completing under 120min
- [ ] No regressions in other jobs

### Phase 2
- [ ] Ubuntu jobs (`lint`, `test-linux`, `validate`, `golden-tests`) share a single `target/` artifact
- [ ] Total CI wall-clock for Ubuntu pipeline is ≤ previous parallel runtime + 5min overhead
- [ ] Cold-cache Ubuntu pipeline completes in <25min (vs ~15min lint + ~15min test + ~6min validate + ~6min golden = ~42min previously)

---

## 5. Test Plan

### Phase 1
1. Push fixes to `feat/v0.3.2-ci-gates`
2. Monitor CI run for Coverage and Windows completion
3. Verify sccache hit rates are reasonable (>50% on warm runs)

### Phase 2
1. Implement artifact sharing in feature branch
2. Run CI with cold cache → measure total time
3. Run CI with warm cache → measure total time
4. Compare against baseline (run #1010 or similar)
5. If total time regression >5min, revisit sequential mega-job approach

---

## 6. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| LLVM engine unavailable on CI | Fallback to skipping timing-sensitive test |
| `target/` artifact too large | Use `tar` with zstd compression; retention-days=1 |
| Artifact download slower than recompilation | Measure and fallback to sequential mega-job |
| Windows 120min still insufficient | Investigate test parallelization (`THREADS` env) or test splitting |
| Cache key collision | Keep `ci-coverage` and `ci-release-<os>` isolated |

---

## 7. Rollback Plan

Phase 1 fixes are reversible by reverting the specific commits.
Phase 2 can be reverted by removing `build-ubuntu-debug` job and restoring `needs` dependencies to original state.
