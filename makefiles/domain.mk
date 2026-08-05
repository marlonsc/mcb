# =============================================================================
# makefiles/domain.mk — MCB domain dispatch (private; invoked via custom.mk).
# Former boot/build/test/check/ship/clean WHAT=/ACT=/SCOPE= phases.
# =============================================================================

MCB_ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST)))/..)
MCB_SH := $(MCB_ROOT)/scripts/lib/mcb.sh
MCB_AUDIT_IGNORES := $(shell bash $(MCB_SH) ignores)
MCB_TOOL := bash $(MCB_SH)
MCB_RUN := bash $(MCB_SH) run

ESC := $(shell printf '\033')
RESET := $(ESC)[0m
BOLD := $(ESC)[1m
RED := $(ESC)[0;31m
ECHO_ERROR = printf "$(RED)%s$(RESET)\n" "$(1)"
require_var = [ -n "$($(1))" ] || { $(ECHO_ERROR) "$(1) is required. Example: make $(1)=<value>"; exit 2; }
gate = [ "$(APPLY)" = "Y" ] || { printf "DRY-RUN: would %s; set APPLY=Y to execute\n" "$(1)" >&2; exit 0; }

export RELEASE ?= 1
export QUICK ?= 0
export FIX ?= 0
export THREADS ?= $(shell nproc 2>/dev/null || echo 1)
export SCOPE ?=
WHAT ?=
ACT ?=
APPLY ?= N
BUMP ?=
FILES ?=
MSG ?=
REF ?=
TAG ?=
BASE ?= main
BRANCH ?= $(shell git rev-parse --abbrev-ref HEAD 2>/dev/null)
PR ?=
RUN ?=
SUB ?=
LOG_N ?=
export RUST_2024_LINTS := -D unsafe_op_in_unsafe_fn -D rust_2024_compatibility -W static_mut_refs
export RUSTC_WRAPPER := sccache
export CARGO_INCREMENTAL := 0

ifeq ($(shell command -v sccache 2>/dev/null),)
$(warning sccache not found in PATH. Attempting install...)
$(shell cargo install sccache --locked 2>/dev/null || true)
endif

WHATS_boot    := hooks hook tools adr venv all
WHATS_build   := build debug release prebuild codegen docs
WHATS_check   := fmt lint validate audit udeps coverage qlty coordination guard fix dev optimize gitops surface python ci all
WHATS_ship    := status diff log show add commit push pull branch checkout tag tags stash stash-pop stash-list merge rebase unstage push-tags pr sub release
WHATS_clean   := build codegen all
ACTS_hook     := pre-commit pre-push
ACTS_docs     := build serve lint validate sync rust check setup adr adr-new diagrams
ACTS_codegen  := all cli db entities conversions clean
ACTS_fix      := fmt lint docs all
ACTS_dev      := run docker-up docker-down docker-logs docker-test
ACTS_pr       := checks view merge rerun
ACTS_sub      := status sync diff commit push propagate
ACTS_release  := package version install install-validate
ACTS_python   := lint lint-staged test test-staged guard all
ACTS_optimize := cache

_ACT_VALID := $(ACTS_hook) $(ACTS_docs) $(ACTS_codegen) $(ACTS_fix) $(ACTS_dev) $(ACTS_pr) $(ACTS_sub) $(ACTS_release) $(ACTS_python) $(ACTS_optimize)
ifneq ($(ACT),)
  ifeq ($(filter $(ACT),$(_ACT_VALID)),)
    ACT :=
  endif
endif

.PHONY: _mcb_internal_boot _mcb_internal_build _mcb_internal_test _mcb_internal_check _mcb_internal_ship _mcb_internal_clean
_mcb_internal_boot:   ; $(call DISPATCH_BOOT)
_mcb_internal_build:  ; $(call DISPATCH_BUILD)
_mcb_internal_test:   ; $(call DISPATCH_TEST)
_mcb_internal_check:  ; $(call DISPATCH_CHECK)
_mcb_internal_ship:   ; $(call DISPATCH_SHIP)
_mcb_internal_clean:  ; $(call DISPATCH_CLEAN)

# --- verb-local variables (single home) --------------------------------------
MDBOOK         := $(shell command -v mdbook 2>/dev/null || echo "$(HOME)/.cargo/bin/mdbook")
MCB_TEST_PORT  ?= 18080
# Cap pre-push test parallelism to avoid overwhelming the host when nproc is high.
# Honor user-supplied THREADS if it is already <= 4; otherwise clamp to 4.
MCB_PUSH_THREADS := $(or $(filter 1 2 3 4,$(THREADS)),4)

# Detect cargo-nextest robustly (the binary is installed as `cargo-nextest`,
# but `cargo nextest --version` is the portable check).
MCB_NEXTEST := $(shell cargo nextest --version >/dev/null 2>&1 && echo 1)

# Choose nextest profile: CI/pre-push uses the `ci` profile unless overridden.
MCB_NEXTEST_PROFILE := $(or $(NEXTEST_PROFILE),$(if $(filter true 1,$(CI)),ci,default))

# Test runner: prefer cargo-nextest (faster, parallel, better output) when installed;
# fall back to `cargo test`. Doctests always use `cargo test --doc` (nextest can't
# run them) — semantics preserved since `cargo test --all-targets` also skips doctests.
ifeq ($(MCB_NEXTEST),1)
  MCB_TEST_UNIT := MCB_MODEL_ID=test-model NEXTEST_TEST_THREADS=$$T $(MCB_RUN) cargo nextest run --profile $(MCB_NEXTEST_PROFILE) --workspace --test unit
  # Pre-commit tier: skip live-workspace quality scans that exceed the hook budget.
  MCB_TEST_UNIT_PRECOMMIT := MCB_MODEL_ID=test-model NEXTEST_TEST_THREADS=$$T $(MCB_RUN) cargo nextest run --profile $(MCB_NEXTEST_PROFILE) --workspace --test unit -E 'not test(/test_validate_(with_specific_validator|mcb_workspace_quality)/)'
  MCB_TEST_ALL  := MCB_MODEL_ID=test-model NEXTEST_TEST_THREADS=$$T $(MCB_RUN) cargo nextest run --profile $(MCB_NEXTEST_PROFILE) --workspace
  # Run only crates that contain changed .rs files vs origin/$(BRANCH).
else
  MCB_TEST_UNIT := MCB_MODEL_ID=test-model RUST_TEST_THREADS=$$T $(MCB_RUN) cargo test --workspace --test unit --test-threads=$$T
  MCB_TEST_UNIT_PRECOMMIT := MCB_MODEL_ID=test-model RUST_TEST_THREADS=$$T $(MCB_RUN) cargo test --workspace --test unit --test-threads=$$T -- --skip test_validate_with_specific_validator --skip test_validate_with_specific_validator_filters_correctly --skip test_validate_with_specific_validator_does_not_fail_on_unrelated_validators --skip test_validate_mcb_workspace_quality_only
  MCB_TEST_ALL  := MCB_MODEL_ID=test-model RUST_TEST_THREADS=$$T $(MCB_RUN) cargo test --workspace --all-targets --test-threads=$$T
endif

# Helper to test a specific set of crates (shell $$CRATES must be set).
define MCB_TEST_CRATES
if [ "$(MCB_NEXTEST)" = "1" ]; then MCB_MODEL_ID=test-model NEXTEST_TEST_THREADS=$$T $(MCB_RUN) cargo nextest run --profile $(MCB_NEXTEST_PROFILE) $$CRATES; else MCB_MODEL_ID=test-model RUST_TEST_THREADS=$$T $(MCB_RUN) cargo test --all-targets --test-threads=$$T $$CRATES; fi
endef

# Install Rust tooling: prefer cargo-binstall when available, else cargo install.
# This is an optimization, not a workaround; environments without binstall keep working.
MCB_BINSTALL := $(shell command -v cargo-binstall >/dev/null 2>&1 && echo 1)
ifeq ($(MCB_BINSTALL),1)
  MCB_INSTALL_CRATES = $(MCB_RUN) cargo binstall -y $(1)
else
  MCB_INSTALL_CRATES = $(MCB_RUN) cargo install --locked $(1)
endif

# Unknown-WHAT error arm (SSOT): the default case of every verb prints this.
BAD_WHAT = printf "ERRO: WHAT '%s' invalido. Validos: $(1)\n" "$(WHAT)" >&2; exit 2

# codegen
CODEGEN_DB         := /tmp/mcb_codegen.db
MIGRATION_RS       := crates/mcb-providers/src/database/seaorm/migration/m20260301_000001_initial_schema.rs
SEA_ORM_CLI        := $(shell command -v sea-orm-cli 2>/dev/null || echo "$(HOME)/.cargo/bin/sea-orm-cli")
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

# =============================================================================
# boot — bootstrap dev environment: git hooks, tooling, ADR tools.
#   (former `setup` verb; WHAT=hooks|tools|adr|hook|all)
#   WHAT=hook installs+drives the tiered git-hook gate via ACT=pre-commit|pre-push.
# =============================================================================
define DISPATCH_BOOT
@case "$(WHAT)" in \
  hooks)     cp scripts/hooks/pre-commit scripts/hooks/pre-push .git/hooks/; chmod +x .git/hooks/pre-commit .git/hooks/pre-push; echo "✓ pre-commit + pre-push hooks installed" ;; \
  tools)     $(call MCB_INSTALL_CRATES,cargo-udeps cargo-audit cargo-tarpaulin cargo-nextest typos-cli cargo-sweep) 2>/dev/null || true; echo "✓ tools installed" ;; \
  adr)       ./scripts/setup/install-adr-tools.sh ;; \
  venv)      UV_CACHE_DIR=.cache/uv uv sync --extra dev --extra gitops ;; \
  hook)      $(call MCB_HOOK) ;; \
  ""|all)    cp scripts/hooks/pre-commit scripts/hooks/pre-push .git/hooks/; chmod +x .git/hooks/pre-commit .git/hooks/pre-push; echo "✓ hooks installed"; $(call MCB_INSTALL_CRATES,cargo-udeps cargo-audit cargo-tarpaulin cargo-nextest typos-cli) 2>/dev/null || true; ./scripts/setup/install-adr-tools.sh 2>/dev/null || true; UV_CACHE_DIR=.cache/uv uv sync --extra dev --extra gitops; echo "✓ boot complete" ;; \
  *)         $(call BAD_WHAT,$(WHATS_boot)) ;; \
