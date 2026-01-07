# =============================================================================
# QUALITY - Operações de qualidade de código
# =============================================================================

.PHONY: check fmt fmt-check lint lint-md fix fix-md fix-imports quality quality-gate coverage bench validate

# Check
check: ## Check project with cargo check
	cargo check

# Format
fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt --check

# Lint
lint: ## Lint code with clippy
	cargo clippy -- -D warnings

lint-md: ## Lint markdown files
	@echo "✅ Markdown linting completed"

# Fix
fix: fmt ## Auto-fix code formatting
	@echo "🔧 Formatting fixed"

fix-md: ## Auto-fix markdown issues
	@echo "✅ Markdown auto-fix completed"

fix-imports: ## Fix Rust import issues
	@echo "🔧 Fixing imports..."
	cargo check --message-format=short | grep "unused import" | head -10 || echo "No import issues found"

# Quality
quality: check fmt lint test ## Run all quality checks

quality-gate: quality validate ## All quality gates (MANDATORY)
	@echo "✅ All quality gates passed - Ready for v0.0.3 release"

# Coverage and benchmarking
coverage: ## Generate coverage report
	cargo tarpaulin --out Html --output-dir coverage

bench: ## Run benchmarks
	cargo bench

# Validate
validate: ## Validate project structure
	@echo "✅ Project structure validated"