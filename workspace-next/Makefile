# Workspace-next Makefile
# For building the refactored crate structure

.PHONY: build test lint fmt clean check verify release
.PHONY: validate validate-report validate-summary validate-arch
.PHONY: validate-deps validate-quality validate-patterns validate-tests validate-docs validate-naming validate-solid validate-org
.PHONY: validate-kiss validate-shaku validate-refactor
.PHONY: validate-all validate-legacy validate-config
.PHONY: check-full ci

# Build all crates
build:
	cargo build --workspace

# Run all tests
test:
	cargo test --workspace

# Run clippy lints
lint:
	cargo clippy --workspace -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Clean build artifacts
clean:
	cargo clean

# Full check (build + lint + test)
check: build lint test
	@echo "All checks passed!"

# Check compilation only (fast)
verify:
	cargo check --workspace

# Build in release mode
release:
	cargo build --workspace --release

# ============================================
# Architecture Validation Targets
# ============================================

# Quick validation summary (shows counts only)
validate:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║           mcb-validate Architecture Report                   ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@cargo test --package mcb-validate test_full_validation_report -- --nocapture 2>&1 | \
		grep -E "(Total Violations:|Dependency:|Quality:|Patterns:|Tests:|Documentation:|Naming:|SOLID:|Organization:|KISS:|DI/Shaku:|Refactoring:|Status:|\[Error\]|\[Warning\])" | \
		head -30
	@echo ""
	@echo "Run 'make validate-report' for full details"

# Full validation report (shows all violations)
validate-report:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║        mcb-validate Full Architecture Report                 ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@cargo test --package mcb-validate test_full_validation_report -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|^$\|Compiling\|Finished\|Doc-tests\|passed\|filtered"

# Full architecture validation (all tests)
validate-arch:
	cargo test --package mcb-validate -- --nocapture
	@echo "Architecture validation completed!"

# Individual validator targets for targeted fixes
validate-deps:
	@echo "=== Dependency Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_dependencies -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-quality:
	@echo "=== Quality Violations (unwrap/expect/panic) ==="
	@cargo test --package mcb-validate test_validate_workspace_quality -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-patterns:
	@echo "=== Pattern Violations (DI, async traits) ==="
	@cargo test --package mcb-validate test_validate_workspace_patterns -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-tests:
	@echo "=== Test Organization Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_tests -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-docs:
	@echo "=== Documentation Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_documentation -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-naming:
	@echo "=== Naming Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_naming -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

# Extended check (build + lint + test + validation summary)
check-full: check
	@echo ""
	@$(MAKE) validate
	@echo ""
	@echo "Full validation completed!"

# CI target (all checks including architecture)
ci: fmt lint test validate-arch
	@echo "CI checks passed!"

# ============================================
# Multi-Directory Validation Targets
# ============================================

# Validate workspace-next + legacy ../src (shows all issues)
validate-all:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║     mcb-validate: Workspace + Legacy Source Validation       ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@cargo test --package mcb-validate test_validation_with_legacy -- --nocapture 2>&1 | \
		grep -E "(Total Violations:|Dependency:|Quality:|Patterns:|Tests:|Documentation:|Naming:|SOLID:|Organization:|KISS:|DI/Shaku:|Refactoring:|Status:|\[Error\]|\[Warning\]|\[Info\])" | \
		head -40
	@echo ""
	@echo "Note: Legacy paths use Info severity level"

# Validate only legacy ../src (for comparison)
validate-legacy:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║          mcb-validate: Legacy Source Only                     ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@cargo test --package mcb-validate test_legacy_only -- --nocapture 2>&1 | \
		grep -E "(Total Violations:|Dependency:|Quality:|Patterns:|Tests:|Documentation:|Naming:|SOLID:|Organization:|KISS:|DI/Shaku:|Refactoring:|Status:|\[Error\]|\[Warning\]|\[Info\])" | \
		head -40

# Show current validation configuration
validate-config:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║          mcb-validate Configuration Test                     ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@cargo test --package mcb-validate test_validation_config -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

# ============================================
# Extended Validation Targets
# ============================================

validate-solid:
	@echo "=== SOLID Principle Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_solid -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-org:
	@echo "=== Organization Violations (file placement, centralization) ==="
	@cargo test --package mcb-validate test_validate_workspace_organization -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-kiss:
	@echo "=== KISS Violations (complexity) ==="
	@cargo test --package mcb-validate test_validate_workspace_kiss -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-shaku:
	@echo "=== DI/Shaku Violations ==="
	@cargo test --package mcb-validate test_validate_workspace_shaku -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"

validate-refactor:
	@echo "=== Refactoring Completeness Violations ==="
	@echo "Detects: orphan imports, duplicate definitions, missing tests, stale re-exports"
	@cargo test --package mcb-validate test_validate_workspace_refactoring -- --nocapture 2>&1 | \
		grep -v "^running\|^test \|Compiling\|Finished"