esac
endef

# tiered native git-hook gates; SSOT for pre-commit/pre-push, selected by ACT=.
# pre-commit (fast): guard --staged + python lint/test + fmt + clippy(workspace) + typos + unit tests.
# pre-push (full): python gates + gitops + fmt + clippy --all-targets + full suite + doctests + validate + guard.
# Same gates the CI runs, one definition. No bypass (AGENTS.md §3).
define MCB_HOOK
case "$(ACT)" in \
  pre-commit) \
    T="$(THREADS)"; case "$$T" in ''|*[!0-9]*|0) T=1;; esac; \
    $(MCB_TOOL) guard --staged && \
    $(MAKE) check WHAT=python ACT=lint-staged && \
    $(MAKE) check WHAT=python ACT=test-staged && \
    $(MCB_RUN) cargo fmt --all -- --check && \
    $(MCB_RUN) cargo clippy --workspace -- -D warnings && \
    { ! command -v typos >/dev/null 2>&1 || typos crates/ scripts/ docs/ config/ AGENTS.md; } && \
    $(MCB_TEST_UNIT_PRECOMMIT) ;; \
  pre-push) \
    $(MAKE) check WHAT=python ACT= && \
    $(MAKE) check WHAT=gitops ACT= && \
    $(MCB_RUN) cargo fmt --all -- --check && \
    $(MCB_RUN) cargo clippy --all-targets -- -D warnings && \
    $(MAKE) test WHAT=all THREADS=$(MCB_PUSH_THREADS) && $(MAKE) test WHAT=doc THREADS=$(MCB_PUSH_THREADS) && \
    $(MCB_TOOL) validate quick && \
    $(MCB_TOOL) guard ;; \
  *)          printf "ERRO: ACT '%s' invalido. Validos: $(ACTS_hook)\n" "$(ACT)" >&2; exit 2 ;; \
