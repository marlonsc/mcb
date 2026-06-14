# CI Fixes & Consolidation Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement task-by-task.

**Goal:** Fix Coverage and Windows CI failures, then consolidate redundant Ubuntu compilations into a single shared build.

**Architecture:** Phase 1 applies immediate fixes (timeout, tarpaulin engine). Phase 2 introduces a `build-ubuntu-debug` job that compiles once and uploads `target/` as artifact for reuse by `lint`, `test-linux`, `validate`, `golden-tests`.

**Tech Stack:** GitHub Actions, cargo-tarpaulin, sccache, rust-cache

---

## Task 1: Fix Windows Test Timeout

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Increase timeout from 90 to 120 minutes**

Find the `test-cross` job `timeout-minutes: 90` and change to `120`.

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "fix(ci): increase windows test timeout to 120min

Cold cache + 1715 tests exceeds 90min. Measured: ~20min compile + ~69min tests.
120min provides safe margin for cold-cache seeding runs."
```

---

## Task 2: Fix Coverage Timing-Sensitive Test

**Files:**
- Modify: `makefiles/dispatch.mk:88`
- Test: Run `make check WHAT=coverage` locally or monitor CI

- [ ] **Step 1: Switch tarpaulin to LLVM engine**

In `makefiles/dispatch.mk`, change the `coverage)` case:

```makefile
coverage) cargo tarpaulin --engine llvm --out Lcov --output-dir coverage --exclude-files 'crates/*/tests/integration/*' --exclude-files 'crates/*/tests/admin/*' --timeout 300 ;;
```

- [ ] **Step 2: Verify `llvm-tools-preview` component**

Check `rust-toolchain.toml` or CI setup for component availability. If missing, add to CI workflow's `dtolnay/rust-toolchain` step for the coverage job:
```yaml
components: rustfmt, clippy, llvm-tools-preview
```

- [ ] **Step 3: Commit**

```bash
git add makefiles/dispatch.mk .github/workflows/ci.yml
git commit -m "fix(ci): use tarpaulin LLVM engine to reduce instrumentation overhead

The ptrace engine slows timing-sensitive tests (highlight_service
took 2304ms vs 2000ms limit). LLVM engine has ~2-5% overhead vs
~15% for ptrace. Adds llvm-tools-preview component to coverage job."
```

---

## Task 3: Create build-ubuntu-debug Job

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add `build-ubuntu-debug` job before existing Ubuntu jobs**

Insert after the `classify` job definition and before `lint`:

```yaml
  build-ubuntu-debug:
    name: Build Ubuntu Debug
    needs: [changes, classify]
    if: |
      (needs.changes.outputs.src == 'true' || needs.changes.outputs.ci == 'true') &&
      ((needs.classify.outputs.run_full == 'true' ||
      github.event_name == 'push' ||
      github.event_name == 'workflow_dispatch') ||
      needs.classify.outputs.run_simplified == 'true')
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
        with:
          persist-credentials: false
          submodules: recursive
      - uses: mozilla-actions/sccache-action@v0.0.9
      - uses: dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1
        with:
          toolchain: stable
      - run: bash .github/setup-ci.sh
      - uses: Swatinem/rust-cache@779680da715d629ac1d338a641029a2f4372abb5 # v2.8.2
        with:
          shared-key: ci-ubuntu-build-debug
          save-if: true
          cache-on-failure: true
      - run: cargo build --workspace
      - run: cargo test --no-run --workspace
      - name: Compress target directory
        run: tar -cJf target-debug.tar.xz target/
      - uses: actions/upload-artifact@v4
        with:
          name: target-debug-ubuntu
          path: target-debug.tar.xz
          retention-days: 1
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): add build-ubuntu-debug job for artifact sharing

Compiles workspace in debug mode once and uploads target/ as artifact.
Dependent jobs (lint, test, validate, golden) will download and reuse,
eliminating 4 redundant compilations on Ubuntu."
```

---

## Task 4: Modify lint Job to Reuse Build Artifact

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add `build-ubuntu-debug` to needs and download artifact**

Change `lint` job:
- `needs:` add `build-ubuntu-debug`
- After checkout, add artifact download and extraction:

```yaml
  lint:
    name: Lint (Rust 2024)
    needs: [changes, classify, build-ubuntu-debug]
    ...
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
        with:
          persist-credentials: false
          submodules: recursive
      - uses: actions/download-artifact@v4
        with:
          name: target-debug-ubuntu
      - run: tar -xJf target-debug.tar.xz
      - uses: mozilla-actions/sccache-action@v0.0.9
      - uses: dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1
        with:
          toolchain: stable
          components: rustfmt, clippy
      - run: bash .github/setup-ci.sh
      ...
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): lint job reuses build-ubuntu-debug artifact"
```

---

## Task 5: Modify test-linux Job to Reuse Build Artifact

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add artifact download after checkout**

Change `test-linux` job `needs:` to include `build-ubuntu-debug`.
Add download + extract steps after checkout.

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): test-linux job reuses build-ubuntu-debug artifact"
```

---

## Task 6: Modify validate Job to Reuse Build Artifact

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add artifact download after checkout**

Change `validate` job `needs:` to include `build-ubuntu-debug`.
Add download + extract steps after checkout.

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): validate job reuses build-ubuntu-debug artifact"
```

---

## Task 7: Modify golden-tests Job to Reuse Build Artifact

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add artifact download after checkout**

Change `golden-tests` job `needs:` to include `build-ubuntu-debug`.
Add download + extract steps after checkout.

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): golden-tests job reuses build-ubuntu-debug artifact"
```

---

## Task 8: Push and Validate

- [ ] **Step 1: Push all commits**

```bash
git push origin feat/v0.3.2-ci-gates
```

- [ ] **Step 2: Monitor CI run**

Watch for:
- Coverage passes (or fails only on pre-existing issues unrelated to our changes)
- Windows test completes under 120min
- Ubuntu jobs (`lint`, `test-linux`, `validate`, `golden-tests`) complete successfully
- Artifact upload/download works (check job logs for `actions/upload-artifact` and `actions/download-artifact`)

- [ ] **Step 3: Measure improvement**

Compare total CI time against baseline (run #1010). Target: Ubuntu pipeline ≤ baseline + 5min overhead.

---

## Rollback

If artifact sharing causes failures or slowdowns:
1. Revert `needs:` changes in `lint`, `test-linux`, `validate`, `golden-tests`
2. Remove `build-ubuntu-debug` job
3. Restore original parallel structure
