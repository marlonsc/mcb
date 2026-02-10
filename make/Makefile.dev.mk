# =============================================================================
# DEVELOPMENT - Dev server and Docker integration
# =============================================================================
# Each verb does ONE action.
# =============================================================================

.PHONY: dev docker docker-up docker-down docker-logs install-hooks

# =============================================================================
# DEV - Development server with watch mode
# =============================================================================

dev: ## Development server with auto-reload
	@echo "Starting development server..."
	cargo watch -x 'run' 2>/dev/null || cargo run

# =============================================================================
# DOCKER - Docker service operations
# =============================================================================

docker: ## Show Docker services status
	@echo "Docker Services:"
	@docker-compose -f tests/docker-compose.yml ps 2>/dev/null || echo "No tests/docker-compose.yml found or Docker not running"

docker-up: ## Start Docker test services
	@echo "Starting Docker test services..."
	docker-compose -f tests/docker-compose.yml up -d
	@sleep 5
	@echo "Services started"

docker-down: ## Stop Docker test services
	@echo "Stopping Docker test services..."
	docker-compose -f tests/docker-compose.yml down -v
	@echo "Services stopped"

docker-logs: ## View Docker service logs
	docker-compose -f tests/docker-compose.yml logs -f

# =============================================================================
# INSTALL-HOOKS - Install git hooks
# =============================================================================

install-hooks: ## Install git pre-commit hook
	@echo "Installing git hooks..."
	@cp scripts/hooks/pre-commit .git/hooks/
	@chmod +x .git/hooks/pre-commit
	@echo "Pre-commit hook installed"
