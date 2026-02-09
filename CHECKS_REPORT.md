# Quality Report: Checks

**Total Issues:** 134

## Severity Distribution

| Severity | Count | Percentage |
|----------|-------|------------|
| 🔴 ERROR | 24 | 17.9% |
| 🟠 WARNING | 25 | 18.7% |
| 🔵 NOTE | 85 | 63.4% |

## Category Breakdown

| Category | Count | Percentage |
|----------|-------|------------|
| rustfmt | 85 | 63.4% |
| zizmor | 44 | 32.8% |
| osv-scanner | 5 | 3.7% |

## Top Rules

| Rule | Count | Percentage |
|------|-------|------------|
| `rustfmt:fmt` | 85 | 63.4% |
| `zizmor:zizmor/artipacked` | 20 | 14.9% |
| `zizmor:zizmor/unpinned-uses` | 15 | 11.2% |
| `zizmor:zizmor/cache-poisoning` | 6 | 4.5% |
| `zizmor:zizmor/dangerous-triggers` | 1 | 0.7% |
| `zizmor:zizmor/bot-conditions` | 1 | 0.7% |
| `zizmor:zizmor/excessive-permissions` | 1 | 0.7% |
| `osv-scanner:RUSTSEC-2023-0089` | 1 | 0.7% |
| `osv-scanner:RUSTSEC-2025-0119` | 1 | 0.7% |
| `osv-scanner:RUSTSEC-2024-0436` | 1 | 0.7% |
| `osv-scanner:CVE-2023-49092` | 1 | 0.7% |
| `osv-scanner:RUSTSEC-2025-0134` | 1 | 0.7% |

## Most Affected Files

| File | Issues |
|------|--------|
| `.github/workflows/ci.yml` | 18 |
| `.github/workflows/docs.yml` | 12 |
| `.github/workflows/release.yml` | 8 |
| `Cargo.lock` | 5 |
| `.github/workflows/codeql.yml` | 3 |
| `.github/workflows/auto-reviewer.yml` | 2 |
| `.github/workflows/retag-on-merge.yml` | 1 |
| `crates/mcb-application/tests/unit/search_tests.rs` | 1 |
| `crates/mcb-application/tests/unit/test_utils.rs` | 1 |
| `crates/mcb-ast-utils/src/lib.rs` | 1 |
| `crates/mcb-ast-utils/tests/unit/complexity_tests.rs` | 1 |
| `crates/mcb-domain/src/registry/mod.rs` | 1 |
| `crates/mcb-domain/tests/integration/semantic_search_workflow.rs` | 1 |
| `crates/mcb-domain/tests/unit/constants_tests.rs` | 1 |
| `crates/mcb-infrastructure/src/infrastructure/prometheus_metrics.rs` | 1 |
| `crates/mcb-infrastructure/tests/di/dispatch_tests.rs` | 1 |
| `crates/mcb-infrastructure/tests/di/handle_tests.rs` | 1 |
| `crates/mcb-language-support/tests/unit/parser_tests.rs` | 1 |
| `crates/mcb-providers/src/database/mod.rs` | 1 |
| `crates/mcb-providers/src/embedding/anthropic.rs` | 1 |

## 🔴 ERROR Issues (24)

### zizmor:zizmor/unpinned-uses (15 issues)

-   `.github/workflows/ci.yml:50`

  > unpinned action reference

-   `.github/workflows/ci.yml:65`

  > unpinned action reference

-   `.github/workflows/ci.yml:81`

  > unpinned action reference

-   `.github/workflows/ci.yml:94`

  > unpinned action reference

-   `.github/workflows/ci.yml:118`

  > unpinned action reference

-   `.github/workflows/ci.yml:128`

  > unpinned action reference

-   `.github/workflows/ci.yml:138`

  > unpinned action reference

-   `.github/workflows/ci.yml:151`

  > unpinned action reference

-   `.github/workflows/ci.yml:176`

  > unpinned action reference

-   `.github/workflows/codeql.yml:44`

  > unpinned action reference
  ... and 5 more

### zizmor:zizmor/cache-poisoning (6 issues)

-   `.github/workflows/codeql.yml:47`

  > runtime artifacts potentially vulnerable to a cache poisoning attack

