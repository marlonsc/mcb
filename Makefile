# MCP Context Browser - Development Makefile

.PHONY: help build test run clean check lint format doc release install deps update

# Default target
help: ## Show this help message
	@echo "MCP Context Browser - Development Commands"
	@echo ""
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

# Build commands
build: ## Build the project in release mode
	cargo build --release

build-dev: ## Build the project in development mode
	cargo build

# Testing
test: ## Run all tests
	cargo test

test-unit: ## Run unit tests only
	cargo test --lib

test-integration: ## Run integration tests
	cargo test --test integration

# Running
run: ## Run the application
	cargo run

run-release: ## Run the application in release mode
	cargo run --release

# Code quality
check: ## Check code without building
	cargo check

lint: ## Run clippy linter
	cargo clippy -- -D warnings

format: ## Format code with rustfmt
	cargo fmt

format-check: ## Check if code is formatted
	cargo fmt --check

# Documentation
doc: ## Generate documentation
	cargo doc --open --no-deps

# Dependencies
deps: ## Show dependency tree
	cargo tree

update: ## Update dependencies
	cargo update

# Cleaning
clean: ## Clean build artifacts
	cargo clean

clean-all: ## Clean everything including Cargo.lock
	cargo clean
	rm -f Cargo.lock

# Development workflow
dev: format lint test build ## Run full development workflow

# Release
release: clean format lint test build ## Prepare for release

# Installation
install: ## Install the binary
	cargo install --path .

# Docker (future)
docker-build: ## Build Docker image
	docker build -t mcp-context-browser .

docker-run: ## Run Docker container
	docker run --rm -it mcp-context-browser

# Git operations
git-status: ## Show git status
	git status

git-add: ## Add all files to git
	git add .

git-commit: ## Commit with message (usage: make git-commit MSG="your message")
	@echo "Committing with message: $(MSG)"
	git commit -m "$(MSG)"

git-push: ## Push to remote repository
	GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=no" git push origin main

git-force-push: ## Force push to remote repository (use with caution)
	GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=no" git push -f origin main

# Version management
version-show: ## Show current version
	@grep '^version' Cargo.toml | head -1 | cut -d'"' -f2

version-bump-patch: ## Bump patch version (0.0.1 -> 0.0.2)
	@CURRENT=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
	PATCH=$$(echo $$CURRENT | cut -d. -f3); \
	NEW_PATCH=$$(expr $$PATCH + 1); \
	NEW_VERSION=$$(echo $$CURRENT | sed "s/\.[0-9]*$$/.$$NEW_PATCH/"); \
	sed -i "s/version = \"$$CURRENT\"/version = \"$$NEW_VERSION\"/" Cargo.toml; \
	echo "Version bumped: $$CURRENT -> $$NEW_VERSION"

version-bump-minor: ## Bump minor version (0.0.1 -> 0.1.0)
	@CURRENT=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
	MINOR=$$(echo $$CURRENT | cut -d. -f2); \
	NEW_MINOR=$$(expr $$MINOR + 1); \
	NEW_VERSION=$$(echo $$CURRENT | sed "s/\.[0-9]*\.[0-9]*$$/.$$NEW_MINOR.0/"); \
	sed -i "s/version = \"$$CURRENT\"/version = \"$$NEW_VERSION\"/" Cargo.toml; \
	echo "Version bumped: $$CURRENT -> $$NEW_VERSION"

version-bump-major: ## Bump major version (0.0.1 -> 1.0.0)
	@CURRENT=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
	MAJOR=$$(echo $$CURRENT | cut -d. -f1); \
	NEW_MAJOR=$$(expr $$MAJOR + 1); \
	NEW_VERSION="$$NEW_MAJOR.0.0"; \
	sed -i "s/version = \"$$CURRENT\"/version = \"$$NEW_VERSION\"/" Cargo.toml; \
	echo "Version bumped: $$CURRENT -> $$NEW_VERSION"

# Release management
release-patch: version-bump-patch release-commit release-tag release-push ## Create patch release
release-minor: version-bump-minor release-commit release-tag release-push ## Create minor release
release-major: version-bump-major release-commit release-tag release-push ## Create major release

release-commit: ## Commit version changes
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
	git add Cargo.toml; \
	git commit -m "chore: bump version to $$VERSION"

release-tag: ## Create git tag for current version
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
	git tag -a "v$$VERSION" -m "Release version $$VERSION"

release-push: ## Push commits and tags
	GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=no" git push origin main --tags

# Package management
package-build: ## Build release package
	cargo build --release

package-tar: ## Create source tarball
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
	tar -czf mcp-context-browser-$$VERSION.tar.gz --exclude='.git' --exclude='target' --exclude='*.bak' .

package-zip: ## Create source zip
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
	zip -r mcp-context-browser-$$VERSION.zip . -x '.git/*' 'target/*' '*.bak'

# GitHub release (requires gh CLI)
github-release: ## Create GitHub release
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
	gh release create "v$$VERSION" --title "MCP Context Browser v$$VERSION" --generate-notes

# Full workflow
workflow-dev: format lint test build ## Run full development workflow
workflow-release: workflow-dev package-build package-tar github-release ## Run full release workflow

# Project info
info: ## Show project information
	@echo "MCP Context Browser v$(shell cargo pkgid | cut -d# -f2 | cut -d: -f2)"
	@echo "Rust version: $(shell rustc --version)"
	@echo "Cargo version: $(shell cargo --version)"

# Performance
bench: ## Run benchmarks
	cargo bench

flamegraph: ## Generate flamegraph (requires cargo-flamegraph)
	cargo flamegraph --bin mcp-context-browser

# Coverage (requires tarpaulin)
coverage: ## Generate test coverage report
	cargo tarpaulin --out Html

# CI/CD simulation
ci: clean format-check lint test build ## Simulate CI pipeline