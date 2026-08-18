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
PYTEST_PROCESS_TIMEOUT_SECONDS ?= 180
# mro-99ae: the pytest process inherits a hard wall-clock boundary, mirroring
# MYPY_BOUNDED, so a hung run is terminated even if the typed runner stalls.
PYTEST_BOUNDED = timeout --signal=TERM --kill-after=5s "$(PYTEST_PROCESS_TIMEOUT_SECONDS)s"
PYTEST_REPORTS_DIR ?= .reports/tests
override PYTEST_CASE_TIMEOUT_SECONDS := 10
override PYTEST_RUN_TIMEOUT_SECONDS := 120
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
# A workspace lane is always registered at the workspace root. Other verbs may
# select a member through PROJECT, but `make work` keeps WORKSPACE at the root
# so one Git worktree owns the complete project matrix.
ifneq ($(filter work,$(MAKECMDGOALS)),work)
ifeq ($(filter command line override,$(origin WORKSPACE)),)
ifneq ($(strip $(PROJECT)),)
ifneq ($(filter $(PROJECT),$(WORKSPACE_MEMBERS)),)
override WORKSPACE := $(PROJECT_ROOT)/$(PROJECT)
endif
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
#        config:make.check_gates_default
PUBLIC_VERBS := help setup deps build check test fmt fix run status clean release gen work mod
BUILTIN_VERBS := help setup deps build check test fmt fix run status clean release gen work mod
SCRIPT_VERBS :=

