<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Validation Module

**Source**: `crates/mcb-validate/src/`
**Crate**: `mcb-validate`
**Lines of Code**: ~8,000+

## ↔ Code ↔ Docs cross-reference

| Direction | Link |
| --------- | ---- |
| Code → Docs | [`crates/mcb-validate/src/lib.rs`](../../crates/mcb-validate/src/lib.rs) links here |
| Docs → Code | [`crates/mcb-validate/src/lib.rs`](../../crates/mcb-validate/src/lib.rs) — crate root |
| Architecture | [`ARCHITECTURE.md`](../architecture/ARCHITECTURE.md) · [`ADR-013`](../adr/013-clean-architecture-crate-separation.md) · [`ADR-020`](../adr/020-testing-strategy-integration.md) |
| Roadmap | [`ROADMAP.md`](../developer/ROADMAP.md) |

## Overview

The validation module provides comprehensive architecture enforcement and code quality validation for the Memory Context Browser project. It implements a multi-phase validation pipeline that ensures Clean Architecture compliance, code quality standards, and architectural decision record (ADR) adherence.

The module uses a **trait-based validator system** (`traits/`) with **macro-based violation definitions** (`macros.rs`) and a **declarative validator pattern** for concise rule implementations.

## Architecture

The validation system follows a layered approach with seven verified phases:

```text
Validation Pipeline (Pure Rust):
┌─────────────────────────────────────────────┐
│ YAML Rules → Rule Loader → Rule Engine     │
│                                             │
│ Layer 1: Linters (Clippy/Ruff) ✅ Verified │
│ Layer 2: AST (Tree-sitter) ✅ Verified     │
│ Layer 3: Rule Engines ✅ Verified          │
│ Layer 4: Metrics (RCA) ✅ Verified         │
│ Layer 5: Duplication ✅ Verified           │
│ Layer 6: Architecture ✅ Verified          │
│ Layer 7: Integration ✅ Verified           │
│                                             │
│ Output: Unified Violation Interface        │
└─────────────────────────────────────────────┘
```

## Rules & Validators

The validation system implements over 100 rules categorized by their architectural intent. Below are the core rule sets.

### 🏗️ Clean Architecture (CA)
Enforces layer boundaries and dependency direction.

| Rule ID | Name | Description | Source |
| ------- | ---- | ----------- | ------ |
| `CA001` | Domain Independence | Domain crate must not depend on any internal crates | [`CA001_domain-independence.yml`](../../crates/mcb-validate/src/rules/clean-architecture/CA001_domain-independence.yml) |
| `CA003` | Domain Traits Only | Domain ports must be traits, not concrete implementations | [`CA003_domain-traits-only.yml`](../../crates/mcb-validate/src/rules/clean-architecture/CA003_domain-traits-only.yml) |
| `CA009` | Infra NO Application | Infrastructure cannot depend on Application services | [`CA009_infrastructure-no-application.yml`](../../crates/mcb-validate/src/rules/clean-architecture/CA009_infrastructure-no-application.yml) |

### 📁 Organization (ORG)
Validates file placement, module structure, and domain purity.

| Rule ID | Name | Description | Source |
| ------- | ---- | ----------- | ------ |
| `ORG015` | Adapter Location | Adapters must reside in `crates/mcb-providers/src/` | [`ORG015_adapter-location.yml`](../../crates/mcb-validate/src/rules/organization/ORG015_adapter-location.yml) |
| `ORG018` | Port Location | Traits/Ports must reside in `crates/mcb-domain/src/ports/` | [`ORG018_port-location.yml`](../../crates/mcb-validate/src/rules/organization/ORG018_port-location.yml) |
| `ORG020` | Domain Purity | Domain logic cannot leak into infrastructure adapters | [`domain_purity.rs`](../../crates/mcb-validate/src/validators/organization/domain_purity.rs) |

### ♻️ Refactoring (REF)
Detects technical debt and refactoring opportunities.

| Rule ID | Name | Description | Source |
| ------- | ---- | ----------- | ------ |
| `REF001` | Module Integrity | Detects `mod` declarations referencing deleted files | [`modules.rs`](../../crates/mcb-validate/src/validators/refactoring/modules.rs) |
| `REF002` | Large Method | Detects methods exceeding 50 lines (RCA-based) | [`metrics/`](../../crates/mcb-validate/src/metrics/) |

### 💎 Quality (QUAL)
Enforces safety and performance standards.

| Rule ID | Name | Description |
| ------- | ---- | ----------- |
| `QUAL001` | No Unwrap | Bans `unwrap()` in production code (use `Result`) |
| `QUAL002` | No Expect | Bans `expect()` in production code |
| `ASYNC001`| Async Patterns | Detects blocking calls in async contexts |

---

## Technical Details

### Registry & Orchestration
- `traits/validator.rs` — `Validator` trait definition and registry
- `traits/violation.rs` — `Violation` trait and violation types

### Fact Extraction (`extractor/`)
The system extracts facts from the AST for rule evaluation:
- `fact.rs` — Fact data model
- `rust_extractor.rs` — Rust-specific fact extraction

### Rule Engines (`engines/`)
- `hybrid_engine.rs` — Combined engine approach (Static + Dynamic)
- `rete_engine.rs` — RETE algorithm for high-performance pattern matching
- `expression_engine.rs` — `evalexpr`-based logic evaluation

