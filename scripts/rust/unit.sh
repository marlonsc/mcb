#!/usr/bin/env bash
# Rust unit test target (old `make test WHAT=unit` selector).
set -euo pipefail
cd "$(dirname "$0")/../.."
if cargo nextest --version >/dev/null 2>&1; then
	bash scripts/lib/mcb.sh run cargo nextest run --workspace --test unit
else
	bash scripts/lib/mcb.sh run cargo test --workspace --test unit
fi
