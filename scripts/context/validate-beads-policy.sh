#!/usr/bin/env bash
set -euo pipefail

# Use the canonical MCB tooling wrapper for repo-root discovery and helpers.
# shellcheck source=../lib/mcb.sh
. "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/mcb.sh"
cd "$MCB_ROOT"

fail() {
  printf 'beads-policy: %s\n' "$*" >&2
  exit 1
}

role_json="$(bd config get beads.role --json)"
printf '%s\n' "$role_json" | rg -q '"value":\s*"maintainer"' \
  || fail "beads.role must be maintainer"

dolt_show="$(bd dolt show)"
printf '%s\n' "$dolt_show" | rg -q 'Mode:\s+shared server' \
  || fail "bd must use Dolt shared-server mode"

bd hooks list --json | python3 -c '
import json
import sys

required = {"pre-commit", "post-merge", "pre-push", "post-checkout", "prepare-commit-msg"}
data = json.load(sys.stdin)
hooks = {hook["Name"]: hook for hook in data.get("hooks", [])}
missing = sorted(required - hooks.keys())
bad = sorted(
    name
    for name in required & hooks.keys()
    if not hooks[name].get("Installed") or hooks[name].get("Outdated")
)
if missing or bad:
    print(f"missing={missing} bad={bad}", file=sys.stderr)
    raise SystemExit(1)
' || fail "bd git hooks must be installed and current"

prepare_commit_msg="$(git -C "$MCB_ROOT" rev-parse --git-path hooks/prepare-commit-msg)"
[ -f "$prepare_commit_msg" ] || fail "prepare-commit-msg hook is missing"
rg -q 'BD_ALLOW_AGENT_COMMIT_TRAILERS' "$prepare_commit_msg" \
  || fail "prepare-commit-msg must guard agent trailers with BD_ALLOW_AGENT_COMMIT_TRAILERS"
rg -q 'bd hooks run prepare-commit-msg' "$prepare_commit_msg" \
  || fail "prepare-commit-msg must still delegate to bd when explicitly enabled"

scan_paths=()
for path in AGENTS.md makefiles docs/modules docs/developer .beads/config.yaml; do
  [ -e "$path" ] && scan_paths+=("$path")
done

if [ "${#scan_paths[@]}" -gt 0 ]; then
  matches="$(
    rg -n 'bd sync|bd --no-db|--no-db|bd export -o|beads-sync|SQLite \(Primary\)|Source of truth for sync|Area Lock|LEDGER\.md|\.agents/coordination/TODO\.md|bd doctor\b\s+(?:is|as|for|primary|authoritative|health\s+gate|source\s+of\s+truth|recommended|use|prefer|routine|normal|default|official)' "${scan_paths[@]}" || true
  )"
  bad="$(
    printf '%s\n' "$matches" |
      rg -v 'Do not|Never|NUNCA|nunca|Não|não|legacy|Legacy|histor|Hist|aposentad|antigo|retired|forbidden|proibid|nao use|não use|treat that as legacy|manual' || true
  )"
  [ -z "$bad" ] || fail "legacy coordination instruction remains: $bad"
fi

printf 'beads-policy: ok\n'
