#!/usr/bin/env bash
# Full mcb-validate structural validation (old `make check WHAT=validate`).
set -euo pipefail
cd "$(dirname "$0")/../.."
bash scripts/lib/mcb.sh validate full
