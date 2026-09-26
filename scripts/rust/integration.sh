#!/usr/bin/env bash
# Rust integration test targets (old `make test WHAT=integration` selector).
set -euo pipefail
cd "$(dirname "$0")/../.."
bash scripts/lib/mcb.sh run cargo test --workspace --test '*integration*'
