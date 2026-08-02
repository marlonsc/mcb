# =============================================================================
# MCB — canonical Make interface. Few verbs, WHAT=/SCOPE= dispatch, mcb.sh monopoly.
# Public verbs (the ONLY ones): help boot build check ship clean (+ test).
# =============================================================================
SHELL := bash
.SHELLFLAGS := -euo pipefail -c
.DEFAULT_GOAL := help

MCB_ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
MCB_SH := $(MCB_ROOT)/scripts/lib/mcb.sh
MCB_AUDIT_IGNORES := $(shell bash $(MCB_SH) ignores)
# Central executor that keeps automation in one place.
# Use for external commands that should run through mise when present.
MCB_TOOL := bash $(MCB_SH)
MCB_RUN := bash $(MCB_SH) run

include makefiles/ui.mk
include makefiles/dispatch.mk

# --- params (single declaration) ---------------------------------------------
export RELEASE ?= 1
export QUICK ?= 0
export FIX ?= 0
export THREADS ?= $(shell nproc 2>/dev/null || echo 1)
export SCOPE ?=
WHAT ?=
# ACT must not leak into sub-makes. Hooks pass ACT on the command line for their
# own dispatch; normalization below drops leaked values so unrelated verbs don't break.
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

# sccache (shared compilation cache) — MANDATORY. Eliminates redundant rebuilds
# across sessions and projects. The local cache size is bounded in
# .cargo/config.toml via SCCACHE_CACHE_SIZE. Incremental compilation is kept
# disabled (CARGO_INCREMENTAL=0) because sccache caches full compilation units;
# incremental artifacts are not cacheable and would bloat target/ while yielding
# 0%% sccache hits. Use `make check WHAT=optimize ACT=cache APPLY=Y` to prune
# stale build artifacts safely.
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
# WHAT= values for each canonical verb. Nested namespaces select with ACT= and
# publish their phase list as ACTS_<namespace>.
WHATS_boot    := hooks hook tools adr venv all
WHATS_build   := build debug release prebuild codegen docs
WHATS_check   := fmt lint validate audit udeps coverage qlty coordination guard fix dev optimize gitops surface python ci all
WHATS_ship    := status diff log show add commit push pull branch checkout tag tags stash stash-pop stash-list merge rebase unstage push-tags pr sub release
WHATS_clean   := build codegen all

# ACTS_<namespace> phase SSOT (nested ACT= dispatch under a WHAT=)
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

# Normalize ACT: hooks/scripts may leak an ACT value into the environment. If the
# current ACT is not a recognized phase, drop it so unrelated verbs do not break.
_ACT_VALID := $(ACTS_hook) $(ACTS_docs) $(ACTS_codegen) $(ACTS_fix) $(ACTS_dev) $(ACTS_pr) $(ACTS_sub) $(ACTS_release) $(ACTS_python) $(ACTS_optimize)
ifneq ($(ACT),)
  ifeq ($(filter $(ACT),$(_ACT_VALID)),)
    ACT :=
  endif
endif

# --- verb targets (the ONLY public verbs) ------------------------------------
.PHONY: help boot build test check ship clean

boot:   ; $(call DISPATCH_BOOT)
build:  ; $(call DISPATCH_BUILD)
test:   ; $(call DISPATCH_TEST)
check:  ; $(call DISPATCH_CHECK)
ship:   ; $(call DISPATCH_SHIP)
clean:  ; $(call DISPATCH_CLEAN)

help:
	@printf "\n$(BOLD)MCB — make <verb> [WHAT=phase] [ACT=sub] [SCOPE=..] [APPLY=Y]$(RESET)\n\n"
	@printf "  %-7s %s\n" help  "Show this help"
	@printf "  %-7s %s\n" boot  "Bootstrap dev env: hooks/tools/adr/venv (WHAT=$(WHATS_boot)); WHAT=hook ACT=$(ACTS_hook)"
	@printf "  %-7s %s\n" build "Build/codegen/docs (WHAT=$(WHATS_build)); RELEASE=0|1"
	@printf "  %-7s %s\n" test  "Test (SCOPE=unit|doc|golden|startup|warmup|integration|e2e|changed|all, THREADS=N)"
	@printf "  %-7s %s\n" check "Gates/fix/scan/CI (WHAT=$(WHATS_check))"
	@printf "  %-7s %s\n" ship  "Git/PR/sub/release (WHAT=$(WHATS_ship)) [mutating: APPLY=Y]"
	@printf "  %-7s %s\n" clean "Clean artifacts [APPLY=Y] (WHAT=$(WHATS_clean))"
	@printf "\n  $(BOLD)Nested ACT= namespaces$(RESET)\n"
	@printf "    build WHAT=codegen ACT=%s [APPLY=Y]\n" "$(ACTS_codegen)"
	@printf "    build WHAT=docs    ACT=%s [QUICK=1] [FIX=1] [mutating: APPLY=Y]\n" "$(ACTS_docs)"
	@printf "    check WHAT=fix     ACT=%s [APPLY=Y]\n" "$(ACTS_fix)"
	@printf "    check WHAT=dev     ACT=%s [APPLY=Y]\n" "$(ACTS_dev)"
	@printf "    check WHAT=guard | WHAT=ci\n"
	@printf "    check WHAT=optimize ACT=%s [APPLY=Y]\n" "$(ACTS_optimize)"
	@printf "    check WHAT=python  ACT=%s [QUICK=1]\n" "$(ACTS_python)"
	@printf "    ship  WHAT=pr      ACT=%s  PR= RUN=\n" "$(ACTS_pr)"
	@printf "    ship  WHAT=sub     ACT=%s  SUB= MSG=\n" "$(ACTS_sub)"
	@printf "    ship  WHAT=release ACT=%s  BUMP=patch|minor|major [APPLY=Y]\n" "$(ACTS_release)"
	@printf "    boot  WHAT=hook    ACT=%s\n" "$(ACTS_hook)"
	@printf "\n"

# Workspace-custom targets (done-check for the shared Stop hook). Kept out of the
# canonical verb interface above; lives in workspace_custom.mk, never sync-clobbered.
-include workspace_custom.mk
