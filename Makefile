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

SHELL := /bin/sh
# GNU MAKE_COMMAND may be a bare name. Resolve it before changing PATH so
# recursive lifecycle calls keep this invoker instead of selecting a Mise shim.
ifneq ($(filter /%,$(MAKE_COMMAND)),)
SELF_MAKE_EXECUTABLE := $(MAKE_COMMAND)
else
SELF_MAKE_EXECUTABLE := $(shell command -v "$(MAKE_COMMAND)")
ifneq ($(.SHELLSTATUS),0)
$(error Cannot resolve current Make executable: $(MAKE_COMMAND))
endif
endif
SELF_MAKE_EXECUTABLE := $(realpath $(SELF_MAKE_EXECUTABLE))
ifeq ($(strip $(SELF_MAKE_EXECUTABLE)),)
$(error Current Make executable has no physical path: $(MAKE_COMMAND))
endif
.DEFAULT_GOAL := help
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

# GITHUB_TOKEN is the one GitHub credential variable every tool reads (mise,
# gh, uv). The caller's environment supplies it (ai-hub propagates it through
# .envrc.ai-hub); when it carries none, the network bootstrap selects the first
# declared toolchain.github_credential_commands entry whose executable is on
# PATH, and that source must deliver. With no source present, GitHub access is
# anonymous. A tool-scoped alias of the same credential never reaches a recipe,
# where it would shadow or outrank it. The value is never printed.
export GITHUB_TOKEN
unexport GH_TOKEN MISE_GITHUB_TOKEN GITHUB_API_TOKEN

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
WORKSPACE_EDITABLES := $(PROJECT_NAME):.
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
PYTEST_BOUNDED = timeout --signal=TERM --kill-after=5s "$(PYTEST_PROCESS_TIMEOUT_SECONDS)s"
PYTEST_REPORTS_DIR := .reports/tests
PYTEST_CACHE_HOME = $(if $(strip $(XDG_CACHE_HOME)),$(XDG_CACHE_HOME),$(if $(strip $(HOME)),$(HOME)/.cache,))
# One persistent testmon database per project, shared by every checkout and
# worktree of it (like the Mypy cache): testmon tracks each checkout's source by
# file checksum, so a new lane starts from the project's measured selection
# instead of a cold full inventory.
override FLEXT_PYTEST_TESTMON_DATABASE = $(if $(strip $(PYTEST_CACHE_HOME)),$(PYTEST_CACHE_HOME)/flext/infra/testmon/$(PROJECT_NAME)/.testmondata)
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
ifeq ($(OS),Windows_NT)
override TRACKED_MISE := $(PROJECT_ROOT)/bin/mise.cmd
else
override TRACKED_MISE := $(PROJECT_ROOT)/bin/mise
endif
override SETUP_MISE := $(TRACKED_MISE)
override export FLEXT_PYTEST_TARGET_RAW := tests
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
PUBLIC_VERBS := help setup upg build check smells test test-full fmt fix fix-namespace fix-accessors audit status verify-clean docs clean bootstrap-candidate release-plan release-version release-tag release-build publication gen initialize mod mod-text mod-text-candidate mod-snapshots waza duplication sonarcloud-sync gitops guard rust validate
BUILTIN_VERBS := help setup upg build check smells test test-full fmt fix fix-namespace fix-accessors audit status verify-clean docs clean bootstrap-candidate release-plan release-version release-tag release-build publication gen initialize mod mod-text mod-text-candidate mod-snapshots waza duplication sonarcloud-sync
SCRIPT_VERBS := gitops guard rust validate

SCRIPT_ROOTS :=scripts

CUSTOM_MAKEFILE := $(MAKEFILE_ROOT)/custom.mk
CUSTOM_DECLARED_TARGETS :=
ifneq ($(wildcard $(CUSTOM_MAKEFILE)),)
CUSTOM_DECLARED_TARGETS := $(shell awk '/^(_custom-[a-z][a-z0-9-]*|(pre|post)-[a-z][a-z0-9-]*):/ { target=$$1; sub(/:.*/, "", target); if (!seen[target]++) printf "%s ", target }' "$(CUSTOM_MAKEFILE)")
ifneq ($(.SHELLSTATUS),0)
$(error Failed to inspect custom Make targets in $(CUSTOM_MAKEFILE))
endif
endif
DOCS_ACTIONS := generate fix fmt validate audit
 # End SECTION: verb dispatch

# === SECTION: lint/type paths (managed) ===
# Source: template + computed (script_dispatch conditional)
RUFF_PATHS := $(strip $(foreach d,src tests examples scripts,$(if $(wildcard $(PROJECT_ROOT)/$(d)/.),$(PROJECT_ROOT)/$(d),)))
MYPY_PATHS := $(strip $(foreach d,src tests examples scripts,$(if $(wildcard $(PROJECT_ROOT)/$(d)/.),$(PROJECT_ROOT)/$(d),)))
# End SECTION: lint/type paths

# === SECTION: project tool owner (managed) ===
# Source: the repository's declared Mise toolchain, provisioned by make setup.
override UV = $(PROJECT_TOOL_EXEC) uv
CALLER_PATH := $(PATH)
CALLER_VIRTUAL_ENV := $(patsubst %/,%,$(VIRTUAL_ENV))
# End SECTION: project tool owner

# === SECTION: profile routing (managed) ===
# Source: repository topology. workspace has .gitmodules; standalone does not.
# Attached members share their Git superproject runtime; standalone owns itself.
override RUNTIME_ROOT := $(REPOSITORY_ROOT)
override export GIT_CEILING_DIRECTORIES := $(abspath $(RUNTIME_ROOT)/..)
override export MISE_CEILING_PATHS := $(abspath $(RUNTIME_ROOT)/..)
# The physical runtime owns both its environment and frozen tool identities.
# Attached members retain their own lock inputs for standalone consumption.
# The pin is `make upg` output: a generated header, then its release line.
override MISE_VERSION_PIN := $(RUNTIME_ROOT)/mise.version
# The pin is read by the bootstrap recipe after any interrupted lock journal
# has been recovered. Parsing it here would freeze a torn pre-recovery value.
# End SECTION: profile routing

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

# Resolve native tools through the same isolated lock reader used by setup.
# Execute the caller's command with that PATH without reloading host Mise tools.
define PROJECT_TOOL_RUNTIME
set -eu; \
	project_root="$(RUNTIME_ROOT)"; \
	mise="$(SETUP_MISE)"; \
	mise_storage_root="$(MISE_DATA_DIR)"; \
	caller_home="$${HOME:-}"; \
	caller_xdg_data_home="$${XDG_DATA_HOME:-}"; \
	if [ -z "$$caller_xdg_data_home" ] && [ -n "$$caller_home" ]; then \
		caller_xdg_data_home="$$caller_home/.local/share"; \
	fi; \
	caller_path="$$PATH"; \
	mise_trusted_config_paths="$$project_root"; \
