# @flext-generated: continuous
# @flext-owner: flext-infra/config/codegen.yaml + flext-infra/src/flext_infra/templates/project/base/Makefile.j2
# @flext-adjust: edit the owner configuration or template; never this projection
# @flext-regenerate: make gen
# mcb-scripts — selector-free generated project interface.
# Managed by flext-infra codegen conform for new and existing repositories.
# === SECTION: header (managed) ===
# Source: template (base/Makefile.j2)
# Free: no
# End SECTION: header

override SHELL := /bin/sh
# GNU MAKE_COMMAND may be a bare name. Resolve it before changing PATH so
# recursive lifecycle calls keep this invoker instead of selecting a Mise shim.
ifneq ($(filter /%,$(MAKE_COMMAND)),)
override SELF_MAKE_EXECUTABLE := $(MAKE_COMMAND)
else
override SELF_MAKE_EXECUTABLE := $(shell command -v "$(MAKE_COMMAND)")
ifneq ($(.SHELLSTATUS),0)
$(error Cannot resolve current Make executable: $(MAKE_COMMAND))
endif
endif
override SELF_MAKE_EXECUTABLE := $(realpath $(SELF_MAKE_EXECUTABLE))
ifeq ($(strip $(SELF_MAKE_EXECUTABLE)),)
$(error Current Make executable has no physical path: $(MAKE_COMMAND))
endif
.DEFAULT_GOAL := help

# Approval fixes its context before any topology, activation or credential read.
ifneq ($(filter pre-commit,$(MAKECMDGOALS)),)
override CI := Y
export CI
ifneq ($(strip $(HELP) $(OPTIONS)),)
$(error Approval cannot run with HELP or OPTIONS)
endif
ifneq ($(strip $(foreach flag,n q t,$(findstring $(flag),$(firstword $(MAKEFLAGS))))),)
$(error Approval requires execution; dry-run, question and touch are forbidden)
endif
ifneq ($(strip $(MAKEFILES)),)
$(error Approval cannot load caller-supplied Makefiles)
endif
ifneq ($(filter command line override,$(origin MAKE_COMMAND)),)
$(error Approval cannot replace the native Make invoker)
endif
endif

# Capture the selected approval mode before any project-owned include.
ifeq ($(strip $(CI)),Y)
override APPROVAL_CONTEXT := Y
ifneq ($(filter upg _upg% dep propagate,$(MAKECMDGOALS)),)
$(error Resolution and member propagation are forbidden in CI)
endif
endif

ifeq ($(filter command line override,$(origin SETUP_BOOTSTRAP_ONLY)),)
ifneq ($(filter setup,$(MAKECMDGOALS)),)
SETUP_BOOTSTRAP_ONLY := Y
export SETUP_BOOTSTRAP_ONLY
endif
endif
ifeq ($(filter command line override,$(origin GEN_INIT_ONLY)),)
ifneq ($(filter initialize,$(MAKECMDGOALS)),)
GEN_INIT_ONLY := Y
export GEN_INIT_ONLY
endif
endif

# One GitHub credential for every verb (operator 2026-10-03): the first
# non-empty of the caller's GITHUB_TOKEN, GH_TOKEN, MISE_GITHUB_TOKEN, then the
# declared credential command when it is installed. Every name gh, uv and mise
# read carries that same value, so no inherited alias can shadow it. The value
# stays in the environment and is never printed.
GITHUB_AUTH_SOURCE := $(if $(strip $(GITHUB_TOKEN)),GITHUB_TOKEN,$(if $(strip $(GH_TOKEN)),GH_TOKEN,$(if $(strip $(MISE_GITHUB_TOKEN)),MISE_GITHUB_TOKEN,none)))
GITHUB_AUTH_STATUS := not-selected
override GITHUB_AUTH_COMMAND := not-selected
override GITHUB_AUTH_COMMAND_STATUS := not-selected
GITHUB_AUTH_HOST_SOURCE := $(if $(strip $(GH_HOST)),GH_HOST,native-default)
GITHUB_AUTH_CONFIG_OVERRIDE := $(if $(strip $(GH_CONFIG_DIR)),yes,no)
GITHUB_AUTH_XDG_OVERRIDE := $(if $(strip $(XDG_CONFIG_HOME)),yes,no)
GITHUB_AUTH_SESSION_BUS := $(if $(strip $(DBUS_SESSION_BUS_ADDRESS)),yes,no)
GITHUB_AUTH_CI := other
ifeq ($(strip $(CI)),Y)
GITHUB_AUTH_CI := ci
else ifeq ($(strip $(CI)),N)
GITHUB_AUTH_CI := local
else ifeq ($(strip $(CI)),)
GITHUB_AUTH_CI := unset
endif
GITHUB_TOKEN := $(firstword $(GITHUB_TOKEN) $(GH_TOKEN) $(MISE_GITHUB_TOKEN))
ifeq ($(GITHUB_TOKEN),)
ifneq ($(filter local unset,$(GITHUB_AUTH_CI)),)
GITHUB_AUTH_SOURCE := gh
override GITHUB_AUTH_COMMAND := $(shell command -v gh 2>/dev/null)
override GITHUB_AUTH_COMMAND_STATUS := $(.SHELLSTATUS)
ifeq ($(GITHUB_AUTH_COMMAND_STATUS),0)
GITHUB_TOKEN := $(shell "$(GITHUB_AUTH_COMMAND)" auth token 2>/dev/null)
GITHUB_AUTH_STATUS := $(.SHELLSTATUS)
else
GITHUB_AUTH_STATUS := $(GITHUB_AUTH_COMMAND_STATUS)
endif
endif
endif
ifneq ($(GITHUB_TOKEN),)
GH_TOKEN := $(GITHUB_TOKEN)
MISE_GITHUB_TOKEN := $(GITHUB_TOKEN)
export GITHUB_TOKEN GH_TOKEN MISE_GITHUB_TOKEN
else
unexport GITHUB_TOKEN GH_TOKEN MISE_GITHUB_TOKEN
endif
unexport GITHUB_API_TOKEN
GITHUB_AUTH_PRESENT := $(if $(strip $(GITHUB_TOKEN)),yes,no)
GITHUB_AUTH_PHYSICAL_COMMAND := $(if $(filter gh,$(GITHUB_AUTH_SOURCE)),$(realpath $(GITHUB_AUTH_COMMAND)),not-selected)
# Diagnostics expose only producer metadata, never credentials or command stderr.
GITHUB_AUTH_DIAGNOSTICS = printf 'github-auth source=%s extraction-exit=%s present=%s ci=%s command-exit=%s command=%s physical-command=%s host-source=%s gh-config-override=%s xdg-config-override=%s session-bus=%s\n' '$(GITHUB_AUTH_SOURCE)' '$(GITHUB_AUTH_STATUS)' '$(GITHUB_AUTH_PRESENT)' '$(GITHUB_AUTH_CI)' '$(GITHUB_AUTH_COMMAND_STATUS)' '$(GITHUB_AUTH_COMMAND)' '$(GITHUB_AUTH_PHYSICAL_COMMAND)' '$(GITHUB_AUTH_HOST_SOURCE)' '$(GITHUB_AUTH_CONFIG_OVERRIDE)' '$(GITHUB_AUTH_XDG_OVERRIDE)' '$(GITHUB_AUTH_SESSION_BUS)'

# === SECTION: project identity (managed) ===
# Source: config:dist / config:make_profile / config:repository_root_rel / config:uv_link_mode
PROJECT_NAME := mcb-scripts
MAKE_PROFILE := standalone
REPOSITORY_ROOT_REL := .
# === SECTION: workspace subprojects (managed) ===
# Source: config:workspace_subprojects (list), config:workspace_repositories (list)
# Computed: MANAGED_GITLINKS mirrors the read-only local .gitmodules topology.
WORKSPACE_SUBPROJECTS :=
MANAGED_GITLINKS :=
UV_LINK_MODE := copy
# End SECTION: project identity

# === SECTION: public boundary (managed) ===
# The Makefile validates no caller input. Every verb
# always applies; a mistyped variable fails where the verb consumes it, and an
# unconsumed variable is ignored.
PYTEST_DIAG_ARGS := -rA --durations=0 --tb=long --showlocals
PYTEST_REPORT_ARGS := -ra --durations=25 --durations-min=0.001 --tb=short
PYTEST_PROCESS_TIMEOUT_SECONDS := 124
# The pytest process inherits a hard wall-clock boundary, so a hung
# run is terminated even if the runner itself stalls.
override PYTEST_BOUNDED = timeout --signal=TERM --kill-after=5s "$(PYTEST_PROCESS_TIMEOUT_SECONDS)s"
PYTEST_REPORTS_DIR := .reports/tests
PYTEST_CACHE_HOME = $(if $(strip $(XDG_CACHE_HOME)),$(XDG_CACHE_HOME),$(if $(strip $(HOME)),$(HOME)/.cache,))
# One persistent testmon database per project, shared by every checkout and
# worktree of it (like the Mypy cache): testmon tracks each checkout's source by
# file checksum, so a new lane starts from the project's measured selection
# instead of a cold full inventory.
override FLEXT_PYTEST_TESTMON_DATABASE = $(if $(strip $(PYTEST_CACHE_HOME)),$(PYTEST_CACHE_HOME)/flext/infra/testmon/$(PROJECT_NAME)/.testmondata)
# Storage law: the pytest scratch root sits under the user home, keyed by the
# absolute checkout path, never under /tmp and never inside a versioned tree.
override FLEXT_PYTEST_SCRATCH_ROOT = $(if $(strip $(HOME)),$(HOME)/tmp/.flext-runtime$(PROJECT_ROOT)/scratch)
# Profiles sit beside the other reports of this checkout (.reports is ignored).
PROFILE_REPORTS_DIR = $(PROJECT_ROOT)/$(dir $(PYTEST_REPORTS_DIR))profiles
override PYTEST_CASE_TIMEOUT_SECONDS := 10
override PYTEST_RUN_TIMEOUT_SECONDS := 120
override PYTEST_TERMINATION_GRACE_SECONDS := 2
override PYTEST_TIMEOUT_EXIT_CODE := 124
override PYTEST_ENFORCEMENT_PLUGIN := flext_tests_enforcement
override PYTEST_PROGRESS_ARGS := --verbose
override PYTEST_REPORT_ARGS := -ra --durations=25 --durations-min=0.001 --tb=short
override PYTEST_DIAG_ARGS := -rA --durations=0 --tb=long --showlocals
override PYTEST_PARALLEL_WORKERS := 1
override PYTEST_PARALLEL_WORKER_MEMORY_GB := 2
override PYTEST_PARALLEL_DISTRIBUTION := load
override PYTEST_PARALLEL_SCHEDULE_CHUNK := 1
override PYTEST_PROFILE_SORT := cumulative
override PYTEST_PROFILE_LIMIT := 50
override PROCESS_TIMEOUT_COMMAND := timeout
override export FLEXT_PYTEST_REPORTS_RAW := $(value PYTEST_REPORTS_DIR)
# End SECTION: public boundary

