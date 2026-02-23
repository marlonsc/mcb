# =============================================================================
# Development
# =============================================================================

.PHONY: build test clean update dev docker-up docker-down docker-logs _test-e2e

# Test ports (avoid conflicts with production on 8080)
export MCB_TEST_PORT ?= 18080

##@ Core

build: ## Build project (RELEASE=1 for release)
ifeq ($(RELEASE),1)
	@echo "Building release..."
	cargo build --release
else
	@echo "Building debug..."
	cargo build
endif

test: ## Run tests (SCOPE=unit|doc|golden|startup|integration|e2e|all, THREADS=N)
	@THREADS="$(THREADS)"; case "$$THREADS" in ''|*[!0-9]*|0) THREADS=1;; esac; \
	case "$(SCOPE)" in \
	  unit)        RUST_TEST_THREADS=$$THREADS cargo test --workspace --lib ;; \
	  doc)         cargo test --workspace --doc ;; \
	  golden)      RUST_TEST_THREADS=$$THREADS cargo test --workspace --tests golden ;; \
	  startup)     cargo test -p mcb --test integration startup_smoke -- --nocapture ;; \
	  integration) RUST_TEST_THREADS=$$THREADS cargo test --workspace --test '*integration*' ;; \
	  e2e)         make _test-e2e ;; \
	  all)         RUST_TEST_THREADS=$$THREADS cargo test --workspace --all-targets && make _test-e2e ;; \
	  '')          RUST_TEST_THREADS=$$THREADS cargo test --workspace --all-targets ;; \
	  *)           echo "Unknown SCOPE '$(SCOPE)'. Valid: unit|doc|golden|startup|integration|e2e|all"; exit 2 ;; \
	esac

clean: ## Clean build artifacts
	@echo "Cleaning..."
	cargo clean

update: ## Update Cargo dependencies
	@echo "Updating dependencies..."
	cargo update

##@ Development

dev: ## Development server with auto-reload
	@echo "Starting development server..."
	cargo watch -x 'run' 2>/dev/null || cargo run

docker-up: ## Start Docker test services
	@echo "Starting Docker test services..."
	docker-compose -f tests/docker-compose.yml up -d
	@sleep 5

docker-down: ## Stop Docker test services
	@echo "Stopping Docker test services..."
	docker-compose -f tests/docker-compose.yml down -v

docker-logs: ## View Docker service logs
	docker-compose -f tests/docker-compose.yml logs -f

_test-e2e:
	@echo "Running Playwright E2E tests on port $(MCB_TEST_PORT)..."
	@lsof -ti:$(MCB_TEST_PORT) | xargs -r kill -9 2>/dev/null || true
	@sleep 1
	@if ! command -v npx > /dev/null; then \
		echo "Error: npx not found. Install Node.js first."; \
		exit 1; \
	fi
	@if [ ! -d tests/node_modules/@playwright ]; then \
		echo "Installing Playwright..."; \
		npm --prefix tests install --save-dev @playwright/test @types/node typescript 2>&1 | grep -v "npm WARN" || true; \
		(cd tests && npx playwright install chromium --with-deps 2>&1 | tail -5); \
	fi
	@cargo build --release --bin mcb
	@MCB_TEST_PORT=$(MCB_TEST_PORT) tests/node_modules/.bin/playwright test --config=tests/playwright.config.ts --reporter=list
