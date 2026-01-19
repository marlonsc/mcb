# =============================================================================
# MAINTENANCE - Health checks, updates and monitoring
# =============================================================================
# Note: `audit` is in quality.mk, `status` is in git.mk
# =============================================================================

.PHONY: update health maintain verify env-check
.PHONY: metrics metrics-test sync-test daemon-test dashboard

# -----------------------------------------------------------------------------
# Dependency Management
# -----------------------------------------------------------------------------

update: ## Update all Cargo dependencies
	@echo "🔄 Updating dependencies..."
	@cargo update
	@echo "✅ Dependencies updated"

# -----------------------------------------------------------------------------
# Health Checks
# -----------------------------------------------------------------------------

health: check test-unit ## Health check (compile + unit tests)
	@echo "✅ Health check passed"

# -----------------------------------------------------------------------------
# Feature Testing (v0.0.3+)
# -----------------------------------------------------------------------------

metrics: ## Start metrics HTTP server (port 3001)
	@echo "📊 Starting metrics server..."
	@cargo run -- --metrics

metrics-test: ## Test metrics collection
	@cargo test --test metrics 2>/dev/null || cargo test metrics

sync-test: ## Test cross-process synchronization
	@cargo test --test sync 2>/dev/null || cargo test sync

daemon-test: ## Test background daemon
	@cargo test daemon

dashboard: ## Open metrics dashboard in browser
	@echo "🌐 Opening http://localhost:3001"
	@python3 -m webbrowser http://localhost:3001 2>/dev/null || \
		xdg-open http://localhost:3001 2>/dev/null || \
		echo "Open http://localhost:3001 in your browser"

env-check: ## Validate environment configuration
	@cargo run -- --env-check 2>/dev/null || echo "⚠️  --env-check not available"

# -----------------------------------------------------------------------------
# Maintenance Workflows
# -----------------------------------------------------------------------------

maintain: update audit clean ## Full maintenance cycle (update + audit + clean)
	@echo "✅ Maintenance complete"
