# =============================================================================
# Quality
# =============================================================================

# Binary lookup chain: MCB_BIN env > PATH mcb > target/release > target/debug > cargo run
MCB_BIN ?= $(or $(shell command -v mcb 2>/dev/null),$(wildcard target/release/mcb),$(wildcard target/debug/mcb))
MCB_CMD := $(if $(MCB_BIN),$(MCB_BIN),cargo run --package mcb --)

.PHONY: fmt lint validate audit coverage
##@ Quality

fmt: ## Format Rust and Markdown (mutating)
	@echo "Formatting Rust..."
	@cargo fmt --all
	@echo "Formatting Markdown..."
	@$(MAKE) docs-lint FIX=1

lint: ## Check code quality (FIX=1 to auto-fix)
ifeq ($(FIX),1)
	@echo "Auto-fixing code..."
	cargo fmt
	cargo clippy --fix --allow-dirty --all-targets
else
	@echo "Checking code quality..."
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
endif

validate: ## Architecture validation (QUICK=1 for fast mode)
ifeq ($(QUICK),1)
	@echo "Quick architecture validation ($(MCB_CMD))..."
	@mkdir -p reports
	@$(MCB_CMD) validate . --quick --format json > reports/mcb-validate-internal-report.json
	@echo "Report generated: reports/mcb-validate-internal-report.json"
else
	@echo "Architecture validation ($(MCB_CMD))..."
	@mkdir -p reports
	@$(MCB_CMD) validate . --format json > reports/mcb-validate-internal-report.json
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

coverage: ## Code coverage (lcov output)
	@echo "Generating lcov coverage (excluding integration tests)..."
	cargo tarpaulin --out Lcov --output-dir coverage \
		--exclude-files 'crates/*/tests/integration/*' \
		--exclude-files 'crates/*/tests/admin/*' \
		--timeout 300 2>/dev/null || echo "Note: cargo-tarpaulin not installed"

qlty: ## Run qlty checks and smells analysis
	@echo "Running qlty checks and smells analysis..."
	@mkdir -p docs/reports
	@./scripts/analyze_qlty.py --scan --check --summary --markdown docs/reports/qlty-check-REPORTS.md
	@./scripts/analyze_qlty.py --scan --smells --summary --markdown docs/reports/qlty-smells-REPORTS.md
