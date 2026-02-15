<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Validation Module

**Source**: `crates/mcb-validate/src/`
**Crate**: `mcb-validate`
**Lines of Code**: ~8,000+

## Overview

The validation module provides comprehensive architecture enforcement and code quality validation for the Memory Context Browser project. It implements a multi-phase validation pipeline that ensures Clean Architecture compliance, code quality standards, and architectural decision record (ADR) adherence.

The module uses a **trait-based validator system** (`traits/`) with **macro-based violation definitions** (`violation_macro.rs`) and a **declarative validator pattern** for concise rule implementations.

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

### Key Components

### Validators (`validators/`)

Domain-specific validators implementing the `Validator` trait:

- **Clean Architecture** (`clean_architecture/`) — CA001-CA009 boundary enforcement
- **SOLID** (`solid/`) — SOLID principle checks
- **Quality** (`quality/`) — Code quality rules
- **Organization** (`organization/`) — Module and file organization rules
- **Hygiene** (`hygiene/`) — Code hygiene checks
- **Implementation** (`implementation/`) — Implementation pattern validation
- `async_patterns.rs` — Async pattern detection
- `config_quality.rs` — Configuration quality checks
- `declarative_validator.rs` — Declarative validation framework
- `dependency.rs` — Dependency validation
- `documentation.rs` — Documentation quality checks
- `error_boundary.rs` — Error handling pattern validation
- `kiss.rs` — KISS principle enforcement
- `layer_flow.rs` — Layer dependency flow validation
- `naming.rs` — Naming convention enforcement
- `pattern_validator.rs` — Pattern-based validation
- `performance.rs` — Performance pattern checks
- `pmat.rs` / `pmat_native.rs` — Process Maturity Analysis
- `port_adapter.rs` — Port/Adapter pattern validation
- `refactoring.rs` — Refactoring opportunity detection
- `test_quality.rs` — Test quality analysis
- `visibility.rs` — Visibility and encapsulation checks

### Traits (`traits/`)

Core validation abstractions:

- `validator.rs` — `Validator` trait definition and registry
- `violation.rs` — `Violation` trait and violation types

### Extractor (`extractor/`)

Fact extraction from source code:

- `fact.rs` — Fact data model
- `rust_extractor.rs` — Rust-specific fact extraction

### Filters (`filters/`)

File and rule filtering:

- `dependency_parser.rs` — Dependency graph parsing
- `file_matcher.rs` — File pattern matching
- `language_detector.rs` — Source language detection
- `rule_filters.rs` — Rule applicability filters

### Reporter (`reporter/`)

Validation report generation:

- `summary.rs` — Summary report generation

### Pattern Registry (`pattern_registry/`)

- `registry.rs` — Pattern registration and lookup

### Dependency Graph (`graph/`)

- `dep_graph.rs` — Dependency graph construction and analysis

### Linters (`linters/`)

Code quality linting via external tools:

- `engine.rs` — Linter engine orchestration
- `executor.rs` — Linter execution
- `parsers.rs` — Output parsing
- `runners.rs` — Linter runners

### AST Queries (`ast/`)

Tree-sitter based AST parsing and querying:

- `core.rs` — AST core abstractions
- `decoder.rs` — AST node decoding
- `query.rs` — AST query execution
- `types.rs` — AST type definitions

### Rule Engines (`engines/`)

Multiple rule engine implementations:

- `expression_engine.rs` — evalexpr-based expression evaluation
- `hybrid_engine.rs` — Combined engine approach
- `rete_engine.rs` — RETE algorithm for pattern matching
- `router.rs` — Rule routing and selection
- `rusty_rules_engine.rs` — Rusty-rules integration
- `validator_engine.rs` — Validator trait integration

### Metrics (`metrics/`)

Code metrics analysis using Rust-code-analysis:

- `rca_analyzer.rs` — Rust-code-analysis integration (feature-gated)
- `thresholds.rs` — Metric threshold definitions
- `violation.rs` — Metrics violation types

### Duplication Detection (`duplication/`)

Code clone detection using Rabin-Karp algorithm:

- `analyzer.rs` — Duplication analysis orchestration
- `detector.rs` — Clone detection logic
- `fingerprint.rs` — Token fingerprinting
- `thresholds.rs` — Duplication type definitions

### Root Modules

- `violation_macro.rs` — Macro-based violation definitions (Display + field formatting)
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
├── violation_macro.rs
└── lib.rs
```

## Related Documentation

- [Architecture Overview](../architecture/ARCHITECTURE.md#validation-layer) - Validation layer details
- [Implementation Status](../developer/IMPLEMENTATION_STATUS.md) - Detailed traceability
- [ADR-013](../adr/013-clean-architecture-crate-separation.md) - Clean Architecture separation
- [ADR-029](../adr/029-hexagonal-architecture-dill.md) - DI architecture (CA007-CA009)

---

**Last Updated**: 2026-02-14
