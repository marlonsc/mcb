# Error Handling

Last updated: 2026-02-11 (America/Sao_Paulo)

## Purpose

Summarize error-handling constraints that must remain visible during
context and code updates.

## Rules

- Prefer typed errors and propagation (`?`) over ad-hoc fallback behavior.
- Do not introduce silent failures (empty catches, ignored results).
- Keep failures actionable: include cause, path, and failed operation.
- In scripts/docs tooling, fail fast on invalid links or broken structure checks.

## Anti-Patterns

- `unwrap()`/`expect()` in production paths.
- Type suppression shortcuts (`as any`, ignore directives).
- "Fix" by removing validation/tests rather than addressing root cause.

## Verification Expectations

- Re-run related checks after fixes.
- Note whether failures are newly introduced or pre-existing.

## Sources

- `docs/adr/019-error-handling-strategy.md`
- `README.md`
- `docs/developer/CONTRIBUTING.md`

## Update Notes

- 2026-02-11: Initial policy snapshot for context-aware maintenance work.
