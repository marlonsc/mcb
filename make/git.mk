# =============================================================================
# Git & GitHub
# =============================================================================

.PHONY: status diff log commit push pull pr pr-checks pr-view submodule-status submodule-sync submodule-push

# Configurable
BRANCH ?= $(shell git rev-parse --abbrev-ref HEAD)
FILES ?=
MSG ?=
PR ?=

##@ Git

status: ## Git status (main + submodules)
	@echo "=== Main repo ==="
	@git status --short
	@echo ""
	@echo "=== Submodules ==="
	@git submodule foreach --quiet 'STATUS=$$(git status --short); if [ -n "$$STATUS" ]; then echo "$$name:"; echo "$$STATUS"; fi'

diff: ## Git diff (staged + unstaged)
	@git diff
	@git diff --cached

log: ## Recent commits (LOG_N=10)
	@git log --oneline -$(or $(LOG_N),10)

commit: ## Commit staged files (MSG=, FILES= optional)
	@if [ -z "$(MSG)" ]; then echo "Error: MSG is required. Usage: make commit MSG='your message' FILES='file1 file2'"; exit 1; fi
	@if [ -n "$(FILES)" ]; then git add $(FILES); fi
	@git commit -m "$(MSG)"

push: ## Push current branch to origin
	@git push origin $(BRANCH)

pull: ## Pull current branch from origin
	@git pull origin $(BRANCH)

##@ GitHub

pr-checks: ## Show PR check status (PR=number)
	@if [ -z "$(PR)" ]; then echo "Error: PR is required. Usage: make pr-checks PR=116"; exit 1; fi
	@gh pr checks $(PR)

pr-view: ## View PR details (PR=number)
	@if [ -z "$(PR)" ]; then echo "Error: PR is required. Usage: make pr-view PR=116"; exit 1; fi
	@gh pr view $(PR)

pr-merge: ## Merge PR with merge commit (PR=number)
	@if [ -z "$(PR)" ]; then echo "Error: PR is required. Usage: make pr-merge PR=116"; exit 1; fi
	@gh pr merge $(PR) --merge

pr-rerun: ## Rerun failed CI jobs (RUN=run_id)
	@if [ -z "$(RUN)" ]; then echo "Error: RUN is required. Usage: make pr-rerun RUN=12345"; exit 1; fi
	@gh run rerun $(RUN) --failed

##@ Submodules

submodule-status: ## Show submodule commit status
	@git submodule status

submodule-sync: ## Sync submodule URLs from .gitmodules
	@git submodule sync --recursive
	@git submodule update --init --recursive

submodule-diff: ## Show diffs in all modified submodules
	@git submodule foreach --quiet 'DIFF=$$(git diff); if [ -n "$$DIFF" ]; then echo "=== $$name ==="; git diff; fi'

submodule-commit: ## Commit changes in a submodule (SUB=name MSG=)
	@if [ -z "$(SUB)" ]; then echo "Error: SUB is required. Usage: make submodule-commit SUB=sea-streamer MSG='fix: ...'"; exit 1; fi
	@if [ -z "$(MSG)" ]; then echo "Error: MSG is required."; exit 1; fi
	@cd third-party/$(SUB) && git add -A && git commit -m "$(MSG)"
	@echo "Submodule $(SUB) committed. Run 'make submodule-push SUB=$(SUB)' to push."

submodule-push: ## Push submodule changes to its remote (SUB=name)
	@if [ -z "$(SUB)" ]; then echo "Error: SUB is required. Usage: make submodule-push SUB=sea-streamer"; exit 1; fi
	@cd third-party/$(SUB) && git push

submodule-propagate: ## Stage updated submodule ref in main repo (SUB=name)
	@if [ -z "$(SUB)" ]; then echo "Error: SUB is required. Usage: make submodule-propagate SUB=sea-streamer"; exit 1; fi
	@git add third-party/$(SUB)
	@echo "Submodule $(SUB) ref staged in main repo. Commit with: make commit MSG='chore: update $(SUB) submodule'"
