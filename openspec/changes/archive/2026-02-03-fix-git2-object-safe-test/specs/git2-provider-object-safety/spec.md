# Spec

## ADDED Requirements

### Requirement: Git2Provider is object-safe

Git2Provider SHALL implement VcsProvider in an object-safe manner, so it can be used as `&dyn VcsProvider` or `Arc<dyn VcsProvider>`.

#### Scenario: Object-safety test passes

- **WHEN** `cargo test -p mcb-providers test_git2_provider_is_object_safe` runs
- **THEN** the test passes
- **AND** the test verifies object safety by passing a `&Git2Provider` to a function expecting `&dyn VcsProvider`