# === SECTION: derived paths (managed) ===
# Source: computed (git rev-parse, MAKEFILE_LIST, abspath)
# Rule: PROJECT_ROOT is the checkout that OWNS this Makefile, never the caller's
# CWD. Deriving it from `pwd -P` made a member validate whatever tree the
# caller happened to stand in: `make -f <member>/Makefile` invoked from the
# superproject resolved RUFF_PATHS to the SUPERPROJECT's src/tests, so the
# member linted files it does not even contain. With many shared worktrees that
# silently validates the wrong tree.
override SELF_MAKEFILE := $(realpath $(firstword $(MAKEFILE_LIST)))
override MAKEFILE_ROOT := $(patsubst %/,%,$(dir $(SELF_MAKEFILE)))
override PROJECT_ROOT := $(MAKEFILE_ROOT)
SETUP_BIN := $(PROJECT_ROOT)/.bin
# Caller-overridable test target (a directory or a single test file under
# it): `make test FLEXT_PYTEST_TARGET_RAW=tests/unit/foo_test.py` runs one
# file through the same bounded runner. The default stays the declared
# target_directory; the runner validates the path's shape.
export FLEXT_PYTEST_TARGET_RAW := tests
# === SECTION: REPOSITORY_ROOT isolation (managed) ===
# Source: physical checkout topology; caller variables cannot select a workspace.
# Inside a workspace every make run, root
# or member, uses the workspace runtime. A member resolves the Git superproject
# that checks it out as a submodule; a checkout without one (a standalone clone,
# a linked worktree) owns its runtime.
ifneq ($(GEN_INIT_ONLY),)
override REPOSITORY_ROOT := $(MAKEFILE_ROOT)
else
override REPOSITORY_ROOT := $(shell cd "$(MAKEFILE_ROOT)" && if root=$$(git rev-parse --show-superproject-working-tree); then if [ -n "$$root" ]; then cd "$$root" && pwd -P; else pwd -P; fi; else status=$$?; if [ "$$status" -ne 1 ]; then exit "$$status"; fi; pwd -P; fi)
ifneq ($(.SHELLSTATUS),0)
$(error Cannot resolve the physical workspace for $(MAKEFILE_ROOT))
endif
endif
# End SECTION: REPOSITORY_ROOT isolation
# === SECTION: verb dispatch (managed) ===
# Source: config:make.verbs and the canonical gate vocabulary. A verb exists
# only in the profiles it declares (make.verbs[].profiles).
PUBLIC_VERBS := help setup pre-commit upg build check smells test test-full test-file file-gate profile-test profile-test-report fmt fix fix-namespace fix-accessors audit status verify-clean docs clean bootstrap-candidate release-plan release-version release-tag release-build publication gen gen-footprint initialize mod mod-text mod-text-candidate mod-snapshots waza duplication sonarcloud-sync sonarcloud-issues rust validate guard
BUILTIN_VERBS := help setup pre-commit upg build check smells test test-full test-file file-gate profile-test profile-test-report fmt fix fix-namespace fix-accessors audit status verify-clean docs clean bootstrap-candidate release-plan release-version release-tag release-build publication gen gen-footprint initialize mod mod-text mod-text-candidate mod-snapshots waza duplication sonarcloud-sync sonarcloud-issues
SCRIPT_VERBS := rust validate guard

CUSTOM_MAKEFILE := $(MAKEFILE_ROOT)/custom.mk
CUSTOM_DECLARED_TARGETS :=
ifneq ($(wildcard $(CUSTOM_MAKEFILE)),)
CUSTOM_DECLARED_TARGETS := $(shell awk '/^[a-z_][a-z0-9_-]*:/ { target=$$1; sub(/:.*/, "", target); if (!seen[target]++) printf "%s ", target }' "$(CUSTOM_MAKEFILE)")
ifneq ($(.SHELLSTATUS),0)
$(error Failed to inspect custom Make targets in $(CUSTOM_MAKEFILE))
endif
ifneq ($(filter pre-commit,$(CUSTOM_DECLARED_TARGETS)),)
$(error Mandatory approval cannot be replaced by custom targets)
endif
ifeq ($(APPROVAL_CONTEXT),Y)
ifneq ($(filter setup audit check test verify-clean,$(CUSTOM_DECLARED_TARGETS)),)
$(error Approval stages cannot be replaced by custom targets)
endif
# Wrapper parity: a custom approval-stage hook is legitimate only while it
# chains the canonical builtin inside its recipe (the host-service harness
# pattern). A declared hook without the builtin reference is a replacement
# and stays forbidden.
ifneq ($(filter _custom-pre-commit,$(CUSTOM_DECLARED_TARGETS)),)
ifeq ($(shell grep -c "_builtin-pre-commit" $(CUSTOM_MAKEFILE) || true),0)
$(error Approval stage _custom-pre-commit must chain _builtin-pre-commit (wrapper parity; replacements are forbidden))
endif
endif

ifneq ($(filter _custom-setup,$(CUSTOM_DECLARED_TARGETS)),)
ifeq ($(shell grep -c "_builtin-setup" $(CUSTOM_MAKEFILE) || true),0)
$(error Approval stage _custom-setup must chain _builtin-setup (wrapper parity; replacements are forbidden))
endif
endif

ifneq ($(filter _custom-audit,$(CUSTOM_DECLARED_TARGETS)),)
ifeq ($(shell grep -c "_builtin-audit" $(CUSTOM_MAKEFILE) || true),0)
$(error Approval stage _custom-audit must chain _builtin-audit (wrapper parity; replacements are forbidden))
endif
endif

ifneq ($(filter _custom-check,$(CUSTOM_DECLARED_TARGETS)),)
ifeq ($(shell grep -c "_builtin-check" $(CUSTOM_MAKEFILE) || true),0)
$(error Approval stage _custom-check must chain _builtin-check (wrapper parity; replacements are forbidden))
endif
endif

ifneq ($(filter _custom-test,$(CUSTOM_DECLARED_TARGETS)),)
ifeq ($(shell grep -c "_builtin-test" $(CUSTOM_MAKEFILE) || true),0)
$(error Approval stage _custom-test must chain _builtin-test (wrapper parity; replacements are forbidden))
endif
endif

ifneq ($(filter _custom-verify-clean,$(CUSTOM_DECLARED_TARGETS)),)
ifeq ($(shell grep -c "_builtin-verify-clean" $(CUSTOM_MAKEFILE) || true),0)
$(error Approval stage _custom-verify-clean must chain _builtin-verify-clean (wrapper parity; replacements are forbidden))
endif
endif

endif
endif
DOCS_ACTIONS := generate fix fmt build validate audit
 # End SECTION: verb dispatch

# === SECTION: lint/type paths (managed) ===
# Source: template + computed (script_dispatch conditional)
RUFF_PATHS := $(strip $(foreach d,src tests examples,$(if $(wildcard $(PROJECT_ROOT)/$(d)/.),$(PROJECT_ROOT)/$(d),)))
MYPY_PATHS := $(strip $(foreach d,src tests examples,$(if $(wildcard $(PROJECT_ROOT)/$(d)/.),$(PROJECT_ROOT)/$(d),)))
# End SECTION: lint/type paths

# === SECTION: project tool owner (managed) ===
# Source: the repository's declared Mise toolchain, provisioned by make setup.
# Every tool resolves from the mise-managed PATH (direnv activation shims, or
# the CI provisioned PATH); mise resolves each tool through the committed
# mise.lock wherever it pins one, and `make audit` proves the pins.
override UV := uv
CALLER_PATH := $(PATH)
CALLER_VIRTUAL_ENV := $(patsubst %/,%,$(VIRTUAL_ENV))
# End SECTION: project tool owner

# === SECTION: profile routing (managed) ===
# Source: repository topology. workspace has .gitmodules; standalone does not.
# Attached members share their Git superproject runtime; standalone owns itself.
override RUNTIME_ROOT := $(REPOSITORY_ROOT)
override export GIT_CEILING_DIRECTORIES := $(abspath $(RUNTIME_ROOT)/..)
override export MISE_CEILING_PATHS := $(abspath $(RUNTIME_ROOT)/..)
override export MISE_CONFIG_DIR := $(RUNTIME_ROOT)/.mise
override export MISE_GLOBAL_CONFIG_FILE := $(MISE_CONFIG_DIR)/config.toml
override export MISE_SYSTEM_CONFIG_DIR := $(MISE_CONFIG_DIR)
# A file override wins over the system directory in native Mise discovery.
unexport MISE_SYSTEM_CONFIG_FILE
# Project provisioning must never install or resolve tools from host config.
# The physical runtime owns both its environment and frozen tool identities.
# Attached members retain their own lock inputs for standalone consumption;
# the pinned mise release is the [tools] entry the committed mise.lock pins.
# End SECTION: profile routing

# D-VENV: the environment is <RUNTIME_ROOT>/.venv only. A member inside a
# workspace uses the workspace environment; a standalone checkout or a linked
# worktree owns its own physical environment inside the checkout. The path is
# law, never configuration, and is never shared with another checkout.
override RUNTIME_VENV := $(RUNTIME_ROOT)/.venv
ifeq ($(OS),Windows_NT)
override RUNTIME_BIN := $(RUNTIME_VENV)/Scripts
override RUNTIME_PYTHON := $(RUNTIME_BIN)/python.exe
NORMALIZED_CALLER_PATH := $(shell cygpath --path "$(CALLER_PATH)")
ifneq ($(.SHELLSTATUS),0)
$(error cygpath failed to normalize PATH)
endif
ifneq ($(strip $(CALLER_VIRTUAL_ENV)),)
NORMALIZED_CALLER_VIRTUAL_ENV := $(shell cygpath --unix "$(CALLER_VIRTUAL_ENV)")
ifneq ($(.SHELLSTATUS),0)
$(error cygpath failed to normalize VIRTUAL_ENV)
endif
endif
CALLER_VIRTUAL_ENV_BIN := $(NORMALIZED_CALLER_VIRTUAL_ENV)/Scripts
else
override RUNTIME_BIN := $(RUNTIME_VENV)/bin
override RUNTIME_PYTHON := $(RUNTIME_BIN)/python
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
override FLEXT_INFRA_PYTHON := $(RUNTIME_PYTHON)
override UV_PROJECT := $(RUNTIME_ROOT)
override UV_PROJECT_ENVIRONMENT := $(RUNTIME_VENV)
override VIRTUAL_ENV := $(RUNTIME_VENV)
override PATH := $(RUNTIME_BIN):$(SANITIZED_CALLER_PATH)
unexport UV
export FLEXT_INFRA_PYTHON UV_PROJECT UV_PROJECT_ENVIRONMENT VIRTUAL_ENV PATH RUNTIME_ROOT

# One bootstrap serves `setup` (frozen install) and `upg` (resolve + install);
# the public verb selects its lifecycle and resolution through target-specific
# variables.
TOOL_BOOTSTRAP_LIFECYCLE := _setup_lifecycle
TOOL_BOOTSTRAP_RESOLVE :=
.PHONY: _bootstrap_setup_tools

