# =============================================================================
# Release
# =============================================================================

.PHONY: release install install-validate version

VERSION := $(shell grep '^version =' Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/')

INSTALL_DIR := $(HOME)/.local/bin
INSTALL_BINARY := mcb
BINARY_NAME := mcb
SYSTEMD_USER_DIR := $(HOME)/.config/systemd/user
CONFIG_DIR := $(HOME)/.config/mcb
DATA_DIR := $(HOME)/.local/share/mcb
SERVICE_HOST := $(shell python3 -c 'import tomllib, os; p=os.path.expanduser("~/.config/mcb/mcb.toml"); d=tomllib.load(open(p,"rb")) if os.path.exists(p) else tomllib.load(open("config/default.toml","rb")); print(d.get("server",{}).get("network",{}).get("host", "127.0.0.1"))')
SERVICE_PORT := $(shell python3 -c 'import tomllib, os; p=os.path.expanduser("~/.config/mcb/mcb.toml"); d=tomllib.load(open(p,"rb")) if os.path.exists(p) else tomllib.load(open("config/default.toml","rb")); print(d.get("server",{}).get("network",{}).get("port", 3000))')

NEXT_PATCH := $(shell echo $(VERSION) | awk -F. '{print $$1"."$$2"."($$3+1)}')
NEXT_MINOR := $(shell echo $(VERSION) | awk -F. '{print $$1"."($$2+1)".0"}')
NEXT_MAJOR := $(shell echo $(VERSION) | awk -F. '{print ($$1+1)".0.0"}')

##@ Release

release: ## Full release pipeline (lint + test + validate + build)
	@echo "Creating release v$(VERSION)..."
	@$(MAKE) lint MCB_CI=1
	@$(MAKE) test
	@$(MAKE) validate QUICK=1
	@$(MAKE) build RELEASE=1
	@echo "Packaging..."
	@mkdir -p dist
	@if [ ! -f "target/release/$(BINARY_NAME)" ]; then \
		echo "Error: Binary target/release/$(BINARY_NAME) not found after build" >&2; \
		exit 1; \
	fi
	@cp target/release/$(BINARY_NAME) dist/
	@cd dist && tar -czf $(BINARY_NAME)-$(VERSION).tar.gz $(BINARY_NAME)
	@echo "Release v$(VERSION) ready: dist/$(BINARY_NAME)-$(VERSION).tar.gz"

install: ## Install release binary + systemd service to user directories
	@echo "Installing MCB v$(VERSION)..."
	@$(MAKE) build RELEASE=1
	@mkdir -p $(INSTALL_DIR) $(SYSTEMD_USER_DIR) $(CONFIG_DIR) $(DATA_DIR) || { echo "Failed to create directories"; exit 1; }
	@echo "Installing deploy config to $(CONFIG_DIR)/mcb.toml..."
	@cp config/deploy.toml "$(CONFIG_DIR)/mcb.toml" || { echo "Failed to install config"; exit 1; }
	@echo "Stopping existing MCB..."
	@-systemctl --user stop mcb.service 2>/dev/null
	@WAIT=0; while [ $$WAIT -lt 10 ]; do \
		if ! pgrep -f "$(INSTALL_DIR)/$(INSTALL_BINARY)" >/dev/null 2>&1; then break; fi; \
		sleep 1; WAIT=$$((WAIT + 1)); \
	done
	@-pkill -9 -f "$(INSTALL_DIR)/$(INSTALL_BINARY)" 2>/dev/null; sleep 1
	@WAIT=0; while [ $$WAIT -lt 5 ]; do \
		if ! fuser "$(INSTALL_DIR)/$(INSTALL_BINARY)" >/dev/null 2>&1; then break; fi; \
		sleep 1; WAIT=$$((WAIT + 1)); \
	done
	@rm -f "$(INSTALL_DIR)/$(INSTALL_BINARY).new" 2>/dev/null
	@cp target/release/$(BINARY_NAME) "$(INSTALL_DIR)/$(INSTALL_BINARY).new" || { echo "Failed to copy binary"; exit 1; }
	@chmod +x "$(INSTALL_DIR)/$(INSTALL_BINARY).new"
	@rm -f "$(INSTALL_DIR)/$(INSTALL_BINARY)" 2>/dev/null
	@mv "$(INSTALL_DIR)/$(INSTALL_BINARY).new" "$(INSTALL_DIR)/$(INSTALL_BINARY)" || { echo "Failed to install binary"; exit 1; }
	@if ! $(INSTALL_DIR)/$(INSTALL_BINARY) --version >/dev/null 2>&1; then echo "Binary validation failed"; exit 1; fi
	@cp systemd/mcb.service $(SYSTEMD_USER_DIR)/mcb.service || { echo "Failed to install systemd service"; exit 1; }
	@systemctl --user daemon-reload || { echo "Failed to reload systemd"; exit 1; }
	@systemctl --user enable mcb.service || { echo "Failed to enable service"; exit 1; }
	@systemctl --user start mcb.service || { echo "Failed to start service"; exit 1; }
	@sleep 2
	@echo "Configuring MCP agent integrations..."
	@command -v jq >/dev/null 2>&1 || { echo "jq is required for MCP config updates"; exit 1; }
	@if [ -f ".mcp.json" ]; then \
		jq '.mcpServers.mcb.args = ["serve", "--config", "$(CONFIG_DIR)/mcb.toml"]' .mcp.json > .mcp.json.tmp && \
		mv .mcp.json.tmp .mcp.json || { echo "Failed to update Claude Code config"; exit 1; }; \
		echo "  Claude Code: .mcp.json configured"; \
	else \
		echo "  Claude Code: .mcp.json not found"; \
	fi
	@if [ -f "$(HOME)/.gemini/antigravity/mcp_config.json" ]; then \
		jq '.mcpServers.mcb.args = ["serve", "--config", "$(CONFIG_DIR)/mcb.toml"]' "$(HOME)/.gemini/antigravity/mcp_config.json" > "$(HOME)/.gemini/antigravity/mcp_config.json.tmp" && \
		mv "$(HOME)/.gemini/antigravity/mcp_config.json.tmp" "$(HOME)/.gemini/antigravity/mcp_config.json" || { echo "Failed to update Gemini config"; exit 1; }; \
		echo "  Gemini: mcp_config.json configured"; \
	else \
		echo "  Gemini: mcp_config.json not found"; \
	fi
	@$(MAKE) install-validate

install-validate: ## Validate MCB installation with retries
	@echo "Validating MCB installation..."
	@if ! $(INSTALL_DIR)/$(INSTALL_BINARY) --version 2>/dev/null | grep -q "mcb"; then \
		echo "Binary failed"; exit 1; \
	fi
	@RETRIES=0; \
	while [ $$RETRIES -lt 5 ]; do \
		if systemctl --user is-active --quiet mcb.service 2>/dev/null; then \
			echo "Service active"; \
			break; \
		fi; \
		RETRIES=$$((RETRIES + 1)); \
		if [ $$RETRIES -lt 5 ]; then sleep 1; fi; \
	done; \
	if [ $$RETRIES -eq 5 ]; then \
		echo "Service failed to start"; \
		journalctl --user -u mcb.service -n 10 --no-pager 2>/dev/null; \
		exit 1; \
	fi
	@RETRIES=0; \
	while [ $$RETRIES -lt 5 ]; do \
		if curl -s --connect-timeout 2 http://$(SERVICE_HOST):$(SERVICE_PORT)/healthz 2>/dev/null | grep -q "OK\|status"; then \
			echo "HTTP server responding"; \
			break; \
		fi; \
		RETRIES=$$((RETRIES + 1)); \
		if [ $$RETRIES -lt 5 ]; then sleep 1; fi; \
	done; \
	if [ $$RETRIES -eq 5 ]; then \
		echo "HTTP server failed"; \
		systemctl --user status mcb.service 2>/dev/null; \
		journalctl --user -u mcb.service -n 30 --no-pager 2>/dev/null; \
		exit 1; \
	fi

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