- `analyzer.rs` — Duplication analysis orchestration
- `detector.rs` — Clone detection logic
- `fingerprint.rs` — Token fingerprinting
- `thresholds.rs` — Duplication type definitions

### Root Modules

- `macros.rs` — Macro-based violation definitions (Display + field formatting)
- `macros.rs` — Helper macros for validator implementation
- `embedded_rules.rs` — Embedded rule definitions
- `thresholds.rs` — Global threshold definitions
- `run_context.rs` — Validation execution context
- `generic_reporter.rs` — Generic report formatting
- `scan.rs` — File scanning
- `constants.rs` — Module constants
- `config/` — Validation configuration

### Rules (`rules/`)

YAML-based rule definitions:

- `yaml_loader.rs` — Rule loading from YAML files
- `yaml_validator.rs` — Rule schema validation
- `templates.rs` — Rule templates

## Usage

### Command Line

```bash

# Run all validation rules
make validate

# Quick validation (skip tests)
make validate QUICK=1

# Strict validation
make validate
```

## Single Source of Truth (SSOT)

The validation module enforces SSOT through the following mechanisms:
1. **Bidirectional Links**: Code headers must link to documentation files, and documentation must reference the relevant code items.
2. **Automated Audits**: `make docs-validate` checks for broken links and missing documentation headers.
3. **Traceability**: All architectural rules in `mcb-validate` are mapped to ADRs or core design principles documented in `docs/architecture/`.

### SSOT Rules
- `SSOT01` - Every `mod.rs` and `lib.rs` must have a documentation header.
- `SSOT02` - Documentation links must be valid and resolve to existing sections.
- `SSOT03` - Architecture decisions must be backed by an ADR.

## Programmatic API

```rust
use mcb_validate::{ValidatorRegistry, ValidationConfig};

let config = ValidationConfig::default();
let registry = ValidatorRegistry::new();
let violations = registry.validate_all(&config)?;
```

## Validation Status

**Phases 1-7**: All VERIFIED (v0.2.1)

- **Total Tests**: 344 test functions in mcb-validate
- **Project-Wide Tests**: 1700+ (includes all crates)
- **Verification Date**: 2026-02-14
- **Architecture Violations**: 0

## File Structure

```text
crates/mcb-validate/src/
├── ast/                    # AST parsing and queries
│   ├── core.rs
│   ├── decoder.rs
│   ├── query.rs
│   └── types.rs
├── config/                 # Validation configuration
│   └── file_config.rs
├── duplication/            # Clone detection
│   ├── analyzer.rs
│   ├── detector.rs
│   ├── fingerprint.rs
│   └── thresholds.rs
├── engines/                # Rule engines
│   ├── expression_engine.rs
│   ├── hybrid_engine.rs
│   ├── rete_engine.rs
│   ├── router.rs
│   ├── rusty_rules_engine.rs
│   └── validator_engine.rs
├── extractor/              # Fact extraction
│   ├── fact.rs
│   └── rust_extractor.rs
├── filters/                # File and rule filters
│   ├── dependency_parser.rs
│   ├── file_matcher.rs
│   ├── language_detector.rs
│   └── rule_filters.rs
├── graph/                  # Dependency graph
│   └── dep_graph.rs
├── linters/                # External linter integration
│   ├── engine.rs
│   ├── executor.rs
│   ├── parsers.rs
│   └── runners.rs
├── metrics/                # Code metrics analysis
│   ├── rca_analyzer.rs
│   ├── thresholds.rs
│   └── violation.rs
├── pattern_registry/       # Pattern registration
│   └── registry.rs
├── reporter/               # Report generation
│   └── summary.rs
├── rules/                  # YAML rule system
│   ├── templates.rs
│   ├── yaml_loader.rs
│   └── yaml_validator.rs
├── traits/                 # Core validation abstractions
│   ├── validator.rs
│   └── violation.rs
├── validators/             # Domain-specific validators
│   ├── clean_architecture/ # CA001-CA009
│   ├── hygiene/            # Code hygiene
│   ├── implementation/     # Implementation patterns
│   ├── organization/       # Module organization
│   ├── quality/            # Quality rules
│   ├── solid/              # SOLID principles
│   ├── async_patterns.rs
│   ├── config_quality.rs
│   ├── declarative_validator.rs
│   ├── dependency.rs
│   ├── documentation.rs
│   ├── error_boundary.rs
│   ├── kiss.rs
│   ├── layer_flow.rs
│   ├── naming.rs
│   ├── pattern_validator.rs
│   ├── performance.rs
│   ├── pmat.rs
│   ├── pmat_native.rs
│   ├── port_adapter.rs
│   ├── refactoring.rs
│   ├── test_quality.rs
│   └── visibility.rs
├── constants.rs
├── embedded_rules.rs
├── generic_reporter.rs
├── macros.rs
├── run_context.rs
├── scan.rs
├── thresholds.rs
├── macros.rs
└── lib.rs
```

## Related Documentation

- [Architecture Overview](../architecture/ARCHITECTURE.md#validation-layer) - Validation layer details
- [ADR-013](../adr/013-clean-architecture-crate-separation.md) - Clean Architecture separation
- [SSOT Principles](./README.md#documentation-principles) - Single Source of Truth
- [Validators Implementation](./validate.md#validators-validators) - List of active validators

---

**Last Updated**: 2026-02-20 - Consolidated SSOT and traceability (v0.2.1)
