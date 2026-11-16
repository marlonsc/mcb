#!/usr/bin/env bash
# Rust workspace suite (old `make test WHAT=rust` selector).
# MATCH=<filter> narrows the run (nextest -E / cargo test filter) without
# leaving the make surface.
set -euo pipefail
cd "$(dirname "$0")/../.."
if [ -n "${MATCH:-}" ]; then
	cargo test --workspace --all-targets "$MATCH"
else
	cargo test --workspace --all-targets
fi
