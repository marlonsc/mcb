#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
FIXTURE="$(mktemp -d)"
trap 'rm -rf "$FIXTURE"' EXIT

PRIMARY="$FIXTURE/primary"
LINKED="$FIXTURE/linked"

git init -q "$PRIMARY"
git -C "$PRIMARY" config user.email test@example.com
git -C "$PRIMARY" config user.name "MCB Test"
git -C "$PRIMARY" commit --allow-empty -q -m initial
git -C "$PRIMARY" worktree add -q -b linked-test "$LINKED"

PRIMARY_HOOKS="$(bash "$ROOT/scripts/lib/mcb.sh" git-hooks-dir "$PRIMARY")"
LINKED_HOOKS="$(bash "$ROOT/scripts/lib/mcb.sh" git-hooks-dir "$LINKED")"
EXPECTED_HOOKS="$(git -C "$PRIMARY" rev-parse --path-format=absolute --git-common-dir)/hooks"
PRIMARY_GIT_DIR="$(git -C "$PRIMARY" rev-parse --path-format=absolute --git-dir)"
LINKED_GIT_DIR="$(git -C "$LINKED" rev-parse --path-format=absolute --git-dir)"

test "$PRIMARY_HOOKS" = "$EXPECTED_HOOKS"
test "$LINKED_HOOKS" = "$EXPECTED_HOOKS"
test "$PRIMARY_GIT_DIR" != "$LINKED_GIT_DIR"
test -f "$LINKED/.git"

cp "$ROOT/scripts/hooks/pre-commit" "$PRIMARY/pre-commit"
cp "$ROOT/scripts/hooks/pre-push" "$PRIMARY/pre-push"
cp "$ROOT/scripts/lib/mcb.sh" "$PRIMARY/mcb.sh"
mkdir -p "$PRIMARY/scripts/hooks" "$PRIMARY/scripts/lib"
mv "$PRIMARY/pre-commit" "$PRIMARY/scripts/hooks/pre-commit"
mv "$PRIMARY/pre-push" "$PRIMARY/scripts/hooks/pre-push"
mv "$PRIMARY/mcb.sh" "$PRIMARY/scripts/lib/mcb.sh"
bash "$PRIMARY/scripts/lib/mcb.sh" install-hooks "$LINKED"
cmp "$ROOT/scripts/hooks/pre-commit" "$EXPECTED_HOOKS/pre-commit"
cmp "$ROOT/scripts/hooks/pre-push" "$EXPECTED_HOOKS/pre-push"
test -x "$EXPECTED_HOOKS/pre-commit"
test -x "$EXPECTED_HOOKS/pre-push"
if grep -q 'make boot' "$EXPECTED_HOOKS/pre-commit"; then
  printf 'installed pre-commit references removed make boot target\n' >&2
  exit 1
fi

printf 'hook fixtures: primary and linked worktree installed into %s\n' "$EXPECTED_HOOKS"
