# =============================================================================
# QUALITY - Lint, validate, audit, coverage
# =============================================================================
# Parameters: FIX, CI_MODE, STRICT, QUICK, LCOV (from main Makefile)
# =============================================================================

.PHONY: lint validate audit coverage update

# =============================================================================
# LINT (FIX=1 to auto-fix, CI_MODE=1 for Rust 2024 strict)
# =============================================================================

lint: ## Check code quality (FIX=1 to auto-fix, CI_MODE=1 for Rust 2024)
ifeq ($(FIX),1)
	@echo "Auto-fixing code..."
	cargo fmt
	cargo clippy --fix --allow-dirty --all-targets --features "full"
else ifeq ($(CI_MODE),1)
	@echo "CI lint with Rust 2024 checks..."
	cargo fmt --all -- --check
	RUSTFLAGS="$(RUST_2024_LINTS)" cargo clippy --all-targets --features "full" -- -D warnings \
		-D clippy::multiple_unsafe_ops_per_block \
		-D clippy::undocumented_unsafe_blocks
else
	@echo "Checking code quality..."
	cargo fmt --all -- --check
	cargo clippy --all-targets --features "full" -- -D warnings
endif

# =============================================================================
# VALIDATE (STRICT=1, QUICK=1)
# =============================================================================

validate: ## Architecture validation (STRICT=1, QUICK=1)
ifeq ($(QUICK),1)
	@echo "Quick architecture validation..."
	@cargo test --package mcb-validate test_full_validation_report -- --nocapture 2>&1 | \
		grep -E "(Total Violations:|Status:|\[Error\])" | head -20
else ifeq ($(STRICT),1)
	@echo "Strict architecture validation..."
	cargo test --package mcb-validate test_full_validation_report -- --nocapture
else
	@echo "Architecture validation..."
	cargo test --package mcb-validate test_full_validation_report -- --nocapture
endif

# =============================================================================
# AUDIT
# =============================================================================

audit: ## Security audit (cargo-audit)
	@echo "Running security audit..."
	cargo audit
	@cargo udeps --workspace 2>/dev/null || echo "Note: cargo-udeps not installed"

# =============================================================================
# COVERAGE (LCOV=1 for CI format)
# Excludes integration tests to prevent timeouts from external dependencies
# (Milvus, Ollama) that aren't available in CI environments
# =============================================================================

coverage: ## Code coverage (LCOV=1 for CI format)
ifeq ($(LCOV),1)
	@echo "Generating LCOV coverage (excluding integration tests)..."
	cargo tarpaulin --out Lcov --output-dir coverage \
		--exclude-files 'crates/*/tests/integration/*' \
		--exclude-files 'crates/*/tests/admin/*' \
		--timeout 300 \
		--test-threads $(if $(TEST_THREADS),$(TEST_THREADS),4)
else
	@echo "Generating HTML coverage (excluding integration tests)..."
	cargo tarpaulin --out Html --output-dir coverage \
		--exclude-files 'crates/*/tests/integration/*' \
		--exclude-files 'crates/*/tests/admin/*' \
		--timeout 300 \
		--test-threads $(if $(TEST_THREADS),$(TEST_THREADS),4) 2>/dev/null || echo "Note: cargo-tarpaulin not installed"
endif

# =============================================================================
# UPDATE - Update dependencies
# =============================================================================

update: ## Update Cargo dependencies
	@echo "Updating dependencies..."
	cargo update
	@echo "Dependencies updated"
