#!/usr/bin/env bash
# Rust workspace suite (old `make test WHAT=rust` selector).
# MATCH=<filter> narrows the run (nextest -E / cargo test filter) without
# leaving the make surface.
set -euo pipefail
cd "$(dirname "$0")/../.."
if cargo nextest --version >/dev/null 2>&1; then
	if [ -n "${MATCH:-}" ]; then
		bash scripts/lib/mcb.sh run cargo nextest run --workspace -E "$MATCH"
	else
		bash scripts/lib/mcb.sh run cargo nextest run --workspace
	fi
else
	if [ -n "${MATCH:-}" ]; then
		bash scripts/lib/mcb.sh run cargo test --workspace --all-targets "$MATCH"
	else
		bash scripts/lib/mcb.sh run cargo test --workspace --all-targets
	fi
fi
