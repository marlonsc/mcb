# MCB Build and Test Guide

## Essential Commands

All development flows through `make` verbs backed by `scripts/lib/mcb.sh`:

### Build

```bash
make build              # Debug build
make build RELEASE=1    # Release build
```

### Test

```bash
make test WHAT=unit          # Unit tests only
make test WHAT=integration   # Integration tests
make test WHAT=doc           # Doctests
make test WHAT=all           # Full suite
```

### Quality Gates

```bash
make check WHAT=fmt        # Formatting (rustfmt)
make check WHAT=lint       # Clippy lints
make check WHAT=validate   # Architecture validation
make check WHAT=audit      # Security audit
make check WHAT=all        # All checks (CI gate)
```

### Fix

```bash
make fix WHAT=fmt          # Auto-format
make fix WHAT=lint         # Auto-fix clippy
make fix WHAT=all          # All auto-fixes
```

### Validation

```bash
make check WHAT=validate    # Run mcb-validate architecture checks
```

### Single Test Debugging

```bash
cargo test -p mcb-server --test unit -- test_name
```

## Pre-commit Hooks

Installed via `make setup WHAT=hooks`:

- Staged `guard` (banned patterns)
- `rustfmt`
- `clippy --workspace`
- `typos`
- Unit tests

## CI Gate

```bash
make check WHAT=all   # All checks (CI gate)
```

## Architecture Validation

The `mcb-validate` crate enforces Clean Architecture compliance:

```bash
make check WHAT=validate
```
