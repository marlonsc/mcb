# @flext-managed: continuous
# @flext-regenerate: make gen WHAT=apply APPLY=Y
# @flext-ssot: flext-infra/config/codegen.yaml + flext-infra/src/flext_infra/templates/project/base/Makefile.j2
# @flext-maintenance: do not edit generated projections; edit the SSOT and regenerate
# mcb-scripts — generated project interface.
# Managed by flext-infra codegen conform for new and existing repositories.
# === SECTION: header (managed) ===
# Source: template (base/Makefile.j2)
# Free: no
# End SECTION: header

SHELL := /bin/sh
.DEFAULT_GOAL := help

# === SECTION: project identity (managed) ===
# Source: config:dist / config:make_profile / config:workspace_root_rel / config:uv_link_mode
PROJECT_NAME := mcb-scripts
MAKE_PROFILE := standalone
WORKSPACE_ROOT_REL := .
# === SECTION: workspace members (managed) ===
# Source: config:workspace_members (list), config:workspace_repositories (list)
# Computed: MANAGED_GITLINKS mirrors WORKSPACE_MEMBERS for workspace-root gitlink
# governance; standalone projects discover managed submodules at runtime from
# .gitmodules (flext-managed=true).
WORKSPACE_MEMBERS :=
MANAGED_GITLINKS :=
WORKSPACE_EDITABLES := $(PROJECT_NAME):.
UV_LINK_MODE := copy
# End SECTION: project identity

# === SECTION: user overrides (managed) ===
# Source: template (canonical public knobs documented by base.mk)
# Free: no — values are caller-supplied each invocation, not preserved in the file.
APPLY ?= N
# The seeded absent value means "not applying", so every guard compares against
# APPLYING and a plain read-only run never trips the write-enable check.
APPLYING := $(if $(filter-out N,$(strip $(APPLY))),$(strip $(APPLY)))
ARGS ?=
CHECK_GATES ?=
DEPENDENCY ?=
FAIL_FAST ?= 0
FILE ?=
MATCH ?=
COV ?=
PROJECT ?=
PROJECTS ?=
BASE ?=
BRANCH ?=
PYTEST_ARGS ?=
PYTEST_DIAG_ARGS ?= -rA --durations=0 --tb=long --showlocals
PYTEST_REPORT_ARGS ?= -ra --durations=25 --durations-min=0.001 --tb=short
PYTEST_PROCESS_TIMEOUT_SECONDS ?= 660
# mro-99ae: the pytest process inherits a hard wall-clock boundary, mirroring
# MYPY_BOUNDED, so a hung run is terminated even if the typed runner stalls.
PYTEST_BOUNDED = timeout --signal=TERM --kill-after=5s "$(PYTEST_PROCESS_TIMEOUT_SECONDS)s"
PYTEST_REPORTS_DIR ?= .reports/tests
override PYTEST_CASE_TIMEOUT_SECONDS := 10
override PYTEST_RUN_TIMEOUT_SECONDS := 600
override PYTEST_TERMINATION_GRACE_SECONDS := 2
override PYTEST_TIMEOUT_EXIT_CODE := 124
override PYTEST_ENFORCEMENT_PLUGIN := flext_tests_enforcement
override PYTEST_PROGRESS_ARGS := --verbose
override PYTEST_REPORT_ARGS := -ra --durations=25 --durations-min=0.001 --tb=short
override PYTEST_DIAG_ARGS := -rA --durations=0 --tb=long --showlocals
override PYTEST_PARALLEL_WORKERS := 4
override PYTEST_PARALLEL_DISTRIBUTION := worksteal
override PYTEST_PROFILE_SORT := cumulative
override PYTEST_PROFILE_LIMIT := 50
override PROCESS_TIMEOUT_COMMAND := timeout
override export FLEXT_PYTEST_ARGS_RAW := $(value PYTEST_ARGS)
override export FLEXT_PYTEST_FILE_RAW := $(value FILE)
override export FLEXT_PYTEST_FILES_RAW := $(value FILES)
override export FLEXT_PYTEST_MATCH_RAW := $(value MATCH)
override export FLEXT_PYTEST_DIAG_RAW := $(value DIAG)
override export FLEXT_PYTEST_FAIL_FAST_RAW := $(value FAIL_FAST)
override export FLEXT_PYTEST_REPORTS_RAW := $(value PYTEST_REPORTS_DIR)
override export FLEXT_PYTEST_WHAT_RAW := $(value WHAT)
override export FLEXT_PYTEST_VERBOSE_RAW := $(value VERBOSE)
override export FLEXT_PYTEST_COV_RAW := $(value COV)
WHAT ?=
# End SECTION: user overrides

# === SECTION: derived paths (managed) ===
# Source: computed (git rev-parse, MAKEFILE_LIST, abspath)
# Rule: PROJECT_ROOT is the checkout that OWNS this Makefile, never the caller's
# CWD. Deriving it from `pwd -P` made a member validate whatever tree the
# caller happened to stand in: `make -f <member>/Makefile` invoked from the
# superproject resolved RUFF_PATHS to the SUPERPROJECT's src/tests, so the
# member linted files it does not even contain. With many shared worktrees that
# silently validates the wrong tree.
SELF_MAKEFILE := $(abspath $(firstword $(MAKEFILE_LIST)))
MAKEFILE_ROOT := $(patsubst %/,%,$(dir $(SELF_MAKEFILE)))
PROJECT_ROOT := $(MAKEFILE_ROOT)
override export FLEXT_PYTEST_TARGET_RAW := tests
WORKSPACE ?= $(PROJECT_ROOT)
# The member-selection block used to appear TWICE: once here against
# PROJECT_ROOT and again below against WORKSPACE_ROOT. Both guarded on the same
# `origin WORKSPACE` condition, so the second `override` always won and the
# first was dead -- while still contributing its `endif`s, which is how the
# generated Makefile ended up with one more `endif` than it had conditionals
# ("extraneous 'endif'"). The surviving block is the correct one:
# WORKSPACE_ROOT is derived from the superproject below, whereas PROJECT_ROOT
# is this checkout, so only the former resolves a member of the governing
# workspace.
# === SECTION: WORKSPACE_ROOT isolation (managed) ===
# Source: computed (rule: derive from current checkout unless caller overrides)
# Rule: WORKSPACE_ROOT is always derived from the current checkout unless the
# caller passed it on the command line or via an override origin. An inherited
# environment WORKSPACE_ROOT (e.g. a leaked .envrc export from a foreign checkout)
# must never redirect verbs to another working tree. The git queries therefore
# run inside MAKEFILE_ROOT: run from a foreign CWD they would report THAT
# checkout's topology and redirect the verb to the wrong tree.
ifeq ($(filter command line override,$(origin WORKSPACE_ROOT)),)
WORKSPACE_ROOT := $(shell cd "$(MAKEFILE_ROOT)" && root=$$(git rev-parse --show-superproject-working-tree 2>/dev/null); if [ -n "$$root" ]; then printf '%s\n' "$$root"; else git rev-parse --show-toplevel 2>/dev/null || printf '%s\n' "$(MAKEFILE_ROOT)"; fi)
endif
# End SECTION: WORKSPACE_ROOT isolation
# A workspace lane is always registered at the workspace root. Other verbs may
# select a member through PROJECT while workspace orchestration keeps the root
# so one Git worktree owns the complete project matrix.
ifeq ($(filter command line override,$(origin WORKSPACE)),)
ifneq ($(strip $(PROJECT)),)
ifneq ($(filter $(PROJECT),$(WORKSPACE_MEMBERS)),)
override WORKSPACE := $(WORKSPACE_ROOT)/$(PROJECT)
endif
endif
endif

