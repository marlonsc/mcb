# =============================================================================
# HELP & INFO - Command documentation and usage information
# =============================================================================

.PHONY: help all

# Default target - complete workflow
all: release ## Complete development workflow

# Help system
help: ## Show all available commands
	@echo "MCP Context Browser v0.0.3 - Organized Makefile"
	@echo "=============================================="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -v '^help' | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'