# =============================================================================
# makefiles/dispatch.mk — one DISPATCH_<verb> macro per canonical verb.
# Each macro case-dispatches on WHAT=; phases run inline or delegate to mcb.sh.
# Destructive phases are gated by $(call gate,<action>) (APPLY=Y required).
# Unknown WHAT prints the WHATS_<verb> SSOT list and exits 2.
# =============================================================================

# --- verb-local variables (single home) --------------------------------------
MDBOOK         := $(shell command -v mdbook 2>/dev/null || echo "$(HOME)/.cargo/bin/mdbook")
MCB_TEST_PORT  ?= 18080

# Test runner: prefer cargo-nextest (faster, parallel, better output) when installed;
# fall back to `cargo test`. Doctests always use `cargo test --doc` (nextest can't
# run them) — semantics preserved since `cargo test --all-targets` also skips doctests.
MCB_NEXTEST := $(shell command -v cargo-nextest >/dev/null 2>&1 && echo 1)
ifeq ($(MCB_NEXTEST),1)
  MCB_TEST_UNIT := cargo nextest run --workspace --lib --test-threads=$$T
  MCB_TEST_ALL  := cargo nextest run --workspace --test-threads=$$T
else
  MCB_TEST_UNIT := RUST_TEST_THREADS=$$T cargo test --workspace --lib
  MCB_TEST_ALL  := RUST_TEST_THREADS=$$T cargo test --workspace --all-targets
endif

# codegen
CODEGEN_DB         := /tmp/mcb_codegen.db
MIGRATION_RS       := crates/mcb-providers/src/database/seaorm/migration/m20260301_000001_initial_schema.rs
SEA_ORM_CLI        := third-party/sea-orm/sea-orm-cli/target/debug/sea-orm-cli
ENTITIES_DIR       := crates/mcb-providers/src/database/seaorm/entities
CONVERSIONS_DIR    := crates/mcb-providers/src/database/seaorm/conversions
CONVERSIONS_TOML   := config/conversions.toml
CONVERSIONS_SCRIPT := scripts/codegen-conversions.py
EXTRACT_SCRIPT     := scripts/extract-migration-sql.py

# release / install
VERSION          := $(shell grep '^version =' Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
BINARY_NAME      := mcb
INSTALL_DIR      := $(HOME)/.local/bin
CARGO_BIN_DIR    := $(HOME)/.cargo/bin
SYSTEMD_USER_DIR := $(HOME)/.config/systemd/user
CONFIG_DIR       := $(HOME)/.config/mcb
CONFIG_YAML_DIR  := $(CONFIG_DIR)/config
DATA_DIR         := $(HOME)/.local/share/mcb
NEXT_PATCH := $(shell echo $(VERSION) | awk -F. '{print $$1"."$$2"."($$3+1)}')
NEXT_MINOR := $(shell echo $(VERSION) | awk -F. '{print $$1"."($$2+1)".0"}')
NEXT_MAJOR := $(shell echo $(VERSION) | awk -F. '{print ($$1+1)".0.0"}')

# --- build -------------------------------------------------------------------
define DISPATCH_BUILD
@if [ "$(RELEASE)" = "1" ]; then echo "Building release..."; cargo build --release; \
 else echo "Building debug..."; cargo build; fi
endef

# --- test (SCOPE dispatch; e2e folded in) ------------------------------------
define DISPATCH_TEST
@T="$(THREADS)"; case "$$T" in ''|*[!0-9]*|0) T=1;; esac; \
case "$(SCOPE)" in \
  unit)        $(MCB_TEST_UNIT) ;; \
  doc)         cargo test --workspace --doc ;; \
  golden)      RUST_TEST_THREADS=$$T cargo test --workspace --tests golden ;; \
  startup)     cargo test -p mcb --test integration startup_smoke -- --nocapture ;; \
  integration) RUST_TEST_THREADS=$$T cargo test --workspace --test '*integration*' ;; \
  e2e)         $(call MCB_E2E) ;; \
  all)         $(MCB_TEST_ALL) && $(call MCB_E2E) ;; \
  '')          $(MCB_TEST_ALL) ;; \
  *)           printf "ERRO: SCOPE '%s' invalido. Validos: unit doc golden startup integration e2e all\n" "$(SCOPE)" >&2; exit 2 ;; \
