#!/usr/bin/env bash
# Rust doc tests (old `make test WHAT=doc` selector).
set -euo pipefail
cd "$(dirname "$0")/../.."
bash scripts/lib/mcb.sh run cargo test --workspace --doc