mise_lockfile_platforms="linux-x64,linux-x64-musl,linux-arm64,macos-x64,macos-arm64,windows-x64"; \
caller_comspec="$${COMSPEC:-}"; \
caller_pathext="$${PATHEXT:-}"; \
caller_systemroot="$${SYSTEMROOT:-}"; \
caller_windir="$${WINDIR:-}"; \
caller_github_token="$${GITHUB_TOKEN:-}"; \
caller_mise_http_timeout="$${MISE_HTTP_TIMEOUT:-}"; \
caller_flext_mypy_profile_output="$${FLEXT_MYPY_PROFILE_OUTPUT:-}"; \
caller_mise_version="$${MISE_VERSION:-}"; \
mise_pin_file="$(MISE_VERSION_PIN)"; \
	mise_pin=; \
	if [ -f "$$mise_pin_file" ]; then \
		mise_pin=$$(awk '!/^[[:space:]]*(#|$$)/ { lines++; release = $$0 } END { if (lines == 1) print release }' "$$mise_pin_file"); \
		if ! printf '%s\n' "$$mise_pin" | grep -Eq '^[0-9]+(\.[0-9]+){2}$$'; then \
			printf 'ERROR: %s records no resolved Mise release; only make upg writes it (delete a hand-edited pin first)\n' "$$mise_pin_file" >&2; \
			exit 2; \
		fi; \
	elif [ "$(TOOL_BOOTSTRAP_RESOLVE)" != "1" ]; then \
		printf 'ERROR: missing %s; make upg resolves and records the Mise release\n' "$$mise_pin_file" >&2; \
		exit 2; \
	fi; \
	if [ "$(TOOL_BOOTSTRAP_RESOLVE)" != "1" ] && [ -n "$$caller_mise_version" ] && [ "$${caller_mise_version#v}" != "$$mise_pin" ]; then \
		printf 'ERROR: MISE_VERSION=%s conflicts with %s=%s\n' "$$caller_mise_version" "$$mise_pin_file" "$$mise_pin" >&2; \
		exit 2; \
	fi; \
	caller_mise_version="$$mise_pin"; \
	if [ -z "$$mise_storage_root" ]; then \
		if [ -n "$$caller_xdg_data_home" ]; then \
			mise_storage_root="$$caller_xdg_data_home/mise"; \
		elif [ -n "$$caller_home" ]; then \
			mise_storage_root="$$caller_home/.local/share/mise"; \
		else \
			printf 'ERROR: MISE_DATA_DIR, XDG_DATA_HOME, or HOME must identify persistent Mise storage\n' >&2; \
			exit 2; \
		fi; \
	fi; \
	case "$$mise_storage_root" in \
		/*) ;; \
		*) printf 'ERROR: MISE_DATA_DIR must be absolute: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	case "$$mise_storage_root/" in \
		*'/../'*|*'/./'*|*'//'*) printf 'ERROR: MISE_DATA_DIR must be normalized: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	if [ -L "$$mise_storage_root" ]; then \
		printf 'ERROR: MISE_DATA_DIR must not be a symlink: %s\n' "$$mise_storage_root" >&2; \
		exit 2; \
	fi; \
	umask 077; \
	mkdir -p "$$mise_storage_root"; \
	if [ -L "$$mise_storage_root" ]; then \
		printf 'ERROR: MISE_DATA_DIR became a symlink: %s\n' "$$mise_storage_root" >&2; \
		exit 2; \
	fi; \
	mise_storage_root=$$(cd "$$mise_storage_root" && pwd -P); \
	case "$$mise_storage_root/" in \
		/tmp/|/tmp/*) printf 'ERROR: persistent Mise storage must not live under /tmp: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	project_root=$$(cd "$$project_root" && pwd -P); \
	project_parent=$${project_root%/*}; \
	if [ -z "$$project_parent" ]; then project_parent=/; fi; \
	case "$$mise_storage_root/" in \
		"$$project_root/"*) printf 'ERROR: persistent Mise storage must be outside the checkout: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	case "$$project_root/" in \
		"$$mise_storage_root/"*) printf 'ERROR: persistent Mise storage must not contain the checkout: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	for persistent_path in "$$mise_storage_root" "$$mise_storage_root/cache" "$$mise_storage_root/state" "$$mise_storage_root/installs" "$$mise_storage_root/shims" "$$mise_storage_root/uv-cache" "$$mise_storage_root/bootstrap"; do \
		if [ -L "$$persistent_path" ]; then \
			printf 'ERROR: persistent Mise path must not be a symlink: %s\n' "$$persistent_path" >&2; exit 2; \
		fi; \
		mkdir -p "$$persistent_path"; \
		if [ -L "$$persistent_path" ]; then \
			printf 'ERROR: persistent Mise path became a symlink: %s\n' "$$persistent_path" >&2; exit 2; \
		fi; \
		persistent_physical=$$(cd "$$persistent_path" && pwd -P); \
		case "$$persistent_physical" in \
			"$$mise_storage_root"|"$$mise_storage_root"/*) ;; \
			*) printf 'ERROR: persistent Mise path escaped storage: %s\n' "$$persistent_physical" >&2; exit 2 ;; \
		esac; \
	done; \
	scratch=$$(mktemp -d "$$project_parent/.$${project_root##*/}.mise-bootstrap.XXXXXX"); \
	trap 'if [ -n "$${lock_stage:-}" ] && [ -d "$$lock_stage" ] && [ ! -f "$$lock_stage/transaction.json" ]; then find "$$lock_stage" -depth -delete; fi; find "$$scratch" -depth -delete' EXIT; \
	mkdir -p "$$scratch/home" "$$scratch/home" "$$scratch/appdata" "$$scratch/appdata" "$$scratch/xdg-config" "$$scratch/xdg-data" "$$scratch/xdg-cache" "$$scratch/xdg-state" "$$scratch/config" "$$scratch/tmp" "$$scratch/." "$$scratch/system-config" "$$scratch/system-data" "$$scratch/system-installs" "$$scratch/system-shims" "$$scratch/tmp" "$$scratch/tmp" "$$scratch/tmp"; \
: > "$$scratch/global-config.toml"; chmod 600 "$$scratch/global-config.toml"; \
: > "$$scratch/system-config/config.toml"; chmod 600 "$$scratch/system-config/config.toml"; \
: > "$$scratch/gitconfig"; chmod 600 "$$scratch/gitconfig"; \
: > "$$scratch/netrc"; chmod 600 "$$scratch/netrc"; \
mise_exec() { \
		mise_config_mode="$$1"; shift; \
		case "$$mise_config_mode" in \
			no-config) mise_config_argument='MISE_NO_CONFIG=1' ;; \
			project) mise_config_argument= ;; \
			*) printf 'ERROR: invalid Mise config mode: %s\n' "$$mise_config_mode" >&2; return 2 ;; \
		esac; \
		mise_runtime_path=; \
		if [ -n "$$caller_mise_version" ]; then \
			mise_runtime_path="$$mise_storage_root/bootstrap/mise-$${caller_mise_version#v}"; \
			if [ "$(OS)" = "Windows_NT" ]; then mise_runtime_path="$$mise_runtime_path.exe"; fi; \
		fi; \
		env -i \
'GIT_CONFIG_NOSYSTEM=1' \
'GIT_TERMINAL_PROMPT=0' \
'LANG=C' \
'LC_ALL=C' \
'MISE_SAFE=1' \
'MISE_PARANOID=true' \
'MISE_QUIET=1' \
'MISE_NO_ENV=1' \
'MISE_NO_HOOKS=1' \
'MISE_AUTO_ENV=false' \
'MISE_AUTO_INSTALL=false' \
'MISE_EXEC_AUTO_INSTALL=false' \
'MISE_TASK_RUN_AUTO_INSTALL=false' \
'MISE_AUTO_UPDATE=false' \
'MISE_HTTP_RETRIES=0' \
'MISE_NETRC=false' \
'MISE_NOT_FOUND_AUTO_INSTALL=false' \
'MISE_NOT_FOUND_SYSTEM_FALLBACK=false' \
'MISE_OVERRIDE_CONFIG_FILENAMES=.mise.toml' \
'MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES=none' \
'MISE_GITHUB_GH_CLI_TOKENS=false' \
'MISE_GITHUB_USE_GIT_CREDENTIALS=false' \
'MISE_GITHUB_OAUTH_CLIENT_ID=' \
'MISE_GITHUB_OAUTH_EXPORT_ENV=' \
'MISE_GITHUB_OAUTH_OPEN_BROWSER=false' \
'MISE_LOCKFILE=true' \
'MISE_LOCKED=true' \
'MISE_MINIMUM_RELEASE_AGE=7d' \
$${mise_lockfile_platforms:+"MISE_LOCKFILE_PLATFORMS=$$mise_lockfile_platforms"} \
"HOME=$$scratch/home" \
"USERPROFILE=$$scratch/home" \
"APPDATA=$$scratch/appdata" \
"LOCALAPPDATA=$$scratch/appdata" \
"XDG_CONFIG_HOME=$$scratch/xdg-config" \
"XDG_DATA_HOME=$$scratch/xdg-data" \
"XDG_CACHE_HOME=$$scratch/xdg-cache" \
"XDG_STATE_HOME=$$scratch/xdg-state" \
"NETRC=$$scratch/netrc" \
"GIT_CONFIG_GLOBAL=$$scratch/gitconfig" \
"MISE_NETRC_FILE=$$scratch/netrc" \
"MISE_GLOBAL_CONFIG_FILE=$$scratch/global-config.toml" \
"MISE_CONFIG_DIR=$$scratch/config" \
"MISE_TMP_DIR=$$scratch/tmp" \
"MISE_GLOBAL_CONFIG_ROOT=$$scratch/." \
"MISE_SYSTEM_CONFIG_DIR=$$scratch/system-config" \
"MISE_SYSTEM_CONFIG_FILE=$$scratch/system-config/config.toml" \
"MISE_SYSTEM_DATA_DIR=$$scratch/system-data" \
"MISE_SYSTEM_INSTALLS_DIR=$$scratch/system-installs" \
"MISE_SYSTEM_SHIMS_DIR=$$scratch/system-shims" \
"TMPDIR=$$scratch/tmp" \
"TMP=$$scratch/tmp" \
"TEMP=$$scratch/tmp" \
"MISE_DATA_DIR=$$mise_storage_root" \
"MISE_CACHE_DIR=$$mise_storage_root/cache" \
"MISE_STATE_DIR=$$mise_storage_root/state" \
"MISE_INSTALLS_DIR=$$mise_storage_root/installs" \
"MISE_SHIMS_DIR=$$mise_storage_root/shims" \
"UV_CACHE_DIR=$$mise_storage_root/uv-cache" \
"GIT_CEILING_DIRECTORIES=$$project_parent" \
			"MISE_CEILING_PATHS=$$project_parent" \
			"MISE_TRUSTED_CONFIG_PATHS=$$mise_trusted_config_paths" \
$${caller_path:+"PATH=$$caller_path"} \
$${caller_comspec:+"COMSPEC=$$caller_comspec"} \
$${caller_pathext:+"PATHEXT=$$caller_pathext"} \
$${caller_systemroot:+"SYSTEMROOT=$$caller_systemroot"} \
$${caller_windir:+"WINDIR=$$caller_windir"} \
$${caller_github_token:+"GITHUB_TOKEN=$$caller_github_token"} \
$${caller_mise_http_timeout:+"MISE_HTTP_TIMEOUT=$$caller_mise_http_timeout"} \
$${caller_flext_mypy_profile_output:+"FLEXT_MYPY_PROFILE_OUTPUT=$$caller_flext_mypy_profile_output"} \
$${caller_mise_version:+"MISE_VERSION=$$caller_mise_version"} \
$${mise_config_argument:+"$$mise_config_argument"} \
			$${mise_runtime_path:+"MISE_INSTALL_PATH=$$mise_runtime_path"} \
			"$$@"; \
	}; \
	mise_offline() { \
		mise_offline_mode="$$1"; shift; \
		mise_exec "$$mise_offline_mode" env 'MISE_OFFLINE=true' "$$@"; \
	}; \
	mise_runtime="$$mise_storage_root/bootstrap/mise-$${caller_mise_version#v}"; \
	if [ "$(OS)" = "Windows_NT" ]; then mise_runtime="$$mise_runtime.exe"; fi; \
	if [ ! -x "$$mise_runtime" ]; then \
		printf 'ERROR: missing pinned Mise runtime %s; run make setup\n' "$$mise_runtime" >&2; exit 2; \
	fi; \
	if mise_offline project "$$mise_runtime" -C "$$project_root" bin-paths >"$$scratch/paths" 2>"$$scratch/stderr"; then \
		cat "$$scratch/stderr" >&2; \
	else \
		mise_status=$$?; cat "$$scratch/stderr" >&2; cat "$$scratch/paths" >&2; exit "$$mise_status"; \
	fi; \
	if [ -s "$$scratch/stderr" ]; then \
		printf 'ERROR: Mise emitted diagnostics during runtime selection\n' >&2; exit 2; \
	fi; \
	tool_paths=; \
	while IFS= read -r tool_path; do \
		case "$$tool_path" in /*) ;; *) printf 'ERROR: Mise returned a non-absolute tool path: %s\n' "$$tool_path" >&2; exit 2 ;; esac; \
		if [ ! -d "$$tool_path" ]; then printf 'ERROR: missing tool directory: %s; run make setup\n' "$$tool_path" >&2; exit 2; fi; \
		tool_paths="$${tool_paths:+$$tool_paths:}$$tool_path"; \
	done < "$$scratch/paths"; \
	if [ -z "$$tool_paths" ]; then printf 'ERROR: Mise returned no installed tool paths; run make setup\n' >&2; exit 2; fi; \
	PATH="$(RUNTIME_BIN):$$tool_paths:$$caller_path" "$$@"
endef
PROJECT_TOOL_EXEC = $(SHELL) -c '$(subst ','"'"',$(PROJECT_TOOL_RUNTIME))' --

# One bootstrap serves `setup` (frozen) and `upg` (resolving); the public verb
# selects its lifecycle, Mise release resolution and tool locking through
# target-specific variables.
TOOL_BOOTSTRAP_LIFECYCLE := _setup_lifecycle
TOOL_BOOTSTRAP_RESOLVE :=
TOOL_BOOTSTRAP_LOCK :=
.PHONY: _bootstrap_setup_tools

_bootstrap_setup_tools:
	# The lifecycle invokes recursive make through mise, so preserve jobserver FDs.
	+@set -eu; \
	uv_selector="latest"; \
	if [ ! -f "$(SETUP_MISE)" ]; then \
		printf 'ERROR: missing generated mise launcher: %s; run make gen\n' "$(SETUP_MISE)" >&2; \
		exit 2; \
	fi; \
	project_root="$(PROJECT_ROOT)"; \
	mise="$(SETUP_MISE)"; \
	mise_storage_root="$(MISE_DATA_DIR)"; \
	caller_home="$${HOME:-}"; \
	caller_xdg_data_home="$${XDG_DATA_HOME:-}"; \
	if [ -z "$$caller_xdg_data_home" ] && [ -n "$$caller_home" ]; then \
		caller_xdg_data_home="$$caller_home/.local/share"; \
	fi; \
	caller_path="$$PATH"; \
	mise_trusted_config_paths="$$project_root"; \
mise_lockfile_platforms="linux-x64,linux-x64-musl,linux-arm64,macos-x64,macos-arm64,windows-x64"; \
caller_comspec="$${COMSPEC:-}"; \
caller_pathext="$${PATHEXT:-}"; \
caller_systemroot="$${SYSTEMROOT:-}"; \
caller_windir="$${WINDIR:-}"; \
caller_github_token="$${GITHUB_TOKEN:-}"; \
caller_mise_http_timeout="$${MISE_HTTP_TIMEOUT:-}"; \
caller_flext_mypy_profile_output="$${FLEXT_MYPY_PROFILE_OUTPUT:-}"; \
caller_mise_version="$${MISE_VERSION:-}"; \
mise_pin_file="$(MISE_VERSION_PIN)"; \
	mise_pin=; \
	if [ -f "$$mise_pin_file" ]; then \
		mise_pin=$$(awk '!/^[[:space:]]*(#|$$)/ { lines++; release = $$0 } END { if (lines == 1) print release }' "$$mise_pin_file"); \
		if ! printf '%s\n' "$$mise_pin" | grep -Eq '^[0-9]+(\.[0-9]+){2}$$'; then \
			printf 'ERROR: %s records no resolved Mise release; only make upg writes it (delete a hand-edited pin first)\n' "$$mise_pin_file" >&2; \
			exit 2; \
		fi; \
	elif [ "$(TOOL_BOOTSTRAP_RESOLVE)" != "1" ]; then \
		printf 'ERROR: missing %s; make upg resolves and records the Mise release\n' "$$mise_pin_file" >&2; \
		exit 2; \
	fi; \
	if [ "$(TOOL_BOOTSTRAP_RESOLVE)" != "1" ] && [ -n "$$caller_mise_version" ] && [ "$${caller_mise_version#v}" != "$$mise_pin" ]; then \
		printf 'ERROR: MISE_VERSION=%s conflicts with %s=%s\n' "$$caller_mise_version" "$$mise_pin_file" "$$mise_pin" >&2; \
		exit 2; \
	fi; \
	caller_mise_version="$$mise_pin"; \
	if [ -z "$$mise_storage_root" ]; then \
		if [ -n "$$caller_xdg_data_home" ]; then \
			mise_storage_root="$$caller_xdg_data_home/mise"; \
		elif [ -n "$$caller_home" ]; then \
			mise_storage_root="$$caller_home/.local/share/mise"; \
		else \
			printf 'ERROR: MISE_DATA_DIR, XDG_DATA_HOME, or HOME must identify persistent Mise storage\n' >&2; \
			exit 2; \
		fi; \
	fi; \
	case "$$mise_storage_root" in \
		/*) ;; \
		*) printf 'ERROR: MISE_DATA_DIR must be absolute: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	case "$$mise_storage_root/" in \
		*'/../'*|*'/./'*|*'//'*) printf 'ERROR: MISE_DATA_DIR must be normalized: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	if [ -L "$$mise_storage_root" ]; then \
		printf 'ERROR: MISE_DATA_DIR must not be a symlink: %s\n' "$$mise_storage_root" >&2; \
		exit 2; \
	fi; \
	umask 077; \
	mkdir -p "$$mise_storage_root"; \
	if [ -L "$$mise_storage_root" ]; then \
		printf 'ERROR: MISE_DATA_DIR became a symlink: %s\n' "$$mise_storage_root" >&2; \
		exit 2; \
	fi; \
	mise_storage_root=$$(cd "$$mise_storage_root" && pwd -P); \
	case "$$mise_storage_root/" in \
		/tmp/|/tmp/*) printf 'ERROR: persistent Mise storage must not live under /tmp: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	project_root=$$(cd "$$project_root" && pwd -P); \
	project_parent=$${project_root%/*}; \
	if [ -z "$$project_parent" ]; then project_parent=/; fi; \
	case "$$mise_storage_root/" in \
		"$$project_root/"*) printf 'ERROR: persistent Mise storage must be outside the checkout: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	case "$$project_root/" in \
		"$$mise_storage_root/"*) printf 'ERROR: persistent Mise storage must not contain the checkout: %s\n' "$$mise_storage_root" >&2; exit 2 ;; \
	esac; \
	for persistent_path in "$$mise_storage_root" "$$mise_storage_root/cache" "$$mise_storage_root/state" "$$mise_storage_root/installs" "$$mise_storage_root/shims" "$$mise_storage_root/uv-cache" "$$mise_storage_root/bootstrap"; do \
		if [ -L "$$persistent_path" ]; then \
			printf 'ERROR: persistent Mise path must not be a symlink: %s\n' "$$persistent_path" >&2; exit 2; \
		fi; \
		mkdir -p "$$persistent_path"; \
		if [ -L "$$persistent_path" ]; then \
			printf 'ERROR: persistent Mise path became a symlink: %s\n' "$$persistent_path" >&2; exit 2; \
		fi; \
		persistent_physical=$$(cd "$$persistent_path" && pwd -P); \
		case "$$persistent_physical" in \
			"$$mise_storage_root"|"$$mise_storage_root"/*) ;; \
			*) printf 'ERROR: persistent Mise path escaped storage: %s\n' "$$persistent_physical" >&2; exit 2 ;; \
		esac; \
	done; \
	scratch=$$(mktemp -d "$$project_parent/.$${project_root##*/}.mise-bootstrap.XXXXXX"); \
	trap 'if [ -n "$${lock_stage:-}" ] && [ -d "$$lock_stage" ] && [ ! -f "$$lock_stage/transaction.json" ]; then find "$$lock_stage" -depth -delete; fi; find "$$scratch" -depth -delete' EXIT; \
	mkdir -p "$$scratch/home" "$$scratch/home" "$$scratch/appdata" "$$scratch/appdata" "$$scratch/xdg-config" "$$scratch/xdg-data" "$$scratch/xdg-cache" "$$scratch/xdg-state" "$$scratch/config" "$$scratch/tmp" "$$scratch/." "$$scratch/system-config" "$$scratch/system-data" "$$scratch/system-installs" "$$scratch/system-shims" "$$scratch/tmp" "$$scratch/tmp" "$$scratch/tmp"; \
: > "$$scratch/global-config.toml"; chmod 600 "$$scratch/global-config.toml"; \
: > "$$scratch/system-config/config.toml"; chmod 600 "$$scratch/system-config/config.toml"; \
: > "$$scratch/gitconfig"; chmod 600 "$$scratch/gitconfig"; \
: > "$$scratch/netrc"; chmod 600 "$$scratch/netrc"; \
mise_exec() { \
		mise_config_mode="$$1"; shift; \
		case "$$mise_config_mode" in \
			no-config) mise_config_argument='MISE_NO_CONFIG=1' ;; \
			project) mise_config_argument= ;; \
			*) printf 'ERROR: invalid Mise config mode: %s\n' "$$mise_config_mode" >&2; return 2 ;; \
		esac; \
		mise_runtime_path=; \
		if [ -n "$$caller_mise_version" ]; then \
			mise_runtime_path="$$mise_storage_root/bootstrap/mise-$${caller_mise_version#v}"; \
			if [ "$(OS)" = "Windows_NT" ]; then mise_runtime_path="$$mise_runtime_path.exe"; fi; \
		fi; \
		env -i \
'GIT_CONFIG_NOSYSTEM=1' \
'GIT_TERMINAL_PROMPT=0' \
'LANG=C' \
'LC_ALL=C' \
'MISE_SAFE=1' \
'MISE_PARANOID=true' \
'MISE_QUIET=1' \
'MISE_NO_ENV=1' \
'MISE_NO_HOOKS=1' \
'MISE_AUTO_ENV=false' \
'MISE_AUTO_INSTALL=false' \
'MISE_EXEC_AUTO_INSTALL=false' \
'MISE_TASK_RUN_AUTO_INSTALL=false' \
'MISE_AUTO_UPDATE=false' \
'MISE_HTTP_RETRIES=0' \
'MISE_NETRC=false' \
'MISE_NOT_FOUND_AUTO_INSTALL=false' \
'MISE_NOT_FOUND_SYSTEM_FALLBACK=false' \
'MISE_OVERRIDE_CONFIG_FILENAMES=.mise.toml' \
'MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES=none' \
'MISE_GITHUB_GH_CLI_TOKENS=false' \
'MISE_GITHUB_USE_GIT_CREDENTIALS=false' \
'MISE_GITHUB_OAUTH_CLIENT_ID=' \
'MISE_GITHUB_OAUTH_EXPORT_ENV=' \
'MISE_GITHUB_OAUTH_OPEN_BROWSER=false' \
'MISE_LOCKFILE=true' \
'MISE_LOCKED=true' \
'MISE_MINIMUM_RELEASE_AGE=7d' \
$${mise_lockfile_platforms:+"MISE_LOCKFILE_PLATFORMS=$$mise_lockfile_platforms"} \
"HOME=$$scratch/home" \
"USERPROFILE=$$scratch/home" \
"APPDATA=$$scratch/appdata" \
"LOCALAPPDATA=$$scratch/appdata" \
"XDG_CONFIG_HOME=$$scratch/xdg-config" \
"XDG_DATA_HOME=$$scratch/xdg-data" \
"XDG_CACHE_HOME=$$scratch/xdg-cache" \
"XDG_STATE_HOME=$$scratch/xdg-state" \
"NETRC=$$scratch/netrc" \
"GIT_CONFIG_GLOBAL=$$scratch/gitconfig" \
"MISE_NETRC_FILE=$$scratch/netrc" \
"MISE_GLOBAL_CONFIG_FILE=$$scratch/global-config.toml" \
"MISE_CONFIG_DIR=$$scratch/config" \
"MISE_TMP_DIR=$$scratch/tmp" \
"MISE_GLOBAL_CONFIG_ROOT=$$scratch/." \
"MISE_SYSTEM_CONFIG_DIR=$$scratch/system-config" \
"MISE_SYSTEM_CONFIG_FILE=$$scratch/system-config/config.toml" \
"MISE_SYSTEM_DATA_DIR=$$scratch/system-data" \
"MISE_SYSTEM_INSTALLS_DIR=$$scratch/system-installs" \
"MISE_SYSTEM_SHIMS_DIR=$$scratch/system-shims" \
"TMPDIR=$$scratch/tmp" \
"TMP=$$scratch/tmp" \
"TEMP=$$scratch/tmp" \
"MISE_DATA_DIR=$$mise_storage_root" \
"MISE_CACHE_DIR=$$mise_storage_root/cache" \
"MISE_STATE_DIR=$$mise_storage_root/state" \
"MISE_INSTALLS_DIR=$$mise_storage_root/installs" \
"MISE_SHIMS_DIR=$$mise_storage_root/shims" \
"UV_CACHE_DIR=$$mise_storage_root/uv-cache" \
"GIT_CEILING_DIRECTORIES=$$project_parent" \
			"MISE_CEILING_PATHS=$$project_parent" \
			"MISE_TRUSTED_CONFIG_PATHS=$$mise_trusted_config_paths" \
$${caller_path:+"PATH=$$caller_path"} \
$${caller_comspec:+"COMSPEC=$$caller_comspec"} \
$${caller_pathext:+"PATHEXT=$$caller_pathext"} \
$${caller_systemroot:+"SYSTEMROOT=$$caller_systemroot"} \
$${caller_windir:+"WINDIR=$$caller_windir"} \
$${caller_github_token:+"GITHUB_TOKEN=$$caller_github_token"} \
$${caller_mise_http_timeout:+"MISE_HTTP_TIMEOUT=$$caller_mise_http_timeout"} \
$${caller_flext_mypy_profile_output:+"FLEXT_MYPY_PROFILE_OUTPUT=$$caller_flext_mypy_profile_output"} \
$${caller_mise_version:+"MISE_VERSION=$$caller_mise_version"} \
$${mise_config_argument:+"$$mise_config_argument"} \
			$${mise_runtime_path:+"MISE_INSTALL_PATH=$$mise_runtime_path"} \
			"$$@"; \
	}; \
	mise_offline() { \
		mise_offline_mode="$$1"; shift; \
		mise_exec "$$mise_offline_mode" env 'MISE_OFFLINE=true' "$$@"; \
	}; \
