# Testing Patterns

Last updated: 2026-02-15 (America/Sao_Paulo)

## Test Frameworks

| Framework | Language | Scope | Config |
|-----------|----------|-------|--------|
| `cargo test` | Rust | Unit, integration, golden | `Cargo.toml` workspace |
| `rstest` 0.26 | Rust | Parametrized tests | In-code `#[rstest]` |
| `mockall` 0.14 | Rust | Mock objects | In-code `#[automock]` |
| `insta` 1.41 | Rust | Snapshot tests (JSON/YAML) | In-code `assert_snapshot!` |
| `serial_test` 3.2 | Rust | Sequential test isolation | `#[serial]` attribute |
| Playwright | TypeScript | E2E admin UI | `tests/playwright.config.ts` |

## Test Organization

```
crates/*/tests/
  unit/           # Per-module unit tests (*_tests.rs)
  integration/    # Cross-module integration tests
  admin/          # Admin UI backend tests
  test_utils/     # Shared helpers, fixtures, providers
  fixtures/       # Test data and sample codebases
tests/
  golden/         # Golden acceptance tests (full MCP flow)
  e2e/            # Playwright UI tests (*.spec.ts)
  docker/         # Dockerfile.test, docker-compose.yml
  fixtures/       # Shared test repos
```

## Make Targets

| Command | What It Runs |
|---------|-------------|
| `make test` | All workspace tests (cargo test --all-targets) |
| `make test SCOPE=unit` | Unit tests only (--lib) |
| `make test SCOPE=golden` | Golden acceptance tests |
| `make test SCOPE=startup` | Startup smoke (DDL/init) |
| `make test SCOPE=integration` | Integration tests |
| `make test SCOPE=e2e` | Playwright E2E (admin UI) |
| `make lint` | clippy + fmt check |
| `make validate` | Architecture rule enforcement |
| `make check` | fmt --check + lint + test + validate |
| `make coverage` | cargo-tarpaulin HTML/lcov |

## CI Pipeline (`.github/workflows/ci.yml`)

```
changes → classify → lint → test (matrix) → startup-smoke
  → validate → golden-tests → audit → coverage → release-build
  → rust-ci (consolidated gate)
```

Matrix: Ubuntu + macOS + Windows, Rust stable + beta.

## Practical Rules

- Run narrowest useful test loop while iterating (`SCOPE=unit`)
- Run broader checks before closing work (`make check`)
- Architecture validation is a **required gate**, not optional
- No `unwrap()`/`expect()` in production code paths

## Sources

- `Makefile`, `make/dev.mk`, `make/quality.mk`
- `.github/workflows/ci.yml`, `tests/playwright.config.ts`

## Update Notes

- 2026-02-15: Full harvest rewrite — framework catalog, test org, CI pipeline from source.
- 2026-02-11: Added condensed validation matrix.
