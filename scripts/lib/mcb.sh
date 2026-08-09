#!/usr/bin/env bash
# scripts/lib/mcb.sh — canonical tooling monopoly for MCB.
# ONE source for cross-cutting concerns: exit codes, logging, colors, the
# APPLY=Y mutation gate, SSOT readers (version, binary, audit ignores), the
# banned-pattern guard, and the agent bash-guard. No script, makefile, hook, or
# CI job calls cargo/git directly — everything flows through here.
#
# Use as a library:   source scripts/lib/mcb.sh ; mcb_require_cmd cargo
# Use as a dispatcher: bash scripts/lib/mcb.sh <command> [args...]
#
# Note: strict mode (set -euo pipefail) is enabled ONLY in the direct-execution
# dispatcher at the bottom, never at source time — sourcing must not flip the
# caller's shell options (make sets its own -euo pipefail; docs scripts source us).
[ -n "${_MCB_SH_LOADED:-}" ] && return 0
_MCB_SH_LOADED=1

# --- Standard exit codes -----------------------------------------------------
export EX_OK=0 EX_FAIL=1 EX_PREREQ=2 EX_GUARD=3 EX_INFRA=5

# --- Paths (repo root = two levels up from scripts/lib) ----------------------
MCB_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export MCB_ROOT

# --- tty-guarded colors (replaces 5 duplicated blocks) -----------------------
if [ -t 1 ]; then
  RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[0;33m'
  CYAN=$'\033[0;36m'; BOLD=$'\033[1m'; RESET=$'\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; CYAN=''; BOLD=''; RESET=''
fi
export RED GREEN YELLOW CYAN BOLD RESET

# --- logging / fail-fast -----------------------------------------------------
mcb_log()  { printf '%s\n' "$*" >&2; }
mcb_ok()   { printf '%b✓%b %s\n' "$GREEN" "$RESET" "$*" >&2; }
mcb_warn() { printf '%b!%b %s\n' "$YELLOW" "$RESET" "$*" >&2; }
mcb_die()  { local c="$1"; shift; printf '%bERRO:%b %s\n' "$RED" "$RESET" "$*" >&2; exit "$c"; }
mcb_require_cmd() { command -v "$1" >/dev/null 2>&1 || mcb_die "$EX_PREREQ" "comando '$1' ausente (instale via: make setup WHAT=tools)"; }

# --- single mutation gate (APPLY=Y, destructive verbs only) ------------------
mcb_require_apply() {
  [ "${APPLY:-N}" = "Y" ] && return 0
  printf 'DRY-RUN: would %s; set APPLY=Y to execute\n' "$*" >&2
  exit "$EX_OK"
}
mcb_apply_y() { [ "${APPLY:-N}" = "Y" ]; }

# --- retry helper ------------------------------------------------------------
mcb_retry() { local n="$1" s="$2"; shift 2; local t=1; while ! "$@"; do [ "$t" -ge "$n" ] && return 1; sleep "$s"; t=$((t+1)); done; }

# --- SSOT readers ------------------------------------------------------------
mcb_version() { grep -m1 '^version =' "$MCB_ROOT/Cargo.toml" | sed 's/.*"\([^"]*\)".*/\1/'; }

mcb_git_hooks_dir() {
  local repo="${1:-$MCB_ROOT}"
  local common_dir
  common_dir="$(git -C "$repo" rev-parse --path-format=absolute --git-common-dir)" \
    || mcb_die "$EX_PREREQ" "nao foi possivel resolver o diretorio Git comum de '$repo'"
  printf '%s/hooks\n' "$common_dir"
}

mcb_install_hooks() {
  local repo="${1:-$MCB_ROOT}"
  local hooks_dir
  hooks_dir="$(mcb_git_hooks_dir "$repo")"
  mkdir -p "$hooks_dir"
  cp "$MCB_ROOT/scripts/hooks/pre-commit" "$MCB_ROOT/scripts/hooks/pre-push" "$hooks_dir/"
  chmod +x "$hooks_dir/pre-commit" "$hooks_dir/pre-push"
  mcb_ok "pre-commit and pre-push hooks installed at $hooks_dir"
}

mcb_sync_submodules() {
  local repo="${1:-$MCB_ROOT}"
  local materialized

  git -C "$repo" submodule sync --recursive
  git -C "$repo" submodule update --init --recursive

  while :; do
    materialized="$(git -C "$repo" submodule foreach --quiet --recursive '
      present="$(git ls-files -z | while IFS= read -r -d "" path; do
        if [ -e "$path" ] || [ -L "$path" ]; then
          printf 1
          break
        fi
      done)"
      [ -n "$present" ] && exit 0
      tracked="$(git ls-files | wc -l)"
      if [ "$tracked" -eq 0 ]; then
        git read-tree HEAD
        tracked="$(git ls-files | wc -l)"
      fi
      [ "$tracked" -eq 0 ] && exit 0
      git checkout-index --all
      printf "%s\n" "$sm_path"
    ')"
    [ -z "$materialized" ] && break
    git -C "$repo" submodule update --init --recursive
  done
}