if [ -z "$$caller_github_token" ] && command -v gh >/dev/null 2>&1; then \
		caller_github_token="$$(gh auth token)" \
			|| { printf 'ERROR: the selected GitHub credential source failed: %s\n' 'gh auth token' >&2; exit 2; }; \
		if [ -z "$$caller_github_token" ]; then \
			printf 'ERROR: the selected GitHub credential source printed nothing: %s\n' 'gh auth token' >&2; exit 2; \
		fi; \
	fi; \
mise_checked() { \
		mise_log="$$1"; shift; \
		printf 'setup probe: begin stage=%s log=%s\n' "$${mise_log##*/}" "$$mise_log" >&2; \
		if "$$@" >"$$mise_log" 2>&1; then :; \
		else mise_status=$$?; cat "$$mise_log"; printf 'setup probe: failed stage=%s exit=%s\n' "$${mise_log##*/}" "$$mise_status" >&2; return "$$mise_status"; fi; \
		cat "$$mise_log"; \
		if grep -Fq 'mise WARN' "$$mise_log"; then \
			printf 'ERROR: Mise emitted a warning; setup stopped (see %s)\n' "$$mise_log" >&2; return 2; \
		fi; \
		printf 'setup probe: end stage=%s exit=0\n' "$${mise_log##*/}" >&2; \
	}; \
	mise_checked_stdout() { \
		mise_stdout_log="$$1"; mise_stderr_log="$$2"; shift 2; \
		if "$$@" >"$$mise_stdout_log" 2>"$$mise_stderr_log"; then :; \
		else mise_status=$$?; cat "$$mise_stderr_log" >&2; cat "$$mise_stdout_log"; return "$$mise_status"; fi; \
		cat "$$mise_stderr_log" >&2; cat "$$mise_stdout_log"; \
		if grep -Fq 'mise WARN' "$$mise_stderr_log" || grep -Fq 'mise WARN' "$$mise_stdout_log"; then \
			printf 'ERROR: Mise emitted a warning; setup stopped (see %s)\n' "$$mise_stderr_log" >&2; return 2; \
		fi; \
	}; \
	mise_receipt() { \
		mise_receipt_log="$$scratch/$$1"; shift; \
		mise_checked_stdout "$$mise_receipt_log.stdout" "$$mise_receipt_log.stderr" mise_offline no-config "$$1" --version; \
		receipt_output=$$(cat "$$mise_receipt_log.stdout"); \
		receipt_release=$$(printf '%s\n' "$$receipt_output" | grep -E '^(mise )?[0-9]+\.[0-9]+\.[0-9]+$$' | tail -1 | sed 's/^mise //'); \
		if [ -z "$$receipt_release" ]; then \
			receipt_release=$${receipt_output%% *}; \
		fi; \
		if ! printf '%s\n' "$$receipt_release" | grep -Eq '^[0-9]+(\.[0-9]+){2}$$'; then \
			printf 'ERROR: Mise receipt returned invalid version: %s\n' "$$receipt_output" >&2; return 2; \
		fi; \
	}; \
	pinned_mise="$$mise"; \
	mise_receipt runtime-version "$$pinned_mise"; \
	runtime_release="$$receipt_release"; \
	if [ "$(TOOL_BOOTSTRAP_RESOLVE)" = "1" ]; then \
		mise_checked_stdout "$$scratch/resolve.stdout" "$$scratch/resolve.stderr" mise_exec no-config env MISE_CACHE_DIR="$$scratch/resolve-cache" MISE_FETCH_REMOTE_VERSIONS_CACHE=0s "$$pinned_mise" latest github:jdx/mise@2026.9.13; \
		resolved_release=$$(cat "$$scratch/resolve.stdout"); \
		if ! printf '%s\n' "$$resolved_release" | grep -Eq '^[0-9]+(\.[0-9]+){2}$$'; then \
			mise ls-remote github:jdx/mise@2026.9.13 >"$$scratch/lsremote.stdout" 2>"$$scratch/lsremote.stderr" || true; \
			resolved_release=$$(grep -E '^[0-9]+(\.[0-9]+){2}$$' "$$scratch/lsremote.stdout" | tail -1); \
		fi; \
		if ! printf '%s\n' "$$resolved_release" | grep -Eq '^[0-9]+(\.[0-9]+){2}$$'; then \
			printf 'ERROR: mise latest github:jdx/mise@2026.9.13 returned an invalid release: %s\n' "$$resolved_release" >&2; exit 2; \
		fi; \
		caller_mise_version="$$resolved_release"; \
		mise_receipt resolved-version "$$pinned_mise"; \
		if [ "$$receipt_release" != "$$resolved_release" ]; then \
			printf 'ERROR: MISE_VERSION=%s launched Mise %s\n' "$$resolved_release" "$$receipt_release" >&2; exit 2; \
		fi; \
		runtime_release="$$resolved_release"; \
	elif [ "$$runtime_release" != "$$mise_pin" ]; then \
		printf 'ERROR: launched Mise %s differs from the pinned %s\n' "$$runtime_release" "$$mise_pin" >&2; \
		exit 2; \
	fi; \
	caller_mise_version="$$runtime_release"; \
	printf 'mise setup receipt=%s storage=%s\n' "$$runtime_release" "$$mise_storage_root"; \
	# Only ``upg`` locks, once per manifest it provisions from. Lock every \
	# configured tool in one pass so removed selectors cannot survive beside \
	# their replacement in mise.lock. The relock half of ``upg`` (lock without \
	# resolve) skips the lock when the .mise.toml ``gen`` rendered is \
	# byte-identical to the manifest the resolve pass of the same run locked: \
	# the lock is a function of manifest, Mise release and platforms, all \
	# unchanged, so a second resolution would only repeat ~70 GitHub requests. \
	bootstrap_lock="$(TOOL_BOOTSTRAP_LOCK)"; \
	if [ "$$bootstrap_lock" = "1" ] && [ "$(TOOL_BOOTSTRAP_RESOLVE)" != "1" ] \
		&& [ -n "$${UPG_LOCKED_MISE_MANIFEST:-}" ] \
		&& cmp -s "$$UPG_LOCKED_MISE_MANIFEST" "$$project_root/.mise.toml"; then \
		printf 'upg relock: .mise.toml is byte-identical to the manifest this upg locked; installing from the published mise.lock\n'; \
		bootstrap_lock=; \
	fi; \
	locked_manifest=; \
	if [ "$$bootstrap_lock" = "1" ]; then \
		# Stage on the destination filesystem: the bumped lock is resolved and \
		# installed from a private stage, and published by one rename only after \
		# both succeed. A failed or killed run leaves mise.lock untouched; no \
		# backup copy exists. \
		lock_stage="$$(mktemp -d "$$project_parent/.$${project_root##*/}.mise-lock-stage.XXXXXX")"; \
		cp "$$project_root/.mise.toml" "$$lock_stage/.mise.toml"; \
		cp "$$project_root/.mise.toml" "$$scratch/locked-manifest.toml"; \
		# The resolver (TOOL_BOOTSTRAP_RESOLVE, the first half of upg) locks \
		# from the declared manifest alone; every other lock pass bumps the \
		# committed lock and its sidecars. \
		if [ "$(TOOL_BOOTSTRAP_RESOLVE)" != "1" ]; then \
			if [ -f "$$project_root/mise.lock" ]; then cp "$$project_root/mise.lock" "$$lock_stage/mise.lock"; fi; \
			if [ -d "$$project_root/.mise/locks" ]; then mkdir -p "$$lock_stage/.mise"; cp -R "$$project_root/.mise/locks" "$$lock_stage/.mise/locks"; fi; \
		fi; \
		mise_trusted_config_paths="$$lock_stage"; \
		if mise_checked "$$scratch/lock.log" mise_exec project "$$pinned_mise" -C "$$lock_stage" lock --bump; then \
			:; \
		elif grep -q "refusing to replace locked version" "$$scratch/lock.log"; then \
			printf 'INFO: kept the current mise.lock: the newest release was refused for missing platform assets; retry the bump when the release regains full platform coverage\n' >&2; \
		else \
			cat "$$scratch/lock.log" >&2; \
			exit 2; \
		fi; \
		mise_checked "$$scratch/install.log" mise_exec project "$$pinned_mise" -C "$$lock_stage" install --yes; \
		mise_checked "$$scratch/staged-python.log" mise_offline project "$$pinned_mise" -C "$$lock_stage" which python; \
		staged_python=$$(cat "$$scratch/staged-python.log"); \
		if [ ! -x "$$staged_python" ]; then printf 'ERROR: staged Mise Python is not executable: %s\n' "$$staged_python" >&2; exit 2; fi; \
		printf '%s\n' "$$staged_python" > "$$lock_stage/python-path"; \
		if [ "$(TOOL_BOOTSTRAP_RESOLVE)" = "1" ]; then \
			# Prepare and verify the new launchers while the old pin and launchers \
			# still match the committed lock. No failed stage may strand an older \
			# Mise release beside a newer lockfile revision. \
			mkdir -p "$$lock_stage/artifacts/bin"; \
			mise_checked "$$scratch/install-script.log" mise_exec no-config "$$pinned_mise" generate install-script --version "$$runtime_release" --write "$$lock_stage/artifacts/bin/mise" --windows; \
chmod 755 "$$lock_stage/artifacts/bin/mise"; \
chmod 644 "$$lock_stage/artifacts/bin/mise.cmd"; \
mise_receipt launcher-version "$$lock_stage/artifacts/bin/mise"; \
			if [ "$$receipt_release" != "$$runtime_release" ]; then \
				printf 'ERROR: staged Mise launcher runs %s, not %s\n' "$$receipt_release" "$$runtime_release" >&2; exit 2; \
			fi; \
			printf '%s\n' '# @flext-generated: upg' '# @flext-owner: flext-infra/src/flext_infra/templates/project/base/tool_bootstrap_recipe.j2 (toolchain.mise_selector and toolchain.mise_version in flext-infra/config/codegen.yaml)' '# @flext-adjust: never hand-edit; bin/mise and bin/mise.cmd are generated by mise for exactly this release' '# @flext-regenerate: make upg' "$$runtime_release" > "$$lock_stage/artifacts/mise.version"; \
		fi; \
		mise_checked "$$scratch/publish-lock.log" "$$staged_python" "$$project_root/bin/mise-lock-transaction.py" publish "$$project_root" "$$lock_stage"; \
		mise_trusted_config_paths="$$project_root"; \
		if [ "$(TOOL_BOOTSTRAP_RESOLVE)" = "1" ]; then locked_manifest="$$scratch/locked-manifest.toml"; fi; \
	else \
		# ``locked`` mode converges every tool on exactly the version the \
		# committed mise.lock pins. Setup never locks (operator 2026-10-02): \
		# an offline dry-run proves the lock satisfies the manifest, and a lock \
		# that does not (mixed-generation merge, interrupted ``upg``, a lock \
		# written by another Mise release) fails loud; ``make upg`` is its only \
		# writer. Only ``install --yes`` may reach the network, for a tool \
		# missing from the persistent cache. \
		if mise_offline project "$$pinned_mise" -C "$$project_root" install --dry-run >"$$scratch/install-probe.log" 2>&1; then \
			:; \
		else \
			cat "$$scratch/install-probe.log" >&2; \
			printf 'ERROR: mise.lock does not satisfy .mise.toml under Mise %s; run make upg\n' "$$runtime_release" >&2; \
			exit 2; \
		fi; \
		mise_checked "$$scratch/install.log" mise_exec project "$$pinned_mise" -C "$$project_root" install --yes; \
	fi; \
	mise_checked_stdout "$$scratch/ast-grep-version.stdout" "$$scratch/ast-grep-version.stderr" mise_offline project "$$pinned_mise" -C "$$project_root" exec -- ast-grep --version; \
	if [ -s "$$scratch/ast-grep-version.stderr" ]; then \
		printf 'ERROR: ast-grep emitted diagnostics after installation\n' >&2; exit 2; \
	fi; \
	mise_checked "$$scratch/uv-version.log" mise_offline project "$$pinned_mise" -C "$$project_root" exec -- uv --version; \
	uv_output=$$(cat "$$scratch/uv-version.log"); \
	case "$$uv_output" in \
		'uv '*) uv_actual=$${uv_output#uv }; uv_actual=$${uv_actual%% *} ;; \
		*) printf 'ERROR: uv --version returned an invalid value\n' >&2; exit 2 ;; \
	esac; \
	case "$$uv_actual" in \
		''|*[!0-9.]*|.*|*.|*..*) printf 'ERROR: uv --version returned an invalid release: %s\n' "$$uv_actual" >&2; exit 2 ;; \
	esac; \
	old_ifs=$$IFS; IFS=.; set -- $$uv_actual; IFS=$$old_ifs; \
	if [ "$$#" -ne 3 ]; then \
		printf 'ERROR: uv --version returned an invalid release: %s\n' "$$uv_actual" >&2; exit 2; \
	fi; \
	printf 'uv setup selector=%s receipt=%s\n' "$$uv_selector" "$$uv_actual"; \
	mise_checked "$$scratch/direnv-path.log" mise_offline project "$$pinned_mise" -C "$$project_root" which direnv; \
	direnv_executable=$$(cat "$$scratch/direnv-path.log"); \
	if [ ! -x "$$direnv_executable" ]; then \
		printf 'ERROR: Mise resolved a non-executable direnv path: %s\n' "$$direnv_executable" >&2; exit 2; \
	fi; \
	mise_checked "$$scratch/python-path.log" mise_offline project "$$pinned_mise" -C "$$project_root" which python; \
	python_executable=$$(cat "$$scratch/python-path.log"); \
	# CI receives only the shim farm: a project bin/ on PATH would bind every \
	# shim to that repository launcher (mise resolves shims through PATH). \
	if [ -n "$${GITHUB_PATH:-}" ]; then \