esac
endef

define MCB_E2E
echo "Running Playwright E2E on port $(MCB_TEST_PORT)..."; \
lsof -ti:$(MCB_TEST_PORT) | xargs -r kill -9 2>/dev/null || true; sleep 1; \
command -v npx >/dev/null || { echo "Error: npx not found. Install Node.js first." >&2; exit 1; }; \
if [ ! -d tests/node_modules/@playwright ]; then echo "Installing Playwright..."; \
  npm --prefix tests install --save-dev @playwright/test @types/node typescript 2>&1 | grep -v "npm WARN" || true; \
  (cd tests && npx playwright install chromium --with-deps 2>&1 | tail -5); fi; \
cargo build --release --bin mcb; \
cd tests && MCB_TEST_PORT=$(MCB_TEST_PORT) node_modules/.bin/playwright test --config=playwright.config.ts --reporter=list
endef

# --- check (read-only gate) --------------------------------------------------
define DISPATCH_CHECK
@case "$(WHAT)" in \
  fmt)      cargo fmt --all -- --check ;; \
  lint)     cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings ;; \
  validate) bash $(MCB_SH) validate $(if $(filter 1,$(QUICK)),quick,full) ;; \
  audit)    cargo audit $(foreach i,$(MCB_AUDIT_IGNORES),--ignore $(i)) && $(MAKE) check WHAT=udeps ;; \
  udeps)    command -v cargo-udeps >/dev/null 2>&1 || cargo install cargo-udeps; cargo +nightly udeps --workspace ;; \
  coverage) cargo tarpaulin --out Lcov --output-dir coverage --exclude-files 'crates/*/tests/integration/*' --exclude-files 'crates/*/tests/admin/*' --timeout 300 ;; \
  qlty)     mkdir -p docs/reports; ./scripts/analyze_qlty.py --scan --check --summary --markdown docs/reports/qlty-check-REPORTS.md; ./scripts/analyze_qlty.py --scan --smells --summary --markdown docs/reports/qlty-smells-REPORTS.md ;; \
  coordination) bd config get beads.role --json && bd status --json && bd hooks list --json && bash scripts/context/validate-beads-policy.sh && bd dep cycles --json && bd stale --status in_progress --days 1 --limit 25 --json && bd graph --all --compact >/dev/null ;; \
  ""|all)   cargo fmt --all -- --check && $(MAKE) lint-impl && $(MAKE) test && bash $(MCB_SH) validate $(if $(filter 1,$(QUICK)),quick,full) ;; \
  *)        printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_check)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- hook (tiered native git-hook gates; SSOT for pre-commit/pre-push) --------
# pre-commit (fast): guard + fmt + clippy(workspace, no test/bench compile) + typos
#   + unit tests. pre-push (full): clippy --all-targets + full suite + doctests +
#   validate. Same gates the CI runs, one definition. No bypass (AGENTS.md §3).
define DISPATCH_HOOK
@case "$(WHAT)" in \
  pre-commit) \
    bash $(MCB_SH) guard --staged && \
    cargo fmt --all -- --check && \
    cargo clippy --workspace -- -D warnings && \
    { ! command -v typos >/dev/null 2>&1 || typos; } && \
    $(if $(filter 1,$(MCB_NEXTEST)),cargo nextest run --workspace --lib,cargo test --workspace --lib) ;; \
  pre-push) \
    cargo fmt --all -- --check && \
    cargo clippy --all-targets -- -D warnings && \
    $(MAKE) test && $(MAKE) test SCOPE=doc && \
    bash $(MCB_SH) validate quick ;; \
  *)          printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_hook)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- fix (mutating auto-fix; runs directly) ----------------------------------
define DISPATCH_FIX
@case "$(WHAT)" in \
  fmt)        cargo fmt --all ;; \
  lint)       cargo fmt --all && cargo clippy --fix --allow-dirty --all-targets ;; \
  docs)       $(MAKE) docs WHAT=lint FIX=1 ;; \
  ""|all)     cargo fmt --all && cargo clippy --fix --allow-dirty --all-targets && $(MAKE) docs WHAT=lint FIX=1 ;; \
  *)          printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_fix)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- dev / docker ------------------------------------------------------------
