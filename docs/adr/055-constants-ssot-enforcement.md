<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 55
title: Constants SSOT Enforcement + Cross-Import Elimination
status: ACCEPTED
created: 2026-03-02
updated: 2026-03-02
related: [13, 23, 54]
supersedes: []
superseded_by: []
implementation_status: Implemented
---

# ADR 055: Constants SSOT Enforcement + Cross-Import Elimination

## Status

**Accepted** (v0.2.1)

> Enforces Single Source of Truth for all project constants via compile-time rules.

## Context

Prior to this ADR, project constants were scattered across multiple crates with proxy/wrapper indirection:

### Before: Scattered Constants

```text
mcb-server (9 files)
  ├── src/constants.rs
  ├── src/handlers/constants.rs
  └── ... (7 more files with local constants)

mcb-providers (8 files)
  ├── src/embedding/constants.rs
  ├── src/vector_store/constants.rs
  └── ... (6 more files with local constants)

mcb-validate (21 files)
  ├── src/constants.rs
  ├── src/rules/constants.rs
  └── ... (19 more files with local constants)
```

### Problems Identified

1. **No Single Source of Truth**: Same constants defined in multiple places (timeouts, limits, magic numbers)
2. **Proxy/Wrapper Indirection**: Crates re-exported `mcb_utils::constants::*` through local `pub mod constants;` creating confusion about canonical import paths
3. **No Enforcement**: No compile-time or validation rules prevented constants from being defined outside mcb-utils
4. **Import Inconsistency**: Some code imported from `mcb_server::constants`, others from `mcb_utils::constants`
5. **Cross-Crate Imports**: Outer crates (mcb-server) imported from other outer crates (mcb-providers) violating Clean Architecture inward-only dependency flow

## Decision

Centralize **ALL** project constants in `mcb-utils/src/constants/` with strict enforcement rules.

### New Constants Structure

```text
mcb-utils/src/constants/
├── ast.rs              # AST parsing constants (moved from mcb-validate)
├── auth.rs             # Authentication constants
├── crypto.rs           # Cryptographic constants
├── display.rs          # NEW: Display/UI constants (from mcb-server)
├── embedding.rs        # Embedding model constants (extended)
├── events.rs           # Event type constants
├── http.rs             # HTTP protocol constants (extended)
├── io.rs               # I/O operation constants
├── keys.rs             # Storage key constants (extended)
├── lang.rs             # Language support constants (extended)
├── limits.rs           # Rate limits, thresholds (extended)
├── protocol.rs         # NEW: MCP protocol constants (from mcb-server)
├── search.rs           # Search operation constants (extended)
├── time.rs             # Time formatting constants
├── use_cases.rs        # Use case identifiers
├── validate.rs         # NEW: Validation rule constants (from mcb-validate)
├── values.rs           # Value object constants
├── vcs.rs              # NEW: Version control constants (from mcb-providers)
└── vector_store.rs     # NEW: Vector store constants (from mcb-providers)
```

### Canonical Import Pattern

All constants must be imported directly from mcb-utils:

```rust
// CORRECT: Direct import from mcb-utils
use mcb_utils::constants::display::{MAX_RESPONSE_LEN, DEFAULT_TRUNCATE_AT};
use mcb_utils::constants::vcs::{GIT_DIFF_CONTEXT_LINES, DEFAULT_BRANCH};
use mcb_utils::constants::protocol::{MCP_PROTOCOL_VERSION, JSONRPC_VERSION};
use mcb_utils::constants::vector_store::{DEFAULT_HNSW_M, DEFAULT_HNSW_EF};
use mcb_utils::constants::validate::{MAX_RULES_PER_PHASE, DEFAULT_THRESHOLD};

// INCORRECT: Re-exports through other crates (now blocked by CA018)
use mcb_server::constants::display::MAX_RESPONSE_LEN;  // ERROR
use mcb_providers::constants::vcs::DEFAULT_BRANCH;      // ERROR
```

### Enforcement Rules (3 New CA Rules)

#### CA016: Constants SSOT Enforcement

**Rule**: Only `mcb-utils` may define `pub mod constants;`

**Detection Pattern**:
```rust
// VIOLATION: Any non-mcb-utils crate with pub mod constants
pub mod constants;  // CA016 violation in mcb-server, mcb-providers, etc.
```

**Rationale**: Ensures constants are defined in exactly one location (mcb-utils).