_bootstrap_setup_tools:
	# The lifecycle invokes recursive make through mise, so preserve jobserver FDs.
	+@set -eu; \
	if ! command -v mise >/dev/null 2>&1; then \
		printf 'ERROR: mise is not installed; install it (https://mise.run) and retry\n' >&2; \
		exit 2; \
	fi; \
	mise_lock="$(PROJECT_ROOT)/mise.lock"; \
	mise_lock_usable=0; \
	if [ -f "$$mise_lock" ] && ! grep -qE '^(<<<<<<< |=======$$|>>>>>>> )' "$$mise_lock"; then \
		mise_lock_usable=1; \
	fi; \
	mise_pin=; \
	mise_url=; \
	mise_checksum=; \
	case "$$(uname -s)/$$(uname -m)" in \
		Linux/x86_64) mise_platform=linux-x64 ;; \
		Linux/aarch64) mise_platform=linux-arm64 ;; \
		Darwin/arm64) mise_platform=macos-arm64 ;; \
		Darwin/x86_64) mise_platform=macos-x64 ;; \
		*) mise_platform= ;; \
	esac; \
	if [ "$$mise_lock_usable" = 1 ] && [ -n "$$mise_platform" ]; then \
		mise_pin="$$( awk 'index($$0, "[[tools.\"github:jdx/mise\"]]") == 1 { inside = 1; next } inside && substr($$0, 1, 1) == "[" { exit } inside && $$1 == "version" { gsub(/[",]/, "", $$3); print $$3; exit }' "$(PROJECT_ROOT)/mise.lock" )"; \
		mise_key=url; \
		mise_url="$$( awk -v section="[tools.\"github:jdx/mise\".\"platforms.$${mise_platform}\"]" -v key="$${mise_key}" \
	'$$0 == section { inside = 1; next } inside && substr($$0, 1, 1) == "[" { exit } inside && $$1 == key { gsub(/"/, "", $$3); print $$3; exit }' "$(PROJECT_ROOT)/mise.lock" )"; \
		mise_key=checksum; \
		mise_checksum="$$( awk -v section="[tools.\"github:jdx/mise\".\"platforms.$${mise_platform}\"]" -v key="$${mise_key}" \
	'$$0 == section { inside = 1; next } inside && substr($$0, 1, 1) == "[" { exit } inside && $$1 == key { gsub(/"/, "", $$3); print $$3; exit }' "$(PROJECT_ROOT)/mise.lock" )"; \
	fi; \
	case "$$mise_url|$$mise_checksum" in \
		https://github.com/*/releases/download/*\|sha256:*) ;; \
		*) mise_pin= ;; \
	esac; \
	mise_bootstrap_root="$${XDG_CACHE_HOME:-$$HOME/.cache}/flext/infra/mise-bootstrap"; \
	if [ -n "$$mise_pin" ]; then \
		mise_sha256="$${mise_checksum#sha256:}"; \
		mise_bootstrap_bin="$$mise_bootstrap_root/$$mise_pin/mise"; \
		if [ ! -x "$$mise_bootstrap_bin" ]; then \
			printf 'setup: recovering github:jdx/mise %s from the mise.lock release asset for %s\n' "$$mise_pin" "$$mise_platform"; \
			mise_stage="$$mise_bootstrap_root/$$mise_pin/stage"; \
			rm -rf "$$mise_stage"; \
			mkdir -p "$$mise_stage" "$$(dirname "$$mise_bootstrap_bin")"; \
			curl --proto '=https' --tlsv1.2 -fsSL -o "$$mise_stage/archive" "$$mise_url"; \
			if command -v sha256sum >/dev/null 2>&1; then \
				echo "$$mise_sha256  $$mise_stage/archive" | sha256sum -c -; \
			else \
				echo "$$mise_sha256  $$mise_stage/archive" | shasum -a 256 -c -; \
			fi; \
			tar -xf "$$mise_stage/archive" -C "$$mise_stage"; \
			mv "$$mise_stage/mise/bin/mise" "$$mise_bootstrap_bin"; \
			chmod +x "$$mise_bootstrap_bin"; \
			rm -rf "$$mise_stage"; \
		fi; \
		mise_receipt="$$("$$mise_bootstrap_bin" --version | cut -d ' ' -f1)"; \
		if [ "$$mise_receipt" != "$$mise_pin" ]; then \
			printf 'ERROR: recovered Mise %s differs from the mise.lock pin %s; delete %s and run make setup\n' "$$mise_receipt" "$$mise_pin" "$$mise_bootstrap_bin" >&2; \
			exit 2; \
		fi; \
	else \
		mise_bootstrap_bin="$$(command -v mise)"; \
		mise_receipt="$$("$$mise_bootstrap_bin" --version | cut -d ' ' -f1)"; \
	fi; \
	export MISE_SHIMS_DIR="$$mise_bootstrap_root/$$mise_receipt/shims"; \
	if [ "$(TOOL_BOOTSTRAP_RESOLVE)" = "1" ]; then \
		if [ "$$mise_lock_usable" = 0 ]; then \
			rm -f "$$mise_lock"; \
		fi; \
		"$$mise_bootstrap_bin" -C "$(PROJECT_ROOT)" lock --upgrade --bump; \
		mise_lock_usable=1; \
	fi; \
	mise_from_lock=; \
	mise_without_lock=; \
	for mise_tool in "python" "github:jdx/mise" "uv" "direnv" "taplo" "aqua:ast-grep/ast-grep" "gitleaks" "aqua:boyter/scc" "node" "github:qltysh/qlty" "github:kucherenko/jscpd" "github:microsoft/waza"; do \
		if [ "$$mise_tool" = "github:jdx/mise" ] && [ -z "$$mise_pin" ]; then \
			continue; \
		fi; \
		if [ "$$mise_lock_usable" = 1 ] && { grep -qxF "[[tools.$$mise_tool]]" "$$mise_lock" || grep -qxF "[[tools.\"$$mise_tool\"]]" "$$mise_lock"; }; then \
			mise_from_lock="$$mise_from_lock $$mise_tool"; \
		else \
			mise_without_lock="$$mise_without_lock $$mise_tool"; \
		fi; \
	done; \
	if [ -n "$$mise_from_lock" ]; then \
		"$$mise_bootstrap_bin" -C "$(PROJECT_ROOT)" install --locked --yes $$mise_from_lock; \
	fi; \
	if [ -n "$$mise_without_lock" ]; then \
		MISE_LOCKFILE=false "$$mise_bootstrap_bin" -C "$(PROJECT_ROOT)" install --yes $$mise_without_lock; \
	fi; \
	if [ "$$mise_lock_usable" = 0 ]; then \
		export MISE_LOCKFILE=false; \
	fi; \
	"$$mise_bootstrap_bin" reshim; \
	printf 'setup: mise %s provisioned\n' "$$mise_receipt"; \
	printf 'setup: entering lifecycle (submodules, environment, hooks) make=%s\n' "$(SELF_MAKE_EXECUTABLE)"; \
	"$$mise_bootstrap_bin" -C "$(PROJECT_ROOT)" exec -- env "PATH=$$(dirname "$$mise_bootstrap_bin"):$${MISE_SHIMS_DIR}:$${PATH}" "CI=$(CI)" $(SELF_MAKE) $(TOOL_BOOTSTRAP_LIFECYCLE)
# The local structural guard runs before any credential or lock owner.
_bootstrap_setup_tools: _builtin_require_workspace
_bootstrap_setup_tools: _builtin_require_upg_lock_owner
_bootstrap_setup_tools: _builtin_require_network_auth

# `upg` writes the lock of the runtime it resolves in. An attached member
# resolves inside its workspace runtime, where `uv lock` rewrites the
# workspace lock and never the member's own, so it stops before any effect.
# The target-specific TOOL_BOOTSTRAP_RESOLVE reaches this prerequisite only
# through `upg`; `setup` passes.
.PHONY: _builtin_require_upg_lock_owner
_builtin_require_upg_lock_owner:
	@if [ "$(TOOL_BOOTSTRAP_RESOLVE)" = "1" ] && [ "$(PROJECT_ROOT)" != "$(RUNTIME_ROOT)" ]; then \
		printf 'ERROR[upg] %s is attached to the workspace %s: `uv lock` here rewrites %s/uv.lock, never %s/uv.lock.\n  Right way: an attached member never resolves its own locks.\n  How: run `make upg` in a linked worktree of this member outside %s (a standalone checkout owns its locks), or `make upg` in %s for the workspace lock.\n' "$(PROJECT_ROOT)" "$(RUNTIME_ROOT)" "$(RUNTIME_ROOT)" "$(PROJECT_ROOT)" "$(RUNTIME_ROOT)" "$(RUNTIME_ROOT)" >&2; \
		exit 2; \
	fi

.PHONY: _builtin_require_network_auth
_builtin_require_network_auth:
	@$(GITHUB_AUTH_DIAGNOSTICS)
	@if [ "$(GITHUB_AUTH_SOURCE)" = gh ]; then \
		if [ "$(GITHUB_AUTH_STATUS)" != 0 ]; then \
			printf 'ERROR: selected gh auth token failed (exit %s); refusing anonymous provisioning\n' '$(GITHUB_AUTH_STATUS)' >&2; \
			exit "$(GITHUB_AUTH_STATUS)"; \
		fi; \
		if [ "$(GITHUB_AUTH_PRESENT)" != yes ]; then \
			printf 'ERROR: selected gh auth token returned no credential; refusing anonymous provisioning\n' >&2; \
			exit 2; \
		fi; \
	fi

# Every repository evaluates only itself, locally exactly as in CI: a workspace
# root consumes its members as installed libraries and never fans a verb out
# across them; each member runs its own lifecycle in its own repository.
# Provisioning is declared once and shared by every profile. Dev environments
# consume present members as LIVE editable installs natively (operator law
# 2026-10-06: a commit never influences dev behavior — the worktree is the
# behavior): [tool.uv.workspace] makes every declared member a workspace
# member and `uv sync --all-packages` provisions them as editables; CI's
# --no-editable keeps the frozen builds.
SETUP_ENVIRONMENT_RECIPE = set -eu; \
	trap 'if [ -n "$${FLEXT_SETUP_CREDENTIAL_STORE:-}" ]; then rm -f "$$FLEXT_SETUP_CREDENTIAL_STORE"; fi' EXIT; \
	$(REQUIRE_WORKSPACE_ENVIRONMENT); \
	credential_env=; \
	if [ -n "$${FLEXT_SETUP_CREDENTIAL_STORE:-}" ]; then \
		credential_env="env GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=credential.helper GIT_CONFIG_VALUE_0=store --file=$$FLEXT_SETUP_CREDENTIAL_STORE"; \
	fi; \
	uv_lock_mode=; \
	if [ -f "$(UV_PROJECT)/uv.lock" ] && ! grep -qE '^(<<<<<<< |=======$$|>>>>>>> )' "$(UV_PROJECT)/uv.lock"; then \
		uv_lock_mode=--frozen; \
	fi; \
	locked_python="$$(mise -C "$(RUNTIME_ROOT)" which python)"; \
	if [ -n "$$uv_lock_mode" ]; then \
		$$credential_env $(UV) sync --project "$(UV_PROJECT)" --python "$$locked_python" $(UV_SYNC_FLAGS) $$uv_lock_mode --link-mode "$(UV_LINK_MODE)"; \
	else \
		$(UV) venv --allow-existing --python "$$locked_python" "$(RUNTIME_VENV)"; \
		uv_groups="$$(awk '/^\[dependency-groups\]/ { inside = 1; next } /^\[/ { inside = 0 } inside && $$2 == "=" { printf " --group $(UV_PROJECT)/pyproject.toml:%s", $$1 }' "$(UV_PROJECT)/pyproject.toml")"; \
		$$credential_env $(UV) pip install --python "$(RUNTIME_VENV)/bin/python" --link-mode "$(UV_LINK_MODE)" --requirements "$(UV_PROJECT)/pyproject.toml" --all-extras $$uv_groups --editable "$(UV_PROJECT)"$(foreach member,$(WORKSPACE_SUBPROJECTS), --editable "$(PROJECT_ROOT)/$(member)"); \
	fi; \
	$(PROJECT_FLEXT_INFRA) workspace sync-environment --repository-root "$(PROJECT_ROOT)"; \
	if [ "$(strip $(CI))" != "Y" ]; then \
		for member in $(WORKSPACE_SUBPROJECTS); do \
			if [ -f "$(PROJECT_ROOT)/$$member/.envrc" ]; then \
				$(PROJECT_FLEXT_INFRA) workspace sync-environment --repository-root "$(PROJECT_ROOT)/$$member"; \
			fi; \
		done; \
	fi

# Reject borrowed environments before bootstrap, activation or custom hooks.
# Members may use their containing workspace, never an unrelated checkout.
REQUIRE_WORKSPACE_ENVIRONMENT = case "$(PROJECT_ROOT)/" in \
	"$(RUNTIME_ROOT)/"*) ;; \
	*) printf 'ERROR: runtime workspace does not contain this project: %s\n' "$(RUNTIME_ROOT)" >&2; exit 2 ;; \
	esac; \
	if [ "$(strip $(CI))" = "Y" ] && [ "$(PROJECT_ROOT)" != "$(RUNTIME_ROOT)" ]; then \
		printf 'ERROR: CI approval is root-only; attached member execution is forbidden\n' >&2; exit 2; \
	fi; \
	for environment_path in "$(patsubst %/,%,$(dir $(RUNTIME_VENV)))" "$(RUNTIME_VENV)" "$(RUNTIME_BIN)" "$(PROJECT_ROOT)/.venv" "$(PROJECT_ROOT)/.venv/bin"; do \
		if [ -L "$$environment_path" ]; then \
			printf 'ERROR: workspace environment must be physical, not a symlink: %s\n' "$$environment_path" >&2; exit 2; \
		fi; \
	done

.PHONY: _builtin_require_workspace
_builtin_require_workspace:
	@$(REQUIRE_WORKSPACE_ENVIRONMENT)

# Execute the interpreter provisioned by setup without discovering a project
# workspace or creating a dependency-resolution file during a runtime command.
override UV_RUN := env -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u PROJECT_ROOT PYTHONPATH="$(PROJECT_ROOT)/src" $(UV) run --directory "$(PROJECT_ROOT)" --no-project --python "$(RUNTIME_PYTHON)"
# The checked-out flext-infra lane owns every lifecycle verb: a workspace
# runs the generator it carries (the submodule src), so a broken published
# dependency tip can never block the local recovery cycle. A checkout without
# the submodule (standalone member) keeps its own installed copy.
FLEXT_INFRA_SUBMODULE_SRC := $(RUNTIME_ROOT)/flext-infra/src
ifeq ($(strip $(CI)),Y)
# Only this candidate's own sources are visible; dependencies stay installed.
override PROJECT_INFRA_PYTHONPATH := $(MAKEFILE_ROOT)/src
else
override PROJECT_INFRA_PYTHONPATH := $(if $(wildcard $(FLEXT_INFRA_SUBMODULE_SRC)/flext_infra/.),$(FLEXT_INFRA_SUBMODULE_SRC),$(MAKEFILE_ROOT)/src)
endif
override PROJECT_INFRA_RUN = if [ ! -x "$(FLEXT_INFRA_PYTHON)" ]; then printf 'ERROR: FLEXT_INFRA_PYTHON must name an executable managed Python\n' >&2; exit 2; fi; env -u PYTHONPATH -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u UV_PROJECT_ENVIRONMENT PYTHONPATH="$(PROJECT_INFRA_PYTHONPATH)" $(FLEXT_INFRA_PYTHON)
override PROJECT_FLEXT_INFRA := $(PROJECT_INFRA_RUN) -m flext_infra
# Scaffold dev tools live in the validated optional dev
# Setup uses the committed uv.lock and never stops on it
# (operator-ruling-2026-10-09-setup-resilient): a present, readable lock always
# installs `--frozen` (exactly its pins, current or stale, never rewritten).
# Without a readable lock (missing, or merge markers uv cannot parse) uv pip
# installs the project and its declared groups from the manifests into the
# environment, never reading or writing uv.lock. `make audit` holds the
# verdict (law 14) and `make upg` cures it.
UV_SYNC_FLAGS := --all-extras --all-groups --all-packages
ifeq ($(strip $(CI)),Y)
override UV_SYNC_FLAGS := --all-extras --all-groups --all-packages --no-editable
endif

ifeq ($(GEN_INIT_ONLY),)
ifneq ($(strip $(CI)),Y)
-include custom.mk
endif
endif
ifeq ($(APPROVAL_CONTEXT),Y)
override CI := Y
export CI
override TOOL_BOOTSTRAP_LIFECYCLE := _setup_lifecycle
endif
override SELF_MAKE := "$(SELF_MAKE_EXECUTABLE)" --no-print-directory -f "$(SELF_MAKEFILE)"
# Digest of the Makefile this make parsed; `upg` compares it after `gen`
# publishes to decide whether the remaining recipe may still name its targets.
override SELF_MAKEFILE_DIGEST := $(shell sha256sum "$(SELF_MAKEFILE)")

define RUN_PUBLIC_POST
	$(if $(filter post-$(1),$(CUSTOM_DECLARED_TARGETS)),+@$(SELF_MAKE) post-$(1))
endef

# A public verb is its producer half (pre hook plus handler) followed by its
# activation half. An activation producer re-enters the environment its
# producer just rendered; `upg` runs the two halves apart so the toolchain
# lock is resolved from that rendered manifest before activation demands it.
define RUN_PUBLIC_PRODUCE
	$(if $(filter pre-$(1),$(CUSTOM_DECLARED_TARGETS)),+@$(SELF_MAKE) pre-$(1))
	$(if $(filter _custom-$(1),$(CUSTOM_DECLARED_TARGETS)),+@$(SELF_MAKE) _custom-$(1),+@$(SELF_MAKE) _builtin-$(1))
endef

# Activation follows the same context rule as every public verb: CI runs the
# activated target directly in its provisioned environment; elsewhere direnv
# activates the checkout first.
define RUN_PUBLIC_ACTIVATE
	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-$(1),direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-$(1))
endef

define RUN_PUBLIC
$(call RUN_PUBLIC_PRODUCE,$(1))
	$(if $(2),$(call RUN_PUBLIC_ACTIVATE,$(1)),$(call RUN_PUBLIC_POST,$(1)))
endef



# `make upg` is the only verb that writes uv.lock (`uv lock --upgrade
# --refresh`, then `uv lock` of the manifest `gen` projected and `uv lock
# --check`). Setup never writes it (lock law above).

.PHONY: $(PUBLIC_VERBS) $(addprefix _builtin-,$(PUBLIC_VERBS))
.PHONY: _builtin_gen_init _builtin_gen_all

# OPTIONS=Y (or HELP=Y) displays a built-in verb's contract without effects:
# no prerequisite, hook or handler of that verb runs. A script verb keeps its
# entry, and the promoted dispatcher renders its contract per WHAT.
VERB_CONTRACT := $(filter 1 TRUE Y YES true y yes True Yes,$(OPTIONS) $(HELP))
ifeq ($(VERB_CONTRACT),)



help:

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-help,$(call RUN_PUBLIC,help))




pre-commit: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-pre-commit,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-pre-commit)

