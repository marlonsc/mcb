#!/usr/bin/env bash
set -euo pipefail

if [[ -f "Makefile" ]]; then
    # Note: Using lighter lint+validate instead of full "make check" for speed in pre-commit context
	if grep -q "^ci:" Makefile; then
		make lint MCB_CI=1 && make validate QUICK=1
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
