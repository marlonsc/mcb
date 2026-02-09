# =============================================================================
# CORE - Build, test, docs, clean
# =============================================================================
# Parameters: RELEASE, SCOPE (from main Makefile)
# =============================================================================

.PHONY: build test test-e2e test-e2e-ui test-e2e-debug clean

# Test port (avoids conflicts with dev server on 3001)
export MCP_PORT ?= 13001

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

test: ## Run tests (SCOPE=unit|doc|golden|integration|modes|e2e|all, TEST_THREADS=N to limit parallelization)
ifeq ($(TEST_THREADS),0)
	# Use default thread count
	TEST_THREADS_FLAG=
else
	# Limit to specified number of threads
	TEST_THREADS_FLAG=--test-threads=$(TEST_THREADS)
endif
ifeq ($(SCOPE),unit)
	@echo "Running unit tests..."
	MCP_PORT=$(MCP_PORT) cargo test --workspace --lib --bins $(TEST_THREADS_FLAG)
else ifeq ($(SCOPE),doc)
	@echo "Running doctests..."
	MCP_PORT=$(MCP_PORT) cargo test --doc --workspace $(TEST_THREADS_FLAG)
else ifeq ($(SCOPE),golden)
	@echo "Running golden tests (acceptance + tools e2e + E2E)..."
	MCP_PORT=$(MCP_PORT) cargo test -p mcb-server golden -- --nocapture $(TEST_THREADS_FLAG)
else ifeq ($(SCOPE),integration)
	@echo "Running integration tests..."
	MCP_PORT=$(MCP_PORT) cargo test -p mcb-server --test integration $(TEST_THREADS_FLAG)
else ifeq ($(SCOPE),modes)
	@echo "Running operating modes tests..."
	MCP_PORT=$(MCP_PORT) cargo test -p mcb-server operating_modes -- --nocapture $(TEST_THREADS_FLAG)
else ifeq ($(SCOPE),e2e)
	@echo "Running E2E tests (Playwright)..."
	@$(MAKE) test-e2e
else
	@echo "Running all tests..."
	MCP_PORT=$(MCP_PORT) cargo test --workspace $(TEST_THREADS_FLAG)
endif

# =============================================================================
# E2E Tests (Playwright)
# =============================================================================

test-e2e: ## Run E2E tests with Playwright
	@echo "🎭 Running Playwright E2E tests..."
	@if ! command -v npx > /dev/null; then \
		echo "❌ Error: npx not found. Install Node.js and npm first."; \
		exit 1; \
	fi
	@if [ ! -f package.json ]; then \
		echo "⚠️  Warning: package.json not found. Run 'npm init -y' first."; \
		exit 1; \
	fi
	@if [ ! -d node_modules/@playwright ]; then \
		echo "📦 Installing Playwright..."; \
		npm install --save-dev @playwright/test; \
		npx playwright install chromium; \
	fi
	@echo "🚀 Starting MCB server for E2E tests..."
	@MCB_BASE_URL=http://localhost:8080 npx playwright test

test-e2e-ui: ## Run E2E tests with Playwright UI
	@echo "🎭 Running Playwright E2E tests in UI mode..."
	@MCB_BASE_URL=http://localhost:8080 npx playwright test --ui

test-e2e-debug: ## Run E2E tests with Playwright debug mode
	@echo "🐛 Running Playwright E2E tests in debug mode..."
	@MCB_BASE_URL=http://localhost:8080 npx playwright test --debug

# =============================================================================
# CLEAN
# =============================================================================

clean: ## Clean all build artifacts
	@echo "Cleaning..."
	cargo clean
	@echo "Clean complete"