.PHONY: _activated-pre-commit
_activated-pre-commit: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-pre-commit,$(call RUN_PUBLIC,pre-commit))




build: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-build,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-build)

.PHONY: _activated-build
_activated-build: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-build,$(call RUN_PUBLIC,build))




check: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-check,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-check)

.PHONY: _activated-check
_activated-check: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-check,$(call RUN_PUBLIC,check))




smells: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-smells,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-smells)

.PHONY: _activated-smells
_activated-smells: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-smells,$(call RUN_PUBLIC,smells))




test: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-test,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-test)

.PHONY: _activated-test
_activated-test: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-test,$(call RUN_PUBLIC,test))




test-full: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-test-full,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-test-full)

.PHONY: _activated-test-full
_activated-test-full: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-test-full,$(call RUN_PUBLIC,test-full))




test-file: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-test-file,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-test-file)

.PHONY: _activated-test-file
_activated-test-file: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-test-file,$(call RUN_PUBLIC,test-file))




file-gate: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-file-gate,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-file-gate)

.PHONY: _activated-file-gate
_activated-file-gate: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-file-gate,$(call RUN_PUBLIC,file-gate))




profile-test: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-profile-test,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-profile-test)

.PHONY: _activated-profile-test
_activated-profile-test: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-profile-test,$(call RUN_PUBLIC,profile-test))




profile-test-report: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-profile-test-report,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-profile-test-report)

.PHONY: _activated-profile-test-report
_activated-profile-test-report: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-profile-test-report,$(call RUN_PUBLIC,profile-test-report))




fmt: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-fmt,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-fmt)

.PHONY: _activated-fmt
_activated-fmt: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-fmt,$(call RUN_PUBLIC,fmt))




fix: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-fix,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-fix)

.PHONY: _activated-fix
_activated-fix: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-fix,$(call RUN_PUBLIC,fix))




fix-namespace: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-fix-namespace,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-fix-namespace)

.PHONY: _activated-fix-namespace
_activated-fix-namespace: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-fix-namespace,$(call RUN_PUBLIC,fix-namespace))




fix-accessors: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-fix-accessors,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-fix-accessors)

.PHONY: _activated-fix-accessors
_activated-fix-accessors: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-fix-accessors,$(call RUN_PUBLIC,fix-accessors))




audit: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-audit,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-audit)

.PHONY: _activated-audit
_activated-audit: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-audit,$(call RUN_PUBLIC,audit))




status: _builtin_require_workspace

	@$(GITHUB_AUTH_DIAGNOSTICS)

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-status,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-status)

.PHONY: _activated-status
_activated-status: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-status,$(call RUN_PUBLIC,status))




verify-clean: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-verify-clean,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-verify-clean)

.PHONY: _activated-verify-clean
_activated-verify-clean: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-verify-clean,$(call RUN_PUBLIC,verify-clean))




docs: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-docs,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-docs)

.PHONY: _activated-docs
_activated-docs: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-docs,$(call RUN_PUBLIC,docs))




clean:

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-clean,$(call RUN_PUBLIC,clean))




bootstrap-candidate: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-bootstrap-candidate,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-bootstrap-candidate)

.PHONY: _activated-bootstrap-candidate
_activated-bootstrap-candidate: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-bootstrap-candidate,$(call RUN_PUBLIC,bootstrap-candidate))




release-plan: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-release-plan,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-release-plan)

.PHONY: _activated-release-plan
_activated-release-plan: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-release-plan,$(call RUN_PUBLIC,release-plan))




release-version: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-release-version,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-release-version)

.PHONY: _activated-release-version
_activated-release-version: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-release-version,$(call RUN_PUBLIC,release-version))




release-tag: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-release-tag,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-release-tag)

.PHONY: _activated-release-tag
_activated-release-tag: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-release-tag,$(call RUN_PUBLIC,release-tag))




release-build: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-release-build,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-release-build)

.PHONY: _activated-release-build
_activated-release-build: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-release-build,$(call RUN_PUBLIC,release-build))




publication: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-publication,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-publication)

.PHONY: _activated-publication
_activated-publication: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-publication,$(call RUN_PUBLIC,publication))




gen: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-gen,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-gen)

.PHONY: _activated-gen
_activated-gen: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-gen,$(call RUN_PUBLIC,gen))



# The pre hook and selected producer run once before activation. The producer
# owns the complete generation transaction; activation adds no second writer.
gen-footprint: _builtin_require_workspace _builtin_require_environment
	$(call RUN_PUBLIC,gen-footprint,1)

.PHONY: _activated-gen-footprint
_activated-gen-footprint: _builtin_require_environment
	$(call RUN_PUBLIC_POST,gen-footprint)




initialize: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-initialize,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-initialize)

.PHONY: _activated-initialize
_activated-initialize: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-initialize,$(call RUN_PUBLIC,initialize))




mod: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-mod,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-mod)

.PHONY: _activated-mod
_activated-mod: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-mod,$(call RUN_PUBLIC,mod))




mod-text: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-mod-text,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-mod-text)

.PHONY: _activated-mod-text
_activated-mod-text: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-mod-text,$(call RUN_PUBLIC,mod-text))




mod-text-candidate: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-mod-text-candidate,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-mod-text-candidate)

.PHONY: _activated-mod-text-candidate
_activated-mod-text-candidate: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-mod-text-candidate,$(call RUN_PUBLIC,mod-text-candidate))




mod-snapshots: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-mod-snapshots,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-mod-snapshots)

.PHONY: _activated-mod-snapshots
_activated-mod-snapshots: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-mod-snapshots,$(call RUN_PUBLIC,mod-snapshots))




waza: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-waza,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-waza)

.PHONY: _activated-waza
_activated-waza: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-waza,$(call RUN_PUBLIC,waza))




duplication: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-duplication,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-duplication)

.PHONY: _activated-duplication
_activated-duplication: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-duplication,$(call RUN_PUBLIC,duplication))




sonarcloud-sync: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-sonarcloud-sync,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-sonarcloud-sync)

.PHONY: _activated-sonarcloud-sync
_activated-sonarcloud-sync: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-sonarcloud-sync,$(call RUN_PUBLIC,sonarcloud-sync))




sonarcloud-issues: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-sonarcloud-issues,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-sonarcloud-issues)

.PHONY: _activated-sonarcloud-issues
_activated-sonarcloud-issues: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-sonarcloud-issues,$(call RUN_PUBLIC,sonarcloud-issues))



# `setup` keeps its own recipe (it must not require the environment it is about
# to build), but it still runs the pre-/post-setup lifecycle hooks so a project
# declaring them in the custom handler surface is actually honoured.
setup: _bootstrap_setup_tools

# The pre-commit hook approval: only the fast external gates the gate
# registry declares (make.check_gates_pre_commit); no setup, no audit and no
# tests. CI runs the ci workflow rows as its own steps.
_builtin-pre-commit: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "format,lint,markdown,conflict-markers"

# `upg` builds the environment from the locks it writes, so like `setup` it
# must not require an existing environment, and as the only resolver it must
# not require a satisfied committed mise.lock either. Its bootstrap half runs
# `mise lock --bump` first (native resolution; no stage and no prior install),
# then installs from the fresh lock. Only a lock owner resolves: an attached
# member stops in _builtin_require_upg_lock_owner before any lock is written.
upg: TOOL_BOOTSTRAP_LIFECYCLE := _upg_lifecycle
upg: TOOL_BOOTSTRAP_RESOLVE := 1
upg: _bootstrap_setup_tools
else

help:
	@printf '  %-16s %s\n' 'help' 'Show the complete selector-free public interface.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make help to execute it.'

setup:
	@printf '  %-16s %s\n' 'setup' 'Provision the declared environment and hooks.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make setup to execute it.'

pre-commit:
	@printf '  %-16s %s\n' 'pre-commit' 'Approve a commit through the fast external gates the gate registry declares; no setup and no tests (the pre-commit hook entry).'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make pre-commit to execute it.'

upg:
	@printf '  %-16s %s\n' 'upg' 'Resolve the newest declared releases, write the uv and mise locks, then prove the upgraded tree still converges; gates stay with make check.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make upg to execute it.'

build:
	@printf '  %-16s %s\n' 'build' 'Build the project distribution artifacts.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make build to execute it.'

