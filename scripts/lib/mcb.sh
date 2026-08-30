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
mcb_require_cmd() { command -v "$1" >/dev/null 2>&1 || mcb_die "$EX_PREREQ" "comando '$1' ausente (instale via: make setup)"; }

# --- single mutation gate (APPLY=Y, destructive verbs only) ------------------
mcb_require_apply() {
  [ "${APPLY:-N}" = "Y" ] && return 0
  printf 'DRY-RUN: would %s; set APPLY=Y to execute\n' "$*" >&2
  exit "$EX_OK"
}
mcb_apply_y() { [ "${APPLY:-N}" = "Y" ]; }

mcb_run() {
  local run_cmd
  local -a env_args
  [ "$#" -gt 0 ] || mcb_die "$EX_PREREQ" "mcb run recebeu nenhum comando"
  run_cmd="$1"
  shift
  while [ "$#" -gt 0 ] && [[ "$run_cmd" == *"="* ]] && printf '%s' "$run_cmd" | grep -qEq '^[A-Za-z_][A-Za-z0-9_]*='; do
    env_args+=("$run_cmd")
    run_cmd="$1"
    shift
  done
  [ -n "$run_cmd" ] || mcb_die "$EX_PREREQ" "mcb run recebeu comando vazio"
  printf '%s' "$run_cmd" | grep -qEq '^[A-Za-z_][A-Za-z0-9_]*=' && mcb_die "$EX_PREREQ" "mcb run recebeu somente variáveis de ambiente"
  if command -v mise >/dev/null 2>&1 && mise which "$run_cmd" >/dev/null 2>&1; then
    env "${env_args[@]}" mise exec --quiet -- "$run_cmd" "$@"
  else
    env "${env_args[@]}" "$run_cmd" "$@"
  fi
}

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

