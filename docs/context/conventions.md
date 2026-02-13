<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Conventions Context

**Last updated:** 2026-02-02
**Source:** `README.md` (developer workflow) and `docs/developer/CONTRIBUTING.md` (contributor guide)

## Overview

The team uses strict tooling, formatting, and safety conventions so the large workspace stays consistent across contributions and CI validations.

## Key Conventions

### Make-first workflow

**Used in:** `README.md` "Development" section

-   Prefer `make build`/`make build-release` for compilation, `make test` for testing, `make validate` for architecture rules, and `make quality` for full checks.
-   Avoid direct `cargo` or `git` commands unless the docs explicitly allow them.
**Why it matters:** `make sync` encapsulates git hooks/formatting, keeping contributions aligned with the release branch.

### Formatting and imports

**Used in:** `docs/developer/CONTRIBUTING.md` (style guide) and `docs/architecture/ARCHITECTURE.md`

-   Rustfmt config: edition 2024, max width 100, tab size 4.
-   Import order: standard library → external crates → workspace crates → local modules.
**When to apply:** Run `make fmt` before committing and keep imports ordered to satisfy linters.

### Safety and error handling

**Used in:** `docs/ADR/019-error-handling-strategy.md`

-   Never use `unwrap()`/`expect()` outside tests.
-   Avoid `as any`/`@ts-ignore` equivalents or hardcoded fallback values.
-   Prefer helper constructors (`Error::io(...)`) and `?` propagation.

## Related Context

-   `docs/context/technical-patterns.md`
-   `docs/ADR/019-error-handling-strategy.md` (deep dive on handling errors)
