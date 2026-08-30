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
# === SECTION: workspace subprojects (managed) ===
# Source: config:workspace_subprojects (list), config:workspace_repositories (list)
# Computed: MANAGED_GITLINKS mirrors the read-only local .gitmodules topology.
WORKSPACE_SUBPROJECTS :=
MANAGED_GITLINKS :=
WORKSPACE_EDITABLES := $(PROJECT_NAME):.
UV_LINK_MODE := copy
# End SECTION: project identity

# === SECTION: user overrides (managed) ===
# Source: template (canonical public invocation knobs)
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
# The pytest process inherits a hard wall-clock boundary, mirroring
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
override PYTEST_PARALLEL_WORKERS := 2
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
# The explicit lazy-init selector is a hermetic, target-local transformation.
# Detect it before any parse-time topology probes so the public invocation and
# its recursive builtin never consult Git, worktrees, remotes, or a parent
# workspace merely to regenerate Python package initializers.
GEN_INIT_ONLY := $(if $(and $(filter init,$(WHAT)),$(filter gen _builtin_gen_init,$(MAKECMDGOALS))),Y,)
SETUP_BOOTSTRAP_ONLY := $(if $(filter setup _bootstrap_setup_tools,$(MAKECMDGOALS)),Y,)
# End SECTION: user overrides

# === SECTION: derived paths (managed) ===
# Source: computed (git rev-parse, MAKEFILE_LIST, abspath)
# Rule: PROJECT_ROOT is the checkout that OWNS this Makefile, never the caller's
# CWD. Deriving it from `pwd -P` made a subproject validate whatever tree the
# caller happened to stand in: `make -f <subproject>/Makefile` invoked from the
# superproject resolved RUFF_PATHS to the SUPERPROJECT's src/tests, so the
# subproject linted files it does not even contain. With many shared worktrees that
# silently validates the wrong tree.
SELF_MAKEFILE := $(abspath $(firstword $(MAKEFILE_LIST)))
MAKEFILE_ROOT := $(patsubst %/,%,$(dir $(SELF_MAKEFILE)))
PROJECT_ROOT := $(MAKEFILE_ROOT)
SETUP_BIN := $(PROJECT_ROOT)/.bin
SETUP_MISE_VERSION := 2026.8.14
ifeq ($(OS),Windows_NT)
TRACKED_MISE := $(PROJECT_ROOT)/bin/mise.cmd
else
TRACKED_MISE := $(PROJECT_ROOT)/bin/mise
endif
override SETUP_MISE := $(TRACKED_MISE)
MISE_LOCK_PLATFORMS := linux-x64,linux-arm64,linux-x64-musl,linux-arm64-musl,macos-x64,macos-arm64,windows-x64
MISE_LOCK_PROJECTS := .
override export FLEXT_PYTEST_TARGET_RAW := tests
WORKSPACE ?= $(PROJECT_ROOT)
# === SECTION: WORKSPACE_ROOT isolation (managed) ===
# Source: the checkout that owns this Makefile.
# Rule: a linked worktree is its own writable lane. Parent, superproject and
# primary-worktree discovery never redirect generated writes or runtime state.
WORKSPACE_ROOT := $(PROJECT_ROOT)
# End SECTION: WORKSPACE_ROOT isolation
# A workspace lane is always registered at the workspace root. Other verbs may
# select a subproject through PROJECT while workspace orchestration keeps the root
# so one Git worktree owns the complete project matrix.
ifeq ($(filter command line override,$(origin WORKSPACE)),)
ifneq ($(strip $(PROJECT)),)
ifneq ($(filter $(PROJECT),$(WORKSPACE_SUBPROJECTS)),)
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
CUSTOM_MAKEFILE := $(MAKEFILE_ROOT)/custom.mk
CUSTOM_DECLARED_TARGETS :=
ifneq ($(GEN_INIT_ONLY),Y)
ifneq ($(wildcard $(CUSTOM_MAKEFILE)),)
CUSTOM_DECLARED_TARGETS := $(shell awk '/^(pre|post)-[a-z][a-z0-9-]*:|^_custom_[a-z][a-z0-9_-]*:/ { target=$$1; sub(/:.*/, "", target); if (!seen[target]++) printf "%s ", target }' "$(CUSTOM_MAKEFILE)")
ifneq ($(.SHELLSTATUS),0)
$(error Failed to inspect custom Make targets in $(CUSTOM_MAKEFILE))
endif
endif
endif

