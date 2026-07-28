#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
FIXTURE="$(mktemp -d)"
trap 'rm -rf "$FIXTURE"' EXIT

PRIMARY="$FIXTURE/primary"
LINKED="$FIXTURE/linked"
SUB_SOURCE="$FIXTURE/sub-source"
SUB_ARCHIVE="$FIXTURE/sub-archive"
BIN="$FIXTURE/bin"
MAKE_LOG="$FIXTURE/make.log"

git init -q "$PRIMARY"
git -C "$PRIMARY" config user.email test@example.com
git -C "$PRIMARY" config user.name "MCB Test"
mkdir -p "$PRIMARY/scripts/hooks" "$PRIMARY/scripts/lib"
cp "$ROOT/scripts/hooks/pre-commit" "$PRIMARY/scripts/hooks/pre-commit"
cp "$ROOT/scripts/hooks/pre-push" "$PRIMARY/scripts/hooks/pre-push"
cp "$ROOT/scripts/lib/mcb.sh" "$PRIMARY/scripts/lib/mcb.sh"
git -C "$PRIMARY" add scripts
git -C "$PRIMARY" commit -q -m initial
git -C "$PRIMARY" worktree add -q -b linked-test "$LINKED"

NON_MCB="$FIXTURE/non-mcb"
git init -q "$NON_MCB"
git -C "$NON_MCB" config user.email test@example.com
git -C "$NON_MCB" config user.name "MCB Test"
git -C "$NON_MCB" commit --allow-empty -q -m initial

PRIMARY_HOOKS="$(bash "$ROOT/scripts/lib/mcb.sh" git-hooks-dir "$PRIMARY")"
LINKED_HOOKS="$(bash "$ROOT/scripts/lib/mcb.sh" git-hooks-dir "$LINKED")"
EXPECTED_HOOKS="$(git -C "$PRIMARY" rev-parse --path-format=absolute --git-common-dir)/hooks"
PRIMARY_GIT_DIR="$(git -C "$PRIMARY" rev-parse --path-format=absolute --git-dir)"
LINKED_GIT_DIR="$(git -C "$LINKED" rev-parse --path-format=absolute --git-dir)"

test "$PRIMARY_HOOKS" = "$EXPECTED_HOOKS"
test "$LINKED_HOOKS" = "$EXPECTED_HOOKS"
test "$PRIMARY_GIT_DIR" != "$LINKED_GIT_DIR"
test -f "$LINKED/.git"

bash "$PRIMARY/scripts/lib/mcb.sh" install-hooks "$LINKED"
cmp "$ROOT/scripts/hooks/pre-commit" "$EXPECTED_HOOKS/pre-commit"
cmp "$ROOT/scripts/hooks/pre-push" "$EXPECTED_HOOKS/pre-push"
test -x "$EXPECTED_HOOKS/pre-commit"
test -x "$EXPECTED_HOOKS/pre-push"
if grep -q 'make boot' "$EXPECTED_HOOKS/pre-commit"; then
  printf 'installed pre-commit references removed make boot target\n' >&2
  exit 1
fi

mkdir -p "$BIN"
cat > "$BIN/make" <<EOF
#!/usr/bin/env bash
printf '%s|%s\n' "\$PWD" "\$*" >> "$MAKE_LOG"
EOF
chmod +x "$BIN/make"

(cd "$PRIMARY" && PATH="$BIN:$PATH" "$EXPECTED_HOOKS/pre-commit")
(cd "$LINKED" && PATH="$BIN:$PATH" "$EXPECTED_HOOKS/pre-commit")
test "$(grep -c 'check WHAT=lint' "$MAKE_LOG")" -eq 2
test "$(grep -c 'check WHAT=validate QUICK=1' "$MAKE_LOG")" -eq 2
grep -q "^$PRIMARY|check WHAT=lint$" "$MAKE_LOG"
grep -q "^$LINKED|check WHAT=lint$" "$MAKE_LOG"

git init -q "$SUB_SOURCE"
git -C "$SUB_SOURCE" config user.email test@example.com
git -C "$SUB_SOURCE" config user.name "MCB Test"
printf '[package]\nname = "fixture"\nversion = "0.1.0"\n' > "$SUB_SOURCE/Cargo.toml"
git -C "$SUB_SOURCE" add Cargo.toml
git -C "$SUB_SOURCE" commit -q -m initial
git -C "$PRIMARY" -c protocol.file.allow=always submodule add -q "$SUB_SOURCE" vendor/sample
PATH="$BIN:$PATH" git -C "$PRIMARY" commit -q -am 'add fixture submodule'

mkdir -p "$SUB_ARCHIVE"
mv "$PRIMARY/vendor/sample/Cargo.toml" "$SUB_ARCHIVE/Cargo.toml"
git -C "$PRIMARY/vendor/sample" read-tree --empty
test ! -f "$PRIMARY/vendor/sample/Cargo.toml"
test "$(git -C "$PRIMARY/vendor/sample" ls-files | wc -l)" -eq 0
bash "$PRIMARY/scripts/lib/mcb.sh" sync-submodules "$PRIMARY"
test -f "$PRIMARY/vendor/sample/Cargo.toml"

printf 'preserved local content\n' > "$PRIMARY/vendor/sample/Cargo.toml"
bash "$PRIMARY/scripts/lib/mcb.sh" sync-submodules "$PRIMARY"
grep -q '^preserved local content$' "$PRIMARY/vendor/sample/Cargo.toml"

printf 'hook fixtures: primary and linked worktree install and commit gates passed via %s\n' "$EXPECTED_HOOKS"
