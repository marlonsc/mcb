# Error Handling

Last updated: 2026-02-11

General policy:

- Fix root causes, not symptoms.
- Avoid silent failures and empty catch blocks.
- Keep failures observable with concrete diagnostics.

Rust project specifics:

- Use domain error factories and `Result<T>` propagation.
- Add contextual error messages when crossing boundaries.
- Keep test-only panics/assertions isolated to test code.

Context maintenance failures:

- Broken context references: fix path or remove stale entry.
- Oversized context files: compact to MVI form.
- Stale context: refresh from current source files and note update date.