esac
endef

# =============================================================================
# build — compile, codegen, docs (former `build` + `codegen` + `docs` verbs).
#   WHAT=""|debug|release : cargo build (RELEASE=0|1 still honored for "")
#   WHAT=codegen ACT=...  : APPLY-gated SeaORM codegen
#   WHAT=docs ACT=...     : docs pipeline (build/serve/lint/validate/sync/rust/...)
# =============================================================================
define DISPATCH_BUILD
@case "$(WHAT)" in \
  ""|build) \
    if [ "$(RELEASE)" = "1" ]; then echo "Building release..."; $(MCB_RUN) cargo build --release; \
    else echo "Building debug..."; $(MCB_RUN) cargo build; fi ;; \
  release) echo "Building release..."; $(MCB_RUN) cargo build --release ;; \
  debug)   echo "Building debug..."; $(MCB_RUN) cargo build ;; \
  prebuild) echo "Pre-building all test targets..."; $(MCB_RUN) cargo test --workspace --all-targets --no-run ;; \
  codegen) $(call MCB_CODEGEN) ;; \
  docs)    $(call MCB_DOCS) ;; \
  *)       $(call BAD_WHAT,$(WHATS_build)) ;; \
esac
endef

# codegen (APPLY-gated; phases overwrite generated code). ACT= selects phase.
define MCB_CODEGEN
case "$(ACT)" in \
  cli)         $(call gate,install sea-orm-cli from fork); echo "Installing sea-orm-cli from fork..."; $(MCB_RUN) cargo install --locked --git https://github.com/marlon-costa-dc/sea-orm.git --rev c1b8409a45c5dc20de91e331ee0cbb86fb9a72d0 sea-orm-cli; echo "✓ $(SEA_ORM_CLI)" ;; \
  db)          $(call gate,regenerate codegen database); rm -f $(CODEGEN_DB); $(MCB_RUN) python3 $(EXTRACT_SCRIPT) $(MIGRATION_RS) | sqlite3 $(CODEGEN_DB); echo "✓ codegen DB at $(CODEGEN_DB)" ;; \
  entities)    $(call gate,regenerate SeaORM entities); $(MAKE) build WHAT=codegen ACT=db APPLY=Y; $(SEA_ORM_CLI) generate entity --database-url "sqlite://$(CODEGEN_DB)?mode=rwc" --output-dir $(ENTITIES_DIR) --with-serde both --ignore-tables seaql_migrations --date-time-crate time; $(MCB_RUN) python3 scripts/codegen-post-process.py $(ENTITIES_DIR)/mod.rs; echo "✓ entities in $(ENTITIES_DIR)/" ;; \
  conversions) $(call gate,regenerate SeaORM conversions); echo "Generating conversions from $(CONVERSIONS_TOML)..."; $(MCB_RUN) python3 $(CONVERSIONS_SCRIPT); echo "✓ conversions in $(CONVERSIONS_DIR)/" ;; \
  clean)       $(call gate,clean codegen artifacts); rm -f $(CODEGEN_DB); echo "✓ cleaned codegen artifacts" ;; \
  ""|all)      $(call gate,regenerate generated code); $(MAKE) build WHAT=codegen ACT=entities APPLY=Y; $(MAKE) build WHAT=codegen ACT=conversions APPLY=Y; echo "✓ codegen complete" ;; \
  *)           printf "ERRO: ACT '%s' invalido. Validos: $(ACTS_codegen)\n" "$(ACT)" >&2; exit 2 ;; \