# Binary lookup chain: PATH > target/release > target/debug > cargo run
mcb_bin() {
  [ -x "$MCB_ROOT/target/debug/mcb" ]   && { echo "$MCB_ROOT/target/debug/mcb";   return 0; }
  [ -x "$MCB_ROOT/target/release/mcb" ] && { echo "$MCB_ROOT/target/release/mcb"; return 0; }
  command -v mcb 2>/dev/null && return 0
  echo "cargo run --package mcb --"
}

# Single source for the RUSTSEC/CVE audit-ignore list.
# RUSTSEC-2026-0194/0195 (quick-xml DoS via duplicate attributes / namespace
# exhaustion): opendal (vendored loco storage driver) pins quick-xml <0.41 and
# no released opendal consumes quick-xml >=0.41 yet; the XML-parsing services
# are not enabled in this workspace (services-memory/services-fs only), so the
# vulnerable paths are unreachable. Revisit on the next opendal bump.
MCB_AUDIT_IGNORES=(RUSTSEC-2023-0071 RUSTSEC-2023-0089 RUSTSEC-2025-0119 \
                   RUSTSEC-2024-0436 RUSTSEC-2025-0134 CVE-2023-49092 \
                   RUSTSEC-2026-0194 RUSTSEC-2026-0195)
export MCB_AUDIT_IGNORES

mcb_validate() {  # $1 = "quick" | "full"
  mkdir -p "$MCB_ROOT/reports"
  local cmd; cmd="$(mcb_bin)"
  local q=""; [ "${1:-full}" = "quick" ] && q="--quick"
  # shellcheck disable=SC2086
  $cmd validate "$MCB_ROOT" $q --format json > "$MCB_ROOT/reports/mcb-validate-internal-report.json" \
    || mcb_die "$EX_FAIL" "validate failed (see output above)"
  mcb_ok "report: reports/mcb-validate-internal-report.json"
}

# FILES word-split safety (ported from cosmos Makefile:80): refuse shell metachars.
mcb_files_safe() { printf '%s' "${1:-}" | grep -qE '[;|&`$()<>]' && mcb_die "$EX_PREREQ" "FILES contem metacaractere de shell perigoso; liste apenas caminhos"; return 0; }