# === SECTION: verb dispatch (managed) ===
# Source: config:make.verbs[*].whats, config:make.check_gates_allowed,
#        config:make.check_gates_default
PUBLIC_VERBS := help setup deps build check test fmt fix run status docs clean release gen mod
BUILTIN_VERBS := help setup deps build check test fmt fix run status docs clean release gen mod
SCRIPT_VERBS :=

_ALLOWED_WHATS_help := usage $(shell sed -n 's/^_custom_help_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_setup := environment $(shell sed -n 's/^_custom_setup_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_deps := check lock upgrade $(shell sed -n 's/^_custom_deps_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_build := artifacts $(shell sed -n 's/^_custom_build_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_check := all lint pyrefly mypy pyright security markdown smells $(shell sed -n 's/^_custom_check_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_test := all cache-status cache-clear cache-checkpoint $(shell sed -n 's/^_custom_test_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_fmt := check all apply $(shell sed -n 's/^_custom_fmt_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_fix := check all apply $(shell sed -n 's/^_custom_fix_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_run := default $(shell sed -n 's/^_custom_run_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_status := diagnostics $(shell sed -n 's/^_custom_status_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_docs := all generate fix audit build validate $(shell sed -n 's/^_custom_docs_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_clean := status generated $(shell sed -n 's/^_custom_clean_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_release := status $(shell sed -n 's/^_custom_release_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_gen := check all apply $(shell sed -n 's/^_custom_gen_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_mod := check all apply $(shell sed -n 's/^_custom_mod_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')

CHECK_GATES_ALLOWED := lint pyrefly mypy pyright security markdown smells
CHECK_GATES_DEFAULT := lint pyrefly mypy pyright security markdown smells
 DOCS_ACTIONS := generate fix audit build validate
 # End SECTION: verb dispatch

# === SECTION: lint/type paths (managed) ===
# Source: template + computed (script_dispatch conditional)
RUFF_PATHS := $(PROJECT_ROOT)/src $(PROJECT_ROOT)/tests $(PROJECT_ROOT)/scripts
MYPY_PATHS := $(PROJECT_ROOT)/src $(PROJECT_ROOT)/tests $(PROJECT_ROOT)/scripts
# End SECTION: lint/type paths

# === SECTION: infra bootstrap (managed) ===
# Source: config:infra_repository.*, config:infra_source_root_rel, template (UV default)
UV ?= uv
UV_REQUESTED := $(UV)
CALLER_PATH := $(PATH)
CALLER_VIRTUAL_ENV := $(patsubst %/,%,$(VIRTUAL_ENV))
# Prefer the recorded flext-infra gitlink OID (immutable) when the workspace
# root can resolve it; otherwise fall back to the provider integration branch.
FLEXT_INFRA_BOOTSTRAP_REF := $(shell git -C "$(WORKSPACE_ROOT)" rev-parse "HEAD:flext-infra" 2>/dev/null)
ifeq ($(strip $(FLEXT_INFRA_BOOTSTRAP_REF)),)
FLEXT_INFRA_BOOTSTRAP_REF := 0.12.0-dev
endif
FLEXT_INFRA_BOOTSTRAP_REQUIREMENT := flext-infra @ git+https://github.com/flext-sh/flext-infra.git@$(FLEXT_INFRA_BOOTSTRAP_REF)
FLEXT_INFRA_SOURCE_ROOT_REL := 
UV_BOOTSTRAP_FLAGS := --isolated --all-groups --all-extras
# End SECTION: infra bootstrap


_DEFAULT_help := usage
_DEFAULT_deps := check
_DEFAULT_build := artifacts
_DEFAULT_check := all
_DEFAULT_test := all
_DEFAULT_fmt := check
_DEFAULT_fix := check
_DEFAULT_run := default
_DEFAULT_status := diagnostics
_DEFAULT_docs := validate
_DEFAULT_clean := status
_DEFAULT_release := status
_DEFAULT_gen := check
_DEFAULT_mod := check

_APPLY_WHAT_deps := upgrade
_APPLY_WHAT_test := all
_APPLY_WHAT_fmt := apply
_APPLY_WHAT_fix := apply
_APPLY_WHAT_run := default
_APPLY_WHAT_docs := generate
_APPLY_WHAT_clean := generated
_APPLY_WHAT_gen := apply
_APPLY_WHAT_mod := apply


# === SECTION: profile routing (managed) ===
# Source: config:workspace manifest (role), computed (WORKSPACE_ROOT)
# Rule: workspace-member delegates runtime to the principal (RUNTIME_ROOT is
# the governing workspace root); workspace-root and standalone own their
# runtime locally. An attached member is never promoted to a local runtime.
ifneq ($(filter $(MAKE_PROFILE),workspace-root workspace-member standalone),$(MAKE_PROFILE))
$(error Invalid MAKE_PROFILE '$(MAKE_PROFILE)')
endif

ifeq ($(MAKE_PROFILE),workspace-member)
RUNTIME_ROOT := $(WORKSPACE_ROOT)
else
RUNTIME_ROOT := $(PROJECT_ROOT)
endif
# End SECTION: profile routing

RUNTIME_VENV := $(RUNTIME_ROOT)/.venv
PROJECT_VENV := $(PROJECT_ROOT)/.venv
FLEXT_INFRA_RUNTIME_ROOT := $(if $(filter $(MAKEFILE_ROOT),$(PROJECT_ROOT)),$(RUNTIME_ROOT),$(MAKEFILE_ROOT))
ifeq ($(OS),Windows_NT)
RUNTIME_BIN := $(RUNTIME_VENV)/Scripts
RUNTIME_PYTHON := $(RUNTIME_BIN)/python.exe
FLEXT_INFRA_RUNTIME_PYTHON := $(FLEXT_INFRA_RUNTIME_ROOT)/.venv/Scripts/python.exe
NORMALIZED_CALLER_PATH := $(shell cygpath --path "$(CALLER_PATH)" 2>/dev/null)
NORMALIZED_CALLER_VIRTUAL_ENV := $(shell cygpath --unix "$(CALLER_VIRTUAL_ENV)" 2>/dev/null)
CALLER_VIRTUAL_ENV_BIN := $(NORMALIZED_CALLER_VIRTUAL_ENV)/Scripts
else
RUNTIME_BIN := $(RUNTIME_VENV)/bin
RUNTIME_PYTHON := $(RUNTIME_BIN)/python
FLEXT_INFRA_RUNTIME_PYTHON := $(FLEXT_INFRA_RUNTIME_ROOT)/.venv/bin/python
NORMALIZED_CALLER_PATH := $(CALLER_PATH)
NORMALIZED_CALLER_VIRTUAL_ENV := $(CALLER_VIRTUAL_ENV)
CALLER_VIRTUAL_ENV_BIN := $(NORMALIZED_CALLER_VIRTUAL_ENV)/bin
endif
SANITIZED_CALLER_PATH := $(NORMALIZED_CALLER_PATH)
ifneq ($(strip $(NORMALIZED_CALLER_VIRTUAL_ENV)),)
SANITIZED_CALLER_PATH := $(subst $(CALLER_VIRTUAL_ENV_BIN):,,$(SANITIZED_CALLER_PATH))
SANITIZED_CALLER_PATH := $(subst :$(CALLER_VIRTUAL_ENV_BIN),,$(SANITIZED_CALLER_PATH))
ifeq ($(SANITIZED_CALLER_PATH),$(CALLER_VIRTUAL_ENV_BIN))
SANITIZED_CALLER_PATH :=
endif
endif
RESOLVED_UV := $(shell PATH="$(SANITIZED_CALLER_PATH)" command -v "$(UV_REQUESTED)" 2>/dev/null)
ifeq ($(strip $(RESOLVED_UV)),)
$(error Required uv executable not found: $(UV_REQUESTED))
endif
override UV := $(RESOLVED_UV)
override FLEXT_INFRA_PYTHON := $(FLEXT_INFRA_RUNTIME_PYTHON)
override UV_PROJECT := $(RUNTIME_ROOT)
override UV_PROJECT_ENVIRONMENT := $(RUNTIME_VENV)
override VIRTUAL_ENV := $(RUNTIME_VENV)
override PATH := $(RUNTIME_BIN):$(SANITIZED_CALLER_PATH)
export FLEXT_INFRA_PYTHON UV UV_PROJECT UV_PROJECT_ENVIRONMENT VIRTUAL_ENV PATH

ifneq ($(strip $(FLEXT_INFRA_SOURCE_ROOT_REL)),)
FLEXT_INFRA_SOURCE_ROOT := $(abspath $(PROJECT_ROOT)/$(FLEXT_INFRA_SOURCE_ROOT_REL))
FLEXT_INFRA_BOOTSTRAP := env -u PYTHONPATH -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u UV_PROJECT_ENVIRONMENT PATH="$(SANITIZED_CALLER_PATH)" $(UV) run --project "$(PROJECT_ROOT)" $(UV_BOOTSTRAP_FLAGS) --with-editable "$(FLEXT_INFRA_SOURCE_ROOT)" python -m flext_infra
else
FLEXT_INFRA_SOURCE_ROOT :=
FLEXT_INFRA_BOOTSTRAP := env -u PYTHONPATH -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u UV_PROJECT_ENVIRONMENT PATH="$(SANITIZED_CALLER_PATH)" $(UV) run --project "$(PROJECT_ROOT)" $(UV_BOOTSTRAP_FLAGS) --with "$(FLEXT_INFRA_BOOTSTRAP_REQUIREMENT)" python -m flext_infra
endif

ifeq ($(MAKE_PROFILE),workspace-root)
CODEGEN_SCOPE := all
ALLOWED_PROJECTS := . $(WORKSPACE_MEMBERS)
else
CODEGEN_SCOPE := self
ALLOWED_PROJECTS := .
endif

# Workspace-root gate verbs fan out across declared members through the generic
# `flext-infra workspace orchestrate` primitive (verb allowlist + CLI group come
# from the constants SSOT, never hardcoded here). Members and standalone projects
# run the gate locally. FAIL_FAST forwards the stop-on-first-failure policy.
# Provisioning is a probe-then-repair pair, declared once and shared by every
# profile so the two branches below can never drift apart. `uv sync --check`
# reports drift without touching the tree; only a non-zero exit escalates to a
# real `uv sync`. Creating a missing venv is provisioning, so it is allowed;
# clearing a present one is destruction, so it never happens.
# A symlinked RUNTIME_VENV is a BORROWED environment: a linked worktree (a
# lane checkout) shares the primary checkout's environment so the two never
# diverge. Syncing it would rewrite the editable pointers the owner and every
# sibling lane resolve through, so the borrower provisions nothing and the owner
# stays the only writer.
SETUP_ENVIRONMENT_RECIPE = set -eu; \
	if [ -L "$(RUNTIME_VENV)" ]; then \
		printf 'setup: borrowed environment %s is owned by another checkout\n' "$(RUNTIME_VENV)"; \
	else \
		if [ ! -x "$(RUNTIME_PYTHON)" ]; then \
			$(UV) venv "$(RUNTIME_VENV)"; \
		fi; \
		if ! $(UV) sync --project "$(PROJECT_ROOT)" $(UV_SYNC_FLAGS) --link-mode "$(UV_LINK_MODE)" --check >/dev/null 2>&1; then \
			$(UV) sync --project "$(PROJECT_ROOT)" $(UV_SYNC_FLAGS) --link-mode "$(UV_LINK_MODE)"; \
		fi; \
	fi

# A delegated runtime lives in another checkout, so this project has no local
# environment of its own. Generated tooling still addresses the environment by
# its project-local name (`$${workspaceFolder}/.venv`), which must never be
# rewritten into a cross-project relative hop: the link makes that name resolve.
# Linking is provisioning, so a real local environment is never replaced.
BORROW_RUNTIME_VENV_RECIPE = set -eu; \
	if [ ! -e "$(PROJECT_VENV)" ] || [ -L "$(PROJECT_VENV)" ]; then \
		ln -sfn "$(RUNTIME_VENV)" "$(PROJECT_VENV)"; \
	fi

WORKSPACE_ORCHESTRATE = $(UV_RUN) python -m flext_infra workspace orchestrate
REQUESTED_PROJECTS := $(strip $(if $(PROJECT),$(PROJECT),$(PROJECTS)))
# A workspace root owns no local gate implementation: its verbs fan out to the
# declared members. Selecting the root (PROJECT=.) would make it orchestrate
# itself forever; map `.` to WORKSPACE_MEMBERS instead of failing closed mid-CI.
DEFAULT_PROJECTS := $(WORKSPACE_MEMBERS) .

SELECTED_PROJECTS := $(if $(strip $(REQUESTED_PROJECTS)),$(REQUESTED_PROJECTS),$(DEFAULT_PROJECTS))

WORKSPACE_PROJECT_ARGS := $(foreach project,$(SELECTED_PROJECTS),--projects $(project))
WORKSPACE_CHECK_ARGS := $(if $(strip $(CHECK_GATES)),--make-arg "CHECK_GATES=$(strip $(CHECK_GATES))")
WORKSPACE_TEST_ARGS := $(if $(strip $(FLEXT_PYTEST_FILE_RAW)),--file "$${FLEXT_PYTEST_FILE_RAW}") $(if $(strip $(FLEXT_PYTEST_MATCH_RAW)),--match "$${FLEXT_PYTEST_MATCH_RAW}") $(if $(strip $(FLEXT_PYTEST_WHAT_RAW)),--what "$${FLEXT_PYTEST_WHAT_RAW}")
DOCS_PROJECT_ARGS := $(foreach project,$(REQUESTED_PROJECTS),--projects $(project))
ORCHESTRATED_VERBS := build check clean docs fmt fix scan test val

# A borrowed RUNTIME_VENV keeps the primary editable install. Clearing
# PYTHONPATH would make `make test` in a linked worktree execute that primary
# tree instead of this checkout. Prefer PROJECT_ROOT/src so the Makefile owner
# always wins over the shared editable (terminus T4 / path-purity).
UV_RUN := env -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u UV_PROJECT_ENVIRONMENT PYTHONPATH="$(PROJECT_ROOT)/src" $(UV) run --project "$(RUNTIME_ROOT)" --no-sync
PROJECT_INFRA_PYTHONPATH ?= $(MAKEFILE_ROOT)/src
PROJECT_FLEXT_INFRA := test -x "$(FLEXT_INFRA_PYTHON)" || { printf 'ERROR: FLEXT_INFRA_PYTHON must name an executable managed Python\n' >&2; exit 2; }; env -u PYTHONPATH -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u UV_PROJECT_ENVIRONMENT PATH="$(dir $(FLEXT_INFRA_PYTHON)):$(SANITIZED_CALLER_PATH)" PYTHONPATH="$(PROJECT_INFRA_PYTHONPATH)" $(FLEXT_INFRA_PYTHON) -m flext_infra
# mro-j47u (codex): scaffold dev tools live in the validated optional dev
# profile; a fresh project must create its lock before later check-mode locks.
# Keyed on the environment's OWNER, not on the caller's profile. A member has
# no local venv -- RUNTIME_VENV is RUNTIME_ROOT/.venv -- so every checkout that
# provisions a shared environment must describe the same contents. A member
# syncing without --all-packages treats the siblings already installed there as
# surplus and uninstalls them, undoing the root's provisioning and leaving
# `uv sync --check` permanently divergent. A standalone project owns its venv
# alone and has no workspace packages to include.
SHARED_RUNTIME := $(if $(filter-out $(PROJECT_ROOT),$(RUNTIME_ROOT)),1,$(if $(strip $(WORKSPACE_MEMBERS)),1,))
UV_SYNC_FLAGS := $(if $(SHARED_RUNTIME),--all-packages ,)--all-extras --all-groups

ifneq ($(strip $(PROJECT)),)
ifneq ($(strip $(PROJECTS)),)
$(error ERROR: Cannot use PROJECT and PROJECTS together)
endif
endif

# Script command framework: non-builtin verbs/WHATs route to scripts/<verb>/<what>.
# A hyphenated WHAT (make check WHAT=no-fallbacks) maps to the underscore module
# stem (scripts/check/no_fallbacks.py) so PEP 8 module names stay valid.
_SCRIPT_DISPATCH_ROOTS := scripts


-include custom.mk
SELF_MAKE := $(MAKE) --no-print-directory -f "$(SELF_MAKEFILE)"

define _dispatch
	@what="$(strip $(WHAT))"; \
	applying="$(strip $(APPLYING))"; \
	if [ -n "$$applying" ] && [ "$$applying" != "Y" ]; then \
		printf 'ERROR: APPLY must be Y when set\n' >&2; exit 2; \
	fi; \
	if [ -n "$$applying" ] && [ -z "$(_APPLY_WHAT_$(1))" ]; then \
		printf 'ERROR: verb %s is read-only and does not accept APPLY\n' "$(1)" >&2; exit 2; \
	fi; \
	if [ -z "$$what" ] && [ -n "$$applying" ] && [ -n "$(_APPLY_WHAT_$(1))" ]; then \
		what="$(_APPLY_WHAT_$(1))"; \
	fi; \
	if [ -z "$$what" ]; then what="$(_DEFAULT_$(1))"; fi; \
	case "$$what" in \
		*[!a-z0-9_-]*|'') printf 'ERROR: invalid WHAT selector %s\n' "$$what" >&2; exit 2 ;; \
	esac; \
	custom="_custom_$(1)_$$what"; \
	$(SELF_MAKE) -q "$$custom" >/dev/null 2>&1; custom_rc=$$?; \
	if [ "$$custom_rc" -eq 2 ]; then \
		case " $(_ALLOWED_WHATS_$(1)) " in \
			*" $$what "*) ;; \
			*) printf 'ERROR: unsupported %s WHAT=%s (allowed:%s)\n' "$(1)" "$$what" "$(_ALLOWED_WHATS_$(1))" >&2; exit 2 ;; \
		esac; \
	fi; \
	builtin="_builtin_$(1)_$$what"; \
	for hook in "pre-$(1)" "pre-$(1)-$$what"; do \
		$(SELF_MAKE) -q "$$hook" >/dev/null 2>&1; rc=$$?; \
		if [ "$$rc" -ne 2 ]; then $(SELF_MAKE) "$$hook" || exit $$?; fi; \
	done; \
	if [ "$$custom_rc" -ne 2 ]; then \
		$(SELF_MAKE) "$$custom" || exit $$?; \
	else \
		what_norm=$$(printf '%s' "$$what" | tr '-' '_'); \
		script=''; script_what=''; \
		for root in $(_SCRIPT_DISPATCH_ROOTS); do \
			for cand in "$$what_norm" "$$what"; do \
				for ext in py sh; do \
					if [ -z "$$script" ] && [ -f "$(PROJECT_ROOT)/$$root/$(1)/$$cand.$$ext" ]; then \
						script="$(PROJECT_ROOT)/$$root/$(1)/$$cand.$$ext"; script_what="$$cand"; \
					fi; \
				done; \
			done; \
		done; \
		case " $(BUILTIN_VERBS) " in \
		*" $(1) "*) $(SELF_MAKE) "$$builtin" || exit $$? ;; \
		*) if [ -n "$$script" ]; then \
			WHAT="$$script_what" $(UV_RUN) python "$(PROJECT_ROOT)/scripts/dispatch.py" "$(1)" || exit $$?; \
		else printf 'ERROR: declared handler script is missing for %s WHAT=%s\n' "$(1)" "$$what" >&2; exit 2; fi ;; \
		esac; \
	fi; \
	for hook in "post-$(1)-$$what" "post-$(1)"; do \
		$(SELF_MAKE) -q "$$hook" >/dev/null 2>&1; rc=$$?; \
		if [ "$$rc" -ne 2 ]; then $(SELF_MAKE) "$$hook" || exit $$?; fi; \
	done
endef

define _require_apply
	@if [ "$(APPLY)" != "Y" ]; then \
		printf 'ERROR: this action requires APPLY=Y\n' >&2; \
		exit 2; \
	fi
endef

define _run_for_selected_projects
	@set -eu; \
	selected="$(strip $(if $(PROJECT),$(PROJECT),$(PROJECTS)))"; \
	if [ -z "$$selected" ]; then selected="."; fi; \
	for project in $$selected; do \
		case " $(ALLOWED_PROJECTS) " in \
			*" $$project "*) ;; \
			*) printf 'ERROR: undeclared project %s\n' "$$project" >&2; exit 2 ;; \
		esac; \
		if [ "$$project" = "." ]; then project_root="$(PROJECT_ROOT)"; \
		else project_root="$(PROJECT_ROOT)/$$project"; fi; \
		$(UV) lock --project "$$project_root" $(1); \
	done
