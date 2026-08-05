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
PROJECT ?=
PROJECTS ?=
BASE ?=
BRANCH ?=
PYTEST_ARGS ?=
PYTEST_DIAG_ARGS ?= -rA --durations=0 --tb=long --showlocals
PYTEST_REPORT_ARGS ?= -ra --durations=25 --durations-min=0.001 --tb=short
PYTEST_PROCESS_TIMEOUT_SECONDS ?= 1920
# mro-99ae: the pytest process inherits a hard wall-clock boundary, mirroring
# MYPY_BOUNDED, so a hung run is terminated even if the typed runner stalls.
PYTEST_BOUNDED = timeout --signal=TERM --kill-after=5s "$(PYTEST_PROCESS_TIMEOUT_SECONDS)s"
PYTEST_REPORTS_DIR ?= .reports/tests
override PYTEST_CASE_TIMEOUT_SECONDS := 30
override PYTEST_RUN_TIMEOUT_SECONDS := 1800
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
# make work targets a member checkout when PROJECT names a workspace member and
# WORKSPACE was not overridden on the command line. PROJECT alone used to keep
# WORKSPACE at the workspace root, so finish looked up lanes in the wrong git
# primary and failed with "worktree branch is not registered".
ifeq ($(filter command line override,$(origin WORKSPACE)),)
ifneq ($(strip $(PROJECT)),)
ifneq ($(filter $(PROJECT),$(WORKSPACE_MEMBERS)),)
override WORKSPACE := $(PROJECT_ROOT)/$(PROJECT)
endif
endif
endif
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

# === SECTION: verb dispatch (managed) ===
# Source: config:make.verbs[*].whats, config:make.check_gates_allowed,
#        config:make.check_gates_default, config:make.serialization.verbs
PUBLIC_VERBS := help setup deps build check test fmt fix run status docs clean release gen work
BUILTIN_VERBS := help setup deps build check test fmt fix run status docs clean release gen work
SCRIPT_VERBS :=

