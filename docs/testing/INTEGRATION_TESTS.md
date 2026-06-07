# Integration Tests

This guide documents the current external-service test path. When this guide
disagrees with code, trust `config/tests.toml`,
`crates/mcb-domain/src/utils/tests/`, `crates/mcb-domain/src/macros/testing.rs`,
`tests/docker-compose.yml`, and `makefiles/dispatch.mk`.

## Canonical Sources

| Concern | Source |
| ------- | ------ |
| Service URLs | `config/tests.toml` under `[test_services]` |
| TCP availability checks | `crates/mcb-domain/src/utils/tests/service_detection.rs` |
| Skip macros | `crates/mcb-domain/src/macros/testing.rs` |
| Docker test services | `tests/docker-compose.yml` |
| Make verbs | `makefiles/dispatch.mk` |
| CI gate | `.github/workflows/ci.yml` |

The old `docs/operations/INTEGRATION_TEST_SKIPPING.md` page was archived
because it pointed at removed helper paths and mixed current test policy with
historical backlog notes.

## Service Detection

External service tests use `config/tests.toml` and the shared helpers in
`mcb-domain`:

```rust
use mcb_domain::utils::tests::service_detection::{
    is_milvus_available,
    is_ollama_available,
    is_postgres_available,
    is_redis_available,
};
```

Available helpers:

- `check_service_available(host, port)`
- `is_milvus_available()`
- `is_ollama_available()`
- `is_redis_available()`
- `is_postgres_available()`
- `is_ci()`
- `should_run_docker_integration_tests()`

The `MCB_RUN_DOCKER_INTEGRATION_TESTS` environment variable controls whether
Docker-backed integration tests run. CI sets it to `0`, so those tests skip
unless explicitly enabled.

## Skip Macros

Use the macro matching the test return type:

```rust
skip_if_service_unavailable!("Milvus", is_milvus_available());
skip_if_any_service_unavailable!(
    "Milvus" => is_milvus_available(),
    "Ollama" => is_ollama_available(),
);
```

For tests returning `TestResult` or another `Result`, use:

```rust
skip_if_service_unavailable_result!("Milvus", is_milvus_available());
skip_if_any_service_unavailable_result!(
    "Milvus" => is_milvus_available(),
    "Ollama" => is_ollama_available(),
);
```

Use `require_service!("milvus")` when the test should skip if the service is
not configured in `config/tests.toml`, before making any network call.

## Run Tests

Run the current test scopes through Make:

```bash
make test SCOPE=integration
make test SCOPE=all
```

Start and stop local Docker services through the `dev` verb:

```bash
make dev WHAT=docker-up
make test SCOPE=integration
make dev WHAT=docker-down
```

For the containerized test runner:

```bash
make dev WHAT=docker-test
```

## Item-by-item Classification Of Archived Future Notes

The archived operations page listed four future improvements. Current
classification:

| Item | Current state | Evidence |
| ---- | ------------- | -------- |
| Service availability reporting | Pending follow-up | No summary reporter exists beyond per-test skip output |
| Conditional test groups | Pending follow-up | Make scopes and `MCB_RUN_DOCKER_INTEGRATION_TESTS` exist, but no all-services gate groups tests dynamically |
| Docker Compose for local E2E | Completed | `tests/docker-compose.yml` and `make dev WHAT=docker-up` / `make dev WHAT=docker-test` exist |
| Coverage integration | Superseded by current gate | `make check WHAT=coverage` excludes integration/admin test files and CI runs a dedicated coverage job |

Pending follow-up work must live in beads, not as loose notes in this document.

## Troubleshooting

If a test skips unexpectedly:

1. Confirm the service URL exists in `config/tests.toml`.
2. Confirm `MCB_RUN_DOCKER_INTEGRATION_TESTS` is not forcing skips.
3. Confirm the service is listening on the configured host and port.
4. Put the skip macro before async setup or provider initialization.

If a test times out before skipping, move the service check to the start of the
test or switch to the `_result` macro for `Result`-returning tests.
