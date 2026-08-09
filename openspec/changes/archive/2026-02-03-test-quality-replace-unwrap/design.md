# Design

## Context

The Test Quality Validator uses seven static regex patterns in `validate()`. Currently each is compiled with `Regex::new(...).unwrap()`, which can panic on invalid regex. Project standards forbid unwrap/expect; other validators (e.g., `documentation.rs`, `error_boundary.rs`) use `ValidationError::InvalidRegex` with `map_err` + `?`.

## Goals / Non-Goals

**Goals:**

- Replace all seven unwrap calls with proper error propagation
- Use existing `ValidationError::InvalidRegex` variant
- Match error-handling style of other validators in the crate

**Non-Goals:**

- Changing regex patterns or validation logic
- Introducing lazy_static/once_cell for pattern compilation
- Modifying `ValidationError` enum

## Decisions

### Decision 1: Use map_err + ? for each Regex::new

**Rationale:** Same pattern used in `documentation.rs` lines 169–228 and `error_boundary.rs` lines 189–191. Keeps implementation consistent. Each pattern gets a descriptive error message (e.g., `"ignore pattern: {e}"`) for debugging.

**Alternative considered:** Compile once at module level with lazy_static. Rejected as overkill for seven trivial patterns used only in one function.

### Decision 2: Descriptive error messages per pattern

Include pattern name in each `InvalidRegex` message (e.g., `ignore_pattern`, `test_pattern`) so failures are debuggable. Format: `"{pattern_name}: {e}"`.

## Risks / Trade-offs

- **Minimal risk**: Regex patterns are compile-time constants; invalid syntax would be caught in development. Change mainly enforces project standards and prevents theoretical panic paths.