ifeq ($(GEN_INIT_ONLY),Y)
_ALLOWED_WHATS_help := usage
_ALLOWED_WHATS_setup := environment
_ALLOWED_WHATS_deps := check lock upgrade
_ALLOWED_WHATS_build := artifacts
_ALLOWED_WHATS_check := all lint pyrefly mypy pyright security markdown smells
_ALLOWED_WHATS_test := all cache-status cache-clear cache-checkpoint
_ALLOWED_WHATS_fmt := check all apply
_ALLOWED_WHATS_fix := check all apply
_ALLOWED_WHATS_run := default
_ALLOWED_WHATS_status := diagnostics
_ALLOWED_WHATS_docs := all fix audit build validate
_ALLOWED_WHATS_clean := status generated
_ALLOWED_WHATS_release := status
_ALLOWED_WHATS_gen := check all apply init
_ALLOWED_WHATS_mod := check all apply
else
_ALLOWED_WHATS_help := usage $(patsubst _custom_help_%,%,$(filter _custom_help_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_setup := environment $(patsubst _custom_setup_%,%,$(filter _custom_setup_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_deps := check lock upgrade $(patsubst _custom_deps_%,%,$(filter _custom_deps_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_build := artifacts $(patsubst _custom_build_%,%,$(filter _custom_build_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_check := all lint pyrefly mypy pyright security markdown smells $(patsubst _custom_check_%,%,$(filter _custom_check_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_test := all cache-status cache-clear cache-checkpoint $(patsubst _custom_test_%,%,$(filter _custom_test_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_fmt := check all apply $(patsubst _custom_fmt_%,%,$(filter _custom_fmt_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_fix := check all apply $(patsubst _custom_fix_%,%,$(filter _custom_fix_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_run := default $(patsubst _custom_run_%,%,$(filter _custom_run_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_status := diagnostics $(patsubst _custom_status_%,%,$(filter _custom_status_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_docs := all fix audit build validate $(patsubst _custom_docs_%,%,$(filter _custom_docs_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_clean := status generated $(patsubst _custom_clean_%,%,$(filter _custom_clean_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_release := status $(patsubst _custom_release_%,%,$(filter _custom_release_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_gen := check all apply init $(patsubst _custom_gen_%,%,$(filter _custom_gen_%,$(CUSTOM_DECLARED_TARGETS)))
_ALLOWED_WHATS_mod := check all apply $(patsubst _custom_mod_%,%,$(filter _custom_mod_%,$(CUSTOM_DECLARED_TARGETS)))
endif
CHECK_GATES_ALLOWED := lint pyrefly mypy pyright security markdown smells
CHECK_GATES_DEFAULT := lint pyrefly mypy pyright security markdown smells
 DOCS_ACTIONS := fix audit build validate
 # End SECTION: verb dispatch

# === SECTION: lint/type paths (managed) ===
# Source: template + computed (script_dispatch conditional)
RUFF_PATHS := $(strip $(foreach d,src tests,$(if $(wildcard $(PROJECT_ROOT)/$(d)/.),$(PROJECT_ROOT)/$(d),)))
MYPY_PATHS := $(strip $(foreach d,src tests,$(if $(wildcard $(PROJECT_ROOT)/$(d)/.),$(PROJECT_ROOT)/$(d),)))
# End SECTION: lint/type paths

# === SECTION: project tool owner (managed) ===
# Source: caller-selected uv command; make setup owns environment provisioning.
UV ?= uv
UV_REQUESTED := $(UV)
CALLER_PATH := $(PATH)
CALLER_VIRTUAL_ENV := $(patsubst %/,%,$(VIRTUAL_ENV))
# End SECTION: project tool owner

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
_APPLY_WHAT_docs := fix
_APPLY_WHAT_clean := generated
_APPLY_WHAT_gen := apply
_APPLY_WHAT_mod := apply


# === SECTION: profile routing (managed) ===
# Source: repository-local profile selected from its own .gitmodules.
ifneq ($(filter $(MAKE_PROFILE),workspace standalone),$(MAKE_PROFILE))
$(error Invalid MAKE_PROFILE '$(MAKE_PROFILE)')
endif

RUNTIME_ROOT := $(PROJECT_ROOT)
# End SECTION: profile routing

RUNTIME_VENV := $(RUNTIME_ROOT)/.venv
PROJECT_VENV := $(PROJECT_ROOT)/.venv
FLEXT_INFRA_RUNTIME_ROOT := $(if $(filter $(MAKEFILE_ROOT),$(PROJECT_ROOT)),$(RUNTIME_ROOT),$(MAKEFILE_ROOT))
ifeq ($(OS),Windows_NT)
RUNTIME_BIN := $(RUNTIME_VENV)/Scripts
RUNTIME_PYTHON := $(RUNTIME_BIN)/python.exe
FLEXT_INFRA_RUNTIME_PYTHON := $(FLEXT_INFRA_RUNTIME_ROOT)/.venv/Scripts/python.exe
NORMALIZED_CALLER_PATH := $(shell cygpath --path "$(CALLER_PATH)")
ifneq ($(.SHELLSTATUS),0)
$(error cygpath failed to normalize PATH)
endif
NORMALIZED_CALLER_VIRTUAL_ENV := $(shell cygpath --unix "$(CALLER_VIRTUAL_ENV)")
ifneq ($(.SHELLSTATUS),0)
$(error cygpath failed to normalize VIRTUAL_ENV)
endif
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
ifneq ($(filter Y,$(GEN_INIT_ONLY) $(SETUP_BOOTSTRAP_ONLY)),)
RESOLVED_UV :=
else
RESOLVED_UV := $(shell PATH="$(SANITIZED_CALLER_PATH)" command -v "$(UV_REQUESTED)")
ifneq ($(.SHELLSTATUS),0)
$(error Required uv executable not found: $(UV_REQUESTED))
endif
endif
override UV := $(if $(strip $(RESOLVED_UV)),$(RESOLVED_UV),$(UV_REQUESTED))
override FLEXT_INFRA_PYTHON := $(FLEXT_INFRA_RUNTIME_PYTHON)
override UV_PROJECT := $(RUNTIME_ROOT)
override UV_PROJECT_ENVIRONMENT := $(RUNTIME_VENV)
override VIRTUAL_ENV := $(RUNTIME_VENV)
override PATH := $(RUNTIME_BIN):$(SANITIZED_CALLER_PATH)
export FLEXT_INFRA_PYTHON UV UV_PROJECT UV_PROJECT_ENVIRONMENT VIRTUAL_ENV PATH

.PHONY: _bootstrap_setup_tools

_bootstrap_setup_tools:
	@set -eu; \
	project_root="$(PROJECT_ROOT)"; \
	mise="$(SETUP_MISE)"; \
	mise_version="$(SETUP_MISE_VERSION)"; \
	uv_required="0.12"; \
	if [ ! -f "$$mise" ]; then \
		printf 'ERROR: missing generated mise launcher: %s; run make gen WHAT=apply APPLY=Y\n' "$$mise" >&2; \
		exit 2; \
	fi; \
	current=$$("$$mise" --version); \
	current=$${current%% *}; \
	if [ "$$current" != "$$mise_version" ]; then \
		printf 'ERROR: mise launcher version mismatch: expected %s, got %s\n' \
			"$$mise_version" "$$current" >&2; \
		exit 2; \
	fi; \
	if [ ! -f "$$project_root/mise.lock" ]; then \
		printf 'ERROR: missing generated mise.lock; run make gen WHAT=apply APPLY=Y and commit it\n' >&2; \
		exit 2; \
	fi; \
	scratch_parent="$$project_root/.test-tmp"; \
	mkdir -p "$$scratch_parent"; \
	scratch=$$(mktemp -d "$$scratch_parent/mise-setup.XXXXXX"); \
	trap 'find "$$scratch" -depth -delete' EXIT; \
	global_config="$$scratch/global-config.toml"; \
	config_dir="$$scratch/config"; \
	mkdir -p "$$config_dir"; \
	: > "$$global_config"; \
	MISE_CONFIG_DIR="$$config_dir" MISE_GLOBAL_CONFIG_FILE="$$global_config" \
		"$$mise" trust "$$project_root/.mise.toml"; \
	MISE_CONFIG_DIR="$$config_dir" MISE_GLOBAL_CONFIG_FILE="$$global_config" \
		"$$mise" -C "$$project_root" install --locked --yes; \
	uv_output=$$(MISE_CONFIG_DIR="$$config_dir" \
		MISE_GLOBAL_CONFIG_FILE="$$global_config" \
		"$$mise" -C "$$project_root" \
		exec -- uv --version); \
	case "$$uv_output" in \
		'uv '*) uv_actual=$${uv_output#uv }; uv_actual=$${uv_actual%% *} ;; \
		*) printf 'ERROR: uv --version returned an invalid value\n' >&2; exit 2 ;; \
	esac; \
	case "$$uv_actual" in \
		"$$uv_required"|"$$uv_required".*) ;; \
		*) printf 'ERROR: mise must install uv %s.x, found %s\n' \
			"$$uv_required" "$$uv_actual" >&2; exit 2 ;; \
	esac

ifeq ($(MAKE_PROFILE),workspace)
CODEGEN_SCOPE := all
ALLOWED_PROJECTS := . $(WORKSPACE_SUBPROJECTS)
else
CODEGEN_SCOPE := self
ALLOWED_PROJECTS := .
endif

# Workspace-root gate verbs fan out across declared subprojects through the generic
# `flext-infra workspace orchestrate` primitive (verb allowlist + CLI group come
# from the constants SSOT, never hardcoded here). Subprojects and standalone projects
# run the gate locally. FAIL_FAST forwards the stop-on-first-failure policy.
# Provisioning is declared once and shared by every profile. Creating a missing
# venv is provisioning; clearing a present one is destruction, so it never happens.
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
		$(UV) sync --project "$(PROJECT_ROOT)" $(UV_SYNC_FLAGS) --link-mode "$(UV_LINK_MODE)"; \
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
# declared subprojects. Selecting the root (PROJECT=.) would make it orchestrate
# itself forever; map `.` to WORKSPACE_SUBPROJECTS instead of failing closed mid-CI.
DEFAULT_PROJECTS := $(WORKSPACE_SUBPROJECTS) .

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
PROJECT_FLEXT_INFRA := if [ ! -x "$(FLEXT_INFRA_PYTHON)" ]; then printf 'ERROR: FLEXT_INFRA_PYTHON must name an executable managed Python\n' >&2; exit 2; fi; env -u PYTHONPATH -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u UV_PROJECT_ENVIRONMENT PATH="$(dir $(FLEXT_INFRA_PYTHON)):$(SANITIZED_CALLER_PATH)" PYTHONPATH="$(PROJECT_INFRA_PYTHONPATH)" $(FLEXT_INFRA_PYTHON) -m flext_infra
# Scaffold dev tools live in the validated optional dev
# profile; a fresh project must create its lock before later check-mode locks.
# Keyed on the environment's OWNER, not on the caller's profile. A subproject has
# no local venv -- RUNTIME_VENV is RUNTIME_ROOT/.venv -- so every checkout that
# provisions a shared environment must describe the same contents. A subproject
# syncing without --all-packages treats the siblings already installed there as
# surplus and uninstalls them, undoing the root's provisioning and leaving
# `uv sync --check` permanently divergent. A standalone project owns its venv
# alone and has no workspace packages to include.
SHARED_RUNTIME := $(if $(filter-out $(PROJECT_ROOT),$(RUNTIME_ROOT)),1,$(if $(strip $(WORKSPACE_SUBPROJECTS)),1,))
UV_SYNC_FLAGS := $(if $(SHARED_RUNTIME),--all-packages ,)--all-extras --all-groups

ifneq ($(strip $(PROJECT)),)
ifneq ($(strip $(PROJECTS)),)
$(error ERROR: Cannot use PROJECT and PROJECTS together)
endif
endif


ifneq ($(GEN_INIT_ONLY),Y)
-include custom.mk
endif
SELF_MAKE := $(MAKE) --no-print-directory -f "$(SELF_MAKEFILE)"

define _dispatch
	@set -eu; \
	what="$(strip $(WHAT))"; \
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
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" $$custom "*) custom_declared=Y ;; \
		*) custom_declared=N ;; \
	esac; \
	if [ "$$custom_declared" = N ]; then \
		case " $(_ALLOWED_WHATS_$(1)) " in \
			*" $$what "*) ;; \
			*) printf 'ERROR: unsupported %s WHAT=%s (allowed:%s)\n' "$(1)" "$$what" "$(_ALLOWED_WHATS_$(1))" >&2; exit 2 ;; \
		esac; \
	fi; \
	builtin="_builtin_$(1)_$$what"; \
	for hook in "pre-$(1)" "pre-$(1)-$$what"; do \
		case " $(CUSTOM_DECLARED_TARGETS) " in \
			*" $$hook "*) $(SELF_MAKE) "$$hook" ;; \
		esac; \
	done; \
	if [ "$$custom_declared" = Y ]; then \
		$(SELF_MAKE) "$$custom"; \
	else \
		$(SELF_MAKE) "$$builtin"; \
	fi; \
	for hook in "post-$(1)-$$what" "post-$(1)"; do \
		case " $(CUSTOM_DECLARED_TARGETS) " in \
			*" $$hook "*) $(SELF_MAKE) "$$hook" ;; \
		esac; \
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

.PHONY: $(PUBLIC_VERBS) _builtin_help_usage _builtin_setup_environment _builtin_deps_check _builtin_deps_lock _builtin_deps_upgrade _builtin_build_artifacts _builtin_check_all _builtin_test_all _builtin_test_cache-status _builtin_test_cache-clear _builtin_test_cache-checkpoint _builtin_fmt_check _builtin_fmt_all _builtin_fmt_apply _builtin_fix_check _builtin_fix_all _builtin_fix_apply _builtin_run_default _builtin_status_diagnostics _builtin_docs_all _builtin_docs_fix _builtin_docs_audit _builtin_docs_build _builtin_docs_validate _builtin_clean_status _builtin_clean_generated _builtin_release_status _builtin_gen_check _builtin_gen_all _builtin_gen_apply _builtin_gen_init _builtin_mod_check _builtin_mod_all _builtin_mod_apply

$(filter-out setup gen,$(PUBLIC_VERBS)):
	$(call _dispatch,$@)

# `gen init` deliberately bypasses generic lifecycle hooks. Hooks are allowed
# to discover workspaces and operational state, which would violate the narrow
# initializer contract before the canonical owner even starts.
gen:
ifeq ($(GEN_INIT_ONLY),Y)
gen: _builtin_gen_init
else
	$(call _dispatch,$@)
endif
# `setup` keeps its own recipe (it must not require the environment it is about
# to build), but it still runs the pre-/post-setup lifecycle hooks so a project
# declaring them in the custom handler surface is actually honoured.
setup: _bootstrap_setup_tools
	@"$(SETUP_MISE)" -C "$(PROJECT_ROOT)" exec -- \
		$(SELF_MAKE) _setup_lifecycle

.PHONY: _setup_lifecycle
_setup_lifecycle:
	@set -eu; \
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" pre-setup "*) $(SELF_MAKE) pre-setup ;; \
	esac
	@$(SELF_MAKE) _builtin_setup_environment
	@set -eu; \
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" post-setup "*) $(SELF_MAKE) post-setup ;; \
	esac

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


	@printf '  %-10s %s\n' 'PROJECT' 'subproject checkout when WORKSPACE unset';
	@printf '  %-10s %s\n' 'WORKSPACE' 'target repository (default: current project)';
	@printf '\n%s\n' 'Custom hooks (custom.mk):';
	@printf '  %s\n' 'Define pre-<verb>, post-<verb>, pre-<verb>-<what>, post-<verb>-<what>';
	@printf '  %s\n' 'in custom.mk to wrap one declared handler.';
	@printf '  %s\n' 'Add _custom_<verb>_<what> to define a new WHAT.';
	@if [ -n "$(strip $(CUSTOM_DECLARED_TARGETS))" ]; then \
		printf '  %s\n' 'Defined in this project:'; \
		for hook in $(CUSTOM_DECLARED_TARGETS); do printf '    %s\n' "$$hook"; done; \
	fi

# A project owns the sources declared by its manifest. The generated setup
# reconciler validates every initialized checkout before mutation, initializes
# only missing modules, and preserves declared branches that fix forward beyond
# the recorded gitlink.
.PHONY: _builtin_setup_submodules

# === SECTION: submodule setup (managed) ===
# Source: template (submodule_setup_recipe.j2)
# Computed: workspace uses WORKSPACE_SUBPROJECTS from config; standalone discovers
#           submodules with flext-managed=true from .gitmodules at runtime.
# Rule: setup PROVISIONS an absent governed gitlink and VERIFIES a present one.
#       An absent checkout holds no work, so setup initializes it at the recorded
#       gitlink. A present checkout is never destroyed: git checkout, git reset,
#       fetch, and branch attachment are forbidden. Pin validity is HEAD contains
#       gitlink. Declared branch is the named integration line;
#       legacy branch=. still resolves to the superproject named branch if present.
#       A checkout is also accepted on the superproject current branch (workspace
#       lane). Any third branch fails. Nested gitlinks belong to their own setup.
# Free: no
# End SECTION: submodule setup
_builtin_setup_submodules:
	@set -eu; \
	root="$(PROJECT_ROOT)"; \
	if [ ! -f "$$root/.gitmodules" ]; then exit 0; fi; \
	profile="$(MAKE_PROFILE)"; \
	if [ "$$profile" = "workspace" ]; then \
		managed="$(MANAGED_GITLINKS)"; \
	else \
		managed=""; \
		keys=""; \
		if keys=$$(git -C "$$root" config -f .gitmodules --name-only --get-regexp '^submodule\..*\.flext-managed$$'); then \
			keys_status=0; \
		else \
			keys_status=$$?; \
		fi; \
		if [ "$$keys_status" -ne 0 ] && [ "$$keys_status" -ne 1 ]; then \
			printf 'ERROR: cannot enumerate governed gitlinks\n' >&2; \
			exit "$$keys_status"; \
		fi; \
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
	validate_submodule() { \
		superproject="$$1"; \
		child_path="$$2"; \
		child_root="$$superproject/$$child_path"; \
		keys=""; \
		if keys=$$(git -C "$$superproject" config -f .gitmodules --name-only --get-regexp '^submodule\..*\.path$$'); then \
			keys_status=0; \
		else \
			keys_status=$$?; \
		fi; \
		if [ "$$keys_status" -ne 0 ] && [ "$$keys_status" -ne 1 ]; then \
			printf 'ERROR: cannot enumerate submodule paths\n' >&2; \
			exit "$$keys_status"; \
		fi; \
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
		validated_branch=$$(git check-ref-format --branch "$$branch"); \
		if [ "$$validated_branch" != "$$branch" ]; then \
			printf 'ERROR: branch validator changed %s to %s\n' "$$branch" "$$validated_branch" >&2; \
			exit 2; \
		fi; \
		gitlink_entry=$$(git -C "$$superproject" ls-files --stage -- "$$child_path"); \
		if [ -z "$$gitlink_entry" ]; then \
			printf 'ERROR: governed gitlink is absent from the index: %s\n' "$$child_path" >&2; \
			exit 2; \
		fi; \
		set -- $$gitlink_entry; \
		if [ "$$1" != 160000 ]; then \
			printf 'ERROR: governed path is not a gitlink: %s\n' "$$child_path" >&2; \
			exit 2; \
		fi; \
		gitlink="$$2"; \
		if [ ! -e "$$child_root/.git" ]; then \
			git -C "$$superproject" submodule update --init -- "$$child_path"; \
		fi; \
		current=$$(git -C "$$child_root" branch --show-current); \
		if [ -n "$$current" ] && [ "$$current" != "$$declared_branch" ] && \
		   [ -n "$$super_branch" ] && [ "$$current" = "$$super_branch" ]; then \
			branch="$$super_branch"; \
		fi; \
		head=$$(git -C "$$child_root" rev-parse HEAD); \
		if [ -n "$$current" ] && [ "$$current" != "$$branch" ]; then \
			printf 'ERROR: %s: conflicting branch %s; expected %s (setup never runs checkout/reset; switch it yourself while keeping dirty)\n' "$$child_path" "$$current" "$$accepted_branches" >&2; \
			exit 1; \
		fi; \
		if git -C "$$child_root" merge-base --is-ancestor "$$gitlink" HEAD; then \
			ancestor=Y; \
		else \
			status=$$?; if [ "$$status" -eq 1 ]; then ancestor=N; else exit "$$status"; fi; \
		fi; \
		if [ "$$ancestor" = N ]; then \
			if [ -z "$$current" ]; then \
				printf 'ERROR: %s: detached HEAD %s does not contain recorded gitlink %s\n' "$$child_path" "$$head" "$$gitlink" >&2; \
			else \
				printf 'ERROR: %s: branch %s does not contain recorded gitlink %s\n' "$$child_path" "$$branch" "$$gitlink" >&2; \
			fi; \
			exit 1; \
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
# Source: computed (MAKE_PROFILE routing) + operator contract
# Operator contract: setup PROVISIONS tooling only — mise, venv, dependencies.
# It never generates, conforms, or mutates project code; `make gen` (APPLY=Y)
# is the single public conformance/generation surface.
# Setup always reconciles directly from the lock. The venv is created when
# missing and is never cleared while present, because a concurrent lane may be
# running against it.
_builtin_setup_environment: _builtin_setup_submodules
	@$(SETUP_ENVIRONMENT_RECIPE)

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

_builtin_status_diagnostics: _builtin_require_environment
	@printf 'profile=%s\nproject=%s\nruntime=%s\n' \
		'$(MAKE_PROFILE)' '$(PROJECT_ROOT)' '$(RUNTIME_ROOT)'
	@$(UV) --version
	@$(UV) lock --project "$(PROJECT_ROOT)" --check
	@if [ -x "$(RUNTIME_PYTHON)" ]; then \
		$(UV) pip check --python "$(RUNTIME_VENV)"; \
	fi
	@git -C "$(PROJECT_ROOT)" status --short

_builtin_docs_all:
	@set -eu; \
	for action in $(DOCS_ACTIONS); do \
		case "$$action" in fix) mode=$(if $(filter Y,$(APPLY)),--apply,--check) ;; *) mode= ;; esac; \
		$(PROJECT_FLEXT_INFRA) docs "$$action" --workspace "$(PROJECT_ROOT)" --output-dir "$(PROJECT_ROOT)/.reports/docs" $$mode $(DOCS_PROJECT_ARGS); \
	done


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
		\( -name __pycache__ -o -name .mypy_cache -o -name .pytest_cache -o -name .ruff_cache -o -name .pyrefly_cache -o -name .benchmarks -o -name .hypothesis \) \
		-prune -exec sh -eu -c 'for target do find "$$target" -depth -delete; done' sh {} +


	@set -eu; \
	for target in "$(PROJECT_ROOT)/build" "$(PROJECT_ROOT)/dist" "$(PROJECT_ROOT)/htmlcov" "$(PROJECT_ROOT)/.reports"; do \
		if [ -e "$$target" ]; then find "$$target" -depth -delete; \
		elif [ -L "$$target" ]; then find "$$target" -depth -delete; fi; \
	done


	@set -eu; \
	for target in "$(PROJECT_ROOT)/.coverage" "$(PROJECT_ROOT)/.testmondata"; do \
		if [ -e "$$target" ]; then rm -- "$$target"; \
		elif [ -L "$$target" ]; then rm -- "$$target"; fi; \
	done


	@find "$(PROJECT_ROOT)" -type f \
		\( -name '*.pstats' \) \
		-delete


_builtin_release_status: _builtin_require_environment
	@$(UV) lock --project "$(PROJECT_ROOT)" --check
	@git -C "$(PROJECT_ROOT)" diff --quiet
	@git -C "$(PROJECT_ROOT)" diff --cached --quiet

# Generation has one owner. Conform preserves the caller's scope and applies
# the complete dependency/tooling projection before it verifies its fixed point.
# Dependency upgrades remain a separate explicit verb because they rewrite lock
# floors; gen must never run a second pyproject writer over conform's result.
define _mise_launcher_apply
	@set -eu; \
	scratch_parent="$(PROJECT_ROOT)/.test-tmp"; \
	mkdir -p "$$scratch_parent"; \
	scratch=$$(mktemp -d "$$scratch_parent/mise-launcher.XXXXXX"); \
	trap 'find "$$scratch" -depth -delete' EXIT; \
	mkdir -p "$$scratch/data" "$$scratch/cache" "$$scratch/state" "$$scratch/tmp"; \
	: > "$$scratch/global-config.toml"; \
	if MISE_GLOBAL_CONFIG_FILE="$$scratch/global-config.toml" \
		XDG_CACHE_HOME="$$scratch/cache" XDG_STATE_HOME="$$scratch/state" \
		MISE_DATA_DIR="$$scratch/data" MISE_CACHE_DIR="$$scratch/cache" \
		MISE_STATE_DIR="$$scratch/state" TMPDIR="$$scratch/tmp" \
		MISE_CEILING_PATHS="$$scratch_parent" MISE_TRUSTED_CONFIG_PATHS="$$scratch" \
		"$(SETUP_MISE)" -C "$$scratch" generate install-script \
		--version "$(SETUP_MISE_VERSION)" \
		--write "$$scratch/mise" --windows \
		>"$$scratch/generate.log" 2>&1; then \
		cat "$$scratch/generate.log"; \
	else \
		status=$$?; cat "$$scratch/generate.log"; exit "$$status"; \
	fi; \
	if grep -Fq 'mise WARN' "$$scratch/generate.log"; then \
		printf 'ERROR: Mise launcher generation emitted warnings\n' >&2; exit 2; \
	else \
		status=$$?; if [ "$$status" -ne 1 ]; then exit "$$status"; fi; \
	fi; \
	for project in $(MISE_LOCK_PROJECTS); do \
		if [ "$$project" = . ]; then project_root="$(PROJECT_ROOT)"; \
		else project_root="$(PROJECT_ROOT)/$$project"; fi; \
		if [ ! -f "$$project_root/.mise.toml" ]; then \
			printf 'ERROR: missing generated .mise.toml in %s\n' "$$project_root" >&2; exit 2; \
		fi; \
		mkdir -p "$$project_root/bin"; \
		shell_tmp="$$project_root/bin/.mise.new.$$$$"; \
		windows_tmp="$$project_root/bin/.mise.cmd.new.$$$$"; \
		cp "$$scratch/mise" "$$shell_tmp"; \
		cp "$$scratch/mise.cmd" "$$windows_tmp"; \
		chmod +x "$$shell_tmp"; \
		mv -f "$$shell_tmp" "$$project_root/bin/mise"; \
		mv -f "$$windows_tmp" "$$project_root/bin/mise.cmd"; \
	done
endef

define _mise_lock_apply
	@set -eu; \
	: "$${MISE_GITHUB_CREDENTIAL_COMMAND:?ERROR: make gen apply requires MISE_GITHUB_CREDENTIAL_COMMAND}"; \
	credential_command="$$MISE_GITHUB_CREDENTIAL_COMMAND"; \
	for project in $(MISE_LOCK_PROJECTS); do \
		if [ "$$project" = . ]; then project_root="$(PROJECT_ROOT)"; \
		else project_root="$(PROJECT_ROOT)/$$project"; fi; \
		if [ ! -f "$$project_root/.mise.toml" ]; then \
			printf 'ERROR: missing generated .mise.toml in %s\n' "$$project_root" >&2; exit 2; \
		fi; \
		scratch_parent="$$project_root/.test-tmp"; mkdir -p "$$scratch_parent"; \
		scratch=$$(mktemp -d "$$scratch_parent/mise-lock.XXXXXX"); \
		trap 'find "$$scratch" -depth -delete' EXIT; \
		cp "$$project_root/.mise.toml" "$$scratch/.mise.toml"; \
		locked_count=$$(awk '$$0 == "locked = true" { count++ } END { print count + 0 }' "$$scratch/.mise.toml"); \
		if [ "$$locked_count" -ne 1 ]; then \
			printf 'ERROR: expected one locked tool_config setting in %s\n' "$$project_root/.mise.toml" >&2; exit 2; \
		fi; \
		sed -i 's/^locked = true$$/locked = false/' "$$scratch/.mise.toml"; \
		mkdir -p "$$scratch/data" "$$scratch/cache" \
			"$$scratch/state" "$$scratch/tmp"; \
		: > "$$scratch/global-config.toml"; \
		if MISE_GITHUB_CREDENTIAL_COMMAND="$$credential_command" \
			MISE_GLOBAL_CONFIG_FILE="$$scratch/global-config.toml" \
			XDG_CACHE_HOME="$$scratch/cache" XDG_STATE_HOME="$$scratch/state" \
			MISE_DATA_DIR="$$scratch/data" MISE_CACHE_DIR="$$scratch/cache" \
			MISE_STATE_DIR="$$scratch/state" TMPDIR="$$scratch/tmp" \
			MISE_CEILING_PATHS="$$scratch_parent" MISE_TRUSTED_CONFIG_PATHS="$$scratch" \
			"$(SETUP_MISE)" -C "$$scratch" lock \
			--platform "$(MISE_LOCK_PLATFORMS)" >"$$scratch/lock.log" 2>&1; then \
			cat "$$scratch/lock.log"; \
		else \
			status=$$?; cat "$$scratch/lock.log"; exit "$$status"; \
		fi; \
		if grep -Fq 'mise WARN' "$$scratch/lock.log"; then \
			printf 'ERROR: Mise lock generation emitted warnings for %s\n' "$$project_root" >&2; exit 2; \
		else \
			status=$$?; if [ "$$status" -ne 1 ]; then exit "$$status"; fi; \
		fi; \
		$(PROJECT_FLEXT_INFRA) codegen mise-artifacts \
			--workspace "$$scratch" --apply; \
		lock_tmp="$$project_root/.mise.lock.new.$$$$"; \
		cp "$$scratch/mise.lock" "$$lock_tmp"; \
		mv -f "$$lock_tmp" "$$project_root/mise.lock"; \
		find "$$scratch" -depth -delete; trap - EXIT; \
	done
endef

define _mise_artifacts_check
	@set -eu; \
	for project in $(MISE_LOCK_PROJECTS); do \
		if [ "$$project" = . ]; then project_root="$(PROJECT_ROOT)"; \
		else project_root="$(PROJECT_ROOT)/$$project"; fi; \
		$(PROJECT_FLEXT_INFRA) codegen mise-artifacts \
			--workspace "$$project_root" --check; \
		if [ "$$project" != . ]; then \
			if cmp "$(PROJECT_ROOT)/bin/mise" "$$project_root/bin/mise"; then :; else \
				status=$$?; if [ "$$status" -eq 1 ]; then \
					printf 'ERROR: Unix Mise launcher differs in %s\n' "$$project_root" >&2; exit 2; \
				else exit "$$status"; fi; \
			fi; \
			if cmp "$(PROJECT_ROOT)/bin/mise.cmd" "$$project_root/bin/mise.cmd"; then :; else \
				status=$$?; if [ "$$status" -eq 1 ]; then \
					printf 'ERROR: Windows Mise launcher differs in %s\n' "$$project_root" >&2; exit 2; \
				else exit "$$status"; fi; \
			fi; \
		fi; \
	done
endef

define _generated_docs
	@$(PROJECT_FLEXT_INFRA) docs generate --workspace "$(PROJECT_ROOT)" --output-dir "$(PROJECT_ROOT)/.reports/docs" $(1) $(DOCS_PROJECT_ARGS)
endef

_builtin_gen_check: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode check
	$(call _generated_docs,--check)
	$(call _mise_artifacts_check)

_builtin_gen_init:
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) codegen init --workspace "$(PROJECT_ROOT)" --apply
	@$(PROJECT_FLEXT_INFRA) codegen init --workspace "$(PROJECT_ROOT)" --check

_builtin_gen_all:
	$(call _require_apply)
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode apply
	$(call _generated_docs,--apply)
	$(call _mise_launcher_apply)
	$(call _mise_lock_apply)
	$(call _mise_artifacts_check)

_builtin_gen_apply: _builtin_gen_all