esac
endef

# docs pipeline. ACT= selects phase.
define MCB_DOCS
case "$(ACT)" in \
  ""|build)  $(call gate,build generated docs); $(MCB_RUN) ./scripts/docs/inject-metrics.sh; $(MCB_RUN) cargo doc --no-deps --workspace; $(MCB_RUN) ./scripts/docs/mdbook-sync.sh; if [ -x "$(MDBOOK)" ]; then $(MDBOOK) build book/; else echo "Warning: mdbook not found, skipping book build" >&2; fi ;; \
  serve)     $(call gate,serve generated docs); $(MCB_RUN) ./scripts/docs/mdbook-sync.sh 2>/dev/null || true; if [ -x "$(MDBOOK)" ]; then $(MDBOOK) serve book/ --open; else echo "mdbook not installed (cargo install mdbook)"; fi ;; \
  lint)      if [ "$(FIX)" = "1" ]; then $(call gate,fix markdown docs); $(MCB_RUN) ./scripts/docs/markdown.sh fix; else $(MCB_RUN) ./scripts/docs/markdown.sh lint; fi ;; \
  validate)  QUICK="$(QUICK)" $(MCB_RUN) ./scripts/docs/validate.sh all ;; \
  sync)      $(call gate,sync generated docs); $(MCB_RUN) ./scripts/docs/mdbook-sync.sh 2>/dev/null || true ;; \
  rust)      $(MCB_RUN) cargo doc --no-deps --workspace ;; \
  check)     [ -d docs ] || { echo "ERROR: docs/ directory not found" >&2; exit 1; } ;; \
  setup)     $(call gate,setup docs workspace); mkdir -p book; [ -f book.toml ] || { echo "ERROR: book.toml not found in root" >&2; exit 1; } ;; \
  adr)       echo "Architecture Decision Records:"; ls -1 docs/adr/[0-9]*.md 2>/dev/null | while read f; do num=$$(basename "$$f" .md | cut -d- -f1); title=$$(head -1 "$$f" | sed 's/^# ADR [0-9]*: //'); printf "  %s: %s\n" "$$num" "$$title"; done ;; \
  adr-new)   $(call gate,create ADR); ./scripts/docs/create-adr.sh 2>/dev/null || echo "create-adr.sh not found" ;; \
  diagrams)  $(call gate,generate diagrams); mkdir -p docs/architecture/diagrams/generated; if command -v plantuml >/dev/null 2>&1; then for f in docs/architecture/diagrams/*.puml; do [ -f "$$f" ] && plantuml -o generated "$$f" 2>/dev/null || true; done; fi ;; \
  *)         printf "ERRO: ACT '%s' invalido. Validos: $(ACTS_docs)\n" "$(ACT)" >&2; exit 2 ;; \
esac
endef

# =============================================================================
# test (SCOPE dispatch; e2e folded in) — unchanged public verb.
# =============================================================================
define DISPATCH_TEST
@T="$(THREADS)"; case "$$T" in ''|*[!0-9]*|0) T=1;; esac; \
case "$(SCOPE)" in \
  unit)        $(MCB_TEST_UNIT) ;; \
  doc)         $(MCB_RUN) cargo test --workspace --doc ;; \
  golden)      RUST_TEST_THREADS=$$T $(MCB_RUN) cargo test --workspace --tests golden ;; \
  startup)     $(MCB_RUN) cargo test -p mcb --test integration startup_smoke -- --nocapture ;; \
  warmup)      $(MCB_RUN) cargo test -p mcb-server --test integration test_init_app_with_default_config_succeeds -- --nocapture ;; \
  integration) MCB_MODEL_ID=test-model RUST_TEST_THREADS=$$T $(MCB_RUN) cargo test --workspace --test '*integration*' ;; \
  external)    UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync python scripts/lib/external_services_check.py && \
                 { MCB_MODEL_ID=test-model RUST_TEST_THREADS=$$T $(MCB_RUN) cargo test --workspace --test '*integration*'; } || \
                 { echo "⊘ External test group skipped: services unavailable."; exit 0; } ;; \
  changed)     MCB_MODEL_ID=test-model $(MCB_RUN) echo "Running tests for changed crates..."; \
               CRATES="$$(git diff --name-only origin/$$(git rev-parse --abbrev-ref HEAD) -- 'crates/**/*.rs' 'crates/**/*.toml' | sed -n 's|^crates/\\([^/]*\\)/.*|-p \\1|p' | sort -u | tr '\\n' ' ')"; \
               [ -z "$$CRATES" ] && { echo "No changed crates; running full workspace tests."; $(MCB_TEST_ALL); } || { echo "Changed crates: $$CRATES"; $(call MCB_TEST_CRATES); } ;; \
  e2e)         $(call MCB_E2E) ;; \
  all)         $(MCB_TEST_ALL) && $(call MCB_E2E) ;; \
  '')          $(MCB_TEST_ALL) ;; \
  *)           printf "ERRO: SCOPE '%s' invalido. Validos: unit doc golden startup warmup integration external e2e changed all\n" "$(SCOPE)" >&2; exit 2 ;; \
esac
endef

