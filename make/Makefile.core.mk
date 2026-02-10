# =============================================================================
# CORE - Build, test, docs, clean
# =============================================================================
# Parameters: RELEASE, SCOPE (from main Makefile)
# =============================================================================

.PHONY: build test test-rust test-startup test-e2e test-e2e-ui test-e2e-debug clean

# Test ports (avoid conflicts with production on 8080)
export MCB_TEST_PORT ?= 18080

# Test thread count (set to 1 for deterministic env-dependent tests)
export TEST_THREADS ?= 1

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
	@THREADS="$(TEST_THREADS)"; \
	case "$$THREADS" in ''|*[!0-9]*|0) THREADS=1;; esac; \
	RUST_TEST_THREADS=$$THREADS cargo test --workspace --all-targets

test-startup: ## Run startup smoke tests (DDL/init failure detection)
	@echo "Running startup smoke tests..."
	@cargo test -p mcb --test startup_smoke_integration -- --nocapture

# =============================================================================
# E2E Tests (Playwright)
# =============================================================================

test-e2e: ## Run E2E tests with Playwright (auto-installs if needed)
	@echo "🎭 Running Playwright E2E tests on port $(MCB_TEST_PORT)..."
	@echo "🧹 Cleaning up test server on port $(MCB_TEST_PORT)..."
	@lsof -ti:$(MCB_TEST_PORT) | xargs -r kill -9 2>/dev/null || true
	@sleep 1
	@if ! command -v npx > /dev/null; then \
		echo "❌ Error: npx not found. Install Node.js first."; \
		exit 1; \
	fi
	@if [ ! -d tests/node_modules/@playwright ]; then \
		echo "📦 Installing Playwright..."; \
		npm --prefix tests install --save-dev @playwright/test @types/node typescript 2>&1 | grep -v "npm WARN" || true; \
		(cd tests && npx playwright install chromium --with-deps 2>&1 | tail -5); \
	fi
	@echo "🏗️ Building release binary once for E2E runs..."
	@cargo build --release --bin mcb
	@echo "🚀 Running Playwright specs sequentially on port $(MCB_TEST_PORT)..."
	@for spec in tests/e2e/*.spec.ts; do \
		echo "▶ Running $$spec"; \
		MCB_TEST_PORT=$(MCB_TEST_PORT) tests/node_modules/.bin/playwright test --config=tests/playwright.config.ts --reporter=list "$$spec" || exit 1; \
	done

test-e2e-ui: ## Run E2E tests with Playwright UI (interactive)
	@echo "🎭 Running Playwright E2E tests in UI mode..."
	@MCB_TEST_PORT=$(MCB_TEST_PORT) tests/node_modules/.bin/playwright test --config=tests/playwright.config.ts --ui

test-e2e-debug: ## Run E2E tests with Playwright debug mode
	@echo "🐛 Running Playwright E2E tests in debug mode..."
	@MCB_TEST_PORT=$(MCB_TEST_PORT) tests/node_modules/.bin/playwright test --config=tests/playwright.config.ts --debug

test-e2e-report: ## Show last E2E test report
	@tests/node_modules/.bin/playwright show-report

# =============================================================================
# CLEAN
# =============================================================================

clean: ## Clean all build artifacts
	@echo "Cleaning..."
	cargo clean
	@echo "Clean complete"