_ALLOWED_WHATS_help := usage $(shell sed -n 's/^_custom_help_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_setup := environment $(shell sed -n 's/^_custom_setup_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_deps := check lock upgrade $(shell sed -n 's/^_custom_deps_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_build := artifacts $(shell sed -n 's/^_custom_build_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_check := all $(shell sed -n 's/^_custom_check_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_test := all cache-status cache-clear cache-checkpoint $(shell sed -n 's/^_custom_test_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_fmt := check all apply $(shell sed -n 's/^_custom_fmt_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_fix := check all apply $(shell sed -n 's/^_custom_fix_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_run := default $(shell sed -n 's/^_custom_run_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_status := diagnostics $(shell sed -n 's/^_custom_status_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_docs := all generate fix audit build validate $(shell sed -n 's/^_custom_docs_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_clean := status generated $(shell sed -n 's/^_custom_clean_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_release := status $(shell sed -n 's/^_custom_release_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_gen := check all apply $(shell sed -n 's/^_custom_gen_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_work := start status land finish $(shell sed -n 's/^_custom_work_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')

CHECK_GATES_ALLOWED := lint format pyrefly mypy pyright security markdown smells
CHECK_GATES_DEFAULT := lint pyrefly mypy pyright security markdown smells
DOCS_ACTIONS := generate fix audit build validate
SERIALIZED_VERBS := check test gen fmt fix deps clean work docs
SERIALIZED_TARGETS := _serialized_check _serialized_test _serialized_gen _serialized_fmt _serialized_fix _serialized_deps _serialized_clean _serialized_work _serialized_docs
# End SECTION: verb dispatch

# === SECTION: lint/type paths (managed) ===
# Source: template + computed (script_dispatch conditional)
RUFF_PATHS := $(PROJECT_ROOT)/src $(PROJECT_ROOT)/tests
MYPY_PATHS := $(PROJECT_ROOT)/src $(PROJECT_ROOT)/tests
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

# === MYPY RESOURCE LIMIT ===
# mro-0ftd.3.11: every Mypy process inherits validated memory and time caps.
MYPY_MEMORY_LIMIT_MB ?= 6144
MYPY_TIMEOUT_SECONDS ?= 600
MYPY_BOUNDED = timeout --signal=TERM --kill-after=5s "$(MYPY_TIMEOUT_SECONDS)s" prlimit --as=$$(( $(MYPY_MEMORY_LIMIT_MB) * 1024 * 1024 )):$$(( $(MYPY_MEMORY_LIMIT_MB) * 1024 * 1024 )) --
VALIDATE_MYPY_LIMITS = case "$(MYPY_MEMORY_LIMIT_MB)" in ""|*[!0-9]*) echo "ERROR: MYPY_MEMORY_LIMIT_MB must be a positive integer"; exit 2;; esac; [ "$(MYPY_MEMORY_LIMIT_MB)" -gt 0 ] || { echo "ERROR: MYPY_MEMORY_LIMIT_MB must be greater than zero"; exit 2; }; [ "$(MYPY_MEMORY_LIMIT_MB)" -le 6144 ] || { echo "ERROR: MYPY_MEMORY_LIMIT_MB must be less than or equal to 6144"; exit 2; }; case "$(MYPY_TIMEOUT_SECONDS)" in ""|*[!0-9]*) echo "ERROR: MYPY_TIMEOUT_SECONDS must be a positive integer"; exit 2;; esac; [ "$(MYPY_TIMEOUT_SECONDS)" -gt 0 ] || { echo "ERROR: MYPY_TIMEOUT_SECONDS must be greater than zero"; exit 2; }; [ "$(MYPY_TIMEOUT_SECONDS)" -le 600 ] || { echo "ERROR: MYPY_TIMEOUT_SECONDS must be less than or equal to 600"; exit 2; }; command -v timeout >/dev/null 2>&1 || { echo "ERROR: required executable not found: timeout"; exit 2; }; command -v prlimit >/dev/null 2>&1 || { echo "ERROR: required executable not found: prlimit"; exit 2; }
REPORT_MYPY_FAILURE = code=$$?; signal=none; if [ "$$code" -ge 128 ]; then signal=$$(( $$code - 128 )); fi; if [ "$$code" -eq 124 ] || [ "$$signal" != none ]; then reason="resource limit triggered"; else reason="type check failed under enforced limits"; fi; echo "ERROR: Mypy $$reason: memory_limit=$(MYPY_MEMORY_LIMIT_MB) MiB; timeout=$(MYPY_TIMEOUT_SECONDS)s; exit=$$code; signal=$$signal" >&2
export MYPY_MEMORY_LIMIT_MB MYPY_TIMEOUT_SECONDS


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
_DEFAULT_work := status

_APPLY_WHAT_deps := upgrade
_APPLY_WHAT_test := all
_APPLY_WHAT_fmt := apply
_APPLY_WHAT_fix := apply
_APPLY_WHAT_run := default
_APPLY_WHAT_docs := generate
_APPLY_WHAT_clean := generated
_APPLY_WHAT_gen := apply
_APPLY_WHAT_work := land


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
# `make work` lane) shares the primary checkout's environment so the two never
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
UV_RUN := env -u MYPYPATH PYTHONPATH="$(PROJECT_ROOT)/src" $(UV) run --project "$(RUNTIME_ROOT)" --no-sync
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
		$(SELF_MAKE) "$$builtin" || exit $$?; \
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

.PHONY: $(PUBLIC_VERBS) $(SERIALIZED_TARGETS) _builtin_help_usage _builtin_setup_environment _builtin_deps_check _builtin_deps_lock _builtin_deps_upgrade _builtin_build_artifacts _builtin_check_all _builtin_test_all _builtin_test_cache-status _builtin_test_cache-clear _builtin_test_cache-checkpoint _builtin_fmt_check _builtin_fmt_all _builtin_fmt_apply _builtin_fix_check _builtin_fix_all _builtin_fix_apply _builtin_run_default _builtin_status_diagnostics _builtin_docs_all _builtin_docs_generate _builtin_docs_fix _builtin_docs_audit _builtin_docs_build _builtin_docs_validate _builtin_clean_status _builtin_clean_generated _builtin_release_status _builtin_gen_check _builtin_gen_all _builtin_gen_apply _builtin_work_start _builtin_work_status _builtin_work_land _builtin_work_finish

$(filter-out setup $(SERIALIZED_VERBS),$(PUBLIC_VERBS)):
	$(call _dispatch,$@)


check: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) workspace serialize-make --workspace "$(PROJECT_ROOT)" --makefile "$(SELF_MAKEFILE)" --verb "check" --selector-value "$(WHAT)" --apply-token "$(APPLY)"

_serialized_check:
	$(call _dispatch,check)


test: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) workspace serialize-make --workspace "$(PROJECT_ROOT)" --makefile "$(SELF_MAKEFILE)" --verb "test" --selector-value "$(WHAT)" --apply-token "$(APPLY)"

_serialized_test:
	$(call _dispatch,test)


gen: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) workspace serialize-make --workspace "$(PROJECT_ROOT)" --makefile "$(SELF_MAKEFILE)" --verb "gen" --selector-value "$(WHAT)" --apply-token "$(APPLY)"

_serialized_gen:
	$(call _dispatch,gen)


fmt: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) workspace serialize-make --workspace "$(PROJECT_ROOT)" --makefile "$(SELF_MAKEFILE)" --verb "fmt" --selector-value "$(WHAT)" --apply-token "$(APPLY)"

_serialized_fmt:
	$(call _dispatch,fmt)


fix: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) workspace serialize-make --workspace "$(PROJECT_ROOT)" --makefile "$(SELF_MAKEFILE)" --verb "fix" --selector-value "$(WHAT)" --apply-token "$(APPLY)"

