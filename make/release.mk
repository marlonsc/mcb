# =============================================================================
# Release
# =============================================================================

.PHONY: release install install-validate version

VERSION := $(shell grep '^version =' Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/')

# Install locations
INSTALL_DIR := $(HOME)/.local/bin
CARGO_BIN_DIR := $(HOME)/.cargo/bin
BINARY_NAME := mcb
SYSTEMD_USER_DIR := $(HOME)/.config/systemd/user
CONFIG_DIR := $(HOME)/.config/mcb
CONFIG_YAML_DIR := $(CONFIG_DIR)/config
DATA_DIR := $(HOME)/.local/share/mcb

# Service network settings (parsed from Loco YAML config)
SERVICE_HOST := $(shell python3 -c "import yaml, os; d=yaml.safe_load(open('config/development.yaml')) if os.path.exists('config/development.yaml') else {}; print(d.get('server',{}).get('binding','0.0.0.0'))" 2>/dev/null || echo "0.0.0.0")
SERVICE_PORT := $(shell python3 -c "import yaml, os; d=yaml.safe_load(open('config/development.yaml')) if os.path.exists('config/development.yaml') else {}; print(d.get('server',{}).get('port',3000))" 2>/dev/null || echo "3000")

# Version bumping helpers
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

install: ## Install release binary + config + systemd service
	@echo "══════════════════════════════════════════════════════════"
	@echo "  Installing MCB v$(VERSION)"
	@echo "══════════════════════════════════════════════════════════"
	@echo ""
	@# ── 1. Build release binary ────────────────────────────────
	@echo "[1/7] Building release binary..."
	@$(MAKE) build RELEASE=1
	@echo ""
	@# ── 2. Create directories ──────────────────────────────────
	@echo "[2/7] Creating directories..."
	@mkdir -p $(INSTALL_DIR) $(CARGO_BIN_DIR) $(SYSTEMD_USER_DIR) \
	          $(CONFIG_YAML_DIR) $(DATA_DIR) \
	    || { echo "FAIL: cannot create directories"; exit 1; }
	@echo "  $(INSTALL_DIR)"
	@echo "  $(CONFIG_YAML_DIR)"
	@echo "  $(DATA_DIR)"
	@echo ""
	@# ── 3. Install Loco YAML config ────────────────────────────
	@echo "[3/7] Installing Loco YAML config..."
	@cp config/development.yaml "$(CONFIG_YAML_DIR)/development.yaml" \
	    || { echo "FAIL: cannot copy development.yaml"; exit 1; }
	@if [ -f config/production.yaml ]; then \
		cp config/production.yaml "$(CONFIG_YAML_DIR)/production.yaml" && \
		echo "  production.yaml installed (dev fallback)"; \
	fi
	@# Patch database URI to use installed data directory
	@sed -i 's|uri: sqlite://mcb.db|uri: sqlite://$(DATA_DIR)/mcb.db|' \
	    "$(CONFIG_YAML_DIR)/development.yaml"
	@echo "  development.yaml installed (db → $(DATA_DIR)/mcb.db)"
	@echo "[3b/7] Installing MCB client config (deploy.toml → mcb.toml)..."
	@cp config/deploy.toml "$(CONFIG_DIR)/mcb.toml" \
	    || { echo "FAIL: cannot copy deploy.toml"; exit 1; }
	@sed -i 's|path = "mcb.db"|path = "$(DATA_DIR)/mcb.db"|' "$(CONFIG_DIR)/mcb.toml"
	@echo "  $(CONFIG_DIR)/mcb.toml (client mode → http://127.0.0.1:8080)"
	@echo "[3c/7] Installing production config for daemon..."
	@mkdir -p "$(DATA_DIR)/config"
	@cp config/production.yaml "$(DATA_DIR)/config/production.yaml" \
	    || { echo "FAIL: cannot copy production.yaml"; exit 1; }
	@sed -i 's|uri: sqlite://mcb.db|uri: sqlite://$(DATA_DIR)/mcb.db|' "$(DATA_DIR)/config/production.yaml"
	@echo "  $(DATA_DIR)/config/production.yaml (standalone + ollama + milvus)"
	@echo ""
	@# ── 4. Kill ALL running mcb processes ───────────────────────
	@echo "[4/7] Stopping all MCB processes..."
	@-systemctl --user stop mcb.service 2>/dev/null
	@-systemctl --user reset-failed mcb.service 2>/dev/null
	@sleep 1
	@# Kill every mcb process (binary name match), including orphans
	@MCB_PIDS=$$(pgrep -x $(BINARY_NAME) 2>/dev/null || true); \
	if [ -n "$$MCB_PIDS" ]; then \
		echo "  Sending SIGTERM to: $$MCB_PIDS"; \
		echo "$$MCB_PIDS" | xargs kill 2>/dev/null || true; \
		WAIT=0; while [ $$WAIT -lt 10 ]; do \
			ALIVE=$$(echo "$$MCB_PIDS" | xargs -I{} sh -c 'kill -0 {} 2>/dev/null && echo {}' | head -1); \
			if [ -z "$$ALIVE" ]; then break; fi; \
			sleep 1; WAIT=$$((WAIT + 1)); \
		done; \
		STILL=$$(echo "$$MCB_PIDS" | xargs -I{} sh -c 'kill -0 {} 2>/dev/null && echo {}'); \
		if [ -n "$$STILL" ]; then \
			echo "  Force-killing: $$STILL"; \
			echo "$$STILL" | xargs kill -9 2>/dev/null || true; \
			sleep 1; \
		fi; \
		echo "  All MCB processes stopped"; \
	else \
		echo "  No running MCB processes found"; \
	fi
	@echo ""
	@# ── 5. Install binary (atomic swap) ────────────────────────
	@echo "[5/7] Installing binary..."
	@cp target/release/$(BINARY_NAME) "$(INSTALL_DIR)/$(BINARY_NAME).new" \
	    || { echo "FAIL: cannot copy binary"; exit 1; }
	@chmod +x "$(INSTALL_DIR)/$(BINARY_NAME).new"
	@mv -f "$(INSTALL_DIR)/$(BINARY_NAME).new" "$(INSTALL_DIR)/$(BINARY_NAME)" \
	    || { echo "FAIL: cannot install binary"; exit 1; }
	@# Also install to ~/.cargo/bin/ (where 'which mcb' finds it)
	@cp "$(INSTALL_DIR)/$(BINARY_NAME)" "$(CARGO_BIN_DIR)/$(BINARY_NAME)" 2>/dev/null || true
	@if ! $(INSTALL_DIR)/$(BINARY_NAME) --version >/dev/null 2>&1; then \
		echo "FAIL: binary validation failed"; exit 1; \
	fi
	@echo "  $(INSTALL_DIR)/$(BINARY_NAME) → $$($(INSTALL_DIR)/$(BINARY_NAME) --version)"
	@if [ -f "$(CARGO_BIN_DIR)/$(BINARY_NAME)" ]; then \
		echo "  $(CARGO_BIN_DIR)/$(BINARY_NAME) → synced"; \
	fi
	@echo ""
	@# ── 6. Install + start systemd service ──────────────────────
	@echo "[6/7] Configuring systemd service..."
	@cp systemd/mcb.service $(SYSTEMD_USER_DIR)/mcb.service \
	    || { echo "FAIL: cannot install service file"; exit 1; }
	@systemctl --user daemon-reload \
	    || { echo "FAIL: systemctl daemon-reload"; exit 1; }
	@systemctl --user enable mcb.service 2>/dev/null || true
	@-systemctl --user reset-failed mcb.service 2>/dev/null
	@systemctl --user start mcb.service \
	    || { echo "FAIL: cannot start service"; exit 1; }
	@echo "  Service started"
	@echo ""
	@# ── 7. Configure MCP agent integrations ─────────────────────
	@echo "[7/7] Configuring MCP agent integrations..."
	@if command -v jq >/dev/null 2>&1; then \
		if [ -f ".mcp.json" ]; then \
			jq '.mcpServers.mcb = {"command": "$(INSTALL_DIR)/$(BINARY_NAME)", "args": ["serve", "--config", "$(CONFIG_DIR)/mcb.toml"], "env": {}}' .mcp.json > .mcp.json.tmp && \
			mv .mcp.json.tmp .mcp.json && \
			echo "  Claude Code: .mcp.json updated" || \
			echo "  Claude Code: failed to update .mcp.json"; \
		else \
			echo "  Claude Code: .mcp.json not found (skipped)"; \
		fi; \
		OPENCODE_CFG="$(HOME)/.config/opencode/opencode.json"; \
		if [ -f "$$OPENCODE_CFG" ]; then \
			jq '.mcp.mcb = {"type": "local", "command": ["$(INSTALL_DIR)/$(BINARY_NAME)", "serve", "--config", "$(CONFIG_DIR)/mcb.toml"]}' "$$OPENCODE_CFG" > "$$OPENCODE_CFG.tmp" && \
			mv "$$OPENCODE_CFG.tmp" "$$OPENCODE_CFG" && \
			echo "  OpenCode: opencode.json updated" || \
			echo "  OpenCode: failed to update config"; \
		else \
			echo "  OpenCode: config not found (skipped)"; \
		fi; \
		GEMINI_CFG="$(HOME)/.gemini/antigravity/mcp_config.json"; \
		if [ -f "$$GEMINI_CFG" ]; then \
			jq '.mcpServers.mcb = {"command": "$(INSTALL_DIR)/$(BINARY_NAME)", "args": ["serve", "--config", "$(CONFIG_DIR)/mcb.toml"], "type": "stdio", "env": {}, "description": "MCB - Semantic code search via daemon bridge"}' "$$GEMINI_CFG" > "$$GEMINI_CFG.tmp" && \
			mv "$$GEMINI_CFG.tmp" "$$GEMINI_CFG" && \
			jq 'del(.mcpServers.mcb.disabled)' "$$GEMINI_CFG" > "$$GEMINI_CFG.tmp" && \
			mv "$$GEMINI_CFG.tmp" "$$GEMINI_CFG" && \
			echo "  Antigravity: mcp_config.json updated" || \
			echo "  Antigravity: failed to update config"; \
		else \
			echo "  Antigravity: config not found (skipped)"; \
		fi; \
		CURSOR_CFG="$$(readlink -f $(HOME)/.cursor/mcp.json 2>/dev/null || echo '$(HOME)/.cursor/mcp.json')"; \
		if [ -f "$$CURSOR_CFG" ]; then \
			jq '.mcpServers.mcb = {"type": "stdio", "command": "$(INSTALL_DIR)/$(BINARY_NAME)", "args": ["serve", "--config", "$(CONFIG_DIR)/mcb.toml"], "env": {}, "description": "MCB - Semantic code search via daemon bridge"}' "$$CURSOR_CFG" > "$$CURSOR_CFG.tmp" && \
			mv "$$CURSOR_CFG.tmp" "$$CURSOR_CFG" && \
			echo "  Cursor: mcp.json updated" || \
			echo "  Cursor: failed to update config"; \
		else \
			echo "  Cursor: config not found (skipped)"; \
		fi; \
	else \
		echo "  jq not found — skipping MCP agent config updates"; \
	fi
	@echo ""
	@# ── Validate ─────────────────────────────────────────────────
	@$(MAKE) install-validate

install-validate: ## Validate MCB installation
	@echo "── Validating installation ──────────────────────────────"
	@# Binary check
	@if ! $(INSTALL_DIR)/$(BINARY_NAME) --version 2>/dev/null | grep -q "mcb"; then \
		echo "  FAIL: binary not responding"; exit 1; \
	fi
	@echo "  Binary: $$($(INSTALL_DIR)/$(BINARY_NAME) --version)"
	@# Installed config check
	@if [ -f "$(CONFIG_YAML_DIR)/development.yaml" ]; then \
		echo "  Config: $(CONFIG_YAML_DIR)/development.yaml"; \
	else \
		echo "  WARN: no installed config found"; \
	fi
	@# Systemd service check (with retries)
	@RETRIES=0; \
	while [ $$RETRIES -lt 8 ]; do \
		if systemctl --user is-active --quiet mcb.service 2>/dev/null; then \
			echo "  Service: active"; \
			break; \
		fi; \
		RETRIES=$$((RETRIES + 1)); \
		if [ $$RETRIES -lt 8 ]; then sleep 2; fi; \
	done; \
	if [ $$RETRIES -eq 8 ]; then \
		echo "  WARN: systemd service not active (check: journalctl --user -u mcb.service)"; \
		echo "  --- systemd status ---"; \
		systemctl --user status mcb.service --no-pager 2>/dev/null || true; \
		echo "  --- recent logs ---"; \
		journalctl --user -u mcb.service -n 10 --no-pager 2>/dev/null || true; \
	fi
	@# MCP stdio smoke test
	@echo "  MCP stdio: testing..."
	@RESULT=$$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"install-validate","version":"1.0"}}}' \
	    | timeout 15 $(INSTALL_DIR)/$(BINARY_NAME) serve --config $(CONFIG_DIR)/mcb.toml 2>/dev/null); \
	if echo "$$RESULT" | grep -q '"serverInfo"'; then \
		echo "  MCP stdio: OK ($$( echo "$$RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['serverInfo']['name'] + ' ' + json.load(sys.stdin)['result']['serverInfo']['version'])" 2>/dev/null || echo 'response valid'))"; \
	else \
		echo "  FAIL: MCP stdio did not respond"; \
		echo "  Response: $$RESULT"; \
		exit 1; \
	fi
	@echo ""
	@echo "══════════════════════════════════════════════════════════"
	@echo "  MCB v$(VERSION) installed successfully"
	@echo ""
	@echo "  Binary:  $(INSTALL_DIR)/$(BINARY_NAME)"
	@echo "  Config:  $(CONFIG_YAML_DIR)/"
	@echo "  Data:    $(DATA_DIR)/"
	@echo "  Service: systemctl --user status mcb"
	@echo ""
	@echo "  MCP integration: mcb serve --config ~/.config/mcb/mcb.toml"
	@echo "══════════════════════════════════════════════════════════"

version: ## Show version (BUMP=patch|minor|major to bump)
ifeq ($(BUMP),patch)
	@echo "Bumping to $(NEXT_PATCH)..."
	@sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_PATCH)"/' Cargo.toml
	@cargo check 2>/dev/null || true
	@echo "Version bumped to $(NEXT_PATCH)"
else ifeq ($(BUMP),minor)
	@echo "Bumping to $(NEXT_MINOR)..."
	@sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MINOR)"/' Cargo.toml
	@cargo check 2>/dev/null || true
	@echo "Version bumped to $(NEXT_MINOR)"
else ifeq ($(BUMP),major)
	@echo "Bumping to $(NEXT_MAJOR)..."
	@sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MAJOR)"/' Cargo.toml
	@cargo check 2>/dev/null || true
	@echo "Version bumped to $(NEXT_MAJOR)"
else
	@echo "Current version: $(VERSION)"
	@echo "Next patch:      $(NEXT_PATCH)"
	@echo "Next minor:      $(NEXT_MINOR)"
	@echo "Next major:      $(NEXT_MAJOR)"
endif