printf '%s\n' "$$mise_storage_root/shims" >> "$$GITHUB_PATH"; \
fi; \
	printf 'setup: entering lifecycle (submodules, environment, hooks) make=%s\n' "$(SELF_MAKE_EXECUTABLE)"; \
	mise_runtime_path="$$mise_storage_root/bootstrap/mise-$${runtime_release}"; \
	if [ "$(OS)" = "Windows_NT" ]; then mise_runtime_path="$$mise_runtime_path.exe"; fi; \
	env \
"MISE_DATA_DIR=$$mise_storage_root" \
"MISE_CACHE_DIR=$$mise_storage_root/cache" \
"MISE_STATE_DIR=$$mise_storage_root/state" \
"MISE_INSTALLS_DIR=$$mise_storage_root/installs" \
"MISE_SHIMS_DIR=$$mise_storage_root/shims" \
"UV_CACHE_DIR=$$mise_storage_root/uv-cache" \
"GIT_CEILING_DIRECTORIES=$$project_parent" \
		"MISE_CEILING_PATHS=$$project_parent" \
		"MISE_TRUSTED_CONFIG_PATHS=$$project_root" \
		"MISE_VERSION=$$runtime_release" \
		"MISE_INSTALL_PATH=$$mise_runtime_path" \
		$(PROJECT_TOOL_EXEC) env \
		$${locked_manifest:+"UPG_LOCKED_MISE_MANIFEST=$$locked_manifest"} \
		"SETUP_DIRENV=$$direnv_executable" \
		"SETUP_PYTHON=$$python_executable" \
		"SETUP_DIRENV_XDG_DATA_HOME=$$caller_xdg_data_home" \
		"CI=$(CI)" $(SELF_MAKE) $(TOOL_BOOTSTRAP_LIFECYCLE)