define MCB_E2E
$(call gate,run Playwright E2E on port $(MCB_TEST_PORT)); \
echo "Running Playwright E2E on port $(MCB_TEST_PORT)..."; \
lsof -ti:$(MCB_TEST_PORT) | xargs -r kill -9 2>/dev/null || true; sleep 1; \
command -v node >/dev/null || { echo "Error: node/npm not found. Install Node.js first." >&2; exit 1; }; \
if [ ! -d tests/node_modules/@playwright ]; then echo "Installing Playwright..."; \
  $(MCB_RUN) npm --prefix tests install --save-dev @playwright/test @types/node typescript 2>&1 | grep -v "npm WARN" || true; \
  (cd tests && $(MCB_RUN) npm exec -- playwright install chromium --with-deps 2>&1 | tail -5); fi; \
$(MCB_RUN) cargo build --release --bin mcb; \
cd tests && $(MCB_RUN) npm exec -- playwright test --config=playwright.config.ts --reporter=list
endef

# =============================================================================
# check — read-only gates + mutating auto-fix + banned-pattern scan + CI gate.
#   (former `check` + `fix` + `guard` + `ci` + `dev` verbs)
#   WHAT=fmt|lint|validate|audit|udeps|coverage|qlty|coordination|all : read-only
#   WHAT=guard : banned-pattern scanner
#   WHAT=ci    : full CI gate (== WHAT=all)
#   WHAT=fix ACT=fmt|lint|docs|all : mutating auto-fix
#   WHAT=dev ACT=run|docker-* : dev server / docker test services
# =============================================================================
define DISPATCH_CHECK
@case "$(WHAT)" in \
  fmt)      $(MCB_RUN) cargo fmt --all -- --check ;; \
  lint)     $(MCB_RUN) cargo fmt --all -- --check && $(MCB_RUN) cargo clippy --all-targets -- -D warnings ;; \
  validate) $(MCB_TOOL) validate $(if $(filter 1,$(QUICK)),quick,full) ;; \
  audit)    $(MCB_RUN) cargo audit $(foreach i,$(MCB_AUDIT_IGNORES),--ignore $(i)) && $(MAKE) check WHAT=udeps ;; \
  udeps)    command -v cargo-udeps >/dev/null 2>&1 || $(MCB_RUN) cargo install cargo-udeps; $(MCB_RUN) cargo +nightly udeps --workspace ;; \
  coverage) $(MCB_RUN) cargo tarpaulin --engine llvm --out Lcov --output-dir coverage --exclude-files 'crates/*/tests/integration/*' --exclude-files 'crates/*/tests/admin/*' --timeout 300 ;; \
  qlty)     mkdir -p docs/reports; $(MCB_RUN) ./scripts/analyze_qlty.py --scan --check --summary-only --report-file docs/reports/qlty-check-REPORTS.md; $(MCB_RUN) ./scripts/analyze_qlty.py --scan --smells --summary-only --report-file docs/reports/qlty-smells-REPORTS.md ;; \
  coordination) bd config get beads.role --json && bd status --json && bd hooks list --json && bash scripts/context/validate-beads-policy.sh && bd dep cycles --json && bd stale --status in_progress --days 1 --limit 25 --json && bd graph --all --compact >/dev/null ;; \
  guard)    $(MCB_TOOL) guard ;; \
  gitops)   UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync python scripts/check/gitops.py ;; \
  surface)  UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync python scripts/check/surface.py ;; \
  python)   $(call MCB_PYTHON_CHECK) ;; \
  fix)      $(call MCB_FIX) ;; \
  dev)      $(call MCB_DEV) ;; \
  optimize) $(call MCB_OPTIMIZE,$(filter $(ACT),$(ACTS_optimize))) ;; \
  ci|""|all) $(MAKE) check WHAT=python && $(MAKE) check WHAT=gitops && $(MCB_RUN) cargo fmt --all -- --check && $(MCB_RUN) cargo clippy --all-targets -- -D warnings && $(MAKE) test && $(MCB_TOOL) validate $(if $(filter 1,$(QUICK)),quick,full) && $(MCB_TOOL) guard ;; \
  *)        UV_CACHE_DIR=.cache/uv PYTHONPATH=scripts $(MCB_RUN) uv run --no-sync python -m lib.cosmos_command check; code=$$?; if [ "$$code" -eq 2 ]; then $(call BAD_WHAT,$(WHATS_check)); else exit $$code; fi ;; \
esac
endef

