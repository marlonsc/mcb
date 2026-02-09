# Quality Report: Checks

**Total Issues:** 24

## Severity Distribution

| Severity | Count | Percentage |
|----------|-------|------------|
| 🔴 ERROR | 24 | 100.0% |
| 🟠 WARNING | 0 | 0.0% |
| 🔵 NOTE | 0 | 0.0% |

## Category Breakdown

| Category | Count | Percentage |
|----------|-------|------------|
| zizmor | 24 | 100.0% |

## Top Rules

| Rule | Count | Percentage |
|------|-------|------------|
| `zizmor:zizmor/unpinned-uses` | 15 | 62.5% |
| `zizmor:zizmor/cache-poisoning` | 6 | 25.0% |
| `zizmor:zizmor/dangerous-triggers` | 1 | 4.2% |
| `zizmor:zizmor/bot-conditions` | 1 | 4.2% |
| `zizmor:zizmor/excessive-permissions` | 1 | 4.2% |

## Most Affected Files

| File | Issues |
|------|--------|
| `.github/workflows/ci.yml` | 9 |
| `.github/workflows/docs.yml` | 6 |
| `.github/workflows/release.yml` | 5 |
| `.github/workflows/auto-reviewer.yml` | 2 |
| `.github/workflows/codeql.yml` | 2 |

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