define DISPATCH_DEV
@case "$(WHAT)" in \
  ""|run)       echo "Starting dev server..."; cargo watch -x 'run' 2>/dev/null || cargo run ;; \
  docker-up)    echo "Starting Docker test services..."; docker-compose -f tests/docker-compose.yml up -d; sleep 5 ;; \
  docker-down)  echo "Stopping Docker test services..."; docker-compose -f tests/docker-compose.yml down -v ;; \
  docker-logs)  docker-compose -f tests/docker-compose.yml logs -f ;; \
  docker-test)  docker-compose -f tests/docker-compose.yml --profile test up --build --abort-on-container-exit test-runner; docker-compose -f tests/docker-compose.yml --profile test rm -f test-runner ;; \
  *)            printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_dev)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- docs --------------------------------------------------------------------
define DISPATCH_DOCS
@case "$(WHAT)" in \
  ""|build)  ./scripts/docs/inject-metrics.sh; cargo doc --no-deps --workspace; ./scripts/docs/mdbook-sync.sh; if [ -x "$(MDBOOK)" ]; then $(MDBOOK) build book/; else echo "Warning: mdbook not found, skipping book build" >&2; fi ;; \
  serve)     ./scripts/docs/mdbook-sync.sh 2>/dev/null || true; if [ -x "$(MDBOOK)" ]; then $(MDBOOK) serve book/ --open; else echo "mdbook not installed (cargo install mdbook)"; fi ;; \
  lint)      if [ "$(FIX)" = "1" ]; then ./scripts/docs/markdown.sh fix; else ./scripts/docs/markdown.sh lint; fi ;; \
  validate)  QUICK="$(QUICK)" ./scripts/docs/validate.sh all ;; \
  sync)      ./scripts/docs/mdbook-sync.sh 2>/dev/null || true ;; \
  rust)      cargo doc --no-deps --workspace ;; \
  check)     [ -d docs ] || { echo "ERROR: docs/ directory not found" >&2; exit 1; } ;; \
  setup)     mkdir -p book; [ -f book.toml ] || { echo "ERROR: book.toml not found in root" >&2; exit 1; } ;; \
  adr)       echo "Architecture Decision Records:"; ls -1 docs/adr/[0-9]*.md 2>/dev/null | while read f; do num=$$(basename "$$f" .md | cut -d- -f1); title=$$(head -1 "$$f" | sed 's/^# ADR [0-9]*: //'); printf "  %s: %s\n" "$$num" "$$title"; done ;; \
  adr-new)   ./scripts/docs/create-adr.sh 2>/dev/null || echo "create-adr.sh not found" ;; \
  diagrams)  mkdir -p docs/architecture/diagrams/generated; if command -v plantuml >/dev/null 2>&1; then for f in docs/architecture/diagrams/*.puml; do [ -f "$$f" ] && plantuml -o generated "$$f" 2>/dev/null || true; done; fi ;; \
  *)         printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_docs)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- codegen (target is APPLY-gated; phases overwrite generated code) --------
define DISPATCH_CODEGEN
@$(call gate,regenerate generated code); case "$(WHAT)" in \
  cli)         echo "Building sea-orm-cli from fork..."; cargo build --manifest-path=third-party/sea-orm/sea-orm-cli/Cargo.toml; echo "✓ $(SEA_ORM_CLI)" ;; \
  db)          rm -f $(CODEGEN_DB); python3 $(EXTRACT_SCRIPT) $(MIGRATION_RS) | sqlite3 $(CODEGEN_DB); echo "✓ codegen DB at $(CODEGEN_DB)" ;; \
  entities)    $(MAKE) codegen WHAT=db APPLY=Y; $(SEA_ORM_CLI) generate entity --database-url "sqlite://$(CODEGEN_DB)?mode=rwc" --output-dir $(ENTITIES_DIR) --with-serde both --ignore-tables seaql_migrations --date-time-crate time; python3 scripts/codegen-post-process.py $(ENTITIES_DIR)/mod.rs; echo "✓ entities in $(ENTITIES_DIR)/" ;; \
  conversions) echo "Generating conversions from $(CONVERSIONS_TOML)..."; python3 $(CONVERSIONS_SCRIPT); echo "✓ conversions in $(CONVERSIONS_DIR)/" ;; \
  clean)       rm -f $(CODEGEN_DB); echo "✓ cleaned codegen artifacts" ;; \
  ""|all)      $(MAKE) codegen WHAT=entities APPLY=Y; $(MAKE) codegen WHAT=conversions APPLY=Y; echo "✓ codegen complete" ;; \
  *)           printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_codegen)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- release / install / version --------------------------------------------
define DISPATCH_RELEASE
@case "$(WHAT)" in \
  ""|package) echo "Creating release v$(VERSION)..."; $(MAKE) check WHAT=lint && $(MAKE) test && bash $(MCB_SH) validate quick && $(MAKE) build RELEASE=1; mkdir -p dist; [ -f "target/release/$(BINARY_NAME)" ] || { echo "Error: target/release/$(BINARY_NAME) not found" >&2; exit 1; }; cp target/release/$(BINARY_NAME) dist/; (cd dist && tar -czf $(BINARY_NAME)-$(VERSION).tar.gz $(BINARY_NAME)); echo "Release ready: dist/$(BINARY_NAME)-$(VERSION).tar.gz" ;; \
  version)    $(call MCB_VERSION_BUMP) ;; \
  install)    $(call gate,install MCB v$(VERSION) to $(INSTALL_DIR) + systemd + MCP configs); $(call MCB_INSTALL) ;; \
  install-validate) $(call MCB_INSTALL_VALIDATE) ;; \
  *)          printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_release)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

define MCB_VERSION_BUMP
case "$(BUMP)" in \
  patch) sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_PATCH)"/' Cargo.toml; cargo check 2>/dev/null || true; echo "Version → $(NEXT_PATCH)" ;; \
  minor) sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MINOR)"/' Cargo.toml; cargo check 2>/dev/null || true; echo "Version → $(NEXT_MINOR)" ;; \
  major) sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MAJOR)"/' Cargo.toml; cargo check 2>/dev/null || true; echo "Version → $(NEXT_MAJOR)" ;; \
  *)     echo "Current: $(VERSION)"; echo "patch:   $(NEXT_PATCH)"; echo "minor:   $(NEXT_MINOR)"; echo "major:   $(NEXT_MAJOR)" ;; \
esac
endef

# Full installer (folds former scripts/install-user-service.sh + migrate-config.sh).
define MCB_INSTALL
echo "Installing MCB v$(VERSION)..."; \
$(MAKE) build RELEASE=1; \
mkdir -p $(INSTALL_DIR) $(CARGO_BIN_DIR) $(SYSTEMD_USER_DIR) $(CONFIG_YAML_DIR) $(DATA_DIR) || { echo "FAIL: mkdir" >&2; exit 1; }; \
cp config/development.yaml "$(CONFIG_YAML_DIR)/development.yaml" || { echo "FAIL: development.yaml" >&2; exit 1; }; \
[ -f config/production.yaml ] && cp config/production.yaml "$(CONFIG_YAML_DIR)/production.yaml" || true; \
sed -i 's|uri: sqlite://mcb.db|uri: sqlite://$(DATA_DIR)/mcb.db|' "$(CONFIG_YAML_DIR)/development.yaml"; \
cp config/deploy.toml "$(CONFIG_DIR)/mcb.toml" || { echo "FAIL: deploy.toml" >&2; exit 1; }; \
sed -i 's|path = "mcb.db"|path = "$(DATA_DIR)/mcb.db"|' "$(CONFIG_DIR)/mcb.toml"; \
mkdir -p "$(DATA_DIR)/config"; \
cp config/production.yaml "$(DATA_DIR)/config/production.yaml" || { echo "FAIL: production.yaml" >&2; exit 1; }; \
sed -i 's|uri: sqlite://mcb.db|uri: sqlite://$(DATA_DIR)/mcb.db|' "$(DATA_DIR)/config/production.yaml"; \
systemctl --user stop mcb.service 2>/dev/null || true; systemctl --user reset-failed mcb.service 2>/dev/null || true; sleep 1; \
P=$$(pgrep -x $(BINARY_NAME) 2>/dev/null || true); [ -n "$$P" ] && { echo "$$P" | xargs kill 2>/dev/null || true; sleep 2; echo "$$P" | xargs kill -9 2>/dev/null || true; } || true; \
cp target/release/$(BINARY_NAME) "$(INSTALL_DIR)/$(BINARY_NAME).new" || { echo "FAIL: copy binary" >&2; exit 1; }; \
chmod +x "$(INSTALL_DIR)/$(BINARY_NAME).new"; \
mv -f "$(INSTALL_DIR)/$(BINARY_NAME).new" "$(INSTALL_DIR)/$(BINARY_NAME)" || { echo "FAIL: install binary" >&2; exit 1; }; \
cp "$(INSTALL_DIR)/$(BINARY_NAME)" "$(CARGO_BIN_DIR)/$(BINARY_NAME)" 2>/dev/null || true; \
$(INSTALL_DIR)/$(BINARY_NAME) --version >/dev/null 2>&1 || { echo "FAIL: binary validation" >&2; exit 1; }; \
cp systemd/mcb.service $(SYSTEMD_USER_DIR)/mcb.service || { echo "FAIL: service file" >&2; exit 1; }; \
systemctl --user daemon-reload || { echo "FAIL: daemon-reload" >&2; exit 1; }; \
systemctl --user enable mcb.service 2>/dev/null || true; systemctl --user reset-failed mcb.service 2>/dev/null || true; \
systemctl --user start mcb.service || { echo "FAIL: start service" >&2; exit 1; }; \
echo "  binary + config + service installed"; \
$(MAKE) release WHAT=install-validate
endef

define MCB_INSTALL_VALIDATE
echo "── Validating installation ──"; \
$(INSTALL_DIR)/$(BINARY_NAME) --version 2>/dev/null | grep -q mcb || { echo "  FAIL: binary not responding" >&2; exit 1; }; \
echo "  Binary: $$($(INSTALL_DIR)/$(BINARY_NAME) --version)"; \
[ -f "$(CONFIG_YAML_DIR)/development.yaml" ] && echo "  Config: $(CONFIG_YAML_DIR)/development.yaml" || echo "  WARN: no installed config"; \
R=0; while [ $$R -lt 8 ]; do systemctl --user is-active --quiet mcb.service 2>/dev/null && { echo "  Service: active"; break; }; R=$$((R+1)); [ $$R -lt 8 ] && sleep 2; done; \
[ $$R -eq 8 ] && echo "  WARN: service not active (journalctl --user -u mcb.service)" || true; \
RES=$$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"install-validate","version":"1.0"}}}' | timeout 15 $(INSTALL_DIR)/$(BINARY_NAME) serve --stdio 2>/dev/null); \
echo "$$RES" | grep -q '"serverInfo"' && echo "  MCP stdio: OK" || { echo "  FAIL: MCP stdio no response" >&2; exit 1; }; \
echo "  MCB v$(VERSION) installed: $(INSTALL_DIR)/$(BINARY_NAME)"
endef

# --- git ---------------------------------------------------------------------
define DISPATCH_GIT
@case "$(WHAT)" in \
  ""|status)  git status --short; git submodule foreach --quiet 'S=$$(git status --short); [ -n "$$S" ] && { echo "$$name:"; echo "$$S"; } || true' ;; \
  diff)       git diff; git diff --cached ;; \
  log)        git log --oneline -$(or $(LOG_N),10) ;; \
  show)       git show --stat $(or $(REF),HEAD) ;; \
  add)        bash $(MCB_SH) files-safe "$(FILES)"; $(call require_var,FILES); git add $(FILES) ;; \
  commit)     $(call require_var,MSG); bash $(MCB_SH) files-safe "$(FILES)"; [ -n "$(FILES)" ] && git add $(FILES) || true; $(call gate,commit); git commit -m "$(MSG)" ;; \
  push)       $(call gate,push $(BRANCH)); git push origin $(BRANCH) ;; \
  pull)       git pull origin $(BRANCH) ;; \
  branch)     [ -z "$(REF)" ] && git branch -a || git branch $(REF) $(BASE) ;; \
  checkout)   $(call require_var,REF); git checkout $(REF) ;; \
  tag)        $(call require_var,TAG); [ -n "$(MSG)" ] && git tag -a $(TAG) -m "$(MSG)" || git tag $(TAG) ;; \
  tags)       git tag -l --sort=-version:refname | head -20 ;; \
  stash)      [ -n "$(MSG)" ] && git stash push -m "$(MSG)" || git stash push ;; \
  stash-pop)  git stash pop ;; \
  stash-list) git stash list ;; \
  merge)      $(call require_var,REF); $(call gate,merge $(REF)); git merge --no-ff $(REF) ;; \
  rebase)     $(call gate,rebase onto $(BASE)); git rebase $(BASE) ;; \
  unstage)    $(call require_var,FILES); git restore --staged $(FILES) ;; \
  push-tags)  $(call require_var,TAG); $(call gate,push tag $(TAG) to origin); git push origin $(TAG) ;; \
  *)          printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_git)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- pr (GitHub) -------------------------------------------------------------
define DISPATCH_PR
@case "$(WHAT)" in \
  checks)     $(call require_var,PR); gh pr checks $(PR) || true ;; \
  ""|view)    $(call require_var,PR); gh pr view $(PR) ;; \
  merge)      $(call require_var,PR); $(call gate,merge PR #$(PR)); gh pr merge $(PR) --merge ;; \
  rerun)      $(call require_var,RUN); gh run rerun $(RUN) --failed ;; \
  *)          printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_pr)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- sub (submodules) --------------------------------------------------------
define DISPATCH_SUB
@case "$(WHAT)" in \
  ""|status)  git submodule status ;; \
  sync)       git submodule sync --recursive; git submodule update --init --recursive ;; \
  diff)       git submodule foreach --quiet 'D=$$(git diff); [ -n "$$D" ] && { echo "=== $$name ==="; git diff; } || true' ;; \
  commit)     $(call require_var,SUB); $(call require_var,MSG); $(call gate,commit in submodule $(SUB)); (cd third-party/$(SUB) && git add -A && git commit -m "$(MSG)") ;; \
  push)       $(call require_var,SUB); $(call gate,push submodule $(SUB)); (cd third-party/$(SUB) && git push) ;; \
  propagate)  $(call require_var,SUB); git add third-party/$(SUB); echo "staged third-party/$(SUB); commit with: make git WHAT=commit MSG='chore: update $(SUB)' APPLY=Y" ;; \
  *)          printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_sub)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- setup -------------------------------------------------------------------
define DISPATCH_SETUP
@case "$(WHAT)" in \
  hooks)     cp scripts/hooks/pre-commit scripts/hooks/pre-push .git/hooks/; chmod +x .git/hooks/pre-commit .git/hooks/pre-push; echo "✓ pre-commit + pre-push hooks installed" ;; \
  tools)     cargo install cargo-udeps cargo-audit cargo-tarpaulin cargo-nextest typos-cli 2>/dev/null || true; echo "✓ tools installed" ;; \
  adr)       ./scripts/setup/install-adr-tools.sh ;; \
  ""|all)    cp scripts/hooks/pre-commit scripts/hooks/pre-push .git/hooks/; chmod +x .git/hooks/pre-commit .git/hooks/pre-push; echo "✓ hooks installed"; cargo install cargo-udeps cargo-audit cargo-tarpaulin cargo-nextest typos-cli 2>/dev/null || true; ./scripts/setup/install-adr-tools.sh 2>/dev/null || true; echo "✓ setup complete" ;; \
  *)         printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_setup)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef

# --- clean (target is APPLY-gated) -------------------------------------------
define DISPATCH_CLEAN
@$(call gate,clean build artifacts); case "$(WHAT)" in \
  ""|build)  cargo clean; echo "✓ build artifacts cleaned" ;; \
  codegen)   rm -f $(CODEGEN_DB); echo "✓ codegen DB removed" ;; \
  all)       cargo clean; rm -f $(CODEGEN_DB); echo "✓ all artifacts cleaned" ;; \
  *)         printf "ERRO: WHAT '%s' invalido. Validos: $(WHATS_clean)\n" "$(WHAT)" >&2; exit 2 ;; \
esac
endef