_ALLOWED_WHATS_help := usage $(shell sed -n 's/^_custom_help_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_setup := environment $(shell sed -n 's/^_custom_setup_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_deps := check lock upgrade $(shell sed -n 's/^_custom_deps_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_build := artifacts $(shell sed -n 's/^_custom_build_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_check := all $(shell sed -n 's/^_custom_check_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_test := all full cache-status cache-clear cache-checkpoint $(shell sed -n 's/^_custom_test_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_fmt := check all apply $(shell sed -n 's/^_custom_fmt_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_fix := check all apply $(shell sed -n 's/^_custom_fix_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_run := default $(shell sed -n 's/^_custom_run_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_status := diagnostics $(shell sed -n 's/^_custom_status_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_clean := status generated $(shell sed -n 's/^_custom_clean_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_release := status rel $(shell sed -n 's/^_custom_release_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_gen := check all apply $(shell sed -n 's/^_custom_gen_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_work := start status land finish $(shell sed -n 's/^_custom_work_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')
_ALLOWED_WHATS_mod := check all apply $(shell sed -n 's/^_custom_mod_\([a-z0-9_-]*\):.*/\1/p' "$(MAKEFILE_ROOT)/custom.mk" 2>/dev/null | sort -u | tr '\n' ' ')

CHECK_GATES_ALLOWED := lint format pyrefly mypy pyright security markdown smells
CHECK_GATES_DEFAULT := lint pyrefly mypy pyright security markdown smells
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
_DEFAULT_clean := status
_DEFAULT_release := status
_DEFAULT_gen := check
_DEFAULT_work := status
_DEFAULT_mod := check

_APPLY_WHAT_deps := upgrade
_APPLY_WHAT_test := all
_APPLY_WHAT_fmt := apply
_APPLY_WHAT_fix := apply
_APPLY_WHAT_run := default
_APPLY_WHAT_clean := generated
_APPLY_WHAT_release := rel
_APPLY_WHAT_gen := apply
_APPLY_WHAT_work := land
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

# Bytecode caching is a Make-owned guarantee, not a shell-profile convention.
# `make` never sources .envrc (that needs direnv), so a policy expressed only
# there is inert for every Make-driven run. An inherited PYTHONDONTWRITEBYTECODE
# then disables the import cache and each verb pays full source recompilation
# (measured: 3341 compile() calls, 9.1s of pure recompilation per run).
# The variable is undefined rather than set to an empty value, because CPython
# treats ANY non-empty value as true and an empty one as unset; clearing it here
# keeps the caller's environment from re-disabling the cache. The prefix keeps
# __pycache__ out of the working tree while still caching.
override undefine PYTHONDONTWRITEBYTECODE
PYTHONPYCACHEPREFIX ?= $(PROJECT_ROOT)/.cache/pycache
export PYTHONPYCACHEPREFIX
unexport PYTHONDONTWRITEBYTECODE

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
# A symlinked RUNTIME_VENV points at ANOTHER checkout's environment. `uv`
# records editable installs as per-environment `.pth` files holding absolute
# paths, so imports through that environment load the owner's sources: a lane
# silently validates the owner's code instead of its own.
# Each checkout therefore owns the environment its own name resolves to. The
# link is replaced (removing a link destroys no environment); a real local
# environment is never cleared, because a concurrent process may be using it.
# FLEXT=<worktree> rebinds this project's flext packages onto that checkout for
# the session. Pinned dependencies stay the default: the flag is opt-in, the
# consumer's pyproject is never modified, and setup without FLEXT= restores the
# pinned resolution. Which distributions are rebound comes from the worktree's
# own manifest, so this never carries a hardcoded package list.
FLEXT_BINDING_RECIPE = if [ -n "$(strip $(FLEXT))" ]; then \
		$(FLEXT_INFRA_RUNTIME_PYTHON) -m flext_infra workspace flext-binding \
			--workspace "$(PROJECT_ROOT)" --flext-root "$(strip $(FLEXT))" \
			--python "$(RUNTIME_PYTHON)"; \
	fi

SETUP_ENVIRONMENT_RECIPE = set -eu; \
	if [ -L "$(RUNTIME_VENV)" ]; then \
		rm -f "$(RUNTIME_VENV)"; \
	fi; \
	if [ ! -x "$(RUNTIME_PYTHON)" ]; then \
		$(UV) venv "$(RUNTIME_VENV)"; \
	fi; \
	if ! $(UV) sync --project "$(PROJECT_ROOT)" $(UV_SYNC_FLAGS) --link-mode "$(UV_LINK_MODE)" --check >/dev/null 2>&1; then \
		$(UV) sync --project "$(PROJECT_ROOT)" $(UV_SYNC_FLAGS) --link-mode "$(UV_LINK_MODE)"; \
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
ORCHESTRATED_VERBS := build check clean fmt fix scan test val

UV_RUN := env -u MYPYPATH PYTHONPATH="$(PROJECT_ROOT)/src" $(UV) run --project "$(RUNTIME_ROOT)" --no-sync
PROJECT_INFRA_PYTHONPATH ?= $(MAKEFILE_ROOT)/src
PROJECT_FLEXT_INFRA := test -x "$(FLEXT_INFRA_PYTHON)" || { printf 'ERROR: FLEXT_INFRA_PYTHON must name an executable managed Python\n' >&2; exit 2; }; env -u PYTHONPATH -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u UV_PROJECT_ENVIRONMENT PATH="$(dir $(FLEXT_INFRA_PYTHON)):$(SANITIZED_CALLER_PATH)" PYTHONPATH="$(PROJECT_INFRA_PYTHONPATH)" $(FLEXT_INFRA_PYTHON) -m flext_infra
# mro-j47u (codex): scaffold dev tools live in the validated optional dev
# profile; a fresh project must create its lock before later check-mode locks.
# Workspace roots sync every declared package into their local runtime. A
# standalone project has no workspace packages to include.
WORKSPACE_SYNC := $(if $(strip $(WORKSPACE_MEMBERS)),1,)
UV_SYNC_FLAGS := $(if $(WORKSPACE_SYNC),--all-packages ,)--all-extras --all-groups

ifneq ($(strip $(PROJECT)),)
ifneq ($(strip $(PROJECTS)),)
$(error ERROR: Cannot use PROJECT and PROJECTS together)
endif
endif

# Script command framework: non-builtin verbs/WHATs route to scripts/<verb>/<what>.
# A hyphenated WHAT (make check WHAT=no-fallbacks) maps to the underscore module
# stem (scripts/check/no_fallbacks.py) so PEP 8 module names stay valid.
_SCRIPT_DISPATCH_ROOTS := scripts


# mro-ga9q (custom.mk blacklist): member projects may define ANY custom
# verb/WHAT through _custom_<verb>_<what> handlers and (pre|post)-<verb>[-<what>]
# hooks EXCEPT the reserved verbs/WHATs below, which stay a flext-infra
# monopoly. Parse-time guard: every make invocation fails loud when custom.mk
# redefines a reserved target; every other target is permitted.
# R12 moved the public verbs out of base.mk into this projection, but the guard
# stayed behind — and a generated project never includes base.mk, so the
# monopoly was unenforced in every real checkout. The guard belongs with the
# verbs it protects.
CUSTOM_MK_RESERVED_TARGETS := _custom_build_artifacts _custom_check_all _custom_clean_generated _custom_clean_status _custom_deps_check _custom_deps_lock _custom_deps_upgrade _custom_fix_all _custom_fix_apply _custom_fix_check _custom_fmt_all _custom_fmt_apply _custom_fmt_check _custom_gen_all _custom_gen_apply _custom_gen_check _custom_help_usage _custom_mod_all _custom_mod_apply _custom_mod_check _custom_release_rel _custom_release_status _custom_run_default _custom_setup_environment _custom_status_diagnostics _custom_test_all _custom_test_cache-checkpoint _custom_test_cache-clear _custom_test_cache-status _custom_test_full _custom_work_finish _custom_work_land _custom_work_start _custom_work_status build check clean deps fix fmt gen help mod release run setup status test work
ifneq ($(wildcard custom.mk),)
# Target definitions at column 0, excluding assignments (=) and dot-directives.
# $(shell) converts the newline-separated results to space-separated lists.
_CUSTOM_MK_DEFINED := $(shell awk '/^[A-Za-z_][A-Za-z0-9_-]*([ \t]+[A-Za-z_][A-Za-z0-9_-]*)*[ \t]*:/ && index($$0, "=") == 0 { line = $$0; sub(/:.*/, "", line); count = split(line, names, /[ \t]+/); for (i = 1; i <= count; i++) print names[i] }' custom.mk | sort -u)
_CUSTOM_MK_OFFENDERS := $(shell printf '%s\n' $(_CUSTOM_MK_DEFINED) | grep -xF $(foreach target,$(CUSTOM_MK_RESERVED_TARGETS),-e $(target)))
ifneq ($(_CUSTOM_MK_OFFENDERS),)
$(error custom.mk redefines reserved flext-infra target(s): $(_CUSTOM_MK_OFFENDERS) - reserved verbs/WHATs are a flext-infra monopoly; use _custom_<verb>_<what> with a non-reserved WHAT or (pre|post)-<verb>[-<what>] hooks)
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

.PHONY: $(PUBLIC_VERBS) _builtin_help_usage _builtin_setup_environment _builtin_deps_check _builtin_deps_lock _builtin_deps_upgrade _builtin_build_artifacts _builtin_check_all _builtin_test_all _builtin_test_full _builtin_test_cache-status _builtin_test_cache-clear _builtin_test_cache-checkpoint _builtin_fmt_check _builtin_fmt_all _builtin_fmt_apply _builtin_fix_check _builtin_fix_all _builtin_fix_apply _builtin_run_default _builtin_status_diagnostics _builtin_clean_status _builtin_clean_generated _builtin_release_status _builtin_release_rel _builtin_gen_check _builtin_gen_all _builtin_gen_apply _builtin_work_start _builtin_work_status _builtin_work_land _builtin_work_finish _builtin_mod_check _builtin_mod_all _builtin_mod_apply

# Every public verb dispatches straight into its private builtin. The verbs
# that used to round-trip through the Python serializer keep the environment
# prerequisite that round-trip carried.
#
# `setup` builds the environment it would otherwise require. `help` documents
# how to build it, so demanding an interpreter to print that documentation
# makes an unprovisioned checkout undiscoverable. Both still dispatch — they
# only drop the prerequisite.
$(filter-out setup help,$(PUBLIC_VERBS)): _builtin_require_environment
	$(call _dispatch,$@)

help:
	$(call _dispatch,$@)


# `all` = clean setup gen fmt fmt fix check test. CI=Y skips pytest inside
# make test (flext_infra._pytest_entry guards mro-v4p5), so all always calls
# test — the pytest gate is self-guarding.
all: _builtin_require_environment
	@$(SELF_MAKE) clean APPLY=Y
	@$(SELF_MAKE) setup
	@$(SELF_MAKE) gen APPLY=Y
	@$(SELF_MAKE) fmt APPLY=Y
	@$(SELF_MAKE) fmt APPLY=Y
	@$(SELF_MAKE) fix APPLY=Y
	@# check/test are read-only verbs. Clear APPLY and the CI token so an
	@# inherited APPLY=Y/CI=Y from the caller environment cannot reach them:
	@# check rejects APPLY, and pytest is forbidden under the CI token.
	@$(SELF_MAKE) check APPLY=
	@env -u CI $(SELF_MAKE) test APPLY=

.PHONY: all

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
		role=$$(git -C "$(PROJECT_ROOT)" config --local --get beads.role 2>/dev/null || echo ""); \
		if [ -z "$$role" ]; then \
			git -C "$(PROJECT_ROOT)" config --local beads.role maintainer; \
		fi; \
	fi
	@# Provision git hooks when the project ships an installer. This was the
	@# workspace-root custom.mk `hooks` + `post-boot` pair: a public target and a
	@# lifecycle hook that only ever ran one script setup already owns. CI skips
	@# it (no local commit hooks are needed there).
	@if [ "$${CI:-}" != "true" ] && [ -x "$(PROJECT_ROOT)/.github/scripts/install-git-hooks.sh" ]; then \
		"$(PROJECT_ROOT)/.github/scripts/install-git-hooks.sh"; \
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



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'clean' "$$(printf '%s' '$(_ALLOWED_WHATS_clean)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'release' "$$(printf '%s' '$(_ALLOWED_WHATS_release)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'gen' "$$(printf '%s' '$(_ALLOWED_WHATS_gen)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";




	@printf '  %-10s WHAT=%s\n' 'work' "$$(printf '%s' '$(_ALLOWED_WHATS_work)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";
	@printf '  %-10s %s\n' '' 'status is read-only; other WHATs require APPLY=Y';



	@printf '  %-10s WHAT=%s APPLY=Y\n' 'mod' "$$(printf '%s' '$(_ALLOWED_WHATS_mod)' | awk '{$$1=$$1; gsub(/ /, "|"); print}')";


	@printf '  %-10s %s\n' 'WORKSPACE' 'target repository (default: current project)';
	@printf '  %-10s %s\n' 'PROJECT' 'member checkout for work when WORKSPACE unset';
	@printf '  %-10s %s\n' 'BEAD' 'lane-root bead id for work start/land/finish';
	@printf '  %-10s %s\n' 'NAME' 'required lane slug for work start';
	@printf '  %-10s %s\n' 'KIND' 'optional feature|bugfix|hotfix|release; omitted derives from Bead issue_type';
	@printf '  %-10s %s\n' 'BASE' 'optional integration base override for work start';
	@printf '  %-10s %s\n' 'EPIC' 'registered epic bead id; nests work start as its child lane';
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

# Every verb depends on this guard. When the interpreter is missing the guard
# provisions it by invoking `setup` instead of failing: a fresh clone, a new
# worktree, or a CI runner has no venv yet, and forcing the operator to run
# `make setup` by hand turns every managed verb into a two-step ritual and
# breaks git hooks (which call `make fmt`/`make fix` in a bare checkout).
# `setup` is idempotent and cheap when the tooling already matches the lock,
# so the auto-provision path costs nothing in the common case. Only a setup
# that itself fails to produce the interpreter is a real error.
_builtin_require_environment:
	@if [ ! -x "$(RUNTIME_PYTHON)" ]; then \
		printf '==> environment interpreter missing; provisioning via setup\n' >&2; \
		$(SELF_MAKE) setup || exit $$?; \
	fi
	@if [ ! -x "$(RUNTIME_PYTHON)" ]; then \
		printf 'ERROR: setup did not produce the environment interpreter %s\n' "$(RUNTIME_PYTHON)" >&2; \
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
		env -u MAKEFILES -u GNUMAKEFLAGS -u MAKEFLAGS -u MAKELEVEL -u MAKEOVERRIDES -u MFLAGS -u PYTHONPATH \
			$(MAKE) -C "$(RUNTIME_ROOT)" _builtin_setup_environment; \
	fi
	@$(FLEXT_BINDING_RECIPE)
else ifeq ($(MAKE_PROFILE),workspace-root)
_builtin_setup_environment: _builtin_setup_submodules
	@$(SETUP_ENVIRONMENT_RECIPE)
	@$(UV) pip check --python "$(RUNTIME_VENV)"
	@$(FLEXT_BINDING_RECIPE)
else
_builtin_setup_environment: _builtin_setup_submodules
	@$(SETUP_ENVIRONMENT_RECIPE)
	@$(FLEXT_BINDING_RECIPE)
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
# CI=Y runs make.ci.check_gates (RULING 2: rules not skip-list).
_builtin_check_all: _builtin_require_environment
	@set -eu; \
	gates="$(strip $(CHECK_GATES))"; \
	if [ -z "$$gates" ]; then gates="$$(printf '%s' '$(CHECK_GATES_DEFAULT)' | tr ' ' ',')"; fi; \
	gates="$$(printf '%s' "$$gates" | tr -d '[:space:]')"; \
	if [ "$(strip $(CI))" = "Y" ]; then \
		gates="mypy,pyright,security,markdown,smells"; \
		printf 'INFO: CI=Y runs check gates: mypy pyright security markdown smells\n'; \
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


_builtin_test_full: _builtin_require_environment

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

# Read-only dual of `make fix APPLY=Y` — never mutate here.
_builtin_fix_check: _builtin_require_environment
	@$(UV_RUN) ruff check $(RUFF_PATHS)

# mro-38p39: `fix` is the mutating dual of `check`, so it routes through the
# same gate pipeline. Running `ruff check --fix` alone left every other fixable
# gate unreachable: a markdown finding the linter itself marks auto-fixable
# blocked `make check` while `make fix APPLY=Y` exited 0 without repairing it,
# so the canonical sequence could never reach green without hand-editing a
# governed file. The gate list is the SSOT can_fix set, not a literal -- an
# unscoped --fix would also run pyright and mypy, which repair nothing and cost
# ~37s, timing the verb out.
_builtin_fix_all: _builtin_require_environment
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) check run --workspace "$(PROJECT_ROOT)" --projects . --fix \
		--gates format,markdown,smells

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

# Disposable artifacts (caches, reports, traces) are owned by the flext-infra
# clean service and declared in config.make.clean, so the recipe stays a thin
# dispatch like every other verb instead of shell that drifts per project.
_builtin_clean_status:
	@$(PROJECT_FLEXT_INFRA) maintenance clean --workspace "$(PROJECT_ROOT)"

_builtin_clean_generated:
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) maintenance clean --workspace "$(PROJECT_ROOT)" --apply-changes

_builtin_release_status: _builtin_require_environment
	@$(UV) lock --project "$(PROJECT_ROOT)" --check
	@git -C "$(PROJECT_ROOT)" diff --quiet
	@git -C "$(PROJECT_ROOT)" diff --cached --quiet

# Release orchestration belongs to the verb that already owns releases. It
# previously lived in the workspace-root custom.mk as _custom_release_rel,
# putting the release pipeline outside the Make monopoly and making Release CI
# depend on a hand-written surface. Knobs come from the caller environment so
# no new selector variables enter the public surface; PUSH=1 opts into pushing.
_builtin_release_rel: _builtin_require_environment
	$(call _require_apply)
	@push_flag=--no-push; \
	if [ "$${PUSH:-0}" = "1" ]; then push_flag=--push; fi; \
	projects_args=""; \
	if [ -n "$${PROJECTS:-}" ]; then projects_args="--projects $${PROJECTS}"; \
	elif [ -n "$(PROJECT)" ] && [ "$(PROJECT)" != "." ]; then projects_args="--projects $(PROJECT)"; fi; \
	$(PROJECT_FLEXT_INFRA) release run \
		--workspace "$(PROJECT_ROOT)" \
		--apply \
		--no-dry-run \
		--phase "$${RELEASE_PHASE:-all}" \
		--version "$${VERSION}" \
		--tag "$${TAG}" \
		--interactive "$${INTERACTIVE:-0}" \
		--create-branches "$${CREATE_BRANCHES:-0}" \
		$$push_flag \
		$$projects_args

# Generation preserves the caller's scope. Conform owns analyzer roots in the
# rendered tooling context; dependency modernization owns only dependency
# settings that conform does not render.
_builtin_gen_check: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode check
	@$(PROJECT_FLEXT_INFRA) deps modernize --workspace "$(PROJECT_ROOT)" --check

_builtin_gen_all: _builtin_require_environment
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode apply
	@$(PROJECT_FLEXT_INFRA) deps modernize --workspace "$(PROJECT_ROOT)" --apply

_builtin_gen_apply: _builtin_gen_all

_builtin_work_status:
	@$(PROJECT_FLEXT_INFRA) workspace work --workspace "$(WORKSPACE)" --operation status --bead "$(BEAD)" --branch "$(BRANCH)"

_builtin_work_start:
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) workspace work --workspace "$(WORKSPACE)" --operation start --bead "$(BEAD)" $(if $(strip $(KIND)),--kind "$(KIND)") --name "$(NAME)" --base "$(BASE)" --epic "$(EPIC)" --apply

_builtin_work_land:
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) workspace work --workspace "$(WORKSPACE)" --operation land --bead "$(BEAD)" --apply

_builtin_work_finish:
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) workspace work --workspace "$(WORKSPACE)" --operation finish --bead "$(BEAD)" --apply

# `mod` was declared in the verb table (and listed in .PHONY) but no builtin
# handler was ever generated, so the dispatcher resolved a non-existent target
# and the verb was unreachable. The codemod engine already exists behind
# `flext-infra refactor mod`; these handlers are the missing dispatch into it.
# check reports pending fixes; apply runs the
# batch under the ruff/pyrefly rollback circuit.
_builtin_mod_check: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) refactor mod --workspace "$(PROJECT_ROOT)"

_builtin_mod_all: _builtin_require_environment
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) refactor mod --workspace "$(PROJECT_ROOT)" --apply

_builtin_mod_apply: _builtin_mod_all
