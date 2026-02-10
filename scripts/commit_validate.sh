#!/usr/bin/env bash
set -euo pipefail

if [[ -f "Makefile" ]]; then
	if grep -q "^ci-local:" Makefile; then
		make ci-local
		exit 0
	fi
fi

if [[ -f "tests/package.json" ]]; then
	if grep -q '"lint"' tests/package.json; then
		pnpm --dir tests lint
	fi
	if grep -q '"type:check"' tests/package.json; then
		pnpm --dir tests type:check
	fi
	exit 0
fi

echo "No known validation targets found."
exit 1
