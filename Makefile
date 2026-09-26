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
SELF_MAKE_EXECUTABLE := $(shell command -v "$(MAKE_COMMAND)")
ifneq ($(.SHELLSTATUS),0)
$(error Cannot resolve current Make executable: $(MAKE_COMMAND))
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

# GitHub CLI's documented source precedence also applies before gh is installed:
# GH_TOKEN, GITHUB_TOKEN, then its stored credential for network bootstrap.
# Local provisioned operations need no authentication preflight or network call.
# Keep the selected value in the environment, never in a rendered recipe.
override GITHUB_CREDENTIAL_READ_STATUS := 0
ifneq ($(strip $(GH_TOKEN)),)
override GITHUB_TOKEN := $(GH_TOKEN)
else ifeq ($(strip $(GITHUB_TOKEN)),)
ifneq ($(strip $(filter setup upg,$(MAKECMDGOALS))),)
override GITHUB_TOKEN := $(shell if command -v gh >/dev/null 2>&1; then gh auth token --hostname "$${GH_HOST:-github.com}"; else printf 'ERROR: GitHub credential source unavailable: set GH_TOKEN/GITHUB_TOKEN or provision gh\n' >&2; exit 127; fi)
override GITHUB_CREDENTIAL_READ_STATUS := $(.SHELLSTATUS)
endif
endif
export GITHUB_TOKEN
override export GH_TOKEN := $(GITHUB_TOKEN)
override export MISE_GITHUB_TOKEN := $(GITHUB_TOKEN)

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
# Operator law 2026-09-14: the Makefile validates no caller input. Every verb
# always applies; a mistyped variable fails where the verb consumes it, and an
# unconsumed variable is ignored.
PYTEST_DIAG_ARGS := -rA --durations=0 --tb=long --showlocals
PYTEST_REPORT_ARGS := -ra --durations=25 --durations-min=0.001 --tb=short
PYTEST_PROCESS_TIMEOUT_SECONDS := 7260
# mro-99ae: the pytest process inherits a hard wall-clock boundary, so a hung
# run is terminated even if the runner itself stalls.
PYTEST_BOUNDED = timeout --signal=TERM --kill-after=5s "$(PYTEST_PROCESS_TIMEOUT_SECONDS)s"
PYTEST_REPORTS_DIR := .reports/tests
override PYTEST_CASE_TIMEOUT_SECONDS := 10
override PYTEST_RUN_TIMEOUT_SECONDS := 7200
override PYTEST_TERMINATION_GRACE_SECONDS := 2
override PYTEST_TIMEOUT_EXIT_CODE := 124
override PYTEST_ENFORCEMENT_PLUGIN := flext_tests_enforcement
override PYTEST_PROGRESS_ARGS := --verbose
override PYTEST_REPORT_ARGS := -ra --durations=25 --durations-min=0.001 --tb=short
override PYTEST_DIAG_ARGS := -rA --durations=0 --tb=long --showlocals
override PYTEST_PARALLEL_WORKERS := 2
override PYTEST_PARALLEL_WORKER_MEMORY_GB := 2
override PYTEST_PARALLEL_DISTRIBUTION := load
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
# The Mise release is frozen like every tool: `upg` records the release it
# resolved in the committed pin file; every other verb exports it so no
# launcher call resolves `latest` (operator law 2026-09-24).
override MISE_VERSION_PIN := $(PROJECT_ROOT)/mise.version
ifneq ($(wildcard $(MISE_VERSION_PIN)),)
export MISE_VERSION := $(strip $(file < $(MISE_VERSION_PIN)))
endif
override export FLEXT_PYTEST_TARGET_RAW := tests
PROJECT_STATE_ROOT := $(abspath $(PROJECT_ROOT)/../.flext-runtime/$(notdir $(PROJECT_ROOT)))
# Scratch never lives inside a versioned tree: the home scratch root mirrors
# the absolute checkout path so a sandbox is never a tracked scope of any
# enclosing repository (workspace or linked worktree). Checkouts nested in a
# VCS directory rename that segment, so the mirror never contains one; two
# substitution passes rename adjacent repeated segments too.
ifeq ($(strip $(HOME)),)
$(error HOME is required to derive the scratch root)
endif
PROJECT_SCRATCH_IDENTITY := $(abspath $(PROJECT_ROOT))/
PROJECT_SCRATCH_IDENTITY := $(subst /.git/,/_git/,$(subst /.git/,/_git/,$(PROJECT_SCRATCH_IDENTITY)))
PROJECT_SCRATCH_ROOT := $(HOME)/tmp/.flext-runtime$(patsubst %/,%,$(PROJECT_SCRATCH_IDENTITY))/scratch
TESTMON_DATAFILE := $(PROJECT_STATE_ROOT)/testmon/.testmondata
export TESTMON_DATAFILE
# === SECTION: REPOSITORY_ROOT isolation (managed) ===
# Source: physical checkout topology; caller variables cannot select a workspace.
# Operator law 2026-09-24 (flext-x8gn6): inside a workspace every make run, root
# or member, uses the workspace runtime. A member resolves the Git superproject
# that checks it out as a submodule; a checkout without one (a standalone clone,
# a linked worktree) owns its runtime.
ifneq ($(GEN_INIT_ONLY),)
override REPOSITORY_ROOT := $(MAKEFILE_ROOT)
else
override REPOSITORY_ROOT := $(shell cd "$(MAKEFILE_ROOT)" && root=$$(git rev-parse --show-superproject-working-tree) && if [ -n "$$root" ]; then cd "$$root" && pwd -P; else pwd -P; fi)
ifneq ($(.SHELLSTATUS),0)
$(error Cannot resolve the physical workspace for $(MAKEFILE_ROOT))
endif
endif
# End SECTION: REPOSITORY_ROOT isolation
# === SECTION: verb dispatch (managed) ===
# Source: config:make.verbs and the canonical gate vocabulary. A verb exists
# only in the profiles it declares (make.verbs[].profiles).
PUBLIC_VERBS := help setup upg build check test test-full fmt fix fix-enforcement audit status docs clean release-plan release-version release-tag release-build publication gen initialize mod waza duplication sonarcloud-sync gitops guard rust validate
BUILTIN_VERBS := help setup upg build check test test-full fmt fix fix-enforcement audit status docs clean release-plan release-version release-tag release-build publication gen initialize mod waza duplication sonarcloud-sync
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
override UV := "$(SETUP_MISE)" -C "$(PROJECT_ROOT)" exec -- uv
CALLER_PATH := $(PATH)
CALLER_VIRTUAL_ENV := $(patsubst %/,%,$(VIRTUAL_ENV))
# End SECTION: project tool owner

