# =============================================================================
# Quality
# =============================================================================

.PHONY: fmt lint validate audit coverage

##@ Quality

fmt: ## Format Rust and Markdown (mutating)
	@echo "Formatting Rust..."
	@cargo fmt --all
	@echo "Formatting Markdown..."
	@$(MAKE) docs-lint FIX=1

lint: ## Check code quality (FIX=1 to fix, MCB_CI=1 for strict)
ifeq ($(FIX),1)
	@echo "Auto-fixing code..."
	cargo fmt
	cargo clippy --fix --allow-dirty --all-targets
else ifeq ($(MCB_CI),1)
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

validate: ## Architecture validation (QUICK=1 for fast mode)
ifeq ($(QUICK),1)
	@echo "Quick architecture validation (internal mcb config)..."
	@mkdir -p reports
	@cargo run --package mcb -- validate . --quick --format json > reports/mcb-validate-internal-report.json
	@echo "Report generated: reports/mcb-validate-internal-report.json"
else
	@echo "Architecture validation (internal mcb config)..."
	@mkdir -p reports
	@cargo run --package mcb -- validate . --format json > reports/mcb-validate-internal-report.json
	@echo "Report generated: reports/mcb-validate-internal-report.json"
endif

audit: ## Security audit (cargo-audit)
	@echo "Running security audit..."
	cargo audit \
		--ignore RUSTSEC-2023-0071 \
		--ignore RUSTSEC-2023-0089 \
		--ignore RUSTSEC-2025-0119 \
		--ignore RUSTSEC-2024-0436 \
		--ignore RUSTSEC-2025-0134
	@cargo udeps --workspace 2>/dev/null || echo "Note: cargo-udeps not installed"

coverage: ## Code coverage (MCB_CI=1 for lcov output)
ifeq ($(MCB_CI),1)
	@echo "Generating lcov coverage (excluding integration tests)..."
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

qlty: ## Run qlty checks and smells analysis
	@echo "Running qlty checks and smells analysis..."
	@mkdir -p docs/reports
	@./scripts/analyze_qlty.py --scan --check --summary --markdown docs/reports/qlty-check-REPORTS.md
	@./scripts/analyze_qlty.py --scan --smells --summary --markdown docs/reports/qlty-smells-REPORTS.md