# Python gates (ruff, mypy, pytest, guard). ACT= selects phase; default runs all.
define MCB_PYTHON_CHECK
case "$(ACT)" in \
  lint)      UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync ruff check scripts/ && UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync mypy scripts/lib ;; \
  lint-staged) \
    STAGED="$$(git diff --cached --name-only --diff-filter=ACM -- '*.py')"; \
    if [ -z "$$STAGED" ]; then echo "lint-staged: no staged Python files"; exit 0; fi; \
    echo "lint-staged: $$STAGED"; \
    UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync ruff check $$STAGED && UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync mypy $$STAGED ;; \
  test)      UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync pytest scripts/lib/tests ;; \
  test-staged) \
    STAGED_TESTS="$$(git diff --cached --name-only --diff-filter=ACM -- '*.py' | grep '^scripts/lib/tests/' || true)"; \
    if [ -z "$$STAGED_TESTS" ]; then echo "test-staged: no staged Python test files"; exit 0; fi; \
    echo "test-staged: $$STAGED_TESTS"; \
    UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync pytest -m "not slow" $$STAGED_TESTS ;; \
  guard)     $(MCB_TOOL) guard ;; \
  ""|all)    UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync ruff check scripts/ && UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync mypy scripts/lib && UV_CACHE_DIR=.cache/uv $(MCB_RUN) uv run --no-sync pytest -m "not slow" scripts/lib/tests && $(MCB_TOOL) guard ;; \
  *)         printf "ERRO: ACT '%s' invalido. Validos: $(ACTS_python)\n" "$(ACT)" >&2; exit 2 ;; \
esac
endef

# mutating auto-fix (rustfmt, clippy --fix, markdown). ACT= selects phase.
define MCB_FIX
case "$(ACT)" in \
  fmt)        $(call gate,auto-fix fmt); $(MCB_RUN) cargo fmt --all ;; \
  lint)       $(call gate,auto-fix lint); $(MCB_RUN) cargo fmt --all && $(MCB_RUN) cargo clippy --fix --allow-dirty --all-targets ;; \
  docs)       $(call gate,auto-fix docs); $(MAKE) build WHAT=docs ACT=lint FIX=1 APPLY=Y ;; \
  ""|all)     $(call gate,auto-fix all); $(MCB_RUN) cargo fmt --all && $(MCB_RUN) cargo clippy --fix --allow-dirty --all-targets && $(MAKE) build WHAT=docs ACT=lint FIX=1 APPLY=Y ;; \
  *)          printf "ERRO: ACT '%s' invalido. Validos: $(ACTS_fix)\n" "$(ACT)" >&2; exit 2 ;; \
esac
endef

# dev server / docker test services. ACT= selects mode.
define MCB_DEV
case "$(ACT)" in \
  ""|run)       $(call gate,start dev server); echo "Starting dev server..."; $(MCB_RUN) cargo watch -x 'run' 2>/dev/null || $(MCB_RUN) cargo run ;; \
  docker-up)    $(call gate,start Docker test services); echo "Starting Docker test services..."; $(MCB_RUN) docker-compose -f tests/docker-compose.yml up -d; sleep 5 ;; \
  docker-down)  $(call gate,stop Docker test services); echo "Stopping Docker test services..."; $(MCB_RUN) docker-compose -f tests/docker-compose.yml down -v ;; \
  docker-logs)  $(call gate,follow Docker logs); $(MCB_RUN) docker-compose -f tests/docker-compose.yml logs -f ;; \
  docker-test)  $(call gate,run Docker test services); $(MCB_RUN) docker-compose -f tests/docker-compose.yml --profile test up --build --abort-on-container-exit test-runner; $(MCB_RUN) docker-compose -f tests/docker-compose.yml --profile test rm -f test-runner ;; \
  *)            printf "ERRO: ACT '%s' invalido. Validos: $(ACTS_dev)\n" "$(ACT)" >&2; exit 2 ;; \
esac
endef

# optimization / cache maintenance. ACT= selects phase.
#   cache)  sccache + target/ cleanup (safe dry-run by default; APPLY=Y to prune)
#   *)      dev-env process cleanup (duplicate rust-analyzer / Serena / cargo)
define MCB_OPTIMIZE
case "$(1)" in \
  cache) \
    bash $(MCB_ROOT)/scripts/cache-maintenance.sh $(if $(filter Y,$(APPLY)),--apply) ;; \
  "") \
    $(MCB_RUN) scripts/dev-env-optimize.sh $(if $(filter Y,$(APPLY)),--apply) ;; \
  *)          printf "ERRO: ACT '%s' invalido. Validos: cache\n" "$(1)" >&2; exit 2 ;; \
esac
endef