-   `.github/workflows/docs.yml:34`

  > runtime artifacts potentially vulnerable to a cache poisoning attack

-   `.github/workflows/docs.yml:51`

  > runtime artifacts potentially vulnerable to a cache poisoning attack

-   `.github/workflows/docs.yml:96`

  > runtime artifacts potentially vulnerable to a cache poisoning attack

-   `.github/workflows/release.yml:40`

  > runtime artifacts potentially vulnerable to a cache poisoning attack

-   `.github/workflows/release.yml:82`

  > runtime artifacts potentially vulnerable to a cache poisoning attack

### zizmor:zizmor/dangerous-triggers (1 issues)

-   `.github/workflows/auto-reviewer.yml:25-29`

  > use of fundamentally insecure workflow trigger

### zizmor:zizmor/bot-conditions (1 issues)

-   `.github/workflows/auto-reviewer.yml:37`

  > spoofable bot actor check

### zizmor:zizmor/excessive-permissions (1 issues)

-   `.github/workflows/release.yml:19`

  > overly broad permissions

## 🟠 WARNING Issues (25)

### zizmor:zizmor/artipacked (20 issues)

-   `.github/workflows/ci.yml:48`

  > credential persistence through GitHub Actions artifacts

-   `.github/workflows/ci.yml:63`

  > credential persistence through GitHub Actions artifacts

-   `.github/workflows/ci.yml:79`

  > credential persistence through GitHub Actions artifacts

-   `.github/workflows/ci.yml:92`

  > credential persistence through GitHub Actions artifacts

-   `.github/workflows/ci.yml:116`

  > credential persistence through GitHub Actions artifacts

-   `.github/workflows/ci.yml:126`

  > credential persistence through GitHub Actions artifacts

-   `.github/workflows/ci.yml:136`

  > credential persistence through GitHub Actions artifacts

-   `.github/workflows/ci.yml:149`

  > credential persistence through GitHub Actions artifacts

-   `.github/workflows/ci.yml:174`

  > credential persistence through GitHub Actions artifacts

-   `.github/workflows/codeql.yml:34-35`

  > credential persistence through GitHub Actions artifacts
  ... and 10 more

### osv-scanner:RUSTSEC-2023-0089 (1 issues)

-   `Cargo.lock:0`

  > Package 'atomic-polyfill@1.0.3' is vulnerable to 'RUSTSEC-2023-0089'.

### osv-scanner:RUSTSEC-2025-0119 (1 issues)

-   `Cargo.lock:0`

  > Package 'number_prefix@0.4.0' is vulnerable to 'RUSTSEC-2025-0119'.

### osv-scanner:RUSTSEC-2024-0436 (1 issues)

-   `Cargo.lock:0`

  > Package 'paste@1.0.15' is vulnerable to 'RUSTSEC-2024-0436'.

### osv-scanner:CVE-2023-49092 (1 issues)

-   `Cargo.lock:0`

  > Package 'rsa@0.9.10' is vulnerable to 'CVE-2023-49092' (also known as 'RUSTSEC-2023-0071', 'GHSA-4gr...

### osv-scanner:RUSTSEC-2025-0134 (1 issues)

-   `Cargo.lock:0`

  > Package 'rustls-pemfile@2.2.0' is vulnerable to 'RUSTSEC-2025-0134'.

## 🔵 NOTE Issues (85)

### rustfmt:fmt (85 issues)

-   `crates/mcb-application/tests/unit/search_tests.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.

-   `crates/mcb-application/tests/unit/test_utils.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.

-   `crates/mcb-ast-utils/src/lib.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.

-   `crates/mcb-ast-utils/tests/unit/complexity_tests.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.

-   `crates/mcb-domain/src/registry/mod.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.

-   `crates/mcb-domain/tests/integration/semantic_search_workflow.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.

-   `crates/mcb-domain/tests/unit/constants_tests.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.

-   `crates/mcb-infrastructure/src/infrastructure/prometheus_metrics.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.

-   `crates/mcb-infrastructure/tests/di/dispatch_tests.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.

-   `crates/mcb-infrastructure/tests/di/handle_tests.rs:0`

  > Incorrect formatting, autoformat by running `qlty fmt`.
  ... and 75 more
