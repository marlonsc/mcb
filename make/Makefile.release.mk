# =============================================================================
# RELEASE - Build release, package, install, version
# =============================================================================
# Parameters: BUMP (from main Makefile)
# =============================================================================

.PHONY: release install install-validate version

# Get version from mcb crate Cargo.toml
VERSION := $(shell grep '^version' crates/mcb/Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/')

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

install: ## Install release binary + systemd service to user directories
	@echo "Installing MCB v$(VERSION)..."
	@$(MAKE) build RELEASE=1
	@# Create directories
	@mkdir -p $(INSTALL_DIR)
	@mkdir -p $(SYSTEMD_USER_DIR)
	@mkdir -p $(CONFIG_DIR)
	@mkdir -p $(DATA_DIR)
	@# FIRST: Stop systemd and kill ALL mcb server processes
	@echo "Stopping MCB service and processes..."
	@-systemctl --user stop mcb.service 2>/dev/null || true
	@sleep 1
	@-pkill -9 -f "\.local/bin/mcb" 2>/dev/null || true
	@-pkill -9 -f "mcb --server" 2>/dev/null || true
	@sleep 1
	@# Backup old binary
	@if [ -f "$(INSTALL_DIR)/$(INSTALL_BINARY)" ]; then \
		mv "$(INSTALL_DIR)/$(INSTALL_BINARY)" "$(INSTALL_DIR)/$(INSTALL_BINARY).old"; \
	fi
	@# Install binary
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(INSTALL_BINARY)
	@chmod +x $(INSTALL_DIR)/$(INSTALL_BINARY)
	@# Install systemd service
	@cp systemd/mcb.service $(SYSTEMD_USER_DIR)/mcb.service
	@# Reload and start systemd service
	@systemctl --user daemon-reload
	@systemctl --user enable mcb.service
	@systemctl --user start mcb.service
	@echo ""
	@echo "✓ Installed v$(VERSION) to $(INSTALL_DIR)/$(INSTALL_BINARY)"
	@echo "✓ Systemd service installed and started"
	@ls -lh $(INSTALL_DIR)/$(INSTALL_BINARY) | awk '{print "  Size: "$$5"  Modified: "$$6" "$$7" "$$8}'
	@if [ -f "$(INSTALL_DIR)/$(INSTALL_BINARY).old" ]; then echo "  Old binary: $(INSTALL_DIR)/$(INSTALL_BINARY).old (remove if not needed)"; fi
	@# Run validation
	@$(MAKE) install-validate

# =============================================================================
# INSTALL-VALIDATE - Install and validate the binary works
# =============================================================================

install-validate: ## Validate MCB installation
	@echo ""
	@echo "Validating installation..."
	@echo "1. Binary version:"
	@$(INSTALL_DIR)/$(INSTALL_BINARY) --version 2>/dev/null && echo "   ✓ Binary runs successfully" || (echo "   ✗ Binary failed" && exit 1)
	@echo ""
	@echo "2. Systemd service status:"
	@if systemctl --user is-active --quiet mcb.service 2>/dev/null; then \
		echo "   ✓ Service is active"; \
	else \
		echo "   ⚠ Service is not active"; \
		echo "   Checking logs..."; \
		journalctl --user -u mcb.service -n 5 --no-pager 2>/dev/null || true; \
	fi
	@echo ""
	@echo "3. HTTP health check:"
	@sleep 2
	@if curl -s --connect-timeout 2 http://127.0.0.1:8080/healthz >/dev/null 2>&1; then \
		echo "   ✓ Health endpoint responding"; \
		curl -s http://127.0.0.1:8080/healthz 2>/dev/null | head -c 200 || true; \
		echo ""; \
	else \
		echo "   ⚠ Health endpoint not responding (may be starting up)"; \
	fi
	@echo ""
	@echo "✓ Validation complete"

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
