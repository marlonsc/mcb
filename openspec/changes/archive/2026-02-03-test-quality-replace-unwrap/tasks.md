# Tasks

## 1. Replace unwrap with error propagation

- [x] 1.1 Add ValidationError to imports in test_quality.rs
- [x] 1.2 Replace all seven Regex::new(...).unwrap() with map_err(|e| ValidationError::InvalidRegex(format!("pattern_name: {e}")))? in validate()

## 2. Verify

- [x] 2.1 Run make test to ensure no regressions
- [x] 2.2 Run make lint to ensure code quality
