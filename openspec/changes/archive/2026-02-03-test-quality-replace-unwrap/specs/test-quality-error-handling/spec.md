# Spec

## ADDED Requirements

### Requirement: Regex patterns use proper error propagation

The Test Quality Validator SHALL NOT use unwrap or expect when compiling regex patterns. It SHALL propagate regex compilation failures via `ValidationError::InvalidRegex` and return `Err` from `validate()`.

#### Scenario: Invalid regex pattern propagates error

- **WHEN** a regex pattern fails to compile (e.g., malformed pattern)
- **THEN** `validate()` returns `Err(ValidationError::InvalidRegex(...))`
- **AND** no panic occurs

#### Scenario: Valid regex patterns compile successfully

- **WHEN** all regex patterns are valid
- **THEN** `validate()` compiles them and proceeds to scan test files
- **AND** violations are returned as before