# Every repository evaluates only itself, locally exactly as in CI: a workspace
# root consumes its members as installed libraries and never fans a verb out
# across them; each member runs its own lifecycle in its own repository.
# Provisioning is declared once and shared by every profile. A venv records the
# exact base interpreter used to create it, so setup replaces it when Mise moves
# the configured Python minor line to a newer patch.
SETUP_ENVIRONMENT_RECIPE = set -eu; \
	$(REQUIRE_WORKSPACE_ENVIRONMENT); \
	desired_python="$${SETUP_PYTHON:?missing Mise-resolved Python executable}"; \
	if [ ! -x "$(RUNTIME_PYTHON)" ]; then \
		$(UV) venv --python "$$desired_python" "$(RUNTIME_VENV)"; \
	else \
		installed_base=$$("$(RUNTIME_PYTHON)" -c 'from pathlib import Path; import sys; print(Path(sys.base_prefix).resolve())'); \
		desired_base=$$("$$desired_python" -c 'from pathlib import Path; import sys; print(Path(sys.prefix).resolve())'); \
		if [ "$$installed_base" != "$$desired_base" ]; then \
			printf 'setup: replacing environment for Python %s\n' "$$desired_python"; \
			$(UV) venv --clear --python "$$desired_python" "$(RUNTIME_VENV)"; \
		fi; \
	fi; \
	$(UV) sync --project "$(UV_PROJECT)" $(UV_SYNC_FLAGS) --link-mode "$(UV_LINK_MODE)"; \
	XDG_DATA_HOME="$${SETUP_DIRENV_XDG_DATA_HOME:?missing persistent direnv data home}" \
		"$${SETUP_DIRENV:?missing Mise-resolved direnv executable}" allow "$(PROJECT_ROOT)"; \
	for member in $(WORKSPACE_SUBPROJECTS); do \
		if [ -f "$(PROJECT_ROOT)/$$member/.envrc" ]; then \
			XDG_DATA_HOME="$$SETUP_DIRENV_XDG_DATA_HOME" "$$SETUP_DIRENV" allow "$(PROJECT_ROOT)/$$member"; \
		fi; \
	done

# Reject borrowed environments before bootstrap, activation or custom hooks.
# Members may use their containing workspace, never an unrelated checkout.
REQUIRE_WORKSPACE_ENVIRONMENT = case "$(PROJECT_ROOT)/" in \
	"$(RUNTIME_ROOT)/"*) ;; \
	*) printf 'ERROR: runtime workspace does not contain this project: %s\n' "$(RUNTIME_ROOT)" >&2; exit 2 ;; \
	esac; \
	for environment_path in "$(RUNTIME_VENV)" "$(RUNTIME_BIN)" "$(PROJECT_ROOT)/.venv" "$(PROJECT_ROOT)/.venv/bin"; do \
		if [ -L "$$environment_path" ]; then \
			printf 'ERROR: workspace environment must be physical, not a symlink: %s\n' "$$environment_path" >&2; exit 2; \
		fi; \
	done

.PHONY: _builtin_require_workspace
_builtin_require_workspace:
	@$(REQUIRE_WORKSPACE_ENVIRONMENT)

_bootstrap_setup_tools: _builtin_require_workspace

# Execute the interpreter provisioned by setup without discovering a project
# workspace or creating a dependency-resolution file during a runtime command.
UV_RUN := env -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u PROJECT_ROOT PYTHONPATH="$(PROJECT_ROOT)/src" $(UV) run --no-project --python "$(RUNTIME_PYTHON)"
# The checked-out flext-infra lane owns every lifecycle verb: a workspace
# runs the generator it carries (the submodule src), so a broken published
# dependency tip can never block the local recovery cycle. A checkout without
# the submodule (standalone member) keeps its own installed copy.
FLEXT_INFRA_SUBMODULE_SRC := $(RUNTIME_ROOT)/flext-infra/src
override PROJECT_INFRA_PYTHONPATH := $(if $(wildcard $(FLEXT_INFRA_SUBMODULE_SRC)/flext_infra/.),$(FLEXT_INFRA_SUBMODULE_SRC),$(MAKEFILE_ROOT)/src)
PROJECT_INFRA_RUN = if [ ! -x "$(FLEXT_INFRA_PYTHON)" ]; then printf 'ERROR: FLEXT_INFRA_PYTHON must name an executable managed Python\n' >&2; exit 2; fi; $(PROJECT_TOOL_EXEC) env -u PYTHONPATH -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u UV_PROJECT_ENVIRONMENT PYTHONPATH="$(PROJECT_INFRA_PYTHONPATH)" $(FLEXT_INFRA_PYTHON)
PROJECT_FLEXT_INFRA := $(PROJECT_INFRA_RUN) -m flext_infra
# Scaffold dev tools live in the validated optional dev
# Setup installs frozen from the committed uv.lock and never re-resolves: it is
# the CI path and must be stable. A missing or stale lock fails through uv's own
# error; `make upg` is the only verb that resolves and rewrites it.
UV_SYNC_FLAGS := --all-extras --all-groups --locked

ifeq ($(GEN_INIT_ONLY),)
-include custom.mk
endif
SELF_MAKE := "$(SELF_MAKE_EXECUTABLE)" --no-print-directory -f "$(SELF_MAKEFILE)"

define RUN_PUBLIC_POST
	$(if $(filter post-$(1),$(CUSTOM_DECLARED_TARGETS)),+@$(SELF_MAKE) post-$(1))
endef

define RUN_PUBLIC
	$(if $(filter pre-$(1),$(CUSTOM_DECLARED_TARGETS)),+@$(SELF_MAKE) pre-$(1))
	$(if $(filter _custom-$(1),$(CUSTOM_DECLARED_TARGETS)),+@$(SELF_MAKE) _custom-$(1),+@$(SELF_MAKE) _builtin-$(1))
	$(if $(2),+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-$(1),$(call RUN_PUBLIC_POST,$(1)))
endef