check:
	@printf '  %-16s %s\n' 'check' 'Run the configured non-test gates except the dedicated smells audit.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make check to execute it.'

smells:
	@printf '  %-16s %s\n' 'smells' 'Run the strict code-smell audit as a dedicated gate.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make smells to execute it.'

test:
	@printf '  %-16s %s\n' 'test' 'Run incremental tests through the persistent testmon cache.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make test to execute it.'

test-full:
	@printf '  %-16s %s\n' 'test-full' 'Run incremental then all tests, including external and CI-excluded markers, through the same persistent testmon cache.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make test-full to execute it.'

test-file:
	@printf '  %-16s %s\n' 'test-file' 'Run one declared test file incremental then complete, slow items included, with the same persistent testmon cache (FILE=<repository-relative path>).'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make test-file to execute it.'

file-gate:
	@printf '  %-16s %s\n' 'file-gate' 'Run configured canonical read-only gates on one literal FILE=<repository-relative path>; invalid selection and missing gate owners fail loud.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make file-gate to execute it.'

profile-test:
	@printf '  %-16s %s\n' 'profile-test' 'Profile the canonical pytest entry and its collection children on the same persistent testmon database, without the outer bounded-gate wrapper.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make profile-test to execute it.'

profile-test-report:
	@printf '  %-16s %s\n' 'profile-test-report' 'Render the parent pytest profile and the aggregated child profiles from that run.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make profile-test-report to execute it.'

fmt:
	@printf '  %-16s %s\n' 'fmt' 'Apply ruff format --preview and every declared formatter gate. Ruff is the rule; change code, never ruff.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make fmt to execute it.'

fix:
	@printf '  %-16s %s\n' 'fix' 'Apply the safe fixes of ruff check --fix --preview plus every other configured safe correction; never deletes information. Ruff is the rule; change code, never ruff.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make fix to execute it.'

fix-namespace:
	@printf '  %-16s %s\n' 'fix-namespace' 'Apply the canonical namespace enforcer to the selected workspace; the same relocation cascade runs as a callback phase of make mod.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make fix-namespace to execute it.'

fix-accessors:
	@printf '  %-16s %s\n' 'fix-accessors' 'Migrate accessor names owned by the rename catalog'"'"'s origin package and every resolved consumer; homonyms are skipped with a warning; the same origin-aware rewrite runs as a callback phase of make mod.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make fix-accessors to execute it.'

audit:
	@printf '  %-16s %s\n' 'audit' 'Inspect ownership, dependency, and generated-state health.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make audit to execute it.'

status:
	@printf '  %-16s %s\n' 'status' 'Report the resolved runtime and repository state.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make status to execute it.'

verify-clean:
	@printf '  %-16s %s\n' 'verify-clean' 'Verify managed artifacts and generated documentation against their sources, then reject staged, unstaged, untracked changes and stash entries through the public Git service.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make verify-clean to execute it.'

docs:
	@printf '  %-16s %s\n' 'docs' 'Generate, fix, format, and check documentation.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make docs to execute it.'

clean:
	@printf '  %-16s %s\n' 'clean' 'Remove every declared disposable artifact.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make clean to execute it.'

bootstrap-candidate:
	@printf '  %-16s %s\n' 'bootstrap-candidate' 'Bootstrap declared candidate worktrees with this branch-matched generator.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make bootstrap-candidate to execute it.'

release-plan:
	@printf '  %-16s %s\n' 'release-plan' 'Resolve the release decision through the public protocol.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make release-plan to execute it.'

release-version:
	@printf '  %-16s %s\n' 'release-version' 'Materialize the planned version.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make release-version to execute it.'

release-tag:
	@printf '  %-16s %s\n' 'release-tag' 'Tag the verified release commit.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make release-tag to execute it.'

release-build:
	@printf '  %-16s %s\n' 'release-build' 'Build the release receipt and artifacts.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make release-build to execute it.'

publication:
	@printf '  %-16s %s\n' 'publication' 'Publish only receipt-attested release artifacts.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make publication to execute it.'

gen:
	@printf '  %-16s %s\n' 'gen' 'Regenerate every managed projection atomically.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make gen to execute it.'

gen-footprint:
	@printf '  %-16s %s\n' 'gen-footprint' 'Inspect the pending generation journal and declared physical effect footprint without leases, recovery, or publication.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make gen-footprint to execute it.'

initialize:
	@printf '  %-16s %s\n' 'initialize' 'Materialize the declared package initializer graph.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make initialize to execute it.'

mod:
	@printf '  %-16s %s\n' 'mod' 'Apply the declared structural codemods; committed rule-test snapshots are verified, never rewritten.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make mod to execute it.'

mod-text:
	@printf '  %-16s %s\n' 'mod-text' 'Apply the declared Sed text rules with exact receipts and guarded publication.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make mod-text to execute it.'

mod-text-candidate:
	@printf '  %-16s %s\n' 'mod-text-candidate' 'Apply declared Sed text rules to the configured candidate workspace.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make mod-text-candidate to execute it.'

mod-snapshots:
	@printf '  %-16s %s\n' 'mod-snapshots' 'Regenerate the owned ast-grep rule-test snapshots from their tests for a reviewed commit.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make mod-snapshots to execute it.'

waza:
	@printf '  %-16s %s\n' 'waza' 'Validate provider-neutral governance semantics with Waza.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make waza to execute it.'

duplication:
	@printf '  %-16s %s\n' 'duplication' 'Run the canonical jscpd duplicate-code gate.'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make duplication to execute it.'

sonarcloud-sync:
	@printf '  %-16s %s\n' 'sonarcloud-sync' 'Write the SSOT SonarCloud issue exclusions to the server-side project settings (requires SONAR_TOKEN).'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make sonarcloud-sync to execute it.'

sonarcloud-issues:
	@printf '  %-16s %s\n' 'sonarcloud-issues' 'Read unresolved new-code SonarCloud issues on the published integration branch (requires SONAR_TOKEN).'
	@printf '%s\n' 'OPTIONS=Y displays this contract without effects; run make sonarcloud-issues to execute it.'

endif



rust: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-rust,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-rust)

.PHONY: _activated-rust
_activated-rust: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-rust,$(call RUN_PUBLIC,rust))




validate: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-validate,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-validate)

.PHONY: _activated-validate
_activated-validate: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-validate,$(call RUN_PUBLIC,validate))




guard: _builtin_require_workspace

	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _activated-guard,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-guard)

.PHONY: _activated-guard
_activated-guard: _builtin_require_environment

	$(if $(filter Y,$(CI)),+@$(SELF_MAKE) _builtin-guard,$(call RUN_PUBLIC,guard))



.PHONY: _setup_lifecycle
_setup_lifecycle:
	@set -eu; \
	case "$(strip $(CI)): $(CUSTOM_DECLARED_TARGETS) " in \
		Y:*) ;; \
		*" pre-setup "*) $(SELF_MAKE) pre-setup ;; \
	esac
	@$(SELF_MAKE) _builtin_setup_environment
	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _setup_activated,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _setup_activated)

# Setup only provisions, in every context, CI included: whatever the committed
# locks pin is installed (a drifted uv.lock installs --frozen; a mise.lock
# without the self-managed release provisions through the host Mise) and a
# lock defect is reported with its cure, never a stop
# (operator-ruling-2026-10-09-setup-resilient). The verdicts - lock drift, lock
# integrity, the toolchain reality proof - belong to `make audit`, which CI runs
# right after setup; `make upg` cures them and never depends on setup.
.PHONY: _setup_activated
_setup_activated:
	@set -eu; \
	case "$(strip $(CI)): $(CUSTOM_DECLARED_TARGETS) " in \
		Y:*) ;; \
		*" post-setup "*) $(SELF_MAKE) post-setup ;; \
	esac

_builtin-help:
	@printf '%s\n' 'mcb-scripts [standalone]' '';

	@printf '  %-16s %s\n' 'help' 'Show the complete selector-free public interface.';

	@printf '  %-16s %s\n' 'setup' 'Provision the declared environment and hooks.';

	@printf '  %-16s %s\n' 'pre-commit' 'Approve a commit through the fast external gates the gate registry declares; no setup and no tests (the pre-commit hook entry).';

	@printf '  %-16s %s\n' 'upg' 'Resolve the newest declared releases, write the uv and mise locks, then prove the upgraded tree still converges; gates stay with make check.';

	@printf '  %-16s %s\n' 'build' 'Build the project distribution artifacts.';

	@printf '  %-16s %s\n' 'check' 'Run the configured non-test gates except the dedicated smells audit.';

	@printf '  %-16s %s\n' 'smells' 'Run the strict code-smell audit as a dedicated gate.';

	@printf '  %-16s %s\n' 'test' 'Run incremental tests through the persistent testmon cache.';

	@printf '  %-16s %s\n' 'test-full' 'Run incremental then all tests, including external and CI-excluded markers, through the same persistent testmon cache.';

	@printf '  %-16s %s\n' 'test-file' 'Run one declared test file incremental then complete, slow items included, with the same persistent testmon cache (FILE=<repository-relative path>).';

	@printf '  %-16s %s\n' 'file-gate' 'Run configured canonical read-only gates on one literal FILE=<repository-relative path>; invalid selection and missing gate owners fail loud.';

	@printf '  %-16s %s\n' 'profile-test' 'Profile the canonical pytest entry and its collection children on the same persistent testmon database, without the outer bounded-gate wrapper.';

	@printf '  %-16s %s\n' 'profile-test-report' 'Render the parent pytest profile and the aggregated child profiles from that run.';

	@printf '  %-16s %s\n' 'fmt' 'Apply ruff format --preview and every declared formatter gate. Ruff is the rule; change code, never ruff.';

	@printf '  %-16s %s\n' 'fix' 'Apply the safe fixes of ruff check --fix --preview plus every other configured safe correction; never deletes information. Ruff is the rule; change code, never ruff.';

	@printf '  %-16s %s\n' 'fix-namespace' 'Apply the canonical namespace enforcer to the selected workspace; the same relocation cascade runs as a callback phase of make mod.';

	@printf '  %-16s %s\n' 'fix-accessors' 'Migrate accessor names owned by the rename catalog'"'"'s origin package and every resolved consumer; homonyms are skipped with a warning; the same origin-aware rewrite runs as a callback phase of make mod.';

	@printf '  %-16s %s\n' 'audit' 'Inspect ownership, dependency, and generated-state health.';

	@printf '  %-16s %s\n' 'status' 'Report the resolved runtime and repository state.';

	@printf '  %-16s %s\n' 'verify-clean' 'Verify managed artifacts and generated documentation against their sources, then reject staged, unstaged, untracked changes and stash entries through the public Git service.';

	@printf '  %-16s %s\n' 'docs' 'Generate, fix, format, and check documentation.';

	@printf '  %-16s %s\n' 'clean' 'Remove every declared disposable artifact.';

	@printf '  %-16s %s\n' 'bootstrap-candidate' 'Bootstrap declared candidate worktrees with this branch-matched generator.';

	@printf '  %-16s %s\n' 'release-plan' 'Resolve the release decision through the public protocol.';

	@printf '  %-16s %s\n' 'release-version' 'Materialize the planned version.';

	@printf '  %-16s %s\n' 'release-tag' 'Tag the verified release commit.';

	@printf '  %-16s %s\n' 'release-build' 'Build the release receipt and artifacts.';

	@printf '  %-16s %s\n' 'publication' 'Publish only receipt-attested release artifacts.';

	@printf '  %-16s %s\n' 'gen' 'Regenerate every managed projection atomically.';

	@printf '  %-16s %s\n' 'gen-footprint' 'Inspect the pending generation journal and declared physical effect footprint without leases, recovery, or publication.';

	@printf '  %-16s %s\n' 'initialize' 'Materialize the declared package initializer graph.';

	@printf '  %-16s %s\n' 'mod' 'Apply the declared structural codemods; committed rule-test snapshots are verified, never rewritten.';

	@printf '  %-16s %s\n' 'mod-text' 'Apply the declared Sed text rules with exact receipts and guarded publication.';

	@printf '  %-16s %s\n' 'mod-text-candidate' 'Apply declared Sed text rules to the configured candidate workspace.';

	@printf '  %-16s %s\n' 'mod-snapshots' 'Regenerate the owned ast-grep rule-test snapshots from their tests for a reviewed commit.';

	@printf '  %-16s %s\n' 'waza' 'Validate provider-neutral governance semantics with Waza.';

	@printf '  %-16s %s\n' 'duplication' 'Run the canonical jscpd duplicate-code gate.';

	@printf '  %-16s %s\n' 'sonarcloud-sync' 'Write the SSOT SonarCloud issue exclusions to the server-side project settings (requires SONAR_TOKEN).';

	@printf '  %-16s %s\n' 'sonarcloud-issues' 'Read unresolved new-code SonarCloud issues on the published integration branch (requires SONAR_TOKEN).';

	@printf '  %-16s %s\n' 'rust' 'Run the complete Rust lint and test lifecycle.';

	@printf '  %-16s %s\n' 'validate' 'Run the MCB product structural validator.';

	@printf '  %-16s %s\n' 'guard' 'Validate declared MCB Rust source rules.';


