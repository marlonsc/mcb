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
mcb_require_cmd() { command -v "$1" >/dev/null 2>&1 || mcb_die "$EX_PREREQ" "comando '$1' ausente (instale via: make boot WHAT=tools)"; }

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

# Binary lookup chain: PATH > target/release > target/debug > cargo run
mcb_bin() {
  command -v mcb 2>/dev/null && return 0
  [ -x "$MCB_ROOT/target/release/mcb" ] && { echo "$MCB_ROOT/target/release/mcb"; return 0; }
  [ -x "$MCB_ROOT/target/debug/mcb" ]   && { echo "$MCB_ROOT/target/debug/mcb";   return 0; }
  echo "cargo run --package mcb --"
}

# Single source for the RUSTSEC/CVE audit-ignore list.
MCB_AUDIT_IGNORES=(RUSTSEC-2023-0071 RUSTSEC-2023-0089 RUSTSEC-2025-0119 \
                   RUSTSEC-2024-0436 RUSTSEC-2025-0134 CVE-2023-49092)
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
  # Exclusion paths per-check:
  # check1 (unwrap/panic/todo): validator source files contain regex patterns
  # and error messages that cite banned constructs by definition.
  local guard_excludes_check1='mcb-validate/src/|mcb-utils/src/constants/validate/'
  # check2/3 (TODO/FIXME, #[allow]): validator source files and constant-definition files.
  local guard_excludes='mcb-validate/src/|mcb-utils/src/constants/validate/'

  # 1. unwrap/expect/panic/todo/unimplemented in non-test .rs
  # Exclude: doc comments (///, //!), const/static declarations, string literals.
  hits=$(grep -rnE '\b(unwrap|expect)\(|\bpanic!|\btodo!|\bunimplemented!' $src 2>/dev/null \
      | grep -vE '//.*(unwrap|expect)' \
      | grep -vE '#\[cfg\(test\)\]' \
      | grep -vE '^[^:]+:[0-9]+:\s*///' \
      | grep -vE '^[^:]+:[0-9]+:\s*//!' \
      | grep -vE '^[^:]+:[0-9]+:\s*(pub\s+)?(const|static)\s+' \
      | grep -vE ':\s*&?str\s*=' \
      | grep -vE 'r#"' \
      | grep -vE "$guard_excludes_check1" || true)
  [ -n "$hits" ] && { mcb_warn "prod unwrap/expect/panic/todo:"; printf '%s\n' "$hits" >&2; rc=$EX_GUARD; }
  # 2. TODO/FIXME markers
  hits=$(grep -rnE '\b(TODO|FIXME)\b' $src 2>/dev/null \
      | grep -vE '^[^:]+:[0-9]+:\s*///' \
      | grep -vE '^[^:]+:[0-9]+:\s*//!' \
      | grep -vE ':\s*&?str\s*=' \
      | grep -vE 'r#"' \
      | grep -vE "$guard_excludes" || true)
  [ -n "$hits" ] && { mcb_warn "TODO/FIXME markers:"; printf '%s\n' "$hits" >&2; rc=$EX_GUARD; }
  # 3. unjustified suppression directives (#[allow(...)] with no // Why:)
  # Why: may appear on the same line or the line immediately after.
  hits=$(grep -rnE '#\[allow\(' $src 2>/dev/null | while IFS= read -r line; do
      file=$(printf '%s' "$line" | cut -d: -f1)
      lineno=$(printf '%s' "$line" | cut -d: -f2)
      # same-line justification
      if printf '%s' "$line" | grep -qE '//\s*Why:'; then continue; fi
      # next-line justification
      nextline=$(sed -n "$((lineno + 1))p" "$file" 2>/dev/null)
      if printf '%s' "$nextline" | grep -qE '^\s*//\s*Why:'; then continue; fi
      printf '%s\n' "$line"
    done | grep -vE "$guard_excludes" || true)
  [ -n "$hits" ] && { mcb_warn "#[allow] without // Why: justification:"; printf '%s\n' "$hits" >&2; rc=$EX_GUARD; }
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
    validate)       mcb_validate "${2:-full}" ;;
    guard)          shift; mcb_guard "$@" ;;
    guard-bash)     mcb_guard_bash ;;
    files-safe)     mcb_files_safe "${2:-}" ;;
    *)              mcb_die "$EX_PREREQ" "unknown command: ${1:-<none>}" ;;
  esac
fi
