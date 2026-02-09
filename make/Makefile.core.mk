# =============================================================================
# CORE - Build, test, docs, clean
# =============================================================================
# Parameters: RELEASE, SCOPE (from main Makefile)
# =============================================================================

.PHONY: build test test-rust test-e2e test-e2e-ui test-e2e-debug clean

# Test ports (avoid conflicts with production on 8080)
export MCP_PORT ?= 13001
export MCB_TEST_PORT ?= 18080

# Test thread count (parallelization - use fewer threads on CI to reduce timeout issues)
export TEST_THREADS ?= 0

# =============================================================================
# BUILD (RELEASE=1 for release)
# =============================================================================

build: ## Build project (RELEASE=1 for release)
ifeq ($(RELEASE),1)
	@echo "Building release..."
	cargo build --release
else
	@echo "Building debug..."
	cargo build
endif

# =============================================================================
# TEST (SCOPE=unit|doc|golden|all, TEST_THREADS=N for parallelization)
# =============================================================================

test: ## Run ALL tests (Rust unit/integration/golden + Playwright E2E)
	@echo "=================================================="
	@echo "🧪 Running COMPLETE Test Suite"
	@echo "=================================================="
	@echo ""
	@echo "Phase 1: Rust Tests (unit + integration + golden)"
	@echo "--------------------------------------------------"
	@$(MAKE) test-rust
	@echo ""
	@echo "Phase 2: E2E Tests (Playwright)"
	@echo "--------------------------------------------------"
	@$(MAKE) test-e2e
	@echo ""
	@echo "=================================================="
	@echo "✅ ALL TESTS PASSED"
	@echo "=================================================="

test-rust: ## Run all Rust tests (unit + integration + golden)
	@echo "Running Rust test suite..."
	MCP_PORT=$(MCP_PORT) cargo test --workspace --all-targets

# =============================================================================
# E2E Tests (Playwright)
# =============================================================================

test-e2e: ## Run E2E tests with Playwright (auto-installs if needed)
	@echo "🎭 Running Playwright E2E tests on port $(MCB_TEST_PORT)..."
	@if ! command -v npx > /dev/null; then \
		echo "❌ Error: npx not found. Install Node.js first."; \
		exit 1; \
	fi
	@if [ ! -d node_modules/@playwright ]; then \
		echo "📦 Installing Playwright..."; \
		npm install --save-dev @playwright/test @types/node typescript 2>&1 | grep -v "npm WARN" || true; \
		npx playwright install chromium --with-deps 2>&1 | tail -5; \
	fi
	@echo "🚀 Starting test server on port $(MCB_TEST_PORT)..."
	@MCB_TEST_PORT=$(MCB_TEST_PORT) npx playwright test --reporter=list

test-e2e-ui: ## Run E2E tests with Playwright UI (interactive)
	@echo "🎭 Running Playwright E2E tests in UI mode..."
	@MCB_TEST_PORT=$(MCB_TEST_PORT) npx playwright test --ui

test-e2e-debug: ## Run E2E tests with Playwright debug mode
	@echo "🐛 Running Playwright E2E tests in debug mode..."
	@MCB_TEST_PORT=$(MCB_TEST_PORT) npx playwright test --debug

test-e2e-report: ## Show last E2E test report
	@npx playwright show-report

# =============================================================================
# CLEAN
# =============================================================================

clean: ## Clean all build artifacts
	@echo "Cleaning..."
	cargo clean
	@echo "Clean complete"