# A project owns the sources declared by its manifest. The generated setup
# reconciler validates every initialized checkout before mutation, initializes
# only missing modules, and preserves declared branches that fix forward beyond
# the recorded gitlink.
.PHONY: _builtin_setup_submodules

# === SECTION: submodule setup (managed) ===
# Source: template (submodule_setup_recipe.j2)
# Computed: workspace uses MANAGED_GITLINKS from config; standalone discovers
#           submodules with flext-managed=true from .gitmodules at runtime.
# Rule: setup PROVISIONS an absent or proven unfinished initial clone and
#       VERIFIES an established checkout. An initial clone has no physical index,
#       no worktree content, and only its clone reflog entry. Established work
#       is never destroyed: git checkout, git reset,
#       fetch, and branch attachment are forbidden. Pin validity is HEAD contains
#       gitlink. Declared branch is the named integration line;
#       legacy branch=. still resolves to the superproject named branch if present.
#       A present checkout may be on its own named change lane. Its branch name is
#       not a safety boundary: exact containment of the recorded gitlink is.
#       Nested gitlinks belong to their own setup.
# Free: no
# End SECTION: submodule setup
# Why: runners expose umask 002 and `submodule update --init`
# materializes tracked files as 0664; canonical Mise artifact gates demand
# exact modes, so provisioning normalizes the umask before checkout.
# An absent gitlink is cloned at depth 1, the same flag private submodule
# init uses. Setup's contract is the recorded commit, and a full history
# cannot finish inside submodule_timeout_seconds when the object database
# is large.
# Derive the physical index from its Git directory: --git-path resolves
# a final symlink and cannot prove that the index entry itself is absent.
_builtin_setup_submodules:
	@set -eu; \
	umask 022; \
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
	if [ "$${GIT_INDEX_FILE+x}" = x ]; then \
		printf 'ERROR: submodule setup cannot authenticate a relocated Git index\n' >&2; \
		exit 2; \
	fi; \
	absent=""; \
	for path in $$managed; do \
		child="$$root/$$path"; \
		if [ ! -e "$$child/.git" ]; then \
			absent="$$absent $$path"; \
			continue; \
		fi; \
		owner=$$(git -C "$$child" rev-parse --show-superproject-working-tree); \
		checkout_root=$$(git -C "$$child" rev-parse --show-toplevel); \
		if [ "$$owner" != "$$root" ] || [ "$$checkout_root" != "$$child" ]; then \
			printf 'ERROR: %s: checkout identity does not match the governed child\n' "$$path" >&2; \
			exit 2; \
		fi; \
		git_dir=$$(git -C "$$child" rev-parse --absolute-git-dir); \
		index="$$git_dir/index"; \
		if [ -e "$$index" ] || [ -L "$$index" ]; then continue; fi; \
		content=$$(git -C "$$child" ls-files --others --directory); \
		reflog=$$(git -C "$$child" reflog show --format=%gs HEAD); \
		initial=$$(git -C "$$child" reflog show --format=%gs -1 HEAD); \
		if [ -n "$$content" ] || [ "$$reflog" != "$$initial" ]; then \
			printf 'ERROR: %s: missing index with unproven initial-clone state; preserve and review\n' "$$path" >&2; \
			exit 2; \
		fi; \
		case "$$initial" in \
			'clone: from '*) ;; \
			*) printf 'ERROR: %s: missing index without initial clone receipt\n' "$$path" >&2; exit 2 ;; \
		esac; \
		local_refs=$$(git -C "$$child" for-each-ref --format='%(refname)' refs/heads refs/stash); \
		if active_ref=$$(git -C "$$child" symbolic-ref -q HEAD); then \
			:; \
		else \
			ref_status=$$?; \
			[ "$$ref_status" -eq 1 ] || exit "$$ref_status"; \
		fi; \
		if [ "$$local_refs" != "$$active_ref" ]; then \
			printf 'ERROR: %s: missing index with local branch or stash work; preserve and review\n' "$$path" >&2; \
			exit 2; \
		fi; \
		entry=$$(git -C "$$root" ls-files --stage -- "$$path"); \
		set -- $$entry; \
		if [ "$$#" -ne 4 ] || [ "$$1" != 160000 ] || [ "$$3" != 0 ] || [ "$$4" != "$$path" ]; then \
			printf 'ERROR: governed path is not a gitlink: %s\n' "$$path" >&2; \
			exit 2; \
		fi; \
		head=$$(git -C "$$child" rev-parse HEAD); \
		if [ "$$head" = "$$2" ]; then \
			printf 'setup: materializing unfinished initial clone at recorded pin: %s\n' "$$path"; \
			GIT_TERMINAL_PROMPT=0 timeout --signal=TERM --kill-after=5s "120s" \
				git -C "$$child" -c submodule.recurse=false checkout --detach --no-overwrite-ignore "$$head"; \
			continue; \
		fi; \
		printf 'setup: resuming unfinished initial clone: %s\n' "$$path"; \
		absent="$$absent $$path"; \
	done; \
	if [ -n "$$absent" ]; then \
		credential_helper='!f() { if [ "$$1" = get ]; then printf "username=x-access-token\npassword=%s\n" "$$GITHUB_TOKEN"; fi; }; f'; \
		GIT_TERMINAL_PROMPT=0 timeout --signal=TERM --kill-after=5s "120s" \
			git -C "$$root" -c credential.helper= \
			-c "credential.https://$${GH_HOST:-github.com}.helper=$$credential_helper" \
			submodule update --init --checkout --depth 1 --jobs "$${FLEXT_SUBMODULE_JOBS:-8}" -- $$absent; \
	fi; \
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
			printf 'ERROR: governed gitlink is not materialized: %s\n' "$$child_path" >&2; \
			exit 2; \
		fi; \
		current=$$(git -C "$$child_root" branch --show-current); \
		head=$$(git -C "$$child_root" rev-parse HEAD); \
		if git -C "$$child_root" merge-base --is-ancestor "$$gitlink" HEAD; then \
			ancestor=Y; \
		else \
			status=$$?; if [ "$$status" -eq 1 ]; then ancestor=N; else exit "$$status"; fi; \
		fi; \
		if [ "$$ancestor" = N ]; then \
			if [ -z "$$current" ]; then \
				printf 'ERROR: %s: detached HEAD %s does not contain recorded gitlink %s; merge the superproject gitlink into this head\n' "$$child_path" "$$head" "$$gitlink" >&2; \
			else \
				printf 'ERROR: %s: checked-out branch %s at %s does not contain recorded gitlink %s; merge the superproject gitlink into this branch\n' "$$child_path" "$$current" "$$head" "$$gitlink" >&2; \
			fi; \
			exit 1; \
		fi; \
	}; \
	for child_path in $$managed; do \
		validate_submodule "$$root" "$$child_path"; \
	done

# A provisioned local setup needs no GitHub credential. Mise and Git receive
# only the explicit process credential above; an operation that actually needs
# network access reports its own failure without a preflight or fallback.

# Every operational verb runs mise exactly as the committed mise.lock pins it:
# the version guard replaces the retired mise.version pin file. Direnv
# activation (or the CI-provisioned PATH) resolves `mise` through the
# mise-managed shims; a stale or absent toolchain fails loud and names the
# native repair verb.
_builtin_require_environment: _builtin_require_workspace
# Documenting the interface (`make help`) must not require the interpreter it
# tells the user how to provision. Only `make help` with no other goal
# skips the check; any combined goal still demands the environment.
ifneq ($(MAKECMDGOALS),help)
	@if [ ! -x "$(RUNTIME_PYTHON)" ]; then \
		printf 'ERROR: missing environment interpreter %s; make setup creates it\n' "$(RUNTIME_PYTHON)" >&2; \
		exit 2; \
	fi
endif

# === SECTION: setup environment (managed) ===
# Source: computed (MAKE_PROFILE routing)
# Setup PROVISIONS tooling only — mise, venv, dependencies.
# It never generates, conforms, or mutates project code; `make gen` is the
# single public conformance/generation surface.
# Setup installs the committed uv.lock and never writes it (lock law: only
# `make upg` writes locks); uv owns the venv: `uv sync --python` creates or
# replaces it against the declared interpreter.
# Governed gitlinks are provisioned in every context, GitHub Actions included:
# the workspace projections (Makefile, pyproject, .gitignore, dependabot, docs)
# derive from the member checkouts, so a member-less CI checkout would render a
# different workspace and break the gen fixed point. Provisioned
# members are read as libraries; no verb gates them from here.
_builtin_setup_environment: $(if $(filter Y,$(CI)),,_builtin_setup_submodules)
	@$(SETUP_ENVIRONMENT_RECIPE)
# End SECTION: setup environment

# `upg` is the only recipe that resolves. Its bootstrap half ran
# `mise lock --bump` on the manifest the generator was provisioned with. This
# lifecycle upgrades uv.lock (which carries the generator itself) and
# provisions the environment frozen from it; then the upgraded generator's
# Mise manifest is rendered (the mise-config surface renders from the
# toolchain owner alone, resolving no tool), locked with `mise lock --bump`,
# installed. That comes before every
# stage that resolves a managed tool (deps modernize, gen), because each one
# authenticates the Mise reader against the lock's self pin, which a consumer
# generated by an earlier generator does not carry yet (C19). One run
# converges. The floors deps modernize writes land in the codegen SSOT and
# gen projects them into every pyproject; the upgraded generator may project
# requirements the first uv resolution never saw, so uv.lock is resolved
# again from the projected pyproject and the environment reinstalled right
# after the producer half, before activation. The convergence fixed point
# must hold before the upgrade publishes.
# Gates are not part of the upgrade: `make check` stays its own verb. Branch-tracked git dependencies are moving sources by
# declaration (workspace.yaml owns the branch): --refresh re-reads their
# metadata so a stale cached requires-dist can never block or skew the
# resolution. Like `setup`, it runs the declared pre-/post-upg lifecycle
# hooks, post-upg inside the activated environment.
# The producer half of `gen` publishes this Makefile. A recipe never names a
# target after that point from a file it did not parse: when publication
# changed the Makefile, the upgrade hands off to a fresh `make upg` that
# parses the new file and converges there (its own producer half is then a
# fixed point); otherwise `_upg_converge`, defined by this same file,
# finishes it. A second change inside the hand-off is a gen fixed-point
# defect and fails loud.
.PHONY: _upg_lifecycle _upg_converge
_upg_lifecycle: _builtin_setup_submodules
	@set -eu; \
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" pre-upg "*) $(SELF_MAKE) pre-upg ;; \
	esac
	@if [ -f "$(PROJECT_ROOT)/uv.lock" ] && grep -qE '^(<<<<<<< |=======$$|>>>>>>> )' "$(PROJECT_ROOT)/uv.lock"; then \
		rm -f "$(PROJECT_ROOT)/uv.lock"; \
	fi
	@$(UV) lock --project "$(PROJECT_ROOT)" --upgrade --refresh
	@$(SELF_MAKE) _builtin_setup_environment
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --what mise-config --scope self --mode apply
	@mise -C "$(PROJECT_ROOT)" lock --bump
	@mise -C "$(PROJECT_ROOT)" install --yes "python" "github:jdx/mise" "uv" "direnv" "taplo" "aqua:ast-grep/ast-grep" "gitleaks" "aqua:boyter/scc" "node" "github:qltysh/qlty" "github:kucherenko/jscpd" "github:microsoft/waza"
	@$(PROJECT_FLEXT_INFRA) deps modernize --repository-root "$(PROJECT_ROOT)" \
		--apply --rewrite-constraints --projects .
	@$(SELF_MAKE) _builtin_require_environment
	$(call RUN_PUBLIC_PRODUCE,gen)
	+@set -eu; \
	published="$$(sha256sum "$(SELF_MAKEFILE)")"; \
	if [ "$$published" = "$(SELF_MAKEFILE_DIGEST)" ]; then \
		$(SELF_MAKE) _upg_converge; \
	elif [ "$(UPG_HANDOFF)" = "Y" ]; then \
		printf 'ERROR: upg: gen republished %s inside the hand-off; gen is not a fixed point\n' "$(SELF_MAKEFILE)" >&2; \
		exit 1; \
	else \
		printf 'upg: gen published a new %s; handing off to make upg on it\n' "$(SELF_MAKEFILE)"; \
		"$(SELF_MAKE_EXECUTABLE)" --no-print-directory -f "$(SELF_MAKEFILE)" upg UPG_HANDOFF=Y; \
	fi

