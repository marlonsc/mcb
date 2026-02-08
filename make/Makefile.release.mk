# =============================================================================
# RELEASE - Build release, package, install, version
# =============================================================================
# Parameters: BUMP (from main Makefile)
# =============================================================================

.PHONY: release install install-validate install-mcp version

# Get version from root Cargo.toml (workspace)
VERSION := $(shell grep '^version =' Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/')

# Installation directories
INSTALL_DIR := $(HOME)/.local/bin
INSTALL_BINARY := mcb
BINARY_NAME := mcb
SYSTEMD_USER_DIR := $(HOME)/.config/systemd/user
CONFIG_DIR := $(HOME)/.config/mcb
DATA_DIR := $(HOME)/.local/share/mcb

# =============================================================================
# RELEASE - Full release pipeline
# =============================================================================

release: ## Full release pipeline (lint + test + validate + build)
	@echo "Creating release v$(VERSION)..."
	@$(MAKE) lint CI_MODE=1
	@$(MAKE) test
	@$(MAKE) validate QUICK=1
	@$(MAKE) build RELEASE=1
	@echo "Packaging..."
	@mkdir -p dist
	@cp target/release/$(BINARY_NAME) dist/ 2>/dev/null || echo "Binary not found"
	@cd dist && tar -czf $(BINARY_NAME)-$(VERSION).tar.gz $(BINARY_NAME) 2>/dev/null || true
	@echo "Release v$(VERSION) ready: dist/$(BINARY_NAME)-$(VERSION).tar.gz"

# =============================================================================
# INSTALL - Install release binary + systemd service
# =============================================================================

install: ## Install release binary + systemd service to user directories (fully automatic)
	@echo "🚀 Installing MCB v$(VERSION)..."
	@$(MAKE) build RELEASE=1
	@# Create all required directories
	@mkdir -p $(INSTALL_DIR) $(SYSTEMD_USER_DIR) $(CONFIG_DIR) $(DATA_DIR) || { echo "❌ Failed to create directories"; exit 1; }
	@# Gracefully stop existing service and processes (no errors if not running)
	@-systemctl --user stop mcb.service 2>/dev/null; sleep 1
	@-pkill -9 -f "\.local/bin/mcb" 2>/dev/null; sleep 1
	@-pkill -9 -f "mcb.*serve.*--server" 2>/dev/null; sleep 1
	@# Backup old binary if exists
	@if [ -f "$(INSTALL_DIR)/$(INSTALL_BINARY)" ]; then mv "$(INSTALL_DIR)/$(INSTALL_BINARY)" "$(INSTALL_DIR)/$(INSTALL_BINARY).bak.$$"; fi
	@# Install new binary with strict validation
	@if ! cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(INSTALL_BINARY); then echo "❌ Failed to copy binary"; exit 1; fi
	@chmod +x $(INSTALL_DIR)/$(INSTALL_BINARY) || { echo "❌ Failed to chmod binary"; exit 1; }
	@# Verify binary works
	@if ! $(INSTALL_DIR)/$(INSTALL_BINARY) --version >/dev/null 2>&1; then echo "❌ Binary validation failed"; exit 1; fi
	@# Install systemd service
	@cp systemd/mcb.service $(SYSTEMD_USER_DIR)/mcb.service || { echo "❌ Failed to install systemd service"; exit 1; }
	@# Reload, enable and start service
	@systemctl --user daemon-reload || { echo "❌ Failed to reload systemd"; exit 1; }
	@systemctl --user enable mcb.service || { echo "❌ Failed to enable service"; exit 1; }
	@systemctl --user start mcb.service || { echo "❌ Failed to start service"; exit 1; }
	@sleep 2
	@# Update MCP agent configurations
	@$(MAKE) install-mcp
	@# Final validation loop with retries
	@$(MAKE) install-validate

# =============================================================================
# INSTALL-MCP - Update agent configs to use MCB server with "serve" subcommand
# =============================================================================

install-mcp: ## Update Claude Code and Gemini MCP configs
	@echo "📋 Configuring MCP agent integrations..."
	@# Require jq for JSON updates (fail early if missing)
	@command -v jq >/dev/null 2>&1 || { echo "❌ jq is required for MCP config updates"; exit 1; }
	@# Claude Code .mcp.json - ensure mcb with ["serve"]
	@if [ -f ".mcp.json" ]; then \
		jq '.mcpServers.mcb.args = ["serve"]' .mcp.json > .mcp.json.tmp && \
		mv .mcp.json.tmp .mcp.json || { echo "❌ Failed to update Claude Code config"; exit 1; }; \
		echo "  ✓ Claude Code: .mcp.json configured"; \
	else \
		echo "  ⚠ Claude Code: .mcp.json not found (create manually if needed)"; \
	fi
	@# Gemini mcp_config.json - ensure mcb with ["serve"]
	@if [ -f "$(HOME)/.gemini/antigravity/mcp_config.json" ]; then \
		jq '.mcpServers.mcb.args = ["serve"]' "$(HOME)/.gemini/antigravity/mcp_config.json" > "$(HOME)/.gemini/antigravity/mcp_config.json.tmp" && \
		mv "$(HOME)/.gemini/antigravity/mcp_config.json.tmp" "$(HOME)/.gemini/antigravity/mcp_config.json" || { echo "❌ Failed to update Gemini config"; exit 1; }; \
		echo "  ✓ Gemini: mcp_config.json configured"; \
	else \
		echo "  ⚠ Gemini: mcp_config.json not found (optional)"; \
	fi
	@echo "  ℹ MCB core config: ~/.config/mcb/mcb.toml (providers, vector store, ports)"

# =============================================================================
# INSTALL-VALIDATE - Install and validate the binary works
# =============================================================================

install-validate: ## Validate MCB installation with retries
	@echo ""
	@echo "🔍 Validating MCB installation..."
	@# 1. Binary version check
	@echo "1. Binary executable:"
	@if ! $(INSTALL_DIR)/$(INSTALL_BINARY) --version 2>/dev/null | grep -q "mcb"; then \
		echo "   ❌ Binary failed"; exit 1; \
	fi
	@echo "   ✓ Binary working"
	@# 2. Service status (with wait for startup)
	@echo "2. Systemd service:"
	@RETRIES=0; \
	while [ $$RETRIES -lt 5 ]; do \
		if systemctl --user is-active --quiet mcb.service 2>/dev/null; then \
			echo "   ✓ Service active"; \
			break; \
		fi; \
		RETRIES=$$((RETRIES + 1)); \
		if [ $$RETRIES -lt 5 ]; then sleep 1; fi; \
	done; \
	if [ $$RETRIES -eq 5 ]; then \
		echo "   ❌ Service failed to start"; \
		journalctl --user -u mcb.service -n 10 --no-pager 2>/dev/null; \
		exit 1; \
	fi
	@# 3. HTTP health check (with retries)
	@echo "3. HTTP server health:"
	@RETRIES=0; \
	while [ $$RETRIES -lt 5 ]; do \
		if curl -s --connect-timeout 2 http://127.0.0.1:8080/healthz 2>/dev/null | grep -q "OK\|status"; then \
			echo "   ✓ HTTP server responding"; \
			break; \
		fi; \
		RETRIES=$$((RETRIES + 1)); \
		if [ $$RETRIES -lt 5 ]; then sleep 1; fi; \
	done; \
	if [ $$RETRIES -eq 5 ]; then \
		echo "   ❌ HTTP server failed"; \
		systemctl --user status mcb.service 2>/dev/null; \
		exit 1; \
	fi
	@echo ""
	@echo "✅ All validation checks passed!"
	@echo ""
	@echo "📊 System Status:"
	@echo "   Version: $$($(INSTALL_DIR)/$(INSTALL_BINARY) --version 2>/dev/null)"
	@echo "   Binary: $(INSTALL_DIR)/$(INSTALL_BINARY)"
	@echo "   Config: $(CONFIG_DIR)/mcb.toml"
	@echo "   Server: http://127.0.0.1:8080"
	@echo ""
	@echo "🎯 Next steps:"
	@echo "   - MCB is running as systemd service"
	@echo "   - Available via MCP in Claude Code (.mcp.json) and Gemini"
	@echo "   - View logs: journalctl --user -u mcb.service -f"

# =============================================================================
# VERSION (BUMP=patch|minor|major|check)
# =============================================================================

# Calculate next versions
NEXT_PATCH := $(shell echo $(VERSION) | awk -F. '{print $$1"."$$2"."($$3+1)}')
NEXT_MINOR := $(shell echo $(VERSION) | awk -F. '{print $$1"."($$2+1)".0"}')
NEXT_MAJOR := $(shell echo $(VERSION) | awk -F. '{print ($$1+1)".0.0"}')

version: ## Show version (BUMP=patch|minor|major to bump)
ifeq ($(BUMP),patch)
	@echo "Bumping to $(NEXT_PATCH)..."
	@sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_PATCH)"/' crates/mcb/Cargo.toml
	@cargo check 2>/dev/null || true
	@echo "Version bumped to $(NEXT_PATCH)"
else ifeq ($(BUMP),minor)
	@echo "Bumping to $(NEXT_MINOR)..."
	@sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MINOR)"/' crates/mcb/Cargo.toml
	@cargo check 2>/dev/null || true
	@echo "Version bumped to $(NEXT_MINOR)"
else ifeq ($(BUMP),major)
	@echo "Bumping to $(NEXT_MAJOR)..."
	@sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MAJOR)"/' crates/mcb/Cargo.toml
	@cargo check 2>/dev/null || true
	@echo "Version bumped to $(NEXT_MAJOR)"
else
	@echo "Current version: $(VERSION)"
	@echo "Next patch:      $(NEXT_PATCH)"
	@echo "Next minor:      $(NEXT_MINOR)"
	@echo "Next major:      $(NEXT_MAJOR)"
endif