mcb_check_staged() {
  local crate_dir deadline=60 manifest package packages=""
  local staged

  mcb_require_cmd timeout
  staged="$(git -C "$MCB_ROOT" diff --cached --name-only --diff-filter=ACMR)"
  [ -n "$staged" ] || { mcb_ok "staged check: no staged paths"; return 0; }

  if printf '%s\n' "$staged" | grep -qE '^Cargo\.(toml|lock)$'; then
    packages="--workspace"
  else
    while IFS= read -r path; do
      case "$path" in
        crates/*/src/*.rs)
          crate_dir="$(printf '%s\n' "$path" | cut -d/ -f1-2)"
          manifest="$crate_dir/Cargo.toml"
          [ -f "$MCB_ROOT/$manifest" ] || continue
          package="$(sed -n '/^\[package\]/,/^\[/s/^name = "\([^"]*\)"/\1/p' "$MCB_ROOT/$manifest" | head -1)"
          [ -n "$package" ] || mcb_die "$EX_PREREQ" "package name ausente em '$manifest'"
          case " $packages " in *" -p $package "*) ;; *) packages="$packages -p $package" ;; esac
          ;;
      esac
    done <<< "$staged"
  fi

  [ -n "$packages" ] || { mcb_ok "staged check: no Rust package affected"; return 0; }
  mcb_log "staged check: cargo fmt/clippy scope:$packages (deadline ${deadline}s each)"
  timeout --signal=TERM --kill-after=5s "${deadline}s" cargo fmt $packages -- --check
  timeout --signal=TERM --kill-after=5s "${deadline}s" cargo clippy $packages --all-targets -- -D warnings
  mcb_ok "staged check: clean"
}

# --- banned-pattern guard ----------------------------------------------------
# Scans first-party crates/ for the constructs AGENTS.md forbids in prod paths.
# Excludes: tests, #[cfg(test)] modules, target/. Fails EX_GUARD.
mcb_guard() {
  local rc=0 hits src staged=0
  [ "${1:-}" = "--staged" ] && staged=1
  if [ "$staged" = "1" ]; then
    # Block only NEW violations in staged prod .rs (added/copied/modified), not
    # the retroactive baseline. Excludes tests/ and benches/ (test-like).
    src=$(git -C "$MCB_ROOT" diff --cached --name-only --diff-filter=ACM -- crates 2>/dev/null \
      | grep -E '\.rs$' | grep -vE '/(tests|benches)/' | sed "s|^|$MCB_ROOT/|" || true)
    [ -z "$src" ] && { mcb_ok "guard: no staged prod .rs to scan"; return 0; }
  else
    src=$(find "$MCB_ROOT/crates" -name '*.rs' -not -path '*/tests/*' -not -path '*/benches/*' -not -path '*/target/*' 2>/dev/null || true)
    [ -z "$src" ] && { mcb_warn "guard: no source files found under crates/"; return 0; }
  fi
  # 1. unwrap/expect/panic/todo/unimplemented in non-test .rs
  hits=$(grep -rnE '\.(unwrap|expect)\(|\b(panic|todo|unimplemented)!\(' $src 2>/dev/null \
      | grep -vE ':[[:space:]]*(//|///|//!)|#\[cfg\(test\)\]|pub const .*: &str = |\.message\("|message = "|suggestion = "|r".*(unwrap|expect|panic|todo|unimplemented)|line\.contains\("todo!"\)' || true)
  [ -n "$hits" ] && { mcb_warn "prod unwrap/expect/panic/todo:"; printf '%s\n' "$hits" >&2; rc=$EX_GUARD; }
  # 2. TODO/FIXME markers
  hits=$(grep -rnE '\b(TODO|FIXME)\b' $src 2>/dev/null || true)
  [ -n "$hits" ] && { mcb_warn "TODO/FIXME markers:"; printf '%s\n' "$hits" >&2; rc=$EX_GUARD; }
  if [ "$staged" = "1" ]; then
    hits=$(git -C "$MCB_ROOT" diff --cached --unified=0 -- crates 2>/dev/null \
      | grep -E '^\+[^+].*#\[allow\(' | grep -vE '#\[allow\([^]]+\)\][[:space:]]*//[[:space:]]*[^[:space:]]' || true)
  else
    hits=$(git -C "$MCB_ROOT" diff origin/main...HEAD --unified=0 -- crates 2>/dev/null \
      | grep -E '^\+[^+].*#\[allow\(' | grep -vE '#\[allow\([^]]+\)\][[:space:]]*//[[:space:]]*[^[:space:]]' || true)
  fi
  [ -n "$hits" ] && { mcb_warn "new #[allow] without a trailing rationale:"; printf '%s\n' "$hits" >&2; rc=$EX_GUARD; }
  [ "$rc" -eq 0 ] && mcb_ok "guard: clean"
  return "$rc"
}

# --- agent bash-guard (PreToolUse hook target) -------------------------------
# Reads the hook JSON/command on stdin, blocks dangerous commands. Mirrors the
# global ~/.claude bash-guard so agents in mcb cannot escape the monopoly.
mcb_guard_bash() {
  local cmd; cmd="$(cat)"
  case "$cmd" in
    *"git push --force"*|*"git push -f"*|*"git reset --hard"*|*"git clean -f"*|\
    *"sed -i"*|*"rm -rf"*|*"rm -fr"*|*"bash -c"*|*"sh -c"*|*"eval "*)
      printf '{"decision":"block","reason":"mcb monopoly: dangerous command blocked — use a make verb"}\n'; return 0;;
  esac
  printf '{"decision":"approve"}\n'
}

# --- dispatcher (only when executed, not when sourced) -----------------------
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
  set -euo pipefail
  case "${1:-}" in
    version)        mcb_version ;;
    bin)            mcb_bin ;;
    ignores)        printf '%s\n' "${MCB_AUDIT_IGNORES[*]}" ;;
    git-hooks-dir)  mcb_git_hooks_dir "${2:-$MCB_ROOT}" ;;
    install-hooks)  mcb_install_hooks "${2:-$MCB_ROOT}" ;;
    sync-submodules) mcb_sync_submodules "${2:-$MCB_ROOT}" ;;
    validate)       mcb_validate "${2:-full}" ;;
    check-staged)   mcb_check_staged ;;
    guard)          shift; mcb_guard "$@" ;;
    guard-bash)     mcb_guard_bash ;;
    files-safe)     mcb_files_safe "${2:-}" ;;
    *)              mcb_die "$EX_PREREQ" "unknown command: ${1:-<none>}" ;;
  esac
fi
