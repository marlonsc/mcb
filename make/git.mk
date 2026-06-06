# =============================================================================
# Git & GitHub
# =============================================================================

.PHONY: status diff log commit push pull add branch checkout tag stash stash-pop \
        diff-stat diff-branch log-graph merge rebase reset-file show \
        pr pr-checks pr-view submodule-status submodule-sync submodule-push

# Configurable
BRANCH ?= $(shell git rev-parse --abbrev-ref HEAD)
BASE ?= main
FILES ?=
MSG ?=
PR ?=
REF ?=
TAG ?=

##@ Git

status: ## Git status (main + submodules)
	@echo "=== Main repo ($(BRANCH)) ==="
	@git status --short
	@echo ""
	@echo "=== Submodules ==="
	@git submodule foreach --quiet 'STATUS=$$(git status --short); if [ -n "$$STATUS" ]; then echo "$$name:"; echo "$$STATUS"; fi'

diff: ## Git diff (staged + unstaged)
	@git diff
	@git diff --cached

diff-stat: ## Diff summary stats (BASE=main)
	@git diff --stat $(BASE)...HEAD

diff-branch: ## Diff between current branch and BASE (BASE=main)
	@git diff $(BASE)...HEAD

log: ## Recent commits (LOG_N=10)
	@git log --oneline -$(or $(LOG_N),10)

log-graph: ## Commits graph (LOG_N=20)
	@git log --oneline --graph --all -$(or $(LOG_N),20)

log-branch: ## Commits on current branch since BASE (BASE=main)
	@git log --oneline $(BASE)..HEAD

show: ## Show a commit (REF=HEAD)
	@git show --stat $(or $(REF),HEAD)

add: ## Stage files (FILES=required)
	@if [ -z "$(FILES)" ]; then echo "Error: FILES is required. Usage: make add FILES='file1 file2'"; exit 1; fi
	@git add $(FILES)

commit: ## Commit staged files (MSG=, FILES= optional)
	@if [ -z "$(MSG)" ]; then echo "Error: MSG is required. Usage: make commit MSG='your message' FILES='file1 file2'"; exit 1; fi
	@if [ -n "$(FILES)" ]; then git add $(FILES); fi
	@git commit -m "$(MSG)"

push: ## Push current branch to origin
	@git push origin $(BRANCH)

push-tags: ## Push all tags to origin
	@git push origin --tags

pull: ## Pull current branch from origin
	@git pull origin $(BRANCH)

branch: ## Show branches or create (REF= to create, BASE=main for start point)
	@if [ -z "$(REF)" ]; then git branch -a; else git branch $(REF) $(BASE); echo "Created branch $(REF) from $(BASE)"; fi

checkout: ## Switch branch (REF=required)
	@if [ -z "$(REF)" ]; then echo "Error: REF is required. Usage: make checkout REF=main"; exit 1; fi
	@git checkout $(REF)

tag: ## Create tag (TAG=required, MSG= optional annotation)
	@if [ -z "$(TAG)" ]; then echo "Error: TAG is required. Usage: make tag TAG=v0.3.1 MSG='Release 0.3.1'"; exit 1; fi
	@if [ -n "$(MSG)" ]; then git tag -a $(TAG) -m "$(MSG)"; else git tag $(TAG); fi
	@echo "Created tag $(TAG)"

tags: ## List tags
	@git tag -l --sort=-version:refname | head -20

stash: ## Stash working changes (MSG= optional)
	@if [ -n "$(MSG)" ]; then git stash push -m "$(MSG)"; else git stash push; fi

stash-pop: ## Pop latest stash
	@git stash pop

stash-list: ## List stashes
	@git stash list

merge: ## Merge branch (REF=required, no-ff)
	@if [ -z "$(REF)" ]; then echo "Error: REF is required. Usage: make merge REF=feature-branch"; exit 1; fi
	@git merge --no-ff $(REF)

rebase: ## Rebase onto BASE (BASE=main)
	@git rebase $(BASE)

reset-file: ## Unstage a file (FILES=required)
	@if [ -z "$(FILES)" ]; then echo "Error: FILES is required. Usage: make reset-file FILES='file1'"; exit 1; fi
	@git restore --staged $(FILES)

##@ GitHub

pr-checks: ## Show PR check status (PR=number)
	@if [ -z "$(PR)" ]; then echo "Error: PR is required. Usage: make pr-checks PR=116"; exit 1; fi
	@gh pr checks $(PR) || true

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
