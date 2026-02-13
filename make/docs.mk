# =============================================================================
# Docs
# =============================================================================

.PHONY: docs docs-serve docs-lint docs-validate adr adr-new diagrams docs-check docs-setup docs-sync docs-build rust-docs

MDBOOK := $(shell command -v mdbook 2>/dev/null || echo "$(HOME)/.cargo/bin/mdbook")

##@ Docs

docs: ## Build all documentation
	@echo "Building documentation..."
	@./scripts/docs/inject-metrics.sh
	@cargo doc --no-deps --workspace
	@./scripts/docs/mdbook-sync.sh
	@if [ -x "$(MDBOOK)" ]; then $(MDBOOK) build book/; else echo "Warning: mdbook not found, skipping book build" >&2; fi

docs-serve: ## Serve documentation with live reload
	@echo "Starting documentation server..."
	@./scripts/docs/mdbook-sync.sh 2>/dev/null || true
	@if [ -x "$(MDBOOK)" ]; then $(MDBOOK) serve book/ --open; else echo "mdbook not installed (cargo install mdbook)"; fi

docs-lint: ## Lint markdown files (FIX=1 to auto-fix)
ifeq ($(FIX),1)
	@echo "Auto-fixing markdown issues..."
	@./scripts/docs/markdown.sh fix
else
	@echo "Checking markdown files..."
	@./scripts/docs/markdown.sh lint
endif

docs-validate: ## Validate documentation (QUICK=1 skips external links)
	@echo "Validating documentation..."
	@QUICK="$(QUICK)" ./scripts/docs/validate.sh all

adr: ## List Architecture Decision Records
	@echo "Architecture Decision Records:"
	@ls -1 docs/adr/[0-9]*.md 2>/dev/null | while read f; do \
		num=$$(basename "$$f" .md | cut -d- -f1); \
		title=$$(head -1 "$$f" | sed 's/^# ADR [0-9]*: //'); \
		printf "  %s: %s\n" "$$num" "$$title"; \
	done

adr-new: ## Create new ADR
	@./scripts/docs/create-adr.sh 2>/dev/null || echo "create-adr.sh not found"

diagrams: ## Generate architecture diagrams with PlantUML
	@mkdir -p docs/architecture/diagrams/generated
	@if command -v plantuml >/dev/null 2>&1; then \
		for f in docs/architecture/diagrams/*.puml; do \
			if [ -f "$$f" ]; then \
				plantuml -o generated "$$f" 2>/dev/null || true; \
			fi; \
		done; \
	fi

##@ Internal

docs-check:
	@if [ ! -d "docs" ]; then echo "ERROR: docs/ directory not found"; exit 1; fi

docs-setup:
	@mkdir -p book
	@if [ ! -f "book.toml" ]; then echo "ERROR: book.toml not found in root"; exit 1; fi

docs-sync:
	@./scripts/docs/mdbook-sync.sh 2>/dev/null || true

docs-build:
	@if [ -x "$(MDBOOK)" ]; then $(MDBOOK) build book/ 2>/dev/null || true; fi

rust-docs:
	@cargo doc --no-deps --workspace