_serialized_fix:
	$(call _dispatch,fix)


deps: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) workspace serialize-make --workspace "$(PROJECT_ROOT)" --makefile "$(SELF_MAKEFILE)" --verb "deps" --selector-value "$(WHAT)" --apply-token "$(APPLY)"

_serialized_deps:
	$(call _dispatch,deps)


clean: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) workspace serialize-make --workspace "$(PROJECT_ROOT)" --makefile "$(SELF_MAKEFILE)" --verb "clean" --selector-value "$(WHAT)" --apply-token "$(APPLY)"

_serialized_clean:
	$(call _dispatch,clean)


work: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) workspace serialize-make --workspace "$(PROJECT_ROOT)" --makefile "$(SELF_MAKEFILE)" --verb "work" --selector-value "$(WHAT)" --apply-token "$(APPLY)"

_serialized_work:
	$(call _dispatch,work)


docs: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) workspace serialize-make --workspace "$(PROJECT_ROOT)" --makefile "$(SELF_MAKEFILE)" --verb "docs" --selector-value "$(WHAT)" --apply-token "$(APPLY)"

_serialized_docs:
	$(call _dispatch,docs)



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
		role=$$(git -C "$(PROJECT_ROOT)" config --local --get beads.role 2>/dev/null || true); \
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




	@printf '  %-10s WHAT=%s\n' 'work' "$$(printf '%s' '$(_ALLOWED_WHATS_work)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";
	@printf '  %-10s %s\n' '' 'status is read-only; other WHATs require APPLY=Y';


	@printf '  %-10s %s\n' 'WORKSPACE' 'target repository (default: current project)';
	@printf '  %-10s %s\n' 'PROJECT' 'member checkout for work when WORKSPACE unset';
	@printf '  %-10s %s\n' 'BEAD' 'lane-root bead id for work start/land/finish';
	@printf '  %-10s %s\n' 'KIND/NAME' 'GitFlow kind and slug for work start';
	@printf '  %-10s %s\n' 'BASE' 'optional integration base override for work start';
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
#       Fetch skips when local already contains pin and origin tip.
# Free: no
# End SECTION: submodule setup
_builtin_setup_submodules:
	@set -eu; \
	root="$(PROJECT_ROOT)"; \
	if [ ! -f "$$root/.gitmodules" ]; then exit 0; fi; \
	profile="$(MAKE_PROFILE)"; \
	if [ "$$profile" = "workspace-root" ]; then \
		managed="$(WORKSPACE_MEMBERS)"; \
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
		if [ "$$branch" = "." ]; then \
			branch=$$(git -C "$$superproject" branch --show-current); \
			if [ -z "$$branch" ]; then \
				printf 'ERROR: %s: branch = . requires a named superproject branch\n' "$$child_path" >&2; \
				exit 1; \
			fi; \
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
		remote_ref="refs/remotes/origin/$$branch"; \
		current=$$(git -C "$$child_root" branch --show-current); \
		head=$$(git -C "$$child_root" rev-parse HEAD); \
		if [ -n "$$current" ] && [ "$$current" != "$$branch" ]; then \
			printf 'ERROR: %s: conflicting branch %s; expected %s (setup never runs checkout/reset; switch it yourself while keeping dirty)\n' "$$child_path" "$$current" "$$branch" >&2; \
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
			git -C "$$child_root" fetch --quiet origin "$$branch" || { \
				printf 'ERROR: %s: fetch origin %s failed\n' "$$child_path" "$$branch" >&2; \
				exit 1; \
			}; \
		fi; \
		current=$$(git -C "$$child_root" branch --show-current); \
		head=$$(git -C "$$child_root" rev-parse HEAD); \
		if [ -n "$$current" ] && [ "$$current" != "$$branch" ]; then \
			printf 'ERROR: %s: conflicting branch %s; expected %s (setup never runs checkout/reset; switch it yourself while keeping dirty)\n' "$$child_path" "$$current" "$$branch" >&2; \
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
		--apply --rewrite-constraints --skip-check "$$@"
	$(call _run_for_selected_projects,)