# Script-dispatch routing: a declared ``_custom_<verb>_<what>`` handler wins
# before the allowlist; otherwise the dispatcher resolves <verb>/<WHAT> across
# SCRIPT_ROOTS. Bare verbs reach the dispatcher, which renders verb help.
define _dispatch
	@set -eu; \
	what="$(WHAT)"; \
	custom="_custom_$(1)_$$what"; \
	if [ -n "$$what" ] && $(SELF_MAKE) -n "$$custom" >/dev/null 2>&1; then \
		$(SELF_MAKE) "$$custom"; \
	elif [ -n "$$what" ]; then \
		what_norm="$$(printf '%s' "$$what" | tr '-' '_')"; \
		found=""; \
		for root in $(SCRIPT_ROOTS); do \
			if [ -f "$(PROJECT_ROOT)/$$root/$(1)/$${what_norm}.py" ] || [ -f "$(PROJECT_ROOT)/$$root/$(1)/$${what_norm}.sh" ] || [ -f "$(PROJECT_ROOT)/$$root/$(1)/all.sh" ]; then \
				found="$$root/$(1)"; \
			fi; \
		done; \
		if [ -n "$$found" ]; then \
			WHAT="$$what" $(UV_RUN) python "$(PROJECT_ROOT)/scripts/dispatch.py" "$(1)"; \
		else \
			printf 'ERROR: no script handler for %s WHAT=%s (roots: %s)\n' "$(1)" "$$what" "$(SCRIPT_ROOTS)" >&2; \
			exit 2; \
		fi; \
	else \
		WHAT="" $(UV_RUN) python "$(PROJECT_ROOT)/scripts/dispatch.py" "$(1)"; \
	fi
endef


# uv resolves the containing workspace and writes its single uv.lock. Invoking
# it once per member re-resolves that same lock for every member.
# uv truncates and rewrites uv.lock in place, so a run killed mid-write leaves
# a partial lock. The lock therefore resolves in a scratch mirror
# of the manifests uv itself reports (`uv workspace dir|list`), must pass
# `uv lock --check` against that mirror (every declared member present), and
# only then replaces the committed lock by one rename inside its directory. An
# interrupted run never touches the committed lock. A full upgrade resolves
# from declared manifests without prior lock preferences, including during
# conflict repair; the final non-upgrade relock retains the resolved lock.
# The second argument seeds the mirror with the committed lock: only that
# retaining relock passes it, so a full upgrade never parses a prior lock.
define _lock_project
	@set -eu; \
	workspace=$$($(UV) workspace dir --project "$(PROJECT_ROOT)"); \
	stage=$$(mktemp -d); candidate="$$workspace/.uv.lock.$$$$"; \
	trap 'find "$${stage}" -depth -delete; rm -f "$$candidate"' EXIT; \
	trap 'exit 129' HUP; trap 'exit 130' INT; trap 'exit 143' TERM; \
	$(UV) workspace list --paths --project "$$workspace" > "$${stage}/.members"; \
	while IFS= read -r member; do \
		relative=$${member#"$$workspace"}; \
		mkdir -p "$${stage}/mirror$$relative"; \
		cp "$$member/pyproject.toml" "$${stage}/mirror$$relative/pyproject.toml"; \
	done < "$${stage}/.members"; \
	$(if $(2),if [ -f "$$workspace/uv.lock" ]; then cp "$$workspace/uv.lock" "$${stage}/mirror/uv.lock"; fi;) \
	$(UV) lock --project "$${stage}/mirror" $(1); \
	$(UV) lock --check --project "$${stage}/mirror"; \
	if [ -e "$$candidate" ]; then printf 'ERROR: lock staging path already exists: %s\n' "$$candidate" >&2; exit 2; fi; \
	cp "$${stage}/mirror/uv.lock" "$$candidate"; \
	mv -f "$$candidate" "$$workspace/uv.lock"
endef

.PHONY: $(PUBLIC_VERBS) $(addprefix _builtin-,$(PUBLIC_VERBS))
.PHONY: _builtin_gen_init _builtin_gen_all
$(filter-out help clean upg,$(PUBLIC_VERBS)): _builtin_require_mise_pin




help:

	$(call RUN_PUBLIC,help)




build: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-build

.PHONY: _activated-build
_activated-build: _builtin_require_environment

	$(call RUN_PUBLIC,build)




check: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-check

.PHONY: _activated-check
_activated-check: _builtin_require_environment

	$(call RUN_PUBLIC,check)




smells: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-smells

.PHONY: _activated-smells
_activated-smells: _builtin_require_environment

	$(call RUN_PUBLIC,smells)




test: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-test

.PHONY: _activated-test
_activated-test: _builtin_require_environment

	$(call RUN_PUBLIC,test)




test-full: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-test-full

.PHONY: _activated-test-full
_activated-test-full: _builtin_require_environment

	$(call RUN_PUBLIC,test-full)




fmt: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-fmt

.PHONY: _activated-fmt
_activated-fmt: _builtin_require_environment

	$(call RUN_PUBLIC,fmt)




fix: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-fix

.PHONY: _activated-fix
_activated-fix: _builtin_require_environment

	$(call RUN_PUBLIC,fix)




fix-namespace: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-fix-namespace

.PHONY: _activated-fix-namespace
_activated-fix-namespace: _builtin_require_environment

	$(call RUN_PUBLIC,fix-namespace)




fix-accessors: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-fix-accessors

.PHONY: _activated-fix-accessors
_activated-fix-accessors: _builtin_require_environment

	$(call RUN_PUBLIC,fix-accessors)




audit: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-audit

.PHONY: _activated-audit
_activated-audit: _builtin_require_environment

	$(call RUN_PUBLIC,audit)




status: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-status

.PHONY: _activated-status
_activated-status: _builtin_require_environment

	$(call RUN_PUBLIC,status)




verify-clean: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-verify-clean

.PHONY: _activated-verify-clean
_activated-verify-clean: _builtin_require_environment

	$(call RUN_PUBLIC,verify-clean)




docs: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-docs

.PHONY: _activated-docs
_activated-docs: _builtin_require_environment

	$(call RUN_PUBLIC,docs)




clean:

	$(call RUN_PUBLIC,clean)




bootstrap-candidate: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-bootstrap-candidate

.PHONY: _activated-bootstrap-candidate
_activated-bootstrap-candidate: _builtin_require_environment

	$(call RUN_PUBLIC,bootstrap-candidate)




release-plan: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-release-plan

.PHONY: _activated-release-plan
_activated-release-plan: _builtin_require_environment

	$(call RUN_PUBLIC,release-plan)




release-version: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-release-version

.PHONY: _activated-release-version
_activated-release-version: _builtin_require_environment

	$(call RUN_PUBLIC,release-version)




release-tag: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-release-tag

.PHONY: _activated-release-tag
_activated-release-tag: _builtin_require_environment

	$(call RUN_PUBLIC,release-tag)




release-build: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-release-build

.PHONY: _activated-release-build
_activated-release-build: _builtin_require_environment

	$(call RUN_PUBLIC,release-build)




publication: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-publication

.PHONY: _activated-publication
_activated-publication: _builtin_require_environment

	$(call RUN_PUBLIC,publication)



# The pre hook and selected producer run once before activation. The producer
# owns the complete generation transaction; activation adds no second writer.
gen: _builtin_require_workspace _builtin_require_environment
	$(call RUN_PUBLIC,gen,1)

.PHONY: _activated-gen
_activated-gen: _builtin_require_environment
	$(call RUN_PUBLIC_POST,gen)




initialize: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-initialize

.PHONY: _activated-initialize
_activated-initialize: _builtin_require_environment

	$(call RUN_PUBLIC,initialize)




mod: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-mod

.PHONY: _activated-mod
_activated-mod: _builtin_require_environment

	$(call RUN_PUBLIC,mod)




mod-text: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-mod-text

.PHONY: _activated-mod-text
_activated-mod-text: _builtin_require_environment

	$(call RUN_PUBLIC,mod-text)




mod-text-candidate: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-mod-text-candidate

.PHONY: _activated-mod-text-candidate
_activated-mod-text-candidate: _builtin_require_environment

	$(call RUN_PUBLIC,mod-text-candidate)




mod-snapshots: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-mod-snapshots

.PHONY: _activated-mod-snapshots
_activated-mod-snapshots: _builtin_require_environment

	$(call RUN_PUBLIC,mod-snapshots)




waza: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-waza

.PHONY: _activated-waza
_activated-waza: _builtin_require_environment

	$(call RUN_PUBLIC,waza)




duplication: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-duplication

.PHONY: _activated-duplication
_activated-duplication: _builtin_require_environment

	$(call RUN_PUBLIC,duplication)




sonarcloud-sync: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-sonarcloud-sync

.PHONY: _activated-sonarcloud-sync
_activated-sonarcloud-sync: _builtin_require_environment

	$(call RUN_PUBLIC,sonarcloud-sync)




gitops: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-gitops

.PHONY: _activated-gitops
_activated-gitops: _builtin_require_environment

	$(call RUN_PUBLIC,gitops)




guard: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-guard

.PHONY: _activated-guard
_activated-guard: _builtin_require_environment

	$(call RUN_PUBLIC,guard)




rust: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-rust

.PHONY: _activated-rust
_activated-rust: _builtin_require_environment

	$(call RUN_PUBLIC,rust)




validate: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-validate

.PHONY: _activated-validate
_activated-validate: _builtin_require_environment

	$(call RUN_PUBLIC,validate)



# `setup` keeps its own recipe (it must not require the environment it is about
# to build), but it still runs the pre-/post-setup lifecycle hooks so a project
# declaring them in the custom handler surface is actually honoured.
setup: _bootstrap_setup_tools

# `upg` builds the environment from the locks it writes, so like `setup` it
# must not require an existing environment, and as the only resolver it must
# not require a readable committed mise.lock either. A lock written by a newer
# Mise than the pinned release ("unsupported lockfile version 3") or one that
# no longer covers the manifest would otherwise stop the very verb that
# rewrites it. Its first half therefore resolves the toolchain afresh, in a
# private stage that never reads the committed lock. `_upg_relock` then bumps
# that lock against the .mise.toml `gen` just rendered, so toolchain policy
# changes (for example the fleet cooldown) still reach every consumer; when
# that manifest is byte-identical to the one the first half locked, the lock
# already satisfies it and the relock installs from it without resolving again.
upg: TOOL_BOOTSTRAP_LIFECYCLE := _upg_lifecycle
upg: TOOL_BOOTSTRAP_RESOLVE := 1
upg: TOOL_BOOTSTRAP_LOCK := 1
upg: _builtin_require_runtime_root _bootstrap_setup_tools

# Only the runtime root resolves the Mise release. An attached member's pin and
# launchers are projections of that root, published by the root's `make gen`.
.PHONY: _builtin_require_runtime_root
_builtin_require_runtime_root:
	@if [ "$(PROJECT_ROOT)" != "$(RUNTIME_ROOT)" ]; then \
		printf 'ERROR: %s projects the Mise pin and launchers of %s; run make upg there\n' "$(PROJECT_ROOT)" "$(RUNTIME_ROOT)" >&2; \
		exit 2; \
	fi

.PHONY: _setup_lifecycle
_setup_lifecycle:
	@set -eu; \
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" pre-setup "*) $(SELF_MAKE) pre-setup ;; \
	esac
	@$(SELF_MAKE) _builtin_setup_environment
	+@XDG_DATA_HOME="$${SETUP_DIRENV_XDG_DATA_HOME:?missing persistent direnv data home}" \
		"$${SETUP_DIRENV:?missing Mise-resolved direnv executable}" exec "$(PROJECT_ROOT)" $(SELF_MAKE) _setup_activated

