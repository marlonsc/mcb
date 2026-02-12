# Testing Patterns

Last updated: 2026-02-11

Project defaults:

- Prefer crate-level integration tests in `tests/`.
- Use shared fixtures/helpers under `tests/test_utils/` where available.
- Keep test names explicit about scenario and expected behavior.

Execution:

- Standard path: `make test`.
- For targeted verification, run crate-specific test commands.

Quality expectation:

- New logic should include test updates in the same session.
- Preserve existing failing tests unless issue explicitly addresses them.
