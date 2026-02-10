# =============================================================================
# QUALITY - Lint, validate, audit, coverage
# =============================================================================
# Parameters: FIX, CI_MODE, STRICT, QUICK, LCOV (from main Makefile)
# =============================================================================

.PHONY: check quality fmt lint validate audit coverage update qlty-check qlty-smells

# =============================================================================
# QUALITY - Full check (fmt + lint + test)
# =============================================================================

quality: ## Full check: fmt + lint + test (pre-commit gate)
	@echo "Running quality checks (fmt + lint + test)..."
	@$(MAKE) fmt
	@$(MAKE) lint
	@$(MAKE) test SCOPE=all
	@echo "Quality checks passed."

check: ## Non-mutating check: lint + test
	@echo "Running checks (lint + test)..."
	@$(MAKE) lint
	@$(MAKE) test SCOPE=all
	@echo "Checks passed."

# =============================================================================
# FMT - Format Rust and Markdown
# =============================================================================

fmt: ## Format Rust and Markdown (cargo fmt + markdownlint -f)
	@echo "Formatting Rust..."
	@cargo fmt --all
	@echo "Formatting Markdown..."
	@$(MAKE) docs-lint FIX=1
	@echo "Format complete"

# =============================================================================
# LINT (FIX=1 to auto-fix, CI_MODE=1 for Rust 2024 strict)
# =============================================================================

lint: ## Check code quality (FIX=1 to auto-fix, CI_MODE=1 for Rust 2024)
ifeq ($(FIX),1)
	@echo "Auto-fixing code..."
	cargo fmt
	cargo clippy --fix --allow-dirty --all-targets
else ifeq ($(CI_MODE),1)
	@echo "CI lint with Rust 2024 checks..."
	cargo fmt --all -- --check
	RUSTFLAGS="$(RUST_2024_LINTS)" cargo clippy --all-targets -- -D warnings \
		-D clippy::multiple_unsafe_ops_per_block \
		-D clippy::undocumented_unsafe_blocks
else
	@echo "Checking code quality..."
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
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
	cargo audit \
		--ignore RUSTSEC-2023-0071 \
		--ignore RUSTSEC-2023-0089 \
		--ignore RUSTSEC-2025-0119 \
		--ignore RUSTSEC-2024-0436 \
		--ignore RUSTSEC-2025-0134
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
		--timeout 300
else
	@echo "Generating HTML coverage (excluding integration tests)..."
	cargo tarpaulin --out Html --output-dir coverage \
		--exclude-files 'crates/*/tests/integration/*' \
		--exclude-files 'crates/*/tests/admin/*' \
		--timeout 300 2>/dev/null || echo "Note: cargo-tarpaulin not installed"
endif

# =============================================================================
# UPDATE - Update dependencies
# =============================================================================

update: ## Update Cargo dependencies
	@echo "Updating dependencies..."
	cargo update
	@echo "Dependencies updated"

# =============================================================================
# QLTY-CHECK - Run qlty check and analyze results
# =============================================================================

qlty-check: ## Run qlty check and analyze results
	@echo "Running qlty check..."
	@qlty check --all --sarif > qlty.check.lst || true
	@echo "Analyzing results..."
	@python3 scripts/analyze_checks.py
	@echo "Done. Results saved to qlty.check.lst"

qlty-smells: ## Run qlty smells and analyze results
	@echo "Running qlty smells..."
	@qlty smells --all --sarif > qlty.smells.lst || true
	@echo "Analyzing smells..."
	@python3 scripts/analyze_smells.py
	@echo "Done. Results saved to qlty.smells.lst"