# =============================================================================
# ship — git + GitHub PR + submodules + beads + release/install.
#   (former `git` + `pr` + `sub` + `release` verbs)
#   git actions run directly under WHAT= (status/diff/log/.../commit/push/...).
#   nested namespaces use ACT=:
#     WHAT=pr      ACT=checks|view|merge|rerun  PR= RUN=
#     WHAT=sub     ACT=status|sync|diff|commit|push|propagate  SUB= MSG=
#     WHAT=release ACT=package|version|install|install-validate  BUMP=
#   Destructive arms require APPLY=Y via $(call gate,...).
# =============================================================================
define DISPATCH_SHIP
@case "$(WHAT)" in \
  ""|status)  git status --short; git submodule foreach --quiet 'S=$$(git status --short); [ -n "$$S" ] && { echo "$$name:"; echo "$$S"; } || true' ;; \
  diff)       git diff; git diff --cached ;; \
  log)        git log --oneline -$(or $(LOG_N),10) ;; \
  show)       git show --stat $(or $(REF),HEAD) ;; \
  add)        $(call require_var,FILES); $(MCB_TOOL) files-safe "$(FILES)"; $(call gate,stage files $(FILES)); git add $(FILES) ;; \
  commit)     $(call require_var,MSG); $(call gate,commit); $(MCB_TOOL) files-safe "$(FILES)"; [ -n "$(FILES)" ] && git add $(FILES) || true; git commit -m "$(MSG)" ;; \
  push)       $(call gate,push $(BRANCH)); git push origin $(BRANCH) ;; \
  pull)       $(call gate,pull origin $(BRANCH)); git pull origin $(BRANCH) ;; \
  branch)     [ -z "$(REF)" ] && git branch -a || { $(call gate,create branch $(REF) from $(BASE)); git branch $(REF) $(BASE); } ;; \
  checkout)   $(call require_var,REF); $(call gate,checkout $(REF)); git checkout $(REF) ;; \
  tag)        $(call require_var,TAG); $(call gate,tag $(TAG)); [ -n "$(MSG)" ] && git tag -a $(TAG) -m "$(MSG)" || git tag $(TAG) ;; \
  tags)       git tag -l --sort=-version:refname | head -20 ;; \
  stash)      $(call gate,stash changes); [ -n "$(MSG)" ] && git stash push -m "$(MSG)" || git stash push ;; \
  stash-pop)  $(call gate,stash pop); git stash pop ;; \
  stash-list) git stash list ;; \
  merge)      $(call require_var,REF); $(call gate,merge $(REF)); git merge --no-ff $(REF) ;; \
  rebase)     $(call gate,rebase onto $(BASE)); git rebase $(BASE) ;; \
  unstage)    $(call require_var,FILES); $(call gate,unstage files $(FILES)); git reset -q HEAD -- $(FILES) ;; \
  push-tags)  $(call require_var,TAG); $(call gate,push tag $(TAG) to origin); git push origin $(TAG) ;; \
  pr)         $(call MCB_PR) ;; \
  sub)        $(call MCB_SUB) ;; \
  release)    $(call MCB_RELEASE) ;; \
  *)          $(call BAD_WHAT,$(WHATS_ship)) ;; \
esac
endef

# GitHub PR. ACT= selects action.
define MCB_PR
case "$(ACT)" in \
  checks)     $(call require_var,PR); $(MCB_RUN) gh pr checks $(PR) || true ;; \
  ""|view)    $(call require_var,PR); $(MCB_RUN) gh pr view $(PR) ;; \
  merge)      $(call require_var,PR); $(call gate,merge PR #$(PR)); $(MCB_RUN) gh pr merge $(PR) --merge ;; \
  rerun)      $(call require_var,RUN); $(call gate,rerun GitHub Actions run $(RUN)); $(MCB_RUN) gh run rerun $(RUN) --failed ;; \
  *)          printf "ERRO: ACT '%s' invalido. Validos: $(ACTS_pr)\n" "$(ACT)" >&2; exit 2 ;; \
esac
endef

# submodules. ACT= selects action.
# NOTE: third-party/ submodules were removed. SeaQL/Loco forks are now
# consumed as git dependencies pinned in Cargo.toml. Only status remains.
define MCB_SUB
case "$(ACT)" in \
  ""|status)  echo "third-party/ submodules removed; forks are git dependencies in Cargo.toml" ;; \
  *)          printf "ERRO: ACT '%s' invalido. Submodules removed; use git deps in Cargo.toml instead.\n" "$(ACT)" >&2; exit 2 ;; \
esac
endef

# release / install / version. ACT= selects phase.
define MCB_RELEASE
case "$(ACT)" in \
  ""|package) $(call gate,package release v$(VERSION)); echo "Creating release v$(VERSION)..."; $(MAKE) check WHAT=lint && $(MAKE) test && $(MCB_TOOL) validate quick && $(MAKE) build WHAT=release; mkdir -p dist; [ -f "target/release/$(BINARY_NAME)" ] || { echo "Error: target/release/$(BINARY_NAME) not found" >&2; exit 1; }; cp target/release/$(BINARY_NAME) dist/; (cd dist && tar -czf $(BINARY_NAME)-$(VERSION).tar.gz $(BINARY_NAME)); echo "Release ready: dist/$(BINARY_NAME)-$(VERSION).tar.gz" ;; \
  version)    $(call MCB_VERSION_BUMP) ;; \
  install)    $(call gate,install MCB v$(VERSION) to $(INSTALL_DIR) + systemd + MCP configs); $(call MCB_INSTALL) ;; \
  install-validate) $(call MCB_INSTALL_VALIDATE) ;; \
  *)          printf "ERRO: ACT '%s' invalido. Validos: $(ACTS_release)\n" "$(ACT)" >&2; exit 2 ;; \
