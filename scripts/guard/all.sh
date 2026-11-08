#!/usr/bin/env bash
# Guard scan over production sources (old `make check WHAT=guard` selector).
set -euo pipefail
cd "$(dirname "$0")/../.."
bash scripts/lib/mcb.sh guard
