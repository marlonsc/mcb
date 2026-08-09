# Proposal: Replace unwrap in Test Quality Validator

## Why

The Test Quality Validator (`test_quality.rs`) uses `Regex::new(...).unwrap()` for seven static patterns. Project rules require no unwrap/expect—use `?` with proper error types. Panicking on invalid regex (however unlikely) violates these standards and diverges from other validators in the crate.

## What Changes

- Replace seven `Regex::new(...).unwrap()` calls with `.map_err(|e| ValidationError::InvalidRegex(...))?` in `test_quality.rs`
- Use `ValidationError::InvalidRegex` (already defined in `lib.rs`) for regex compilation failures
- Align error handling with `documentation.rs`, `error_boundary.rs`, and other validators

## Capabilities

### New Capabilities

- `test-quality-error-handling`: Test Quality Validator compiles regex patterns with proper error propagation instead of panicking

### Modified Capabilities

<!-- None - this is internal implementation, not spec-level requirement change -->

## Impact

- `crates/mcb-validate/src/test_quality.rs`: Lines 210–217 modified to use `map_err` + `?` instead of `unwrap()`
- No API changes, no new dependencies
- `validate()` signature unchanged; it already returns `Result<Vec<TestQualityViolation>>`
