#!/usr/bin/env bash
set -euo pipefail

if [[ -f "Makefile" ]]; then
	if grep -q "^ci-local:" Makefile; then
		make ci-local
		exit 0
	fi
fi

if [[ -f "package.json" ]]; then
	if grep -q '"lint"' package.json; then
		pnpm lint
	fi
	if grep -q '"type:check"' package.json; then
		pnpm type:check
	fi
	exit 0
fi

echo "No known validation targets found."
exit 1