_upg_converge:
	@$(UV) lock --project "$(PROJECT_ROOT)"
	@$(UV) lock --check --project "$(PROJECT_ROOT)"
	@$(SELF_MAKE) _builtin_setup_environment
	$(call RUN_PUBLIC_ACTIVATE,gen)
	@$(SELF_MAKE) gen
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --mode check
	@$(PROJECT_FLEXT_INFRA) deps verify-locks --repository-root "$(PROJECT_ROOT)"
	+@$(if $(filter Y,$(CI)),$(SELF_MAKE) _upg_activated,direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _upg_activated)

.PHONY: _upg_activated
_upg_activated:
	@set -eu; \
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" post-upg "*) $(SELF_MAKE) post-upg ;; \
	esac


# Gate, fix and build verbs act on this repository only, with one body per verb
# in every profile: a workspace root evaluates itself exactly as CI does.
_builtin_build_artifacts:

	@$(UV) build --project "$(PROJECT_ROOT)"


# Check is read-only: it runs the gates without --apply, so the tree is left
# unchanged; fix applies the declared repairs of the fixable gates.
# CI=Y keeps make.check_gates_ci, the strict complement of
# make.check_gates_local; CI=N runs that local partition.
# An absent CI token runs every active default gate.
_builtin_check_all: _builtin_require_environment
	@set -eu; \
		gates="lint,format,security,markdown,markdown-format,markdown-code,duplication,pyrefly,mypy,pyright,conflict-markers,loc-cap,runtime-census,fresh-import,index-declarations,codemod,layout,direnv"; \
		if [ "$(strip $(CI))" = "Y" ]; then \
			gates="lint,format,security,markdown,markdown-format,markdown-code,duplication,pyrefly,conflict-markers,loc-cap,runtime-census,fresh-import,index-declarations,layout"; \
			printf 'INFO: CI=Y runs check gates: lint format security markdown markdown-format markdown-code duplication pyrefly conflict-markers loc-cap runtime-census fresh-import index-declarations layout\n'; \
		elif [ "$(strip $(CI))" = "N" ]; then \
			gates="lint,format,security,markdown,markdown-format,markdown-code,duplication,pyrefly,mypy,pyright,conflict-markers,loc-cap,runtime-census,fresh-import,index-declarations,codemod,layout,direnv"; \
			printf 'INFO: CI=N runs check gates: lint format security markdown markdown-format markdown-code duplication pyrefly mypy pyright conflict-markers loc-cap runtime-census fresh-import index-declarations codemod layout direnv\n'; \
		else \
			printf 'INFO: default context runs check gates: lint format security markdown markdown-format markdown-code duplication pyrefly mypy pyright conflict-markers loc-cap runtime-census fresh-import index-declarations codemod layout direnv\n'; \
		fi; \
		if [ -z "$$gates" ]; then \
			printf 'ERROR: no active check gates remain in the selected context\n' >&2; \
			exit 2; \
		fi; \
		$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "$$gates"

_builtin_test_all: _builtin_require_environment
	@set -eu; \