_builtin_build_artifacts:
	@$(UV) build --project "$(PROJECT_ROOT)"

# `check` is read-only by contract: it never mutates the tree. Fixing is owned
# by `make fix APPLY=Y` and formatting by `make fmt APPLY=Y`, both run BEFORE
# check. APPLY here made the same tools run twice with conflicting intents,
# so it is rejected instead of silently honoured; FIX=1 became the `fix` verb.
# CI=Y omits make.ci.check_gates_skip (ruff + pyrefly).
_builtin_check_all: _builtin_require_environment
	@set -eu; \
	gates="$(strip $(CHECK_GATES))"; \
	if [ -z "$$gates" ]; then gates="$$(printf '%s' '$(CHECK_GATES_DEFAULT)' | tr ' ' ',')"; fi; \
	gates="$$(printf '%s' "$$gates" | tr -d '[:space:]')"; \
	if [ "$(strip $(CI))" = "Y" ]; then \
		filtered=""; \
		for gate in $$(printf '%s' "$$gates" | tr ',' ' '); do \
			skip=0; \
			if [ "$$gate" = "lint" ]; then skip=1; fi; \
			if [ "$$gate" = "format" ]; then skip=1; fi; \
			if [ "$$gate" = "pyrefly" ]; then skip=1; fi; \
			if [ "$$skip" -eq 0 ]; then \
				if [ -n "$$filtered" ]; then filtered="$$filtered,$$gate"; else filtered="$$gate"; fi; \
			fi; \
		done; \
		gates="$$filtered"; \
		printf 'INFO: CI=Y omits check gates: lint format pyrefly\n'; \
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

# Read-only fixed-point after `make fix APPLY=Y` (serialize-make strips APPLY and
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

# Every command here writes to the SAME root, derived from the invocation
# point. `deps modernize`/`extra-paths` used to receive WORKSPACE_ROOT while
# `conform` received PROJECT_ROOT, so a gen run inside one member rewrote the
# pyproject of ~30 siblings and left each dirty. Because gen runs inside check
# and check runs in the pre-commit hook, one commit in any lane dirtied every
# sibling -- the "workspace changed during serialized Make check" abort. It
# also kept the fixed point out of reach: each run rewrote the siblings, so
# the next run found a difference again. At the workspace root PROJECT_ROOT is
# already the workspace, so fan-out survives exactly where it belongs.
_builtin_gen_check: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode check
	@$(PROJECT_FLEXT_INFRA) deps modernize --workspace "$(PROJECT_ROOT)" --check
	@$(PROJECT_FLEXT_INFRA) deps extra-paths --workspace "$(PROJECT_ROOT)" --check

_builtin_gen_all: _builtin_require_environment
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode apply
	@$(PROJECT_FLEXT_INFRA) deps modernize --workspace "$(PROJECT_ROOT)" --apply
	@$(PROJECT_FLEXT_INFRA) deps extra-paths --workspace "$(PROJECT_ROOT)" --apply

_builtin_gen_apply: _builtin_gen_all

_builtin_work_status:
	@$(PROJECT_FLEXT_INFRA) workspace work --workspace "$(WORKSPACE)" --operation status --bead "$(BEAD)" --branch "$(BRANCH)"

_builtin_work_start:
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) workspace work --workspace "$(WORKSPACE)" --operation start --bead "$(BEAD)" --kind "$(KIND)" --name "$(NAME)" --base "$(BASE)" --apply

_builtin_work_land:
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) workspace work --workspace "$(WORKSPACE)" --operation land --bead "$(BEAD)" --apply

_builtin_work_finish:
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) workspace work --workspace "$(WORKSPACE)" --operation finish --bead "$(BEAD)" --apply
