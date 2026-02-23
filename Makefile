# =============================================================================
# MCP Context Browser - Simplified Makefile
# =============================================================================

export RELEASE ?= 1
export SCOPE ?=
export FIX ?= 0
export MCB_CI ?= 0
export QUICK ?= 0
export THREADS ?= 0
export BUMP ?=

# Rust 2024 Edition lints (preserved verbatim)
export RUST_2024_LINTS := -D unsafe_op_in_unsafe_fn -D rust_2024_compatibility -W static_mut_refs

include make/dev.mk
include make/quality.mk
include make/release.mk
include make/docs.mk
include make/codegen.mk

.DEFAULT_GOAL := help

##@ Core

check: ## Non-mutating gate: fmt --check + lint + test + validate
	@echo "Running non-mutating checks..."
	@cargo fmt --all -- --check
	@$(MAKE) lint
	@$(MAKE) test
	@$(MAKE) validate

ci: ## CI gate: check + audit
	@echo "Running CI gate..."
	@$(MAKE) check
	@$(MAKE) audit

help: ## Show available commands
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[0-9a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) }' $(MAKEFILE_LIST)