database="$(FLEXT_PYTEST_TESTMON_DATABASE)"; \
case "$$database" in /*) ;; *) printf 'ERROR: persistent testmon database requires XDG_CACHE_HOME or HOME\n' >&2; exit 2 ;; esac; \
case "$$database" in "$(PROJECT_ROOT)"/*) printf 'ERROR: persistent testmon database must be outside the checkout: %s\n' "$$database" >&2; exit 2 ;; esac; \
case "$$database" in "$${TMPDIR:-/tmp}"/*|/tmp/*) printf 'ERROR: persistent testmon database must not live under the temporary directory: %s\n' "$$database" >&2; exit 2 ;; esac; \
mkdir -p "$$(dirname "$$database")"; \
scratch_root="$(FLEXT_PYTEST_SCRATCH_ROOT)"; \
case "$$scratch_root" in /*) ;; *) printf 'ERROR: pytest scratch root requires HOME: %s\n' "$$scratch_root" >&2; exit 2 ;; esac; \
mkdir -p "$$scratch_root"; \
scratch="$$(mktemp -d "$$scratch_root/pytest.XXXXXX")"; \
trap 'find "$$scratch" -depth -delete' EXIT; \
mkdir -p "$$scratch/tmp"; \
scratch_tmp="$$(cd "$$scratch/tmp" && pwd -P)"; \
TMPDIR="$$scratch_tmp"; TMP="$$scratch_tmp"; TEMP="$$scratch_tmp"; \
export TMPDIR TMP TEMP; \
TESTMON_DATAFILE="$$database" $(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry

_builtin_test_full_all: _builtin_require_environment
	@set -eu; \
database="$(FLEXT_PYTEST_TESTMON_DATABASE)"; \
case "$$database" in /*) ;; *) printf 'ERROR: persistent testmon database requires XDG_CACHE_HOME or HOME\n' >&2; exit 2 ;; esac; \
case "$$database" in "$(PROJECT_ROOT)"/*) printf 'ERROR: persistent testmon database must be outside the checkout: %s\n' "$$database" >&2; exit 2 ;; esac; \
case "$$database" in "$${TMPDIR:-/tmp}"/*|/tmp/*) printf 'ERROR: persistent testmon database must not live under the temporary directory: %s\n' "$$database" >&2; exit 2 ;; esac; \
mkdir -p "$$(dirname "$$database")"; \
scratch_root="$(FLEXT_PYTEST_SCRATCH_ROOT)"; \
case "$$scratch_root" in /*) ;; *) printf 'ERROR: pytest scratch root requires HOME: %s\n' "$$scratch_root" >&2; exit 2 ;; esac; \
mkdir -p "$$scratch_root"; \
scratch="$$(mktemp -d "$$scratch_root/pytest.XXXXXX")"; \
trap 'find "$$scratch" -depth -delete' EXIT; \
mkdir -p "$$scratch/tmp"; \
scratch_tmp="$$(cd "$$scratch/tmp" && pwd -P)"; \
TMPDIR="$$scratch_tmp"; TMP="$$scratch_tmp"; TEMP="$$scratch_tmp"; \
export TMPDIR TMP TEMP; \
TESTMON_DATAFILE="$$database" $(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry full; \
TESTMON_DATAFILE="$$database" $(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry full-slow

_builtin_test_file_all: _builtin_require_environment
	@if [ -z "$(strip $(FILE))" ]; then printf 'ERROR: test-file requires FILE=<repository-relative test file path>\n' >&2; exit 2; fi; \
case "$(FILE)" in /*|*..*) printf 'ERROR: FILE must stay a repository-relative path: %s\n' "$(FILE)" >&2; exit 2 ;; esac; \
if [ ! -f "$(PROJECT_ROOT)/$(FILE)" ]; then printf 'ERROR: FILE is not an existing repository file: %s\n' "$(FILE)" >&2; exit 2; fi; \
export FLEXT_PYTEST_TARGET_FILE="$(FILE)"; \
set -eu; \
database="$(FLEXT_PYTEST_TESTMON_DATABASE)"; \
case "$$database" in /*) ;; *) printf 'ERROR: persistent testmon database requires XDG_CACHE_HOME or HOME\n' >&2; exit 2 ;; esac; \
case "$$database" in "$(PROJECT_ROOT)"/*) printf 'ERROR: persistent testmon database must be outside the checkout: %s\n' "$$database" >&2; exit 2 ;; esac; \
case "$$database" in "$${TMPDIR:-/tmp}"/*|/tmp/*) printf 'ERROR: persistent testmon database must not live under the temporary directory: %s\n' "$$database" >&2; exit 2 ;; esac; \
mkdir -p "$$(dirname "$$database")"; \
scratch_root="$(FLEXT_PYTEST_SCRATCH_ROOT)"; \
case "$$scratch_root" in /*) ;; *) printf 'ERROR: pytest scratch root requires HOME: %s\n' "$$scratch_root" >&2; exit 2 ;; esac; \
mkdir -p "$$scratch_root"; \
scratch="$$(mktemp -d "$$scratch_root/pytest.XXXXXX")"; \
trap 'find "$$scratch" -depth -delete' EXIT; \
mkdir -p "$$scratch/tmp"; \
scratch_tmp="$$(cd "$$scratch/tmp" && pwd -P)"; \
TMPDIR="$$scratch_tmp"; TMP="$$scratch_tmp"; TEMP="$$scratch_tmp"; \
export TMPDIR TMP TEMP; \
TESTMON_DATAFILE="$$database" $(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry file

# Literal-file selection and verdicts belong to the existing canonical checker.
# Export the raw Make value instead of interpolating operator input into shell code.
export FLEXT_FILE_GATE_FILE := $(value FILE)
_builtin_file_gate_all: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "lint,format,pyrefly,mypy,pyright,codemod" --file "$$FLEXT_FILE_GATE_FILE"

_builtin_tests_all: _builtin_require_environment
	+@$(SELF_MAKE) test
	@$(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry full

# fmt is format-only (single-pass verb law): ruff formats Python, the
# fmt_gates formatters run once through the checker's apply mode, and every
# lint repair belongs to `make fix`. Only a real tool failure (exit >= 2)
# breaks the verb; a formatter's residual findings stay reportable and are
# enforced by `make check`.
_builtin_fmt_all: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "format,markdown-format" --apply

_builtin_fix_all: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "lint,markdown,markdown-code" --apply

# SonarCloud server-side issue exclusions (SSOT: codegen.sonarcloud). The verb
# writes an external service with SONAR_TOKEN from the environment; it belongs
# to no setup/gen/check/test workflow row and never runs implicitly.
_builtin_sonarcloud_sync_all: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) maintenance sonarcloud-sync --repository-root "$(PROJECT_ROOT)"

# Read the complete unresolved new-code issue set on the published integration
# branch. This diagnostic never writes SonarCloud settings or issue state.
_builtin_sonarcloud_issues_all: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) maintenance sonarcloud-issues --repository-root "$(PROJECT_ROOT)"


_builtin_run_default: _builtin_require_environment
	@$(UV_RUN) $(PROJECT_NAME) $(ARGS)

# Profile the real runtime-census gate through the same installed CLI selected
# by the root dispatcher. The report stays in this checkout's .reports tree.
.PHONY: profile-census
profile-census: _builtin_require_environment
	@mkdir -p "$(PROFILE_REPORTS_DIR)"
	@$(RUNTIME_PYTHON) -c \
		'import cProfile, sys; from flext_infra.cli import main; profile = cProfile.Profile(); status = profile.runcall(main, sys.argv[2:]); profile.dump_stats(sys.argv[1]); raise SystemExit(status)' \
		"$(PROFILE_REPORTS_DIR)/runtime-census.pstats" check run \
		--repository-root "$(PROJECT_ROOT)" --gates runtime-census

.PHONY: profile-census-report
profile-census-report: _builtin_require_environment
	@$(RUNTIME_PYTHON) -c \
		'import pstats, sys; pstats.Stats(sys.argv[1]).sort_stats("cumtime").print_stats(35)' \
		"$(PROFILE_REPORTS_DIR)/runtime-census.pstats"

# Profile the checker process itself while retaining the canonical Mypy gate,
# its resource limit, native source inventory, and report directory.
.PHONY: profile-mypy
profile-mypy: _builtin_require_environment
	@mkdir -p "$(PROFILE_REPORTS_DIR)"
	@export FLEXT_MYPY_PROFILE_OUTPUT="$(PROFILE_REPORTS_DIR)/mypy.pstats"; \
		$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" \
		--gates mypy

.PHONY: profile-mypy-report
profile-mypy-report: _builtin_require_environment
	@$(RUNTIME_PYTHON) -c \
		'import pstats, sys; pstats.Stats(sys.argv[1]).sort_stats("cumtime").print_stats(50)' \
		"$(PROFILE_REPORTS_DIR)/mypy.pstats"

# Profile the whole read-only generation plan `make gen` executes (every
# governed repository, lazy-init and docs phases) without publishing.
.PHONY: profile-gen
profile-gen: _builtin_require_environment
	@mkdir -p "$(PROFILE_REPORTS_DIR)"
	@$(RUNTIME_PYTHON) -c \
		'import cProfile, sys; from flext_infra.cli import main; profile = cProfile.Profile(); status = profile.runcall(main, sys.argv[2:]); profile.dump_stats(sys.argv[1]); raise SystemExit(status)' \
		"$(PROFILE_REPORTS_DIR)/conform.pstats" codegen conform \
		--root "$(PROJECT_ROOT)" --scope all --mode check

.PHONY: profile-gen-report
profile-gen-report: _builtin_require_environment
	@$(RUNTIME_PYTHON) -c \
		'import pstats, sys; stats = pstats.Stats(sys.argv[1]); stats.sort_stats("cumtime").print_stats(50); stats.sort_stats("tottime").print_callers(15)' \
		"$(PROFILE_REPORTS_DIR)/conform.pstats"

# Profile the cold canonical pytest execution through its thin entrypoint
# for startup diagnosis: the same persistent testmon database
# guard and environment as the bounded gate, but deliberately NOT wrapped in
# PYTEST_BOUNDED. The runner's own deadline still applies. Central collection
# children also write profiles beside their manifests and print their paths.
# The parent adapter starts profiling before runner/model/pytest imports; each
# child runs under a stdlib-only launcher, so pytest imports before any plugin
# package. The runner binds every child profile to the exact run;
# reports never combine a parent profile with the mutable latest.txt pointer.
# Public names come from make.verbs; these targets are the implementations.
_builtin-profile-test: _builtin_require_environment
	@mkdir -p "$(PROFILE_REPORTS_DIR)"
	@set -eu; \
database="$(FLEXT_PYTEST_TESTMON_DATABASE)"; \
case "$$database" in /*) ;; *) printf 'ERROR: persistent testmon database requires XDG_CACHE_HOME or HOME\n' >&2; exit 2 ;; esac; \
case "$$database" in "$(PROJECT_ROOT)"/*) printf 'ERROR: persistent testmon database must be outside the checkout: %s\n' "$$database" >&2; exit 2 ;; esac; \
case "$$database" in "$${TMPDIR:-/tmp}"/*|/tmp/*) printf 'ERROR: persistent testmon database must not live under the temporary directory: %s\n' "$$database" >&2; exit 2 ;; esac; \
mkdir -p "$$(dirname "$$database")"; \
scratch_root="$(FLEXT_PYTEST_SCRATCH_ROOT)"; \
case "$$scratch_root" in /*) ;; *) printf 'ERROR: pytest scratch root requires HOME: %s\n' "$$scratch_root" >&2; exit 2 ;; esac; \
mkdir -p "$$scratch_root"; \
scratch="$$(mktemp -d "$$scratch_root/pytest.XXXXXX")"; \
trap 'find "$$scratch" -depth -delete' EXIT; \
mkdir -p "$$scratch/tmp"; \
scratch_tmp="$$(cd "$$scratch/tmp" && pwd -P)"; \
TMPDIR="$$scratch_tmp"; TMP="$$scratch_tmp"; TEMP="$$scratch_tmp"; \
export TMPDIR TMP TEMP; \
	TESTMON_DATAFILE="$$database" $(RUNTIME_PYTHON) -m flext_infra._pytest_entry profile \
		"$(PROFILE_REPORTS_DIR)/pytest.pstats"

_builtin-profile-test-report: _builtin_require_environment
	@$(RUNTIME_PYTHON) -m flext_infra._cprofile_entry \
		"$(PROFILE_REPORTS_DIR)/pytest.pstats" "$(PROFILE_REPORTS_DIR)/pytest.pstats.json"

_builtin_status_diagnostics: _builtin_require_environment
	@$(GITHUB_AUTH_DIAGNOSTICS)
	@printf 'profile=%s\nproject=%s\nruntime=%s\n' \
		'$(MAKE_PROFILE)' '$(PROJECT_ROOT)' '$(RUNTIME_ROOT)'
	@$(SHELL) -c 'command -v uv'
	@$(UV) --version
	@if [ -x "$(RUNTIME_PYTHON)" ]; then \
		$(UV) pip check --python "$(RUNTIME_VENV)"; \
	fi
	@git -C "$(PROJECT_ROOT)" status --short

# `docs` is a docs-only lifecycle (generate, fix,
# fmt, validate, audit); the actions render from make.docs.actions and never
# depend on the workspace-wide codegen conform — docs actions render their
# own outputs from the live configuration and sources.
_builtin_docs_all:
	@set -eu; \
	for action in $(DOCS_ACTIONS); do \
		mode=; \
		case "$$action" in fix|fmt) mode=--apply ;; esac; \
		$(PROJECT_FLEXT_INFRA) docs "$$action" --repository-root "$(PROJECT_ROOT)" --output-dir ".reports/docs" $$mode --projects .; \
	done

_builtin_clean_generated:

	@find "$(PROJECT_ROOT)" -type d \
		\( -name __pycache__ -o -name .mypy_cache -o -name .pytest_cache -o -name .ruff_cache -o -name .pyrefly_cache -o -name .benchmarks -o -name .hypothesis \) \
		-prune -exec sh -eu -c 'for target do find "$$target" -depth -delete; done' sh {} +


	@set -eu; \
	for target in "$(PROJECT_ROOT)/build" "$(PROJECT_ROOT)/dist" "$(PROJECT_ROOT)/htmlcov" "$(PROJECT_ROOT)/.reports"; do \
		if [ -e "$$target" ]; then find "$$target" -depth -delete; \
		elif [ -L "$$target" ]; then find "$$target" -depth -delete; fi; \
	done


	@set -eu; \
	for target in "$(PROJECT_ROOT)/.coverage" "$(PROJECT_ROOT)/flext-infra-codegen-transaction-journal.json.lock"; do \
		if [ -e "$$target" ]; then rm -- "$$target"; \
		elif [ -L "$$target" ]; then rm -- "$$target"; fi; \
	done


	@find "$(PROJECT_ROOT)" -type f \
		\( -name '*.pstats' \) \
		-delete


# Release protocol. `plan` derives the next version from merged pull-request
# titles and guards against any version change made outside the protocol;
# `version` opens the release pull request; `tag` marks the merged release
# commit; `build` writes the artifact receipt; `publish` uploads exactly what
# the receipt attests (INDEX=Y adds the package index).
_builtin_release_plan: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) release run --phase plan $(if $(strip $(PR_TITLE)),--pr-title "$(PR_TITLE)")

_builtin_release_version: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) release run --phase version --apply

_builtin_release_tag: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) release run --phase tag --apply

_builtin_release_build: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) release run --phase build --apply

_builtin_release_publish: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) release run --phase publish --apply $(if $(filter Y,$(INDEX)),--index)

# Generation has one transaction owner. Conform covers this root and all declared
# members and journals ordinary, Mise, lazy-init, and documentation phases through
# one fixed point. Only `upg` resolves and rewrites the locks; gen installs
# nothing and never runs another writer before or after conform's journal.
_builtin_gen_init:
	@$(PROJECT_FLEXT_INFRA) codegen init --repository-root "$(PROJECT_ROOT)"
	@$(PROJECT_FLEXT_INFRA) codegen init --repository-root "$(PROJECT_ROOT)" --check-only

_builtin_gen_all:
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope all --mode apply

_builtin-gen-footprint:
	@$(PROJECT_FLEXT_INFRA) codegen footprint --root "$(PROJECT_ROOT)" --scope all

_builtin-bootstrap-candidate: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) codegen candidate-bootstrap --repository-root "$(PROJECT_ROOT)"

# Structural rewrites use mod; mod-text replays the same declared text phase
# independently when malformed Python prevents the Rope phases from loading.
# The current directory defines scope; callers never address tools directly.
_builtin_mod_apply: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) refactor mod --repository-root "$(PROJECT_ROOT)" --apply

_builtin_mod_text: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) refactor mod-text --apply

_builtin_mod_text_candidate: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) refactor mod-text-candidate --apply

# `mod` verifies committed rule-test snapshots and never rewrites them; this is
# the one explicit regeneration, whose diff is reviewed and committed.
_builtin_mod_snapshots: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) refactor mod-snapshots --apply

# Namespace and accessor migration are the same selector-free refactor surface
# as `mod`: each public verb owns one fixed rewrite of every resolved consumer.
# The workspace profile sweeps every namespace-enabled project of the topology
# (the root repository and each declared member) in one process: the report
# aggregates per project and one project's findings never stop the sweep. A
# member profile enforces only itself.

_builtin_fix_namespace: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) refactor namespace-enforce --repository-root "$(PROJECT_ROOT)" --projects . --apply


_builtin_fix_accessors: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) refactor accessor-migrate --repository-root "$(PROJECT_ROOT)" --projects . --apply

# Selector-free public verbs map one-to-one to their canonical implementation;
# each implementation owns one fixed operation.
_builtin-build: _builtin_build_artifacts

_builtin-check: _builtin_check_all
_builtin-test: _builtin_test_all
_builtin-test-full: _builtin_test_full_all
_builtin-test-file: _builtin_test_file_all
_builtin-file-gate: _builtin_file_gate_all
_builtin-fmt: _builtin_fmt_all
_builtin-fix: _builtin_fix_all
_builtin-fix-namespace: _builtin_fix_namespace
_builtin-fix-accessors: _builtin_fix_accessors
_builtin-audit:
ifneq ($(CI),Y)
	@$(PROJECT_FLEXT_INFRA) workspace verify-lanes --repo-root "$(PROJECT_ROOT)"
endif
	@set -eu; \
	if ! uv_lock_report=$$($(UV) lock --check --project "$(UV_PROJECT)" 2>&1); then \
		printf 'ERROR[audit] uv.lock does not match the manifests of %s:\n%s\n  Right way: only `make upg` writes uv.lock and cures the drift.\n  How: run `make upg` in %s, then commit uv.lock.\n' "$(UV_PROJECT)" "$$uv_lock_report" "$(PROJECT_ROOT)" >&2; \
		exit 2; \
	fi
	@$(PROJECT_FLEXT_INFRA) deps verify-locks --repository-root "$(PROJECT_ROOT)"
	@$(PROJECT_FLEXT_INFRA) codegen mise-proof --repository-root "$(PROJECT_ROOT)" --uv-executable "$$(mise which uv)"
	@$(UV) pip check --python "$(RUNTIME_VENV)"
	@$(if $(filter Y,$(CI)),$(PROJECT_FLEXT_INFRA) workspace verify-environment --repository-root "$(PROJECT_ROOT)",:)
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope self --mode check
_builtin-status: _builtin_status_diagnostics
_builtin-verify-clean: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --mode check
	@$(PROJECT_FLEXT_INFRA) docs audit --repository-root "$(PROJECT_ROOT)" --output-dir ".reports/docs" --projects .
	@$(PROJECT_FLEXT_INFRA) workspace verify-clean --repo-root "$(PROJECT_ROOT)"
_builtin-docs: _builtin_docs_all
_builtin-clean: _builtin_clean_generated
_builtin-release-plan: _builtin_release_plan
_builtin-release-version: _builtin_release_version
_builtin-release-tag: _builtin_release_tag
_builtin-release-build: _builtin_release_build
_builtin-publication: _builtin_release_publish
_builtin-gen: _builtin_gen_all
_builtin-initialize: _builtin_gen_init
_builtin-mod: _builtin_mod_apply
_builtin-mod-text: _builtin_mod_text
_builtin-mod-text-candidate: _builtin_mod_text_candidate
_builtin-mod-snapshots: _builtin_mod_snapshots
_builtin-waza:
	@cd "$(PROJECT_ROOT)" && waza check --no-update-check

_builtin-smells:
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "smells"

_builtin-duplication:
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "duplication"
_builtin-sonarcloud-sync: _builtin_sonarcloud_sync_all
_builtin-sonarcloud-issues: _builtin_sonarcloud_issues_all