.PHONY: _setup_activated
_setup_activated:
	@set -eu; \
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" post-setup "*) $(SELF_MAKE) post-setup ;; \
	esac

_builtin-help:
	@printf '%s\n' 'mcb-scripts [standalone]' '';

	@printf '  %-16s %s\n' 'help' 'Show the complete selector-free public interface.';

	@printf '  %-16s %s\n' 'setup' 'Provision the declared environment and hooks.';

	@printf '  %-16s %s\n' 'upg' 'Resolve the newest declared releases, write the uv and mise locks, then prove the upgraded tree still converges and passes every active check gate.';

	@printf '  %-16s %s\n' 'build' 'Build the project distribution artifacts.';

	@printf '  %-16s %s\n' 'check' 'Run the configured non-test gates except the dedicated smells audit.';

	@printf '  %-16s %s\n' 'smells' 'Run the strict code-smell audit as a dedicated gate.';

	@printf '  %-16s %s\n' 'test' 'Run incremental tests through the persistent testmon cache.';

	@printf '  %-16s %s\n' 'test-full' 'Run incremental then all tests, including external and CI-excluded markers, through the same persistent testmon cache.';

	@printf '  %-16s %s\n' 'fmt' 'Apply ruff format --preview and every declared formatter gate. Ruff is the rule; change code, never ruff.';

	@printf '  %-16s %s\n' 'fix' 'Apply the safe fixes of ruff check --fix --preview plus every other configured safe correction; never deletes information. Ruff is the rule; change code, never ruff.';

	@printf '  %-16s %s\n' 'fix-namespace' 'Apply the canonical namespace enforcer to the selected workspace.';

	@printf '  %-16s %s\n' 'fix-accessors' 'Migrate forbidden accessor names and every resolved consumer.';

	@printf '  %-16s %s\n' 'audit' 'Inspect ownership, dependency, and generated-state health.';

	@printf '  %-16s %s\n' 'status' 'Report the resolved runtime and repository state.';

	@printf '  %-16s %s\n' 'verify-clean' 'Verify that managed artifacts and generated documentation match their sources and leave no unstaged change to a tracked file.';

	@printf '  %-16s %s\n' 'docs' 'Generate, fix, format, and check documentation.';

	@printf '  %-16s %s\n' 'clean' 'Remove every declared disposable artifact.';

	@printf '  %-16s %s\n' 'bootstrap-candidate' 'Bootstrap declared candidate worktrees with this branch-matched generator.';

	@printf '  %-16s %s\n' 'release-plan' 'Resolve the release decision through the public protocol.';

	@printf '  %-16s %s\n' 'release-version' 'Materialize the planned version.';

	@printf '  %-16s %s\n' 'release-tag' 'Tag the verified release commit.';

	@printf '  %-16s %s\n' 'release-build' 'Build the release receipt and artifacts.';

	@printf '  %-16s %s\n' 'publication' 'Publish only receipt-attested release artifacts.';

	@printf '  %-16s %s\n' 'gen' 'Regenerate every managed projection atomically.';

	@printf '  %-16s %s\n' 'initialize' 'Materialize the declared package initializer graph.';

	@printf '  %-16s %s\n' 'mod' 'Apply the declared structural codemods; committed rule-test snapshots are verified, never rewritten.';

	@printf '  %-16s %s\n' 'mod-text' 'Apply the declared Sed text rules with exact receipts and guarded publication.';

	@printf '  %-16s %s\n' 'mod-text-candidate' 'Apply declared Sed text rules to the configured candidate workspace.';

	@printf '  %-16s %s\n' 'mod-snapshots' 'Regenerate the owned ast-grep rule-test snapshots from their tests for a reviewed commit.';

	@printf '  %-16s %s\n' 'waza' 'Validate provider-neutral governance semantics with Waza.';

	@printf '  %-16s %s\n' 'duplication' 'Run the canonical jscpd duplicate-code gate.';

	@printf '  %-16s %s\n' 'sonarcloud-sync' 'Write the SSOT SonarCloud issue exclusions to the server-side project settings (requires SONAR_TOKEN).';

	@printf '  %-16s %s\n' 'gitops' 'Script command: gitops';

	@printf '  %-16s %s\n' 'guard' 'Script command: guard';

	@printf '  %-16s %s\n' 'rust' 'Script command: rust';

	@printf '  %-16s %s\n' 'validate' 'Script command: validate';


# A project owns the sources declared by its manifest. The generated setup
# reconciler validates every initialized checkout before mutation, initializes
# only missing modules, and preserves declared branches that fix forward beyond
# the recorded gitlink.
.PHONY: _builtin_setup_submodules

# === SECTION: submodule setup (managed) ===
# Source: template (submodule_setup_recipe.j2)
# Computed: workspace uses MANAGED_GITLINKS from config; standalone discovers
#           submodules with flext-managed=true from .gitmodules at runtime.
# Rule: setup PROVISIONS an absent governed gitlink and VERIFIES a present one.
#       An absent checkout holds no work, so setup initializes it at the recorded
#       gitlink. A present checkout is never destroyed: git checkout, git reset,
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
	absent=""; \
	for path in $$managed; do \
		[ -e "$$root/$$path/.git" ] || absent="$$absent $$path"; \
	done; \
	if [ -n "$$absent" ]; then \
		credential_helper='!f() { if [ "$$1" = get ]; then printf "username=x-access-token\npassword=%s\n" "$$GITHUB_TOKEN"; fi; }; f'; \
		GIT_TERMINAL_PROMPT=0 timeout --signal=TERM --kill-after=5s "120s" \
			git -C "$$root" -c credential.helper= \
			-c "credential.https://$${GH_HOST:-github.com}.helper=$$credential_helper" \
			submodule update --init --jobs "$${FLEXT_SUBMODULE_JOBS:-8}" -- $$absent; \
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
_bootstrap_setup_tools: _builtin_recover_mise $(if $(filter upg,$(MAKECMDGOALS)),,_builtin_require_mise_pin)

.PHONY: _builtin_recover_mise
_builtin_recover_mise:
	@set -eu; \
	project_root=$$(cd "$(RUNTIME_ROOT)" && pwd -P); \
	project_parent=$${project_root%/*}; \
	if [ -z "$$project_parent" ]; then project_parent=/; fi; \
	for prior in "$$project_parent/.$${project_root##*/}.mise-lock-stage."*; do \
		if [ ! -d "$$prior" ]; then continue; fi; \
		if [ ! -f "$$prior/transaction.json" ]; then printf 'ERROR: uncommitted Mise stage has no recovery journal: %s\n' "$$prior" >&2; exit 2; fi; \
		if [ ! -f "$$prior/python-path" ]; then printf 'ERROR: Mise transaction lacks its Python receipt: %s\n' "$$prior" >&2; exit 2; fi; \
		IFS= read -r recovery_python < "$$prior/python-path"; \
		if [ ! -x "$$recovery_python" ]; then printf 'ERROR: Mise transaction Python is unavailable: %s\n' "$$recovery_python" >&2; exit 2; fi; \
		"$$recovery_python" "$$project_root/bin/mise-lock-transaction.py" recover "$$project_root" "$$prior"; \
	done

.PHONY: _builtin_require_mise_pin
_builtin_require_mise_pin: _builtin_recover_mise
	@set -eu; \
	if [ ! -s "$(MISE_VERSION_PIN)" ]; then \
		printf 'ERROR: missing or empty %s; make upg records the Mise release\n' "$(MISE_VERSION_PIN)" >&2; \
		exit 2; \
	fi; \
	mise_pin=$$(awk '!/^[[:space:]]*(#|$$)/ { lines++; release = $$0 } END { if (lines == 1) print release }' "$(MISE_VERSION_PIN)"); \
	if ! printf '%s\n' "$$mise_pin" | grep -Eq '^[0-9]+(\.[0-9]+){2}$$'; then \
		printf 'ERROR: %s must contain a resolved Mise release; make upg writes it\n' "$(MISE_VERSION_PIN)" >&2; \
		exit 2; \
	fi; \
	if [ -n "$${MISE_VERSION:-}" ] && [ "$${MISE_VERSION#v}" != "$$mise_pin" ]; then \
		printf 'ERROR: MISE_VERSION conflicts with %s\n' "$(MISE_VERSION_PIN)" >&2; \
		exit 2; \
	fi

_builtin_require_environment: _builtin_require_workspace _builtin_require_mise_pin
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
# Setup always reconciles directly from the lock. The venv is created when
# missing and is never cleared while present, because a concurrent lane may be
# running against it.
# Governed gitlinks are provisioned in every context, GitHub Actions included:
# the workspace projections (Makefile, pyproject, .gitignore, dependabot, docs)
# derive from the member checkouts, so a member-less CI checkout would render a
# different workspace and break the gen fixed point. Provisioned
# members are read as libraries; no verb gates them from here.
_builtin_setup_environment: _builtin_setup_submodules
ifeq ($(MAKE_PROFILE),workspace)
	@$(SETUP_ENVIRONMENT_RECIPE)
	@$(UV) pip check --python "$(RUNTIME_VENV)"
else
	@$(SETUP_ENVIRONMENT_RECIPE)
endif
# End SECTION: setup environment

# `upg` is the only recipe that resolves. Its first half provisions the
# generator: the bootstrap above resolves the Mise release and locks the tools
# the committed manifest declares, then this lifecycle upgrades every uv.lock
# (which carries the generator itself), provisions the environment frozen from
# it, and conforms dependency floors. The floors land in the codegen SSOT, so
# `gen` projects them into every pyproject and renders the managed tool
# manifests (.mise.toml) of the upgraded generator.
# Branch-tracked git dependencies are moving sources by declaration
# (workspace.yaml owns the branch): --refresh re-reads their metadata so a
# stale cached requires-dist can never block or skew the resolution.
# Like `setup`, it runs the declared pre-/post-upg lifecycle
# hooks, post-upg inside the activated environment.
.PHONY: _upg_lifecycle
_upg_lifecycle: _builtin_setup_submodules
	@set -eu; \
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" pre-upg "*) $(SELF_MAKE) pre-upg ;; \
	esac
	$(call _lock_project,--upgrade --refresh)
	@$(SELF_MAKE) _builtin_setup_environment
	@$(PROJECT_FLEXT_INFRA) deps modernize --repository-root "$(PROJECT_ROOT)" \
		--apply --rewrite-constraints --projects .
	@$(SELF_MAKE) gen
	@$(SELF_MAKE) _upg_relock
	@set -eu; \
	if [ -d .mise/locks ]; then \
		git add -- .mise/locks; \
		printf 'INFO: staged the .mise/locks sidecars written by mise lock (declared tracked by the generated .gitignore; commit them with the relock)\n'; \
	fi

# The second half belongs to the Makefile `gen` just rendered, so it runs as a
# fresh make invocation rather than as lines of the recipe already expanded
# above. It locks mise.lock from the rendered .mise.toml (never from the
# manifest the generator was provisioned with) unless that manifest is the one
# the first half already locked, installs exactly that lock,
# re-resolves uv.lock against the raised floors and reprovisions frozen from
# both: the committed locks must match the committed manifests, or `setup`
# (the CI path, `--locked`) rejects them. The Mise release is not resolved
# again; the first half already pinned it.
.PHONY: _upg_relock
_upg_relock: TOOL_BOOTSTRAP_LIFECYCLE := _upg_converge
_upg_relock: TOOL_BOOTSTRAP_LOCK := 1
_upg_relock: _bootstrap_setup_tools