# === SECTION: profile routing (managed) ===
# Source: repository topology. workspace has .gitmodules; standalone does not.
# Attached members share their Git superproject runtime; standalone owns itself.
override RUNTIME_ROOT := $(REPOSITORY_ROOT)
override export GIT_CEILING_DIRECTORIES := $(abspath $(RUNTIME_ROOT)/..)
override export MISE_CEILING_PATHS := $(abspath $(RUNTIME_ROOT)/..)
# End SECTION: profile routing

override RUNTIME_VENV := $(RUNTIME_ROOT)/.venv
ifeq ($(OS),Windows_NT)
override RUNTIME_BIN := $(RUNTIME_VENV)/Scripts
override RUNTIME_PYTHON := $(RUNTIME_BIN)/python.exe
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
export FLEXT_INFRA_PYTHON UV_PROJECT UV_PROJECT_ENVIRONMENT VIRTUAL_ENV PATH

# One bootstrap serves `setup` (frozen) and `upg` (resolving); the public verb
# selects its lifecycle and resolution through target-specific variables.
TOOL_BOOTSTRAP_LIFECYCLE := _setup_lifecycle
TOOL_BOOTSTRAP_RESOLVE :=
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
mise_lockfile_platforms="linux-x64,linux-arm64,macos-x64,macos-arm64,windows-x64"; \
caller_comspec="$${COMSPEC:-}"; \
caller_pathext="$${PATHEXT:-}"; \
caller_systemroot="$${SYSTEMROOT:-}"; \
caller_windir="$${WINDIR:-}"; \
caller_github_token="$${GITHUB_TOKEN:-}"; \
caller_gh_token="$${GH_TOKEN:-}"; \
caller_mise_github_token="$${MISE_GITHUB_TOKEN:-}"; \
caller_mise_github_credential_command="$${MISE_GITHUB_CREDENTIAL_COMMAND:-}"; \
caller_mise_http_timeout="$${MISE_HTTP_TIMEOUT:-}"; \
caller_mise_version="$${MISE_VERSION:-}"; \
# Only ``upg`` resolves the Mise release; every other lifecycle launches \
	# exactly the release ``upg`` recorded in the committed pin file. \
	mise_pin_file="$$project_root/mise.version"; \
	if [ "$(TOOL_BOOTSTRAP_RESOLVE)" != "1" ]; then \
		if [ ! -f "$$mise_pin_file" ]; then \
			printf 'ERROR: missing %s; make upg resolves and records the Mise release\n' "$$mise_pin_file" >&2; \
			exit 2; \
		fi; \
		mise_pin=$$(cat "$$mise_pin_file"); \
		if [ -n "$$caller_mise_version" ] && [ "$${caller_mise_version#v}" != "$$mise_pin" ]; then \
			printf 'ERROR: MISE_VERSION=%s conflicts with %s=%s\n' "$$caller_mise_version" "$$mise_pin_file" "$$mise_pin" >&2; \
			exit 2; \
		fi; \
		caller_mise_version="$$mise_pin"; \
	fi; \
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
	project_root=$$(cd "$$project_root" && pwd -P); \
	project_parent=$${project_root%/*}; \
	if [ -z "$$project_parent" ]; then project_parent=/; fi; \
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
	scratch_parent="$(PROJECT_SCRATCH_ROOT)"; \
	if [ -L "$$scratch_parent" ]; then \
		printf 'ERROR: Mise scratch parent must not be a symlink: %s\n' "$$scratch_parent" >&2; exit 2; \
	fi; \
	mkdir -p "$$scratch_parent"; \
	scratch=$$(mktemp -d "$$scratch_parent/mise-toolchain.XXXXXX"); \
	trap 'find "$$scratch" -depth -delete' EXIT; \
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
			"MISE_TRUSTED_CONFIG_PATHS=$$project_root" \
$${caller_path:+"PATH=$$caller_path"} \
$${caller_comspec:+"COMSPEC=$$caller_comspec"} \
$${caller_pathext:+"PATHEXT=$$caller_pathext"} \
$${caller_systemroot:+"SYSTEMROOT=$$caller_systemroot"} \
$${caller_windir:+"WINDIR=$$caller_windir"} \
$${caller_github_token:+"GITHUB_TOKEN=$$caller_github_token"} \
$${caller_gh_token:+"GH_TOKEN=$$caller_gh_token"} \
$${caller_mise_github_token:+"MISE_GITHUB_TOKEN=$$caller_mise_github_token"} \
$${caller_mise_github_credential_command:+"MISE_GITHUB_CREDENTIAL_COMMAND=$$caller_mise_github_credential_command"} \
$${caller_mise_http_timeout:+"MISE_HTTP_TIMEOUT=$$caller_mise_http_timeout"} \
$${caller_mise_version:+"MISE_VERSION=$$caller_mise_version"} \
$${mise_config_argument:+"$$mise_config_argument"} \
			$${mise_runtime_path:+"MISE_INSTALL_PATH=$$mise_runtime_path"} \
			"$$@"; \
	}; \
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
	latest_mise="$$mise"; \
	mise_checked_stdout "$$scratch/runtime-version.stdout" "$$scratch/runtime-version.stderr" mise_exec no-config "$$latest_mise" --version; \
	receipt_runtime=$$(cat "$$scratch/runtime-version.stdout"); \
	case "$$receipt_runtime" in \
		'mise '*) runtime_release=$${receipt_runtime#mise }; runtime_release=$${runtime_release%% *} ;; \
		*) runtime_release=$${receipt_runtime%% *} ;; \
	esac; \
	if ! printf '%s\n' "$$runtime_release" | grep -Eq '^[0-9]+(\.[0-9]+){2}$$'; then \
		printf 'ERROR: Mise receipt returned invalid version: %s\n' "$$receipt_runtime" >&2; exit 2; \
	fi; \
	if [ "$(TOOL_BOOTSTRAP_RESOLVE)" = "1" ]; then \
		printf '%s\n' "$$runtime_release" > "$$mise_pin_file"; \
	elif [ "$$runtime_release" != "$$mise_pin" ]; then \
		printf 'ERROR: launched Mise %s differs from the pinned %s\n' "$$runtime_release" "$$mise_pin" >&2; \
		exit 2; \
	fi; \
	caller_mise_version="$$runtime_release"; \
	printf 'mise setup receipt=%s storage=%s\n' "$$runtime_release" "$$mise_storage_root"; \
	# Only ``upg`` resolves. Artifact tools own a five-platform URL/checksum \
	# matrix; npm owns one platform-independent Aube dependency graph. \
	if [ "$(TOOL_BOOTSTRAP_RESOLVE)" = "1" ]; then \
		mise_checked "$$scratch/lock-artifacts.log" mise_exec project "$$latest_mise" -C "$$project_root" lock --bump python uv kubectl helm kind direnv taplo ast-grep gitleaks "aqua:boyter/scc" kubeconform node go make "github:qltysh/qlty" "github:kucherenko/jscpd" "github:microsoft/waza"; \
		mise_lockfile_platforms=; \
		mise_checked "$$scratch/lock-npm-prettier.log" mise_exec project "$$latest_mise" -C "$$project_root" lock --bump "npm:prettier"; \
		mise_lockfile_platforms="linux-x64,linux-arm64,macos-x64,macos-arm64,windows-x64"; \
	fi; \
	# ``locked`` mode installs exactly what the committed mise.lock pins. \
	mise_checked "$$scratch/install.log" mise_exec project "$$latest_mise" -C "$$project_root" install --yes; \
	mise_checked "$$scratch/uv-version.log" mise_exec project "$$latest_mise" -C "$$project_root" exec -- uv --version; \
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
	mise_checked "$$scratch/direnv-path.log" mise_exec project "$$latest_mise" -C "$$project_root" which direnv; \
	direnv_executable=$$(cat "$$scratch/direnv-path.log"); \
	if [ ! -x "$$direnv_executable" ]; then \
		printf 'ERROR: Mise resolved a non-executable direnv path: %s\n' "$$direnv_executable" >&2; exit 2; \
	fi; \
	if [ -n "$${GITHUB_PATH:-}" ]; then \
		printf '%s\n' "$$project_root/bin" >> "$$GITHUB_PATH"; \
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
		"$$latest_mise" -C "$$project_root" exec -- env \
		"SETUP_DIRENV=$$direnv_executable" \
		"SETUP_DIRENV_XDG_DATA_HOME=$$caller_xdg_data_home" \
		"CI=$(CI)" $(SELF_MAKE) $(TOOL_BOOTSTRAP_LIFECYCLE)

ifeq ($(MAKE_PROFILE),workspace)
CODEGEN_SCOPE := all
ALLOWED_PROJECTS := . $(WORKSPACE_SUBPROJECTS)
else
CODEGEN_SCOPE := self
ALLOWED_PROJECTS := .
endif

# Workspace-root gate verbs fan out across declared members through the generic
# `flext-infra workspace orchestrate` primitive (verb allowlist + CLI group come
# from the constants SSOT, never hardcoded here). Members and standalone projects
# run the gate locally. Every member runs; the summary names every failure.
# Provisioning is declared once and shared by every profile. A venv records the
# exact base interpreter used to create it, so setup replaces it when Mise moves
# the configured Python minor line to a newer patch.
SETUP_ENVIRONMENT_RECIPE = set -eu; \
	$(REQUIRE_WORKSPACE_ENVIRONMENT); \
	desired_python=$$("$(SETUP_MISE)" -C "$(PROJECT_ROOT)" which python); \
	if [ ! -x "$(RUNTIME_PYTHON)" ]; then \
		$(UV) venv --python "$$desired_python" "$(RUNTIME_VENV)"; \
	elif [ "$$(readlink -f "$(RUNTIME_PYTHON)")" != "$$(readlink -f "$$desired_python")" ]; then \
		printf 'setup: replacing environment for Python %s\n' "$$desired_python"; \
		$(UV) venv --clear --python "$$desired_python" "$(RUNTIME_VENV)"; \
	fi; \
	$(UV) sync --project "$(PROJECT_ROOT)" $(UV_SYNC_FLAGS) --link-mode "$(UV_LINK_MODE)"; \
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
	for environment_path in "$(RUNTIME_VENV)" "$(RUNTIME_BIN)" "$(PROJECT_ROOT)/.venv"; do \
		if [ -L "$$environment_path" ]; then \
			printf 'ERROR: workspace environment must be physical, not a symlink: %s\n' "$$environment_path" >&2; exit 2; \
		fi; \
	done

.PHONY: _builtin_require_workspace
_builtin_require_workspace:
	@$(REQUIRE_WORKSPACE_ENVIRONMENT)

_bootstrap_setup_tools: _builtin_require_workspace

WORKSPACE_ORCHESTRATE = $(UV_RUN) python -m flext_infra workspace orchestrate
# Workspace runs include the root project itself: `.` maps to the
# _builtin-self-* targets, so all 32 distributions execute their own gates
# (plan contract: no member of the fleet is excluded from required cycles).
DEFAULT_PROJECTS := $(WORKSPACE_SUBPROJECTS) .
SELECTED_PROJECTS := $(DEFAULT_PROJECTS)
WORKSPACE_PROJECT_ARGS := $(foreach project,$(SELECTED_PROJECTS),--projects $(project))
DOCS_PROJECT_ARGS := $(foreach project,$(SELECTED_PROJECTS),--projects $(project))

# Execute the interpreter provisioned by setup without discovering a project
# workspace or creating a dependency-resolution file during a runtime command.
UV_RUN := env -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u PROJECT_ROOT PYTHONPATH="$(PROJECT_ROOT)/src" $(UV) run --no-project --python "$(RUNTIME_PYTHON)"
override PROJECT_INFRA_PYTHONPATH := $(MAKEFILE_ROOT)/src
PROJECT_INFRA_RUN := if [ ! -x "$(FLEXT_INFRA_PYTHON)" ]; then printf 'ERROR: FLEXT_INFRA_PYTHON must name an executable managed Python\n' >&2; exit 2; fi; "$(SETUP_MISE)" -C "$(PROJECT_ROOT)" exec -- env -u PYTHONPATH -u MYPYPATH -u VIRTUAL_ENV -u UV_PROJECT -u UV_PROJECT_ENVIRONMENT PYTHONPATH="$(PROJECT_INFRA_PYTHONPATH)" $(FLEXT_INFRA_PYTHON)
PROJECT_FLEXT_INFRA := $(PROJECT_INFRA_RUN) -m flext_infra
# Scaffold dev tools live in the validated optional dev
# profile; setup resolves the declared dependency branches at their current tips.
# Keyed on the environment's OWNER, not on the caller's profile. A member has
# no local venv -- RUNTIME_VENV is RUNTIME_ROOT/.venv -- so every checkout that
# provisions a shared environment must describe the same contents. A member
# syncing without --all-packages treats the siblings already installed there as
# surplus and uninstalls them, undoing the root's provisioning and leaving
# `uv sync --check` permanently divergent. A standalone project owns its venv
# alone and has no workspace packages to include.
SHARED_RUNTIME := $(if $(filter-out $(PROJECT_ROOT),$(RUNTIME_ROOT)),1,$(if $(strip $(WORKSPACE_SUBPROJECTS)),1,))
# Setup installs frozen from the committed uv.lock and never re-resolves: it is
# the CI path and must be stable. A missing or stale lock fails through uv's own
# error; `make upg` is the only verb that resolves and rewrites it
# (operator 2026-09-24).
UV_SYNC_FLAGS := $(if $(SHARED_RUNTIME),--all-packages --reinstall-package flext-infra ,)--all-extras --all-groups --locked

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


define _run_for_all_projects
	@set -eu; \
	for project in $(SELECTED_PROJECTS); do \
		if [ "$$project" = "." ]; then project_root="$(PROJECT_ROOT)"; \
		else project_root="$(PROJECT_ROOT)/$$project"; fi; \
		$(UV) lock --project "$$project_root" $(1); \
	done
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




fix-enforcement: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-fix-enforcement

.PHONY: _activated-fix-enforcement
_activated-fix-enforcement: _builtin_require_environment

	$(call RUN_PUBLIC,fix-enforcement)




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




docs: _builtin_require_workspace
	+@direnv exec "$(PROJECT_ROOT)" $(SELF_MAKE) _activated-docs

.PHONY: _activated-docs
_activated-docs: _builtin_require_environment

	$(call RUN_PUBLIC,docs)




clean:

	$(call RUN_PUBLIC,clean)




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
# must not require an existing environment.
upg: TOOL_BOOTSTRAP_LIFECYCLE := _upg_lifecycle
upg: TOOL_BOOTSTRAP_RESOLVE := 1
upg: export MISE_VERSION :=
upg: _bootstrap_setup_tools

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

	@printf '  %-16s %s\n' 'upg' 'Resolve the newest declared releases and write the uv and mise locks.';

	@printf '  %-16s %s\n' 'build' 'Build the project distribution artifacts.';

	@printf '  %-16s %s\n' 'check' 'Run every configured non-test gate.';

	@printf '  %-16s %s\n' 'test' 'Run incremental tests through the persistent testmon cache.';

	@printf '  %-16s %s\n' 'test-full' 'Run incremental then all tests, including external and CI-excluded markers, through the same persistent testmon cache.';

	@printf '  %-16s %s\n' 'fmt' 'Apply ruff format --preview and ruff check --fix --unsafe-fixes --preview. Ruff is the rule; change code, never ruff.';

	@printf '  %-16s %s\n' 'fix' 'Apply ruff check --fix --unsafe-fixes --preview plus every other configured safe correction. Ruff is the rule; change code, never ruff.';

	@printf '  %-16s %s\n' 'fix-enforcement' 'Apply the safe fix actions declared by the enforcement catalog.';

	@printf '  %-16s %s\n' 'audit' 'Inspect ownership, dependency, and generated-state health.';

	@printf '  %-16s %s\n' 'status' 'Report the resolved runtime and repository state.';

	@printf '  %-16s %s\n' 'docs' 'Generate, fix, format, and check documentation.';

	@printf '  %-16s %s\n' 'clean' 'Remove every declared disposable artifact.';

	@printf '  %-16s %s\n' 'release-plan' 'Resolve the release decision through the public protocol.';

	@printf '  %-16s %s\n' 'release-version' 'Materialize the planned version.';

	@printf '  %-16s %s\n' 'release-tag' 'Tag the verified release commit.';

	@printf '  %-16s %s\n' 'release-build' 'Build the release receipt and artifacts.';

	@printf '  %-16s %s\n' 'publication' 'Publish only receipt-attested release artifacts.';

	@printf '  %-16s %s\n' 'gen' 'Regenerate every managed projection atomically.';

	@printf '  %-16s %s\n' 'initialize' 'Materialize the declared package initializer graph.';

	@printf '  %-16s %s\n' 'mod' 'Apply the declared structural codemods.';

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
# Why (flext-mphw1): runners expose umask 002 and `submodule update --init`
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
			git -C "$$superproject" submodule update --init -- "$$child_path"; \
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

.PHONY: _builtin_require_github_auth
# The credential check precedes the Mise pin check even under -j. `make setup`
# first runs on the host's make before Mise installs the declared one, so the
# ordering uses .NOTPARALLEL (every GNU Make; 4.4+ serializes only this target's
# prerequisites) instead of .WAIT, which older releases read as a missing target.
.NOTPARALLEL: _bootstrap_setup_tools
_bootstrap_setup_tools: _builtin_require_github_auth $(if $(filter upg,$(MAKECMDGOALS)),,_builtin_require_mise_pin)
_builtin_require_github_auth:
	@if [ "$(GITHUB_CREDENTIAL_READ_STATUS)" != "0" ]; then \
		printf 'ERROR: gh credential source failed with exit %s\n' "$(GITHUB_CREDENTIAL_READ_STATUS)" >&2; \
		exit "$(GITHUB_CREDENTIAL_READ_STATUS)"; \
	fi
	@if [ -z "$${GITHUB_TOKEN:-}" ]; then \
		printf 'ERROR: GitHub credential is absent for network bootstrap\n' >&2; \
		exit 1; \
	fi

.PHONY: _builtin_require_mise_pin
_builtin_require_mise_pin:
	@set -eu; \
	if [ ! -s "$(MISE_VERSION_PIN)" ]; then \
		printf 'ERROR: missing or empty %s; make upg records the Mise release\n' "$(MISE_VERSION_PIN)" >&2; \
		exit 2; \
	fi; \
	case "$(MISE_VERSION)" in ''|latest) \
		printf 'ERROR: %s must contain a resolved Mise release\n' "$(MISE_VERSION_PIN)" >&2; \
		exit 2 ;; \
	esac; \
	if [ "$(MISE_VERSION)" != "$$(cat "$(MISE_VERSION_PIN)")" ]; then \
		printf 'ERROR: MISE_VERSION conflicts with %s\n' "$(MISE_VERSION_PIN)" >&2; \
		exit 2; \
	fi

_builtin_require_environment: _builtin_require_workspace _builtin_require_mise_pin
# Documenting the interface (`make help`) must not require the interpreter it
# tells the operator how to provision. Only `make help` with no other goal
# skips the check; any combined goal still demands the environment.
ifneq ($(MAKECMDGOALS),help)
	@if [ ! -x "$(RUNTIME_PYTHON)" ]; then \
		printf 'ERROR: missing environment interpreter %s; make setup creates it\n' "$(RUNTIME_PYTHON)" >&2; \
		exit 2; \
	fi
endif

# === SECTION: setup environment (managed) ===
# Source: computed (MAKE_PROFILE routing) + operator contract (mro-e9j0.6 C7)
# Operator contract: setup PROVISIONS tooling only — mise, venv, dependencies.
# It never generates, conforms, or mutates project code; `make gen` is the
# is the single public conformance/generation surface.
# Setup always reconciles directly from the lock. The venv is created when
# missing and is never cleared while present, because a concurrent lane may be
# running against it.
ifeq ($(MAKE_PROFILE),workspace)
_builtin_setup_environment: _builtin_setup_submodules
	@$(SETUP_ENVIRONMENT_RECIPE)
	@$(UV) pip check --python "$(RUNTIME_VENV)"
else
_builtin_setup_environment: _builtin_setup_submodules
	@$(SETUP_ENVIRONMENT_RECIPE)
endif
# End SECTION: setup environment

# `upg` is the only recipe that resolves: the bootstrap above bumps mise.lock
# before installing, and this lifecycle upgrades every uv.lock, provisions the
# environment frozen from the new locks, and conforms dependency floors.
# The floors land in the codegen SSOT, so `gen` projects them into every
# pyproject and the locks are re-resolved against those raised floors before
# the frozen reprovision: the committed lock must match the committed
# pyproject, or `setup --locked` (the CI path) rejects it.
# Branch-tracked git dependencies are moving sources by declaration
# (workspace.yaml owns the branch): --refresh re-reads their metadata so a
# stale cached requires-dist can never block or skew the resolution
# (flext-62fbu). Like `setup`, it runs the declared pre-/post-upg lifecycle
# hooks, post-upg inside the activated environment.
.PHONY: _upg_lifecycle
_upg_lifecycle: _builtin_setup_submodules
	@set -eu; \
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" pre-upg "*) $(SELF_MAKE) pre-upg ;; \
	esac
	$(call _run_for_all_projects,--upgrade --refresh)
	@$(SELF_MAKE) _builtin_setup_environment
	@set -eu; \
	selected="$(strip $(SELECTED_PROJECTS))"; \
	if [ -z "$$selected" ]; then selected="."; fi; \
	set --; \
	for project in $$selected; do set -- "$$@" --projects "$$project"; done; \
	$(PROJECT_FLEXT_INFRA) deps modernize --repository-root "$(PROJECT_ROOT)" \
		--apply --rewrite-constraints "$$@"
	@$(SELF_MAKE) gen
	$(call _run_for_all_projects,)
	@$(SELF_MAKE) _builtin_setup_environment
	$(call _run_for_all_projects,--check)
	+@XDG_DATA_HOME="$${SETUP_DIRENV_XDG_DATA_HOME:?missing persistent direnv data home}" \
		"$${SETUP_DIRENV:?missing Mise-resolved direnv executable}" exec "$(PROJECT_ROOT)" $(SELF_MAKE) _upg_activated

.PHONY: _upg_activated
_upg_activated:
	@set -eu; \
	case " $(CUSTOM_DECLARED_TARGETS) " in \
		*" post-upg "*) $(SELF_MAKE) post-upg ;; \
	esac


# _builtin-self-* targets serve the workspace root itself (project selector
# `.` from the orchestrator). They apply the same member-style gate recipes to
# PROJECT_ROOT without recursing into submodules, so the root distribution
# runs its own evidence in the global cycles.
_builtin-self-test: _builtin_require_environment

	@set -eu; \
		test_tmp_parent="$(PROJECT_SCRATCH_ROOT)/pytest"; \
		mkdir -p "$$test_tmp_parent"; \
		test_tmp=$$(mktemp -d "$$test_tmp_parent/invocation.XXXXXX"); \
		cleanup_test_tmp() { rm -rf "$$test_tmp"; }; \
		trap cleanup_test_tmp EXIT INT TERM; \
		TMPDIR="$$test_tmp" GOTMPDIR="$$test_tmp" $(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry

_builtin-self-check: _builtin_require_environment
	@set -eu; \
printf '%s\n' 'INFO: SUSPENDED check gate duplication; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
printf '%s\n' 'INFO: SUSPENDED check gate codemod; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
printf '%s\n' 'INFO: SUSPENDED check gate boundary; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
printf '%s\n' 'INFO: SUSPENDED check gate namespace; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
printf '%s\n' 'INFO: SUSPENDED check gate runtime-census; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
gates="lint,pyrefly,mypy,pyright,silent-failure,deferred-self-reference,security,markdown,loc-cap,tier-whitelist,index-declarations,smells,layout,canonical-alias,direnv"; \
		if [ "$(strip $(CI))" = "Y" ]; then \
			gates="lint,pyright,silent-failure,deferred-self-reference,security,markdown,loc-cap,tier-whitelist,index-declarations,smells,layout,canonical-alias,direnv"; \
			printf 'INFO: CI=Y runs check gates: lint pyright silent-failure deferred-self-reference security markdown loc-cap tier-whitelist index-declarations smells layout canonical-alias direnv\n'; \
		elif [ "$(strip $(CI))" = "N" ]; then \
			gates="pyrefly,mypy"; \
			printf 'INFO: CI=N runs check gates: pyrefly mypy\n'; \
		else \
			printf 'INFO: default context runs check gates: lint pyrefly mypy pyright silent-failure deferred-self-reference security markdown loc-cap tier-whitelist index-declarations smells layout canonical-alias direnv\n'; \
		fi; \
		if [ -z "$$gates" ]; then \
			printf 'ERROR: no active check gates remain in the selected context\n' >&2; \
			exit 2; \
		fi; \
		$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "$$gates" --projects .

_builtin-self-test-full: _builtin_require_environment

	@set -eu; \
		test_tmp_parent="$(PROJECT_SCRATCH_ROOT)/pytest"; \
		mkdir -p "$$test_tmp_parent"; \
		test_tmp=$$(mktemp -d "$$test_tmp_parent/invocation.XXXXXX"); \
		cleanup_test_tmp() { rm -rf "$$test_tmp"; }; \
		trap cleanup_test_tmp EXIT INT TERM; \
		TMPDIR="$$test_tmp" GOTMPDIR="$$test_tmp" $(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry full

_builtin-self-fmt: _builtin_require_environment
	@$(UV_RUN) ruff format --preview $(RUFF_PATHS)
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "markdown-format" --projects . --apply

_builtin-self-fix: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "lint,markdown,markdown-code,canonical-alias,smells" --projects . --apply --report-findings

_builtin-self-fix-enforcement: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check fix-enforcement --repository-root "$(PROJECT_ROOT)" --safe-only --apply

_builtin-self-build:
	@$(UV) build --project "$(PROJECT_ROOT)"

_builtin-self-clean: _builtin_clean_generated

_builtin-self-docs: _builtin_docs_all

# SonarCloud server-side issue exclusions (SSOT: codegen.sonarcloud). The verb
# writes an external service with SONAR_TOKEN from the environment; it belongs
# to no setup/gen/check/test workflow row and never runs implicitly.
_builtin_sonarcloud_sync_project: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) maintenance sonarcloud-sync --repository-root "$(PROJECT_ROOT)"

_builtin-self-sonarcloud-sync: _builtin_sonarcloud_sync_project


_builtin_build_artifacts:
	@$(UV) build --project "$(PROJECT_ROOT)"

# Check is read-only: it runs the gates without --apply, so the tree is left
# unchanged; fix applies the declared repairs of the fixable gates.
# CI=Y keeps make.check_gates_ci, the strict complement of
# make.check_gates_local; CI=N runs that local partition.
# An absent CI token runs every active default gate.
_builtin_check_all: _builtin_require_environment
	@set -eu; \
printf '%s\n' 'INFO: SUSPENDED check gate duplication; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
printf '%s\n' 'INFO: SUSPENDED check gate codemod; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
printf '%s\n' 'INFO: SUSPENDED check gate boundary; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
printf '%s\n' 'INFO: SUSPENDED check gate namespace; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
printf '%s\n' 'INFO: SUSPENDED check gate runtime-census; authority=flext-itpd1.3 / operator 2026-09-24 / flext-xp6ec; reason=Custom policy check suspended during the approved recovery.'; \
gates="lint,pyrefly,mypy,pyright,silent-failure,deferred-self-reference,security,markdown,loc-cap,tier-whitelist,index-declarations,smells,layout,canonical-alias,direnv"; \
		if [ "$(strip $(CI))" = "Y" ]; then \
			gates="lint,pyright,silent-failure,deferred-self-reference,security,markdown,loc-cap,tier-whitelist,index-declarations,smells,layout,canonical-alias,direnv"; \
			printf 'INFO: CI=Y runs check gates: lint pyright silent-failure deferred-self-reference security markdown loc-cap tier-whitelist index-declarations smells layout canonical-alias direnv\n'; \
		elif [ "$(strip $(CI))" = "N" ]; then \
			gates="pyrefly,mypy"; \
			printf 'INFO: CI=N runs check gates: pyrefly mypy\n'; \
		else \
			printf 'INFO: default context runs check gates: lint pyrefly mypy pyright silent-failure deferred-self-reference security markdown loc-cap tier-whitelist index-declarations smells layout canonical-alias direnv\n'; \
		fi; \
		if [ -z "$$gates" ]; then \
			printf 'ERROR: no active check gates remain in the selected context\n' >&2; \
			exit 2; \
		fi; \
		$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "$$gates" --projects .

_builtin_test_all: _builtin_require_environment

	@set -eu; \
		test_tmp_parent="$(PROJECT_SCRATCH_ROOT)/pytest"; \
		mkdir -p "$$test_tmp_parent"; \
		test_tmp=$$(mktemp -d "$$test_tmp_parent/invocation.XXXXXX"); \
		cleanup_test_tmp() { rm -rf "$$test_tmp"; }; \
		trap cleanup_test_tmp EXIT INT TERM; \
		TMPDIR="$$test_tmp" GOTMPDIR="$$test_tmp" $(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry

_builtin_test_full_all: _builtin_require_environment

	@set -eu; \
		test_tmp_parent="$(PROJECT_SCRATCH_ROOT)/pytest"; \
		mkdir -p "$$test_tmp_parent"; \
		test_tmp=$$(mktemp -d "$$test_tmp_parent/invocation.XXXXXX"); \
		cleanup_test_tmp() { rm -rf "$$test_tmp"; }; \
		trap cleanup_test_tmp EXIT INT TERM; \
		TMPDIR="$$test_tmp" GOTMPDIR="$$test_tmp" $(PYTEST_BOUNDED) $(UV_RUN) python -m flext_infra._pytest_entry full

# fmt is format-only (single-pass verb law): ruff formats Python, the
# fmt_gates formatters run once through the checker's apply mode, and every
# lint repair belongs to `make fix`. Only a real tool failure (exit >= 2)
# breaks the verb; a formatter's residual findings stay reportable and are
# enforced by `make check`.
_builtin_fmt_all: _builtin_require_environment
	@$(UV_RUN) ruff format --preview $(RUFF_PATHS)
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "markdown-format" --projects . --apply

_builtin_fix_all: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "lint,markdown,markdown-code,canonical-alias,smells" --projects . --apply --report-findings

# Catalog-driven enforcement fixes: every ENFORCE rule whose fix action is
# declared safe, applied through its registered adapter.
_builtin_fix_enforcement: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) check fix-enforcement --repository-root "$(PROJECT_ROOT)" --safe-only --apply

_builtin_sonarcloud_sync_all: _builtin_sonarcloud_sync_project


_builtin_run_default: _builtin_require_environment
	@$(UV_RUN) $(PROJECT_NAME) $(ARGS)

_builtin_status_diagnostics: _builtin_require_environment
	@printf 'profile=%s\nproject=%s\nruntime=%s\n' \
		'$(MAKE_PROFILE)' '$(PROJECT_ROOT)' '$(RUNTIME_ROOT)'
	@$(UV) --version
	@if [ -x "$(RUNTIME_PYTHON)" ]; then \
		$(UV) pip check --python "$(RUNTIME_VENV)"; \
	fi
	@git -C "$(PROJECT_ROOT)" status --short

# Operator law 2026-09-22: `docs` is a docs-only lifecycle (generate, fix,
# fmt, validate, audit); the actions render from make.docs.actions and never
# depend on the workspace-wide codegen conform — docs actions render their
# own outputs from the live configuration and sources.
_builtin_docs_all:
	@set -eu; \
	for action in $(DOCS_ACTIONS); do \
		mode=; \
		case "$$action" in fix|fmt) mode=--apply ;; esac; \
		$(PROJECT_FLEXT_INFRA) docs "$$action" --repository-root "$(PROJECT_ROOT)" --output-dir ".reports/docs" $$mode $(DOCS_PROJECT_ARGS); \
	done

_builtin_clean_generated:

	@find "$(PROJECT_ROOT)" -type d \
		\( -name __pycache__ -o -name .mypy_cache -o -name .pytest_cache -o -name .ruff_cache -o -name .pyrefly_cache -o -name .benchmarks -o -name .hypothesis \) \
		-prune -exec sh -eu -c 'for target do find "$$target" -depth -delete; done' sh {} +


	@set -eu; \
	for target in "$(PROJECT_ROOT)/.flext-runtime" "$(PROJECT_ROOT)/build" "$(PROJECT_ROOT)/dist" "$(PROJECT_ROOT)/htmlcov" "$(PROJECT_ROOT)/.reports"; do \
		if [ -e "$$target" ]; then find "$$target" -depth -delete; \
		elif [ -L "$$target" ]; then find "$$target" -depth -delete; fi; \
	done


	@set -eu; \
	for target in "$(PROJECT_ROOT)/.coverage" "$(PROJECT_ROOT)/.testmondata" "$(PROJECT_ROOT)/flext-infra-codegen-transaction-journal.json.lock"; do \
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

# Generation has one transaction owner. Conform preserves the caller's scope and
# journals ordinary, Mise, lazy-init, and documentation phases through one fixed
# point. Only `upg` resolves and rewrites the locks; gen installs nothing and
# never runs another writer before or after conform's journal.
_builtin_gen_init:
	@$(PROJECT_FLEXT_INFRA) codegen init --repository-root "$(PROJECT_ROOT)" --apply
	@$(PROJECT_FLEXT_INFRA) codegen init --repository-root "$(PROJECT_ROOT)" --check

_builtin_gen_all:
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode apply

# Structural rewrites have one selector-free public Make surface. The current
# directory defines scope; callers never address ast-grep, Rope, or LSP directly.
_builtin_mod_apply: _builtin_require_environment
	@$(PROJECT_FLEXT_INFRA) refactor mod --apply

# Selector-free public verbs map one-to-one to their canonical implementation;
# each implementation owns one fixed operation.
_builtin-build: _builtin_build_artifacts
_builtin-check: _builtin_check_all
_builtin-test: _builtin_test_all
_builtin-test-full: _builtin_test_full_all
_builtin-fmt: _builtin_fmt_all
_builtin-fix: _builtin_fix_all
_builtin-fix-enforcement: _builtin_fix_enforcement
_builtin-audit:
	@$(UV) pip check --python "$(RUNTIME_VENV)"
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --scope "$(CODEGEN_SCOPE)" --mode check
_builtin-status: _builtin_status_diagnostics
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
_builtin-waza:
	@cd "$(PROJECT_ROOT)" && "$(SETUP_MISE)" exec -- waza check --no-update-check
_builtin-duplication:
	@$(PROJECT_FLEXT_INFRA) check run --repository-root "$(PROJECT_ROOT)" --gates "duplication" --projects .
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