mcb_sync_submodules() {
  local repo="${1:-$MCB_ROOT}"
  local materialized

  git -C "$repo" submodule sync --recursive
  git -C "$repo" submodule update --init --recursive

  while :; do
    materialized="$(git -C "$repo" submodule foreach --quiet --recursive '
      present="$(git ls-files | while IFS= read -r path; do
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

# Binary lookup chain: newest workspace target > PATH > cargo run.
# Why: preferring debug unconditionally let a stale artifact from an older
# build shadow a freshly built release binary, so `make check WHAT=validate`
# could run obsolete rules and pass or fail against code that no longer
# exists. Whichever target binary was built last is the current one.
mcb_bin() {
  local debug="$MCB_ROOT/target/debug/mcb" release="$MCB_ROOT/target/release/mcb"
  local newest=""
  [ -x "$debug" ] && newest="$debug"
  if [ -x "$release" ]; then
    if [ -z "$newest" ] || [ "$release" -nt "$newest" ]; then newest="$release"; fi
  fi
  [ -n "$newest" ] && { echo "$newest"; return 0; }
  command -v mcb && return 0
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

mcb_guard_ast_hits() {
  local ast_grep="$1" pattern file output scan_status
  shift
  for pattern in '$VALUE.unwrap()' '$VALUE.expect($$$ARGS)' 'panic!($$$ARGS)' 'todo!($$$ARGS)' 'unimplemented!($$$ARGS)'; do
    while IFS= read -r file; do
      [ -n "$file" ] || continue
      scan_status=0
      if output=$("$ast_grep" run --pattern "$pattern" --lang rust --json "$file"); then
        scan_status=0
      else
        scan_status=$?
      fi
      [ "$scan_status" -eq 0 ] || { [ "$scan_status" -eq 1 ] && [ "$output" = '[]' ]; } \
        || { mcb_log "guard ast-grep scan failed for $file"; return "$EX_INFRA"; }
      [ "$output" != '[]' ] && printf '%s\n' "$output"
    done <<< "$1"
  done
  return 0
}

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
      if [[ "$path" =~ ^crates/[^/]+/src/.*\.rs$ ]]; then
        crate_dir="$(printf '%s\n' "$path" | cut -d/ -f1-2)"
        manifest="$crate_dir/Cargo.toml"
        [ -f "$MCB_ROOT/$manifest" ] || continue
        package="$(sed -n '/^\[package\]/,/^\[/s/^name = "\([^"]*\)"/\1/p' "$MCB_ROOT/$manifest" | head -1)"
        [ -n "$package" ] || mcb_die "$EX_PREREQ" "package name ausente em '$manifest'"
        case " $packages " in *" -p $package "*) ;; *) packages="$packages -p $package" ;; esac
      fi
    done <<< "$staged"
  fi

  [ -n "$packages" ] || { mcb_ok "staged check: no Rust package affected"; return 0; }
  mcb_log "staged check: cargo fmt/clippy scope:$packages (deadline ${deadline}s each)"
  if [ "$packages" = "--workspace" ]; then
    timeout --signal=TERM --kill-after=5s "${deadline}s" cargo fmt --all -- --check
    timeout --signal=TERM --kill-after=5s "${deadline}s" cargo clippy --workspace --all-targets -- -D warnings
  else
    # shellcheck disable=SC2086
    timeout --signal=TERM --kill-after=5s "${deadline}s" cargo fmt $packages -- --check
    # shellcheck disable=SC2086
    timeout --signal=TERM --kill-after=5s "${deadline}s" cargo clippy $packages --all-targets -- -D warnings
  fi
  mcb_ok "staged check: clean"
}

mcb_conflict_markers() {
  local grep_args=(-nE '^(<<<<<<<|=======|>>>>>>>)') hits
  [ "${1:-}" = "--staged" ] && grep_args=(--cached "${grep_args[@]}")
  # git grep exits 1 for "no matches" and >1 for a real failure. Collapsing
  # both into an empty result made the guard pass silently whenever git
  # itself errored, which is the opposite of what a guard is for.
  local status=0
  hits=$(git -C "$MCB_ROOT" grep "${grep_args[@]}" -- .) || status=$?
  [ "$status" -gt 1 ] && mcb_die "$EX_INFRA" "git grep failed with exit status $status"
  [ -z "$hits" ] && return 0
  mcb_warn "merge conflict markers:"
  printf '%s\n' "$hits" >&2
  return "$EX_GUARD"
}

# --- banned-pattern guard ----------------------------------------------------
# Scans first-party crates/ for the constructs AGENTS.md forbids in prod paths.
# Excludes: tests, #[cfg(test)] modules, target/. Fails EX_GUARD.
mcb_guard() {
  local ast_grep rc=0 hits src staged=0
  ast_grep=$(command -v ast-grep) || mcb_die "$EX_PREREQ" "guard requires ast-grep (install via: make setup)"
  "$ast_grep" --version >/dev/null \
    || mcb_die "$EX_INFRA" "guard ast-grep verification failed: $ast_grep"
  [ "${1:-}" = "--staged" ] && staged=1
  if [ "$staged" = "1" ]; then
    mcb_conflict_markers --staged || rc=$EX_GUARD
  else
    mcb_conflict_markers || rc=$EX_GUARD
  fi
  if [ "$staged" = "1" ]; then
    # Block only NEW violations in staged prod .rs (added/copied/modified), not
    # the retroactive baseline. Excludes tests/ and benches/ (test-like).
    src=$(git -C "$MCB_ROOT" diff --cached --name-only --diff-filter=ACM -- crates 2>/dev/null \
      | grep -E '\.rs$' | grep -vE '/(tests|benches)/' | sed "s|^|$MCB_ROOT/|" || true)
    [ -z "$src" ] && { mcb_ok "guard: no staged prod .rs to scan"; return "$rc"; }
  else
    src=$(find "$MCB_ROOT/crates" -name '*.rs' -not -path '*/tests/*' -not -path '*/benches/*' -not -path '*/target/*' 2>/dev/null || true)
    [ -z "$src" ] && { mcb_warn "guard: no source files found under crates/"; return "$rc"; }
  fi
  local guard_excludes='mcb-utils/src/constants/validate/'

  # 1. unwrap/expect/panic/todo/unimplemented in non-test .rs
  hits=$(mcb_guard_ast_hits "$ast_grep" "$src") \
    || mcb_die "$EX_INFRA" "guard ast-grep invocation failed"
  hits=$(printf '%s\n' "$hits" | grep -vE "$guard_excludes" || true)
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
    git-hooks-dir)  mcb_git_hooks_dir "${2:-$MCB_ROOT}" ;;
    sync-submodules) mcb_sync_submodules "${2:-$MCB_ROOT}" ;;
    validate)       mcb_validate "${2:-full}" ;;
    check-staged)   mcb_check_staged ;;
    conflict-markers) shift; mcb_conflict_markers "$@" ;;
    guard)          shift; mcb_guard "$@" ;;
    guard-bash)     mcb_guard_bash ;;
    run)            shift; [ "$#" -gt 0 ] || mcb_die "$EX_PREREQ" "mcb run requires a command"; mcb_run "$@" ;;
    files-safe)     mcb_files_safe "${2:-}" ;;
    *)              mcb_die "$EX_PREREQ" "unknown command: ${1:-<none>}" ;;
  esac
fi