esac
endef

define MCB_VERSION_BUMP
case "$(BUMP)" in \
  patch) $(call gate,version bump to $(NEXT_PATCH)); sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_PATCH)"/' Cargo.toml; $(MCB_RUN) cargo check 2>/dev/null || true; echo "Version → $(NEXT_PATCH)" ;; \
  minor) $(call gate,version bump to $(NEXT_MINOR)); sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MINOR)"/' Cargo.toml; $(MCB_RUN) cargo check 2>/dev/null || true; echo "Version → $(NEXT_MINOR)" ;; \
  major) $(call gate,version bump to $(NEXT_MAJOR)); sed -i 's/^version = "$(VERSION)"/version = "$(NEXT_MAJOR)"/' Cargo.toml; $(MCB_RUN) cargo check 2>/dev/null || true; echo "Version → $(NEXT_MAJOR)" ;; \
  *)     echo "Current: $(VERSION)"; echo "patch:   $(NEXT_PATCH)"; echo "minor:   $(NEXT_MINOR)"; echo "major:   $(NEXT_MAJOR)" ;; \
esac
endef

# Full installer (folds former scripts/install-user-service.sh + migrate-config.sh).
define MCB_INSTALL
echo "Installing MCB v$(VERSION)..."; \
$(MAKE) build WHAT=release; \
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
JWT_SECRET_FILE="$(DATA_DIR)/.jwt_secret"; \
if [ -f "$$JWT_SECRET_FILE" ]; then \
  JWT_SECRET=$$(cat "$$JWT_SECRET_FILE"); \
else \
  JWT_SECRET=$$(head -c 48 /dev/urandom | base64 | tr -d '\n'); \
  echo "$$JWT_SECRET" > "$$JWT_SECRET_FILE"; \
  chmod 600 "$$JWT_SECRET_FILE"; \
fi; \
cp systemd/mcb.service $(SYSTEMD_USER_DIR)/mcb.service || { echo "FAIL: service file" >&2; exit 1; }; \
sed -i "s|Environment=LOCO_ENV=production|Environment=LOCO_ENV=production\\nEnvironment=JWT_SECRET=$$JWT_SECRET|" $(SYSTEMD_USER_DIR)/mcb.service; \
systemctl --user daemon-reload || { echo "FAIL: daemon-reload" >&2; exit 1; }; \
systemctl --user enable mcb.service 2>/dev/null || true; systemctl --user reset-failed mcb.service 2>/dev/null || true; \
systemctl --user start mcb.service || { echo "FAIL: start service" >&2; exit 1; }; \
echo "  binary + config + service installed"; \
$(MAKE) ship WHAT=release ACT=install-validate
endef

define MCB_INSTALL_VALIDATE
echo "── Validating installation ──"; \
$(INSTALL_DIR)/$(BINARY_NAME) --version 2>/dev/null | grep -q mcb || { echo "  FAIL: binary not responding" >&2; exit 1; }; \
echo "  Binary: $$($(INSTALL_DIR)/$(BINARY_NAME) --version)"; \
[ -f "$(CONFIG_YAML_DIR)/development.yaml" ] && echo "  Config: $(CONFIG_YAML_DIR)/development.yaml" || echo "  WARN: no installed config"; \
R=0; while [ $$R -lt 8 ]; do systemctl --user is-active --quiet mcb.service 2>/dev/null && { echo "  Service: active"; break; }; R=$$((R+1)); [ $$R -lt 8 ] && sleep 2; done; \
[ $$R -eq 8 ] && { echo "  FAIL: service not active"; exit 1; } || true; \
H=0; while [ $$H -lt 10 ]; do curl -sf http://127.0.0.1:8080/ >/dev/null 2>&1 && { echo "  HTTP server: OK"; break; }; H=$$((H+1)); [ $$H -lt 10 ] && sleep 1; done; [ $$H -eq 10 ] && { echo "  FAIL: HTTP server not responding" >&2; exit 1; }; \
echo "  MCB v$(VERSION) installed: $(INSTALL_DIR)/$(BINARY_NAME)"
endef

# =============================================================================
# clean (APPLY-gated). WHAT=build|codegen|all
# =============================================================================
define DISPATCH_CLEAN
@case "$(WHAT)" in \
  ""|build)  $(call gate,clean build artifacts); $(MCB_RUN) cargo clean; echo "✓ build artifacts cleaned" ;; \
  codegen)   $(call gate,clean codegen artifacts); rm -f $(CODEGEN_DB); echo "✓ codegen DB removed" ;; \
  all)       $(call gate,clean all artifacts); $(MCB_RUN) cargo clean; rm -f $(CODEGEN_DB); echo "✓ all artifacts cleaned" ;; \
  *)         $(call BAD_WHAT,$(WHATS_clean)) ;; \
esac
endef