endef

.PHONY: $(PUBLIC_VERBS) _builtin_help_usage _builtin_setup_environment _builtin_deps_check _builtin_deps_lock _builtin_deps_upgrade _builtin_build_artifacts _builtin_check_all _builtin_test_all _builtin_test_cache-status _builtin_test_cache-clear _builtin_test_cache-checkpoint _builtin_fmt_check _builtin_fmt_all _builtin_fmt_apply _builtin_fix_check _builtin_fix_all _builtin_fix_apply _builtin_run_default _builtin_status_diagnostics _builtin_docs_all _builtin_docs_generate _builtin_docs_fix _builtin_docs_audit _builtin_docs_build _builtin_docs_validate _builtin_clean_status _builtin_clean_generated _builtin_release_status _builtin_gen_check _builtin_gen_all _builtin_gen_apply _builtin_mod_check _builtin_mod_all _builtin_mod_apply

$(filter-out setup,$(PUBLIC_VERBS)):
	$(call _dispatch,$@)
# `setup` keeps its own recipe (it must not require the environment it is about
# to build), but it still runs the pre-/post-setup lifecycle hooks so a project
# declaring them in the custom handler surface is actually honoured.
setup:
	@for hook in "pre-setup"; do \
		$(SELF_MAKE) -q "$$hook" >/dev/null 2>&1; rc=$$?; \
		if [ "$$rc" -ne 2 ]; then $(SELF_MAKE) "$$hook" || exit $$?; fi; \
	done
	@$(SELF_MAKE) _builtin_setup_environment
	@# Provision Beads local role so `bd` writes do not warn (GH#2950).
	@if git -C "$(PROJECT_ROOT)" rev-parse --is-inside-work-tree >/dev/null 2>&1; then \
		if role=$$(git -C "$(PROJECT_ROOT)" config --local --get beads.role 2>/dev/null); then :; else role=; fi; \
		if [ -z "$$role" ]; then \
			git -C "$(PROJECT_ROOT)" config --local beads.role maintainer; \
		fi; \
	fi
	@for hook in "post-setup"; do \
		$(SELF_MAKE) -q "$$hook" >/dev/null 2>&1; rc=$$?; \
		if [ "$$rc" -ne 2 ]; then $(SELF_MAKE) "$$hook" || exit $$?; fi; \
	done