# An upgrade publishes only after the cycle it changed still passes: the
# generation fixed point is proven above, and every active gate must be
# green on the upgraded tree, so a package update that breaks types, lint
# or consistency fails the upgrade itself instead of surfacing later as
# red tests or red CI.
.PHONY: _upg_converge
_upg_converge:
	$(call _lock_project,,retain)
	@$(SELF_MAKE) _builtin_setup_environment
	@$(UV) lock --check --project "$(PROJECT_ROOT)"
	@set -eu; \
	before="$$(git -C "$(PROJECT_ROOT)" status --porcelain --untracked-files=all --ignore-submodules=none | sort)"; \
	$(SELF_MAKE) gen; \
	after="$$(git -C "$(PROJECT_ROOT)" status --porcelain --untracked-files=all --ignore-submodules=none | sort)"; \
	if [ "$$before" != "$$after" ]; then \
		printf 'ERROR: make upg did not converge; `make gen` still rewrites:\n%s\n' "$$after" >&2; \
		exit 2; \
	fi
	@$(PROJECT_FLEXT_INFRA) deps verify-locks --repository-root "$(PROJECT_ROOT)"
	+@XDG_DATA_HOME="$${SETUP_DIRENV_XDG_DATA_HOME:?missing persistent direnv data home}" \
		"$${SETUP_DIRENV:?missing Mise-resolved direnv executable}" exec "$(PROJECT_ROOT)" $(SELF_MAKE) _upg_activated
	@$(SELF_MAKE) check

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
		gates="lint,security,markdown,duplication,pyrefly,mypy,pyright,loc-cap,runtime-census,fresh-import,index-declarations,codemod,layout,direnv"; \
		if [ "$(strip $(CI))" = "Y" ]; then \
			gates="lint,security,markdown,duplication"; \
			printf 'INFO: CI=Y runs check gates: lint security markdown duplication\n'; \
		elif [ "$(strip $(CI))" = "N" ]; then \
			gates="pyrefly,mypy,pyright,loc-cap,runtime-census,fresh-import,index-declarations,codemod,layout,direnv"; \
			printf 'INFO: CI=N runs check gates: pyrefly mypy pyright loc-cap runtime-census fresh-import index-declarations codemod layout direnv\n'; \
		else \
			printf 'INFO: default context runs check gates: lint security markdown duplication pyrefly mypy pyright loc-cap runtime-census fresh-import index-declarations codemod layout direnv\n'; \
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
TESTMON_DATAFILE="$$database" $(PYTEST_BOUNDED) $(PROJECT_TOOL_EXEC) $(UV_RUN) python -m flext_infra._pytest_entry; \
TESTMON_DATAFILE="$$database" $(PYTEST_BOUNDED) $(PROJECT_TOOL_EXEC) $(UV_RUN) python -m flext_infra._pytest_entry slow

_builtin_test_full_all: _builtin_require_environment
	@set -eu; \
database="$(FLEXT_PYTEST_TESTMON_DATABASE)"; \
case "$$database" in /*) ;; *) printf 'ERROR: persistent testmon database requires XDG_CACHE_HOME or HOME\n' >&2; exit 2 ;; esac; \
case "$$database" in "$(PROJECT_ROOT)"/*) printf 'ERROR: persistent testmon database must be outside the checkout: %s\n' "$$database" >&2; exit 2 ;; esac; \
case "$$database" in "$${TMPDIR:-/tmp}"/*|/tmp/*) printf 'ERROR: persistent testmon database must not live under the temporary directory: %s\n' "$$database" >&2; exit 2 ;; esac; \
mkdir -p "$$(dirname "$$database")"; \
TESTMON_DATAFILE="$$database" $(PYTEST_BOUNDED) $(PROJECT_TOOL_EXEC) $(UV_RUN) python -m flext_infra._pytest_entry full; \
TESTMON_DATAFILE="$$database" $(PYTEST_BOUNDED) $(PROJECT_TOOL_EXEC) $(UV_RUN) python -m flext_infra._pytest_entry full-slow

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


_builtin_run_default: _builtin_require_environment
	@$(UV_RUN) $(PROJECT_NAME) $(ARGS)

# Profile the real runtime-census gate through the same installed CLI selected
# by the root dispatcher. The report stays in this checkout's .reports tree.
.PHONY: profile-census
profile-census: _builtin_require_environment
	@mkdir -p "$(PROFILE_REPORTS_DIR)"
	@$(PROJECT_TOOL_EXEC) "$(RUNTIME_PYTHON)" -c \
		'import cProfile, sys; from flext_infra.cli import main; profile = cProfile.Profile(); status = profile.runcall(main, sys.argv[2:]); profile.dump_stats(sys.argv[1]); raise SystemExit(status)' \
		"$(PROFILE_REPORTS_DIR)/runtime-census.pstats" check run \
		--repository-root "$(PROJECT_ROOT)" --gates runtime-census

.PHONY: profile-census-report
profile-census-report: _builtin_require_environment
	@$(PROJECT_TOOL_EXEC) "$(RUNTIME_PYTHON)" -c \
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
	@$(PROJECT_TOOL_EXEC) "$(RUNTIME_PYTHON)" -c \
		'import pstats, sys; pstats.Stats(sys.argv[1]).sort_stats("cumtime").print_stats(50)' \
		"$(PROFILE_REPORTS_DIR)/mypy.pstats"

.PHONY: profile-gen
profile-gen: _builtin_require_environment
	@mkdir -p "$(PROFILE_REPORTS_DIR)"
	@$(PROJECT_TOOL_EXEC) "$(RUNTIME_PYTHON)" -c \
		'import cProfile, sys; from flext_infra.cli import main; profile = cProfile.Profile(); status = profile.runcall(main, sys.argv[2:]); profile.dump_stats(sys.argv[1]); raise SystemExit(status)' \
		"$(PROFILE_REPORTS_DIR)/lazy-init.pstats" codegen lazy-init \
		--repository-root "$(PROJECT_ROOT)" --module mcb_scripts --dry-run

.PHONY: profile-gen-report
profile-gen-report: _builtin_require_environment
	@$(PROJECT_TOOL_EXEC) "$(RUNTIME_PYTHON)" -c \
		'import pstats, sys; pstats.Stats(sys.argv[1]).sort_stats("cumtime").print_stats(50)' \
		"$(PROFILE_REPORTS_DIR)/lazy-init.pstats"

# Profile the cold canonical pytest execution through its thin entrypoint
# for startup diagnosis: the same persistent testmon database
# guard and environment as the bounded gate, but deliberately NOT wrapped in
# PYTEST_BOUNDED. The runner's own deadline still applies. Central collection
# children also write profiles beside their manifests and print their paths.
# The stdlib-only adapter starts profiling before runner/model/pytest imports.
# The parent sidecar binds the exact run;
# reports never combine a parent profile with the mutable latest.txt pointer.
.PHONY: profile-test
profile-test: _builtin_require_environment
	@mkdir -p "$(PROFILE_REPORTS_DIR)"
	@set -eu; \
database="$(FLEXT_PYTEST_TESTMON_DATABASE)"; \
case "$$database" in /*) ;; *) printf 'ERROR: persistent testmon database requires XDG_CACHE_HOME or HOME\n' >&2; exit 2 ;; esac; \
case "$$database" in "$(PROJECT_ROOT)"/*) printf 'ERROR: persistent testmon database must be outside the checkout: %s\n' "$$database" >&2; exit 2 ;; esac; \
case "$$database" in "$${TMPDIR:-/tmp}"/*|/tmp/*) printf 'ERROR: persistent testmon database must not live under the temporary directory: %s\n' "$$database" >&2; exit 2 ;; esac; \
mkdir -p "$$(dirname "$$database")"; \
	TESTMON_DATAFILE="$$database" $(PROJECT_TOOL_EXEC) "$(RUNTIME_PYTHON)" -m flext_infra._pytest_entry profile \
		"$(PROFILE_REPORTS_DIR)/pytest.pstats"

.PHONY: profile-test-report
profile-test-report: _builtin_require_environment
	@$(PROJECT_TOOL_EXEC) "$(RUNTIME_PYTHON)" -m flext_infra._cprofile_entry \
		"$(PROFILE_REPORTS_DIR)/pytest.pstats" "$(PROFILE_REPORTS_DIR)/pytest.pstats.json"

_builtin_status_diagnostics: _builtin_require_environment
	@printf 'profile=%s\nproject=%s\nruntime=%s\n' \
		'$(MAKE_PROFILE)' '$(PROJECT_ROOT)' '$(RUNTIME_ROOT)'
	@$(PROJECT_TOOL_EXEC) $(SHELL) -c 'command -v uv'
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

# Generation has one transaction owner. Conform runs at this repository's own
# scope and journals ordinary, Mise, lazy-init, and documentation phases through
# one fixed point. Only `upg` resolves and rewrites the locks; gen installs
# nothing and never runs another writer before or after conform's journal.
_builtin_gen_init:
	@$(PROJECT_FLEXT_INFRA) codegen init --repository-root "$(PROJECT_ROOT)"
	@$(PROJECT_FLEXT_INFRA) codegen init --repository-root "$(PROJECT_ROOT)" --check-only

_builtin_gen_all:
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --mode apply

_builtin-bootstrap-candidate: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) codegen candidate-bootstrap --repository-root "$(PROJECT_ROOT)"

# Structural rewrites use mod; mod-text replays the same declared text phase
# independently when malformed Python prevents the Rope phases from loading.
# The current directory defines scope; callers never address tools directly.
_builtin_mod_apply: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) refactor mod --apply

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
_builtin-fmt: _builtin_fmt_all
_builtin-fix: _builtin_fix_all
_builtin-fix-namespace: _builtin_fix_namespace
_builtin-fix-accessors: _builtin_fix_accessors
_builtin-audit:
	@$(UV) pip check --python "$(RUNTIME_VENV)"
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --mode check
_builtin-status: _builtin_status_diagnostics
_builtin-verify-clean: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --mode check
	@$(PROJECT_FLEXT_INFRA) docs audit --repository-root "$(PROJECT_ROOT)" --output-dir ".reports/docs" --projects .
	@git -C "$(PROJECT_ROOT)" diff --exit-code
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
	@cd "$(PROJECT_ROOT)" && $(PROJECT_TOOL_EXEC) waza check --no-update-check

_builtin-smells:
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "smells"

_builtin-duplication:
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "duplication"
_builtin-sonarcloud-sync: _builtin_sonarcloud_sync_all

# Repository-owned script verbs route through the promoted command framework:
# custom.mk handlers override via RUN_PUBLIC; otherwise the dispatcher resolves
# <verb>/<WHAT> across SCRIPT_ROOTS (auto-detected surfaces, no manual lists).

_builtin-gitops: _builtin_require_environment
	$(call _dispatch,gitops)

_builtin-guard: _builtin_require_environment
	$(call _dispatch,guard)

_builtin-rust: _builtin_require_environment
	$(call _dispatch,rust)

_builtin-validate: _builtin_require_environment
	$(call _dispatch,validate)

