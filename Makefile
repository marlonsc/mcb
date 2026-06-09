# =============================================================================
# MCB — canonical Make interface. Few verbs, WHAT=/SCOPE= dispatch, mcb.sh monopoly.
# =============================================================================
SHELL := bash
.SHELLFLAGS := -euo pipefail -c
.DEFAULT_GOAL := help

MCB_SH := scripts/lib/mcb.sh
MCB_AUDIT_IGNORES := $(shell bash $(MCB_SH) ignores)

include makefiles/ui.mk
include makefiles/dispatch.mk

# --- params (single declaration) ---------------------------------------------
export RELEASE ?= 1
export QUICK ?= 0
export FIX ?= 0
export THREADS ?= 1
export SCOPE ?=
WHAT ?=
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

# sccache (shared compilation cache) — MANDATORY. Eliminates redundant rebuilds
# across sessions and projects. Mutually exclusive with incremental compilation.
export RUSTC_WRAPPER := sccache
export CARGO_INCREMENTAL := 0

# Verify sccache is installed; if not, warn and attempt install.
ifeq ($(shell command -v sccache 2>/dev/null),)
$(warning sccache not found in PATH. Attempting install...)
$(shell cargo install sccache --locked 2>/dev/null || true)
endif

# Destructive-verb gate: dry-run unless APPLY=Y. Usage: $(call gate,<action>)
gate = [ "$(APPLY)" = "Y" ] || { printf "DRY-RUN: would %s; set APPLY=Y to execute\n" "$(1)" >&2; exit 0; }

# --- WHATS_<verb> phase SSOT (drives sub-help + error arms) -------------------
WHATS_check   := fmt lint validate audit udeps coverage qlty coordination all
WHATS_fix     := fmt lint docs all
WHATS_dev     := run docker-up docker-down docker-logs docker-test
WHATS_docs    := build serve lint validate sync rust check setup adr adr-new diagrams
WHATS_codegen := all cli db entities conversions clean
WHATS_release := package version install install-validate
WHATS_git     := status diff log show add commit push pull branch checkout tag tags stash stash-pop stash-list merge rebase unstage push-tags
WHATS_pr      := checks view merge rerun
WHATS_sub     := status sync diff commit push propagate
WHATS_setup   := hooks tools adr all
WHATS_hook    := pre-commit pre-push
WHATS_clean   := build codegen all

# --- verb targets ------------------------------------------------------------
.PHONY: build test check lint-impl fix dev docs codegen release git pr sub setup clean ci guard hook help

build:     ; $(call DISPATCH_BUILD)
test:      ; $(call DISPATCH_TEST)
check:     ; $(call DISPATCH_CHECK)
lint-impl: ; @cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings
fix:       ; $(call DISPATCH_FIX)
dev:       ; $(call DISPATCH_DEV)
docs:      ; $(call DISPATCH_DOCS)
codegen:   ; $(call DISPATCH_CODEGEN)
release:   ; $(call DISPATCH_RELEASE)
git:       ; $(call DISPATCH_GIT)
pr:        ; $(call DISPATCH_PR)
sub:       ; $(call DISPATCH_SUB)
setup:     ; $(call DISPATCH_SETUP)
clean:     ; $(call DISPATCH_CLEAN)
ci:        ; @$(MAKE) check WHAT=all
guard:     ; @bash $(MCB_SH) guard
hook:      ; $(call DISPATCH_HOOK)
dev-env-optimize: ; @bash scripts/dev-env-optimize.sh $(if $(filter Y,$(APPLY)),--apply,)

help:
	@printf "\n$(BOLD)MCB — make <verb> [WHAT=phase] [SCOPE=..] [APPLY=Y]$(RESET)\n\n"
	@printf "  %-10s %s\n" build   "Build (RELEASE=0|1)"
	@printf "  %-10s %s\n" test    "Test (SCOPE=unit|doc|golden|startup|integration|e2e|all, THREADS=N)"
	@printf "  %-10s %s\n" check   "Read-only gate (WHAT=$(WHATS_check))"
	@printf "  %-10s %s\n" fix     "Auto-fix (WHAT=$(WHATS_fix))"
	@printf "  %-10s %s\n" dev     "Dev/docker (WHAT=$(WHATS_dev))"
	@printf "  %-10s %s\n" docs    "Docs (WHAT=$(WHATS_docs))"
	@printf "  %-10s %s\n" codegen "Codegen [APPLY=Y] (WHAT=$(WHATS_codegen))"
	@printf "  %-10s %s\n" release "Release (WHAT=$(WHATS_release), BUMP=patch|minor|major)"
	@printf "  %-10s %s\n" git     "Git (WHAT=$(WHATS_git)) [commit/push/merge/rebase: APPLY=Y]"
	@printf "  %-10s %s\n" pr      "GitHub PR (WHAT=$(WHATS_pr), PR=, RUN=)"
	@printf "  %-10s %s\n" sub     "Submodules (WHAT=$(WHATS_sub), SUB=, MSG=)"
	@printf "  %-10s %s\n" setup   "Setup (WHAT=$(WHATS_setup))"
	@printf "  %-10s %s\n" clean   "Clean [APPLY=Y] (WHAT=$(WHATS_clean))"
	@printf "  %-10s %s\n" ci               "CI gate (check WHAT=all)"
	@printf "  %-10s %s\n" guard            "Banned-pattern scanner"
	@printf "  %-10s %s\n" hook             "Tiered git-hook gate (WHAT=$(WHATS_hook))"
	@printf "  %-10s %s\n" dev-env-optimize "Clean duplicate rust-analyzer/Serena [APPLY=Y]"
	@printf "\n"