_builtin_help_usage:
	@printf '%s\n' 'mcb-scripts [standalone]' '';


	@printf '  %-10s WHAT=%s\n' 'help' "$$(printf '%s' '$(_ALLOWED_WHATS_help)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s\n' 'setup';



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'deps' "$$(printf '%s' '$(_ALLOWED_WHATS_deps)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s\n' 'build' "$$(printf '%s' '$(_ALLOWED_WHATS_build)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s\n' 'check' "$$(printf '%s' '$(_ALLOWED_WHATS_check)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'test' "$$(printf '%s' '$(_ALLOWED_WHATS_test)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'fmt' "$$(printf '%s' '$(_ALLOWED_WHATS_fmt)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'fix' "$$(printf '%s' '$(_ALLOWED_WHATS_fix)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'run' "$$(printf '%s' '$(_ALLOWED_WHATS_run)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s\n' 'status' "$$(printf '%s' '$(_ALLOWED_WHATS_status)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'docs' "$$(printf '%s' '$(_ALLOWED_WHATS_docs)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'clean' "$$(printf '%s' '$(_ALLOWED_WHATS_clean)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s\n' 'release' "$$(printf '%s' '$(_ALLOWED_WHATS_release)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'gen' "$$(printf '%s' '$(_ALLOWED_WHATS_gen)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'mod' "$$(printf '%s' '$(_ALLOWED_WHATS_mod)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";


	@printf '  %-10s %s\n' 'PROJECT' 'member checkout when WORKSPACE unset';
	@printf '  %-10s %s\n' 'BEAD' 'lane-root bead id for lane tracking';
	@printf '  %-10s %s\n' 'WORKSPACE' 'target repository (default: current project)';
	@printf '\n%s\n' 'Custom hooks (custom.mk):';
	@printf '  %s\n' 'Define pre-<verb>, post-<verb>, pre-<verb>-<what>, post-<verb>-<what>';
	@printf '  %s\n' 'in custom.mk to wrap one declared handler.';
	@printf '  %s\n' 'Add _custom_<verb>_<what> to define a new WHAT.';
	@if [ -f custom.mk ]; then \
		hooks=$$(grep -oE '^(pre|post)-[a-z][a-z0-9-]*|^_custom_[a-z][a-z0-9_-]*' custom.mk 2>/dev/null | sort -u); \
		if [ -n "$$hooks" ]; then \
			printf '  %s\n' 'Defined in this project:'; \
			for hook in $$hooks; do printf '    %s\n' "$$hook"; done; \
		fi; \
	fi

# A project owns the sources declared by its manifest. The generated setup
# reconciler validates every initialized checkout before mutation, initializes
# only missing modules, and preserves declared branches that fix forward beyond
# the recorded gitlink.
.PHONY: _builtin_setup_submodules

# === SECTION: submodule setup (managed) ===
# Source: template (submodule_setup_recipe.j2)
# Computed: workspace-root uses WORKSPACE_MEMBERS from config; standalone discovers
#           submodules with flext-managed=true from .gitmodules at runtime.
# Rule: setup PROVISIONS an absent governed gitlink and VERIFIES a present one.
#       An absent checkout holds no work, so setup initializes it at the recorded
#       gitlink. A present checkout is never destroyed: git checkout and git reset
#       are forbidden. Detached HEAD is attached via branch + symbolic-ref so dirty
#       work is carried. Pin validity is HEAD contains gitlink — origin may lag the
#       pin without failing verify. Declared branch is the named integration line;
#       legacy branch=. still resolves to the superproject named branch if present.
#       A checkout is also accepted on the superproject current branch (workspace
#       lane): that lane branch then becomes the verified branch, and its fetch is
#       skipped when origin carries no counterpart. Any third branch still fails.
#       Fetch skips when local already contains pin and origin tip.
# Free: no
# End SECTION: submodule setup
_builtin_setup_submodules:
	@set -eu; \
	root="$(PROJECT_ROOT)"; \
	if [ ! -f "$$root/.gitmodules" ]; then exit 0; fi; \
	profile="$(MAKE_PROFILE)"; \
	if [ "$$profile" = "workspace-root" ]; then \
		managed="$(MANAGED_GITLINKS)"; \
	else \
		managed=""; \
		keys=$$(git -C "$$root" config -f .gitmodules --name-only --get-regexp '^submodule\..*\.flext-managed$$' || :); \
		for key in $$keys; do \
			value=$$(git -C "$$root" config -f .gitmodules --get "$$key"); \
			if [ "$$value" = "true" ]; then \
				section=$${key%.flext-managed}; \
				path=$$(git -C "$$root" config -f .gitmodules --get --default "" "$$section.path"); \
				if [ -n "$$path" ]; then \
					managed="$$managed $$path"; \
				fi; \
			fi; \
		done; \
	fi; \
	managed=$$(printf '%s' "$$managed" | tr ' ' '\n' | sort -u | tr '\n' ' '); \
	if [ -z "$$managed" ]; then exit 0; fi; \
	attach_branch_at_head() { \
		child_root="$$1"; \
		branch="$$2"; \
		git -C "$$child_root" branch --quiet -f "$$branch" HEAD || { \
			printf 'ERROR: %s: could not create branch %s at HEAD\n' "$$child_root" "$$branch" >&2; \
			exit 1; \
		}; \
		git -C "$$child_root" symbolic-ref HEAD "refs/heads/$$branch" || { \
			printf 'ERROR: %s: could not attach HEAD to %s without moving the tree\n' "$$child_root" "$$branch" >&2; \
			exit 1; \
		}; \
		git -C "$$child_root" branch --quiet --set-upstream-to "origin/$$branch" "$$branch" >/dev/null 2>&1 || :; \
	}; \
	validate_submodule() { \
		superproject="$$1"; \
		child_path="$$2"; \
		child_root="$$superproject/$$child_path"; \
		keys=$$(git -C "$$superproject" config -f .gitmodules --name-only --get-regexp '^submodule\..*\.path$$' || :); \
		section=""; \
		for key in $$keys; do \
			declared=$$(git -C "$$superproject" config -f .gitmodules --get "$$key"); \
			if [ "$$declared" = "$$child_path" ]; then \
				if [ -n "$$section" ]; then \
					printf 'ERROR: governed gitlink path is duplicated: %s\n' "$$child_path" >&2; \
					exit 2; \
				fi; \
				section=$${key%.path}; \
			fi; \
		done; \
		if [ -z "$$section" ]; then \
			printf 'ERROR: governed gitlink is absent from .gitmodules: %s\n' "$$child_path" >&2; \
			exit 2; \
		fi; \
		branch=$$(git -C "$$superproject" config -f .gitmodules --get --default "" "$$section.branch"); \
		if [ -z "$$branch" ]; then \
			printf 'ERROR: governed gitlink has no declared branch: %s\n' "$$child_path" >&2; \
			exit 2; \
		fi; \
		super_branch=$$(git -C "$$superproject" branch --show-current); \
		if [ "$$branch" = "." ]; then \
			branch="$$super_branch"; \
			if [ -z "$$branch" ]; then \
				printf 'ERROR: %s: branch = . requires a named superproject branch\n' "$$child_path" >&2; \
				exit 1; \
			fi; \
		fi; \
		declared_branch="$$branch"; \
		accepted_branches="$$declared_branch"; \
		if [ -n "$$super_branch" ] && [ "$$super_branch" != "$$declared_branch" ]; then \
			accepted_branches="$$declared_branch or $$super_branch"; \
		fi; \
		git check-ref-format --branch "$$branch" >/dev/null || { \
			printf 'ERROR: %s: invalid declared branch %s\n' "$$child_path" "$$branch" >&2; \
			exit 1; \
		}; \
		gitlink=$$(git -C "$$superproject" ls-files --stage -- "$$child_path" | awk '$$1 == "160000" {print $$2}'); \
		if [ -z "$$gitlink" ]; then \
			printf 'ERROR: governed gitlink is absent from the index: %s\n' "$$child_path" >&2; \
			exit 2; \
		fi; \
		if [ ! -e "$$child_root/.git" ]; then \
			git -C "$$superproject" submodule update --init -- "$$child_path" || { \
				printf 'ERROR: %s: could not initialize the governed gitlink\n' "$$child_path" >&2; \
				exit 1; \
			}; \
			attach_branch_at_head "$$child_root" "$$branch"; \
		fi; \
		current=$$(git -C "$$child_root" branch --show-current); \
		if [ -n "$$current" ] && [ "$$current" != "$$declared_branch" ] && \
		   [ -n "$$super_branch" ] && [ "$$current" = "$$super_branch" ]; then \
			branch="$$super_branch"; \
		fi; \
		remote_ref="refs/remotes/origin/$$branch"; \
		head=$$(git -C "$$child_root" rev-parse HEAD); \
		if [ -n "$$current" ] && [ "$$current" != "$$branch" ]; then \
			printf 'ERROR: %s: conflicting branch %s; expected %s (setup never runs checkout/reset; switch it yourself while keeping dirty)\n' "$$child_path" "$$current" "$$accepted_branches" >&2; \
			exit 1; \
		fi; \
		need_fetch=1; \
		if git -C "$$child_root" merge-base --is-ancestor "$$gitlink" HEAD; then \
			if git -C "$$child_root" rev-parse --verify "$$remote_ref" >/dev/null 2>&1; then \
				if git -C "$$child_root" merge-base --is-ancestor "$$remote_ref" HEAD; then \
					need_fetch=0; \
				fi; \
			else \
				# Pin is already present; origin tip may be absent on a shallow CI \
				# clone. Origin lag must not fail verify (setup never destroys). \
				need_fetch=0; \
			fi; \
		fi; \
		if [ "$$need_fetch" -eq 1 ]; then \
			fetch_allowed=1; \
			if [ "$$branch" != "$$declared_branch" ] && \
			   ! git -C "$$child_root" ls-remote --exit-code --heads origin "$$branch" >/dev/null 2>&1; then \
				fetch_allowed=0; \
			fi; \
			if [ "$$fetch_allowed" -eq 1 ]; then \
				git -C "$$child_root" fetch --quiet origin "$$branch" || { \
					printf 'ERROR: %s: fetch origin %s failed\n' "$$child_path" "$$branch" >&2; \
					exit 1; \
				}; \
			fi; \
		fi; \
		current=$$(git -C "$$child_root" branch --show-current); \
		head=$$(git -C "$$child_root" rev-parse HEAD); \
		if [ -n "$$current" ] && [ "$$current" != "$$branch" ]; then \
			printf 'ERROR: %s: conflicting branch %s; expected %s (setup never runs checkout/reset; switch it yourself while keeping dirty)\n' "$$child_path" "$$current" "$$accepted_branches" >&2; \
			exit 1; \
		fi; \
		if [ -z "$$current" ]; then \
			if git -C "$$child_root" merge-base --is-ancestor "$$gitlink" HEAD; then \
				attach_branch_at_head "$$child_root" "$$branch"; \
			elif git -C "$$child_root" rev-parse --verify "$$remote_ref" >/dev/null 2>&1 && \
			     git -C "$$child_root" merge-base --is-ancestor "$$head" "$$remote_ref"; then \
				attach_branch_at_head "$$child_root" "$$branch"; \
			else \
				printf 'ERROR: %s: detached HEAD %s is not on the recorded gitlink and not contained in origin/%s; reconcile it yourself (setup never discards commits)\n' "$$child_path" "$$head" "$$branch" >&2; \
				exit 1; \
			fi; \
			current="$$branch"; \
		fi; \
		if ! git -C "$$child_root" merge-base --is-ancestor "$$gitlink" HEAD; then \
			printf 'ERROR: %s: branch %s diverges from recorded gitlink %s (setup never runs checkout/reset; advance or switch it yourself while keeping dirty)\n' "$$child_path" "$$branch" "$$gitlink" >&2; \
			exit 1; \
		fi; \
		if [ -f "$$child_root/.gitmodules" ]; then \
			nested_keys=$$(git -C "$$child_root" config -f .gitmodules --name-only --get-regexp '^submodule\..*\.path$$' || :); \
			for nested_key in $$nested_keys; do \
				nested_path=$$(git -C "$$child_root" config -f .gitmodules --get "$$nested_key"); \
				validate_submodule "$$child_root" "$$nested_path"; \
				done; \
		fi; \
	}; \
	for child_path in $$managed; do \
		validate_submodule "$$root" "$$child_path"; \
	done

_builtin_require_environment:
	@if [ ! -x "$(RUNTIME_PYTHON)" ]; then \
		printf 'ERROR: missing environment interpreter %s; make setup creates it\n' "$(RUNTIME_PYTHON)" >&2; \
		exit 2; \
	fi

# === SECTION: setup environment (managed) ===
# Source: computed (MAKE_PROFILE routing) + operator contract (mro-e9j0.6 C7)
# Operator contract: setup PROVISIONS tooling only — mise, venv, dependencies.
# It never generates, conforms, or mutates project code; `make gen` (APPLY=Y)
# is the single public conformance/generation surface.
# Every verb invokes setup, so it must be cheap when the tooling already
# matches the lock and must repair it when it does not. `uv sync --check` is
# that probe: it compares the live venv against the resolved lock and exits
# non-zero on any drift, so it can never report a broken environment as good.
# The venv is disposable and is rebuilt whenever it is missing; it is never
# cleared while present, because a concurrent lane may be running against it.
# Profile routing: workspace-member delegates the environment to the
# principal (the uv workspace venv lives at RUNTIME_ROOT); workspace-root and
# standalone build their own environment locally.
# The delegation only means something when the principal is another checkout.
# An isolated `git worktree` of a member has no superproject, so WORKSPACE_ROOT
# falls back to the worktree itself and RUNTIME_ROOT equals PROJECT_ROOT -- while
# MAKE_PROFILE stays workspace-member, because it is fixed at generation time.
# Delegating there re-entered Make on the same target, which Make treats as
# already satisfied: setup exited 0 having created nothing, and the next verb
# failed with "missing environment interpreter". Provision locally instead.
ifeq ($(MAKE_PROFILE),workspace-member)
_builtin_setup_environment: _builtin_setup_submodules
	@if [ "$(RUNTIME_ROOT)" = "$(PROJECT_ROOT)" ]; then \
		$(SETUP_ENVIRONMENT_RECIPE); \
	else \
		$(MAKE) -C "$(RUNTIME_ROOT)" _builtin_setup_environment; \
		$(BORROW_RUNTIME_VENV_RECIPE); \
	fi
else ifeq ($(MAKE_PROFILE),workspace-root)
_builtin_setup_environment: _builtin_setup_submodules
	@$(SETUP_ENVIRONMENT_RECIPE)
	@$(UV) pip check --python "$(RUNTIME_VENV)"
else
_builtin_setup_environment: _builtin_setup_submodules
	@$(SETUP_ENVIRONMENT_RECIPE)
endif
# End SECTION: setup environment

_builtin_deps_check: _builtin_require_environment
	$(call _run_for_selected_projects,--check)

_builtin_deps_lock:
	$(call _require_apply)
	$(call _run_for_selected_projects,)

_builtin_deps_upgrade: _builtin_require_environment
	$(call _require_apply)
	@dependency="$(strip $(DEPENDENCY))"; \
	if [ -n "$$dependency" ]; then \
		case "$$dependency" in \
			[-._]*|*[!A-Za-z0-9._-]*) \
				printf 'ERROR: DEPENDENCY must be one normalized distribution name\n' >&2; \
				exit 2 ;; \
		esac; \
	fi
	$(call _run_for_selected_projects,$(if $(strip $(DEPENDENCY)),--upgrade-package "$(strip $(DEPENDENCY))",--upgrade))
	@set -eu; \
	selected="$(strip $(SELECTED_PROJECTS))"; \
	if [ -z "$$selected" ]; then selected="."; fi; \
	set --; \
	for project in $$selected; do set -- "$$@" --projects "$$project"; done; \
	$(PROJECT_FLEXT_INFRA) deps modernize --workspace "$(PROJECT_ROOT)" \
		--apply $(if $(strip $(DEPENDENCY)),,--rewrite-constraints) --skip-check "$$@"
	$(call _run_for_selected_projects,)


_builtin_build_artifacts:
	@$(UV) build --project "$(PROJECT_ROOT)"

# `check` is read-only by contract: it never mutates the tree. Fixing is owned
# by `make fix APPLY=Y` and formatting by `make fmt APPLY=Y`, both run BEFORE
# check. APPLY here made the same tools run twice with conflicting intents,
# so it is rejected instead of silently honoured; FIX=1 became the `fix` verb.
# Under CI=Y the run is narrowed to make.ci.check_gates --
# the strict complement of make.ci.local_check_gates, derived at the config
# owner so the two contexts can never overlap nor leave a gate unowned.
_builtin_check_all: _builtin_require_environment
	@set -eu; \
	gates="$(strip $(CHECK_GATES))"; \
	if [ -z "$$gates" ]; then gates="$$(printf '%s' '$(CHECK_GATES_DEFAULT)' | tr ' ' ',')"; fi; \
	gates="$$(printf '%s' "$$gates" | tr -d '[:space:]')"; \
	if [ "$(strip $(CI))" = "Y" ]; then \
		filtered=""; \
		for gate in $$(printf '%s' "$$gates" | tr ',' ' '); do \
			owned=0; \
			if [ "$$gate" = "lint" ]; then owned=1; fi; \
			if [ "$$gate" = "pyright" ]; then owned=1; fi; \
			if [ "$$gate" = "security" ]; then owned=1; fi; \
			if [ "$$gate" = "markdown" ]; then owned=1; fi; \
			if [ "$$gate" = "smells" ]; then owned=1; fi; \
			if [ "$$owned" -eq 1 ]; then \
				if [ -n "$$filtered" ]; then filtered="$$filtered,$$gate"; else filtered="$$gate"; fi; \
			fi; \
		done; \
		gates="$$filtered"; \
		printf 'INFO: CI=Y runs check gates: lint pyright security markdown smells\n'; \
	fi; \
	for gate in $$(printf '%s' "$$gates" | tr ',' ' '); do \
		case " $(CHECK_GATES_ALLOWED) " in *" $$gate "*) ;; \
			*) printf 'ERROR: unknown CHECK_GATES value: %s (allowed: %s)\n' "$$gate" "$(CHECK_GATES_ALLOWED)" >&2; exit 2 ;; \
		esac; \
	done; \
	if [ -z "$$gates" ]; then \
		printf 'ERROR: no check gates remain after CI=Y filtering\n' >&2; \
		exit 2; \
	fi; \
	$(PROJECT_FLEXT_INFRA) check run --workspace "$(PROJECT_ROOT)" --gates "$$gates" --projects .


_builtin_check_lint: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --workspace "$(PROJECT_ROOT)" --gates "lint" --projects .

_builtin_check_pyrefly: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --workspace "$(PROJECT_ROOT)" --gates "pyrefly" --projects .

_builtin_check_mypy: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --workspace "$(PROJECT_ROOT)" --gates "mypy" --projects .

_builtin_check_pyright: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --workspace "$(PROJECT_ROOT)" --gates "pyright" --projects .

_builtin_check_security: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --workspace "$(PROJECT_ROOT)" --gates "security" --projects .

_builtin_check_markdown: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --workspace "$(PROJECT_ROOT)" --gates "markdown" --projects .

_builtin_check_smells: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --workspace "$(PROJECT_ROOT)" --gates "smells" --projects .


_builtin_test_all: _builtin_require_environment

	@$(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry


_builtin_test_cache-status: _builtin_require_environment

	@$(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry

_builtin_test_cache-clear: _builtin_require_environment

	@$(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry

_builtin_test_cache-checkpoint: _builtin_require_environment

	@$(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry


# One tool, one verb: `fmt` only formats, `check` only lints (--no-fix) and
# `fix` owns the mutating lint pass. Running ruff twice per gate was the
# duplication this split removes.
_builtin_fmt_check: _builtin_require_environment
	@$(UV_RUN) ruff format --check $(RUFF_PATHS)

_builtin_fmt_all: _builtin_require_environment
	$(call _require_apply)
	@$(UV_RUN) ruff format $(RUFF_PATHS)

_builtin_fmt_apply: _builtin_fmt_all

# Read-only fixed-point after `make fix APPLY=Y` (strips APPLY and
# re-runs default_what=check). Dual of `ruff check --fix` — never mutate here.
_builtin_fix_check: _builtin_require_environment
	@$(UV_RUN) ruff check $(RUFF_PATHS)

_builtin_fix_all: _builtin_require_environment
	$(call _require_apply)
	@$(UV_RUN) ruff check --fix $(RUFF_PATHS)

_builtin_fix_apply: _builtin_fix_all


_builtin_run_default: _builtin_require_environment
	@$(UV_RUN) $(PROJECT_NAME) $(ARGS)

# workspace-member Make files attach to the governing workspace root; report
# the member path so status diagnostics stay meaningful after profile routing.
ATTACHED_MEMBER := $(if $(filter workspace-member,$(MAKE_PROFILE)),$(PROJECT_ROOT),)
_builtin_status_diagnostics: _builtin_require_environment
	@printf 'profile=%s\nattached=%s\nproject=%s\nruntime=%s\n' \
		'$(MAKE_PROFILE)' '$(ATTACHED_MEMBER)' '$(PROJECT_ROOT)' '$(RUNTIME_ROOT)'
	@$(UV) --version
	@$(UV) lock --project "$(PROJECT_ROOT)" --check
	@if [ -x "$(RUNTIME_PYTHON)" ]; then \
		$(UV) pip check --python "$(RUNTIME_VENV)"; \
	fi
	@git -C "$(PROJECT_ROOT)" status --short

_builtin_docs_all:
	@set -eu; \
	for action in $(DOCS_ACTIONS); do \
		case "$$action" in generate|fix) mode=$(if $(filter Y,$(APPLY)),--apply,--check) ;; *) mode= ;; esac; \
		$(PROJECT_FLEXT_INFRA) docs "$$action" --workspace "$(PROJECT_ROOT)" --output-dir "$(PROJECT_ROOT)/.reports/docs" $$mode $(DOCS_PROJECT_ARGS); \
	done


_builtin_docs_generate:
	@$(PROJECT_FLEXT_INFRA) docs generate --workspace "$(PROJECT_ROOT)" --output-dir "$(PROJECT_ROOT)/.reports/docs" $(if $(filter Y,$(APPLY)),--apply,--check) $(DOCS_PROJECT_ARGS)


_builtin_docs_fix:
	@$(PROJECT_FLEXT_INFRA) docs fix --workspace "$(PROJECT_ROOT)" --output-dir "$(PROJECT_ROOT)/.reports/docs" $(if $(filter Y,$(APPLY)),--apply,--check) $(DOCS_PROJECT_ARGS)


_builtin_docs_audit:
	@$(PROJECT_FLEXT_INFRA) docs audit --workspace "$(PROJECT_ROOT)" --output-dir "$(PROJECT_ROOT)/.reports/docs" $(DOCS_PROJECT_ARGS)


_builtin_docs_build:
	@$(PROJECT_FLEXT_INFRA) docs build --workspace "$(PROJECT_ROOT)" --output-dir "$(PROJECT_ROOT)/.reports/docs" $(DOCS_PROJECT_ARGS)


_builtin_docs_validate:
	@$(PROJECT_FLEXT_INFRA) docs validate --workspace "$(PROJECT_ROOT)" --output-dir "$(PROJECT_ROOT)/.reports/docs" $(DOCS_PROJECT_ARGS)



_builtin_clean_generated:
	$(call _require_apply)
	@find "$(PROJECT_ROOT)" -type d \
		\( -name __pycache__ -o -name .mypy_cache -o -name .pytest_cache -o -name .ruff_cache \) \
		-prune -exec rm -rf {} +
	@rm -rf "$(PROJECT_ROOT)/build" "$(PROJECT_ROOT)/dist" "$(PROJECT_ROOT)/htmlcov"
	@rm -f "$(PROJECT_ROOT)/.coverage"

_builtin_release_status: _builtin_require_environment
	@$(UV) lock --project "$(PROJECT_ROOT)" --check
	@git -C "$(PROJECT_ROOT)" diff --quiet
	@git -C "$(PROJECT_ROOT)" diff --cached --quiet

# Generation has one owner. Conform preserves the caller's scope and applies
# the complete dependency/tooling projection before it verifies its fixed point.
# Dependency upgrades remain a separate explicit verb because they rewrite lock
# floors; gen must never run a second pyproject writer over conform's result.
_builtin_gen_check: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode check

_builtin_gen_all: _builtin_require_environment
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode apply

_builtin_gen_apply: _builtin_gen_all