#### CA018: No Proxy/Wrapper Re-exports

**Rule**: Non-mcb-utils crates MUST NOT re-export mcb_utils items via `pub use mcb_utils::...`

**Detection Pattern**:
```rust
// VIOLATION: Re-exporting mcb_utils through another crate
pub use mcb_utils::constants::display::*;  // CA018 violation
pub use mcb_utils::UtilsError;             // CA018 violation
```

**Rationale**: Eliminates indirection and ensures direct imports from the source.

#### CA019: Outer Crate Isolation

**Rule**: `mcb-server` src/ MUST NOT import from other outer crates (`mcb-providers`, `mcb-infrastructure`, `mcb-validate`)

**Detection Pattern**:
```rust
// VIOLATION: Outer crate importing from another outer crate
use mcb_providers::embedding::ollama::OllamaProvider;     // CA019 violation
use mcb_infrastructure::config::loader::ConfigLoader;     // CA019 violation
use mcb_validate::rules::validator::RuleValidator;        // CA019 violation
```

**Allowed**: Outer crates may only import from `mcb-domain` (Layer 1) and `mcb-utils` (Layer 0).

**Rationale**: Enforces Clean Architecture dependency direction (inward only).

### Migration Summary

| Source Location | Destination | Constants Moved |
|-----------------|-------------|-----------------|
| `mcb-server/src/constants.rs` | `mcb-utils/src/constants/display.rs` | Display/UI constants |
| `mcb-server/src/protocol/constants.rs` | `mcb-utils/src/constants/protocol.rs` | MCP protocol constants |
| `mcb-providers/src/vcs/constants.rs` | `mcb-utils/src/constants/vcs.rs` | VCS constants |
| `mcb-providers/src/vector_store/constants.rs` | `mcb-utils/src/constants/vector_store.rs` | Vector store constants |
| `mcb-validate/src/constants.rs` | `mcb-utils/src/constants/validate.rs` | Validation rule constants |

### Extended Modules

The following existing modules were extended with additional constants:

- `keys.rs` - Added storage key patterns from mcb-validate
- `search.rs` - Added search parameter limits
- `limits.rs` - Added rate limiting thresholds
- `embedding.rs` - Added model dimension constants
- `http.rs` - Added timeout constants
- `lang.rs` - Added language detection constants

## Consequences

### Positive

1. **Single Source of Truth**: All constants exist in exactly one location (mcb-utils)
2. **Clear Import Path**: Developers always know to import from `mcb_utils::constants`
3. **Compile-Time Enforcement**: 3 CA rules prevent regression at build time
4. **Zero Proxy Files**: No indirection or confusion about canonical paths
5. **Clean Architecture Compliance**: CA019 ensures proper layer isolation
6. **Simplified Maintenance**: Update constant in one place, affects all crates
7. **Better Discoverability**: All constants organized by domain in mcb-utils

### Negative

1. **Import Path Updates**: Existing code must update imports (one-time migration)
2. **mcb-utils Growth**: Innermost crate now larger (mitigated by clear module structure)
3. **Rule Violation Cleanup**: Initial cleanup required to achieve zero violations

### Neutral

1. **No Functional Changes**: Pure refactoring - no behavior modification
2. **Zero Runtime Impact**: Compile-time enforcement only
3. **Backward Compatible**: Old import paths still work during deprecation period

## Architecture Validation Updates

The `mcb-validate` crate implements the 3 new CA rules:

| Rule | Phase | Status |
|------|-------|--------|
| CA016 | Phase 6 (Architecture) | Implemented |
| CA018 | Phase 6 (Architecture) | Implemented |
| CA019 | Phase 6 (Architecture) | Implemented |

### Validation Output

```bash
$ make validate
Architecture validation: 0 violations
```

## References

- [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) — Layer separation principles
- [ADR 023: Inventory to Linkme Migration](023-inventory-to-linkme-migration.md) — Compile-time registration pattern
- [ADR 054: mcb-utils as Innermost Layer 0 Crate](054-mcb-utils-innermost-crate.md) — Establishment of mcb-utils as Layer 0
- [Architecture Boundaries](../architecture/ARCHITECTURE_BOUNDARIES.md) — CA rule documentation
- [Clean Architecture](../architecture/CLEAN_ARCHITECTURE.md) — Layer rules and dependency flow
