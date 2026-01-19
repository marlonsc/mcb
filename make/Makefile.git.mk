# =============================================================================
# GIT - Git operations
# =============================================================================
# Each verb does ONE action. sync uses prerequisites for composition.
# =============================================================================

.PHONY: status commit push tag sync

# =============================================================================
# STATUS - Show git status
# =============================================================================

status: ## Show git status
	@echo "Git Status:"
	@git status --short --branch

# =============================================================================
# COMMIT - Stage and commit changes
# =============================================================================

commit: ## Stage and commit changes
	@git add -A
	@git commit || echo "Nothing to commit"

# =============================================================================
# PUSH - Push to remote
# =============================================================================

push: ## Push to remote
	@git push origin $$(git branch --show-current)
	@echo "Pushed to origin"

# =============================================================================
# TAG - Create and push version tag
# =============================================================================

tag: ## Create and push version tag
	@VERSION=$$(grep '^version' crates/mcb/Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/'); \
	echo "Tagging v$$VERSION..."; \
	git tag -a "v$$VERSION" -m "Release v$$VERSION" 2>/dev/null || echo "Tag exists"; \
	git push origin "v$$VERSION" 2>/dev/null || echo "Tag already pushed"

# =============================================================================
# SYNC - Full sync: add + commit + push (uses prerequisites)
# =============================================================================

sync: commit push ## Sync all changes (commit + push)
	@echo "Synced to remote"
